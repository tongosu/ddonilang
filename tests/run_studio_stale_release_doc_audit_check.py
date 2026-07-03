#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "queue" / "STUDIO_STALE_RELEASE_DOC_AUDIT_V1.md"
PREV = ROOT / "docs" / "context" / "queue" / "STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
REPORT = ROOT / "docs" / "studio" / "STALE_RELEASE_DOC_AUDIT_V1.md"
PACK = ROOT / "pack" / "studio_stale_release_doc_audit_v1"
AUDIT = PACK / "audit.detjson"
REQUIRED_APPROVAL = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


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
        AUDIT,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "run_studio_release_approval_fast_check.py",
    ]
    if AUDIT.exists():
        required.extend(ROOT / path for path in load_json(AUDIT).get("audited_paths", []))
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_STALE_RELEASE_DOC_AUDIT_MISSING", str(missing))
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
                "STUDIO_STALE_RELEASE_DOC_AUDIT_V1",
                "local non-SSOT Studio release documents",
                "no release archives",
                "STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1",
                "docs/ssot/**",
            ],
            "E_STUDIO_STALE_RELEASE_DOC_AUDIT_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1",
                "STUDIO_STALE_RELEASE_DOC_AUDIT_V1",
            ],
            "E_STUDIO_STALE_RELEASE_DOC_AUDIT_PREV",
        ),
        (
            INDEX,
            [
                "STUDIO_STALE_RELEASE_DOC_AUDIT_V1",
                "docs/studio/STALE_RELEASE_DOC_AUDIT_V1.md",
                "pack/studio_stale_release_doc_audit_v1",
                "tests/run_studio_stale_release_doc_audit_check.py",
            ],
            "E_STUDIO_STALE_RELEASE_DOC_AUDIT_INDEX",
        ),
        (
            REPORT,
            [
                "Studio Stale Release Doc Audit V1",
                "Status: local non-SSOT release wording audit",
                REQUIRED_APPROVAL,
                "stale approval wording: not found",
                "opens no release execution item",
            ],
            "E_STUDIO_STALE_RELEASE_DOC_AUDIT_REPORT",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_audit() -> int:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_stale_release_doc_audit_v1",
        "kind": "studio_stale_release_doc_audit",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_STALE_RELEASE_DOC_AUDIT_V1",
        "audit": "pack/studio_stale_release_doc_audit_v1/audit.detjson",
        "report": "docs/studio/STALE_RELEASE_DOC_AUDIT_V1.md",
        "based_on": "STUDIO_RELEASE_APPROVAL_FAST_CHECK_V1",
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
            return fail("E_STUDIO_STALE_RELEASE_DOC_AUDIT_CONTRACT", f"{key}={contract.get(key)!r}")

    audit = load_json(AUDIT)
    if audit.get("schema") != "ddn.studio.stale_release_doc_audit.v1":
        return fail("E_STUDIO_STALE_RELEASE_DOC_AUDIT_SCHEMA", repr(audit.get("schema")))
    if audit.get("required_approval_phrase") != REQUIRED_APPROVAL:
        return fail("E_STUDIO_STALE_RELEASE_DOC_AUDIT_APPROVAL", repr(audit.get("required_approval_phrase")))
    if audit.get("audit_scope") != "local_non_ssot_release_docs":
        return fail("E_STUDIO_STALE_RELEASE_DOC_AUDIT_SCOPE", repr(audit.get("audit_scope")))
    for flag in ("release_execution_claim", "public_release_claim", "github_release_claim", "public_upload_claim", "asset_generation_claim", "execution_approval_claim"):
        if audit.get(flag) is not False:
            return fail("E_STUDIO_STALE_RELEASE_DOC_AUDIT_FLAG", f"{flag}={audit.get(flag)!r}")
    return 0


def check_audited_text() -> int:
    audit = load_json(AUDIT)
    forbidden = audit["forbidden_phrases"]
    required_any = audit["required_boundary_tokens"]
    for item in audit["audited_paths"]:
        path = ROOT / item
        text = read(path)
        for line in text.splitlines():
            for phrase in forbidden:
                idx = line.find(phrase)
                if idx >= 0 and "No " not in line[:idx]:
                    return fail("E_STUDIO_STALE_RELEASE_DOC_AUDIT_FORBIDDEN", f"{item}: {phrase}")
        if not any(token in text for token in required_any):
            return fail("E_STUDIO_STALE_RELEASE_DOC_AUDIT_NO_BOUNDARY", item)
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_STALE_RELEASE_DOC_AUDIT_V1",
        "studio stale release doc audit sealed",
        "next: AWAIT_EXPLICIT_RELEASE_APPROVAL",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_STALE_RELEASE_DOC_AUDIT_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_stale_release_doc_audit_v1"],
        ["python", "tests/run_studio_release_approval_fast_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd)
        if proc.returncode != 0:
            return fail("E_STUDIO_STALE_RELEASE_DOC_AUDIT_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_STALE_RELEASE_DOC_AUDIT_V1",
        "studio_stale_release_doc_audit_v1",
        "docs/studio/STALE_RELEASE_DOC_AUDIT_V1.md",
        "run_studio_stale_release_doc_audit_check.py",
        "AWAIT_EXPLICIT_RELEASE_APPROVAL",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_STALE_RELEASE_DOC_AUDIT_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_STALE_RELEASE_DOC_AUDIT_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_STALE_RELEASE_DOC_AUDIT_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_audit,
        check_audited_text,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-stale-release-doc-audit-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
