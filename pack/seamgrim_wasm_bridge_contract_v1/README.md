# seamgrim_wasm_bridge_contract_v1

WASM 브릿지 계약(`columns`/`set_param`/`reset`) 전용 팩.

검증 항목:

- `get_state_json` raw schema가 `seamgrim.engine_response.v0`인가
- raw payload에 `state`/`view_meta`/`view_hash` 채널이 존재하는가
- `step` 이후 `columns`에 핵심 상태 키가 노출되는가
- `columns`의 `row.length === columns.length`가 유지되는가
- `set_param` 호출이 성공하고 `state_hash`를 반환하는가
- `reset(keep_params=true)`에서 파라미터가 유지되는가
- `reset(keep_params=false)`에서 파라미터가 제거되는가
- `reset keep/drop` 이후에도 `row.length === columns.length` 정합이 유지되는가

검증:

- `python tests/run_seamgrim_wasm_smoke.py seamgrim_wasm_bridge_contract_v1`
