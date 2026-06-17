from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_PRODUCTIZATION_STAGE_REBASE_V1.md"
REPORT = ROOT / "docs" / "studio" / "PRODUCTIZATION_STAGE_REBASE_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "studio_productization_stage_rebase_v1"
CONTRACT = PACK / "contract.detjson"
REBASE = PACK / "productization_stage_rebase.detjson"
SOURCE_CLOSURE = ROOT / "pack" / "studio_operations_preview_stage_closure_v1" / "operations_preview_stage_closure.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_productization_stage_rebase.js"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
RUNNER = ROOT / "tests" / "studio_productization_stage_rebase_runner.mjs"
SOURCE_CLOSURE_CHECK = ROOT / "tests" / "run_studio_operations_preview_stage_closure_check.py"
PRODUCT_SMOKE = ROOT / "tests" / "run_seamgrim_product_stabilization_smoke_check.py"
NEXT = "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1"


def fail(message: str) -> None:
    print(f"studio_productization_stage_rebase_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")


def require(path: Path) -> None:
    if not path.exists():
        fail(f"missing required path: {path.relative_to(ROOT)}")


def require_contains(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing tokens: {missing}")


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
        check=False,
    )


def load_json(path: Path) -> dict:
    return json.loads(read(path))


def require_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed: {proc.stdout.strip()}")
    status_lines = [
        line for line in proc.stdout.splitlines()
        if line.strip() and not line.startswith("warning:")
    ]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def expected_rows() -> list[dict[str, object]]:
    common = {
        "rebase_surface": "local_studio_productization_stage_rebase",
        "stage_rebase_only": True,
        "generated_now": False,
        "new_automatic_queue_claim": False,
        "release_approval_claim": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "public_upload_claim": False,
        "registry_publish_claim": False,
        "benchmark_execution_claim": False,
        "performance_baseline_generation_claim": False,
        "active_allowlist_mutation": False,
        "lesson_schema_change": False,
        "product_ui_change": True,
    }
    rows: list[tuple[str, str, str, str]] = [
        (
            "operations_preview_closure_anchor",
            "pack/studio_operations_preview_stage_closure_v1/operations_preview_stage_closure.detjson",
            "마-3",
            "stage_handoff",
        ),
        (
            "micro_slice_consolidation_priority",
            NEXT,
            "마-3",
            "micro_slice_consolidation",
        ),
        (
            "productization_stage_denominator",
            "STUDIO_PRODUCTIZATION_STAGE_REBASE_V1",
            "마-3",
            "stage_denominator",
        ),
        (
            "release_boundary_guard",
            "AWAIT_EXPLICIT_RELEASE_APPROVAL",
            "타-3",
            "release_boundary",
        ),
        (
            "next_item_selection",
            NEXT,
            "마-3",
            "next_selection",
        ),
    ]
    return [
        {
            "id": row_id,
            "source_anchor": source_anchor,
            "coordinate": coordinate,
            "rebase_lane": rebase_lane,
            **common,
        }
        for row_id, source_anchor, coordinate, rebase_lane in rows
    ]


def check_required_files() -> None:
    for path in [
        DOC,
        REPORT,
        DEV_SUMMARY,
        PACK / "README.md",
        CONTRACT,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        REBASE,
        SOURCE_CLOSURE,
        UI_MODULE,
        APP_JS,
        DEV_SURFACES_JS,
        INDEX_HTML,
        STYLES_CSS,
        RUNNER,
        SOURCE_CLOSURE_CHECK,
        PRODUCT_SMOKE,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "STUDIO_PRODUCTIZATION_STAGE_REBASE_V1",
        "ddn.studio.productization_stage_rebase.v1",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        "닫힘-동작",
        "rebase rows: 5/5 = 100%",
        "전체 초장기 계획: 9/18 = 50%",
        "현재 스테이지: Studio productization rebase 1/5 = 20%",
        "ROADMAP_V2 matrix behavior baseline: 51/90 = 57%",
        "studio_productization_stage_rebase_runner.mjs",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens)
    require_contains(REPORT, tokens[:10])
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_PRODUCTIZATION_STAGE_REBASE_V1",
            "studio_productization_stage_rebase_runner.mjs",
            "rebase rows: 5/5 = 100%",
            "전체 초장기 계획: 9/18 = 50%",
            "현재 스테이지: Studio productization rebase 1/5 = 20%",
            "ROADMAP_V2 matrix behavior baseline: 51/90 = 57%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_ui_source() -> None:
    require_contains(
        UI_MODULE,
        [
            "ddn.studio.productization_stage_rebase.v1",
            "buildProductizationStageRebase",
            "formatProductizationStageRebaseText",
            "renderProductizationStageRebase",
            "selected_next_item: \"SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1\"",
            "stage_rebase_only: true",
            "product_ui_change: true",
            "release_execution_claim: false",
            "super_long_behavior_closed: 9",
            "current_stage_percent: 20",
            "roadmap_v2_percent: 57",
        ],
    )
    require_contains(
        DEV_SURFACES_JS,
        [
            "studio_productization_stage_rebase.js",
            "__SEAMGRIM_PRODUCTIZATION_STAGE_REBASE__",
            "buildProductizationStageRebase",
        ],
    )
    require_contains(APP_JS, ["shouldEnableDevSurfaces", "./dev_surfaces.js"])
    require_contains(DEV_SURFACES_JS, ["productization-stage-rebase", "elementId: \"productization-stage-rebase\""])
    require_contains(STYLES_CSS, [".productization-stage-rebase", ".productization-rebase-btn.active"])
    require_contains(
        RUNNER,
        [
            "studio_productization_stage_rebase: ok",
            "data-productization-stage-rebase-status='productization_stage_rebased'",
            "current_stage_percent",
        ],
    )


def check_contract_and_rebase() -> None:
    contract = load_json(CONTRACT)
    expected_contract = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_productization_stage_rebase_v1",
        "kind": "studio_productization_stage_rebase",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "stage_rebase_claim": True,
        "new_automatic_queue_claim": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "release_execution_claim": False,
        "public_upload_claim": False,
        "closed_by": "STUDIO_PRODUCTIZATION_STAGE_REBASE_V1",
        "based_on": "STUDIO_OPERATIONS_PREVIEW_STAGE_CLOSURE_V1",
        "rebase_row_count": 5,
        "all_rows_stage_rebase_only": True,
        "all_rows_generated_now": False,
        "all_rows_release_execution_claim": False,
        "selected_next_item": NEXT,
        "primary_coordinate": "마-3",
        "support_coordinate": "타-3",
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "rebase_rows_closed": 5,
        "rebase_rows_total": 5,
        "rebase_rows_percent": 100,
        "super_long_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "current_stage_closed": 1,
        "current_stage_total": 5,
        "current_stage_percent": 20,
        "roadmap_v2_behavior_closed": 51,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 57,
        "closure_tier": "닫힘-동작",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected_contract.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")
    for file in [
        "solutions/seamgrim_ui_mvp/ui/studio_productization_stage_rebase.js",
        "solutions/seamgrim_ui_mvp/ui/app.js",
        "solutions/seamgrim_ui_mvp/ui/index.html",
        "solutions/seamgrim_ui_mvp/ui/styles.css",
    ]:
        if file not in contract.get("changed_product_files", []):
            fail(f"contract missing changed product file: {file}")

    rebase = load_json(REBASE)
    if rebase.get("schema") != "ddn.studio.productization_stage_rebase.v1":
        fail(f"rebase schema mismatch: {rebase.get('schema')!r}")
    if rebase.get("work_item") != "STUDIO_PRODUCTIZATION_STAGE_REBASE_V1":
        fail(f"rebase work item mismatch: {rebase.get('work_item')!r}")
    if rebase.get("selected_next_item") != NEXT:
        fail(f"selected next item mismatch: {rebase.get('selected_next_item')!r}")
    for flag, expected in (
        ("runtime_claim", False),
        ("product_code_change", True),
        ("product_ui_change", True),
        ("stage_rebase_only", True),
        ("new_automatic_queue_claim", False),
        ("release_execution_claim", False),
        ("public_upload_claim", False),
        ("registry_publish_claim", False),
        ("benchmark_execution_claim", False),
        ("performance_baseline_generation_claim", False),
        ("lesson_schema_change", False),
        ("active_allowlist_mutation", False),
    ):
        if rebase.get(flag) is not expected:
            fail(f"rebase {flag} expected {expected!r}, got {rebase.get(flag)!r}")
    if rebase.get("rebase_rows") != expected_rows():
        fail(f"rebase rows mismatch: {rebase.get('rebase_rows')!r}")
    if rebase.get("progress") != {
        "super_long_behavior_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "current_stage_closed": 1,
        "current_stage_total": 5,
        "current_stage_percent": 20,
        "roadmap_v2_behavior_closed": 51,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 57,
    }:
        fail(f"progress mismatch: {rebase.get('progress')!r}")
    if rebase.get("closure_tier") != "닫힘-동작":
        fail(f"closure tier mismatch: {rebase.get('closure_tier')!r}")
    if rebase.get("next_item") != NEXT:
        fail(f"next item mismatch: {rebase.get('next_item')!r}")


def check_source_alignment() -> None:
    source_closure = load_json(SOURCE_CLOSURE)
    if source_closure.get("schema") != "ddn.studio.operations_preview_stage_closure.v1":
        fail(f"source closure schema mismatch: {source_closure.get('schema')!r}")
    if source_closure.get("next_item") != "STUDIO_PRODUCTIZATION_STAGE_REBASE_V1":
        fail(f"source closure next item mismatch: {source_closure.get('next_item')!r}")
    if source_closure.get("progress", {}).get("current_stage_percent") != 100:
        fail(f"source closure progress mismatch: {source_closure.get('progress')!r}")
    if source_closure.get("progress", {}).get("roadmap_v2_behavior_closed") != 51:
        fail(f"source closure roadmap closed mismatch: {source_closure.get('progress')!r}")
    if source_closure.get("progress", {}).get("roadmap_v2_percent") != 57:
        fail(f"source closure roadmap percent mismatch: {source_closure.get('progress')!r}")
    closure_ids = [row.get("id") for row in source_closure.get("closure_rows", [])]
    if "operations_preview_stage_closure" not in closure_ids:
        fail(f"missing operations preview closure row: {closure_ids!r}")
    require(ROOT / "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1.md")
    require(ROOT / "tests" / "run_seamgrim_numeric_track_consolidation_check.py")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected_stdout = [
        "STUDIO_PRODUCTIZATION_STAGE_REBASE_V1",
        "studio productization stage rebase behavior sealed",
        "productization stage rebase schema: ddn.studio.productization_stage_rebase.v1",
        "rebase rows: 5/5 = 100%",
        "current stage: 1/5 = 20%",
        "overall super-long behavior: 9/18 = 50%",
        "roadmap v2 behavior: 51/90 = 57%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_productization_stage_rebase_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected_stdout:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_productization_stage_rebase_v1"],
        ["node", "tests/studio_productization_stage_rebase_runner.mjs"],
        ["python", "tests/run_studio_operations_preview_stage_closure_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_ui_source()
    check_contract_and_rebase()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_productization_stage_rebase_check: ok")


if __name__ == "__main__":
    main()
