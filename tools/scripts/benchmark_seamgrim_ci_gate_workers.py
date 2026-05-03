#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import statistics
import subprocess
import sys
import time
from pathlib import Path


def _parse_int_list(text: str, *, default: list[int]) -> list[int]:
    raw = [part.strip() for part in str(text or "").split(",")]
    values: list[int] = []
    for part in raw:
        if not part:
            continue
        try:
            value = int(part)
        except ValueError:
            continue
        if value > 0:
            values.append(value)
    if values:
        return values
    return list(default)


def _run_once(root: Path, worker: int, family_worker: int, *, profile: str, only_step: str) -> tuple[int, int]:
    env = os.environ.copy()
    env["DDN_SEAMGRIM_CI_GATE_MAX_WORKERS"] = str(worker)
    env["DDN_SEAMGRIM_CI_GATE_FAMILY_MAX_WORKERS"] = str(family_worker)
    cmd = [
        sys.executable,
        "tests/run_ci_sanity_gate.py",
        "--profile",
        profile,
        "--only-step",
        only_step,
    ]
    started = time.perf_counter()
    proc = subprocess.run(
        cmd,
        cwd=root,
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    elapsed_ms = int((time.perf_counter() - started) * 1000)
    return proc.returncode, elapsed_ms


def main() -> int:
    parser = argparse.ArgumentParser(description="Benchmark seamgrim ci gate worker/family-worker combinations")
    parser.add_argument("--workers", default="10,12,14", help="comma-separated DDN_SEAMGRIM_CI_GATE_MAX_WORKERS values")
    parser.add_argument(
        "--family-workers",
        default="8,10,12",
        help="comma-separated DDN_SEAMGRIM_CI_GATE_FAMILY_MAX_WORKERS values",
    )
    parser.add_argument("--runs", type=int, default=3, help="runs per pair (default: 3)")
    parser.add_argument("--profile", default="seamgrim", help="ci_sanity profile (default: seamgrim)")
    parser.add_argument("--only-step", default="age3_close", help="ci_sanity step (default: age3_close)")
    parser.add_argument("--json-out", default="", help="optional JSON summary path")
    parser.add_argument(
        "--env-out",
        default="",
        help="optional shell env snippet output path for best worker pair",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[2]
    workers = _parse_int_list(args.workers, default=[10, 12, 14])
    family_workers = _parse_int_list(args.family_workers, default=[8, 10, 12])
    runs = max(1, int(args.runs))

    rows: list[dict[str, object]] = []
    for worker in workers:
        for family_worker in family_workers:
            samples: list[int] = []
            for run_idx in range(1, runs + 1):
                code, elapsed_ms = _run_once(
                    root,
                    worker,
                    family_worker,
                    profile=str(args.profile),
                    only_step=str(args.only_step),
                )
                print(
                    f"worker={worker} family_worker={family_worker} run={run_idx}/{runs} "
                    f"elapsed_ms={elapsed_ms} code={code}"
                )
                if code != 0:
                    print(f"[fail] benchmark aborted worker={worker} family_worker={family_worker} run={run_idx}")
                    return 1
                samples.append(elapsed_ms)
            row = {
                "worker": worker,
                "family_worker": family_worker,
                "runs": runs,
                "samples_ms": samples,
                "min_ms": min(samples),
                "max_ms": max(samples),
                "mean_ms": int(statistics.fmean(samples)),
                "median_ms": int(statistics.median(samples)),
            }
            rows.append(row)

    rows_sorted = sorted(
        rows,
        key=lambda row: (
            int(row["median_ms"]),
            int(row["mean_ms"]),
            int(row["max_ms"]),
            int(row["worker"]),
            int(row["family_worker"]),
        ),
    )
    print("\n=== summary (sorted by median_ms) ===")
    for row in rows_sorted:
        print(
            f"worker={row['worker']} family_worker={row['family_worker']} "
            f"median={row['median_ms']} mean={row['mean_ms']} min={row['min_ms']} max={row['max_ms']}"
        )
    best = rows_sorted[0] if rows_sorted else {}
    if best:
        print(
            f"\n[best] worker={best['worker']} family_worker={best['family_worker']} "
            f"median={best['median_ms']}ms"
        )

    if str(args.json_out).strip():
        payload = {
            "schema": "seamgrim.ci_gate.worker_benchmark.v1",
            "profile": str(args.profile),
            "only_step": str(args.only_step),
            "runs": runs,
            "workers": workers,
            "family_workers": family_workers,
            "results": rows_sorted,
            "best": best,
        }
        out_path = (root / str(args.json_out)).resolve() if not Path(str(args.json_out)).is_absolute() else Path(str(args.json_out))
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(f"[json] {out_path}")
    if str(args.env_out).strip() and best:
        env_lines = [
            f"DDN_SEAMGRIM_CI_GATE_MAX_WORKERS={int(best['worker'])}",
            f"DDN_SEAMGRIM_CI_GATE_FAMILY_MAX_WORKERS={int(best['family_worker'])}",
        ]
        env_path = (root / str(args.env_out)).resolve() if not Path(str(args.env_out)).is_absolute() else Path(str(args.env_out))
        env_path.parent.mkdir(parents=True, exist_ok=True)
        env_path.write_text("\n".join(env_lines) + "\n", encoding="utf-8")
        print(f"[env] {env_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
