# D-PACK: open_replay_schema_v2_meta_scalar

## 목적
- v2 schema의 meta가 문자열/숫자 스칼라인 경우에도 replay가 통과하는지 확인한다.

## 구성
- `input.ddn`: 시각 + 파일 읽기 호출
- `hello.txt`: 입력 파일
- `open.log.jsonl`: v2 schema + meta scalar 로그
- `golden.jsonl`: replay 출력 기대
- `tests/README.md`: 수동 실행 가이드
