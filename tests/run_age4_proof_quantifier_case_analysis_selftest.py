#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("tests/age4_proof_quantifier_case_analysis/README.md")
TRANSLATION_README = Path("pack/age4_proof_solver_translation_smoke_v1/README.md")
FORALL_README = Path("pack/age4_proof_case_analysis_forall_solver_open_search_replay_v1/README.md")
EXISTS_README = Path("pack/age4_proof_case_analysis_exists_solver_open_search_replay_v1/README.md")
UNIQUE_README = Path("pack/age4_proof_case_analysis_solver_open_search_replay_v1/README.md")
ELSE_README = Path("pack/age4_proof_case_analysis_else_solver_open_search_replay_v1/README.md")
TRANSLATION_PROOF = Path("pack/age4_proof_solver_translation_smoke_v1/expected/proof.detjson")
FORALL_PROOF = Path("pack/age4_proof_case_analysis_forall_solver_open_search_replay_v1/expected/proof.detjson")
EXISTS_PROOF = Path("pack/age4_proof_case_analysis_exists_solver_open_search_replay_v1/expected/proof.detjson")
UNIQUE_PROOF = Path("pack/age4_proof_case_analysis_solver_open_search_replay_v1/expected/proof.detjson")
ELSE_PROOF = Path("pack/age4_proof_case_analysis_else_solver_open_search_replay_v1/expected/proof.detjson")
README_SNIPPETS = (
    "## Stable Contract",
    "`pack/age4_proof_solver_translation_smoke_v1/expected/proof.detjson`",
    "`pack/age4_proof_case_analysis_forall_solver_open_search_replay_v1/expected/proof.detjson`",
    "`pack/age4_proof_case_analysis_exists_solver_open_search_replay_v1/expected/proof.detjson`",
    "`pack/age4_proof_case_analysis_solver_open_search_replay_v1/expected/proof.detjson`",
    "`pack/age4_proof_case_analysis_else_solver_open_search_replay_v1/expected/proof.detjson`",
    "`forall`",
    "`exists`",
    "`exists_unique`",
    "`check`",
    "`counterexample`",
    "`solve`",
    "`exhaustive`",
    "`else`",
    "`python tests/run_age4_proof_quantifier_case_analysis_selftest.py`",
)
POINTERS = (
    "`tests/age4_proof_quantifier_case_analysis/README.md`",
    "`python tests/run_age4_proof_quantifier_case_analysis_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[age4-proof-quantifier-case-analysis-selftest] fail: {message}")
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


def validate_translation_smoke() -> None:
    doc = load_json(TRANSLATION_PROOF)
    translation = doc["solver_translation"]
    runtime = doc["proof_runtime"]
    if translation["quantifier_count"] != 3:
        raise ValueError("translation smoke: quantifier_count != 3")
    quantifiers = [
        str(item.get("quantifier", "")).strip()
        for item in translation["items"]
        if item.get("kind") == "quantifier"
    ]
    if quantifiers != ["forall", "exists", "exists_unique"]:
        raise ValueError(f"translation smoke: quantifier order mismatch {quantifiers}")
    if translation["case_analysis_count"] != 2:
        raise ValueError("translation smoke: case_analysis_count != 2")
    completions = [
        str(item.get("completion", "")).strip()
        for item in translation["items"]
        if item.get("kind") == "case_analysis"
    ]
    if completions != ["exhaustive", "else"]:
        raise ValueError(f"translation smoke: completion order mismatch {completions}")
    if translation["solver_open_count"] != 0:
        raise ValueError("translation smoke: solver_open_count != 0")
    if runtime["proof_check_count"] != 0:
        raise ValueError("translation smoke: runtime proof_check_count != 0")
    if runtime["solver_check_count"] != 0 or runtime["solver_search_count"] != 0:
        raise ValueError("translation smoke: runtime solver counts must be zero")


def validate_mixed(path: Path, *, quantifier: str, completion: str, label: str) -> None:
    doc = load_json(path)
    translation = doc["solver_translation"]
    runtime = doc["proof_runtime"]
    if translation["quantifier_count"] != 1:
        raise ValueError(f"{label}: quantifier_count != 1")
    q = find_item(translation["items"], "quantifier")
    if q is None or q.get("quantifier") != quantifier:
        raise ValueError(f"{label}: quantifier mismatch")
    if translation["case_analysis_count"] != 1:
        raise ValueError(f"{label}: case_analysis_count != 1")
    case = find_item(translation["items"], "case_analysis")
    if case is None or case.get("completion") != completion:
        raise ValueError(f"{label}: completion mismatch")
    if translation["solver_open_count"] != 3:
        raise ValueError(f"{label}: solver_open_count != 3")
    for op in ("check", "counterexample", "solve"):
        if find_item(translation["items"], "solver_open", op) is None:
            raise ValueError(f"{label}: missing translation solver_open {op}")
    if runtime["proof_check_count"] != 0:
        raise ValueError(f"{label}: runtime proof_check_count != 0")
    if runtime["solver_check_count"] != 1:
        raise ValueError(f"{label}: runtime solver_check_count != 1")
    if runtime["solver_search_count"] != 2:
        raise ValueError(f"{label}: runtime solver_search_count != 2")
    if find_item(runtime["items"], "solver_check") is None:
        raise ValueError(f"{label}: missing runtime solver_check")
    for op in ("counterexample", "solve"):
        if find_item(runtime["items"], "solver_search", op) is None:
            raise ValueError(f"{label}: missing runtime solver_search {op}")


def main() -> int:
    if not README_PATH.exists():
        return fail(f"missing readme: {README_PATH}")
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")
    try:
        for path in (TRANSLATION_README, FORALL_README, EXISTS_README, UNIQUE_README, ELSE_README):
            ensure_pointers(path)
        validate_translation_smoke()
        validate_mixed(FORALL_PROOF, quantifier="forall", completion="exhaustive", label="forall mixed")
        validate_mixed(EXISTS_PROOF, quantifier="exists", completion="exhaustive", label="exists mixed")
        validate_mixed(UNIQUE_PROOF, quantifier="exists_unique", completion="exhaustive", label="exists_unique mixed")
        validate_mixed(ELSE_PROOF, quantifier="exists_unique", completion="else", label="else mixed")
    except ValueError as exc:
        return fail(str(exc))
    print("[age4-proof-quantifier-case-analysis-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
