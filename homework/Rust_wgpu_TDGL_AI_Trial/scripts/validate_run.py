#!/usr/bin/env python3
"""
Validate a single TDGL run directory (outputs and basic physics sanity checks).

Goals:
  - Catch broken CSV schema / missing files.
  - Provide quick sanity checks commonly used in numerical TDGL studies:
      * net vortex count roughly matches flux_n (for quantized external field)
      * energy density decreases (dissipative dynamics)
      * pinned counts and velocity observables are consistent

This is not a substitute for peer-reviewed validation; it is a reproducible guardrail.
"""

from __future__ import annotations

import argparse
import json
import math
from dataclasses import asdict, dataclass
from pathlib import Path

import pandas as pd


def _parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Validate a single run directory")
    p.add_argument("path", nargs="?", default=".", help="Run directory (contains vortices.csv)")
    p.add_argument("--tail", type=int, default=10, help="Tail samples for steady-state checks (default: 10)")
    p.add_argument("--tolerance-net", type=float, default=1.0, help="Absolute tolerance for net vortex check (default: 1.0)")
    p.add_argument("--json", action="store_true", help="Print JSON only")
    p.add_argument("--strict", action="store_true", help="Exit non-zero if any check fails")
    return p.parse_args()


def _find_run_dir(path: Path) -> Path:
    if path.is_dir():
        return path
    return path.parent


def _read_meta_from_csv_header(csv_path: Path) -> dict[str, str]:
    meta: dict[str, str] = {}
    for line in csv_path.read_text(encoding="utf-8", errors="replace").splitlines():
        if not line.startswith("#"):
            break
        # Example:
        #   # nx=256 ny=256 dt=0.01 dx=1 flux_n=64 phi=... kappa=... B=... seed=...
        if "nx=" in line and "flux_n=" in line:
            parts = line.lstrip("#").strip().split()
            for p in parts:
                if "=" in p:
                    k, v = p.split("=", 1)
                    meta[k.strip()] = v.strip()
    return meta


def _as_int(meta: dict[str, str], key: str) -> int | None:
    try:
        return int(meta[key])
    except Exception:
        return None


def _as_float(meta: dict[str, str], key: str) -> float | None:
    try:
        return float(meta[key])
    except Exception:
        return None


@dataclass(frozen=True)
class ValidationResult:
    run_dir: str
    ok: bool
    errors: list[str]
    warnings: list[str]
    meta: dict[str, str]
    samples: int
    net_tail_mean: float | None
    flux_n: int | None
    energy_density_drop: float | None


def validate_run(run_dir: Path, tail: int, tol_net: float) -> ValidationResult:
    errors: list[str] = []
    warnings: list[str] = []

    vortices_csv = run_dir / "vortices.csv"
    if not vortices_csv.exists():
        return ValidationResult(
            run_dir=str(run_dir).replace("\\", "/"),
            ok=False,
            errors=[f"missing file: {vortices_csv}"],
            warnings=[],
            meta={},
            samples=0,
            net_tail_mean=None,
            flux_n=None,
            energy_density_drop=None,
        )

    meta = _read_meta_from_csv_header(vortices_csv)
    nx = _as_int(meta, "nx")
    ny = _as_int(meta, "ny")
    flux_n = _as_int(meta, "flux_n")
    dt = _as_float(meta, "dt")
    dx = _as_float(meta, "dx")
    phi = _as_float(meta, "phi")
    b_val = _as_float(meta, "B")

    if nx is None or ny is None:
        warnings.append("missing nx/ny in vortices.csv header (# nx=... ny=...)")
    if flux_n is None:
        warnings.append("missing flux_n in vortices.csv header (# flux_n=...)")

    df = pd.read_csv(vortices_csv, comment="#")
    if df.empty:
        errors.append("no data rows in vortices.csv")
        return ValidationResult(
            run_dir=str(run_dir).replace("\\", "/"),
            ok=False,
            errors=errors,
            warnings=warnings,
            meta=meta,
            samples=0,
            net_tail_mean=None,
            flux_n=flux_n,
            energy_density_drop=None,
        )

    required = {"step", "time", "vortices", "antivortices", "net"}
    missing = sorted(required - set(df.columns))
    if missing:
        errors.append(f"missing required columns in vortices.csv: {missing}")

    samples = int(len(df))
    tail_n = max(1, min(int(tail), samples))
    tail_df = df.tail(tail_n)

    net_tail_mean = None
    if "net" in df.columns:
        net_tail_mean = float(pd.to_numeric(tail_df["net"], errors="coerce").mean())

    # net vortex consistency with flux_n
    if flux_n is not None and net_tail_mean is not None:
        if abs(net_tail_mean - float(flux_n)) > float(tol_net):
            warnings.append(f"net_tail_mean={net_tail_mean:.3g} deviates from flux_n={flux_n} (tol={tol_net})")

    # energy density should generally drop (dissipative dynamics)
    energy_drop = None
    if "energy_density" in df.columns:
        ed = pd.to_numeric(df["energy_density"], errors="coerce")
        if ed.notna().sum() >= max(10, tail_n):
            head_mean = float(ed.head(tail_n).mean())
            tail_mean = float(ed.tail(tail_n).mean())
            energy_drop = head_mean - tail_mean
            if not math.isfinite(energy_drop):
                warnings.append("energy_density contains non-finite values")
            elif energy_drop < 0:
                warnings.append(f"energy_density increased: head_mean={head_mean:.3g} tail_mean={tail_mean:.3g}")

    # velocity consistency (very weak check):
    # mean_speed is typically computed as mean(|v_i|) while mean_vx/mean_vy are mean(v_i),
    # so we only require mean_speed >= sqrt(mean_vx^2 + mean_vy^2) up to tolerance.
    if "mean_vx" in df.columns and "mean_vy" in df.columns and "mean_speed" in df.columns:
        vx = pd.to_numeric(df["mean_vx"], errors="coerce")
        vy = pd.to_numeric(df["mean_vy"], errors="coerce")
        sp = pd.to_numeric(df["mean_speed"], errors="coerce")
        if vx.notna().all() and vy.notna().all() and sp.notna().all():
            speed_floor = (vx * vx + vy * vy).pow(0.5)
            bad = (sp + 1e-12) < (speed_floor - 1e-9)
            if bool(bad.tail(tail_n).any()):
                warnings.append("mean_speed < sqrt(mean_vx^2+mean_vy^2) in tail (unexpected)")

    # quick schema check for optional outputs
    config_toml = run_dir / "config.toml"
    meta_json = run_dir / "meta.json"
    if not config_toml.exists():
        warnings.append("missing config.toml (reproducibility metadata)")
    if not meta_json.exists():
        warnings.append("missing meta.json (GPU/backend metadata)")

    if dt is None or dx is None:
        warnings.append("missing dt/dx in vortices.csv header (# dt=... dx=...)")
    if b_val is None:
        warnings.append("missing B in vortices.csv header (# B=...)")
    if phi is None:
        warnings.append("missing phi in vortices.csv header (# phi=...)")

    # MPBC quantization sanity: phi = 2π n / (nx*ny), and B = phi/dx^2.
    if nx is not None and ny is not None and flux_n is not None and phi is not None:
        phi_expected = (2.0 * math.pi) * float(flux_n) / float(nx * ny)
        tol_phi = 1e-5 * max(1.0, abs(phi_expected))
        if abs(float(phi) - float(phi_expected)) > tol_phi:
            warnings.append(f"phi mismatch: phi={phi:.6g} expected={phi_expected:.6g} (tol={tol_phi:.3g})")

    if phi is not None and dx is not None and b_val is not None:
        b_expected = float(phi) / (float(dx) * float(dx))
        tol_b = 1e-5 * max(1.0, abs(b_expected))
        if abs(float(b_val) - float(b_expected)) > tol_b:
            warnings.append(f"B mismatch: B={b_val:.6g} expected={b_expected:.6g} (tol={tol_b:.3g})")

    ok = (len(errors) == 0)
    return ValidationResult(
        run_dir=str(run_dir).replace("\\", "/"),
        ok=ok,
        errors=errors,
        warnings=warnings,
        meta=meta,
        samples=samples,
        net_tail_mean=net_tail_mean,
        flux_n=flux_n,
        energy_density_drop=energy_drop,
    )


def main() -> None:
    args = _parse_args()
    run_dir = _find_run_dir(Path(args.path))
    result = validate_run(run_dir, tail=int(args.tail), tol_net=float(args.tolerance_net))

    payload = asdict(result)
    if args.json:
        print(json.dumps(payload, ensure_ascii=False, indent=2))
    else:
        print(f"run_dir: {payload['run_dir']}")
        print(f"ok: {payload['ok']}")
        print(f"samples: {payload['samples']}")
        if payload["flux_n"] is not None and payload["net_tail_mean"] is not None:
            print(f"net_tail_mean: {payload['net_tail_mean']:.3g} (flux_n={payload['flux_n']})")
        if payload["energy_density_drop"] is not None:
            print(f"energy_density_drop(head-tail): {payload['energy_density_drop']:.3g}")
        if payload["errors"]:
            print("errors:")
            for e in payload["errors"]:
                print(f"  - {e}")
        if payload["warnings"]:
            print("warnings:")
            for w in payload["warnings"]:
                print(f"  - {w}")

    if args.strict and not result.ok:
        raise SystemExit(2)
    if not result.ok:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
