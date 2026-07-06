# SSOT_MUST_RULE_AUDIT_TIER2_V1

> 작성: Codex (2026-07-06)
> 범위: `docs/ssot/ssot/SSOT_LANG_v24.12.9.md` 2690~3978행, MUST 섹션 45개
> 성격: 진단 전용. `docs/ssot/**`, 코드, pack, golden 수정 없음.

## 방법

- 대상 추출: `rg -n "^#{2,4} .*(MUST)" docs/ssot/ssot/SSOT_LANG_v24.12.9.md`
- Tier2 기준: 헤더 시작 행 `>= 2690 && < 3979`
- 제품 근거 경로: `tools/teul-cli/src/**`, `core/src/**`, 필요시 `tool/src/**`
- 판정 기준: `구현됨` / `부분구현` / `미구현` / `확인불가`

## 판정표

| # | 규칙 ID/제목 | 줄 범위 | 핵심 주장 | 판정 | 근거(file:line 또는 검색식) |
|---:|---|---:|---|---|---|
| 1 | 5.10.4 정본화 규칙 | 2690-2790 | 고름씨 tag/constructor를 정본화하고 payload/표면을 안정화 | 부분구현 | bare atom은 AST/parser에 있음(`tools/teul-cli/src/lang/ast.rs:298`, `tools/teul-cli/src/lang/parser.rs:5045-5048`). 다만 `고름씨` 타입 surface/constructor payload 정본화는 제품 lang/runtime에 없음: `rg -n '고름씨' tools/teul-cli/src/lang tools/teul-cli/src/runtime`는 제품 타입 구현 근거를 주지 않음. |
| 2 | 6.2.1 정의 헤더 공통 문법 | 2791-2803 | 정의 헤더의 공통 형식과 kind/tag를 일관되게 파싱 | 부분구현 | seed 정의 파서는 `name:kind = { ... }` 계열을 처리하고(`tools/teul-cli/src/lang/parser.rs:3869-3925`), seed kind는 `셈씨/움직씨` 정본 kind를 갖는다(`tools/teul-cli/src/lang/ast.rs:16-30`). SSOT 표 전체의 공통 헤더를 테이블로 집행하는 경로는 확인되지 않음. |
| 3 | 6.2.2 말 이음씨 | 2804-2817 | 기호 없는 사용자 정의 중위연산을 선언/호출 | 미구현 | 제품 teul-cli 경로에서 사용자 정의 이음씨/중위연산 선언을 찾지 못함: `rg -n '이음씨|중위연산|기호 이음|확장 이음|operator registry|precedence registry' tools/teul-cli/src` = NO_MATCH. |
| 4 | 6.2.3 기호 이음씨 | 2818-2841 | 확장 이음기호를 사용자 정의 연산자로 선언 | 미구현 | 제품 parser는 builtin precedence chain만 갖고 사용자 정의 기호 이음씨 선언/registry가 없음(`tools/teul-cli/src/lang/parser.rs:4711-4878`). 위 검색식도 NO_MATCH. |
| 5 | 6.2.4 확장 이음기호 집합 | 2842-2867 | 렉서가 확장 이음기호 집합을 토큰화 | 미구현 | 제품 lexer/token에는 고정 토큰 집합만 있고 사용자 확장 이음기호 token class가 없음(`tools/teul-cli/src/lang/token.rs:61-77`, `tools/teul-cli/src/lang/lexer.rs:180-340`). 확장 이음 검색식은 NO_MATCH. |
| 6 | 6.2.5 사용자 정의 이음씨 우선순위 | 2868-2900 | 사용자 연산자의 우선순위를 선언하고 결정적으로 적용 | 미구현 | 표현식 파서는 고정 precedence 함수 체인이다(`tools/teul-cli/src/lang/parser.rs:4711-4878`). 사용자 precedence registry 검색 `rg -n 'precedence registry|operator registry|이음씨' tools/teul-cli/src` = NO_MATCH. |
| 7 | 6.3 표현식 우선순위 | 2901-2926 | 논리/비교/range/산술/unary/postfix 우선순위 고정 | 구현됨 | `parse_expr`가 `parse_logical_or -> and -> comparison -> range -> additive -> multiplicative -> unary -> postfix` 순서로 내려간다(`tools/teul-cli/src/lang/parser.rs:4711-4898`). |
| 8 | 6.3.1 `!` 토큰 모호성 대장 | 2927-2969 | `!`의 postfix/prefix/이름 꼬리 등 모호성을 닫힌 규칙으로 해소 | 부분구현 | `입력키?`/`입력키!` 같은 이름은 ident로 다뤄지는 테스트/금지 목록이 있음(`tools/teul-cli/src/lang/lexer.rs:1163`, `tools/teul-cli/src/lang/parser.rs:3729`). 그러나 `TokenKind::Bang` 같은 독립 토큰과 SSOT ledger 전체 구현은 없음(`tools/teul-cli/src/lang/token.rs` 확인). |
| 9 | 7.2 피드백과 시간 지연: 이전값보기 | 2970-2985 | 현재 마디 계산에서 이전 값 읽기 표면을 제공 | 미구현 | 제품 경로 검색에서 표면/런타임 구현을 찾지 못함: `rg -n '이전값보기' tools/teul-cli/src core/src tool/src` = NO_MATCH. |
| 10 | 7.3 흐름씨-훅 위상 분리 | 2986-3028 | `<<-` 흐름씨 fixed-point와 hook 실행 위상을 분리 | 미구현 | GOAL-B 실측에서 제품 lexer/parser는 `<<-`를 받지 못하고 네 케이스가 모두 parse fail임을 확인함(`docs/context/reports/FLOW_HOOK_PHASE_SEPARATION_VERIFICATION_V1.md:12`, `:35-50`). 제품 코드 검색 `rg -n 'E_FLOW_MULTIPLE_SOURCE_CONFLICT|E_FLOW_CIRCULAR_REFERENCE|DoubleArrow|<<-' tools/teul-cli/src core/src tool/src` = NO_MATCH. |
| 11 | 8. 진입점 및 이벤트 기반 실행 모델 | 3029-3092 | 시작/매마디/끝/조건/알림 기반 실행 모델 | 부분구현 | parser가 start/end/every/every-n/condition hook을 파싱함(`tools/teul-cli/src/lang/parser.rs:1213-1277`). runtime은 seed def/init/start/every/end hook을 순서대로 실행함(`tools/teul-cli/src/runtime/eval.rs:630-834`). 다만 SSOT의 전체 이벤트 모델과 흐름씨 결합은 #10처럼 미착륙. |
| 12 | 8.3 알림 디스패치 순서 | 3093-3104 | 알림 수신 handler를 결정적 순서로 처리 | 구현됨 | 송신은 pending queue에 들어가고 FIFO로 drain됨(`tools/teul-cli/src/runtime/eval.rs:6692-6745`). 수신 handler는 rank 0~3 순회로 typed+binding+condition 우선순위를 결정함(`tools/teul-cli/src/runtime/eval.rs:6837-6904`). |
| 13 | 8.4 시스템/훅 실행 순서 | 3105-3113 | 시스템/훅 실행 순서를 결정적으로 고정 | 부분구현 | runtime은 top-level을 init/seed/hook/lifecycle로 분리하고 start, tick, every, every-n, becomes, while, proof, end 순서로 실행함(`tools/teul-cli/src/runtime/eval.rs:630-834`). 그러나 SSOT의 시스템 단위 전체 정렬키/flow fixed-point 연동은 확인되지 않음. |
| 14 | 9. 단위 시스템 | 3114-3137 | 단위 literal, 단위식, 수량 차원을 값에 보존 | 구현됨 | numeric literal postfix `@`와 단위식 parser가 있음(`tools/teul-cli/src/lang/parser.rs:5856-5926`). `UnitDim`/`UnitExpr`/`eval_unit_expr`가 차원과 scale을 계산함(`tools/teul-cli/src/core/unit.rs:15-40`, `:189-215`). |
| 15 | 9.4 차원 대수 | 3138-3168 | 곱/나눗셈/거듭제곱 차원 대수를 적용 | 구현됨 | `UnitDim::add/scale`와 `eval_unit_expr`가 단위 factor exp를 합성함(`tools/teul-cli/src/core/unit.rs:40-63`, `:189-215`). 런타임은 dimensionless 요구를 검사하는 지점들을 갖는다(`tools/teul-cli/src/runtime/eval.rs:1287-1326`, `:9238-9240`). |
| 16 | 9.5 자동 환산 | 3169-3181 | 같은 차원 단위의 scale/온도 환산을 적용 | 구현됨 | 런타임은 단위 literal 평가 때 `eval_unit_expr` scale을 곱하고 온도 literal을 정규화함(`tools/teul-cli/src/runtime/eval.rs:2117-2130`). 템플릿 출력도 대상 단위로 변환한다(`tools/teul-cli/src/runtime/template.rs:589-618`). |
| 17 | 10. 조사/별칭 시스템 | 3182-3194 | 조사/별칭을 token/역할로 canonicalize | 부분구현 | lexer는 `canonicalize_josa` 결과를 `TokenKind::Josa`로 낸다(`tools/teul-cli/src/lang/lexer.rs:606-625`), dialect wrapper도 josa canonicalize를 제공함(`tools/teul-cli/src/lang/dialect.rs:23-25`). 말씨별 전체 role 정책/진단은 부분적이다. |
| 18 | 10.2.1 이름 바인딩 | 3195-3217 | named argument와 명시 pin을 지원 | 구현됨 | `parse_call_arg`가 `ident = expr` named arg를 `resolved_pin`으로 만들고(`tools/teul-cli/src/lang/parser.rs:5330-5350`), `parse_arg_suffix`는 `:pin`과 josa suffix를 분리한다(`:5291-5328`). |
| 19 | 10.2.4 정본 출력 우선순위 | 3218-3255 | canon 출력에서 명시/별칭/권장형 우선순위를 적용 | 부분구현 | canon은 alias warning과 정본 출력 경로를 갖고(`tools/teul-cli/src/canon.rs:4143-4170`), parser binding reason에는 `UserFixed/Dictionary/Positional`이 있음(`tools/teul-cli/src/lang/ast.rs:369-376`). 다만 SSOT 우선순위 표 전체를 제품 canon에 강제하는 단일 테이블은 확인되지 않음. |
| 20 | 10.3 바인딩 스타일 통일 | 3256-3266 | named, josa, positional binding 스타일을 통일 처리 | 부분구현 | `ArgBinding`은 `josa/resolved_pin/binding_reason`을 같이 보존하고(`tools/teul-cli/src/lang/parser.rs:5255-5263`, `:5291-5328`), 일부 builtin은 binding 이름 alias를 positional로 내린다(`:5270-5288`). 전 builtin/사용자 seed 전체의 통일 검사는 확인되지 않음. |
| 21 | 10.4 조사/별칭 토큰화 알고리즘 | 3267-3290 | `~` 뒤를 조사 우선, 아니면 별칭으로 토큰화 | 구현됨 | `read_josa`가 먼저 ident-only josa candidate를 읽어 `canonicalize_josa`를 시도하고, 실패하면 rewind 후 full tilde alias로 읽는다(`tools/teul-cli/src/lang/lexer.rs:606-641`). |
| 22 | 10.4.1 숫자 리터럴 접미 분리 | 3291-3316 | 숫자 뒤 접미를 결정적으로 분리 | 미구현 | 제품 숫자 parser는 fixed64 숫자를 만들고 unit postfix는 별도 `@`로 처리한다(`tools/teul-cli/src/lang/parser.rs:5856-5865`). SSOT의 일반 숫자 접미 분리/진단 구현 근거는 확인되지 않음: `rg -n 'i32|u64|f32|suffix' tools/teul-cli/src/lang tools/teul-cli/src/runtime`는 전역 suffix 정책을 보이지 않음. |
| 23 | 10.5 명시적 접미 접착자 `@`, `:`, `~` | 3317-3346 | 단위/핀/조사·별칭 접착자를 구분 | 부분구현 | `@` 단위 postfix parser가 있고(`tools/teul-cli/src/lang/parser.rs:5856-5926`), `:pin`/josa suffix parser가 있으며(`:5291-5328`), `~` josa/alias lexer가 있다(`tools/teul-cli/src/lang/lexer.rs:606-641`). 전체 suffix chain 규격은 #24처럼 부분. |
| 24 | 10.5.1 접미 체인 파싱 알고리즘 | 3347-3429 | 접미 chain을 순서/역할별로 결정 파싱 | 부분구현 | call arg suffix와 unit postfix는 각각 구현되어 있음(`tools/teul-cli/src/lang/parser.rs:5291-5328`, `:5856-5926`). 그러나 SSOT의 통합 chain parser/모호성 진단 전체는 별도 구현으로 확인되지 않음. |
| 25 | 10.5.2 바인딩 모호성 진단 + LSP QuickFix | 3430-3462 | 바인딩 모호성을 진단하고 QuickFix 제공 | 미구현 | LSP 서버에 일반 QuickFix 골격은 있음(`tool/src/lsp/server.rs:386`, `:720-728`). 그러나 조사/핀 바인딩 모호성 전용 진단/QuickFix는 제품 검색으로 확인되지 않음: `rg -n 'binding.*ambig|ambig.*binding|QuickFix.*josa|Josa.*QuickFix' tools/teul-cli/src tool/src/lsp` = NO_MATCH. |
| 26 | STYLE-CANON-01 정본 출력 | 3463-3589 | ko 정본 표면으로 출력하고 alias/style을 안정화 | 부분구현 | canon 경로와 warning 출력은 존재함(`tools/teul-cli/src/canon.rs:4135-4170`, `:6085-6107`). 다만 SSOT STYLE-CANON 전체 표를 단일 데이터로 집행하거나 모든 항목을 고정하는 근거는 확인되지 않음. |
| 27 | DIALECT-KO-01 ko 정본 | 3590-3607 | ko 말씨가 기본 정본이며 비정본은 ko로 수렴 | 부분구현 | 기본 source는 ko token을 그대로 canonicalize하고, 다른 말씨 tag가 없으면 en/ja token이 활성화되지 않는 테스트가 있음(`tools/teul-cli/src/lang/dialect.rs:34-45`). 다만 전체 ko 정본 표 기반 검사는 부분. |
| 28 | DIALECT-SYM3-01 sym3 별칭 | 3608-3707 | sym3 기호 별칭을 token으로 허용/정본화 | 부분구현 | lexer가 `try_read_sym3_token`을 먼저 시도하고 `DialectConfig::sym3_tokens`/`canonicalize_symbol`로 정본 token을 만든다(`tools/teul-cli/src/lang/lexer.rs:180`, `:685-707`). lint에도 sym3 hint가 있음(`tools/teul-cli/src/cli/lint.rs:185-193`). 전체 sym3 별칭 표의 제품 검증은 부분. |
| 29 | JOSA-ROLE-01 role 집합 | 3708-3721 | 조사 role 집합을 닫힌 set으로 정의 | 부분구현 | 조사 token은 role 문자열을 담는 `TokenKind::Josa(String)` 형태임(`tools/teul-cli/src/lang/token.rs:41`)이고 dialect canonicalize가 있다(`tools/teul-cli/src/lang/dialect.rs:23-25`). 닫힌 role enum/전역 role set 타입은 확인되지 않음. |
| 30 | JOSA-ROLE-02 말씨별 권장형/허용형 | 3722-3739 | 말씨별 권장/허용 조사를 구분 | 부분구현 | dialect/lint는 말씨 tag와 일부 sym3 권장을 다룬다(`tools/teul-cli/src/lang/dialect.rs:34-63`, `tools/teul-cli/src/cli/lint.rs:211-212`). 조사별 권장형/허용형 matrix를 제품에서 집행하는 근거는 확인되지 않음. |
| 31 | JOSA-ROLE-03 무표 처리 | 3740-3766 | 무표는 제한된 위치에서만 허용 | 부분구현 | call arg는 josa/pin이 없으면 `BindingReason::Positional`로 처리된다(`tools/teul-cli/src/lang/parser.rs:5255-5263`, `:5291-5328`). 무표 허용 위치를 SSOT 표대로 제한하는 전역 검사는 확인되지 않음. |
| 32 | 11.1 기본 어미 | 3767-3789 | 기본 어미/꼬리 표면을 닫힌 규칙으로 해석 | 부분구현 | 제품에는 제한적 bare call/seed alias 기반이 있으나 일반 꼬리 해석기는 없음(`tools/teul-cli/src/lang/parser.rs:492-493`, `:4018-4045`). `rg -n 'E_CALL_TAIL|CALL_TAIL|tail.*candidate|candidate.*tail' tools/teul-cli/src` = NO_MATCH. |
| 33 | 11.3.1 원칙 | 3790-3794 | 어간/활용 모호성은 추측하지 않고 오류 | 미구현 | Q20/T1 근거처럼 제품 경로에 call-tail ambiguity 오류/후보 수집이 없다: `rg -n 'E_CALL_TAIL|CALL_TAIL|tail.*candidate|candidate.*tail' tools/teul-cli/src` = NO_MATCH. |
| 34 | 11.3.2 어간 별칭 `~` | 3795-3809 | `돕~도우`류 어간 별칭을 등록 | 부분구현 | seed alias parser는 `~별명`을 읽고 중복 alias를 거부한다(`tools/teul-cli/src/lang/parser.rs:4018-4045`). 그러나 별칭을 활용 꼬리 후보 해석/모호성 검사와 연결하는 제품 근거는 없음(#33). |
| 35 | 11.3.3 교대 트리거 | 3810-3820 | `기/하기` 등 trigger로 어간 교대를 적용 | 미구현 | 제품 경로에 trigger 집합/활용 파이프라인이 확인되지 않음: `rg -n 'CALL_TAIL|E_CALL_TAIL|tail.*candidate|candidate.*tail' tools/teul-cli/src` = NO_MATCH. |
| 36 | 11.3.4 정본 처리 파이프라인 | 3821-3841 | 후보 생성, 중복 검사, AST 확정의 3-step 처리 | 미구현 | 제품 parser는 seed alias 선언만 처리하고 call-tail 후보 생성/2개 이상 오류가 없다(`tools/teul-cli/src/lang/parser.rs:4018-4045`). `E_CALL_TAIL_*` 검색도 NO_MATCH. |
| 37 | 11.3.5 조사 vs 어미 구분 | 3842-3846 | 조사와 어미를 충돌 없이 구분 | 부분구현 | `~` 뒤 token은 josa 우선 후 alias rewind로 구분된다(`tools/teul-cli/src/lang/lexer.rs:606-641`). 하지만 어미 활용 trigger까지 포함한 전체 구분은 #35처럼 미구현. |
| 38 | 11.3.6 관문 0 골든 데이터 | 3847-3880 | 어간 별칭/꼬리 해석 goldens가 제품 동작을 잠금 | 부분구현 | conformance pack은 존재하지만 Q20 진단처럼 `stem_alias_ambiguous`가 의도 오류 없이 실행되는 상태라 제품 모호성 집행이 닫히지 않았다(`docs/context/briefs/BRIEF_AMBIGUITY_ENFORCEMENT_DIAGNOSIS_V1.md:84-95`). |
| 39 | 11.4.1 흐름값 결정 규칙 | 3881-3885 | pipe/흐름값을 값으로 결정 | 부분구현 | `TokenKind::Pipe`와 seed literal param pipe parser는 있음(`tools/teul-cli/src/lang/token.rs:76`, `tools/teul-cli/src/lang/parser.rs:5148-5188`). 그러나 SSOT의 `흐름값` 타입/결정 규칙을 제품 런타임에서 확인하지 못함. |
| 40 | 11.4.2 핀 주입 규칙 | 3886-3895 | 흐름값/pipe에서 target pin을 결정적으로 주입 | 부분구현 | `ArgBinding`은 `resolved_pin`과 `binding_reason`을 보존하고 `:pin`을 처리함(`tools/teul-cli/src/lang/parser.rs:5255-5350`). 흐름값 pipe 전체에 대한 target injection 의미론은 확인되지 않음. |
| 41 | 11.4.3 결정적 스케줄링 | 3896-3901 | 흐름/핀 주입 스케줄이 결정적 | 부분구현 | runtime hook/알림 처리 자체는 결정적 순서가 있다(#12, #13). 다만 흐름값/pipe 스케줄링 표면은 부분 구현이라 SSOT 전체는 닫히지 않음. |
| 42 | 12. 알림/자원/목표 ID화 | 3902-3914 | 알림, 자원, 목표를 ID로 안정 식별 | 부분구현 | 자원 handle/display는 존재함(`core/src/lib.rs:29`, `tools/teul-cli/src/core/value.rs:281`). 알림은 pending signal payload/receiver로 처리됨(`tools/teul-cli/src/runtime/eval.rs:6692-6745`). 목표까지 포함한 통합 ID 체계는 확인되지 않음. |
| 43 | 12.2 ID 생성 | 3915-3933 | ID 생성이 결정적이고 충돌 가능성을 관리 | 부분구현 | 상태 hash와 자원 handle 경로는 존재함(`tools/teul-cli/src/core/hash.rs:8-10`, `tool/src/gate0_registry.rs:51-60`). 알림/목표/자원 전체의 SSOT ID 생성 규격을 단일 구현으로 확인하지 못함. |
| 44 | 13. 출력 규칙: 상태 기반 | 3934-3949 | 출력은 상태에 쌓고 rendering은 상태 기반으로 처리 | 구현됨 | `보여주기`/`보임`은 runtime에서 상태 tag에 값을 쌓는다(`tools/teul-cli/src/runtime/eval.rs:958-993`, `:1365-1405`). CLI serialization도 `Stmt::Boim`을 JSON으로 다룬다(`tools/teul-cli/src/cli/run.rs:6226`, `:6610`). |
| 45 | 13.2A `보임 {}` structured view sugar | 3950-3978 | `보임 { key: expr. }`를 structured view로 파싱/평가 | 구현됨 | parser가 `보임 { ... }`/`보임: { ... }` 블록을 `Stmt::Boim`으로 만든다(`tools/teul-cli/src/lang/parser.rs:1316-1382`). runtime `eval_boim`은 table row와 graph point 상태 tag를 갱신한다(`tools/teul-cli/src/runtime/eval.rs:1365-1405`). |

## 요약

| 판정 | 건수 |
|---|---:|
| 구현됨 | 9 |
| 부분구현 | 25 |
| 미구현 | 11 |
| 확인불가 | 0 |
| 합계 | 45 |

## 심각도가 높은 미구현 후보

- 사용자 정의 이음씨/기호 이음씨/우선순위 registry는 SSOT에서 MUST지만 제품 teul-cli 경로에 없다.
- `이전값보기` 표면은 제품 lang/core/tool 검색에서 구현 근거가 없다.
- `<<-` 흐름씨와 flow fixed-point는 GOAL-B 실측처럼 제품 파서 단계에서 막힌다.
- 조사/핀 바인딩 모호성 LSP QuickFix는 일반 LSP QuickFix 골격만 있고 전용 구현은 없다.
- `기/하기` 어간 trigger, call-tail 후보 생성, ambiguous/unresolved 오류는 제품 코드에 없다.
- 숫자 literal 일반 suffix 분리와 전체 suffix chain 진단은 `@` 단위/`:pin`/`~` 일부만 구현된 상태다.
