# seamgrim_wasm_streams_serialization_v1

WASM `state.streams` 직렬화 계약 검증 팩.

검증 항목:

- 리스트+사이드카(`_head/_len/_capacity`) 기반 스트림 수집
- 한글 사이드카(`_머리/_길이/_용량`) 기반 스트림 수집
- 맵 기반 스트림(`버퍼/머리/길이/용량`) 수집
- top-level `streams`와 `state.streams` 동형성
- wrapper 정규화 결과의 `streams` 키 동형성

검증:

- `python tests/run_seamgrim_wasm_smoke.py seamgrim_wasm_streams_serialization_v1`
