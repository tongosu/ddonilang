# lang_teulcli_parser_parity_v1

- 상태: ACTIVE
- 주제: `tool canon` ↔ `teul-cli canon` frontdoor 파라미터 동등성(parity) 회귀
- SSOT: PLN-20260322-FRONTDOOR-PARITY-CLOSE-01
- 범위:
  - 입력 별칭 → 정본 canon 변환 결과 고정
  - deprecated 경고 코드 emission 고정
  - 두 frontdoor(`tool`, `teul-cli`)가 같은 입력에서 같은 canon bytes + 같은 diag code를 내는 것을 pack-first로 봉인

## 케이스 구성

- `c01_block_header_colon_deprecated`: `채비:` → `채비 {` 정본화 + `W_BLOCK_HEADER_COLON_DEPRECATED` 경고 고정
- `c02_bogae_madang_alias_parity`: `보개장면` → `보개마당` 정본화 + `W_BOGAE_MADANG_ALIAS_DEPRECATED` 경고 고정
- `c03_jjaim_alias_warning_parity`: `구성` → `짜임` 정본화 + `W_JJAIM_ALIAS_DEPRECATED` 경고 고정
- `c04_decl_maegim_alias_parity`: `채비` 선언값의 `조건 {}` 입력 별칭 → `매김 {}` 정본화 고정

## 관련 selftests

- `python tests/run_lang_teulcli_parser_parity_selftest.py`
- `python tests/run_lang_surface_family_contract_selftest.py`
