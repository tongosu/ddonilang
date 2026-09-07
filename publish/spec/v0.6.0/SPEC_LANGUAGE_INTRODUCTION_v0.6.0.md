# 또니랑 Language SPEC v0.6.0 공개판 — 소개

> 상태: `public current / owner-approved / independently reviewed / docs-first / product-open`
> 내부 투영 기준: `v25.15.0 live current / RD-001~437`
> 공개 current: `v0.6.0`
> 충돌 시 독립 승격된 내부 live SSOT가 우선한다.
> 이 공개판의 문서 current 승격은 제품 parser/runtime 지원을 주장하지 않는다.

## 이번 판본의 성격

v0.6.0은 공개 source 표면의 새 epoch 공개판이다. 외부 사용자가 아직 0명이므로
실재 사용자 호환 의무를 발명하지 않고, 언어의 Definition·결속·선언·형태론·
다중표면을 한 번 정돈한다.

새 source 정본은 다음이다.

- 이름 있는 Definition: `:=`
- ordinary Binding/Rebinding: `<-`
- 완전 선언 머리와 본문의 단일 Definition identity
- 자연어 citation form과 typed morphology form
- 닫힌 surface-shape class와 typed unique resolution
- semantic canon과 presentation canon의 분리

pre-v25 source는 일반 parser의 영구 별칭이 아니다. 옛 source는 판본을 명시한
migration 입력으로만 다루며 모호한 표면은 추측하지 않는다.

## 이번 R2 successor가 더하는 공개 의미

- 한국어·수학·ASCII presentation이 같은 typed semantic identity를 가리키는 경계
- Proof equality와 별도인 HoTT Path identity 연구 축
- 묶음씨 Product definition과 고름씨 Sum definition의 다른 body shape
- `$갈래` owned identity와 schema-only sum body
- target type이 의미를 소유하는 typed `[]` Selection
- `.field`, `$갈래`, `[]`의 역할 분리

HoTT-1/2 exact grammar와 kernel은 demand-gated OPEN이며, 이 문서는 제품 지원을
선언하지 않는다.
