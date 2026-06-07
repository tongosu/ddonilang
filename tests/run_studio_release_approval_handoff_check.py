#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_RELEASE_APPROVAL_HANDOFF_V1.md"
PREV = ROOT / "STUDIO_RELEASE_APPROVAL_PACKET_TEXT_EXPORT_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
HANDOFF_DOC = ROOT / "docs" / "studio" / "RELEASE_APPROVAL_HANDOFF_V1.md"
PACK = ROOT / "pack" / "studio_release_approval_handoff_v1"
MANIFEST = PACK / "handoff.detjson"
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
PREFLIGHTS = [
    "python tests/run_studio_public_release_smoke_matrix_check.py",
    "python tests/run_studio_public_release_asset_plan_check.py",
    "python tests/run_studio_release_candidate_check.py",
    "git status --short -- docs/ssot",
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
        HANDOFF_DOC,
        PACK / "README.md",
        PACK / "contract.detjson",
        MANIFEST,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "run_studio_release_approval_packet_text_export_check.py",
    ]
    if MANIFEST.exists():
        for source in load_json(MANIFEST).get("review_packet", []):
            required.append(ROOT / source)
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_MISSING", str(missing))
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
                "STUDIO_RELEASE_APPROVAL_HANDOFF_V1",
                "local handoff checklist",
                "no release archives",
                "STUDIO_RELEASE_APPROVAL_PACKET_TEXT_EXPORT_V1",
                "STUDIO_PUBLIC_RELEASE_EXECUTION_V1",
                "docs/ssot/**",
            ],
            "E_STUDIO_RELEASE_APPROVAL_HANDOFF_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_RELEASE_APPROVAL_PACKET_TEXT_EXPORT_V1",
                "STUDIO_RELEASE_APPROVAL_HANDOFF_V1",
            ],
            "E_STUDIO_RELEASE_APPROVAL_HANDOFF_PREV",
        ),
        (
            INDEX,
            [
                "STUDIO_RELEASE_APPROVAL_HANDOFF_V1",
                "docs/studio/RELEASE_APPROVAL_HANDOFF_V1.md",
                "pack/studio_release_approval_handoff_v1",
                "tests/run_studio_release_approval_handoff_check.py",
            ],
            "E_STUDIO_RELEASE_APPROVAL_HANDOFF_INDEX",
        ),
        (
            HANDOFF_DOC,
            [
                "Studio Release Approval Handoff V1",
                "Status: local handoff checklist",
                REQUIRED_APPROVAL,
                "Generic requests",
                "Review Packet",
                "Required Preflight Before Any Execution",
                "Still Blocked Without Approval",
                "release_execution_claim=false",
                "The executable release step remains blocked",
            ],
            "E_STUDIO_RELEASE_APPROVAL_HANDOFF_REPORT",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_manifest() -> int:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_release_approval_handoff_v1",
        "kind": "studio_release_approval_handoff",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_RELEASE_APPROVAL_HANDOFF_V1",
        "handoff": "docs/studio/RELEASE_APPROVAL_HANDOFF_V1.md",
        "manifest": "pack/studio_release_approval_handoff_v1/handoff.detjson",
        "based_on": "STUDIO_RELEASE_APPROVAL_PACKET_TEXT_EXPORT_V1",
        "required_approval_phrase": REQUIRED_APPROVAL,
        "release_execution_claim": False,
        "public_release_claim": False,
        "github_release_claim": False,
        "public_upload_claim": False,
        "asset_generation_claim": False,
        "execution_approval_claim": False,
        "next_boundary": "STUDIO_PUBLIC_RELEASE_EXECUTION_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_CONTRACT", f"{key}={contract.get(key)!r}")

    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.studio.release_approval_handoff.v1":
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_SCHEMA", repr(manifest.get("schema")))
    if manifest.get("required_approval_phrase") != REQUIRED_APPROVAL:
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_APPROVAL", repr(manifest.get("required_approval_phrase")))
    if manifest.get("generic_next_dev_request_is_approval") is not False:
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_GENERIC_APPROVAL", repr(manifest.get("generic_next_dev_request_is_approval")))
    if manifest.get("preflight_commands") != PREFLIGHTS:
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_PREFLIGHTS", repr(manifest.get("preflight_commands")))
    if manifest.get("blocked_until_approval") != BLOCKED:
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_BLOCKED", repr(manifest.get("blocked_until_approval")))
    for flag in ("release_execution_claim", "public_release_claim", "github_release_claim", "public_upload_claim", "asset_generation_claim", "execution_approval_claim"):
        if manifest.get(flag) is not False:
            return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_FLAG", f"{flag}={manifest.get(flag)!r}")
    return 0


def check_handoff_sources() -> int:
    manifest = load_json(MANIFEST)
    text = read(HANDOFF_DOC)
    for source in manifest["review_packet"]:
        if not (ROOT / source).exists():
            return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_SOURCE_MISSING", source)
        if source not in text:
            return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_SOURCE_UNLISTED", source)
    for cmd in manifest["preflight_commands"]:
        if cmd not in text:
            return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_PREFLIGHT_UNLISTED", cmd)
    for blocked in manifest["blocked_until_approval"]:
        if blocked not in text:
            return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_BLOCKED_UNLISTED", blocked)
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_RELEASE_APPROVAL_HANDOFF_V1",
        "studio release approval handoff sealed",
        "next: STUDIO_PUBLIC_RELEASE_EXECUTION_V1 requires exact approval",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_release_approval_handoff_v1"],
        ["python", "tests/run_studio_release_approval_packet_text_export_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd)
        if proc.returncode != 0:
            return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_RELEASE_APPROVAL_HANDOFF_V1",
        "studio_release_approval_handoff_v1",
        "docs/studio/RELEASE_APPROVAL_HANDOFF_V1.md",
        "run_studio_release_approval_handoff_check.py",
        "STUDIO_PUBLIC_RELEASE_EXECUTION_V1",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_manifest,
        check_handoff_sources,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-release-approval-handoff-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
