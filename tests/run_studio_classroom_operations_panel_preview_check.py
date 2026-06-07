from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1.md"
REPORT = ROOT / "docs" / "studio" / "CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_classroom_operations_panel_preview_v1"
PANEL = PACK / "classroom_operations_panel_preview.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_classroom_operations_panel_preview.js"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
RUNNER = ROOT / "tests" / "studio_classroom_operations_panel_preview_runner.mjs"
SOURCE_TRIAGE = ROOT / "pack" / "studio_classroom_operations_triage_v1" / "classroom_operations_triage.detjson"
SOURCE_PREVIEW = ROOT / "pack" / "studio_teacher_feedback_surface_preview_v1" / "teacher_feedback_surface_preview.detjson"
NEXT = "STUDIO_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1"


def fail(message: str) -> None:
    print(f"studio_classroom_operations_panel_preview_check: FAIL: {message}", file=sys.stderr)
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


def run(cmd: list[str], *, timeout: int = 240) -> subprocess.CompletedProcess[str]:
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
    status_lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if status_lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def check_required_files() -> None:
    for path in [
        DOC,
        REPORT,
        INDEX,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        PANEL,
        UI_MODULE,
        APP_JS,
        INDEX_HTML,
        STYLES,
        RUNNER,
        SOURCE_TRIAGE,
        SOURCE_PREVIEW,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "STUDIO_CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1",
        "ddn.studio.classroom_operations_panel_preview.v1",
        "product UI behavior",
        "닫힘-동작",
        "panel rows: 6/6 = 100%",
        "전체 초장기 계획: 18/18 = 100%",
        "현재 스테이지: 새 마-3 개발 계획 3/8 = 38%",
        "ROADMAP_V2 product behavior baseline: 90/90 = 100%",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens)
    require_contains(REPORT, tokens[:9])
    require_contains(
        INDEX,
        [
            "STUDIO_CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1",
            "docs/studio/CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1.md",
            "pack/studio_classroom_operations_panel_preview_v1",
            "tests/run_studio_classroom_operations_panel_preview_check.py",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "STUDIO_CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1",
            "studio_classroom_operations_panel_preview_v1",
            "studio_classroom_operations_panel_preview_runner.mjs",
            "panel rows: 6/6 = 100%",
            "전체 초장기 계획: 18/18 = 100%",
            "현재 스테이지: 새 마-3 개발 계획 3/8 = 38%",
            "ROADMAP_V2 product behavior baseline: 90/90 = 100%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_source_tokens() -> None:
    require_contains(
        UI_MODULE,
        [
            "buildClassroomOperationsPanelPreview",
            "renderClassroomOperationsPanelPreview",
            "formatClassroomOperationsPanelPreviewText",
            "DEFAULT_CLASSROOM_OPERATIONS_TRIAGE_ROWS",
            "ddn.studio.classroom_operations_panel_preview.v1",
            "product_ui_change: true",
            "student_data_collection_claim: false",
            "panel_write_claim: false",
        ],
    )
    require_contains(
        APP_JS,
        [
            "buildClassroomOperationsPanelPreview",
            "publishClassroomOperationsPanelPreview",
            "__SEAMGRIM_CLASSROOM_OPERATIONS_PANEL_PREVIEW__",
            "classroom-operations-panel-preview",
        ],
    )
    require_contains(INDEX_HTML, ["classroom-operations-panel-preview", "data-classroom-operations-panel-preview"])
    require_contains(STYLES, ["classroom-operations-panel-preview", "classroom-operations-panel-btn", "classroom-operations-detail"])
    require_contains(RUNNER, ["studio_classroom_operations_panel_preview: ok", "product_ui_change === true", "buttonCount === 6"])


def check_contract_and_panel() -> None:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_classroom_operations_panel_preview_v1",
        "kind": "studio_classroom_operations_panel_preview",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "classroom_operations_panel_preview_claim": True,
        "classroom_operations_runtime_claim": False,
        "teacher_feedback_runtime_claim": False,
        "student_data_collection_claim": False,
        "panel_write_claim": False,
        "triage_write_claim": False,
        "feedback_write_claim": False,
        "remote_save_claim": False,
        "cloud_sync_claim": False,
        "account_setup_claim": False,
        "permission_system_claim": False,
        "result_replay_claim": False,
        "closed_by": "STUDIO_CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1",
        "based_on": "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1",
        "panel_manifest": "pack/studio_classroom_operations_panel_preview_v1/classroom_operations_panel_preview.detjson",
        "source_classroom_operations_triage": "pack/studio_classroom_operations_triage_v1/classroom_operations_triage.detjson",
        "source_teacher_feedback_surface_preview": "pack/studio_teacher_feedback_surface_preview_v1/teacher_feedback_surface_preview.detjson",
        "panel_row_count": 6,
        "all_panels_panel_preview_only": True,
        "all_panels_generated_now": False,
        "all_panels_write_claim": False,
        "primary_coordinate": "하-3",
        "support_coordinate": "마-3",
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "panel_rows_closed": 6,
        "panel_rows_total": 6,
        "panel_rows_percent": 100,
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "current_stage_closed": 3,
        "current_stage_total": 8,
        "current_stage_percent": 38,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
        "closure_tier": "닫힘-동작",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")
    if "solutions/seamgrim_ui_mvp/ui/studio_classroom_operations_panel_preview.js" not in contract.get("changed_product_files", []):
        fail("contract missing classroom operations UI module")
    if "tests/studio_classroom_operations_panel_preview_runner.mjs" not in contract.get("verified_by", []):
        fail("contract missing browser runner")

    panel = load_json(PANEL)
    if panel.get("schema") != "ddn.studio.classroom_operations_panel_preview.v1":
        fail(f"panel schema mismatch: {panel.get('schema')!r}")
    if panel.get("product_ui_change") is not True:
        fail("panel must record product_ui_change=true")
    for flag in [
        "runtime_claim",
        "classroom_operations_runtime_claim",
        "teacher_feedback_runtime_claim",
        "student_data_collection_claim",
        "panel_write_claim",
        "triage_write_claim",
        "feedback_write_claim",
        "remote_save_claim",
        "cloud_sync_claim",
        "account_setup_claim",
        "permission_system_claim",
        "result_replay_claim",
    ]:
        if panel.get(flag) is not False:
            fail(f"panel {flag} expected false, got {panel.get(flag)!r}")
    rows = panel.get("panel_rows", [])
    if len(rows) != 6:
        fail(f"panel row count mismatch: {len(rows)}")
    for row in rows:
        if row.get("panel_preview_only") is not True:
            fail(f"row panel_preview_only mismatch: {row}")
        if row.get("generated_now") is not False:
            fail(f"row generated_now mismatch: {row}")
        if row.get("write_claim") is not False:
            fail(f"row write_claim mismatch: {row}")
        if row.get("product_ui_change") is not True:
            fail(f"row product_ui_change mismatch: {row}")
    progress = panel.get("progress", {})
    expected_progress = {
        "super_long_behavior_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "current_stage_closed": 3,
        "current_stage_total": 8,
        "current_stage_percent": 38,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
    }
    for key, value in expected_progress.items():
        if progress.get(key) != value:
            fail(f"panel progress {key} expected {value!r}, got {progress.get(key)!r}")

    triage = load_json(SOURCE_TRIAGE)
    if triage.get("schema") != "ddn.studio.classroom_operations_triage.v1":
        fail(f"source triage schema mismatch: {triage.get('schema')!r}")
    triage_ids = [str(row.get("id", "")) for row in triage.get("triage_rows", [])]
    expected_ids = [str(row.get("source_triage_row", "")) for row in rows]
    if triage_ids != expected_ids:
        fail(f"source triage row mismatch: {triage_ids!r}")
    preview = load_json(SOURCE_PREVIEW)
    if preview.get("schema") != "ddn.studio.teacher_feedback_surface_preview.v1":
        fail(f"source preview schema mismatch: {preview.get('schema')!r}")
    if preview.get("product_ui_change") is not True:
        fail("source preview must be product UI closed")
    if preview.get("progress", {}).get("roadmap_v2_behavior_closed") != 90:
        fail(f"source preview roadmap closed mismatch: {preview.get('progress')!r}")
    if preview.get("progress", {}).get("roadmap_v2_percent") != 100:
        fail(f"source preview roadmap percent mismatch: {preview.get('progress')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1",
        "studio classroom operations panel preview behavior sealed",
        "classroom operations panel preview schema: ddn.studio.classroom_operations_panel_preview.v1",
        "panel rows: 6/6 = 100%",
        "overall super-long behavior: 18/18 = 100%",
        "current stage: 3/8 = 38%",
        "roadmap v2 behavior: 90/90 = 100%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_classroom_operations_panel_preview_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_classroom_operations_panel_preview_v1"],
        ["node", "tests/studio_classroom_operations_panel_preview_runner.mjs"],
        ["python", "tests/run_studio_teacher_feedback_surface_preview_check.py"],
    ]:
        proc = run(cmd, timeout=900)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_source_tokens()
    check_contract_and_panel()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_classroom_operations_panel_preview_check: ok")


if __name__ == "__main__":
    main()
