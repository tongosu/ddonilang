#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1.md"
PACK = ROOT / "pack" / "lang_core_2_v1"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        DOC,
        ROOT / "ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1.md",
        PACK / "README.md",
        PACK / "fixtures" / "cases.detjson",
        PACK / "expected" / "lang_core_2.detjson",
        ROOT / "tests" / "run_lang_core_2_check.py",
        QUEUE,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_LANG_CORE_2_REPRESENTATIVE_MISSING", str(missing))
    return 0


def check_doc() -> int:
    text = read(DOC)
    required = [
        "ROADMAP_V2 `가-2`",
        "pack/lang_core_2_v1",
        "tests/run_lang_core_2_check.py",
        "채비",
        "훅",
        "조건",
        "임자",
        "계약",
        "adds no new grammar",
        "Python/JS-only lowering",
        "ROADMAP_V2_GA2_FINAL_CLOSURE_V1",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_LANG_CORE_2_REPRESENTATIVE_DOC", str(missing))
    return 0


def check_pack_contract() -> int:
    cases = json.loads((PACK / "fixtures" / "cases.detjson").read_text(encoding="utf-8"))
    expected = json.loads((PACK / "expected" / "lang_core_2.detjson").read_text(encoding="utf-8"))
    if cases.get("schema") != "ddn.lang_core_2_cases.v1":
        return fail("E_LANG_CORE_2_REPRESENTATIVE_CASE_SCHEMA", str(cases.get("schema")))
    if expected.get("schema") != "ddn.lang_core_2_report.v1":
        return fail("E_LANG_CORE_2_REPRESENTATIVE_EXPECTED_SCHEMA", str(expected.get("schema")))
    covered = set(expected.get("covered_surfaces", []))
    required = {"채비", "훅", "조건", "임자", "계약"}
    missing = sorted(required - covered)
    if missing:
        return fail("E_LANG_CORE_2_REPRESENTATIVE_SURFACES", str(missing))
    if len(cases.get("cases", [])) != len(expected.get("cases", [])):
        return fail(
            "E_LANG_CORE_2_REPRESENTATIVE_CASE_COUNT",
            f"{len(cases.get('cases', []))} != {len(expected.get('cases', []))}",
        )
    return 0


def run_lang_core_2() -> int:
    result = subprocess.run(
        ["python", "tests/run_lang_core_2_check.py"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode != 0:
        return fail("E_LANG_CORE_2_REPRESENTATIVE_CHECK", result.stdout.strip())
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1",
        "closed by `LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1.md`",
        "pack/lang_core_2_v1",
        "ROADMAP_V2_GA2_FINAL_CLOSURE_V1",
        "closed by `ROADMAP_V2_GA2_FINAL_CLOSURE_V1.md`",
        "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1",
        "closed by `ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1.md`",
        "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1",
        "closed by `BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1.md`",
        "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1",
        "closed by `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1.md`",
        "ROADMAP_V2_LA2_FINAL_CLOSURE_V1",
        "closed by `ROADMAP_V2_LA2_FINAL_CLOSURE_V1.md`",
        "ROADMAP_V2_A1_NURIGYM_REBASE_V1",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_LANG_CORE_2_REPRESENTATIVE_QUEUE", str(missing))
    if "1. `LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1`" in text:
        return fail(
            "E_LANG_CORE_2_REPRESENTATIVE_STILL_OPEN",
            "LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1 is still listed as open",
        )
    if "1. `ROADMAP_V2_GA2_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_LANG_CORE_2_REPRESENTATIVE_GA2_FINAL_OPEN",
            "ROADMAP_V2_GA2_FINAL_CLOSURE_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1`" in text:
        return fail(
            "E_LANG_CORE_2_REPRESENTATIVE_LA2_OPEN",
            "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1 is still listed as the next open item",
        )
    if "1. `BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1`" in text:
        return fail(
            "E_LANG_CORE_2_REPRESENTATIVE_BLOCK_EDITOR_REFRESH_OPEN",
            "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1 is still listed as the next open item",
        )
    if "1. `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1`" in text:
        return fail(
            "E_LANG_CORE_2_REPRESENTATIVE_MALBLOCK_ROUNDTRIP_NEXT",
            "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_LA2_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_LANG_CORE_2_REPRESENTATIVE_LA2_FINAL_NEXT",
            "ROADMAP_V2_LA2_FINAL_CLOSURE_V1 is still listed as the next open item",
        )
    return 0


def check_docs_ssot_clean() -> int:
    result = subprocess.run(
        ["git", "status", "--short", "--", "docs/ssot"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode != 0:
        return fail("E_LANG_CORE_2_REPRESENTATIVE_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_LANG_CORE_2_REPRESENTATIVE_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_doc,
        check_pack_contract,
        run_lang_core_2,
        check_queue,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[lang-core-2-representative-grammar-pack-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
