#!/usr/bin/env python
from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "creator_gaji_round3_ai_prompt_current_ssot_v1"
EVIDENCE = PACK / "r3_evidence.detjson"
TEMPLATE = ROOT / "tool" / "assets" / "ai_prompt_template_v20.0.3.txt"
SOURCE = ROOT / "tool" / "src" / "ai_prompt.rs"
GOLDEN = ROOT / "tests" / "toolchain_golden" / "ai_prompt_lean.txt"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def main() -> int:
    evidence = json.loads(EVIDENCE.read_text(encoding="utf-8"))
    if evidence.get("schema") != "ddn.creator_gaji_round3_ai_prompt_current_ssot.evidence.v1":
        return fail("E_R3_SCHEMA", repr(evidence.get("schema")))
    if evidence.get("internal_ssot_version") != "v24.15.1":
        return fail("E_R3_INTERNAL_VERSION", repr(evidence.get("internal_ssot_version")))
    if evidence.get("public_language_spec_version") != "v0.2.1" or evidence.get("version_namespaces_separate") is not True:
        return fail("E_R3_PUBLIC_VERSION_NAMESPACE", repr(evidence))
    for rel in evidence.get("authority_sources", {}).values():
        if not (ROOT / rel).is_file():
            return fail("E_R3_AUTHORITY_SOURCE", rel)

    template = TEMPLATE.read_text(encoding="utf-8")
    if any(line.startswith("SSOT_VERSION =") for line in template.splitlines()):
        return fail("E_R3_TEMPLATE_VERSION_OWNER", "static template contains SSOT_VERSION")
    source = SOURCE.read_text(encoding="utf-8")
    for token in [
        "SSOT_ALL_MANIFEST_",
        "require_matching_authority_versions",
        "SSOT authority version mismatch",
        "ai_prompt_rejects_index_manifest_version_mismatch",
    ]:
        if token not in source:
            return fail("E_R3_SOURCE_TOKEN", token)

    golden = GOLDEN.read_bytes()
    try:
        text = golden.decode("utf-8")
    except UnicodeDecodeError as err:
        return fail("E_R3_GOLDEN_UTF8", str(err))
    if golden.startswith(b"\xef\xbb\xbf"):
        return fail("E_R3_GOLDEN_BOM", "UTF-8 BOM present")
    if b"\r" in golden:
        return fail("E_R3_GOLDEN_CRLF", "CR byte present")
    if any(line != line.rstrip(" \t") for line in text.splitlines()):
        return fail("E_R3_GOLDEN_TRAILING_WS", "trailing whitespace present")
    version_lines = [line for line in text.splitlines() if line.startswith("SSOT_VERSION =")]
    if version_lines != ["SSOT_VERSION = v24.15.1"]:
        return fail("E_R3_VERSION_LINES", repr(version_lines))
    if "v20.6.33" in text:
        return fail("E_R3_STALE_VERSION", "v20.6.33 remains in generated output")
    if "SSOT_VERSION = v0.2.1" in text:
        return fail("E_R3_PUBLIC_AS_INTERNAL", "public spec version used as SSOT version")

    generation = evidence.get("generation", {})
    expected_header = [
        f"SSOT_VERSION = {evidence['internal_ssot_version']}",
        f"BUNDLE_KIND = {generation.get('bundle_kind')}",
        f"PROFILE = {generation.get('profile')}",
        f"BUNDLE_HASH = {generation.get('bundle_hash')}",
    ]
    for line in expected_header:
        if line not in text:
            return fail("E_R3_HEADER", line)
    for name in generation.get("file_list", []):
        if f"- {name}" not in text or f"===== BEGIN {name} =====" not in text:
            return fail("E_R3_FILE_LIST", name)
    if len(golden) != generation.get("golden_bytes"):
        return fail("E_R3_GOLDEN_BYTES", f"{len(golden)} != {generation.get('golden_bytes')}")
    digest = hashlib.sha256(golden).hexdigest()
    if digest != generation.get("golden_sha256"):
        return fail("E_R3_GOLDEN_HASH", digest)

    tests = evidence.get("tests", [])
    if len(tests) != 4 or any(row.get("result") != "PASS" for row in tests):
        return fail("E_R3_TESTS", repr(tests))
    claims = evidence.get("claims", {})
    for key, expected in {
        "r3_complete": True,
        "r0_through_r7_complete": False,
        "overall_goal_complete": False,
        "external_creator_pilot_success_count": 0,
        "docs_ssot_change": False,
    }.items():
        if claims.get(key) != expected:
            return fail("E_R3_CLAIM", f"{key}={claims.get(key)!r}")
    ssot = subprocess.run(
        ["git", "status", "--short", "--", "docs/ssot"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=60,
    )
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_R3_SSOT_DIRTY", ssot.stdout.strip())
    print("[creator-gaji-round3-ai-prompt-current-ssot-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
