# Gate0 Surface Family

## Stable Contract

- 목적:
  - 현재 `core_lang` gate에서 닫힌 Gate0/language 상위 줄을 한 단계 위에서 함께 읽는 최상위 umbrella family contract를 고정한다.
  - 이 문서는 하위 line의 세부 transport를 다시 정의하지 않고, `lang_surface`, `lang_runtime`, `gate0_runtime`, `gate0_family`, `gate0_transport_family`가 같은 Gate0 surface를 이룬다는 점만 확인한다.
- 대상 surface:
  - `tests/lang_surface_family/README.md`
  - `tests/lang_runtime_family/README.md`
  - `tests/gate0_runtime_family/README.md`
  - `tests/gate0_family/README.md`
  - `tests/gate0_transport_family/README.md`
- selftest:
  - `python tests/run_lang_surface_family_selftest.py`
  - `python tests/run_lang_runtime_family_selftest.py`
  - `python tests/run_gate0_runtime_family_selftest.py`
  - `python tests/run_gate0_family_selftest.py`
  - `python tests/run_gate0_transport_family_selftest.py`
  - `python tests/run_gate0_surface_family_selftest.py`
  - `python tests/run_gate0_surface_family_contract_selftest.py`
  - `python tests/run_gate0_surface_family_contract_summary_selftest.py`
- sanity steps:
  - `lang_surface_family_selftest`
  - `lang_runtime_family_selftest`
  - `gate0_runtime_family_selftest`
  - `gate0_family_selftest`
  - `gate0_transport_family_selftest`
  - `gate0_surface_family_selftest`
  - `gate0_surface_family_contract_selftest`
  - `gate0_surface_family_contract_summary_selftest`

## Stable Bundle Contract

- bundle `checks_text`:
  - `lang_surface_family,lang_runtime_family,gate0_runtime_family,gate0_family,gate0_transport_family`
- progress schema:
  - `ddn.ci.gate0_surface_family_contract_selftest.progress.v1`
- sanity steps:
  - `gate0_surface_family_selftest`
  - `gate0_surface_family_contract_selftest`
  - `gate0_surface_family_contract_summary_selftest`
- direct surface:
  - `ci_sanity_gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport bundle `checks_text`:
  - `family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index`
- progress schema:
  - `ddn.ci.gate0_surface_family_transport_contract_selftest.progress.v1`
- raw field surface:
  - `age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks`
  - `age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks`
  - `age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text`
  - `age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe`
  - `age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe`
  - `age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present`
- compact token surface:
  - `age5_gate0_surface_family_transport_contract_completed`
  - `age5_gate0_surface_family_transport_contract_total`
  - `age5_gate0_surface_family_transport_contract_checks_text`
  - `age5_gate0_surface_family_transport_contract_current_probe`
  - `age5_gate0_surface_family_transport_contract_last_completed_probe`
  - `age5_gate0_surface_family_transport_contract_progress`
- sanity steps:
  - `gate0_surface_family_transport_contract_selftest`
  - `gate0_surface_family_transport_contract_summary_selftest`
- direct surface:
  - `ci_sanity_gate stdout / json-out`
  - `age5_close full-real report`
  - `aggregate preview summary`
  - `aggregate status line`
  - `final status line`
  - `gate result / summary compact`
  - `ci_fail_brief / triage`
  - `ci_gate_report_index`

## Stable Upstream Transport

- raw field:
  - `age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks`
  - `age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks`
  - `age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text`
  - `age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe`
  - `age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe`
  - `age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present`
- direct surface:
  - `ci_sanity_gate stdout/json-out`
  - `age5_close full-real report`
  - `aggregate preview summary`

## Stable Downstream Transport

- compact token:
  - `age5_gate0_surface_family_transport_contract_completed`
  - `age5_gate0_surface_family_transport_contract_total`
  - `age5_gate0_surface_family_transport_contract_checks_text`
  - `age5_gate0_surface_family_transport_contract_current_probe`
  - `age5_gate0_surface_family_transport_contract_last_completed_probe`
  - `age5_gate0_surface_family_transport_contract_progress`
- direct surface:
  - `aggregate status line`
  - `final status line`
  - `gate result/summary compact`
  - `ci_fail_brief/triage`
  - `ci_gate_report_index`

## Matrix

| surface line | summary | primary contract |
| --- | --- | --- |
| lang surface line | `proof + bogae alias + compound update reject` | `lang_surface_family`가 핵심 언어 표면 contract를 닫는다 |
| lang runtime line | `lang surface + stdlib + tensor` | `lang_runtime_family`가 언어/runtime 상위 line을 닫는다 |
| gate0 runtime line | `lang runtime + W95/W96/W97` | `gate0_runtime_family`가 Gate0 runtime line을 닫는다 |
| gate0 family line | `gate0 runtime + W92/W93/W94` | `gate0_family`가 Gate0 상위 runtime family를 닫는다 |
| gate0 transport line | `lang/gate0 transport umbrella` | `gate0_transport_family`가 transport 관측 line을 닫는다 |

## Consumer Surface

- 상위 umbrella transport family:
  - `tests/gate0_surface_transport_family/README.md`
  - `python tests/run_gate0_surface_transport_family_selftest.py`
- `tests/lang_surface_family/README.md`
- `tests/lang_runtime_family/README.md`
- `tests/gate0_runtime_family/README.md`
- `tests/gate0_family/README.md`
- `tests/gate0_transport_family/README.md`
- `python tests/run_lang_surface_family_selftest.py`
- `python tests/run_lang_runtime_family_selftest.py`
- `python tests/run_gate0_runtime_family_selftest.py`
- `python tests/run_gate0_family_selftest.py`
- `python tests/run_gate0_transport_family_selftest.py`
- `python tests/run_gate0_surface_family_selftest.py`
- `python tests/run_gate0_surface_family_contract_selftest.py`
- `python tests/run_gate0_surface_family_contract_summary_selftest.py`
- `python tests/run_gate0_surface_family_transport_contract_selftest.py`
- `python tests/run_gate0_surface_family_transport_contract_summary_selftest.py`
- `python tests/run_ci_aggregate_age5_child_summary_gate0_surface_family_transport_selftest.py`
- `python tests/run_ci_aggregate_status_line_selftest.py`
- `python tests/run_ci_gate_final_status_line_selftest.py`
- `python tests/run_ci_gate_result_check_selftest.py`
- `python tests/run_ci_gate_outputs_consistency_check_selftest.py`
- `python tests/run_ci_gate_summary_line_check_selftest.py`
- `python tests/run_ci_final_line_emitter_check.py`
- `python tests/run_ci_gate_report_index_check_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
