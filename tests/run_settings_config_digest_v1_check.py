#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "settings_config_digest_v1"
sys.path.insert(0, str(ROOT / "tests" / "tools"))
import config_digest_proto  # noqa: E402


def fail(message: str) -> None:
    print(f"settings_config_digest_v1_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def run(cmd: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
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


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8-sig"))


def digest_report(path: Path) -> dict:
    payload = config_digest_proto.canonical_payload(path.read_text(encoding="utf-8"))
    report = config_digest_proto.hash_report(payload)
    report["canonical"] = payload
    return report


def parse_selftest(stdout: str) -> dict:
    start = stdout.find("{")
    if start < 0:
        fail("config digest selftest JSON missing")
    return json.loads(stdout[start:])


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "settings_config_digest_v1"])
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_contract() -> None:
    contract = read_json(PACK / "contract.detjson")
    for field in ("parser_landed_claim", "runtime_claim", "product_code_change", "state_hash_formula_change"):
        if contract.get(field) is not False:
            fail(f"{field} must remain false")
    policy = contract.get("run_hash_policy", {})
    if policy.get("doc_section_excluded") != "문서":
        fail("contract must exclude only 문서 from run hash")
    if policy.get("story_entry_included") != "이야기.들머리":
        fail("contract must include 이야기.들머리 in run hash")


def check_selftest_policy_cases() -> None:
    proc = run([sys.executable, "tests/tools/config_digest_proto.py", "--selftest"])
    if proc.returncode != 0:
        fail(f"config digest selftest failed:\n{proc.stdout}")
    payload = parse_selftest(proc.stdout)
    cases = payload.get("policy_cases", {})
    if cases.get("doc_title_run_hash_unchanged") is not True:
        fail("selftest must confirm 문서 title does not affect run_config_hash")
    if cases.get("story_entry_run_hash_changed") is not True:
        fail("selftest must confirm 이야기.들머리 affects run_config_hash")
    if cases.get("story_entry_class") != "result_affecting":
        fail(f"story_entry_class mismatch: {cases}")


def check_case_pairs() -> None:
    alias_left = digest_report(PACK / "cases" / "c01_alias_left.ddn")
    alias_right = digest_report(PACK / "cases" / "c01_alias_right.ddn")
    if alias_left["canonical"] != alias_right["canonical"]:
        fail("alias pair canonical payload mismatch")
    if alias_left["hashes"] != alias_right["hashes"]:
        fail("alias pair hash mismatch")

    order_left = digest_report(PACK / "cases" / "c02_section_order_left.ddn")
    order_right = digest_report(PACK / "cases" / "c02_section_order_right.ddn")
    if order_left["canonical"] != order_right["canonical"]:
        fail("section-order pair canonical payload mismatch")
    if order_left["hashes"] != order_right["hashes"]:
        fail("section-order pair hash mismatch")

    doc_left = digest_report(PACK / "cases" / "c03_doc_excluded_left.ddn")
    doc_right = digest_report(PACK / "cases" / "c03_doc_excluded_right.ddn")
    if doc_left["hashes"]["run_config_hash"] != doc_right["hashes"]["run_config_hash"]:
        fail("문서 title must not affect run_config_hash")
    if doc_left["hashes"]["doc_config_hash"] == doc_right["hashes"]["doc_config_hash"]:
        fail("문서 title should affect doc_config_hash")

    story_left = digest_report(PACK / "cases" / "c04_story_entry_left.ddn")
    story_right = digest_report(PACK / "cases" / "c04_story_entry_right.ddn")
    if story_left["hashes"]["run_config_hash"] == story_right["hashes"]["run_config_hash"]:
        fail("이야기.들머리 must affect run_config_hash")
    rows = {
        row["path"]: row["class"]
        for row in story_left["result_affecting_classifier"]["rows"]
    }
    if rows.get("이야기.들머리") != "result_affecting":
        fail(f"이야기.들머리 classifier mismatch: {rows}")


def check_tool_boundaries() -> None:
    if config_digest_proto.DOC_SECTIONS != {"문서"}:
        fail(f"DOC_SECTIONS must contain only 문서: {config_digest_proto.DOC_SECTIONS}")
    if "이야기" not in config_digest_proto.RUN_SECTIONS:
        fail("이야기 section must be classified as run-affecting")
    if config_digest_proto.RESULT_AFFECTING_KEYS.get("이야기") != {"들머리"}:
        fail("이야기 result-affecting keys must be exactly 들머리")
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed:\n{proc.stdout}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")
    product = run(["git", "status", "--short", "--", "lang", "core", "tool", "tools/teul-cli/src"], timeout=60)
    if product.returncode != 0:
        fail(f"git status product paths failed:\n{product.stdout}")
    if product.stdout.strip():
        fail(f"product paths changed:\n{product.stdout}")


def main() -> None:
    check_pack_golden()
    check_contract()
    check_selftest_policy_cases()
    check_case_pairs()
    check_tool_boundaries()
    print("settings_config_digest_v1_check: PASS")


if __name__ == "__main__":
    main()
