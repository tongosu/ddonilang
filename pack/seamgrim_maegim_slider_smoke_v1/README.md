# seamgrim_maegim_slider_smoke_v1

WASM `maegim_control_plan`을 웹 `SliderPanel`이 실제로 소비하는지 고정하는 스모크 팩.

검증 항목:

- `hydrateLessonCanon()`이 `maegimControlJson`을 채운다
- `SliderPanel.parseFromDdn()`이 `maegim_control_json`을 우선 사용한다
- 슬라이더 상태 문자열이 `control 매김` 경로를 가리킨다
- `applyControlValuesToDdnText()`가 `매김 {}` 선언값을 다시 쓴다
- 깨진 `maegimControlJson`에서는 legacy ``매김 { 범위: a..b. 간격: c. }`` fallback이 유지된다

검증:

- `python tests/run_seamgrim_wasm_smoke.py seamgrim_maegim_slider_smoke_v1`
