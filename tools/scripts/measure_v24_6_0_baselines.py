#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TEUL = ["cargo", "run", "-q", "--manifest-path", "tools/teul-cli/Cargo.toml", "--"]


def run_case_once(
    label: str,
    cmd: list[str],
    *,
    throughput_kind: str | None = None,
    count_hint: int | None = None,
) -> dict[str, object]:
    started = time.perf_counter()
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    elapsed_ms = int((time.perf_counter() - started) * 1000)
    result: dict[str, object] = {
        "label": label,
        "cmd": cmd,
        "exit_code": proc.returncode,
        "wall_clock_ms": elapsed_ms,
        "stdout_preview": (proc.stdout or "").strip().splitlines()[:8],
        "stderr_preview": (proc.stderr or "").strip().splitlines()[:8],
        "throughput_kind": throughput_kind,
        "count_hint": count_hint,
    }
    if proc.returncode == 0 and throughput_kind and count_hint and elapsed_ms > 0:
        rate = round((count_hint * 1000.0) / elapsed_ms, 3)
        if throughput_kind == "madi":
            result["madi_per_sec"] = rate
        elif throughput_kind == "step":
            result["step_per_sec"] = rate
    return result


def summarize_runs(runs: list[dict[str, object]]) -> dict[str, object]:
    wall_times = [int(run["wall_clock_ms"]) for run in runs if isinstance(run.get("wall_clock_ms"), int)]
    summary: dict[str, object] = {"run_count": len(runs)}
    if wall_times:
        ordered = sorted(wall_times)
        summary["best_wall_clock_ms"] = ordered[0]
        summary["median_wall_clock_ms"] = ordered[len(ordered) // 2]
        summary["worst_wall_clock_ms"] = ordered[-1]
        summary["spread_wall_clock_ms"] = ordered[-1] - ordered[0]
        summary["spread_ms"] = summary["spread_wall_clock_ms"]
        spread = summary["spread_wall_clock_ms"]
        if spread == 0:
            summary["spread_note"] = "stable"
        elif spread <= 150:
            summary["spread_note"] = "tight"
        elif spread <= 400:
            summary["spread_note"] = "moderate"
        else:
            summary["spread_note"] = "wide"
    metric_key = None
    for candidate in ("madi_per_sec", "step_per_sec"):
        if any(isinstance(run.get(candidate), (int, float)) for run in runs):
            metric_key = candidate
            break
    if metric_key:
        values = [float(run[metric_key]) for run in runs if isinstance(run.get(metric_key), (int, float))]
        ordered = sorted(values)
        summary[f"best_{metric_key}"] = round(ordered[-1], 3)
        summary[f"median_{metric_key}"] = round(ordered[len(ordered) // 2], 3)
        summary[f"worst_{metric_key}"] = round(ordered[0], 3)
        summary[f"spread_{metric_key}"] = round(ordered[-1] - ordered[0], 3)
    return summary


def run_case(
    label: str,
    cmd: list[str],
    *,
    repeat: int,
    throughput_kind: str | None = None,
    count_hint: int | None = None,
) -> dict[str, object]:
    runs = [
        run_case_once(label, cmd, throughput_kind=throughput_kind, count_hint=count_hint)
        for _ in range(repeat)
    ]
    return {
        "label": label,
        "cmd": cmd,
        "throughput_kind": throughput_kind,
        "count_hint": count_hint,
        "runs": runs,
        "summary": summarize_runs(runs),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Measure representative v24.6.0 immediate dev baselines")
    parser.add_argument("--json-out", default="", help="optional JSON output path")
    parser.add_argument("--repeat", type=int, default=1, help="repeat each case this many times")
    args = parser.parse_args()
    repeat = max(1, int(args.repeat))

    with tempfile.TemporaryDirectory(prefix="v24_6_0_benchmark_") as temp_dir:
        nurigym_out = str(Path(temp_dir) / "nurigym_out")
        cases = [
            run_case(
                "education_preview",
                TEUL + ["run", "pack/benchmark_baseline_v1/c01_education_preview.ddn"],
                repeat=repeat,
            ),
            run_case(
                "seamgrim_local_run",
                TEUL + ["run", "pack/benchmark_baseline_v1/c02_seamgrim_local_run.ddn"],
                repeat=repeat,
                throughput_kind="madi",
                count_hint=601,
            ),
            run_case(
                "nurigym_proxy_baseline",
                [
                    sys.executable,
                    "tools/scripts/benchmark_seamgrim_ci_gate_workers.py",
                    "--workers",
                    "10",
                    "--family-workers",
                    "8",
                    "--runs",
                    "1",
                    "--profile",
                    "core_lang",
                    "--only-step",
                    "nurigym_shared_sync_priority_tiebreak_pack_check",
                    "--json-out",
                    nurigym_out + ".json",
                ],
                repeat=repeat,
            ),
            run_case(
                "nurigym_short_run",
                TEUL + ["nurigym", "run", "pack/benchmark_baseline_v1/input.nurigym.json", "--out", nurigym_out],
                repeat=repeat,
                throughput_kind="step",
                count_hint=2,
            ),
        ]

        payload = {
            "schema": "ddn.v24_6_0.benchmark_baseline_report.v1",
            "repeat_count": repeat,
            "cases": cases,
        }
        if str(args.json_out).strip():
            out_path = Path(str(args.json_out))
            if not out_path.is_absolute():
                out_path = (ROOT / out_path).resolve()
            out_path.parent.mkdir(parents=True, exist_ok=True)
            out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            print(out_path)
        else:
            print(json.dumps(payload, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
