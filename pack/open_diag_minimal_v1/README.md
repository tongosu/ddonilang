# open_diag_minimal_v1

Open diag 최소 계약을 검증하는 작은 팩이다.

## 목표
- open 호출 1건당 `geoul.diag.jsonl`에 1레코드 기록.
- det_tier/trace_tier enum 강제(AGE2).
- replay 모드에서도 diag 기록 유지.

## 입력
- `input.ddn`: `열림.시각.지금` 1회 호출.

## 사용 예시
```bash
teul-cli run input.ddn --open record --open-log open.log.jsonl --diag geoul.diag.jsonl --trace-tier T-OFF
teul-cli run input.ddn --open replay --open-log open.log.jsonl --diag geoul.diag.jsonl --trace-tier T-OFF
```
