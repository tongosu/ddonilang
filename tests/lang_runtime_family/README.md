# Lang Runtime Family

## Stable Contract

- 목적:
  - 현재 `core_lang` gate에서 닫힌 상위 언어/runtime 핵심 line을 한 단계 위에서 함께 읽는 family contract를 고정한다.
  - 이 문서는 하위 line의 세부 transport를 다시 정의하지 않고, `lang surface`, `stdlib catalog`, `tensor.v0` runtime line이 같은 상위 runtime family를 이룬다는 점만 확인한다.
- 대상 surface:
  - `tests/lang_surface_family/README.md`
  - `tests/run_stdlib_catalog_check_selftest.py`
  - `tests/run_tensor_v0_pack_check.py`
  - `tests/run_tensor_v0_cli_check.py`
- selftest:
  - `python tests/run_lang_surface_family_selftest.py`
  - `python tests/run_stdlib_catalog_check_selftest.py`
  - `python tests/run_tensor_v0_pack_check.py`
  - `python tests/run_tensor_v0_cli_check.py`
  - `python tests/run_lang_runtime_family_selftest.py`
  - `python tests/run_lang_runtime_family_contract_selftest.py`
  - `python tests/run_lang_runtime_family_contract_summary_selftest.py`
  - `lang_surface_family_selftest`
  - `stdlib_catalog_check_selftest`
  - `tensor_v0_pack_check`
  - `tensor_v0_cli_check`
  - `lang_runtime_family_selftest`
  - `lang_runtime_family_contract_selftest`

## Stable Bundle Contract

- bundle `checks_text`:
  - `lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family`
- progress schema:
  - `ddn.ci.lang_runtime_family_contract_selftest.progress.v1`
- sanity steps:
  - `lang_runtime_family_selftest`
  - `lang_runtime_family_contract_selftest`
  - `lang_runtime_family_contract_summary_selftest`
- direct surface:
  - `ci_sanity_gate stdout`
  - `*.progress.detjson`

## Stable Upstream Transport

- raw field:
  - `age5_full_real_lang_runtime_family_contract_selftest_completed_checks`
  - `age5_full_real_lang_runtime_family_contract_selftest_total_checks`
  - `age5_full_real_lang_runtime_family_contract_selftest_checks_text`
  - `age5_full_real_lang_runtime_family_contract_selftest_current_probe`
  - `age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe`
  - `age5_full_real_lang_runtime_family_contract_selftest_progress_present`
- direct surface:
  - `ci_sanity_gate stdout/json-out`
  - `age5_close full-real report`
  - `aggregate preview summary`
  - `aggregate status line`
  - `final status line`
  - `gate result/summary compact`
  - `ci_fail_brief/triage`
  - `ci_gate_report_index`
- compact token:
  - `age5_lang_runtime_family_contract_completed`
  - `age5_lang_runtime_family_contract_total`
  - `age5_lang_runtime_family_contract_checks_text`
  - `age5_lang_runtime_family_contract_current_probe`
  - `age5_lang_runtime_family_contract_last_completed_probe`
  - `age5_lang_runtime_family_contract_progress`
- selftest:
  - `python tests/run_ci_aggregate_age5_child_summary_lang_runtime_family_transport_selftest.py`
  - `python tests/run_ci_gate_summary_report_check_selftest.py`
  - `python tests/run_ci_aggregate_gate_age5_diagnostics_check.py`
  - `python tests/run_age5_close_combined_report_contract_selftest.py`

## Stable Transport Contract

- transport bundle `checks_text`:
  - `family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index`
- progress schema:
  - `ddn.ci.lang_runtime_family_transport_contract_selftest.progress.v1`
- raw field surface:
  - `age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks`
  - `age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks`
  - `age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text`
  - `age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe`
  - `age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe`
  - `age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present`
- compact token surface:
  - `age5_lang_runtime_family_transport_contract_completed`
  - `age5_lang_runtime_family_transport_contract_total`
  - `age5_lang_runtime_family_transport_contract_checks_text`
  - `age5_lang_runtime_family_transport_contract_current_probe`
  - `age5_lang_runtime_family_transport_contract_last_completed_probe`
  - `age5_lang_runtime_family_transport_contract_progress`
- sanity steps:
  - `lang_runtime_family_transport_contract_selftest`
  - `lang_runtime_family_transport_contract_summary_selftest`
- downstream parity:
  - `age5_full_real_lang_runtime_family_transport_contract_selftest_*` raw field와 `age5_lang_runtime_family_transport_contract_*` compact token이 같은 line으로 유지된다.
- direct surface:
  - `ci_sanity_gate stdout / json-out`
  - `age5_close full-real report`
  - `aggregate preview summary`
  - `aggregate status line`
  - `final status line`
  - `gate result / summary compact`
  - `ci_fail_brief / triage`
  - `ci_gate_report_index`

## Matrix

| surface line | summary | primary contract |
| --- | --- | --- |
| lang surface line | `proof family + bogae alias family + compound update reject` | 핵심 문법/표면 contract가 `lang_surface_family`로 닫혀 있다 |
| stdlib catalog line | `impl matrix + pack coverage` | 표준 함수 구현/pack coverage 문서 contract가 `stdlib_catalog_check`로 닫혀 있다 |
| tensor pack line | `tensor.v0 dense + sparse pack` | `tensor.v0` 저장형 pack contract가 `run_tensor_v0_pack_check.py`로 닫혀 있다 |
| tensor cli line | `tensor.v0 cli positive + negative` | `teul-cli` tensor runtime/CLI contract가 `run_tensor_v0_cli_check.py`로 닫혀 있다 |

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
- 상위 family:
  - `tests/gate0_runtime_family/README.md`
  - `python tests/run_gate0_runtime_family_selftest.py`
- `tests/lang_surface_family/README.md`
- `python tests/run_lang_surface_family_selftest.py`
- `python tests/run_stdlib_catalog_check_selftest.py`
- `python tests/run_tensor_v0_pack_check.py`
- `python tests/run_tensor_v0_cli_check.py`
- `python tests/run_lang_runtime_family_selftest.py`
- `python tests/run_lang_runtime_family_contract_selftest.py`
- `python tests/run_lang_runtime_family_contract_summary_selftest.py`
- `python tests/run_lang_runtime_family_transport_contract_selftest.py`
- `python tests/run_lang_runtime_family_transport_contract_summary_selftest.py`
- `python tests/run_ci_aggregate_age5_child_summary_lang_runtime_family_transport_selftest.py`
- `python tests/run_ci_aggregate_status_line_selftest.py`
- `python tests/run_ci_gate_final_status_line_selftest.py`
- `python tests/run_ci_gate_result_check_selftest.py`
- `python tests/run_ci_gate_outputs_consistency_check_selftest.py`
- `python tests/run_ci_gate_summary_line_check_selftest.py`
- `python tests/run_ci_final_line_emitter_check.py`
- `python tests/run_ci_gate_report_index_check_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
- `python tests/run_ci_aggregate_age5_child_summary_lang_runtime_family_transport_selftest.py`
- `python tests/run_ci_gate_summary_report_check_selftest.py`
- `python tests/run_ci_aggregate_gate_age5_diagnostics_check.py`
