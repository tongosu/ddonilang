# Seamgrim Stack Family

## Stable Contract

- parent line:
  - `seamgrim stack family`
- child lines:
  - `tests/seamgrim_surface_family/README.md`
  - `tests/seamgrim_runtime_family/README.md`
  - `tests/seamgrim_gate_family/README.md`
- child checks:
  - `python tests/run_seamgrim_surface_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_runtime_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_gate_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_stack_family_selftest.py`
- fixed family line:
  - `surface transport + runtime transport + gate transport`

## Stable Bundle Contract

- bundle runner:
  - `python tests/run_seamgrim_stack_family_contract_selftest.py`
- summary check:
  - `python tests/run_seamgrim_stack_family_contract_summary_selftest.py`
- bundle `checks_text`:
  - `surface_transport,runtime_transport,gate_transport,seamgrim_stack_family`
- progress schema:
  - `ddn.ci.seamgrim_stack_family_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport runner:
  - `python tests/run_seamgrim_stack_family_transport_contract_selftest.py`
- transport summary check:
  - `python tests/run_seamgrim_stack_family_transport_contract_summary_selftest.py`
- transport bundle `checks_text`:
  - `family_contract,surface_transport,runtime_transport,gate_transport`
- progress schema:
  - `ddn.ci.seamgrim_stack_family_transport_contract_selftest.progress.v1`
- sanity steps:
  - `seamgrim_stack_family_transport_contract_selftest`
  - `seamgrim_stack_family_transport_contract_summary_selftest`
- direct surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Notes

- 이 family는 seamgrim의 상위 transport stack을 `surface -> runtime -> gate` 한 줄로 다시 묶는다.
- 현재 포함 범위:
  - `seamgrim_surface_family_transport_contract`
  - `seamgrim_runtime_family_transport_contract`
  - `seamgrim_gate_family_transport_contract`
- parent family:
  - `tests/seamgrim_application_family/README.md`
  - `python tests/run_seamgrim_application_family_selftest.py`
  - `tests/seamgrim_system_family/README.md`
  - `python tests/run_seamgrim_system_family_selftest.py`
