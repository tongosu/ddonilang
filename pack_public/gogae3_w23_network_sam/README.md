# pack/gogae3_w23_network_sam

W23(네트워크 입력샘 동기) D-PACK 스켈레톤이다.

## 목표
- 네트워크 입력을 샘(InputSnapshot)으로 봉인하고 안정 정렬 규칙을 고정한다.
- 동일 입력샘 리플레이 -> 동일 state_hash를 보장한다.

## 구성
- input.ddn: 데모 월드 스크립트(초안)
- inputs/: 네트워크 입력샘 샘플(형식 확정 후 추가)
- expect/: state_hash 기대값 확정, bogae_hash는 추후 추가
- diag/: 진단/오류 케이스 정의
- tests/: 골든/회귀 테스트 계획

## 참고
- docs/ssot/walks/gogae3/w23_network_sam/SPEC.md
- docs/context/roadmap/gogae3/PROPOSAL_GOGAE3_PLAN_v20.1.10.md

