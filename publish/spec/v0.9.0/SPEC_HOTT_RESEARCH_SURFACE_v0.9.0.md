> v0.9.0 로컬 문서판 효력: `publish/spec/CURRENT.md`가 이 family를 지정한 뒤에만 적용됩니다.
> 규범 투영 기준: 내부 v25.151.0 / RD001~709. 내부 현행 v25.153.0 / RD001~711의 전체 투영판은 아닙니다.
> [판본 범위와 내부 후속 효력](판본범위와_내부후속효력_v0.9.0.md)이 아래 고정 원문의 당시 상태 표기보다 현재 판본 범위를 우선 설명합니다.
> 제품 지원·실행·Human·GitHub 게시 승인과 분리합니다. 아래 후보/당시 CURRENT/게시허용 표기는 고정 입력의 역사 상태이며 이번 GitHub 게시 권한이 아닙니다.

<!-- REVIEWED_R21_BODY_START -->
# HoTT 공식 표면과 연구 경계 — Language SPEC v0.9.0 교정본

> CANDIDATE_R2 / NOT_CURRENT / DOCS_FIRST / INDEPENDENT_REVIEW_PENDING
> 규범 기준: 내부 v25.151.0 / RD001~709. 공개 CURRENT는 v0.8.1로 유지한다.
> 제품 지원·실행·독립 제품검토·공개 게시·Human은 이 문서 후보의 상태와 별개다.




RD-508은 다음 세 표면에 한해 과거 RD-342/343/375의 연구 후보 상태를 대체한다.

- typed Path 문맥의 `x =p= y`: Path type
- `|:A:|`: PropTrunc(A)
- `p·q`: 끝점이 맞는 p 다음 q의 Path composition

끝점 불일치나 ordinary multiplication과의 복수 해소는 실패한다.
Proof R1 `=:=`, 일반 equality, product/norm으로 재해석하지 않는다.
`x =-= y` 및 나머지 연구 표면을 위 세 표면의 승인으로 자동 승격하지 않는다.

정확한 의미 계산체계는 기존 SPEC_PROOF_KERNEL_PROFILES_v0.9.0.md를 따른다.
물리 kernel/schema/wire/certificate encoding 및 실행 제품은 별도 OPEN이다.
HOTT-1/2 provider·checker·assumption과 RD-345 demand gate를 유지한다.
Lean 등 외부 도구 활용의 설계 방향을 현재 backend 연결·검증 완료로 주장하지 않는다.
<!-- REVIEWED_R21_BODY_END -->
