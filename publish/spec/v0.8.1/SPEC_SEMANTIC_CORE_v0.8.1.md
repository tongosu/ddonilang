# 의미 핵심 — Language SPEC v0.8.1 공개판

> 후속 교정본: 정본 v25.137.0에 따른 설명 교정. CURRENT·제품 지원·독립 PASS 변경 없음.

> 상태: `public current / owner-approved / independently reviewed / docs-first / product-open`
> public current: `v0.8.1`; predecessor: `v0.8.0`
> source internal SSOT: `v25.137.0 / RD-001~690`
> 내부 투영 기준: `v25.137.0 live source / RD-001~690`
> 교정 기준 공개판: `v0.8.0` — 이 파일은 미승격 교정본
> 충돌 시 독립 승격된 내부 live SSOT가 우선한다.
> 이 공개판의 문서 current 승격은 제품 parser/runtime 지원을 주장하지 않는다.

## Definition identity

Definition은 선언 머리와 본문을 결속한 이름 있는 의미 객체다. alternate lexical
form이나 presentation surface는 별도 Definition을 만들지 않는다.

## Binding/Rebinding

ordinary binding과 rebinding은 값의 현재 결속을 소유한다. persistent update는
새 값을 만든 뒤 최종 root를 `<-`로 다시 결속한다. Definition과 별도 typed
semantic node를 사용한다.

## 표면과 의미

명시 등록 accepted-surface 집합과 closed multisurface는 하나의 semantic identity에 이르는
결정적 표면이다. RD-570에 따라 이름 뒤 `{...}`는 exception-only inflection이 아니며,
compiler는 자연어 어근·문법성·규칙성·파생 관계를 판정하지 않는다. morphology profile은
작성 편의일 뿐 admission·veto·hidden generation·fallback 권위가 없다. 후보 선택·pin identity
결정에서 source/declaration/import/registry order, hidden priority, fuzzy match, AI의 자연어
상식은 authority가 아니다.

이는 인자 식의 평가 순서를 없애는 규칙이 아니다. RD-242에 따라 인자 식은 원문
순서로 평가하고 호출마다 fresh callee pin을 결속한다. 이름형·조사형 결속은 pin
identity를 따르되 정본화가 평가 순서를 바꾸지 않는다. ordinary pin은 caller 저장
위치의 alias가 아니며 암묵 ref/out/inout이 없다. 따라서 효과가 있는 인자 식의
작성 순서를 바꾼 두 호출까지 무조건 같은 결과라고 주장하지 않는다.

## 비파급

source epoch 변경은 state, receipt, lock 또는 wire identity의 폐기를 뜻하지
않는다. 이 축은 별도 호환성 감사와 owner 결정이 필요하다.

## 수학 다중표면과 HoTT 방화벽

한국어·수학·ASCII source artifact는 다를 수 있지만 typed resolution 뒤 같은 의미면
semantic IR identity와 hash는 하나다. Proof R1 equality `=:=`를 HoTT higher path로
재해석하지 않는다. HoTT Path는 Type-valued 별도 identity family다.
정확한 의미 계산체계는 SPEC_PROOF_KERNEL_PROFILES_v0.8.1.md를 따른다.
RD-508의 Path·PropTrunc·composition 표면은 확정됐지만, 물리 schema·wire·kernel
제품 구현과 demand gate는 별도다. 의미 계산체계까지 일괄 OPEN으로 되돌리지 않는다.

RD-506의 `수식{E}`와 `세움{P}`는 각각 MathIR과 PropositionIR 구성이다.
구성만으로 수치평가·정리·증명이 실행되지 않는다. 계산은 명시적인 `풀기`가
담당하며, 명제 구성 성공을 증명 성공으로 표시하지 않는다.

## Product·Sum·Selection

ProductTypeDefinition, SumTypeDefinition, CallableDefinition은 하나의 Definition 경계를
공유하지만 typed category와 body shape는 다르다. Selection(Target, Selector)은
TargetType의 selector/result/failure/update 계약으로 해소하며 0개·복수 후보는
fail-closed한다.

## OwnedBranchIdentity 계보

고름씨의 `$갈래` 선언, `소유씨.$갈래(...)` 구성·참조, typed `에 따라` pattern은
같은 OwnedBranchIdentity를 가리킨다. 갈래 ordinal·source order·declaration order는
identity나 priority가 아니며 숫자 `[]`로 갈래를 해체하지 않는다.

## Selection 의미 행렬

Selection은 TargetType이 selector·result·failure·update 계약을 함께 소유한다.
동질 `(T)차림` index는 `T`, 0-based 반열림 range는 `(T)차림`를 내며 음수 뒤세기와
범위 밖 silent default를 금지한다. `(K,V)짝맞춤[K]`는 V를 내는 strict lookup이고,
missing key를 자동 option/default로 바꾸지 않는다. 안전 조회는 RD-476의 ordinary typed overload `찾기`가
`(V)있없` 계열을 반환한다. `찾기?`는 역사·명시적 이행 입력이지 영구 별칭이 아니다. `글[위치]`는 grapheme cluster 하나를 담은 `글`이며
code point·UTF unit·byte index로 조용히 바꾸지 않는다.

SelectionPlace와 `<-`는 기존 항목 교체·새 immutable 값·root rebinding으로 합성된다.
insert/delete/splice/중간 삽입/길이 변경 range replacement는 이 표면의 의미가 아니다.
`A[i,j]` MultiAxisSelection과 `A[i][j]` chained Selection은 별도 AST 의미다. 순서
없는 모음의 numeric `[]`는 금지한다. CollectionConstructor·PostfixSelection·Proof
projection은 canon roundtrip에서도 category를 보존한다.

## 호출별 ResultPlace

결과씨가 있는 named callable invocation은 `CallableDefinitionIdentity +
InvocationIdentity`에 결속된 하나의 ResultPlace를 가진다. ResultPlace는 Definition이나
전역 binding이 아니며 R1 source에서는 쓰기 전용이다. 순차 다시결속은 허용하지만
physical completion order나 병렬 child capture는 결과 authority가 아니다.

## 실패·종료·publish

typed value, execution disposition, failure family는 직교한다. 움직씨 ResultPlace는
local result candidate이고 atomic publish 성공 전에는 외부 성공 결과가 아니다.
실패·abort·publish 실패는 staged patch/outbox와 success candidate를 폐기한다.
