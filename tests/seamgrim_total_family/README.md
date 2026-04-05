# Seamgrim Total Family

## Stable Contract

- parent line:
  - `seamgrim total family`
- child lines:
  - `tests/seamgrim_system_family/README.md`
- child checks:
  - `python tests/run_seamgrim_system_family_selftest.py`
  - `python tests/run_seamgrim_full_gate_check.py`
  - `python tests/run_seamgrim_total_family_selftest.py`
- fixed family line:
  - `system transport + full gate`

## Stable Bundle Contract

- bundle runner:
  - `python tests/run_seamgrim_total_family_contract_selftest.py`
- summary check:
  - `python tests/run_seamgrim_total_family_contract_summary_selftest.py`
- bundle `checks_text`:
  - `system_transport,full_gate,seamgrim_total_family`
- progress schema:
  - `ddn.ci.seamgrim_total_family_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport runner:
  - `python tests/run_seamgrim_total_family_transport_contract_selftest.py`
- transport summary check:
  - `python tests/run_seamgrim_total_family_transport_contract_summary_selftest.py`
- transport bundle `checks_text`:
  - `family_contract,system_transport,full_gate`
- progress schema:
  - `ddn.ci.seamgrim_total_family_transport_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`
- sanity steps:
  - `seamgrim_total_family_transport_contract_selftest`
  - `seamgrim_total_family_transport_contract_summary_selftest`

## Notes

- 이 family는 seamgrim 최상위 전달면에서 `system transport`와 `full gate`를 한 줄의 total umbrella로 다시 묶는다.
- 현재 포함 범위:
  - `seamgrim_system_family`의 transport line
  - `run_seamgrim_full_gate_check`
- parent family:
  - `tests/seamgrim_assurance_family/README.md`
  - `python tests/run_seamgrim_assurance_family_selftest.py`
