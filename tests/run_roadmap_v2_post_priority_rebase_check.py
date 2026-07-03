#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROADMAP_V2_POST_PRIORITY_REBASE_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"
TRACKER = ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_TRACKER.md"
MATRIX = ROOT / "docs" / "context" / "roadmap" / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md"
GUIDE = (
    ROOT
    / "docs"
    / "context"
    / "roadmap"
    / "ROADMAP_V2_GANADA_MILESTONE_MATRIX_CODEX_GUIDE_20260426.md"
)


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        DOC,
        ROOT / "ROADMAP_V2_DA1_FINAL_CLOSURE_V1.md",
        ROOT / "ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1.md",
        QUEUE,
        TRACKER,
        ROOT / "docs" / "status" / "roadmap_v2" / "ROADMAP_V2_EVIDENCE_MANIFEST.md",
        MATRIX,
        GUIDE,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROADMAP_V2_POST_PRIORITY_REBASE_MISSING", str(missing))
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
            "post-priority",
            "ROADMAP_V2_DA1_FINAL_CLOSURE_V1",
            "priority list",
            "가-2",
            "대표 문법 pack",
            "ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1",
            "Do not start `거-1+`",
            "public registry final",
            "matrix/tensor",
            "ROOT_LOW_RISK_RETIRE_DELETE_V1",
            "docs/ssot/**",
        ],
        "E_ROADMAP_V2_POST_PRIORITY_REBASE_DOC",
    )


def check_source_docs() -> int:
    for path, tokens in [
        (
            TRACKER,
            [
                "가-2",
                "대표 문법 pack",
                "거-1+",
                "public registry final",
            ],
        ),
        (
            MATRIX,
            [
                "가-2 대표 문법 pack",
                "라-2",
                "아-1",
                "아-2",
                "거-1",
            ],
        ),
        (
            GUIDE,
            [
                "#### 가-2",
                "대표 문법 pack 닫힘",
                "라-2 | 가-2",
                "아-1 | 가-2",
                "아-2 | 가-2",
            ],
        ),
    ]:
        rc = require_tokens(path, tokens, "E_ROADMAP_V2_POST_PRIORITY_REBASE_SOURCE")
        if rc:
            return rc
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "ROADMAP_V2_POST_PRIORITY_REBASE_V1",
        "closed by `ROADMAP_V2_POST_PRIORITY_REBASE_V1.md`",
        "ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1",
        "closed by `ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1.md`",
        "LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1",
        "closed by `LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1.md`",
        "ROADMAP_V2_GA2_FINAL_CLOSURE_V1",
        "closed by `ROADMAP_V2_GA2_FINAL_CLOSURE_V1.md`",
        "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1",
        "closed by `ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1.md`",
        "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1",
        "closed by `BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1.md`",
        "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1",
        "ROADMAP_V2_LA2_FINAL_CLOSURE_V1",
        "ROADMAP_V2_A1_NURIGYM_REBASE_V1",
        "가-2",
        "대표 문법 pack",
        "do not start `거-1+`",
        "public registry final",
        "matrix/tensor",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "Approval-gated",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_ROADMAP_V2_POST_PRIORITY_REBASE_QUEUE", str(missing))
    if "1. `ROADMAP_V2_POST_PRIORITY_REBASE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_POST_PRIORITY_REBASE_STILL_OPEN",
            "post-priority rebase is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_POST_PRIORITY_REBASE_GA2_OPEN",
            "GA2 representative grammar rebase is still listed as the next open item",
        )
    if "1. `LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1`" in text:
        return fail(
            "E_ROADMAP_V2_POST_PRIORITY_REBASE_LANG_CORE_2_OPEN",
            "LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_GA2_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_POST_PRIORITY_REBASE_GA2_FINAL_OPEN",
            "ROADMAP_V2_GA2_FINAL_CLOSURE_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_POST_PRIORITY_REBASE_LA2_OPEN",
            "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1 is still listed as the next open item",
        )
    if "1. `BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1`" in text:
        return fail(
            "E_ROADMAP_V2_POST_PRIORITY_REBASE_BLOCK_EDITOR_REFRESH_OPEN",
            "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1 is still listed as the next open item",
        )
    if "1. `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1`" in text:
        return fail(
            "E_ROADMAP_V2_POST_PRIORITY_REBASE_MALBLOCK_ROUNDTRIP_NEXT",
            "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_LA2_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_POST_PRIORITY_REBASE_LA2_FINAL_OPEN",
            "ROADMAP_V2_LA2_FINAL_CLOSURE_V1 is still listed as the next open item",
        )
    if "1. `ROOT_LOW_RISK_RETIRE_DELETE_V1`" not in text:
        return fail(
            "E_ROADMAP_V2_POST_PRIORITY_REBASE_A1_NEXT",
            "ROOT_LOW_RISK_RETIRE_DELETE_V1 is not the next open item",
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
        return fail("E_ROADMAP_V2_POST_PRIORITY_REBASE_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_ROADMAP_V2_POST_PRIORITY_REBASE_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_doc,
        check_source_docs,
        check_queue,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[roadmap-v2-post-priority-rebase-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
