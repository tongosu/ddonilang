# seamgrim_wasm_observation_channels_v1

WASM Observation Protocol(`channels`/`row`) 계약 검증 팩.

검증 항목:

- `get_state_json`의 top-level/state 채널이 동일한가
- `step_one`의 top-level/state 채널이 동일한가
- `columns()` 결과와 `channels` 목록이 동일한가
- `row` 길이가 채널 개수와 일치하는가
- `set_param` 이후 파라미터 키가 채널 목록에 노출되는가

검증:

- `python tests/run_seamgrim_wasm_smoke.py seamgrim_wasm_observation_channels_v1`