#!/usr/bin/env python3
"""
Run convergence (dt/dx) and finite-size studies for the TDGL solver.

This script is intentionally dependency-light and works offline:
  - Runs the Rust binary in headless kappa-sweep mode.
  - Extracts kappa_c from kappa_sweep.csv (same logic as phase-diagram tooling).
  - Writes a single summary CSV for downstream plotting/reporting.

Typical studies:
  1) dt convergence: dt vs dt/2 at fixed (nx,ny,dx) and fixed physical times.
  2) dx convergence: keep domain length L = nx*dx constant while refining grid.
  3) finite-size: vary nx/ny at fixed dx and fixed target B (--b).
"""

from __future__ import annotations

import argparse
import csv
import math
import os
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path


def _default_exe_path() -> Path:
    exe = "Rust_wgpu_TDGL_AI_Trial.exe" if os.name == "nt" else "Rust_wgpu_TDGL_AI_Trial"
    return Path("target") / "debug" / exe


def _ensure_binary(binary: Path, build: bool) -> None:
    if binary.exists():
        return
    if not build:
        raise SystemExit(f"Binary not found: {binary} (use --build)")
    print("Building binary (cargo build)...")
    subprocess.check_call(["cargo", "build"])
    if not binary.exists():
        raise SystemExit(f"Build succeeded but binary still not found: {binary}")


def _parse_list(text: str, cast):
    items = []
    for part in (p.strip() for p in str(text).split(",")):
        if not part:
            continue
        items.append(cast(part))
    if not items:
        raise ValueError(f"empty list: {text!r}")
    return items


def _format_float_for_path(value: float) -> str:
    return f"{value:.4g}".replace("-", "m").replace("+", "p").replace(".", "d")


def _compute_order_parameter_from_row(order_parameter: str, row: list[str], col: dict[str, int]) -> float:
    if order_parameter == "mean_speed":
        return float(row[col["mean_speed"]])
    if order_parameter == "abs_mean_vx":
        return abs(float(row[col["mean_vx"]]))
    if order_parameter == "abs_mean_vy":
        return abs(float(row[col["mean_vy"]]))
    if order_parameter == "abs_mean_v":
        vx = float(row[col["mean_vx"]])
        vy = float(row[col["mean_vy"]])
        return float((vx * vx + vy * vy) ** 0.5)
    raise ValueError(f"unknown order parameter: {order_parameter!r}")


def _required_columns_for_order_parameter(order_parameter: str) -> set[str]:
    if order_parameter == "mean_speed":
        return {"kappa", "mean_speed"}
    if order_parameter == "abs_mean_vx":
        return {"kappa", "mean_vx"}
    if order_parameter == "abs_mean_vy":
        return {"kappa", "mean_vy"}
    if order_parameter == "abs_mean_v":
        return {"kappa", "mean_vx", "mean_vy"}
    raise ValueError(f"unknown order parameter: {order_parameter!r}")


def _read_kappa_sweep(csv_path: Path, order_parameter: str) -> list[tuple[float, float]]:
    rows: list[list[str]] = []
    with csv_path.open("r", encoding="utf-8", newline="") as f:
        for line in f:
            if not line.strip() or line.startswith("#"):
                continue
            rows.append(next(csv.reader([line])))
            break
        else:
            raise ValueError(f"no header in {csv_path}")

        header = rows[0]
        col = {name: i for i, name in enumerate(header)}
        missing = sorted(_required_columns_for_order_parameter(order_parameter) - set(col))
        if missing:
            raise ValueError(f"missing required columns in {csv_path}: {missing}")

        points: list[tuple[float, float]] = []
        for row in csv.reader((ln for ln in f if ln.strip() and not ln.startswith("#"))):
            try:
                kappa = float(row[col["kappa"]])
                value = _compute_order_parameter_from_row(order_parameter, row, col)
            except (ValueError, IndexError) as e:
                raise ValueError(f"bad row in {csv_path}: {row!r}") from e
            points.append((kappa, value))

    points.sort(key=lambda t: t[0])
    return points


def _estimate_kappa_c_threshold(points: list[tuple[float, float]], epsilon: float, consecutive: int) -> float | None:
    if consecutive <= 0:
        raise ValueError("--consecutive must be >= 1")
    above = [value > epsilon for _, value in points]
    for i in range(0, len(points) - consecutive + 1):
        if all(above[i : i + consecutive]):
            return float(points[i][0])
    return None


def _estimate_kappa_c_baseline_threshold(
    points: list[tuple[float, float]],
    epsilon: float,
    consecutive: int,
    baseline_points: int,
) -> float | None:
    if baseline_points <= 0:
        raise ValueError("--baseline-points must be >= 1")
    if len(points) < baseline_points:
        return None
    baseline = sorted(v for _, v in points[:baseline_points])[baseline_points // 2]
    threshold = float(baseline + epsilon)
    for i in range(baseline_points, len(points)):
        if points[i][1] <= threshold:
            continue
        window = points[i : i + consecutive]
        if len(window) < consecutive:
            break
        if all(v > threshold for _, v in window):
            return float(points[i][0])
    return None


def _fit_line(x: list[float], y: list[float]) -> tuple[float, float, float]:
    # y = a + b x; returns (a, b, sse)
    n = len(x)
    if n == 0:
        raise ValueError("empty fit")
    if n == 1:
        a = float(y[0])
        b = 0.0
        return a, b, 0.0

    sx = sum(x)
    sy = sum(y)
    sxx = sum(v * v for v in x)
    sxy = sum(xi * yi for xi, yi in zip(x, y))

    denom = n * sxx - sx * sx
    if abs(denom) < 1e-12:
        a = sy / n
        b = 0.0
    else:
        b = (n * sxy - sx * sy) / denom
        a = (sy - b * sx) / n

    sse = sum((yi - (a + b * xi)) ** 2 for xi, yi in zip(x, y))
    return float(a), float(b), float(sse)


def _estimate_kappa_c_two_phase_fit(points: list[tuple[float, float]], min_segment_points: int) -> float | None:
    # Model 1 (pinned): constant; Model 2 (flow): linear.
    # Find a split that minimizes SSE, then return intersection (clamped).
    n = len(points)
    if min_segment_points < 1:
        raise ValueError("--min-segment-points must be >= 1")
    if n < 2 * min_segment_points:
        return None

    xs = [float(k) for k, _ in points]
    ys = [float(v) for _, v in points]

    best: tuple[float, int, float, float, float] | None = None  # (sse, split, c1, a2, b2)
    for split in range(min_segment_points, n - min_segment_points + 1):
        y1 = ys[:split]
        c1 = sum(y1) / len(y1)
        sse1 = sum((v - c1) ** 2 for v in y1)

        a2, b2, sse2 = _fit_line(xs[split:], ys[split:])
        if b2 <= 0.0:
            continue
        total = sse1 + sse2
        if best is None or total < best[0]:
            best = (total, split, c1, a2, b2)

    if best is None:
        return None
    _, split, c1, a2, b2 = best

    k_min = xs[0]
    k_max = xs[-1]
    kappa_c = (c1 - a2) / b2
    if not (k_min <= kappa_c <= k_max):
        kappa_c = max(k_min, min(k_max, kappa_c))

    if (a2 + b2 * k_max) <= c1:
        return None

    for k, _ in points:
        if k >= kappa_c - 1e-12:
            return float(k)
    return float(points[-1][0])


def estimate_kappa_c(
    csv_path: Path,
    order_parameter: str,
    method: str,
    epsilon: float,
    consecutive: int,
    baseline_points: int,
    min_segment_points: int,
) -> float | None:
    points = _read_kappa_sweep(csv_path, order_parameter=order_parameter)
    if method == "threshold":
        return _estimate_kappa_c_threshold(points, epsilon=epsilon, consecutive=consecutive)
    if method == "baseline_threshold":
        return _estimate_kappa_c_baseline_threshold(
            points,
            epsilon=epsilon,
            consecutive=consecutive,
            baseline_points=baseline_points,
        )
    if method == "two_phase_fit":
        return _estimate_kappa_c_two_phase_fit(points, min_segment_points=min_segment_points)
    raise SystemExit(f"Unknown --kappa-c-method: {method!r}")


def quantize_flux_n(b_target: float, nx: int, ny: int, dx: float) -> int:
    # Matches the Rust helper: flux_n_from_b_field
    area = float(nx * ny) * dx * dx
    return int(round(b_target * area / (2.0 * math.pi)))


def b_from_flux_n(flux_n: int, nx: int, ny: int, dx: float) -> float:
    # B = 2π n / (nx*ny*dx^2)
    return (2.0 * math.pi * float(flux_n)) / (float(nx * ny) * dx * dx)


@dataclass(frozen=True)
class SweepConfig:
    kappa_start: float
    kappa_end: float
    kappa_step: float
    initial_relax_steps: int
    relax_steps: int
    measure_steps: int
    sample_period: int


def _run_one(
    *,
    binary: Path,
    out_dir: Path,
    nx: int,
    ny: int,
    dt: float,
    dx: float,
    b_target: float | None,
    flux_n: int | None,
    seed: int,
    defect_mode: str,
    defect_spacing: int,
    alpha_default: float,
    alpha_defect: float,
    defect_radius: int,
    defect_count: int,
    dump_positions: bool,
    sweep: SweepConfig,
) -> tuple[int, float]:
    if (b_target is None) == (flux_n is None):
        raise SystemExit("Provide exactly one of: --b-target or --flux-n")

    cmd: list[str] = [str(binary)]
    cmd += ["--headless"]
    cmd += ["--nx", str(nx), "--ny", str(ny)]
    cmd += ["--dt", str(dt), "--dx", str(dx)]
    if b_target is not None:
        cmd += ["--b", str(b_target)]
        flux_n_eff = quantize_flux_n(float(b_target), nx=nx, ny=ny, dx=float(dx))
    else:
        cmd += ["--flux-n", str(int(flux_n))]
        flux_n_eff = int(flux_n)
    b_eff = b_from_flux_n(flux_n_eff, nx=nx, ny=ny, dx=float(dx))

    cmd += ["--seed", str(int(seed))]
    cmd += ["--defect-mode", str(defect_mode)]
    cmd += ["--defect-spacing", str(int(defect_spacing))]
    cmd += ["--alpha-default", str(float(alpha_default))]
    cmd += ["--alpha-defect", str(float(alpha_defect))]
    cmd += ["--defect-radius", str(int(defect_radius))]
    cmd += ["--defect-count", str(int(defect_count))]
    if dump_positions:
        cmd += ["--dump-positions"]

    cmd += ["--kappa-start", str(float(sweep.kappa_start))]
    cmd += ["--kappa-end", str(float(sweep.kappa_end))]
    cmd += ["--kappa-step", str(float(sweep.kappa_step))]
    cmd += ["--kappa-initial-relax-steps", str(int(sweep.initial_relax_steps))]
    cmd += ["--kappa-relax-steps", str(int(sweep.relax_steps))]
    cmd += ["--kappa-measure-steps", str(int(sweep.measure_steps))]
    cmd += ["--sample-period", str(int(sweep.sample_period))]

    cmd += ["--out-dir", str(out_dir)]

    out_dir.mkdir(parents=True, exist_ok=True)
    subprocess.check_call(cmd)
    return flux_n_eff, b_eff


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Run dt/dx convergence and finite-size studies (kappa sweep)")
    p.add_argument("--binary", type=str, default=str(_default_exe_path()), help="Path to Rust binary (default: target/debug)")
    p.add_argument("--build", action="store_true", help="Build with cargo if binary is missing")
    p.add_argument("--out-root", type=str, default="", help="Output root directory (default: runs/convergence_<ts>)")

    p.add_argument("--study", type=str, choices=["dt", "dx", "size"], required=True)

    p.add_argument("--nx", type=int, default=256)
    p.add_argument("--ny", type=int, default=256)
    p.add_argument("--dt", type=float, default=0.01)
    p.add_argument("--dx", type=float, default=1.0)

    p.add_argument("--dt-list", type=str, default="", help="Comma list for study=dt (e.g. 0.01,0.005)")
    p.add_argument("--dx-list", type=str, default="", help="Comma list for study=dx (e.g. 1.0,0.5)")
    p.add_argument("--n-list", type=str, default="", help="Comma list for study=size (e.g. 128,256,512)")
    p.add_argument("--domain-length", type=float, default=256.0, help="For study=dx: L = nx*dx (default: 256)")

    g = p.add_mutually_exclusive_group(required=True)
    g.add_argument("--b-target", type=float, default=None, help="Target B (quantized via flux_n)")
    g.add_argument("--flux-n", type=int, default=None, help="Exact flux_n (total flux quanta)")

    p.add_argument("--seed", type=int, default=1234)

    p.add_argument("--defect-mode", type=str, choices=["random", "lattice"], default="random")
    p.add_argument("--defect-spacing", type=int, default=32)
    p.add_argument("--alpha-default", type=float, default=1.0)
    p.add_argument("--alpha-defect", type=float, default=-0.5)
    p.add_argument("--defect-radius", type=int, default=3)
    p.add_argument("--defect-count", type=int, default=50)
    p.add_argument(
        "--scale-defects-with-dx",
        action="store_true",
        help="For study=dx, scale defect_radius/spacing to keep physical sizes fixed",
    )

    p.add_argument("--kappa-start", type=float, default=0.0)
    p.add_argument("--kappa-end", type=float, default=0.05)
    p.add_argument("--kappa-step", type=float, default=0.01)
    p.add_argument("--initial-relax-steps", type=int, default=20000)
    p.add_argument("--relax-steps", type=int, default=2000)
    p.add_argument("--measure-steps", type=int, default=5000)
    p.add_argument("--sample-period", type=int, default=100)

    p.add_argument("--keep-physical-time", action="store_true", help="Scale relax/measure steps by dt (study=dt)")
    p.add_argument("--dump-positions", action="store_true", help="Also write vortex_positions.csv (larger outputs)")

    p.add_argument("--order-parameter", type=str, default="abs_mean_vx", choices=["mean_speed", "abs_mean_vx", "abs_mean_vy", "abs_mean_v"])
    p.add_argument("--kappa-c-method", type=str, default="two_phase_fit", choices=["threshold", "baseline_threshold", "two_phase_fit"])
    p.add_argument("--epsilon", type=float, default=1e-3)
    p.add_argument("--consecutive", type=int, default=1)
    p.add_argument("--baseline-points", type=int, default=2)
    p.add_argument("--min-segment-points", type=int, default=2)

    return p.parse_args()


def main() -> None:
    args = parse_args()
    binary = Path(args.binary)
    _ensure_binary(binary, build=bool(args.build))

    out_root = Path(args.out_root) if args.out_root else Path("runs") / f"convergence_{datetime.now().strftime('%Y%m%d_%H%M%S')}"
    out_root.mkdir(parents=True, exist_ok=True)
    summary_path = out_root / "convergence_study.csv"

    if args.study == "dt":
        if not args.dt_list:
            raise SystemExit("--dt-list is required for study=dt")
        dts = _parse_list(args.dt_list, float)
        cases = [(args.nx, args.ny, float(dt), float(args.dx), f"dt_{_format_float_for_path(float(dt))}") for dt in dts]
    elif args.study == "dx":
        if not args.dx_list:
            raise SystemExit("--dx-list is required for study=dx")
        dxs = _parse_list(args.dx_list, float)
        L = float(args.domain_length)
        cases = []
        for dx in dxs:
            nx = int(round(L / float(dx)))
            ny = int(round(L / float(dx)))
            if nx <= 0 or ny <= 0:
                raise SystemExit(f"Bad (nx,ny) from L/dx: L={L} dx={dx} -> nx={nx} ny={ny}")
            cases.append((nx, ny, float(args.dt), float(dx), f"dx_{_format_float_for_path(float(dx))}_n_{nx}"))
    else:
        if not args.n_list:
            raise SystemExit("--n-list is required for study=size")
        ns = _parse_list(args.n_list, int)
        cases = [(int(n), int(n), float(args.dt), float(args.dx), f"n_{int(n)}") for n in ns]

    base_dt = float(cases[0][2])
    base_dx_for_defects = float(cases[0][3])
    base_sweep = SweepConfig(
        kappa_start=float(args.kappa_start),
        kappa_end=float(args.kappa_end),
        kappa_step=float(args.kappa_step),
        initial_relax_steps=int(args.initial_relax_steps),
        relax_steps=int(args.relax_steps),
        measure_steps=int(args.measure_steps),
        sample_period=int(args.sample_period),
    )

    with summary_path.open("w", encoding="utf-8", newline="") as f:
        w = csv.writer(f)
        w.writerow(
            [
                "study",
                "run_dir",
                "nx",
                "ny",
                "dt",
                "dx",
                "b_target",
                "flux_n",
                "b_eff",
                "seed",
                "defect_mode",
                "defect_spacing",
                "alpha_default",
                "alpha_defect",
                "defect_radius",
                "defect_count",
                "kappa_start",
                "kappa_end",
                "kappa_step",
                "initial_relax_steps",
                "relax_steps",
                "measure_steps",
                "sample_period",
                "order_parameter",
                "kappa_c_method",
                "epsilon",
                "consecutive",
                "baseline_points",
                "min_segment_points",
                "kappa_c",
            ]
        )

        for nx, ny, dt, dx, label in cases:
            sweep = base_sweep
            if args.study == "dt" and args.keep_physical_time:
                scale = float(base_dt) / float(dt)
                sweep = SweepConfig(
                    kappa_start=sweep.kappa_start,
                    kappa_end=sweep.kappa_end,
                    kappa_step=sweep.kappa_step,
                    initial_relax_steps=max(1, int(round(sweep.initial_relax_steps * scale))),
                    relax_steps=max(1, int(round(sweep.relax_steps * scale))),
                    measure_steps=max(1, int(round(sweep.measure_steps * scale))),
                    sample_period=max(1, int(sweep.sample_period)),
                )

            defect_radius = int(args.defect_radius)
            defect_spacing = int(args.defect_spacing)
            defect_count = int(args.defect_count)
            if str(args.study) == "dx" and bool(args.scale_defects_with_dx):
                if dx <= 0.0:
                    raise SystemExit(f"dx must be > 0, got {dx}")
                scale = float(base_dx_for_defects) / float(dx)
                defect_radius = max(1, int(round(defect_radius * scale)))
                defect_spacing = max(1, int(round(defect_spacing * scale)))

            run_dir = out_root / f"{args.study}_{label}_nx{nx}_ny{ny}_dt{_format_float_for_path(dt)}_dx{_format_float_for_path(dx)}"
            flux_n_eff, b_eff = _run_one(
                binary=binary,
                out_dir=run_dir,
                nx=nx,
                ny=ny,
                dt=dt,
                dx=dx,
                b_target=args.b_target,
                flux_n=args.flux_n,
                seed=int(args.seed),
                defect_mode=str(args.defect_mode),
                defect_spacing=int(defect_spacing),
                alpha_default=float(args.alpha_default),
                alpha_defect=float(args.alpha_defect),
                defect_radius=int(defect_radius),
                defect_count=int(defect_count),
                dump_positions=bool(args.dump_positions),
                sweep=sweep,
            )

            kappa_csv = run_dir / "kappa_sweep.csv"
            kappa_c = estimate_kappa_c(
                kappa_csv,
                order_parameter=str(args.order_parameter),
                method=str(args.kappa_c_method),
                epsilon=float(args.epsilon),
                consecutive=int(args.consecutive),
                baseline_points=int(args.baseline_points),
                min_segment_points=int(args.min_segment_points),
            )

            w.writerow(
                [
                    str(args.study),
                    str(run_dir).replace("\\", "/"),
                    nx,
                    ny,
                    f"{dt:.8g}",
                    f"{dx:.8g}",
                    "" if args.b_target is None else f"{float(args.b_target):.8g}",
                    int(flux_n_eff),
                    f"{b_eff:.8e}",
                    int(args.seed),
                    str(args.defect_mode),
                    int(defect_spacing),
                    f"{float(args.alpha_default):.8g}",
                    f"{float(args.alpha_defect):.8g}",
                    int(defect_radius),
                    int(defect_count),
                    f"{sweep.kappa_start:.8g}",
                    f"{sweep.kappa_end:.8g}",
                    f"{sweep.kappa_step:.8g}",
                    int(sweep.initial_relax_steps),
                    int(sweep.relax_steps),
                    int(sweep.measure_steps),
                    int(sweep.sample_period),
                    str(args.order_parameter),
                    str(args.kappa_c_method),
                    f"{float(args.epsilon):.8g}",
                    int(args.consecutive),
                    int(args.baseline_points),
                    int(args.min_segment_points),
                    "" if kappa_c is None else f"{float(kappa_c):.8e}",
                ]
            )
            f.flush()
            print(f"[{args.study}] {label}: nx={nx} ny={ny} dt={dt} dx={dx} -> kappa_c={kappa_c}")

    print(f"Wrote summary: {summary_path}")


if __name__ == "__main__":
    main()
