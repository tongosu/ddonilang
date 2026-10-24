# 결정성 계약 — Language SPEC v0.8.1 공개판

> 후속 교정본: 정본 v25.137.0에 따른 설명 교정. CURRENT·제품 지원·독립 PASS 변경 없음.

> 상태: `public current / owner-approved / independently reviewed / docs-first / product-open`
> public current: `v0.8.1`; predecessor: `v0.8.0`
> source internal SSOT: `v25.137.0 / RD-001~690`
> 내부 투영 기준: `v25.137.0 live source / RD-001~690`
> 교정 기준 공개판: `v0.8.0` — 이 파일은 미승격 교정본
> 충돌 시 독립 승격된 내부 live SSOT가 우선한다.
> 이 공개판의 문서 current 승격은 제품 parser/runtime 지원을 주장하지 않는다.

## source 해석

- Definition과 Binding/Rebinding을 분리한다.
- morphology profile은 version-pinned다.
- surface-shape class와 presentation profile은 version-pinned다.
- 후보가 0개 또는 복수면 fail-closed한다.
- source/declaration order나 AI 추측으로 모호성을 해소하지 않는다.

## migration

migration은 원본 판본과 source bytes를 명시적으로 결속한다. 변환이 유일하지
않으면 멈추고 사람에게 선택을 요구한다. migration 결과는 새 입력 identity로
다시 검사한다.

## 추가 결정성 경계

- 수학 surface는 role·type으로 유일 결속하며 source/declaration order를 쓰지 않는다.
- HoTT provider·profile·checker·assumption은 receipt에 명시적으로 봉인한다.
- 갈래 ordinal과 field authored order를 semantic priority로 쓰지 않는다.
- tuple index는 결과씨가 위치별로 다르면 compile-time static이어야 한다.
- map missing key를 default나 option으로 조용히 바꾸지 않는다.
- text selection은 Unicode grapheme cluster 기준이다.

## Selection 결정성

- 차림 index/range는 0-based이며 음수 뒤세기·silent clipping·silent default가 없다.
- map missing key를 default나 `(V)있없`으로 자동 변환하지 않는다.
- `A[i,j]`와 `A[i][j]`를 source order·hidden priority로 동치화하지 않는다.
- 순서 없는 모음에 numeric order를 발명하지 않는다.
- constructor·postfix selection·Proof projection은 glyph 유사성으로 합치지 않는다.
- 후보가 0개 또는 복수이면 fail-closed한다.

## 동일 조건과 평가 순서

RD-181의 다섯 축은 정확히 `결과 / 성능 / 외부유입 / 증거·재생 / 자원`이다.
`증거·재생`은 두 축이 아니라 artifact·wire 봉인을 하위 계약으로 갖는 하나의 결합 축이다.
실행계약 identity는 입력 동일성의 필수 축이며 state_hash와 별개다. 따라서 프로그램
문자열이나 state_hash 하나만 같다고 모든 실행 조건이 같다고 판정하지 않는다.
본 문서는 임의 플랫폼 사이의 bit-identical 실행이나 무제한 자원을 보장하지 않는다.

RD-242에서 조사·이름은 pin identity를 결속하지만 인자 식은 원문 순서로 평가한다.
후보 선택의 source-order fallback 금지와 평가 순서는 서로 다른 규칙이다.
정본화가 인자 평가 순서를 바꾸어서는 안 된다.
