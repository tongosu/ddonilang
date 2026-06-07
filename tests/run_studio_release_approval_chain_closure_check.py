#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_RELEASE_APPROVAL_CHAIN_CLOSURE_V1.md"
PREV = ROOT / "STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_EXPORT_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
REPORT = ROOT / "docs" / "studio" / "RELEASE_APPROVAL_CHAIN_CLOSURE_V1.md"
PACK = ROOT / "pack" / "studio_release_approval_chain_closure_v1"
CLOSURE = PACK / "closure.detjson"
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


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def run(cmd: list[str], *, timeout: int = 420) -> subprocess.CompletedProcess[str]:
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
        REPORT,
        PACK / "README.md",
        PACK / "contract.detjson",
        CLOSURE,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "run_studio_release_approval_handoff_text_export_check.py",
    ]
    if CLOSURE.exists():
        data = load_json(CLOSURE)
        required.extend(ROOT / path for path in data.get("review_docs", []))
        required.extend(ROOT / path / "contract.detjson" for path in data.get("closed_packs", []))
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_RELEASE_APPROVAL_CHAIN_MISSING", str(missing))
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
                "STUDIO_RELEASE_APPROVAL_CHAIN_CLOSURE_V1",
                "local Studio release approval chain",
                "no release archives",
                "STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_EXPORT_V1",
                "AWAIT_EXPLICIT_RELEASE_APPROVAL",
                "docs/ssot/**",
            ],
            "E_STUDIO_RELEASE_APPROVAL_CHAIN_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_EXPORT_V1",
                "STUDIO_RELEASE_APPROVAL_CHAIN_CLOSURE_V1",
            ],
            "E_STUDIO_RELEASE_APPROVAL_CHAIN_PREV",
        ),
        (
            INDEX,
            [
                "STUDIO_RELEASE_APPROVAL_CHAIN_CLOSURE_V1",
                "docs/studio/RELEASE_APPROVAL_CHAIN_CLOSURE_V1.md",
                "pack/studio_release_approval_chain_closure_v1",
                "tests/run_studio_release_approval_chain_closure_check.py",
            ],
            "E_STUDIO_RELEASE_APPROVAL_CHAIN_INDEX",
        ),
        (
            REPORT,
            [
                "Studio Release Approval Chain Closure V1",
                "Status: local approval-chain closure",
                REQUIRED_APPROVAL,
                "Closed Chain",
                "Still Blocked",
                "Current Claims",
                "No automatic next release execution item is open",
            ],
            "E_STUDIO_RELEASE_APPROVAL_CHAIN_REPORT",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_closure() -> int:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_release_approval_chain_closure_v1",
        "kind": "studio_release_approval_chain_closure",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_RELEASE_APPROVAL_CHAIN_CLOSURE_V1",
        "closure": "pack/studio_release_approval_chain_closure_v1/closure.detjson",
        "report": "docs/studio/RELEASE_APPROVAL_CHAIN_CLOSURE_V1.md",
        "based_on": "STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_EXPORT_V1",
        "required_approval_phrase": REQUIRED_APPROVAL,
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
            return fail("E_STUDIO_RELEASE_APPROVAL_CHAIN_CONTRACT", f"{key}={contract.get(key)!r}")

    closure = load_json(CLOSURE)
    if closure.get("schema") != "ddn.studio.release_approval_chain_closure.v1":
        return fail("E_STUDIO_RELEASE_APPROVAL_CHAIN_SCHEMA", repr(closure.get("schema")))
    if closure.get("required_approval_phrase") != REQUIRED_APPROVAL:
        return fail("E_STUDIO_RELEASE_APPROVAL_CHAIN_APPROVAL", repr(closure.get("required_approval_phrase")))
    if closure.get("generic_next_dev_request_is_approval") is not False:
        return fail("E_STUDIO_RELEASE_APPROVAL_CHAIN_GENERIC_APPROVAL", repr(closure.get("generic_next_dev_request_is_approval")))
    if closure.get("blocked_until_approval") != BLOCKED:
        return fail("E_STUDIO_RELEASE_APPROVAL_CHAIN_BLOCKED", repr(closure.get("blocked_until_approval")))
    for flag in ("release_execution_claim", "public_release_claim", "github_release_claim", "public_upload_claim", "asset_generation_claim", "execution_approval_claim"):
        if closure.get(flag) is not False:
            return fail("E_STUDIO_RELEASE_APPROVAL_CHAIN_FLAG", f"{flag}={closure.get(flag)!r}")
    return 0


def check_chain_alignment() -> int:
    closure = load_json(CLOSURE)
    report_text = read(REPORT)
    for pack_path in closure["closed_packs"]:
        contract = load_json(ROOT / pack_path / "contract.detjson")
        for flag in ("runtime_claim", "product_code_change"):
            if contract.get(flag) is not False:
                return fail("E_STUDIO_RELEASE_APPROVAL_CHAIN_PACK_FLAG", f"{pack_path} {flag}={contract.get(flag)!r}")
        if contract.get("required_approval_phrase", REQUIRED_APPROVAL) != REQUIRED_APPROVAL:
            return fail("E_STUDIO_RELEASE_APPROVAL_CHAIN_PACK_APPROVAL", pack_path)
        if pack_path not in report_text:
            return fail("E_STUDIO_RELEASE_APPROVAL_CHAIN_PACK_UNLISTED", pack_path)
    for doc_path in closure["review_docs"]:
        if not (ROOT / doc_path).exists():
            return fail("E_STUDIO_RELEASE_APPROVAL_CHAIN_REVIEW_DOC_MISSING", doc_path)
    for blocked in closure["blocked_until_approval"]:
        if blocked not in report_text:
            return fail("E_STUDIO_RELEASE_APPROVAL_CHAIN_BLOCKED_UNLISTED", blocked)
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_RELEASE_APPROVAL_CHAIN_CLOSURE_V1",
        "studio release approval chain closure sealed",
        "next: AWAIT_EXPLICIT_RELEASE_APPROVAL",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_RELEASE_APPROVAL_CHAIN_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_release_approval_chain_closure_v1"],
        ["python", "tests/run_studio_release_approval_handoff_text_export_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd)
        if proc.returncode != 0:
            return fail("E_STUDIO_RELEASE_APPROVAL_CHAIN_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_RELEASE_APPROVAL_CHAIN_CLOSURE_V1",
        "studio_release_approval_chain_closure_v1",
        "docs/studio/RELEASE_APPROVAL_CHAIN_CLOSURE_V1.md",
        "run_studio_release_approval_chain_closure_check.py",
        "AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_RELEASE_APPROVAL_CHAIN_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_RELEASE_APPROVAL_CHAIN_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_RELEASE_APPROVAL_CHAIN_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_closure,
        check_chain_alignment,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-release-approval-chain-closure-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
