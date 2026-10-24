# 또니랑 Language SPEC v0.8.1 공개판 — 소개

> 후속 교정본: 정본 v25.137.0에 따른 설명 교정. CURRENT·제품 지원·독립 PASS 변경 없음.

> 상태: `public current / owner-approved / independently reviewed / docs-first / product-open`
> public current: `v0.8.1`; predecessor: `v0.8.0`
> source internal SSOT: `v25.137.0 / RD-001~690`
> 내부 투영 기준: `v25.137.0 live source / RD-001~690`
> 교정 기준 공개판: `v0.8.0` — 이 파일은 미승격 교정본
> 충돌 시 독립 승격된 내부 live SSOT가 우선한다.
> 이 공개판의 문서 current 승격은 제품 parser/runtime 지원을 주장하지 않는다.

## 읽기 순서

현행 규칙의 reader-first 진입점은 `SPEC_CURRENT_VIEW_v0.8.1.md`다. 기존 상세 본문,
예제, RD trace와 disposition은 삭제되지 않으며 현행뷰가 각 세부 계약으로 연결한다.
RD-570이 좁힌 명시 등록 표면 규칙은 옛 exception-only/morphology-admission 설명보다 우선한다.

## 이번 판본의 성격

v0.8.1은 internal v25.137.0 / RD-001~690의 public-impact rows를 v0.7.0 baseline 위에
current-view로 전면 재투영하는 minor 공개판이다. 공개 의미와 제품·지원·release
상태를 분리하고, 이전 공개판과 후보 계보를 immutable history로 보존한다.

새 source 정본은 다음이다.

- 이름 있는 Definition: `:=`
- ordinary Binding/Rebinding: `<-`
- 완전 선언 머리와 본문의 단일 Definition identity
- 자연어 citation form과 typed morphology form
- 닫힌 surface-shape class와 typed unique resolution
- semantic canon과 presentation canon의 분리

pre-v25 source는 일반 parser의 영구 별칭이 아니다. 옛 source는 판본을 명시한
migration 입력으로만 다루며 모호한 표면은 추측하지 않는다.

## 현행 공개 의미의 주요 축

- 한국어·수학·ASCII presentation이 같은 typed semantic identity를 가리키는 경계
- Proof equality와 별도인 HoTT Path identity와 RD-508의 한정된 공식 기호 표면
- 묶음씨 Product definition과 고름씨 Sum definition의 다른 body shape
- `$갈래` owned identity와 schema-only sum body
- target type이 의미를 소유하는 typed `[]` Selection
- `.field`, `$갈래`, `[]`의 역할 분리

HoTT 의미 계산체계와 RD-508의 세 공식 기호 표면을 물리 kernel 미구현과
구분한다. 나머지 연구 표면을 자동 승인하지 않으며, 구현은 demand gate를 따른다.
이 문서는 제품 지원을 선언하지 않는다.
