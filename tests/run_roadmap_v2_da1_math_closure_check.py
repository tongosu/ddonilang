#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROADMAP_V2_DA1_MATH_CLOSURE_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        DOC,
        ROOT / "ROADMAP_V2_DA1_MATH_REBASE_V1.md",
        ROOT / "MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1.md",
        ROOT / "MATH_VECTOR_MINIMUM_FIRST_RUN_V1.md",
        QUEUE,
        ROOT / "pack" / "formula_relation_solve_v1" / "golden.jsonl",
        ROOT / "pack" / "relation_solve_system_2x2_v1" / "golden.jsonl",
        ROOT / "pack" / "relation_solve_ddn_bridge_v2" / "golden.jsonl",
        ROOT / "pack" / "relation_solve_wasm_cli_parity_v2" / "golden.jsonl",
        ROOT / "pack" / "math_numeric_int_v1" / "golden.jsonl",
        ROOT / "pack" / "math_numeric_diff_v1" / "golden.jsonl",
        ROOT / "pack" / "math_calculus_v1" / "golden.jsonl",
        ROOT / "pack" / "edu_seamgrim_rep_math_function_line_v1" / "README.md",
        ROOT / "tests" / "run_seamgrim_subject_representative_examples_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROADMAP_V2_DA1_MATH_CLOSURE_MISSING", str(missing))
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
            "no product code",
            "formula_relation_solve_v1",
            "relation_solve_system_2x2_v1",
            "relation_solve_ddn_bridge_v2",
            "relation_solve_wasm_cli_parity_v2",
            "math_numeric_int_v1",
            "math_numeric_diff_v1",
            "math_calculus_v1",
            "computed derivative/integral output",
            "edu_seamgrim_rep_math_function_line_v1",
            "vector-specific first-run evidence is still missing",
            "does not claim `다-1` complete",
            "MATH_VECTOR_MINIMUM_FIRST_RUN_V1",
            "ROOT_LOW_RISK_RETIRE_DELETE_V1",
            "docs/ssot/**",
        ],
        "E_ROADMAP_V2_DA1_MATH_CLOSURE_DOC",
    )


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "ROADMAP_V2_DA1_MATH_CLOSURE_V1",
        "closed by `ROADMAP_V2_DA1_MATH_CLOSURE_V1.md`",
        "vector-specific first-run evidence",
        "MATH_VECTOR_MINIMUM_FIRST_RUN_V1",
        "closed by `MATH_VECTOR_MINIMUM_FIRST_RUN_V1.md`",
        "ROADMAP_V2_DA1_FINAL_CLOSURE_V1",
        "closed by `ROADMAP_V2_DA1_FINAL_CLOSURE_V1.md`",
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
        "ROADMAP_V2 `다-1` remains open",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "Approval-gated",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_ROADMAP_V2_DA1_MATH_CLOSURE_QUEUE", str(missing))
    if "1. `ROADMAP_V2_DA1_MATH_CLOSURE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_DA1_MATH_CLOSURE_QUEUE_OPEN",
            "DA1 math closure is still listed as the next open item",
        )
    if "1. `MATH_VECTOR_MINIMUM_FIRST_RUN_V1`" in text:
        return fail(
            "E_ROADMAP_V2_DA1_MATH_CLOSURE_VECTOR_OPEN",
            "MATH_VECTOR_MINIMUM_FIRST_RUN_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_DA1_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_DA1_MATH_CLOSURE_FINAL_OPEN",
            "ROADMAP_V2_DA1_FINAL_CLOSURE_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_POST_PRIORITY_REBASE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_DA1_MATH_CLOSURE_POST_PRIORITY_OPEN",
            "ROADMAP_V2_POST_PRIORITY_REBASE_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_DA1_MATH_CLOSURE_GA2_OPEN",
            "ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1 is still listed as the next open item",
        )
    if "1. `LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1`" in text:
        return fail(
            "E_ROADMAP_V2_DA1_MATH_CLOSURE_LANG_CORE_2_OPEN",
            "LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_GA2_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_DA1_MATH_CLOSURE_GA2_FINAL_OPEN",
            "ROADMAP_V2_GA2_FINAL_CLOSURE_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_DA1_MATH_CLOSURE_LA2_OPEN",
            "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1 is still listed as the next open item",
        )
    if "1. `BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1`" in text:
        return fail(
            "E_ROADMAP_V2_DA1_MATH_CLOSURE_BLOCK_EDITOR_REFRESH_OPEN",
            "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1 is still listed as the next open item",
        )
    if "1. `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1`" in text:
        return fail(
            "E_ROADMAP_V2_DA1_MATH_CLOSURE_MALBLOCK_ROUNDTRIP_NEXT",
            "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_LA2_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_DA1_MATH_CLOSURE_LA2_FINAL_OPEN",
            "ROADMAP_V2_LA2_FINAL_CLOSURE_V1 is still listed as the next open item",
        )
    if "1. `ROOT_LOW_RISK_RETIRE_DELETE_V1`" not in text:
        return fail(
            "E_ROADMAP_V2_DA1_MATH_CLOSURE_A1_NEXT",
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
        return fail("E_ROADMAP_V2_DA1_MATH_CLOSURE_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_ROADMAP_V2_DA1_MATH_CLOSURE_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_doc,
        check_queue,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[roadmap-v2-da1-math-closure-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
