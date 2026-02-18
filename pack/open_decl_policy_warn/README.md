# D-PACK: open_decl_policy_warn

## 목적
- `#열림 허용(...)`에 없는 open_kind 호출 시 경고가 나오는지 확인한다.

## 구성
- `input.ddn`: 시각만 허용하고 파일읽기를 호출
- `hello.txt`: 파일 읽기 입력
- `open.log.jsonl`: replay용 로그(결정적 고정값)
- `tests/README.md`: 수동 실행 가이드
