# proof_certificate_v1_runtime_emit_v1

`proof_certificate_v1` runtime emit surface를 clean/abort 두 profile의 relative path 실행으로 고정하는 pack.

## 계약
- `teul-cli run --proof-out`가 clean/abort 두 profile에서 runtime sidecar 2종을 결정적으로 재생성해야 한다.
- 각 profile마다 `proof.detjson`, `proof_certificate_v1_candidate`, `proof_certificate_v1_draft_artifact`의 raw-byte `sha256`이 expected와 같아야 한다.
- `entry`, `source_proof_path`, `verified`, `contract_diag_count`, `state_hash`, `trace_hash` 같은 핵심 semantic field도 expected와 정확히 맞아야 한다.

## 구성
- `fixtures/input.ddn`
- `fixtures/input_abort.ddn`
- `expected/clean.summary.detjson`
- `expected/abort.summary.detjson`
