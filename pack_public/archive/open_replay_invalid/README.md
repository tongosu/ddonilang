# D-PACK: open_replay_invalid

## 목적
- replay 모드에서 잘못된 로그 스키마가 `E_OPEN_REPLAY_INVALID`로 진단되는지 확인한다.

## 구성
- `input.ddn`: 열림 호출 샘플
- `open.log.jsonl`: unix_sec 누락 로그
- `golden.jsonl`: 오류 출력 기대
- `tests/README.md`: 수동 실행 가이드
