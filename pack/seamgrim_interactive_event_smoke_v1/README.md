# seamgrim_interactive_event_smoke_v1

웹 `RunScreen` 입력 브리지(`handleRuntimeInputKeyDown/Up` -> `getStepInput()` -> `stepClientOne()`)가
실제 WASM lesson과 보개 렌더 경로를 바꾸는지 고정하는 스모크 팩.

검증 항목:

- `ArrowRight`/`ArrowLeft`가 보개 원 위치를 좌우로 바꾼다
- `ArrowUp`의 pulse 입력은 `막눌렸나` 1프레임 계약으로만 반응한다
- keyup 이후 held 입력이 제거된다
- `inputEnabled=false`일 때 웹 입력 브리지가 이벤트 주입을 막는다
- 최종 `space2d`가 `renderSpace2dCanvas2d()`로 실제 렌더된다

검증:

- `python tests/run_seamgrim_wasm_smoke.py seamgrim_interactive_event_smoke_v1`
- `tests/seamgrim_wasm_web_smoke_contract/README.md`
- `python tests/run_seamgrim_wasm_web_smoke_contract_selftest.py`
