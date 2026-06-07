from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1.md"
ROADMAP = ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
REPORT = ROOT / "docs" / "studio" / "LOCAL_RELEASE_REHEARSAL_CHECK_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_local_release_rehearsal_check_v1"
REHEARSAL = PACK / "local_release_rehearsal_check.detjson"
UI = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_local_release_rehearsal_check.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
RUNNER = ROOT / "tests" / "studio_local_release_rehearsal_check_runner.mjs"
CHECKER = ROOT / "tests" / "run_studio_local_release_rehearsal_check.py"
SOURCE_APPROVAL = ROOT / "pack" / "studio_public_release_approval_recheck_v1" / "public_release_approval_recheck.detjson"
SOURCE_DRY_RUN = ROOT / "pack" / "studio_release_pre_execution_dry_run_v1" / "dry_run.detjson"
SOURCE_ASSET_PLAN = ROOT / "pack" / "studio_public_release_asset_plan_v1" / "release_assets.detjson"
SOURCE_CONTINUITY = ROOT / "pack" / "studio_release_approval_packet_continuity_v1" / "continuity.detjson"
NEXT = "STUDIO_PUBLICATION_ARTIFACT_DRY_RUN_V1"

EXPECTED_ROWS = [
    "approval_recheck_anchor",
    "pre_execution_dry_run_anchor",
    "asset_plan_anchor",
    "approval_continuity_anchor",
    "publication_artifact_handoff",
]


def fail(message: str) -> None:
    print(f"studio_local_release_rehearsal_check: FAIL: {message}", file=sys.stderr)
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


def check_required_files() -> None:
    for path in [
        DOC,
        ROADMAP,
        DEV_SUMMARY,
        REPORT,
        INDEX,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        REHEARSAL,
        UI,
        APP,
        HTML,
        STYLES,
        RUNNER,
        CHECKER,
        SOURCE_APPROVAL,
        SOURCE_DRY_RUN,
        SOURCE_ASSET_PLAN,
        SOURCE_CONTINUITY,
        ROOT / "tests" / "run_studio_public_release_approval_recheck_check.py",
        ROOT / "tests" / "run_studio_release_pre_execution_dry_run_check.py",
        ROOT / "tests" / "run_studio_public_release_asset_plan_check.py",
    ]:
        require(path)


def check_docs() -> None:
    doc_tokens = [
        "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1",
        "ddn.studio.local_release_rehearsal_check.v1",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        "dry-run-only status remains true",
        "planned assets remain `generated_now=false`",
        "Product Changes",
        "No release approval",
        "No release execution",
        "No archive generation",
        "No checksum generation for publication",
        "작업 단위: 6/6 = 100% (`닫힘-동작`)",
        "초장기 계획: 1시대 5/5 = 100%, 2시대 7/7 = 100%, 3시대 6/6 = 100%, 전체 18/18 = 100%",
        "현재 스테이지: post-super-long follow-up 3/8 = 38%",
        "ROADMAP_V2 product behavior baseline: 90/90 = 100%",
        "node tests/studio_local_release_rehearsal_check_runner.mjs",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, doc_tokens)
    require_contains(
        REPORT,
        [
            "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1",
            "ddn.studio.local_release_rehearsal_check.v1",
            "Primary coordinate: `마-3`",
            "Support coordinate: `타-3`",
            "dry-run-only status remains true",
            "planned assets remain `generated_now=false`",
            "This is product UI behavior plus checker/manifest evidence",
            "No release approval",
            "No release execution",
            "No archive generation",
            "No checksum generation for publication",
            "작업 단위: 6/6 = 100% (`닫힘-동작`)",
            "초장기 계획: 1시대 5/5 = 100%, 2시대 7/7 = 100%, 3시대 6/6 = 100%, 전체 18/18 = 100%",
            "현재 스테이지: post-super-long follow-up 3/8 = 38%",
            "ROADMAP_V2 product behavior baseline: 90/90 = 100%",
            NEXT,
        ],
    )
    require_contains(
        INDEX,
        [
            "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1",
            "docs/studio/LOCAL_RELEASE_REHEARSAL_CHECK_V1.md",
            "pack/studio_local_release_rehearsal_check_v1",
            "tests/run_studio_local_release_rehearsal_check.py",
        ],
    )
    require_contains(
        ROADMAP,
        [
            "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1",
            "ddn.studio.local_release_rehearsal_check.v1",
            NEXT,
            "전체 초장기 계획 18/18 = 100%",
            "post-super-long follow-up 3/8 = 38%",
            "ROADMAP_V2 product behavior baseline 90/90 = 100%",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "Local release rehearsal UI",
            "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1",
            "studio_local_release_rehearsal_check_v1",
            "ddn.studio.local_release_rehearsal_check.v1",
            "node tests/studio_local_release_rehearsal_check_runner.mjs` PASS",
            "현재 스테이지: post-super-long follow-up 3/8 = 38%",
            "ROADMAP_V2 product behavior baseline: 90/90 = 100%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_ui_contract() -> None:
    require_contains(
        UI,
        [
            "DEFAULT_LOCAL_RELEASE_REHEARSAL_ROWS",
            "buildLocalReleaseRehearsalCheck",
            "formatLocalReleaseRehearsalCheckText",
            "renderLocalReleaseRehearsalCheck",
            "ddn.studio.local_release_rehearsal_check.v1",
            "local_release_rehearsal_ready",
            "roadmap_v2_behavior_closed: 90",
            "current_stage_closed: 3",
            "current_stage_percent: 38",
        ],
    )
    for row_id in EXPECTED_ROWS:
        require_contains(UI, [row_id])
    require_contains(
        APP,
        [
            "DEFAULT_LOCAL_RELEASE_REHEARSAL_ROWS",
            "buildLocalReleaseRehearsalCheck",
            "formatLocalReleaseRehearsalCheckText",
            "renderLocalReleaseRehearsalCheck",
            "localReleaseRehearsalCheck",
            "__SEAMGRIM_LOCAL_RELEASE_REHEARSAL_CHECK__",
            "publishLocalReleaseRehearsalCheck",
        ],
    )
    require_contains(
        HTML,
        [
            "id=\"local-release-rehearsal-check\"",
            "data-local-release-rehearsal-check",
            "Studio local release rehearsal check",
        ],
    )
    require_contains(
        STYLES,
        [
            ".local-release-rehearsal-check",
            ".rehearsal-check-head",
            ".rehearsal-check-progress",
            ".rehearsal-check-btn.active",
            ".rehearsal-check-detail",
        ],
    )
    require_contains(
        RUNNER,
        [
            "studio_local_release_rehearsal_check: ok",
            "data-local-release-rehearsal-check-status='local_release_rehearsal_ready'",
            "roadmap_v2_behavior_closed === 90",
            "roadmap_v2_percent === 100",
            "3/8 follow-up",
        ],
    )


def check_contract_and_rehearsal() -> None:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_local_release_rehearsal_check_v1",
        "kind": "studio_local_release_rehearsal_check",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "local_rehearsal_check_claim": True,
        "release_approval_claim": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "archive_generation_claim": False,
        "publication_checksum_generation_claim": False,
        "artifact_signing_claim": False,
        "lts_certification_claim": False,
        "benchmark_execution_claim": False,
        "performance_baseline_claim": False,
        "github_release_claim": False,
        "public_upload_claim": False,
        "registry_publish_claim": False,
        "public_link_creation_claim": False,
        "install_enablement_claim": False,
        "publication_snapshot_emit_claim": False,
        "cloud_sync_claim": False,
        "account_setup_claim": False,
        "permission_system_claim": False,
        "replay_claim": False,
        "closed_by": "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1",
        "based_on": "STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1",
        "rehearsal": "pack/studio_local_release_rehearsal_check_v1/local_release_rehearsal_check.detjson",
        "source_approval_recheck": "pack/studio_public_release_approval_recheck_v1/public_release_approval_recheck.detjson",
        "source_pre_execution_dry_run": "pack/studio_release_pre_execution_dry_run_v1/dry_run.detjson",
        "source_release_asset_plan": "pack/studio_public_release_asset_plan_v1/release_assets.detjson",
        "source_approval_continuity": "pack/studio_release_approval_packet_continuity_v1/continuity.detjson",
        "dry_run_only": True,
        "primary_coordinate": "마-3",
        "support_coordinate": "타-3",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "post_super_long_closed": 3,
        "post_super_long_total": 8,
        "post_super_long_percent": 38,
        "ma_followup_closed": 3,
        "ma_followup_total": 8,
        "ma_followup_percent": 38,
        "ma3_closed": 4,
        "ma3_total": 4,
        "ma3_percent": 100,
        "ta3_closed": 3,
        "ta3_total": 3,
        "ta3_percent": 100,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
        "browser_runner": "tests/studio_local_release_rehearsal_check_runner.mjs",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    rehearsal = load_json(REHEARSAL)
    if rehearsal.get("schema") != "ddn.studio.local_release_rehearsal_check.v1":
        fail(f"rehearsal schema mismatch: {rehearsal.get('schema')!r}")
    if rehearsal.get("work_item") != "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1":
        fail(f"rehearsal work item mismatch: {rehearsal.get('work_item')!r}")
    if rehearsal.get("product_code_change") is not True:
        fail(f"product_code_change expected true, got {rehearsal.get('product_code_change')!r}")
    if rehearsal.get("product_ui_change") is not True:
        fail(f"product_ui_change expected true, got {rehearsal.get('product_ui_change')!r}")
    for flag in (
        "runtime_claim",
        "release_approval_claim",
        "release_execution_claim",
        "public_release_claim",
        "archive_generation_claim",
        "publication_checksum_generation_claim",
        "artifact_signing_claim",
        "lts_certification_claim",
        "benchmark_execution_claim",
        "performance_baseline_claim",
        "github_release_claim",
        "public_upload_claim",
        "registry_publish_claim",
        "public_link_creation_claim",
        "install_enablement_claim",
        "publication_snapshot_emit_claim",
        "cloud_sync_claim",
        "account_setup_claim",
        "permission_system_claim",
        "active_allowlist_mutation",
    ):
        if rehearsal.get(flag) is not False:
            fail(f"rehearsal {flag} expected false, got {rehearsal.get(flag)!r}")
    if rehearsal.get("dry_run_only") is not True:
        fail(f"dry_run_only expected true, got {rehearsal.get('dry_run_only')!r}")
    if rehearsal.get("all_planned_assets_generated_now") is not False:
        fail(f"all planned assets generated flag mismatch: {rehearsal.get('all_planned_assets_generated_now')!r}")
    if rehearsal.get("rehearsal_rows") != EXPECTED_ROWS:
        fail(f"rehearsal rows mismatch: {rehearsal.get('rehearsal_rows')!r}")
    if rehearsal.get("closure_tier") != "닫힘-동작":
        fail(f"closure tier mismatch: {rehearsal.get('closure_tier')!r}")
    if rehearsal.get("browser_runner") != "tests/studio_local_release_rehearsal_check_runner.mjs":
        fail(f"browser runner mismatch: {rehearsal.get('browser_runner')!r}")
    changed_product_files = rehearsal.get("changed_product_files")
    if not isinstance(changed_product_files, list) or len(changed_product_files) != 5:
        fail(f"changed product files mismatch: {changed_product_files!r}")
    for rel in changed_product_files:
        require(ROOT / rel)
    if rehearsal.get("post_super_long_plan") != {"closed": 3, "total": 8, "percent": 38}:
        fail(f"post-super-long progress mismatch: {rehearsal.get('post_super_long_plan')!r}")
    if rehearsal.get("roadmap_v2_product_behavior") != {"closed": 90, "total": 90, "percent": 100}:
        fail(f"roadmap progress mismatch: {rehearsal.get('roadmap_v2_product_behavior')!r}")
    if rehearsal.get("next_item") != NEXT:
        fail(f"next item mismatch: {rehearsal.get('next_item')!r}")


def check_source_alignment() -> None:
    approval = load_json(SOURCE_APPROVAL)
    dry_run = load_json(SOURCE_DRY_RUN)
    asset_plan = load_json(SOURCE_ASSET_PLAN)
    continuity = load_json(SOURCE_CONTINUITY)
    if approval.get("next_item") != "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1":
        fail(f"approval source next item mismatch: {approval.get('next_item')!r}")
    if approval.get("next_state") != "AWAIT_EXPLICIT_RELEASE_APPROVAL":
        fail(f"approval source state mismatch: {approval.get('next_state')!r}")
    if approval.get("progress", {}).get("roadmap_v2_behavior_closed") != 90:
        fail(f"approval source ROADMAP_V2 closed mismatch: {approval.get('progress')!r}")
    if approval.get("progress", {}).get("roadmap_v2_percent") != 100:
        fail(f"approval source ROADMAP_V2 percent mismatch: {approval.get('progress')!r}")
    if dry_run.get("dry_run_only") is not True:
        fail(f"source dry_run_only mismatch: {dry_run.get('dry_run_only')!r}")
    for asset in dry_run.get("planned_assets", []):
        if asset.get("generated_now") is not False:
            fail(f"dry-run planned asset generated_now mismatch: {asset!r}")
    assets = asset_plan.get("assets")
    expected_asset_ids = [
        "studio-static-bundle",
        "studio-local-package-sample",
        "studio-rc-matrix",
        "studio-checksum-manifest",
    ]
    if [asset.get("id") for asset in assets] != expected_asset_ids:
        fail(f"asset plan ids mismatch: {assets!r}")
    if any(asset.get("generated_now") is not False for asset in assets):
        fail(f"asset plan generated_now mismatch: {assets!r}")
    if continuity.get("next_state") != "AWAIT_EXPLICIT_RELEASE_APPROVAL":
        fail(f"continuity state mismatch: {continuity.get('next_state')!r}")
    for source_name, source in (("approval", approval), ("dry_run", dry_run), ("continuity", continuity)):
        for flag in ("release_execution_claim", "public_release_claim", "github_release_claim", "public_upload_claim"):
            if source.get(flag) is not False:
                fail(f"{source_name} {flag} expected false, got {source.get(flag)!r}")
    for flag in ("asset_generation_claim", "public_release_claim", "github_release_claim", "cloud_sync_claim", "public_registry_claim"):
        if asset_plan.get(flag) is not False:
            fail(f"asset_plan {flag} expected false, got {asset_plan.get(flag)!r}")

    rehearsal = load_json(REHEARSAL)
    if rehearsal.get("planned_asset_ids") != expected_asset_ids:
        fail(f"rehearsal asset ids mismatch: {rehearsal.get('planned_asset_ids')!r}")
    expected_preflights = [
        "node tests/studio_local_release_rehearsal_check_runner.mjs",
        "python tests/run_studio_public_release_approval_recheck_check.py",
        "python tests/run_studio_release_pre_execution_dry_run_check.py",
        "python tests/run_studio_public_release_asset_plan_check.py",
        "git status --short -- docs/ssot",
    ]
    if rehearsal.get("preflight_commands") != expected_preflights:
        fail(f"preflight commands mismatch: {rehearsal.get('preflight_commands')!r}")
    required_blocked = {
        "release_approval",
        "release_execution",
        "github_release_create",
        "public_upload",
        "registry_publish",
        "public_link_create",
        "package_install_enable",
        "publication_snapshot_emit",
        "publication_archive_generation",
        "checksum_manifest_generation_for_publication",
        "artifact_signing",
        "cloud_sync",
        "account_setup",
        "permission_system_change",
    }
    if set(rehearsal.get("blocked_in_rehearsal", [])) != required_blocked:
        fail(f"blocked actions mismatch: {rehearsal.get('blocked_in_rehearsal')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1",
        "studio local release rehearsal check sealed",
        "local release rehearsal schema: ddn.studio.local_release_rehearsal_check.v1",
        "dry-run only: true",
        "follow-up plan: 3/8 = 38%",
        "roadmap v2 behavior: 90/90 = 100%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_local_release_rehearsal_check_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_local_release_rehearsal_check_v1"],
        ["node", "tests/studio_local_release_rehearsal_check_runner.mjs"],
        ["python", "tests/run_studio_public_release_approval_recheck_check.py"],
        ["python", "tests/run_studio_release_pre_execution_dry_run_check.py"],
        ["python", "tests/run_studio_public_release_asset_plan_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_ui_contract()
    check_contract_and_rehearsal()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_local_release_rehearsal_check: ok")


if __name__ == "__main__":
    main()
