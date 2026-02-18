# D-PACK: open_file_read_unc_space_special

## 목적
- UNC 경로에 공백/특수문자(+)가 포함되어도 정규화 key로 replay 매칭되는지 확인한다.

## 구성
- `input.ddn`: 공백/특수문자 포함 UNC 경로로 파일 읽기 호출
- `hello space+mix.txt`: 입력 파일
- `open.log.jsonl`: 정규화 key 로그
- `golden.jsonl`: replay 출력 기대
- `tests/README.md`: 수동 실행 가이드
