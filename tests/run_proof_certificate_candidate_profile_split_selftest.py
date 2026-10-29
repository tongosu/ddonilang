#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("tests/proof_certificate_candidate_profile_split/README.md")
CANDIDATE_README = Path("tests/proof_certificate_candidate_manifest/README.md")
CLEAN_FIXTURE = Path("tests/proof_certificate_candidate_manifest/fixtures/clean.proof_certificate_candidate.detjson")
ABORT_FIXTURE = Path("tests/proof_certificate_candidate_manifest/fixtures/abort.proof_certificate_candidate.detjson")

README_SNIPPETS = (
    "## Stable Contract",
    "`tests/proof_certificate_candidate_manifest/fixtures/clean.proof_certificate_candidate.detjson`",
    "`tests/proof_certificate_candidate_manifest/fixtures/abort.proof_certificate_candidate.detjson`",
    "`tests/proof_certificate_candidate_manifest/README.md`",
    "`python tests/run_proof_certificate_candidate_profile_split_selftest.py`",
    "`proof_certificate_candidate_profile_split_selftest`",
    "`schema`",
    "`proof_schema`",
    "`proof_kind`",
    "`cert_manifest_schema`",
    "`cert_algo`",
    "`proof_runtime_hash`",
    "`solver_translation_hash`",
    "`verified`",
    "`contract_diag_count`",
    "`proof_subject_hash`",
    "`canonical_body_hash`",
    "`state_hash`",
    "`trace_hash`",
)
POINTERS = (
    "`tests/proof_certificate_candidate_profile_split/README.md`",
    "`python tests/run_proof_certificate_candidate_profile_split_selftest.py`",
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
    print(f"[proof-certificate-candidate-profile-split-selftest] fail: {message}")
    return 1


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def main() -> int:
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")

    try:
        ensure_pointers(CANDIDATE_README)
        clean = load_json(CLEAN_FIXTURE)
        abort = load_json(ABORT_FIXTURE)
        for key in SHARED_KEYS:
            if str(clean.get(key, "")).strip() != str(abort.get(key, "")).strip():
                raise ValueError(f"shared key mismatch: {key}")
        for key in STATE_KEYS:
            if str(clean.get(key, "")).strip() == str(abort.get(key, "")).strip():
                raise ValueError(f"state key should differ: {key}")
    except ValueError as exc:
        return fail(str(exc))

    print(
        "[proof-certificate-candidate-profile-split-selftest] ok "
        f"shared={len(SHARED_KEYS)} state={len(STATE_KEYS)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
