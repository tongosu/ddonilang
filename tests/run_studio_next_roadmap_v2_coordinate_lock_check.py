from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1.md"
REPORT = ROOT / "docs" / "studio" / "NEXT_ROADMAP_V2_COORDINATE_LOCK_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_next_roadmap_v2_coordinate_lock_v1"
LOCK = PACK / "next_roadmap_v2_coordinate_lock.detjson"
CHECKER = ROOT / "tests" / "run_studio_next_roadmap_v2_coordinate_lock_check.py"
UI = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_next_roadmap_v2_coordinate_lock.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
RUNNER = ROOT / "tests" / "studio_next_roadmap_v2_coordinate_lock_runner.mjs"
SOURCE_PREP = ROOT / "pack" / "studio_benchmark_baseline_prep_dry_run_v1" / "benchmark_baseline_prep_dry_run.detjson"
SOURCE_REBASE = ROOT / "pack" / "studio_post_super_long_rebase_v1" / "post_super_long_rebase.detjson"
NEXT_STATE = "AWAIT_NEXT_DEVELOPMENT_SELECTION"


def fail(message: str) -> None:
    print(f"studio_next_roadmap_v2_coordinate_lock_check: FAIL: {message}", file=sys.stderr)
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


def expected_decisions() -> list[dict[str, object]]:
    return [
        {
            "id": "default_next_coordinate",
            "decision": "마-3",
            "reason": "Studio-first productization remains the default next ROADMAP_V2 coordinate after the follow-up queue closes.",
            "locked": True,
            "opens_new_queue": False,
            "runtime_claim": False,
        },
        {
            "id": "studio_first_continuity",
            "decision": "preserve_studio_first",
            "reason": "The closed super-long and post-super-long evidence stays centered on Studio productization.",
            "locked": True,
            "opens_new_queue": False,
            "runtime_claim": False,
        },
        {
            "id": "post_followup_denominator_closed",
            "decision": "8/8_closed",
            "reason": "The post-super-long follow-up denominator is complete and should not be extended implicitly.",
            "locked": True,
            "opens_new_queue": False,
            "runtime_claim": False,
        },
        {
            "id": "release_execution_still_approval_gated",
            "decision": "approval_phrase_required",
            "reason": "Release execution remains blocked without the exact approval phrase.",
            "locked": True,
            "opens_new_queue": False,
            "runtime_claim": False,
        },
        {
            "id": "next_queue_requires_explicit_selection",
            "decision": NEXT_STATE,
            "reason": "A new denominator or implementation queue requires the next explicit development selection.",
            "locked": True,
            "opens_new_queue": False,
            "runtime_claim": False,
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
        LOCK,
        CHECKER,
        UI,
        APP,
        HTML,
        STYLES,
        RUNNER,
        SOURCE_PREP,
        SOURCE_REBASE,
        ROOT / "tests" / "run_studio_benchmark_baseline_prep_dry_run_check.py",
        ROOT / "tests" / "run_studio_post_super_long_rebase_check.py",
    ]:
        require(path)


def check_docs() -> None:
    doc_tokens = [
        "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1",
        "ddn.studio.next_roadmap_v2_coordinate_lock.v1",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        "Every decision keeps `locked=true`, `opens_new_queue=false`, and `runtime_claim=false`",
        "default_next_coordinate",
        "studio_first_continuity",
        "post_followup_denominator_closed",
        "release_execution_still_approval_gated",
        "next_queue_requires_explicit_selection",
        "Product Changes",
        "tests/studio_next_roadmap_v2_coordinate_lock_runner.mjs",
        "작업 단위: 6/6 = 100% (`닫힘-동작`)",
        "후속 장기 계획: 8/8 = 100%",
        "마줄기 후속 8/8 = 100%",
        "ROADMAP_V2 product behavior baseline: 88/90 = 98%",
        NEXT_STATE,
        "docs/ssot/**",
    ]
    require_contains(DOC, doc_tokens)
    require_contains(
        REPORT,
        [
            "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1",
            "ddn.studio.next_roadmap_v2_coordinate_lock.v1",
            "This is product UI behavior plus coordinate-lock evidence",
            "solutions/seamgrim_ui_mvp/ui/studio_next_roadmap_v2_coordinate_lock.js",
            "tests/studio_next_roadmap_v2_coordinate_lock_runner.mjs",
            "작업 단위: 6/6 = 100% (`닫힘-동작`)",
            "후속 장기 계획: 8/8 = 100%",
            "ROADMAP_V2 product behavior baseline: 88/90 = 98%",
            NEXT_STATE,
        ],
    )
    require_contains(
        INDEX,
        [
            "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1",
            "docs/studio/NEXT_ROADMAP_V2_COORDINATE_LOCK_V1.md",
            "pack/studio_next_roadmap_v2_coordinate_lock_v1",
            "tests/run_studio_next_roadmap_v2_coordinate_lock_check.py",
        ],
    )
    require_contains(
        ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md",
        [
            "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1",
            "ddn.studio.next_roadmap_v2_coordinate_lock.v1",
            "post-super-long follow-up 8/8 = 100%",
            "coordinate decisions 5/5 = 100%",
            "ROADMAP_V2 product behavior baseline 88/90 = 98%",
            NEXT_STATE,
        ],
    )
    require_contains(
        ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
        [
            "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1",
            "studio_next_roadmap_v2_coordinate_lock_v1",
            "ddn.studio.next_roadmap_v2_coordinate_lock.v1",
            "node tests/studio_next_roadmap_v2_coordinate_lock_runner.mjs` PASS",
            "현재 스테이지: post-super-long follow-up 8/8 = 100%",
            "ROADMAP_V2 product behavior baseline: 88/90 = 98%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_ui_contract() -> None:
    require_contains(
        UI,
        [
            "DEFAULT_NEXT_ROADMAP_V2_COORDINATE_LOCK_DECISIONS",
            "buildNextRoadmapV2CoordinateLock",
            "formatNextRoadmapV2CoordinateLockText",
            "renderNextRoadmapV2CoordinateLock",
            "ddn.studio.next_roadmap_v2_coordinate_lock.v1",
            "next_roadmap_v2_coordinate_lock_ready",
            "product_ui_change: true",
            "product_code_change: true",
            "current_stage_closed: 8",
            "current_stage_percent: 100",
            "roadmap_v2_behavior_closed: 88",
            "roadmap_v2_percent: 98",
            "new_automatic_queue_claim: false",
            NEXT_STATE,
        ],
    )
    require_contains(
        APP,
        [
            "studio_next_roadmap_v2_coordinate_lock.js",
            "nextRoadmapV2CoordinateLock",
            "publishNextRoadmapV2CoordinateLock",
            "__SEAMGRIM_NEXT_ROADMAP_V2_COORDINATE_LOCK__",
            "__SEAMGRIM_NEXT_ROADMAP_V2_COORDINATE_LOCK_TEXT__",
        ],
    )
    require_contains(
        HTML,
        [
            'id="next-roadmap-v2-coordinate-lock"',
            "data-next-roadmap-v2-coordinate-lock",
            'aria-label="Studio next ROADMAP_V2 coordinate lock"',
        ],
    )
    require_contains(
        STYLES,
        [
            ".next-roadmap-v2-coordinate-lock",
            ".next-roadmap-lock-head",
            ".next-roadmap-lock-progress",
            ".next-roadmap-lock-btn",
            ".next-roadmap-lock-detail",
        ],
    )
    require_contains(
        RUNNER,
        [
            "studio_next_roadmap_v2_coordinate_lock: ok",
            "buildNextRoadmapV2CoordinateLock",
            "formatNextRoadmapV2CoordinateLockText",
            "next_roadmap_v2_coordinate_lock_ready",
            "roadmap_v2_behavior_closed === 88",
            "current_stage_percent === 100",
            "next_queue_requires_explicit_selection",
        ],
    )


def check_contract_and_lock() -> None:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_next_roadmap_v2_coordinate_lock_v1",
        "kind": "studio_next_roadmap_v2_coordinate_lock",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "next_roadmap_v2_coordinate_lock_claim": True,
        "new_automatic_queue_claim": False,
        "benchmark_execution_claim": False,
        "performance_baseline_generation_claim": False,
        "performance_baseline_publication_claim": False,
        "lts_certification_claim": False,
        "release_approval_claim": False,
        "release_execution_claim": False,
        "public_release_claim": False,
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
        "closed_by": "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1",
        "based_on": "STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1",
        "coordinate_lock_manifest": "pack/studio_next_roadmap_v2_coordinate_lock_v1/next_roadmap_v2_coordinate_lock.detjson",
        "source_benchmark_baseline_prep_dry_run": "pack/studio_benchmark_baseline_prep_dry_run_v1/benchmark_baseline_prep_dry_run.detjson",
        "source_post_super_long_rebase": "pack/studio_post_super_long_rebase_v1/post_super_long_rebase.detjson",
        "decision_count": 5,
        "selected_default_coordinate": "마-3",
        "next_state": NEXT_STATE,
        "all_decisions_locked": True,
        "all_decisions_open_new_queue": False,
        "all_decisions_runtime_claim": False,
        "primary_coordinate": "마-3",
        "support_coordinate": "타-3",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "work_unit_percent": 100,
        "post_super_long_closed": 8,
        "post_super_long_total": 8,
        "post_super_long_percent": 100,
        "ma_followup_closed": 8,
        "ma_followup_total": 8,
        "ma_followup_percent": 100,
        "ha3_followup_closed": 2,
        "ha3_followup_total": 2,
        "ha3_followup_percent": 100,
        "ma3_closed": 4,
        "ma3_total": 4,
        "ma3_percent": 100,
        "ta3_followup_closed": 2,
        "ta3_followup_total": 2,
        "ta3_followup_percent": 100,
        "roadmap_v2_behavior_closed": 88,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 98,
        "browser_runner": "tests/studio_next_roadmap_v2_coordinate_lock_runner.mjs",
        "next_item": None,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    lock = load_json(LOCK)
    if lock.get("schema") != "ddn.studio.next_roadmap_v2_coordinate_lock.v1":
        fail(f"lock schema mismatch: {lock.get('schema')!r}")
    if lock.get("work_item") != "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1":
        fail(f"lock work item mismatch: {lock.get('work_item')!r}")
    for flag in (
        "runtime_claim",
        "new_automatic_queue_claim",
        "benchmark_execution_claim",
        "performance_baseline_generation_claim",
        "performance_baseline_publication_claim",
        "lts_certification_claim",
        "release_approval_claim",
        "release_execution_claim",
        "public_release_claim",
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
        if lock.get(flag) is not False:
            fail(f"lock {flag} expected false, got {lock.get(flag)!r}")
    for flag in ("product_code_change", "product_ui_change"):
        if lock.get(flag) is not True:
            fail(f"lock {flag} expected true, got {lock.get(flag)!r}")
    if lock.get("closure_tier") != "닫힘-동작":
        fail(f"closure tier mismatch: {lock.get('closure_tier')!r}")
    if lock.get("browser_runner") != "tests/studio_next_roadmap_v2_coordinate_lock_runner.mjs":
        fail(f"browser runner mismatch: {lock.get('browser_runner')!r}")
    changed = lock.get("changed_product_files", [])
    if len(changed) != 5:
        fail(f"changed product files count mismatch: {changed!r}")
    for rel_path in changed:
        require(ROOT / rel_path)
    if lock.get("selected_default_coordinate") != "마-3":
        fail(f"selected default coordinate mismatch: {lock.get('selected_default_coordinate')!r}")
    if lock.get("next_state") != NEXT_STATE:
        fail(f"next state mismatch: {lock.get('next_state')!r}")
    if lock.get("coordinate_decisions") != expected_decisions():
        fail(f"coordinate decisions mismatch: {lock.get('coordinate_decisions')!r}")
    if lock.get("post_super_long_plan") != {"closed": 8, "total": 8, "percent": 100, "status": "sealed"}:
        fail(f"post-super-long progress mismatch: {lock.get('post_super_long_plan')!r}")
    if lock.get("roadmap_v2_product_behavior") != {"closed": 88, "total": 90, "percent": 98}:
        fail(f"roadmap behavior progress mismatch: {lock.get('roadmap_v2_product_behavior')!r}")
    if lock.get("next_item") is not None:
        fail(f"next item expected null, got {lock.get('next_item')!r}")


def check_source_alignment() -> None:
    prep = load_json(SOURCE_PREP)
    rebase = load_json(SOURCE_REBASE)
    if prep.get("next_item") != "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1":
        fail(f"prep source next item mismatch: {prep.get('next_item')!r}")
    if prep.get("schema") != "ddn.studio.benchmark_baseline_prep_dry_run.v1":
        fail(f"prep source schema mismatch: {prep.get('schema')!r}")
    if prep.get("roadmap_v2_product_behavior", {}).get("closed") != 87:
        fail(f"prep source roadmap closed mismatch: {prep.get('roadmap_v2_product_behavior')!r}")
    if prep.get("roadmap_v2_product_behavior", {}).get("percent") != 97:
        fail(f"prep source roadmap percent mismatch: {prep.get('roadmap_v2_product_behavior')!r}")
    for flag in ("benchmark_execution_claim", "performance_baseline_generation_claim", "release_execution_claim", "public_upload_claim", "cloud_sync_claim", "account_setup_claim", "permission_system_claim"):
        if prep.get(flag) is not False:
            fail(f"prep source {flag} expected false, got {prep.get(flag)!r}")

    if rebase.get("schema") != "ddn.studio.post_super_long_rebase.v1":
        fail(f"rebase source schema mismatch: {rebase.get('schema')!r}")
    plan = rebase.get("post_super_long_plan", {})
    items = plan.get("items", [])
    if len(items) != 8:
        fail(f"post-super-long item count mismatch: {len(items)!r}")
    expected_last = {
        "id": "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1",
        "coordinate": "마-3",
        "status": "planned",
        "claim": "next ROADMAP_V2 coordinate lock",
    }
    if items[-1] != expected_last:
        fail(f"post-super-long last item mismatch: {items[-1]!r}")
    if rebase.get("super_long_plan") != {"closed": 18, "total": 18, "percent": 100, "status": "sealed"}:
        fail(f"super-long source plan mismatch: {rebase.get('super_long_plan')!r}")

    lock = load_json(LOCK)
    expected_preflights = [
        "node tests/studio_next_roadmap_v2_coordinate_lock_runner.mjs",
        "python tests/run_studio_benchmark_baseline_prep_dry_run_check.py",
        "python tests/run_studio_post_super_long_rebase_check.py",
        "git status --short -- docs/ssot",
    ]
    if lock.get("preflight_commands") != expected_preflights:
        fail(f"preflight commands mismatch: {lock.get('preflight_commands')!r}")
    required_blocked = {
        "new_automatic_queue_open",
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
    }
    if set(lock.get("blocked_actions", [])) != required_blocked:
        fail(f"blocked actions mismatch: {lock.get('blocked_actions')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1",
        "studio next ROADMAP_V2 coordinate lock sealed",
        "coordinate lock schema: ddn.studio.next_roadmap_v2_coordinate_lock.v1",
        "selected coordinate: 마-3",
        "follow-up plan: 8/8 = 100%",
        "roadmap v2 behavior: 88/90 = 98%",
        f"next state: {NEXT_STATE}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_next_roadmap_v2_coordinate_lock_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_next_roadmap_v2_coordinate_lock_v1"],
        ["node", "tests/studio_next_roadmap_v2_coordinate_lock_runner.mjs"],
        ["python", "tests/run_studio_benchmark_baseline_prep_dry_run_check.py"],
        ["python", "tests/run_studio_post_super_long_rebase_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_ui_contract()
    check_contract_and_lock()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_next_roadmap_v2_coordinate_lock_check: ok")


if __name__ == "__main__":
    main()
