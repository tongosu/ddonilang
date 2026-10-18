# HoTT 공식 표면과 연구 경계 — Language SPEC v0.8.1 교정본

> 상태: `public current / owner-approved / independently reviewed / docs-first / product-open`
> public current: `v0.8.1`; predecessor: `v0.8.0`
> source internal SSOT: `v25.137.0 / RD-001~690`

> 후속 교정본: 정본 v25.137.0에 따른 설명 교정. CURRENT·제품 지원·독립 PASS 변경 없음.

RD-508은 다음 세 표면에 한해 과거 RD-342/343/375의 연구 후보 상태를 대체한다.

- typed Path 문맥의 `x =p= y`: Path type
- `|:A:|`: PropTrunc(A)
- `p·q`: 끝점이 맞는 p 다음 q의 Path composition

끝점 불일치나 ordinary multiplication과의 복수 해소는 실패한다.
Proof R1 `=:=`, 일반 equality, product/norm으로 재해석하지 않는다.
`x =-= y` 및 나머지 연구 표면을 위 세 표면의 승인으로 자동 승격하지 않는다.

정확한 의미 계산체계는 기존 SPEC_PROOF_KERNEL_PROFILES_v0.8.1.md를 따른다.
물리 kernel/schema/wire/certificate encoding 및 실행 제품은 별도 OPEN이다.
HOTT-1/2 provider·checker·assumption과 RD-345 demand gate를 유지한다.
Lean 등 외부 도구 활용의 설계 방향을 현재 backend 연결·검증 완료로 주장하지 않는다.
