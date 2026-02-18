# D-PACK: open_clock_record_replay

## 목적
- `열림.시각.지금.` 호출을 record/replay로 봉인한다.
- replay에서 동일 출력이 재현되는지 확인한다.

## 구성
- `input.ddn`: 벽시계 호출 샘플
- `open.log.jsonl`: replay용 로그(결정적 고정값)
- `tests/README.md`: 수동 실행 가이드
