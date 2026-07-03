from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "queue" / "STUDIO_POST_SUPER_LONG_REBASE_V1.md"
ROADMAP = ROOT / "docs" / "context" / "queue" / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
REPORT = ROOT / "docs" / "studio" / "POST_SUPER_LONG_REBASE_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "studio_post_super_long_rebase_v1"
REBASE = PACK / "post_super_long_rebase.detjson"
CHECKER = ROOT / "tests" / "run_studio_post_super_long_rebase_check.py"
SOURCE = ROOT / "pack" / "studio_productization_stage_closure_v1" / "productization_stage_closure.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_post_super_long_rebase.js"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
DEV_SURFACES_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
DEV_SURFACES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "dev_surfaces.css"
RUNNER = ROOT / "tests" / "studio_post_super_long_rebase_runner.mjs"
SOURCE_CHECK = ROOT / "tests" / "run_studio_productization_stage_closure_check.py"
NEXT = "STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1"


def fail(message: str) -> None:
    print(f"studio_post_super_long_rebase_check: FAIL: {message}", file=sys.stderr)
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
            "id": "STUDIO_POST_SUPER_LONG_REBASE_V1",
            "coordinate": "마-3",
            "status": "closed",
            "claim": "post-super-long denominator lock",
        },
        {
            "id": "STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1",
            "coordinate": "마-3",
            "status": "next",
            "claim": "approval readiness recheck only",
        },
        {
            "id": "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1",
            "coordinate": "마-3",
            "status": "planned",
            "claim": "local rehearsal check only",
        },
        {
            "id": "STUDIO_PUBLICATION_ARTIFACT_DRY_RUN_V1",
            "coordinate": "타-3",
            "status": "planned",
            "claim": "dry-run artifact manifest only",
        },
        {
            "id": "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1",
            "coordinate": "하-3",
            "status": "planned",
            "claim": "local feedback seed only",
        },
        {
            "id": "STUDIO_CLASSROOM_OPERATIONS_TRIAGE_V1",
            "coordinate": "하-3",
            "status": "planned",
            "claim": "local triage evidence only",
        },
        {
            "id": "STUDIO_BENCHMARK_BASELINE_PREP_DRY_RUN_V1",
            "coordinate": "타-3",
            "status": "planned",
            "claim": "baseline prep dry-run only",
        },
        {
            "id": "STUDIO_NEXT_ROADMAP_V2_COORDINATE_LOCK_V1",
            "coordinate": "마-3",
            "status": "planned",
            "claim": "next ROADMAP_V2 coordinate lock",
        },
    ]


def check_required_files() -> None:
    for path in [
        DOC,
        ROADMAP,
        REPORT,
        INDEX,
        DEV_SUMMARY,
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        REBASE,
        CHECKER,
        SOURCE,
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
    doc_tokens = [
        "STUDIO_POST_SUPER_LONG_REBASE_V1",
        "ddn.studio.post_super_long_rebase.v1",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        "V6.1 baseline of 9/18",
        "작업 단위: 6/6 = 100% (`닫힘-동작`)",
        "follow-up rows: 8/8 = 100%",
        "rebase stages: 5/5 = 100%",
        "전체 초장기 계획: 9/18 = 50%",
        "현재 스테이지: post-super-long follow-up 1/8 = 13%",
        "ROADMAP_V2 matrix behavior baseline: 51/90 = 57%",
        "No release approval",
        "No release execution",
        "No LTS certification",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, doc_tokens)
    require_contains(REPORT, ["ddn.studio.post_super_long_rebase.v1", "6/6 = 100%", "8/8 = 100%", "9/18 = 50%", "1/8 = 13%", "51/90 = 57%"])
    require_contains(
        INDEX,
        [
            "STUDIO_POST_SUPER_LONG_REBASE_V1",
            "docs/studio/POST_SUPER_LONG_REBASE_V1.md",
            "pack/studio_post_super_long_rebase_v1",
            "tests/run_studio_post_super_long_rebase_check.py",
        ],
    )
    require_contains(
        ROADMAP,
        [
            "STUDIO_POST_SUPER_LONG_REBASE_V1",
            "ddn.studio.post_super_long_rebase.v1",
            NEXT,
            "전체 초장기 계획 9/18 = 50%",
            "post-super-long follow-up 1/8 = 13%",
            "ROADMAP_V2 matrix behavior baseline 51/90 = 57%",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_POST_SUPER_LONG_REBASE_V1",
            "studio_post_super_long_rebase_runner.mjs",
            "follow-up rows: 8/8 = 100%",
            "전체 초장기 계획: 9/18 = 50%",
            "현재 스테이지: post-super-long follow-up 1/8 = 13%",
            "ROADMAP_V2 matrix behavior baseline: 51/90 = 57%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_product_tokens() -> None:
    require_contains(
        UI_MODULE,
        [
            "ddn.studio.post_super_long_rebase.v1",
            "buildPostSuperLongRebase",
            "formatPostSuperLongRebaseText",
            "renderPostSuperLongRebase",
            "super_long_behavior_closed: 9",
            "current_stage_percent: 13",
            "roadmap_v2_behavior_closed: 51",
            "roadmap_v2_percent: 57",
            "release_execution_claim: false",
            "runtime_claim: false",
        ],
    )
    require_contains(
        DEV_SURFACES_JS,
        [
            "studio_post_super_long_rebase.js",
            "__SEAMGRIM_POST_SUPER_LONG_REBASE__",
            "buildPostSuperLongRebase",
        ],
    )
    require_contains(DEV_SURFACES_JS, ["post-super-long-rebase", "elementId: \"post-super-long-rebase\""])
    require_contains(APP_JS, ["shouldEnableDevSurfaces", "./dev_surfaces.js"])
    require_contains(DEV_SURFACES_CSS, [".post-super-long-rebase", ".post-super-rebase-btn.active"])
    require_contains(RUNNER, ["studio_post_super_long_rebase: ok", "post_super_long_rebased", "followup\\t1/8"])


def check_contract_and_rebase() -> None:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_post_super_long_rebase_v1",
        "kind": "studio_post_super_long_rebase",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "solver_implementation_change": False,
        "post_super_long_rebase_claim": True,
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
        "archive_generation_claim": False,
        "publication_checksum_generation_claim": False,
        "cloud_sync_claim": False,
        "account_setup_claim": False,
        "permission_system_claim": False,
        "replay_claim": False,
        "closed_by": "STUDIO_POST_SUPER_LONG_REBASE_V1",
        "based_on": "STUDIO_PRODUCTIZATION_STAGE_CLOSURE_V1",
        "rebase": "pack/studio_post_super_long_rebase_v1/post_super_long_rebase.detjson",
        "source_productization_stage_closure": "pack/studio_productization_stage_closure_v1/productization_stage_closure.detjson",
        "browser_runner": "tests/studio_post_super_long_rebase_runner.mjs",
        "primary_coordinate": "마-3",
        "support_coordinate": "타-3",
        "super_long_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "post_super_long_closed": 1,
        "post_super_long_total": 8,
        "post_super_long_percent": 13,
        "ma_followup_closed": 1,
        "ma_followup_total": 8,
        "ma_followup_percent": 13,
        "roadmap_v2_behavior_closed": 51,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 57,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    rebase = load_json(REBASE)
    if rebase.get("schema") != "ddn.studio.post_super_long_rebase.v1":
        fail(f"rebase schema mismatch: {rebase.get('schema')!r}")
    if rebase.get("work_item") != "STUDIO_POST_SUPER_LONG_REBASE_V1":
        fail(f"rebase work item mismatch: {rebase.get('work_item')!r}")
    if rebase.get("post_super_long_plan", {}).get("items") != expected_items():
        fail(f"post-super-long items mismatch: {rebase.get('post_super_long_plan', {}).get('items')!r}")
    if rebase.get("progress") != {
        "super_long_behavior_closed": 9,
        "super_long_total": 18,
        "super_long_percent": 50,
        "current_stage_closed": 1,
        "current_stage_total": 8,
        "current_stage_percent": 13,
        "roadmap_v2_behavior_closed": 51,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 57,
    }:
        fail(f"rebase progress mismatch: {rebase.get('progress')!r}")
    for flag, expected_value in (
        ("product_code_change", True),
        ("product_ui_change", True),
        ("runtime_claim", False),
        ("release_approval_claim", False),
        ("release_execution_claim", False),
        ("public_release_claim", False),
        ("lts_certification_claim", False),
        ("benchmark_execution_claim", False),
        ("public_upload_claim", False),
        ("registry_publish_claim", False),
        ("active_allowlist_mutation", False),
        ("lesson_schema_change", False),
        ("parser_frontdoor_change", False),
        ("solver_implementation_change", False),
    ):
        if rebase.get(flag) is not expected_value:
            fail(f"rebase {flag} expected {expected_value!r}, got {rebase.get(flag)!r}")
    if rebase.get("next_item") != NEXT:
        fail(f"next item mismatch: {rebase.get('next_item')!r}")


def check_source_alignment() -> None:
    source = load_json(SOURCE)
    if source.get("schema") != "ddn.studio.productization_stage_closure.v1":
        fail(f"source schema mismatch: {source.get('schema')!r}")
    if source.get("next_item") != "STUDIO_POST_SUPER_LONG_REBASE_V1":
        fail(f"source next item mismatch: {source.get('next_item')!r}")
    if source.get("progress", {}).get("super_long_percent") != 50:
        fail(f"source progress mismatch: {source.get('progress')!r}")
    if source.get("progress", {}).get("roadmap_v2_behavior_closed") != 51:
        fail(f"source roadmap closed mismatch: {source.get('progress')!r}")
    if source.get("progress", {}).get("roadmap_v2_percent") != 57:
        fail(f"source roadmap progress mismatch: {source.get('progress')!r}")
    rebase = load_json(REBASE)
    if rebase.get("super_long_plan") != {
        "closed": 9,
        "total": 18,
        "percent": 50,
        "status": "v6_1_frozen",
    }:
        fail(f"super-long seal mismatch: {rebase.get('super_long_plan')!r}")
    blocked = set(rebase.get("blocked_actions", []))
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
        "lts_certification_publish",
        "benchmark_baseline_publish",
        "cloud_sync",
        "account_setup",
        "permission_system_change",
        "artifact_signing",
    }
    if blocked != required_blocked:
        fail(f"blocked actions mismatch: {rebase.get('blocked_actions')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_POST_SUPER_LONG_REBASE_V1",
        "studio post-super-long rebase sealed",
        "post-super-long schema: ddn.studio.post_super_long_rebase.v1",
        "follow-up plan: 1/8 = 13%",
        "super-long plan remains: 9/18 = 50%",
        "roadmap v2 behavior: 51/90 = 57%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_post_super_long_rebase_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["node", "tests/studio_post_super_long_rebase_runner.mjs"],
        ["python", "tests/run_pack_golden.py", "studio_post_super_long_rebase_v1"],
        ["python", "tests/run_studio_productization_stage_closure_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_product_tokens()
    check_contract_and_rebase()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_post_super_long_rebase_check: ok")


if __name__ == "__main__":
    main()
