# Gate0 Transport Family

## Stable Contract

- 목적:
  - `lang_runtime_family`, `gate0_runtime_family`, `gate0_family`가 이미 가진 `Stable Transport Contract` line을 한 단계 위에서 함께 읽는 상위 Gate0 transport family contract를 고정한다.
  - 이 문서는 하위 transport line의 세부 field를 다시 정의하지 않고, 세 transport line이 같은 Gate0 transport family를 이룬다는 점만 확인한다.
- 대상 surface:
  - `tests/lang_runtime_family/README.md`
  - `tests/gate0_runtime_family/README.md`
  - `tests/gate0_family/README.md`
- selftest:
  - `python tests/run_lang_runtime_family_selftest.py`
  - `python tests/run_gate0_runtime_family_selftest.py`
  - `python tests/run_gate0_family_selftest.py`
  - `python tests/run_gate0_transport_family_selftest.py`
  - `python tests/run_gate0_transport_family_contract_selftest.py`
  - `python tests/run_gate0_transport_family_contract_summary_selftest.py`
- sanity steps:
  - `lang_runtime_family_transport_contract_selftest`
  - `gate0_runtime_family_transport_contract_selftest`
  - `gate0_family_transport_contract_selftest`
  - `gate0_transport_family_selftest`
  - `gate0_transport_family_contract_selftest`
  - `gate0_transport_family_contract_summary_selftest`

## Stable Bundle Contract

- bundle `checks_text`:
  - `lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family`
- progress schema:
  - `ddn.ci.gate0_transport_family_contract_selftest.progress.v1`
- sanity steps:
  - `gate0_transport_family_selftest`
  - `gate0_transport_family_contract_selftest`
  - `gate0_transport_family_contract_summary_selftest`
- direct surface:
  - `ci_sanity_gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport bundle `checks_text`:
  - `family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index`
- progress schema:
  - `ddn.ci.gate0_transport_family_transport_contract_selftest.progress.v1`
- sanity steps:
  - `gate0_transport_family_transport_contract_selftest`
  - `gate0_transport_family_transport_contract_summary_selftest`
- direct surface:
  - `ci_sanity_gate stdout`
  - `*.progress.detjson`
- selftest:
  - `python tests/run_gate0_transport_family_transport_contract_selftest.py`
  - `python tests/run_gate0_transport_family_transport_contract_summary_selftest.py`
  - `python tests/run_ci_aggregate_age5_child_summary_gate0_transport_family_transport_selftest.py`

## Stable Upstream Transport Contract

- raw field:
  - `age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks`
  - `age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks`
  - `age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text`
  - `age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe`
  - `age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe`
  - `age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present`
- direct surface:
  - `ci_sanity_gate stdout/json-out`
  - `age5_close full-real report`
  - `aggregate preview summary`
- selftest:
  - `python tests/run_ci_aggregate_age5_child_summary_gate0_transport_family_transport_selftest.py`
  - `python tests/run_age5_close_combined_report_contract_selftest.py`
  - `python tests/run_ci_gate_summary_report_check_selftest.py`
  - `python tests/run_ci_aggregate_gate_age5_diagnostics_check.py`

## Stable Downstream Transport Contract

- compact token:
  - `age5_gate0_transport_family_transport_contract_completed`
  - `age5_gate0_transport_family_transport_contract_total`
  - `age5_gate0_transport_family_transport_contract_checks_text`
  - `age5_gate0_transport_family_transport_contract_current_probe`
  - `age5_gate0_transport_family_transport_contract_last_completed_probe`
  - `age5_gate0_transport_family_transport_contract_progress`
- direct surface:
  - `aggregate status line`
  - `final status line`
  - `ci_gate_result/summary compact`
  - `ci_fail_brief/triage`
  - `ci_gate_report_index`
- selftest:
  - `python tests/run_ci_aggregate_status_line_selftest.py`
  - `python tests/run_ci_gate_final_status_line_selftest.py`
  - `python tests/run_ci_gate_result_check_selftest.py`
  - `python tests/run_ci_gate_outputs_consistency_check_selftest.py`
  - `python tests/run_ci_gate_summary_line_check_selftest.py`
  - `python tests/run_ci_final_line_emitter_check.py`
  - `python tests/run_ci_gate_report_index_check_selftest.py`

## Stable Upstream Transport

- raw field:
  - `age5_full_real_gate0_transport_family_contract_selftest_completed_checks`
  - `age5_full_real_gate0_transport_family_contract_selftest_total_checks`
  - `age5_full_real_gate0_transport_family_contract_selftest_checks_text`
  - `age5_full_real_gate0_transport_family_contract_selftest_current_probe`
  - `age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe`
  - `age5_full_real_gate0_transport_family_contract_selftest_progress_present`
- direct surface:
  - `ci_sanity_gate stdout/json-out`
  - `age5_close full-real report`
  - `aggregate preview summary`
- selftest:
  - `python tests/run_ci_aggregate_age5_child_summary_gate0_transport_family_selftest.py`
  - `python tests/run_age5_close_combined_report_contract_selftest.py`
  - `python tests/run_ci_gate_summary_report_check_selftest.py`
  - `python tests/run_ci_aggregate_gate_age5_diagnostics_check.py`

## Stable Downstream Transport

- compact token:
  - `age5_gate0_transport_family_contract_completed`
  - `age5_gate0_transport_family_contract_total`
  - `age5_gate0_transport_family_contract_checks_text`
  - `age5_gate0_transport_family_contract_current_probe`
  - `age5_gate0_transport_family_contract_last_completed_probe`
  - `age5_gate0_transport_family_contract_progress`
- direct surface:
  - `aggregate status line`
  - `final status line`
  - `ci_gate_result/summary compact`
  - `ci_fail_brief/triage`
- selftest:
  - `python tests/run_ci_aggregate_status_line_selftest.py`
  - `python tests/run_ci_gate_final_status_line_selftest.py`
  - `python tests/run_ci_gate_summary_line_check_selftest.py`

## Matrix

| surface line | summary | primary contract |
| --- | --- | --- |
| lang runtime transport | `lang surface + stdlib + tensor` downstream transport | `lang_runtime_family`가 언어/runtime line의 transport를 닫는다 |
| gate0 runtime transport | `lang runtime + W95/W96/W97` downstream transport | `gate0_runtime_family`가 Gate0 runtime transport를 닫는다 |
| gate0 family transport | `gate0 runtime + W92/W93/W94` downstream transport | `gate0_family`가 Gate0 상위 transport를 닫는다 |

## Consumer Surface

- 상위 umbrella transport family:
  - `tests/gate0_surface_transport_family/README.md`
  - `python tests/run_gate0_surface_transport_family_selftest.py`
- 상위 umbrella family:
  - `tests/gate0_surface_family/README.md`
  - `python tests/run_gate0_surface_family_selftest.py`
- `tests/lang_runtime_family/README.md`
- `tests/gate0_runtime_family/README.md`
- `tests/gate0_family/README.md`
- `python tests/run_lang_runtime_family_selftest.py`
- `python tests/run_gate0_runtime_family_selftest.py`
- `python tests/run_gate0_family_selftest.py`
- `python tests/run_gate0_transport_family_selftest.py`
- `python tests/run_gate0_transport_family_contract_selftest.py`
- `python tests/run_gate0_transport_family_contract_summary_selftest.py`
- `python tests/run_gate0_transport_family_transport_contract_selftest.py`
- `python tests/run_gate0_transport_family_transport_contract_summary_selftest.py`
- `python tests/run_ci_aggregate_age5_child_summary_gate0_transport_family_selftest.py`
- `python tests/run_ci_aggregate_age5_child_summary_gate0_transport_family_transport_selftest.py`
- `python tests/run_age5_close_combined_report_contract_selftest.py`
- `python tests/run_ci_gate_summary_report_check_selftest.py`
- `python tests/run_ci_aggregate_gate_age5_diagnostics_check.py`
- `python tests/run_ci_aggregate_status_line_selftest.py`
- `python tests/run_ci_gate_final_status_line_selftest.py`
- `python tests/run_ci_gate_result_check_selftest.py`
- `python tests/run_ci_gate_outputs_consistency_check_selftest.py`
- `python tests/run_ci_gate_summary_line_check_selftest.py`
- `python tests/run_ci_final_line_emitter_check.py`
- `python tests/run_ci_gate_report_index_check_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
