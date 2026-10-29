> v0.9.0 로컬 문서판 효력: `publish/spec/CURRENT.md`가 이 family를 지정한 뒤에만 적용됩니다.
> 규범 투영 기준: 내부 v25.151.0 / RD001~709. 내부 현행 v25.153.0 / RD001~711의 전체 투영판은 아닙니다.
> [판본 범위와 내부 후속 효력](판본범위와_내부후속효력_v0.9.0.md)이 아래 고정 원문의 당시 상태 표기보다 현재 판본 범위를 우선 설명합니다.
> 제품 지원·실행·Human·GitHub 게시 승인과 분리합니다. 아래 후보/당시 CURRENT/게시허용 표기는 고정 입력의 역사 상태이며 이번 GitHub 게시 권한이 아닙니다.

<!-- REVIEWED_R21_BODY_START -->
# 결정성 계약 — Language SPEC v0.9.0 후속 후보

> CANDIDATE_R2 / NOT_CURRENT / DOCS_FIRST / INDEPENDENT_REVIEW_PENDING
> 규범 기준: 내부 v25.151.0 / RD001~709. 공개 CURRENT는 v0.8.1로 유지한다.
> 제품 지원·실행·독립 제품검토·공개 게시·Human은 이 문서 후보의 상태와 별개다.

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

## 논리 순서·반복 관찰·재생

등록/선언 순서로 의미 후보를 고르지 않는 규칙과, 이미 해소된 프로그램을 원문 순서로 실행하는 규칙은 다르다. RD242 인자 평가 순서를 유지하고 RD709 동일 진입 단위의 root/hook은 원문 순서다. RD696 조회는 같은 admitted snapshot을 소비하지 않는다. RD708의 정확 비교는 관찰·대상/사건 대응·논리 시점·기록 완전성에 결속한다. 실제 저장 정체가 고정된 작품/봉인 기록 재생과, 저장 없는 run의 임시 정체 차이를 구분하며 같은 원문 문자열만으로 모든 state_hash 동일성을 약속하지 않는다.

정확한 발동·저장 재열기·재생·비교 경계는 [lifecycle 계약](SPEC_LIFECYCLE_AND_EXACT_COMPARISON_v0.9.0.md)을 따른다. 근사 비교·무제한 자원·플랫폼 전체 bit 동일성은 새로 보장하지 않는다.

<!-- REVIEWED_R21_BODY_END -->
