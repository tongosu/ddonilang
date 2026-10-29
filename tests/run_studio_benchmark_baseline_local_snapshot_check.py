from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "queue" / "STUDIO_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1.md"
REPORT = ROOT / "docs" / "studio" / "BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "studio_benchmark_baseline_local_snapshot_v1"
CONTRACT = PACK / "contract.detjson"
SNAPSHOT = PACK / "benchmark_baseline_local_snapshot.detjson"
SOURCE_PREP = ROOT / "pack" / "studio_benchmark_baseline_prep_dry_run_v1" / "benchmark_baseline_prep_dry_run.detjson"
SOURCE_PANEL = ROOT / "pack" / "studio_classroom_operations_panel_preview_v1" / "classroom_operations_panel_preview.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_benchmark_baseline_local_snapshot.js"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
RUNNER = ROOT / "tests" / "studio_benchmark_baseline_local_snapshot_runner.mjs"
SOURCE_PANEL_CHECK = ROOT / "tests" / "run_studio_classroom_operations_panel_preview_check.py"
NEXT = "STUDIO_RELEASE_REVIEW_PACKET_DASHBOARD_V1"


def fail(message: str) -> None:
    print(f"studio_benchmark_baseline_local_snapshot_check: FAIL: {message}", file=sys.stderr)
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
        "snapshot_surface": "local_benchmark_baseline_snapshot",
        "snapshot_only": True,
        "generated_now": False,
        "benchmark_execution_claim": False,
        "performance_baseline_generation_claim": False,
        "performance_baseline_publication_claim": False,
        "lts_certification_claim": False,
        "public_release_claim": False,
    }
    rows: list[tuple[str, str | None, str | None, str]] = [
        ("benchmark_lts_matrix_snapshot", "benchmark_lts_matrix_input", None, "benchmark_lts_matrix"),
        ("classroom_operations_triage_snapshot", "classroom_operations_triage_input", None, "classroom_operations_triage"),
        ("browser_smoke_matrix_snapshot", "browser_smoke_matrix_input", None, "browser_smoke_matrix"),
        ("local_packaging_snapshot", "local_packaging_input", None, "local_packaging"),
        ("approval_continuity_snapshot", "approval_continuity_input", None, "approval_continuity"),
        ("classroom_operations_panel_snapshot", None, "classroom_report_status_panel", "classroom_operations_panel"),
    ]
    return [
        {
            "id": row_id,
            "source_planned_input": planned_input,
            "source_panel_row": panel_row,
            "snapshot_lane": lane,
            **common,
        }
        for row_id, planned_input, panel_row, lane in rows
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
        SNAPSHOT,
        SOURCE_PREP,
        SOURCE_PANEL,
        UI_MODULE,
        APP_JS,
        DEV_SURFACES_JS,
        INDEX_HTML,
        DEV_SURFACES_CSS,
        RUNNER,
        SOURCE_PANEL_CHECK,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "STUDIO_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1",
        "ddn.studio.benchmark_baseline_local_snapshot.v1",
        "Primary coordinate: `타-3`",
        "Support coordinate: `마-3`",
        "닫힘-동작",
        "snapshot rows: 6/6 = 100%",
        "전체 초장기 계획: 9/18 = 50%",
        "현재 스테이지: 새 마-3 개발 계획 4/8 = 50%",
        "ROADMAP_V2 behavior-closed progress: 90/90 = 100%",
        "studio_benchmark_baseline_local_snapshot_runner.mjs",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens)
    require_contains(REPORT, tokens[:10])
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1",
            "studio_benchmark_baseline_local_snapshot_runner.mjs",
            "snapshot rows: 6/6 = 100%",
            "전체 초장기 계획: 9/18 = 50%",
            "현재 스테이지: 새 마-3 개발 계획 4/8 = 50%",
            "ROADMAP_V2 behavior-closed progress: 90/90 = 100%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_ui_source() -> None:
    require_contains(
        UI_MODULE,
        [
            "ddn.studio.benchmark_baseline_local_snapshot.v1",
            "buildBenchmarkBaselineLocalSnapshot",
            "formatBenchmarkBaselineLocalSnapshotText",
            "renderBenchmarkBaselineLocalSnapshot",
            "product_ui_change: true",
            "benchmark_execution_claim: false",
            "performance_baseline_generation_claim: false",
            "super_long_behavior_closed: 9",
            "current_stage_percent: 50",
            "roadmap_v2_percent: 100",
        ],
    )
    require_contains(APP_JS, ["shouldEnableDevSurfaces", "./dev_surfaces.js"])
    require_contains(
        DEV_SURFACES_JS,
        [
            "studio_benchmark_baseline_local_snapshot.js",
            "benchmark-baseline-local-snapshot",
            "__SEAMGRIM_BENCHMARK_BASELINE_LOCAL_SNAPSHOT__",
            "__SEAMGRIM_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_TEXT__",
            "buildBenchmarkBaselineLocalSnapshot",
            "formatBenchmarkBaselineLocalSnapshotText",
            "renderBenchmarkBaselineLocalSnapshot",
        ],
    )
    require_contains(
        DEV_SURFACES_CSS,
        [
            ".benchmark-baseline-local-snapshot",
            ".benchmark-baseline-snapshot-btn.active",
        ],
    )
    require_contains(
        RUNNER,
        [
            "studio_benchmark_baseline_local_snapshot: ok",
            "data-benchmark-baseline-status='benchmark_baseline_snapshot_ready'",
            "product_ui_change",
            "benchmark_execution_claim",
        ],
    )


def check_contract_and_snapshot() -> None:
    contract = load_json(CONTRACT)
    expected_contract = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_benchmark_baseline_local_snapshot_v1",
        "kind": "studio_benchmark_baseline_local_snapshot",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "benchmark_baseline_local_snapshot_claim": True,
        "benchmark_execution_claim": False,
        "performance_baseline_generation_claim": False,
        "performance_baseline_publication_claim": False,
        "lts_certification_claim": False,
        "closed_by": "STUDIO_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1",
        "based_on": "STUDIO_CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1",
        "snapshot_row_count": 6,
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "snapshot_rows_closed": 6,
        "snapshot_rows_total": 6,
        "snapshot_rows_percent": 100,
        "super_long_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "current_stage_closed": 4,
        "current_stage_total": 8,
        "current_stage_percent": 50,
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
        "solutions/seamgrim_ui_mvp/ui/studio_benchmark_baseline_local_snapshot.js",
        "solutions/seamgrim_ui_mvp/ui/app.js",
        "solutions/seamgrim_ui_mvp/ui/index.html",
        "solutions/seamgrim_ui_mvp/ui/styles.css",
    ]:
        if file not in contract.get("changed_product_files", []):
            fail(f"contract missing changed product file: {file}")

    snapshot = load_json(SNAPSHOT)
    if snapshot.get("schema") != "ddn.studio.benchmark_baseline_local_snapshot.v1":
        fail(f"snapshot schema mismatch: {snapshot.get('schema')!r}")
    if snapshot.get("work_item") != "STUDIO_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1":
        fail(f"snapshot work item mismatch: {snapshot.get('work_item')!r}")
    for flag, expected in (
        ("runtime_claim", False),
        ("product_code_change", True),
        ("product_ui_change", True),
        ("benchmark_execution_claim", False),
        ("performance_baseline_generation_claim", False),
        ("performance_baseline_publication_claim", False),
        ("lts_certification_claim", False),
        ("student_data_collection_claim", False),
        ("panel_write_claim", False),
        ("feedback_write_claim", False),
        ("release_execution_claim", False),
        ("public_release_claim", False),
    ):
        if snapshot.get(flag) is not expected:
            fail(f"snapshot {flag} expected {expected!r}, got {snapshot.get(flag)!r}")
    if snapshot.get("snapshot_rows") != expected_rows():
        fail(f"snapshot rows mismatch: {snapshot.get('snapshot_rows')!r}")
    if snapshot.get("progress") != {
        "super_long_behavior_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "current_stage_closed": 4,
        "current_stage_total": 8,
        "current_stage_percent": 50,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
    }:
        fail(f"progress mismatch: {snapshot.get('progress')!r}")
    if snapshot.get("closure_tier") != "닫힘-동작":
        fail(f"closure tier mismatch: {snapshot.get('closure_tier')!r}")
    if snapshot.get("next_item") != NEXT:
        fail(f"next item mismatch: {snapshot.get('next_item')!r}")


def check_source_alignment() -> None:
    prep = load_json(SOURCE_PREP)
    panel = load_json(SOURCE_PANEL)
    if prep.get("schema") != "ddn.studio.benchmark_baseline_prep_dry_run.v1":
        fail(f"source prep schema mismatch: {prep.get('schema')!r}")
    if panel.get("schema") != "ddn.studio.classroom_operations_panel_preview.v1":
        fail(f"source panel schema mismatch: {panel.get('schema')!r}")
    if panel.get("next_item") != "STUDIO_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1":
        fail(f"source panel next item mismatch: {panel.get('next_item')!r}")
    if panel.get("progress", {}).get("roadmap_v2_behavior_closed") != 90:
        fail(f"source panel roadmap closed mismatch: {panel.get('progress')!r}")
    if panel.get("progress", {}).get("roadmap_v2_percent") != 100:
        fail(f"source panel roadmap percent mismatch: {panel.get('progress')!r}")

    planned_ids = [row.get("id") for row in prep.get("planned_baseline_inputs", [])]
    required_planned_ids = [
        row["source_planned_input"]
        for row in expected_rows()
        if row["source_planned_input"] is not None
    ]
    if planned_ids != required_planned_ids:
        fail(f"planned baseline input alignment mismatch: {planned_ids!r}")
    panel_ids = [row.get("id") for row in panel.get("panel_rows", [])]
    if "classroom_report_status_panel" not in panel_ids:
        fail(f"missing classroom panel anchor: {panel_ids!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected_stdout = [
        "STUDIO_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1",
        "studio benchmark baseline local snapshot behavior sealed",
        "benchmark baseline local snapshot schema: ddn.studio.benchmark_baseline_local_snapshot.v1",
        "snapshot rows: 6/6 = 100%",
        "official studio local progress: 9/18 = 50%",
        "current stage: 4/8 = 50%",
        "roadmap v2 behavior-closed: 90/90 = 100%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_benchmark_baseline_local_snapshot_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected_stdout:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_benchmark_baseline_local_snapshot_v1"],
        ["node", "tests/studio_benchmark_baseline_local_snapshot_runner.mjs"],
        ["python", "tests/run_studio_classroom_operations_panel_preview_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_ui_source()
    check_contract_and_snapshot()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_benchmark_baseline_local_snapshot_check: ok")


if __name__ == "__main__":
    main()
