# pack/gogae3_w25_query_batch

W25(쿼리 기반 군집 실행) D-PACK 스켈레톤이다.

## 목표
- 쿼리 결과의 안정 정렬과 `모두 {}` 실행 결정성을 확인한다.
- 스냅샷 의미론(단계 시작 시 결과 고정)을 보장한다.

## 구성
- input.ddn: 쿼리/모두{} 데모 스크립트(초안)
- inputs/: 입력샘 detjson 샘플/스키마
- expect/: state_hash/쿼리 결과 기대값(확정 후 추가)
- diag/: 쿼리 정렬/스냅샷 위반 오류 정의
- tests/: 골든/성능 테스트 계획

## 참고
- docs/ssot/walks/gogae3/w25_query_batch/SPEC.md
- docs/context/roadmap/gogae3/PROPOSAL_GOGAE3_PLAN_v20.1.10.md

