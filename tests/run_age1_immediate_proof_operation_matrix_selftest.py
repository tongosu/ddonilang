#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("tests/age1_immediate_proof_operation/README.md")
PACK_POINTERS = (
    "`tests/age1_immediate_proof_operation/README.md`",
    "`python tests/run_age1_immediate_proof_operation_matrix_selftest.py`",
)
README_SNIPPETS = (
    "## Stable Contract",
    "`pack/age1_immediate_proof_smoke_v1`",
    "`pack/age1_immediate_proof_solver_search_solve_smoke_v1`",
    "`pack/age1_immediate_proof_case_analysis_smoke_v1`",
    "`pack/age1_immediate_proof_case_analysis_solver_open_smoke_v1`",
    "`pack/age1_immediate_proof_case_analysis_solver_search_smoke_v1`",
    "`pack/age1_immediate_proof_case_analysis_solver_search_solve_smoke_v1`",
    "`pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1`",
    "`pack/age1_immediate_proof_case_analysis_else_solver_open_search_smoke_v1`",
    "`check`",
    "`counterexample`",
    "`solve`",
    "`exists_unique`",
    "`exhaustive`",
    "`else`",
    "`python tests/run_ci_sanity_gate.py --profile core_lang`",
)

PACKS = (
    (
        Path("pack/age1_immediate_proof_smoke_v1"),
        {"check"},
        {"counterexample"},
        False,
        None,
    ),
    (
        Path("pack/age1_immediate_proof_solver_search_solve_smoke_v1"),
        set(),
        {"solve"},
        False,
        None,
    ),
    (
        Path("pack/age1_immediate_proof_case_analysis_smoke_v1"),
        set(),
        set(),
        True,
        "exhaustive",
    ),
    (
        Path("pack/age1_immediate_proof_case_analysis_solver_open_smoke_v1"),
        {"check"},
        set(),
        True,
        "exhaustive",
    ),
    (
        Path("pack/age1_immediate_proof_case_analysis_solver_search_smoke_v1"),
        set(),
        {"counterexample"},
        True,
        "exhaustive",
    ),
    (
        Path("pack/age1_immediate_proof_case_analysis_solver_search_solve_smoke_v1"),
        set(),
        {"solve"},
        True,
        "exhaustive",
    ),
    (
        Path("pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1"),
        {"check"},
        {"counterexample", "solve"},
        True,
        "exhaustive",
    ),
    (
        Path("pack/age1_immediate_proof_case_analysis_else_solver_open_search_smoke_v1"),
        {"check"},
        {"counterexample", "solve"},
        True,
        "else",
    ),
)


def fail(message: str) -> int:
    print(f"[age1-immediate-proof-operation-matrix-selftest] fail: {message}")
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
    expected_checks: set[str],
    expected_searches: set[str],
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
    if runtime["proof_check_count"] != 1:
        raise ValueError(f"{proof_path}: runtime proof_check_count != 1")
    if translation["quantifier_count"] != 1:
        raise ValueError(f"{proof_path}: quantifier_count != 1")
    quantifier = find_item(translation["items"], "quantifier")
    if quantifier is None or quantifier.get("quantifier") != "exists_unique":
        raise ValueError(f"{proof_path}: missing exists_unique quantifier")

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

    for op in expected_checks | expected_searches:
        if find_item(translation["items"], "solver_open", op) is None:
            raise ValueError(f"{proof_path}: missing translation solver_open operation={op}")

    actual_check_ops = {
        item.get("operation")
        for item in runtime["items"]
        if item.get("kind") == "solver_check"
    }
    if expected_checks:
        if runtime["solver_check_count"] != len(expected_checks):
            raise ValueError(
                f"{proof_path}: runtime solver_check_count mismatch expected={len(expected_checks)} actual={runtime['solver_check_count']}"
            )
        if actual_check_ops != set():
            # runtime solver_check items do not carry operation; count check is enough.
            pass
    elif runtime["solver_check_count"] != 0:
        raise ValueError(f"{proof_path}: runtime solver_check_count != 0")

    actual_search_ops = {
        item.get("operation")
        for item in runtime["items"]
        if item.get("kind") == "solver_search"
    }
    if runtime["solver_search_count"] != len(expected_searches):
        raise ValueError(
            f"{proof_path}: runtime solver_search_count mismatch expected={len(expected_searches)} actual={runtime['solver_search_count']}"
        )
    if actual_search_ops != expected_searches:
        raise ValueError(
            f"{proof_path}: runtime solver_search operations mismatch expected={sorted(expected_searches)} actual={sorted(actual_search_ops)}"
        )


def main() -> int:
    if not README_PATH.exists():
        return fail(f"missing readme: {README_PATH}")
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")
    try:
        for pack_dir, checks, searches, case_analysis, completion in PACKS:
            validate_pack(
                pack_dir,
                expected_checks=checks,
                expected_searches=searches,
                expect_case_analysis=case_analysis,
                expected_completion=completion,
            )
    except ValueError as exc:
        return fail(str(exc))
    print("[age1-immediate-proof-operation-matrix-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
