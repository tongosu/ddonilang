# Seamgrim Delivery Family

## Stable Contract

- parent line:
  - `seamgrim delivery family`
- child lines:
  - `tests/seamgrim_application_family/README.md`
  - `tests/seamgrim_runtime_family/README.md`
- child checks:
  - `python tests/run_seamgrim_application_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_runtime_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_full_gate_check.py`
  - `python tests/run_seamgrim_delivery_family_selftest.py`
- fixed family line:
  - `application transport + runtime transport + full gate`

## Stable Bundle Contract

- bundle runner:
  - `python tests/run_seamgrim_delivery_family_contract_selftest.py`
- summary check:
  - `python tests/run_seamgrim_delivery_family_contract_summary_selftest.py`
- bundle `checks_text`:
  - `application_transport,runtime_transport,full_gate,seamgrim_delivery_family`
- progress schema:
  - `ddn.ci.seamgrim_delivery_family_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport runner:
  - `python tests/run_seamgrim_delivery_family_transport_contract_selftest.py`
- transport summary check:
  - `python tests/run_seamgrim_delivery_family_transport_contract_summary_selftest.py`
- transport bundle `checks_text`:
  - `family_contract,application_transport,runtime_transport,full_gate`
- progress schema:
  - `ddn.ci.seamgrim_delivery_family_transport_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`
- sanity steps:
  - `seamgrim_delivery_family_transport_contract_selftest`
  - `seamgrim_delivery_family_transport_contract_summary_selftest`

## Notes

- 이 family는 seamgrim 전달면을 `application transport + runtime transport + full gate` 한 줄로 다시 묶는다.
- 현재 포함 범위:
  - `seamgrim_application_family_transport_contract`
  - `seamgrim_runtime_family_transport_contract`
  - `run_seamgrim_full_gate_check`
- parent family:
  - `tests/seamgrim_release_family/README.md`
  - `python tests/run_seamgrim_release_family_selftest.py`
