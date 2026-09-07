# PathIR·Proof kernel profile — Language SPEC v0.6.0 공개판

> 상태: `NORMATIVE EXACT SEMANTIC CALCULUS / PHYSICAL KERNEL PRODUCT-OPEN`

## 최소 constructive dependent core

최소 term family는 Universe, Bound/Global, Π/Lambda/Application, Σ/Pair/First/Second,
internal Path 또는 Identity/Refl/J, Let, checked inductive family·constructor·recursor다.
binder spelling은 identity가 아니고 alpha-equivalence를 구조적으로 보장한다. global
reference는 stable DefinitionIdentity다. K/UIP, equality reflection, `Type : Type`, Path
proof-irrelevance 강제와 cubical 비호환 equality primitive를 금지한다.

Π는 formation/introduction/elimination/β와 judgmental η를 가진다. Σ는 formation/pair/
projection/projection computation을 가지며 Σ-η는 judgmental equality가 아니라 필요한
경우 propositional theorem이다. Identity는 `Id(A,a,b)`, `Refl(a)`, `J`를 최소 calculus로
둔다. J는 internal certificate calculus이고 public tactic surface가 아니다.

## DefEq·universe·inductive admission

DefEq는 versioned KernelProfile이 허용한 deterministic β, transparent definition의 δ,
inductive/recursor ι, let ζ, Σ projection, `J(Refl)`, recursor computation과 Π η만 사용한다.
CAS·SMT·AI normalization, 일반 rewrite theorem과 사용자 수학 정리는 DefEq 권위가 아니다.
transparency/reducibility policy는 KernelProfile, ProofProblemIdentity,
CheckExecutionIdentity와 receipt에 봉인한다. transformation obligation과 assumption
closure를 분리하며 checked evidence 없이 conditional rewrite를 unconditional equality로
승격하지 않는다.

Universe는 predicative, universe-polymorphic, non-cumulative hierarchy이며
zero/successor/max/parameter를 표현한다. strictly-positive inductive family의 parameter,
index와 generated recursor를 admission 전에 검사한다. inductive-inductive,
inductive-recursive, complex nested inductive와 coinduction은 후속 확장이다. general
recursive `Fix`를 trusted core에 넣지 않고 source recursion은 primitive recursor 또는
검증된 well-founded construction으로 elaboration한 뒤 kernel term만 검사한다.

## quotient와 HoTT derived notions

R1 kernel에는 general primitive quotient를 넣지 않고 setoid library/profile을 사용한다.
relation은 reflexive/symmetric/transitive evidence를, quotient 위 함수는 representative
선택과 무관한 respectfulness proof를 요구한다. `Decidable(R)`, quotient equality
decidability와 executable canonical representative는 formation과 별도 evidence다. general
heterogeneous/dependent quotient elimination은 열지 않는다.

Path/Id, Refl, J가 primitive calculus이고 inverse/composition/transport/ap/apd는 stable
derived definition으로 낮춘다. Equiv와 Homotopy는 Π·Σ·Path 기반 derived notion이다.
HOTT-1 PropTrunc는 `in : A -> PropTrunc(A)`, proposition 성질과 proposition-valued target
으로만 가능한 elimination을 요구한다. kernel primitive·봉인 provider·library encoding 중
물리 착지 방식은 OPEN이다.

## independent checker와 identity

certificate는 fully elaborated kernel proof term을 기본으로 한다. independent checker는
context well-formedness, term typing, universe constraints, DefEq, inductive admission,
recursor validity와 provider identity를 검사한다. provider 이름·hash 존재만 보지 않고
KernelProfile이 허용한 declaration, inference rule, assumption closure, version과 sealed hash를
검증한다. tactic search, general unification, CAS·SMT·AI inference, network와 implicit
external theorem trust는 trusted core 밖이다. symbolic/CAS output은 obligation을 생성할 수
있지만 DDN checker evidence 없이 스스로 discharge하지 못한다.

semantic verdict는 `PROVED`, `REFUTED`, `UNKNOWN`, `UNSUPPORTED` 네 갈래다. REFUTED는
`Not(P)` 또는 checked counterexample certificate가 있을 때만 성립한다. predecessor
`DISPROVED`는 같은 의미의 superseded 표현이며 다섯 번째 verdict나 permanent alias가
아니다. malformed/certificate/hash/provider failure는 수학 verdict가 아니다.

ProofProblemIdentity는 target semantic identity, declared assumption/definition/provider
closure, universe constraints와 KernelProfile을 결속한다. 같은 target도 assumption closure가
다르면 별도 problem이다. CertificateIdentity는 problem, normalized proof term, referenced
definitions/providers를 결속한다. CheckExecutionIdentity는 checker spec/implementation과
execution/resource contract를 theorem identity와 분리한다. receipt는 problem/certificate/
check identity, verdict/reason, provider/work evidence, external provenance와 parent receipt를
결속한다.

## sealed profiles와 OPEN

HOTT-0/1/2는 숨은 flag가 아닌 서로 다른 sealed KernelProfile이다. HOTT-0은 constructive
dependent core와 internal identity/J를 포함하지만 public PathIR·FunExt·Univalence·
PropTrunc·HIT·cubical computation을 활성화하지 않는다. HOTT-1은 PathIR admission,
Equiv/Homotopy와 explicit FunExt/Univalence/PropTrunc provider를 추가하고 identity와
assumptions를 problem/receipt에 봉인하며 computational univalence를 주장하지 않는다.
HOTT-2는 별도 cubical calculus profile이고 기존 profile DefEq·certificate identity를 소급
변경하지 않는다. HOTT-1/2 제품 활성화는 demand-gated다.

Exact physical kernel schema·wire·parser/runtime·certificate encoding·hash prefix·
executable oracle와 제품 구현은 OPEN이다. 이 physical OPEN은 위 semantic calculus를
미정으로 되돌리지 않는다.
