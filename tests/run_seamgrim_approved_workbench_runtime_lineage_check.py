#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_approved_workbench_runtime_lineage_v1"
RUNNER = ROOT / "tests" / "seamgrim_approved_workbench_runtime_lineage_runner.mjs"


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


def require_files() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "index.html",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "runtime" / "runtime_bridge.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "runtime" / "wasm_vm_runtime.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm" / "ddonirang_tool.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "wasm" / "ddonirang_tool_bg.wasm",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_RUNTIME_MISSING", str(missing))
    return 0


def require_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_approved_workbench_runtime_lineage_v1",
        "product_behavior_change": True,
        "behavior_closed_claim": True,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
        "mock_runtime_as_wasm": False,
        "cloud_or_public_registry_claim": False,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_RUNTIME_CONTRACT", f"{key}={payload.get(key)!r}")
    checks = payload.get("checks")
    if not isinstance(checks, list):
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_RUNTIME_CHECKS", repr(checks))
    for required in [
        "run_pause_resume_stop_step_forward_current_madi_hash",
        "setup_control_creates_new_run_recipe_only",
        "action_control_creates_next_inputsnapshot_only",
        "view_role_switch_keeps_state_hash",
        "history_back_read_only",
        "branch_creates_child_run",
        "overwrite_same_run_id_parent_hash_mutation_rejected",
    ]:
        if required not in checks:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_RUNTIME_CHECK_MISSING", required)
    return 0


def require_product_tokens() -> int:
    html = (ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "index.html").read_text(encoding="utf-8")
    js = (ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js").read_text(encoding="utf-8")
    for token in [
        "lifecycle-run",
        "lifecycle-resume",
        "history-back",
        "history-branch",
        "setup-controls",
        "action-controls",
        "view-controls",
    ]:
        if token not in html:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_RUNTIME_HTML_TOKEN", token)
    for token in [
        "createRuntimeBridgeV1",
        "seamgrim.run_record.v1",
        "seamgrim.input_snapshot.ui.v1",
        "E_RUN_HISTORY_READ_ONLY",
        "E_RUN_IMMUTABLE_OVERWRITE_REJECTED",
        "E_PARENT_HASH_MUTATION_REJECTED",
    ]:
        if token not in js:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_RUNTIME_JS_TOKEN", token)
    return 0


def run_browser() -> int:
    proc = run(["node", str(RUNNER.relative_to(ROOT))], timeout=180)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_RUNTIME_BROWSER", proc.stdout.strip())
    if "seamgrim_approved_workbench_runtime_lineage: ok" not in proc.stdout:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_RUNTIME_BROWSER_OK", proc.stdout.strip())
    return 0


def run_pack_golden() -> int:
    proc = run(["python", "tests/run_pack_golden.py", "seamgrim_approved_workbench_runtime_lineage_v1"], timeout=120)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_RUNTIME_PACK", proc.stdout.strip())
    return 0


def require_docs_ssot_clean() -> int:
    proc = run(["git", "-c", "core.excludesfile=", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_RUNTIME_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_RUNTIME_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (
        require_files,
        require_contract,
        require_product_tokens,
        run_browser,
        run_pack_golden,
        require_docs_ssot_clean,
    ):
        rc = check()
        if rc:
            return rc
    print("[seamgrim-approved-workbench-runtime-lineage-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
