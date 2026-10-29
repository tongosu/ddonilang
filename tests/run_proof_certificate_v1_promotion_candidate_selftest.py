#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("tests/proof_certificate_v1_promotion_candidate/README.md")
ARTIFACT_CERT_README = Path("tests/proof_artifact_certificate_contract/README.md")
DIGEST_AXES_README = Path("tests/proof_certificate_digest_axes/README.md")
CANDIDATE_README = Path("tests/proof_certificate_candidate_manifest/README.md")
PROFILE_SPLIT_README = Path("tests/proof_certificate_candidate_profile_split/README.md")
LAYERS_README = Path("tests/proof_certificate_candidate_layers/README.md")
BRIDGE_README = Path("pack/age4_proof_artifact_cert_subject_v1/README.md")
CLEAN_CANDIDATE = Path("tests/proof_certificate_candidate_manifest/fixtures/clean.proof_certificate_candidate.detjson")
ABORT_CANDIDATE = Path("tests/proof_certificate_candidate_manifest/fixtures/abort.proof_certificate_candidate.detjson")
SHARED_SHELL = Path("tests/proof_certificate_candidate_layers/fixtures/shared.proof_certificate_candidate_shell.detjson")
CLEAN_STATE = Path("tests/proof_certificate_candidate_layers/fixtures/clean.proof_certificate_candidate_state.detjson")
ABORT_STATE = Path("tests/proof_certificate_candidate_layers/fixtures/abort.proof_certificate_candidate_state.detjson")

README_SNIPPETS = (
    "## Stable Contract",
    "`tests/proof_artifact_certificate_contract/README.md`",
    "`tests/proof_certificate_digest_axes/README.md`",
    "`tests/proof_certificate_candidate_manifest/README.md`",
    "`tests/proof_certificate_candidate_profile_split/README.md`",
    "`tests/proof_certificate_candidate_layers/README.md`",
    "`tests/proof_certificate_candidate_manifest/fixtures/clean.proof_certificate_candidate.detjson`",
    "`tests/proof_certificate_candidate_manifest/fixtures/abort.proof_certificate_candidate.detjson`",
    "`tests/proof_certificate_candidate_layers/fixtures/shared.proof_certificate_candidate_shell.detjson`",
    "`tests/proof_certificate_candidate_layers/fixtures/clean.proof_certificate_candidate_state.detjson`",
    "`tests/proof_certificate_candidate_layers/fixtures/abort.proof_certificate_candidate_state.detjson`",
    "`python tests/run_proof_certificate_v1_promotion_candidate_selftest.py`",
    "`proof_certificate_v1_promotion_candidate_selftest`",
)
POINTERS = (
    "`tests/proof_certificate_v1_promotion_candidate/README.md`",
    "`python tests/run_proof_certificate_v1_promotion_candidate_selftest.py`",
)

MANIFEST_KEYS = (
    "schema",
    "proof_schema",
    "proof_kind",
    "cert_manifest_schema",
    "cert_algo",
    "verified",
    "contract_diag_count",
    "proof_subject_hash",
    "canonical_body_hash",
    "proof_runtime_hash",
    "solver_translation_hash",
    "state_hash",
    "trace_hash",
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
    print(f"[proof-certificate-v1-promotion-candidate-selftest] fail: {message}")
    return 1


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def build_candidate(shell: dict, state: dict) -> dict:
    out = dict(shell)
    out.update(state)
    return out


def main() -> int:
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")

    try:
        ensure_pointers(ARTIFACT_CERT_README)
        ensure_pointers(DIGEST_AXES_README)
        ensure_pointers(CANDIDATE_README)
        ensure_pointers(PROFILE_SPLIT_README)
        ensure_pointers(LAYERS_README)
        ensure_pointers(BRIDGE_README)

        clean_candidate = load_json(CLEAN_CANDIDATE)
        abort_candidate = load_json(ABORT_CANDIDATE)
        shared_shell = load_json(SHARED_SHELL)
        clean_state = load_json(CLEAN_STATE)
        abort_state = load_json(ABORT_STATE)

        if sorted(clean_candidate.keys()) != sorted(MANIFEST_KEYS):
            raise ValueError("clean candidate manifest keys mismatch")
        if sorted(abort_candidate.keys()) != sorted(MANIFEST_KEYS):
            raise ValueError("abort candidate manifest keys mismatch")
        if sorted(shared_shell.keys()) != sorted(SHARED_KEYS):
            raise ValueError("shared shell keys mismatch")
        if sorted(clean_state.keys()) != sorted(STATE_KEYS):
            raise ValueError("clean state keys mismatch")
        if sorted(abort_state.keys()) != sorted(STATE_KEYS):
            raise ValueError("abort state keys mismatch")

        if build_candidate(shared_shell, clean_state) != clean_candidate:
            raise ValueError("clean promotion candidate reconstruction mismatch")
        if build_candidate(shared_shell, abort_state) != abort_candidate:
            raise ValueError("abort promotion candidate reconstruction mismatch")

        for key in SHARED_KEYS:
            if str(clean_candidate.get(key, "")).strip() != str(abort_candidate.get(key, "")).strip():
                raise ValueError(f"shared promotion key mismatch: {key}")
        for key in STATE_KEYS:
            if str(clean_candidate.get(key, "")).strip() == str(abort_candidate.get(key, "")).strip():
                raise ValueError(f"state promotion key should differ: {key}")
    except ValueError as exc:
        return fail(str(exc))

    print(
        "[proof-certificate-v1-promotion-candidate-selftest] ok "
        f"manifest={len(MANIFEST_KEYS)} shell={len(SHARED_KEYS)} state={len(STATE_KEYS)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
