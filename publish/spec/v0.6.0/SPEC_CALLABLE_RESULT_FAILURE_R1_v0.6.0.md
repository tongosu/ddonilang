# callable 결과·실패 R1 — Language SPEC v0.6.0 공개판

> 상태: `NORMATIVE SEMANTICS AND SUCCESSOR SURFACE / PRODUCT-OPEN`

## ResultPlace

결과씨 `R`가 있는 named callable invocation은 `CallableDefinitionIdentity +
InvocationIdentity`로 구별되는 ResultPlace 하나를 UNASSIGNED로 시작한다.
`callable정본이름 <- 값.`은 current invocation의 결과 후보를 결속·다시결속하며
Definition `:=`, callee-position callable identity와 root binding을 바꾸지 않는다.
result target에는 preferred canonical callable name만 쓰고 alternate morphology·alias·
presentation surface를 자동 허용하지 않는다. current self-name의 local shadow와 R1
source의 ResultPlace bare read를 금지한다. 결과씨가 없는 callable은 ResultPlace를 갖지
않는다.

같은 invocation에서 명시 순차 다시결속을 허용한다. 결속은 즉시 return이 아니며 뒤
문장을 계속 실행하고 정상 종료 결과는 마지막으로 실행된 결속값이다. 각 write는 선언
결과씨를 독립 만족해야 한다. hidden common-supertype·conversion chain·`있없/맺음`
wrapping·`없음` 삽입은 없다. canon은 `preferredName <- value`만 출력한다. `돌려줘`는
identity가 유일한 pre-v25 migration input일 뿐 permanent alias가 아니다.

compiler는 `UNASSIGNED/ASSIGNED/TERMINAL`을 추적한다. 모든 정상 reachable 합류 경로는
ASSIGNED여야 한다. 0회 가능한 loop body만의 최초 결속은 결과 보장이 아니고 generic
return·일반 block implicit tail·`그침 값`을 도입하지 않는다. 병렬 child는 enclosing
ResultPlace를 capture/write하지 않는다. typed local result를 explicit join/reducer/merge한
뒤 sequential frame에서 결과를 기록한다. sealed `에 따라`의 모든 정상 reachable 갈래가
결과를 보장해야 하며 누락을 `없음`, `맺음.$안됨`, default 또는 첫/마지막 갈래로 자동
보정하지 않는다.

움직씨 ResultPlace는 local result candidate다. definite-result와 PatchCandidate 검증,
atomic publish 성공 뒤에만 외부 성공 결과를 관찰한다. failure·abort·publish 실패는
staged patch/outbox와 success candidate를 폐기한다.

## typed failure와 recovery

recoverable 사용자 실패는 `맺음.$안됨(까닭)` typed value다. typed value, execution
disposition, failure family는 직교한다. contract·integrity·capability·hard resource·host
abort를 `$안됨`으로 silent lowering하지 않는다.

`A 해보고 안되면 { ... }`와 `A 해보고 안되면 까닭에 따라 { ... }`가 canonical recovery
family다. `까닭`은 scope binder이며 고정 이름이 아니다. abort·hard failure는 catch하지
않는다. `값 <- A 해보고 안되면 그대로둠.`은 진입 시 definite-assigned된 기존 binding만
보존하는 conditional rebinding이다.

`값 아니면 되돌림`은 successor surface에서 retire하며 permanent alias나 unchecked
propagation을 만들지 않는다. 실패를 공개 결과로 내는 callable은 ResultPlace에
`맺음.$안됨`을 기록하고 필요하면 `마침.`을 쓴다. throw/try/catch/finally, generic
exception hierarchy, generic success return과 implicit fallback은 도입하지 않는다.

`마침.`은 current 셈씨·움직씨의 normal control flow terminal이다. 결과가 필요한 모든
도달 경로는 ResultPlace가 이미 definite-assigned여야 한다. `마침`은 값·return·commit·
publish가 아니다. 결과씨가 `글`인 callable에 `맺음.$안됨`을 쓰면 type error다.
single-expression named 셈씨는 RHS 평가 후 정상 완료하므로 별도 `마침`이 필요 없다.

structured execution에서 child typed `$안됨`은 정상 typed value일 수 있다. 실제 hard
failure/abort/cancel은 stop-admission·cancel·join·discard를 따르고 최초 원인과 파생
cancellation causal chain을 보존한다.

## 이름 있는 단일식 pure 셈씨

```ddn
두배하다 (값:셈수~을 --> 셈수) := 값 * 2.
```

`이름 (받는자리들 --> 결과씨) := 식.`은 이름 있는 단일식 pure 셈씨 Definition이다.
source shape 자체가 pure-by-shape 계약을 선언한다. RHS가 effectful이면 움직씨로 추론·승격하지
않고 fail-closed한다. 의미상 `이름 (...) : 셈씨 := { 이름 <- 식. }.` ResultPlace body로
lowering하지만 FunctionLiteral이나 local binding을 만들지 않고 named
CallableDefinitionIdentity를 유지한다.

`-->`는 function type/signature relation, `|->`는 anonymous FunctionLiteral, `:=`는
Definition binder다. 세 표면과 named/anonymous identity를 합치지 않는다. 복수 문장
callable은 `: 셈씨` 또는 `: 움직씨`를 명시하고 body effect로 종류를 추론하지 않는다.

callable canonical 이름은 `…다` 기본형이고 독립 호출은 등록된 `…기` 계열이다. 활용형은
versioned morphology profile과 author-committed form이 소유하며 AI·동의어·무제한 자연어
추측을 authority로 쓰지 않는다. `마침`·`그대로둠`은 language-owned marker지만 blanket
`~ㅁ/~ㄴ` grammar가 아니다. `이것`·`나`·`맡은이`·`제`의 self vocabulary는 alias가 아니다.

## failure identity와 replay

FailureSemanticIdentity와 FailureOccurrenceIdentity를 구별한다. source 위치와 locale
message는 diagnostic evidence다. retry attempt는 같은 WorkIdentity 아래
RetryPolicyIdentity와 AttemptOrdinal을 결속한 distinct AttemptIdentity를 가진다. semantic
failure·pause·cancel은 worker 수와 physical scheduling에 무관한 replay 동등성을 요구한다.
HOST_ABORT·INTERNAL_FAULT는 evidence에 기록하지만 동일 semantic replay outcome을 요구하지
않는다. exact hash·enum·schema·receipt encoding·wire와 제품은 OPEN이다.
