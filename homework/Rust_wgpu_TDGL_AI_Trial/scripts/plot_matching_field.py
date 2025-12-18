#!/usr/bin/env python3
"""
Plot matching-field scan results from `matching_field.csv`.

Typical use:
  python scripts/plot_matching_field.py runs/matching_field_scan/matching_field.csv --no-show

To highlight commensurability (lattice mode):
  python scripts/plot_matching_field.py runs/matching_field_scan/matching_field.csv --show-matching --no-show
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import matplotlib.pyplot as plt
import pandas as pd


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
        try:
            v_num = float(value)
            col_num = pd.to_numeric(col, errors="coerce")
            mask = col_num.notna() & (col_num == v_num)
        except ValueError:
            mask = col.astype(str) == value
        out = out[mask]
    return out


def _parse_int_list(text: str) -> list[int]:
    items: list[int] = []
    for part in (p.strip() for p in text.split(",")):
        if not part:
            continue
        items.append(int(part))
    return items


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Plot matching field scan from matching_field.csv")
    parser.add_argument(
        "csv",
        nargs="?",
        default="matching_field.csv",
        help="Path to matching_field.csv (default: matching_field.csv)",
    )
    parser.add_argument(
        "--x",
        default="flux_n",
        help="X axis column (default: flux_n); special: b_over_bphi (uses flux_n/N_pins)",
    )
    parser.add_argument("--y", default="kappa_c", help="Y axis column (default: kappa_c)")
    parser.add_argument("--group", default="defect_mode", help="Grouping column (default: defect_mode)")
    parser.add_argument(
        "--filter",
        action="append",
        default=[],
        help="Filter rows by equality, e.g. --filter alpha_defect=-0.5 (repeatable)",
    )
    parser.add_argument("--title", default="", help="Plot title (default: auto)")
    parser.add_argument("--no-show", action="store_true", help="Do not call plt.show()")

    parser.add_argument(
        "--show-matching",
        action="store_true",
        help="Draw vertical lines at matching flux_n = m*N_pins (requires single defect_spacing)",
    )
    parser.add_argument("--nx", type=int, default=256, help="nx used for N_pins (default: 256)")
    parser.add_argument("--ny", type=int, default=256, help="ny used for N_pins (default: 256)")
    parser.add_argument(
        "--match-spacing",
        type=int,
        default=0,
        help="Override defect spacing used for N_pins (default: infer from data if unique)",
    )
    parser.add_argument(
        "--match-multiples",
        type=str,
        default="1,2,3",
        help="Multiples m for matching lines (default: 1,2,3)",
    )
    return parser.parse_args()


def _infer_n_pins(df: pd.DataFrame, nx: int, ny: int, match_spacing: int) -> tuple[int, int] | None:
    spacing = int(match_spacing)
    if spacing <= 0:
        if "defect_spacing" not in df.columns:
            return None
        uniq = sorted(pd.unique(pd.to_numeric(df["defect_spacing"], errors="coerce").dropna()))
        if len(uniq) != 1:
            return None
        spacing = int(uniq[0])

    if spacing <= 0:
        return None

    npx = (int(nx) + spacing - 1) // spacing
    npy = (int(ny) + spacing - 1) // spacing
    return (int(npx * npy), spacing)


def main() -> None:
    args = parse_args()
    csv_path = Path(args.csv)

    df = pd.read_csv(csv_path)
    if df.empty:
        raise SystemExit(f"No rows in: {csv_path}")
    if "status" in df.columns:
        df = df[df["status"] == "ok"]
    df = _apply_filters(df, _parse_filters(list(args.filter)))
    if df.empty:
        raise SystemExit("No rows after filtering")

    x_key = str(args.x).strip()
    if x_key.lower() in {"b_over_bphi", "b/bphi", "b_over_b_phi"}:
        if "flux_n" not in df.columns:
            raise SystemExit("b_over_bphi requires flux_n column")
        n_pins_info = _infer_n_pins(df, nx=int(args.nx), ny=int(args.ny), match_spacing=int(args.match_spacing))
        if not n_pins_info:
            raise SystemExit(
                "b_over_bphi requires a single defect_spacing value or an explicit --match-spacing"
            )
        n_pins, spacing = n_pins_info
        df = df.copy()
        df["_xval"] = pd.to_numeric(df["flux_n"], errors="coerce") / float(n_pins)
        x_col = "_xval"
        x_label = f"B/B_phi (N_pins={n_pins}, spacing={spacing})"
    else:
        x_col = x_key
        x_label = x_key

    for col in (x_col, args.y, args.group):
        if col not in df.columns:
            raise SystemExit(f"Missing column {col!r} in: {csv_path}")

    df = df.copy()
    df[x_col] = pd.to_numeric(df[x_col], errors="coerce")
    df[args.y] = pd.to_numeric(df[args.y], errors="coerce")
    df = df[df[x_col].notna() & df[args.y].notna()]
    if df.empty:
        raise SystemExit("No numeric x/y values after parsing")

    df = df.sort_values(x_col)
    groups = list(df.groupby(args.group, dropna=False))

    fig, ax = plt.subplots(figsize=(10, 6))
    markers = ["o", "s", "^", "D", "x", "v"]

    for i, (gname, gdf) in enumerate(groups):
        ax.plot(
            gdf[x_col],
            gdf[args.y],
            marker=markers[i % len(markers)],
            linewidth=2,
            markersize=4,
            label=str(gname),
        )

    ax.set_xlabel(x_label)
    ax.set_ylabel(args.y)
    ax.grid(True, alpha=0.3)
    title = args.title or f"{args.y} vs {x_label} (group={args.group})"
    ax.set_title(title)
    ax.legend(loc="best")

    if args.show_matching:
        n_pins_info = _infer_n_pins(df, nx=int(args.nx), ny=int(args.ny), match_spacing=int(args.match_spacing))
        if not n_pins_info:
            print(
                "NOTE: cannot infer N_pins (need single defect_spacing or pass --match-spacing); skipping matching lines",
                file=sys.stderr,
            )
        else:
            n_pins, spacing = n_pins_info
            multiples = _parse_int_list(str(args.match_multiples))
            if not multiples:
                multiples = [1]

            x_min = float(df[x_col].min())
            x_max = float(df[x_col].max())
            for m in multiples:
                if m <= 0:
                    continue
                x0 = float(m) if x_col == "_xval" else float(m * n_pins)
                if x_min <= x0 <= x_max:
                    ax.axvline(x0, color="k", linestyle="--", linewidth=1, alpha=0.4)
                    ax.text(
                        x0,
                        ax.get_ylim()[1],
                        f"m={m}",
                        rotation=90,
                        va="top",
                        ha="right",
                        fontsize=8,
                        alpha=0.6,
                    )

            ax.text(
                0.02,
                0.98,
                f"N_pins={n_pins} (spacing={spacing})",
                transform=ax.transAxes,
                va="top",
                ha="left",
                fontsize=9,
                alpha=0.8,
            )

    suffix = "matching_field_plot.png" if x_col != "_xval" else "matching_field_plot_b_over_bphi.png"
    out_path = csv_path.with_name(suffix)
    fig.tight_layout()
    fig.savefig(out_path, dpi=150, bbox_inches="tight")
    print(f"Saved plot: {out_path}")

    if not args.no_show:
        plt.show()


if __name__ == "__main__":
    main()
