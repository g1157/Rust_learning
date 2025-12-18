#!/usr/bin/env python3
"""
Plot results from scripts/run_convergence_study.py (convergence_study.csv).
"""

from __future__ import annotations

import argparse
from pathlib import Path

import pandas as pd

try:
    import matplotlib.pyplot as plt
except Exception:  # pragma: no cover
    plt = None  # type: ignore[assignment]


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Plot convergence_study.csv")
    p.add_argument("csv", nargs="?", default="convergence_study.csv")
    p.add_argument("--no-show", action="store_true")
    return p.parse_args()


def main() -> None:
    args = parse_args()
    csv_path = Path(args.csv)
    df = pd.read_csv(csv_path)
    if df.empty:
        raise SystemExit(f"No rows in: {csv_path}")
    if "kappa_c" not in df.columns:
        raise SystemExit("Missing required column: kappa_c")

    df = df.copy()
    df["kappa_c"] = pd.to_numeric(df["kappa_c"], errors="coerce")
    df = df[df["kappa_c"].notna()]
    if df.empty:
        raise SystemExit("No numeric kappa_c values")

    study = str(df["study"].iloc[0]) if "study" in df.columns else "unknown"
    out_path = csv_path.with_name("convergence_plot.png")

    if plt is None:
        raise SystemExit("matplotlib is required for plotting")

    fig, ax = plt.subplots(figsize=(9, 5))

    if study == "dt":
        x = df["dt"].astype(float).to_numpy()
        y = df["kappa_c"].astype(float).to_numpy()
        ax.plot(x, y, "o-", linewidth=2)
        ax.set_xscale("log")
        ax.set_xlabel("dt (log scale)")
        ax.set_title("dt convergence: kappa_c(dt)")
    elif study == "dx":
        x = df["dx"].astype(float).to_numpy()
        y = df["kappa_c"].astype(float).to_numpy()
        ax.plot(x, y, "o-", linewidth=2)
        ax.set_xscale("log")
        ax.set_xlabel("dx (log scale)")
        ax.set_title("dx convergence: kappa_c(dx)")
    elif study == "size":
        x = df["nx"].astype(int).to_numpy()
        y = df["kappa_c"].astype(float).to_numpy()
        ax.plot(x, y, "o-", linewidth=2)
        ax.set_xlabel("nx (=ny)")
        ax.set_title("finite-size: kappa_c(nx)")
    else:
        x = df.index.to_numpy()
        y = df["kappa_c"].astype(float).to_numpy()
        ax.plot(x, y, "o-", linewidth=2)
        ax.set_xlabel("row")
        ax.set_title(f"study={study}: kappa_c")

    ax.set_ylabel("kappa_c")
    ax.grid(True, alpha=0.3)
    fig.tight_layout()
    fig.savefig(out_path, dpi=150, bbox_inches="tight")
    print(f"Saved plot: {out_path}")
    if not args.no_show:
        plt.show()


if __name__ == "__main__":
    main()

