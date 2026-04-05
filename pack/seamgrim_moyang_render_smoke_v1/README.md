# seamgrim_moyang_render_smoke_v1

`모양 {}` 출력이 WASM에서 `space2d.shape`로 나오고, 웹 canvas 렌더러가 반지름/색 차이를 실제로 소비하는지 고정하는 스모크 팩.

검증 항목:

- `모양 {}` 원 primitive가 `space2d.shape`의 circle로 복원된다
- 반지름만 다른 입력은 같은 `state_hash`를 유지하면서 다른 shape 반지름을 낸다
- 색만 다른 입력은 같은 `state_hash`를 유지하면서 다른 fill 색을 낸다
- 웹 `renderSpace2dCanvas2d()`가 세 입력 모두에서 실제 circle draw 경로를 탄다

검증:

- `python tests/run_seamgrim_wasm_smoke.py seamgrim_moyang_render_smoke_v1`
- `tests/seamgrim_wasm_web_smoke_contract/README.md`
- `python tests/run_seamgrim_wasm_web_smoke_contract_selftest.py`
