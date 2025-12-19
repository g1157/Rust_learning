#!/usr/bin/env python3
"""
Plot literature comparison figures for matching field experiments.

Reference: Reichhardt et al. PRB 64, 052503 (2001)

Usage:
  python scripts/plot_lit_compare.py runs/lit_compare/matching_field.csv --no-show
"""

from __future__ import annotations

import argparse
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Plot literature comparison figures")
    parser.add_argument(
        "csv",
        nargs="?",
        default="runs/lit_compare/matching_field.csv",
        help="Path to matching_field.csv",
    )
    parser.add_argument("--no-show", action="store_true", help="Do not call plt.show()")
    parser.add_argument("--out-dir", type=str, default="", help="Output directory (default: same as CSV)")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    csv_path = Path(args.csv)
    out_dir = Path(args.out_dir) if args.out_dir else csv_path.parent

    df = pd.read_csv(csv_path, comment="#")
    if df.empty:
        raise SystemExit(f"No data in: {csv_path}")

    # 分离 random 和 lattice 数据
    random_df = df[df["defect_mode"] == "random"].sort_values("flux_n")
    lattice_df = df[df["defect_mode"] == "lattice"].sort_values("flux_n")

    # 图1: 钉扎涡旋数 vs B/B_phi
    fig1, ax1 = plt.subplots(figsize=(10, 6))

    ax1.plot(
        random_df["b_over_bphi"],
        random_df["pinned_net"],
        "o-",
        linewidth=2,
        markersize=8,
        label="Random defects",
        color="blue",
    )
    ax1.plot(
        lattice_df["b_over_bphi"],
        lattice_df["pinned_net"],
        "s-",
        linewidth=2,
        markersize=8,
        label="Periodic lattice",
        color="red",
    )

    # 标记整数匹配场
    for m in [1, 2]:
        ax1.axvline(m, color="gray", linestyle="--", linewidth=1, alpha=0.5)
        ax1.text(m, ax1.get_ylim()[1] * 0.95, f"B/B_φ={m}", ha="center", fontsize=9, alpha=0.7)

    ax1.set_xlabel("B/B_φ (Matching field ratio)", fontsize=12)
    ax1.set_ylabel("Pinned vortex count (net)", fontsize=12)
    ax1.set_title("Matching Field Effect: Random vs Periodic Pinning Arrays\n(Ref: Reichhardt et al. PRB 64, 052503 (2001))", fontsize=11)
    ax1.legend(loc="upper left", fontsize=10)
    ax1.grid(True, alpha=0.3)
    ax1.set_xlim(0.3, 2.2)

    out1 = out_dir / "lit_compare_pinned_vs_bphi.png"
    fig1.tight_layout()
    fig1.savefig(out1, dpi=150, bbox_inches="tight")
    print(f"Saved: {out1}")

    # 图2: 增强比 (lattice/random) vs B/B_phi
    fig2, ax2 = plt.subplots(figsize=(10, 6))

    # 合并数据计算增强比
    merged = pd.merge(
        random_df[["flux_n", "b_over_bphi", "pinned_net"]],
        lattice_df[["flux_n", "pinned_net"]],
        on="flux_n",
        suffixes=("_random", "_lattice"),
    )
    merged["enhancement"] = merged["pinned_net_lattice"] / merged["pinned_net_random"].replace(0, np.nan)

    ax2.bar(
        merged["b_over_bphi"],
        merged["enhancement"],
        width=0.3,
        color=["green" if x in [1.0, 2.0] else "steelblue" for x in merged["b_over_bphi"]],
        edgecolor="black",
        alpha=0.8,
    )

    ax2.axhline(1.0, color="gray", linestyle="--", linewidth=1, alpha=0.7)
    ax2.set_xlabel("B/B_φ (Matching field ratio)", fontsize=12)
    ax2.set_ylabel("Enhancement ratio (Lattice / Random)", fontsize=12)
    ax2.set_title("Pinning Enhancement at Integer Matching Fields\n(Green bars: integer B/B_φ)", fontsize=11)
    ax2.grid(True, alpha=0.3, axis="y")

    # 添加数值标签
    for i, row in merged.iterrows():
        ax2.text(
            row["b_over_bphi"],
            row["enhancement"] + 0.05,
            f"{row['enhancement']:.2f}×",
            ha="center",
            fontsize=10,
            fontweight="bold",
        )

    out2 = out_dir / "lit_compare_enhancement.png"
    fig2.tight_layout()
    fig2.savefig(out2, dpi=150, bbox_inches="tight")
    print(f"Saved: {out2}")

    # 图3: 涡旋总数 vs B/B_phi
    fig3, ax3 = plt.subplots(figsize=(10, 6))

    ax3.plot(
        random_df["b_over_bphi"],
        random_df["vortices"],
        "o-",
        linewidth=2,
        markersize=8,
        label="Random (vortices)",
        color="blue",
    )
    ax3.plot(
        lattice_df["b_over_bphi"],
        lattice_df["vortices"],
        "s-",
        linewidth=2,
        markersize=8,
        label="Lattice (vortices)",
        color="red",
    )
    ax3.plot(
        random_df["b_over_bphi"],
        random_df["net"],
        "o--",
        linewidth=1.5,
        markersize=6,
        label="Random (net = flux_n)",
        color="blue",
        alpha=0.5,
    )
    ax3.plot(
        lattice_df["b_over_bphi"],
        lattice_df["net"],
        "s--",
        linewidth=1.5,
        markersize=6,
        label="Lattice (net = flux_n)",
        color="red",
        alpha=0.5,
    )

    ax3.set_xlabel("B/B_φ (Matching field ratio)", fontsize=12)
    ax3.set_ylabel("Vortex count", fontsize=12)
    ax3.set_title("Total Vortex Count vs Matching Field", fontsize=11)
    ax3.legend(loc="upper left", fontsize=10)
    ax3.grid(True, alpha=0.3)

    out3 = out_dir / "lit_compare_vortex_count.png"
    fig3.tight_layout()
    fig3.savefig(out3, dpi=150, bbox_inches="tight")
    print(f"Saved: {out3}")

    # 图4: 综合对比表格图
    fig4, ax4 = plt.subplots(figsize=(12, 5))
    ax4.axis("off")

    table_data = []
    headers = ["B/B_φ", "flux_n", "Random\npinned", "Lattice\npinned", "Enhancement", "Physical interpretation"]

    interpretations = {
        0.5: "Sub-matching: weak enhancement",
        1.0: "First matching field: strong enhancement",
        1.5: "Fractional: moderate enhancement",
        2.0: "Second matching field: strong enhancement",
    }

    for _, row in merged.iterrows():
        b_ratio = row["b_over_bphi"]
        table_data.append([
            f"{b_ratio:.1f}",
            f"{int(row['flux_n'])}",
            f"{int(row['pinned_net_random'])}",
            f"{int(row['pinned_net_lattice'])}",
            f"{row['enhancement']:.2f}×",
            interpretations.get(b_ratio, ""),
        ])

    table = ax4.table(
        cellText=table_data,
        colLabels=headers,
        loc="center",
        cellLoc="center",
        colColours=["#4472C4"] * len(headers),
    )
    table.auto_set_font_size(False)
    table.set_fontsize(10)
    table.scale(1.2, 1.8)

    # 设置表头颜色
    for i in range(len(headers)):
        table[(0, i)].set_text_props(color="white", fontweight="bold")

    # 高亮整数匹配场行
    for row_idx, row in enumerate(merged.itertuples(), start=1):
        if row.b_over_bphi in [1.0, 2.0]:
            for col_idx in range(len(headers)):
                table[(row_idx, col_idx)].set_facecolor("#E2EFDA")

    ax4.set_title(
        "Summary: Matching Field Comparison with Reichhardt et al. PRB 64, 052503 (2001)\n"
        "(Green rows: integer matching fields with enhanced pinning)",
        fontsize=11,
        pad=20,
    )

    out4 = out_dir / "lit_compare_summary_table.png"
    fig4.tight_layout()
    fig4.savefig(out4, dpi=150, bbox_inches="tight")
    print(f"Saved: {out4}")

    if not args.no_show:
        plt.show()

    plt.close("all")
    print(f"\nAll figures saved to: {out_dir}")


if __name__ == "__main__":
    main()
