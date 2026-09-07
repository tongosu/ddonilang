# Tensor R1 — Language SPEC v0.6.0 공개판

> 상태: `NORMATIVE SEMANTICS / SUCCESSOR SURFACE PARTIALLY CLOSED / PHYSICAL PRODUCT-OPEN`

## identity와 static shape

Tensor와 ordinary nested `차림`은 다른 semantic/type identity다. Tensor TypeIdentity는
Tensor family identity, element type, finite ordered tuple of static nonnegative axis
sizes로 구성한다. rank-0 `Tensor<T;()>`와 zero-length axis를 허용한다. axis는 ordered
shape tuple의 위치 identity만 소유한다. named axis와 axis-name selection은
DEFERRED_V1_NONBLOCKER다. rank·size·backend ceiling은 profile/resource 문제이고 semantic
identity가 아니다. nested collection의 깊이·길이만으로 Tensor identity를 암묵 부여하지
않는다.

runtime shape는 static type parameter가 아니다. runtime data가 `Tensor<T;shape>`에
들어가려면 source에서 shape가 유일하거나 conversion site에 static target shape가
명시되어야 한다. ambient expected type만으로 숨은 shape를 발명하지 않으며 V1은 별도
dynamic-shape Tensor family를 두지 않는다.

## MultiAxisSelection과 결과 rank

`A[s1,s2,...]`는 하나의 MultiAxisSelection이다. selector 하나가 axis 하나를 소비한다.
scalar selector는 axis를 제거하고 range와 full-axis `:`는 axis를 유지한다. selector가
부족하면 trailing axis를 semantic full-axis로 보존하되 source/AST distinction은 유지한다.
selector 초과와 `A[]`는 오류이며 `A[i,j]`와 `A[i][j]`는 다른 AST다.

range는 0-based·half-open·static이고 negative index/range를 금지한다. `start==end`는
valid zero-length range, `start>end`는 compile error다. slice extent는 static하게 결정
가능해야 한다.

scalar selector가 모든 axis를 제거하면 결과는 ordinary element `T`다. rank-0
`Tensor<T;()>`와 `T`는 다른 type이며 자동 boxing/unboxing하지 않는다. `A[]`는 extraction
syntax가 아니다. rank-0 extraction은 별도 명시 named provider가 소유하며 정확한 provider
이름은 PRODUCT-OPEN이다.

## slice·replacement·broadcasting

V1 Tensor slice는 immutable Tensor value이며 observable alias/view semantics가 없다.
backend는 의미가 관찰되지 않는 범위에서 shared storage, copy-on-write, strided
representation을 사용할 수 있다. observable TensorView는 후속 additive feature다.

`A[...] <- replacement`는 SelectionPlace와 `<-`를 재사용하는 persistent update다.
scalar cell과 exact same-shape slice replacement만 허용한다. shape change·resize·
insert/splice·automatic broadcasting·temporary target update·observable in-place sharing은
금지한다. world/resource Tensor는 capability·Patch·atomic commit 경계를 따른다.

V1 generic Tensor-Tensor elementwise operation은 shape equality를 기본으로 한다. 수학적
scalar action은 Vector/Matrix provider가 명시적으로 소유할 수 있다. general broadcasting은
DEFERRED_V1_NONBLOCKER다.

## 명시 변환과 수학 role

nested collection→Tensor는 명시 provider 의미명 `텐서로`, Tensor→nested collection은
`차림으로`가 소유한다. 전자는 element type 일치·recursive rectangularity·unique static
shape를 요구한다. ragged input과 target shape가 없는 empty shape-underdetermined input은
실패한다. 후자는 rank 0→`T`, rank 1→`차림<T>`, rank N→N-depth nested collection이며
변환 뒤 Tensor shape identity는 ordinary collection type에 보존되지 않는다. exact 호출
머리·pin spelling·registry schema·wire는 PRODUCT-OPEN이다.

Vector/Matrix는 Tensor parser/type alias가 아니다. rank-1/2 Tensor를 자동 Vector/Matrix로
승격하지 않는다. dimension은 mathematical role identity에 참여하고 generic Tensor
operation과 vector/matrix provider identity를 구별한다.

## deferred selection과 failure

ellipsis, newaxis, boolean mask, index tensor, gather/scatter, fancy indexing,
broadcasting selection, dynamic range, dynamic-shape Tensor/slice, named axis와 axis-name
selection은 DEFERRED_V1_NONBLOCKER다.

static invalid shape/selector/known out-of-bounds와 malformed static range는 compile
error다. bare typed `[]`의 runtime scalar out-of-range는
`STRICT_SELECTION_CONTRACT_ABORT` 계열이며 `없음`, silent clipping/fallback, ordinary
typed result 또는 generic recovery로 낮추지 않는다. safe index는 별도 named provider가
typed result를 반환한다. explicit nested↔Tensor의 ragged/shape mismatch는 typed domain
conversion failure이며 profile capacity excess는 resource ceiling failure다.

## logical canon과 physical OPEN

Tensor canon/type identity에는 family·element type·ordered static shape가 참여한다.
logical value canon/hash는 shape와 index tuple의 lexicographic axis order element values를
사용한다. axis 0은 outermost, last axis는 fastest-varying이다. backend·device·dense/sparse
storage·memory layout·stride·allocation identity·execution schedule은 기본적으로 참여하지
않고 physical layout을 강제하지 않는다.

`겹차림`은 educational/explanatory term일 뿐 parser alias·TypeIdentity alias·automatic
conversion 근거가 아니다. physical storage schema, backend, GPU ABI, BLAS/cuDNN,
exact parser·AST·wire·diagnostic·runtime·executable oracle과 제품 parity는 OPEN이다.
