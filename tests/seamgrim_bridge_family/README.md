# Seamgrim Bridge Family

## Stable Contract

- parent line:
  - `seamgrim bridge family`
- child lines:
  - `tests/seamgrim_graph_bridge_contract/README.md`
  - `tests/seamgrim_graph_api_parity/README.md`
  - `tests/seamgrim_bridge_surface_api_parity/README.md`
  - `tests/seamgrim_space2d_api_parity/README.md`
- child checks:
  - `python tests/run_seamgrim_graph_bridge_contract_selftest.py`
  - `python tests/run_seamgrim_bridge_check_selftest.py`
  - `python tests/run_seamgrim_graph_api_parity_check_selftest.py`
  - `python tests/run_seamgrim_bridge_surface_api_parity_check_selftest.py`
  - `python tests/run_seamgrim_space2d_api_parity_check_selftest.py`
  - `python tests/run_seamgrim_bridge_family_selftest.py`
- fixed family line:
  - `graph bridge contract + bridge hash cross check + graph api parity + bridge surface api parity + space2d api parity`

## Stable Bundle Contract

- bundle runner:
  - `python tests/run_seamgrim_bridge_family_contract_selftest.py`
- summary check:
  - `python tests/run_seamgrim_bridge_family_contract_summary_selftest.py`
- bundle `checks_text`:
  - `graph_bridge_contract,bridge_hash_cross_check,graph_api_parity,bridge_surface_api_parity,space2d_api_parity,seamgrim_bridge_family`
- progress schema:
  - `ddn.ci.seamgrim_bridge_family_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport runner:
  - `python tests/run_seamgrim_bridge_family_transport_contract_selftest.py`
- transport summary check:
  - `python tests/run_seamgrim_bridge_family_transport_contract_summary_selftest.py`
- transport bundle `checks_text`:
  - `family_contract,graph_bridge_contract,bridge_hash_cross_check,graph_api_parity,bridge_surface_api_parity,space2d_api_parity`
- progress schema:
  - `ddn.ci.seamgrim_bridge_family_transport_contract_selftest.progress.v1`
- sanity steps:
  - `seamgrim_bridge_family_transport_contract_selftest`
  - `seamgrim_bridge_family_transport_contract_summary_selftest`
- direct surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Notes

- 이 family는 seamgrim bridge/export line의 핵심 consumer surface를 한 번에 다시 묶는다.
- 현재 포함 범위:
  - `seamgrim.graph.v0` deterministic bridge contract
  - `seamgrim.graph.v0` standalone/API parity
  - `seamgrim.text.v0` / `seamgrim.table.v0` / `seamgrim.structure.v0` standalone/API parity
  - `seamgrim.space2d.v0` standalone/API parity
- parent family:
  - `tests/state_view_hash_separation_family/README.md`
  - `python tests/run_state_view_hash_separation_family_selftest.py`
  - `tests/seamgrim_state_view_boundary_family/README.md`
  - `python tests/run_seamgrim_state_view_boundary_family_selftest.py`
  - `tests/seamgrim_surface_family/README.md`
  - `python tests/run_seamgrim_surface_family_selftest.py`
