#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_approved_workbench_shell_v1"
RUNNER = ROOT / "tests" / "seamgrim_approved_workbench_shell_runner.mjs"


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
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "index.html",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.css",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_projectile_xy_v1" / "lesson.ddn",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_pendulum_wave1_v1" / "lesson.ddn",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_grid_game_state_drop_v1" / "lesson.ddn",
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_MISSING", str(missing))
    return 0


def require_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_approved_workbench_shell_v1",
        "goal": "approved_deterministic_workbench",
        "product_path": "solutions/seamgrim_ui_mvp/ui/workbench/index.html",
        "product_behavior_change": True,
        "behavior_closed_claim": True,
        "sealed_wasm_lifecycle_claim": False,
        "immutable_lineage_claim": False,
        "docs_ssot_change": False,
        "new_ddn_syntax": False,
        "state_hash_formula_change": False,
        "cloud_or_public_registry_claim": False,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_CONTRACT", f"{key}={payload.get(key)!r}")
    checks = payload.get("checks")
    if not isinstance(checks, list) or len(checks) < 8:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_CHECKS", repr(checks))
    for required in [
        "global_entries_ddn_search_official_collection_my_work",
        "single_file_hides_module_ui",
        "multi_file_tabs_and_drawer_search",
        "ddn_main_60_40_layout",
        "bogae_main_40_60_layout",
        "bogae_full_layout_with_source_access",
    ]:
        if required not in checks:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_CHECK_MISSING", required)
    return 0


def require_product_tokens() -> int:
    html = (ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "index.html").read_text(encoding="utf-8")
    js = (ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js").read_text(encoding="utf-8")
    css = (ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.css").read_text(encoding="utf-8")
    for token in ["global-ddn-search", "global-official-collection", "global-my-work", "mode-ddn-main", "mode-bogae-main", "mode-bogae-full"]:
        if token not in html:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_HTML_TOKEN", token)
    for token in ["rep_phys_projectile_xy_v1", "rep_phys_pendulum_wave1_v1", "rep_grid_game_state_drop_v1", "single-file, module UI hidden", "multi-file drawer/search"]:
        if token not in js:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_JS_TOKEN", token)
    for token in ['grid-template-columns: minmax(0, 3fr) minmax(0, 2fr)', 'grid-template-columns: minmax(0, 2fr) minmax(0, 3fr)', '@media (max-width: 720px)']:
        if token not in css:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_CSS_TOKEN", token)
    banned = ["학생", "교사", "프로필"]
    shell_text = html + "\n" + js
    for token in banned:
        if token in shell_text:
            return fail("E_SEAMGRIM_APPROVED_WORKBENCH_PROFILE_SHELF", token)
    return 0


def run_browser() -> int:
    proc = run(["node", str(RUNNER.relative_to(ROOT))], timeout=180)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_BROWSER", proc.stdout.strip())
    if "seamgrim_approved_workbench_shell: ok" not in proc.stdout:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_BROWSER_OK", proc.stdout.strip())
    return 0


def run_pack_golden() -> int:
    proc = run(["python", "tests/run_pack_golden.py", "seamgrim_approved_workbench_shell_v1"], timeout=120)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_PACK", proc.stdout.strip())
    return 0


def require_docs_ssot_clean() -> int:
    proc = run(
        ["git", "-c", "core.excludesfile=", "status", "--short", "--", "docs/ssot"],
        timeout=60,
    )
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_APPROVED_WORKBENCH_SSOT_DIRTY", proc.stdout.strip())
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
    print("[seamgrim-approved-workbench-shell-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
