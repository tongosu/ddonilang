#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROADMAP_V2_GA2_FINAL_CLOSURE_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"
REPORT = ROOT / "docs" / "status" / "roadmap_v2" / "가-2_REPORT_20260604.md"
LANG_CORE_2_EXPECTED = ROOT / "pack" / "lang_core_2_v1" / "expected" / "lang_core_2.detjson"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        DOC,
        ROOT / "LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1.md",
        ROOT / "ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1.md",
        ROOT / "pack" / "lang_core_2_v1" / "fixtures" / "cases.detjson",
        LANG_CORE_2_EXPECTED,
        ROOT / "tests" / "run_lang_core_2_check.py",
        ROOT / "tests" / "run_lang_core_2_representative_grammar_pack_check.py",
        REPORT,
        QUEUE,
        ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROADMAP_V2_GA2_FINAL_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_doc() -> int:
    return require_tokens(
        DOC,
        [
            "documentation/checker-only",
            "ROADMAP_V2 `가-2` is closed",
            "LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1",
            "lang_core_2_v1",
            "채비",
            "훅",
            "조건",
            "임자",
            "계약",
            "does not claim",
            "full actor/event dispatch semantics",
            "full contract/proof semantics",
            "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1",
            "docs/ssot/**",
        ],
        "E_ROADMAP_V2_GA2_FINAL_DOC",
    )


def check_report() -> int:
    return require_tokens(
        REPORT,
        [
            "ROADMAP_V2_GA2_FINAL_CLOSURE_V1",
            "대표 문법 pack",
            "lang_core_2_v1",
            "채비/훅/조건/임자/계약",
            "Matrix 상태 제안",
            "닫힘",
            "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1",
            "docs/ssot/**",
        ],
        "E_ROADMAP_V2_GA2_FINAL_REPORT",
    )


def check_lang_core_2_surfaces() -> int:
    payload = json.loads(LANG_CORE_2_EXPECTED.read_text(encoding="utf-8"))
    if payload.get("schema") != "ddn.lang_core_2_report.v1":
        return fail("E_ROADMAP_V2_GA2_FINAL_LANG_CORE_2_SCHEMA", str(payload.get("schema")))
    covered = set(payload.get("covered_surfaces", []))
    missing = sorted({"채비", "훅", "조건", "임자", "계약"} - covered)
    if missing:
        return fail("E_ROADMAP_V2_GA2_FINAL_SURFACES", str(missing))
    if len(payload.get("cases", [])) < 5:
        return fail("E_ROADMAP_V2_GA2_FINAL_CASE_COUNT", str(len(payload.get("cases", []))))
    return 0


def run_lang_core_2_wrapper() -> int:
    result = subprocess.run(
        ["python", "tests/run_lang_core_2_representative_grammar_pack_check.py"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode != 0:
        return fail("E_ROADMAP_V2_GA2_FINAL_LANG_CORE_2_CHECK", result.stdout.strip())
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "ROADMAP_V2_GA2_FINAL_CLOSURE_V1",
        "closed by `ROADMAP_V2_GA2_FINAL_CLOSURE_V1.md`",
        "ROADMAP_V2 `가-2` is closed",
        "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1",
        "closed by `ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1.md`",
        "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1",
        "closed by `BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1.md`",
        "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1",
        "closed by `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1.md`",
        "ROADMAP_V2_LA2_FINAL_CLOSURE_V1",
        "closed by `ROADMAP_V2_LA2_FINAL_CLOSURE_V1.md`",
        "ROADMAP_V2_A1_NURIGYM_REBASE_V1",
        "seamgrim_malblock_roundtrip_subset_v1",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_ROADMAP_V2_GA2_FINAL_QUEUE", str(missing))
    if "1. `ROADMAP_V2_GA2_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_GA2_FINAL_STILL_OPEN",
            "GA2 final closure is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_GA2_FINAL_LA2_OPEN",
            "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1 is still listed as the next open item",
        )
    if "1. `BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1`" in text:
        return fail(
            "E_ROADMAP_V2_GA2_FINAL_BLOCK_EDITOR_REFRESH_OPEN",
            "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1 is still listed as the next open item",
        )
    if "1. `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1`" in text:
        return fail(
            "E_ROADMAP_V2_GA2_FINAL_MALBLOCK_ROUNDTRIP_NEXT",
            "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_LA2_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_GA2_FINAL_LA2_FINAL_NEXT",
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
        return fail("E_ROADMAP_V2_GA2_FINAL_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_ROADMAP_V2_GA2_FINAL_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_doc,
        check_report,
        check_lang_core_2_surfaces,
        run_lang_core_2_wrapper,
        check_queue,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[roadmap-v2-ga2-final-closure-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
