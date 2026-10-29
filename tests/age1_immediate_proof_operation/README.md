# AGE1 Immediate Proof Operation Matrix

## Stable Contract

- 목적:
  - `AGE1 immediate proof`에서 `proof_check` + `exists_unique`를 공통축으로 두고 `case_analysis`, `completion`, `check`, `counterexample`, `solve` 조합이 어떤 pack에서 봉인되는지 한 번에 확인한다.
- 대상 pack:
  - `pack/age1_immediate_proof_smoke_v1`
  - `pack/age1_immediate_proof_solver_search_solve_smoke_v1`
  - `pack/age1_immediate_proof_case_analysis_smoke_v1`
  - `pack/age1_immediate_proof_case_analysis_solver_open_smoke_v1`
  - `pack/age1_immediate_proof_case_analysis_solver_search_smoke_v1`
  - `pack/age1_immediate_proof_case_analysis_solver_search_solve_smoke_v1`
  - `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1`
  - `pack/age1_immediate_proof_case_analysis_else_solver_open_search_smoke_v1`
- selftest:
  - `python tests/run_age1_immediate_proof_operation_matrix_selftest.py`
  - `age1_immediate_proof_operation_matrix_selftest`
  - `python tests/run_age1_immediate_proof_operation_contract_selftest.py`
  - `age1_immediate_proof_operation_contract_selftest`
  - `python tests/run_age1_immediate_proof_operation_contract_summary_selftest.py`
  - `age1_immediate_proof_operation_contract_summary_selftest`
- bundle `checks_text`:
  - `operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family`
- progress schema:
  - `ddn.ci.age1_immediate_proof_operation_contract_selftest.progress.v1`

## Stable Transport Contract

- bundle `checks_text`:
  - `operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family`
- sanity steps:
  - `age1_immediate_proof_operation_contract_selftest`
  - `age1_immediate_proof_operation_contract_summary_selftest`
  - `age1_immediate_proof_operation_transport_contract_summary_selftest`
  - `ci_gate_summary_line_check_selftest`
- upstream raw field:
  - `age5_full_real_age1_immediate_proof_operation_contract_selftest_*`
- compact token:
  - `age5_age1_immediate_proof_operation_contract_checks_text`
- direct/consumer surface:
  - `ci_gate_summary_line`
  - aggregate preview summary
  - aggregate status line
  - final status line
  - `ci_gate_result`
  - `ci_fail_brief.txt`
  - `ci_fail_triage.detjson`
  - `ci_gate_report_index`

## Matrix

| pack | proof_check | quantifier | case_analysis | completion | check | counterexample | solve |
| --- | --- | --- | --- | --- | --- | --- |
| `pack/age1_immediate_proof_smoke_v1` | `1` | `exists_unique` | `0` | `-` | `1` | `1` | `0` |
| `pack/age1_immediate_proof_solver_search_solve_smoke_v1` | `1` | `exists_unique` | `0` | `-` | `0` | `0` | `1` |
| `pack/age1_immediate_proof_case_analysis_smoke_v1` | `1` | `exists_unique` | `1` | `exhaustive` | `0` | `0` | `0` |
| `pack/age1_immediate_proof_case_analysis_solver_open_smoke_v1` | `1` | `exists_unique` | `1` | `exhaustive` | `1` | `0` | `0` |
| `pack/age1_immediate_proof_case_analysis_solver_search_smoke_v1` | `1` | `exists_unique` | `1` | `exhaustive` | `0` | `1` | `0` |
| `pack/age1_immediate_proof_case_analysis_solver_search_solve_smoke_v1` | `1` | `exists_unique` | `1` | `exhaustive` | `0` | `0` | `1` |
| `pack/age1_immediate_proof_case_analysis_solver_open_search_smoke_v1` | `1` | `exists_unique` | `1` | `exhaustive` | `1` | `1` | `1` |
| `pack/age1_immediate_proof_case_analysis_else_solver_open_search_smoke_v1` | `1` | `exists_unique` | `1` | `else` | `1` | `1` | `1` |

## Consumer Surface

- `pack/*/README.md`
- `pack/*/expected/proof.detjson`
- `python tests/run_age1_immediate_proof_operation_matrix_selftest.py`
- `python tests/run_age1_immediate_proof_operation_contract_selftest.py`
- `python tests/run_age1_immediate_proof_operation_contract_summary_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
- `python tests/run_age5_close_combined_report_contract_selftest.py`
- `python tests/run_ci_aggregate_age5_child_summary_age1_immediate_proof_operation_selftest.py`
- `python tests/run_ci_aggregate_gate_age5_diagnostics_check.py`
- `python tests/run_ci_aggregate_status_line_selftest.py`
- `python tests/run_ci_gate_final_status_line_selftest.py`
- `python tests/run_ci_gate_result_check_selftest.py`
- `python tests/run_ci_gate_outputs_consistency_check_selftest.py`
- `python tests/run_ci_final_line_emitter_check.py`
- `python tests/run_ci_gate_report_index_check_selftest.py`
- `python tests/run_age1_immediate_proof_operation_transport_contract_summary_selftest.py`
- completion parity:
  - `tests/proof_case_analysis_completion_parity/README.md`
  - `python tests/run_proof_case_analysis_completion_parity_selftest.py`
- 하위 search matrix:
  - `tests/age1_immediate_proof_solver_search/README.md`
  - `python tests/run_age1_immediate_proof_solver_search_matrix_selftest.py`
- 상위 family 인덱스:
  - `tests/proof_solver_operation_family/README.md`
  - `python tests/run_proof_solver_operation_family_selftest.py`
  - `tests/proof_operation_family/README.md`
  - `python tests/run_proof_operation_family_selftest.py`

## 참고

- matrix selftest script: `run_age1_immediate_proof_operation_matrix_selftest.py`
- bundle selftest script: `run_age1_immediate_proof_operation_contract_selftest.py`
- summary selftest script: `run_age1_immediate_proof_operation_contract_summary_selftest.py`
- transport summary selftest script: `run_age1_immediate_proof_operation_transport_contract_summary_selftest.py`
- age5 close/aggregate preview transport fields:
  - `age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks`
  - `age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks`
  - `age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text`
  - `age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe`
  - `age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe`
  - `age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present`
- aggregate/final/result compact tokens:
  - `age5_age1_immediate_proof_operation_contract_completed`
  - `age5_age1_immediate_proof_operation_contract_total`
  - `age5_age1_immediate_proof_operation_contract_checks_text`
  - `age5_age1_immediate_proof_operation_contract_current_probe`
  - `age5_age1_immediate_proof_operation_contract_last_completed_probe`
  - `age5_age1_immediate_proof_operation_contract_progress`
- failure handoff/report-index consumer surface:
  - `tools/scripts/emit_ci_final_line.py`
  - `tests/run_ci_final_line_emitter_check.py`
  - `tests/run_ci_gate_report_index_check.py`
  - `tests/run_ci_gate_report_index_check_selftest.py`
  - `ci_fail_brief.txt`
  - `ci_fail_triage.detjson`
  - `ci_gate_report_index.detjson`
