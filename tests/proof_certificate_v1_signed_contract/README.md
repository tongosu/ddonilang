# Proof Certificate V1 Signed Contract

## Stable Contract

- 목적:
  - `proof_certificate_v1`의 실제 emit line을 `runtime emit -> signed emit -> promotion` 3단 contract로 한 번에 묶는다.
  - 이 문서는 하위 selftest를 대체하지 않고, runtime sidecar와 signed bundle, promotion surface가 같은 상위 line을 가리키는지만 확인한다.
- pack 계약:
  - `pack/proof_certificate_v1_signed_contract_v1/README.md`
- 대상 surface:
  - `pack/age4_proof_detjson_smoke_v1/README.md`
  - `pack/proof_certificate_v1_signed_emit_profiles_v1/README.md`
  - `tests/proof_certificate_v1_runtime_emit/README.md`
  - `tests/proof_certificate_v1_signed_emit/README.md`
  - `tests/proof_certificate_v1_signed_emit_profiles/README.md`
  - `tests/proof_certificate_v1_verify_bundle/README.md`
  - `tests/proof_certificate_v1_verify_report/README.md`
  - `tests/proof_certificate_v1_consumer_contract/README.md`
  - `tests/proof_certificate_v1_promotion/README.md`
  - `tests/proof_certificate_v1_family/README.md`
  - `python tests/run_proof_certificate_v1_runtime_emit_selftest.py`
  - `python tests/run_proof_certificate_v1_signed_emit_selftest.py`
  - `python tests/run_proof_certificate_v1_signed_emit_profile_selftest.py`
  - `python tests/run_proof_certificate_v1_verify_bundle_selftest.py`
  - `python tests/run_proof_certificate_v1_verify_report_selftest.py`
  - `python tests/run_proof_certificate_v1_consumer_contract_selftest.py`
  - `python tests/run_proof_certificate_v1_promotion_selftest.py`
  - `python tests/run_proof_certificate_v1_family_selftest.py`
  - `python tests/run_proof_certificate_v1_signed_contract_selftest.py`
  - `proof_certificate_v1_signed_contract_selftest`

## Contract Chain

| layer | schema/surface | contract |
| --- | --- | --- |
| runtime emit | `ddn.proof_certificate_v1_runtime_candidate.v1` / `ddn.proof_certificate_v1_runtime_draft_artifact.v1` | `--proof-out`만으로 runtime candidate/artifact sidecar가 나온다 |
| signed emit | `ddn.cert_manifest.v1` / `ddn.proof_certificate_v1.v1` | `--proof-out --proof-cert-key`로 cert manifest와 signed proof bundle이 runtime line 위에 추가된다 |
| promotion | `proof_certificate_v1` 최종 후보 line | runtime/signed surface가 draft/promotion 문서선과 직접 연결된다 |

## Consumer Surface

- `pack/age4_proof_detjson_smoke_v1/README.md`
- `pack/proof_certificate_v1_signed_emit_profiles_v1/README.md`
- `tests/proof_certificate_v1_runtime_emit/README.md`
- `tests/proof_certificate_v1_signed_emit/README.md`
- `tests/proof_certificate_v1_signed_emit_profiles/README.md`
- `tests/proof_certificate_v1_verify_bundle/README.md`
- `tests/proof_certificate_v1_verify_report/README.md`
- `tests/proof_certificate_v1_verify_report_digest_contract/README.md`
- `tests/proof_certificate_v1_consumer_contract/README.md`
- `tests/proof_certificate_v1_promotion/README.md`
- `tests/proof_certificate_v1_family/README.md`
- `python tests/run_proof_certificate_v1_runtime_emit_selftest.py`
- `python tests/run_proof_certificate_v1_signed_emit_selftest.py`
- `python tests/run_proof_certificate_v1_signed_emit_profile_selftest.py`
- `python tests/run_proof_certificate_v1_verify_bundle_selftest.py`
- `python tests/run_proof_certificate_v1_verify_report_selftest.py`
- `python tests/run_proof_certificate_v1_verify_report_digest_contract_selftest.py`
- `python tests/run_proof_certificate_v1_consumer_contract_selftest.py`
- `python tests/run_proof_certificate_v1_consumer_transport_contract_selftest.py`
- `python tests/run_proof_certificate_v1_promotion_selftest.py`
- `python tests/run_proof_certificate_v1_family_selftest.py`
- `python tests/run_proof_certificate_v1_signed_contract_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
