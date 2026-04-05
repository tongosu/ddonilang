# seamgrim_wasm_web_smoke_contract_v1

`seamgrim wasm web smoke contract` selftest의 stdout surface를 relative path 기준으로 고정하는 pack.

## 계약
- `run_seamgrim_wasm_web_smoke_contract_selftest.py`는 결정적인 stdout 한 줄을 재생성해야 한다.
- selftest는 `seamgrim_wasm_v0_smoke`, `seamgrim_interactive_event_smoke_v1`, `seamgrim_temp_lesson_smoke_v1`, `seamgrim_moyang_render_smoke_v1`의 expected surface 계약이 유지됨을 검증해야 한다.
- `run_seamgrim_wasm_web_smoke_contract_pack_check.py`는 selftest stdout과 실제 wasm/web smoke 실행 stdout을 모두 pack expected와 비교해야 한다.
- 실행:
  - `python tests/run_seamgrim_wasm_web_smoke_contract_pack_check.py`

## 구성
- `expected/seamgrim_wasm_web_smoke_contract.stdout.txt`
- `expected/seamgrim_wasm_web_real_smoke.stdout.txt`
