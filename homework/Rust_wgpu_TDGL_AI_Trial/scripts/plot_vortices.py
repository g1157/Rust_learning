#!/usr/bin/env python3
"""
Plot time series from vortices.csv produced by Rust_wgpu_TDGL_AI_Trial.

Usage:
  python scripts/plot_vortices.py runs/my_run/vortices.csv --no-show
"""

from __future__ import annotations

import argparse
from pathlib import Path

import matplotlib.pyplot as plt
import pandas as pd


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Plot vortex-count time series from vortices.csv")
    p.add_argument(
        "csv",
        nargs="?",
        default="vortices.csv",
        help="Path to vortices.csv (default: vortices.csv)",
    )
    p.add_argument("--title", type=str, default="", help="Plot title (default: auto)")
    p.add_argument("--out", type=str, default="", help="Output image path (default: next to CSV)")
    p.add_argument("--no-show", action="store_true", help="Do not call plt.show()")
    p.add_argument(
        "--kappa",
        type=float,
        default=float("nan"),
        help="Optional: select a single kappa value when the CSV contains multiple kappas",
    )
    p.add_argument("--kappa-tol", type=float, default=1e-9, help="Tolerance for --kappa matching (default: 1e-9)")
    return p.parse_args()


def _pick_kappa(df: pd.DataFrame, kappa: float, tol: float) -> tuple[pd.DataFrame, float | None]:
    if "kappa" not in df.columns:
        return df, None

    ks = pd.to_numeric(df["kappa"], errors="coerce")
    uniq = sorted(float(v) for v in pd.unique(ks.dropna()))
    if not uniq:
        return df, None

    if pd.notna(kappa):
        chosen = float(kappa)
    elif len(uniq) == 1:
        chosen = float(uniq[0])
    else:
        chosen = float(uniq[-1])

    df2 = df[(ks - chosen).abs() <= float(tol)]
    return (df2 if not df2.empty else df), chosen


def main() -> None:
    args = parse_args()
    csv_path = Path(args.csv)

    df = pd.read_csv(csv_path, comment="#")
    if df.empty:
        raise SystemExit(f"No data rows found in: {csv_path}")

    required = {"time", "vortices", "antivortices"}
    missing = sorted(required - set(df.columns))
    if missing:
        raise SystemExit(f"Missing required columns in {csv_path}: {missing}")

    df, chosen_kappa = _pick_kappa(df, kappa=float(args.kappa), tol=float(args.kappa_tol))

    t = pd.to_numeric(df["time"], errors="coerce")
    v = pd.to_numeric(df["vortices"], errors="coerce")
    av = pd.to_numeric(df["antivortices"], errors="coerce")
    df = df[t.notna() & v.notna() & av.notna()]
    if df.empty:
        raise SystemExit("No numeric rows after parsing time/vortices/antivortices")

    fig, ax = plt.subplots(figsize=(10, 6))

    ax.plot(df["time"], df["vortices"], "b-", label="Vortices (+)", linewidth=2)
    ax.plot(df["time"], df["antivortices"], "r--", label="Antivortices (-)", linewidth=2)

    if "net" in df.columns:
        net = pd.to_numeric(df["net"], errors="coerce")
        if net.notna().any():
            ax.plot(df["time"], net, "k:", label="Net", linewidth=1.5)

    if "pinned_net" in df.columns:
        pinned_net = pd.to_numeric(df["pinned_net"], errors="coerce")
        if pinned_net.notna().any():
            ax.plot(df["time"], pinned_net, "m-.", label="Pinned net", linewidth=1.0)

    title = str(args.title).strip()
    if not title:
        title = "Vortex dynamics"
        if chosen_kappa is not None:
            title += f" (kappa={chosen_kappa:g})"

    ax.set_xlabel("Time (t)")
    ax.set_ylabel("Vortex count")
    ax.set_title(title)
    ax.grid(True, alpha=0.3)

    have_energy = "energy_density" in df.columns
    have_speed = "mean_speed" in df.columns
    if have_energy or have_speed:
        ax2 = ax.twinx()
        right_labels: list[str] = []

        if have_energy:
            ed = pd.to_numeric(df["energy_density"], errors="coerce")
            if ed.notna().any():
                ax2.plot(df["time"], ed, "g-", label="Energy density", linewidth=1)
                right_labels.append("Energy density")

        if have_speed:
            ms = pd.to_numeric(df["mean_speed"], errors="coerce")
            if ms.notna().any():
                ax2.plot(df["time"], ms, "c--", label="Mean speed", linewidth=1)
                right_labels.append("Mean speed")

        if right_labels:
            ax2.set_ylabel(" / ".join(right_labels))
            lines1, labels1 = ax.get_legend_handles_labels()
            lines2, labels2 = ax2.get_legend_handles_labels()
            ax.legend(lines1 + lines2, labels1 + labels2, loc="best")
        else:
            ax.legend(loc="best")
    else:
        ax.legend(loc="best")

    out_path = Path(args.out) if str(args.out).strip() else csv_path.with_name("vortices_plot.png")
    fig.tight_layout()
    fig.savefig(out_path, dpi=150, bbox_inches="tight")
    print(f"Saved plot: {out_path}")

    if not bool(args.no_show):
        plt.show()
    plt.close(fig)


if __name__ == "__main__":
    main()
