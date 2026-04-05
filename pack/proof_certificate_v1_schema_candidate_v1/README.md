# proof_certificate_v1_schema_candidate_v1

`proof_certificate_v1` schema candidate selftest의 stdout surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_proof_certificate_v1_schema_candidate_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- selftest는 draft artifact에서 flat candidate fixture를 정확히 평탄화할 수 있음을 검증해야 한다.

## 구성
- `expected/schema_candidate.stdout.txt`
