# proof_certificate_candidate_layers_v1

`proof_certificate` candidate layers selftest의 stdout surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_proof_certificate_candidate_layers_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- selftest는 candidate manifest fixture pair가 `shared shell(7)`과 `state delta(6)`로 분해되고 다시 합쳐짐을 검증해야 한다.

## 구성
- `expected/candidate_layers.stdout.txt`
