> v0.9.0 로컬 문서판 효력: `publish/spec/CURRENT.md`가 이 family를 지정한 뒤에만 적용됩니다.
> 규범 투영 기준: 내부 v25.151.0 / RD001~709. 내부 현행 v25.153.0 / RD001~711의 전체 투영판은 아닙니다.
> [판본 범위와 내부 후속 효력](판본범위와_내부후속효력_v0.9.0.md)이 아래 고정 원문의 당시 상태 표기보다 현재 판본 범위를 우선 설명합니다.
> 제품 지원·실행·Human·GitHub 게시 승인과 분리합니다. 아래 후보/당시 CURRENT/게시허용 표기는 고정 입력의 역사 상태이며 이번 GitHub 게시 권한이 아닙니다.

<!-- REVIEWED_R21_BODY_START -->
# 또니랑 문법 요약 — Language SPEC v0.9.0 후속 후보

> CANDIDATE_R2 / NOT_CURRENT / DOCS_FIRST / INDEPENDENT_REVIEW_PENDING
> 규범 기준: 내부 v25.151.0 / RD001~709. 공개 CURRENT는 v0.8.1로 유지한다.
> 제품 지원·실행·독립 제품검토·공개 게시·Human은 이 문서 후보의 상태와 별개다.

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

## 명시 등록 accepted surface

```ddn
(대상=대상:셈수~을)
돕다{도우~도와}
    : 셈수
    : 셈씨
:= {
    돕다 <- 대상.
}.
```

`돕다`는 preferred authored form이고 `도우`, `도와`는 explicit authored accepted
surface set이다. compiler는 자연어적 활용의 옳고 그름을 판단하지 않으며 등록 여부,
정확 일치, typed context와 유일 해소만 판단한다. morphology profile은 convenience
expansion일 뿐 semantic authority가 아니고 explicit set이 우선한다. 파생어 identity는
별도이며 UNKNOWN·AMBIGUOUS는 오류다.

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
| 동질 `(T)차림` | runtime 바른수 index | `T` | 0-based, 음수 뒤세기 금지, 범위 밖 silent default 금지 |
| 동질 `(T)차림` | 0-based 반열림 range | `(T)차림` | 음수 범위·숨은 clipping·fallback 금지 |
| `(K,V)짝맞춤` | `K` | `V` | missing key는 strict failure; safe lookup은 별도 이름 있는 provider가 `(V)있없`을 반환 |
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

recoverable 사용자 실패는 적용 맺음의 정확 소유씨·공개 핀에 결속된 안됨 typed value다. abort·integrity·capability·
hard resource failure와 분리한다. canonical recovery family는 `해보고 안되면`이며,
`그대로둠`은 이미 결속된 값의 conditional rebinding 실패 시 기존 값을 보존한다.
`마침.`은 결과값을 만들지 않고 definite result가 있는 현재 callable의 정상 흐름만
끝낸다.

## ordinary optional

ordinary optional의 preferred canonical type surface는 `(T)있없`이다. `T?`는
현행 parser alias가 아니라 pre-v25 migration-only 입력이다.

## 이름 있는 단일식 pure 셈씨

```ddn
두배하다 (값:셈수~을 --> 셈수) := 값 * 2.
```

`이름 (받는자리들 --> 결과씨) := 식.`은 pure-by-shape named callable Definition이다.
effectful RHS를 움직씨로 자동 승격하지 않고 fail-closed한다. 이는 named ResultPlace body로
lowering하지만 `x |-> E` FunctionLiteral과 identity를 합치지 않는다. `-->`는 signature,
`|->`는 anonymous literal, `:=`는 Definition binder다.


## 계승된 현행 표면

- same-name ordinary pin surface는 full과 exact short를 허용하고 short로 canon한다. public ParamPinIdentity와 local BinderIdentity는 합쳐지지 않는다.
- PG-CLI-05 branch construction과 typed match arm은 explicit TypeIdentity-qualified branch를 요구하며 bare branch는 실패한다.
- ProgressionRange callable은 positional, named-pin, Korean-particle의 세 complete pattern만 허용하고 모두 named-pin `범위짓기`로 canon한다.
- LanguageProfile header는 exactly one canonical header와 finite explicit aliases를 가지며 canon은 canonical header만 출력한다.
- official CLI machine output은 explicit `--json` 또는 `--summary-json`; ExecutionIR ingress는 explicit `--execution-profile DDN-EXECUTION-IR-R1@1`이다.


## 계승된 원문·바이트열·반복 표면

- SourceSnapshot provider entry uses the canonical namespace `원문고정본` and its exact
  typed callables; path, mtime and content-only digest are not identity authority.
- Bytes conversion callables are `UTF8바이트열로` and `UTF8NFC글로`; aliases, fuzzy
  resolution and implicit String/List conversion are forbidden.
- Ordinary while is `{ 조건 }인 동안 { ... }.` and hook while is `(조건)인 동안 { ... }.`.
- V1 Basic Tensor element TypeIdentity is canonical `셈수`; rank-0 Tensor is not a scalar.
- VC17 REQUIRED15 follows the exact empty/search/replace/transform/join/concat laws and
  accepts only sealed named PURE callable references for collection transform.

## 현행 C2/C3 작성과 명시 집합

C3는 직접 등록된 유한 accepted-surface 집합이다. C2는 명시적·유한·정확한 판본의 입력을 작성 도구가 전개하는 선택적 압축이다. 전개 결과는 같은 profile/version·Definition·role의 직접 C3와 정확히 같은 집합이어야 한다. 자동 줄기 추출·기본 적용·숨은 활용 생성·등록 순서나 최장 일치에 의한 승자 선택은 없다. provisional/HOLD 프로필은 등록 완료가 아니다.

정본 이름·ResultPlace·호출 표면·함수값·연결어미는 역할이 다르다. 명사형 호출·무인자 호출의 비현행 후속안과 일본어 A{A}는 이 현행 C3 설명의 자동 허용 항목이 아니다. 정확 역할/작성 규격이 OPEN이면 문자 유사성으로 메우지 않는다.

## 조건별 함수의 작성 문맥

조건별은 함수 머리에 귀속된 typed 몸통이며 일반 if의 순서/우선순위를 바꾸지 않는다. 정의역 항목 하나를 첫 항목으로 쓰고 나머지는 값식:조건 행이다. bare 조건별 블록·일반 실행문 혼입·중복/생략 정의역은 첫 범위가 아니다. 조건은 수학 명제를 구성하며 Bool 실행값에서 명제로 역추론하지 않는다.

```ddn
요금 (사용량:나눔수~을 --> 나눔수) := 조건별 {
    정의역: 0 <= 사용량 <= 6.
    10 * 사용량 : 0 <= 사용량 <= 3.
    30 + 20 * (사용량 - 3) : 3 < 사용량 <= 6.
}.
```

위 표면은 RD706 승격 의미의 설명이며 현 제품 parser 실행 PASS가 아니다. 같은 binder·carrier의 정의역/조건/값식과 정확 closure를 공급해야 한다. 전체 문법·검사·빈 함수 경계는 [조건별 계약](SPEC_PIECEWISE_FUNCTION_v0.9.0.md)을 따른다.

## 보임 생성의 목표 문맥

괄호쉼표 표면 (p,q)는 유일하게 해소된 이름 있는 보임 결과의 논리곱 목표가 직접 기대되는 자리에서만 생성으로 해소한다. 항 개수·순서·중첩·가정/정의 closure를 전체 검사한 뒤 투영/후속 소비한다. ordinary Tuple·값 모양·표시 문자열·성공 플래그는 보임 발급 근거가 아니다. 지역 기대씨·임의 중첩 인수·일반 run 값·저장 wire로 확대하지 않는다(RD707).

## 시작·매마디 root 문맥

정확 root 표면은 `(시작)할때 { ... }.`와 `(매마디)마다 { ... }.`다. 첫 범위는 하나의 명시 진입/등록 단위다. 시작은 새 의미 실행의 첫 실제 마디, 매마디는 첫 마디를 포함한 각 실제 마디에 해당한다. 일반 root와 해당 hook body는 같은 단위의 원문 순서다. 초기화 시작을 앞에 쓰는 것은 권장이며 자동 hoist·종류별 phase·동명 body 합치기·재정렬은 없다(RD709).

<!-- REVIEWED_R21_BODY_END -->
