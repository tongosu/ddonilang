#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "STUDIO_RELEASE_NOTES_TEXT_EXPORT_V1.md"
PREV = ROOT / "STUDIO_RELEASE_NOTES_DRAFT_V1.md"
INDEX = ROOT / "docs" / "studio" / "INDEX.md"
SOURCE_NOTES = ROOT / "docs" / "studio" / "RELEASE_NOTES_DRAFT_V1.md"
TEXT_EXPORT = ROOT / "docs" / "studio" / "RELEASE_NOTES_DRAFT_V1.txt"
PACK = ROOT / "pack" / "studio_release_notes_text_export_v1"
MANIFEST = PACK / "text_export_manifest.detjson"
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


def manifest() -> dict:
    return json.loads(MANIFEST.read_text(encoding="utf-8"))


def require_files() -> int:
    required = [
        DOC,
        PREV,
        INDEX,
        SOURCE_NOTES,
        TEXT_EXPORT,
        PACK / "README.md",
        PACK / "contract.detjson",
        MANIFEST,
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "pack" / "studio_release_notes_draft_v1" / "contract.detjson",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_STUDIO_RELEASE_NOTES_TEXT_MISSING", str(missing))
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
                "STUDIO_RELEASE_NOTES_TEXT_EXPORT_V1",
                "deterministic plain-text file",
                "not a public release",
                "STUDIO_RELEASE_NOTES_DRAFT_V1",
                "STUDIO_RELEASE_APPROVAL_READINESS_RECHECK_V1",
                "docs/ssot/**",
            ],
            "E_STUDIO_RELEASE_NOTES_TEXT_DOC",
        ),
        (
            PREV,
            [
                "STUDIO_RELEASE_NOTES_DRAFT_V1",
                "STUDIO_RELEASE_NOTES_TEXT_EXPORT_V1",
            ],
            "E_STUDIO_RELEASE_NOTES_TEXT_PREV",
        ),
        (
            INDEX,
            [
                "STUDIO_RELEASE_NOTES_TEXT_EXPORT_V1",
                "docs/studio/RELEASE_NOTES_DRAFT_V1.txt",
                "pack/studio_release_notes_text_export_v1",
                "tests/run_studio_release_notes_text_export_check.py",
            ],
            "E_STUDIO_RELEASE_NOTES_TEXT_INDEX",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_contract_and_manifest() -> int:
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "studio_release_notes_text_export_v1",
        "kind": "studio_release_notes_text_export",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "STUDIO_RELEASE_NOTES_TEXT_EXPORT_V1",
        "source_release_notes": "docs/studio/RELEASE_NOTES_DRAFT_V1.md",
        "text_export": "docs/studio/RELEASE_NOTES_DRAFT_V1.txt",
        "manifest": "pack/studio_release_notes_text_export_v1/text_export_manifest.detjson",
        "based_on": "STUDIO_RELEASE_NOTES_DRAFT_V1",
        "release_status": "local_draft_only",
        "public_release_claim": False,
        "github_release_claim": False,
        "public_upload_claim": False,
        "asset_generation_claim": False,
        "next_item": "STUDIO_RELEASE_APPROVAL_READINESS_RECHECK_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_STUDIO_RELEASE_NOTES_TEXT_CONTRACT", f"{key}={contract.get(key)!r}")

    data = manifest()
    if data.get("schema") != "ddn.studio.release_notes_text_export.v1":
        return fail("E_STUDIO_RELEASE_NOTES_TEXT_SCHEMA", repr(data.get("schema")))
    if data.get("text_export") != "docs/studio/RELEASE_NOTES_DRAFT_V1.txt":
        return fail("E_STUDIO_RELEASE_NOTES_TEXT_PATH", repr(data.get("text_export")))
    for flag in ("public_release_claim", "github_release_claim", "public_upload_claim", "asset_generation_claim"):
        if data.get(flag) is not False:
            return fail("E_STUDIO_RELEASE_NOTES_TEXT_FLAG", f"{flag}={data.get(flag)!r}")
    return 0


def check_text_export_content() -> int:
    text = read(TEXT_EXPORT)
    data = manifest()
    if "#" in text or "```" in text:
        return fail("E_STUDIO_RELEASE_NOTES_TEXT_MARKDOWN", "text export should stay plain text")
    for section in data["required_sections"]:
        if section not in text:
            return fail("E_STUDIO_RELEASE_NOTES_TEXT_SECTION", section)
    for token in data["required_tokens"]:
        if token not in text:
            return fail("E_STUDIO_RELEASE_NOTES_TEXT_TOKEN", token)
    for phrase in [
        "Status: local draft only; not a public release announcement.",
        "No GitHub Release was created.",
        "No public upload was performed.",
        "No public registry publishing was performed.",
        "No cloud sync, account setup, or permission system was added.",
        "No artifact signing was performed.",
        "No publication archive or public checksum manifest was generated.",
        "Generic next development requests are not release execution approval.",
    ]:
        if phrase not in text:
            return fail("E_STUDIO_RELEASE_NOTES_TEXT_BOUNDARY", phrase)
    if REQUIRED_APPROVAL not in read(SOURCE_NOTES):
        return fail("E_STUDIO_RELEASE_NOTES_TEXT_SOURCE_APPROVAL", REQUIRED_APPROVAL)
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "STUDIO_RELEASE_NOTES_TEXT_EXPORT_V1",
        "studio release notes text export sealed",
        "next: STUDIO_RELEASE_APPROVAL_READINESS_RECHECK_V1",
    ]
    if payload.get("stdout") != expected:
        return fail("E_STUDIO_RELEASE_NOTES_TEXT_GOLDEN", repr(payload.get("stdout")))
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "studio_release_notes_text_export_v1"],
        ["python", "tests/run_studio_release_notes_draft_check.py"],
    ]
    for cmd in commands:
        proc = run(cmd)
        if proc.returncode != 0:
            return fail("E_STUDIO_RELEASE_NOTES_TEXT_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_dev_summary() -> int:
    text = read(ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md")
    required = [
        "STUDIO_RELEASE_NOTES_TEXT_EXPORT_V1",
        "studio_release_notes_text_export_v1",
        "docs/studio/RELEASE_NOTES_DRAFT_V1.txt",
        "run_studio_release_notes_text_export_check.py",
        "STUDIO_RELEASE_APPROVAL_READINESS_RECHECK_V1",
        "docs/ssot/** 변경 없음",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_STUDIO_RELEASE_NOTES_TEXT_DEV_SUMMARY", str(missing))
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_STUDIO_RELEASE_NOTES_TEXT_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_STUDIO_RELEASE_NOTES_TEXT_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_docs,
        check_contract_and_manifest,
        check_text_export_content,
        check_golden,
        run_required_gates,
        check_dev_summary,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[studio-release-notes-text-export-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
