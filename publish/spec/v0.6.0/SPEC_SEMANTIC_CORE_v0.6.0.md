# 의미 핵심 — Language SPEC v0.6.0 공개판

> 상태: `public current / owner-approved / independently reviewed / docs-first / product-open`
> 내부 투영 기준: `v25.15.0 live current / RD-001~437`
> 공개 current: `v0.6.0`
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

typed morphology와 closed multisurface는 하나의 semantic identity에 이르는
결정적 표면이다. source order, declaration order, hidden priority, AI의 자연어
상식은 semantic authority가 아니다.

## 비파급

source epoch 변경은 state, receipt, lock 또는 wire identity의 폐기를 뜻하지
않는다. 이 축은 별도 호환성 감사와 owner 결정이 필요하다.

## 수학 다중표면과 HoTT 방화벽

한국어·수학·ASCII source artifact는 다를 수 있지만 typed resolution 뒤 같은 의미면
semantic IR identity와 hash는 하나다. Proof R1 equality `=:=`를 HoTT higher path로
재해석하지 않는다. HoTT Path는 Type-valued 별도 identity family이며 exact PathIR,
calculus, schema, wire와 proof kernel은 OPEN이다.

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
동질 `차림<T>` index는 `T`, 0-based 반열림 range는 `차림<T>`를 내며 음수 뒤세기와
범위 밖 silent default를 금지한다. `짝맞춤<K,V>[K]`는 V를 내는 strict lookup이고,
missing key를 자동 option/default로 바꾸지 않는다. 안전 조회는 별도 provider가
`(V)있없` 계열을 반환한다. `글[위치]`는 grapheme cluster 하나를 담은 `글`이며
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
