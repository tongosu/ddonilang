# proof_certificate_v1_schema_candidate_split_v1

`proof_certificate_v1` schema candidate split selftest의 stdout surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_proof_certificate_v1_schema_candidate_split_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- selftest는 flat candidate fixture pair를 `shared shell(9)`와 `state delta(7)`로 분리하고 다시 합칠 수 있음을 검증해야 한다.

## 구성
- `expected/schema_candidate_split.stdout.txt`
