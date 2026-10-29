# Proof Case Analysis Completion Parity

## Stable Contract

- 목적:
  - `AGE1 immediate proof`와 `AGE4 proof replay`가 `case_analysis` completion literal을 `exhaustive`, `else` 두 값으로 어떻게 proof artifact에 봉인하는지 한 번에 확인한다.
  - `completion` literal은 `exhaustive`, `else` 두 값으로 고정한다.
- pack 계약:
  - `pack/proof_case_analysis_completion_parity_v1/README.md`
- 대상 surface:
  - `tests/age1_immediate_proof_operation/README.md`
  - `tests/age4_proof_quantifier_case_analysis/README.md`
  - `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1/expected/proof.detjson`
  - `pack/age1_immediate_proof_case_analysis_else_solver_open_search_smoke_v1/expected/proof.detjson`
  - `pack/age4_proof_case_analysis_solver_open_search_replay_v1/expected/proof.detjson`
  - `pack/age4_proof_case_analysis_else_solver_open_search_replay_v1/expected/proof.detjson`
- selftest:
  - `python tests/run_proof_case_analysis_completion_parity_selftest.py`
  - `proof_case_analysis_completion_parity_selftest`

## Matrix

| surface | completion | quantifier | proof_check | solver_check | counterexample | solve |
| --- | --- | --- | --- | --- | --- | --- |
| `AGE1 immediate proof` mixed exhaustive | `exhaustive` | `exists_unique` | `1` | `1` | `1` | `1` |
| `AGE1 immediate proof` mixed else | `else` | `exists_unique` | `1` | `1` | `1` | `1` |
| `AGE4 replay` mixed exhaustive | `exhaustive` | `exists_unique` | `0` | `1` | `1` | `1` |
| `AGE4 replay` mixed else | `else` | `exists_unique` | `0` | `1` | `1` | `1` |

## Consumer Surface

- `tests/age1_immediate_proof_operation/README.md`
- `tests/age4_proof_quantifier_case_analysis/README.md`
- `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1/README.md`
- `pack/age1_immediate_proof_case_analysis_else_solver_open_search_smoke_v1/README.md`
- `pack/age4_proof_case_analysis_solver_open_search_replay_v1/README.md`
- `pack/age4_proof_case_analysis_else_solver_open_search_replay_v1/README.md`
- `tests/proof_solver_operation_family/README.md`
- `tests/proof_operation_family/README.md`
- `python tests/run_proof_case_analysis_completion_parity_selftest.py`
- `python tests/run_proof_solver_operation_family_selftest.py`
- `python tests/run_proof_operation_family_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
