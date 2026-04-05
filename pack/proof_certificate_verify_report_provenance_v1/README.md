# proof_certificate_verify_report_provenance_v1

`proof certificate verify report`의 provenance를 clean/abort 두 profile에서 relative path 기준으로 고정하는 pack.

## 계약
- `ddn.proof_certificate_v1.verify_report.v1`가 clean/abort 두 profile 모두 결정적으로 재생성돼야 한다.
- report는 `source_hash/source_provenance`를 포함하고 bundle/proof 경로와 raw-byte `sha256`을 정확히 기록해야 한다.
- `keygen --seed` 고정값과 relative path 실행으로 `cert_pubkey/cert_signature`까지 expected JSON과 exact match여야 한다.
- tampered bundle verify는 반드시 실패하고 report를 남기지 않아야 한다.

## 구성
- `fixtures/input.ddn`
- `fixtures/input_abort.ddn`
- `expected/clean.verify.report.detjson`
- `expected/abort.verify.report.detjson`
