# Proof Certificate V1 Verify Report

## Stable Contract

- 목적:
  - `teul-cli cert verify-proof-certificate --in <bundle> --out <report>`가 verify 결과를 detjson report로 남기는 현재 consumer artifact contract를 고정한다.
  - 이 문서는 signed bundle verify가 stdout token만이 아니라 저장 가능한 report surface도 갖는지 확인한다.
- 대상 surface:
  - `tools/teul-cli/src/cli/cert.rs`
  - `tools/teul-cli/src/main.rs`
  - `tests/proof_certificate_v1_verify_bundle/README.md`
  - `tests/proof_certificate_v1_verify_report_digest_contract/README.md`
  - `tests/proof_certificate_v1_signed_contract/README.md`
  - `tests/proof_certificate_v1_family/README.md`
  - `python tests/run_proof_certificate_v1_verify_report_selftest.py`
  - `python tests/run_proof_certificate_v1_verify_report_digest_contract_selftest.py`
  - `proof_certificate_v1_verify_report_selftest`

## Verify Report

- schema:
  - `ddn.proof_certificate_v1.verify_report.v1`
- required fields:
  - `schema`
  - `ok`
  - `input_path`
  - `source_hash`
  - `source_provenance`
  - `profile`
  - `verified`
  - `contract_diag_count`
  - `source_proof_path`
  - `source_proof_schema`
  - `source_proof_kind`
  - `proof_subject_hash`
  - `canonical_body_hash`
  - `proof_runtime_hash`
  - `solver_translation_hash`
  - `state_hash`
  - `trace_hash`
  - `cert_manifest_schema`
  - `cert_algo`
  - `cert_subject_hash`
  - `cert_pubkey`
  - `cert_signature`

## Source Provenance

- `source_hash`
  - verify 입력 bundle detjson raw bytes의 `sha256`
- `source_provenance`
  - schema: `ddn.proof_certificate_v1.verify_report_source_provenance.v1`
  - source_kind: `proof_certificate_bundle.v1`
  - required fields:
    - `input_bundle_file`
    - `input_bundle_hash`
    - `source_proof_file`
    - `source_proof_hash`

## Consumer Surface

- `pack/age4_proof_detjson_smoke_v1/README.md`
- `tests/proof_certificate_v1_consumer_contract/README.md`
- `tests/proof_certificate_v1_verify_report_digest_contract/README.md`
- `tests/proof_certificate_v1_verify_bundle/README.md`
- `tests/proof_certificate_v1_signed_contract/README.md`
- `tests/proof_certificate_v1_promotion/README.md`
- `tests/proof_certificate_v1_family/README.md`
- `python tests/run_proof_certificate_v1_verify_report_selftest.py`
- `python tests/run_proof_certificate_v1_verify_report_digest_contract_selftest.py`
- `python tests/run_proof_certificate_v1_consumer_contract_selftest.py`
- `python tests/run_proof_certificate_v1_consumer_transport_contract_selftest.py`
- `python tests/run_proof_certificate_v1_signed_contract_selftest.py`
- `python tests/run_proof_certificate_v1_family_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
