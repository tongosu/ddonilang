from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1.md"
REPORT = ROOT / "docs" / "studio" / "RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
ROADMAP = ROOT / "docs" / "context" / "queue" / "STUDIO_LONG_HORIZON_ROADMAP_V1.md"
DEV_SUMMARY = ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md"
PROJECT_STATUS = ROOT / "docs" / "status" / "PROJECT_STATUS.md"
CHANGELOG = ROOT / "docs" / "status" / "CHANGELOG.md"
PACK = ROOT / "pack" / "studio_release_approval_gate_status_recheck_v1"
CONTRACT = PACK / "contract.detjson"
GATE = PACK / "gate.detjson"
APPROVAL_PHRASE = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다"


def fail(message: str) -> None:
    print(f"studio_release_approval_gate_status_recheck: FAIL: {message}", file=sys.stderr)
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
        ROADMAP,
        DEV_SUMMARY,
        PROJECT_STATUS,
        CHANGELOG,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        CONTRACT,
        GATE,
        ROOT / "docs" / "context" / "queue" / "STUDIO_RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1.md",
        ROOT / "docs" / "context" / "queue" / "STUDIO_PUBLIC_RELEASE_EXECUTION_GATE_V1.md",
        ROOT / "SEAMGRIM_PRIVATE_PRODUCTIZATION_CONSOLIDATION_AUDIT_V1.md",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        fail(f"missing required paths: {missing}")


def check_docs() -> None:
    common = [
        "STUDIO_RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1",
        APPROVAL_PHRASE,
        "AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "9/18 = 50%",
        "90/90 = 100%",
        "0/90 = 0%",
        "0/1 = 0% approval-gated",
        "docs/ssot/**",
    ]
    require_contains(DOC, common + ["approval gate checks: 5/5 = 100%", "release execution authorized: 0/1 = 0%"])
    require_contains(REPORT, common + ["Status: docs-closed approval gate recheck", "Required phrase present as user approval: no"])
    require_contains(ROADMAP, ["STUDIO_RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1", "AWAIT_EXPLICIT_RELEASE_APPROVAL"])
    require_contains(
        INDEX,
        [
            "STUDIO_RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1",
            "STUDIO_RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1.md",
            "pack/studio_release_approval_gate_status_recheck_v1",
            "tests/run_studio_release_approval_gate_status_recheck.py",
            "docs/studio/RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1.md",
        ],
    )
    require_contains(
        DEV_SUMMARY,
        [
            "STUDIO_RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1",
            "studio_release_approval_gate_status_recheck_v1",
            "release approval gate status recheck 5/5 = 100%",
            "approval gate checks: 5/5 = 100%",
            "release execution authorized: 0/1 = 0%",
            "AWAIT_EXPLICIT_RELEASE_APPROVAL",
            "Studio-local 초장기 계획: 9/18 = 50%",
        ],
    )
    require_contains(PROJECT_STATUS, ["STUDIO_RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1", "gate status recheck `5/5 = 100%`"])
    require_contains(CHANGELOG, ["Studio release approval gate status recheck", "gate status recheck is `5/5 = 100%`"])


def check_contract_and_gate() -> None:
    contract = load_json(CONTRACT)
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_release_approval_gate_status_recheck_v1",
        "kind": "studio_release_approval_gate_status_recheck",
        "closed_by": "STUDIO_RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1",
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
        "parser_frontdoor_change": False,
        "stdlib_change": False,
        "lesson_schema_change": False,
        "active_allowlist_mutation": False,
        "approval_gate_checks_closed": 5,
        "approval_gate_checks_total": 5,
        "release_execution_authorized_closed": 0,
        "release_execution_authorized_total": 1,
        "official_studio_local_closed": 9,
        "official_studio_local_total": 18,
        "official_studio_local_percent": 50,
        "roadmap_v2_behavior_closed": 90,
        "roadmap_v2_total": 90,
        "roadmap_v2_percent": 100,
        "roadmap_v2_docs_closed": 0,
        "public_release_execution_closed": 0,
        "public_release_execution_total": 1,
        "requires_docs_ssot_clean": True,
        "next_state": "AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "next_item": None,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            fail(f"contract {key} expected {value!r}, got {contract.get(key)!r}")

    gate = load_json(GATE)
    if gate.get("schema") != "ddn.studio.release_approval_gate_status_recheck.v1":
        fail(f"gate schema mismatch: {gate.get('schema')!r}")
    if gate.get("required_approval_phrase") != APPROVAL_PHRASE:
        fail("required approval phrase changed")
    for key in [
        "required_phrase_present_as_user_approval",
        "release_execution_authorized",
        "public_release_execution_selected",
        "public_upload_selected",
        "github_release_selected",
        "registry_publish_selected",
    ]:
        if gate.get(key) is not False:
            fail(f"gate {key} expected False, got {gate.get(key)!r}")
    if gate.get("next_state") != "AWAIT_EXPLICIT_RELEASE_APPROVAL":
        fail(f"next state mismatch: {gate.get('next_state')!r}")
    if gate.get("next_safe_action") is not None:
        fail(f"next safe action should be null, got {gate.get('next_safe_action')!r}")
    if len(gate.get("gate_checks", [])) != 5:
        fail(f"gate check count mismatch: {gate.get('gate_checks')!r}")


def check_golden() -> None:
    payload = json.loads(read(PACK / "golden.jsonl").strip())
    expected = [
        "STUDIO_RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1",
        "release approval gate status recheck sealed",
        "approval gate checks: 5/5 = 100%",
        "release execution authorized: 0/1 = 0%",
        "next state: AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "public release execution: approval-gated",
        "studio local progress: 9/18 = 50%",
        "roadmap v2 behavior: 90/90 = 100%",
    ]
    if payload.get("cmd") != ["run", "pack/studio_release_approval_gate_status_recheck_v1/input.ddn"]:
        fail(f"unexpected golden cmd: {payload.get('cmd')!r}")
    if payload.get("exit_code") != 0:
        fail(f"unexpected golden exit_code: {payload.get('exit_code')!r}")
    if payload.get("stdout") != expected:
        fail(f"unexpected golden stdout: {payload.get('stdout')!r}")


def run_required_gates() -> None:
    for cmd in [
        ["python", "tests/run_pack_golden.py", "studio_release_approval_gate_status_recheck_v1"],
        ["python", "tests/run_seamgrim_private_productization_consolidation_audit.py"],
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
    check_contract_and_gate()
    check_golden()
    run_required_gates()
    check_docs_ssot_clean()
    print("studio_release_approval_gate_status_recheck: ok")


if __name__ == "__main__":
    main()
