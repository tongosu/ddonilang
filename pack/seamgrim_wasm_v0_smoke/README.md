# seamgrim_wasm_v0_smoke

WASM 브릿지 최소 스모크 팩.

- `reset` -> `step` -> `set_param` -> `step` 시나리오를 실행한다.
- 각 `step` 블록 종료 시점의 `state_hash`를 골든으로 고정한다.

검증:

- `python tests/run_seamgrim_wasm_smoke.py seamgrim_wasm_v0_smoke`
