from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1.md"
ROADMAP = ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
REPORT = ROOT / "docs" / "studio" / "BENCHMARK_BASELINE_PREP_DRY_RUN_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_benchmark_baseline_prep_dry_run_v1"
DRY_RUN = PACK / "benchmark_baseline_prep_dry_run.detjson"
CONTRACT = PACK / "contract.detjson"
INPUT = PACK / "input.ddn"
GOLDEN = PACK / "golden.jsonl"
UI = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_benchmark_baseline_prep_dry_run.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
RUNNER = ROOT / "tests" / "studio_benchmark_baseline_prep_dry_run_runner.mjs"
SOURCE_MATRIX = ROOT / "pack" / "studio_benchmark_lts_matrix_v1" / "benchmark_lts_matrix.detjson"
SOURCE_TRIAGE = ROOT / "pack" / "studio_classroom_operations_triage_v1" / "classroom_operations_triage.detjson"
SOURCE_REBASE = ROOT / "pack" / "roadmap_v2_studio_productization_rebase_v1" / "rebase.detjson"
NEXT = "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1"


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(path: Path) -> dict:
    return json.loads(read_text(path))


def assert_true(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def assert_contains(text: str, token: str, label: str) -> None:
    assert_true(token in text, f"{label} missing token: {token}")


def expected_inputs() -> list[dict[str, str]]:
    return [
        {
            "id": "benchmark_lts_matrix_input",
            "source_anchor": "benchmark_lts_matrix",
            "baseline_lane": "benchmark_lts_matrix",
            "planned_path": "build/studio_benchmark/baseline/benchmark_lts_matrix.detjson",
            "prep_only": True,
            "generated_now": False,
            "benchmark_execution_claim": False,
            "performance_baseline_generation_claim": False,
            "performance_baseline_publication_claim": False,
        },
        {
            "id": "classroom_operations_triage_input",
            "source_anchor": "classroom_operations_triage",
            "baseline_lane": "classroom_operations",
            "planned_path": "build/studio_benchmark/baseline/classroom_operations_triage.detjson",
            "prep_only": True,
            "generated_now": False,
            "benchmark_execution_claim": False,
            "performance_baseline_generation_claim": False,
            "performance_baseline_publication_claim": False,
        },
        {
            "id": "browser_smoke_matrix_input",
            "source_anchor": "benchmark_lts_matrix",
            "baseline_lane": "browser_smoke_matrix",
            "planned_path": "build/studio_benchmark/baseline/browser_smoke_matrix.detjson",
            "prep_only": True,
            "generated_now": False,
            "benchmark_execution_claim": False,
            "performance_baseline_generation_claim": False,
            "performance_baseline_publication_claim": False,
        },
        {
            "id": "local_packaging_input",
            "source_anchor": "benchmark_lts_matrix",
            "baseline_lane": "local_packaging",
            "planned_path": "build/studio_benchmark/baseline/local_packaging.detjson",
            "prep_only": True,
            "generated_now": False,
            "benchmark_execution_claim": False,
            "performance_baseline_generation_claim": False,
            "performance_baseline_publication_claim": False,
        },
        {
            "id": "approval_continuity_input",
            "source_anchor": "benchmark_lts_matrix",
            "baseline_lane": "approval_continuity",
            "planned_path": "build/studio_benchmark/baseline/approval_continuity.detjson",
            "prep_only": True,
            "generated_now": False,
            "benchmark_execution_claim": False,
            "performance_baseline_generation_claim": False,
            "performance_baseline_publication_claim": False,
        },
    ]


def check_required_files() -> None:
    for path in [
        DOC,
        ROADMAP,
        DEV_SUMMARY,
        REPORT,
        INDEX,
        DRY_RUN,
        CONTRACT,
        INPUT,
        GOLDEN,
        UI,
        APP,
        DEV_SURFACES,
        HTML,
        STYLES,
        RUNNER,
        SOURCE_MATRIX,
        SOURCE_TRIAGE,
        SOURCE_REBASE,
    ]:
        assert_true(path.exists(), f"missing required file: {path.relative_to(ROOT)}")


def check_docs() -> None:
    doc = read_text(DOC)
    for token in [
        "## Planning Evidence",
        "Every row keeps `prep_only=true`, `generated_now=false`, and `benchmark_execution_claim=false`.",
        "작업 단위: 6/6 = 100% (`닫힘-문서`)",
        "초장기 계획 닫힘-동작: 1시대 5/5 = 100%, 전체 5/18 = 28%",
        "현재 스테이지 닫힘-문서: post-super-long follow-up 7/8 = 88%",
        "ROADMAP_V2 product behavior baseline: 21/90 = 23%",
        "ROADMAP_V2 닫힘-문서 참고값: 72/90 = 80%",
        "tests/run_studio_baseline_reassessment_progress_unlock_check.py",
        "node tests/studio_benchmark_baseline_prep_dry_run_runner.mjs",
        NEXT,
    ]:
        assert_contains(doc, token, "root doc")

    report = read_text(REPORT)
    for token in [
        "This is local benchmark baseline preparation dry-run planning evidence",
        "작업 단위: 6/6 = 100% (`닫힘-문서`)",
        "작업 단위: 6/6 = 100%",
        "초장기 계획 닫힘-동작: 1시대 5/5 = 100%, 전체 5/18 = 28%",
        "현재 스테이지 닫힘-문서: post-super-long follow-up 7/8 = 88%",
        "ROADMAP_V2 product behavior baseline: 21/90 = 23%",
    ]:
        assert_contains(report, token, "studio report")

    index = read_text(INDEX)
    assert_contains(index, "BENCHMARK_BASELINE_PREP_DRY_RUN_V1", "studio index")

    roadmap = read_text(ROADMAP)
    for token in [
        "STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1",
        "전체 초장기 계획 닫힘-동작 5/18 = 28%",
        "post-super-long follow-up 닫힘-문서 7/8 = 88%",
        "ROADMAP_V2 product behavior baseline 21/90 = 23%",
    ]:
        assert_contains(roadmap, token, "long horizon roadmap")

    summary = read_text(DEV_SUMMARY)
    for token in [
        "[STUDIO][BENCHMARK] Benchmark baseline prep dry-run alignment",
        "node tests/studio_benchmark_baseline_prep_dry_run_runner.mjs` PASS",
        "현재 스테이지: post-super-long follow-up 7/8 = 88%",
        "ROADMAP_V2 product behavior baseline: 21/90 = 23%",
        "git status --short -- docs/ssot` PASS",
    ]:
        assert_contains(summary, token, "DEV_SUMMARY")


def check_ui_contract() -> None:
    ui = read_text(UI)
    for token in [
        "DEFAULT_BENCHMARK_BASELINE_PREP_INPUT_ROWS",
        "buildBenchmarkBaselinePrepDryRun",
        "formatBenchmarkBaselinePrepDryRunText",
        "renderBenchmarkBaselinePrepDryRun",
        "ddn.studio.benchmark_baseline_prep_dry_run.v1",
        "benchmark_baseline_prep_ready",
        "roadmap_v2_behavior_closed: 21",
        "roadmap_v2_percent: 23",
        "current_stage_closed: 7",
        "current_stage_percent: 88",
        "benchmark_execution_claim: false",
        "performance_baseline_generation_claim: false",
        "lts_certification_claim: false",
        NEXT,
    ]:
        assert_contains(ui, token, "UI module")

    app = read_text(APP)
    for token in [
        "shouldEnableDevSurfaces",
        "./dev_surfaces.js",
    ]:
        assert_contains(app, token, "app")

    dev_surfaces = read_text(DEV_SURFACES)
    for token in [
        "buildBenchmarkBaselinePrepDryRun",
        "formatBenchmarkBaselinePrepDryRunText",
        "renderBenchmarkBaselinePrepDryRun",
        "benchmark-baseline-prep-dry-run",
        "__SEAMGRIM_BENCHMARK_BASELINE_PREP_DRY_RUN__",
        "__SEAMGRIM_BENCHMARK_BASELINE_PREP_DRY_RUN_TEXT__",
    ]:
        assert_contains(dev_surfaces, token, "dev_surfaces")

    styles = read_text(DEV_SURFACES_CSS)
    for token in [
        ".benchmark-baseline-prep-dry-run",
        ".benchmark-prep-head",
        ".benchmark-prep-btn",
        ".benchmark-prep-detail",
        ".benchmark-prep-progress",
    ]:
        assert_contains(styles, token, "dev_surfaces_css")

    runner = read_text(RUNNER)
    for token in [
        "studio_benchmark_baseline_prep_dry_run: ok",
        "buildBenchmarkBaselinePrepDryRun",
        "formatBenchmarkBaselinePrepDryRunText",
        "data-benchmark-baseline-prep-dry-run-status='benchmark_baseline_prep_ready'",
        "benchmark_baseline_prep_ready",
        "roadmap_v2_behavior_closed === roadmapProgress.behavior_closed_cells",
        "current_stage_percent === 88",
        "classroom_operations_triage_input",
    ]:
        assert_contains(runner, token, "runner")


def check_contract_and_dry_run() -> None:
    contract = load_json(CONTRACT)
    assert_true(contract["schema"] == "ddn.pack.contract.v1", "bad contract schema")
    assert_true(contract["pack"] == "studio_benchmark_baseline_prep_dry_run_v1", "bad contract pack")
    assert_true(contract["closed_by"] == "STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1", "bad contract closed_by")
    rebase = load_json(SOURCE_REBASE)
    super_long = rebase["super_long_progress"]
    roadmap = rebase["roadmap_progress"]

    assert_true(contract["product_code_change"] is False, "contract must not declare product code change")
    assert_true(contract["product_ui_change"] is False, "contract must not declare product UI change")
    assert_true(contract["work_unit_closed"] == 6, "bad work unit closed")
    assert_true(contract["work_unit_total"] == 6, "bad work unit total")
    assert_true(contract["work_unit_percent"] == 100, "bad work unit percent")
    assert_true(contract["super_long_closed"] == super_long["closed_items"], "bad super-long closed")
    assert_true(contract["super_long_total"] == super_long["total_items"], "bad super-long total")
    assert_true(contract["super_long_percent"] == super_long["percent"], "bad super-long percent")
    assert_true(contract["roadmap_v2_behavior_closed"] == roadmap["behavior_closed_cells"], "bad roadmap closed")
    assert_true(contract["roadmap_v2_total"] == 90, "bad roadmap total")
    assert_true(contract["roadmap_v2_percent"] == roadmap["behavior_percent"], "bad roadmap percent")
    assert_true(contract["browser_runner"] == "tests/studio_benchmark_baseline_prep_dry_run_runner.mjs", "bad browser runner")

    dry_run = load_json(DRY_RUN)
    assert_true(dry_run["schema"] == "ddn.studio.benchmark_baseline_prep_dry_run.v1", "bad dry-run schema")
    assert_true(dry_run["product_code_change"] is False, "dry-run must not declare product code change")
    assert_true(dry_run["product_ui_change"] is False, "dry-run must not declare product UI change")
    assert_true(dry_run["planned_baseline_inputs"] == expected_inputs(), "planned baseline inputs mismatch")
    assert_true(dry_run["closure_tier"] == "닫힘-문서", "bad closure tier")
    assert_true(dry_run["browser_runner"] == "tests/studio_benchmark_baseline_prep_dry_run_runner.mjs", "bad dry-run browser runner")
    assert_true(dry_run["post_super_long_plan"] == {"closed": 7, "total": 8, "percent": 88, "closure_tier": "닫힘-문서"}, "bad follow-up progress")
    assert_true(dry_run["roadmap_v2_product_behavior"] == {"closed": roadmap["behavior_closed_cells"], "total": roadmap["roadmap_v2_total_cells"], "percent": roadmap["behavior_percent"]}, "bad roadmap behavior progress")
    assert_true(dry_run["super_long_product_behavior"] == {"closed": super_long["closed_items"], "total": super_long["total_items"], "percent": super_long["percent"]}, "bad super-long behavior progress")
    assert_true(dry_run["documentation_reference_progress"]["roadmap_v2_percent"] == roadmap["documentation_reference_percent"], "bad roadmap docs reference percent")
    assert_true(dry_run["known_failed_baseline_checks"] == [], "known failed baseline checks must be unlocked")
    assert_true(len(dry_run["reassessed_pass_baseline_checks"]) == 4, "reassessed baseline check count mismatch")
    assert_true(dry_run["next_item"] == NEXT, "bad next recommendation")
    assert_true(len(dry_run["changed_product_files"]) == 0, "changed product files must remain empty")
    for rel_path in dry_run["changed_product_files"]:
        assert_true((ROOT / rel_path).exists(), f"changed product file missing: {rel_path}")

    for key in [
        "benchmark_execution_claim",
        "performance_baseline_generation_claim",
        "performance_baseline_publication_claim",
        "lts_certification_claim",
        "release_execution_claim",
        "public_upload_claim",
        "cloud_sync_claim",
        "account_setup_claim",
        "permission_system_claim",
    ]:
        assert_true(dry_run[key] is False, f"boundary claim must remain false: {key}")

    commands = dry_run["preflight_commands"]
    for command in [
        "node tests/studio_benchmark_baseline_prep_dry_run_runner.mjs",
        "python tests/run_studio_classroom_operations_triage_check.py",
        "python tests/run_studio_benchmark_lts_matrix_check.py",
        "git status --short -- docs/ssot",
    ]:
        assert_true(command in commands, f"missing preflight command: {command}")


def check_source_alignment() -> None:
    matrix = load_json(SOURCE_MATRIX)
    assert_true(matrix["schema"] == "ddn.studio.benchmark_lts_matrix.v1", "bad matrix schema")
    assert_true(len(matrix["matrix_entries"]) == 5, "matrix entry count mismatch")
    for key in [
        "performance_baseline_claim",
        "benchmark_execution_claim",
        "lts_certification_claim",
        "release_execution_claim",
        "public_upload_claim",
        "cloud_sync_claim",
        "account_setup_claim",
        "permission_system_claim",
    ]:
        assert_true(matrix[key] is False, f"matrix boundary must remain false: {key}")

    triage = load_json(SOURCE_TRIAGE)
    assert_true(triage["schema"] == "ddn.studio.classroom_operations_triage.v1", "bad triage schema")
    assert_true(len(triage["triage_rows"]) == 6, "triage row count mismatch")
    assert_true(triage["next_item"] == "STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1", "bad triage next recommendation")
    for key in [
        "classroom_operations_runtime_claim",
        "student_data_collection_claim",
        "release_execution_claim",
        "public_upload_claim",
        "cloud_sync_claim",
        "account_setup_claim",
        "permission_system_claim",
        "result_replay_claim",
    ]:
        assert_true(triage[key] is False, f"triage boundary must remain false: {key}")


def check_golden() -> None:
    input_text = read_text(INPUT)
    assert_contains(input_text, '"follow-up plan docs: 7/8 = 88%" 보여주기.', "input.ddn")
    assert_contains(input_text, '"super-long behavior: 5/18 = 28%" 보여주기.', "input.ddn")
    assert_contains(input_text, '"roadmap v2 behavior: 21/90 = 23%" 보여주기.', "input.ddn")

    lines = [json.loads(line) for line in read_text(GOLDEN).splitlines() if line.strip()]
    assert_true(len(lines) == 1, "golden must contain one JSONL row")
    stdout = "\n".join(lines[0]["stdout"])
    for token in [
        "STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1",
        "studio benchmark baseline prep dry run sealed",
        "benchmark baseline prep dry-run schema: ddn.studio.benchmark_baseline_prep_dry_run.v1",
        "planned baseline inputs: 5",
        "follow-up plan docs: 7/8 = 88%",
        "super-long behavior: 5/18 = 28%",
        "roadmap v2 behavior: 21/90 = 23%",
        "next: STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1",
    ]:
        assert_contains(stdout, token, "golden stdout")


def run_command(command: list[str]) -> None:
    subprocess.run(command, cwd=ROOT, check=True)


def run_required_gates() -> None:
    run_command([sys.executable, "tests/run_pack_golden.py", "studio_benchmark_baseline_prep_dry_run_v1"])
    run_command(["node", "tests/studio_benchmark_baseline_prep_dry_run_runner.mjs"])
    run_command([sys.executable, "tests/run_studio_classroom_operations_triage_check.py"])
    run_command([sys.executable, "tests/run_studio_benchmark_lts_matrix_check.py"])


def main() -> None:
    check_required_files()
    check_docs()
    check_ui_contract()
    check_contract_and_dry_run()
    check_source_alignment()
    check_golden()
    run_required_gates()
    print("studio_benchmark_baseline_prep_dry_run_check: ok")


if __name__ == "__main__":
    main()
