# D-PACK: open_file_read_path_normalize

## 목적
- file_read 경로의 `./` 정규화가 replay 매칭에 반영되는지 확인한다.

## 구성
- `input.ddn`: `./` 경로로 파일 읽기 호출
- `hello.txt`: 입력 파일
- `open.log.jsonl`: 정규화된 key 로그
- `golden.jsonl`: replay 출력 기대
- `tests/README.md`: 수동 실행 가이드
