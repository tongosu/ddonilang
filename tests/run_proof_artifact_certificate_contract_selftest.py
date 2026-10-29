#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path


README_PATH = Path("tests/proof_artifact_certificate_contract/README.md")
AGE4_PROOF_README = Path("pack/age4_proof_detjson_smoke_v1/README.md")
AGE4_BRIDGE_README = Path("pack/age4_proof_artifact_cert_subject_v1/README.md")
W95_CERT_README = Path("pack/gogae9_w95_cert/README.md")
AGE4_CLEAN_PROOF = Path("pack/age4_proof_detjson_smoke_v1/expected/clean.proof.detjson")
AGE4_ABORT_PROOF = Path("pack/age4_proof_detjson_smoke_v1/expected/abort.proof.detjson")
AGE4_BRIDGE_GOLDEN = Path("pack/age4_proof_artifact_cert_subject_v1/golden.detjson")
AGE4_BRIDGE_CASES = Path("pack/age4_proof_artifact_cert_subject_v1/cert_cases.json")
W95_GOLDEN = Path("pack/gogae9_w95_cert/golden.detjson")
W95_CASES = Path("pack/gogae9_w95_cert/cert_cases.json")
README_SNIPPETS = (
    "## Stable Contract",
    "`ddn.proof.detjson.v0`",
    "`ddn.proof_artifact_summary.v1`",
    "`ddn.gogae9.w95.cert_report.v1`",
    "`pack/age4_proof_detjson_smoke_v1/expected/clean.proof.detjson`",
    "`pack/age4_proof_detjson_smoke_v1/expected/abort.proof.detjson`",
    "`pack/age4_proof_artifact_cert_subject_v1/cert_cases.json`",
    "`pack/age4_proof_artifact_cert_subject_v1/golden.detjson`",
    "`tests/run_proof_artifact_digest_selftest.py`",
    "`tests/run_age4_proof_artifact_report_selftest.py`",
    "`pack/gogae9_w95_cert/golden.detjson`",
    "`pack/gogae9_w95_cert/cert_cases.json`",
    "`tests/run_w95_cert_pack_check.py`",
    "`python tests/run_w95_cert_pack_check.py --pack pack/age4_proof_artifact_cert_subject_v1`",
    "`python tests/run_proof_artifact_certificate_contract_selftest.py`",
    "`proof_artifact_certificate_contract_selftest`",
)
POINTERS = (
    "`tests/proof_artifact_certificate_contract/README.md`",
    "`python tests/run_proof_artifact_certificate_contract_selftest.py`",
)


def fail(message: str) -> int:
    print(f"[proof-artifact-certificate-contract-selftest] fail: {message}")
    return 1


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def ensure_pointers(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    for pointer in POINTERS:
        if pointer not in text:
            raise ValueError(f"missing pointer in {path}: {pointer}")


def validate_age4_proof(path: Path, *, verified: bool, contract_diag_count: int, label: str) -> None:
    doc = load_json(path)
    if str(doc.get("schema", "")).strip() != "ddn.proof.detjson.v0":
        raise ValueError(f"{label}: schema mismatch")
    if str(doc.get("kind", "")).strip() != "run_contract_certificate_v1":
        raise ValueError(f"{label}: kind mismatch")
    if bool(doc.get("verified", False)) != verified:
        raise ValueError(f"{label}: verified mismatch")
    if int(doc.get("contract_diag_count", -1)) != contract_diag_count:
        raise ValueError(f"{label}: contract_diag_count mismatch")
    if not str(doc.get("state_hash", "")).startswith("blake3:"):
        raise ValueError(f"{label}: state_hash missing")
    if not str(doc.get("trace_hash", "")).startswith("blake3:"):
        raise ValueError(f"{label}: trace_hash missing")


def validate_w95() -> None:
    cases = load_json(W95_CASES)
    golden = load_json(W95_GOLDEN)
    if str(cases.get("schema", "")).strip() != "ddn.gogae9.w95.cert_cases.v1":
        raise ValueError("w95 cases schema mismatch")
    case_rows = cases.get("cases")
    if not isinstance(case_rows, list) or len(case_rows) != 2:
        raise ValueError("w95 cases count mismatch")
    subject_hashes = {str(row.get("expected_subject_hash", "")).strip() for row in case_rows if isinstance(row, dict)}
    if len(subject_hashes) != 1:
        raise ValueError("w95 subject hash determinism mismatch")
    subject_hash = next(iter(subject_hashes))
    if not subject_hash.startswith("sha256:"):
        raise ValueError("w95 subject hash format mismatch")
    if str(golden.get("schema", "")).strip() != "ddn.gogae9.w95.cert_report.v1":
        raise ValueError("w95 golden schema mismatch")
    if not bool(golden.get("overall_pass", False)):
        raise ValueError("w95 overall_pass mismatch")
    if not bool(golden.get("deterministic_subject_hash", False)):
        raise ValueError("w95 deterministic_subject_hash mismatch")
    if not bool(golden.get("tamper_detection", False)):
        raise ValueError("w95 tamper_detection mismatch")
    golden_rows = golden.get("cases")
    if not isinstance(golden_rows, list) or len(golden_rows) != 2:
        raise ValueError("w95 golden cases count mismatch")
    if any(str(row.get("expected_subject_hash", "")).strip() != subject_hash for row in golden_rows if isinstance(row, dict)):
        raise ValueError("w95 golden subject hash mismatch")


def validate_bridge_pack() -> None:
    cases = load_json(AGE4_BRIDGE_CASES)
    golden = load_json(AGE4_BRIDGE_GOLDEN)
    if str(cases.get("schema", "")).strip() != "ddn.gogae9.w95.cert_cases.v1":
        raise ValueError("bridge cases schema mismatch")
    case_rows = cases.get("cases")
    if not isinstance(case_rows, list) or len(case_rows) != 4:
        raise ValueError("bridge cases count mismatch")
    expected_by_id = {
        str(row.get("id", "")).strip(): str(row.get("expected_subject_hash", "")).strip()
        for row in case_rows
        if isinstance(row, dict)
    }
    clean_hash = "sha256:5f60345843e9bfd49ad1bc1a49d5229c97aa2b9c5c4f560a970288674f53f1a0"
    abort_hash = "sha256:371b3f75b60c6e1c54fb27cf57b813616250cb01945cc0d878a28cf0a6df2598"
    if expected_by_id.get("c01_clean_verify_pass") != clean_hash:
        raise ValueError("bridge clean pass subject hash mismatch")
    if expected_by_id.get("c02_clean_verify_fail_tamper") != clean_hash:
        raise ValueError("bridge clean tamper subject hash mismatch")
    if expected_by_id.get("c03_abort_verify_pass") != abort_hash:
        raise ValueError("bridge abort pass subject hash mismatch")
    if expected_by_id.get("c04_abort_verify_fail_tamper") != abort_hash:
        raise ValueError("bridge abort tamper subject hash mismatch")
    if str(golden.get("schema", "")).strip() != "ddn.gogae9.w95.cert_report.v1":
        raise ValueError("bridge golden schema mismatch")
    if not bool(golden.get("overall_pass", False)):
        raise ValueError("bridge overall_pass mismatch")
    if not bool(golden.get("deterministic_subject_hash", False)):
        raise ValueError("bridge deterministic_subject_hash mismatch")
    if not bool(golden.get("tamper_detection", False)):
        raise ValueError("bridge tamper_detection mismatch")
    golden_rows = golden.get("cases")
    if not isinstance(golden_rows, list) or len(golden_rows) != 4:
        raise ValueError("bridge golden cases count mismatch")


def main() -> int:
    if not README_PATH.exists():
        return fail(f"missing readme: {README_PATH}")
    text = README_PATH.read_text(encoding="utf-8")
    for snippet in README_SNIPPETS:
        if snippet not in text:
            return fail(f"missing snippet: {snippet}")
    try:
        ensure_pointers(AGE4_PROOF_README)
        ensure_pointers(AGE4_BRIDGE_README)
        ensure_pointers(W95_CERT_README)
        validate_age4_proof(AGE4_CLEAN_PROOF, verified=True, contract_diag_count=0, label="age4 clean")
        validate_age4_proof(AGE4_ABORT_PROOF, verified=False, contract_diag_count=1, label="age4 abort")
        validate_bridge_pack()
        validate_w95()
    except ValueError as exc:
        return fail(str(exc))
    print("[proof-artifact-certificate-contract-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

