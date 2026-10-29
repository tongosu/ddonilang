#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "benchmark_baseline_v1"
SCRIPT = ROOT / "tools" / "scripts" / "benchmark_seamgrim_ci_gate_workers.py"
HELPER = ROOT / "tools" / "scripts" / "measure_v24_6_0_baselines.py"


def fail(detail: str) -> int:
    print(f"check=benchmark_baseline_pack detail={detail}")
    return 1


def main() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "golden.jsonl",
        PACK / "c01_education_preview.ddn",
        PACK / "c02_seamgrim_local_run.ddn",
        PACK / "input.nurigym.json",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("missing:" + ",".join(missing))
    if not SCRIPT.exists():
        return fail(f"missing_script:{SCRIPT.relative_to(ROOT).as_posix()}")
    if not HELPER.exists():
        return fail(f"missing_helper:{HELPER.relative_to(ROOT).as_posix()}")

    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.benchmark_baseline.pack.contract.v1":
        return fail(f"schema:{contract.get('schema')}")
    if contract.get("evidence_tier") != "runner_backed":
        return fail("tier")
    axes = contract.get("axes", [])
    if axes != ["education_preview", "seamgrim_local_run", "nurigym_million_step_baseline"]:
        return fail(f"axes:{axes}")

    with tempfile.TemporaryDirectory(prefix="benchmark_baseline_v1_") as temp_dir:
        report_path = Path(temp_dir) / "benchmark.detjson"
        proc = subprocess.run(
            [
                sys.executable,
                str(SCRIPT),
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
                str(report_path),
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=900,
        )
        if proc.returncode != 0:
            detail = proc.stderr.strip() or proc.stdout.strip() or f"returncode={proc.returncode}"
            return fail(f"benchmark_failed:{detail}")
        if not report_path.exists():
            return fail("report_missing")
        report = json.loads(report_path.read_text(encoding="utf-8"))
        if report.get("schema") != "seamgrim.ci_gate.worker_benchmark.v1":
            return fail(f"report_schema:{report.get('schema')}")
        if report.get("profile") != "core_lang" or report.get("only_step") != "nurigym_shared_sync_priority_tiebreak_pack_check":
            return fail("report_target")
        if not isinstance(report.get("results"), list) or not report["results"]:
            return fail("report_results_missing")

        helper_report_path = Path(temp_dir) / "v24_6_0_baseline.detjson"
        helper_proc = subprocess.run(
            [
                sys.executable,
                str(HELPER),
                "--repeat",
                "2",
                "--json-out",
                str(helper_report_path),
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=900,
        )
        if helper_proc.returncode != 0:
            detail = helper_proc.stderr.strip() or helper_proc.stdout.strip() or f"returncode={helper_proc.returncode}"
            return fail(f"helper_failed:{detail}")
        if not helper_report_path.exists():
            return fail("helper_report_missing")
        helper_report = json.loads(helper_report_path.read_text(encoding="utf-8"))
        if helper_report.get("schema") != "ddn.v24_6_0.benchmark_baseline_report.v1":
            return fail(f"helper_schema:{helper_report.get('schema')}")
        if helper_report.get("repeat_count") != 2:
            return fail(f"helper_repeat_count:{helper_report.get('repeat_count')}")
        cases = helper_report.get("cases")
        if not isinstance(cases, list) or not cases:
            return fail("helper_cases_missing")
        required_labels = {
            "education_preview",
            "seamgrim_local_run",
            "nurigym_proxy_baseline",
            "nurigym_short_run",
        }
        case_map = {str(case.get("label", "")).strip(): case for case in cases}
        if set(case_map) != required_labels:
            return fail(f"helper_labels:{sorted(case_map)}")
        for label in sorted(required_labels):
            case = case_map[label]
            runs = case.get("runs")
            summary = case.get("summary")
            if not isinstance(runs, list) or len(runs) != 2:
                return fail(f"helper_runs:{label}")
            if not isinstance(summary, dict):
                return fail(f"helper_summary:{label}")
            if summary.get("run_count") != 2:
                return fail(f"helper_summary_run_count:{label}:{summary.get('run_count')}")
            for field in (
                "best_wall_clock_ms",
                "median_wall_clock_ms",
                "worst_wall_clock_ms",
                "spread_wall_clock_ms",
            ):
                if not isinstance(summary.get(field), int):
                    return fail(f"helper_summary_field:{label}:{field}")

    print("benchmark baseline pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
