# SSOT_MUST_RULE_AUDIT_TIER3_V1

> 작성: Codex (2026-07-06)
> 범위: `docs/ssot/ssot/SSOT_LANG_v24.12.9.md` 3979~5405행, MUST 섹션 45개
> 성격: 진단 전용. `docs/ssot/**`, 코드, pack, golden 수정 없음.

## 방법

- 대상 추출: `rg -n "^#{2,4} .*(MUST)" docs/ssot/ssot/SSOT_LANG_v24.12.9.md`
- Tier3 기준: 헤더 시작 행 `>= 3979 && < 5406`
- 제품 근거 경로: `tools/teul-cli/src/**`, `core/src/**`, 필요시 `tool/src/**`
- 판정 기준: `구현됨` / `부분구현` / `미구현` / `확인불가`

## 판정표

| # | 규칙 ID/제목 | 줄 범위 | 핵심 주장 | 판정 | 근거(file:line 또는 검색식) |
|---:|---|---:|---|---|---|
| 1 | 14. 다중 디스덧댐 | 3979-4019 | overload 후보 중 가장 구체적 타입 선택, 동률은 컴파일 오류 | 미구현 | 제품 teul-cli parser/runtime에서 다중 dispatch 후보/구체성 비교/모호성 오류 구현을 찾지 못함: `rg -n '다중 디스|overload|dispatch.*candidate|specific.*type' tools/teul-cli/src core/src tool/src`는 해당 규칙 집행 근거를 주지 않음. |
| 2 | 15. 비목표 | 4020-4030 | 클래스/OOP와 Rust식 borrow checker는 AGE0 비목표 | 부분구현 | core에는 ECS/archetype 기반 `NuriWorld`가 있음(`core/src/platform.rs:23-69`, `:129-137`). 제품 언어 표면에 class/borrow checker 구현은 확인되지 않지만, 비목표를 금지 진단으로 고정한 별도 집행 경로도 확인되지 않음. |
| 3 | 16. 검사 훅 `늘지켜보고` | 4031-4038 | commit 직전 invariant 검사 훅 | 부분구현 | 계약/지킴이 계열은 존재함(`tools/teul-cli/src/runtime/eval.rs:1153-1260`, `:2440-2460`). 하지만 `늘지켜보고` 표면은 제품 경로 검색에서 없음: `rg -n '늘지켜보고|덧댐거부' tools/teul-cli/src core/src tool/src` = NO_MATCH. |
| 4 | 16.2 문법 | 4039-4046 | `(조건) 늘지켜보고 { ... }` 문법 | 미구현 | `늘지켜보고` token/parser 분기 없음: `rg -n '늘지켜보고' tools/teul-cli/src/lang tools/teul-cli/src/runtime` = NO_MATCH. 기존 계약 문법은 `{ ... }인것 바탕으로/다짐하고 ...` 계열이다(`tools/teul-cli/src/lang/parser.rs:4562-4607`). |
| 5 | 16.3 실행 시점 | 4047-4060 | Patch 생성 뒤 Commit 직전 검사 | 부분구현 | tick loop에 계약 frame begin/commit/rollback 흐름은 있음(`tools/teul-cli/src/runtime/eval.rs:735-811`). 그러나 `늘지켜보고` hook을 Patch/Commit 사이에 넣는 제품 단계는 없음(#4). |
| 6 | 16.4 제약(읽기 전용) | 4061-4072 | 검사 블록 내 상태 변경/I/O/예약 금지 | 부분구현 | parser에는 contract/assertion 관련 expression mutation/show/io 검출기가 있음(`tools/teul-cli/src/lang/parser.rs:3442-3468`, `:3573-3599`, `:3740-3767`). 다만 `늘지켜보고` 블록 자체가 없어 이 규칙 전체를 해당 표면에 집행하지 못함. |
| 7 | 16.5 거부/실패 정책 | 4073-4098 | `덧댐거부`는 commit 폐기, `중단`은 즉시 중단 | 부분구현 | 계약 실패 rollback은 구현되어 있음(`tools/teul-cli/src/runtime/eval.rs:1801-1844`, `:1153-1260`). 하지만 `덧댐거부` 표면/감사 이벤트는 제품 검색에서 없음: `rg -n '덧댐거부' tools/teul-cli/src core/src tool/src` = NO_MATCH. |
| 8 | 17. 말결/퍼지 토큰 | 4099-4115 | `$꽤`, `$매우` 등 말결 토큰을 fuzzy weight로 적용 | 부분구현 | 런타임 builtin `말결값`은 tag를 수치로 바꿈(`tools/teul-cli/src/runtime/eval.rs:5021-5024`, `:15403-15412`). 그러나 `$` 말결 token/문장 앞 적용 parser는 확인되지 않음: `rg -n '말결|\\$꽤|\\$매우' tools/teul-cli/src/lang`는 표면 구현 근거를 주지 않음. |
| 9 | 17.3 표준 가중치 매핑 | 4116-4136 | 정해진 8개 말결 token을 표준 weight로 매핑 | 부분구현 | `nuance_weight`는 `매우/꽤/조금/약간/거의`만 매핑하고 기본값은 `1.0`이다(`tools/teul-cli/src/runtime/eval.rs:15403-15412`). SSOT의 8개 표와 기본 0.5에는 미달. |
| 10 | 17.5 결합 규칙 | 4137-4157 | 말결 없으면 0.5, 다중 token 마지막 적용, 부정어 부호 반전 | 미구현 | 제품 parser/runtime에서 `$` 말결 token sequence나 부정어 결합 처리 경로를 찾지 못함: `rg -n '말결|부정|\\$매우|\\$꽤' tools/teul-cli/src/lang tools/teul-cli/src/runtime`는 `말결값` 함수 외 결합 규칙을 보이지 않음. |
| 11 | 18. GOAP 목표 어미 | 4158-4174 | `-도록/-게` 목표 표면을 언어 레벨에서 지원 | 부분구현 | core는 `parse_dorok`으로 `도록` goal text를 `TargetState`로 정규화함(`core/src/seulgi/goal.rs:17-29`, `:85-124`), CLI GOAP planner도 있음(`tools/teul-cli/src/cli/goap.rs:5-24`). 하지만 DDN parser의 `(대상)을 ...하도록.` 표면 문법은 확인되지 않음. |
| 12 | 18.3 의미론 | 4175-4330 | GoalComponent 생성, 외부 planner/AI 관찰, 행동 주입 | 부분구현 | GOAP planner는 deterministic plan/validate를 제공함(`core/src/seulgi/goap.rs:138-203`). 다만 DDN 실행 중 GoalComponent를 누리에 생성하고 Sam/Seulgi intent로 주입하는 제품 연결은 확인되지 않음. |
| 13 | 16.1 누리 쿼리 목적/원칙 | 4331-4338 | 전수 스캔 금지, archetype index, 결정적 정렬 | 부분구현 | core에는 archetype 구조와 tag query API가 있음(`core/src/platform.rs:23-69`, `:129-137`). 하지만 `query_entities_with_all_tags`는 locations를 순회하므로 전수 스캔 금지/bitset 교집합 표준과는 다름(`core/src/platform.rs:129-137`). |
| 14 | 16.2 쿼리 표현식과 필터 | 4339-4355 | `(조건...) 임자들` query 표면과 포함/제외/술어 필터 | 미구현 | 제품 parser에서 `임자들` query expression/filter 표면을 찾지 못함: `rg -n '임자들|아키타입|query expression|QRY' tools/teul-cli/src/lang tools/teul-cli/src/runtime`는 해당 문법 근거 없음. |
| 15 | 16.3 군집 실행 블록 | 4356-4379 | query 결과에 `모두`, `첫째`, `마지막`, `개수` 실행 | 미구현 | runtime에는 `마지막`, `개수` 같은 일반 helper는 있으나(`tools/teul-cli/src/runtime/eval.rs:3474`, `:9759`), `집합 모두 {}` query loop 표면은 parser에서 확인되지 않음: `rg -n '임자들|모두 \\{' tools/teul-cli/src/lang tools/teul-cli/src/runtime` 근거 없음. |
| 16 | 16.4 스냅샷 의미론 | 4380-4384 | query 멤버십은 phase 시작 시 고정 | 부분구현 | core query API는 `Vec<EntityId>`를 반환하므로 호출 결과 자체는 snapshot 값이 됨(`core/src/platform.rs:129-137`). 그러나 DDN query phase/군집 실행 표면이 없어 SSOT 전체 의미론은 미착륙. |
| 17 | 16.5 구현 요구 사항 | 4385-4394 | archetype indexing/bitset intersection 표준, debug fallback만 scan | 부분구현 | archetype data structure는 있음(`core/src/platform.rs:23-69`)이나 현재 query는 `locations` 순회 기반임(`core/src/platform.rs:129-137`). bitset intersection 표준 구현은 확인되지 않음. |
| 18 | 17.1 D-STRICT 산술 원칙 | 4395-4404 | 산술 오류를 예외가 아닌 결정적 값+고장 기록으로 전이 | 부분구현 | `core::Fixed64`는 saturating add/sub/mul과 div fault signal을 갖는다(`core/src/fixed64.rs:42-94`). teul-cli formula/runtime 일부는 `RuntimeError::MathDivZero` 등 오류를 반환하므로 전체 런타임 정책은 혼재(`tools/teul-cli/src/runtime/formula.rs:235-247`). |
| 19 | 17.2 고장(Fault) 표준 구조 | 4405-4415 | `고장:이름씨` 또는 `#고장`/LogComponent 표준 기록 | 부분구현 | core signal에는 `ArithmeticFault`가 있고 div-by-zero에서 emit됨(`core/src/fixed64.rs:88-94`, `core/src/platform.rs:1339`). DDN 표면 `고장:이름씨`/`#고장` component 표준은 제품에서 확인되지 않음. |
| 20 | 17.3 산술 규칙 | 4416-4446 | add/sub/mul 포화, div0 대입 무효화와 고장 기록 | 부분구현 | core Fixed64는 add/sub/mul 포화와 `div_assign_det` 대입 무효화/신호를 구현함(`core/src/fixed64.rs:42-94`). teul-cli formula div0는 `FormulaError::DivZero`로 runtime error화되어 전 표면이 같은 정책은 아님(`tools/teul-cli/src/runtime/formula.rs:235-247`). |
| 21 | 17.4 DetMath 함수 결정론 | 4447-4513 | sin/cos/tan/sqrt/exp/log/pow 등은 호스트 float 직접 호출 금지, DetMath LUT/차림새 사용 | 부분구현 | teul-cli는 `detmath::sin/cos` CORDIC 기반 함수를 사용함(`tools/teul-cli/src/runtime/detmath.rs:44-54`, `tools/teul-cli/src/runtime/eval.rs:2763-2772`)이고 sqrt는 fixed64 sqrt를 사용함(`:2746-2758`). 그러나 tan/asin/exp/log/pow와 차림새 `#빠름/#정밀`/LUT manifest 정책은 teul-cli 제품 경로에서 닫히지 않음. |
| 22 | 25.3 Reactive 패스 규칙 | 4514-4525 | 패스 시작 시 queue snapshot만 처리하고 tag/append 순으로 알림 처리 | 부분구현 | teul-cli 알림은 pending queue FIFO drain으로 처리됨(`tools/teul-cli/src/runtime/eval.rs:6692-6745`)이고 handler rank 순서는 있음(`:6837-6904`). 하지만 pass snapshot/tag sort reactive loop는 제품 teul-cli 알림 모델에 없음. |
| 23 | 25.4 `ReactiveMaxPass` | 4526-4562 | reactive pass 상한과 초과 정책 | 부분구현 | `core/src/gogae3.rs`에는 `reactive_max_pass` 파라미터와 상한 루프/진단 카운트가 있음(`core/src/gogae3.rs:50`, `:779-833`). 일반 world manifest `limits.reactive_max_pass`와 teul-cli 알림 runtime 중단 정책은 확인되지 않음. |
| 24 | 19.2 쓸감 리터럴 및 `@` 용도 | 4563-4604 | `"경로" 쓸감.` 정본, `@"..."` sugar, `@식별자` 금지, `숫자@단위` 전용 | 부분구현 | 단위 postfix `숫자@단위`는 teul-cli에 구현됨(`tools/teul-cli/src/lang/parser.rs:5856-5926`). `@"..."`/쓸감 sugar는 `lang/src/parser.rs`에는 있으나(`lang/src/parser.rs:2062`, `:3771-3880`), 제품 teul-cli parser에는 확인되지 않음. |
| 25 | 27.1 PinSpec 표준 | 4605-4667 | 함수 signature의 pin/type/josa/optional/default 규칙 | 부분구현 | seed params와 josa/pin suffix는 제품 parser에 있음(`tools/teul-cli/src/lang/parser.rs:4092-4127`, `:5291-5350`). 하지만 PinSpec의 단위/조사 registry 충돌 금지, optional/default 정규화 전체는 확인되지 않음. |
| 26 | 27.1.1 갈래 분기 `~에 따라` | 4668-4674 | 옵션/참거짓/고름씨를 `~에 따라`로 안전 분해 | 미구현 | 제품 parser에서 `~에 따라` match expression을 찾지 못함: `rg -n '에 따라|없으면|MATCH-|Coalesce' tools/teul-cli/src/lang tools/teul-cli/src/runtime`는 해당 표면 구현 근거 없음. |
| 27 | 27.1.1.1 문법 | 4675-4711 | `E에 따라 { 패턴이면 몸 ... }` 정본 문법 | 미구현 | #26과 동일. 현재 유사 분기는 `고르기` statement이며(`tools/teul-cli/src/lang/parser.rs:4137-4231`), `~에 따라` expression 문법은 아님. |
| 28 | 27.1.1.2 바인딩/스코프 | 4712-4726 | branch payload binding은 가지 내부 scope만 보임 | 미구현 | `~에 따라` AST/패턴 binding 자체가 없어 스코프 규칙 집행도 없음(#26). |
| 29 | 27.1.1.3 타입 규칙 | 4727-4739 | 대상은 갈래형이어야 하고 불가능 패턴은 오류 | 미구현 | `~에 따라` 타입 검사 경로 없음. 제품 `Value::None`/`Atom`은 있으나 갈래형 match type checker는 확인되지 않음(`tools/teul-cli/src/core/value.rs:255-270`). |
| 30 | 27.1.1.4 완전성/중복 | 4740-4755 | exhaustive/overlap/order 검사 | 미구현 | `고르기`에는 exhaustive 표지가 일부 있으나(`tools/teul-cli/src/lang/parser.rs:4149-4229`, `tools/teul-cli/src/runtime/eval.rs:1030-1047`), `~에 따라` variant completeness/overlap 검사는 없음. |
| 31 | 27.1.1.5 반환 타입 | 4756-4770 | 모든 branch 결과 타입 통합 | 미구현 | `~에 따라` expression이 없어 branch return type unification도 없음(#26). |
| 32 | 27.1.1.6 `없으면` sugar | 4771-4788 | `X 없으면 Y`를 option match로 정본화 | 미구현 | 제품 lang/runtime에서 `없으면`/coalesce 구현 없음: `rg -n '없으면|Coalesce' tools/teul-cli/src/lang tools/teul-cli/src/runtime` = NO_MATCH. |
| 33 | DOT-ACCESS-01 | 4789-4812 | 공백 없는 `.`는 member access, 문장 끝 `.`는 terminator | 구현됨 | postfix parser가 dot segment를 `Expr::FieldAccess`로 만든다(`tools/teul-cli/src/lang/parser.rs:4888-4914`). call/path parser도 dot segment를 읽고(`:5598-5612`), terminator는 별도 `consume_terminator`에서 처리됨(`:6076-6088`). |
| 34 | JOSA-SPLIT-01 | 4813-4870 | `사과를` 같은 조사 접미사를 scope 기반으로 자동 분리하고 canon은 `~` 삽입 | 부분구현 | 명시 `~` 조사/별칭 tokenization은 있음(`tools/teul-cli/src/lang/lexer.rs:606-641`)이고 call arg josa suffix도 있음(`tools/teul-cli/src/lang/parser.rs:5291-5328`). 하지만 `~` 없는 어절을 scope lookup으로 `S~조사`로 자동 분리하는 구현은 확인되지 않음. |
| 35 | 27.2 곳간(Registry) | 4871-5184 | compiler/IDE 공유 registry(term map, fatal terms, josa, reserved words 등) | 부분구현 | `tool/src/gate0_registry.rs`에 asset/unit registry가 있고(`tool/src/gate0_registry.rs:9-60`, `:159`), term map 일부도 `lang/src/term_map.rs`에 있음. 그러나 teul-cli 제품 parser/IDE 공유 JSON registry 전체와 SSOT fatal_terms 단일 근거는 확인되지 않음. |
| 36 | 20.3 파서 Reverse-Lookup | 5185-5225 | 조사/타입/선언 순서로 pin을 자동 확정 | 부분구현 | `:pin`, named arg, josa suffix는 `ArgBinding`에 들어감(`tools/teul-cli/src/lang/parser.rs:5255-5350`)이고 일부 builtin alias mapping이 있음(`:5270-5288`). registry 기반 조사→pin reverse lookup/모호성 실패 정책은 전역 구현으로 확인되지 않음. |
| 37 | 28. 설탕 구문 계층 | 5226-5274 | sugar는 정본 AST로 해소되고 mood metadata 보존 | 부분구현 | 일부 sugar/canon 전개는 제품 parser/canon에 있음(예: formula/template/boim 계열). 그러나 `어조:갈래씨` metadata, 형용사 상태 매핑, 암시적 임자 생성의 정본화 정책은 제품 경로에서 확인되지 않음. |
| 38 | §M1 수식 체계 | 5275-5278 | 수식은 입력/알맹이/표시 삼층으로 분리 | 부분구현 | teul-cli에는 `Formula` AST/value와 runtime formula parser/evaluator가 있음(`tools/teul-cli/src/lang/ast.rs:323-333`, `tools/teul-cli/src/runtime/formula.rs:79-119`). 다만 SSOT의 MathIR/표시층까지 닫힌 삼층 구현은 아님. |
| 39 | §M1.1 삼층 분리 | 5279-5283 | Syntax를 MathIR로 정규화하고 View는 rendering만 담당 | 부분구현 | formula body는 `format_formula_body`로 정규화된 text를 `Value::Math`에 저장함(`tools/teul-cli/src/runtime/eval.rs:5875-5886`). 해시 가능한 MathIR/표시층 분리 구현은 확인되지 않음. |
| 40 | §M1.2 `수식{...}` 표지 단일화 | 5284-5299 | `수식{}`만 formula marker이고 방언은 Atom tag | 부분구현 | parser는 `(#ascii)`/`(#ascii1)` 뒤 `수식{...}` 또는 formula block을 `Expr::Formula`/`FormulaEval`로 만든다(`tools/teul-cli/src/lang/parser.rs:5656-5713`). 하지만 지원 tag는 `#ascii/#ascii1`뿐이고 tagless default/latex/raytag/그림은 없음(`tools/teul-cli/src/lang/ast.rs:406-423`). |
| 41 | §M1.3 `#ascii` vs `#ascii1` | 5300-5334 | ascii는 암묵곱 금지, ascii1은 1글자 변수와 암묵곱 허용 | 부분구현 | formula parser는 ascii1에서만 implicit multiplication을 허용함(`tools/teul-cli/src/runtime/formula.rs:476-493`). ascii1 ident는 `split_ascii1_ident`로 1글자+숫자 단위로 분리됨(`:300`, `:415-427`). 다만 `acc`를 오류로 막기보다는 `a*c*c`류로 쪼개는 동작이라 SSOT 오류 정책과 완전 일치하지 않음. |
| 42 | §M1.4 결정성 규칙 | 5335-5338 | 동일 의미 수식은 동일 MathIR, 평가는 Fixed64+단위 | 부분구현 | formula evaluation은 Fixed64/UnitDim을 사용하고 단위 mismatch/div0를 deterministic error로 낸다(`tools/teul-cli/src/runtime/formula.rs:190-247`). 의미 동치 MathIR 정규화/해시까지는 확인되지 않음. |
| 43 | §M1.5 MathIR v1 정본 | 5339-5355 | ascii/ascii1을 MathIR v1로 정규화하고 De Bruijn 등 사용 | 미구현 | 제품 teul-cli에는 내부 `FormulaAst`는 있으나 MathIR v1/De Bruijn/DetBin 직렬화 구현은 확인되지 않음: `rg -n 'MathIR|De Bruijn|DetBin|debruijn' tools/teul-cli/src core/src tool/src`는 제품 MathIR 근거 없음. |
| 44 | §M2.1 실행과 결과 분리 | 5356-5396 | `미분하기/적분하기/풀기`는 동작 호출, `-서`는 금지, `해서` pipe | 부분구현 | runtime builtin 목록에는 `미분하기`와 `풀기`가 있고(`tools/teul-cli/src/runtime/eval.rs:5748-5756`, `:3868-3914`), formula fill parser도 있음(`tools/teul-cli/src/lang/parser.rs:5505-5526`). 그러나 `해서` pipe/call-tail 정본 및 `-서` 금지 전용 진단은 제품 경로에서 확인되지 않음. |
| 45 | §M2.2.1 변수 주입 규칙 | 5397-5405 | named binding만 허용, 누락/여분/없음/중복 key는 FATAL | 부분구현 | `풀기` pack 경로는 required/provided를 비교해 누락/여분/없음 값을 오류로 처리함(`tools/teul-cli/src/runtime/eval.rs:3880-3914`). `FormulaFill` 직접 경로는 named binding으로 map을 만들지만 중복 key를 별도 오류로 막지 않고 덮어쓸 수 있음(`tools/teul-cli/src/runtime/eval.rs:5891-5921`). |

## 요약

| 판정 | 건수 |
|---|---:|
| 구현됨 | 1 |
| 부분구현 | 31 |
| 미구현 | 13 |
| 확인불가 | 0 |
| 합계 | 45 |

## 심각도가 높은 미구현 후보

- 다중 디스덧댐은 제품 teul-cli 경로에서 후보/구체성/모호성 오류 구현이 없다.
- `늘지켜보고` 표면 문법과 `덧댐거부` 정책은 계약 시스템 일부와 별개로 미착륙이다.
- 말결 `$...` 표면과 결합 규칙은 `말결값` builtin 일부만 있고 문장 표면이 없다.
- 누리 query 표면 `임자들`, 군집 블록 `모두`, query filter는 core archetype API와 연결되지 않았다.
- `~에 따라` 갈래 match와 `없으면` sugar는 제품 lang/runtime에 없다.
- MathIR v1/De Bruijn/DetBin 정본은 formula parser/evaluator와 별개로 아직 제품 구현이 없다.
