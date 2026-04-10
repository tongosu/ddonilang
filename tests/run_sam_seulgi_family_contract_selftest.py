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


SCRIPT_TAG = "sam-seulgi-family-contract-selftest"
SELF_SCRIPT_PATH = "tests/run_sam_seulgi_family_contract_selftest.py"
PROGRESS_ENV_KEY = "DDN_SAM_SEULGI_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON"
ROOT = Path(__file__).resolve().parents[1]
CHECKS = (
    ("external_intent_boundary_pack", "tests/run_external_intent_boundary_pack_check.py"),
    ("seulgi_v1_pack", "tests/run_seulgi_v1_pack_check.py"),
    ("sam_inputsnapshot_contract_pack", "tests/run_sam_inputsnapshot_contract_pack_check.py"),
    ("sam_ai_ordering_pack", "tests/run_sam_ai_ordering_pack_check.py"),
    ("seulgi_gatekeeper_pack", "tests/run_seulgi_gatekeeper_pack_check.py"),
    ("external_intent_seulgi_walk_alignment", "tests/run_external_intent_seulgi_walk_alignment_check.py"),
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
        "schema": "ddn.ci.sam_seulgi_family_contract_selftest.progress.v1",
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
    out.write_text(
        json.dumps(payload, ensure_ascii=False, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


def decode_stream(raw: bytes | str | None) -> str:
    if isinstance(raw, bytes):
        return raw.decode("utf-8", errors="replace")
    if raw is None:
        return ""
    return str(raw)


def fail(name: str, proc: subprocess.CompletedProcess[bytes]) -> int:
    stderr_text = decode_stream(proc.stderr).strip()
    stdout_text = decode_stream(proc.stdout).strip()
    detail = stderr_text or stdout_text or "-"
    print(f"[{SCRIPT_TAG}] fail: check={name} rc={proc.returncode} detail={detail}")
    return 1


def run_check(script: str) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        [sys.executable, "-S", script],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def main() -> int:
    progress_path = os.environ.get(PROGRESS_ENV_KEY, "")
    total_checks = len(CHECKS)
    last_completed_probe = "-"
    script_futures: dict[str, object] = {}
    uncached_scripts: list[str] = []
    script_cached: dict[str, bool] = {}
    for _, script in CHECKS:
        cached = is_script_cached(script)
        script_cached[script] = cached
        if cached:
            continue
        if script not in script_futures:
            script_futures[script] = None
            uncached_scripts.append(script)
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
    cpu_workers = os.cpu_count() or 4
    max_workers = max(1, min(8, cpu_workers, len(uncached_scripts)))
    with ThreadPoolExecutor(max_workers=max_workers) as executor:
        for script in uncached_scripts:
            script_futures[script] = executor.submit(run_check, script)
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
            if script_cached.get(script, False):
                print(f"[{SCRIPT_TAG}] cache-hit check={name} script={script}")
            else:
                future = script_futures.get(script)
                if future is None:
                    continue
                proc = future.result()
                if proc.returncode != 0:
                    retry = run_check(script)
                    if retry.returncode == 0:
                        proc = retry
                    else:
                        proc = retry
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
                script_cached[script] = True
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
