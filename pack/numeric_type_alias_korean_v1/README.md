# numeric_type_alias_korean_v1

`채비` 타입 핀에서 한국어 별칭/영문 별칭(`string/none/non`)의 런타임 타입검사 계약을 검증하는 팩.

검증:

- `python tests/run_pack_golden.py numeric_type_alias_korean_v1`
- `python tests/run_pack_golden.py --update numeric_type_alias_korean_v1`

포함 범위:

- 한국어 컬렉션 별칭 성공 경로: `목록/모둠/그림표/값꾸러미`
- 영문 `string/none/non` 별칭 성공 경로 (`c02`, `c07`)
- 별칭 타입 핀/초기값 불일치 시 `E_RUNTIME_TYPE_MISMATCH` (`none`, `non` 포함)
