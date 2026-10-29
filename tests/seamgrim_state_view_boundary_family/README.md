# Seamgrim State View Boundary Family

## Stable Contract

- parent line:
  - `seamgrim state view boundary family`
- child lines:
  - `tests/seamgrim_bridge_family/README.md`
  - `tests/state_view_hash_separation_family/README.md`
  - `tests/seamgrim_view_hash_family/README.md`
- child checks:
  - `python tests/run_seamgrim_bridge_family_transport_contract_selftest.py`
  - `python tests/run_state_view_hash_separation_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_view_hash_family_transport_contract_selftest.py`
  - `python tests/run_seamgrim_state_view_boundary_family_selftest.py`
- fixed family line:
  - `bridge/export transport + state/view separation transport + view_hash consumer transport`

## Stable Bundle Contract

- bundle runner:
  - `python tests/run_seamgrim_state_view_boundary_family_contract_selftest.py`
- summary check:
  - `python tests/run_seamgrim_state_view_boundary_family_contract_summary_selftest.py`
- bundle `checks_text`:
  - `bridge_family_transport,state_view_hash_separation_transport,view_hash_family_transport,seamgrim_state_view_boundary_family`
- progress schema:
  - `ddn.ci.seamgrim_state_view_boundary_family_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport runner:
  - `python tests/run_seamgrim_state_view_boundary_family_transport_contract_selftest.py`
- transport summary check:
  - `python tests/run_seamgrim_state_view_boundary_family_transport_contract_summary_selftest.py`
- transport bundle `checks_text`:
  - `family_contract,bridge_family_transport,state_view_hash_separation_transport,view_hash_family_transport`
- progress schema:
  - `ddn.ci.seamgrim_state_view_boundary_family_transport_contract_selftest.progress.v1`
- sanity steps:
  - `seamgrim_state_view_boundary_family_transport_contract_selftest`
  - `seamgrim_state_view_boundary_family_transport_contract_summary_selftest`
- direct surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Notes

- 이 family는 seamgrim의 `state_hash`/`view_hash` 경계를 exporter, bridge, consumer transport line까지 묶어 다시 확인한다.
- 현재 포함 범위:
  - `seamgrim_bridge_family_transport_contract`
  - `state_view_hash_separation_family_transport_contract`
  - `seamgrim_view_hash_family_transport_contract`
- parent family:
  - `tests/seamgrim_consumer_surface_family/README.md`
  - `python tests/run_seamgrim_consumer_surface_family_selftest.py`
  - `tests/seamgrim_surface_family/README.md`
  - `python tests/run_seamgrim_surface_family_selftest.py`
