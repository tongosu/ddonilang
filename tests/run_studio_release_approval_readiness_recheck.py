#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_RELEASE_APPROVAL_READINESS_RECHECK_V1.md"
PREV = ROOT / "STUDIO_RELEASE_NOTES_TEXT_EXPORT_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
PACK = ROOT / "pack" / "studio_release_approval_readiness_recheck_v1"
READINESS = PACK / "readiness.detjson"
EXECUTION_GATE = ROOT / "pack" / "studio_public_release_execution_gate_v1" / "execution_gate.detjson"
TEXT_EXPORT_MANIFEST = ROOT / "pack" / "studio_release_notes_text_export_v1" / "text_export_manifest.detjson"
DRAFT_NOTES = ROOT / "docs" / "studio" / "RELEASE_NOTES_DRAFT_V1.md"
TEXT_EXPORT = ROOT / "docs" / "studio" / "RELEASE_NOTES_DRAFT_V1.txt"
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
        PACK / "README.md",
        PACK / "contract.detjson",
        READINESS,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        EXECUTION_GATE,
        TEXT_EXPORT_MANIFEST,
        DRAFT_NOTES,
        TEXT_EXPORT,
        ROOT / "tests" / "run_studio_release_notes_text_export_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_RELEASE_READINESS_MISSING", str(missing))
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
                "STUDIO_RELEASE_APPROVAL_READINESS_RECHECK_V1",
                "does not execute a public release",
                REQUIRED_APPROVAL,
                "Generic \"next development\" requests are not approval",
                "STUDIO_RELEASE_PRE_EXECUTION_DRY_RUN_V1",
                "docs/ssot/**",
            ],
            "E_STUDIO_RELEASE_READINESS_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_RELEASE_NOTES_TEXT_EXPORT_V1",
                "STUDIO_RELEASE_APPROVAL_READINESS_RECHECK_V1",
            ],
            "E_STUDIO_RELEASE_READINESS_PREV",
        ),
        (
            INDEX,
            [
                "STUDIO_RELEASE_APPROVAL_READINESS_RECHECK_V1",
                "pack/studio_release_approval_readiness_recheck_v1",
                "tests/run_studio_release_approval_readiness_recheck.py",
            ],
            "E_STUDIO_RELEASE_READINESS_INDEX",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_readiness() -> int:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_release_approval_readiness_recheck_v1",
        "kind": "studio_release_approval_readiness_recheck",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_RELEASE_APPROVAL_READINESS_RECHECK_V1",
        "readiness": "pack/studio_release_approval_readiness_recheck_v1/readiness.detjson",
        "based_on": "STUDIO_RELEASE_NOTES_TEXT_EXPORT_V1",
        "required_approval_phrase": REQUIRED_APPROVAL,
        "generic_next_dev_request_is_approval": False,
        "release_execution_claim": False,
        "public_release_claim": False,
        "github_release_claim": False,
        "public_upload_claim": False,
        "asset_generation_claim": False,
        "next_item": "STUDIO_RELEASE_PRE_EXECUTION_DRY_RUN_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_RELEASE_READINESS_CONTRACT", f"{key}={contract.get(key)!r}")

    readiness = load_json(READINESS)
    if readiness.get("schema") != "ddn.studio.release_approval_readiness_recheck.v1":
        return fail("E_STUDIO_RELEASE_READINESS_SCHEMA", repr(readiness.get("schema")))
    if readiness.get("required_approval_phrase") != REQUIRED_APPROVAL:
        return fail("E_STUDIO_RELEASE_READINESS_APPROVAL", repr(readiness.get("required_approval_phrase")))
    if readiness.get("blocked_until_approval") != BLOCKED:
        return fail("E_STUDIO_RELEASE_READINESS_BLOCKED", repr(readiness.get("blocked_until_approval")))
    for flag in ("release_execution_claim", "public_release_claim", "github_release_claim", "public_upload_claim", "asset_generation_claim"):
        if readiness.get(flag) is not False:
            return fail("E_STUDIO_RELEASE_READINESS_FLAG", f"{flag}={readiness.get(flag)!r}")
    return 0


def check_cross_sources() -> int:
    readiness = load_json(READINESS)
    gate = load_json(EXECUTION_GATE)
    text_export = load_json(TEXT_EXPORT_MANIFEST)

    if gate.get("required_approval_phrase") != REQUIRED_APPROVAL:
        return fail("E_STUDIO_RELEASE_READINESS_GATE_APPROVAL", repr(gate.get("required_approval_phrase")))
    if gate.get("generic_next_dev_request_is_approval") is not False:
        return fail("E_STUDIO_RELEASE_READINESS_GATE_GENERIC", repr(gate.get("generic_next_dev_request_is_approval")))
    gate_ids = [item.get("id") for item in gate.get("preflight_gates", [])]
    if gate_ids != readiness.get("preflight_ids"):
        return fail("E_STUDIO_RELEASE_READINESS_PREFLIGHT_IDS", repr({"gate": gate_ids, "readiness": readiness.get("preflight_ids")}))
    if gate.get("blocked_until_approval") != BLOCKED:
        return fail("E_STUDIO_RELEASE_READINESS_GATE_BLOCKED", repr(gate.get("blocked_until_approval")))

    export_blocked = text_export.get("blocked_actions")
    normalized_export = ["checksum_manifest_generation_for_publication" if item == "public_checksum_manifest_generation" else item for item in export_blocked]
    if normalized_export != BLOCKED:
        return fail("E_STUDIO_RELEASE_READINESS_EXPORT_BLOCKED", repr(export_blocked))
    if REQUIRED_APPROVAL not in read(DRAFT_NOTES):
        return fail("E_STUDIO_RELEASE_READINESS_DRAFT_APPROVAL", REQUIRED_APPROVAL)
    if REQUIRED_APPROVAL not in read(TEXT_EXPORT):
        return fail("E_STUDIO_RELEASE_READINESS_TEXT_APPROVAL", REQUIRED_APPROVAL)
    if "Generic next development requests are not release execution approval." not in read(TEXT_EXPORT):
        return fail("E_STUDIO_RELEASE_READINESS_TEXT_GENERIC", "generic next request boundary missing")
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_RELEASE_APPROVAL_READINESS_RECHECK_V1",
        "studio release approval readiness recheck sealed",
        "next: STUDIO_RELEASE_PRE_EXECUTION_DRY_RUN_V1",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_RELEASE_READINESS_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_release_approval_readiness_recheck_v1"],
        ["python", "tests/run_studio_release_notes_text_export_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd)
        if proc.returncode != 0:
            return fail("E_STUDIO_RELEASE_READINESS_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_RELEASE_APPROVAL_READINESS_RECHECK_V1",
        "studio_release_approval_readiness_recheck_v1",
        "readiness.detjson",
        "run_studio_release_approval_readiness_recheck.py",
        "STUDIO_RELEASE_PRE_EXECUTION_DRY_RUN_V1",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_RELEASE_READINESS_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_RELEASE_READINESS_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_RELEASE_READINESS_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_readiness,
        check_cross_sources,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-release-approval-readiness-recheck-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
