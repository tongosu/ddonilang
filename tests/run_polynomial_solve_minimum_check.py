#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def fail(message: str) -> None:
    print(f"polynomial_solve_minimum_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")


def require(path: str) -> Path:
    candidate = ROOT / path
    if not candidate.exists():
        fail(f"missing required path: {path}")
    return candidate


def require_contains(path: str, tokens: list[str]) -> None:
    text = read_text(ROOT / path)
    for token in tokens:
        if token not in text:
            fail(f"{path} missing token: {token}")


def require_docs_ssot_clean() -> None:
    result = subprocess.run(
        ["git", "status", "--short", "--", "docs/ssot"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        fail(f"git status docs/ssot failed: {result.stderr.strip()}")
    if result.stdout.strip():
        fail(f"docs/ssot changed:\n{result.stdout}")


def run_pack() -> None:
    result = subprocess.run(
        [sys.executable, "tests/run_pack_golden.py", "polynomial_solve_minimum_v1"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        fail(
            "run_pack_golden polynomial_solve_minimum_v1 failed\n"
            + result.stdout
            + result.stderr
        )


def main() -> None:
    require("POLYNOMIAL_SOLVE_MINIMUM_V1.md")
    require("NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md")
    require("pack/polynomial_solve_minimum_v1/contract.detjson")
    require("pack/polynomial_solve_minimum_v1/golden.jsonl")
    require("pack/formula_relation_solve_quadratic_v1/golden.jsonl")
    require("pack/numeric_root_finding_bisection_v1/golden.jsonl")

    require_contains(
        "POLYNOMIAL_SOLVE_MINIMUM_V1.md",
        [
            "POLYNOMIAL_SOLVE_MINIMUM_V1",
            "다항식.풀기",
            "방정식풀기",
            "not a generalized polynomial solver",
            "No multi-root list",
            "docs/ssot/**",
        ],
    )
    require_contains(
        "NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md",
        [
            "POLYNOMIAL_SOLVE_MINIMUM_V1",
            "pack/polynomial_solve_minimum_v1",
            "CONSTRAINT_SOLVE_REBASE_V1",
        ],
    )
    require_contains("lang/src/stdlib.rs", ["다항식.풀기", "polynomial_solve_minimum_v1"])
    require_contains("tools/teul-cli/src/runtime/eval.rs", ["다항식.풀기", "eval_polynomial_solve_result"])
    require_contains("tool/src/ddn_runtime.rs", ["다항식.풀기", "eval_polynomial_solve_result"])
    require_contains("docs/context/all/DEV_SUMMARY.md", ["POLYNOMIAL_SOLVE_MINIMUM_V1", "다항식.풀기"])
    require_contains("docs/status/LANG_STATUS.md", ["POLYNOMIAL_SOLVE_MINIMUM_V1", "다항식.풀기"])

    contract = json.loads(read_text(ROOT / "pack/polynomial_solve_minimum_v1/contract.detjson"))
    if contract.get("surface") != "다항식.풀기":
        fail("contract surface must be 다항식.풀기")
    if contract.get("solver_path") != "existing_relation_solver":
        fail("contract solver_path must be existing_relation_solver")

    golden_lines = [
        json.loads(line)
        for line in read_text(ROOT / "pack/polynomial_solve_minimum_v1/golden.jsonl").splitlines()
        if line.strip()
    ]
    if len(golden_lines) != 3:
        fail("golden must contain three cases")
    stdout = [case.get("stdout") for case in golden_lines]
    expected = [
        ['#성공(미지수="x", 값=2)'],
        ['#실패(사유="non_unique")'],
        ['#실패(사유="unsupported")'],
    ]
    if stdout != expected:
        fail(f"unexpected golden stdout: {stdout!r}")

    run_pack()
    require_docs_ssot_clean()
    print("polynomial_solve_minimum_check: ok")


if __name__ == "__main__":
    main()

