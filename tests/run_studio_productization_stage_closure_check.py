from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_PRODUCTIZATION_STAGE_CLOSURE_V1.md"
ROADMAP = ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
REPORT = ROOT / "docs" / "studio" / "PRODUCTIZATION_STAGE_CLOSURE_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "studio_productization_stage_closure_v1"
CONTRACT = PACK / "contract.detjson"
CLOSURE = PACK / "productization_stage_closure.detjson"
SOURCE_REBASE = ROOT / "pack" / "studio_productization_stage_rebase_v1" / "productization_stage_rebase.detjson"
SOURCE_RESULT = ROOT / "pack" / "studio_numeric_result_report_consolidation_v1" / "numeric_result_report_stage.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_productization_stage_closure.js"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
RUNNER = ROOT / "tests" / "studio_productization_stage_closure_runner.mjs"
SOURCE_CHECK = ROOT / "tests" / "run_studio_numeric_result_report_consolidation_check.py"
NEXT = "STUDIO_POST_SUPER_LONG_REBASE_V1"


def fail(message: str) -> None:
    print(f"studio_productization_stage_closure_check: FAIL: {message}", file=sys.stderr)
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
        "closure_surface": "local_studio_productization_stage_closure",
        "closure_stage_only": True,
        "generated_now": False,
        "release_approval_claim": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "public_upload_claim": False,
        "registry_publish_claim": False,
        "github_release_claim": False,
        "runtime_claim": False,
        "replay_claim": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "solver_implementation_change": False,
        "benchmark_execution_claim": False,
        "product_ui_change": True,
    }
    rows: list[tuple[str, str, str]] = [
        (
            "stage_rebase_anchor",
            "pack/studio_productization_stage_rebase_v1/productization_stage_rebase.detjson",
            "stage_anchor",
        ),
        (
            "numeric_track_anchor",
            "pack/seamgrim_numeric_track_consolidation_v1/numeric_track_consolidation.detjson",
            "numeric_track",
        ),
        (
            "report_workflow_anchor",
            "pack/studio_numeric_report_workflow_consolidation_v1/numeric_report_workflow_stage.detjson",
            "report_workflow",
        ),
        (
            "result_report_anchor",
            "pack/studio_numeric_result_report_consolidation_v1/numeric_result_report_stage.detjson",
            "result_report",
        ),
        (
            "post_super_long_handoff",
            NEXT,
            "next_handoff",
        ),
    ]
    return [
        {
            "id": row_id,
            "source_anchor": source_anchor,
            "closure_lane": lane,
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
        CLOSURE,
        SOURCE_REBASE,
        SOURCE_RESULT,
        UI_MODULE,
        APP_JS,
        DEV_SURFACES_JS,
        INDEX_HTML,
        STYLES_CSS,
        RUNNER,
        SOURCE_CHECK,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "STUDIO_PRODUCTIZATION_STAGE_CLOSURE_V1",
        "ddn.studio.productization_stage_closure.v1",
        "productization_stage_closure",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        "작업 단위: 6/6 = 100% (`닫힘-동작`)",
        "closure rows: 5/5 = 100%",
        "current stage closure stages: 6/6 = 100%",
        "전체 초장기 계획: 9/18 = 50%",
        "현재 스테이지: Studio productization rebase 5/5 = 100%",
        "ROADMAP_V2 matrix behavior baseline: 51/90 = 57%",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens)
    require_contains(REPORT, ["ddn.studio.productization_stage_closure.v1", "6/6 = 100%", "5/5 = 100%", "9/18 = 50%", "51/90 = 57%"])
    require_contains(
        ROADMAP,
        [
            "STUDIO_PRODUCTIZATION_STAGE_CLOSURE_V1",
            "ddn.studio.productization_stage_closure.v1",
            "전체 초장기 계획 9/18 = 50%",
            "Studio productization rebase 5/5 = 100%",
            "ROADMAP_V2 matrix behavior baseline 51/90 = 57%",
            NEXT,
        ],
    )
    require_contains(
        INDEX,
        [
            "STUDIO_PRODUCTIZATION_STAGE_CLOSURE_V1",
            "STUDIO_PRODUCTIZATION_STAGE_CLOSURE_V1.md",
            "pack/studio_productization_stage_closure_v1",
            "tests/run_studio_productization_stage_closure_check.py",
            "docs/studio/PRODUCTIZATION_STAGE_CLOSURE_V1.md",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_PRODUCTIZATION_STAGE_CLOSURE_V1",
            "studio_productization_stage_closure_runner.mjs",
            "closure rows: 5/5 = 100%",
            "전체 초장기 계획: 9/18 = 50%",
            "현재 스테이지: Studio productization rebase 5/5 = 100%",
            "ROADMAP_V2 matrix behavior baseline: 51/90 = 57%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_product_tokens() -> None:
    require_contains(
        UI_MODULE,
        [
            "ddn.studio.productization_stage_closure.v1",
            "buildProductizationStageClosure",
            "formatProductizationStageClosureText",
            "renderProductizationStageClosure",
            "stage_chain_closed: 5",
            "super_long_behavior_closed: 9",
            "current_stage_percent: 100",
            "roadmap_v2_behavior_closed: 51",
            "roadmap_v2_percent: 57",
            "release_execution_claim: false",
            "runtime_claim: false",
        ],
    )
    require_contains(
        DEV_SURFACES_JS,
        [
            "studio_productization_stage_closure.js",
            "__SEAMGRIM_PRODUCTIZATION_STAGE_CLOSURE__",
            "buildProductizationStageClosure",
        ],
    )
    require_contains(APP_JS, ["shouldEnableDevSurfaces", "./dev_surfaces.js"])
    require_contains(DEV_SURFACES_JS, ["productization-stage-closure", "elementId: \"productization-stage-closure\""])
    require_contains(STYLES_CSS, [".productization-stage-closure", ".productization-closure-btn.active"])
    require_contains(RUNNER, ["studio_productization_stage_closure: ok", "productization_stage_closed", "stage_chain\\t5/5"])


def check_contract_and_manifest() -> None:
    payload = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_productization_stage_closure_v1",
        "kind": "studio_productization_stage_closure",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "closure_stage_only": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "solver_implementation_change": False,
        "replay_claim": False,
        "release_approval_claim": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "public_upload_claim": False,
        "registry_publish_claim": False,
        "github_release_claim": False,
        "benchmark_execution_claim": False,
        "closed_by": "STUDIO_PRODUCTIZATION_STAGE_CLOSURE_V1",
        "based_on": "STUDIO_NUMERIC_RESULT_REPORT_CONSOLIDATION_V1",
        "closure_manifest": "pack/studio_productization_stage_closure_v1/productization_stage_closure.detjson",
        "source_numeric_result_report_stage": "pack/studio_numeric_result_report_consolidation_v1/numeric_result_report_stage.detjson",
        "browser_runner": "tests/studio_productization_stage_closure_runner.mjs",
        "primary_coordinate": "마-3",
        "support_coordinate": "타-3",
        "workflow_schema": "ddn.studio.productization_stage_closure.v1",
        "workflow_claim": "productization_stage_closure",
        "closure_row_count": 5,
        "stage_chain_closed": 5,
        "stage_chain_total": 5,
        "stage_count": 6,
        "ready_stage_count": 6,
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "super_long_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_behavior_closed": 51,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 57,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {payload.get(key)!r}")

    manifest = load_json(CLOSURE)
    if manifest.get("schema") != "ddn.studio.productization_stage_closure.v1":
        fail(f"manifest schema mismatch: {manifest.get('schema')!r}")
    if manifest.get("closure_rows") != expected_rows():
        fail(f"manifest rows mismatch: {manifest.get('closure_rows')!r}")
    if manifest.get("progress") != {
        "super_long_behavior_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "current_stage_closed": 5,
        "current_stage_total": 5,
        "current_stage_percent": 100,
        "roadmap_v2_behavior_closed": 51,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 57,
    }:
        fail(f"manifest progress mismatch: {manifest.get('progress')!r}")
    if manifest.get("next_item") != NEXT:
        fail(f"manifest next mismatch: {manifest.get('next_item')!r}")


def check_source_alignment() -> None:
    rebase = load_json(SOURCE_REBASE)
    result = load_json(SOURCE_RESULT)
    if rebase.get("schema") != "ddn.studio.productization_stage_rebase.v1":
        fail(f"source rebase schema mismatch: {rebase.get('schema')!r}")
    if rebase.get("progress", {}).get("current_stage_percent") != 20:
        fail(f"source rebase progress mismatch: {rebase.get('progress')!r}")
    if result.get("schema") != "ddn.studio.numeric_result_report_stage.v1":
        fail(f"source result schema mismatch: {result.get('schema')!r}")
    if result.get("next_item") != "STUDIO_PRODUCTIZATION_STAGE_CLOSURE_V1":
        fail(f"source result next mismatch: {result.get('next_item')!r}")
    if result.get("progress", {}).get("current_stage_percent") != 80:
        fail(f"source result progress mismatch: {result.get('progress')!r}")
    if result.get("progress", {}).get("roadmap_v2_behavior_closed") != 51:
        fail(f"source result roadmap closed mismatch: {result.get('progress')!r}")
    if result.get("progress", {}).get("roadmap_v2_percent") != 57:
        fail(f"source result roadmap progress mismatch: {result.get('progress')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_PRODUCTIZATION_STAGE_CLOSURE_V1",
        "studio productization stage closure sealed",
        "productization stage closure schema: ddn.studio.productization_stage_closure.v1",
        "current stage: 5/5 = 100%",
        "overall super-long behavior: 9/18 = 50%",
        "roadmap v2 behavior: 51/90 = 57%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_productization_stage_closure_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["node", "tests/studio_productization_stage_closure_runner.mjs"],
        ["python", "tests/run_pack_golden.py", "studio_productization_stage_closure_v1"],
        ["python", "tests/run_studio_numeric_result_report_consolidation_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_product_tokens()
    check_contract_and_manifest()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_productization_stage_closure_check: ok")


if __name__ == "__main__":
    main()
