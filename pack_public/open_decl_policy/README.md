# D-PACK: open_decl_policy

## 목적
- #열림 허용(...) 지시문으로 open 종류 선언을 확인한다.
- replay에서 동일 출력이 재현되는지 확인한다.

## 구성
- input.ddn: open 허용 지시문 + 시각 호출
- open.log.jsonl: replay용 로그(결정적 고정값)
- 	ests/README.md: 수동 실행 가이드
