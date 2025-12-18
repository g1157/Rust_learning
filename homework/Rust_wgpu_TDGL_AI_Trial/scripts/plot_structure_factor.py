#!/usr/bin/env python3
"""
Compute and plot a 2D structure factor S(k) from `vortex_positions.csv`.

Input file format (produced by `--dump-positions`):
  step,time,kappa,x_cell,y_cell,sign

This is an offline diagnostic to quantify ordering (e.g. Abrikosov lattice peaks)
and to support matching-field / geometry studies.

Examples:
  # Plot S(k) at the last sampled (kappa, step)
  python scripts/plot_structure_factor.py runs/kappa_sweep/vortex_positions.csv --no-show

  # Plot S(k) for a specific kappa (within tolerance)
  python scripts/plot_structure_factor.py runs/kappa_sweep/vortex_positions.csv --kappa 0.02 --no-show
"""

from __future__ import annotations

import argparse
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Plot vortex structure factor from vortex_positions.csv")
    parser.add_argument(
        "csv",
        nargs="?",
        default="vortex_positions.csv",
        help="Path to vortex_positions.csv (default: vortex_positions.csv)",
    )
    parser.add_argument("--nx", type=int, default=256, help="Grid nx (default: 256)")
    parser.add_argument("--ny", type=int, default=256, help="Grid ny (default: 256)")
    parser.add_argument(
        "--kappa",
        type=float,
        default=None,
        help="Select a specific kappa (default: last kappa in file)",
    )
    parser.add_argument(
        "--kappa-tol",
        type=float,
        default=1e-6,
        help="Tolerance when selecting kappa (default: 1e-6)",
    )
    parser.add_argument(
        "--step",
        type=int,
        default=None,
        help="Select a specific step (default: last step for selected kappa)",
    )
    parser.add_argument(
        "--sign",
        type=str,
        default="vortex",
        choices=["vortex", "antivortex", "net", "all"],
        help="Which objects to include (default: vortex)",
    )
    parser.add_argument(
        "--log10",
        action="store_true",
        help="Plot log10(S+1) instead of linear S (default: off)",
    )
    parser.add_argument("--title", type=str, default="", help="Plot title (default: auto)")
    parser.add_argument("--no-show", action="store_true", help="Do not call plt.show()")
    return parser.parse_args()


def _select_kappa(df: pd.DataFrame, kappa: float | None, tol: float) -> float:
    kappa_vals = pd.to_numeric(df["kappa"], errors="coerce").dropna().to_numpy(dtype=float)
    if kappa_vals.size == 0:
        raise SystemExit("No numeric kappa values found")
    if kappa is None:
        return float(np.max(kappa_vals))
    # pick the closest available kappa in file
    uniq = np.unique(kappa_vals)
    idx = int(np.argmin(np.abs(uniq - float(kappa))))
    chosen = float(uniq[idx])
    if abs(chosen - float(kappa)) > float(tol):
        raise SystemExit(f"Requested kappa={kappa:g} but nearest in file is {chosen:g} (tol={tol:g})")
    return chosen


def _build_density(
    df: pd.DataFrame, nx: int, ny: int, sign_mode: str
) -> tuple[np.ndarray, int, int, float, float]:
    # Returns (rho, n_pos, n_neg, step, time)
    rho = np.zeros((ny, nx), dtype=np.float32)

    x = pd.to_numeric(df["x_cell"], errors="coerce").to_numpy(dtype=int)
    y = pd.to_numeric(df["y_cell"], errors="coerce").to_numpy(dtype=int)
    s = pd.to_numeric(df["sign"], errors="coerce").to_numpy(dtype=int)

    valid = (x >= 0) & (x < nx) & (y >= 0) & (y < ny) & (s != 0)
    x = x[valid]
    y = y[valid]
    s = s[valid]

    if sign_mode == "vortex":
        mask = s > 0
        x, y, s = x[mask], y[mask], s[mask]
        w = np.ones_like(s, dtype=np.float32)
    elif sign_mode == "antivortex":
        mask = s < 0
        x, y, s = x[mask], y[mask], s[mask]
        w = np.ones_like(s, dtype=np.float32)
    elif sign_mode == "all":
        w = np.ones_like(s, dtype=np.float32)
    elif sign_mode == "net":
        w = s.astype(np.float32)
    else:
        raise SystemExit(f"Unknown --sign: {sign_mode!r}")

    n_pos = int(np.sum(s > 0))
    n_neg = int(np.sum(s < 0))

    # Accumulate onto grid
    np.add.at(rho, (y, x), w)

    step = float(pd.to_numeric(df["step"], errors="coerce").dropna().iloc[0])
    time = float(pd.to_numeric(df["time"], errors="coerce").dropna().iloc[0])
    return rho, n_pos, n_neg, step, time


def main() -> None:
    args = parse_args()
    csv_path = Path(args.csv)

    df = pd.read_csv(csv_path, comment="#")
    if df.empty:
        raise SystemExit(f"No data rows found in: {csv_path}")

    required = {"step", "time", "kappa", "x_cell", "y_cell", "sign"}
    missing = sorted(required - set(df.columns))
    if missing:
        raise SystemExit(f"Missing required columns {missing} in: {csv_path}")

    chosen_kappa = _select_kappa(df, args.kappa, tol=float(args.kappa_tol))
    dfk = df[np.abs(pd.to_numeric(df["kappa"], errors="coerce") - chosen_kappa) <= float(args.kappa_tol)]
    if dfk.empty:
        raise SystemExit(f"No rows for kappa={chosen_kappa:g} in: {csv_path}")

    if args.step is None:
        chosen_step = int(pd.to_numeric(dfk["step"], errors="coerce").dropna().max())
    else:
        chosen_step = int(args.step)

    dfs = dfk[pd.to_numeric(dfk["step"], errors="coerce") == float(chosen_step)]
    if dfs.empty:
        raise SystemExit(f"No rows for (kappa={chosen_kappa:g}, step={chosen_step}) in: {csv_path}")

    rho, n_pos, n_neg, step0, time0 = _build_density(dfs, nx=int(args.nx), ny=int(args.ny), sign_mode=str(args.sign))

    # FFT-based structure factor
    fft = np.fft.fft2(rho)
    s2 = np.abs(np.fft.fftshift(fft)) ** 2
    s2 = s2.astype(np.float64)

    # Remove DC peak for peak search
    cx = s2.shape[1] // 2
    cy = s2.shape[0] // 2
    s2_nodc = s2.copy()
    s2_nodc[cy, cx] = 0.0
    peak_idx = int(np.argmax(s2_nodc))
    py, px = np.unravel_index(peak_idx, s2_nodc.shape)
    peak_val = float(s2_nodc[py, px])

    # Convert to k coordinates (cycles per cell)
    kx = np.fft.fftshift(np.fft.fftfreq(int(args.nx), d=1.0))
    ky = np.fft.fftshift(np.fft.fftfreq(int(args.ny), d=1.0))
    peak_kx = float(kx[px])
    peak_ky = float(ky[py])

    print("=== Structure factor ===")
    print(f"file: {csv_path}")
    print(f"selected: kappa={chosen_kappa:g}  step={int(step0)}  time={time0:g}")
    print(f"counts: vortices={n_pos}  antivortices={n_neg}  mode={args.sign}")
    print(f"peak (excluding DC): S={peak_val:.6g} at (kx,ky)=({peak_kx:.6g},{peak_ky:.6g}) cycles/cell")

    data = np.log10(s2 + 1.0) if args.log10 else s2
    cmap = "magma"

    fig, ax = plt.subplots(figsize=(7, 6))
    im = ax.imshow(
        data,
        origin="lower",
        cmap=cmap,
        extent=[float(kx[0]), float(kx[-1]), float(ky[0]), float(ky[-1])],
        aspect="equal",
    )
    ax.plot([peak_kx], [peak_ky], "c+", markersize=10, markeredgewidth=1.5)
    ax.set_xlabel("kx (cycles/cell)")
    ax.set_ylabel("ky (cycles/cell)")
    title = args.title or f"S(k) at kappa={chosen_kappa:g}, step={chosen_step} ({args.sign})"
    ax.set_title(title)
    cbar = fig.colorbar(im, ax=ax)
    cbar.set_label("log10(S+1)" if args.log10 else "S")

    out_name = f"structure_factor_kappa_{_format_float_for_path(float(chosen_kappa))}_step_{chosen_step}.png"
    out_path = csv_path.with_name(out_name)
    fig.tight_layout()
    fig.savefig(out_path, dpi=150, bbox_inches="tight")
    print(f"Saved plot: {out_path}")

    if not args.no_show:
        plt.show()


def _format_float_for_path(value: float) -> str:
    return f"{value:.4g}".replace("-", "m").replace("+", "p").replace(".", "d")


if __name__ == "__main__":
    main()

