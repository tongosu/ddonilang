# 또니랑 문법 요약 — Language SPEC v0.6.0 공개판

> 상태: `public current / owner-approved / independently reviewed / docs-first / product-open`
> 내부 투영 기준: `v25.15.0 live current / RD-001~437`
> 공개 current: `v0.6.0`
> 충돌 시 독립 승격된 내부 live SSOT가 우선한다.
> 이 공개판의 문서 current 승격은 제품 parser/runtime 지원을 주장하지 않는다.

## 정의와 다시결속

```ddn
둘 := 2.
현재 <- 둘.
현재 <- 3.
```

`:=`는 새 Definition identity를 만든다. `<-`는 현재 값 결속을 만들거나
바꾸며 새 Definition identity를 만들지 않는다. 두 기호는 alias가 아니다.

## 완전 선언

```ddn
(값=값:셈수~을)
두배하다
    : 셈수
    : 셈씨
:= {
    두배하다 <- 값 * 2.
}.
```

정적 문맥이 있으면 실행 머리 앞에서 `두고`로 분리한다. 선언 머리에는 public
pin, 지역 binder, 씨, 역할 표면, 기본값, preferred 이름, 꼴바꿈, 결과씨,
선언 종류와 등록 표면이 들어갈 수 있다. `:=`는 이 머리 전체와 본문을 한
Definition identity로 묶는다. 생략이 유일하지 않으면 명시 선언을 요구한다.

## typed morphology

```ddn
(대상=대상:셈수~을)
돕다{도우~도와}
    : 셈수
    : 셈씨
:= {
    돕다 <- 대상.
}.
```

`돕다`는 preferred citation form이고 `도우`, `도와`는 pinned morphology
profile이 역할을 판정하는 typed alternate form이다. raw 문자열 alias가 아니며
`도움`, `도우미` 같은 파생어는 별도 identity다. UNKNOWN·AMBIGUOUS는 오류다.

## closed multisurface와 presentation canon

등록 surface는 arbitrary parser macro가 아니다. 원자형·앞붙임·뒤붙임·
이항형·둘레형·mixfix형·한국어 관계/조사형 같은 versioned closed class만
허용한다. typed placeholder 결속 뒤 callable이 정확히 하나일 때만 성공한다.
사용자가 global precedence·priority·associativity를 주입할 수 없다.

semantic identity와 hash는 하나지만 한국어·수학·ASCII·교육 presentation
profile은 같은 의미의 preferred source 표현을 각각 가질 수 있다.

## 묶음씨와 고름씨

```ddn
(
    가로: 셈수,
    세로: 셈수
)
좌표 : 묶음씨 := {
}.

계산결과 : 고름씨 := {
    $성공(값: 셈수).
    $실패(이유: 글).
}.
```

묶음씨는 동시에 존재하는 field product다. 고름씨는 하나를 고르는 sum이며 오른쪽
body에는 `$갈래` schema만 둔다. 실행문은 sum body 안에 두지 않는다.

## typed Selection `[]`

```ddn
첫째 <- 값들[0].
값들[1] <- 20.
키값 <- 표[키].
문자묶음 <- 글값[위치].
```

`대상[고름]`의 의미는 TargetType이 소유한다. 묶음 field는 `.field`, 고름 갈래는
`$갈래`와 `에 따라`를 사용한다. `A[i,j]`와 `A[i][j]`는 전역 동치가 아니다.

## typed Selection 대상별 계약

| 대상 | selector | 결과 | 실패·경계 |
|---|---|---|---|
| 이질 위치형 값꾸러미 | compile-time static 위치·범위 | 위치별 선언씨 | 결과씨가 위치에 따라 달라 runtime selector를 허용하지 않음 |
| 동질 `차림<T>` | runtime 바른수 index | `T` | 0-based, 음수 뒤세기 금지, 범위 밖 silent default 금지 |
| 동질 `차림<T>` | 0-based 반열림 range | `차림<T>` | 음수 범위·숨은 clipping·fallback 금지 |
| `짝맞춤<K,V>` | `K` | `V` | missing key는 strict failure; safe lookup은 별도 이름 있는 provider가 `(V)있없`을 반환 |
| `글` | 위치 | grapheme cluster 하나를 담은 `글` | code point·UTF unit·byte index로 암묵 전환 금지 |
| 순서 없는 모음 | numeric selector | 허용하지 않음 | 순서나 ordinal을 숨겨 만들지 않음 |

strict selection의 exact failure code family와 safe lookup provider의 최종 한국어
이름은 OPEN이다.

## SelectionPlace와 immutable replacement

```ddn
값들 := 차림[10, 20, 30].
값들[1] <- 25.
```

`대상[selector] <- 값`은 새 연산자가 아니라 SelectionPlace와 `<-` Rebinding의
합성이다. 기존 항목을 교체한 새 immutable 구조값을 만들고 최종 root를 다시
결속한다. missing map key insert, delete, splice, 중간 삽입과 길이가 다른 range
replacement를 숨은 의미로 넣지 않는다.

## multi-axis·chained·constructor 분리

- `A[i,j]`는 대상 rank 계약이 소유하는 하나의 MultiAxisSelection이다.
- `A[i][j]`는 `Selection(Selection(A,i),j)`인 chained Selection이다.
- 차림·짝맞춤의 다중 selector는 기본 거부한다.
- Matrix/Tensor만 별도 rank·arity 계약에 따라 multi-axis를 허용할 수 있다.
- `차림[...]` CollectionConstructor, `xs[i]` PostfixSelection, Proof typed projection은
  서로 다른 AST·canon category다.
- `.field`와 `[selector]`는 closed postfix access class에서 왼쪽부터 typed 결속한다.

Tensor rank reduction·static slice result·immutable value·positional axis의 semantic 계약은
닫혔다. named axis·dynamic shape/range·TensorView·broadcast/newaxis 등 deferred feature와
exact physical parser·AST·wire·runtime·diagnostic은 OPEN이다.

## 익명함수와 이름 있는 callable 결과

`x |-> E`의 결과는 RHS expression `E`다. 여러 문장 익명함수는 block의
tail-expression을 결과로 사용한다. 반면 이름 있는 결과 callable은 호출마다 하나인
ResultPlace에 preferred callable 이름으로 값을 기록한다.

```ddn
(값=값:셈수~을)
두배하다
    : 셈수
    : 셈씨
:= {
    두배하다 <- 값 * 2.
}.
```

결과 결속은 즉시 return이 아니며 뒤 문장이 계속 실행된다. 모든 정상 종료 경로는
결과가 definite-assigned되어야 한다. 일반 callable의 implicit last-expression 결과,
generic `돌려줘`, ResultPlace bare read는 허용하지 않는다.

## typed failure와 recovery

recoverable 사용자 실패는 `맺음.$안됨(까닭)` typed value다. abort·integrity·capability·
hard resource failure와 분리한다. canonical recovery family는 `해보고 안되면`이며,
`그대로둠`은 이미 결속된 값의 conditional rebinding 실패 시 기존 값을 보존한다.
`마침.`은 결과값을 만들지 않고 definite result가 있는 현재 callable의 정상 흐름만
끝낸다.

## ordinary optional

ordinary optional의 preferred canonical type surface는 `(T)있없`이다. `T?`는
v0.6.0 current parser alias가 아니라 pre-v25 migration-only 입력이다.

## 이름 있는 단일식 pure 셈씨

```ddn
두배하다 (값:셈수~을 --> 셈수) := 값 * 2.
```

`이름 (받는자리들 --> 결과씨) := 식.`은 pure-by-shape named callable Definition이다.
effectful RHS를 움직씨로 자동 승격하지 않고 fail-closed한다. 이는 named ResultPlace body로
lowering하지만 `x |-> E` FunctionLiteral과 identity를 합치지 않는다. `-->`는 signature,
`|->`는 anonymous literal, `:=`는 Definition binder다.
