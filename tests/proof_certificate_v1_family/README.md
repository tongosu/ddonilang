# Proof Certificate V1 Family

## Stable Contract

- 목적:
  - `proof_certificate_v1`의 `signed contract`, `consumer contract`, `promotion` 세 줄을 상위 family contract로 한 번에 묶는다.
  - 이 문서는 emit/verify/schema line을 각각 다시 정의하지 않고, 세 상위 contract가 같은 `proof_certificate_v1` family를 가리키는지만 확인한다.
- pack 계약:
  - `pack/proof_certificate_v1_family_v1/README.md`
- 대상 surface:
  - `tests/proof_certificate_v1_signed_contract/README.md`
  - `tests/proof_certificate_v1_consumer_contract/README.md`
  - `tests/proof_certificate_v1_promotion/README.md`
  - `pack/proof_certificate_v1_family_contract_v1/README.md`
  - `pack/proof_certificate_v1_family_transport_contract_v1/README.md`
  - `pack/age4_proof_detjson_smoke_v1/README.md`
- selftest:
  - `python tests/run_proof_certificate_v1_signed_contract_selftest.py`
  - `python tests/run_proof_certificate_v1_consumer_contract_selftest.py`
  - `python tests/run_proof_certificate_v1_promotion_selftest.py`
  - `python tests/run_proof_certificate_v1_family_selftest.py`
  - `python tests/run_proof_certificate_v1_family_contract_selftest.py`
  - `python tests/run_proof_certificate_v1_family_contract_summary_selftest.py`
  - `python tests/run_proof_certificate_v1_family_contract_pack_check.py`
  - `python tests/run_proof_certificate_v1_family_transport_pack_check.py`
  - `proof_certificate_v1_family_selftest`
  - `proof_certificate_v1_family_contract_selftest`

## Stable Transport Contract

- bundle `checks_text`:
  - `signed_contract,consumer_contract,promotion,family`
- progress schema:
  - `ddn.ci.proof_certificate_v1_family_contract_selftest.progress.v1`
- transport bundle `checks_text`:
  - `family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index`
- transport progress schema:
  - `ddn.ci.proof_certificate_v1_family_transport_contract_selftest.progress.v1`
- sanity steps:
  - `proof_certificate_v1_family_contract_selftest`
  - `proof_certificate_v1_family_contract_summary_selftest`
  - `proof_certificate_v1_family_transport_contract_selftest`
  - `proof_certificate_v1_family_transport_contract_summary_selftest`
- direct surface:
  - `ci_sanity_gate stdout`
  - `*.progress.detjson`
  - `age5 close full-real report`
  - `aggregate preview summary`
  - `aggregate status line`
  - `final status line`
  - `gate result/summary compact`
  - `ci_fail_brief/triage`
  - `ci_gate_report_index`
  - `python tests/run_proof_certificate_v1_family_contract_selftest.py`
  - `python tests/run_proof_certificate_v1_family_contract_summary_selftest.py`
  - `python tests/run_proof_certificate_v1_family_contract_pack_check.py`
  - `python tests/run_age5_close_combined_report_contract_selftest.py`
  - `python tests/run_ci_aggregate_age5_child_summary_proof_certificate_v1_family_transport_selftest.py`
  - `python tests/run_ci_aggregate_gate_age5_diagnostics_check.py`
  - `python tests/run_ci_aggregate_status_line_selftest.py`
  - `python tests/run_ci_gate_final_status_line_selftest.py`
  - `python tests/run_ci_gate_result_check_selftest.py`
  - `python tests/run_ci_gate_outputs_consistency_check_selftest.py`
  - `python tests/run_ci_gate_summary_line_check_selftest.py`
  - `python tests/run_ci_final_line_emitter_check.py`
  - `python tests/run_ci_gate_report_index_check_selftest.py`
  - `python tests/run_proof_certificate_v1_family_transport_contract_selftest.py`
  - `python tests/run_proof_certificate_v1_family_transport_contract_summary_selftest.py`
  - `python tests/run_proof_certificate_v1_family_transport_pack_check.py`

## Matrix

| family line | summary | primary contract |
| --- | --- | --- |
| signed line | `runtime emit -> signed emit -> signed emit profiles -> signed contract` | signed bundle이 runtime sidecar/promotion line과 끊기지 않는다 |
| consumer line | `signed emit profiles -> verify bundle -> verify report -> verify report digest contract -> consumer transport -> consumer contract` | signed bundle 소비 surface와 verify artifact line이 같은 family를 이룬다 |
| promotion line | `draft contract -> flat schema candidate -> flat schema split -> promotion` | 후보/초안/schema line이 최종 승격 후보 line으로 닫혀 있다 |

## Consumer Surface

- `tests/proof_certificate_v1_signed_contract/README.md`
- `tests/proof_certificate_v1_consumer_contract/README.md`
- `tests/proof_certificate_v1_promotion/README.md`
- `pack/proof_certificate_v1_family_contract_v1/README.md`
- `pack/proof_certificate_v1_family_transport_contract_v1/README.md`
- `pack/age4_proof_detjson_smoke_v1/README.md`
- `python tests/run_proof_certificate_v1_signed_contract_selftest.py`
- `python tests/run_proof_certificate_v1_consumer_contract_selftest.py`
- `python tests/run_proof_certificate_v1_promotion_selftest.py`
- `python tests/run_proof_certificate_v1_family_selftest.py`
- `python tests/run_proof_certificate_v1_family_contract_selftest.py`
- `python tests/run_proof_certificate_v1_family_contract_summary_selftest.py`
- `python tests/run_proof_certificate_v1_family_contract_pack_check.py`
- `python tests/run_proof_certificate_v1_family_transport_contract_selftest.py`
- `python tests/run_proof_certificate_v1_family_transport_contract_summary_selftest.py`
- `python tests/run_proof_certificate_v1_family_transport_pack_check.py`
- `tests/proof_certificate_family/README.md`
- `python tests/run_proof_certificate_family_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
