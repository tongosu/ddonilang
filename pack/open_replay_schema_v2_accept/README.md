# D-PACK: open_replay_schema_v2_accept

## 목적
- replay 모드에서 v2 schema(open.clock.v2/open.file_read.v2)를 허용하는지 확인한다.

## 구성
- `input.ddn`: 시각 + 파일 읽기 호출
- `hello.txt`: 입력 파일
- `open.log.jsonl`: v2 schema 로그
- `golden.jsonl`: replay 출력 기대
- `tests/README.md`: 수동 실행 가이드
