#!/usr/bin/env python3
"""
AI inverse design (baseline) for depinning experiments.

Given a dataset produced by:
  scripts/run_depinning_phase_diagram.py -> phase_diagram.csv

We fit a simple ridge-regression surrogate:
  kappa_c ~= f(defect parameters)

Then we invert by searching a discrete candidate set to match a target kappa_c.
This is intentionally dependency-light (numpy/pandas only) to work offline.

Usage:
    # Train surrogate model
    python scripts/ai_inverse_design.py train phase_diagram.csv

    # Invert to find parameters for target kappa_c
    python scripts/ai_inverse_design.py invert phase_diagram.csv --target 0.03 --search-from-data

Example:
    >>> from scripts.ai_inverse_design import _fit_ridge, _predict
    >>> import numpy as np
    >>> x = np.array([[1, 2], [3, 4], [5, 6]])
    >>> y = np.array([1.0, 2.0, 3.0])
    >>> w = _fit_ridge(x, y, lam=1.0)
    >>> pred = _predict(x, w)
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable, TypeVar

import numpy as np
import pandas as pd

# Type variable for generic parsing
T = TypeVar("T")


def _parse_list(text: str, cast: Callable[[str], T]) -> list[T]:
    """Parse a comma-separated string into a list of typed values.

    Args:
        text: Comma-separated string (e.g., "1,2,3" or "a,b,c").
        cast: Function to convert each string part to the desired type.

    Returns:
        List of parsed values.

    Example:
        >>> _parse_list("1,2,3", int)
        [1, 2, 3]
        >>> _parse_list("a, b, c", str)
        ['a', 'b', 'c']
    """
    items: list[T] = []
    for part in (p.strip() for p in text.split(",")):
        if not part:
            continue
        items.append(cast(part))
    return items


def _parse_filters(filters: list[str]) -> list[tuple[str, str]]:
    """Parse filter strings in 'key=value' format.

    Args:
        filters: List of filter strings (e.g., ["flux_n=209", "seed=1234"]).

    Returns:
        List of (key, value) tuples.

    Raises:
        SystemExit: If any filter string is malformed.
    """
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
    """Apply equality filters to a DataFrame.

    Args:
        df: Input DataFrame.
        filters: List of (column_name, value) tuples to filter by.

    Returns:
        Filtered DataFrame containing only rows matching all filters.

    Raises:
        SystemExit: If a filter column doesn't exist in the DataFrame.
    """
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


def _load_dataset(csv_path: Path, filters: list[tuple[str, str]]) -> pd.DataFrame:
    """Load and preprocess a phase diagram CSV file.

    Args:
        csv_path: Path to the phase_diagram.csv file.
        filters: List of (column_name, value) tuples to filter by.

    Returns:
        Preprocessed DataFrame with numeric kappa_c values.

    Raises:
        SystemExit: If the file is empty, missing required columns, or has no valid data.
    """
    df = pd.read_csv(csv_path)
    if df.empty:
        raise SystemExit(f"No rows in: {csv_path}")

    if "status" in df.columns:
        df = df[df["status"] == "ok"]
    if df.empty:
        raise SystemExit("No ok rows after filtering by status")

    df = _apply_filters(df, filters)
    if df.empty:
        raise SystemExit("No rows after applying --filter")

    if "kappa_c" not in df.columns:
        raise SystemExit("Missing required column: kappa_c")
    df = df.copy()
    df["kappa_c"] = pd.to_numeric(df["kappa_c"], errors="coerce")
    df = df[df["kappa_c"].notna()]
    if df.empty:
        raise SystemExit("No numeric kappa_c values after parsing")

    return df


@dataclass(frozen=True)
class FeatureSpec:
    numeric: list[str]
    include_defect_mode: bool
    use_defect_count_effective: bool
    nx: int
    ny: int


def _defect_count_effective_row(defect_mode: str, defect_count: float, defect_spacing: float, nx: int, ny: int) -> float:
    """Calculate effective defect count for a single row.

    For lattice mode, computes the number of lattice sites based on grid size and spacing.
    For random mode, returns the original defect_count.

    Args:
        defect_mode: Either "lattice" or "random".
        defect_count: Number of defects (used for random mode).
        defect_spacing: Lattice spacing in cells (used for lattice mode).
        nx: Grid size in x direction.
        ny: Grid size in y direction.

    Returns:
        Effective number of defects.
    """
    if defect_mode == "lattice":
        if defect_spacing <= 0:
            return float("nan")
        npx = int(np.ceil(nx / defect_spacing))
        npy = int(np.ceil(ny / defect_spacing))
        return float(npx * npy)
    return float(defect_count)


def _build_design_matrix(df: pd.DataFrame, spec: FeatureSpec, degree: int) -> tuple[np.ndarray, np.ndarray, list[str]]:
    """Build feature matrix X and target vector y from DataFrame.

    Args:
        df: Input DataFrame with feature columns and kappa_c.
        spec: Feature specification defining which columns to use.
        degree: Polynomial degree (1 for linear, 2 for quadratic with interactions).

    Returns:
        Tuple of (X, y, feature_names) where:
        - X: Feature matrix of shape (n_samples, n_features).
        - y: Target vector of shape (n_samples,).
        - feature_names: List of feature names corresponding to columns of X.

    Raises:
        SystemExit: If degree is not 1 or 2, or if required columns are missing.
    """
    if degree not in (1, 2):
        raise SystemExit("--degree must be 1 or 2")

    required = set(spec.numeric)
    if spec.include_defect_mode:
        required.add("defect_mode")
    if spec.use_defect_count_effective:
        required.update({"defect_mode", "defect_count", "defect_spacing"})

    missing = sorted(required - set(df.columns))
    if missing:
        raise SystemExit(f"Missing required columns in dataset: {missing}")

    df_local = df.copy()
    if spec.use_defect_count_effective:
        df_local["defect_count_effective"] = [
            _defect_count_effective_row(
                str(dm),
                float(dc),
                float(ds),
                nx=int(spec.nx),
                ny=int(spec.ny),
            )
            for dm, dc, ds in zip(df_local["defect_mode"], df_local["defect_count"], df_local["defect_spacing"])
        ]

    feature_names: list[str] = []
    cols: list[np.ndarray] = []

    for name in spec.numeric:
        src = "defect_count_effective" if (spec.use_defect_count_effective and name == "defect_count") else name
        col = pd.to_numeric(df_local[src], errors="coerce").to_numpy(dtype=float)
        cols.append(col)
        feature_names.append(src)

    if spec.include_defect_mode:
        dm = df_local["defect_mode"].astype(str)
        cols.append((dm == "lattice").to_numpy(dtype=float))
        feature_names.append("defect_mode_is_lattice")

    x1 = np.column_stack(cols) if cols else np.zeros((len(df_local), 0), dtype=float)

    if degree == 1:
        x = x1
        names = feature_names
    else:
        # degree 2: [x, x^2, pairwise products]
        x_parts = [x1]
        names = list(feature_names)

        # squares
        x_parts.append(x1 * x1)
        names.extend([f"{n}^2" for n in feature_names])

        # pairwise products
        for i in range(x1.shape[1]):
            for j in range(i + 1, x1.shape[1]):
                x_parts.append((x1[:, i] * x1[:, j]).reshape(-1, 1))
                names.append(f"{feature_names[i]}*{feature_names[j]}")

        x = np.column_stack(x_parts) if x_parts else x1

    y = df_local["kappa_c"].to_numpy(dtype=float)
    return x, y, names


def _standardize(x: np.ndarray) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    """Standardize features to zero mean and unit variance.

    Args:
        x: Feature matrix of shape (n_samples, n_features).

    Returns:
        Tuple of (x_standardized, mean, std) where:
        - x_standardized: Standardized feature matrix.
        - mean: Mean of each feature.
        - std: Standard deviation of each feature (with floor of 1e-12).
    """
    mu = x.mean(axis=0)
    sigma = x.std(axis=0)
    sigma = np.where(sigma <= 1e-12, 1.0, sigma)
    return (x - mu) / sigma, mu, sigma


def _fit_ridge(x: np.ndarray, y: np.ndarray, lam: float) -> np.ndarray:
    """Fit a ridge regression model.

    Solves: min_w ||Xw - y||^2 + lambda * ||w||^2 (excluding intercept).

    Args:
        x: Standardized feature matrix of shape (n_samples, n_features).
        y: Target vector of shape (n_samples,).
        lam: Ridge regularization parameter (lambda >= 0).

    Returns:
        Weight vector of shape (n_features + 1,) where w[0] is the intercept.

    Raises:
        SystemExit: If lambda is negative.
    """
    if lam < 0.0:
        raise SystemExit("--lambda must be >= 0")
    n = x.shape[0]
    xb = np.column_stack([np.ones(n, dtype=float), x])
    p = xb.shape[1]
    xtx = xb.T @ xb
    reg = np.eye(p, dtype=float)
    reg[0, 0] = 0.0  # don't regularize intercept
    w = np.linalg.solve(xtx + lam * reg, xb.T @ y)
    return w


def _predict(x: np.ndarray, w: np.ndarray) -> np.ndarray:
    """Predict using fitted ridge regression weights.

    Args:
        x: Standardized feature matrix of shape (n_samples, n_features).
        w: Weight vector from _fit_ridge of shape (n_features + 1,).

    Returns:
        Predictions of shape (n_samples,).
    """
    xb = np.column_stack([np.ones(x.shape[0], dtype=float), x])
    return xb @ w


def _rmse(y_true: np.ndarray, y_pred: np.ndarray) -> float:
    """Calculate Root Mean Squared Error.

    Args:
        y_true: Ground truth values.
        y_pred: Predicted values.

    Returns:
        RMSE value.
    """
    return float(np.sqrt(np.mean((y_true - y_pred) ** 2)))


def _r2(y_true: np.ndarray, y_pred: np.ndarray) -> float:
    """Calculate R-squared (coefficient of determination).

    Args:
        y_true: Ground truth values.
        y_pred: Predicted values.

    Returns:
        R² value, or NaN if variance is zero.
    """
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


def _cross_val_rmse(x: np.ndarray, y: np.ndarray, lam: float, k: int, seed: int) -> float:
    n = len(y)
    folds = _kfold_indices(n, k=k, seed=seed)
    if len(folds) == 1:
        return float("nan")

    rmses: list[float] = []
    for i in range(len(folds)):
        test_idx = folds[i]
        train_idx = np.concatenate([f for j, f in enumerate(folds) if j != i])
        x_train, y_train = x[train_idx], y[train_idx]
        x_test, y_test = x[test_idx], y[test_idx]

        x_train_s, mu, sigma = _standardize(x_train)
        w = _fit_ridge(x_train_s, y_train, lam=lam)

        x_test_s = (x_test - mu) / sigma
        y_pred = _predict(x_test_s, w)
        rmses.append(_rmse(y_test, y_pred))

    return float(np.mean(rmses))


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Baseline AI inversion for phase_diagram.csv")
    sub = parser.add_subparsers(dest="cmd", required=True)

    def add_common(p: argparse.ArgumentParser) -> None:
        p.add_argument("csv", type=str, help="Path to phase_diagram.csv")
        p.add_argument(
            "--filter",
            action="append",
            default=[],
            help="Filter training rows by equality, e.g. --filter flux_n=209 (repeatable)",
        )
        p.add_argument(
            "--numeric-features",
            type=str,
            default="alpha_defect,defect_count,defect_radius,defect_spacing",
            help="Comma-separated numeric feature columns (default: alpha_defect,defect_count,defect_radius,defect_spacing)",
        )
        p.add_argument(
            "--include-defect-mode",
            action="store_true",
            help="Add one-hot feature defect_mode_is_lattice (default: off)",
        )
        p.add_argument(
            "--use-defect-count-effective",
            action="store_true",
            help="Replace defect_count with an estimated effective count for lattice mode (default: off)",
        )
        p.add_argument("--nx", type=int, default=256, help="nx used for defect_count_effective (default: 256)")
        p.add_argument("--ny", type=int, default=256, help="ny used for defect_count_effective (default: 256)")
        p.add_argument("--degree", type=int, default=2, help="Feature degree: 1 or 2 (default: 2)")
        p.add_argument("--lambda", dest="lam", type=float, default=1.0, help="Ridge lambda (default: 1.0)")
        p.add_argument("--seed", type=int, default=0, help="Random seed for CV splits (default: 0)")
        p.add_argument("--kfold", type=int, default=5, help="K-fold CV (default: 5)")

    p_train = sub.add_parser("train", help="Fit surrogate model and optionally save it")
    add_common(p_train)
    p_train.add_argument("--save", type=str, default="", help="Save fitted model JSON to this path")

    p_invert = sub.add_parser("invert", help="Train surrogate and search for parameters matching target kappa_c")
    add_common(p_invert)
    p_invert.add_argument("--target", type=float, required=True, help="Target kappa_c to invert for")
    p_invert.add_argument("--top", type=int, default=10, help="Number of candidates to print (default: 10)")
    p_invert.add_argument(
        "--search-from-data",
        action="store_true",
        help="Use unique values from the dataset for the search grid (default: off)",
    )
    p_invert.add_argument("--alpha-defect-list", type=str, default="", help="Candidate list (comma-separated)")
    p_invert.add_argument("--defect-count-list", type=str, default="", help="Candidate list (comma-separated)")
    p_invert.add_argument("--defect-radius-list", type=str, default="", help="Candidate list (comma-separated)")
    p_invert.add_argument("--defect-spacing-list", type=str, default="", help="Candidate list (comma-separated)")
    p_invert.add_argument("--defect-mode-list", type=str, default="", help="Candidate list: random,lattice")

    return parser.parse_args()


def _feature_spec_from_args(args: argparse.Namespace) -> FeatureSpec:
    numeric = _parse_list(str(args.numeric_features), str)
    if not numeric:
        raise SystemExit("--numeric-features must not be empty")
    return FeatureSpec(
        numeric=numeric,
        include_defect_mode=bool(args.include_defect_mode),
        use_defect_count_effective=bool(args.use_defect_count_effective),
        nx=int(args.nx),
        ny=int(args.ny),
    )


def _train(df: pd.DataFrame, spec: FeatureSpec, degree: int, lam: float, kfold: int, seed: int) -> dict:
    x, y, names = _build_design_matrix(df, spec=spec, degree=int(degree))
    x_s, mu, sigma = _standardize(x)
    w = _fit_ridge(x_s, y, lam=float(lam))
    y_pred = _predict(x_s, w)

    report = {
        "n": int(len(y)),
        "degree": int(degree),
        "lambda": float(lam),
        "features": names,
        "standardize": {"mean": mu.tolist(), "std": sigma.tolist()},
        "weights": {"intercept": float(w[0]), "coef": w[1:].tolist()},
        "metrics": {"rmse_train": _rmse(y, y_pred), "r2_train": _r2(y, y_pred), "rmse_kfold": _cross_val_rmse(x, y, lam=float(lam), k=int(kfold), seed=int(seed))},
    }
    return report


def _search_candidates(
    df: pd.DataFrame,
    spec: FeatureSpec,
    degree: int,
    model: dict,
    target: float,
    top: int,
    search_from_data: bool,
    alpha_defect_list: str,
    defect_count_list: str,
    defect_radius_list: str,
    defect_spacing_list: str,
    defect_mode_list: str,
) -> pd.DataFrame:
    numeric = list(spec.numeric)
    need = set(numeric)
    if spec.include_defect_mode:
        need.add("defect_mode")
    if spec.use_defect_count_effective:
        need.update({"defect_mode", "defect_count", "defect_spacing"})

    if search_from_data:
        candidates = {}
        for col in need:
            if col not in df.columns:
                raise SystemExit(f"Missing column for search-from-data: {col}")
            candidates[col] = sorted(pd.unique(df[col].astype(str if df[col].dtype == object else float)))
    else:
        candidates = {}
        if alpha_defect_list:
            candidates["alpha_defect"] = _parse_list(alpha_defect_list, float)
        if defect_count_list:
            candidates["defect_count"] = _parse_list(defect_count_list, float)
        if defect_radius_list:
            candidates["defect_radius"] = _parse_list(defect_radius_list, float)
        if defect_spacing_list:
            candidates["defect_spacing"] = _parse_list(defect_spacing_list, float)
        if defect_mode_list:
            candidates["defect_mode"] = _parse_list(defect_mode_list, str)

        missing = sorted(need - set(candidates))
        if missing:
            raise SystemExit(
                f"Missing candidate lists for: {missing}. Use --search-from-data or provide the corresponding --*-list flags."
            )

    # Cartesian product (small grids only).
    keys = sorted(candidates.keys())
    grids = [candidates[k] for k in keys]
    total = int(np.prod([len(g) for g in grids])) if grids else 0
    if total <= 0:
        raise SystemExit("Empty search grid")
    if total > 2_000_000:
        raise SystemExit(f"Search grid too large ({total} > 2,000,000). Reduce candidate lists.")

    # Build candidate DataFrame.
    rows = []
    for values in np.array(np.meshgrid(*grids, indexing="ij"), dtype=object).reshape(len(keys), -1).T:
        rows.append({k: v for k, v in zip(keys, values)})
    cand = pd.DataFrame(rows)

    # Build X and predict.
    x, _, _ = _build_design_matrix(cand.assign(kappa_c=0.0), spec=spec, degree=int(degree))
    mu = np.asarray(model["standardize"]["mean"], dtype=float)
    sigma = np.asarray(model["standardize"]["std"], dtype=float)
    w = np.asarray([model["weights"]["intercept"]] + model["weights"]["coef"], dtype=float)
    x_s = (x - mu) / sigma
    pred = _predict(x_s, w)
    cand["kappa_c_pred"] = pred
    cand["abs_error"] = np.abs(pred - float(target))
    cand = cand.sort_values(["abs_error", "kappa_c_pred"]).head(int(top)).reset_index(drop=True)
    return cand


def main() -> None:
    args = _parse_args()
    csv_path = Path(args.csv)
    filters = _parse_filters(list(args.filter))
    df = _load_dataset(csv_path, filters=filters)

    spec = _feature_spec_from_args(args)
    model = _train(
        df,
        spec=spec,
        degree=int(args.degree),
        lam=float(args.lam),
        kfold=int(args.kfold),
        seed=int(args.seed),
    )

    print("=== Surrogate model ===")
    print(f"rows: {model['n']}")
    print(f"degree: {model['degree']}  lambda: {model['lambda']}")
    print(f"train_rmse: {model['metrics']['rmse_train']:.6g}  train_r2: {model['metrics']['r2_train']:.6g}")
    print(f"kfold_rmse: {model['metrics']['rmse_kfold']:.6g}")

    if args.cmd == "train":
        if args.save:
            out = Path(args.save)
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_text(json.dumps(model, indent=2), encoding="utf-8")
            print(f"Saved model: {out}")
        return

    if args.cmd == "invert":
        cand = _search_candidates(
            df,
            spec=spec,
            degree=int(args.degree),
            model=model,
            target=float(args.target),
            top=int(args.top),
            search_from_data=bool(args.search_from_data),
            alpha_defect_list=str(args.alpha_defect_list),
            defect_count_list=str(args.defect_count_list),
            defect_radius_list=str(args.defect_radius_list),
            defect_spacing_list=str(args.defect_spacing_list),
            defect_mode_list=str(args.defect_mode_list),
        )
        print("=== Inversion candidates ===")
        print(cand.to_string(index=False))
        return

    raise SystemExit(f"Unknown cmd: {args.cmd}")


if __name__ == "__main__":
    main()

