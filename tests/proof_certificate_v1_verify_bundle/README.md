# Proof Certificate V1 Verify Bundle

## Stable Contract

- 목적:
  - `teul-cli cert verify-proof-certificate --in <bundle>`가 signed `proof_certificate_v1` bundle을 직접 읽어 cert/proof/runtime parity를 검증하는 최소 consumer surface를 고정한다.
  - 이 문서는 embedded cert manifest만 보는 것이 아니라, bundle과 source proof, runtime candidate/artifact의 일관성까지 함께 확인한다.
- 대상 surface:
  - `tools/teul-cli/src/cli/cert.rs`
  - `tools/teul-cli/src/main.rs`
  - `pack/age4_proof_detjson_smoke_v1/input.ddn`
  - `pack/age4_proof_detjson_smoke_v1/input_abort.ddn`
  - `tests/proof_certificate_v1_signed_emit_profiles/README.md`
  - `tests/proof_certificate_v1_signed_contract/README.md`
  - `tests/proof_certificate_v1_family/README.md`
  - `python tests/run_proof_certificate_v1_verify_bundle_selftest.py`
  - `proof_certificate_v1_verify_bundle_selftest`

## Verify Scope

- verify command:
  - `teul-cli cert verify-proof-certificate --in <proof_certificate_v1.detjson>`
- required parity:
  - embedded `ddn.cert_manifest.v1` signature가 유효해야 한다
  - bundle `proof_subject_hash/cert_pubkey/cert_signature`가 embedded cert manifest와 같아야 한다
  - embedded runtime candidate/artifact가 bundle top-level profile/verified/diag count와 같아야 한다
  - `source_proof_path`가 가리키는 실제 proof bytes/hash/schema/kind/hash fields가 bundle과 같아야 한다

## Consumer Surface

- `pack/age4_proof_detjson_smoke_v1/README.md`
- `pack/proof_certificate_verify_bundle_v1/README.md`
- `tests/proof_certificate_v1_consumer_contract/README.md`
- `tests/proof_certificate_v1_verify_report/README.md`
- `tests/proof_certificate_v1_signed_emit_profiles/README.md`
- `tests/proof_certificate_v1_signed_contract/README.md`
- `tests/proof_certificate_v1_promotion/README.md`
- `tests/proof_certificate_v1_family/README.md`
- `python tests/run_proof_certificate_v1_verify_bundle_selftest.py`
- `python tests/run_proof_certificate_verify_bundle_pack_check.py`
- `python tests/run_proof_certificate_v1_consumer_contract_selftest.py`
- `python tests/run_proof_certificate_v1_consumer_transport_contract_selftest.py`
- `python tests/run_proof_certificate_v1_verify_report_selftest.py`
- `python tests/run_proof_certificate_v1_signed_contract_selftest.py`
- `python tests/run_proof_certificate_v1_family_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
