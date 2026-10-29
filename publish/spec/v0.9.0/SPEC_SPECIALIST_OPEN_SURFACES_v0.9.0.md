> v0.9.0 로컬 문서판 효력: `publish/spec/CURRENT.md`가 이 family를 지정한 뒤에만 적용됩니다.
> 규범 투영 기준: 내부 v25.151.0 / RD001~709. 내부 현행 v25.153.0 / RD001~711의 전체 투영판은 아닙니다.
> [판본 범위와 내부 후속 효력](판본범위와_내부후속효력_v0.9.0.md)이 아래 고정 원문의 당시 상태 표기보다 현재 판본 범위를 우선 설명합니다.
> 제품 지원·실행·Human·GitHub 게시 승인과 분리합니다. 아래 후보/당시 CURRENT/게시허용 표기는 고정 입력의 역사 상태이며 이번 GitHub 게시 권한이 아닙니다.

<!-- REVIEWED_R21_BODY_START -->
# 전문 기능의 확정 표면과 미완료 경계 — Language SPEC v0.9.0 교정본

> CANDIDATE_R2 / NOT_CURRENT / DOCS_FIRST / INDEPENDENT_REVIEW_PENDING
> 규범 기준: 내부 v25.151.0 / RD001~709. 공개 CURRENT는 v0.8.1로 유지한다.
> 제품 지원·실행·독립 제품검토·공개 게시·Human은 이 문서 후보의 상태와 별개다.

## 상태머신 — RD-448

`상태머신{ 상태=차림[$갈래...]. 처음=$갈래. 전이=차림[(전=..., 후=..., 사건=알림씨, 내용=payloadBinder, 조건=pureCallable, 행동=action)...]. }`는 확정된 작성 형식이다.
이 표기는 계약 설명용 골격이며 실행 예제가 아니다.
`내용`은 사건 payload의 typed binder다. 행동은 없음, named 움직씨 또는 typed PlanLiteral이다.
guard는 pure이며 활성 전이 0개는 무전이, 1개는 상태 변경과 행동을 하나의 atomic candidate로 실행, 2개 이상은 실패다. 행동 실패 시 모두 되돌린다.
계층·병렬·history·timeout·retry·cancel 및 물리 AST/schema/wire/진단은 별도 OPEN이다.

## 글무늬 — RD-458/475

정본 carrier는 `(#또니정규)글무늬{...}`다. 일반 typed callable `맞추기/찾기/나누기/바꾸기`가 각각 전체 일치, 첫 일치의 `(글)있없`, 분리, 치환 역할을 소유한다.
`#regex`, 정규맞추기·정규찾기·정규바꾸기·정규나누기, `정규식{}`는 명시적 판본 이행·역사 입력이며 영구 별칭이 아니다.
RD-475가 허용 패턴·scanner 상태·capture·고정 Unicode profile과 거부 범위를 한정한다. host 정규식의 의미를 상속하지 않으며 backreference/lookaround/recursion/host callback 등은 실패한다.
과거 RD-461의 포괄적인 exact closure를 되살리지 않는다. 제품 지원은 별도다.

## 말무늬·역말무늬 — RD-450

간단한 작성 형식은 `말무늬{ 항목=(무늬="literal {자리:씨}", 대상=callable). }`다.
간단한 작성과 구조화된 말무늬항목은 검증·모호성 검사·승인·판본 봉인을 거쳐 정본 periphery registry가 된다.
정확히 하나의 봉인된 후보만 성공하며 0개·2개 이상은 실패한다. 문자열 순위, AI 확신도, source order가 선택 권위가 되지 않는다.
역말무늬는 deterministic profile presentation이지 역함수나 identity 발급 권위가 아니다.
물리 registry/schema/wire/진단 및 editor/LSP는 OPEN이다.

이 문서는 이미 확정된 의미를 재투영하며 parser 지원·stdlib availability·V1 membership을 승인하지 않는다.

<!-- REVIEWED_R21_BODY_END -->
