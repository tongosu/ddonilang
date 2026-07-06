# SSOT_MUST_RULE_AUDIT_TIER1_V1

> 작성: Codex (2026-07-06)
> 범위: `docs/ssot/ssot/SSOT_LANG_v24.12.9.md` 1~2689행, MUST 섹션 44개
> 성격: 진단 전용. `docs/ssot/**`, 코드, pack, golden 수정 없음.

## 방법

- 대상 추출: `rg -n "^#{2,4} .*(MUST)" docs/ssot/ssot/SSOT_LANG_v24.12.9.md`
- Tier1 기준: 헤더 시작 행 `< 2690`
- 제품 근거 경로: `tools/teul-cli/src/**`, `core/src/**`, 필요시 `tool/src/**`
- 판정 기준: `구현됨` / `부분구현` / `미구현` / `확인불가`

## 판정표

| # | 규칙 ID/제목 | 줄 범위 | 핵심 주장 | 판정 | 근거(file:line 또는 검색식) |
|---:|---|---:|---|---|---|
| 1 | §V23.2.3B ResultPolicyV1 | 525-742 | 실패 가능 작업은 `#성공/#실패`, 선택 조회는 `T?`/`없음`으로 표현 | 부분구현 | `Value::None`은 있음(`tools/teul-cli/src/core/value.rs:255-257`, `:276`), 방정식 풀이 pack 표시가 `#성공/#실패`를 출력함(`tools/teul-cli/src/core/value.rs:486-503`, `tool/src/ddn_runtime.rs:13217-13238`). 다만 `ResultPolicyV1` 전역 강제 검색 `rg -n 'ResultPolicy|#성공|#실패' tools/teul-cli/src core/src tool/src`상 전역 타입/정책 검사는 없음. |
| 2 | §0A AI/LLM 코드 생성 규칙 | 743-804 | AI 생성 코드는 정본 키워드만 쓰고 레거시 계약어를 피해야 함 | 부분구현 | 제품 lexer는 `바탕으로/다짐하고` 등 정본 키워드를 토큰화함(`tools/teul-cli/src/lang/lexer.rs:649-671`). canon 전용 lexer는 `전제하에/보장하고` 별칭도 받음(`tools/teul-cli/src/canon.rs:508-511`). AI 산출물 전용 강제 경로는 확인되지 않음. |
| 3 | [V18-00C] 실행 꼬리 동치 | 805-823 | `기/하기` 등 호출 꼬리는 같은 실행 AST로 해석되어야 함 | 부분구현 | 제품 파서에는 제한적인 zero-arg prefix call 경로가 있음(`tools/teul-cli/src/lang/parser.rs:492-493`). 그러나 일반 호출 꼬리 후보 수집/동치 강제는 없음: `rg -n 'E_CALL_TAIL|CALL_TAIL|tail.*candidate|candidate.*tail' tools/teul-cli/src` = NO_MATCH. |
| 4 | CALL-TAIL-RESOLVE-01 해석 규칙 | 824-874 | 꼬리 후보 전부 생성, 0개/2개 이상이면 각각 미해결/모호성 오류 | 미구현 | 제품 경로에 `E_CALL_TAIL_UNRESOLVED`/`E_CALL_TAIL_AMBIGUOUS`가 없음: `rg -n 'E_CALL_TAIL|CALL_TAIL|tail.*candidate|candidate.*tail' tools/teul-cli/src` = NO_MATCH. Q20 진단도 같은 결론을 기록함(`docs/context/briefs/BRIEF_AMBIGUITY_ENFORCEMENT_DIAGNOSIS_V1.md:84-95`). |
| 5 | [V18-00D] `X` vs `X하` 이름 충돌 금지 | 875-891 | 같은 스코프에서 `X`와 `X하` 별도 정의 금지 | 미구현 | 씨앗 별칭 파서는 `~별명` 중복만 확인함(`tools/teul-cli/src/lang/parser.rs:4018-4045`). `X/X하` 충돌 검사나 `E_CALL_TAIL_AMBIGUOUS` 제품 오류는 없음: `rg -n 'E_CALL_TAIL|CALL_TAIL' tools/teul-cli/src` = NO_MATCH. |
| 6 | [V18-00A] 정본 어휘·별칭 정책 | 892-905 | 순우리말 정본 우선, 별칭은 입력 허용/정본 유도 | 부분구현 | 정본 키워드 토큰화는 있음(`tools/teul-cli/src/lang/lexer.rs:649-681`), canon 별칭 일부도 있음(`tools/teul-cli/src/canon.rs:508-511`, `:4143-4159`). 전체 SSOT 용어 맵 기반 lint/승격 정책은 없음. |
| 7 | 용어 정책 | 906-910 | `곳간/쓸감/바탕/샘` 정본으로 수렴, `자원.` 폐기 | 부분구현 | `바탕`은 lexer에서 `Salim`으로 토큰화됨(`tools/teul-cli/src/lang/lexer.rs:653-655`), 런타임 경로도 `살림/바탕/샘`을 인식함(`tools/teul-cli/src/runtime/eval.rs:1988-1992`). 그러나 값 표시에는 아직 `자원#...`이 남아 있음(`tools/teul-cli/src/core/value.rs:281`, `:301`, `core/src/platform.rs:340`). |
| 8 | §TERM 순우리말 정본/별칭 | 911-934 | 정본 하나, 별칭은 입력 전용, 자동 수정 금지/승인 패치 | 부분구현 | canon은 별칭 경고를 생성함(`tools/teul-cli/src/canon.rs:4143-4159`)이고 patch CLI도 존재하지만, SSOT TERM-MAP 전역 기반 승인형 자동 패치 파이프라인은 확인되지 않음: `rg -n 'TERM-MAP|TERM_MAP|fatal_terms' tools/teul-cli/src core/src tool/src`는 `tool/src/main.rs:60`, `:749`의 버전 문자열만 확인. |
| 9 | [V18-00A-1] TERM-MAP 단일 소스 | 935-992 | TERM-MAP을 lint/canon의 단일 근거로 사용하고 FATAL/LEGACY 정책 강제 | 미구현 | 제품 경로에서 기계 TERM-MAP 테이블/추출/치명어 테이블을 찾지 못함: `rg -n 'TERM-MAP|TERM_MAP|fatal_terms' tools/teul-cli/src core/src tool/src` 결과는 `tool/src/main.rs:60`, `:749`의 `TERM_MAP_VERSION`뿐. |
| 10 | 기계 판독용 TERM-MAP 블록 | 993-1354 | SSOT JSON 블록을 직접 추출해 lint 상수/해시 근거로 사용 | 미구현 | `rg -n 'TERM-MAP|TERM_MAP|fatal_terms|term map' tools/teul-cli/src core/src tool/src`에서 extractor/codegen 없음. `tool/src/main.rs:60`의 버전 상수만 존재. |
| 11 | [V18-00B] 버전 표기 정책 | 1355-1370 | `r1/r2` 대신 semver식 PATCH/MINOR/MAJOR 사용 | 미구현 | 버전 문자열 파싱은 있음(`tool/src/ai_prompt.rs:325-344`, `tools/teul-cli/src/ai_prompt.rs:290-309`)이나 SSOT 문서/언어 리비전 표기 금지 검사는 없음. `rg -n 'revision|r1|r2' tools/teul-cli/src core/src tool/src`는 정책 집행 지점을 보여주지 않음. |
| 12 | [V18-01A] 움직씨 호출과 `}하고` 경계 | 1371-1434 | `먹기.` 즉시 실행, `{...}`는 thunk, `{...}하고.`은 thunk 실행 | 부분구현 | 일반 문장식 실행은 `Stmt::Expr`가 평가됨(`tools/teul-cli/src/runtime/eval.rs:968-970`). 제한적 bare call 파서도 있음(`tools/teul-cli/src/lang/parser.rs:492-493`). 그러나 `{...}하고.` thunk 실행 문법/AST는 제품 경로에서 확인되지 않음: `rg -n 'Thunk|하고|\\}하고' tools/teul-cli/src/lang tools/teul-cli/src/runtime` 근거 없음. |
| 13 | [V18-02] `~해보고:` + `그것` | 1435-1447 | action 실행 결과를 `그것`에 바인딩하고 `고르기`로 분기 | 미구현 | 제품 lang/runtime 경로에서 해당 표면을 찾지 못함: `rg -n '해보고|해보기|TryLook|그것' tools/teul-cli/src/lang tools/teul-cli/src/runtime` = NO_MATCH. |
| 14 | [V18-03] `고르기:` | 1448-1513 | 결정적 선택 분기, 기본 `아니면`/완전성 검출 | 구현됨 | 파서가 `고르기`를 파싱하고 `아니면` 또는 완전성 표지를 요구함(`tools/teul-cli/src/lang/parser.rs:4137-4231`). 런타임은 분기를 순서대로 평가하고 첫 참 분기를 실행함(`tools/teul-cli/src/runtime/eval.rs:1027-1047`). |
| 15 | 계약 정본 문법 | 1514-1532 | 계약 블록은 `아니면` 필수, `맞으면` 선택 | 구현됨 | `parse_contract_stmt`가 `아니면` 없으면 오류를 내고(`tools/teul-cli/src/lang/parser.rs:4562-4570`), `맞으면` 블록은 선택적으로 파싱함(`:4587-4607`). |
| 16 | 계약 물림 의미론 | 1533-1550 | 계약 실패 시 상태/알림/open checkpoint 롤백 및 재검사 | 구현됨 | 계약 프레임이 상태/알림/open checkpoint를 저장함(`tools/teul-cli/src/runtime/eval.rs:1801-1816`), rollback이 상태와 pending signal/open을 복원함(`:1828-1844`). 계약 실행부는 실패 시 else 실행/재검사/rollback을 수행함(`tools/teul-cli/src/runtime/eval.rs:1153-1260`). |
| 17 | [V18-05] 운(Random) 소비 규칙 | 1551-1569 | RNG는 결정적으로 소비되고 조건/계약/검증에서는 금지 | 부분구현 | 런타임 RNG 상태와 deterministic next 함수는 있음(`tools/teul-cli/src/runtime/eval.rs:92`, `:5575-5584`), `무작위*` builtin도 있음(`:4292-4333`). 그러나 조건/계약/고르기 조건에서 RNG 호출을 금지하는 파서 검사는 확인되지 않음(`parse_condition_expr`는 일반 expr 파싱, `tools/teul-cli/src/lang/parser.rs:4621-4673`). |
| 18 | [V18-05A] HeadSpec | 1570-1634 | 블록 헤더는 닫힌 집합, 임의 사용자 헤더 금지 | 부분구현 | 제품 파서는 알려진 헤더만 분기함(`tools/teul-cli/src/lang/parser.rs:488-539`, `:1835-1844`). 다만 SSOT의 HeadSpec 전체 표와 1:1 테이블화/진단은 없고, 일부 colon 허용/비권장 경로가 남아 있음(`tools/teul-cli/src/lang/parser.rs:2191-2194`, `tools/teul-cli/src/canon.rs:4155-4159`). |
| 19 | [V18-05B] 순회/반복 머릿말씨 | 1635-1686 | `반복`, `동안`, `~에 대해` 결정적 루프/스코프 | 구현됨 | 파서가 `반복`, `동안`, foreach를 분기함(`tools/teul-cli/src/lang/parser.rs:513-529`, `:2903-3078`). 런타임이 repeat/while/foreach를 실행함(`tools/teul-cli/src/runtime/eval.rs:1048-1149`). |
| 20 | 기호 규칙 | 1687-1720 | 괄호/대괄호/중괄호/화살표/점 등의 기능 고정 | 부분구현 | 토큰은 상당수 정의됨(`tools/teul-cli/src/lang/token.rs:61-77`)이고 lexer가 `<-`, `.`, `~~>`를 읽음(`tools/teul-cli/src/lang/lexer.rs:308-340`, `:688-691`). 다만 `->` 우향 대입은 없음(아래 #25), `< >` 타입 파라미터 등 표 전체는 제품 집행 근거가 없음. |
| 21 | 시스템 예약 자음 식별자 | 1721-1800 | 단자음 식별자를 사용자 이름으로 금지 | 미구현 | 제품 lang/runtime 검색에서 자음 예약 검사를 찾지 못함: `rg -n 'Jamo|jamo|자음|단자음|ㄱ|ㄴ|ㄷ' tools/teul-cli/src/lang tools/teul-cli/src/runtime` = NO_MATCH. |
| 22 | [CHAEVI-MAEGIM-01] `매김` | 1801-1864 | `채비` 항목 제어 메타로만 허용, 그룹 값 필수, 구조 보존 | 구현됨 | `채비/성질` 블록만 파싱함(`tools/teul-cli/src/lang/parser.rs:1835-1844`, `:2178-2291`), `매김`은 그룹 값 뒤에서만 허용하고 아니면 오류(`:2348-2366`), nested field 제한도 있음(`:2438-2456`). canon은 `maegim_control_json`을 출력함(`tools/teul-cli/src/canon.rs:4135-4170`, `:6085-6107`). |
| 23 | [EQUAL-DEF-01] `=` 정의 전용 | 1865-1887 | `=`는 정의/붙박이 선언 전용, 일반 대입 금지 | 구현됨 | 일반 문장 payload 뒤 `=`는 `CompatEqualDisabled`로 거부됨(`tools/teul-cli/src/lang/parser.rs:669-686`). 씨앗 정의는 `=`를 요구함(`tools/teul-cli/src/lang/parser.rs:3917-3925`), `채비` item의 `=`는 `DeclKind::Butbak`으로 처리됨(`:2260-2269`). |
| 24 | [ASSIGN-ARROW-01] `<-` 대입 전용 | 1888-1907 | `<-`만 상태/변수 대입으로 사용 | 구현됨 | lexer가 `<-`를 `TokenKind::Arrow`로 읽음(`tools/teul-cli/src/lang/lexer.rs:308-315`), parser가 `Arrow`를 assignment로 파싱함(`tools/teul-cli/src/lang/parser.rs:689-703`), runtime이 assignment에서 상태를 갱신함(`tools/teul-cli/src/runtime/eval.rs:950-955`). |
| 25 | [ASSIGN-ARROW-RIGHT-01] `->` 우향 대입 | 1908-1940 | `식 -> 자리.`를 `<-` 정본으로 정규화 | 미구현 | lexer에는 `<-`만 `Arrow`로 읽는 분기(`tools/teul-cli/src/lang/lexer.rs:308-315`)가 있고 `->`/`RightArrow` 토큰은 없음: `rg -n -- '->|RightArrow' tools/teul-cli/src/lang/lexer.rs tools/teul-cli/src/lang/token.rs tools/teul-cli/src/lang/parser.rs`에서 우향 대입 구현 없음. |
| 26 | [SIGNAL-ARROW-RESERVE-01] `~~>` | 1941-2000 | `~~>`는 알림 송신 전용 토큰/문장 | 구현됨 | lexer/token에 `SignalArrow`가 있음(`tools/teul-cli/src/lang/token.rs:75`, `tools/teul-cli/src/lang/lexer.rs:688-691`), parser가 send 문장을 만듦(`tools/teul-cli/src/lang/parser.rs:613-628`), runtime이 `dispatch_signal_send`를 실행함(`tools/teul-cli/src/runtime/eval.rs:6692-6728`). |
| 27 | [IMJA-DECL-01] `임자`와 `제` | 2001-2028 | `임자` seed와 `제` 자기참조 | 구현됨 | seed kind는 named kind를 허용함(`tools/teul-cli/src/lang/ast.rs:16-21`, `tools/teul-cli/src/lang/parser.rs:3869-3894`). runtime은 `임자` seed를 식별하고(`tools/teul-cli/src/runtime/eval.rs:6714-6718`) `제`를 현재 entity 경로로 해석함(`:1994-2002`, `:6814-6818`). |
| 28 | [ALRIM-DECL-01] `알림씨` | 2029-2044 | `알림씨` payload 구조 정의 | 구현됨 | `알림씨`는 named seed kind로 파싱 가능함(`tools/teul-cli/src/lang/parser.rs:3869-3894`). runtime은 송신 payload가 `알림씨` seed call인지 확인함(`tools/teul-cli/src/runtime/eval.rs:6769-6805`). |
| 29 | [ALRIM-RECV-HOOK-01] `...받으면` | 2045-2101 | `임자` 내부 수신 훅, typed/generic/binding/condition 및 결정적 우선순위 | 구현됨 | parser/canon은 receive outside imja를 오류로 둠(`tools/teul-cli/src/lang/parser.rs:85`, `:174`, `tools/teul-cli/src/canon.rs:1691-1730`). runtime은 rank 0~3 순서로 handler를 평가함(`tools/teul-cli/src/runtime/eval.rs:6837-6904`)하고 binding/condition을 처리함(`:6906-6953`). |
| 30 | 문장 경계 | 2102-2110 | 줄바꿈 또는 점이 문장 경계, 세미콜론 없음 | 구현됨 | lexer가 newline과 dot을 별도 토큰으로 냄(`tools/teul-cli/src/lang/lexer.rs:184`, `:320-340`), parser `consume_terminator`가 dot/newline/eof만 허용함(`tools/teul-cli/src/lang/parser.rs:6076-6088`). |
| 31 | 점의 2중 역할 | 2111-2132 | 공백 없는 dot+ident는 접근, 아니면 종결 | 구현됨 | call/path 파서가 dot 뒤 ident를 segment로 읽음(`tools/teul-cli/src/lang/parser.rs:5588-5612`, `:6043-6063`), 문장 종결은 `consume_terminator`에서 처리함(`:6076-6088`). |
| 32 | [MAP-DOT-READ-01] 짝맞춤 dot read | 2133-2380 | `m.key` map/dict 읽기, missing key fatal | 구현됨 | map 값은 `MapValue`로 존재함(`tools/teul-cli/src/core/value.rs:226-252`). runtime member access가 `Value::Map`이면 `map_get_required`를 호출하고 missing key를 오류로 냄(`tools/teul-cli/src/runtime/eval.rs:2054-2104`). |
| 33 | 수치 정책 det_tier + trace_tier | 2381-2408 | 결정성 tier와 trace tier를 분리하고 정책화 | 부분구현 | `tool/src/project_meta.rs`에 `DetTier::{Strict,Fast,Ultra}`와 `trace_tier` 필드가 있음(`tool/src/project_meta.rs:34-45`, `:150-168`). teul-cli run에는 `TraceTier`와 frame payload 분기가 있음(`tools/teul-cli/src/cli/run.rs:1704-1719`, `:3548-3577`). 단, SSOT의 D-STRICT/D-FAST/D-ULTRA 산술 의미 전체가 제품 실행에 통합되었다고 볼 근거는 부족. |
| 34 | 실행 중 전환 금지 | 2409-2421 | det/trace tier는 realm 시작 후 변경 금지 | 부분구현 | project meta parse 시 정책을 만들고 AGE0 제약을 검사함(`tool/src/project_meta.rs:150-159`, `:202-215`), run trace tier는 CLI 옵션으로 frame 기록에 쓰임(`tools/teul-cli/src/cli/run.rs:3548-3577`). 런타임 중 변경 API/금지 진단을 직접 집행하는 제품 근거는 확인하지 못함. |
| 35 | 리터럴과 승격 규칙 | 2422-2458 | 수치 suffix와 승격 규칙 | 부분구현 | fixed64 파서/표시는 있음(`tools/teul-cli/src/core/fixed64.rs:23-44`, `:132-143`)이고 core 쪽 `from_i32`도 있음(`core/src/fixed64.rs:39-41`). 하지만 `i32/u64/f32` suffix 승격 표 전반의 제품 파서/타입 승격 구현은 확인되지 않음: `rg -n 'i32|u64|f32|suffix' tools/teul-cli/src/lang tools/teul-cli/src/runtime`는 범용 suffix 정책 근거를 주지 않음. |
| 36 | 구조/주체/관계 경계 | 2459-2465 | `이름씨`/`임자`/`맞물림씨` 역할 분리 | 부분구현 | `임자` named seed와 runtime entity 처리는 구현됨(#27). 그러나 제품 lang/runtime에서 `이름씨/맞물림씨` type surface는 없음: `rg -n '고름씨|묶음씨|이름씨|맞물림씨' tools/teul-cli/src/lang tools/teul-cli/src/runtime` = NO_MATCH. |
| 37 | 이름씨 본문 초기화 규칙 | 2466-2541 | `이름씨` 본문은 생성 시 1회 실행되어 field 초기화 | 미구현 | 제품 lang/runtime 검색에서 `이름씨` surface/constructor init 경로를 찾지 못함: `rg -n '이름씨' tools/teul-cli/src/lang tools/teul-cli/src/runtime` = NO_MATCH. |
| 38 | 자기 참조 `나` | 2542-2580 | `이름씨` 본문 안에서 `나`가 현재 객체 | 미구현 | 구현된 자기참조는 `제`뿐임(`tools/teul-cli/src/runtime/eval.rs:1994-2002`, `:6814-6818`). `나` self 경로 구현은 확인되지 않음: `rg -n '\"나\"|\\b나\\b' tools/teul-cli/src/lang tools/teul-cli/src/runtime`에서 type self 구현 근거 없음. |
| 39 | 묶음씨(레코드/구조체) | 2581-2640 | named record/struct 타입과 필드 구조 | 부분구현 | runtime 값 모델에는 `PackValue`와 `Value::Pack`이 있고(`tools/teul-cli/src/core/value.rs:157-188`, `:255-270`), parser도 pack literal AST를 다룸(`tools/teul-cli/src/lang/parser.rs:2427`). 그러나 `묶음씨` named type 선언 표면은 없음: `rg -n '묶음씨' tools/teul-cli/src/lang tools/teul-cli/src/runtime` = NO_MATCH. |
| 40 | 튜플 정규화 | 2641-2648 | `(A)`는 scalar, trailing comma는 tuple | 부분구현 | 괄호 grouping과 call/list/pack 구문은 제품 파서에 있으나, 별도 tuple value/type은 `Value` enum에 없음(`tools/teul-cli/src/core/value.rs:255-270`). trailing-comma tuple 정규화 구현 근거도 확인되지 않음. |
| 41 | 고름씨 정의 | 2649-2653 | closed tag union 타입 정의 | 미구현 | `#atom` 표현식은 있으나(`tools/teul-cli/src/lang/ast.rs:298-301`, `tools/teul-cli/src/lang/parser.rs:5045-5048`) 제품 lang/runtime에 `고름씨` 타입 surface가 없음: `rg -n '고름씨' tools/teul-cli/src/lang tools/teul-cli/src/runtime` = NO_MATCH. |
| 42 | 고름씨 정의 문법 | 2654-2667 | `행동:고름씨 = { ... }` 정의 문법 | 미구현 | `SeedKind::from_name`은 `셈씨/움직씨`만 정본 kind로 인식함(`tools/teul-cli/src/lang/ast.rs:23-30`), `고름씨` 정의 파서 없음: `rg -n '고름씨' tools/teul-cli/src/lang tools/teul-cli/src/runtime` = NO_MATCH. |
| 43 | 고름씨 생성 문법 | 2668-2682 | `#대기`, `#공격(...)` 등 tag 생성 | 부분구현 | bare atom은 lexer/parser/AST에 있음(`tools/teul-cli/src/lang/token.rs:10`, `tools/teul-cli/src/lang/lexer.rs:473-523`, `tools/teul-cli/src/lang/parser.rs:5045-5048`). 그러나 closed union constructor payload `#공격(...)`와 type 연계는 확인되지 않음. |
| 44 | 고름씨 전역 유일성 | 2683-2689 | tag 이름은 전역 유일해야 함 | 부분구현 | parser에는 context tag 중복 진단이 있음(`tools/teul-cli/src/lang/parser.rs:309-318`, `:5045-5048`). 다만 이는 현재 parser context tag scope 중복이고, `고름씨` 정의 기반 전역 constructor registry는 확인되지 않음. |

## 요약

| 판정 | 건수 |
|---|---:|
| 구현됨 | 14 |
| 부분구현 | 18 |
| 미구현 | 12 |
| 확인불가 | 0 |
| 합계 | 44 |

## 심각도가 높은 미구현 후보

- `CALL-TAIL-RESOLVE-01`과 `X/X하` 충돌 금지는 Q20 실측처럼 실제 제품 경로에 오류 코드/후보 수집이 없다.
- TERM-MAP 단일 소스와 기계 추출은 문서상 MUST지만 제품 lint/canon의 근거 테이블로 착지하지 않았다.
- `해보고`/`그것` 상태 대조 표면은 제품 lang/runtime에서 검색되지 않는다.
- 단자음 예약 식별자 금지는 제품 lang/runtime 검색상 집행 근거가 없다.
- `->` 우향 대입 sugar는 lexer/token/parser에 대응 토큰이 없다.
- `이름씨`/`나`/`고름씨` 타입군은 값 모델 일부(`Pack`, `Atom`)만 있고 선언/타입 의미론은 미착륙이다.
