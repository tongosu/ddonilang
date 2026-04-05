# proof_operation_family_v1

`proof operation family` selftest의 stdout surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_proof_operation_family_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- selftest는 AGE1 immediate proof와 AGE4 replay가 `proof_check`, `solver_check`, `solver_search(counterexample/solve)` family를 어떻게 보존하는지 검증해야 한다.

## 구성
- `expected/proof_operation_family.stdout.txt`
