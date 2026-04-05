# D-PACK: diag_fixit_json_schema_v1

## 목적
- `fixits-json` 스키마 v1의 핵심 변형을 고정한다.
- `[]`, `replace`, `insert`, `delete`와 optional `old/new/note` 조합을 회귀 검증한다.

## 구성
- `input_empty_ok.ddn`: 빈 배열.
- `input_jjaim_alias.ddn`: `replace` + `old/new`.
- `input_expected_rbrace.ddn`: `insert` + `new/note`.
- `input_block_header_forbidden.ddn`: `delete` + `old/note`.

## 검증
- `python tests/run_pack_golden.py --manifest-path tools/teul-cli/Cargo.toml diag_fixit_json_schema_v1`
- `python tests/run_pack_golden.py --manifest-path tools/teul-cli/Cargo.toml --update diag_fixit_json_schema_v1`
