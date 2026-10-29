#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_runner_lifecycle_reliability_v1"
MATRIX_EVIDENCE = ROOT / "out" / "seamgrim_runner_lifecycle_reliability_v1" / "g4_matrix_latest.detjson"


EXPECTED_COUNTS = {
    "lineage_consecutive": 20,
    "public_wedge_consecutive": 5,
    "durable_workbench_consecutive": 5,
    "private_pilot_local_rehearsal_consecutive": 3,
    "four_viewport_workflow_consecutive": 2,
    "parallel_lineage_4x_consecutive": 3,
    "bounded_contention_consecutive": 3,
}


def fail(code: str, message: str) -> int:
    safe = str(message).encode("ascii", errors="backslashreplace").decode("ascii")
    print(f"{code}: {safe}")
    return 1


def run(cmd: list[str], *, timeout: int = 600) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def validate_matrix_evidence() -> str | None:
    if not MATRIX_EVIDENCE.exists():
        return f"missing matrix evidence: {MATRIX_EVIDENCE.relative_to(ROOT)}"
    payload = json.loads(MATRIX_EVIDENCE.read_text(encoding="utf-8"))
    if payload.get("status") != "PASS":
        return f"matrix status={payload.get('status')!r}"
    if payload.get("rerun_masked_failures") != 0:
        return f"rerun_masked_failures={payload.get('rerun_masked_failures')!r}"
    if payload.get("owned_residual_processes") != 0:
        return f"owned_residual_processes={payload.get('owned_residual_processes')!r}"
    if payload.get("owned_residual_sockets") != 0:
        return f"owned_residual_sockets={payload.get('owned_residual_sockets')!r}"
    if payload.get("owned_residual_temp_dirs") != 0:
        return f"owned_residual_temp_dirs={payload.get('owned_residual_temp_dirs')!r}"
    if payload.get("double_slash_requests") != 0:
        return f"double_slash_requests={payload.get('double_slash_requests')!r}"
    sequences = payload.get("sequences", {})
    for key, expected in EXPECTED_COUNTS.items():
        actual = len(sequences.get(key) or [])
        if actual != expected:
            return f"{key} count {actual} != {expected}"
    return None


def main() -> int:
    required = [PACK / "contract.detjson", PACK / "g4_matrix_contract.detjson"]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_RELIABILITY_MISSING", ",".join(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("current_gate") != "G4" or contract.get("reliability_closed_claim") is not True:
        return fail("E_SEAMGRIM_RELIABILITY_CONTRACT", repr({k: contract.get(k) for k in ["current_gate", "reliability_closed_claim"]}))
    matrix_problem = validate_matrix_evidence()
    if matrix_problem:
        return fail("E_SEAMGRIM_RELIABILITY_MATRIX_EVIDENCE", matrix_problem)

    commands = [
        [sys.executable, "tests/run_seamgrim_project_run_prerequisite_check.py"],
        [sys.executable, "tests/run_seamgrim_runner_lifecycle_process_check.py"],
        [sys.executable, "tests/run_pack_golden.py", "seamgrim_runner_lifecycle_reliability_v1"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=900)
        if proc.returncode:
            return fail("E_SEAMGRIM_RELIABILITY_CMD", f"{cmd!r}: {proc.stdout[-2500:]}")
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_SEAMGRIM_RELIABILITY_SSOT", ssot.stdout.strip())
    print("[seamgrim-runner-lifecycle-reliability] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
