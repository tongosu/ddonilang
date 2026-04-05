# Seamgrim Release Family

## Stable Contract

- parent line:
  - `seamgrim release family`
- child lines:
  - `tests/seamgrim_delivery_family/README.md`
  - `tests/seamgrim_gate_family/README.md`
- child checks:
  - `python tests/run_seamgrim_delivery_family_selftest.py`
  - `python tests/run_seamgrim_gate_family_selftest.py`
  - `python tests/run_seamgrim_release_family_selftest.py`
- fixed family line:
  - `delivery transport + gate transport`

## Stable Bundle Contract

- bundle runner:
  - `python tests/run_seamgrim_release_family_contract_selftest.py`
- summary check:
  - `python tests/run_seamgrim_release_family_contract_summary_selftest.py`
- bundle `checks_text`:
  - `delivery_transport,gate_transport,seamgrim_release_family`
- progress schema:
  - `ddn.ci.seamgrim_release_family_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport runner:
  - `python tests/run_seamgrim_release_family_transport_contract_selftest.py`
- transport summary check:
  - `python tests/run_seamgrim_release_family_transport_contract_summary_selftest.py`
- transport bundle `checks_text`:
  - `family_contract,delivery_transport,gate_transport`
- progress schema:
  - `ddn.ci.seamgrim_release_family_transport_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`
- sanity steps:
  - `seamgrim_release_family_transport_contract_selftest`
  - `seamgrim_release_family_transport_contract_summary_selftest`

## Notes

- 이 family는 seamgrim 상위 배포면에서 `delivery transport`와 `gate transport`를 한 줄의 release umbrella로 다시 묶는다.
- 현재 포함 범위:
  - `seamgrim_delivery_family`의 transport line
  - `seamgrim_gate_family`의 transport line
- parent family:
  - `tests/seamgrim_system_family/README.md`
  - `python tests/run_seamgrim_system_family_selftest.py`
