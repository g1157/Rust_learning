#!/usr/bin/env python3
"""
Plot depinning curve from `kappa_sweep.csv` produced by Rust_wgpu_TDGL_AI_Trial.

Usage:
  python scripts/plot_kappa_sweep.py path/to/kappa_sweep.csv

Output:
  - kappa_sweep_plot.png (saved next to the CSV)
"""

from __future__ import annotations

import argparse
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Plot a depinning order parameter from kappa_sweep.csv")
    parser.add_argument(
        "csv",
        nargs="?",
        default="kappa_sweep.csv",
        help="Path to kappa_sweep.csv (default: kappa_sweep.csv)",
    )
    parser.add_argument(
        "--epsilon",
        type=float,
        default=1e-3,
        help="Threshold for depinning estimate (default: 1e-3)",
    )
    parser.add_argument(
        "--order-parameter",
        type=str,
        default="abs_mean_vx",
        choices=["mean_speed", "abs_mean_vx", "abs_mean_vy", "abs_mean_v"],
        help="Order parameter to plot and use for kappa_c estimate (default: abs_mean_vx)",
    )
    parser.add_argument(
        "--kappa-c-method",
        type=str,
        default="two_phase_fit",
        choices=["threshold", "baseline_threshold", "two_phase_fit"],
        help="Method to estimate kappa_c (default: two_phase_fit)",
    )
    parser.add_argument(
        "--baseline-points",
        type=int,
        default=2,
        help="Baseline points for baseline_threshold (default: 2)",
    )
    parser.add_argument(
        "--min-segment-points",
        type=int,
        default=2,
        help="Minimum points per segment for two_phase_fit (default: 2)",
    )
    parser.add_argument(
        "--no-show",
        action="store_true",
        help="Do not call plt.show() (useful for batch runs)",
    )
    return parser.parse_args()


def _compute_order_parameter(df: pd.DataFrame, order_parameter: str) -> tuple[str, pd.Series]:
    if order_parameter == "mean_speed":
        if "mean_speed" not in df.columns:
            raise SystemExit("Missing required column: mean_speed")
        return "mean_speed", df["mean_speed"].astype(float)

    if order_parameter == "abs_mean_vx":
        if "mean_vx" not in df.columns:
            raise SystemExit("Missing required column: mean_vx")
        return "|mean_vx|", df["mean_vx"].astype(float).abs()

    if order_parameter == "abs_mean_vy":
        if "mean_vy" not in df.columns:
            raise SystemExit("Missing required column: mean_vy")
        return "|mean_vy|", df["mean_vy"].astype(float).abs()

    if order_parameter == "abs_mean_v":
        if "mean_vx" not in df.columns or "mean_vy" not in df.columns:
            raise SystemExit("Missing required columns: mean_vx, mean_vy")
        vx = df["mean_vx"].astype(float)
        vy = df["mean_vy"].astype(float)
        return "sqrt(mean_vx^2 + mean_vy^2)", np.sqrt(vx * vx + vy * vy)

    raise SystemExit(f"Unknown order parameter: {order_parameter!r}")


def _estimate_kappa_c_threshold(kappa: np.ndarray, v: np.ndarray, epsilon: float) -> float | None:
    mask = v > epsilon
    if not mask.any():
        return None
    return float(kappa[mask.argmax()])


def _estimate_kappa_c_baseline_threshold(
    kappa: np.ndarray, v: np.ndarray, epsilon: float, baseline_points: int
) -> float | None:
    if baseline_points <= 0:
        raise ValueError("--baseline-points must be >= 1")
    if len(v) < baseline_points:
        return None
    baseline = float(np.median(v[:baseline_points]))
    threshold = baseline + float(epsilon)
    mask = v > threshold
    if not mask.any():
        return None
    # Skip the baseline window when searching for crossings.
    for i in range(baseline_points, len(kappa)):
        if mask[i]:
            return float(kappa[i])
    return None


def _fit_line(x: np.ndarray, y: np.ndarray) -> tuple[float, float, float]:
    # y = a + b x
    if len(x) == 0:
        raise ValueError("empty fit")
    if len(x) == 1:
        a = float(y[0])
        b = 0.0
        return a, b, 0.0
    A = np.column_stack([np.ones_like(x), x])
    coef, *_ = np.linalg.lstsq(A, y, rcond=None)
    a, b = float(coef[0]), float(coef[1])
    yhat = A @ coef
    sse = float(((y - yhat) ** 2).sum())
    return a, b, sse


def _estimate_kappa_c_two_phase_fit(kappa: np.ndarray, v: np.ndarray, min_segment_points: int) -> float | None:
    # Model 1: constant; Model 2: linear with positive slope.
    n = len(kappa)
    if min_segment_points < 1:
        raise ValueError("--min-segment-points must be >= 1")
    if n < 2 * min_segment_points:
        return None

    best = None  # (sse, split, c1, a2, b2)
    for split in range(min_segment_points, n - min_segment_points + 1):
        v1 = v[:split]
        c1 = float(v1.mean())
        sse1 = float(((v1 - c1) ** 2).sum())

        a2, b2, sse2 = _fit_line(kappa[split:], v[split:])
        if b2 <= 0.0:
            continue
        total = sse1 + sse2
        if best is None or total < best[0]:
            best = (total, split, c1, a2, b2)

    if best is None:
        return None
    _, split, c1, a2, b2 = best

    k_min = float(kappa[0])
    k_max = float(kappa[-1])
    kappa_c = (c1 - a2) / b2
    kappa_c = float(max(k_min, min(k_max, kappa_c)))

    if (a2 + b2 * k_max) <= c1:
        return None

    # Quantize: first measured kappa >= kappa_c.
    for k in kappa:
        if float(k) >= kappa_c - 1e-12:
            return float(k)
    return float(kappa[-1])


def main() -> None:
    args = parse_args()
    csv_path = Path(args.csv)

    df = pd.read_csv(csv_path, comment="#")
    if df.empty:
        raise SystemExit(f"No data rows found in: {csv_path}")

    df = df.sort_values("kappa")
    if "kappa" not in df.columns:
        raise SystemExit(f"Missing required column 'kappa' in: {csv_path}")

    ylabel, v = _compute_order_parameter(df, str(args.order_parameter))
    kappa = df["kappa"].astype(float).to_numpy()
    v_np = v.to_numpy(dtype=float)

    have_pinning = "pinned_net_mean" in df.columns
    have_net = "net_mean" in df.columns
    have_energy = "energy_density_mean" in df.columns

    nrows = 2 if (have_pinning or have_net or have_energy) else 1
    fig, axes = plt.subplots(nrows=nrows, ncols=1, figsize=(10, 6), sharex=(nrows == 2))
    if nrows == 1:
        axes = [axes]

    ax0 = axes[0]
    ax0.plot(kappa, v_np, "o-", linewidth=2, markersize=4, label=ylabel)
    ax0.set_ylabel(ylabel)
    ax0.set_title("Depinning curve")
    ax0.grid(True, alpha=0.3)
    ax0.legend(loc="best")

    if nrows == 2:
        ax1 = axes[1]
        left_lines = []
        left_labels = []

        if have_pinning:
            left_lines += ax1.plot(
                df["kappa"],
                df["pinned_net_mean"],
                "m.-",
                linewidth=1,
                markersize=3,
                label="Pinned net (mean)",
            )
            left_labels.append("Pinned net (mean)")

        if have_net:
            left_lines += ax1.plot(
                df["kappa"],
                df["net_mean"],
                "k--",
                linewidth=1,
                label="Net (mean)",
            )
            left_labels.append("Net (mean)")

        ax1.set_xlabel("kappa")
        ax1.set_ylabel("counts")
        ax1.grid(True, alpha=0.3)

        right_lines = []
        right_labels = []
        ax1r = None
        if have_energy:
            ax1r = ax1.twinx()
            right_lines += ax1r.plot(
                df["kappa"],
                df["energy_density_mean"],
                "g:",
                linewidth=1,
                label="Energy density (mean)",
            )
            right_labels.append("Energy density (mean)")
            ax1r.set_ylabel("energy_density_mean")

        if left_lines or right_lines:
            ax1.legend(left_lines + right_lines, left_labels + right_labels, loc="best")

    eps = float(args.epsilon)
    kappa_c: float | None = None
    method = str(args.kappa_c_method)
    if method == "threshold":
        kappa_c = _estimate_kappa_c_threshold(kappa, v_np, epsilon=eps)
    elif method == "baseline_threshold":
        kappa_c = _estimate_kappa_c_baseline_threshold(
            kappa, v_np, epsilon=eps, baseline_points=int(args.baseline_points)
        )
    elif method == "two_phase_fit":
        kappa_c = _estimate_kappa_c_two_phase_fit(kappa, v_np, min_segment_points=int(args.min_segment_points))
    else:
        raise SystemExit(f"Unknown --kappa-c-method: {method!r}")

    if kappa_c is not None:
        ax0.axvline(kappa_c, color="r", linestyle="--", linewidth=1, label=f"kappa_c={kappa_c:g}")
        ax0.legend(loc="best")
        print(f"Estimated kappa_c ({method}, {args.order_parameter}): {kappa_c:g}")
    else:
        print(f"No depinning detected by method={method} for: {csv_path}")

    out_path = csv_path.with_name("kappa_sweep_plot.png")
    fig.tight_layout()
    fig.savefig(out_path, dpi=150, bbox_inches="tight")
    print(f"Saved plot: {out_path}")

    if not args.no_show:
        plt.show()


if __name__ == "__main__":
    main()
