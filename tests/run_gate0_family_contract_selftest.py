#!/usr/bin/env python
from __future__ import annotations

import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


SCRIPT_TAG = "gate0-family-contract-selftest"
PROGRESS_ENV_KEY = "DDN_GATE0_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON"
CHECKS = (
    ("gate0_runtime_family", "tests/run_gate0_runtime_family_selftest.py"),
    ("w92_aot", "tests/run_w92_aot_pack_check.py"),
    ("w93_universe", "tests/run_w93_universe_pack_check.py"),
    ("w94_social", "tests/run_w94_social_pack_check.py"),
    ("gate0_family", "tests/run_gate0_family_selftest.py"),
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
        "schema": "ddn.ci.gate0_family_contract_selftest.progress.v1",
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


def run_check(script: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, script],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


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
    for name, script in CHECKS:
        if name == "w92_aot":
            break
        write_progress_snapshot(
            progress_path,
            status="running",
            current_probe=name,
            last_completed_probe=last_completed_probe,
            completed_checks=len(passed),
            total_checks=total_checks,
            checks_text=CHECKS_TEXT,
        )
        proc = run_check(script)
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

    parallel_checks = (
        ("w92_aot", "tests/run_w92_aot_pack_check.py"),
        ("w93_universe", "tests/run_w93_universe_pack_check.py"),
        ("w94_social", "tests/run_w94_social_pack_check.py"),
    )
    write_progress_snapshot(
        progress_path,
        status="running",
        current_probe="w92_aot,w93_universe,w94_social",
        last_completed_probe=last_completed_probe,
        completed_checks=len(passed),
        total_checks=total_checks,
        checks_text=CHECKS_TEXT,
    )
    procs: dict[str, subprocess.Popen[str]] = {}
    for name, script in parallel_checks:
        procs[name] = subprocess.Popen(
            [sys.executable, script],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
    parallel_results: dict[str, subprocess.CompletedProcess[str]] = {}
    for name, _ in parallel_checks:
        proc = procs[name]
        stdout, stderr = proc.communicate()
        parallel_results[name] = subprocess.CompletedProcess(
            args=proc.args,
            returncode=int(proc.returncode or 0),
            stdout=stdout,
            stderr=stderr,
        )
    for name, _ in parallel_checks:
        proc = parallel_results[name]
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

    name = "gate0_family"
    script = "tests/run_gate0_family_selftest.py"
    write_progress_snapshot(
        progress_path,
        status="running",
        current_probe=name,
        last_completed_probe=last_completed_probe,
        completed_checks=len(passed),
        total_checks=total_checks,
        checks_text=CHECKS_TEXT,
    )
    proc = run_check(script)
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
