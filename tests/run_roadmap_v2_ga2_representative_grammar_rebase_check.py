#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"
GUIDE = (
    ROOT
    / "docs"
    / "context"
    / "roadmap"
    / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
)
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
EVIDENCE = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        DOC,
        ROOT / "ROADMAP_V2_POST_PRIORITY_REBASE_V1.md",
        QUEUE,
        GUIDE,
        TRACKER,
        EVIDENCE,
        ROOT / "pack" / "lang_core_0_v1" / "contract.detjson",
        ROOT / "pack" / "lang_core_1_v1" / "fixtures" / "cases.detjson",
        ROOT / "tests" / "run_lang_core_0_check.py",
        ROOT / "tests" / "run_lang_core_1_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROADMAP_V2_GA2_REBASE_MISSING", str(missing))
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
            "ROADMAP_V2 `가-2`",
            "대표 문법 pack",
            "lang_core_0_v1",
            "lang_core_1_v1",
            "채비",
            "훅",
            "조건",
            "임자",
            "계약",
            "LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1",
            "pack/lang_core_2_v1",
            "product parser/runtime paths",
            "Do not add new grammar",
            "test-only lowering",
            "docs/ssot/**",
        ],
        "E_ROADMAP_V2_GA2_REBASE_DOC",
    )


def check_source_docs() -> int:
    checks = [
        (
            GUIDE,
            [
                "#### 가-2",
                "대표 문법 pack 닫힘",
                "채비/훅/조건/임자/계약",
                "lang_core_2_v1",
            ],
        ),
        (
            TRACKER,
            [
                "가-1",
                "lang_core_1_v1",
                "가-2",
                "대표 문법 pack",
            ],
        ),
        (
            EVIDENCE,
            [
                "가-0",
                "lang_core_0_v1",
                "가-1",
                "lang_core_1_v1",
            ],
        ),
    ]
    for path, tokens in checks:
        rc = require_tokens(path, tokens, "E_ROADMAP_V2_GA2_REBASE_SOURCE")
        if rc:
            return rc
    return 0


def check_existing_evidence() -> int:
    pack_names = {
        path.name
        for path in (ROOT / "pack").iterdir()
        if path.is_dir()
    }
    expected_support = {
        "lang_core_0_v1",
        "lang_core_1_v1",
        "lang_chaebi_scope_v1",
        "lang_flow_hook_interaction_v1",
        "lang_hook_when_edge_v1",
        "lang_hook_while_condition_v1",
        "diag_contract_mode_v1",
        "diag_contract_standard",
        "diag_contract_seulgi_hook",
    }
    missing_support = sorted(expected_support - pack_names)
    if missing_support:
        return fail("E_ROADMAP_V2_GA2_REBASE_SUPPORT_PACKS", str(missing_support))
    if "lang_core_2_v1" not in pack_names:
        return fail(
            "E_ROADMAP_V2_GA2_REBASE_LANG_CORE_2_MISSING",
            "lang_core_2_v1 should exist after LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1",
        )
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1",
        "closed by `ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1.md`",
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
        "ROADMAP_V2_LA2_FINAL_CLOSURE_V1",
        "ROADMAP_V2_A1_NURIGYM_REBASE_V1",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_ROADMAP_V2_GA2_REBASE_QUEUE", str(missing))
    if "1. `ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_GA2_REBASE_STILL_OPEN",
            "GA2 rebase is still listed as the next open item",
        )
    if "1. `LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1`" in text:
        return fail(
            "E_ROADMAP_V2_GA2_REBASE_LANG_CORE_2_OPEN",
            "LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_GA2_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_GA2_REBASE_FINAL_OPEN",
            "ROADMAP_V2_GA2_FINAL_CLOSURE_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_GA2_REBASE_LA2_OPEN",
            "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1 is still listed as the next open item",
        )
    if "1. `BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1`" in text:
        return fail(
            "E_ROADMAP_V2_GA2_REBASE_BLOCK_EDITOR_REFRESH_OPEN",
            "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1 is still listed as the next open item",
        )
    if "1. `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1`" in text:
        return fail(
            "E_ROADMAP_V2_GA2_REBASE_MALBLOCK_ROUNDTRIP_NEXT",
            "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_LA2_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_GA2_REBASE_LA2_FINAL_OPEN",
            "ROADMAP_V2_LA2_FINAL_CLOSURE_V1 is still listed as the next open item",
        )
    return 0


def run_lang_core_smokes() -> int:
    for command in [
        ["python", "tests/run_lang_core_0_check.py"],
        ["python", "tests/run_lang_core_1_check.py"],
    ]:
        result = subprocess.run(
            command,
            cwd=ROOT,
            text=True,
            encoding="utf-8",
            errors="replace",
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        if result.returncode != 0:
            return fail("E_ROADMAP_V2_GA2_REBASE_LANG_CORE", result.stdout.strip())
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
        return fail("E_ROADMAP_V2_GA2_REBASE_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_ROADMAP_V2_GA2_REBASE_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_doc,
        check_source_docs,
        check_existing_evidence,
        check_queue,
        run_lang_core_smokes,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[roadmap-v2-ga2-representative-grammar-rebase-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
