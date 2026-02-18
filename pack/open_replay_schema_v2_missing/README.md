# D-PACK: open_replay_schema_v2_missing

## 목적
- v2 schema에서도 필수 필드(unix_sec)가 누락되면 `E_OPEN_REPLAY_INVALID`가 발생하는지 확인한다.

## 구성
- `input.ddn`: 시각 호출 샘플
- `open.log.jsonl`: unix_sec 누락 로그
- `golden.jsonl`: 오류 출력 기대
- `tests/README.md`: 수동 실행 가이드
