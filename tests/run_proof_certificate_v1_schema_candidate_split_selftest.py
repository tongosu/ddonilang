#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("tests/proof_certificate_v1_schema_candidate_split/README.md")
SCHEMA_CANDIDATE_README = Path("tests/proof_certificate_v1_schema_candidate/README.md")
DRAFT_CONTRACT_README = Path("tests/proof_certificate_v1_draft_contract/README.md")
PROMOTION_README = Path("tests/proof_certificate_v1_promotion_candidate/README.md")
CLEAN_FIXTURE = Path("tests/proof_certificate_v1_schema_candidate/fixtures/clean.proof_certificate_v1_candidate.detjson")
ABORT_FIXTURE = Path("tests/proof_certificate_v1_schema_candidate/fixtures/abort.proof_certificate_v1_candidate.detjson")
SHARED_FIXTURE = Path("tests/proof_certificate_v1_schema_candidate_split/fixtures/shared.proof_certificate_v1_candidate_shell.detjson")
CLEAN_STATE_FIXTURE = Path("tests/proof_certificate_v1_schema_candidate_split/fixtures/clean.proof_certificate_v1_candidate_state.detjson")
ABORT_STATE_FIXTURE = Path("tests/proof_certificate_v1_schema_candidate_split/fixtures/abort.proof_certificate_v1_candidate_state.detjson")

README_SNIPPETS = (
    "## Stable Contract",
    "`tests/proof_certificate_v1_schema_candidate/fixtures/clean.proof_certificate_v1_candidate.detjson`",
    "`tests/proof_certificate_v1_schema_candidate/fixtures/abort.proof_certificate_v1_candidate.detjson`",
    "`tests/proof_certificate_v1_schema_candidate_split/fixtures/shared.proof_certificate_v1_candidate_shell.detjson`",
    "`tests/proof_certificate_v1_schema_candidate_split/fixtures/clean.proof_certificate_v1_candidate_state.detjson`",
    "`tests/proof_certificate_v1_schema_candidate_split/fixtures/abort.proof_certificate_v1_candidate_state.detjson`",
    "`python tests/run_proof_certificate_v1_schema_candidate_split_selftest.py`",
    "`proof_certificate_v1_schema_candidate_split_selftest`",
    "`ddn.proof_certificate_v1_candidate.v1`",
)
POINTERS = (
    "`tests/proof_certificate_v1_schema_candidate_split/README.md`",
    "`python tests/run_proof_certificate_v1_schema_candidate_split_selftest.py`",
)
SHARED_KEYS = (
    "schema",
    "source_pack_schema",
    "source_pack_id",
    "proof_schema",
    "proof_kind",
    "cert_manifest_schema",
    "cert_algo",
    "proof_runtime_hash",
    "solver_translation_hash",
)
STATE_KEYS = (
    "profile",
    "verified",
    "contract_diag_count",
    "proof_subject_hash",
    "canonical_body_hash",
    "state_hash",
    "trace_hash",
)


def fail(message: str) -> int:
    print(f"[proof-certificate-v1-schema-candidate-split-selftest] fail: {message}")
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
        ensure_pointers(SCHEMA_CANDIDATE_README)
        ensure_pointers(DRAFT_CONTRACT_README)
        ensure_pointers(PROMOTION_README)
        clean = load_json(CLEAN_FIXTURE)
        abort = load_json(ABORT_FIXTURE)
        shared = load_json(SHARED_FIXTURE)
        clean_state = load_json(CLEAN_STATE_FIXTURE)
        abort_state = load_json(ABORT_STATE_FIXTURE)

        if sorted(shared.keys()) != sorted(SHARED_KEYS):
            raise ValueError("shared shell keys mismatch")
        if sorted(clean_state.keys()) != sorted(STATE_KEYS):
            raise ValueError("clean state keys mismatch")
        if sorted(abort_state.keys()) != sorted(STATE_KEYS):
            raise ValueError("abort state keys mismatch")

        if build_candidate(shared, clean_state) != clean:
            raise ValueError("clean schema candidate reconstruction mismatch")
        if build_candidate(shared, abort_state) != abort:
            raise ValueError("abort schema candidate reconstruction mismatch")
    except ValueError as exc:
        return fail(str(exc))

    print("[proof-certificate-v1-schema-candidate-split-selftest] ok shared=9 state=7")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
