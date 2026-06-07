from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_V1.md"
REPORT = ROOT / "docs" / "studio" / "MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_ma3_next_development_queue_rebase_v1"
QUEUE = PACK / "ma3_next_development_queue_rebase.detjson"
CHECKER = ROOT / "tests" / "run_studio_ma3_next_development_queue_rebase_check.py"
UI = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_ma3_next_development_queue_rebase.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
RUNNER = ROOT / "tests" / "studio_ma3_next_development_queue_rebase_runner.mjs"
SOURCE_LOCK = ROOT / "pack" / "studio_next_roadmap_v2_coordinate_lock_v1" / "next_roadmap_v2_coordinate_lock.detjson"
NEXT = "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1"


def fail(message: str) -> None:
    print(f"studio_ma3_next_development_queue_rebase_check: FAIL: {message}", file=sys.stderr)
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


def expected_items() -> list[dict[str, str]]:
    return [
        {
            "id": "STUDIO_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_V1",
            "coordinate": "마-3",
            "status": "closed",
            "claim": "explicit next development denominator lock",
        },
        {
            "id": "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1",
            "coordinate": "하-3",
            "status": "next",
            "claim": "local teacher feedback preview surface only",
        },
        {
            "id": "STUDIO_CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1",
            "coordinate": "하-3",
            "status": "planned",
            "claim": "local classroom operations panel preview only",
        },
        {
            "id": "STUDIO_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1",
            "coordinate": "타-3",
            "status": "planned",
            "claim": "local benchmark baseline snapshot only",
        },
        {
            "id": "STUDIO_RELEASE_REVIEW_PACKET_DASHBOARD_V1",
            "coordinate": "마-3",
            "status": "planned",
            "claim": "approval-safe release review dashboard only",
        },
        {
            "id": "STUDIO_LESSON_PUBLICATION_REVIEW_SURFACE_V1",
            "coordinate": "마-3",
            "status": "planned",
            "claim": "local lesson publication review surface only",
        },
        {
            "id": "STUDIO_MA3_REGRESSION_GATE_MATRIX_V1",
            "coordinate": "타-3",
            "status": "planned",
            "claim": "local regression gate matrix only",
        },
        {
            "id": "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1",
            "coordinate": "마-3",
            "status": "planned",
            "claim": "next queue coordinate lock only",
        },
    ]


def check_required_files() -> None:
    for path in [
        DOC,
        REPORT,
        INDEX,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        QUEUE,
        CHECKER,
        UI,
        APP,
        HTML,
        STYLES,
        RUNNER,
        SOURCE_LOCK,
        ROOT / "tests" / "run_studio_next_roadmap_v2_coordinate_lock_check.py",
    ]:
        require(path)


def check_docs() -> None:
    doc_tokens = [
        "STUDIO_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_V1",
        "ddn.studio.ma3_next_development_queue_rebase.v1",
        "Primary coordinate: `마-3`",
        "Support coordinates: `하-3`, `타-3`",
        "STUDIO_TEACHER_FEEDBACK_SURFACE_PREVIEW_V1",
        "STUDIO_CLASSROOM_OPERATIONS_PANEL_PREVIEW_V1",
        "STUDIO_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_V1",
        "STUDIO_RELEASE_REVIEW_PACKET_DASHBOARD_V1",
        "STUDIO_LESSON_PUBLICATION_REVIEW_SURFACE_V1",
        "STUDIO_MA3_REGRESSION_GATE_MATRIX_V1",
        "STUDIO_MA3_NEXT_QUEUE_COORDINATE_LOCK_V1",
        "Product Changes",
        "tests/studio_ma3_next_development_queue_rebase_runner.mjs",
        "작업 단위: 6/6 = 100% (`닫힘-동작`)",
        "새 마-3 개발 계획: 1/8 = 13%",
        "마줄기 신규 1/8 = 13%",
        "마-3 신규 1/4 = 25%",
        "ROADMAP_V2 product behavior baseline: 89/90 = 99%",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, doc_tokens)
    require_contains(
        REPORT,
        [
            "STUDIO_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_V1",
            "ddn.studio.ma3_next_development_queue_rebase.v1",
            "This is product UI behavior plus queue rebase evidence",
            "solutions/seamgrim_ui_mvp/ui/studio_ma3_next_development_queue_rebase.js",
            "tests/studio_ma3_next_development_queue_rebase_runner.mjs",
            "작업 단위: 6/6 = 100% (`닫힘-동작`)",
            "새 마-3 개발 계획: 1/8 = 13%",
            "ROADMAP_V2 product behavior baseline: 89/90 = 99%",
            NEXT,
        ],
    )
    require_contains(
        INDEX,
        [
            "STUDIO_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_V1",
            "docs/studio/MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_V1.md",
            "pack/studio_ma3_next_development_queue_rebase_v1",
            "tests/run_studio_ma3_next_development_queue_rebase_check.py",
        ],
    )
    require_contains(
        ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md",
        [
            "STUDIO_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_V1",
            "ddn.studio.ma3_next_development_queue_rebase.v1",
            NEXT,
            "새 마-3 개발 계획 1/8 = 13%",
            "ROADMAP_V2 product behavior baseline 89/90 = 99%",
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "STUDIO_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_V1",
            "studio_ma3_next_development_queue_rebase_v1",
            "ddn.studio.ma3_next_development_queue_rebase.v1",
            "node tests/studio_ma3_next_development_queue_rebase_runner.mjs` PASS",
            "전체 초장기 계획: 18/18 = 100%",
            "새 마-3 개발 계획: 1/8 = 13%",
            "ROADMAP_V2 product behavior baseline: 89/90 = 99%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_ui_contract() -> None:
    require_contains(
        UI,
        [
            "DEFAULT_MA3_NEXT_DEVELOPMENT_QUEUE_ROWS",
            "buildMa3NextDevelopmentQueueRebase",
            "formatMa3NextDevelopmentQueueRebaseText",
            "renderMa3NextDevelopmentQueueRebase",
            "ddn.studio.ma3_next_development_queue_rebase.v1",
            "ma3_next_development_queue_rebased",
            "product_ui_change: true",
            "product_code_change: true",
            "current_stage_closed: 1",
            "current_stage_percent: 13",
            "roadmap_v2_behavior_closed: 89",
            "roadmap_v2_percent: 99",
            "new_automatic_queue_claim: false",
            NEXT,
        ],
    )
    require_contains(
        APP,
        [
            "studio_ma3_next_development_queue_rebase.js",
            "ma3NextDevelopmentQueueRebase",
            "publishMa3NextDevelopmentQueueRebase",
            "__SEAMGRIM_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE__",
            "__SEAMGRIM_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_TEXT__",
        ],
    )
    require_contains(
        HTML,
        [
            'id="ma3-next-development-queue-rebase"',
            "data-ma3-next-development-queue-rebase",
            'aria-label="Studio MA3 next development queue rebase"',
        ],
    )
    require_contains(
        STYLES,
        [
            ".ma3-next-development-queue-rebase",
            ".ma3-dev-queue-head",
            ".ma3-dev-queue-progress",
            ".ma3-dev-queue-btn",
            ".ma3-dev-queue-detail",
        ],
    )
    require_contains(
        RUNNER,
        [
            "studio_ma3_next_development_queue_rebase: ok",
            "buildMa3NextDevelopmentQueueRebase",
            "formatMa3NextDevelopmentQueueRebaseText",
            "ma3_next_development_queue_rebased",
            "roadmap_v2_behavior_closed === 89",
            "current_stage_percent === 13",
            NEXT,
        ],
    )


def check_contract_and_queue() -> None:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_ma3_next_development_queue_rebase_v1",
        "kind": "studio_ma3_next_development_queue_rebase",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "ma3_next_development_queue_rebase_claim": True,
        "explicit_next_development_selection_claim": True,
        "new_automatic_queue_claim": False,
        "release_approval_claim": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "benchmark_execution_claim": False,
        "performance_baseline_generation_claim": False,
        "performance_baseline_publication_claim": False,
        "lts_certification_claim": False,
        "github_release_claim": False,
        "public_upload_claim": False,
        "registry_publish_claim": False,
        "public_link_creation_claim": False,
        "install_enablement_claim": False,
        "publication_snapshot_emit_claim": False,
        "archive_generation_claim": False,
        "publication_checksum_generation_claim": False,
        "artifact_signing_claim": False,
        "cloud_sync_claim": False,
        "account_setup_claim": False,
        "permission_system_claim": False,
        "result_replay_claim": False,
        "closed_by": "STUDIO_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_V1",
        "based_on": "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1",
        "queue_manifest": "pack/studio_ma3_next_development_queue_rebase_v1/ma3_next_development_queue_rebase.detjson",
        "source_coordinate_lock": "pack/studio_next_roadmap_v2_coordinate_lock_v1/next_roadmap_v2_coordinate_lock.detjson",
        "queue_item_count": 8,
        "queue_closed": 1,
        "queue_total": 8,
        "queue_percent": 13,
        "selected_default_coordinate": "마-3",
        "primary_coordinate": "마-3",
        "support_coordinates": ["하-3", "타-3"],
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "work_unit_percent": 100,
        "previous_followup_closed": 8,
        "previous_followup_total": 8,
        "previous_followup_percent": 100,
        "ma_new_closed": 1,
        "ma_new_total": 8,
        "ma_new_percent": 13,
        "ma3_new_closed": 1,
        "ma3_new_total": 4,
        "ma3_new_percent": 25,
        "ha3_new_closed": 0,
        "ha3_new_total": 2,
        "ha3_new_percent": 0,
        "ta3_new_closed": 0,
        "ta3_new_total": 2,
        "ta3_new_percent": 0,
        "roadmap_v2_behavior_closed": 89,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 99,
        "browser_runner": "tests/studio_ma3_next_development_queue_rebase_runner.mjs",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    queue = load_json(QUEUE)
    if queue.get("schema") != "ddn.studio.ma3_next_development_queue_rebase.v1":
        fail(f"queue schema mismatch: {queue.get('schema')!r}")
    if queue.get("work_item") != "STUDIO_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_V1":
        fail(f"queue work item mismatch: {queue.get('work_item')!r}")
    for flag in (
        "runtime_claim",
        "new_automatic_queue_claim",
        "release_approval_claim",
        "release_execution_claim",
        "public_release_claim",
        "benchmark_execution_claim",
        "performance_baseline_generation_claim",
        "performance_baseline_publication_claim",
        "lts_certification_claim",
        "github_release_claim",
        "public_upload_claim",
        "registry_publish_claim",
        "public_link_creation_claim",
        "install_enablement_claim",
        "publication_snapshot_emit_claim",
        "archive_generation_claim",
        "publication_checksum_generation_claim",
        "artifact_signing_claim",
        "cloud_sync_claim",
        "account_setup_claim",
        "permission_system_claim",
        "result_replay_claim",
    ):
        if queue.get(flag) is not False:
            fail(f"queue {flag} expected false, got {queue.get(flag)!r}")
    for flag in ("product_code_change", "product_ui_change"):
        if queue.get(flag) is not True:
            fail(f"queue {flag} expected true, got {queue.get(flag)!r}")
    if queue.get("closure_tier") != "닫힘-동작":
        fail(f"closure tier mismatch: {queue.get('closure_tier')!r}")
    if queue.get("browser_runner") != "tests/studio_ma3_next_development_queue_rebase_runner.mjs":
        fail(f"browser runner mismatch: {queue.get('browser_runner')!r}")
    changed = queue.get("changed_product_files", [])
    if len(changed) != 5:
        fail(f"changed product files count mismatch: {changed!r}")
    for rel_path in changed:
        require(ROOT / rel_path)
    plan = queue.get("queue_plan")
    if plan.get("closed") != 1 or plan.get("total") != 8 or plan.get("percent") != 13 or plan.get("status") != "open":
        fail(f"queue progress mismatch: {plan!r}")
    if plan.get("items") != expected_items():
        fail(f"queue items mismatch: {plan.get('items')!r}")
    if queue.get("previous_followup_plan") != {"closed": 8, "total": 8, "percent": 100, "status": "sealed"}:
        fail(f"previous followup mismatch: {queue.get('previous_followup_plan')!r}")
    if queue.get("roadmap_v2_product_behavior") != {"closed": 89, "total": 90, "percent": 99}:
        fail(f"roadmap behavior progress mismatch: {queue.get('roadmap_v2_product_behavior')!r}")
    if queue.get("next_item") != NEXT:
        fail(f"next item mismatch: {queue.get('next_item')!r}")


def check_source_alignment() -> None:
    lock = load_json(SOURCE_LOCK)
    if lock.get("schema") != "ddn.studio.next_roadmap_v2_coordinate_lock.v1":
        fail(f"source lock schema mismatch: {lock.get('schema')!r}")
    if lock.get("selected_default_coordinate") != "마-3":
        fail(f"source lock coordinate mismatch: {lock.get('selected_default_coordinate')!r}")
    if lock.get("next_state") != "AWAIT_NEXT_DEVELOPMENT_SELECTION":
        fail(f"source lock state mismatch: {lock.get('next_state')!r}")
    if lock.get("next_item") is not None:
        fail(f"source lock next item expected null, got {lock.get('next_item')!r}")
    if lock.get("roadmap_v2_product_behavior", {}).get("closed") != 88:
        fail(f"source lock roadmap closed mismatch: {lock.get('roadmap_v2_product_behavior')!r}")
    if lock.get("roadmap_v2_product_behavior", {}).get("percent") != 98:
        fail(f"source lock roadmap percent mismatch: {lock.get('roadmap_v2_product_behavior')!r}")
    if lock.get("post_super_long_plan") != {"closed": 8, "total": 8, "percent": 100, "status": "sealed"}:
        fail(f"source lock followup mismatch: {lock.get('post_super_long_plan')!r}")
    for flag in ("runtime_claim", "new_automatic_queue_claim", "release_execution_claim", "public_upload_claim", "cloud_sync_claim", "account_setup_claim", "permission_system_claim"):
        if lock.get(flag) is not False:
            fail(f"source lock {flag} expected false, got {lock.get(flag)!r}")
    for flag in ("product_code_change", "product_ui_change"):
        if lock.get(flag) is not True:
            fail(f"source lock {flag} expected true, got {lock.get(flag)!r}")

    queue = load_json(QUEUE)
    expected_preflights = [
        "node tests/studio_ma3_next_development_queue_rebase_runner.mjs",
        "python tests/run_studio_next_roadmap_v2_coordinate_lock_check.py",
        "git status --short -- docs/ssot",
    ]
    if queue.get("preflight_commands") != expected_preflights:
        fail(f"preflight commands mismatch: {queue.get('preflight_commands')!r}")
    required_blocked = {
        "release_approval",
        "release_execution",
        "github_release_create",
        "public_upload",
        "registry_publish",
        "public_link_create",
        "package_install_enable",
        "publication_snapshot_emit",
        "archive_generation",
        "checksum_manifest_generation_for_publication",
        "artifact_signing",
        "benchmark_execution",
        "performance_baseline_generation",
        "performance_baseline_publication",
        "lts_certification_publish",
        "cloud_sync",
        "account_setup",
        "permission_system_change",
        "parser_frontdoor_change",
        "runtime_surface_change",
        "stdlib_surface_change",
        "lesson_schema_change",
        "active_allowlist_mutation",
    }
    if set(queue.get("blocked_actions", [])) != required_blocked:
        fail(f"blocked actions mismatch: {queue.get('blocked_actions')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_V1",
        "studio ma3 next development queue rebase sealed",
        "queue schema: ddn.studio.ma3_next_development_queue_rebase.v1",
        "super-long plan: 18/18 = 100%",
        "new queue: 1/8 = 13%",
        "roadmap v2 behavior: 89/90 = 99%",
        "selected coordinate: 마-3",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_ma3_next_development_queue_rebase_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_ma3_next_development_queue_rebase_v1"],
        ["node", "tests/studio_ma3_next_development_queue_rebase_runner.mjs"],
        ["python", "tests/run_studio_next_roadmap_v2_coordinate_lock_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_ui_contract()
    check_contract_and_queue()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_ma3_next_development_queue_rebase_check: ok")


if __name__ == "__main__":
    main()
