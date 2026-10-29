#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "settings_keyset_policy_v1"
sys.path.insert(0, str(ROOT / "tests" / "tools"))
import config_digest_proto  # noqa: E402


def fail(message: str) -> None:
    print(f"settings_keyset_policy_check: FAIL: {message}", file=sys.stderr)
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


def digest_report(source: str) -> dict:
    payload = config_digest_proto.canonical_payload(source)
    report = config_digest_proto.hash_report(payload)
    report["canonical"] = payload
    return report


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "settings_keyset_policy_v1"])
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_contract() -> None:
    contract = read_json(PACK / "contract.detjson")
    for field in ("parser_landed_claim", "runtime_claim", "product_code_change", "state_hash_formula_change"):
        if contract.get(field) is not False:
            fail(f"{field} must remain false")

    keysets = contract.get("keysets", {})
    nuri = keysets.get("설정.누리", {})
    if nuri.get("closed_keys") != ["시앗", "시앗묶음"]:
        fail(f"설정.누리 closed keyset mismatch: {nuri.get('closed_keys')}")
    if nuri.get("result_affecting") != ["시앗", "시앗묶음"]:
        fail(f"설정.누리 result-affecting keyset mismatch: {nuri.get('result_affecting')}")
    if nuri.get("aliases") != {"seed": "시앗", "seed_bag": "시앗묶음", "seed_streams": "시앗묶음"}:
        fail(f"설정.누리 alias map mismatch: {nuri.get('aliases')}")

    story = keysets.get("설정.이야기", {})
    if story.get("closed_keys") != ["들머리"]:
        fail(f"설정.이야기 closed keyset mismatch: {story.get('closed_keys')}")
    if story.get("result_affecting") != ["들머리"]:
        fail(f"설정.이야기 result-affecting keyset mismatch: {story.get('result_affecting')}")
    if story.get("aliases") != {"entry": "들머리"}:
        fail(f"설정.이야기 alias map mismatch: {story.get('aliases')}")

    sem = keysets.get("설정.셈", {})
    if sem.get("status") != "deferred_until_separate_keyset_decision":
        fail(f"설정.셈 must remain deferred: {sem}")
    if sem.get("known_artifact_behavior") != ["적분"]:
        fail(f"설정.셈 known artifact behavior mismatch: {sem}")

    policy = contract.get("hash_policy", {})
    expected_true = (
        "doc_run_hash_excluded",
        "story_entry_run_hash_included",
        "nuri_seed_run_hash_included",
        "nuri_seed_streams_run_hash_included",
    )
    for key in expected_true:
        if policy.get(key) is not True:
            fail(f"hash_policy.{key} must be true")
    if policy.get("state_hash_formula_change") is not False:
        fail("hash_policy.state_hash_formula_change must be false")


def check_digest_policy() -> None:
    base = """
설정 {
  누리 { 시앗: 7. 시앗묶음: "physics,policy". }.
  이야기 { 들머리: "start". }.
  문서 { 제목: "A". }.
}.
"""
    alias_variant = """
설정 {
  world { seed: 7. seed_streams: "physics,policy". }.
  story { entry: "start". }.
  doc { title: "A". }.
}.
"""
    doc_variant = """
설정 {
  누리 { 시앗: 7. 시앗묶음: "physics,policy". }.
  이야기 { 들머리: "start". }.
  문서 { 제목: "B". }.
}.
"""
    story_variant = """
설정 {
  누리 { 시앗: 7. 시앗묶음: "physics,policy". }.
  이야기 { 들머리: "alternate". }.
  문서 { 제목: "A". }.
}.
"""
    nuri_seed_variant = """
설정 {
  누리 { 시앗: 8. 시앗묶음: "physics,policy". }.
  이야기 { 들머리: "start". }.
  문서 { 제목: "A". }.
}.
"""
    nuri_stream_variant = """
설정 {
  누리 { 시앗: 7. 시앗묶음: "physics,policy,agent". }.
  이야기 { 들머리: "start". }.
  문서 { 제목: "A". }.
}.
"""

    base_report = digest_report(base)
    alias_report = digest_report(alias_variant)
    if base_report["canonical"] != alias_report["canonical"]:
        fail("누리/이야기 alias variant canonical payload mismatch")
    if base_report["hashes"] != alias_report["hashes"]:
        fail("누리/이야기 alias variant hash mismatch")

    doc_report = digest_report(doc_variant)
    if base_report["hashes"]["run_config_hash"] != doc_report["hashes"]["run_config_hash"]:
        fail("문서.제목 must not affect run_config_hash")
    if base_report["hashes"]["doc_config_hash"] == doc_report["hashes"]["doc_config_hash"]:
        fail("문서.제목 should affect doc_config_hash")

    story_report = digest_report(story_variant)
    if base_report["hashes"]["run_config_hash"] == story_report["hashes"]["run_config_hash"]:
        fail("이야기.들머리 must affect run_config_hash")

    seed_report = digest_report(nuri_seed_variant)
    if base_report["hashes"]["run_config_hash"] == seed_report["hashes"]["run_config_hash"]:
        fail("누리.시앗 must affect run_config_hash")

    stream_report = digest_report(nuri_stream_variant)
    if base_report["hashes"]["run_config_hash"] == stream_report["hashes"]["run_config_hash"]:
        fail("누리.시앗묶음 must affect run_config_hash")
    base_streams = base_report["seed_stream_fixture"]["streams"]
    variant_streams = stream_report["seed_stream_fixture"]["streams"]
    if base_streams == variant_streams:
        fail("시앗묶음 change must affect seed stream derivation fixture")

    rows = {
        row["path"]: row["class"]
        for row in base_report["result_affecting_classifier"]["rows"]
    }
    for path in ("누리.시앗", "누리.시앗묶음", "이야기.들머리"):
        if rows.get(path) != "result_affecting":
            fail(f"{path} classifier mismatch: {rows}")
    if rows.get("문서.제목") != "doc_affecting":
        fail(f"문서.제목 classifier mismatch: {rows}")


def check_tool_boundaries() -> None:
    if config_digest_proto.DOC_SECTIONS != {"문서"}:
        fail(f"DOC_SECTIONS must contain only 문서: {config_digest_proto.DOC_SECTIONS}")
    for section in ("누리", "이야기"):
        if section not in config_digest_proto.RUN_SECTIONS:
            fail(f"{section} section must be classified as run-affecting")
    if config_digest_proto.RESULT_AFFECTING_KEYS.get("이야기") != {"들머리"}:
        fail("이야기 result-affecting keys must be exactly 들머리")
    nuri_keys = config_digest_proto.RESULT_AFFECTING_KEYS.get("누리", set())
    for key in ("시앗", "시앗묶음"):
        if key not in nuri_keys:
            fail(f"누리.{key} must be result-affecting in digest prototype")

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
    check_digest_policy()
    check_tool_boundaries()
    print("settings_keyset_policy_check: PASS")


if __name__ == "__main__":
    main()
