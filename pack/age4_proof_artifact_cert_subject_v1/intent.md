# age4_proof_artifact_cert_subject_v1 intent

- 대상: `teul-cli cert keygen|sign|verify`
- subject:
  - `pack/age4_proof_detjson_smoke_v1/expected/clean.proof.detjson`
  - `pack/age4_proof_detjson_smoke_v1/expected/abort.proof.detjson`
- 핵심 계약:
  - proof artifact의 `verified=true/false`와 무관하게 cert subject 바이트 해시는 결정적이어야 한다.
  - 동일 proof artifact + 동일 seed -> 동일 `subject_hash/signature`
  - proof manifest signature를 1바이트 변조하면 verify가 실패해야 한다.
