#!/usr/bin/env python
from __future__ import annotations

import json
import sys
from pathlib import Path


README_PATH = Path("tests/proof_operation_family/README.md")
AGE1_OPERATION_MATRIX_README = Path("tests/age1_immediate_proof_operation/README.md")
SOLVER_FAMILY_README = Path("tests/proof_solver_operation_family/README.md")
CASE_COMPLETION_PARITY_README = Path("tests/proof_case_analysis_completion_parity/README.md")
AGE1_COUNTER_README = Path("pack/age1_immediate_proof_smoke_v1/README.md")
AGE1_SOLVE_README = Path("pack/age1_immediate_proof_solver_search_solve_smoke_v1/README.md")
AGE1_MIXED_README = Path("pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1/README.md")
AGE4_MIXED_README = Path("pack/age4_proof_case_analysis_solver_open_search_replay_v1/README.md")
AGE4_CHECK_README = Path("pack/age4_proof_solver_open_replay_v1/README.md")
AGE4_SEARCH_README = Path("pack/age4_proof_solver_search_replay_v1/README.md")
AGE1_COUNTER_PROOF = Path("pack/age1_immediate_proof_smoke_v1/expected/proof.detjson")
AGE1_SOLVE_PROOF = Path("pack/age1_immediate_proof_solver_search_solve_smoke_v1/expected/proof.detjson")
AGE1_MIXED_PROOF = Path("pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1/expected/proof.detjson")
AGE4_MIXED_PROOF = Path("pack/age4_proof_case_analysis_solver_open_search_replay_v1/expected/proof.detjson")
AGE4_CHECK_PROOF = Path("pack/age4_proof_solver_open_replay_v1/expected/proof.detjson")
AGE4_SEARCH_PROOF = Path("pack/age4_proof_solver_search_replay_v1/expected/proof.detjson")
README_SNIPPETS = (
    "## Stable Contract",
    "`proof_check`",
    "`solver_check`",
    "`solver_search`",
    "`counterexample`",
    "`solve`",
    "`tests/age1_immediate_proof_operation/README.md`",
    "`pack/age1_immediate_proof_smoke_v1/expected/proof.detjson`",
    "`pack/age1_immediate_proof_solver_search_solve_smoke_v1/expected/proof.detjson`",
    "`pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1/expected/proof.detjson`",
    "`pack/age4_proof_case_analysis_solver_open_search_replay_v1/expected/proof.detjson`",
    "`pack/age4_proof_solver_open_replay_v1/expected/proof.detjson`",
    "`pack/age4_proof_solver_search_replay_v1/expected/proof.detjson`",
    "`tests/proof_solver_operation_family/README.md`",
    "`tests/proof_case_analysis_completion_parity/README.md`",
    "`tests/proof_family/README.md`",
    "`python tests/run_proof_operation_family_selftest.py`",
    "`python tests/run_proof_family_selftest.py`",
    "`proof_operation_family_selftest`",
)
POINTERS = (
    "`tests/proof_operation_family/README.md`",
    "`python tests/run_proof_operation_family_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-operation-family-selftest] fail: {message}")
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


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def ensure_exists_unique(doc: dict, label: str) -> None:
    translation = doc["solver_translation"]
    if translation["quantifier_count"] != 1:
        raise ValueError(f"{label}: quantifier_count != 1")
    quantifier = find_item(translation["items"], "quantifier")
    if quantifier is None or quantifier.get("quantifier") != "exists_unique":
        raise ValueError(f"{label}: missing exists_unique")


def validate_age1_counter() -> None:
    doc = load_json(AGE1_COUNTER_PROOF)
    ensure_exists_unique(doc, "AGE1 counter")
    if doc["solver_translation"]["proof_check_count"] != 1:
        raise ValueError("AGE1 counter: translation proof_check_count != 1")
    if doc["proof_runtime"]["proof_check_count"] != 1:
        raise ValueError("AGE1 counter: runtime proof_check_count != 1")
    if doc["proof_runtime"]["solver_check_count"] != 1:
        raise ValueError("AGE1 counter: runtime solver_check_count != 1")
    if doc["proof_runtime"]["solver_search_count"] != 1:
        raise ValueError("AGE1 counter: runtime solver_search_count != 1")
    if find_item(doc["proof_runtime"]["items"], "solver_search", "counterexample") is None:
        raise ValueError("AGE1 counter: missing runtime counterexample")


def validate_age1_solve() -> None:
    doc = load_json(AGE1_SOLVE_PROOF)
    ensure_exists_unique(doc, "AGE1 solve")
    if doc["solver_translation"]["proof_check_count"] != 1:
        raise ValueError("AGE1 solve: translation proof_check_count != 1")
    if doc["proof_runtime"]["proof_check_count"] != 1:
        raise ValueError("AGE1 solve: runtime proof_check_count != 1")
    if doc["proof_runtime"]["solver_check_count"] != 0:
        raise ValueError("AGE1 solve: runtime solver_check_count != 0")
    if doc["proof_runtime"]["solver_search_count"] != 1:
        raise ValueError("AGE1 solve: runtime solver_search_count != 1")
    if find_item(doc["proof_runtime"]["items"], "solver_search", "solve") is None:
        raise ValueError("AGE1 solve: missing runtime solve")


def validate_age1_mixed() -> None:
    doc = load_json(AGE1_MIXED_PROOF)
    ensure_exists_unique(doc, "AGE1 mixed")
    if doc["solver_translation"]["proof_check_count"] != 1:
        raise ValueError("AGE1 mixed: translation proof_check_count != 1")
    if doc["proof_runtime"]["proof_check_count"] != 1:
        raise ValueError("AGE1 mixed: runtime proof_check_count != 1")
    if doc["proof_runtime"]["solver_check_count"] != 1:
        raise ValueError("AGE1 mixed: runtime solver_check_count != 1")
    if doc["proof_runtime"]["solver_search_count"] != 2:
        raise ValueError("AGE1 mixed: runtime solver_search_count != 2")
    if find_item(doc["proof_runtime"]["items"], "solver_check") is None:
        raise ValueError("AGE1 mixed: missing runtime solver_check")
    for operation in ("counterexample", "solve"):
        if find_item(doc["proof_runtime"]["items"], "solver_search", operation) is None:
            raise ValueError(f"AGE1 mixed: missing runtime {operation}")


def validate_age4_check() -> None:
    doc = load_json(AGE4_CHECK_PROOF)
    ensure_exists_unique(doc, "AGE4 check")
    if doc["solver_translation"]["proof_check_count"] != 0:
        raise ValueError("AGE4 check: translation proof_check_count != 0")
    if doc["proof_runtime"]["proof_check_count"] != 0:
        raise ValueError("AGE4 check: runtime proof_check_count != 0")
    if doc["proof_runtime"]["solver_check_count"] != 1:
        raise ValueError("AGE4 check: runtime solver_check_count != 1")
    if doc["proof_runtime"]["solver_search_count"] != 0:
        raise ValueError("AGE4 check: runtime solver_search_count != 0")


def validate_age4_mixed() -> None:
    doc = load_json(AGE4_MIXED_PROOF)
    ensure_exists_unique(doc, "AGE4 mixed")
    if doc["solver_translation"]["proof_check_count"] != 0:
        raise ValueError("AGE4 mixed: translation proof_check_count != 0")
    if doc["proof_runtime"]["proof_check_count"] != 0:
        raise ValueError("AGE4 mixed: runtime proof_check_count != 0")
    if doc["proof_runtime"]["solver_check_count"] != 1:
        raise ValueError("AGE4 mixed: runtime solver_check_count != 1")
    if doc["proof_runtime"]["solver_search_count"] != 2:
        raise ValueError("AGE4 mixed: runtime solver_search_count != 2")
    if find_item(doc["proof_runtime"]["items"], "solver_check") is None:
        raise ValueError("AGE4 mixed: missing runtime solver_check")
    for operation in ("counterexample", "solve"):
        if find_item(doc["proof_runtime"]["items"], "solver_search", operation) is None:
            raise ValueError(f"AGE4 mixed: missing runtime {operation}")


def validate_age4_search() -> None:
    doc = load_json(AGE4_SEARCH_PROOF)
    ensure_exists_unique(doc, "AGE4 search")
    if doc["solver_translation"]["proof_check_count"] != 0:
        raise ValueError("AGE4 search: translation proof_check_count != 0")
    if doc["proof_runtime"]["proof_check_count"] != 0:
        raise ValueError("AGE4 search: runtime proof_check_count != 0")
    if doc["proof_runtime"]["solver_check_count"] != 0:
        raise ValueError("AGE4 search: runtime solver_check_count != 0")
    if doc["proof_runtime"]["solver_search_count"] != 2:
        raise ValueError("AGE4 search: runtime solver_search_count != 2")
    for operation in ("counterexample", "solve"):
        if find_item(doc["proof_runtime"]["items"], "solver_search", operation) is None:
            raise ValueError(f"AGE4 search: missing runtime {operation}")


def main() -> int:
    if not README_PATH.exists():
        return fail(f"missing readme: {README_PATH}")
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")
    try:
        for path in (
            AGE1_OPERATION_MATRIX_README,
            SOLVER_FAMILY_README,
            CASE_COMPLETION_PARITY_README,
            AGE1_COUNTER_README,
            AGE1_SOLVE_README,
            AGE1_MIXED_README,
            AGE4_MIXED_README,
            AGE4_CHECK_README,
            AGE4_SEARCH_README,
        ):
            ensure_pointers(path)
        validate_age1_counter()
        validate_age1_solve()
        validate_age1_mixed()
        validate_age4_mixed()
        validate_age4_check()
        validate_age4_search()
    except ValueError as exc:
        return fail(str(exc))
    print("[proof-operation-family-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
