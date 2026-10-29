# AGE1 Immediate Proof Solver Search Matrix

## Stable Contract

- 목적:
  - `AGE1 immediate proof`에서 `proof_check + exists_unique + solver_search` 조합이 `counterexample`, `solve`, `counterexample + case_analysis`, `solve + case_analysis`, `counterexample + solve + else case_analysis` 표면으로 봉인됐는지 한 번에 확인한다.
  - case completion literal은 `exhaustive`, `else`로 고정한다.
- 대상 pack:
  - `pack/age1_immediate_proof_smoke_v1`
  - `pack/age1_immediate_proof_solver_search_solve_smoke_v1`
  - `pack/age1_immediate_proof_case_analysis_solver_search_smoke_v1`
  - `pack/age1_immediate_proof_case_analysis_solver_search_solve_smoke_v1`
  - `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1`
  - `pack/age1_immediate_proof_case_analysis_else_solver_open_search_smoke_v1`
- selftest:
  - `python tests/run_age1_immediate_proof_solver_search_matrix_selftest.py`
  - `age1_immediate_proof_solver_search_matrix_selftest`

## Matrix

| pack | operation | proof_check | quantifier | solver_search | extra |
| --- | --- | --- | --- | --- | --- |
| `pack/age1_immediate_proof_smoke_v1` | `counterexample` | `1` | `exists_unique` | `counterexample` 1건 | `solver_check` 1건 포함 |
| `pack/age1_immediate_proof_solver_search_solve_smoke_v1` | `solve` | `1` | `exists_unique` | `solve` 1건 | `solver_check` 없음 |
| `pack/age1_immediate_proof_case_analysis_solver_search_smoke_v1` | `counterexample` | `1` | `exists_unique` | `counterexample` 1건 | `case_analysis` 1건 포함 |
| `pack/age1_immediate_proof_case_analysis_solver_search_solve_smoke_v1` | `solve` | `1` | `exists_unique` | `solve` 1건 | `case_analysis` 1건 포함 |
| `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1` | `counterexample + solve` | `1` | `exists_unique` | `counterexample`, `solve` 각 1건 | `case_analysis` 1건 + `solver_check` 1건 포함 |
| `pack/age1_immediate_proof_case_analysis_else_solver_open_search_smoke_v1` | `counterexample + solve` | `1` | `exists_unique` | `counterexample`, `solve` 각 1건 | `case_analysis(else)` 1건 + `solver_check` 1건 포함 |

## Consumer Surface

- `pack/*/README.md`
- `pack/*/expected/proof.detjson`
- `python tests/run_age1_immediate_proof_solver_search_matrix_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
- 상위 parity 인덱스:
  - `tests/proof_solver_search_operation_parity/README.md`
  - `python tests/run_proof_solver_search_operation_parity_selftest.py`
