# Proof Family

## Stable Contract

- 목적:
  - `proof operation family` line과 `proof_certificate family` line을 한 단계 위에서 함께 읽는 최상위 proof family contract를 고정한다.
  - 이 문서는 proof operation 하위 matrix/bundle이나 proof certificate 하위 emit/verify/schema line을 다시 정의하지 않고, 두 상위 contract가 같은 proof family를 이룬다는 점만 확인한다.
- pack 계약:
  - `pack/proof_family_v1/README.md`
  - `pack/proof_family_contract_v1/README.md`
  - `pack/proof_family_transport_contract_v1/README.md`
- 대상 surface:
  - `tests/proof_operation_family/README.md`
  - `tests/proof_certificate_family/README.md`
- selftest:
  - `python tests/run_proof_operation_family_selftest.py`
  - `python tests/run_proof_certificate_family_selftest.py`
  - `python tests/run_proof_family_selftest.py`
  - `python tests/run_proof_family_contract_selftest.py`
  - `python tests/run_proof_family_contract_summary_selftest.py`
  - `proof_operation_family_selftest`
  - `proof_certificate_family_selftest`
  - `proof_family_selftest`
  - `proof_family_contract_selftest`

## Stable Bundle Contract

- bundle `checks_text`:
  - `proof_operation_family,proof_certificate_family,proof_family`
- progress schema:
  - `ddn.ci.proof_family_contract_selftest.progress.v1`
- transport bundle `checks_text`:
  - `family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index`
- transport progress schema:
  - `ddn.ci.proof_family_transport_contract_selftest.progress.v1`
- upstream raw field:
  - `age5_full_real_proof_family_contract_selftest_*`
- downstream compact token:
  - `age5_proof_family_contract_*`
- sanity steps:
  - `proof_family_contract_selftest`
  - `proof_family_contract_summary_selftest`
- direct surface:
  - `ci_sanity_gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport bundle `checks_text`:
  - `family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index`
- transport progress schema:
  - `ddn.ci.proof_family_transport_contract_selftest.progress.v1`
- sanity steps:
  - `proof_family_transport_contract_selftest`
  - `proof_family_transport_contract_summary_selftest`
- direct surface:
  - `ci_sanity_gate stdout`
  - `*.progress.detjson`
  - `age5 close full-real report`
  - `aggregate preview summary`
  - `aggregate status line`
  - `final status line`
  - `gate result / summary compact`
  - `ci_fail_brief / triage`
  - `ci_gate_report_index`
  - `python tests/run_proof_family_transport_contract_selftest.py`
  - `python tests/run_proof_family_transport_contract_summary_selftest.py`
  - `python tests/run_ci_aggregate_age5_child_summary_proof_family_transport_selftest.py`
  - `python tests/run_ci_aggregate_status_line_selftest.py`
  - `python tests/run_ci_gate_final_status_line_selftest.py`
  - `python tests/run_ci_gate_result_check_selftest.py`
  - `python tests/run_ci_gate_outputs_consistency_check_selftest.py`
  - `python tests/run_ci_gate_summary_line_check_selftest.py`
  - `python tests/run_ci_final_line_emitter_check.py`
  - `python tests/run_ci_gate_report_index_check_selftest.py`
  - `python tests/run_proof_family_contract_selftest.py`
  - `python tests/run_proof_family_contract_summary_selftest.py`
  - `age5 close full-real report`
  - `aggregate preview summary`
  - `aggregate status line`
  - `final status line`
  - `gate result / summary compact`
  - `ci_fail_brief / triage`
  - `ci_gate_report_index`
  - `python tests/run_age5_close_combined_report_contract_selftest.py`
  - `python tests/run_ci_aggregate_age5_child_summary_proof_family_transport_selftest.py`
  - `python tests/run_ci_gate_summary_report_check_selftest.py`
  - `python tests/run_ci_aggregate_gate_age5_diagnostics_check.py`
  - `python tests/run_ci_aggregate_status_line_selftest.py`
  - `python tests/run_ci_gate_final_status_line_selftest.py`
  - `python tests/run_ci_gate_result_check_selftest.py`
  - `python tests/run_ci_gate_outputs_consistency_check_selftest.py`
  - `python tests/run_ci_gate_summary_line_check_selftest.py`
  - `python tests/run_ci_final_line_emitter_check.py`
  - `python tests/run_ci_gate_report_index_check_selftest.py`

## Matrix

| family line | summary | primary contract |
| --- | --- | --- |
| proof operation line | `age1 immediate proof -> solver search/check parity -> proof solver family -> proof operation family` | proof artifact 안의 proof/solver operation 보존 contract가 상위 family로 닫혀 있다 |
| proof certificate line | `proof artifact cert bridge -> proof_certificate_v1 family -> proof_certificate family` | proof artifact bytes와 cert/sign/verify/schema line이 상위 cert family로 닫혀 있다 |

## Consumer Surface

- `tests/proof_operation_family/README.md`
- `tests/proof_certificate_family/README.md`
- `tests/lang_surface_family/README.md`
- `python tests/run_proof_operation_family_selftest.py`
- `python tests/run_proof_certificate_family_selftest.py`
- `python tests/run_proof_family_selftest.py`
- `python tests/run_lang_surface_family_selftest.py`
- `python tests/run_proof_family_contract_selftest.py`
- `python tests/run_proof_family_contract_summary_selftest.py`
- `python tests/run_proof_family_transport_contract_selftest.py`
- `python tests/run_proof_family_transport_contract_summary_selftest.py`
- `python tests/run_age5_close_combined_report_contract_selftest.py`
- `python tests/run_ci_aggregate_age5_child_summary_proof_family_transport_selftest.py`
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
