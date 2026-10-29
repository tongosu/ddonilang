#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_workbench_capability_share_clone_v1"
RUNNER = ROOT / "tests" / "seamgrim_workbench_capability_share_clone_runner.mjs"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
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
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "golden.jsonl",
        PACK / "input.ddn",
        RUNNER,
        ROOT / "solutions/seamgrim_ui_mvp/ui/workbench/workbench.js",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_WORKBENCH_G6_MISSING", json.dumps(missing, ensure_ascii=False))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    for key, expected in {
        "pack": "seamgrim_workbench_capability_share_clone_v1",
        "gate": "G6",
        "product_behavior_change": True,
        "behavior_closed_claim": True,
        "edit_view_capability_split": True,
        "view_write_denied": True,
        "view_local_wasm_run_allowed": True,
        "view_local_run_server_append_absent": True,
        "explicit_control_readiness": True,
        "runner_owned_residual_zero": True,
        "clone_issues_new_edit_capability": True,
        "public_listing_claim": False,
        "account_claim": False,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
    }.items():
        if contract.get(key) != expected:
            return fail("E_SEAMGRIM_WORKBENCH_G6_CONTRACT", f"{key}={contract.get(key)!r}")
    js = (ROOT / "solutions/seamgrim_ui_mvp/ui/workbench/workbench.js").read_text(encoding="utf-8")
    for token in [
        "cloneCurrentServerProject",
        "E_PROJECT_VIEW_WRITE_REJECTED",
        "project-clone",
        "cloneServerProject",
        "capability_redacted",
        "workbench_api_ready",
        "completed_generation",
        "view_write_forbidden",
    ]:
        if token not in js:
            return fail("E_SEAMGRIM_WORKBENCH_G6_JS_TOKEN", token)
    pack_proc = run(["python", "tests/run_pack_golden.py", "seamgrim_workbench_capability_share_clone_v1"], timeout=320)
    if pack_proc.returncode:
        return fail("E_SEAMGRIM_WORKBENCH_G6_CMD", pack_proc.stdout.strip())
    proc = run(["node", str(RUNNER.relative_to(ROOT))], timeout=320)
    if proc.returncode:
        return fail("E_SEAMGRIM_WORKBENCH_G6_CMD", proc.stdout.strip())
    try:
        payload = next(json.loads(line) for line in proc.stdout.splitlines() if line.startswith("{"))
    except (StopIteration, json.JSONDecodeError) as err:
        return fail("E_SEAMGRIM_WORKBENCH_G6_RESULT", repr(err))
    if payload.get("schema") != "seamgrim.workbench_capability_share_clone.runner_result.v2":
        return fail("E_SEAMGRIM_WORKBENCH_G6_RESULT_SCHEMA", repr(payload.get("schema")))
    if payload.get("view_controls", {}).get("lifecycle_run", {}).get("enabled") is not True:
        return fail("E_SEAMGRIM_WORKBENCH_G6_VIEW_RUN", repr(payload.get("view_controls")))
    if payload.get("view_run_server_append_absent") is not True:
        return fail("E_SEAMGRIM_WORKBENCH_G6_VIEW_ARCHIVE", repr(payload))
    if payload.get("owned_residual") != {"processes": 0, "sockets": 0, "temp_dirs": 0}:
        return fail("E_SEAMGRIM_WORKBENCH_G6_RESIDUAL", repr(payload.get("owned_residual")))
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_SEAMGRIM_WORKBENCH_G6_SSOT", ssot.stdout.strip())
    print("[seamgrim-workbench-capability-share-clone-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
