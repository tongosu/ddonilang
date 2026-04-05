#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("tests/proof_case_analysis_completion_parity/README.md")
AGE1_MATRIX_README = Path("tests/age1_immediate_proof_operation/README.md")
AGE4_MATRIX_README = Path("tests/age4_proof_quantifier_case_analysis/README.md")
AGE1_EXHAUSTIVE_README = Path("pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1/README.md")
AGE1_ELSE_README = Path("pack/age1_immediate_proof_case_analysis_else_solver_open_search_smoke_v1/README.md")
AGE4_EXHAUSTIVE_README = Path("pack/age4_proof_case_analysis_solver_open_search_replay_v1/README.md")
AGE4_ELSE_README = Path("pack/age4_proof_case_analysis_else_solver_open_search_replay_v1/README.md")
AGE1_EXHAUSTIVE_PROOF = Path("pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1/expected/proof.detjson")
AGE1_ELSE_PROOF = Path("pack/age1_immediate_proof_case_analysis_else_solver_open_search_smoke_v1/expected/proof.detjson")
AGE4_EXHAUSTIVE_PROOF = Path("pack/age4_proof_case_analysis_solver_open_search_replay_v1/expected/proof.detjson")
AGE4_ELSE_PROOF = Path("pack/age4_proof_case_analysis_else_solver_open_search_replay_v1/expected/proof.detjson")
README_SNIPPETS = (
    "## Stable Contract",
    "`exhaustive`",
    "`else`",
    "`tests/age1_immediate_proof_operation/README.md`",
    "`tests/age4_proof_quantifier_case_analysis/README.md`",
    "`pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1/expected/proof.detjson`",
    "`pack/age1_immediate_proof_case_analysis_else_solver_open_search_smoke_v1/expected/proof.detjson`",
    "`pack/age4_proof_case_analysis_solver_open_search_replay_v1/expected/proof.detjson`",
    "`pack/age4_proof_case_analysis_else_solver_open_search_replay_v1/expected/proof.detjson`",
    "`python tests/run_proof_case_analysis_completion_parity_selftest.py`",
    "`proof_case_analysis_completion_parity_selftest`",
)
POINTERS = (
    "`tests/proof_case_analysis_completion_parity/README.md`",
    "`python tests/run_proof_case_analysis_completion_parity_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-case-analysis-completion-parity-selftest] fail: {message}")
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


def validate_pack(path: Path, *, completion: str, expect_proof_check: int, label: str) -> None:
    doc = load_json(path)
    translation = doc["solver_translation"]
    runtime = doc["proof_runtime"]
    if translation["quantifier_count"] != 1:
        raise ValueError(f"{label}: quantifier_count != 1")
    quantifier = find_item(translation["items"], "quantifier")
    if quantifier is None or quantifier.get("quantifier") != "exists_unique":
        raise ValueError(f"{label}: missing exists_unique quantifier")
    if translation["case_analysis_count"] != 1:
        raise ValueError(f"{label}: case_analysis_count != 1")
    case_item = find_item(translation["items"], "case_analysis")
    if case_item is None or case_item.get("completion") != completion:
        raise ValueError(f"{label}: completion mismatch")
    if translation["solver_open_count"] != 3:
        raise ValueError(f"{label}: solver_open_count != 3")
    if translation["proof_check_count"] != expect_proof_check:
        raise ValueError(f"{label}: translation proof_check_count mismatch")
    if runtime["proof_check_count"] != expect_proof_check:
        raise ValueError(f"{label}: runtime proof_check_count mismatch")
    if runtime["solver_check_count"] != 1:
        raise ValueError(f"{label}: runtime solver_check_count != 1")
    if runtime["solver_search_count"] != 2:
        raise ValueError(f"{label}: runtime solver_search_count != 2")
    if find_item(runtime["items"], "solver_check") is None:
        raise ValueError(f"{label}: missing runtime solver_check")
    for operation in ("check", "counterexample", "solve"):
        if find_item(translation["items"], "solver_open", operation) is None:
            raise ValueError(f"{label}: missing translation solver_open {operation}")
    for operation in ("counterexample", "solve"):
        if find_item(runtime["items"], "solver_search", operation) is None:
            raise ValueError(f"{label}: missing runtime solver_search {operation}")


def main() -> int:
    if not README_PATH.exists():
        return fail(f"missing readme: {README_PATH}")
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")
    try:
        for path in (
            AGE1_MATRIX_README,
            AGE4_MATRIX_README,
            AGE1_EXHAUSTIVE_README,
            AGE1_ELSE_README,
            AGE4_EXHAUSTIVE_README,
            AGE4_ELSE_README,
        ):
            ensure_pointers(path)
        validate_pack(
            AGE1_EXHAUSTIVE_PROOF,
            completion="exhaustive",
            expect_proof_check=1,
            label="AGE1 exhaustive",
        )
        validate_pack(
            AGE1_ELSE_PROOF,
            completion="else",
            expect_proof_check=1,
            label="AGE1 else",
        )
        validate_pack(
            AGE4_EXHAUSTIVE_PROOF,
            completion="exhaustive",
            expect_proof_check=0,
            label="AGE4 exhaustive",
        )
        validate_pack(
            AGE4_ELSE_PROOF,
            completion="else",
            expect_proof_check=0,
            label="AGE4 else",
        )
    except ValueError as exc:
        return fail(str(exc))
    print("[proof-case-analysis-completion-parity-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
