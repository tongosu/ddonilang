from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_PUBLICATION_ARTIFACT_DRY_RUN_V1.md"
ROADMAP = ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
REPORT = ROOT / "docs" / "studio" / "PUBLICATION_ARTIFACT_DRY_RUN_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_publication_artifact_dry_run_v1"
DRY_RUN = PACK / "publication_artifact_dry_run.detjson"
UI = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_publication_artifact_dry_run.js"
APP = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
RUNNER = ROOT / "tests" / "studio_publication_artifact_dry_run_runner.mjs"
CHECKER = ROOT / "tests" / "run_studio_publication_artifact_dry_run_check.py"
SOURCE_REHEARSAL = ROOT / "pack" / "studio_local_release_rehearsal_check_v1" / "local_release_rehearsal_check.detjson"
SOURCE_ASSET_PLAN = ROOT / "pack" / "studio_public_release_asset_plan_v1" / "release_assets.detjson"
NEXT = "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1"


def fail(message: str) -> None:
    print(f"studio_publication_artifact_dry_run_check: FAIL: {message}", file=sys.stderr)
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


def expected_artifacts() -> list[dict[str, object]]:
    return [
        {
            "id": "studio-static-bundle",
            "planned_path": "build/studio_release/studio-static-bundle.zip",
            "kind": "static_bundle",
            "generated_now": False,
        },
        {
            "id": "studio-local-package-sample",
            "planned_path": "build/studio_release/studio-local-package-sample.detjson",
            "kind": "local_package_sample",
            "generated_now": False,
        },
        {
            "id": "studio-rc-matrix",
            "planned_path": "build/studio_release/studio-rc-matrix.detjson",
            "kind": "release_candidate_matrix",
            "generated_now": False,
        },
        {
            "id": "studio-checksum-manifest",
            "planned_path": "build/studio_release/SHA256SUMS.txt",
            "kind": "checksum_manifest",
            "generated_now": False,
        },
    ]


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
        DRY_RUN,
        UI,
        APP,
        HTML,
        STYLES,
        RUNNER,
        CHECKER,
        SOURCE_REHEARSAL,
        SOURCE_ASSET_PLAN,
        ROOT / "tests" / "run_studio_local_release_rehearsal_check.py",
        ROOT / "tests" / "run_studio_public_release_asset_plan_check.py",
    ]:
        require(path)


def check_docs() -> None:
    doc_tokens = [
        "STUDIO_PUBLICATION_ARTIFACT_DRY_RUN_V1",
        "ddn.studio.publication_artifact_dry_run.v1",
        "Primary coordinate: `타-3`",
        "Support coordinate: `마-3`",
        "Product Changes",
        "Every planned artifact keeps `generated_now=false`",
        "No artifact generation",
        "No archive generation",
        "No checksum generation for publication",
        "No artifact signing",
        "작업 단위: 6/6 = 100% (`닫힘-동작`)",
        "초장기 계획: 1시대 5/5 = 100%, 2시대 7/7 = 100%, 3시대 6/6 = 100%, 전체 18/18 = 100%",
        "현재 스테이지: post-super-long follow-up 4/8 = 50%",
        "ROADMAP_V2 product behavior baseline: 90/90 = 100%",
        "node tests/studio_publication_artifact_dry_run_runner.mjs",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, doc_tokens)
    require_contains(
        REPORT,
        [
            "STUDIO_PUBLICATION_ARTIFACT_DRY_RUN_V1",
            "ddn.studio.publication_artifact_dry_run.v1",
            "Primary coordinate: `타-3`",
            "Support coordinate: `마-3`",
            "The product UI panel lists",
            "Every planned artifact keeps `generated_now=false`",
            "This is product UI behavior plus checker/manifest evidence",
            "No artifact generation",
            "No archive generation",
            "No checksum generation for publication",
            "No artifact signing",
            "작업 단위: 6/6 = 100% (`닫힘-동작`)",
            "초장기 계획: 1시대 5/5 = 100%, 2시대 7/7 = 100%, 3시대 6/6 = 100%, 전체 18/18 = 100%",
            "현재 스테이지: post-super-long follow-up 4/8 = 50%",
            "ROADMAP_V2 product behavior baseline: 90/90 = 100%",
            NEXT,
        ],
    )
    require_contains(
        INDEX,
        [
            "STUDIO_PUBLICATION_ARTIFACT_DRY_RUN_V1",
            "docs/studio/PUBLICATION_ARTIFACT_DRY_RUN_V1.md",
            "pack/studio_publication_artifact_dry_run_v1",
            "tests/run_studio_publication_artifact_dry_run_check.py",
        ],
    )
    require_contains(
        ROADMAP,
        [
            "STUDIO_PUBLICATION_ARTIFACT_DRY_RUN_V1",
            "ddn.studio.publication_artifact_dry_run.v1",
            NEXT,
            "전체 초장기 계획 18/18 = 100%",
            "post-super-long follow-up 4/8 = 50%",
            "ROADMAP_V2 product behavior baseline 90/90 = 100%",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "Publication artifact dry-run UI",
            "STUDIO_PUBLICATION_ARTIFACT_DRY_RUN_V1",
            "studio_publication_artifact_dry_run_v1",
            "ddn.studio.publication_artifact_dry_run.v1",
            "node tests/studio_publication_artifact_dry_run_runner.mjs` PASS",
            "현재 스테이지: post-super-long follow-up 4/8 = 50%",
            "ROADMAP_V2 product behavior baseline: 90/90 = 100%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_ui_contract() -> None:
    require_contains(
        UI,
        [
            "DEFAULT_PUBLICATION_ARTIFACT_DRY_RUN_ROWS",
            "buildPublicationArtifactDryRun",
            "formatPublicationArtifactDryRunText",
            "renderPublicationArtifactDryRun",
            "ddn.studio.publication_artifact_dry_run.v1",
            "publication_artifact_dry_run_ready",
            "roadmap_v2_behavior_closed: 90",
            "current_stage_closed: 4",
            "current_stage_percent: 50",
            "studio-checksum-manifest",
        ],
    )
    require_contains(
        APP,
        [
            "DEFAULT_PUBLICATION_ARTIFACT_DRY_RUN_ROWS",
            "buildPublicationArtifactDryRun",
            "formatPublicationArtifactDryRunText",
            "renderPublicationArtifactDryRun",
            "publicationArtifactDryRun",
            "__SEAMGRIM_PUBLICATION_ARTIFACT_DRY_RUN__",
            "publishPublicationArtifactDryRun",
        ],
    )
    require_contains(
        HTML,
        [
            "id=\"publication-artifact-dry-run\"",
            "data-publication-artifact-dry-run",
            "Studio publication artifact dry-run",
        ],
    )
    require_contains(
        STYLES,
        [
            ".publication-artifact-dry-run",
            ".artifact-dry-run-head",
            ".artifact-dry-run-progress",
            ".artifact-dry-run-btn.active",
            ".artifact-dry-run-detail",
        ],
    )
    require_contains(
        RUNNER,
        [
            "studio_publication_artifact_dry_run: ok",
            "data-publication-artifact-dry-run-status='publication_artifact_dry_run_ready'",
            "roadmap_v2_behavior_closed === 90",
            "roadmap_v2_percent === 100",
            "4/8 follow-up",
        ],
    )


def check_contract_and_dry_run() -> None:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_publication_artifact_dry_run_v1",
        "kind": "studio_publication_artifact_dry_run",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "publication_artifact_dry_run_claim": True,
        "artifact_generation_claim": False,
        "archive_generation_claim": False,
        "publication_checksum_generation_claim": False,
        "artifact_signing_claim": False,
        "release_approval_claim": False,
        "release_execution_claim": False,
        "public_release_claim": False,
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
        "closed_by": "STUDIO_PUBLICATION_ARTIFACT_DRY_RUN_V1",
        "based_on": "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1",
        "dry_run": "pack/studio_publication_artifact_dry_run_v1/publication_artifact_dry_run.detjson",
        "source_local_release_rehearsal": "pack/studio_local_release_rehearsal_check_v1/local_release_rehearsal_check.detjson",
        "source_release_asset_plan": "pack/studio_public_release_asset_plan_v1/release_assets.detjson",
        "planned_artifact_count": 4,
        "all_planned_artifacts_generated_now": False,
        "primary_coordinate": "타-3",
        "support_coordinate": "마-3",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "post_super_long_closed": 4,
        "post_super_long_total": 8,
        "post_super_long_percent": 50,
        "ma_followup_closed": 4,
        "ma_followup_total": 8,
        "ma_followup_percent": 50,
        "ma3_closed": 4,
        "ma3_total": 4,
        "ma3_percent": 100,
        "ta3_followup_closed": 1,
        "ta3_followup_total": 2,
        "ta3_followup_percent": 50,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
        "browser_runner": "tests/studio_publication_artifact_dry_run_runner.mjs",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    dry_run = load_json(DRY_RUN)
    if dry_run.get("schema") != "ddn.studio.publication_artifact_dry_run.v1":
        fail(f"dry-run schema mismatch: {dry_run.get('schema')!r}")
    if dry_run.get("work_item") != "STUDIO_PUBLICATION_ARTIFACT_DRY_RUN_V1":
        fail(f"dry-run work item mismatch: {dry_run.get('work_item')!r}")
    if dry_run.get("product_code_change") is not True:
        fail(f"product_code_change expected true, got {dry_run.get('product_code_change')!r}")
    if dry_run.get("product_ui_change") is not True:
        fail(f"product_ui_change expected true, got {dry_run.get('product_ui_change')!r}")
    for flag in (
        "runtime_claim",
        "artifact_generation_claim",
        "archive_generation_claim",
        "publication_checksum_generation_claim",
        "artifact_signing_claim",
        "release_approval_claim",
        "release_execution_claim",
        "public_release_claim",
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
        if dry_run.get(flag) is not False:
            fail(f"dry-run {flag} expected false, got {dry_run.get(flag)!r}")
    if dry_run.get("planned_artifacts") != expected_artifacts():
        fail(f"planned artifacts mismatch: {dry_run.get('planned_artifacts')!r}")
    if dry_run.get("all_planned_artifacts_generated_now") is not False:
        fail(f"all planned artifacts generated flag mismatch: {dry_run.get('all_planned_artifacts_generated_now')!r}")
    if dry_run.get("closure_tier") != "닫힘-동작":
        fail(f"closure tier mismatch: {dry_run.get('closure_tier')!r}")
    if dry_run.get("browser_runner") != "tests/studio_publication_artifact_dry_run_runner.mjs":
        fail(f"browser runner mismatch: {dry_run.get('browser_runner')!r}")
    changed_product_files = dry_run.get("changed_product_files")
    if not isinstance(changed_product_files, list) or len(changed_product_files) != 5:
        fail(f"changed product files mismatch: {changed_product_files!r}")
    for rel in changed_product_files:
        require(ROOT / rel)
    if dry_run.get("post_super_long_plan") != {"closed": 4, "total": 8, "percent": 50}:
        fail(f"post-super-long progress mismatch: {dry_run.get('post_super_long_plan')!r}")
    if dry_run.get("roadmap_v2_product_behavior") != {"closed": 90, "total": 90, "percent": 100}:
        fail(f"roadmap progress mismatch: {dry_run.get('roadmap_v2_product_behavior')!r}")
    if dry_run.get("next_item") != NEXT:
        fail(f"next item mismatch: {dry_run.get('next_item')!r}")


def check_source_alignment() -> None:
    rehearsal = load_json(SOURCE_REHEARSAL)
    asset_plan = load_json(SOURCE_ASSET_PLAN)
    if rehearsal.get("next_item") != "STUDIO_PUBLICATION_ARTIFACT_DRY_RUN_V1":
        fail(f"rehearsal source next item mismatch: {rehearsal.get('next_item')!r}")
    if rehearsal.get("dry_run_only") is not True:
        fail(f"rehearsal dry_run_only mismatch: {rehearsal.get('dry_run_only')!r}")
    if rehearsal.get("all_planned_assets_generated_now") is not False:
        fail(f"rehearsal generated flag mismatch: {rehearsal.get('all_planned_assets_generated_now')!r}")
    if rehearsal.get("roadmap_v2_product_behavior", {}).get("closed") != 90:
        fail(f"rehearsal ROADMAP_V2 closed mismatch: {rehearsal.get('roadmap_v2_product_behavior')!r}")
    if rehearsal.get("roadmap_v2_product_behavior", {}).get("percent") != 100:
        fail(f"rehearsal ROADMAP_V2 percent mismatch: {rehearsal.get('roadmap_v2_product_behavior')!r}")
    assets = asset_plan.get("assets")
    if [asset.get("id") for asset in assets] != [artifact["id"] for artifact in expected_artifacts()]:
        fail(f"source asset ids mismatch: {assets!r}")
    if any(asset.get("generated_now") is not False for asset in assets):
        fail(f"source asset generated_now mismatch: {assets!r}")
    checksum_policy = asset_plan.get("checksum_policy")
    expected_checksum = {
        "algorithm": "sha256",
        "manifest_path": "build/studio_release/SHA256SUMS.txt",
        "ordering": "path_lexicographic",
        "scope": "local_build_artifacts_only",
        "signing": "excluded_v1_approval_gated",
    }
    for key, value in expected_checksum.items():
        if checksum_policy.get(key) != value:
            fail(f"checksum policy {key} expected {value!r}, got {checksum_policy.get(key)!r}")
    dry_run = load_json(DRY_RUN)
    checksum = dict(expected_checksum)
    checksum["generated_now"] = False
    if dry_run.get("checksum_policy") != checksum:
        fail(f"dry-run checksum policy mismatch: {dry_run.get('checksum_policy')!r}")
    expected_preflights = [
        "node tests/studio_publication_artifact_dry_run_runner.mjs",
        "python tests/run_studio_local_release_rehearsal_check.py",
        "python tests/run_studio_public_release_asset_plan_check.py",
        "git status --short -- docs/ssot",
    ]
    if dry_run.get("preflight_commands") != expected_preflights:
        fail(f"preflight commands mismatch: {dry_run.get('preflight_commands')!r}")
    required_blocked = {
        "artifact_generation",
        "publication_archive_generation",
        "checksum_manifest_generation_for_publication",
        "artifact_signing",
        "release_approval",
        "release_execution",
        "github_release_create",
        "public_upload",
        "registry_publish",
        "public_link_create",
        "package_install_enable",
        "publication_snapshot_emit",
        "cloud_sync",
        "account_setup",
        "permission_system_change",
    }
    if set(dry_run.get("blocked_in_dry_run", [])) != required_blocked:
        fail(f"blocked actions mismatch: {dry_run.get('blocked_in_dry_run')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_PUBLICATION_ARTIFACT_DRY_RUN_V1",
        "studio publication artifact dry run sealed",
        "publication artifact dry-run schema: ddn.studio.publication_artifact_dry_run.v1",
        "planned artifacts: 4",
        "follow-up plan: 4/8 = 50%",
        "roadmap v2 behavior: 90/90 = 100%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_publication_artifact_dry_run_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_publication_artifact_dry_run_v1"],
        ["node", "tests/studio_publication_artifact_dry_run_runner.mjs"],
        ["python", "tests/run_studio_local_release_rehearsal_check.py"],
        ["python", "tests/run_studio_public_release_asset_plan_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_ui_contract()
    check_contract_and_dry_run()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_publication_artifact_dry_run_check: ok")


if __name__ == "__main__":
    main()
