#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("tests/proof_certificate_candidate_layers/README.md")
CANDIDATE_README = Path("tests/proof_certificate_candidate_manifest/README.md")
PROFILE_SPLIT_README = Path("tests/proof_certificate_candidate_profile_split/README.md")
SHARED_FIXTURE = Path("tests/proof_certificate_candidate_layers/fixtures/shared.proof_certificate_candidate_shell.detjson")
CLEAN_STATE_FIXTURE = Path("tests/proof_certificate_candidate_layers/fixtures/clean.proof_certificate_candidate_state.detjson")
ABORT_STATE_FIXTURE = Path("tests/proof_certificate_candidate_layers/fixtures/abort.proof_certificate_candidate_state.detjson")
CLEAN_CANDIDATE = Path("tests/proof_certificate_candidate_manifest/fixtures/clean.proof_certificate_candidate.detjson")
ABORT_CANDIDATE = Path("tests/proof_certificate_candidate_manifest/fixtures/abort.proof_certificate_candidate.detjson")

README_SNIPPETS = (
    "## Stable Contract",
    "`tests/proof_certificate_candidate_manifest/fixtures/clean.proof_certificate_candidate.detjson`",
    "`tests/proof_certificate_candidate_manifest/fixtures/abort.proof_certificate_candidate.detjson`",
    "`tests/proof_certificate_candidate_layers/fixtures/shared.proof_certificate_candidate_shell.detjson`",
    "`tests/proof_certificate_candidate_layers/fixtures/clean.proof_certificate_candidate_state.detjson`",
    "`tests/proof_certificate_candidate_layers/fixtures/abort.proof_certificate_candidate_state.detjson`",
    "`python tests/run_proof_certificate_candidate_layers_selftest.py`",
    "`proof_certificate_candidate_layers_selftest`",
)
POINTERS = (
    "`tests/proof_certificate_candidate_layers/README.md`",
    "`python tests/run_proof_certificate_candidate_layers_selftest.py`",
)

SHARED_KEYS = (
    "schema",
    "proof_schema",
    "proof_kind",
    "cert_manifest_schema",
    "cert_algo",
    "proof_runtime_hash",
    "solver_translation_hash",
)
STATE_KEYS = (
    "verified",
    "contract_diag_count",
    "proof_subject_hash",
    "canonical_body_hash",
    "state_hash",
    "trace_hash",
)


def fail(message: str) -> int:
    print(f"[proof-certificate-candidate-layers-selftest] fail: {message}")
    return 1


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def build_candidate(shared: dict, state: dict) -> dict:
    merged = dict(shared)
    merged.update(state)
    return merged


def main() -> int:
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")

    try:
        ensure_pointers(CANDIDATE_README)
        ensure_pointers(PROFILE_SPLIT_README)
        shared = load_json(SHARED_FIXTURE)
        clean_state = load_json(CLEAN_STATE_FIXTURE)
        abort_state = load_json(ABORT_STATE_FIXTURE)
        clean_candidate = load_json(CLEAN_CANDIDATE)
        abort_candidate = load_json(ABORT_CANDIDATE)

        if sorted(shared.keys()) != sorted(SHARED_KEYS):
            raise ValueError("shared shell keys mismatch")
        if sorted(clean_state.keys()) != sorted(STATE_KEYS):
            raise ValueError("clean state keys mismatch")
        if sorted(abort_state.keys()) != sorted(STATE_KEYS):
            raise ValueError("abort state keys mismatch")

        if build_candidate(shared, clean_state) != clean_candidate:
            raise ValueError("clean candidate reconstruction mismatch")
        if build_candidate(shared, abort_state) != abort_candidate:
            raise ValueError("abort candidate reconstruction mismatch")
    except ValueError as exc:
        return fail(str(exc))

    print("[proof-certificate-candidate-layers-selftest] ok shared=7 state=6")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
