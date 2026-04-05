# numeric_sized_variants_v1

숫자 sized variant (`셈수2/4/8`, `바른수1/2/4/8`)의
정본화/런타임 수용 경로를 고정하는 팩.

검증:

- `python tests/run_pack_golden.py numeric_sized_variants_v1`
- `python tests/run_pack_golden.py --update numeric_sized_variants_v1`

포함 범위:

- 선언 타입 핀 정본화(`셈수* -> 수`, `바른수* -> 바른수`)
- 생성자 alias 실행 경로
- 선언 타입 핀 불일치 시 `E_RUNTIME_TYPE_MISMATCH`
