# Specialist-OPEN 표면 — Language SPEC v0.6.0 공개판

아래 항목은 역할·권위 방향만 승인됐으며 exact public grammar가 아니다.

- `상태머신{}`: immutable typed StateMachineSpec 역할. guard pure, selected transition의
  state/action atomic candidate, multiple enabled transition은 fail-closed. exact clauses OPEN.
- RegexIR: `(#regex)글무늬` 입력과 정규맞추기·정규찾기·정규바꾸기·정규나누기 역할.
  matching/capture/replacement/Unicode/position/profile exact contract OPEN.
- `말무늬`: sealed phrase→typed action/event candidate periphery 방향. 실행 권한이 아님.
- `역말무늬`: typed action/API→profile별 deterministic presentation 방향. semantic inverse가 아님.

이 항목의 이름을 본 문서가 parser 지원·stdlib availability·V1 membership으로 만들지 않는다.
