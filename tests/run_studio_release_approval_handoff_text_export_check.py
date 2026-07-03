#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "docs" / "context" / "queue" / "STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_EXPORT_V1.md"
PREV = ROOT / "docs" / "context" / "queue" / "STUDIO_RELEASE_APPROVAL_HANDOFF_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
SOURCE_HANDOFF = ROOT / "docs" / "studio" / "RELEASE_APPROVAL_HANDOFF_V1.md"
TEXT_EXPORT = ROOT / "docs" / "studio" / "RELEASE_APPROVAL_HANDOFF_V1.txt"
PACK = ROOT / "pack" / "studio_release_approval_handoff_text_export_v1"
MANIFEST = PACK / "text_export_manifest.detjson"
HANDOFF = ROOT / "pack" / "studio_release_approval_handoff_v1" / "handoff.detjson"
REQUIRED_APPROVAL = "STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다"


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
        SOURCE_HANDOFF,
        TEXT_EXPORT,
        PACK / "README.md",
        PACK / "contract.detjson",
        MANIFEST,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        HANDOFF,
        ROOT / "tests" / "run_studio_release_approval_handoff_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_MISSING", str(missing))
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
                "STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_EXPORT_V1",
                "deterministic plain text",
                "no release archives",
                "STUDIO_RELEASE_APPROVAL_HANDOFF_V1",
                "STUDIO_PUBLIC_RELEASE_EXECUTION_V1",
                "docs/ssot/**",
            ],
            "E_STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_RELEASE_APPROVAL_HANDOFF_V1",
                "STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_EXPORT_V1",
            ],
            "E_STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_PREV",
        ),
        (
            INDEX,
            [
                "STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_EXPORT_V1",
                "docs/studio/RELEASE_APPROVAL_HANDOFF_V1.txt",
                "pack/studio_release_approval_handoff_text_export_v1",
                "tests/run_studio_release_approval_handoff_text_export_check.py",
            ],
            "E_STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_INDEX",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def expected_text(handoff: dict) -> str:
    lines = [
        "Studio Release Approval Handoff V1",
        "Status: local handoff checklist; not approval and not public release execution.",
        f"Required approval phrase: {handoff['required_approval_phrase']}",
        "Generic requests such as next development, continue, or proceed are not approval.",
        "",
        "Review packet:",
    ]
    lines.extend(f"- {item}" for item in handoff["review_packet"])
    lines.extend(["", "Required preflight before any execution:"])
    lines.extend(f"- {item}" for item in handoff["preflight_commands"])
    lines.extend(["", "Still blocked without approval:"])
    lines.extend(f"- {item}" for item in handoff["blocked_until_approval"])
    lines.extend(
        [
            "",
            "Current claims:",
            f"- release_execution_claim={str(handoff['release_execution_claim']).lower()}",
            f"- public_release_claim={str(handoff['public_release_claim']).lower()}",
            f"- github_release_claim={str(handoff['github_release_claim']).lower()}",
            f"- public_upload_claim={str(handoff['public_upload_claim']).lower()}",
            f"- asset_generation_claim={str(handoff['asset_generation_claim']).lower()}",
            f"- execution_approval_claim={str(handoff['execution_approval_claim']).lower()}",
            "",
            "Handoff result: local approval materials are ready for review. STUDIO_PUBLIC_RELEASE_EXECUTION_V1 remains blocked until the exact approval phrase is provided.",
        ]
    )
    return "\n".join(lines)


def check_contract_and_manifest() -> int:
    contract = load_json(PACK / "contract.detjson")
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_release_approval_handoff_text_export_v1",
        "kind": "studio_release_approval_handoff_text_export",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_EXPORT_V1",
        "source_handoff": "docs/studio/RELEASE_APPROVAL_HANDOFF_V1.md",
        "text_export": "docs/studio/RELEASE_APPROVAL_HANDOFF_V1.txt",
        "manifest": "pack/studio_release_approval_handoff_text_export_v1/text_export_manifest.detjson",
        "based_on": "STUDIO_RELEASE_APPROVAL_HANDOFF_V1",
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
            return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_CONTRACT", f"{key}={contract.get(key)!r}")

    manifest = load_json(MANIFEST)
    if manifest.get("schema") != "ddn.studio.release_approval_handoff_text_export.v1":
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_SCHEMA", repr(manifest.get("schema")))
    if manifest.get("text_export") != "docs/studio/RELEASE_APPROVAL_HANDOFF_V1.txt":
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_PATH", repr(manifest.get("text_export")))
    for flag in ("release_execution_claim", "public_release_claim", "github_release_claim", "public_upload_claim", "asset_generation_claim", "execution_approval_claim"):
        if manifest.get(flag) is not False:
            return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_FLAG", f"{flag}={manifest.get(flag)!r}")
    return 0


def check_text_export() -> int:
    handoff = load_json(HANDOFF)
    text = read(TEXT_EXPORT).rstrip("\n")
    if text != expected_text(handoff):
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_CONTENT", "text export does not match handoff manifest")
    manifest = load_json(MANIFEST)
    for section in manifest["required_sections"]:
        if section not in text:
            return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_SECTION", section)
    for token in manifest["required_tokens"]:
        if token not in text:
            return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_TOKEN", token)
    if "#" in text or "```" in text or "`" in text:
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_MARKDOWN", "text export should stay plain text")
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_EXPORT_V1",
        "studio release approval handoff text export sealed",
        "next: STUDIO_PUBLIC_RELEASE_EXECUTION_V1 requires exact approval",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_release_approval_handoff_text_export_v1"],
        ["python", "tests/run_studio_release_approval_handoff_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd)
        if proc.returncode != 0:
            return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_EXPORT_V1",
        "studio_release_approval_handoff_text_export_v1",
        "docs/studio/RELEASE_APPROVAL_HANDOFF_V1.txt",
        "run_studio_release_approval_handoff_text_export_check.py",
        "STUDIO_PUBLIC_RELEASE_EXECUTION_V1",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_RELEASE_APPROVAL_HANDOFF_TEXT_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_manifest,
        check_text_export,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-release-approval-handoff-text-export-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
