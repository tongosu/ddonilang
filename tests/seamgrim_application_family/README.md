# Seamgrim Application Family

## Stable Contract

- parent line:
  - `seamgrim application family`
- child lines:
  - `tests/seamgrim_stack_family/README.md`
  - `tests/seamgrim_interaction_family/README.md`
- child checks:
  - `python tests/run_seamgrim_stack_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_interaction_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_application_family_selftest.py`
- fixed family line:
  - `stack transport + interaction transport`

## Stable Bundle Contract

- bundle runner:
  - `python tests/run_seamgrim_application_family_contract_selftest.py`
- summary check:
  - `python tests/run_seamgrim_application_family_contract_summary_selftest.py`
- bundle `checks_text`:
  - `stack_transport,interaction_transport,seamgrim_application_family`
- progress schema:
  - `ddn.ci.seamgrim_application_family_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport runner:
  - `python tests/run_seamgrim_application_family_transport_contract_selftest.py`
- transport summary check:
  - `python tests/run_seamgrim_application_family_transport_contract_summary_selftest.py`
- transport bundle `checks_text`:
  - `family_contract,stack_transport,interaction_transport`
- progress schema:
  - `ddn.ci.seamgrim_application_family_transport_contract_selftest.progress.v1`
- sanity steps:
  - `seamgrim_application_family_transport_contract_selftest`
  - `seamgrim_application_family_transport_contract_summary_selftest`
- direct surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Notes

- 이 family는 seamgrim application layer를 `stack transport + interaction transport` 한 줄로 다시 묶는다.
- 현재 포함 범위:
  - `seamgrim_stack_family_transport_contract`
  - `seamgrim_interaction_family_transport_contract`
- parent family:
  - `tests/seamgrim_delivery_family/README.md`
  - `python tests/run_seamgrim_delivery_family_selftest.py`
