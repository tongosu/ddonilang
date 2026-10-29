# Proof Certificate V1 Signed Emit

## Stable Contract

- 목적:
  - `teul-cli run --proof-out --proof-cert-key`가 runtime candidate/artifact 위에 실제 cert manifest와 signed `proof_certificate_v1` 번들을 함께 쓰는 최소 signed emit 경로를 고정한다.
  - 이 문서는 최종 정본 스키마 논의를 멈추지 않고, 현재 구현이 실제 서명된 bundle까지 만들 수 있음을 검증한다.
- 대상 surface:
  - `pack/age4_proof_detjson_smoke_v1/input.ddn`
  - `pack/age4_proof_detjson_smoke_v1/input_abort.ddn`
  - `pack/proof_certificate_v1_signed_emit_v1/README.md`
  - `tools/teul-cli/src/cli/cert.rs`
  - `tools/teul-cli/src/cli/run.rs`
  - `tests/proof_certificate_v1_signed_emit_profiles/README.md`
  - `tests/proof_certificate_v1_runtime_emit/README.md`
  - `tests/proof_certificate_v1_promotion/README.md`
  - `tests/proof_certificate_v1_family/README.md`
  - `python tests/run_proof_certificate_v1_signed_emit_selftest.py`
  - `python tests/run_proof_certificate_v1_signed_emit_pack_check.py`
  - `proof_certificate_v1_signed_emit_selftest`

## Signed Sidecars

- cert manifest:
  - schema: `ddn.cert_manifest.v1`
  - naming: `<proof-out stem>.cert_manifest.detjson`
- proof certificate bundle:
  - schema: `ddn.proof_certificate_v1.v1`
  - naming: `<proof-out stem>.proof_certificate_v1.detjson`

## Consumer Surface

- `pack/age4_proof_detjson_smoke_v1/README.md`
- `pack/proof_certificate_v1_signed_emit_v1/README.md`
- `tests/proof_certificate_v1_runtime_emit/README.md`
- `tests/proof_certificate_v1_signed_emit_profiles/README.md`
- `tests/proof_certificate_v1_signed_contract/README.md`
- `tests/proof_certificate_v1_promotion/README.md`
- `tests/proof_certificate_v1_family/README.md`
- `python tests/run_proof_certificate_v1_signed_emit_selftest.py`
- `python tests/run_proof_certificate_v1_signed_emit_pack_check.py`
- `python tests/run_proof_certificate_v1_signed_emit_profile_selftest.py`
- `python tests/run_proof_certificate_v1_signed_contract_selftest.py`
- `python tests/run_proof_certificate_v1_family_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
