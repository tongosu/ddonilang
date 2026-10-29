#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def fail(message: str) -> None:
    print(f"linear_inequality_solve_minimum_check: FAIL: {message}", file=sys.stderr)
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
        [sys.executable, "tests/run_pack_golden.py", "linear_inequality_solve_minimum_v1"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        fail(
            "run_pack_golden linear_inequality_solve_minimum_v1 failed\n"
            + result.stdout
            + result.stderr
        )


def run_unsupported_case() -> None:
    result = subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            "run",
            "pack/linear_inequality_solve_minimum_v1/input_unsupported.ddn",
        ],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
        check=False,
    )
    combined = (result.stdout or "") + (result.stderr or "")
    if result.returncode == 0:
        fail("nonlinear unsupported case unexpectedly succeeded")
    if "E_LINEAR_INEQUALITY_NONLINEAR" not in combined:
        fail("unsupported case did not emit E_LINEAR_INEQUALITY_NONLINEAR")


def main() -> None:
    require("LINEAR_INEQUALITY_SOLVE_MINIMUM_V1.md")
    require("CONSTRAINT_SOLVE_REBASE_V1.md")
    require("NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md")
    require("pack/linear_inequality_solve_minimum_v1/contract.detjson")
    require("pack/linear_inequality_solve_minimum_v1/golden.jsonl")
    require("pack/constraint_solve_rebase_v1/golden.jsonl")
    require("pack/polynomial_solve_minimum_v1/golden.jsonl")

    require_contains(
        "LINEAR_INEQUALITY_SOLVE_MINIMUM_V1.md",
        [
            "LINEAR_INEQUALITY_SOLVE_MINIMUM_V1",
            "선형부등식.풀기",
            "one-variable linear",
            "linear_inequality_solution",
            "이하",
            "이상",
            "미만",
            "초과",
            "방정식풀기",
            "No multi-variable linear programming",
            "docs/ssot/**",
        ],
    )
    require_contains(
        "NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md",
        [
            "LINEAR_INEQUALITY_SOLVE_MINIMUM_V1",
            "pack/linear_inequality_solve_minimum_v1",
            "STUDIO_NUMERIC_CURRICULUM_TRACK_V1",
        ],
    )
    require_contains("lang/src/stdlib.rs", ["선형부등식.풀기", "linear_inequality_solve_minimum_v1"])
    require_contains("tools/teul-cli/src/runtime/eval.rs", ["선형부등식.풀기", "eval_linear_inequality_solve"])
    require_contains("tool/src/ddn_runtime.rs", ["선형부등식.풀기", "eval_linear_inequality_solve"])
    require_contains("docs/context/all/DEV_SUMMARY.md", ["LINEAR_INEQUALITY_SOLVE_MINIMUM_V1", "선형부등식.풀기"])
    require_contains("docs/status/LANG_STATUS.md", ["LINEAR_INEQUALITY_SOLVE_MINIMUM_V1", "선형부등식.풀기"])

    contract = json.loads(read_text(ROOT / "pack/linear_inequality_solve_minimum_v1/contract.detjson"))
    if contract.get("surface") != "선형부등식.풀기":
        fail("contract surface must be 선형부등식.풀기")
    if contract.get("scope") != "one_variable_linear_interval":
        fail("contract scope must be one_variable_linear_interval")
    if contract.get("overloads_bangjeongsikpulgi") is not False:
        fail("contract must not overload 방정식풀기")
    if contract.get("solver_internal_inequality_claim") is not False:
        fail("contract must not claim solver-internal inequality support")
    if contract.get("next_recommended_item") != "STUDIO_NUMERIC_CURRICULUM_TRACK_V1":
        fail("next recommended item must be STUDIO_NUMERIC_CURRICULUM_TRACK_V1")

    golden_lines = [
        json.loads(line)
        for line in read_text(ROOT / "pack/linear_inequality_solve_minimum_v1/golden.jsonl").splitlines()
        if line.strip()
    ]
    if len(golden_lines) != 3:
        fail("golden must contain three supported cases")
    expected_stdout = [
        ["구간", "3", "참", "4", "참"],
        ["공집합"],
        ["구간", "-3", "참"],
    ]
    stdout = [case.get("stdout") for case in golden_lines]
    if stdout != expected_stdout:
        fail(f"unexpected golden stdout: {stdout!r}")

    run_pack()
    run_unsupported_case()
    require_docs_ssot_clean()
    print("linear_inequality_solve_minimum_check: ok")


if __name__ == "__main__":
    main()
