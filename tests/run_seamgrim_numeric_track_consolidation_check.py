from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "queue" / "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1.md"
REPORT = ROOT / "docs" / "studio" / "NUMERIC_TRACK_CONSOLIDATION_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "seamgrim_numeric_track_consolidation_v1"
CONTRACT = PACK / "contract.detjson"
CONSOLIDATION = PACK / "numeric_track_consolidation.detjson"
SOURCE_REBASE = ROOT / "pack" / "studio_productization_stage_rebase_v1" / "productization_stage_rebase.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "seamgrim_numeric_track_consolidation.js"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
BROWSE_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "browse.js"
NUMERIC_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "numeric_curriculum_track.js"
RUNNER = ROOT / "tests" / "seamgrim_numeric_track_consolidation_runner.mjs"
REPORT_RUNNER = ROOT / "tests" / "studio_numeric_report_workflow_consolidation_runner.mjs"
RESULT_RUNNER = ROOT / "tests" / "studio_numeric_result_report_consolidation_runner.mjs"
EDU_CHECK = ROOT / "tests" / "run_seamgrim_education_curriculum_template_check.py"
SOURCE_REBASE_CHECK = ROOT / "tests" / "run_studio_productization_stage_rebase_check.py"
NEXT = "STUDIO_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_V1"


def fail(message: str) -> None:
    print(f"seamgrim_numeric_track_consolidation_check: FAIL: {message}", file=sys.stderr)
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
    lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def expected_rows() -> list[dict[str, object]]:
    common = {
        "consolidation_surface": "local_seamgrim_numeric_track_consolidation",
        "consolidation_only": True,
        "generated_now": False,
        "new_export_wrapper_claim": False,
        "new_long_runner_claim": False,
        "runtime_claim": False,
        "replay_claim": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "product_ui_change": True,
    }
    rows: list[tuple[str, str, str]] = [
        (
            "productization_rebase_anchor",
            "pack/studio_productization_stage_rebase_v1/productization_stage_rebase.detjson",
            "stage_handoff",
        ),
        (
            "numeric_report_workflow_gate",
            "tests/studio_numeric_report_workflow_consolidation_runner.mjs",
            "report_workflow",
        ),
        (
            "numeric_result_report_gate",
            "tests/studio_numeric_result_report_consolidation_runner.mjs",
            "result_report",
        ),
        (
            "legacy_runner_audit",
            "tests/seamgrim_numeric_track*_runner.mjs",
            "micro_slice_audit",
        ),
        (
            "browse_detail_dataset_guard",
            "solutions/seamgrim_ui_mvp/ui/screens/browse.js",
            "baseline_guard",
        ),
    ]
    return [
        {
            "id": row_id,
            "source_anchor": source_anchor,
            "consolidation_lane": lane,
            **common,
        }
        for row_id, source_anchor, lane in rows
    ]


def check_required_files() -> None:
    for path in [
        DOC,
        REPORT,
        INDEX,
        DEV_SUMMARY,
        PACK / "README.md",
        CONTRACT,
        CONSOLIDATION,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        SOURCE_REBASE,
        UI_MODULE,
        APP_JS,
        DEV_SURFACES_JS,
        INDEX_HTML,
        STYLES_CSS,
        BROWSE_JS,
        NUMERIC_MODULE,
        RUNNER,
        REPORT_RUNNER,
        RESULT_RUNNER,
        EDU_CHECK,
        SOURCE_REBASE_CHECK,
    ]:
        require(path)


def check_runner_audit() -> None:
    runners = sorted((ROOT / "tests").glob("seamgrim_numeric_track*_runner.mjs"))
    over_60 = [path for path in runners if len(path.name) > 60]
    over_100 = [path for path in runners if len(path.name) > 100]
    if len(runners) != 29:
        fail(f"legacy numeric runner count expected 29 including consolidation UI runner, got {len(runners)}")
    legacy = [path for path in runners if path.name != "seamgrim_numeric_track_consolidation_runner.mjs"]
    legacy_over_60 = [path for path in legacy if len(path.name) > 60]
    legacy_over_100 = [path for path in legacy if len(path.name) > 100]
    if len(legacy) != 28:
        fail(f"legacy numeric runner count expected 28, got {len(legacy)}")
    if len(legacy_over_60) != 16:
        fail(f"legacy numeric runner >60 count expected 16, got {len(legacy_over_60)}")
    if len(legacy_over_100) != 2:
        fail(f"legacy numeric runner >100 count expected 2, got {len(legacy_over_100)}")
    if len(over_60) != 16:
        fail(f"all numeric runner >60 count expected 16, got {len(over_60)}")
    if len(over_100) != 2:
        fail(f"all numeric runner >100 count expected 2, got {len(over_100)}")


def check_docs() -> None:
    tokens = [
        "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "seamgrim.numeric_track_consolidation.v1",
        "seamgrim.numeric_report_workflow_consolidation.v1",
        "seamgrim.numeric_result_report_consolidation.v1",
        "28 browser runners",
        "16 runner filenames longer than 60 characters",
        "2 longer than 100 characters",
        "deferred micro-slice candidates: 1/1 = 100% recorded",
        "SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_SUMMARY_EXPORT_V1",
        "baseline repair: 1/1 = 100%",
        "numeric consolidated gates: 2/2 = 100%",
        "consolidation rows: 5/5 = 100%",
        "전체 초장기 계획: 9/18 = 50%",
        "현재 스테이지: Studio productization rebase 2/5 = 40%",
        "ROADMAP_V2 matrix behavior baseline: 51/90 = 57%",
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens)
    require_contains(REPORT, ["28 total", "16/28 = 57%", "2/28 = 7%", "1/1 = 100% recorded", "5/5 = 100%", "9/18 = 50%", "2/5 = 40%", "51/90 = 57%"])
    require_contains(
        INDEX,
        [
            "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
            "docs/studio/NUMERIC_TRACK_CONSOLIDATION_V1.md",
            "pack/seamgrim_numeric_track_consolidation_v1",
            "tests/run_seamgrim_numeric_track_consolidation_check.py",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
            "seamgrim_numeric_track_consolidation_runner.mjs",
            "consolidation rows: 5/5 = 100%",
            "전체 초장기 계획: 9/18 = 50%",
            "현재 스테이지: Studio productization rebase 2/5 = 40%",
            "ROADMAP_V2 matrix behavior baseline: 51/90 = 57%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_product_tokens() -> None:
    require_contains(
        BROWSE_JS,
        [
            "setElementDatasetValue",
            'setElementDatasetValue(this.detailPanelEl, "numericTrack"',
            'setElementDatasetValue(this.detailPanelEl, "numericTrackReopen"',
            'setElementDatasetValue(this.detailPanelEl, "reopenLessonId"',
        ],
    )
    require_contains(
        NUMERIC_MODULE,
        [
            "buildNumericReportWorkflowConsolidation",
            "buildNumericResultReportConsolidation",
            "seamgrim.numeric_report_workflow_consolidation.v1",
            "seamgrim.numeric_result_report_consolidation.v1",
        ],
    )
    require_contains(
        UI_MODULE,
        [
            "seamgrim.numeric_track_consolidation.v1",
            "buildSeamgrimNumericTrackConsolidation",
            "formatSeamgrimNumericTrackConsolidationText",
            "renderSeamgrimNumericTrackConsolidation",
            "legacy_numeric_runner_count: 28",
            "deferred_micro_slice_count: 1",
            "micro_slice_wrapper_name_over_60",
            "new_long_runner_claim: false",
            "super_long_behavior_closed: 9",
            "super_long_percent: 50",
            "current_stage_percent: 40",
            "roadmap_v2_behavior_closed: 51",
            "roadmap_v2_percent: 57",
        ],
    )
    require_contains(
        DEV_SURFACES_JS,
        [
            "seamgrim_numeric_track_consolidation.js",
            "__SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION__",
            "buildSeamgrimNumericTrackConsolidation",
        ],
    )
    require_contains(DEV_SURFACES_JS, ["seamgrim-numeric-track-consolidation", "elementId: \"seamgrim-numeric-track-consolidation\""])
    require_contains(APP_JS, ["shouldEnableDevSurfaces", "./dev_surfaces.js"])
    require_contains(STYLES_CSS, [".seamgrim-numeric-track-consolidation", ".numeric-consolidation-btn.active"])
    require_contains(RUNNER, ["seamgrim_numeric_track_consolidation: ok", "numeric_track_consolidated", "new_long_runner_claim"])


def check_contract_and_manifest() -> None:
    payload = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_numeric_track_consolidation_v1",
        "kind": "seamgrim_numeric_track_consolidation",
        "product_code_change": True,
        "product_ui_change": True,
        "runtime_claim": False,
        "parser_frontdoor_change": False,
        "stdlib_change": False,
        "solver_implementation_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "replay_claim": False,
        "closed_by": "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "based_on": "STUDIO_PRODUCTIZATION_STAGE_REBASE_V1",
        "track_id": "studio_numeric_curriculum_track_v1",
        "legacy_numeric_runner_count": 28,
        "legacy_numeric_runner_over_60": 16,
        "legacy_numeric_runner_over_100": 2,
        "deferred_micro_slice_candidate": "SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_SUMMARY_EXPORT_V1",
        "deferred_micro_slice_reason": "micro_slice_wrapper_name_over_60",
        "deferred_micro_slice_candidate_name_length": 108,
        "deferred_micro_slice_runner_name_length": 118,
        "fold_deferred_micro_slice_into_existing_consolidation": True,
        "deferred_micro_slice_count": 1,
        "baseline_repair_closed": 1,
        "baseline_repair_total": 1,
        "consolidated_gate_closed": 2,
        "consolidated_gate_total": 2,
        "consolidation_row_count": 5,
        "all_rows_consolidation_only": True,
        "all_rows_generated_now": False,
        "new_export_wrapper_claim": False,
        "new_long_runner_claim": False,
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "super_long_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "current_stage_closed": 2,
        "current_stage_total": 5,
        "current_stage_percent": 40,
        "roadmap_v2_behavior_closed": 51,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 57,
        "closure_tier": "닫힘-동작",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {payload.get(key)!r}")

    manifest = load_json(CONSOLIDATION)
    if manifest.get("schema") != "seamgrim.numeric_track_consolidation.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("work_item") != "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1":
        fail(f"manifest work item mismatch: {manifest.get('work_item')!r}")
    if manifest.get("selected_next_item") != NEXT:
        fail(f"manifest selected next mismatch: {manifest.get('selected_next_item')!r}")
    for flag, expected_value in (
        ("runtime_claim", False),
        ("product_code_change", True),
        ("product_ui_change", True),
        ("consolidation_only", True),
        ("replay_claim", False),
        ("new_export_wrapper_claim", False),
        ("new_long_runner_claim", False),
        ("lesson_schema_change", False),
        ("active_allowlist_mutation", False),
        ("parser_frontdoor_change", False),
    ):
        if manifest.get(flag) is not expected_value:
            fail(f"manifest {flag} expected {expected_value!r}, got {manifest.get(flag)!r}")
    if manifest.get("legacy_numeric_runner_count") != 28:
        fail(f"manifest legacy runner count mismatch: {manifest.get('legacy_numeric_runner_count')!r}")
    if manifest.get("deferred_micro_slice_candidate") != {
        "id": "SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_SUMMARY_EXPORT_V1",
        "reason": "micro_slice_wrapper_name_over_60",
        "name_length": 108,
        "runner_name_length": 118,
        "fold_into_existing_consolidation": True,
    }:
        fail(f"manifest deferred micro-slice candidate mismatch: {manifest.get('deferred_micro_slice_candidate')!r}")
    if manifest.get("deferred_micro_slice_count") != 1:
        fail(f"manifest deferred micro-slice count mismatch: {manifest.get('deferred_micro_slice_count')!r}")
    if manifest.get("consolidation_rows") != expected_rows():
        fail(f"manifest rows mismatch: {manifest.get('consolidation_rows')!r}")
    if manifest.get("progress") != {
        "super_long_behavior_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "current_stage_closed": 2,
        "current_stage_total": 5,
        "current_stage_percent": 40,
        "roadmap_v2_behavior_closed": 51,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 57,
    }:
        fail(f"manifest progress mismatch: {manifest.get('progress')!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"manifest next mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    source = load_json(SOURCE_REBASE)
    if source.get("schema") != "ddn.studio.productization_stage_rebase.v1":
        fail(f"source rebase schema mismatch: {source.get('schema')!r}")
    if source.get("next_item") != "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1":
        fail(f"source rebase next item mismatch: {source.get('next_item')!r}")
    if source.get("progress", {}).get("current_stage_percent") != 20:
        fail(f"source rebase progress mismatch: {source.get('progress')!r}")
    if source.get("progress", {}).get("roadmap_v2_behavior_closed") != 51:
        fail(f"source rebase ROADMAP closed mismatch: {source.get('progress')!r}")
    if source.get("progress", {}).get("roadmap_v2_percent") != 57:
        fail(f"source rebase ROADMAP percent mismatch: {source.get('progress')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_V1",
        "seamgrim numeric track consolidation sealed",
        "preferred gates: numeric_report_workflow + numeric_result_report",
        "legacy numeric runners: 28 total, 16 over 60 chars, 2 over 100 chars",
        "deferred micro-slice: SEAMGRIM_NUMERIC_TRACK_RESULT_COMPARE_HISTORY_REPORT_TABLE_STATUS_BADGE_A11Y_STATUS_EXPORT_SUMMARY_EXPORT_V1 folded into consolidation",
        "baseline repair: browse detail dataset guard",
        "current stage: 2/5 = 40%",
        "overall super-long behavior: 9/18 = 50%",
        "roadmap v2 behavior: 51/90 = 57%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/seamgrim_numeric_track_consolidation_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_seamgrim_education_curriculum_template_check.py"],
        ["node", "tests/seamgrim_numeric_track_consolidation_runner.mjs"],
        ["node", "tests/studio_numeric_report_workflow_consolidation_runner.mjs"],
        ["node", "tests/studio_numeric_result_report_consolidation_runner.mjs"],
        ["python", "tests/run_pack_golden.py", "seamgrim_numeric_track_consolidation_v1"],
        ["python", "tests/run_studio_productization_stage_rebase_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_runner_audit()
    check_docs()
    check_product_tokens()
    check_contract_and_manifest()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("seamgrim_numeric_track_consolidation_check: ok")


if __name__ == "__main__":
    main()
