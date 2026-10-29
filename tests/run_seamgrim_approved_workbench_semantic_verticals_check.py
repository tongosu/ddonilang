#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_approved_workbench_semantic_verticals_v1"
RUNNER = ROOT / "tests" / "seamgrim_approved_workbench_semantic_verticals_runner.mjs"


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
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_pendulum_wave1_v1" / "lesson.ddn",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_grid_game_state_drop_v1" / "lesson.ddn",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_VERTICALS_MISSING", str(missing))
    return 0


def require_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_approved_workbench_semantic_verticals_v1",
        "product_behavior_change": True,
        "behavior_closed_claim": True,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
        "public_registry_claim": False,
        "korean_manim_global_completion_claim": False,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_VERTICALS_CONTRACT", f"{key}={payload.get(key)!r}")
    checks = payload.get("checks")
    if not isinstance(checks, list):
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_VERTICALS_CHECKS", repr(checks))
    for required in [
        "semantic_roles_primary_side_overlay_magnifier",
        "layout_policy_owned_by_ui_not_ddn_pixel_css",
        "pendulum_one_file_wasm_run_branch_save_reopen",
        "grid_game_wasm_run_action_branch_save_reopen",
    ]:
        if required not in checks:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_VERTICALS_CHECK_MISSING", required)
    return 0


def require_product_tokens() -> int:
    html = (ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "index.html").read_text(encoding="utf-8")
    js = (ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js").read_text(encoding="utf-8")
    for token in [
        "semantic-view-contract",
        "layout-policy",
        "view-role-summary",
        "project-save",
        "project-reopen",
        "side-view-panel",
    ]:
        if token not in html:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_VERTICALS_HTML_TOKEN", token)
    for token in [
        "viewGrade: \"explicit\"",
        "viewGrade: \"auto\"",
        "viewGrade: \"output-only\"",
        "seamgrim.approved_workbench.project_snapshot.v1",
        "stableCanonicalJson",
        "projectStorageKey",
    ]:
        if token not in js:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_VERTICALS_JS_TOKEN", token)
    return 0


def run_browser() -> int:
    proc = run(["node", str(RUNNER.relative_to(ROOT))], timeout=240)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_VERTICALS_BROWSER", proc.stdout.strip())
    if "seamgrim_approved_workbench_semantic_verticals: ok" not in proc.stdout:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_VERTICALS_BROWSER_OK", proc.stdout.strip())
    return 0


def run_pack_golden() -> int:
    proc = run(["python", "tests/run_pack_golden.py", "seamgrim_approved_workbench_semantic_verticals_v1"], timeout=120)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_VERTICALS_PACK", proc.stdout.strip())
    return 0


def require_docs_ssot_clean() -> int:
    proc = run(
        ["git", "-c", "core.excludesfile=", "status", "--short", "--", "docs/ssot"],
        timeout=60,
    )
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_VERTICALS_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_VERTICALS_SSOT_DIRTY", proc.stdout.strip())
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
    print("[seamgrim-approved-workbench-semantic-verticals-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
