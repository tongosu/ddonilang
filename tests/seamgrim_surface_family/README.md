# Seamgrim Surface Family

## Stable Contract

- parent line:
  - `seamgrim surface family`
- child lines:
  - `tests/seamgrim_bridge_family/README.md`
  - `tests/seamgrim_state_view_boundary_family/README.md`
  - `tests/seamgrim_consumer_surface_family/README.md`
  - `tests/seamgrim_wasm_web_smoke_contract/README.md`
- child checks:
  - `python tests/run_seamgrim_bridge_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_state_view_boundary_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_consumer_surface_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_wasm_web_smoke_contract_selftest.py`
  - `python tests/run_seamgrim_surface_family_selftest.py`
- fixed family line:
  - `bridge/export transport + state/view boundary transport + consumer surface transport + wasm/web smoke contract`

## Stable Bundle Contract

- bundle runner:
  - `python tests/run_seamgrim_surface_family_contract_selftest.py`
- summary check:
  - `python tests/run_seamgrim_surface_family_contract_summary_selftest.py`
- bundle `checks_text`:
  - `bridge_family_transport,state_view_boundary_transport,consumer_surface_transport,wasm_web_smoke_contract,seamgrim_surface_family`
- progress schema:
  - `ddn.ci.seamgrim_surface_family_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport runner:
  - `python tests/run_seamgrim_surface_family_transport_contract_selftest.py`
- transport summary check:
  - `python tests/run_seamgrim_surface_family_transport_contract_summary_selftest.py`
- transport bundle `checks_text`:
  - `family_contract,bridge_family_transport,state_view_boundary_transport,consumer_surface_transport,wasm_web_smoke_contract`
- progress schema:
  - `ddn.ci.seamgrim_surface_family_transport_contract_selftest.progress.v1`
- sanity steps:
  - `seamgrim_surface_family_transport_contract_selftest`
  - `seamgrim_surface_family_transport_contract_summary_selftest`
- direct surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Notes

- 이 family는 seamgrim의 transport 계층 전체를 상위 umbrella 한 줄로 다시 묶는다.
- 현재 포함 범위:
  - `seamgrim_bridge_family_transport_contract`
  - `seamgrim_state_view_boundary_family_transport_contract`
  - `seamgrim_consumer_surface_family_transport_contract`
  - `seamgrim_wasm_web_smoke_contract`
- parent family:
  - `tests/seamgrim_runtime_family/README.md`
  - `python tests/run_seamgrim_runtime_family_selftest.py`
  - `tests/seamgrim_stack_family/README.md`
  - `python tests/run_seamgrim_stack_family_selftest.py`
