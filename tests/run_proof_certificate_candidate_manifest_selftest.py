#!/usr/bin/env python
from __future__ import annotations

import hashlib
import json
from pathlib import Path


README_PATH = Path("tests/proof_certificate_candidate_manifest/README.md")
DIGEST_AXES_README = Path("tests/proof_certificate_digest_axes/README.md")
ARTIFACT_CERT_README = Path("tests/proof_artifact_certificate_contract/README.md")
BRIDGE_README = Path("pack/age4_proof_artifact_cert_subject_v1/README.md")
CERT_RS = Path("tools/teul-cli/src/cli/cert.rs")
CLEAN_PROOF = Path("pack/age4_proof_detjson_smoke_v1/expected/clean.proof.detjson")
ABORT_PROOF = Path("pack/age4_proof_detjson_smoke_v1/expected/abort.proof.detjson")
BRIDGE_CASES = Path("pack/age4_proof_artifact_cert_subject_v1/cert_cases.json")
CLEAN_FIXTURE = Path("tests/proof_certificate_candidate_manifest/fixtures/clean.proof_certificate_candidate.detjson")
ABORT_FIXTURE = Path("tests/proof_certificate_candidate_manifest/fixtures/abort.proof_certificate_candidate.detjson")

README_SNIPPETS = (
    "## Stable Contract",
    "`ddn.proof_certificate_manifest_candidate.v1`",
    "`pack/age4_proof_detjson_smoke_v1/expected/clean.proof.detjson`",
    "`pack/age4_proof_detjson_smoke_v1/expected/abort.proof.detjson`",
    "`pack/age4_proof_artifact_cert_subject_v1/cert_cases.json`",
    "`tools/teul-cli/src/cli/cert.rs`",
    "`tests/proof_certificate_candidate_manifest/fixtures/clean.proof_certificate_candidate.detjson`",
    "`tests/proof_certificate_candidate_manifest/fixtures/abort.proof_certificate_candidate.detjson`",
    "`python tests/run_proof_certificate_candidate_manifest_selftest.py`",
    "`proof_certificate_candidate_manifest_selftest`",
    "`proof_subject_hash`",
    "`canonical_body_hash`",
    "`proof_runtime_hash`",
    "`solver_translation_hash`",
    "`state_hash`",
    "`trace_hash`",
)
POINTERS = (
    "`tests/proof_certificate_candidate_manifest/README.md`",
    "`python tests/run_proof_certificate_candidate_manifest_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-certificate-candidate-manifest-selftest] fail: {message}")
    return 1


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_file(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def validate_fixture(
    fixture_path: Path,
    proof_path: Path,
    bridge_rows: dict[str, dict],
    *,
    pass_case_id: str,
    tamper_case_id: str,
    verified: bool,
    contract_diag_count: int,
) -> None:
    fixture = load_json(fixture_path)
    proof = load_json(proof_path)
    if str(fixture.get("schema", "")).strip() != "ddn.proof_certificate_manifest_candidate.v1":
        raise ValueError(f"{fixture_path}: schema mismatch")
    if str(fixture.get("proof_schema", "")).strip() != "ddn.proof.detjson.v0":
        raise ValueError(f"{fixture_path}: proof_schema mismatch")
    if str(fixture.get("proof_kind", "")).strip() != "run_contract_certificate_v1":
        raise ValueError(f"{fixture_path}: proof_kind mismatch")
    if str(fixture.get("cert_manifest_schema", "")).strip() != "ddn.cert_manifest.v1":
        raise ValueError(f"{fixture_path}: cert_manifest_schema mismatch")
    if str(fixture.get("cert_algo", "")).strip() != "sha256-proto":
        raise ValueError(f"{fixture_path}: cert_algo mismatch")
    if bool(fixture.get("verified", not verified)) != verified:
        raise ValueError(f"{fixture_path}: verified mismatch")
    if int(fixture.get("contract_diag_count", -1)) != contract_diag_count:
        raise ValueError(f"{fixture_path}: contract_diag_count mismatch")

    subject_hash = sha256_file(proof_path)
    if str(fixture.get("proof_subject_hash", "")).strip() != subject_hash:
        raise ValueError(f"{fixture_path}: proof_subject_hash mismatch")
    if str(bridge_rows[pass_case_id].get("expected_subject_hash", "")).strip() != subject_hash:
        raise ValueError(f"{fixture_path}: bridge pass subject hash mismatch")
    if str(bridge_rows[tamper_case_id].get("expected_subject_hash", "")).strip() != subject_hash:
        raise ValueError(f"{fixture_path}: bridge tamper subject hash mismatch")

    for key in (
        "canonical_body_hash",
        "proof_runtime_hash",
        "solver_translation_hash",
        "state_hash",
        "trace_hash",
    ):
        if str(fixture.get(key, "")).strip() != str(proof.get(key, "")).strip():
            raise ValueError(f"{fixture_path}: {key} mismatch")


def main() -> int:
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")

    try:
        ensure_pointers(DIGEST_AXES_README)
        ensure_pointers(ARTIFACT_CERT_README)
        ensure_pointers(BRIDGE_README)

        cert_src = CERT_RS.read_text(encoding="utf-8")
        if 'const CERT_MANIFEST_SCHEMA: &str = "ddn.cert_manifest.v1";' not in cert_src:
            raise ValueError("cert manifest schema literal missing")
        if 'const CERT_ALGO: &str = "sha256-proto";' not in cert_src:
            raise ValueError("cert algo literal missing")

        bridge_doc = load_json(BRIDGE_CASES)
        rows = bridge_doc.get("cases")
        if not isinstance(rows, list) or len(rows) != 4:
            raise ValueError("bridge cases count mismatch")
        bridge_rows = {str(row.get("id", "")).strip(): row for row in rows if isinstance(row, dict)}

        validate_fixture(
            CLEAN_FIXTURE,
            CLEAN_PROOF,
            bridge_rows,
            pass_case_id="c01_clean_verify_pass",
            tamper_case_id="c02_clean_verify_fail_tamper",
            verified=True,
            contract_diag_count=0,
        )
        validate_fixture(
            ABORT_FIXTURE,
            ABORT_PROOF,
            bridge_rows,
            pass_case_id="c03_abort_verify_pass",
            tamper_case_id="c04_abort_verify_fail_tamper",
            verified=False,
            contract_diag_count=1,
        )

        clean_fixture = load_json(CLEAN_FIXTURE)
        abort_fixture = load_json(ABORT_FIXTURE)
        if str(clean_fixture.get("proof_runtime_hash", "")).strip() != str(abort_fixture.get("proof_runtime_hash", "")).strip():
            raise ValueError("proof_runtime_hash pair mismatch")
        if str(clean_fixture.get("solver_translation_hash", "")).strip() != str(abort_fixture.get("solver_translation_hash", "")).strip():
            raise ValueError("solver_translation_hash pair mismatch")
        for key in ("proof_subject_hash", "canonical_body_hash", "state_hash", "trace_hash"):
            if str(clean_fixture.get(key, "")).strip() == str(abort_fixture.get(key, "")).strip():
                raise ValueError(f"{key} pair should differ")
    except (ValueError, KeyError) as exc:
        return fail(str(exc))

    print("[proof-certificate-candidate-manifest-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
