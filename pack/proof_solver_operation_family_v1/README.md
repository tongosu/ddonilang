# proof_solver_operation_family_v1

`proof solver operation family` selftest의 stdout surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_proof_solver_operation_family_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- selftest는 AGE1 immediate proof와 AGE4 replay에서 `check/counterexample/solve` operation family 보존이 유지됨을 검증해야 한다.

## 구성
- `expected/proof_solver_operation_family.stdout.txt`
