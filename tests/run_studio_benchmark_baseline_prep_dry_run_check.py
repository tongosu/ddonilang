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
HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
RUNNER = ROOT / "tests" / "studio_benchmark_baseline_prep_dry_run_runner.mjs"
SOURCE_MATRIX = ROOT / "pack" / "studio_benchmark_lts_matrix_v1" / "benchmark_lts_matrix.detjson"
SOURCE_TRIAGE = ROOT / "pack" / "studio_classroom_operations_triage_v1" / "classroom_operations_triage.detjson"
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
        HTML,
        STYLES,
        RUNNER,
        SOURCE_MATRIX,
        SOURCE_TRIAGE,
    ]:
        assert_true(path.exists(), f"missing required file: {path.relative_to(ROOT)}")


def check_docs() -> None:
    doc = read_text(DOC)
    for token in [
        "## Product Changes",
        "Every row keeps `prep_only=true`, `generated_now=false`, and `benchmark_execution_claim=false`.",
        "작업 단위: 6/6 = 100% (`닫힘-동작`)",
        "초장기 계획: 1시대 5/5 = 100%, 2시대 7/7 = 100%, 3시대 6/6 = 100%, 전체 18/18 = 100%",
        "현재 스테이지: post-super-long follow-up 7/8 = 88%",
        "ROADMAP_V2 product behavior baseline: 87/90 = 97%",
        "node tests/studio_benchmark_baseline_prep_dry_run_runner.mjs",
        NEXT,
    ]:
        assert_contains(doc, token, "root doc")

    report = read_text(REPORT)
    for token in [
        "This is product UI behavior plus local benchmark baseline preparation dry-run evidence",
        "It renders planned baseline input rows",
        "작업 단위: 6/6 = 100% (`닫힘-동작`)",
        "작업 단위: 6/6 = 100%",
        "초장기 계획: 1시대 5/5 = 100%, 2시대 7/7 = 100%, 3시대 6/6 = 100%, 전체 18/18 = 100%",
        "현재 스테이지: post-super-long follow-up 7/8 = 88%",
        "ROADMAP_V2 product behavior baseline: 87/90 = 97%",
    ]:
        assert_contains(report, token, "studio report")

    index = read_text(INDEX)
    assert_contains(index, "BENCHMARK_BASELINE_PREP_DRY_RUN_V1", "studio index")

    roadmap = read_text(ROADMAP)
    for token in [
        "STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1",
        "전체 초장기 계획 18/18 = 100%",
        "post-super-long follow-up 7/8 = 88%",
        "ROADMAP_V2 product behavior baseline 87/90 = 97%",
    ]:
        assert_contains(roadmap, token, "long horizon roadmap")

    summary = read_text(DEV_SUMMARY)
    for token in [
        "[STUDIO][BENCHMARK] Benchmark baseline prep dry-run UI",
        "node tests/studio_benchmark_baseline_prep_dry_run_runner.mjs` PASS",
        "현재 스테이지: post-super-long follow-up 7/8 = 88%",
        "ROADMAP_V2 product behavior baseline: 87/90 = 97%",
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
        "roadmap_v2_behavior_closed: 87",
        "roadmap_v2_percent: 97",
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
        "studio_benchmark_baseline_prep_dry_run.js",
        "benchmarkBaselinePrepDryRun",
        "publishBenchmarkBaselinePrepDryRun",
        "__SEAMGRIM_BENCHMARK_BASELINE_PREP_DRY_RUN__",
        "__SEAMGRIM_BENCHMARK_BASELINE_PREP_DRY_RUN_TEXT__",
    ]:
        assert_contains(app, token, "app")

    html = read_text(HTML)
    for token in [
        'id="benchmark-baseline-prep-dry-run"',
        "data-benchmark-baseline-prep-dry-run",
        'aria-label="Studio benchmark baseline prep dry-run"',
    ]:
        assert_contains(html, token, "html")

    styles = read_text(STYLES)
    for token in [
        ".benchmark-baseline-prep-dry-run",
        ".benchmark-prep-head",
        ".benchmark-prep-btn",
        ".benchmark-prep-detail",
        ".benchmark-prep-progress",
    ]:
        assert_contains(styles, token, "styles")

    runner = read_text(RUNNER)
    for token in [
        "studio_benchmark_baseline_prep_dry_run: ok",
        "buildBenchmarkBaselinePrepDryRun",
        "formatBenchmarkBaselinePrepDryRunText",
        "data-benchmark-baseline-prep-dry-run-status='benchmark_baseline_prep_ready'",
        "benchmark_baseline_prep_ready",
        "roadmap_v2_behavior_closed === 87",
        "current_stage_percent === 88",
        "classroom_operations_triage_input",
    ]:
        assert_contains(runner, token, "runner")


def check_contract_and_dry_run() -> None:
    contract = load_json(CONTRACT)
    assert_true(contract["schema"] == "ddn.pack.contract.v1", "bad contract schema")
    assert_true(contract["pack"] == "studio_benchmark_baseline_prep_dry_run_v1", "bad contract pack")
    assert_true(contract["closed_by"] == "STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1", "bad contract closed_by")
    assert_true(contract["product_code_change"] is True, "contract must declare product code change")
    assert_true(contract["product_ui_change"] is True, "contract must declare product UI change")
    assert_true(contract["work_unit_closed"] == 6, "bad work unit closed")
    assert_true(contract["work_unit_total"] == 6, "bad work unit total")
    assert_true(contract["work_unit_percent"] == 100, "bad work unit percent")
    assert_true(contract["roadmap_v2_behavior_closed"] == 87, "bad roadmap closed")
    assert_true(contract["roadmap_v2_total"] == 90, "bad roadmap total")
    assert_true(contract["roadmap_v2_percent"] == 97, "bad roadmap percent")
    assert_true(contract["browser_runner"] == "tests/studio_benchmark_baseline_prep_dry_run_runner.mjs", "bad browser runner")

    dry_run = load_json(DRY_RUN)
    assert_true(dry_run["schema"] == "ddn.studio.benchmark_baseline_prep_dry_run.v1", "bad dry-run schema")
    assert_true(dry_run["product_code_change"] is True, "dry-run must declare product code change")
    assert_true(dry_run["product_ui_change"] is True, "dry-run must declare product UI change")
    assert_true(dry_run["planned_baseline_inputs"] == expected_inputs(), "planned baseline inputs mismatch")
    assert_true(dry_run["closure_tier"] == "닫힘-동작", "bad closure tier")
    assert_true(dry_run["browser_runner"] == "tests/studio_benchmark_baseline_prep_dry_run_runner.mjs", "bad dry-run browser runner")
    assert_true(dry_run["post_super_long_plan"] == {"closed": 7, "total": 8, "percent": 88}, "bad follow-up progress")
    assert_true(dry_run["roadmap_v2_product_behavior"] == {"closed": 87, "total": 90, "percent": 97}, "bad roadmap behavior progress")
    assert_true(dry_run["next_item"] == NEXT, "bad next recommendation")
    assert_true(len(dry_run["changed_product_files"]) == 5, "changed product file count mismatch")
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
    assert_true(triage["roadmap_v2_product_behavior"]["closed"] == 90, "triage roadmap closed mismatch")
    assert_true(triage["roadmap_v2_product_behavior"]["percent"] == 100, "triage roadmap percent mismatch")
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
    assert_contains(input_text, '"roadmap v2 behavior: 87/90 = 97%" 보여주기.', "input.ddn")

    lines = [json.loads(line) for line in read_text(GOLDEN).splitlines() if line.strip()]
    assert_true(len(lines) == 1, "golden must contain one JSONL row")
    stdout = "\n".join(lines[0]["stdout"])
    for token in [
        "STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1",
        "studio benchmark baseline prep dry run sealed",
        "benchmark baseline prep dry-run schema: ddn.studio.benchmark_baseline_prep_dry_run.v1",
        "planned baseline inputs: 5",
        "follow-up plan: 7/8 = 88%",
        "roadmap v2 behavior: 87/90 = 97%",
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
