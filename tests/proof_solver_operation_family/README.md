# Proof Solver Operation Family

## Stable Contract

- 목적:
  - `AGE1 immediate proof`와 `AGE4 proof replay`가 `check`, `counterexample`, `solve` 세 operation family를 proof artifact 안에서 어떻게 보존하는지 한 번에 확인한다.
  - operation literal은 `check`, `counterexample`, `solve` 세 값으로 고정한다.
- pack 계약:
  - `pack/proof_solver_operation_family_v1/README.md`
- 대상 surface:
  - `tests/age1_immediate_proof_operation/README.md`
  - `pack/age1_immediate_proof_smoke_v1/expected/proof.detjson`
  - `pack/age1_immediate_proof_solver_search_solve_smoke_v1/expected/proof.detjson`
  - `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1/expected/proof.detjson`
  - `pack/age4_proof_case_analysis_solver_open_search_replay_v1/expected/proof.detjson`
  - `pack/age4_proof_solver_open_replay_v1/expected/proof.detjson`
  - `pack/age4_proof_solver_search_replay_v1/expected/proof.detjson`
  - `tests/proof_solver_search_operation_parity/README.md`
  - `tests/proof_case_analysis_completion_parity/README.md`
- selftest:
  - `python tests/run_proof_solver_operation_family_selftest.py`
  - `proof_solver_operation_family_selftest`

## Matrix

| surface | check | counterexample | solve | quantifier | proof_check |
| --- | --- | --- | --- | --- | --- |
| `AGE1 immediate proof` | 분산 증거는 `pack/age1_immediate_proof_smoke_v1`, mixed 증거는 `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1` 안에 포함 | 분산 증거는 `pack/age1_immediate_proof_smoke_v1`, mixed 증거는 `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1` 안에 포함 | 분산 증거는 `pack/age1_immediate_proof_solver_search_solve_smoke_v1`, mixed 증거는 `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1` 안에 포함 | 모두 `exists_unique` | 모두 `1` |
| `AGE4 replay` | `pack/age4_proof_case_analysis_solver_open_search_replay_v1` 안에 포함 | `pack/age4_proof_case_analysis_solver_open_search_replay_v1` 안에 포함 | `pack/age4_proof_case_analysis_solver_open_search_replay_v1` 안에 포함 | 모두 `exists_unique` | 모두 `0` |

## Consumer Surface

- `tests/age1_immediate_proof_operation/README.md`
- `pack/age1_immediate_proof_smoke_v1/README.md`
- `pack/age1_immediate_proof_solver_search_solve_smoke_v1/README.md`
- `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1/README.md`
- `pack/age4_proof_case_analysis_solver_open_search_replay_v1/README.md`
- `pack/age4_proof_solver_open_replay_v1/README.md`
- `pack/age4_proof_solver_search_replay_v1/README.md`
- `tests/proof_solver_search_operation_parity/README.md`
- `tests/proof_case_analysis_completion_parity/README.md`
- `python tests/run_proof_solver_operation_family_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
- 상위 proof operation family:
  - `tests/proof_operation_family/README.md`
  - `python tests/run_proof_operation_family_selftest.py`
