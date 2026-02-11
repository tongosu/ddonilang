# D-PACK: open_replay_missing

## 목적
- replay 모드에서 로그가 비어 있으면 `E_OPEN_REPLAY_MISSING`이 발생하는지 확인한다.

## 구성
- `input.ddn`: 열림 호출 샘플
- `open.log.jsonl`: 빈 로그
- `golden.jsonl`: 오류 출력 기대
- `tests/README.md`: 수동 실행 가이드
