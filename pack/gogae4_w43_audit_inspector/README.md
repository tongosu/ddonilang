# D-PACK: gogae4_w43_audit_inspector

## 목적
- geoul query/backtrace로 특정 마디 상태 조회 및 변경 추적을 검증한다.

## 구성(권장)
- input.ddn : 세계/시나리오 정의
- sam/      : 입력 테이프
- expect/   : 기대 결과
- tests/    : 실행/검증 커맨드

## DoD(최소)
- geoul query가 지정 마디의 값을 반환한다.
- geoul backtrace가 값 변경 마디를 나열한다.
