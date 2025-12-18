#!/usr/bin/env python3
"""
Run automated kappa-sweep experiments over defect parameters and extract kappa_c.

This script is meant to implement Roadmap Stage 2:
  F_c(V_p, n_p, r_p) / kappa_c(alpha_defect, defect_count, defect_radius, ...)

It calls the project binary in headless kappa-sweep mode and writes a single
summary CSV for downstream plotting (heatmaps/phase diagrams).

Tip:
  Use --nx/--ny to trade accuracy vs speed. Note that flux_n fixes the total
  flux quanta; changing nx/ny changes the quantized B via phi=2*pi*flux_n/(nx*ny).

Usage examples:
  # single point (still writes a summary)
  python scripts/run_depinning_phase_diagram.py --flux-n 209 --seed 1234 --out-root runs/phase_diagram_smoke

  # sweep defect strength + density
  python scripts/run_depinning_phase_diagram.py ^
    --flux-n 209 --seed 1234 ^
    --alpha-defect-list=-0.2,-0.5,-1.0 ^
    --defect-count-list 0,20,50,100 ^
    --defect-radius-list 3 ^
    --kappa-start 0.0 --kappa-end 0.05 --kappa-step 0.005 ^
    --relax-steps 2000 --initial-relax-steps 20000 --measure-steps 5000 --sample-period 100 ^
    --order-parameter abs_mean_vx --kappa-c-method two_phase_fit ^
    --epsilon 1e-3 --consecutive 1 --min-segment-points 2 ^
    --out-root runs/phase_diagram
"""

from __future__ import annotations

import argparse
import csv
import itertools
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

    # If the fitted flow line never exceeds the pinned level in-range, treat as no depinning.
    if (a2 + b2 * k_max) <= c1:
        return None

    # Quantize to grid for consistency with discrete sweep points.
    # Return the first measured kappa >= kappa_c.
    for k, _ in points:
        if k >= kappa_c - 1e-12:
            return float(k)
    return float(points[-1][0])


def _format_float_for_path(value: float) -> str:
    # keep filenames stable and filesystem-friendly
    return f"{value:.4g}".replace("-", "m").replace("+", "p").replace(".", "d")


@dataclass(frozen=True)
class Job:
    alpha_default: float
    alpha_defect: float
    defect_radius: int
    defect_count: int


def _iter_jobs(
    alpha_default: float,
    alpha_defect_list: Iterable[float],
    defect_radius_list: Iterable[int],
    defect_count_list: Iterable[int],
) -> list[Job]:
    jobs: list[Job] = []
    for alpha_defect, defect_radius, defect_count in itertools.product(
        alpha_defect_list, defect_radius_list, defect_count_list
    ):
        jobs.append(
            Job(
                alpha_default=alpha_default,
                alpha_defect=float(alpha_defect),
                defect_radius=int(defect_radius),
                defect_count=int(defect_count),
            )
        )
    return jobs


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run depinning phase-diagram sweeps via kappa_sweep.csv")
    parser.add_argument("--binary", type=str, default="", help="Path to built binary (default: target/debug)")
    parser.add_argument("--build", action="store_true", help="Build with cargo if binary is missing")

    parser.add_argument("--nx", type=int, default=256, help="Simulation grid nx (default: 256)")
    parser.add_argument("--ny", type=int, default=256, help="Simulation grid ny (default: 256)")
    parser.add_argument("--flux-n", type=int, required=True)
    parser.add_argument("--seed", type=int, default=1234)

    parser.add_argument("--alpha-default", type=float, default=1.0)
    parser.add_argument(
        "--alpha-defect-list",
        type=str,
        default="-0.5",
        help="Comma-separated list (for negative values, use '=': --alpha-defect-list=-0.2,-0.5)",
    )
    parser.add_argument("--defect-radius-list", type=str, default="3")
    parser.add_argument("--defect-count-list", type=str, default="50")
    parser.add_argument(
        "--defect-mode",
        type=str,
        default="random",
        help="Defect geometry: random|lattice (default: random)",
    )
    parser.add_argument(
        "--defect-spacing",
        type=int,
        default=32,
        help="Lattice spacing in cells when defect-mode=lattice (default: 32)",
    )

    parser.add_argument("--kappa-start", type=float, default=0.0)
    parser.add_argument("--kappa-end", type=float, default=0.05)
    parser.add_argument("--kappa-step", type=float, default=0.01)
    parser.add_argument("--relax-steps", type=int, default=2000)
    parser.add_argument(
        "--initial-relax-steps",
        type=int,
        default=0,
        help="Relax steps for the first kappa point (0 = use --relax-steps)",
    )
    parser.add_argument("--measure-steps", type=int, default=5000)
    parser.add_argument("--sample-period", type=int, default=100)

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
        default="two_phase_fit",
        choices=["threshold", "baseline_threshold", "two_phase_fit"],
        help="Method to estimate kappa_c from the kappa sweep (default: two_phase_fit)",
    )
    parser.add_argument("--epsilon", type=float, default=1e-3)
    parser.add_argument("--consecutive", type=int, default=1)
    parser.add_argument(
        "--baseline-points",
        type=int,
        default=2,
        help="Number of first kappa points used as baseline for baseline_threshold (default: 2)",
    )
    parser.add_argument(
        "--min-segment-points",
        type=int,
        default=2,
        help="Minimum points per segment for two_phase_fit (default: 2)",
    )

    parser.add_argument(
        "--out-root",
        type=str,
        default="",
        help="Output root directory (default: runs/phase_diagram_<timestamp>)",
    )
    parser.add_argument(
        "--overwrite-summary",
        action="store_true",
        help="Overwrite phase_diagram.csv in out-root instead of appending",
    )
    parser.add_argument("--resume", action="store_true", help="Skip jobs whose kappa_sweep.csv exists")
    parser.add_argument("--dry-run", action="store_true", help="Print commands only")
    parser.add_argument("--max-jobs", type=int, default=0, help="Limit number of jobs (0 = no limit)")
    return parser.parse_args()


def main() -> None:
    args = parse_args()

    binary = Path(args.binary) if args.binary else _default_exe_path()
    _ensure_binary(binary, build=bool(args.build))

    out_root = Path(args.out_root) if args.out_root else Path("runs") / f"phase_diagram_{datetime.now():%Y%m%d_%H%M%S}"
    out_root.mkdir(parents=True, exist_ok=True)

    alpha_defect_list = _parse_list(args.alpha_defect_list, float)
    defect_radius_list = _parse_list(args.defect_radius_list, int)
    defect_count_list = _parse_list(args.defect_count_list, int)

    jobs = _iter_jobs(args.alpha_default, alpha_defect_list, defect_radius_list, defect_count_list)
    if args.max_jobs and args.max_jobs > 0:
        jobs = jobs[: args.max_jobs]

    summary_path = out_root / "phase_diagram.csv"
    mode = "w" if args.overwrite_summary else "a"
    wrote_header = (not args.overwrite_summary) and summary_path.exists() and summary_path.stat().st_size > 0
    with summary_path.open(mode, encoding="utf-8", newline="") as sf:
        writer = csv.writer(sf)
        if not wrote_header:
            writer.writerow(
                [
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

        for idx, job in enumerate(jobs, start=1):
            initial_relax_steps = int(args.initial_relax_steps) if int(args.initial_relax_steps) > 0 else int(args.relax_steps)
            run_dir = out_root / (
                f"dm_{args.defect_mode}"
                f"_ds_{args.defect_spacing}"
                f"_nx{int(args.nx)}_ny{int(args.ny)}"
                f"_ad_{_format_float_for_path(job.alpha_defect)}"
                f"_dc_{job.defect_count}"
                f"_dr_{job.defect_radius}"
                f"_n_{args.flux_n}"
                f"_seed_{args.seed}"
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
                    str(args.flux_n),
                    "--seed",
                    str(args.seed),
                    "--alpha-default",
                    str(job.alpha_default),
                    "--alpha-defect",
                    str(job.alpha_defect),
                    "--defect-radius",
                    str(job.defect_radius),
                    "--defect-count",
                    str(job.defect_count),
                    "--defect-mode",
                    str(args.defect_mode),
                    "--defect-spacing",
                    str(args.defect_spacing),
                    "--kappa-start",
                    str(args.kappa_start),
                    "--kappa-end",
                    str(args.kappa_end),
                    "--kappa-step",
                    str(args.kappa_step),
                    "--kappa-initial-relax-steps",
                    str(initial_relax_steps),
                    "--kappa-relax-steps",
                    str(args.relax_steps),
                    "--kappa-measure-steps",
                    str(args.measure_steps),
                    "--sample-period",
                    str(args.sample_period),
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
                        writer.writerow(
                            [
                                "fail",
                                str(run_dir).replace("\\", "/"),
                                int(args.nx),
                                int(args.ny),
                                args.flux_n,
                                args.seed,
                                args.defect_mode,
                                args.defect_spacing,
                                job.alpha_default,
                                job.alpha_defect,
                                job.defect_radius,
                                job.defect_count,
                                args.kappa_start,
                                args.kappa_end,
                                args.kappa_step,
                                initial_relax_steps,
                                args.relax_steps,
                                args.measure_steps,
                                args.sample_period,
                                args.order_parameter,
                                args.kappa_c_method,
                                args.epsilon,
                                args.consecutive,
                                args.baseline_points,
                                args.min_segment_points,
                                "",
                            ]
                        )
                        sf.flush()
                        print(f"  ERROR: job failed: {e}", file=sys.stderr)
                        continue

            try:
                points = _read_kappa_sweep(kappa_csv, order_parameter=str(args.order_parameter))
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
                writer.writerow(
                    [
                        "fail",
                        str(run_dir).replace("\\", "/"),
                        int(args.nx),
                        int(args.ny),
                        args.flux_n,
                        args.seed,
                        args.defect_mode,
                        args.defect_spacing,
                        job.alpha_default,
                        job.alpha_defect,
                        job.defect_radius,
                        job.defect_count,
                        args.kappa_start,
                        args.kappa_end,
                        args.kappa_step,
                        initial_relax_steps,
                        args.relax_steps,
                        args.measure_steps,
                        args.sample_period,
                        args.order_parameter,
                        args.kappa_c_method,
                        args.epsilon,
                        args.consecutive,
                        args.baseline_points,
                        args.min_segment_points,
                        "",
                    ]
                )
                sf.flush()
                print(f"  ERROR: parse failed for {kappa_csv}: {e}", file=sys.stderr)
                continue

            writer.writerow(
                [
                    "ok",
                    str(run_dir).replace("\\", "/"),
                    int(args.nx),
                    int(args.ny),
                    args.flux_n,
                    args.seed,
                    args.defect_mode,
                    args.defect_spacing,
                    job.alpha_default,
                    job.alpha_defect,
                    job.defect_radius,
                    job.defect_count,
                    args.kappa_start,
                    args.kappa_end,
                    args.kappa_step,
                    initial_relax_steps,
                    args.relax_steps,
                    args.measure_steps,
                    args.sample_period,
                    args.order_parameter,
                    args.kappa_c_method,
                    args.epsilon,
                    args.consecutive,
                    args.baseline_points,
                    args.min_segment_points,
                    "" if kappa_c is None else f"{kappa_c:.8e}",
                ]
            )
            sf.flush()

    print(f"Summary written: {summary_path}")


if __name__ == "__main__":
    main()
