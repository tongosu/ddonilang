# seamgrim_wasm_canon_contract_v1

WASM Step 0 canon API (`flat_json`/`maegim_plan`/`alrim_plan`) 계약 팩.

검증 항목:

- `wasm_canon_flat_json()`이 `ddn.guseong_flatten_plan.v1`를 반환하는가
- flatten 결과의 `instances`/`links`/`topo_order`가 기대값과 맞는가
- `wasm_canon_maegim_plan()`이 `ddn.maegim_control_plan.v1`를 반환하는가
- `controls` 이름/decl kind/split-count 축이 기대값과 맞는가
- `wasm_canon_alrim_plan()`이 `ddn.alrim_event_plan.v1`를 반환하는가
- `handlers`의 kind/scope/order 축이 기대값과 맞는가

검증:

- `python tests/run_seamgrim_wasm_smoke.py seamgrim_wasm_canon_contract_v1`
