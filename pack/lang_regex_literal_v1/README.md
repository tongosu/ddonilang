# lang_regex_literal_v1

DEC-REGEX-01(AGE3+) Phase 1 회귀 팩.

## 목표
- `정규식{}` 값 리터럴 파싱/실행 기본 경로 검증
- `정규맞추기/정규찾기/정규캡처하기/정규이름캡처하기/정규바꾸기/정규나누기` 표준 API 검증
- optional named group은 빈 글, no-match named capture는 빈 `짝맞춤{}`로 수렴하는지 검증
- named backreference replace가 `${num}:${word}` 표면으로 결정적으로 동작하는지 검증
- bare named backreference replace도 `$num:$word` 표면으로 같은 결과를 내는지 검증
- capture group이 들어간 패턴으로 `정규나누기` 해도 결과에는 separator capture가 섞이지 않는지 검증
- 잘못된 치환 참조(`${missing}`)가 `E_REGEX_REPLACEMENT_INVALID`로 고정되는지 검증
- `$10` 같은 숫자 치환 참조는 greedy하게 해석되고, capture 10이 없으면 `E_REGEX_REPLACEMENT_INVALID`로 실패하는지 검증
- `${}` 빈 치환 참조, dangling `$`, 큰 숫자 backreference overflow도 모두 `E_REGEX_REPLACEMENT_INVALID`로 고정되는지 검증
- 유효 깃발 조합(`i/m/s`)의 정본 순서와 실행 의미 검증
- 오류 코드 검증:
  - `E_AGE_NOT_AVAILABLE`
  - `E_REGEX_FLAGS_INVALID`
  - `E_REGEX_PATTERN_INVALID`
  - `E_REGEX_REPLACEMENT_INVALID`

## 실행
python tests/run_pack_golden.py lang_regex_literal_v1
