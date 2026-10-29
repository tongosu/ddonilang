# Proof Solver Search Operation Parity

## Stable Contract

- 목적:
  - `AGE1 immediate proof`와 `AGE4 proof replay`가 `solver_search(counterexample/solve)` operation pair를 같은 proof artifact 계열로 보존하는지 한 번에 확인한다.
  - operation literal은 `counterexample`, `solve` 두 값으로 고정한다.
- pack 계약:
  - `pack/proof_solver_search_operation_parity_v1/README.md`
- 대상 surface:
  - `tests/age1_immediate_proof_solver_search/README.md`
  - `pack/age1_immediate_proof_smoke_v1/expected/proof.detjson`
  - `pack/age1_immediate_proof_solver_search_solve_smoke_v1/expected/proof.detjson`
  - `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1/expected/proof.detjson`
  - `pack/age4_proof_solver_search_replay_v1/expected/proof.detjson`
- selftest:
  - `python tests/run_proof_solver_search_operation_parity_selftest.py`
  - `proof_solver_search_operation_parity_selftest`

## Matrix

| surface | counterexample | solve | quantifier | proof_check | solver_check |
| --- | --- | --- | --- | --- | --- |
| `AGE1 immediate proof` | 분산 증거는 `pack/age1_immediate_proof_smoke_v1`, mixed 증거는 `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1` 안에 포함 | 분산 증거는 `pack/age1_immediate_proof_solver_search_solve_smoke_v1`, mixed 증거는 `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1` 안에 포함 | 모두 `exists_unique` | 모두 `1` | mixed surface와 counterexample surface에서 `1` |
| `AGE4 replay` | `pack/age4_proof_solver_search_replay_v1` 안에 같이 포함 | `pack/age4_proof_solver_search_replay_v1` 안에 같이 포함 | `exists_unique` | `0` | `0` |

## Consumer Surface

- `pack/age4_proof_solver_search_replay_v1/README.md`
- `tests/age1_immediate_proof_solver_search/README.md`
- `python tests/run_proof_solver_search_operation_parity_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
- 상위 family 인덱스:
  - `tests/proof_solver_operation_family/README.md`
  - `python tests/run_proof_solver_operation_family_selftest.py`
- 최상위 proof family:
  - `tests/proof_operation_family/README.md`
  - `python tests/run_proof_operation_family_selftest.py`
