#!/usr/bin/env python3
"""
Scan matching-field (commensurability) effects by sweeping flux_n for different defect geometries.

This implements Roadmap Stage 2 "matching field" experiments:
  Compare lattice vs random defects at fixed defect parameters while scanning flux_n (B field).

Outputs:
  - matching_field.csv (summary for plotting)
  - Per-run folders with kappa_sweep.csv + vortices.csv + config/meta

Tip:
  Use --nx/--ny to change the simulation domain size. For defect_mode=lattice,
  the effective pin count N_pins depends on (nx, ny, defect_spacing).

Example:
  python scripts/run_matching_field_scan.py ^
    --flux-n-list 32,48,64,80,96 ^
    --defect-mode-list random,lattice ^
    --defect-spacing 32 ^
    --alpha-defect -0.5 --defect-radius 3 --defect-count 64 ^
    --kappa-start 0 --kappa-end 0.05 --kappa-step 0.01 ^
    --initial-relax-steps 20000 --relax-steps 2000 --measure-steps 5000 --sample-period 100 ^
    --order-parameter abs_mean_vx --kappa-c-method baseline_threshold --epsilon 1e-3 ^
    --out-root runs/matching_field_scan
"""

from __future__ import annotations

import argparse
import csv
import os
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Iterable


def _parse_list(text: str, cast):
    items = []
    for part in (p.strip() for p in text.split(",")):
        if not part:
            continue
        items.append(cast(part))
    if not items:
        raise ValueError(f"empty list: {text!r}")
    return items


def _format_float_for_path(value: float) -> str:
    return f"{value:.4g}".replace("-", "m").replace("+", "p").replace(".", "d")


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


def _read_kappa_sweep(csv_path: Path, order_parameter: str) -> tuple[list[tuple[float, float]], dict[str, float]]:
    """
    Returns (points, summary_at_kappa0).

    summary_at_kappa0 keys (best-effort):
      pinned_net_mean_k0, net_mean_k0, energy_density_mean_k0
    """
    header: list[str] | None = None
    first_row: list[str] | None = None

    points: list[tuple[float, float]] = []
    with csv_path.open("r", encoding="utf-8", newline="") as f:
        for line in f:
            if not line.strip() or line.startswith("#"):
                continue
            header = next(csv.reader([line]))
            break
        if header is None:
            raise ValueError(f"no header in {csv_path}")

        col = {name: i for i, name in enumerate(header)}
        missing = sorted(_required_columns_for_order_parameter(order_parameter) - set(col))
        if missing:
            raise ValueError(f"missing required columns in {csv_path}: {missing}")

        for row in csv.reader((ln for ln in f if ln.strip() and not ln.startswith("#"))):
            if first_row is None:
                first_row = row
            try:
                kappa = float(row[col["kappa"]])
                value = _compute_order_parameter_from_row(order_parameter, row, col)
            except (ValueError, IndexError) as e:
                raise ValueError(f"bad row in {csv_path}: {row!r}") from e
            points.append((kappa, value))

    points.sort(key=lambda t: t[0])

    summary: dict[str, float] = {}
    if first_row is not None and header is not None:
        col = {name: i for i, name in enumerate(header)}
        for key in ("pinned_net_mean", "net_mean", "energy_density_mean"):
            if key in col:
                try:
                    summary[f"{key}_k0"] = float(first_row[col[key]])
                except Exception:
                    pass

    return points, summary


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
    n = len(x)
    if n == 0:
        raise ValueError("empty fit")
    if n == 1:
        return float(y[0]), 0.0, 0.0

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
    n = len(points)
    if min_segment_points < 1:
        raise ValueError("--min-segment-points must be >= 1")
    if n < 2 * min_segment_points:
        return None

    xs = [float(k) for k, _ in points]
    ys = [float(v) for _, v in points]

    best: tuple[float, int, float, float, float] | None = None
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

    if (a2 + b2 * k_max) <= c1:
        return None

    kappa_c = (c1 - a2) / b2
    kappa_c = float(max(k_min, min(k_max, kappa_c)))
    for k, _ in points:
        if k >= kappa_c - 1e-12:
            return float(k)
    return float(points[-1][0])


def _defect_count_effective(defect_mode: str, defect_count: int, defect_spacing: int, nx: int, ny: int) -> int:
    if defect_mode == "lattice":
        if defect_spacing <= 0:
            return 0
        npx = (nx + defect_spacing - 1) // defect_spacing
        npy = (ny + defect_spacing - 1) // defect_spacing
        return int(npx * npy)
    return int(defect_count)


@dataclass(frozen=True)
class Job:
    flux_n: int
    defect_mode: str


def _iter_jobs(flux_n_list: Iterable[int], defect_mode_list: Iterable[str]) -> list[Job]:
    jobs: list[Job] = []
    for n in flux_n_list:
        for mode in defect_mode_list:
            jobs.append(Job(flux_n=int(n), defect_mode=str(mode)))
    return jobs


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Scan matching-field effects via flux_n sweeps")
    parser.add_argument("--binary", type=str, default="", help="Path to built binary (default: target/debug)")
    parser.add_argument("--build", action="store_true", help="Build with cargo if binary is missing")

    parser.add_argument("--flux-n-list", type=str, required=True, help="Comma-separated flux_n values to scan")
    parser.add_argument("--seed", type=int, default=1234)
    parser.add_argument("--nx", type=int, default=256, help="For defect_count_effective estimation (default: 256)")
    parser.add_argument("--ny", type=int, default=256, help="For defect_count_effective estimation (default: 256)")

    parser.add_argument("--alpha-default", type=float, default=1.0)
    parser.add_argument("--alpha-defect", type=float, default=-0.5)
    parser.add_argument("--defect-radius", type=int, default=3)
    parser.add_argument(
        "--defect-count",
        type=int,
        default=64,
        help="Defect count for random mode (lattice mode ignores this for geometry)",
    )
    parser.add_argument(
        "--defect-mode-list",
        type=str,
        default="random,lattice",
        help="Comma-separated defect modes to compare: random,lattice",
    )
    parser.add_argument("--defect-spacing", type=int, default=32, help="Spacing used for lattice mode")

    parser.add_argument("--kappa-start", type=float, default=0.0)
    parser.add_argument("--kappa-end", type=float, default=0.05)
    parser.add_argument("--kappa-step", type=float, default=0.01)
    parser.add_argument("--initial-relax-steps", type=int, default=0, help="0 = use --relax-steps")
    parser.add_argument("--relax-steps", type=int, default=2000)
    parser.add_argument("--measure-steps", type=int, default=5000)
    parser.add_argument("--sample-period", type=int, default=100)
    parser.add_argument(
        "--dump-positions",
        action="store_true",
        help="Also write vortex_positions.csv for structure-factor / tracking analysis",
    )

    parser.add_argument(
        "--order-parameter",
        type=str,
        default="abs_mean_vx",
        choices=["mean_speed", "abs_mean_vx", "abs_mean_vy", "abs_mean_v"],
        help="Order parameter used to detect depinning (default: abs_mean_vx)",
    )
    parser.add_argument(
        "--kappa-c-method",
        type=str,
        default="baseline_threshold",
        choices=["threshold", "baseline_threshold", "two_phase_fit"],
        help="Method to estimate kappa_c (default: baseline_threshold)",
    )
    parser.add_argument("--epsilon", type=float, default=1e-3)
    parser.add_argument("--consecutive", type=int, default=1)
    parser.add_argument("--baseline-points", type=int, default=2)
    parser.add_argument("--min-segment-points", type=int, default=2)

    parser.add_argument("--out-root", type=str, default="", help="Output directory root")
    parser.add_argument("--overwrite-summary", action="store_true", help="Overwrite matching_field.csv in out-root")
    parser.add_argument("--resume", action="store_true", help="Skip jobs whose kappa_sweep.csv exists")
    parser.add_argument("--dry-run", action="store_true", help="Print commands only")
    parser.add_argument("--max-jobs", type=int, default=0, help="Limit number of jobs (0 = no limit)")
    return parser.parse_args()


def main() -> None:
    args = parse_args()

    binary = Path(args.binary) if args.binary else _default_exe_path()
    _ensure_binary(binary, build=bool(args.build))

    out_root = Path(args.out_root) if args.out_root else Path("runs") / f"matching_field_{datetime.now():%Y%m%d_%H%M%S}"
    out_root.mkdir(parents=True, exist_ok=True)

    flux_n_list = _parse_list(args.flux_n_list, int)
    defect_mode_list = [m.strip() for m in _parse_list(args.defect_mode_list, str)]
    for m in defect_mode_list:
        if m not in ("random", "lattice"):
            raise SystemExit(f"Unknown defect mode in --defect-mode-list: {m!r}")

    jobs = _iter_jobs(flux_n_list, defect_mode_list)
    if args.max_jobs and int(args.max_jobs) > 0:
        jobs = jobs[: int(args.max_jobs)]

    summary_path = out_root / "matching_field.csv"
    mode = "w" if args.overwrite_summary else "a"
    existing_header: list[str] | None = None
    if (not args.overwrite_summary) and summary_path.exists() and summary_path.stat().st_size > 0:
        with summary_path.open("r", encoding="utf-8", newline="") as rf:
            try:
                existing_header = next(csv.reader(rf))
            except StopIteration:
                existing_header = None

    initial_relax_steps = int(args.initial_relax_steps) if int(args.initial_relax_steps) > 0 else int(args.relax_steps)

    with summary_path.open(mode, encoding="utf-8", newline="") as sf:
        writer = csv.writer(sf)
        header = existing_header or [
            "status",
            "run_dir",
            "nx",
            "ny",
            "flux_n",
            "seed",
            "defect_mode",
            "defect_spacing",
            "alpha_default",
            "alpha_defect",
            "defect_radius",
            "defect_count_requested",
            "defect_count_effective",
            "kappa_start",
            "kappa_end",
            "kappa_step",
            "initial_relax_steps",
            "relax_steps",
            "measure_steps",
            "sample_period",
            "dump_positions",
            "order_parameter",
            "kappa_c_method",
            "epsilon",
            "consecutive",
            "baseline_points",
            "min_segment_points",
            "kappa_c",
            "pinned_fraction_k0",
            "pinned_net_mean_k0",
            "net_mean_k0",
            "energy_density_mean_k0",
        ]
        if existing_header is None:
            writer.writerow(header)

        for idx, job in enumerate(jobs, start=1):
            defect_count_eff = _defect_count_effective(
                job.defect_mode,
                int(args.defect_count),
                int(args.defect_spacing),
                nx=int(args.nx),
                ny=int(args.ny),
            )

            run_dir = out_root / (
                f"mf_dm_{job.defect_mode}"
                f"_ds_{int(args.defect_spacing)}"
                f"_nx{int(args.nx)}_ny{int(args.ny)}"
                f"_n_{job.flux_n}"
                f"_ad_{_format_float_for_path(float(args.alpha_defect))}"
                f"_dr_{int(args.defect_radius)}"
                f"_dc_{int(args.defect_count)}"
                f"_seed_{int(args.seed)}"
            )
            run_dir.mkdir(parents=True, exist_ok=True)

            kappa_csv = run_dir / "kappa_sweep.csv"
            if args.resume and kappa_csv.exists():
                print(f"[{idx}/{len(jobs)}] resume: {run_dir}")
            else:
                cmd = [
                    str(binary),
                    "--headless",
                    "--nx",
                    str(int(args.nx)),
                    "--ny",
                    str(int(args.ny)),
                    "--flux-n",
                    str(job.flux_n),
                    "--seed",
                    str(int(args.seed)),
                    "--alpha-default",
                    str(float(args.alpha_default)),
                    "--alpha-defect",
                    str(float(args.alpha_defect)),
                    "--defect-radius",
                    str(int(args.defect_radius)),
                    "--defect-count",
                    str(int(args.defect_count)),
                    "--defect-mode",
                    str(job.defect_mode),
                    "--defect-spacing",
                    str(int(args.defect_spacing)),
                    "--kappa-start",
                    str(float(args.kappa_start)),
                    "--kappa-end",
                    str(float(args.kappa_end)),
                    "--kappa-step",
                    str(float(args.kappa_step)),
                    "--kappa-initial-relax-steps",
                    str(int(initial_relax_steps)),
                    "--kappa-relax-steps",
                    str(int(args.relax_steps)),
                    "--kappa-measure-steps",
                    str(int(args.measure_steps)),
                    "--sample-period",
                    str(int(args.sample_period)),
                    *([] if not args.dump_positions else ["--dump-positions"]),
                    "--out-dir",
                    str(run_dir),
                ]
                print(f"[{idx}/{len(jobs)}] run: {run_dir}")
                if args.dry_run:
                    print("  " + " ".join(cmd))
                else:
                    try:
                        subprocess.check_call(cmd)
                    except subprocess.CalledProcessError as e:
                        row_map = {
                            "status": "fail",
                            "run_dir": str(run_dir).replace("\\", "/"),
                            "nx": args.nx,
                            "ny": args.ny,
                            "flux_n": job.flux_n,
                            "seed": args.seed,
                            "defect_mode": job.defect_mode,
                            "defect_spacing": args.defect_spacing,
                            "alpha_default": args.alpha_default,
                            "alpha_defect": args.alpha_defect,
                            "defect_radius": args.defect_radius,
                            "defect_count_requested": args.defect_count,
                            "defect_count_effective": defect_count_eff,
                            "kappa_start": args.kappa_start,
                            "kappa_end": args.kappa_end,
                            "kappa_step": args.kappa_step,
                            "initial_relax_steps": initial_relax_steps,
                            "relax_steps": args.relax_steps,
                            "measure_steps": args.measure_steps,
                            "sample_period": args.sample_period,
                            "dump_positions": int(bool(args.dump_positions)),
                            "order_parameter": args.order_parameter,
                            "kappa_c_method": args.kappa_c_method,
                            "epsilon": args.epsilon,
                            "consecutive": args.consecutive,
                            "baseline_points": args.baseline_points,
                            "min_segment_points": args.min_segment_points,
                            "kappa_c": "",
                            "pinned_fraction_k0": "",
                            "pinned_net_mean_k0": "",
                            "net_mean_k0": "",
                            "energy_density_mean_k0": "",
                        }
                        writer.writerow([row_map.get(col, "") for col in header])
                        sf.flush()
                        print(f"  ERROR: job failed: {e}", file=sys.stderr)
                        continue

            try:
                points, k0 = _read_kappa_sweep(kappa_csv, order_parameter=str(args.order_parameter))
                method = str(args.kappa_c_method)
                if method == "threshold":
                    kappa_c = _estimate_kappa_c_threshold(
                        points, epsilon=float(args.epsilon), consecutive=int(args.consecutive)
                    )
                elif method == "baseline_threshold":
                    kappa_c = _estimate_kappa_c_baseline_threshold(
                        points,
                        epsilon=float(args.epsilon),
                        consecutive=int(args.consecutive),
                        baseline_points=int(args.baseline_points),
                    )
                elif method == "two_phase_fit":
                    kappa_c = _estimate_kappa_c_two_phase_fit(points, min_segment_points=int(args.min_segment_points))
                else:
                    raise ValueError(f"unknown --kappa-c-method: {method}")
            except Exception as e:
                row_map = {
                    "status": "fail",
                    "run_dir": str(run_dir).replace("\\", "/"),
                    "nx": args.nx,
                    "ny": args.ny,
                    "flux_n": job.flux_n,
                    "seed": args.seed,
                    "defect_mode": job.defect_mode,
                    "defect_spacing": args.defect_spacing,
                    "alpha_default": args.alpha_default,
                    "alpha_defect": args.alpha_defect,
                    "defect_radius": args.defect_radius,
                    "defect_count_requested": args.defect_count,
                    "defect_count_effective": defect_count_eff,
                    "kappa_start": args.kappa_start,
                    "kappa_end": args.kappa_end,
                    "kappa_step": args.kappa_step,
                    "initial_relax_steps": initial_relax_steps,
                    "relax_steps": args.relax_steps,
                    "measure_steps": args.measure_steps,
                    "sample_period": args.sample_period,
                    "dump_positions": int(bool(args.dump_positions)),
                    "order_parameter": args.order_parameter,
                    "kappa_c_method": args.kappa_c_method,
                    "epsilon": args.epsilon,
                    "consecutive": args.consecutive,
                    "baseline_points": args.baseline_points,
                    "min_segment_points": args.min_segment_points,
                    "kappa_c": "",
                    "pinned_fraction_k0": "",
                    "pinned_net_mean_k0": "",
                    "net_mean_k0": "",
                    "energy_density_mean_k0": "",
                }
                writer.writerow([row_map.get(col, "") for col in header])
                sf.flush()
                print(f"  ERROR: parse failed for {kappa_csv}: {e}", file=sys.stderr)
                continue

            pinned_net_mean_k0 = k0.get("pinned_net_mean_k0")
            net_mean_k0 = k0.get("net_mean_k0")
            energy_density_mean_k0 = k0.get("energy_density_mean_k0")
            pinned_fraction_k0 = ""
            if pinned_net_mean_k0 is not None and net_mean_k0 is not None and abs(net_mean_k0) > 1e-12:
                pinned_fraction_k0 = f"{(pinned_net_mean_k0 / net_mean_k0):.8e}"

            row_map = {
                "status": "ok",
                "run_dir": str(run_dir).replace("\\", "/"),
                "nx": args.nx,
                "ny": args.ny,
                "flux_n": job.flux_n,
                "seed": args.seed,
                "defect_mode": job.defect_mode,
                "defect_spacing": args.defect_spacing,
                "alpha_default": args.alpha_default,
                "alpha_defect": args.alpha_defect,
                "defect_radius": args.defect_radius,
                "defect_count_requested": args.defect_count,
                "defect_count_effective": defect_count_eff,
                "kappa_start": args.kappa_start,
                "kappa_end": args.kappa_end,
                "kappa_step": args.kappa_step,
                "initial_relax_steps": initial_relax_steps,
                "relax_steps": args.relax_steps,
                "measure_steps": args.measure_steps,
                "sample_period": args.sample_period,
                "dump_positions": int(bool(args.dump_positions)),
                "order_parameter": args.order_parameter,
                "kappa_c_method": args.kappa_c_method,
                "epsilon": args.epsilon,
                "consecutive": args.consecutive,
                "baseline_points": args.baseline_points,
                "min_segment_points": args.min_segment_points,
                "kappa_c": "" if kappa_c is None else f"{kappa_c:.8e}",
                "pinned_fraction_k0": pinned_fraction_k0,
                "pinned_net_mean_k0": "" if pinned_net_mean_k0 is None else f"{pinned_net_mean_k0:.8e}",
                "net_mean_k0": "" if net_mean_k0 is None else f"{net_mean_k0:.8e}",
                "energy_density_mean_k0": "" if energy_density_mean_k0 is None else f"{energy_density_mean_k0:.8e}",
            }
            writer.writerow([row_map.get(col, "") for col in header])
            sf.flush()

    print(f"Summary written: {summary_path}")


if __name__ == "__main__":
    main()
