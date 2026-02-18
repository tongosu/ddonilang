# D-PACK: open_file_read_abs_path

## 목적
- file_read가 절대 경로 입력에서도 정규화된 key로 replay 매칭되는지 확인한다.

## 구성
- `hello.txt`: 입력 파일
- `open.log.jsonl`: 정규화 key 로그
- `golden.jsonl`: 절대 경로 입력을 포함한 replay 기대
- `tests/README.md`: 수동 실행 가이드
