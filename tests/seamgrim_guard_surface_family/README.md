# Seamgrim Guard Surface Family

## Stable Contract

- parent line:
  - `seamgrim guard surface family`
- child lines:
  - `tests/seamgrim_full_gate_surface_contract/README.md`
  - `tests/seamgrim_ddn_exec_server_surface_contract/README.md`
  - `tests/seamgrim_runtime_fallback_surface_contract/README.md`
- child checks:
  - `python tests/run_seamgrim_ddn_exec_server_surface_contract_selftest.py`
  - `python tests/run_seamgrim_runtime_fallback_surface_contract_selftest.py`
  - `python tests/run_seamgrim_guard_surface_family_selftest.py`
- optional preflight:
  - `python tests/run_seamgrim_full_gate_surface_contract_selftest.py`
- fixed family line:
  - `ddn exec server surface + runtime fallback surface (+ optional full gate surface preflight)`

## Stable Bundle Contract

- bundle runner:
  - `python tests/run_seamgrim_guard_surface_family_contract_selftest.py`
- summary check:
  - `python tests/run_seamgrim_guard_surface_family_contract_summary_selftest.py`
- bundle `checks_text`:
  - `ddn_exec_server_surface,runtime_fallback_surface,seamgrim_guard_surface_family`
- progress schema:
  - `ddn.ci.seamgrim_guard_surface_family_contract_selftest.progress.v1`
- progress surface:
  - `stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport runner:
  - `python tests/run_seamgrim_guard_surface_family_transport_contract_selftest.py`
- transport summary check:
  - `python tests/run_seamgrim_guard_surface_family_transport_contract_summary_selftest.py`
- transport bundle `checks_text`:
  - `family_contract,ddn_exec_server_surface,runtime_fallback_surface`
- progress schema:
  - `ddn.ci.seamgrim_guard_surface_family_transport_contract_selftest.progress.v1`
- progress surface:
  - `stdout`
  - `*.progress.detjson`

## Notes

- 이 family는 gate 비용이 큰 standalone contract들을 `guard surface` 한 줄로 다시 묶는다.
- transport line도 같은 thin layer를 다시 묶되, `run_seamgrim_ci_gate.py`에 직접 배선하지 않고 standalone/progress 면으로 유지한다.
- 현재 포함 범위:
  - `seamgrim_ddn_exec_server_surface_contract`
  - `seamgrim_runtime_fallback_surface_contract`
- optional preflight:
  - `seamgrim_full_gate_surface_contract`
- parent family:
  - `tests/seamgrim_assurance_family/README.md`
  - `python tests/run_seamgrim_assurance_family_selftest.py`
