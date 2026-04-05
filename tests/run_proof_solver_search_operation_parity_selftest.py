#!/usr/bin/env python
from __future__ import annotations

import json
import sys
from pathlib import Path


README_PATH = Path("tests/proof_solver_search_operation_parity/README.md")
AGE1_MATRIX_README = Path("tests/age1_immediate_proof_solver_search/README.md")
AGE1_COUNTER_PROOF = Path("pack/age1_immediate_proof_smoke_v1/expected/proof.detjson")
AGE1_SOLVE_PROOF = Path("pack/age1_immediate_proof_solver_search_solve_smoke_v1/expected/proof.detjson")
AGE1_MIXED_PROOF = Path("pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1/expected/proof.detjson")
AGE4_REPLAY_README = Path("pack/age4_proof_solver_search_replay_v1/README.md")
AGE4_REPLAY_PROOF = Path("pack/age4_proof_solver_search_replay_v1/expected/proof.detjson")
REQUIRED_README_SNIPPETS = (
    "## Stable Contract",
    "`tests/age1_immediate_proof_solver_search/README.md`",
    "`pack/age1_immediate_proof_smoke_v1/expected/proof.detjson`",
    "`pack/age1_immediate_proof_solver_search_solve_smoke_v1/expected/proof.detjson`",
    "`pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1/expected/proof.detjson`",
    "`pack/age4_proof_solver_search_replay_v1/expected/proof.detjson`",
    "`python tests/run_proof_solver_search_operation_parity_selftest.py`",
    "`proof_solver_search_operation_parity_selftest`",
    "`counterexample`",
    "`solve`",
    "`exists_unique`",
)
POINTERS = (
    "`tests/proof_solver_search_operation_parity/README.md`",
    "`python tests/run_proof_solver_search_operation_parity_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-solver-search-operation-parity-selftest] fail: {message}")
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


def validate_age1() -> None:
    counter = load_json(AGE1_COUNTER_PROOF)
    solve = load_json(AGE1_SOLVE_PROOF)
    for name, doc, operation, solver_check_count in (
        ("counter", counter, "counterexample", 1),
        ("solve", solve, "solve", 0),
    ):
        translation = doc["solver_translation"]
        runtime = doc["proof_runtime"]
        if translation["quantifier_count"] != 1:
            raise ValueError(f"AGE1 {name}: quantifier_count != 1")
        quantifier = find_item(translation["items"], "quantifier")
        if quantifier is None or quantifier.get("quantifier") != "exists_unique":
            raise ValueError(f"AGE1 {name}: missing exists_unique")
        if translation["proof_check_count"] != 1:
            raise ValueError(f"AGE1 {name}: proof_check_count != 1")
        if runtime["proof_check_count"] != 1:
            raise ValueError(f"AGE1 {name}: runtime proof_check_count != 1")
        if runtime["solver_search_count"] != 1:
            raise ValueError(f"AGE1 {name}: runtime solver_search_count != 1")
        if runtime["solver_check_count"] != solver_check_count:
            raise ValueError(
                f"AGE1 {name}: solver_check_count mismatch expected={solver_check_count} actual={runtime['solver_check_count']}"
            )
        if find_item(runtime["items"], "solver_search", operation) is None:
            raise ValueError(f"AGE1 {name}: missing runtime solver_search operation={operation}")
        if find_item(translation["items"], "solver_open", operation) is None:
            raise ValueError(f"AGE1 {name}: missing translation solver_open operation={operation}")
    mixed = load_json(AGE1_MIXED_PROOF)
    translation = mixed["solver_translation"]
    runtime = mixed["proof_runtime"]
    if translation["quantifier_count"] != 1:
        raise ValueError("AGE1 mixed: quantifier_count != 1")
    quantifier = find_item(translation["items"], "quantifier")
    if quantifier is None or quantifier.get("quantifier") != "exists_unique":
        raise ValueError("AGE1 mixed: missing exists_unique")
    if translation["proof_check_count"] != 1:
        raise ValueError("AGE1 mixed: proof_check_count != 1")
    if runtime["proof_check_count"] != 1:
        raise ValueError("AGE1 mixed: runtime proof_check_count != 1")
    if runtime["solver_check_count"] != 1:
        raise ValueError("AGE1 mixed: runtime solver_check_count != 1")
    if runtime["solver_search_count"] != 2:
        raise ValueError("AGE1 mixed: runtime solver_search_count != 2")
    for operation in ("counterexample", "solve"):
        if find_item(runtime["items"], "solver_search", operation) is None:
            raise ValueError(f"AGE1 mixed: missing runtime solver_search operation={operation}")
        if find_item(translation["items"], "solver_open", operation) is None:
            raise ValueError(f"AGE1 mixed: missing translation solver_open operation={operation}")


def validate_age4() -> None:
    doc = load_json(AGE4_REPLAY_PROOF)
    translation = doc["solver_translation"]
    runtime = doc["proof_runtime"]
    if translation["quantifier_count"] != 1:
        raise ValueError("AGE4 replay: quantifier_count != 1")
    quantifier = find_item(translation["items"], "quantifier")
    if quantifier is None or quantifier.get("quantifier") != "exists_unique":
        raise ValueError("AGE4 replay: missing exists_unique")
    if translation["solver_open_count"] != 2:
        raise ValueError("AGE4 replay: solver_open_count != 2")
    if runtime["solver_search_count"] != 2:
        raise ValueError("AGE4 replay: runtime solver_search_count != 2")
    if runtime["proof_check_count"] != 0:
        raise ValueError("AGE4 replay: runtime proof_check_count != 0")
    if runtime["solver_check_count"] != 0:
        raise ValueError("AGE4 replay: runtime solver_check_count != 0")
    for operation in ("counterexample", "solve"):
        if find_item(translation["items"], "solver_open", operation) is None:
            raise ValueError(f"AGE4 replay: missing translation solver_open operation={operation}")
        if find_item(runtime["items"], "solver_search", operation) is None:
            raise ValueError(f"AGE4 replay: missing runtime solver_search operation={operation}")


def ensure_pointer(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def main() -> int:
    if not README_PATH.exists():
        return fail(f"missing readme: {README_PATH}")
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in REQUIRED_README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")
    try:
        ensure_pointer(AGE1_MATRIX_README)
        ensure_pointer(AGE4_REPLAY_README)
        validate_age1()
        validate_age4()
    except ValueError as exc:
        return fail(str(exc))
    print("[proof-solver-search-operation-parity-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
