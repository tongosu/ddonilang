# D-PACK: open_file_read_unc_case

## 목적
- UNC 경로 입력이 정규화 key로 replay 매칭되는지 확인한다.

## 구성
- `input.ddn`: UNC 경로로 파일 읽기 호출
- `hello.txt`: 입력 파일
- `open.log.jsonl`: UNC 정규화 key 로그
- `golden.jsonl`: replay 출력 기대
- `tests/README.md`: 수동 실행 가이드
