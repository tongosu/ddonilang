#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1.md"
PREV = ROOT / "STUDIO_RELEASE_APPROVAL_STATUS_SNAPSHOT_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_release_approval_fast_check_v1"
FAST = PACK / "fast_check.detjson"
REQUIRED_APPROVAL = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다"
BLOCKED = [
    "github_release_create",
    "public_upload",
    "registry_publish",
    "cloud_sync",
    "account_setup",
    "artifact_signing",
    "publication_archive_generation",
    "checksum_manifest_generation_for_publication",
]
FALSE_FLAGS = [
    "release_execution_claim",
    "public_release_claim",
    "github_release_claim",
    "public_upload_claim",
    "asset_generation_claim",
]


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def run(cmd: list[str], *, timeout: int = 90) -> subprocess.CompletedProcess[str]:
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


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def require_files() -> int:
    required = [
        DOC,
        PREV,
        INDEX,
        PACK / "README.md",
        PACK / "contract.detjson",
        FAST,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
    ]
    if FAST.exists():
        required.extend(ROOT / source for source in load_json(FAST).get("structural_sources", []))
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_RELEASE_APPROVAL_FAST_MISSING", str(missing))
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
                "STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1",
                "fast structural checker",
                "does not invoke the full nested readiness/checker chain",
                "STUDIO_RELEASE_APPROVAL_STATUS_SNAPSHOT_V1",
                "STUDIO_STALE_RELEASE_DOC_AUDIT_V1",
                "docs/ssot/**",
            ],
            "E_STUDIO_RELEASE_APPROVAL_FAST_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_RELEASE_APPROVAL_STATUS_SNAPSHOT_V1",
                "STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1",
            ],
            "E_STUDIO_RELEASE_APPROVAL_FAST_PREV",
        ),
        (
            INDEX,
            [
                "STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1",
                "pack/studio_release_approval_fast_check_v1",
                "tests/run_studio_release_approval_fast_check.py",
            ],
            "E_STUDIO_RELEASE_APPROVAL_FAST_INDEX",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_fast_manifest() -> int:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_release_approval_fast_check_v1",
        "kind": "studio_release_approval_fast_check",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1",
        "fast_check": "pack/studio_release_approval_fast_check_v1/fast_check.detjson",
        "based_on": "STUDIO_RELEASE_APPROVAL_STATUS_SNAPSHOT_V1",
        "current_state": "AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "required_approval_phrase": REQUIRED_APPROVAL,
        "nested_readiness_execution": False,
        "release_execution_selected": False,
        "generic_next_dev_request_is_approval": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "github_release_claim": False,
        "public_upload_claim": False,
        "asset_generation_claim": False,
        "execution_approval_claim": False,
        "next_item": "STUDIO_STALE_RELEASE_DOC_AUDIT_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_RELEASE_APPROVAL_FAST_CONTRACT", f"{key}={contract.get(key)!r}")

    fast = load_json(FAST)
    if fast.get("schema") != "ddn.studio.release_approval_fast_check.v1":
        return fail("E_STUDIO_RELEASE_APPROVAL_FAST_SCHEMA", repr(fast.get("schema")))
    if fast.get("nested_readiness_execution") is not False:
        return fail("E_STUDIO_RELEASE_APPROVAL_FAST_NESTED", repr(fast.get("nested_readiness_execution")))
    if fast.get("blocked_until_approval") != BLOCKED:
        return fail("E_STUDIO_RELEASE_APPROVAL_FAST_BLOCKED", repr(fast.get("blocked_until_approval")))
    if fast.get("next_item") != "STUDIO_STALE_RELEASE_DOC_AUDIT_V1":
        return fail("E_STUDIO_RELEASE_APPROVAL_FAST_NEXT", repr(fast.get("next_item")))
    return 0


def optional_bool(data: dict, key: str) -> bool | None:
    value = data.get(key)
    if isinstance(value, bool):
        return value
    return None


def check_structural_sources() -> int:
    fast = load_json(FAST)
    for source in fast["structural_sources"]:
        data = load_json(ROOT / source)
        if data.get("required_approval_phrase", REQUIRED_APPROVAL) != REQUIRED_APPROVAL:
            return fail("E_STUDIO_RELEASE_APPROVAL_FAST_APPROVAL", source)
        if data.get("current_state", "AWAIT_EXPLICIT_RELEASE_APPROVAL") != "AWAIT_EXPLICIT_RELEASE_APPROVAL":
            return fail("E_STUDIO_RELEASE_APPROVAL_FAST_STATE", source)
        generic = optional_bool(data, "generic_next_dev_request_is_approval")
        if generic is True:
            return fail("E_STUDIO_RELEASE_APPROVAL_FAST_GENERIC_APPROVAL", source)
        selected = optional_bool(data, "release_execution_selected")
        if selected is True:
            return fail("E_STUDIO_RELEASE_APPROVAL_FAST_RELEASE_SELECTED", source)
        blocked = data.get("blocked_until_approval") or data.get("blocked_in_dry_run")
        if blocked is not None and blocked != BLOCKED:
            return fail("E_STUDIO_RELEASE_APPROVAL_FAST_SOURCE_BLOCKED", source)
        for flag in FALSE_FLAGS:
            value = optional_bool(data, flag)
            if value is True:
                return fail("E_STUDIO_RELEASE_APPROVAL_FAST_SOURCE_FLAG", f"{source} {flag}=true")
        approval_flag = optional_bool(data, "execution_approval_claim")
        if approval_flag is True:
            return fail("E_STUDIO_RELEASE_APPROVAL_FAST_APPROVAL_FLAG", source)
    return 0


def check_no_nested_checker_call() -> int:
    text = read(ROOT / "tests" / "run_studio_release_approval_fast_check.py")
    forbidden = [
        "run_" + "studio_release_approval_status_snapshot_check.py",
        "run_" + "studio_post_approval_chain_maintenance_queue_check.py",
        "run_" + "studio_release_approval_chain_closure_check.py",
    ]
    used = [item for item in forbidden if item in text]
    if used:
        return fail("E_STUDIO_RELEASE_APPROVAL_FAST_FORBIDDEN_NESTED_CALL", repr(used))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1",
        "studio release approval fast check sealed",
        "next: STUDIO_STALE_RELEASE_DOC_AUDIT_V1",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_RELEASE_APPROVAL_FAST_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    proc = run(["python", "tests/run_pack_golden.py", "studio_release_approval_fast_check_v1"])
    if proc.returncode != 0:
        return fail("E_STUDIO_RELEASE_APPROVAL_FAST_GOLDEN_GATE", proc.stdout.strip())
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1",
        "studio_release_approval_fast_check_v1",
        "run_studio_release_approval_fast_check.py",
        "STUDIO_STALE_RELEASE_DOC_AUDIT_V1",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_RELEASE_APPROVAL_FAST_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_RELEASE_APPROVAL_FAST_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_RELEASE_APPROVAL_FAST_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_fast_manifest,
        check_structural_sources,
        check_no_nested_checker_call,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-release-approval-fast-check-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
