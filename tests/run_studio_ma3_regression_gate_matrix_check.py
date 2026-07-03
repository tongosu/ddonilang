from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_MA3_REGRESSION_GATE_MATRIX_V1.md"
REPORT = ROOT / "docs" / "studio" / "MA3_REGRESSION_GATE_MATRIX_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "studio_ma3_regression_gate_matrix_v1"
CONTRACT = PACK / "contract.detjson"
MATRIX = PACK / "ma3_regression_gate_matrix.detjson"
SOURCE_SURFACE = ROOT / "pack" / "studio_lesson_publication_review_surface_v1" / "lesson_publication_review_surface.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_ma3_regression_gate_matrix.js"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
RUNNER = ROOT / "tests" / "studio_ma3_regression_gate_matrix_runner.mjs"
SOURCE_SURFACE_CHECK = ROOT / "tests" / "run_studio_lesson_publication_review_surface_check.py"
PRODUCT_SMOKE = ROOT / "tests" / "run_seamgrim_product_stabilization_smoke_check.py"
NEXT = "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1"


def fail(message: str) -> None:
    print(f"studio_ma3_regression_gate_matrix_check: FAIL: {message}", file=sys.stderr)
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
        "matrix_surface": "local_ma3_regression_gate_matrix",
        "gate_matrix_only": True,
        "generated_now": False,
        "test_execution_claim": False,
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
    rows: list[tuple[str, str, str | None, str]] = [
        ("teacher_feedback_surface_gate", "tests/studio_teacher_feedback_surface_preview_runner.mjs", "teacher_summary_panel", "teacher_feedback"),
        ("classroom_operations_panel_gate", "tests/studio_classroom_operations_panel_preview_runner.mjs", "classroom_report_status_panel", "classroom_operations"),
        ("benchmark_baseline_snapshot_gate", "tests/studio_benchmark_baseline_local_snapshot_runner.mjs", "benchmark_lts_matrix_snapshot", "benchmark_baseline"),
        ("release_review_packet_gate", "tests/studio_release_review_packet_dashboard_runner.mjs", "approval_state_dashboard_card", "release_review"),
        ("lesson_publication_surface_gate", "tests/studio_lesson_publication_review_surface_runner.mjs", "candidate_catalog_review_surface", "lesson_publication"),
        ("product_stabilization_smoke_gate", "tests/run_seamgrim_product_stabilization_smoke_check.py", None, "product_stabilization"),
    ]
    return [
        {
            "id": row_id,
            "source_runner": runner,
            "source_surface_row": surface_row,
            "gate_lane": lane,
            **common,
        }
        for row_id, runner, surface_row, lane in rows
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
        MATRIX,
        SOURCE_SURFACE,
        UI_MODULE,
        APP_JS,
        DEV_SURFACES_JS,
        INDEX_HTML,
        STYLES_CSS,
        RUNNER,
        SOURCE_SURFACE_CHECK,
        PRODUCT_SMOKE,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "STUDIO_MA3_REGRESSION_GATE_MATRIX_V1",
        "ddn.studio.ma3_regression_gate_matrix.v1",
        "Primary coordinate: `타-3`",
        "Support coordinate: `마-3`",
        "닫힘-동작",
        "gate rows: 6/6 = 100%",
        "전체 초장기 계획: 8/18 = 44%",
        "현재 스테이지: 새 마-3 개발 계획 7/8 = 88%",
        "ROADMAP_V2 matrix behavior: 6/90 = 7%",
        "studio_ma3_regression_gate_matrix_runner.mjs",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens)
    require_contains(REPORT, tokens[:10])
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_MA3_REGRESSION_GATE_MATRIX_V1",
            "studio_ma3_regression_gate_matrix_runner.mjs",
            "gate rows: 6/6 = 100%",
            "전체 초장기 계획: 8/18 = 44%",
            "현재 스테이지: 새 마-3 개발 계획 7/8 = 88%",
            "ROADMAP_V2 matrix behavior: 6/90 = 7%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_ui_source() -> None:
    require_contains(
        UI_MODULE,
        [
            "ddn.studio.ma3_regression_gate_matrix.v1",
            "buildMa3RegressionGateMatrix",
            "formatMa3RegressionGateMatrixText",
            "renderMa3RegressionGateMatrix",
            "product_ui_change: true",
            "test_execution_claim: false",
            "release_execution_claim: false",
            "super_long_behavior_closed: 8",
            "current_stage_percent: 88",
            "roadmap_v2_percent: 100",
        ],
    )
    require_contains(
        DEV_SURFACES_JS,
        [
            "studio_ma3_regression_gate_matrix.js",
            "__SEAMGRIM_MA3_REGRESSION_GATE_MATRIX__",
            "buildMa3RegressionGateMatrix",
        ],
    )
    require_contains(DEV_SURFACES_JS, ["ma3-regression-gate-matrix", "elementId: \"ma3-regression-gate-matrix\""])
    require_contains(APP_JS, ["shouldEnableDevSurfaces", "./dev_surfaces.js"])
    require_contains(DEV_SURFACES_CSS, [".ma3-regression-gate-matrix", ".ma3-regression-matrix-btn.active"])
    require_contains(RUNNER, ["studio_ma3_regression_gate_matrix: ok", "data-ma3-regression-status='ma3_regression_gate_matrix_ready'", "test_execution_claim"])


def check_contract_and_matrix() -> None:
    contract = load_json(CONTRACT)
    expected_contract = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_ma3_regression_gate_matrix_v1",
        "kind": "studio_ma3_regression_gate_matrix",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "test_execution_claim": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "ma3_regression_gate_matrix_claim": True,
        "release_execution_claim": False,
        "public_upload_claim": False,
        "closed_by": "STUDIO_MA3_REGRESSION_GATE_MATRIX_V1",
        "based_on": "STUDIO_LESSON_PUBLICATION_REVIEW_SURFACE_V1",
        "gate_row_count": 6,
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "gate_rows_closed": 6,
        "gate_rows_total": 6,
        "gate_rows_percent": 100,
        "super_long_closed": 8,
        "super_long_total": 18,
        "super_long_percent": 44,
        "current_stage_closed": 7,
        "current_stage_total": 8,
        "current_stage_percent": 88,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
        "closure_tier": "닫힘-동작",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected_contract.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")
    for file in [
        "solutions/seamgrim_ui_mvp/ui/studio_ma3_regression_gate_matrix.js",
        "solutions/seamgrim_ui_mvp/ui/app.js",
        "solutions/seamgrim_ui_mvp/ui/index.html",
        "solutions/seamgrim_ui_mvp/ui/styles.css",
    ]:
        if file not in contract.get("changed_product_files", []):
            fail(f"contract missing changed product file: {file}")

    matrix = load_json(MATRIX)
    if matrix.get("schema") != "ddn.studio.ma3_regression_gate_matrix.v1":
        fail(f"matrix schema mismatch: {matrix.get('schema')!r}")
    if matrix.get("work_item") != "STUDIO_MA3_REGRESSION_GATE_MATRIX_V1":
        fail(f"matrix work item mismatch: {matrix.get('work_item')!r}")
    for flag, expected in (
        ("runtime_claim", False),
        ("product_code_change", True),
        ("product_ui_change", True),
        ("test_execution_claim", False),
        ("release_execution_claim", False),
        ("public_upload_claim", False),
        ("registry_publish_claim", False),
        ("benchmark_execution_claim", False),
        ("lesson_schema_change", False),
        ("active_allowlist_mutation", False),
    ):
        if matrix.get(flag) is not expected:
            fail(f"matrix {flag} expected {expected!r}, got {matrix.get(flag)!r}")
    if matrix.get("gate_rows") != expected_rows():
        fail(f"matrix rows mismatch: {matrix.get('gate_rows')!r}")
    if matrix.get("progress") != {
        "super_long_behavior_closed": 8,
        "super_long_total": 18,
        "super_long_percent": 44,
        "current_stage_closed": 7,
        "current_stage_total": 8,
        "current_stage_percent": 88,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
    }:
        fail(f"progress mismatch: {matrix.get('progress')!r}")
    if matrix.get("closure_tier") != "닫힘-동작":
        fail(f"closure tier mismatch: {matrix.get('closure_tier')!r}")
    if matrix.get("next_item") != NEXT:
        fail(f"next item mismatch: {matrix.get('next_item')!r}")


def check_source_alignment() -> None:
    surface = load_json(SOURCE_SURFACE)
    if surface.get("schema") != "ddn.studio.lesson_publication_review_surface.v1":
        fail(f"source surface schema mismatch: {surface.get('schema')!r}")
    if surface.get("next_item") != "STUDIO_MA3_REGRESSION_GATE_MATRIX_V1":
        fail(f"source surface next item mismatch: {surface.get('next_item')!r}")
    if surface.get("progress", {}).get("super_long_behavior_closed") != 9:
        fail(f"source surface progress mismatch: {surface.get('progress')!r}")
    if surface.get("progress", {}).get("roadmap_v2_behavior_closed") != 90:
        fail(f"source surface roadmap closed mismatch: {surface.get('progress')!r}")
    if surface.get("progress", {}).get("roadmap_v2_percent") != 100:
        fail(f"source surface roadmap percent mismatch: {surface.get('progress')!r}")
    surface_ids = [row.get("id") for row in surface.get("surface_rows", [])]
    if "candidate_catalog_review_surface" not in surface_ids:
        fail(f"missing source surface anchor: {surface_ids!r}")
    for row in expected_rows():
        require(ROOT / row["source_runner"])


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected_stdout = [
        "STUDIO_MA3_REGRESSION_GATE_MATRIX_V1",
        "studio ma3 regression gate matrix behavior sealed",
        "ma3 regression gate matrix schema: ddn.studio.ma3_regression_gate_matrix.v1",
        "gate rows: 6/6 = 100%",
        "overall super-long behavior: 8/18 = 44%",
        "current stage: 7/8 = 88%",
        "roadmap v2 matrix behavior: 6/90 = 7%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_ma3_regression_gate_matrix_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected_stdout:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_ma3_regression_gate_matrix_v1"],
        ["node", "tests/studio_ma3_regression_gate_matrix_runner.mjs"],
        ["python", "tests/run_studio_lesson_publication_review_surface_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    proc = run(["python", "tests/run_roadmap_v2_ma5_lts_candidate_progress_boundary_check.py"], timeout=420)
    if proc.returncode != 0:
        fail(f"MA5 LTS candidate progress boundary failed:\n{proc.stdout}")
    print("studio_ma3_regression_gate_matrix_check: ok")


if __name__ == "__main__":
    main()
