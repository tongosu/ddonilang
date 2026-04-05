#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("tests/proof_certificate_v1_promotion/README.md")
DRAFT_CONTRACT_README = Path("tests/proof_certificate_v1_draft_contract/README.md")
SCHEMA_CANDIDATE_README = Path("tests/proof_certificate_v1_schema_candidate/README.md")
SCHEMA_SPLIT_README = Path("tests/proof_certificate_v1_schema_candidate_split/README.md")
PROMOTION_CANDIDATE_README = Path("tests/proof_certificate_v1_promotion_candidate/README.md")
SIGNED_CONTRACT_README = Path("tests/proof_certificate_v1_signed_contract/README.md")
CONSUMER_CONTRACT_README = Path("tests/proof_certificate_v1_consumer_contract/README.md")
FAMILY_README = Path("tests/proof_certificate_v1_family/README.md")
CERT_PACK_README = Path("pack/age4_proof_detjson_smoke_v1/README.md")
CLEAN_CANDIDATE = Path("tests/proof_certificate_v1_schema_candidate/fixtures/clean.proof_certificate_v1_candidate.detjson")
ABORT_CANDIDATE = Path("tests/proof_certificate_v1_schema_candidate/fixtures/abort.proof_certificate_v1_candidate.detjson")
SHARED_SHELL = Path("tests/proof_certificate_v1_schema_candidate_split/fixtures/shared.proof_certificate_v1_candidate_shell.detjson")
CLEAN_STATE = Path("tests/proof_certificate_v1_schema_candidate_split/fixtures/clean.proof_certificate_v1_candidate_state.detjson")
ABORT_STATE = Path("tests/proof_certificate_v1_schema_candidate_split/fixtures/abort.proof_certificate_v1_candidate_state.detjson")

README_SNIPPETS = (
    "## Stable Contract",
    "`tests/proof_certificate_v1_draft_contract/README.md`",
    "`tests/proof_certificate_v1_schema_candidate/README.md`",
    "`tests/proof_certificate_v1_schema_candidate_split/README.md`",
    "`tests/proof_certificate_v1_signed_contract/README.md`",
    "`tests/proof_certificate_v1_consumer_contract/README.md`",
    "`tests/proof_certificate_v1_family/README.md`",
    "`tests/proof_certificate_v1_schema_candidate/fixtures/clean.proof_certificate_v1_candidate.detjson`",
    "`tests/proof_certificate_v1_schema_candidate/fixtures/abort.proof_certificate_v1_candidate.detjson`",
    "`python tests/run_proof_certificate_v1_promotion_selftest.py`",
    "`python tests/run_proof_certificate_v1_family_selftest.py`",
    "`proof_certificate_v1_promotion_selftest`",
    "`ddn.proof_certificate_v1_candidate.v1`",
    "`shared shell(9)` + `state delta(7)`",
)
POINTERS = (
    "`tests/proof_certificate_v1_promotion/README.md`",
    "`python tests/run_proof_certificate_v1_promotion_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-certificate-v1-promotion-selftest] fail: {message}")
    return 1


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def build_candidate(shell: dict, state: dict) -> dict:
    merged = dict(shell)
    merged.update(state)
    return merged


def main() -> int:
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")

    try:
        ensure_pointers(DRAFT_CONTRACT_README)
        ensure_pointers(SCHEMA_CANDIDATE_README)
        ensure_pointers(SCHEMA_SPLIT_README)
        ensure_pointers(PROMOTION_CANDIDATE_README)
        ensure_pointers(SIGNED_CONTRACT_README)
        ensure_pointers(CONSUMER_CONTRACT_README)
        ensure_pointers(FAMILY_README)
        ensure_pointers(CERT_PACK_README)

        clean = load_json(CLEAN_CANDIDATE)
        abort = load_json(ABORT_CANDIDATE)
        shared = load_json(SHARED_SHELL)
        clean_state = load_json(CLEAN_STATE)
        abort_state = load_json(ABORT_STATE)

        if build_candidate(shared, clean_state) != clean:
            raise ValueError("clean promotion reconstruction mismatch")
        if build_candidate(shared, abort_state) != abort:
            raise ValueError("abort promotion reconstruction mismatch")
        if str(clean.get("schema", "")).strip() != "ddn.proof_certificate_v1_candidate.v1":
            raise ValueError("clean schema mismatch")
        if str(abort.get("schema", "")).strip() != "ddn.proof_certificate_v1_candidate.v1":
            raise ValueError("abort schema mismatch")
        sanity_gate_text = Path("tests/run_ci_sanity_gate.py").read_text(encoding="utf-8")
        if '"proof_certificate_v1_family_selftest"' not in sanity_gate_text:
            raise ValueError("missing family step in sanity gate")
        if '[py, "tests/run_proof_certificate_v1_family_selftest.py"]' not in sanity_gate_text:
            raise ValueError("missing family command in sanity gate")
    except ValueError as exc:
        return fail(str(exc))

    print("[proof-certificate-v1-promotion-selftest] ok profiles=2 shared=9 state=7")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
