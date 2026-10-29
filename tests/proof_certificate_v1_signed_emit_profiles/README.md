# Proof Certificate V1 Signed Emit Profiles

## Stable Contract

- 목적:
  - `teul-cli run --proof-out --proof-cert-key`의 signed emit가 `clean/abort` 두 profile에서 모두 같은 file set을 쓰는지 고정한다.
  - 이 문서는 clean proof만이 아니라 abort proof도 cert manifest와 signed `proof_certificate_v1` bundle을 남긴다는 현재 구현을 검증한다.
- 대상 surface:
  - `pack/age4_proof_detjson_smoke_v1/input.ddn`
  - `pack/age4_proof_detjson_smoke_v1/input_abort.ddn`
  - `pack/proof_certificate_v1_signed_emit_profiles_v1/README.md`
  - `tests/proof_certificate_v1_signed_emit/README.md`
  - `tests/proof_certificate_v1_signed_contract/README.md`
  - `tests/proof_certificate_v1_family/README.md`
  - `python tests/run_proof_certificate_v1_signed_emit_profile_selftest.py`
  - `python tests/run_proof_certificate_v1_signed_emit_profiles_pack_check.py`
  - `proof_certificate_v1_signed_emit_profile_selftest`

## Profile Matrix

| profile | verified | contract_diag_count | required sidecars |
| --- | --- | --- | --- |
| clean | `true` | `0` | `cert_manifest`, `proof_certificate_v1` |
| abort | `false` | `1` | `cert_manifest`, `proof_certificate_v1` |

## Consumer Surface

- `pack/age4_proof_detjson_smoke_v1/README.md`
- `pack/proof_certificate_v1_signed_emit_profiles_v1/README.md`
- `tests/proof_certificate_v1_signed_emit/README.md`
- `tests/proof_certificate_v1_consumer_contract/README.md`
- `tests/proof_certificate_v1_verify_bundle/README.md`
- `tests/proof_certificate_v1_signed_contract/README.md`
- `tests/proof_certificate_v1_promotion/README.md`
- `tests/proof_certificate_v1_family/README.md`
- `python tests/run_proof_certificate_v1_signed_emit_profile_selftest.py`
- `python tests/run_proof_certificate_v1_signed_emit_profiles_pack_check.py`
- `python tests/run_proof_certificate_v1_consumer_contract_selftest.py`
- `python tests/run_proof_certificate_v1_consumer_transport_contract_selftest.py`
- `python tests/run_proof_certificate_v1_verify_bundle_selftest.py`
- `python tests/run_proof_certificate_v1_signed_contract_selftest.py`
- `python tests/run_proof_certificate_v1_family_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
