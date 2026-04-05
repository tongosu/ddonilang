# D-PACK: diag_fixit_coverage_v1

## 목적
- SSOT `PLN-20260322-DIAG-FIXIT-COVERAGE-01` 기준으로 `teul-cli canon --fixits-json` 대표 경로를 고정한다.
- 성공/실패 경로 모두에서 fixit payload가 결정적으로 생성되는지 검증한다.

## 구성
- `input_legacy_terms.ddn`: legacy term 치환 3건.
- `input_header_colon.ddn`: deprecated block header colon 삭제 제안.
- `input_jjaim_alias.ddn`: `구성 -> 짜임` 치환 제안.
- `input_maegim_grouped.ddn`: 매김 grouped value 보정 제안.
- `input_expected_rparen.ddn`: 닫는 `)` 삽입 제안.
- `input_expected_rbrace.ddn`: 닫는 `}` 삽입 제안.

## 검증
- `python tests/run_pack_golden.py --manifest-path tools/teul-cli/Cargo.toml diag_fixit_coverage_v1`
- `python tests/run_pack_golden.py --manifest-path tools/teul-cli/Cargo.toml --update diag_fixit_coverage_v1`
