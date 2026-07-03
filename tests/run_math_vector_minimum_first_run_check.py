#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "MATH_VECTOR_MINIMUM_FIRST_RUN_V1.md"
PACK = ROOT / "pack" / "math_vector_minimum_first_run_v1"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"

EXPECTED_STDOUT = [
    "차림[3, 4]",
    "차림[1, -2]",
    "차림[4, 2]",
    "차림[2, 6]",
    "차림[6, 8]",
    "-5",
    "25",
]


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        DOC,
        ROOT / "ROADMAP_V2_DA1_MATH_CLOSURE_V1.md",
        QUEUE,
        PACK / "README.md",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_MATH_VECTOR_MINIMUM_FIRST_RUN_MISSING", str(missing))
    return 0


def check_doc() -> int:
    text = read(DOC)
    required = [
        "minimum product-path evidence",
        "numeric `차림`",
        "component-wise addition",
        "component-wise subtraction",
        "scalar multiplication",
        "dot product",
        "squared length",
        "no new stdlib surface",
        "ROADMAP_V2_DA1_FINAL_CLOSURE_V1",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_MATH_VECTOR_MINIMUM_FIRST_RUN_DOC", str(missing))
    return 0


def check_input_surface() -> int:
    text = read(PACK / "input.ddn")
    required = [
        "(3, 4) 차림",
        "(1, -2) 차림",
        "차림.값",
        "ux + vx",
        "ux - vx",
        "2 * ux",
        "ux * vx + uy * vy",
        "ux * ux + uy * uy",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_MATH_VECTOR_MINIMUM_FIRST_RUN_INPUT", str(missing))
    forbidden = ["벡터.", "vector.", "텐서."]
    present = [token for token in forbidden if token in text]
    if present:
        return fail("E_MATH_VECTOR_MINIMUM_FIRST_RUN_FORBIDDEN_SURFACE", str(present))
    return 0


def check_golden() -> int:
    rows = [json.loads(line) for line in read(PACK / "golden.jsonl").splitlines() if line.strip()]
    if len(rows) != 1:
        return fail("E_MATH_VECTOR_MINIMUM_FIRST_RUN_GOLDEN_ROWS", str(len(rows)))
    row = rows[0]
    if row.get("stdout") != EXPECTED_STDOUT:
        return fail("E_MATH_VECTOR_MINIMUM_FIRST_RUN_STDOUT", str(row.get("stdout")))
    if row.get("exit_code") != 0:
        return fail("E_MATH_VECTOR_MINIMUM_FIRST_RUN_EXIT", str(row.get("exit_code")))
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
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
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "approval-gated",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_MATH_VECTOR_MINIMUM_FIRST_RUN_QUEUE", str(missing))
    if "1. `MATH_VECTOR_MINIMUM_FIRST_RUN_V1`" in text:
        return fail(
            "E_MATH_VECTOR_MINIMUM_FIRST_RUN_QUEUE_OPEN",
            "vector first-run is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_DA1_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_MATH_VECTOR_MINIMUM_FIRST_RUN_FINAL_OPEN",
            "ROADMAP_V2_DA1_FINAL_CLOSURE_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_POST_PRIORITY_REBASE_V1`" in text:
        return fail(
            "E_MATH_VECTOR_MINIMUM_FIRST_RUN_POST_PRIORITY_OPEN",
            "ROADMAP_V2_POST_PRIORITY_REBASE_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1`" in text:
        return fail(
            "E_MATH_VECTOR_MINIMUM_FIRST_RUN_GA2_OPEN",
            "ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1 is still listed as the next open item",
        )
    if "1. `LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1`" in text:
        return fail(
            "E_MATH_VECTOR_MINIMUM_FIRST_RUN_LANG_CORE_2_OPEN",
            "LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_GA2_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_MATH_VECTOR_MINIMUM_FIRST_RUN_GA2_FINAL_OPEN",
            "ROADMAP_V2_GA2_FINAL_CLOSURE_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1`" in text:
        return fail(
            "E_MATH_VECTOR_MINIMUM_FIRST_RUN_LA2_OPEN",
            "ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1 is still listed as the next open item",
        )
    if "1. `BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1`" in text:
        return fail(
            "E_MATH_VECTOR_MINIMUM_FIRST_RUN_BLOCK_EDITOR_REFRESH_OPEN",
            "BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1 is still listed as the next open item",
        )
    if "1. `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1`" in text:
        return fail(
            "E_MATH_VECTOR_MINIMUM_FIRST_RUN_MALBLOCK_ROUNDTRIP_NEXT",
            "SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_V1 is still listed as the next open item",
        )
    if "1. `ROADMAP_V2_LA2_FINAL_CLOSURE_V1`" in text:
        return fail(
            "E_MATH_VECTOR_MINIMUM_FIRST_RUN_LA2_FINAL_OPEN",
            "ROADMAP_V2_LA2_FINAL_CLOSURE_V1 is still listed as the next open item",
        )
    if "1. `ROOT_LOW_RISK_RETIRE_DELETE_V1`" not in text and "No automatic next development item is selected." not in text:
        return fail(
            "E_MATH_VECTOR_MINIMUM_FIRST_RUN_A1_NEXT",
            "ROOT_LOW_RISK_RETIRE_DELETE_V1 is not the next open item and queue is not closed",
        )
    return 0


def run_pack_golden() -> int:
    result = subprocess.run(
        ["python", "tests/run_pack_golden.py", "math_vector_minimum_first_run_v1"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode != 0:
        return fail("E_MATH_VECTOR_MINIMUM_FIRST_RUN_PACK", result.stdout.strip())
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
        return fail("E_MATH_VECTOR_MINIMUM_FIRST_RUN_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_MATH_VECTOR_MINIMUM_FIRST_RUN_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_doc,
        check_input_surface,
        check_golden,
        check_queue,
        run_pack_golden,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[math-vector-minimum-first-run-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
