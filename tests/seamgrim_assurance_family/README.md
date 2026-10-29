# Seamgrim Assurance Family

## Stable Contract

- parent line:
  - `seamgrim assurance family`
- child lines:
  - `tests/seamgrim_total_family/README.md`
  - `tests/seamgrim_guard_surface_family/README.md`
- child checks:
  - `python tests/run_seamgrim_total_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_guard_surface_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_assurance_family_selftest.py`
- inherited optional preflight:
  - `python tests/run_seamgrim_full_gate_surface_contract_selftest.py` (via guard surface family)
- fixed family line:
  - `total transport + guard surface transport (+ optional full gate surface preflight via guard surface)`

## Stable Bundle Contract

- bundle runner:
  - `python tests/run_seamgrim_assurance_family_contract_selftest.py`
- summary check:
  - `python tests/run_seamgrim_assurance_family_contract_summary_selftest.py`
- bundle `checks_text`:
  - `total_transport,guard_surface_transport,seamgrim_assurance_family`
- progress schema:
  - `ddn.ci.seamgrim_assurance_family_contract_selftest.progress.v1`
- progress surface:
  - `stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport runner:
  - `python tests/run_seamgrim_assurance_family_transport_contract_selftest.py`
- transport summary check:
  - `python tests/run_seamgrim_assurance_family_transport_contract_summary_selftest.py`
- transport bundle `checks_text`:
  - `family_contract,total_transport,guard_surface_transport`
- progress schema:
  - `ddn.ci.seamgrim_assurance_family_transport_contract_selftest.progress.v1`
- progress surface:
  - `stdout`
  - `*.progress.detjson`

## Notes

- 이 family는 gate line(`total`)과 standalone guard line(`guard surface`)을 한 줄의 assurance umbrella로 다시 묶는다.
- transport line도 같은 thin layer를 다시 묶되, `run_seamgrim_ci_gate.py`에 직접 배선하지 않고 standalone/progress 면으로 유지한다.
- 현재 포함 범위:
  - `seamgrim_total_family`의 transport line
  - `seamgrim_guard_surface_family`의 transport line
- inherited optional preflight:
  - `seamgrim_full_gate_surface_contract` (via `seamgrim_guard_surface_family`)
