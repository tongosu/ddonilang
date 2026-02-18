# D-PACK: gogae4_w37_memory_seed

## 목적
- 기억씨(events) append-only 기록이 state_hash에 포함됨을 검증한다.
- 마지막기억(kind) 조회가 최신 이벤트를 반환함을 확인한다.

## 구성(권장)
- input.ddn : 세계/시나리오 정의
- expect/   : 기대 결과
- tests/    : 실행/검증 커맨드

## DoD(최소)
- 마지막기억 출력이 기대한 이벤트와 일치한다.
- state_hash가 결정적으로 고정된다.
