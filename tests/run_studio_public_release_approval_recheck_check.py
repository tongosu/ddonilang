from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1.md"
ROADMAP = ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
REPORT = ROOT / "docs" / "studio" / "PUBLIC_RELEASE_APPROVAL_RECHECK_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PACK = ROOT / "pack" / "studio_public_release_approval_recheck_v1"
RECHECK = PACK / "public_release_approval_recheck.detjson"
CHECKER = ROOT / "tests" / "run_studio_public_release_approval_recheck_check.py"
SOURCE_REBASE = ROOT / "pack" / "studio_post_super_long_rebase_v1" / "post_super_long_rebase.detjson"
SOURCE_CONTINUITY = ROOT / "pack" / "studio_release_approval_packet_continuity_v1" / "continuity.detjson"
SOURCE_CLOSURE = ROOT / "pack" / "studio_release_approval_chain_closure_v1" / "closure.detjson"
UI_MODULE = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_public_release_approval_recheck.js"
APP_JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js"
INDEX_HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html"
STYLES_CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css"
RUNNER = ROOT / "tests" / "studio_public_release_approval_recheck_runner.mjs"
REQUIRED_APPROVAL = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다"
NEXT = "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1"


def fail(message: str) -> None:
    print(f"studio_public_release_approval_recheck_check: FAIL: {message}", file=sys.stderr)
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
        "approval_surface": "local_studio_public_release_approval_recheck",
        "approval_recheck_only": True,
        "generated_now": False,
        "release_approval_claim": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "public_upload_claim": False,
        "registry_publish_claim": False,
        "github_release_claim": False,
        "runtime_claim": False,
        "replay_claim": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "solver_implementation_change": False,
        "benchmark_execution_claim": False,
        "lts_certification_claim": False,
        "product_ui_change": True,
    }
    rows: list[tuple[str, str, str]] = [
        ("required_phrase_lock", "pack/studio_release_approval_packet_continuity_v1/continuity.detjson", "phrase_lock"),
        ("generic_request_rejected", "STUDIO_POST_SUPER_LONG_REBASE_V1", "generic_request_boundary"),
        ("current_request_rejected", "current_user_request", "current_request_boundary"),
        ("await_state_guard", "pack/studio_release_approval_chain_closure_v1/closure.detjson", "await_state"),
        ("local_rehearsal_handoff", NEXT, "next_handoff"),
    ]
    return [
        {
            "id": row_id,
            "source_anchor": source_anchor,
            "approval_lane": lane,
            **common,
        }
        for row_id, source_anchor, lane in rows
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
        RECHECK,
        CHECKER,
        SOURCE_REBASE,
        SOURCE_CONTINUITY,
        SOURCE_CLOSURE,
        UI_MODULE,
        APP_JS,
        INDEX_HTML,
        STYLES_CSS,
        RUNNER,
        ROOT / "tests" / "run_studio_post_super_long_rebase_check.py",
        ROOT / "tests" / "run_studio_release_approval_packet_continuity_check.py",
    ]:
        require(path)


def check_docs() -> None:
    doc_tokens = [
        "STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1",
        "ddn.studio.public_release_approval_recheck.v1",
        "Primary coordinate: `마-3`",
        "Support coordinate: `타-3`",
        REQUIRED_APPROVAL,
        "generic next-development requests are not approval",
        "current development request is not release approval",
        "AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "작업 단위: 6/6 = 100% (`닫힘-동작`)",
        "approval rows: 5/5 = 100%",
        "approval recheck stages: 6/6 = 100%",
        "전체 초장기 계획: 18/18 = 100%",
        "현재 스테이지: post-super-long follow-up 2/8 = 25%",
        "ROADMAP_V2 product behavior baseline: 90/90 = 100%",
        "No release approval",
        "No release execution",
        NEXT,
        "docs/ssot/**",
    ]
    require_contains(DOC, doc_tokens)
    require_contains(REPORT, ["ddn.studio.public_release_approval_recheck.v1", "6/6 = 100%", "18/18 = 100%", "2/8 = 25%", "90/90 = 100%"])
    require_contains(
        INDEX,
        [
            "STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1",
            "docs/studio/PUBLIC_RELEASE_APPROVAL_RECHECK_V1.md",
            "pack/studio_public_release_approval_recheck_v1",
            "tests/run_studio_public_release_approval_recheck_check.py",
        ],
    )
    require_contains(
        ROADMAP,
        [
            "STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1",
            "ddn.studio.public_release_approval_recheck.v1",
            "local Studio approval recheck panel",
            "전체 초장기 계획 18/18 = 100%",
            "post-super-long follow-up 2/8 = 25%",
            "ROADMAP_V2 product behavior baseline 90/90 = 100%",
            NEXT,
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1",
            "studio_public_release_approval_recheck_runner.mjs",
            "approval rows: 5/5 = 100%",
            "전체 초장기 계획: 18/18 = 100%",
            "현재 스테이지: post-super-long follow-up 2/8 = 25%",
            "ROADMAP_V2 product behavior baseline: 90/90 = 100%",
            "docs/ssot/** 변경 없음",
        ],
    )


def check_product_tokens() -> None:
    require_contains(
        UI_MODULE,
        [
            "ddn.studio.public_release_approval_recheck.v1",
            "REQUIRED_PUBLIC_RELEASE_APPROVAL_PHRASE",
            "buildPublicReleaseApprovalRecheck",
            "formatPublicReleaseApprovalRecheckText",
            "renderPublicReleaseApprovalRecheck",
            "AWAIT_EXPLICIT_RELEASE_APPROVAL",
            "current_request_is_release_approval: false",
            "release_approval_claim: false",
            "release_execution_claim: false",
            "current_stage_percent: 25",
            "roadmap_v2_percent: 100",
        ],
    )
    require_contains(
        APP_JS,
        [
            "studio_public_release_approval_recheck.js",
            "publishPublicReleaseApprovalRecheck",
            "__SEAMGRIM_PUBLIC_RELEASE_APPROVAL_RECHECK__",
            "buildPublicReleaseApprovalRecheck",
        ],
    )
    require_contains(INDEX_HTML, ["public-release-approval-recheck", "data-public-release-approval-recheck"])
    require_contains(STYLES_CSS, [".public-release-approval-recheck", ".approval-recheck-btn.active"])
    require_contains(RUNNER, ["studio_public_release_approval_recheck: ok", "approval_recheck_waiting", "required_approval_phrase"])


def check_contract_and_recheck() -> None:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_public_release_approval_recheck_v1",
        "kind": "studio_public_release_approval_recheck",
        "runtime_claim": False,
        "product_code_change": True,
        "product_ui_change": True,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "parser_frontdoor_change": False,
        "solver_implementation_change": False,
        "approval_recheck_claim": True,
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
        "closed_by": "STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1",
        "based_on": "STUDIO_POST_SUPER_LONG_REBASE_V1",
        "recheck": "pack/studio_public_release_approval_recheck_v1/public_release_approval_recheck.detjson",
        "source_post_super_long_rebase": "pack/studio_post_super_long_rebase_v1/post_super_long_rebase.detjson",
        "source_approval_continuity": "pack/studio_release_approval_packet_continuity_v1/continuity.detjson",
        "source_approval_chain_closure": "pack/studio_release_approval_chain_closure_v1/closure.detjson",
        "browser_runner": "tests/studio_public_release_approval_recheck_runner.mjs",
        "required_approval_phrase": REQUIRED_APPROVAL,
        "generic_next_dev_request_is_approval": False,
        "current_request_is_release_approval": False,
        "next_state": "AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "primary_coordinate": "마-3",
        "support_coordinate": "타-3",
        "super_long_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "work_unit_closed": 6,
        "work_unit_total": 6,
        "post_super_long_closed": 2,
        "post_super_long_total": 8,
        "post_super_long_percent": 25,
        "ma_followup_closed": 2,
        "ma_followup_total": 8,
        "ma_followup_percent": 25,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    recheck = load_json(RECHECK)
    if recheck.get("schema") != "ddn.studio.public_release_approval_recheck.v1":
        fail(f"recheck schema mismatch: {recheck.get('schema')!r}")
    if recheck.get("approval_rows") != expected_rows():
        fail(f"approval rows mismatch: {recheck.get('approval_rows')!r}")
    if recheck.get("progress") != {
        "super_long_behavior_closed": 18,
        "super_long_total": 18,
        "super_long_percent": 100,
        "current_stage_closed": 2,
        "current_stage_total": 8,
        "current_stage_percent": 25,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
    }:
        fail(f"recheck progress mismatch: {recheck.get('progress')!r}")
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
        if recheck.get(flag) is not expected_value:
            fail(f"recheck {flag} expected {expected_value!r}, got {recheck.get(flag)!r}")
    if recheck.get("required_approval_phrase") != REQUIRED_APPROVAL:
        fail(f"approval phrase mismatch: {recheck.get('required_approval_phrase')!r}")
    if recheck.get("generic_next_dev_request_is_approval") is not False:
        fail(f"generic approval flag mismatch: {recheck.get('generic_next_dev_request_is_approval')!r}")
    if recheck.get("current_request_is_release_approval") is not False:
        fail(f"current request approval flag mismatch: {recheck.get('current_request_is_release_approval')!r}")
    if recheck.get("next_state") != "AWAIT_EXPLICIT_RELEASE_APPROVAL":
        fail(f"next state mismatch: {recheck.get('next_state')!r}")
    if recheck.get("post_super_long_plan") != {"closed": 2, "total": 8, "percent": 25}:
        fail(f"post-super-long progress mismatch: {recheck.get('post_super_long_plan')!r}")
    if recheck.get("next_item") != NEXT:
        fail(f"next item mismatch: {recheck.get('next_item')!r}")


def check_source_alignment() -> None:
    rebase = load_json(SOURCE_REBASE)
    continuity = load_json(SOURCE_CONTINUITY)
    closure = load_json(SOURCE_CLOSURE)
    if rebase.get("schema") != "ddn.studio.post_super_long_rebase.v1":
        fail(f"source rebase schema mismatch: {rebase.get('schema')!r}")
    if rebase.get("next_item") != "STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1":
        fail(f"source rebase next item mismatch: {rebase.get('next_item')!r}")
    if rebase.get("progress", {}).get("current_stage_percent") != 13:
        fail(f"source rebase progress mismatch: {rebase.get('progress')!r}")
    if rebase.get("progress", {}).get("roadmap_v2_behavior_closed") != 90:
        fail(f"source rebase ROADMAP_V2 closed mismatch: {rebase.get('progress')!r}")
    if rebase.get("progress", {}).get("roadmap_v2_percent") != 100:
        fail(f"source rebase ROADMAP_V2 percent mismatch: {rebase.get('progress')!r}")
    if continuity.get("required_approval_phrase") != REQUIRED_APPROVAL:
        fail(f"continuity approval phrase mismatch: {continuity.get('required_approval_phrase')!r}")
    if closure.get("required_approval_phrase") != REQUIRED_APPROVAL:
        fail(f"closure approval phrase mismatch: {closure.get('required_approval_phrase')!r}")
    for source_name, source in (("continuity", continuity), ("closure", closure)):
        if source.get("generic_next_dev_request_is_approval") is not False:
            fail(f"{source_name} generic approval flag mismatch: {source.get('generic_next_dev_request_is_approval')!r}")
        if source.get("next_state") != "AWAIT_EXPLICIT_RELEASE_APPROVAL":
            fail(f"{source_name} state mismatch: {source.get('next_state')!r}")
        for flag in ("release_execution_claim", "public_release_claim", "github_release_claim", "public_upload_claim"):
            if source.get(flag) is not False:
                fail(f"{source_name} {flag} expected false, got {source.get(flag)!r}")
    expected_preflights = [
        "node tests/studio_public_release_approval_recheck_runner.mjs",
        "python tests/run_studio_post_super_long_rebase_check.py",
        "python tests/run_studio_release_approval_packet_continuity_check.py",
        "git status --short -- docs/ssot",
    ]
    recheck = load_json(RECHECK)
    if recheck.get("preflight_commands") != expected_preflights:
        fail(f"preflight commands mismatch: {recheck.get('preflight_commands')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1",
        "studio public release approval recheck sealed",
        "approval recheck schema: ddn.studio.public_release_approval_recheck.v1",
        "state: AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "follow-up plan: 2/8 = 25%",
        "roadmap v2 behavior: 90/90 = 100%",
        f"next: {NEXT}",
    ]
    if payload.get("cmd") != ["run", "pack/studio_public_release_approval_recheck_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["node", "tests/studio_public_release_approval_recheck_runner.mjs"],
        ["python", "tests/run_pack_golden.py", "studio_public_release_approval_recheck_v1"],
        ["python", "tests/run_studio_post_super_long_rebase_check.py"],
        ["python", "tests/run_studio_release_approval_packet_continuity_check.py"],
    ]:
        proc = run(cmd, timeout=1800)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def main() -> None:
    check_required_files()
    check_docs()
    check_product_tokens()
    check_contract_and_recheck()
    check_source_alignment()
    check_golden()
    run_required_gates()
    require_docs_ssot_clean()
    print("studio_public_release_approval_recheck_check: ok")


if __name__ == "__main__":
    main()
