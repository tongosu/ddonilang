from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_LESSON_PUBLICATION_REVIEW_SURFACE_V1.md"
REPORT = ROOT / "docs" / "studio" / "LESSON_PUBLICATION_REVIEW_SURFACE_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "studio_lesson_publication_review_surface_v1"
CONTRACT = PACK / "contract.detjson"
SURFACE = PACK / "lesson_publication_review_surface.detjson"
SOURCE_DASHBOARD = ROOT / "pack" / "studio_release_review_packet_dashboard_v1" / "release_review_packet_dashboard.detjson"
SOURCE_PREP = ROOT / "pack" / "studio_public_lesson_publication_prep_v1" / "publication_prep.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_lesson_publication_review_surface.js"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
RUNNER = ROOT / "tests" / "studio_lesson_publication_review_surface_runner.mjs"
SOURCE_DASHBOARD_CHECK = ROOT / "tests" / "run_studio_release_review_packet_dashboard_check.py"
NEXT = "STUDIO_MA3_REGRESSION_GATE_MATRIX_V1"


def fail(message: str) -> None:
    print(f"studio_lesson_publication_review_surface_check: FAIL: {message}", file=sys.stderr)
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
        "surface_kind": "local_lesson_publication_review_surface",
        "candidate_count": 12,
        "surface_only": True,
        "generated_now": False,
        "public_upload_claim": False,
        "registry_publish_claim": False,
        "publication_snapshot_emit_claim": False,
        "active_allowlist_mutation": False,
        "lesson_schema_change": False,
        "product_ui_change": True,
    }
    rows: list[tuple[str, str, str | None, str]] = [
        ("candidate_catalog_review_surface", "candidate_ids_present_in_lesson_index", None, "candidate_catalog"),
        ("active_allowlist_review_surface", "candidate_ids_match_active_allowlist", None, "active_allowlist_review"),
        ("lesson_index_alignment_surface", "candidate_ids_present_in_lesson_index", None, "lesson_index_alignment"),
        ("local_packaging_review_surface", "local_packaging_consolidation_checker_passes", "local_packaging_review_dashboard_card", "local_packaging_review"),
        ("release_dashboard_publication_surface", "docs_ssot_clean", "publication_prep_review_dashboard_card", "release_dashboard_publication"),
        ("registry_share_handoff_surface", "docs_ssot_clean", "registry_share_review_dashboard_card", "registry_share_handoff"),
    ]
    return [
        {
            "id": row_id,
            "source_review_gate": review_gate,
            "source_dashboard_row": dashboard_row,
            "surface_lane": lane,
            **common,
        }
        for row_id, review_gate, dashboard_row, lane in rows
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
        SURFACE,
        SOURCE_DASHBOARD,
        SOURCE_PREP,
        UI_MODULE,
        APP_JS,
        INDEX_HTML,
        STYLES_CSS,
        RUNNER,
        SOURCE_DASHBOARD_CHECK,
    ]:
        require(path)


def check_docs() -> None:
    tokens = [
        "STUDIO_LESSON_PUBLICATION_REVIEW_SURFACE_V1",
        "ddn.studio.lesson_publication_review_surface.v1",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        "닫힘-동작",
        "surface rows: 6/6 = 100%",
        "전체 초장기 계획: 18/18 = 100%",
        "현재 스테이지: 새 마-3 개발 계획 6/8 = 75%",
        "ROADMAP_V2 product behavior baseline: 90/90 = 100%",
        "studio_lesson_publication_review_surface_runner.mjs",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, tokens)
    require_contains(REPORT, tokens[:10])
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_LESSON_PUBLICATION_REVIEW_SURFACE_V1",
            "studio_lesson_publication_review_surface_runner.mjs",
            "surface rows: 6/6 = 100%",
            "전체 초장기 계획: 18/18 = 100%",
            "현재 스테이지: 새 마-3 개발 계획 6/8 = 75%",
            "ROADMAP_V2 product behavior baseline: 90/90 = 100%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_ui_source() -> None:
    require_contains(
        UI_MODULE,
        [
            "ddn.studio.lesson_publication_review_surface.v1",
            "buildLessonPublicationReviewSurface",
            "formatLessonPublicationReviewSurfaceText",
            "renderLessonPublicationReviewSurface",
            "product_ui_change: true",
            "public_upload_claim: false",
            "registry_publish_claim: false",
            "active_allowlist_mutation: false",
            "super_long_behavior_closed: 18",
            "current_stage_percent: 75",
            "roadmap_v2_percent: 100",
        ],
    )
    require_contains(
        APP_JS,
        [
            "studio_lesson_publication_review_surface.js",
            "publishLessonPublicationReviewSurface",
            "__SEAMGRIM_LESSON_PUBLICATION_REVIEW_SURFACE__",
            "buildLessonPublicationReviewSurface",
        ],
    )
    require_contains(
        INDEX_HTML,
        [
            "lesson-publication-review-surface",
            "data-lesson-publication-review-surface",
        ],
    )
    require_contains(
        STYLES_CSS,
        [
            ".lesson-publication-review-surface",
            ".lesson-publication-surface-btn.active",
        ],
    )
    require_contains(
        RUNNER,
        [
            "studio_lesson_publication_review_surface: ok",
            "data-lesson-publication-status='lesson_publication_review_surface_ready'",
            "public_upload_claim",
            "active_allowlist_mutation",
        ],
    )


def check_contract_and_surface() -> None:
    contract = load_json(CONTRACT)
    expected_contract = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_lesson_publication_review_surface_v1",
        "kind": "studio_lesson_publication_review_surface",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "lesson_publication_review_surface_claim": True,
        "public_upload_claim": False,
        "registry_publish_claim": False,
        "publication_snapshot_emit_claim": False,
        "release_approval_claim": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "closed_by": "STUDIO_LESSON_PUBLICATION_REVIEW_SURFACE_V1",
        "based_on": "STUDIO_RELEASE_REVIEW_PACKET_DASHBOARD_V1",
        "surface_row_count": 6,
        "candidate_count": 12,
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "surface_rows_closed": 6,
        "surface_rows_total": 6,
        "surface_rows_percent": 100,
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "current_stage_closed": 6,
        "current_stage_total": 8,
        "current_stage_percent": 75,
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
        "solutions/seamgrim_ui_mvp/ui/studio_lesson_publication_review_surface.js",
        "solutions/seamgrim_ui_mvp/ui/app.js",
        "solutions/seamgrim_ui_mvp/ui/index.html",
        "solutions/seamgrim_ui_mvp/ui/styles.css",
    ]:
        if file not in contract.get("changed_product_files", []):
            fail(f"contract missing changed product file: {file}")

    surface = load_json(SURFACE)
    if surface.get("schema") != "ddn.studio.lesson_publication_review_surface.v1":
        fail(f"surface schema mismatch: {surface.get('schema')!r}")
    if surface.get("work_item") != "STUDIO_LESSON_PUBLICATION_REVIEW_SURFACE_V1":
        fail(f"surface work item mismatch: {surface.get('work_item')!r}")
    for flag, expected in (
        ("runtime_claim", False),
        ("product_code_change", True),
        ("product_ui_change", True),
        ("public_upload_claim", False),
        ("registry_publish_claim", False),
        ("github_release_claim", False),
        ("publication_snapshot_emit_claim", False),
        ("artifact_signing_claim", False),
        ("release_approval_claim", False),
        ("release_execution_claim", False),
        ("public_release_claim", False),
        ("lesson_schema_change", False),
        ("active_allowlist_mutation", False),
        ("cloud_sync_claim", False),
        ("account_setup_claim", False),
        ("permission_system_claim", False),
    ):
        if surface.get(flag) is not expected:
            fail(f"surface {flag} expected {expected!r}, got {surface.get(flag)!r}")
    if surface.get("candidate_count") != 12:
        fail(f"candidate count mismatch: {surface.get('candidate_count')!r}")
    if len(surface.get("candidate_lesson_ids", [])) != 12:
        fail("surface candidate lesson list must contain 12 ids")
    if surface.get("surface_rows") != expected_rows():
        fail(f"surface rows mismatch: {surface.get('surface_rows')!r}")
    if surface.get("progress") != {
        "super_long_behavior_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "current_stage_closed": 6,
        "current_stage_total": 8,
        "current_stage_percent": 75,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
    }:
        fail(f"progress mismatch: {surface.get('progress')!r}")
    if surface.get("closure_tier") != "닫힘-동작":
        fail(f"closure tier mismatch: {surface.get('closure_tier')!r}")
    if surface.get("next_item") != NEXT:
        fail(f"next item mismatch: {surface.get('next_item')!r}")


def check_source_alignment() -> None:
    dashboard = load_json(SOURCE_DASHBOARD)
    prep = load_json(SOURCE_PREP)
    if dashboard.get("schema") != "ddn.studio.release_review_packet_dashboard.v1":
        fail(f"source dashboard schema mismatch: {dashboard.get('schema')!r}")
    if prep.get("schema") != "ddn.studio.public_lesson_publication_prep.v1":
        fail(f"source prep schema mismatch: {prep.get('schema')!r}")
    if dashboard.get("next_item") != "STUDIO_LESSON_PUBLICATION_REVIEW_SURFACE_V1":
        fail(f"source dashboard next item mismatch: {dashboard.get('next_item')!r}")
    if dashboard.get("progress", {}).get("super_long_behavior_closed") != 18:
        fail(f"source dashboard progress mismatch: {dashboard.get('progress')!r}")
    if dashboard.get("progress", {}).get("roadmap_v2_behavior_closed") != 90:
        fail(f"source dashboard roadmap closed mismatch: {dashboard.get('progress')!r}")
    if dashboard.get("progress", {}).get("roadmap_v2_percent") != 100:
        fail(f"source dashboard roadmap percent mismatch: {dashboard.get('progress')!r}")
    if prep.get("candidate_count") != 12:
        fail(f"source prep candidate count mismatch: {prep.get('candidate_count')!r}")
    if len(prep.get("candidate_lesson_ids", [])) != 12:
        fail("source prep candidate lesson list must contain 12 ids")

    prep_gates = set(prep.get("review_gates", []))
    required_gates = {row["source_review_gate"] for row in expected_rows()}
    missing_gates = sorted(required_gates - prep_gates)
    if missing_gates:
        fail(f"missing publication prep review gates: {missing_gates!r}")

    dashboard_ids = [row.get("id") for row in dashboard.get("dashboard_rows", [])]
    required_dashboard_ids = [
        row["source_dashboard_row"]
        for row in expected_rows()
        if row["source_dashboard_row"] is not None
    ]
    missing_dashboard = [row_id for row_id in required_dashboard_ids if row_id not in dashboard_ids]
    if missing_dashboard:
        fail(f"missing dashboard row anchors: {missing_dashboard!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected_stdout = [
        "STUDIO_LESSON_PUBLICATION_REVIEW_SURFACE_V1",
        "studio lesson publication review surface behavior sealed",
        "lesson publication review surface schema: ddn.studio.lesson_publication_review_surface.v1",
        "surface rows: 6/6 = 100%",
        "overall super-long behavior: 18/18 = 100%",
        "current stage: 6/8 = 75%",
        "roadmap v2 behavior: 90/90 = 100%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_lesson_publication_review_surface_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected_stdout:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_lesson_publication_review_surface_v1"],
        ["node", "tests/studio_lesson_publication_review_surface_runner.mjs"],
        ["python", "tests/run_studio_release_review_packet_dashboard_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_ui_source()
    check_contract_and_surface()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_lesson_publication_review_surface_check: ok")


if __name__ == "__main__":
    main()
