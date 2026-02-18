# pack/gogae3_w24_ecs_archetype

W24(ECS/아키타입) D-PACK 스켈레톤이다.

## 목표
- 아키타입 기반 ECS로 전환해도 결정성(state_hash)이 유지됨을 확인한다.
- 대규모 엔티티 처리에서 성능 캡/회귀 기준을 만든다.

## 구성
- input.ddn: 대량 엔티티/컴포넌트 생성 스크립트(초안)
- inputs/: 입력샘 detjson 샘플/스키마
- expect/: state_hash/벤치 결과 기대값(확정 후 추가)
- diag/: 결정성/정렬 위반 오류 정의
- tests/: 골든/벤치 테스트 계획

## 참고
- docs/ssot/walks/gogae3/w24_ecs_archetype/SPEC.md
- docs/context/roadmap/gogae3/PROPOSAL_GOGAE3_PLAN_v20.1.10.md

