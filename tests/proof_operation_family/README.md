# Proof Operation Family

## Stable Contract

- 목적:
  - `AGE1 immediate proof`와 `AGE4 proof replay`가 `proof_check`, `solver_check`, `solver_search(counterexample/solve)`를 proof artifact 안에서 어떤 family로 보존하는지 한 번에 확인한다.
  - family literal은 `proof_check`, `solver_check`, `solver_search`로 고정하고, `solver_search` operation literal은 `counterexample`, `solve` 두 값으로 고정한다.
- pack 계약:
  - `pack/proof_operation_family_v1/README.md`
  - `pack/proof_operation_family_contract_v1/README.md`
- 대상 surface:
  - `tests/age1_immediate_proof_operation/README.md`
  - `pack/age1_immediate_proof_smoke_v1/expected/proof.detjson`
  - `pack/age1_immediate_proof_solver_search_solve_smoke_v1/expected/proof.detjson`
  - `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1/expected/proof.detjson`
  - `pack/age4_proof_case_analysis_solver_open_search_replay_v1/expected/proof.detjson`
  - `pack/age4_proof_solver_open_replay_v1/expected/proof.detjson`
  - `pack/age4_proof_solver_search_replay_v1/expected/proof.detjson`
  - `tests/proof_solver_operation_family/README.md`
  - `tests/proof_case_analysis_completion_parity/README.md`
- selftest:
  - `python tests/run_proof_operation_family_selftest.py`
  - `proof_operation_family_selftest`
  - `python tests/run_proof_operation_family_contract_selftest.py`
  - `proof_operation_family_contract_selftest`

## Matrix

| surface | proof_check | solver_check | solver_search(counterexample) | solver_search(solve) | quantifier |
| --- | --- | --- | --- | --- | --- |
| `AGE1 immediate proof` | 분산 증거와 mixed 증거 모두 `1` | `pack/age1_immediate_proof_smoke_v1`, `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1` | 분산 증거는 `pack/age1_immediate_proof_smoke_v1`, mixed 증거는 `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1` | 분산 증거는 `pack/age1_immediate_proof_solver_search_solve_smoke_v1`, mixed 증거는 `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1` | 모두 `exists_unique` |
| `AGE4 replay` | 모두 `0` | `pack/age4_proof_case_analysis_solver_open_search_replay_v1` 안에 포함 | `pack/age4_proof_case_analysis_solver_open_search_replay_v1` 안에 포함 | `pack/age4_proof_case_analysis_solver_open_search_replay_v1` 안에 포함 | 모두 `exists_unique` |

## Consumer Surface

- `tests/age1_immediate_proof_operation/README.md`
- `pack/age1_immediate_proof_smoke_v1/README.md`
- `pack/age1_immediate_proof_solver_search_solve_smoke_v1/README.md`
- `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1/README.md`
- `pack/age4_proof_case_analysis_solver_open_search_replay_v1/README.md`
- `pack/age4_proof_solver_open_replay_v1/README.md`
- `pack/age4_proof_solver_search_replay_v1/README.md`
- `tests/proof_solver_operation_family/README.md`
- `tests/proof_case_analysis_completion_parity/README.md`
- `tests/proof_family/README.md`
- `python tests/run_proof_operation_family_selftest.py`
- `python tests/run_proof_operation_family_contract_selftest.py`
- `python tests/run_proof_family_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
