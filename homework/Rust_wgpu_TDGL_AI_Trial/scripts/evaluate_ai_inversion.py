#!/usr/bin/env python3
"""
Evaluate AI inversion accuracy on an existing phase_diagram.csv dataset (offline).

This script answers a practical question:
  "If I give you a target kappa_c, how well can the surrogate pick a parameter
   point that achieves it?"

To keep evaluation grounded in known ground-truth values, the candidate set is
restricted to rows that already exist in the dataset (so we can compare the
selected point's true kappa_c against the target).

If your dataset has missing kappa_c values because depinning was not observed
within the scanned kappa range, you can use --fill-missing-with-kappa-end to
treat them as censored (lower-bounded by kappa_end) for evaluation.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import pandas as pd


def _parse_list(text: str, cast):
    items = []
    for part in (p.strip() for p in str(text).split(",")):
        if not part:
            continue
        items.append(cast(part))
    if not items:
        raise SystemExit(f"empty list: {text!r}")
    return items


def _parse_filters(filters: list[str]) -> list[tuple[str, str]]:
    out: list[tuple[str, str]] = []
    for f in filters:
        if "=" not in f:
            raise SystemExit(f"Bad --filter (expected key=value): {f!r}")
        k, v = f.split("=", 1)
        k = k.strip()
        v = v.strip()
        if not k:
            raise SystemExit(f"Bad --filter key: {f!r}")
        out.append((k, v))
    return out


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


def _standardize(x: np.ndarray) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    mu = x.mean(axis=0)
    sigma = x.std(axis=0)
    sigma = np.where(sigma <= 1e-12, 1.0, sigma)
    return (x - mu) / sigma, mu, sigma


def _fit_ridge(x: np.ndarray, y: np.ndarray, lam: float) -> np.ndarray:
    if lam < 0.0:
        raise SystemExit("--lambda must be >= 0")
    n = x.shape[0]
    xb = np.column_stack([np.ones(n, dtype=float), x])
    p = xb.shape[1]
    xtx = xb.T @ xb
    reg = np.eye(p, dtype=float)
    reg[0, 0] = 0.0
    w = np.linalg.solve(xtx + lam * reg, xb.T @ y)
    return w


def _predict(x: np.ndarray, w: np.ndarray) -> np.ndarray:
    xb = np.column_stack([np.ones(x.shape[0], dtype=float), x])
    return xb @ w


def _rmse(y_true: np.ndarray, y_pred: np.ndarray) -> float:
    return float(np.sqrt(np.mean((y_true - y_pred) ** 2)))


def _r2(y_true: np.ndarray, y_pred: np.ndarray) -> float:
    denom = float(np.sum((y_true - float(np.mean(y_true))) ** 2))
    if denom <= 1e-12:
        return float("nan")
    return float(1.0 - (np.sum((y_true - y_pred) ** 2) / denom))


def _kfold_indices(n: int, k: int, seed: int) -> list[np.ndarray]:
    if k <= 1:
        return [np.arange(n)]
    rng = np.random.default_rng(seed)
    idx = np.arange(n)
    rng.shuffle(idx)
    folds = np.array_split(idx, k)
    return [np.array(f, dtype=int) for f in folds]


def _defect_count_effective_row(defect_mode: str, defect_count: float, defect_spacing: float, nx: int, ny: int) -> float:
    if str(defect_mode) == "lattice":
        if defect_spacing <= 0:
            return float("nan")
        npx = int(np.ceil(nx / defect_spacing))
        npy = int(np.ceil(ny / defect_spacing))
        return float(npx * npy)
    return float(defect_count)


@dataclass(frozen=True)
class FeatureSpec:
    numeric: list[str]
    include_defect_mode: bool
    use_defect_count_effective: bool
    nx: int
    ny: int


def _build_design_matrix(df: pd.DataFrame, spec: FeatureSpec, degree: int) -> tuple[np.ndarray, np.ndarray]:
    if degree not in (1, 2):
        raise SystemExit("--degree must be 1 or 2")
    df_local = df.copy()

    if spec.use_defect_count_effective:
        if not {"defect_mode", "defect_count", "defect_spacing"} <= set(df_local.columns):
            raise SystemExit("use_defect_count_effective requires columns: defect_mode, defect_count, defect_spacing")
        eff = []
        for _, r in df_local.iterrows():
            eff.append(
                _defect_count_effective_row(
                    str(r["defect_mode"]),
                    float(r["defect_count"]),
                    float(r["defect_spacing"]),
                    nx=int(spec.nx),
                    ny=int(spec.ny),
                )
            )
        df_local["defect_count_effective"] = eff

    numeric_cols: list[str] = []
    for c in spec.numeric:
        c = str(c)
        if spec.use_defect_count_effective and c == "defect_count":
            numeric_cols.append("defect_count_effective")
        else:
            numeric_cols.append(c)

    missing = sorted(set(numeric_cols) - set(df_local.columns))
    if missing:
        raise SystemExit(f"Missing numeric feature columns: {missing}")

    x_parts: list[np.ndarray] = []
    x1 = df_local[numeric_cols].astype(float).to_numpy(dtype=float)
    x_parts.append(x1)

    if spec.include_defect_mode:
        if "defect_mode" not in df_local.columns:
            raise SystemExit("Missing column: defect_mode")
        is_lattice = (df_local["defect_mode"].astype(str) == "lattice").astype(float).to_numpy().reshape(-1, 1)
        x_parts.append(is_lattice)

    x = np.column_stack(x_parts) if len(x_parts) > 1 else x_parts[0]

    if degree == 2:
        x2_parts = [x]
        # squares
        x2_parts.append(x * x)
        # pairwise products
        for i in range(x.shape[1]):
            for j in range(i + 1, x.shape[1]):
                x2_parts.append((x[:, i] * x[:, j]).reshape(-1, 1))
        x = np.column_stack(x2_parts)

    if "kappa_c" not in df_local.columns:
        raise SystemExit("Missing required column: kappa_c")
    y = df_local["kappa_c"].to_numpy(dtype=float)
    return x, y


def _load_dataset(csv_path: Path, filters: list[tuple[str, str]], fill_missing_with_kappa_end: bool) -> pd.DataFrame:
    df = pd.read_csv(csv_path)
    if "status" in df.columns:
        df = df[df["status"] == "ok"]
    df = _apply_filters(df, filters)
    if df.empty:
        raise SystemExit("No rows after filtering")
    df = df.copy()
    df["kappa_c"] = pd.to_numeric(df["kappa_c"], errors="coerce")
    if fill_missing_with_kappa_end:
        if "kappa_end" not in df.columns:
            raise SystemExit("--fill-missing-with-kappa-end requires column: kappa_end")
        k_end = pd.to_numeric(df["kappa_end"], errors="coerce")
        df["kappa_c"] = df["kappa_c"].fillna(k_end)
    df = df[df["kappa_c"].notna()]
    if df.empty:
        raise SystemExit("No numeric kappa_c values after parsing")
    return df


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Evaluate surrogate inversion accuracy (offline)")
    p.add_argument("csv", type=str, help="Path to phase_diagram.csv")
    p.add_argument("--filter", action="append", default=[], help="Filter by equality, e.g. --filter flux_n=64")
    p.add_argument(
        "--fill-missing-with-kappa-end",
        action="store_true",
        help="Treat missing kappa_c as kappa_end (censored) instead of dropping those rows",
    )
    p.add_argument("--numeric-features", type=str, default="alpha_defect,defect_count,defect_radius,defect_spacing")
    p.add_argument("--include-defect-mode", action="store_true")
    p.add_argument("--use-defect-count-effective", action="store_true")
    p.add_argument("--nx", type=int, default=256)
    p.add_argument("--ny", type=int, default=256)
    p.add_argument("--degree", type=int, default=2)
    p.add_argument("--lambda", dest="lam", type=float, default=1.0)
    p.add_argument("--seed", type=int, default=0)
    p.add_argument("--kfold", type=int, default=5)
    p.add_argument("--method", type=str, choices=["kfold", "loo"], default="kfold")
    p.add_argument("--delta", type=float, default=0.01, help="Report hit rate within delta (default: 0.01)")
    return p.parse_args()


def main() -> None:
    args = parse_args()
    df = _load_dataset(
        Path(args.csv),
        filters=_parse_filters(list(args.filter)),
        fill_missing_with_kappa_end=bool(args.fill_missing_with_kappa_end),
    )

    spec = FeatureSpec(
        numeric=[s.strip() for s in str(args.numeric_features).split(",") if s.strip()],
        include_defect_mode=bool(args.include_defect_mode),
        use_defect_count_effective=bool(args.use_defect_count_effective),
        nx=int(args.nx),
        ny=int(args.ny),
    )
    x, y = _build_design_matrix(df, spec=spec, degree=int(args.degree))

    n = len(y)
    if n < 3:
        raise SystemExit(f"Need >=3 rows for evaluation, got {n}")

    if args.method == "loo":
        folds = [np.array([i], dtype=int) for i in range(n)]
    else:
        folds = _kfold_indices(n, k=int(args.kfold), seed=int(args.seed))
        if len(folds) <= 1:
            raise SystemExit("kfold produced <=1 fold; increase n or decrease kfold")

    inv_errs: list[float] = []
    oracle_errs: list[float] = []
    pred_errs: list[float] = []
    hit = 0

    for test_idx in folds:
        train_idx = np.array([i for i in range(n) if i not in set(test_idx)], dtype=int)
        x_train, y_train = x[train_idx], y[train_idx]
        x_test, y_test = x[test_idx], y[test_idx]

        x_train_s, mu, sigma = _standardize(x_train)
        w = _fit_ridge(x_train_s, y_train, lam=float(args.lam))

        x_test_s = (x_test - mu) / sigma
        y_test_pred = _predict(x_test_s, w)
        pred_errs.extend(list(np.abs(y_test_pred - y_test)))

        # Inversion on candidate set = TRAIN rows (known ground truth).
        x_cand_s = (x_train - mu) / sigma
        y_cand_pred = _predict(x_cand_s, w)

        for yt in y_test:
            i_best = int(np.argmin(np.abs(y_cand_pred - yt)))
            y_best_true = float(y_train[i_best])
            inv_err = abs(float(y_best_true) - float(yt))
            inv_errs.append(float(inv_err))

            oracle = float(np.min(np.abs(y_train - yt)))
            oracle_errs.append(float(oracle))

            if inv_err <= float(args.delta):
                hit += 1

    inv_errs_np = np.array(inv_errs, dtype=float)
    oracle_np = np.array(oracle_errs, dtype=float)
    pred_np = np.array(pred_errs, dtype=float)

    print(f"rows: {n}")
    print(f"predict_abs_err: mean={float(pred_np.mean()):.4g} median={float(np.median(pred_np)):.4g}")
    print(f"invert_abs_err:  mean={float(inv_errs_np.mean()):.4g} median={float(np.median(inv_errs_np)):.4g}")
    print(f"oracle_abs_err:  mean={float(oracle_np.mean()):.4g} median={float(np.median(oracle_np)):.4g}")
    print(f"hit_rate(|err|<=delta={args.delta}): {hit}/{len(inv_errs)} = {hit/len(inv_errs):.3f}")

    # Also report global predictive fit (no CV), for context.
    x_s, mu, sigma = _standardize(x)
    w_all = _fit_ridge(x_s, y, lam=float(args.lam))
    y_pred = _predict(x_s, w_all)
    print(f"fit_rmse(all): {_rmse(y, y_pred):.4g}")
    print(f"fit_r2(all):   {_r2(y, y_pred):.4g}")


if __name__ == "__main__":
    main()
