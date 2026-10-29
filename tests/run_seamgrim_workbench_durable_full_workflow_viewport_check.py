#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_workbench_durable_full_workflow_viewport_v1"
RUNNER = ROOT / "tests" / "seamgrim_workbench_durable_full_workflow_viewport_runner.mjs"


def fail(code: str, message: str) -> int:
    safe = str(message).encode("ascii", errors="backslashreplace").decode("ascii")
    print(f"{code}: {safe}")
    return 1


def run(cmd: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
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


def main() -> int:
    missing = [str(path.relative_to(ROOT)) for path in [PACK / "README.md", PACK / "contract.detjson", PACK / "golden.jsonl", PACK / "input.ddn", RUNNER] if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_WORKBENCH_G9_MISSING", json.dumps(missing, ensure_ascii=False))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    for key, expected in {
        "pack": "seamgrim_workbench_durable_full_workflow_viewport_v1",
        "gate": "G9",
        "behavior_closed_claim": True,
        "actual_browser_required": True,
        "canvas_nonblank_required": True,
        "horizontal_overflow_allowed": False,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
    }.items():
        if contract.get(key) != expected:
            return fail("E_SEAMGRIM_WORKBENCH_G9_CONTRACT", f"{key}={contract.get(key)!r}")
    proc = run(["node", str(RUNNER.relative_to(ROOT))], timeout=360)
    if proc.returncode:
        return fail("E_SEAMGRIM_WORKBENCH_G9_BROWSER", proc.stdout.strip())
    if "workbench_durable_full_workflow_viewport.runner_result.v1" not in proc.stdout:
        return fail("E_SEAMGRIM_WORKBENCH_G9_BROWSER_OK", proc.stdout.strip())
    try:
        browser_result = json.loads(proc.stdout.strip().splitlines()[-1])
    except Exception as exc:
        return fail("E_SEAMGRIM_WORKBENCH_G9_BROWSER_JSON", f"{exc}: {proc.stdout.strip()}")
    if browser_result.get("viewports") != ["1440x900", "1024x768", "778x912", "390x844"]:
        return fail("E_SEAMGRIM_WORKBENCH_G9_VIEWPORTS", json.dumps(browser_result.get("viewports"), ensure_ascii=False))
    representative = {
        str(row.get("project_id"))
        for row in browser_result.get("representative_projects", [])
        if isinstance(row, dict)
    }
    if representative != {"rep_phys_pendulum_wave1_v1", "rep_grid_game_state_drop_v1"}:
        return fail("E_SEAMGRIM_WORKBENCH_G9_REPRESENTATIVE_PROJECTS", json.dumps(sorted(representative), ensure_ascii=False))
    for cmd in [
        ["python", "tests/run_pack_golden.py", "seamgrim_workbench_durable_full_workflow_viewport_v1"],
        ["python", "tests/run_seamgrim_workbench_revision_conflict_restore_check.py"],
        ["python", "tests/run_seamgrim_workbench_storage_security_recovery_check.py"],
    ]:
        check = run(cmd, timeout=360)
        if check.returncode:
            return fail("E_SEAMGRIM_WORKBENCH_G9_DEP", f"{cmd!r}: {check.stdout.strip()}")
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_SEAMGRIM_WORKBENCH_G9_SSOT", ssot.stdout.strip())
    print("[seamgrim-workbench-durable-full-workflow-viewport-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
