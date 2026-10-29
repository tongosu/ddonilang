# AGE4 Proof Quantifier Case Analysis Matrix

## Stable Contract

- 목적:
  - `AGE4 proof replay`에서 양화(`forall`, `exists`, `exists_unique`)와 `case_analysis`, `solver_open(check)`, `solver_search(counterexample/solve)` 조합이 어떤 artifact에 봉인되는지 한 번에 확인한다.
  - operation literal은 `check`, `counterexample`, `solve`로 고정한다.
  - case completion literal은 `exhaustive`, `else`로 고정한다.
- 대상 surface:
  - `pack/age4_proof_solver_translation_smoke_v1/expected/proof.detjson`
  - `pack/age4_proof_case_analysis_forall_solver_open_search_replay_v1/expected/proof.detjson`
  - `pack/age4_proof_case_analysis_exists_solver_open_search_replay_v1/expected/proof.detjson`
  - `pack/age4_proof_case_analysis_solver_open_search_replay_v1/expected/proof.detjson`
  - `pack/age4_proof_case_analysis_else_solver_open_search_replay_v1/expected/proof.detjson`
- selftest:
  - `python tests/run_age4_proof_quantifier_case_analysis_selftest.py`
  - `age4_proof_quantifier_case_analysis_selftest`

## Matrix

| surface | quantifier | completion | case_analysis | check | counterexample | solve | proof_check |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `pack/age4_proof_solver_translation_smoke_v1` | `forall`, `exists`, `exists_unique` | `exhaustive`, `else` | `2` | `0` | `0` | `0` | `0` |
| `pack/age4_proof_case_analysis_forall_solver_open_search_replay_v1` | `forall` | `exhaustive` | `1` | `1` | `1` | `1` | `0` |
| `pack/age4_proof_case_analysis_exists_solver_open_search_replay_v1` | `exists` | `exhaustive` | `1` | `1` | `1` | `1` | `0` |
| `pack/age4_proof_case_analysis_solver_open_search_replay_v1` | `exists_unique` | `exhaustive` | `1` | `1` | `1` | `1` | `0` |
| `pack/age4_proof_case_analysis_else_solver_open_search_replay_v1` | `exists_unique` | `else` | `1` | `1` | `1` | `1` | `0` |

## Consumer Surface

- `pack/age4_proof_solver_translation_smoke_v1/README.md`
- `pack/age4_proof_case_analysis_forall_solver_open_search_replay_v1/README.md`
- `pack/age4_proof_case_analysis_exists_solver_open_search_replay_v1/README.md`
- `pack/age4_proof_case_analysis_solver_open_search_replay_v1/README.md`
- `pack/age4_proof_case_analysis_else_solver_open_search_replay_v1/README.md`
- `tests/proof_case_analysis_completion_parity/README.md`
- `python tests/run_age4_proof_quantifier_case_analysis_selftest.py`
- `python tests/run_proof_case_analysis_completion_parity_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
