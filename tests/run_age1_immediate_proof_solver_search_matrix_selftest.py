#!/usr/bin/env python
from __future__ import annotations

import json
import sys
from pathlib import Path


README_PATH = Path("tests/age1_immediate_proof_solver_search/README.md")
COUNTER_PACK = Path("pack/age1_immediate_proof_smoke_v1")
SOLVE_PACK = Path("pack/age1_immediate_proof_solver_search_solve_smoke_v1")
CASE_COUNTER_PACK = Path("pack/age1_immediate_proof_case_analysis_solver_search_smoke_v1")
CASE_SOLVE_PACK = Path("pack/age1_immediate_proof_case_analysis_solver_search_solve_smoke_v1")
CASE_MIXED_PACK = Path("pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1")
CASE_ELSE_MIXED_PACK = Path("pack/age1_immediate_proof_case_analysis_else_solver_open_search_smoke_v1")
PACK_POINTERS = (
    "`tests/age1_immediate_proof_solver_search/README.md`",
    "`python tests/run_age1_immediate_proof_solver_search_matrix_selftest.py`",
)
README_SNIPPETS = (
    "## Stable Contract",
    "`pack/age1_immediate_proof_smoke_v1`",
    "`pack/age1_immediate_proof_solver_search_solve_smoke_v1`",
    "`pack/age1_immediate_proof_case_analysis_solver_search_smoke_v1`",
    "`pack/age1_immediate_proof_case_analysis_solver_search_solve_smoke_v1`",
    "`pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1`",
    "`pack/age1_immediate_proof_case_analysis_else_solver_open_search_smoke_v1`",
    "`counterexample`",
    "`solve`",
    "`exists_unique`",
    "`else`",
    "`age1_immediate_proof_solver_search_matrix_selftest`",
    "`python tests/run_ci_sanity_gate.py --profile core_lang`",
)


def fail(message: str) -> int:
    print(f"[age1-immediate-proof-solver-search-matrix-selftest] fail: {message}")
    return 1


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def find_item(items: list[dict], kind: str, operation: str | None = None) -> dict | None:
    for item in items:
        if item.get("kind") != kind:
            continue
        if operation is not None and item.get("operation") != operation:
            continue
        return item
    return None


def validate_pack(
    pack_dir: Path,
    *,
    expected_operations: set[str],
    expect_solver_check: bool,
    expect_case_analysis: bool,
    expected_completion: str | None,
) -> None:
    readme_path = pack_dir / "README.md"
    proof_path = pack_dir / "expected" / "proof.detjson"
    if not readme_path.exists():
        raise ValueError(f"missing readme: {readme_path}")
    if not proof_path.exists():
        raise ValueError(f"missing expected proof: {proof_path}")
    readme_text = readme_path.read_text(encoding="utf-8")
    for pointer in PACK_POINTERS:
        if pointer not in readme_text:
            raise ValueError(f"missing pointer in {readme_path}: {pointer}")

    doc = load_json(proof_path)
    translation = doc["solver_translation"]
    runtime = doc["proof_runtime"]
    if translation["proof_check_count"] != 1:
        raise ValueError(f"{proof_path}: proof_check_count != 1")
    actual_case_analysis = translation["case_analysis_count"] == 1
    if actual_case_analysis != expect_case_analysis:
        raise ValueError(
            f"{proof_path}: case_analysis_count mismatch expected={int(expect_case_analysis)} actual={translation['case_analysis_count']}"
        )
    case_item = find_item(translation["items"], "case_analysis")
    actual_completion = None if case_item is None else case_item.get("completion")
    if actual_completion != expected_completion:
        raise ValueError(
            f"{proof_path}: case_analysis completion mismatch expected={expected_completion} actual={actual_completion}"
        )
    if translation["quantifier_count"] != 1:
        raise ValueError(f"{proof_path}: quantifier_count != 1")
    quantifier = find_item(translation["items"], "quantifier")
    if quantifier is None or quantifier.get("quantifier") != "exists_unique":
        raise ValueError(f"{proof_path}: missing exists_unique quantifier")
    proof_check = find_item(translation["items"], "proof_check")
    if proof_check is None:
        raise ValueError(f"{proof_path}: missing proof_check item")
    for operation in expected_operations:
        solver_open = find_item(translation["items"], "solver_open", operation)
        if solver_open is None:
            raise ValueError(f"{proof_path}: missing solver_open operation={operation}")
    if runtime["proof_check_count"] != 1:
        raise ValueError(f"{proof_path}: runtime proof_check_count != 1")
    if runtime["solver_search_count"] != len(expected_operations):
        raise ValueError(
            f"{proof_path}: runtime solver_search_count mismatch expected={len(expected_operations)} actual={runtime['solver_search_count']}"
        )
    for operation in expected_operations:
        solver_search = find_item(runtime["items"], "solver_search", operation)
        if solver_search is None:
            raise ValueError(f"{proof_path}: missing runtime solver_search operation={operation}")
    actual_solver_check = runtime["solver_check_count"] == 1
    if actual_solver_check != expect_solver_check:
        raise ValueError(
            f"{proof_path}: solver_check_count mismatch expected={int(expect_solver_check)} actual={runtime['solver_check_count']}"
        )


def main() -> int:
    if not README_PATH.exists():
        return fail(f"missing readme: {README_PATH}")
    readme_text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in readme_text:
            return fail(f"missing snippet: {snippet}")
    try:
        validate_pack(
            COUNTER_PACK,
            expected_operations={"counterexample"},
            expect_solver_check=True,
            expect_case_analysis=False,
            expected_completion=None,
        )
        validate_pack(
            SOLVE_PACK,
            expected_operations={"solve"},
            expect_solver_check=False,
            expect_case_analysis=False,
            expected_completion=None,
        )
        validate_pack(
            CASE_COUNTER_PACK,
            expected_operations={"counterexample"},
            expect_solver_check=False,
            expect_case_analysis=True,
            expected_completion="exhaustive",
        )
        validate_pack(
            CASE_SOLVE_PACK,
            expected_operations={"solve"},
            expect_solver_check=False,
            expect_case_analysis=True,
            expected_completion="exhaustive",
        )
        validate_pack(
            CASE_MIXED_PACK,
            expected_operations={"counterexample", "solve"},
            expect_solver_check=True,
            expect_case_analysis=True,
            expected_completion="exhaustive",
        )
        validate_pack(
            CASE_ELSE_MIXED_PACK,
            expected_operations={"counterexample", "solve"},
            expect_solver_check=True,
            expect_case_analysis=True,
            expected_completion="else",
        )
    except ValueError as exc:
        return fail(str(exc))
    print("[age1-immediate-proof-solver-search-matrix-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
