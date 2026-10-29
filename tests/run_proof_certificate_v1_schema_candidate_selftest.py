#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("tests/proof_certificate_v1_schema_candidate/README.md")
DRAFT_CONTRACT_README = Path("tests/proof_certificate_v1_draft_contract/README.md")
DRAFT_ARTIFACT_README = Path("tests/proof_certificate_v1_draft_artifact/README.md")
PROMOTION_README = Path("tests/proof_certificate_v1_promotion_candidate/README.md")
CLEAN_ARTIFACT = Path("tests/proof_certificate_v1_draft_artifact/fixtures/clean.proof_certificate_v1_draft_artifact.detjson")
ABORT_ARTIFACT = Path("tests/proof_certificate_v1_draft_artifact/fixtures/abort.proof_certificate_v1_draft_artifact.detjson")
CLEAN_FIXTURE = Path("tests/proof_certificate_v1_schema_candidate/fixtures/clean.proof_certificate_v1_candidate.detjson")
ABORT_FIXTURE = Path("tests/proof_certificate_v1_schema_candidate/fixtures/abort.proof_certificate_v1_candidate.detjson")

README_SNIPPETS = (
    "## Stable Contract",
    "`tests/proof_certificate_v1_draft_contract/README.md`",
    "`tests/proof_certificate_v1_draft_artifact/fixtures/clean.proof_certificate_v1_draft_artifact.detjson`",
    "`tests/proof_certificate_v1_draft_artifact/fixtures/abort.proof_certificate_v1_draft_artifact.detjson`",
    "`tests/proof_certificate_v1_schema_candidate/fixtures/clean.proof_certificate_v1_candidate.detjson`",
    "`tests/proof_certificate_v1_schema_candidate/fixtures/abort.proof_certificate_v1_candidate.detjson`",
    "`python tests/run_proof_certificate_v1_schema_candidate_selftest.py`",
    "`proof_certificate_v1_schema_candidate_selftest`",
    "`ddn.proof_certificate_v1_candidate.v1`",
)
POINTERS = (
    "`tests/proof_certificate_v1_schema_candidate/README.md`",
    "`python tests/run_proof_certificate_v1_schema_candidate_selftest.py`",
)
EXPECTED_KEYS = (
    "schema",
    "source_pack_schema",
    "source_pack_id",
    "profile",
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


def fail(message: str) -> int:
    print(f"[proof-certificate-v1-schema-candidate-selftest] fail: {message}")
    return 1


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def flatten_artifact(artifact: dict) -> dict:
    manifest = artifact["candidate_manifest"]
    return {
        "schema": "ddn.proof_certificate_v1_candidate.v1",
        "source_pack_schema": artifact["source_pack_schema"],
        "source_pack_id": artifact["source_pack_id"],
        "profile": artifact["profile"],
        "proof_schema": manifest["proof_schema"],
        "proof_kind": manifest["proof_kind"],
        "cert_manifest_schema": manifest["cert_manifest_schema"],
        "cert_algo": manifest["cert_algo"],
        "verified": manifest["verified"],
        "contract_diag_count": manifest["contract_diag_count"],
        "proof_subject_hash": manifest["proof_subject_hash"],
        "canonical_body_hash": manifest["canonical_body_hash"],
        "proof_runtime_hash": manifest["proof_runtime_hash"],
        "solver_translation_hash": manifest["solver_translation_hash"],
        "state_hash": manifest["state_hash"],
        "trace_hash": manifest["trace_hash"],
    }


def validate_fixture(fixture_path: Path, artifact_path: Path, *, profile: str) -> None:
    fixture = load_json(fixture_path)
    artifact = load_json(artifact_path)
    if sorted(fixture.keys()) != sorted(EXPECTED_KEYS):
        raise ValueError(f"{fixture_path}: keys mismatch")
    if str(fixture.get("schema", "")).strip() != "ddn.proof_certificate_v1_candidate.v1":
        raise ValueError(f"{fixture_path}: schema mismatch")
    if str(fixture.get("profile", "")).strip() != profile:
        raise ValueError(f"{fixture_path}: profile mismatch")
    if fixture != flatten_artifact(artifact):
        raise ValueError(f"{fixture_path}: flatten mismatch")


def main() -> int:
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")

    try:
        ensure_pointers(DRAFT_CONTRACT_README)
        ensure_pointers(DRAFT_ARTIFACT_README)
        ensure_pointers(PROMOTION_README)
        validate_fixture(CLEAN_FIXTURE, CLEAN_ARTIFACT, profile="clean")
        validate_fixture(ABORT_FIXTURE, ABORT_ARTIFACT, profile="abort")
    except ValueError as exc:
        return fail(str(exc))

    print("[proof-certificate-v1-schema-candidate-selftest] ok profiles=2 fields=16")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
