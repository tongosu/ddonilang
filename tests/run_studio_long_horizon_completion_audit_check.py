from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_LONG_HORIZON_COMPLETION_AUDIT_V1.md"
REPORT = ROOT / "docs" / "studio" / "LONG_HORIZON_COMPLETION_AUDIT_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
ROADMAP = ROOT / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
PACK = ROOT / "pack" / "studio_long_horizon_completion_audit_v1"
CONTRACT = PACK / "contract.detjson"
AUDIT = PACK / "audit.detjson"


def fail(message: str) -> None:
    print(f"studio_long_horizon_completion_audit_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")


def load_json(path: Path) -> dict:
    return json.loads(read(path))


def run(cmd: list[str], *, timeout: int = 120) -> subprocess.CompletedProcess[str]:
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


def require_contains(path: Path, tokens: list[str]) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing tokens: {missing}")


def require_files() -> None:
    required = [
        DOC,
        REPORT,
        INDEX,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        ROADMAP,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        AUDIT,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        fail(f"missing required paths: {missing}")


def check_docs() -> None:
    common = [
        "STUDIO_LONG_HORIZON_COMPLETION_AUDIT_V1",
        "9/18 = 50%",
        "90/90 = 100%",
        "0/90 = 0%",
        "12/12 = 100%",
        "approval-gated",
        "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다",
        "STUDIO_LONG_HORIZON_NEXT_JIT_RECONCILIATION_V1",
        "docs/ssot/**",
    ]
    require_contains(DOC, common + ["overall goal completion claim: false", "public release execution: 0/1 = 0%"])
    require_contains(REPORT, common + ["Status: docs-closed audit", "current request does not contain that exact phrase"])
    require_contains(
        INDEX,
        [
            "STUDIO_LONG_HORIZON_COMPLETION_AUDIT_V1",
            "STUDIO_LONG_HORIZON_COMPLETION_AUDIT_V1.md",
            "pack/studio_long_horizon_completion_audit_v1",
            "tests/run_studio_long_horizon_completion_audit_check.py",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_LONG_HORIZON_COMPLETION_AUDIT_V1",
            "studio_long_horizon_completion_audit_v1",
            "completion audit 5/5 = 100%",
            "overall goal completion claim: false",
            "public release execution: 0/1 = 0%",
            "Studio-local 초장기 계획: 9/18 = 50%",
        ],
    )
    require_contains(PROJECT_STATUS, ["STUDIO_LONG_HORIZON_COMPLETION_AUDIT_V1", "completion audit `5/5 = 100%`"])
    require_contains(CHANGELOG, ["Studio long-horizon completion audit", "completion audit is `5/5 = 100%`"])


def check_canonical_ledger() -> None:
    require_contains(
        ROADMAP,
        [
            "## Canonical Studio-local super-long closure ledger",
            "authoritative progress counter",
            "browser-verifiable local export/action behavior closure",
            "`닫힘-동작: 18/18 = 100%`",
            "`ROADMAP_V2: 90/90 = 100%`",
            "`external publish readiness: 0/4 = 0%`",
            "`docs/ssot/** unchanged`",
            "`scope_status: conditional substitute`",
            "`scope_status: direct behavior closure`",
            "original long-horizon item",
            "accepted export-action closure",
            "`STUDIO_DIAGNOSTIC_FIXIT_INTEGRATION_V1`",
            "`STUDIO_DIAGNOSTIC_FIXIT_EDITOR_APPLY_V1`",
            "`STUDIO_CLASSROOM_MODE_V1`",
            "`STUDIO_CLASSROOM_MODE_SWITCH_V1`",
            "`STUDIO_BROWSER_SMOKE_MATRIX_HARDENING_V1`",
            "`STUDIO_CLASSROOM_REPORT_EXPORT_ACTION_V1`",
            "`STUDIO_LOCAL_PACKAGING_CONSOLIDATION_V1`",
            "`STUDIO_LOCAL_PACKAGE_EXPORT_ACTION_V1`",
            "`STUDIO_PUBLIC_LESSON_PUBLICATION_PREP_V1`",
            "`STUDIO_PUBLICATION_PREP_EXPORT_ACTION_V1`",
            "`STUDIO_REGISTRY_SHARE_SEED_V1`",
            "`STUDIO_REGISTRY_SHARE_SEED_EXPORT_ACTION_V1`",
            "`STUDIO_RELEASE_APPROVAL_PACKET_CONTINUITY_V1`",
            "`STUDIO_RELEASE_APPROVAL_CONTINUITY_EXPORT_ACTION_V1`",
            "`STUDIO_BENCHMARK_LTS_MATRIX_V1`",
            "`STUDIO_BENCHMARK_LTS_MATRIX_EXPORT_ACTION_V1`",
            "`STUDIO_EDUCATION_OPERATIONS_LTS_V1`",
            "`STUDIO_EDUCATION_OPERATIONS_LTS_EXPORT_ACTION_V1`",
            "## Historical closure log",
            "not the latest authoritative progress counter",
            "Older `9/18 = 50%`, `8/18 = 44%`, and docs-only closure wording",
            "The latest Studio-local progress basis is the canonical ledger above.",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "accepted export-action behavior closure",
            "조건부 인정",
            "closure mapping policy: `scope_status: conditional substitute`",
            "Studio-local 초장기 계획: 18/18 = 100%",
        ],
    )
    require_contains(
        PROJECT_STATUS,
        [
            "accepted export-action behavior closure",
            "조건부 인정",
            "closure mapping policy: `scope_status: conditional substitute`",
            "Studio-local 초장기 계획: 18/18 = 100%",
        ],
    )


def check_contract_and_audit() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_long_horizon_completion_audit_v1",
        "kind": "studio_long_horizon_completion_audit",
        "closed_by": "STUDIO_LONG_HORIZON_COMPLETION_AUDIT_V1",
        "closure_tier": "닫힘-문서",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "behavior_closed_claim": False,
        "goal_completion_claim": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "github_release_claim": False,
        "public_upload_claim": False,
        "registry_publish_claim": False,
        "benchmark_execution_claim": False,
        "lts_certification_claim": False,
        "parser_frontdoor_change": False,
        "stdlib_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "official_studio_local_closed": 9,
        "official_studio_local_total": 18,
        "official_studio_local_percent": 50,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
        "roadmap_v2_docs_closed": 0,
        "stale_progress_repair_closed": 12,
        "stale_progress_repair_total": 12,
        "public_release_execution_closed": 0,
        "public_release_execution_total": 1,
        "requires_release_approval_phrase": True,
        "requires_docs_ssot_clean": True,
        "next_item": "STUDIO_LONG_HORIZON_NEXT_JIT_RECONCILIATION_V1",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    audit = load_json(AUDIT)
    expected_audit = {
        "schema": "ddn.studio.long_horizon_completion_audit.v1",
        "work_item": "STUDIO_LONG_HORIZON_COMPLETION_AUDIT_V1",
        "closure_tier": "닫힘-문서",
        "runtime_claim": False,
        "product_code_change": False,
        "product_ui_change": False,
        "goal_completion_claim": False,
        "public_release_execution_claim": False,
        "release_execution_state": "AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "required_release_approval_phrase": "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다",
        "approval_phrase_present_in_current_request": False,
        "next_safe_action": "STUDIO_LONG_HORIZON_NEXT_JIT_RECONCILIATION_V1",
    }
    for key, value in expected_audit.items():
        if audit.get(key) != value:
            fail(f"audit {key} expected {value!r}, got {audit.get(key)!r}")
    progress = audit.get("official_progress", {})
    for key, value in {
        "studio_local_super_long": "9/18 = 50%",
        "roadmap_v2_behavior_closed": "90/90 = 100%",
        "roadmap_v2_docs_closed": "0/90 = 0%",
        "roadmap_v2_pack_evidence": "90/90 = 100%",
        "stale_progress_repair": "12/12 = 100%",
        "public_release_execution": "0/1 = 0% approval-gated",
    }.items():
        if progress.get(key) != value:
            fail(f"audit progress {key} expected {value!r}, got {progress.get(key)!r}")
    statuses = {item["requirement"]: item["status"] for item in audit.get("completion_findings", [])}
    expected_statuses = {
        "progress_visibility": "satisfied",
        "stale_progress_boundary": "satisfied",
        "roadmap_v2_behavior": "satisfied",
        "studio_local_super_long": "in_progress",
        "public_release_execution": "approval_gated",
    }
    if statuses != expected_statuses:
        fail(f"completion findings mismatch: {statuses!r}")
    if audit.get("blocked_actions_without_approval") != [
        "public release execution",
        "GitHub Release",
        "public upload",
        "registry publish",
    ]:
        fail(f"blocked actions mismatch: {audit.get('blocked_actions_without_approval')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected_stdout = [
        "STUDIO_LONG_HORIZON_COMPLETION_AUDIT_V1",
        "studio long horizon completion audit sealed",
        "overall goal completion claim: false",
        "studio local progress: 9/18 = 50%",
        "roadmap v2 behavior: 90/90 = 100%",
        "stale progress repair: 12/12 = 100%",
        "public release execution: approval-gated",
        "next: STUDIO_LONG_HORIZON_NEXT_JIT_RECONCILIATION_V1",
    ]
    if payload.get("cmd") != ["run", "pack/studio_long_horizon_completion_audit_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected_stdout:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_long_horizon_completion_audit_v1"],
    ]:
        proc = run(cmd, timeout=180)
        if proc.returncode != 0:
            fail(f"{' '.join(cmd)} failed:\n{proc.stdout}")


def check_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed: {proc.stdout.strip()}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    require_files()
    check_docs()
    check_canonical_ledger()
    check_contract_and_audit()
    check_golden()
    run_required_gates()
    check_docs_ssot_clean()
    print("studio_long_horizon_completion_audit_check: ok")


if __name__ == "__main__":
    main()
