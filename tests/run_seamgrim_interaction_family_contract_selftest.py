#!/usr/bin/env python
from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

from _selftest_exec_cache import is_script_cached, mark_script_ok


SCRIPT_TAG = "seamgrim-interaction-family-contract-selftest"
SELF_SCRIPT_PATH = "tests/run_seamgrim_interaction_family_contract_selftest.py"
PROGRESS_ENV_KEY = "DDN_SEAMGRIM_INTERACTION_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON"
CHECKS = (
    ("consumer_surface_transport", "tests/run_seamgrim_consumer_surface_family_transport_contract_selftest.py"),
    ("block_editor_smoke", "tests/run_seamgrim_block_editor_smoke_check.py"),
    ("playground_smoke", "tests/run_seamgrim_playground_smoke_check.py"),
    ("seamgrim_interaction_family", "tests/run_seamgrim_interaction_family_selftest.py"),
)
CHECKS_TEXT = ",".join(name for name, _ in CHECKS)


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
        "schema": "ddn.ci.seamgrim_interaction_family_contract_selftest.progress.v1",
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
    assume_interaction_smoke = str(os.environ.get("DDN_ASSUME_INTERACTION_SMOKE_PASSED", "")).strip() == "1"
    assume_consumer_transport = (
        str(os.environ.get("DDN_ASSUME_CONSUMER_SURFACE_TRANSPORT_PASSED", "")).strip() == "1"
    )
    assume_targets = {"block_editor_smoke", "playground_smoke"}
    def should_assume(name: str) -> bool:
        if assume_consumer_transport and name == "consumer_surface_transport":
            return True
        if assume_interaction_smoke and name in assume_targets:
            return True
        return False
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
    cached: dict[str, bool] = {}
    for name, script in CHECKS:
        cached[name] = is_script_cached(script)
    pending_checks = [
        (name, script)
        for name, script in CHECKS
        if (not cached.get(name, False)) and (not should_assume(name))
    ]

    pending_results: dict[str, subprocess.CompletedProcess[str]] = {}
    if pending_checks:
        write_progress_snapshot(
            progress_path,
            status="running",
            current_probe="parallel_exec",
            last_completed_probe=last_completed_probe,
            completed_checks=len(passed),
            total_checks=total_checks,
            checks_text=CHECKS_TEXT,
        )
        workers = max(1, min(len(pending_checks), os.cpu_count() or 4))
        with ThreadPoolExecutor(max_workers=workers) as pool:
            futures = {
                name: pool.submit(
                    subprocess.run,
                    [sys.executable, script],
                    capture_output=True,
                    text=True,
                    encoding="utf-8",
                    errors="replace",
                )
                for name, script in pending_checks
            }
            for name, _script in pending_checks:
                pending_results[name] = futures[name].result()

    for name, script in CHECKS:
        write_progress_snapshot(
            progress_path,
            status="running",
            current_probe=name,
            last_completed_probe=last_completed_probe,
            completed_checks=len(passed),
            total_checks=total_checks,
            checks_text=CHECKS_TEXT,
        )
        if should_assume(name) and name in assume_targets:
            print(f"[{SCRIPT_TAG}] assume-pass check={name} reason=DDN_ASSUME_INTERACTION_SMOKE_PASSED")
            mark_script_ok(script)
            passed.append(name)
            last_completed_probe = name
            continue
        if should_assume(name) and name == "consumer_surface_transport":
            print(f"[{SCRIPT_TAG}] assume-pass check={name} reason=DDN_ASSUME_CONSUMER_SURFACE_TRANSPORT_PASSED")
            mark_script_ok(script)
            passed.append(name)
            last_completed_probe = name
            continue
        if cached.get(name, False):
            print(f"[{SCRIPT_TAG}] cache-hit check={name} script={script}")
        else:
            proc = pending_results.get(name)
            if proc is None:
                proc = subprocess.CompletedProcess(
                    args=[sys.executable, script],
                    returncode=1,
                    stdout="",
                    stderr="missing_parallel_result",
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
            mark_script_ok(script)
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
    mark_script_ok(SELF_SCRIPT_PATH)
    print(f"[{SCRIPT_TAG}] ok checks={len(passed)} names={','.join(passed)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
