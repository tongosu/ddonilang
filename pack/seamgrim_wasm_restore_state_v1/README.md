# seamgrim_wasm_restore_state_v1

WASM `restore_state` API 계약 검증 팩.

검증 항목:

- `get_state_json()` 스냅샷을 `restore_state()`로 복원할 수 있는가
- 복원 직후 `state_hash`가 스냅샷 해시와 정확히 일치하는가
- 복원 전에 상태를 변경하면 해시가 달라지는가
- 복원 후 schema/tick/hash가 일관적인가

검증:

- `python tests/run_seamgrim_wasm_smoke.py seamgrim_wasm_restore_state_v1`

