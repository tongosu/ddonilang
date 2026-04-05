#!/usr/bin/env python
from __future__ import annotations

import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

from _selftest_exec_cache import is_script_cached, mark_script_ok


SCRIPT_TAG = "seamgrim-view-hash-family-transport-contract-selftest"
SELF_SCRIPT_PATH = "tests/run_seamgrim_view_hash_family_transport_contract_selftest.py"
PROGRESS_ENV_KEY = "DDN_SEAMGRIM_VIEW_HASH_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON"
CHECKS = (
    ("family_contract", [sys.executable, "tests/run_seamgrim_view_hash_family_contract_selftest.py"]),
    (
        "patent_b_view_hash_isolation",
        [sys.executable, "tests/run_patent_b_state_view_hash_isolation_check.py"],
    ),
    ("moyang_view_boundary", [sys.executable, "tests/run_seamgrim_moyang_view_boundary_pack_check.py"]),
    ("dotbogi_view_meta_hash", [sys.executable, "tests/run_dotbogi_view_meta_hash_pack_check.py"]),
    ("state_view_hash_separation_family", [sys.executable, "tests/run_state_view_hash_separation_family_selftest.py"]),
)
CHECKS_TEXT = ",".join(name for name, _ in CHECKS)
FAMILY_CONTRACT_NAME = "family_contract"
FAMILY_CONTRACT_COVERED_CHECKS = {
    "patent_b_view_hash_isolation",
    "moyang_view_boundary",
    "dotbogi_view_meta_hash",
    "state_view_hash_separation_family",
}
ASSUME_FAMILY_CONTRACT_PASSED_ENV = "DDN_ASSUME_FAMILY_CONTRACT_PASSED"


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
        "schema": "ddn.ci.seamgrim_view_hash_family_transport_contract_selftest.progress.v1",
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
    root = Path(__file__).resolve().parent.parent
    family_contract_passed = os.environ.get(ASSUME_FAMILY_CONTRACT_PASSED_ENV, "").strip() == "1"
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
        if family_contract_passed and (name == FAMILY_CONTRACT_NAME or name in FAMILY_CONTRACT_COVERED_CHECKS):
            passed.append(name)
            last_completed_probe = name
            continue
        cache_key = " ".join(str(part).strip() for part in cmd[1:] if str(part).strip())
        if cache_key and is_script_cached(cache_key):
            print(f"[{SCRIPT_TAG}] cache-hit check={name} script={cache_key}")
        else:
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
            if cache_key:
                mark_script_ok(cache_key)
        passed.append(name)
        if name == FAMILY_CONTRACT_NAME:
            family_contract_passed = True
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
    mark_script_ok(SELF_SCRIPT_PATH)
    print(f"[{SCRIPT_TAG}] ok checks={len(passed)} names={','.join(passed)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
