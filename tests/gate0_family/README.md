# Gate0 Family

## Stable Contract

- 목적:
  - `gate0_runtime_family`와 W92/W93/W94 runtime pack line을 한 단계 위에서 함께 읽는 상위 Gate0 family contract를 고정한다.
  - 이 문서는 하위 transport를 다시 정의하지 않고, Gate0 구현의 핵심 runtime 줄이 같은 family를 이룬다는 점만 확인한다.
- 대상 surface:
  - `tests/gate0_runtime_family/README.md`
  - `pack/gogae9_w92_aot_compiler_v2/README.md`
  - `pack/gogae9_w93_universe_gui/README.md`
  - `pack/gogae9_w94_social_sim/README.md`
- selftest:
  - `python tests/run_gate0_runtime_family_selftest.py`
  - `python tests/run_w92_aot_pack_check.py`
  - `python tests/run_w93_universe_pack_check.py`
  - `python tests/run_w94_social_pack_check.py`
  - `python tests/run_gate0_family_selftest.py`
  - `python tests/run_gate0_family_contract_selftest.py`
  - `python tests/run_gate0_family_contract_summary_selftest.py`
- sanity steps:
  - `gate0_runtime_family_selftest`
  - `w92_aot_pack_check`
  - `w93_universe_pack_check`
  - `w94_social_pack_check`
  - `gate0_family_selftest`
  - `gate0_family_contract_selftest`
  - `gate0_family_contract_summary_selftest`

## Stable Bundle Contract

- bundle `checks_text`:
  - `gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family`
- progress schema:
  - `ddn.ci.gate0_family_contract_selftest.progress.v1`
- sanity steps:
  - `gate0_family_selftest`
  - `gate0_family_contract_selftest`
  - `gate0_family_contract_summary_selftest`
- direct surface:
  - `ci_sanity_gate stdout`
  - `*.progress.detjson`

## Stable Upstream Transport

- raw field:
  - `age5_full_real_gate0_family_contract_selftest_completed_checks`
  - `age5_full_real_gate0_family_contract_selftest_total_checks`
  - `age5_full_real_gate0_family_contract_selftest_checks_text`
  - `age5_full_real_gate0_family_contract_selftest_current_probe`
  - `age5_full_real_gate0_family_contract_selftest_last_completed_probe`
  - `age5_full_real_gate0_family_contract_selftest_progress_present`
- direct surface:
  - `ci_sanity_gate stdout/json-out`
  - `age5_close full-real report`
  - `aggregate preview summary`
- selftest:
  - `python tests/run_ci_aggregate_age5_child_summary_gate0_family_transport_selftest.py`
  - `python tests/run_age5_close_combined_report_contract_selftest.py`
  - `python tests/run_ci_gate_summary_report_check_selftest.py`
  - `python tests/run_ci_aggregate_gate_age5_diagnostics_check.py`

## Stable Downstream Transport

- compact token:
  - `age5_gate0_family_contract_completed`
  - `age5_gate0_family_contract_total`
  - `age5_gate0_family_contract_checks_text`
  - `age5_gate0_family_contract_current_probe`
  - `age5_gate0_family_contract_last_completed_probe`
  - `age5_gate0_family_contract_progress`
- direct surface:
  - `aggregate status line`
  - `final status line`
  - `gate result/summary compact`
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

## Stable Transport Contract

- transport bundle `checks_text`:
  - `family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index`
- progress schema:
  - `ddn.ci.gate0_family_transport_contract_selftest.progress.v1`
- sanity steps:
  - `gate0_family_transport_contract_selftest`
  - `gate0_family_transport_contract_summary_selftest`
- upstream raw fields:
  - `age5_full_real_gate0_family_transport_contract_selftest_completed_checks`
  - `age5_full_real_gate0_family_transport_contract_selftest_total_checks`
  - `age5_full_real_gate0_family_transport_contract_selftest_checks_text`
  - `age5_full_real_gate0_family_transport_contract_selftest_current_probe`
  - `age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe`
  - `age5_full_real_gate0_family_transport_contract_selftest_progress_present`
- downstream compact tokens:
  - `age5_gate0_family_transport_contract_completed`
  - `age5_gate0_family_transport_contract_total`
  - `age5_gate0_family_transport_contract_checks_text`
  - `age5_gate0_family_transport_contract_current_probe`
  - `age5_gate0_family_transport_contract_last_completed_probe`
  - `age5_gate0_family_transport_contract_progress`
- direct surface:
  - `ci_sanity_gate stdout`
  - `*.progress.detjson`
  - `age5_close full-real report`
  - `aggregate preview summary`
  - `aggregate status line`
  - `final status line`
  - `gate result/summary compact`
  - `ci_fail_brief/triage`
  - `ci_gate_report_index`

## Matrix

| surface line | summary | primary contract |
| --- | --- | --- |
| gate0 runtime line | `lang runtime + W95/W96/W97` | `gate0_runtime_family`가 하위 Gate0 runtime line을 닫는다 |
| W92 AOT line | `bench_cases + parity/speedup floor` | `run_w92_aot_pack_check.py`가 W92 최소 pack contract를 닫는다 |
| W93 universe line | `universe pack/unpack determinism` | `run_w93_universe_pack_check.py`가 W93 pack/runtime contract를 닫는다 |
| W94 social line | `simulate determinism + progress snapshot` | `run_w94_social_pack_check.py`가 W94 social runtime contract를 닫는다 |

## Consumer Surface

- 상위 umbrella transport family:
  - `tests/gate0_surface_transport_family/README.md`
  - `python tests/run_gate0_surface_transport_family_selftest.py`
- 상위 umbrella family:
  - `tests/gate0_surface_family/README.md`
  - `python tests/run_gate0_surface_family_selftest.py`
- 상위 transport family:
  - `tests/gate0_transport_family/README.md`
  - `python tests/run_gate0_transport_family_selftest.py`
- 하위 family:
  - `tests/gate0_runtime_family/README.md`
  - `python tests/run_gate0_runtime_family_selftest.py`
- `pack/gogae9_w92_aot_compiler_v2/README.md`
- `pack/gogae9_w93_universe_gui/README.md`
- `pack/gogae9_w94_social_sim/README.md`
- `python tests/run_w92_aot_pack_check.py`
- `python tests/run_w93_universe_pack_check.py`
- `python tests/run_w94_social_pack_check.py`
- `python tests/run_gate0_family_selftest.py`
- `python tests/run_gate0_family_contract_selftest.py`
- `python tests/run_gate0_family_contract_summary_selftest.py`
- `python tests/run_gate0_family_transport_contract_selftest.py`
- `python tests/run_gate0_family_transport_contract_summary_selftest.py`
- `python tests/run_ci_aggregate_age5_child_summary_gate0_family_transport_selftest.py`
- `python tests/run_ci_aggregate_status_line_selftest.py`
- `python tests/run_ci_gate_final_status_line_selftest.py`
- `python tests/run_ci_gate_result_check_selftest.py`
- `python tests/run_ci_gate_outputs_consistency_check_selftest.py`
- `python tests/run_ci_final_line_emitter_check.py`
- `python tests/run_ci_gate_report_index_check_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
