#!/usr/bin/env python
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_wave2c_local_success_v1"
FIVE_MIN_MS = 300_000

try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except Exception:
    pass


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8-sig")


def run(cmd: list[str], *, timeout: int = 420) -> subprocess.CompletedProcess[str]:
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


def check_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "seamgrim_deployment_profile_v0_runner.mjs",
        ROOT / "tests" / "seamgrim_wave2b_representative_worlds_runner.mjs",
        ROOT / "tests" / "seamgrim_responsive_lesson_workflow_runner.mjs",
        ROOT / "tests" / "seamgrim_first_delight_projectile_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_W2C_LOCAL_SUCCESS_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("pack") != "seamgrim_wave2c_local_success_v1":
        return fail("E_SEAMGRIM_W2C_LOCAL_SUCCESS_PACK", repr(contract))
    if contract.get("gates") != ["G5", "G6", "G7"]:
        return fail("E_SEAMGRIM_W2C_LOCAL_SUCCESS_GATES", repr(contract.get("gates")))
    for key in ("docs_ssot_change", "new_ddn_syntax", "state_hash_formula_change", "server_login_cloud_claim"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_W2C_LOCAL_SUCCESS_BOUNDARY", f"{key}={contract.get(key)!r}")
    required_checks = {
        "deployment_package_profile_roundtrip",
        "representative_world_verticals",
        "five_minute_local_success_timing",
    }
    if not required_checks.issubset(set(contract.get("checks") or [])):
        return fail("E_SEAMGRIM_W2C_LOCAL_SUCCESS_CHECKS", repr(contract.get("checks")))
    return 0


def run_gate(cmd: list[str], code: str, timeout: int = 420) -> tuple[int, str]:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}\n{(proc.stdout or '').strip()}"), proc.stdout
    return 0, proc.stdout


def check_five_minute_timing() -> int:
    rc, stdout = run_gate(["node", "tests/seamgrim_responsive_lesson_workflow_runner.mjs"], "E_SEAMGRIM_W2C_LOCAL_SUCCESS_TIMING_BROWSER", 520)
    if rc:
        return rc
    match = re.search(r"seamgrim_responsive_lesson_workflow:\s+ok\s+(\{.*\})", stdout.strip(), re.S)
    if not match:
        return fail("E_SEAMGRIM_W2C_LOCAL_SUCCESS_TIMING_PARSE", stdout.strip())
    payload = json.loads(match.group(1))
    reports = payload.get("reports")
    if not isinstance(reports, list) or len(reports) != 4:
        return fail("E_SEAMGRIM_W2C_LOCAL_SUCCESS_TIMING_REPORTS", repr(payload))
    for row in reports:
        viewport = str(row.get("viewport") or "")
        for key in ("first_open_ms", "first_run_ms", "first_control_change_ms", "first_compare_semantic_ms"):
            value = int(row.get(key) or 0)
            if value <= 0 or value > FIVE_MIN_MS:
                return fail("E_SEAMGRIM_W2C_LOCAL_SUCCESS_TIMING_LIMIT", f"{viewport} {key}={value}")
    print("[seamgrim-wave2c-local-success-v1] timing", json.dumps(reports, ensure_ascii=False))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_W2C_LOCAL_SUCCESS_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_W2C_LOCAL_SUCCESS_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    rc = check_contract()
    if rc:
        return rc
    gates = [
        (["python", "tests/run_seamgrim_deployment_profile_v0_check.py"], "E_SEAMGRIM_W2C_LOCAL_SUCCESS_DEPLOYMENT", 360),
        (["python", "tests/run_seamgrim_wave2b_representative_worlds_check.py"], "E_SEAMGRIM_W2C_LOCAL_SUCCESS_REP_WORLDS", 520),
        (["python", "tests/run_seamgrim_first_delight_projectile_check.py"], "E_SEAMGRIM_W2C_LOCAL_SUCCESS_FIRST_DELIGHT", 360),
        (
            ["python", "tests/run_pack_golden.py", "seamgrim_wave2c_local_success_v1", "seamgrim_deployment_profile_v0_v1", "seamgrim_wave2b_representative_worlds_v1", "seamgrim_responsive_lesson_workflow_v1"],
            "E_SEAMGRIM_W2C_LOCAL_SUCCESS_PACK_GOLDEN",
            360,
        ),
    ]
    for cmd, code, timeout in gates:
        rc, _stdout = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_five_minute_timing()
    if rc:
        return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-wave2c-local-success-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
