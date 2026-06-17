from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_OPERATIONS_PREVIEW_STAGE_CLOSURE_V1.md"
REPORT = ROOT / "docs" / "studio" / "OPERATIONS_PREVIEW_STAGE_CLOSURE_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "studio_operations_preview_stage_closure_v1"
CONTRACT = PACK / "contract.detjson"
CLOSURE = PACK / "operations_preview_stage_closure.detjson"
SOURCE_LOCK = ROOT / "pack" / "studio_ma3_next_queue_coordinate_lock_v1" / "ma3_next_queue_coordinate_lock.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_operations_preview_stage_closure.js"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
RUNNER = ROOT / "tests" / "studio_operations_preview_stage_closure_runner.mjs"
SOURCE_LOCK_CHECK = ROOT / "tests" / "run_studio_ma3_next_queue_coordinate_lock_check.py"
PRODUCT_SMOKE = ROOT / "tests" / "run_seamgrim_product_stabilization_smoke_check.py"
NEXT = "STUDIO_PRODUCTIZATION_STAGE_REBASE_V1"


def fail(message: str) -> None:
    print(f"studio_operations_preview_stage_closure_check: FAIL: {message}", file=sys.stderr)
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
        "closure_surface": "local_studio_operations_preview_stage_closure",
        "behavior_closed": True,
        "docs_closed": True,
        "stage_closure_only": True,
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
    rows: list[tuple[str, str, str, str, str]] = [
        (
            "teacher_feedback_surface_preview",
            "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1",
            "tests/studio_teacher_feedback_surface_preview_runner.mjs",
            "하-3",
            "teacher_feedback",
        ),
        (
            "classroom_operations_panel_preview",
            "STUDIO_CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1",
            "tests/studio_classroom_operations_panel_preview_runner.mjs",
            "하-3",
            "classroom_operations",
        ),
        (
            "benchmark_baseline_local_snapshot",
            "STUDIO_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1",
            "tests/studio_benchmark_baseline_local_snapshot_runner.mjs",
            "타-3",
            "benchmark_baseline",
        ),
        (
            "release_review_packet_dashboard",
            "STUDIO_RELEASE_REVIEW_PACKET_DASHBOARD_V1",
            "tests/studio_release_review_packet_dashboard_runner.mjs",
            "마-3",
            "release_review",
        ),
        (
            "lesson_publication_review_surface",
            "STUDIO_LESSON_PUBLICATION_REVIEW_SURFACE_V1",
            "tests/studio_lesson_publication_review_surface_runner.mjs",
            "마-3",
            "lesson_publication",
        ),
        (
            "ma3_regression_gate_matrix",
            "STUDIO_MA3_REGRESSION_GATE_MATRIX_V1",
            "tests/studio_ma3_regression_gate_matrix_runner.mjs",
            "타-3",
            "regression_gate",
        ),
        (
            "ma3_next_queue_coordinate_lock",
            "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1",
            "tests/studio_ma3_next_queue_coordinate_lock_runner.mjs",
            "마-3",
            "coordinate_lock",
        ),
        (
            "operations_preview_stage_closure",
            "STUDIO_OPERATIONS_PREVIEW_STAGE_CLOSURE_V1",
            "solutions/seamgrim_ui_mvp/ui/studio_operations_preview_stage_closure.js",
            "마-3",
            "stage_closure",
        ),
    ]
    return [
        {
            "id": row_id,
            "work_item": work_item,
            "source_anchor": source_anchor,
            "coordinate": coordinate,
            "closure_lane": closure_lane,
            **common,
        }
        for row_id, work_item, source_anchor, coordinate, closure_lane in rows
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
        CLOSURE,
        SOURCE_LOCK,
        UI_MODULE,
        APP_JS,
        DEV_SURFACES_JS,
        INDEX_HTML,
        STYLES_CSS,
        RUNNER,
        SOURCE_LOCK_CHECK,
        PRODUCT_SMOKE,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "STUDIO_OPERATIONS_PREVIEW_STAGE_CLOSURE_V1",
        "ddn.studio.operations_preview_stage_closure.v1",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        "닫힘-동작",
        "closure rows: 8/8 = 100%",
        "전체 초장기 계획: 9/18 = 50%",
        "현재 스테이지: 새 마-3 개발 계획 8/8 = 100%",
        "ROADMAP_V2 matrix behavior baseline: 51/90 = 57%",
        "studio_operations_preview_stage_closure_runner.mjs",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens)
    require_contains(REPORT, tokens[:10])
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_OPERATIONS_PREVIEW_STAGE_CLOSURE_V1",
            "studio_operations_preview_stage_closure_runner.mjs",
            "closure rows: 8/8 = 100%",
            "전체 초장기 계획: 9/18 = 50%",
            "현재 스테이지: 새 마-3 개발 계획 8/8 = 100%",
            "ROADMAP_V2 matrix behavior baseline: 51/90 = 57%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_ui_source() -> None:
    require_contains(
        UI_MODULE,
        [
            "ddn.studio.operations_preview_stage_closure.v1",
            "buildOperationsPreviewStageClosure",
            "formatOperationsPreviewStageClosureText",
            "renderOperationsPreviewStageClosure",
            "stage_closure_only: true",
            "product_ui_change: true",
            "release_execution_claim: false",
            "super_long_behavior_closed: 9",
            "current_stage_percent: 100",
            "roadmap_v2_percent: 57",
        ],
    )
    require_contains(
        DEV_SURFACES_JS,
        [
            "studio_operations_preview_stage_closure.js",
            "__SEAMGRIM_OPERATIONS_PREVIEW_STAGE_CLOSURE__",
            "buildOperationsPreviewStageClosure",
        ],
    )
    require_contains(DEV_SURFACES_JS, ["operations-preview-stage-closure", "elementId: \"operations-preview-stage-closure\""])
    require_contains(APP_JS, ["shouldEnableDevSurfaces", "./dev_surfaces.js"])
    require_contains(STYLES_CSS, [".operations-preview-stage-closure", ".operations-stage-closure-btn.active"])
    require_contains(
        RUNNER,
        [
            "studio_operations_preview_stage_closure: ok",
            "data-operations-preview-stage-status='operations_preview_stage_closed'",
            "current_stage_percent",
        ],
    )


def check_contract_and_closure() -> None:
    contract = load_json(CONTRACT)
    expected_contract = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_operations_preview_stage_closure_v1",
        "kind": "studio_operations_preview_stage_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "stage_closure_claim": True,
        "new_automatic_queue_claim": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "release_execution_claim": False,
        "public_upload_claim": False,
        "closed_by": "STUDIO_OPERATIONS_PREVIEW_STAGE_CLOSURE_V1",
        "based_on": "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1",
        "closure_row_count": 8,
        "all_rows_stage_closure_only": True,
        "all_rows_behavior_closed": True,
        "all_rows_generated_now": False,
        "all_rows_release_execution_claim": False,
        "primary_coordinate": "마-3",
        "support_coordinate": "타-3",
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "closure_rows_closed": 8,
        "closure_rows_total": 8,
        "closure_rows_percent": 100,
        "super_long_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "current_stage_closed": 8,
        "current_stage_total": 8,
        "current_stage_percent": 100,
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
        "solutions/seamgrim_ui_mvp/ui/studio_operations_preview_stage_closure.js",
        "solutions/seamgrim_ui_mvp/ui/app.js",
        "solutions/seamgrim_ui_mvp/ui/index.html",
        "solutions/seamgrim_ui_mvp/ui/styles.css",
    ]:
        if file not in contract.get("changed_product_files", []):
            fail(f"contract missing changed product file: {file}")

    closure = load_json(CLOSURE)
    if closure.get("schema") != "ddn.studio.operations_preview_stage_closure.v1":
        fail(f"closure schema mismatch: {closure.get('schema')!r}")
    if closure.get("work_item") != "STUDIO_OPERATIONS_PREVIEW_STAGE_CLOSURE_V1":
        fail(f"closure work item mismatch: {closure.get('work_item')!r}")
    for flag, expected in (
        ("runtime_claim", False),
        ("product_code_change", True),
        ("product_ui_change", True),
        ("stage_closure_only", True),
        ("new_automatic_queue_claim", False),
        ("release_execution_claim", False),
        ("public_upload_claim", False),
        ("registry_publish_claim", False),
        ("benchmark_execution_claim", False),
        ("performance_baseline_generation_claim", False),
        ("lesson_schema_change", False),
        ("active_allowlist_mutation", False),
    ):
        if closure.get(flag) is not expected:
            fail(f"closure {flag} expected {expected!r}, got {closure.get(flag)!r}")
    if closure.get("closure_rows") != expected_rows():
        fail(f"closure rows mismatch: {closure.get('closure_rows')!r}")
    if closure.get("progress") != {
        "super_long_behavior_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "current_stage_closed": 8,
        "current_stage_total": 8,
        "current_stage_percent": 100,
        "roadmap_v2_behavior_closed": 51,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 57,
    }:
        fail(f"progress mismatch: {closure.get('progress')!r}")
    if closure.get("closure_tier") != "닫힘-동작":
        fail(f"closure tier mismatch: {closure.get('closure_tier')!r}")
    if closure.get("next_item") != NEXT:
        fail(f"next item mismatch: {closure.get('next_item')!r}")


def check_source_alignment() -> None:
    source_lock = load_json(SOURCE_LOCK)
    if source_lock.get("schema") != "ddn.studio.ma3_next_queue_coordinate_lock.v1":
        fail(f"source lock schema mismatch: {source_lock.get('schema')!r}")
    if source_lock.get("next_item") != "STUDIO_OPERATIONS_PREVIEW_STAGE_CLOSURE_V1":
        fail(f"source lock next item mismatch: {source_lock.get('next_item')!r}")
    if source_lock.get("progress", {}).get("super_long_behavior_closed") != 9:
        fail(f"source lock progress mismatch: {source_lock.get('progress')!r}")
    if source_lock.get("progress", {}).get("roadmap_v2_behavior_closed") != 51:
        fail(f"source lock roadmap closed mismatch: {source_lock.get('progress')!r}")
    if source_lock.get("progress", {}).get("roadmap_v2_percent") != 57:
        fail(f"source lock roadmap percent mismatch: {source_lock.get('progress')!r}")
    lock_ids = [row.get("id") for row in source_lock.get("lock_rows", [])]
    if "next_stage_handoff_lock" not in lock_ids:
        fail(f"missing next stage handoff lock source anchor: {lock_ids!r}")
    for row in expected_rows():
        source_anchor = str(row["source_anchor"])
        if source_anchor.startswith("tests/") or source_anchor.startswith("solutions/"):
            require(ROOT / source_anchor)


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected_stdout = [
        "STUDIO_OPERATIONS_PREVIEW_STAGE_CLOSURE_V1",
        "studio operations preview stage closure behavior sealed",
        "operations preview stage closure schema: ddn.studio.operations_preview_stage_closure.v1",
        "closure rows: 8/8 = 100%",
        "current stage: 8/8 = 100%",
        "overall super-long behavior: 9/18 = 50%",
        "roadmap v2 behavior: 51/90 = 57%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_operations_preview_stage_closure_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected_stdout:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_operations_preview_stage_closure_v1"],
        ["node", "tests/studio_operations_preview_stage_closure_runner.mjs"],
        ["python", "tests/run_studio_ma3_next_queue_coordinate_lock_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_ui_source()
    check_contract_and_closure()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_operations_preview_stage_closure_check: ok")


if __name__ == "__main__":
    main()
