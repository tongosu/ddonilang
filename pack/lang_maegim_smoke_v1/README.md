# lang_maegim_smoke_v1

`매김 {}` 채비 제어 메타의 파서/정본화 표면을 고정하는 스모크 팩.

검증:

- `python tests/run_pack_golden.py lang_maegim_smoke_v1`
- `python tests/run_pack_golden.py --update lang_maegim_smoke_v1`

포함 범위:

- `매김` control JSON 출력
- `조건 {}` 별칭의 `매김 {}` 정규화
- 괄호 없는 초기값 거절
- `간격`/`분할수` 동시 사용 거절
