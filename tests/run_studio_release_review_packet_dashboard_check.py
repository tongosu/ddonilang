from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_RELEASE_REVIEW_PACKET_DASHBOARD_V1.md"
REPORT = ROOT / "docs" / "studio" / "RELEASE_REVIEW_PACKET_DASHBOARD_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "studio_release_review_packet_dashboard_v1"
CONTRACT = PACK / "contract.detjson"
DASHBOARD = PACK / "release_review_packet_dashboard.detjson"
SOURCE_SNAPSHOT = ROOT / "pack" / "studio_benchmark_baseline_local_snapshot_v1" / "benchmark_baseline_local_snapshot.detjson"
SOURCE_CONTINUITY = ROOT / "pack" / "studio_release_approval_packet_continuity_v1" / "continuity.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_release_review_packet_dashboard.js"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
RUNNER = ROOT / "tests" / "studio_release_review_packet_dashboard_runner.mjs"
SOURCE_SNAPSHOT_CHECK = ROOT / "tests" / "run_studio_benchmark_baseline_local_snapshot_check.py"
REQUIRED_APPROVAL = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다"
NEXT = "STUDIO_LESSON_PUBLICATION_REVIEW_SURFACE_V1"


def fail(message: str) -> None:
    print(f"studio_release_review_packet_dashboard_check: FAIL: {message}", file=sys.stderr)
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
        "dashboard_surface": "local_release_review_packet_dashboard",
        "dashboard_only": True,
        "generated_now": False,
        "release_approval_claim": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "product_ui_change": True,
        "cloud_sync_claim": False,
        "account_setup_claim": False,
        "permission_system_claim": False,
    }
    rows: list[tuple[str, str | None, str | None, str]] = [
        ("approval_state_dashboard_card", "approval_continuity_snapshot", "pack/studio_release_approval_chain_closure_v1/closure.detjson", "approval_state"),
        ("benchmark_snapshot_dashboard_card", "benchmark_lts_matrix_snapshot", None, "benchmark_snapshot"),
        ("classroom_operations_dashboard_card", "classroom_operations_panel_snapshot", None, "classroom_operations"),
        ("local_packaging_review_dashboard_card", "local_packaging_snapshot", "pack/studio_local_packaging_consolidation_v1/local_package_manifest.detjson", "local_packaging_review"),
        ("publication_prep_review_dashboard_card", None, "pack/studio_public_lesson_publication_prep_v1/publication_prep.detjson", "publication_prep_review"),
        ("registry_share_review_dashboard_card", None, "pack/studio_registry_share_seed_v1/registry_share_seed.detjson", "registry_share_review"),
    ]
    return [
        {
            "id": row_id,
            "source_snapshot_row": snapshot_row,
            "source_review_material": review_material,
            "dashboard_lane": lane,
            **common,
        }
        for row_id, snapshot_row, review_material, lane in rows
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
        DASHBOARD,
        SOURCE_SNAPSHOT,
        SOURCE_CONTINUITY,
        UI_MODULE,
        APP_JS,
        INDEX_HTML,
        STYLES_CSS,
        RUNNER,
        SOURCE_SNAPSHOT_CHECK,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "STUDIO_RELEASE_REVIEW_PACKET_DASHBOARD_V1",
        "ddn.studio.release_review_packet_dashboard.v1",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        "닫힘-동작",
        "dashboard rows: 6/6 = 100%",
        "전체 초장기 계획: 18/18 = 100%",
        "현재 스테이지: 새 마-3 개발 계획 5/8 = 63%",
        "ROADMAP_V2 product behavior baseline: 90/90 = 100%",
        "studio_release_review_packet_dashboard_runner.mjs",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens)
    require_contains(REPORT, tokens[:10])
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_RELEASE_REVIEW_PACKET_DASHBOARD_V1",
            "studio_release_review_packet_dashboard_runner.mjs",
            "dashboard rows: 6/6 = 100%",
            "전체 초장기 계획: 18/18 = 100%",
            "현재 스테이지: 새 마-3 개발 계획 5/8 = 63%",
            "ROADMAP_V2 product behavior baseline: 90/90 = 100%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_ui_source() -> None:
    require_contains(
        UI_MODULE,
        [
            "ddn.studio.release_review_packet_dashboard.v1",
            "buildReleaseReviewPacketDashboard",
            "formatReleaseReviewPacketDashboardText",
            "renderReleaseReviewPacketDashboard",
            "product_ui_change: true",
            "release_approval_claim: false",
            "release_execution_claim: false",
            "public_release_claim: false",
            "super_long_behavior_closed: 18",
            "current_stage_percent: 63",
            "roadmap_v2_percent: 100",
            REQUIRED_APPROVAL,
        ],
    )
    require_contains(
        APP_JS,
        [
            "studio_release_review_packet_dashboard.js",
            "publishReleaseReviewPacketDashboard",
            "__SEAMGRIM_RELEASE_REVIEW_PACKET_DASHBOARD__",
            "buildReleaseReviewPacketDashboard",
        ],
    )
    require_contains(
        INDEX_HTML,
        [
            "release-review-packet-dashboard",
            "data-release-review-packet-dashboard",
        ],
    )
    require_contains(
        STYLES_CSS,
        [
            ".release-review-packet-dashboard",
            ".release-review-dashboard-btn.active",
        ],
    )
    require_contains(
        RUNNER,
        [
            "studio_release_review_packet_dashboard: ok",
            "data-release-review-status='release_review_dashboard_ready'",
            "release_approval_claim",
            REQUIRED_APPROVAL,
        ],
    )


def check_contract_and_dashboard() -> None:
    contract = load_json(CONTRACT)
    expected_contract = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_release_review_packet_dashboard_v1",
        "kind": "studio_release_review_packet_dashboard",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "release_review_packet_dashboard_claim": True,
        "release_approval_claim": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "closed_by": "STUDIO_RELEASE_REVIEW_PACKET_DASHBOARD_V1",
        "based_on": "STUDIO_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1",
        "dashboard_row_count": 6,
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "dashboard_rows_closed": 6,
        "dashboard_rows_total": 6,
        "dashboard_rows_percent": 100,
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "current_stage_closed": 5,
        "current_stage_total": 8,
        "current_stage_percent": 63,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
        "closure_tier": "닫힘-동작",
        "required_approval_phrase": REQUIRED_APPROVAL,
        "generic_next_dev_request_is_approval": False,
        "next_state": "AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected_contract.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")
    for file in [
        "solutions/seamgrim_ui_mvp/ui/studio_release_review_packet_dashboard.js",
        "solutions/seamgrim_ui_mvp/ui/app.js",
        "solutions/seamgrim_ui_mvp/ui/index.html",
        "solutions/seamgrim_ui_mvp/ui/styles.css",
    ]:
        if file not in contract.get("changed_product_files", []):
            fail(f"contract missing changed product file: {file}")

    dashboard = load_json(DASHBOARD)
    if dashboard.get("schema") != "ddn.studio.release_review_packet_dashboard.v1":
        fail(f"dashboard schema mismatch: {dashboard.get('schema')!r}")
    if dashboard.get("work_item") != "STUDIO_RELEASE_REVIEW_PACKET_DASHBOARD_V1":
        fail(f"dashboard work item mismatch: {dashboard.get('work_item')!r}")
    for flag, expected in (
        ("runtime_claim", False),
        ("product_code_change", True),
        ("product_ui_change", True),
        ("release_approval_claim", False),
        ("release_execution_claim", False),
        ("public_release_claim", False),
        ("github_release_claim", False),
        ("public_upload_claim", False),
        ("registry_publish_claim", False),
        ("publication_snapshot_emit_claim", False),
        ("artifact_signing_claim", False),
        ("benchmark_execution_claim", False),
        ("performance_baseline_generation_claim", False),
        ("performance_baseline_publication_claim", False),
        ("student_data_collection_claim", False),
        ("cloud_sync_claim", False),
        ("account_setup_claim", False),
        ("permission_system_claim", False),
    ):
        if dashboard.get(flag) is not expected:
            fail(f"dashboard {flag} expected {expected!r}, got {dashboard.get(flag)!r}")
    if dashboard.get("dashboard_rows") != expected_rows():
        fail(f"dashboard rows mismatch: {dashboard.get('dashboard_rows')!r}")
    if dashboard.get("progress") != {
        "super_long_behavior_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "current_stage_closed": 5,
        "current_stage_total": 8,
        "current_stage_percent": 63,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
    }:
        fail(f"progress mismatch: {dashboard.get('progress')!r}")
    if dashboard.get("required_approval_phrase") != REQUIRED_APPROVAL:
        fail("required approval phrase mismatch")
    if dashboard.get("generic_next_dev_request_is_approval") is not False:
        fail("generic next-dev request must not approve")
    if dashboard.get("next_state") != "AWAIT_EXPLICIT_RELEASE_APPROVAL":
        fail(f"next state mismatch: {dashboard.get('next_state')!r}")
    if dashboard.get("closure_tier") != "닫힘-동작":
        fail(f"closure tier mismatch: {dashboard.get('closure_tier')!r}")
    if dashboard.get("next_item") != NEXT:
        fail(f"next item mismatch: {dashboard.get('next_item')!r}")


def check_source_alignment() -> None:
    snapshot = load_json(SOURCE_SNAPSHOT)
    continuity = load_json(SOURCE_CONTINUITY)
    if snapshot.get("schema") != "ddn.studio.benchmark_baseline_local_snapshot.v1":
        fail(f"source snapshot schema mismatch: {snapshot.get('schema')!r}")
    if continuity.get("schema") != "ddn.studio.release_approval_packet_continuity.v1":
        fail(f"source continuity schema mismatch: {continuity.get('schema')!r}")
    if snapshot.get("next_item") != "STUDIO_RELEASE_REVIEW_PACKET_DASHBOARD_V1":
        fail(f"source snapshot next item mismatch: {snapshot.get('next_item')!r}")
    if snapshot.get("progress", {}).get("super_long_behavior_closed") != 18:
        fail(f"source snapshot progress mismatch: {snapshot.get('progress')!r}")
    if snapshot.get("progress", {}).get("roadmap_v2_behavior_closed") != 90:
        fail(f"source snapshot roadmap closed mismatch: {snapshot.get('progress')!r}")
    if snapshot.get("progress", {}).get("roadmap_v2_percent") != 100:
        fail(f"source snapshot roadmap percent mismatch: {snapshot.get('progress')!r}")
    if continuity.get("required_approval_phrase") != REQUIRED_APPROVAL:
        fail("source continuity required approval phrase mismatch")
    if continuity.get("generic_next_dev_request_is_approval") is not False:
        fail("source continuity generic request must not approve")
    if continuity.get("next_state") != "AWAIT_EXPLICIT_RELEASE_APPROVAL":
        fail(f"source continuity next state mismatch: {continuity.get('next_state')!r}")

    snapshot_ids = [row.get("id") for row in snapshot.get("snapshot_rows", [])]
    required_snapshot_ids = [
        row["source_snapshot_row"]
        for row in expected_rows()
        if row["source_snapshot_row"] is not None
    ]
    missing_snapshot = [row_id for row_id in required_snapshot_ids if row_id not in snapshot_ids]
    if missing_snapshot:
        fail(f"missing snapshot row anchors: {missing_snapshot!r}")

    continuity_materials = set(continuity.get("new_review_materials", []))
    continuity_materials.add(continuity.get("approval_chain_closure"))
    required_materials = [
        row["source_review_material"]
        for row in expected_rows()
        if row["source_review_material"] is not None
    ]
    missing_materials = [item for item in required_materials if item not in continuity_materials]
    if missing_materials:
        fail(f"missing continuity review materials: {missing_materials!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected_stdout = [
        "STUDIO_RELEASE_REVIEW_PACKET_DASHBOARD_V1",
        "studio release review packet dashboard behavior sealed",
        "release review packet dashboard schema: ddn.studio.release_review_packet_dashboard.v1",
        "dashboard rows: 6/6 = 100%",
        "overall super-long behavior: 18/18 = 100%",
        "current stage: 5/8 = 63%",
        "roadmap v2 behavior: 90/90 = 100%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_release_review_packet_dashboard_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected_stdout:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_release_review_packet_dashboard_v1"],
        ["node", "tests/studio_release_review_packet_dashboard_runner.mjs"],
        ["python", "tests/run_studio_benchmark_baseline_local_snapshot_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_ui_source()
    check_contract_and_dashboard()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_release_review_packet_dashboard_check: ok")


if __name__ == "__main__":
    main()
