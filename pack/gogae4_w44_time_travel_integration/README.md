# D-PACK: gogae4_w44_time_travel_integration

## 목적
- 기록/분기/서사/타임라인을 통합해 시간 여행 파이프라인을 검증한다.

## 구성(권장)
- input.ddn : 세계/시나리오 정의
- sam/      : base/fix 입력 테이프
- expect/   : 기대 결과
- tests/    : 실행/검증 커맨드

## DoD(최소)
- base/branch geoul이 생성되고 replay diff에서 분기가 확인된다.
- branch에서 story/timeline 산출물이 생성된다.
