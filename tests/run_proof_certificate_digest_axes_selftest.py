#!/usr/bin/env python
from __future__ import annotations

import hashlib
import json
from pathlib import Path


README_PATH = Path("tests/proof_certificate_digest_axes/README.md")
CONTRACT_README = Path("tests/proof_artifact_certificate_contract/README.md")
BRIDGE_README = Path("pack/age4_proof_artifact_cert_subject_v1/README.md")
CLEAN_PROOF = Path("pack/age4_proof_detjson_smoke_v1/expected/clean.proof.detjson")
ABORT_PROOF = Path("pack/age4_proof_detjson_smoke_v1/expected/abort.proof.detjson")
BRIDGE_CASES = Path("pack/age4_proof_artifact_cert_subject_v1/cert_cases.json")

README_SNIPPETS = (
    "## Stable Contract",
    "`pack/age4_proof_detjson_smoke_v1/expected/clean.proof.detjson`",
    "`pack/age4_proof_detjson_smoke_v1/expected/abort.proof.detjson`",
    "`pack/age4_proof_artifact_cert_subject_v1/cert_cases.json`",
    "`tests/proof_artifact_certificate_contract/README.md`",
    "`python tests/run_proof_certificate_digest_axes_selftest.py`",
    "`proof_certificate_digest_axes_selftest`",
    "`subject_hash(sha256(file bytes))`",
    "`canonical_body_hash`",
    "`proof_runtime_hash`",
    "`solver_translation_hash`",
    "`state_hash`",
    "`trace_hash`",
)
POINTERS = (
    "`tests/proof_certificate_digest_axes/README.md`",
    "`python tests/run_proof_certificate_digest_axes_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-certificate-digest-axes-selftest] fail: {message}")
    return 1


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def sha256_file(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")

    try:
        ensure_pointers(CONTRACT_README)
        ensure_pointers(BRIDGE_README)
        clean_doc = load_json(CLEAN_PROOF)
        abort_doc = load_json(ABORT_PROOF)
        bridge_doc = load_json(BRIDGE_CASES)
        if str(clean_doc.get("schema", "")).strip() != "ddn.proof.detjson.v0":
            raise ValueError("clean schema mismatch")
        if str(abort_doc.get("schema", "")).strip() != "ddn.proof.detjson.v0":
            raise ValueError("abort schema mismatch")
        if str(clean_doc.get("kind", "")).strip() != "run_contract_certificate_v1":
            raise ValueError("clean kind mismatch")
        if str(abort_doc.get("kind", "")).strip() != "run_contract_certificate_v1":
            raise ValueError("abort kind mismatch")
        if bool(clean_doc.get("verified", False)) is not True:
            raise ValueError("clean verified mismatch")
        if bool(abort_doc.get("verified", True)) is not False:
            raise ValueError("abort verified mismatch")

        clean_subject = sha256_file(CLEAN_PROOF)
        abort_subject = sha256_file(ABORT_PROOF)
        if clean_subject == abort_subject:
            raise ValueError("subject hash should differ")
        if str(clean_doc.get("canonical_body_hash", "")).strip() == str(abort_doc.get("canonical_body_hash", "")).strip():
            raise ValueError("canonical_body_hash should differ")
        if str(clean_doc.get("state_hash", "")).strip() == str(abort_doc.get("state_hash", "")).strip():
            raise ValueError("state_hash should differ")
        if str(clean_doc.get("trace_hash", "")).strip() == str(abort_doc.get("trace_hash", "")).strip():
            raise ValueError("trace_hash should differ")
        if str(clean_doc.get("proof_runtime_hash", "")).strip() != str(abort_doc.get("proof_runtime_hash", "")).strip():
            raise ValueError("proof_runtime_hash should match")
        if str(clean_doc.get("solver_translation_hash", "")).strip() != str(abort_doc.get("solver_translation_hash", "")).strip():
            raise ValueError("solver_translation_hash should match")

        if str(bridge_doc.get("schema", "")).strip() != "ddn.gogae9.w95.cert_cases.v1":
            raise ValueError("bridge schema mismatch")
        case_rows = bridge_doc.get("cases")
        if not isinstance(case_rows, list) or len(case_rows) != 4:
            raise ValueError("bridge cases count mismatch")
        by_id = {str(row.get("id", "")).strip(): row for row in case_rows if isinstance(row, dict)}
        if str(by_id["c01_clean_verify_pass"].get("expected_subject_hash", "")).strip() != clean_subject:
            raise ValueError("clean pass subject hash mismatch")
        if str(by_id["c02_clean_verify_fail_tamper"].get("expected_subject_hash", "")).strip() != clean_subject:
            raise ValueError("clean tamper subject hash mismatch")
        if str(by_id["c03_abort_verify_pass"].get("expected_subject_hash", "")).strip() != abort_subject:
            raise ValueError("abort pass subject hash mismatch")
        if str(by_id["c04_abort_verify_fail_tamper"].get("expected_subject_hash", "")).strip() != abort_subject:
            raise ValueError("abort tamper subject hash mismatch")
    except (ValueError, KeyError) as exc:
        return fail(str(exc))

    print(
        "[proof-certificate-digest-axes-selftest] ok "
        f"clean_subject_hash={clean_subject} "
        f"abort_subject_hash={abort_subject}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
