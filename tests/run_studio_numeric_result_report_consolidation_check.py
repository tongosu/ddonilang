from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1.md"
ROADMAP = ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
REPORT = ROOT / "docs" / "studio" / "NUMERIC_RESULT_REPORT_CONSOLIDATION_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "studio_numeric_result_report_consolidation_v1"
CONTRACT = PACK / "contract.detjson"
STAGE = PACK / "numeric_result_report_stage.detjson"
SOURCE_STAGE = ROOT / "pack" / "studio_numeric_report_workflow_consolidation_v1" / "numeric_report_workflow_stage.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_numeric_result_report_stage.js"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
NUMERIC_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "numeric_curriculum_track.js"
STAGE_RUNNER = ROOT / "tests" / "studio_numeric_result_stage_runner.mjs"
WORKFLOW_RUNNER = ROOT / "tests" / "studio_numeric_result_report_consolidation_runner.mjs"
SOURCE_CHECK = ROOT / "tests" / "run_studio_numeric_report_workflow_consolidation_check.py"
NEXT = "STUDIO_PRODUCTIZATION_STAGE_CLOSURE_V1"


def fail(message: str) -> None:
    print(f"studio_numeric_result_report_consolidation_check: FAIL: {message}", file=sys.stderr)
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
        "workflow_surface": "local_studio_numeric_result_report_stage",
        "result_report_stage_only": True,
        "generated_now": False,
        "new_export_wrapper_claim": False,
        "replay_claim": False,
        "runtime_claim": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "solver_implementation_change": False,
        "product_ui_change": True,
    }
    rows: list[tuple[str, str, str]] = [
        (
            "report_workflow_anchor",
            "pack/studio_numeric_report_workflow_consolidation_v1/numeric_report_workflow_stage.detjson",
            "stage_handoff",
        ),
        (
            "result_report_gate",
            "tests/studio_numeric_result_report_consolidation_runner.mjs",
            "result_report_gate",
        ),
        (
            "evidence_pack_rollup",
            "pack/studio_numeric_result_report_consolidation_v1/contract.detjson",
            "evidence_rollup",
        ),
        (
            "copy_text_export_gate",
            "formatNumericResultReportConsolidationText",
            "text_export",
        ),
        (
            "final_stage_handoff",
            NEXT,
            "next_handoff",
        ),
    ]
    return [
        {
            "id": row_id,
            "source_anchor": source_anchor,
            "workflow_lane": lane,
            **common,
        }
        for row_id, source_anchor, lane in rows
    ]


def check_required_files() -> None:
    for path in [
        DOC,
        ROADMAP,
        REPORT,
        INDEX,
        DEV_SUMMARY,
        PACK / "README.md",
        CONTRACT,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        STAGE,
        SOURCE_STAGE,
        UI_MODULE,
        APP_JS,
        DEV_SURFACES_JS,
        INDEX_HTML,
        STYLES_CSS,
        NUMERIC_MODULE,
        STAGE_RUNNER,
        WORKFLOW_RUNNER,
        SOURCE_CHECK,
    ]:
        require(path)


def check_docs() -> None:
    doc_tokens = [
        "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1",
        "ddn.studio.numeric_result_report_stage.v1",
        "seamgrim.numeric_result_report_consolidation.v1",
        "numeric_result_report_consolidation",
        "Primary coordinate: `마-3`",
        "Support coordinate: `다-2`",
        "5/5 result rows",
        "작업 단위: 6/6 = 100% (`닫힘-동작`)",
        "result rows: 5/5 = 100%",
        "result report stages: 10/10 = 100%",
        "report workflow stages: 17/17 = 100%",
        "전체 초장기 계획: 9/18 = 50%",
        "현재 스테이지: Studio productization rebase 4/5 = 80%",
        "ROADMAP_V2 matrix behavior baseline: 51/90 = 57%",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, doc_tokens)
    require_contains(REPORT, ["ddn.studio.numeric_result_report_stage.v1", "6/6 = 100%", "5/5 = 100%", "9/18 = 50%", "4/5 = 80%", "51/90 = 57%"])
    require_contains(
        ROADMAP,
        [
            "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1",
            "ddn.studio.numeric_result_report_stage.v1",
            "전체 초장기 계획 9/18 = 50%",
            "Studio productization rebase 4/5 = 80%",
            "ROADMAP_V2 matrix behavior baseline 51/90 = 57%",
            NEXT,
        ],
    )
    require_contains(
        INDEX,
        [
            "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1",
            "docs/studio/NUMERIC_RESULT_REPORT_CONSOLIDATION_V1.md",
            "pack/studio_numeric_result_report_consolidation_v1",
            "tests/run_studio_numeric_result_report_consolidation_check.py",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1",
            "studio_numeric_result_stage_runner.mjs",
            "result rows: 5/5 = 100%",
            "전체 초장기 계획: 9/18 = 50%",
            "현재 스테이지: Studio productization rebase 4/5 = 80%",
            "ROADMAP_V2 matrix behavior baseline: 51/90 = 57%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_product_tokens() -> None:
    require_contains(
        NUMERIC_MODULE,
        [
            "buildNumericResultReportConsolidation",
            "formatNumericResultReportConsolidationText",
            "seamgrim.numeric_result_report_consolidation.v1",
            "numeric_result_report_consolidation",
            "numeric_result_report_ready",
            "support_coordinate",
            "다-2",
            "report_workflow_stage_count",
            "runtime_claim",
        ],
    )
    require_contains(
        UI_MODULE,
        [
            "ddn.studio.numeric_result_report_stage.v1",
            "buildNumericResultReportStage",
            "formatNumericResultReportStageText",
            "renderNumericResultReportStage",
            "result_report_stage_count: 10",
            "report_workflow_stage_count: 17",
            "new_export_wrapper_claim: false",
            "super_long_behavior_closed: 9",
            "super_long_percent: 50",
            "current_stage_percent: 80",
            "roadmap_v2_behavior_closed: 51",
            "roadmap_v2_percent: 57",
        ],
    )
    require_contains(
        DEV_SURFACES_JS,
        [
            "studio_numeric_result_report_stage.js",
            "__SEAMGRIM_NUMERIC_RESULT_REPORT_STAGE__",
            "buildNumericResultReportStage",
        ],
    )
    require_contains(DEV_SURFACES_JS, ["numeric-result-report-stage", "elementId: \"numeric-result-report-stage\""])
    require_contains(APP_JS, ["shouldEnableDevSurfaces", "./dev_surfaces.js"])
    require_contains(STYLES_CSS, [".numeric-result-report-stage", ".numeric-result-stage-btn.active"])
    require_contains(STAGE_RUNNER, ["studio_numeric_result_stage: ok", "numeric_result_report_stage_ready", "result_report_stage_count"])
    require_contains(
        WORKFLOW_RUNNER,
        [
            "studio_numeric_result_report_consolidation: ok",
            "seamgrim.numeric_result_report_consolidation.v1",
            "numeric_result_report_consolidation",
            "numeric_result_report_ready",
            "support_coordinate\\t다-2",
            "report_workflow_stage_count\\t17",
        ],
    )


def check_contract_and_stage() -> None:
    payload = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_numeric_result_report_consolidation_v1",
        "kind": "studio_numeric_result_report_consolidation",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "solver_implementation_change": False,
        "replay_claim": False,
        "closed_by": "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1",
        "based_on": "STUDIO_NUMERIC_REPORT_WORKFLOW_CONSOLIDATION_V1",
        "stage_manifest": "pack/studio_numeric_result_report_consolidation_v1/numeric_result_report_stage.detjson",
        "source_report_workflow_stage": "pack/studio_numeric_report_workflow_consolidation_v1/numeric_report_workflow_stage.detjson",
        "browser_runner": "tests/studio_numeric_result_report_consolidation_runner.mjs",
        "stage_browser_runner": "tests/studio_numeric_result_stage_runner.mjs",
        "track_id": "studio_numeric_curriculum_track_v1",
        "workflow_schema": "seamgrim.numeric_result_report_consolidation.v1",
        "workflow_claim": "numeric_result_report_consolidation",
        "primary_coordinate": "마-3",
        "support_coordinate": "다-2",
        "stage_count": 10,
        "ready_stage_count": 10,
        "result_count": 3,
        "pair_count": 2,
        "evidence_pack_count": 3,
        "report_workflow_schema": "seamgrim.numeric_report_workflow_consolidation.v1",
        "report_workflow_stage_count": 17,
        "report_workflow_ready_stage_count": 17,
        "result_row_count": 5,
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "super_long_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "current_stage_closed": 4,
        "current_stage_total": 5,
        "current_stage_percent": 80,
        "roadmap_v2_behavior_closed": 51,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 57,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {payload.get(key)!r}")

    stage = load_json(STAGE)
    if stage.get("schema") != "ddn.studio.numeric_result_report_stage.v1":
        fail(f"stage schema mismatch: {stage.get('schema')!r}")
    if stage.get("work_item") != "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1":
        fail(f"stage work item mismatch: {stage.get('work_item')!r}")
    if stage.get("result_rows") != expected_rows():
        fail(f"stage rows mismatch: {stage.get('result_rows')!r}")
    if stage.get("progress") != {
        "super_long_behavior_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "current_stage_closed": 4,
        "current_stage_total": 5,
        "current_stage_percent": 80,
        "roadmap_v2_behavior_closed": 51,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 57,
    }:
        fail(f"stage progress mismatch: {stage.get('progress')!r}")
    for flag, expected_value in (
        ("runtime_claim", False),
        ("product_code_change", True),
        ("product_ui_change", True),
        ("result_report_stage_only", True),
        ("replay_claim", False),
        ("new_export_wrapper_claim", False),
        ("lesson_schema_change", False),
        ("active_allowlist_mutation", False),
        ("parser_frontdoor_change", False),
        ("solver_implementation_change", False),
    ):
        if stage.get(flag) is not expected_value:
            fail(f"stage {flag} expected {expected_value!r}, got {stage.get(flag)!r}")
    if stage.get("next_item") != NEXT:
        fail(f"stage next mismatch: {stage.get('next_item')!r}")


def check_source_alignment() -> None:
    source = load_json(SOURCE_STAGE)
    if source.get("schema") != "ddn.studio.numeric_report_workflow_stage.v1":
        fail(f"source schema mismatch: {source.get('schema')!r}")
    if source.get("next_item") != "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1":
        fail(f"source next item mismatch: {source.get('next_item')!r}")
    if source.get("progress", {}).get("current_stage_percent") != 60:
        fail(f"source progress mismatch: {source.get('progress')!r}")
    if source.get("progress", {}).get("roadmap_v2_behavior_closed") != 51:
        fail(f"source roadmap closed mismatch: {source.get('progress')!r}")
    if source.get("progress", {}).get("roadmap_v2_percent") != 57:
        fail(f"source roadmap progress mismatch: {source.get('progress')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1",
        "studio numeric result report consolidation sealed",
        "numeric result report consolidation schema: seamgrim.numeric_result_report_consolidation.v1",
        "coordinate: 마-3 + 다-2 evidence anchor",
        "current stage: 4/5 = 80%",
        "overall super-long behavior: 9/18 = 50%",
        "roadmap v2 behavior: 51/90 = 57%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_numeric_result_report_consolidation_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["node", "tests/studio_numeric_result_stage_runner.mjs"],
        ["node", "tests/studio_numeric_result_report_consolidation_runner.mjs"],
        ["python", "tests/run_pack_golden.py", "studio_numeric_result_report_consolidation_v1"],
        ["python", "tests/run_studio_numeric_report_workflow_consolidation_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_product_tokens()
    check_contract_and_stage()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_numeric_result_report_consolidation_check: ok")


if __name__ == "__main__":
    main()
