#!/usr/bin/env python
from __future__ import annotations

import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


SCRIPT_TAG = "seamgrim-consumer-surface-family-contract-selftest"
PROGRESS_ENV_KEY = "DDN_SEAMGRIM_CONSUMER_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON"
CHECKS = (
    ("workflow_contract", [sys.executable, "tests/run_seamgrim_workflow_contract_check.py"]),
    ("schema_gate", [sys.executable, "tests/run_seamgrim_lesson_schema_gate.py"]),
    ("visual_contract", [sys.executable, "tests/run_seamgrim_visual_contract_check.py"]),
    ("browse_selection_flow", [sys.executable, "tests/run_seamgrim_browse_selection_flow_check.py"]),
    (
        "state_view_boundary_transport",
        [sys.executable, "tests/run_seamgrim_state_view_boundary_family_transport_contract_selftest.py"],
    ),
    ("seamgrim_consumer_surface_family", [sys.executable, "tests/run_seamgrim_consumer_surface_family_selftest.py"]),
)
CHECKS_TEXT = ",".join(name for name, _ in CHECKS)
ASSUME_CONTRACT_PREREQS_PASSED_ENV = "DDN_ASSUME_CONTRACT_PREREQS_PASSED"
CONTRACT_PREREQ_COVERED_CHECKS = {
    "workflow_contract",
    "schema_gate",
    "visual_contract",
    "browse_selection_flow",
    "state_view_boundary_transport",
    "seamgrim_consumer_surface_family",
}


def write_progress_snapshot(
    path_text: str,
    *,
    status: str,
    current_probe: str,
    last_completed_probe: str,
    completed_checks: int,
    total_checks: int,
    checks_text: str,
) -> None:
    if not str(path_text).strip():
        return
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "ddn.ci.seamgrim_consumer_surface_family_contract_selftest.progress.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": status,
        "current_case": "-",
        "last_completed_case": "-",
        "current_probe": current_probe,
        "last_completed_probe": last_completed_probe,
        "completed_checks": int(completed_checks),
        "total_checks": int(total_checks),
        "checks_text": str(checks_text).strip() or "-",
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def fail(name: str, proc: subprocess.CompletedProcess[str]) -> int:
    stdout = (proc.stdout or "").strip()
    stderr = (proc.stderr or "").strip()
    detail = stderr or stdout or "-"
    print(f"[{SCRIPT_TAG}] fail: check={name} rc={proc.returncode} detail={detail}")
    return 1


def main() -> int:
    progress_path = os.environ.get(PROGRESS_ENV_KEY, "")
    total_checks = len(CHECKS)
    last_completed_probe = "-"
    root = Path(__file__).resolve().parent.parent
    write_progress_snapshot(
        progress_path,
        status="running",
        current_probe=CHECKS[0][0] if CHECKS else "-",
        last_completed_probe="-",
        completed_checks=0,
        total_checks=total_checks,
        checks_text=CHECKS_TEXT,
    )
    passed: list[str] = []
    assume_prereqs_passed = os.environ.get(ASSUME_CONTRACT_PREREQS_PASSED_ENV, "").strip() == "1"
    for name, cmd in CHECKS:
        write_progress_snapshot(
            progress_path,
            status="running",
            current_probe=name,
            last_completed_probe=last_completed_probe,
            completed_checks=len(passed),
            total_checks=total_checks,
            checks_text=CHECKS_TEXT,
        )
        if assume_prereqs_passed and name in CONTRACT_PREREQ_COVERED_CHECKS:
            passed.append(name)
            last_completed_probe = name
            continue
        proc = subprocess.run(
            cmd,
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if proc.returncode != 0:
            write_progress_snapshot(
                progress_path,
                status="failed",
                current_probe=name,
                last_completed_probe=last_completed_probe,
                completed_checks=len(passed),
                total_checks=total_checks,
                checks_text=CHECKS_TEXT,
            )
            return fail(name, proc)
        passed.append(name)
        last_completed_probe = name
    write_progress_snapshot(
        progress_path,
        status="completed",
        current_probe="-",
        last_completed_probe=last_completed_probe,
        completed_checks=len(passed),
        total_checks=total_checks,
        checks_text=CHECKS_TEXT,
    )
    print(f"[{SCRIPT_TAG}] ok checks={len(passed)} names={','.join(passed)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
