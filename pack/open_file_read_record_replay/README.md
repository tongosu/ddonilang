# D-PACK: open_file_read_record_replay

## 목적
- `열림.파일.읽기(경로=...)` 호출을 record/replay로 봉인한다.
- replay에서 동일 출력이 재현되는지 확인한다.

## 구성
- `input.ddn`: 파일 읽기 호출 샘플
- `hello.txt`: 읽기 대상 파일
- `open.log.jsonl`: replay용 로그(결정적 고정값)
- `tests/README.md`: 수동 실행 가이드
