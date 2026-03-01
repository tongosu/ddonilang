# lang_regex_literal_v1

DEC-REGEX-01(AGE3+) Phase 1 회귀 팩.

## 목표
- `정규식{}` 값 리터럴 파싱/실행 기본 경로 검증
- `정규맞추기/정규찾기/정규바꾸기/정규나누기` 표준 API 검증
- 오류 코드 검증:
  - `E_AGE_NOT_AVAILABLE`
  - `E_REGEX_FLAGS_INVALID`
  - `E_REGEX_PATTERN_INVALID`

## 실행
python tests/run_pack_golden.py lang_regex_literal_v1
