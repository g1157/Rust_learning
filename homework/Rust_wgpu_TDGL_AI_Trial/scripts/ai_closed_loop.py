#!/usr/bin/env python3
"""
AI closed-loop runner for TDGL depinning experiments (active learning loop).

This script builds on the offline baseline:
  scripts/run_depinning_phase_diagram.py  -> phase_diagram.csv
  scripts/ai_inverse_design.py            -> ridge surrogate + grid search inversion

Closed-loop workflow (minimal dependency design):
  1) Maintain a dataset: phase_diagram.csv
  2) Fit a ridge surrogate for kappa_c(defect params)
  3) Estimate uncertainty via bootstrap ensemble
  4) Select next candidate via an acquisition function
  5) Run Rust binary (headless kappa sweep), extract kappa_c
  6) Append to dataset + JSONL log + optional progress plot

Notes:
  - --nx/--ny are passed to the Rust simulation and also used for lattice N_pins
    estimation when --use-defect-count-effective is enabled.
  - If kappa_c cannot be detected within [kappa_start, kappa_end], the dataset
    keeps kappa_c empty (censored); the JSONL log records kappa_c_filled=kappa_end
    for plotting/objective evaluation.
  - objective=target: loop_progress.png plots best-so-far |kappa_c-target|.
"""

from __future__ import annotations

import argparse
import csv
import itertools
import json
import math
import os
import random
import shutil
import subprocess
import sys
import time
from dataclasses import asdict, dataclass
from datetime import datetime
from pathlib import Path
from typing import Iterable

import numpy as np
import pandas as pd

try:
    import matplotlib.pyplot as plt
except Exception:  # pragma: no cover
    plt = None  # type: ignore[assignment]


DATASET_COLUMNS: list[str] = [
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


@dataclass(frozen=True)
class Candidate:
    flux_n: int
    seed: int
    defect_mode: str
    defect_spacing: int
    alpha_default: float
    alpha_defect: float
    defect_radius: int
    defect_count: int


def _parse_list(text: str, cast) -> list:
    items: list = []
    for part in (p.strip() for p in str(text).split(",")):
        if not part:
            continue
        items.append(cast(part))
    if not items:
        raise ValueError(f"empty list: {text!r}")
    return items


def _format_float_for_path(value: float) -> str:
    return f"{value:.4g}".replace("-", "m").replace("+", "p").replace(".", "d")


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
    # Model 1 (pinned): constant; Model 2 (flow): linear with positive slope.
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

    if (a2 + b2 * k_max) <= c1:
        return None

    # Quantize to grid for consistency with discrete sweep points.
    # Return the first measured kappa >= kappa_c.
    for k, _ in points:
        if k >= kappa_c - 1e-12:
            return float(k)
    return float(points[-1][0])


def _read_header_line(path: Path) -> list[str]:
    with path.open("r", encoding="utf-8", newline="") as f:
        return next(csv.reader([f.readline()]))


def _seed_dataset_if_needed(dataset_path: Path, seed_csv: str) -> None:
    if dataset_path.exists():
        return
    if not seed_csv:
        dataset_path.parent.mkdir(parents=True, exist_ok=True)
        with dataset_path.open("w", encoding="utf-8", newline="") as f:
            csv.writer(f).writerow(DATASET_COLUMNS)
        return

    src = Path(seed_csv)
    if not src.exists():
        raise SystemExit(f"--seed-dataset not found: {src}")

    dataset_path.parent.mkdir(parents=True, exist_ok=True)
    try:
        header = _read_header_line(src)
    except Exception as e:
        raise SystemExit(f"Failed to read seed dataset header: {src} ({e})") from e

    if header == DATASET_COLUMNS:
        shutil.copyfile(src, dataset_path)
        return

    df = pd.read_csv(src)
    for col in DATASET_COLUMNS:
        if col not in df.columns:
            df[col] = ""
    df = df[DATASET_COLUMNS]
    df.to_csv(dataset_path, index=False)


def _ensure_dataset_schema(dataset_path: Path, args: argparse.Namespace) -> None:
    if not dataset_path.exists():
        dataset_path.parent.mkdir(parents=True, exist_ok=True)
        with dataset_path.open("w", encoding="utf-8", newline="") as f:
            csv.writer(f).writerow(DATASET_COLUMNS)
        return

    try:
        header = _read_header_line(dataset_path)
    except Exception as e:
        raise SystemExit(f"Failed to read dataset header: {dataset_path} ({e})") from e

    if header == DATASET_COLUMNS:
        return

    df = pd.read_csv(dataset_path)
    for col in DATASET_COLUMNS:
        if col in df.columns:
            continue
        if col == "nx":
            df[col] = int(args.nx)
        elif col == "ny":
            df[col] = int(args.ny)
        else:
            df[col] = ""
    df["nx"] = pd.to_numeric(df["nx"], errors="coerce").fillna(int(args.nx)).astype(int)
    df["ny"] = pd.to_numeric(df["ny"], errors="coerce").fillna(int(args.ny)).astype(int)
    df = df[DATASET_COLUMNS]

    bak = dataset_path.with_suffix(dataset_path.suffix + f".bak_{datetime.now():%Y%m%d_%H%M%S}")
    shutil.copyfile(dataset_path, bak)
    df.to_csv(dataset_path, index=False)


def _norm_float(value: float, ndigits: int = 12) -> float:
    return float(round(float(value), ndigits))


def _effective_initial_relax_steps(args: argparse.Namespace) -> int:
    return int(args.initial_relax_steps) if int(args.initial_relax_steps) > 0 else int(args.relax_steps)


def _sweep_key_from_args(args: argparse.Namespace) -> tuple:
    return (
        _norm_float(args.kappa_start),
        _norm_float(args.kappa_end),
        _norm_float(args.kappa_step),
        _effective_initial_relax_steps(args),
        int(args.relax_steps),
        int(args.measure_steps),
        int(args.sample_period),
        str(args.order_parameter),
        str(args.kappa_c_method),
        _norm_float(args.epsilon),
        int(args.consecutive),
        int(args.baseline_points),
        int(args.min_segment_points),
    )


def _key_for_candidate(candidate: Candidate, args: argparse.Namespace) -> tuple:
    return (
        int(args.nx),
        int(args.ny),
        int(candidate.flux_n),
        int(candidate.seed),
        str(candidate.defect_mode),
        int(candidate.defect_spacing),
        _norm_float(candidate.alpha_default),
        _norm_float(candidate.alpha_defect),
        int(candidate.defect_radius),
        int(candidate.defect_count),
        *_sweep_key_from_args(args),
    )


def _key_for_row(row: pd.Series) -> tuple | None:
    try:
        return (
            int(row["nx"]),
            int(row["ny"]),
            int(row["flux_n"]),
            int(row["seed"]),
            str(row["defect_mode"]),
            int(row["defect_spacing"]),
            _norm_float(float(row["alpha_default"])),
            _norm_float(float(row["alpha_defect"])),
            int(row["defect_radius"]),
            int(row["defect_count"]),
            _norm_float(float(row["kappa_start"])),
            _norm_float(float(row["kappa_end"])),
            _norm_float(float(row["kappa_step"])),
            int(row["initial_relax_steps"]),
            int(row["relax_steps"]),
            int(row["measure_steps"]),
            int(row["sample_period"]),
            str(row["order_parameter"]),
            str(row["kappa_c_method"]),
            _norm_float(float(row["epsilon"])),
            int(row["consecutive"]),
            int(row["baseline_points"]),
            int(row["min_segment_points"]),
        )
    except Exception:
        return None


def _load_dataset(dataset_path: Path) -> pd.DataFrame:
    df = pd.read_csv(dataset_path)
    for col in DATASET_COLUMNS:
        if col not in df.columns:
            df[col] = ""
    return df[DATASET_COLUMNS]


def _build_candidate_grid(args: argparse.Namespace) -> list[Candidate]:
    flux_n_list = _parse_list(args.flux_n_list, int)
    seed_list = _parse_list(args.seed_list, int)
    defect_mode_list = _parse_list(args.defect_mode_list, str)
    defect_spacing_list = _parse_list(args.defect_spacing_list, int)
    alpha_defect_list = _parse_list(args.alpha_defect_list, float)
    defect_radius_list = _parse_list(args.defect_radius_list, int)
    defect_count_list = _parse_list(args.defect_count_list, int)

    for m in defect_mode_list:
        if m not in ("random", "lattice"):
            raise SystemExit(f"Unknown defect mode: {m!r} (expected: random,lattice)")
    for ds in defect_spacing_list:
        if int(ds) <= 0:
            raise SystemExit(f"defect_spacing must be > 0 (got: {ds})")

    grid: list[Candidate] = []
    for flux_n, seed, defect_mode, defect_spacing, alpha_defect, defect_radius, defect_count in itertools.product(
        flux_n_list,
        seed_list,
        defect_mode_list,
        defect_spacing_list,
        alpha_defect_list,
        defect_radius_list,
        defect_count_list,
    ):
        grid.append(
            Candidate(
                flux_n=int(flux_n),
                seed=int(seed),
                defect_mode=str(defect_mode),
                defect_spacing=int(defect_spacing),
                alpha_default=float(args.alpha_default),
                alpha_defect=float(alpha_defect),
                defect_radius=int(defect_radius),
                defect_count=int(defect_count),
            )
        )
    return grid


def _run_dir_for_candidate(out_root: Path, c: Candidate) -> Path:
    return out_root / (
        f"dm_{c.defect_mode}"
        f"_ds_{c.defect_spacing}"
        f"ad_{_format_float_for_path(c.alpha_defect)}"
        f"_dc_{c.defect_count}"
        f"_dr_{c.defect_radius}"
        f"_n_{c.flux_n}"
        f"_seed_{c.seed}"
    )


def _append_dataset_row(dataset_path: Path, row: dict) -> None:
    with dataset_path.open("a", encoding="utf-8", newline="") as f:
        w = csv.writer(f)
        w.writerow([row.get(col, "") for col in DATASET_COLUMNS])


def _load_jsonl(path: Path) -> list[dict]:
    if not path.exists():
        return []
    out: list[dict] = []
    with path.open("r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            out.append(json.loads(line))
    return out


def _build_feature_matrix(df: pd.DataFrame, args: argparse.Namespace) -> tuple[np.ndarray, list[str]]:
    degree = int(args.degree)
    if degree not in (1, 2):
        raise SystemExit("--degree must be 1 or 2")

    flux_n = pd.to_numeric(df["flux_n"], errors="coerce").to_numpy(dtype=float)
    alpha_defect = pd.to_numeric(df["alpha_defect"], errors="coerce").to_numpy(dtype=float)
    defect_radius = pd.to_numeric(df["defect_radius"], errors="coerce").to_numpy(dtype=float)
    defect_spacing = pd.to_numeric(df["defect_spacing"], errors="coerce").to_numpy(dtype=float)
    defect_count = pd.to_numeric(df["defect_count"], errors="coerce").to_numpy(dtype=float)
    defect_mode = df["defect_mode"].astype(str)
    is_lattice = (defect_mode == "lattice").to_numpy(dtype=float)

    if bool(args.use_defect_count_effective):
        ds = defect_spacing
        npx = np.ceil(float(args.nx) / ds)
        npy = np.ceil(float(args.ny) / ds)
        defect_count_eff = np.where(is_lattice > 0.5, npx * npy, defect_count)
        defect_count_vec = defect_count_eff
        defect_count_name = "defect_count_effective"
    else:
        defect_count_vec = defect_count
        defect_count_name = "defect_count"

    cols: list[np.ndarray] = [flux_n, alpha_defect, defect_radius, defect_spacing, defect_count_vec]
    names: list[str] = ["flux_n", "alpha_defect", "defect_radius", "defect_spacing", defect_count_name]

    if bool(args.include_defect_mode):
        cols.append(is_lattice)
        names.append("defect_mode_is_lattice")

    x1 = np.column_stack(cols)
    if not np.isfinite(x1).all():
        raise ValueError("Non-finite values in feature matrix (check inputs / defect_spacing).")

    if degree == 1:
        return x1, names

    x_parts = [x1]
    out_names = list(names)

    x_parts.append(x1 * x1)
    out_names.extend([f"{n}^2" for n in names])

    for i in range(x1.shape[1]):
        for j in range(i + 1, x1.shape[1]):
            x_parts.append((x1[:, i] * x1[:, j]).reshape(-1, 1))
            out_names.append(f"{names[i]}*{names[j]}")

    return np.column_stack(x_parts), out_names


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
    try:
        return np.linalg.solve(xtx + float(lam) * reg, xb.T @ y)
    except np.linalg.LinAlgError:
        w, *_ = np.linalg.lstsq(xtx + float(lam) * reg, xb.T @ y, rcond=None)
        return w


def _predict(x: np.ndarray, w: np.ndarray) -> np.ndarray:
    xb = np.column_stack([np.ones(x.shape[0], dtype=float), x])
    return xb @ w


def _bootstrap_predict(
    x_train: np.ndarray,
    y_train: np.ndarray,
    x_pool: np.ndarray,
    lam: float,
    ensemble: int,
    rng: np.random.Generator,
) -> tuple[np.ndarray, np.ndarray]:
    m = x_pool.shape[0]
    if ensemble <= 0:
        ensemble = 1
    sum_pred = np.zeros(m, dtype=float)
    sum_sq = np.zeros(m, dtype=float)

    for _ in range(int(ensemble)):
        idx = rng.integers(0, x_train.shape[0], size=x_train.shape[0])
        w = _fit_ridge(x_train[idx], y_train[idx], lam=float(lam))
        pred = _predict(x_pool, w)
        sum_pred += pred
        sum_sq += pred * pred

    mean = sum_pred / float(ensemble)
    var = sum_sq / float(ensemble) - mean * mean
    std = np.sqrt(np.maximum(var, 0.0))
    return mean, std


def _filter_ok_rows_for_sweep(df: pd.DataFrame, args: argparse.Namespace) -> pd.DataFrame:
    out = df.copy()
    out = out[out["status"].astype(str) == "ok"]

    float_eq = [
        ("kappa_start", float(args.kappa_start)),
        ("kappa_end", float(args.kappa_end)),
        ("kappa_step", float(args.kappa_step)),
        ("epsilon", float(args.epsilon)),
    ]
    int_eq = [
        ("nx", int(args.nx)),
        ("ny", int(args.ny)),
        ("initial_relax_steps", _effective_initial_relax_steps(args)),
        ("relax_steps", int(args.relax_steps)),
        ("measure_steps", int(args.measure_steps)),
        ("sample_period", int(args.sample_period)),
        ("consecutive", int(args.consecutive)),
        ("baseline_points", int(args.baseline_points)),
        ("min_segment_points", int(args.min_segment_points)),
    ]
    str_eq = [
        ("order_parameter", str(args.order_parameter)),
        ("kappa_c_method", str(args.kappa_c_method)),
    ]

    for col, value in float_eq:
        s = pd.to_numeric(out[col], errors="coerce").astype(float)
        out = out[(s - float(value)).abs() <= 1e-12]
    for col, value in int_eq:
        s = pd.to_numeric(out[col], errors="coerce")
        out = out[s.notna() & (s.astype(int) == int(value))]
    for col, value in str_eq:
        out = out[out[col].astype(str) == str(value)]

    return out


def _extract_training_xy(df_ok: pd.DataFrame, args: argparse.Namespace) -> tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    x, _ = _build_feature_matrix(df_ok, args=args)
    x_s, mu, sigma = _standardize(x)

    kappa_c = pd.to_numeric(df_ok["kappa_c"], errors="coerce")
    kappa_end = pd.to_numeric(df_ok["kappa_end"], errors="coerce")
    y = kappa_c.fillna(kappa_end).to_numpy(dtype=float)
    ok = np.isfinite(y)
    if not ok.all():
        x_s = x_s[ok]
        y = y[ok]
    if len(y) == 0:
        raise ValueError("no usable y values after parsing (kappa_c / kappa_end)")
    return x_s, y, mu, sigma


def _estimate_kappa_c_from_csv(kappa_csv: Path, args: argparse.Namespace) -> float | None:
    points = _read_kappa_sweep(kappa_csv, order_parameter=str(args.order_parameter))
    method = str(args.kappa_c_method)
    if method == "threshold":
        return _estimate_kappa_c_threshold(points, epsilon=float(args.epsilon), consecutive=int(args.consecutive))
    if method == "baseline_threshold":
        return _estimate_kappa_c_baseline_threshold(
            points,
            epsilon=float(args.epsilon),
            consecutive=int(args.consecutive),
            baseline_points=int(args.baseline_points),
        )
    if method == "two_phase_fit":
        return _estimate_kappa_c_two_phase_fit(points, min_segment_points=int(args.min_segment_points))
    raise ValueError(f"unknown --kappa-c-method: {method}")


def _evaluate_candidate(
    *,
    binary: Path,
    out_root: Path,
    candidate: Candidate,
    args: argparse.Namespace,
) -> dict:
    initial_relax_steps = _effective_initial_relax_steps(args)
    run_dir = _run_dir_for_candidate(out_root, candidate)
    run_dir.mkdir(parents=True, exist_ok=True)
    kappa_csv = run_dir / "kappa_sweep.csv"

    cmd = [
        str(binary),
        "--headless",
        "--nx",
        str(int(args.nx)),
        "--ny",
        str(int(args.ny)),
        "--flux-n",
        str(candidate.flux_n),
        "--seed",
        str(candidate.seed),
        "--alpha-default",
        str(candidate.alpha_default),
        "--alpha-defect",
        str(candidate.alpha_defect),
        "--defect-radius",
        str(candidate.defect_radius),
        "--defect-count",
        str(candidate.defect_count),
        "--defect-mode",
        str(candidate.defect_mode),
        "--defect-spacing",
        str(candidate.defect_spacing),
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

    ran = False
    if args.resume and kappa_csv.exists():
        pass
    elif args.dry_run:
        print("  " + " ".join(cmd))
        return {
            "status": "dry_run",
            "run_dir": str(run_dir).replace("\\", "/"),
            "cmd": cmd,
            "initial_relax_steps": initial_relax_steps,
            "kappa_c": None,
            "message": "dry-run",
        }
    else:
        try:
            subprocess.check_call(cmd)
            ran = True
        except subprocess.CalledProcessError as e:
            return {
                "status": "fail",
                "run_dir": str(run_dir).replace("\\", "/"),
                "cmd": cmd,
                "initial_relax_steps": initial_relax_steps,
                "kappa_c": None,
                "message": f"subprocess failed: {e}",
            }

    try:
        kappa_c = _estimate_kappa_c_from_csv(kappa_csv, args=args)
    except Exception as e:
        return {
            "status": "fail",
            "run_dir": str(run_dir).replace("\\", "/"),
            "cmd": cmd,
            "initial_relax_steps": initial_relax_steps,
            "kappa_c": None,
            "message": f"parse failed: {e}",
        }

    return {
        "status": "ok",
        "run_dir": str(run_dir).replace("\\", "/"),
        "cmd": cmd,
        "initial_relax_steps": initial_relax_steps,
        "kappa_c": kappa_c,
        "message": ("ran" if ran else "resume"),
    }


def _write_progress_plot(out_root: Path, entries: list[dict], args: argparse.Namespace) -> None:
    if bool(args.no_plot) or plt is None:
        return
    xs: list[int] = []
    ys: list[float] = []
    ys_pred: list[float] = []

    for e in entries:
        if e.get("event") != "eval":
            continue
        xs.append(int(e.get("iter", len(xs) + 1)))
        k = e.get("kappa_c_filled", e.get("kappa_c", None))
        pm = e.get("pred_mean", None)
        if str(args.objective) == "target":
            target = float(args.target)
            ys.append(float("nan") if k is None else abs(float(k) - target))
            ys_pred.append(float("nan") if pm is None else abs(float(pm) - target))
        else:
            ys.append(float("nan") if k is None else float(k))
            ys_pred.append(float("nan") if pm is None else float(pm))

    if not xs:
        return

    best: list[float] = []
    if str(args.objective) == "target":
        cur = float("inf")
        for y in ys:
            if math.isfinite(y):
                cur = min(cur, y)
            best.append(cur if cur != float("inf") else float("nan"))
    else:
        cur = float("-inf")
        for y in ys:
            if math.isfinite(y):
                cur = max(cur, y)
            best.append(cur if cur != float("-inf") else float("nan"))

    fig, ax = plt.subplots(nrows=1, ncols=1, figsize=(10, 5))
    if str(args.objective) == "target":
        ax.plot(xs, ys, "o-", linewidth=1.5, markersize=4, label="|kappa_c-target| (measured)")
        ax.plot(xs, best, "k--", linewidth=1.0, label="best so far (min)")
    else:
        ax.plot(xs, ys, "o-", linewidth=1.5, markersize=4, label="kappa_c (measured)")
        ax.plot(xs, best, "k--", linewidth=1.0, label="best so far (max)")
    if any(math.isfinite(v) for v in ys_pred):
        if str(args.objective) == "target":
            ax.plot(xs, ys_pred, "g:", linewidth=1.0, label="|kappa_c-target| (pred mean)")
        else:
            ax.plot(xs, ys_pred, "g:", linewidth=1.0, label="kappa_c (pred mean)")
    ax.set_xlabel("evaluation")
    ax.set_ylabel("|kappa_c-target|" if str(args.objective) == "target" else "kappa_c")
    ax.grid(True, alpha=0.3)
    ax.legend(loc="best")
    fig.tight_layout()

    out_path = out_root / "loop_progress.png"
    fig.savefig(out_path, dpi=150)
    plt.close(fig)


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="AI closed-loop runner (bootstrap ridge + acquisition + TDGL simulation)")

    p.add_argument("--binary", type=str, default="", help="Path to built binary (default: target/debug)")
    p.add_argument("--build", action="store_true", help="Build with cargo if binary is missing")

    p.add_argument("--out-root", type=str, default="", help="Output root directory (default: runs/ai_closed_loop_<ts>)")
    p.add_argument("--seed-dataset", type=str, default="", help="Seed an initial phase_diagram.csv (optional)")
    p.add_argument("--resume", action="store_true", help="Resume if dataset/run dirs already exist")
    p.add_argument("--dry-run", action="store_true", help="Print commands but do not run simulation")

    # Candidate domain (discrete grid).
    p.add_argument("--flux-n-list", type=str, default="209")
    p.add_argument("--seed-list", type=str, default="1234")
    p.add_argument("--defect-mode-list", type=str, default="random")
    p.add_argument("--defect-spacing-list", type=str, default="32")
    p.add_argument("--alpha-default", type=float, default=1.0)
    p.add_argument(
        "--alpha-defect-list",
        type=str,
        default="-0.5",
        help="Comma-separated list (for negative values, use '=': --alpha-defect-list=-0.2,-0.5)",
    )
    p.add_argument("--defect-radius-list", type=str, default="3")
    p.add_argument("--defect-count-list", type=str, default="50")

    # Kappa sweep settings.
    p.add_argument("--kappa-start", type=float, default=0.0)
    p.add_argument("--kappa-end", type=float, default=0.05)
    p.add_argument("--kappa-step", type=float, default=0.01)
    p.add_argument("--initial-relax-steps", type=int, default=20000, help="Warm-up for first kappa point")
    p.add_argument("--relax-steps", type=int, default=2000)
    p.add_argument("--measure-steps", type=int, default=5000)
    p.add_argument("--sample-period", type=int, default=100)

    # kappa_c extraction settings (same choices as other scripts).
    p.add_argument(
        "--order-parameter",
        type=str,
        default="abs_mean_vx",
        choices=["mean_speed", "abs_mean_vx", "abs_mean_vy", "abs_mean_v"],
    )
    p.add_argument(
        "--kappa-c-method",
        type=str,
        default="two_phase_fit",
        choices=["threshold", "baseline_threshold", "two_phase_fit"],
    )
    p.add_argument("--epsilon", type=float, default=1e-3)
    p.add_argument("--consecutive", type=int, default=1)
    p.add_argument("--baseline-points", type=int, default=2)
    p.add_argument("--min-segment-points", type=int, default=2)

    # Model + acquisition.
    p.add_argument("--degree", type=int, default=2, help="Feature degree: 1 or 2 (default: 2)")
    p.add_argument("--lambda", dest="lam", type=float, default=1.0, help="Ridge lambda (default: 1.0)")
    p.add_argument("--ensemble", type=int, default=32, help="Bootstrap ensemble size (default: 32)")
    p.add_argument("--beta", type=float, default=1.0, help="UCB beta (default: 1.0)")
    p.add_argument("--objective", type=str, default="maximize", choices=["maximize", "target"])
    p.add_argument("--target", type=float, default=0.03, help="Target kappa_c for objective=target")
    p.add_argument("--init-random", type=int, default=4, help="Random evals before model (default: 4)")
    p.add_argument("--iters", type=int, default=10, help="Number of new evaluations to run (default: 10)")
    p.add_argument("--bootstrap-seed", type=int, default=0, help="Seed for bootstrap sampling (default: 0)")

    # Feature engineering knobs.
    p.add_argument("--include-defect-mode", action="store_true", help="Add defect_mode_is_lattice feature")
    p.add_argument("--use-defect-count-effective", action="store_true", help="Use effective defect count for lattice")
    p.add_argument("--nx", type=int, default=256, help="Simulation grid nx (also for defect_count_effective)")
    p.add_argument("--ny", type=int, default=256, help="Simulation grid ny (also for defect_count_effective)")

    # Plotting.
    p.add_argument("--no-plot", action="store_true", help="Disable progress plot generation")

    return p.parse_args()


def main() -> None:
    args = parse_args()
    out_root = Path(args.out_root) if args.out_root else Path("runs") / f"ai_closed_loop_{datetime.now():%Y%m%d_%H%M%S}"
    out_root.mkdir(parents=True, exist_ok=True)

    dataset_path = out_root / "phase_diagram.csv"
    _seed_dataset_if_needed(dataset_path, seed_csv=str(args.seed_dataset))
    _ensure_dataset_schema(dataset_path, args=args)

    binary = Path(args.binary) if args.binary else _default_exe_path()
    _ensure_binary(binary, build=bool(args.build))

    log_path = out_root / "loop_log.jsonl"
    print(f"out_root: {out_root}")
    print(f"dataset:  {dataset_path}")
    print(f"log:      {log_path}")
    print(
        "settings:",
        f"objective={args.objective}",
        f"beta={args.beta}",
        f"degree={args.degree}",
        f"lambda={args.lam}",
        f"ensemble={args.ensemble}",
    )

    all_candidates = _build_candidate_grid(args)
    if not all_candidates:
        raise SystemExit("Empty candidate grid (check your --*-list flags).")
    print(f"candidate_grid: {len(all_candidates)}")

    entries = _load_jsonl(log_path)
    next_iter = 1
    if entries:
        iters = [int(e.get("iter", 0)) for e in entries if isinstance(e, dict)]
        next_iter = max([0, *iters]) + 1

    rng_select = random.Random(int(args.bootstrap_seed))
    rng_boot = np.random.default_rng(int(args.bootstrap_seed))

    done_keys_session: set[tuple] = set()
    evals_done = 0

    while evals_done < int(args.iters):
        df = _load_dataset(dataset_path)

        ok_keys: set[tuple] = set()
        for _, row in df.iterrows():
            if str(row["status"]) != "ok":
                continue
            key = _key_for_row(row)
            if key is not None:
                ok_keys.add(key)

        pool = [c for c in all_candidates if (_key_for_candidate(c, args) not in ok_keys and _key_for_candidate(c, args) not in done_keys_session)]
        if not pool:
            print("No candidates left (for the current sweep settings).")
            break

        df_ok_sweep = _filter_ok_rows_for_sweep(df, args=args)
        n_ok = int(len(df_ok_sweep))
        use_random = n_ok < int(args.init_random) or n_ok < 2

        selected: Candidate
        pred_mean: float | None = None
        pred_std: float | None = None
        acq_best: float | None = None
        policy = "random"

        if use_random:
            selected = rng_select.choice(pool)
        else:
            try:
                x_train_s, y_train, mu, sigma = _extract_training_xy(df_ok_sweep, args=args)
                pool_df = pd.DataFrame([asdict(c) for c in pool])
                x_pool, _ = _build_feature_matrix(pool_df, args=args)
                x_pool_s = (x_pool - mu) / sigma
                mean, std = _bootstrap_predict(
                    x_train=x_train_s,
                    y_train=y_train,
                    x_pool=x_pool_s,
                    lam=float(args.lam),
                    ensemble=int(args.ensemble),
                    rng=rng_boot,
                )
                mean = np.clip(mean, float(args.kappa_start), float(args.kappa_end))
                beta = float(args.beta)
                if str(args.objective) == "maximize":
                    acq = mean + beta * std
                else:
                    acq = -np.abs(mean - float(args.target)) + beta * std
                best_idx = int(np.nanargmax(acq))
                selected = pool[best_idx]
                pred_mean = float(mean[best_idx])
                pred_std = float(std[best_idx])
                acq_best = float(acq[best_idx])
                policy = "model"
            except Exception as e:
                print(f"  WARN: model selection failed ({e}); falling back to random", file=sys.stderr)
                selected = rng_select.choice(pool)
                policy = "random_fallback"

        iter_no = next_iter + evals_done
        print(
            f"[{iter_no}] select ({policy})",
            f"dm={selected.defect_mode}",
            f"ds={selected.defect_spacing}",
            f"ad={selected.alpha_defect:g}",
            f"dc={selected.defect_count}",
            f"dr={selected.defect_radius}",
            f"n={selected.flux_n}",
            f"seed={selected.seed}",
            ("" if pred_mean is None else f"pred={pred_mean:g}±{(pred_std or 0.0):g} acq={acq_best:g}"),
        )

        t0 = time.time()
        result = _evaluate_candidate(binary=binary, out_root=out_root, candidate=selected, args=args)
        elapsed_s = float(time.time() - t0)

        if result["status"] == "dry_run":
            done_keys_session.add(_key_for_candidate(selected, args))
            evals_done += 1
            continue

        status = "ok" if result["status"] == "ok" else "fail"
        kappa_c = result.get("kappa_c", None)
        kappa_c_filled = None
        if status == "ok":
            kappa_c_filled = float(args.kappa_end) if kappa_c is None else float(kappa_c)
        run_dir = str(result["run_dir"])

        dataset_row = {
            "status": status,
            "run_dir": run_dir,
            "nx": int(args.nx),
            "ny": int(args.ny),
            "flux_n": selected.flux_n,
            "seed": selected.seed,
            "defect_mode": selected.defect_mode,
            "defect_spacing": selected.defect_spacing,
            "alpha_default": selected.alpha_default,
            "alpha_defect": selected.alpha_defect,
            "defect_radius": selected.defect_radius,
            "defect_count": selected.defect_count,
            "kappa_start": args.kappa_start,
            "kappa_end": args.kappa_end,
            "kappa_step": args.kappa_step,
            "initial_relax_steps": int(result["initial_relax_steps"]),
            "relax_steps": args.relax_steps,
            "measure_steps": args.measure_steps,
            "sample_period": args.sample_period,
            "order_parameter": args.order_parameter,
            "kappa_c_method": args.kappa_c_method,
            "epsilon": args.epsilon,
            "consecutive": args.consecutive,
            "baseline_points": args.baseline_points,
            "min_segment_points": args.min_segment_points,
            "kappa_c": "" if kappa_c is None else f"{float(kappa_c):.8e}",
        }
        _append_dataset_row(dataset_path, dataset_row)

        entry = {
            "event": "eval",
            "iter": int(iter_no),
            "time": datetime.now().isoformat(timespec="seconds"),
            "policy": policy,
            "objective": str(args.objective),
            "target": float(args.target),
            "beta": float(args.beta),
            "pred_mean": pred_mean,
            "pred_std": pred_std,
            "pred_abs_err": (None if pred_mean is None else abs(float(pred_mean) - float(args.target))),
            "acq": acq_best,
            "train_n_ok": n_ok,
            "pool_n": len(pool),
            "candidate": asdict(selected),
            "run_dir": run_dir,
            "status": status,
            "kappa_c": kappa_c,
            "kappa_c_filled": kappa_c_filled,
            "abs_err": (None if kappa_c_filled is None else abs(float(kappa_c_filled) - float(args.target))),
            "elapsed_s": elapsed_s,
            "message": str(result.get("message", "")),
            "cmd": list(result.get("cmd", [])),
        }

        with log_path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(entry, ensure_ascii=True) + "\n")
        entries.append(entry)
        _write_progress_plot(out_root, entries, args=args)

        evals_done += 1

    eval_ok = [
        e
        for e in entries
        if e.get("event") == "eval" and e.get("status") == "ok" and e.get("kappa_c_filled") is not None
    ]
    if eval_ok:
        measured_f = [float(e["kappa_c_filled"]) for e in eval_ok]
        if str(args.objective) == "target":
            target = float(args.target)
            best_e = min(eval_ok, key=lambda e: abs(float(e["kappa_c_filled"]) - target))
            best_k = float(best_e["kappa_c_filled"])
            best_err = abs(best_k - target)
            print(f"best_abs_err: {best_err:g}  (target={target:g}, best_kappa_c={best_k:g}, n_ok={len(eval_ok)})")
            print(f"best_run_dir: {str(best_e.get('run_dir',''))}")
        else:
            print(f"best_kappa_c: {max(measured_f):g}  (n_ok={len(eval_ok)})")
    print("done.")


if __name__ == "__main__":
    main()
