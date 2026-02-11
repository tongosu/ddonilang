# open_ffi_record_replay

## 목적
- open.ffi 기록/리플레이가 동일 출력으로 재현되는지 확인한다.

## 구성
- `input.ddn`: 호스트 FFI 열림 호출 샘플
- `open.log.jsonl`: record 결과
- `golden.jsonl`: replay 출력 기대
