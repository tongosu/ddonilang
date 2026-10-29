#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "perf_baseline_v1"
RUNNER = ROOT / "tools" / "scripts" / "run_perf_baseline_v1.py"


def fail(message: str) -> int:
    print(f"perf_baseline_v1_pack_check: FAIL: {message}", file=sys.stderr)
    return 1


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "baseline.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        ROOT / "pack" / "benchmark_baseline_v1" / "contract.detjson",
    ]
    missing = [path.relative_to(ROOT).as_posix() for path in required if not path.exists()]
    if missing:
        return fail(f"missing files: {missing}")

    contract = load_json(PACK / "contract.detjson")
    if contract.get("schema") != "ddn.perf_baseline.pack.contract.v1":
        return fail(f"contract schema mismatch: {contract.get('schema')}")
    if contract.get("source_pack") != "benchmark_baseline_v1":
        return fail("source_pack must remain benchmark_baseline_v1")
    if contract.get("closure_claim") != "no" or contract.get("product_perf_claim") is not False:
        return fail("contract must not claim performance closure")
    axes = contract.get("axes")
    if axes != ["tick_throughput", "wasm_load_first_frame", "gym_rollout_step_per_sec"]:
        return fail(f"axis mismatch: {axes}")
    gate = contract.get("regression_gate", {})
    if gate.get("allowed_worse_ratio") != 0.2:
        return fail(f"gate ratio mismatch: {gate}")

    baseline = load_json(PACK / "baseline.detjson")
    if baseline.get("schema") != "ddn.perf_baseline.baseline.v1":
        return fail(f"baseline schema mismatch: {baseline.get('schema')}")
    if baseline.get("baseline_values_status") != "pending_first_measurement":
        return fail("baseline must stay pending until measured")
    cases = baseline.get("cases")
    if not isinstance(cases, list) or len(cases) != 3:
        return fail("baseline cases must be exactly 3")

    with tempfile.TemporaryDirectory(prefix="perf_baseline_v1_check_") as temp_dir:
        report_path = Path(temp_dir) / "report.detjson"
        proc = subprocess.run(
            [
                sys.executable,
                str(RUNNER),
                "--dry-run",
                "--json-out",
                str(report_path),
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=60,
        )
        if proc.returncode != 0:
            return fail(proc.stderr.strip() or proc.stdout.strip() or "runner dry-run failed")
        report = load_json(report_path)
        if report.get("schema") != "ddn.perf_baseline.report.v1":
            return fail(f"report schema mismatch: {report.get('schema')}")
        if report.get("dry_run") is not True:
            return fail("runner dry-run flag missing")
        if report.get("axes") != axes:
            return fail(f"runner axes mismatch: {report.get('axes')}")
        gate_report = report.get("regression_gate", {})
        if gate_report.get("schema") != "ddn.perf_baseline.regression_gate_result.v1":
            return fail("regression gate report missing")
        if gate_report.get("ok") is not True:
            return fail(f"dry-run regression gate should be ok: {gate_report}")

    print("perf_baseline_v1_pack_check: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
