# numeric_type_pin_vs_constructor_v1

숫자 exact 계열 타입 핀과 생성자 조합을 검증하는 팩.

검증:

- `python tests/run_pack_golden.py numeric_type_pin_vs_constructor_v1`
- `python tests/run_pack_golden.py --update numeric_type_pin_vs_constructor_v1`

포함 범위:

- 타입 핀 + 정확 수 생성자 성공 경로
- 영문 alias 타입 핀 성공 경로
- 타입 핀/생성자 불일치에서 `E_RUNTIME_TYPE_MISMATCH`
