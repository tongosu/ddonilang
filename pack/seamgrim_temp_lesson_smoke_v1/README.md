# seamgrim_temp_lesson_smoke_v1

WASM 온도 문자열 출력과 웹 표 렌더를 함께 고정하는 스모크 팩.

검증 항목:

- 전처리된 lesson 본문에 `@.1C`/`@.1F` 포맷이 남아 있다
- WASM 실행 결과가 `80.0@C`, `176.0@F` 같은 온도 문자열을 만든다
- `보개_출력_줄들`의 `table.row` 토큰에서 온도 표 행을 복원할 수 있다
- 웹 `renderRuntimeTable()`가 `@C/@F` 문자열을 그대로 HTML table로 렌더한다

검증:

- `python tests/run_seamgrim_wasm_smoke.py seamgrim_temp_lesson_smoke_v1`
- `tests/seamgrim_wasm_web_smoke_contract/README.md`
- `python tests/run_seamgrim_wasm_web_smoke_contract_selftest.py`
