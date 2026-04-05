# W94 의도서 (최소)

## 목표
- 다중 에이전트 사회 시뮬을 결정적으로 실행하고, 보고서/상태 해시를 고정한다.

## 범위
- agent_id 정렬 스케줄러
- seed 기반 step drift
- 이벤트 3종: `redistribute`, `conflict`, `cooperate`
- 출력 보고서: `ddn.social.report.v1`

## 비범위
- 현실 정책 정합성 모델링
- 외부 네트워크/실시간 개입
