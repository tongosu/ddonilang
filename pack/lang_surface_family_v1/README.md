# lang_surface_family_v1

- 상태: ACTIVE
- 주제: 언어 표면(surface) 패밀리 계약 회귀
- SSOT: PLN-20260322-FRONTDOOR-PARITY-CLOSE-01
- 범위:
  - 깨끗한 DDN의 canon 왕복(roundtrip) 동등성 고정
  - 복수 alias 마이그레이션이 한 파일에서 동시에 올바르게 처리됨을 고정
  - `lang_teulcli_parser_parity_v1`와 쌍으로, 표면 패밀리 전체 동작을 봉인

## 케이스 구성

- `c01_canon_roundtrip`: 경고 없는 기본 DDN이 canon을 통과해 동일 출력을 내는지 고정
- `c02_alias_bundle_roundtrip`: `채비:` + `구성` 복수 alias가 동시 마이그레이션(`채비 {` + `짜임`)되는 canon 결과 고정

## 관련 selftests

- `python tests/run_lang_surface_family_selftest.py`
- `python tests/run_lang_surface_family_contract_selftest.py`
- `python tests/run_lang_teulcli_parser_parity_selftest.py`
