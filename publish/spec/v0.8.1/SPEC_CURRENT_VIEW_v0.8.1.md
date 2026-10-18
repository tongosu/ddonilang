# 또니랑 Language SPEC v0.8.1 — 통합 현행뷰

> 상태: `public current / owner-approved / independently reviewed / docs-first / product-open`
>
> source internal SSOT: `v25.137.0 / RD-001~690`
>
> public predecessor: `v0.8.0`

이 문서는 RD 역사를 시간순으로 재조립하지 않고, v25.137.0에서 지금 적용되는 공개 의미를
한 번에 읽게 하는 진입점이다. 기존 상세 본문·예제·trace·disposition은 삭제하지 않는다.
여기서 후속 결정이 바꾼 정확한 절만 현행 규칙으로 읽고, 나머지 세부 계약은 연결된 상세
문서와 RD를 함께 따른다. 내부 live SSOT와 충돌하면 내부 live SSOT가 우선한다.

## 1. 완전 선언과 명시 등록 표면

완전 선언의 골격은 다음이다.

```ddn
(정적 씨자리·구실·요구) 두고
(실행 받는자리들)
정본이름다{명시적으로 등록한 accepted surface들}
    : 결과씨
    : 셈씨|움직씨
    { 등록 호출 표면 1 }
    { 등록 호출 표면 2 }
:= {
    정의본문
}.
```

- `두고` 앞은 정적 씨자리·구실·요구 문맥이다.
- 두 번째 `(...)`는 실행 받는자리와 public ParamPinIdentity, 지역 binder, 씨, 조사·역할,
  typed default를 결속한다.
- 이름 바로 뒤 `A{A1~...~An}`은 같은 Definition에 명시적으로 등록한 accepted-surface
  집합이다.
- 선언 종류 뒤의 반복 `{...}`는 닫힌 호출 표면이다.
- `:=` 오른쪽 `{...}`는 정의본문이다.
- 결과씨와 `셈씨|움직씨`는 각각 결과 계약과 실행 의미 범주를 결속한다.

세 위치의 중괄호는 서로 바꿀 수 없다. `:=`는 이 머리 전체와 본문을 하나의
DefinitionIdentity로 결속한다. public ParamPinIdentity와 정의 내부 LocalBinderIdentity도
합치지 않는다. 현행 anchor는 RD-317~323이다.

RD-570에 따라 `A{A1~...~An}`은 exception-only inflection이 아니다. parser/compiler는
자연어 어근, 문법성, 규칙성, 파생의 정당성을 판정하지 않는다. `먹{괚괘~히히}`도 exact
registration, typed context, unique resolution, ambiguity gate를 통과하면 허용된다.
morphology profile은 작성 편의 확장 자료일 뿐 semantic identity, admission, veto, 숨은
생성, fallback 권위가 없다. RD-326·RD-328의 옛 exception/morphology-admission 설명은
역사 근거로만 남고 이 절의 현행 판정을 이기지 못한다.

source/declaration/import/registry 순서, fuzzy/prefix/longest match, 자연어 상식, AI는
후보 선택 권위가 아니다. 등록 표면은 임의 parser macro가 아니며 versioned closed
surface class, declared pin identity, typed unique resolution을 따른다.

상세: `SPEC_LANGUAGE_SYNTAX_SUMMARY_v0.8.1.md`, `SPEC_SEMANTIC_CORE_v0.8.1.md`.

## 2. 씨자리·구실·generic

generic type parameter의 정본 개념명은 `씨자리`다. `아무 씨`, `어느 씨든`은 교육 설명어이고
별도 kind나 compiler validation 규칙을 만들지 않는다. `갖추어야함`, `맡음`, `맡아야함`은
각각 구실 서명·대상의 정적 채택·씨자리 요구를 구분한다. 역할 중심 작명 지침은 의미 권위가
아니다. 현행 anchor는 RD-234~238이다.

generic/type-constructor Definition, ordered applied type arguments, 구실 Definition,
대상-구실 implementation binding/provider, specialization은 서로 다른 역할이다. 과거 generic
전용 hash/schema는 HISTORY_ONLY이며 부활하지 않는다. 정확한 physical field·byte layout·hash는
OPEN이다. 여러 구실 요구 묶음은 별도 구실 identity를 만들지 않고 canon에서 단일 요구문
반복으로 펼친다(RD-269).

V1 inference는 finite callable과 stable pin mapping 뒤 실제로 결속된 required execution
input에서만 시작한다. optional pin, callable default, result place, return/assignment target,
expected context, 자연어 모양, locale, AI는 inference 권위가 아니다. 해답 0/1/2+는 각각
UNRESOLVED/SUCCESS/AMBIGUOUS이며 완전 해소된 후보만 최종 호출 해소에 들어간다(RD-572).

## 3. meta-program과 반영 경계

`지음씨`는 compile/tool phase 전용 typed callable kind이고 runtime callable/value가 아니다.
phase-only `...보기` family는 닫힌 typed view만 제공하며 raw source, compiler AST, I/O,
환경, 시간, 난수, network, process, host callback, world state를 의미 입력으로 쓰지 않는다.
적용은 최상위 `지음 { 대상=..., 원형=..., 설정=(이름: 값) }.` 요청이며 생성 결과는 ordinary
resolve·type·generic·구실·effect·canon·lock을 다시 통과한다. typed 후보 전체가 성공한 뒤에만
원자 합성하고 실패 시 부분 게시하지 않는다(RD-246~248).

용어 projection이나 AI/도구의 초안은 parser alias, keyword, type, AST kind, wire rename,
semantic authority를 만들지 않는다(RD-516). MathIR·PropositionIR·StructureIR·PathIR·RegexIR·
ExecutionIR은 family별 owner를 유지하며 UniversalIR/CoreIR을 만들지 않는다(RD-603).
V1 self-host 구현과 일정은 OPEN이고 Stage0 Rust와 미래 Stage1은 physical AST 동일성이 아니라
semantic equivalence로 비교한다(RD-605, RD-621).

## 4. 수학·증명·구조

Typed MathIR은 수식 구조, PropositionIR은 truth-bearing proposition 구조, Proof/Certificate는
전제·추론·검증 증거를 소유한다. 문자열, 외부 CAS/prover AST, 화면 표시는 identity가 아니다.
외부 backend와 AI는 후보·계산·증거 제공자이며 의미 권위가 아니다(RD-140~147, RD-168~171,
RD-210, RD-272~279).

`수식{E}`와 `세움{P}`는 각각 MathIR·PropositionIR을 구성할 뿐 계산·정리·증명을 실행하지
않는다. 계산은 명시 `풀기`가 소유한다(RD-506). 수학 표면은 완전 선언의 registered
multisurface를 재사용하며 별도 notation language/operator macro가 아니다. binder와 placeholder는
spelling이 아니라 BinderIdentity/ParamPinIdentity와 lexical scope를 따르고 capture는 fail-closed한다
(RD-507).

StructureIR, MathIR, PropositionIR, PathIR, proof artifact는 다른 family다. 한 artifact를 다른
family나 proof로 암묵 승격하지 않는다(RD-271, RD-603). HoTT의 공식 Path·PropTrunc·composition
표면과 constructive dependent core의 의미는 상세 proof 문서가 소유한다. exact physical
schema·wire·kernel 제품과 demand gate는 별도다. Proof equality `=:=`를 HoTT higher path로
재해석하지 않는다.

상세: `SPEC_PROOF_KERNEL_PROFILES_v0.8.1.md`, `SPEC_HOTT_RESEARCH_SURFACE_v0.8.1.md`.

## 5. DDL·LanguageProfile·별칭·조사

DDL은 구현 본문의 번역이나 새 callable 생성이 아니라 하나의 기존 DefinitionIdentity에
결속된 언어별 공개 선언 투영이다. RD-537의 optional
`옮김=(대상말씨=<LanguageProfileIdentity>, 대상판본=<exact LanguageProfileVersion>)`은 현재
`쓰임` binding record와 bound ResolvedModuleIdentity의 exported DDL surface만 활성화한다.
별도 `(주인=..., 대상말씨=..., 대상판본=..., 항목=차림[...]) 옮김.` artifact는 public
DefinitionIdentity와 ParamPinIdentity에 결속된 projection을 담는다. source/import order,
latest/range/default profile, hidden current는 활성화 권위가 아니다.

DDL은 type parameter·요구, ParamPinIdentity 집합과 의미 순서, pin type/default, result/effect/kind를
보존하고 구현 본문을 소유하지 않는다. 언어별 occurrence order와 semantic pin order는 다를 수
있다. 정확히 ASCII 점 세 개인 `:={...}`만 해소된 DDL payload 자리에서 callable body 또는
immutable initializer 생략을 뜻한다. ordinary `.ddn`에는 생략 의미가 없고 public type·field·
branch·default schema를 숨길 수 없다(RD-571, RD-574~577).

`.ddn`은 파일당 단일 언어이고 `.ddl`·`.옮김`만 서로 다른 exact profile의 여러 top-level
section을 가질 수 있다. 동일 profile 중복, unknown profile, generic Quechua/ambiguous `qu`는
fail-closed한다. apparent header가 문자열·주석·nested construct·Definition body 안에 있으면
section boundary가 아니다.

각 기존 LanguageProfileIdentity는 canonical header 하나와 유한한 explicit alias 집합 하나를
가진다. exact-match alias는 같은 profile identity를 가리키며 canon은 canonical header만 낸다.
fuzzy/prefix/locale/source/import/registry-order 추측, cross-profile 중복, alias-specific identity,
language-specific parser universe는 금지한다(RD-628).

LanguageProfile은 identity/version/status/native name/specific extension/grammar/formatter/diagnostic
profile을 소유하지만 CLI spelling은 소유하지 않는다(RD-529). 이 소유권은 RD-570의 제한 아래
읽는다. morphology profile은 명시 등록 표면의 admission/veto/숨은 생성/fallback/자연어 문법
판정 권위가 아니다.

조사·relation surface는 전역 자연어 semantic-role dictionary가 아니다. definition-owned family는
해당 Definition과 ParamPinIdentity에 결속되고 hybrid family는 core grammar owner와 definition
projection owner를 분리한다. compiler는 PURPOSE/RECIPIENT/SOURCE 같은 ontology를 만들지 않는다
(RD-552). `마다`는 RD-201 iteration을 설명하지만 `들`의 유무는 collection/plurality/iterability/
binder identity 권위가 아니며 자동 단복수화·사전·통계·AI 기반 binder 생성을 금지한다(RD-573).

선언작성 도구는 원본 선언 골격을 추출할 수 있고 사람/AI는 언어별 표면 초안을 만들 수 있다.
결정적 검증기는 기존 identity/type/default/result/effect/kind와 public schema를 비교한다. AI는
의미 발급 권위가 아니다. exact source binding을 유지하고 명시 갱신 때 공개 계약 차이를 검사한다.
내부 구현 변경만으로 전부 재번역하지 않으며 pin/type/default/effect 변경은 영향 검사를 한다.
automatic latest/fallback은 없다.

exact DDL grammar·escaping·header registry·physical AST/schema/wire·stale 판정·parser/canon/IDE corpus는
OPEN이다. 제품 지원 언어 수, 문서 번역 수, registered profile 수, native Human PASS 수는 서로
다른 분모다.

상세: `SPEC_KOREAN_MASTER_AND_LOCALIZATION_AUTHORITY_v0.8.1.md`.

## 6. 전문 표면과 공개 안정 경계

regex, 수학, HoTT, specialist surface는 각 owner와 demand gate를 따른다. compact syntax가
있다고 default activation, broad runtime support, public support 또는 release membership을 뜻하지
않는다. source surface, semantic contract, physical schema/wire, product landing, official
Native/WASM/Workbench parity, release/Human PASS를 분리한다.

V1 Basic Tensor의 공개 원소 TypeIdentity는 canonical `셈수`이고 rank-0 Tensor는 scalar가 아니다.
VC17 REQUIRED15는 exact empty/search/replace/transform/join/concat 법칙과 sealed named PURE callable
reference 경계를 따른다. 이는 해당 제품/complete-row/release PASS를 자동 생성하지 않는다.

상세: `SPEC_SPECIALIST_OPEN_SURFACES_v0.8.1.md`, `SPEC_STABLE_SUPPORT_SURFACE_v0.8.1.md`,
`SPEC_TENSOR_R1_v0.8.1.md`.

## 7. 결정성·실행·게시

RD-181의 다섯 축은 `결과 / 성능 / 외부유입 / 증거·재생 / 자원`이다. `증거·재생`은 artifact·wire
봉인을 하위 계약으로 갖는 하나의 결합 축이다. source 생략은 canon/typed IR에서 versioned
profile로 펼치며 실행계약 identity는 state_hash 밖이지만 입력 동일성의 필수 축이다.

인자 식은 RD-242에 따라 원문 순서로 평가한다. 이름·조사·surface canon이 평가 순서를 바꾸지
않는다. physical thread/worker/GPU/network completion order, renderer repair, wall-clock은 별도 sealed
권위 없이 의미 순서를 소유하지 않는다. 실패·abort·publish 실패는 staged patch/outbox와 success
candidate를 폐기하고 partial publication을 만들지 않는다.

상세: `SPEC_DETERMINISM_CONTRACT_v0.8.1.md`, `SPEC_CALLABLE_RESULT_FAILURE_R1_v0.8.1.md`.

## 8. 설계·제품·지원 상태

이 문서의 의미 설명은 제품 구현 완료 선언이 아니다. 각 항목은 최소 다음 축을 분리한다.

1. semantic/source contract
2. physical schema/wire/registry
3. product implementation
4. independent product review
5. public support/readiness
6. publication/release/Human PASS

미구현 의미를 문서에서 지우지 않는다. 고위험 설계는 작은 실험으로 확인할 수 있지만 실험·초안·
AI 결과는 SSOT나 제품 PASS를 대신하지 않는다. 모든 번역 문서를 완성할 때까지 독립 제품 개발을
멈추지도 않는다.

## 9. 보존·대체·OPEN 행선지

| 범위 | 현행 처분 | anchor/상세 |
|---|---|---|
| 완전 선언과 단일 Definition identity | 유지 | RD-317~323 / 문법 요약 |
| exception-only·morphology-admission 해석 | RD-570으로 대체, 역사만 보존 | RD-326·328·529 → RD-570 |
| explicit registered accepted-surface set | 현행 | RD-570 |
| 씨자리·구실·specialization | 유지 | RD-234~238, RD-269, RD-572 |
| typed compile-time generation | 유지 | RD-246~248 |
| family-preserving IR·self-host firewall | 유지 | RD-516, RD-603, RD-605, RD-621 |
| MathIR·PropositionIR·Proof·Structure 분리 | 유지 | RD-140~147, RD-168~171, RD-210, RD-271~279, RD-506~515 |
| DDL activation·projection·omission·multi-profile | 유지 | RD-537, RD-571, RD-574~577 |
| relation은 자연어 role dictionary가 아님 | 유지 | RD-552 |
| `마다`와 plurality 비권위 | 유지 | RD-573 |
| canonical header와 explicit aliases | 유지 | RD-628 |
| exact physical grammar/schema/wire/product | OPEN 유지 | 각 owner의 PRODUCT/PHYSICAL OPEN |
| 과거 공개판·후보·실패판 | immutable history | manifest/trace/disposition |

이 현행뷰가 기존 상세 본문을 대체하지 않으므로 기존 항목의 행선지 없는 삭제는 0이다.
새 semantic identity, public syntax, physical wire, 제품 지원, release 상태도 만들지 않는다.
