# D-PACK: open_replay_schema_v2_extra

## 목적
- v2 schema의 추가 필드(meta)를 무시하고 replay가 통과하는지 확인한다.

## 구성
- `input.ddn`: 시각 + 파일 읽기 호출
- `hello.txt`: 입력 파일
- `open.log.jsonl`: v2 schema + meta 필드 로그
- `golden.jsonl`: replay 출력 기대
- `tests/README.md`: 수동 실행 가이드
