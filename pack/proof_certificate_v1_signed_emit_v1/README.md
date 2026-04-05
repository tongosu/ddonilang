# proof_certificate_v1_signed_emit_v1

`proof_certificate_v1` signed emit surface를 clean profile의 relative path 실행으로 고정하는 pack.

## 계약
- `teul-cli run --proof-out --proof-cert-key`가 clean profile에서 `cert_manifest`와 `proof_certificate_v1`를 결정적으로 재생성해야 한다.
- `proof.detjson`, `cert_manifest`, `proof_certificate_v1`의 raw-byte `sha256`이 expected와 같아야 한다.
- `subject_path`, `source_proof_path`, `verified`, `contract_diag_count`, `cert_pubkey`, `cert_signature` 같은 핵심 semantic field도 expected와 정확히 맞아야 한다.

## 구성
- `fixtures/input.ddn`
- `expected/clean.summary.detjson`
