# proof_certificate_candidate_profile_split_v1

`proof_certificate` candidate profile split selftest의 stdout surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_proof_certificate_candidate_profile_split_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- selftest는 candidate manifest fixture pair가 `shared profile(7)`과 `state profile(6)`로 자연스럽게 나뉨을 검증해야 한다.

## 구성
- `expected/candidate_profile_split.stdout.txt`
