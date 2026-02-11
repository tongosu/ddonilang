# D-PACK: open_file_read_key_mismatch

## 목적
- replay 모드에서 file_read key(경로) 불일치가 `E_OPEN_REPLAY_MISSING`으로 진단되는지 확인한다.

## 구성
- `input.ddn`: 파일 읽기 호출 샘플
- `alpha.txt`: 실제 입력 파일
- `open.log.jsonl`: key 불일치 로그
- `golden.jsonl`: 오류 출력 기대
- `tests/README.md`: 수동 실행 가이드
