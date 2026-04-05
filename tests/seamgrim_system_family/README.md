# Seamgrim System Family

## Stable Contract

- parent line:
  - `seamgrim system family`
- child lines:
  - `tests/seamgrim_stack_family/README.md`
  - `tests/seamgrim_release_family/README.md`
- child checks:
  - `python tests/run_seamgrim_stack_family_selftest.py`
  - `python tests/run_seamgrim_release_family_selftest.py`
  - `python tests/run_seamgrim_system_family_selftest.py`
- fixed family line:
  - `stack transport + release transport`

## Stable Bundle Contract

- bundle runner:
  - `python tests/run_seamgrim_system_family_contract_selftest.py`
- summary check:
  - `python tests/run_seamgrim_system_family_contract_summary_selftest.py`
- bundle `checks_text`:
  - `stack_transport,release_transport,seamgrim_system_family`
- progress schema:
  - `ddn.ci.seamgrim_system_family_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport runner:
  - `python tests/run_seamgrim_system_family_transport_contract_selftest.py`
- transport summary check:
  - `python tests/run_seamgrim_system_family_transport_contract_summary_selftest.py`
- transport bundle `checks_text`:
  - `family_contract,stack_transport,release_transport`
- progress schema:
  - `ddn.ci.seamgrim_system_family_transport_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`
- sanity steps:
  - `seamgrim_system_family_transport_contract_selftest`
  - `seamgrim_system_family_transport_contract_summary_selftest`

## Notes

- 이 family는 seamgrim 상위 구조면에서 `stack transport`와 `release transport`를 한 줄의 system umbrella로 다시 묶는다.
- 현재 포함 범위:
  - `seamgrim_stack_family`의 transport line
  - `seamgrim_release_family`의 transport line
- parent family:
  - `tests/seamgrim_total_family/README.md`
  - `python tests/run_seamgrim_total_family_selftest.py`
