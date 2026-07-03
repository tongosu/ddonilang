#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REBASE = ROOT / "ROADMAP_V2_DA1_MATH_REBASE_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        REBASE,
        QUEUE,
        ROOT / "ROADMAP_V2_FOLLOWON_REBASE_V1.md",
        ROOT / "MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1.md",
        ROOT / "pack" / "formula_relation_solve_v1" / "golden.jsonl",
        ROOT / "pack" / "relation_solve_system_2x2_v1" / "golden.jsonl",
        ROOT / "pack" / "relation_solve_ddn_bridge_v2" / "golden.jsonl",
        ROOT / "pack" / "relation_solve_wasm_cli_parity_v2" / "golden.jsonl",
        ROOT / "pack" / "math_numeric_int_v1" / "golden.jsonl",
        ROOT / "pack" / "math_numeric_diff_v1" / "golden.jsonl",
        ROOT / "pack" / "math_calculus_v1" / "input.ddn",
        ROOT / "pack" / "math_calculus_v1" / "golden.jsonl",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROADMAP_V2_DA1_MATH_REBASE_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_rebase_doc() -> int:
    return require_tokens(
        REBASE,
        [
            "documentation/checker-only",
            "no product code",
            "formula_relation_solve_v1",
            "relation_solve_system_2x2_v1",
            "math_numeric_int_v1",
            "math_numeric_diff_v1",
            "pack/math_calculus_v1/golden.jsonl` is stale",
            "current product output: `2*x`, `2`, `y = 2*x`, `1/4*x^4`, `1/4*x^4 + C`",
            "Do not close ROADMAP_V2 `다-1`",
            "MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1",
            "ROADMAP_V2_DA1_MATH_CLOSURE_V1",
            "ROOT_LOW_RISK_RETIRE_DELETE_V1",
            "docs/ssot/**",
        ],
        "E_ROADMAP_V2_DA1_MATH_REBASE_DOC",
    )


def check_calculus_computed_golden() -> int:
    input_text = read(ROOT / "pack" / "math_calculus_v1" / "input.ddn")
    for token in ["미분하기", "적분하기", "df1 보여주기", "if3c 보여주기"]:
        if token not in input_text:
            return fail("E_ROADMAP_V2_DA1_MATH_REBASE_CALCULUS_INPUT", token)

    golden_path = ROOT / "pack" / "math_calculus_v1" / "golden.jsonl"
    rows = [json.loads(line) for line in read(golden_path).splitlines() if line.strip()]
    if len(rows) != 1 or "stdout" not in rows[0]:
        return fail("E_ROADMAP_V2_DA1_MATH_REBASE_CALCULUS_GOLDEN_SHAPE", str(rows))
    stdout = rows[0]["stdout"]
    expected = ["2*x", "2", "y = 2*x", "1/4*x^4", "1/4*x^4 + C"]
    for token in expected:
        if not any(token in item for item in stdout):
            return fail("E_ROADMAP_V2_DA1_MATH_REBASE_CALCULUS_COMPUTED", token)
    if any("diff(" in item or "int(" in item for item in stdout):
        return fail("E_ROADMAP_V2_DA1_MATH_REBASE_CALCULUS_PASSTHROUGH", str(stdout))
    return 0


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "ROADMAP_V2_DA1_MATH_REBASE_V1",
        "closed by `ROADMAP_V2_DA1_MATH_REBASE_V1.md`",
        "MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1",
        "closed by `MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1.md`",
        "ROADMAP_V2_DA1_MATH_CLOSURE_V1",
        "closed by `ROADMAP_V2_DA1_MATH_CLOSURE_V1.md`",
        "MATH_VECTOR_MINIMUM_FIRST_RUN_V1",
        "math_calculus_v1",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "Approval-gated",
        "docs/ssot/**",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_ROADMAP_V2_DA1_MATH_REBASE_QUEUE", str(missing))
    if "1. `ROADMAP_V2_DA1_MATH_REBASE_V1`" in text:
        return fail(
            "E_ROADMAP_V2_DA1_MATH_REBASE_QUEUE_OPEN",
            "DA1 rebase is still listed as the next open item",
        )
    if "1. `MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1`" in text:
        return fail(
            "E_ROADMAP_V2_DA1_MATH_REBASE_REFRESH_OPEN",
            "math calculus refresh is still listed as the next open item",
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
        return fail("E_ROADMAP_V2_DA1_MATH_REBASE_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_ROADMAP_V2_DA1_MATH_REBASE_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_rebase_doc,
        check_calculus_computed_golden,
        check_queue,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[roadmap-v2-da1-math-rebase-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
