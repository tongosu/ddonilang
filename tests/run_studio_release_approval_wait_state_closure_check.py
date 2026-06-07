#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1.md"
PREV = ROOT / "STUDIO_STALE_RELEASE_DOC_AUDIT_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
REPORT = ROOT / "docs" / "studio" / "RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1.md"
PACK = ROOT / "pack" / "studio_release_approval_wait_state_closure_v1"
WAIT_STATE = PACK / "wait_state.detjson"
STALE_AUDIT = ROOT / "pack" / "studio_stale_release_doc_audit_v1" / "audit.detjson"
REQUIRED_APPROVAL = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


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
    )


def require_files() -> int:
    required = [
        DOC,
        PREV,
        INDEX,
        REPORT,
        PACK / "README.md",
        PACK / "contract.detjson",
        WAIT_STATE,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        STALE_AUDIT,
        ROOT / "tests" / "run_studio_stale_release_doc_audit_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_RELEASE_WAIT_STATE_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_docs() -> int:
    checks = [
        (
            DOC,
            [
                "STUDIO_RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1",
                "STUDIO_STALE_RELEASE_DOC_AUDIT_V1",
                "AWAIT_EXPLICIT_RELEASE_APPROVAL",
                REQUIRED_APPROVAL,
                "Generic next-development requests are not approval",
                "no release archives",
                "docs/ssot/**",
            ],
            "E_STUDIO_RELEASE_WAIT_STATE_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_STALE_RELEASE_DOC_AUDIT_V1",
                "STUDIO_RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1",
                "AWAIT_EXPLICIT_RELEASE_APPROVAL",
            ],
            "E_STUDIO_RELEASE_WAIT_STATE_PREV",
        ),
        (
            INDEX,
            [
                "STUDIO_RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1",
                "docs/studio/RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1.md",
                "pack/studio_release_approval_wait_state_closure_v1",
                "tests/run_studio_release_approval_wait_state_closure_check.py",
            ],
            "E_STUDIO_RELEASE_WAIT_STATE_INDEX",
        ),
        (
            REPORT,
            [
                "Studio Release Approval Wait State Closure V1",
                "approval wait state sealed",
                REQUIRED_APPROVAL,
                "AWAIT_EXPLICIT_RELEASE_APPROVAL",
                "release execution selected: no",
                "opens no release execution item",
            ],
            "E_STUDIO_RELEASE_WAIT_STATE_REPORT",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_wait_state() -> int:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_release_approval_wait_state_closure_v1",
        "kind": "studio_release_approval_wait_state_closure",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1",
        "wait_state": "pack/studio_release_approval_wait_state_closure_v1/wait_state.detjson",
        "report": "docs/studio/RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1.md",
        "based_on": "STUDIO_STALE_RELEASE_DOC_AUDIT_V1",
        "current_state": "AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "required_approval_phrase": REQUIRED_APPROVAL,
        "release_execution_selected": False,
        "generic_next_dev_request_is_approval": False,
        "automatic_next_release_item": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "github_release_claim": False,
        "public_upload_claim": False,
        "asset_generation_claim": False,
        "execution_approval_claim": False,
        "next_state": "AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_RELEASE_WAIT_STATE_CONTRACT", f"{key}={contract.get(key)!r}")

    wait_state = load_json(WAIT_STATE)
    if wait_state.get("schema") != "ddn.studio.release_approval_wait_state_closure.v1":
        return fail("E_STUDIO_RELEASE_WAIT_STATE_SCHEMA", repr(wait_state.get("schema")))
    for key in (
        "current_state",
        "next_state",
    ):
        if wait_state.get(key) != "AWAIT_EXPLICIT_RELEASE_APPROVAL":
            return fail("E_STUDIO_RELEASE_WAIT_STATE_STATE", f"{key}={wait_state.get(key)!r}")
    if wait_state.get("required_approval_phrase") != REQUIRED_APPROVAL:
        return fail("E_STUDIO_RELEASE_WAIT_STATE_APPROVAL", repr(wait_state.get("required_approval_phrase")))
    for key in (
        "generic_next_dev_request_is_approval",
        "release_execution_selected",
        "automatic_next_release_item",
        "release_execution_claim",
        "public_release_claim",
        "github_release_claim",
        "public_upload_claim",
        "asset_generation_claim",
        "execution_approval_claim",
    ):
        if wait_state.get(key) is not False:
            return fail("E_STUDIO_RELEASE_WAIT_STATE_FALSE_FLAG", f"{key}={wait_state.get(key)!r}")
    expected_closed = [
        "STUDIO_POST_APPROVAL_CHAIN_MAINTENANCE_QUEUE_V1",
        "STUDIO_RELEASE_APPROVAL_STATUS_SNAPSHOT_V1",
        "STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1",
        "STUDIO_STALE_RELEASE_DOC_AUDIT_V1",
    ]
    if wait_state.get("closed_maintenance_items") != expected_closed:
        return fail("E_STUDIO_RELEASE_WAIT_STATE_CLOSED_ITEMS", repr(wait_state.get("closed_maintenance_items")))
    return 0


def check_previous_audit_boundary() -> int:
    audit = load_json(STALE_AUDIT)
    if audit.get("next_state") != "AWAIT_EXPLICIT_RELEASE_APPROVAL":
        return fail("E_STUDIO_RELEASE_WAIT_STATE_PREV_AUDIT_STATE", repr(audit.get("next_state")))
    for key in (
        "release_execution_claim",
        "public_release_claim",
        "github_release_claim",
        "public_upload_claim",
        "asset_generation_claim",
        "execution_approval_claim",
    ):
        if audit.get(key) is not False:
            return fail("E_STUDIO_RELEASE_WAIT_STATE_PREV_AUDIT_FLAG", f"{key}={audit.get(key)!r}")
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1",
        "studio release approval wait state closure sealed",
        "next: AWAIT_EXPLICIT_RELEASE_APPROVAL",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_RELEASE_WAIT_STATE_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_release_approval_wait_state_closure_v1"],
        ["python", "tests/run_studio_stale_release_doc_audit_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd)
        if proc.returncode != 0:
            return fail("E_STUDIO_RELEASE_WAIT_STATE_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1",
        "studio_release_approval_wait_state_closure_v1",
        "docs/studio/RELEASE_APPROVAL_WAIT_STATE_CLOSURE_V1.md",
        "run_studio_release_approval_wait_state_closure_check.py",
        "AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_RELEASE_WAIT_STATE_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_RELEASE_WAIT_STATE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_RELEASE_WAIT_STATE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_wait_state,
        check_previous_audit_boundary,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-release-approval-wait-state-closure-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
