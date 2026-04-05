# proof_certificate_v1_signed_emit_profiles_v1

`proof_certificate_v1` signed emit profile surface를 clean/abort 두 profile의 relative path 실행으로 고정하는 pack.

## 계약
- `teul-cli run --proof-out --proof-cert-key`가 clean/abort 두 profile에서 결정적인 sidecar 3종을 재생성해야 한다.
- 각 profile마다 `proof.detjson`, `proof.cert_manifest.detjson`, `proof.proof_certificate_v1.detjson`의 raw-byte `sha256`이 expected와 같아야 한다.
- `entry`, `subject_path`, `source_proof_path`, `verified`, `contract_diag_count` 같은 핵심 semantic field도 expected와 정확히 맞아야 한다.

## 구성
- `fixtures/input.ddn`
- `fixtures/input_abort.ddn`
- `expected/clean.summary.detjson`
- `expected/abort.summary.detjson`
