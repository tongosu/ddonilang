# D-PACK: gogae4_w36_trace_tier

## 목적
- trace tier(T-OFF~T-FULL)에 따라 로그 상세도/크기가 결정적으로 달라짐을 확인한다.
- 동일 입력에서 state_hash 스트림은 동일해야 한다.

## 구성(권장)
- input.ddn : 세계/시나리오 정의
- expect/   : 기대 결과
- tests/    : 실행/검증 커맨드

## DoD(최소)
- T-OFF/T-FULL 실행에서 state_hash가 동일하다.
- audit.ddni 크기(audit_size)가 tier에 따라 다르다.
