# lang_unit_temp_smoke_v1

온도 단위 `@K/@C/@F`의 파싱/정규화/차원 오류 표면을 고정하는 스모크 팩.

검증:

- `python tests/run_pack_golden.py lang_unit_temp_smoke_v1`
- `python tests/run_pack_golden.py --update lang_unit_temp_smoke_v1`

포함 범위:

- Kelvin 리터럴 출력
- Celsius/Fahrenheit 와 Kelvin 동치 비교
- Kelvin 산술
- 혼합 온도 합산
- 차원 불일치/미지원 단위 오류
