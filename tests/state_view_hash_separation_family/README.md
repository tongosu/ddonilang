# State View Hash Separation Family

## Stable Contract

- parent line:
  - `state view hash separation family`
- child lines:
  - `pack/seamgrim_wasm_viewmeta_statehash_v1/README.md`
  - `pack/seamgrim_state_hash_view_boundary_smoke_v1/README.md`
  - `pack/seamgrim_wasm_bridge_contract_v1/README.md`
  - `tests/seamgrim_bridge_family/README.md`
- child checks:
  - `python tests/run_seamgrim_wasm_smoke.py seamgrim_wasm_viewmeta_statehash_v1`
  - `python tests/run_pack_golden.py seamgrim_state_hash_view_boundary_smoke_v1`
  - `python tests/run_seamgrim_wasm_smoke.py seamgrim_wasm_bridge_contract_v1`
  - `python tests/run_seamgrim_bridge_family_selftest.py`
  - `python tests/run_state_view_hash_separation_family_selftest.py`
- fixed family line:
  - `wasm viewmeta state_hash/view_hash boundary + state_hash view boundary smoke + wasm bridge raw channels + seamgrim bridge family`

## Stable Bundle Contract

- bundle runner:
  - `python tests/run_state_view_hash_separation_family_contract_selftest.py`
- summary check:
  - `python tests/run_state_view_hash_separation_family_contract_summary_selftest.py`
- bundle `checks_text`:
  - `wasm_viewmeta_statehash,state_hash_view_boundary,wasm_bridge_contract,seamgrim_bridge_family,state_view_hash_separation_family`
- progress schema:
  - `ddn.ci.state_view_hash_separation_family_contract_selftest.progress.v1`
- progress surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Stable Transport Contract

- transport runner:
  - `python tests/run_state_view_hash_separation_family_transport_contract_selftest.py`
- transport summary check:
  - `python tests/run_state_view_hash_separation_family_transport_contract_summary_selftest.py`
- transport bundle `checks_text`:
  - `family_contract,wasm_viewmeta_statehash,state_hash_view_boundary,wasm_bridge_contract,seamgrim_bridge_family`
- progress schema:
  - `ddn.ci.state_view_hash_separation_family_transport_contract_selftest.progress.v1`
- sanity steps:
  - `state_view_hash_separation_family_transport_contract_selftest`
  - `state_view_hash_separation_family_transport_contract_summary_selftest`
- direct surface:
  - `ci gate stdout`
  - `*.progress.detjson`

## Notes

- 이 family는 `state_hash`와 `view_hash`를 분리해서 유지해야 하는 seamgrim consumer 경계를 한 줄로 다시 묶는다.
- 현재 포함 범위:
  - `seamgrim_wasm_viewmeta_statehash_v1`
  - `seamgrim_state_hash_view_boundary_smoke_v1`
  - `seamgrim_wasm_bridge_contract_v1`
  - `seamgrim_bridge_family`
- parent family:
  - `tests/seamgrim_view_hash_family/README.md`
  - `python tests/run_seamgrim_view_hash_family_selftest.py`
  - `tests/seamgrim_state_view_boundary_family/README.md`
  - `python tests/run_seamgrim_state_view_boundary_family_selftest.py`
