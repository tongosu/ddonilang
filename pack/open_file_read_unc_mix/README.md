# D-PACK: open_file_read_unc_mix

## 목적
- UNC 경로의 대소문자/슬래시 혼용 입력이 정규화 key로 replay 매칭되는지 확인한다.

## 구성
- `input.ddn`: 혼합 슬래시/대소문자 UNC 경로로 파일 읽기 호출
- `hello.txt`: 입력 파일
- `open.log.jsonl`: 정규화 key 로그
- `golden.jsonl`: replay 출력 기대
- `tests/README.md`: 수동 실행 가이드
