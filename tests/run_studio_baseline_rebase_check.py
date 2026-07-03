#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ROADMAP = ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
REBASE = ROOT / "STUDIO_BASELINE_REBASE_V1.md"
PACK = ROOT / "pack" / "studio_baseline_rebase_v1"
RETIRED_ROOT_DOCS = [
    ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md",
    ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_CASE_SUITE_CHECK_RUNNER_V1V.md",
    ROOT / "ROOT_LOW_RISK_RETIRE_DELETE_V1.md",
]


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        ROADMAP,
        REBASE,
        ROOT / "pack" / "connect_flow_v1v_closure_v1" / "contract.detjson",
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "browse.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "editor.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "block_editor.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "rpg_box.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "runtime" / "wasm_vm_runtime.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "runtime" / "wasm_canon_runtime.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "runtime" / "lesson_canon_runtime.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "lesson_loader_contract.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "inspector_contract.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_edit_run_contract.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "block_editor" / "ddn_block_codec.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "block_editor" / "ddn_block_engine.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "block_editor" / "seamgrim_palette.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "seed_lessons_v1" / "seed_manifest.detjson",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons",
        ROOT / "tests" / "run_seamgrim_product_stabilization_smoke_check.py",
        ROOT / "tests" / "run_seamgrim_live_repl_check.py",
        ROOT / "tests" / "run_seamgrim_wasm_smoke.py",
        ROOT / "tests" / "run_seamgrim_wasm_cli_runtime_parity_check.py",
        ROOT / "tests" / "run_block_editor_roundtrip_check.py",
        ROOT / "tests" / "run_seamgrim_malblock_roundtrip_subset_check.py",
        ROOT / "tests" / "run_seamgrim_grid_pathfind_lesson_check.py",
        ROOT / "tests" / "seamgrim_ui_common_runner.mjs",
        ROOT / "tests" / "seamgrim_studio_layout_contract_runner.mjs",
        ROOT / "tests" / "seamgrim_lesson_loader_runner.mjs",
        ROOT / "pack" / "block_editor_roundtrip_v1" / "expected" / "block_editor_roundtrip.detjson",
        ROOT / "pack" / "seamgrim_malblock_roundtrip_subset_v1" / "expected" / "malblock_roundtrip_subset.detjson",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_BASELINE_REBASE_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_roadmap_doc() -> int:
    return require_tokens(
        ROADMAP,
        [
            "STUDIO_LONG_HORIZON_ROADMAP_V1",
            "Seamgrim/Studio productization",
            "STUDIO_BASELINE_REBASE_V1",
            "SEAMGRIM_WORKBENCH_SHELL_V1",
            "SEAMGRIM_LESSON_AUTHORING_FLOW_V1",
            "MALBLOCK_AUTHORING_UI_V1",
            "STUDIO_DIAGNOSTIC_FIXIT_PREVIEW_V1",
            "STUDIO_CLASSROOM_MODE_V1",
            "STUDIO_LOCAL_SHARE_AND_PACKAGING_V1",
            "STUDIO_RELEASE_CANDIDATE_V1",
            "no product code",
            "parser/frontdoor grammar",
            "docs/ssot/**",
        ],
        "E_STUDIO_LONG_HORIZON_ROADMAP_DOC",
    )


def check_rebase_doc() -> int:
    return require_tokens(
        REBASE,
        [
            "STUDIO_BASELINE_REBASE_V1",
            "documentation/checker evidence only",
            "Baseline Inventory",
            "UI shell",
            "Runtime and contracts",
            "Block editor",
            "Lesson evidence",
            "Runtime/check evidence",
            "connect_flow_v1v_closure_v1",
            "ROOT_LOW_RISK_RETIRE_DELETE_V1",
            "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md",
            "SEAMGRIM_WORKBENCH_SHELL_V1",
            "docs/ssot/**",
        ],
        "E_STUDIO_BASELINE_REBASE_DOC",
    )


def check_pack_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_baseline_rebase_v1",
        "kind": "planning_inventory_marker",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_BASELINE_REBASE_V1",
        "next_item": "SEAMGRIM_WORKBENCH_SHELL_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_STUDIO_BASELINE_REBASE_CONTRACT", f"{key}={payload.get(key)!r}")
    groups = payload.get("inventory_groups")
    if not isinstance(groups, list) or len(groups) < 5:
        return fail("E_STUDIO_BASELINE_REBASE_CONTRACT_GROUPS", repr(groups))
    return 0


def check_pack_golden() -> int:
    line = (PACK / "golden.jsonl").read_text(encoding="utf-8").strip()
    payload = json.loads(line)
    if payload.get("id") != "c01_studio_baseline_rebase_marker":
        return fail("E_STUDIO_BASELINE_REBASE_GOLDEN_ID", repr(payload.get("id")))
    stdout = payload.get("stdout")
    expected = [
        "STUDIO_BASELINE_REBASE_V1",
        "studio baseline inventory sealed",
        "next: SEAMGRIM_WORKBENCH_SHELL_V1",
    ]
    if stdout != expected:
        return fail("E_STUDIO_BASELINE_REBASE_GOLDEN_STDOUT", repr(stdout))
    return 0


def check_retired_root_docs_absent() -> int:
    present = [str(path.relative_to(ROOT)) for path in RETIRED_ROOT_DOCS if path.exists()]
    if present:
        return fail("E_STUDIO_BASELINE_REBASE_RETIRED_ROOT_DOCS_PRESENT", str(present))
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_LONG_HORIZON_ROADMAP_V1",
        "STUDIO_BASELINE_REBASE_V1",
        "studio_baseline_rebase_v1",
        "SEAMGRIM_WORKBENCH_SHELL_V1",
        "python tests/run_studio_baseline_rebase_check.py",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_BASELINE_REBASE_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    result = subprocess.run(
        ["git", "status", "--short", "--", "docs/ssot"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode != 0:
        return fail("E_STUDIO_BASELINE_REBASE_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_STUDIO_BASELINE_REBASE_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_roadmap_doc,
        check_rebase_doc,
        check_pack_contract,
        check_pack_golden,
        check_retired_root_docs_absent,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-baseline-rebase-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
