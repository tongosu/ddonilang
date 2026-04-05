# numeric_maegim_binding_v1

숫자 타입 핀과 `매김 {}` 결합 표면을 고정하는 팩.

검증:

- `python tests/run_pack_golden.py numeric_maegim_binding_v1`
- `python tests/run_pack_golden.py --update numeric_maegim_binding_v1`

포함 범위:

- 숫자 타입 선언의 `매김` 제어 JSON 산출
- `조건 {}` 별칭의 `매김 {}` 정규화
- `간격`/`분할수` 동시 사용 거절
- 괄호 없는 초기값 거절
