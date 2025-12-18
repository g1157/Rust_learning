#!/usr/bin/env python3
"""
Plot a phase diagram (heatmap) from `phase_diagram.csv`.

The input is produced by:
  scripts/run_depinning_phase_diagram.py  -> phase_diagram.csv

Usage:
  python scripts/plot_phase_diagram.py runs/phase_diagram/phase_diagram.csv

Common:
  python scripts/plot_phase_diagram.py runs/phase_diagram/phase_diagram.csv --x alpha_defect --y defect_count --z kappa_c
  python scripts/plot_phase_diagram.py runs/phase_diagram/phase_diagram.csv --filter flux_n=209 --filter seed=1234 --filter defect_radius=3
"""

from __future__ import annotations

import argparse
from pathlib import Path

import matplotlib.pyplot as plt
import pandas as pd


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Plot a heatmap from phase_diagram.csv")
    parser.add_argument(
        "csv",
        nargs="?",
        default="phase_diagram.csv",
        help="Path to phase_diagram.csv (default: phase_diagram.csv)",
    )
    parser.add_argument("--x", default="alpha_defect", help="Column for x axis (default: alpha_defect)")
    parser.add_argument("--y", default="defect_count", help="Column for y axis (default: defect_count)")
    parser.add_argument("--z", default="kappa_c", help="Column for heatmap value (default: kappa_c)")
    parser.add_argument(
        "--agg",
        default="mean",
        choices=["mean", "min", "max"],
        help="Aggregation when multiple rows share the same (x,y) (default: mean)",
    )
    parser.add_argument(
        "--filter",
        action="append",
        default=[],
        help="Filter rows by equality, e.g. --filter flux_n=209 (can be repeated)",
    )
    parser.add_argument("--cmap", default="viridis", help="Matplotlib colormap (default: viridis)")
    parser.add_argument("--title", default="", help="Plot title (default: auto)")
    parser.add_argument("--no-show", action="store_true", help="Do not call plt.show()")
    return parser.parse_args()


def _parse_filters(filters: list[str]) -> list[tuple[str, str]]:
    parsed: list[tuple[str, str]] = []
    for f in filters:
        if "=" not in f:
            raise SystemExit(f"Bad --filter (expected key=value): {f!r}")
        key, value = f.split("=", 1)
        key = key.strip()
        value = value.strip()
        if not key:
            raise SystemExit(f"Bad --filter key: {f!r}")
        parsed.append((key, value))
    return parsed


def _apply_filters(df: pd.DataFrame, filters: list[tuple[str, str]]) -> pd.DataFrame:
    out = df
    for key, value in filters:
        if key not in out.columns:
            raise SystemExit(f"Unknown filter column: {key!r}")
        col = out[key]
        # try numeric match first
        try:
            v_num = float(value)
            col_num = pd.to_numeric(col, errors="coerce")
            mask = col_num.notna() & (col_num == v_num)
        except ValueError:
            mask = col.astype(str) == value
        out = out[mask]
    return out


def main() -> None:
    args = parse_args()
    csv_path = Path(args.csv)

    df = pd.read_csv(csv_path)
    if df.empty:
        raise SystemExit(f"No rows in: {csv_path}")

    df = df[df.get("status", "") == "ok"]
    df = _apply_filters(df, _parse_filters(args.filter))

    for col in (args.x, args.y, args.z):
        if col not in df.columns:
            raise SystemExit(f"Missing column {col!r} in: {csv_path}")

    df = df.copy()
    df[args.z] = pd.to_numeric(df[args.z], errors="coerce")
    df = df[df[args.z].notna()]
    if df.empty:
        raise SystemExit("No valid numeric z values after filtering")

    agg_map = {"mean": "mean", "min": "min", "max": "max"}
    pivot = df.pivot_table(index=args.y, columns=args.x, values=args.z, aggfunc=agg_map[args.agg])

    # stable ordering for axes
    pivot = pivot.sort_index(axis=0).sort_index(axis=1)

    fig, ax = plt.subplots(figsize=(10, 6))
    im = ax.imshow(pivot.values, origin="lower", aspect="auto", cmap=args.cmap)

    ax.set_xlabel(args.x)
    ax.set_ylabel(args.y)

    ax.set_xticks(range(len(pivot.columns)))
    ax.set_xticklabels([str(v) for v in pivot.columns], rotation=45, ha="right")
    ax.set_yticks(range(len(pivot.index)))
    ax.set_yticklabels([str(v) for v in pivot.index])

    title = args.title or f"{args.z} vs ({args.x}, {args.y})"
    ax.set_title(title)

    cbar = fig.colorbar(im, ax=ax)
    cbar.set_label(args.z)

    fig.tight_layout()
    out_path = csv_path.with_name("phase_diagram_plot.png")
    fig.savefig(out_path, dpi=150, bbox_inches="tight")
    print(f"Saved plot: {out_path}")

    if not args.no_show:
        plt.show()


if __name__ == "__main__":
    main()

