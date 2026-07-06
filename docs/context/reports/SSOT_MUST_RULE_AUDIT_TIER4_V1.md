# SSOT_MUST_RULE_AUDIT_TIER4_V1

> 작성: Codex (2026-07-06)
> 범위: `docs/ssot/ssot/SSOT_LANG_v24.12.9.md` 5406~8151행, MUST 섹션 45개
> 성격: 진단 전용. `docs/ssot/**`, 코드, pack, golden 수정 없음.

## 방법

- 대상 추출: `rg -n "^#{2,4} .*(MUST)" docs/ssot/ssot/SSOT_LANG_v24.12.9.md`
- Tier4 기준: 헤더 시작 행 `>= 5406`
- 제품 근거 경로: `tools/teul-cli/src/**`, `core/src/**`, 필요시 `tool/src/**`, `pack/**`
- 판정 기준: `구현됨` / `부분구현` / `미구현` / `확인불가`

## 판정표

| # | 규칙 ID/제목 | 줄 범위 | 핵심 주장 | 판정 | 근거(file:line 또는 검색식) |
|---:|---|---:|---|---|---|
| 1 | §M2.2.2 평가 규칙 | 5406-5946 | 수식 평가는 주입된 변수/단위로 결정적으로 수행되고 오류는 표준화되어야 함 | 부분구현 | `FormulaEval`/`FormulaFill` AST와 runtime 평가가 있음(`tools/teul-cli/src/lang/parser.rs:5505-5526`, `:5662-5713`, `tools/teul-cli/src/runtime/eval.rs:5875-5921`). `풀기` pack 경로는 누락/여분/없음 일부를 막음(`tools/teul-cli/src/runtime/eval.rs:3880-3914`). 다만 MathIR v1/중복 key fatal/전체 오류 표준은 닫히지 않음. |
| 2 | §P1 파이프(해서) | 5947-5954 | `해서` 파이프는 흐름값/주입/모호성 금지 규칙을 갖는 정식 표면 | 미구현 | 제품 parser/runtime에서 `해서` 파이프 AST/평가 경로를 찾지 못함. `Pipe` token은 `|`이며 씨앗 literal params에 쓰임(`tools/teul-cli/src/lang/token.rs:76`, `tools/teul-cli/src/lang/parser.rs:5150`, `:5184`). |
| 3 | §P1.2 호출식만 허용 | 5955-5959 | 파이프 stage는 호출식만 허용 | 미구현 | #2와 동일하게 `해서` 파이프 표면이 없으므로 호출식-only 검사를 집행할 제품 경로가 없음. `rg -n '해서|PIPE-CALL|Pipe' tools/teul-cli/src/lang tools/teul-cli/src/runtime` 결과는 `Pipe` token/씨앗 literal 근거뿐임. |
| 4 | §P1.3 암묵 주입 금지 | 5960-5981 | 파이프 흐름값은 암묵 주입하지 않고 명시 pin만 허용 | 미구현 | #2와 동일. 제품 `ArgBinding`은 named/pin/josa 인자를 다루지만(`tools/teul-cli/src/lang/parser.rs:5255-5350`) `해서` 파이프 주입 금지 알고리즘은 없음. |
| 5 | §P1.6 `{ ... }해서` 파이프 꼬리 | 5982-5990 | block tail `{ ... }해서`를 파이프 stage로 다룸 | 미구현 | block tail `해서` parser/runtime 분기 없음: `rg -n '해서|Pipe' tools/teul-cli/src/lang/parser.rs tools/teul-cli/src/runtime/eval.rs`는 `|` seed literal만 보임(`tools/teul-cli/src/lang/parser.rs:5150`, `:5184`). |
| 6 | §P1.7 흐름값 주입 알고리즘 | 5991-6068 | 파이프 흐름값을 명시 pin/위치 규칙으로 주입하고 모호성은 오류 | 미구현 | `해서` 파이프가 미구현이라 흐름값 주입 알고리즘도 제품 경로에 없음(#2). 기존 call arg parser는 일반 호출 인자만 처리함(`tools/teul-cli/src/lang/parser.rs:5255-5350`). |
| 7 | 2. 핵심 함수 | 6069-6082 | 보여주기/말하기/생각하기 등 핵심 함수 표준 | 부분구현 | `보여주기` statement/runtime 출력은 구현됨(`tools/teul-cli/src/lang/lexer.rs:655`, `tools/teul-cli/src/runtime/eval.rs:958-960`). 그러나 `말하기`/`생각하기` 표준 함수와 전체 핵심 함수 표는 제품 runtime builtin 목록에서 확인되지 않음. |
| 8 | 2.2 알림(alrim) | 6083-6097 | 알림 생성/전달 표준 함수와 결정적 처리 | 부분구현 | `~~>` 송신 문장, pending signal queue, 수신 hook rank 처리는 있음(`tools/teul-cli/src/lang/parser.rs:613-628`, `tools/teul-cli/src/runtime/eval.rs:6692-6745`, `:6837-6904`). 다만 표준 함수 `알리기` 자체는 제품 builtin에서 확인되지 않음. |
| 9 | 2.3 쓸감(Asset) | 6098-6135 | 쓸감/자원 값과 로딩/조회 표준 | 부분구현 | `자원` builtin과 core resource handle은 있음(`tools/teul-cli/src/runtime/eval.rs:4966`, `core/src/resource.rs:4-26`). `쓸감` alias/`@"..."` sugar는 `lang/src` 쪽 근거는 있으나 제품 teul-cli parser에는 닫히지 않음(`lang/src/parser.rs:3771-3880`, `lang/src/stdlib.rs:67`). |
| 10 | 3. DetMath | 6136-6241 | 결정적 수학 함수 전체 표준 | 부분구현 | teul-cli는 Fixed64 기반 `sqrt`와 CORDIC `sin/cos`를 사용함(`tools/teul-cli/src/runtime/eval.rs:2746-2772`, `tools/teul-cli/src/runtime/detmath.rs:44-54`). tan/exp/log/pow 등 전체 표준과 LUT/차림새 정책은 제품 경로에서 닫히지 않음. |
| 11 | 4. 단위 시스템 | 6242-6294 | 숫자@단위, 차원 대수, 환산, 출력 정규화 | 구현됨 | 단위 postfix parser와 unit expression, 차원 계산, 출력 환산 경로가 있음(`tools/teul-cli/src/lang/parser.rs:5856-5926`, `tools/teul-cli/src/core/unit.rs:15-40`, `:189-215`, `tools/teul-cli/src/runtime/template.rs:589-618`). |
| 12 | 4.3.2 환산 실패 규칙 | 6295-6302 | 환산 불가/브리지 없음/차원 불일치 오류를 표준화 | 부분구현 | unknown unit/dimension mismatch 오류 경로는 있음(`tools/teul-cli/src/core/unit.rs:189-215`, `tools/teul-cli/src/runtime/formula.rs:211-247`). 다만 SSOT의 모든 실패 코드/브리지 정책 이름이 제품 오류 체계로 완전히 고정된 근거는 확인되지 않음. |
| 13 | 4.3.3 온도 단위 | 6303-6332 | `@K/@C/@F`를 Kelvin으로 정규화하고 환산 | 구현됨 | 온도 literal normalization과 회귀 테스트가 있음(`tools/teul-cli/src/runtime/eval.rs:2117-2130`, `:17043-17085`). |
| 14 | 5. 말결(Nuance) 매핑 | 6333-6358 | 말결 token을 표준 weight로 매핑 | 부분구현 | `말결값` builtin은 `매우/꽤/조금/약간/거의`를 수치화함(`tools/teul-cli/src/runtime/eval.rs:5021-5024`, `:15403-15412`). SSOT의 `$...` 표면 token과 전체 표준표는 미착륙. |
| 15 | 5.4 규칙 | 6359-6366 | 말결 없음/중복/부정 결합 규칙 | 미구현 | `$` 말결 token sequence, 중복 처리, 부정 결합 parser/runtime 경로 없음: `rg -n '말결|부정' tools/teul-cli/src/lang tools/teul-cli/src/runtime`는 `말결값` 일부 외 결합 규칙을 보이지 않음. |
| 16 | 6. 표준 컴포넌트 | 6367-6406 | 감정/기억/관계 등 표준 컴포넌트 스키마 | 부분구현 | 일부 helper/builtin은 존재함(`tools/teul-cli/src/runtime/eval.rs:5111`, `core/src/platform.rs:23-69`). 그러나 SSOT 표준 컴포넌트 전체 스키마와 타입 검사는 제품 표면에서 닫힌 근거가 부족함. |
| 17 | 7. 입력 함수 | 6407-6427 | 키/마우스/질문 등 입력 함수 표준 | 부분구현 | `눌렸나`, `막눌렸나`는 구현됨(`tools/teul-cli/src/runtime/eval.rs:4195-4202`, builtin 목록 `:5730`, `:5754`). `물어보기`, 마우스 좌표/버튼 등 전체 표준 입력 함수는 제품 builtin에서 확인되지 않음. |
| 18 | 8. 결정적 난수 | 6428-6437 | 시드 기반 결정적 난수 함수 | 구현됨 | `무작위`, `무작위정수`, `무작위선택`, `무작위가방.*` builtin과 결정적 상태 갱신이 있음(`tools/teul-cli/src/runtime/eval.rs:4292-4411`, `:5575-5584`, `tool/src/ddn_runtime.rs:6821-6925`). |
| 19 | 8.2 결정론 규칙 | 6438-6445 | 같은 seed/소비 순서에서 같은 난수 결과 | 구현됨 | runtime RNG 상태가 evaluator에 포함되고 함수 호출로 소비됨(`tools/teul-cli/src/runtime/eval.rs:92-93`, `:4292-4411`, `:5575-5584`). 별도 비결정 source 사용 근거는 확인되지 않음. |
| 20 | 9. 시간 함수 | 6446-6457 | 마디번호/델타시간/게임시간 계열 시간 함수 | 부분구현 | evaluator는 `current_madi`를 추적하고 tick마다 설정함(`tools/teul-cli/src/runtime/eval.rs:93`, `:735-737`). `마디번호`, `델타시간`, `게임시간` 표준 builtin 이름은 제품 runtime 목록에서 확인되지 않음. |
| 21 | 10. 엔티티 관리 | 6458-6468 | 임자 생성/삭제/조회 표준 함수 | 부분구현 | core ECS world와 entity/component 구조는 있음(`core/src/platform.rs:23-69`, `:1078-1193`). DDN 표면의 표준 엔티티 관리 함수 전체는 teul-cli builtin으로 닫힌 근거가 부족함. |
| 22 | 11. 컴포넌트 접근 | 6469-6479 | 임자에 컴포넌트를 붙이고 떼고 읽는 표준 함수 | 부분구현 | core component/resource API와 signal 이벤트는 존재함(`core/src/platform.rs:1078-1193`, `:1356-1411`). 제품 DDN 표면의 `새로/지우기/붙이기/떼기/가져오기` 전체 표준은 확인되지 않음. |
| 23 | 12. 차림 연산 | 6480-6514 | list/array 표준 연산 | 구현됨 | `길이`, `첫번째`, `마지막`, `추가`, `제거`, `정렬`, `거르기`, `변환`, `합치기`가 runtime builtin으로 구현됨(`tools/teul-cli/src/runtime/eval.rs:3244`, `:3470-3512`, `:3618`, builtin 목록 `:5696-5860`). |
| 24 | 13. 문자열 연산 | 6515-6540 | 문자열 자르기/붙이기/검사/숫자 변환 | 구현됨 | `자르기`, `붙이기`, `포함하나`, `시작하나`, `끝나나`, `숫자로`, `글로` 구현 및 builtin 목록이 있음(`tools/teul-cli/src/runtime/eval.rs:3648-3717`, `:5775-5855`). |
| 25 | 15. 모듬 구조 | 6541-6594 | 모듬/가지 구조와 공개/가져오기 | 부분구현 | gaji/registry CLI와 package scaffold는 존재하지만 완전한 install/publish/discover landed는 앞선 감사에서 부분 착지로 확인됨(`tools/teul-cli/src/cli/gaji.rs`, `docs/context/reports/LOCAL_REGISTRY_LANDING_AUDIT_V1.md`, `docs/context/reports/GAJI_SCAFFOLD_SURVEY_V1.md`). DDN 표면 `쓰임/드러냄` 전체는 미완. |
| 26 | 16.2 결정성 정책 연동 | 6595-6609 | 모듬/가지가 det/open/effect 정책과 연결 | 부분구현 | 프로젝트/도구 쪽 det tier, open/replay 구조는 존재함(`tools/teul-cli/src/runtime/open.rs:695-700`, `tool/src/project_meta.rs`). 그러나 모듬 manifest와 제품 검사의 전면 연동은 닫힌 근거가 부족함. |
| 27 | 15.1 이름공간(모듬/가지) | 6610-6637 | 모듬/가지 이름공간과 import/export 해석 | 부분구현 | dot access/member path와 일부 legacy import/canon 경로는 있음(`tools/teul-cli/src/lang/parser.rs:4888-4914`, `:5598-5612`, `:6199`). 모듬/가지 이름공간 resolver 전체는 제품 경로에서 확인되지 않음. |
| 28 | 17. 산술 연산 의미론 | 6638-6669 | 산술 overflow/div0/unit 오류의 결정적 의미 | 부분구현 | `Fixed64`는 saturating add/sub/mul과 div fault signal을 갖고(`core/src/fixed64.rs:42-94`), formula는 Fixed64/UnitDim을 사용함(`tools/teul-cli/src/runtime/formula.rs:190-247`). 런타임 전체가 예외 대신 고장값으로 통일된 것은 아님. |
| 29 | AI.1 끝내 | 6670-6690 | AI/Gym episode 종료 함수 `끝내` | 미구현 | `끝내` DDN stdlib/runtime builtin을 찾지 못함: `rg -n '끝내' tools/teul-cli/src/runtime tools/teul-cli/src/lang tools/teul-cli/src/cli/nurigym.rs`는 표준 함수 구현 근거 없음. |
| 30 | AI.2 지금관찰 | 6691-6711 | 현재 관찰 반환 함수, 별칭 `눈떠` | 부분구현 | NuriGym CLI에는 observation/action pipeline이 있음(`tools/teul-cli/src/cli/nurigym.rs:126-130`, `:1720-1727`). 그러나 DDN stdlib `지금관찰`/`눈떠` 함수는 제품 runtime builtin에서 확인되지 않음. |
| 31 | AI.3 보상 | 6712-6736 | reward 함수/보상 값 표준 | 부분구현 | reward CLI/evaluator 모듈은 있음(`tools/teul-cli/src/cli/reward.rs:449`, `tools/teul-cli/src/cli/nurigym.rs:1951-2118`). DDN 표면 `보상` stdlib 함수는 runtime builtin에서 확인되지 않음. |
| 32 | 뉘.1 말결 토큰 정의 | 6737-6768 | 말결 token 표면과 weight 정의 | 부분구현 | `말결값`은 tag 문자열을 weight로 바꾸지만(`tools/teul-cli/src/runtime/eval.rs:5021-5024`, `:15403-15412`), `$` 말결 token lexer/parser는 제품 lang 경로에서 확인되지 않음. |
| 33 | 뉘.3 부정과의 조합 | 6769-6796 | 부정 표현과 말결 weight 조합 | 미구현 | `말결값` 외에 부정 결합 규칙 없음: `rg -n '말결|부정|아님' tools/teul-cli/src/lang tools/teul-cli/src/runtime`는 `$` token/부정 weight 결합 경로를 보이지 않음. |
| 34 | DetMath.2 입력/출력 | 6797-6920 | DetMath 입력 domain/출력 정밀도/오류 표준 | 부분구현 | sin/cos/sqrt는 Fixed64 계열로 구현됨(`tools/teul-cli/src/runtime/eval.rs:2746-2772`, `tools/teul-cli/src/runtime/detmath.rs:44-54`). 전체 함수 domain/정밀도/오류 코드 표준은 제품에 완전히 닫히지 않음. |
| 35 | 쓸감.1 쓸감 리터럴 | 6921-6953 | `쓸감` 리터럴과 asset resource 정본 | 부분구현 | core resource와 `자원` builtin은 있음(`core/src/resource.rs:4-26`, `tools/teul-cli/src/runtime/eval.rs:4966`). `@"..."`/`쓸감` literal은 `lang/src`에는 있으나 teul-cli 제품 parser에는 확인되지 않음(`lang/src/parser.rs:3771-3880`). |
| 36 | 추가 §G. 확장 구문 프레임 | 6954-6970 | 확장 구문은 registry/frame으로 닫힌 형태여야 함 | 부분구현 | frontdoor/canon/preprocess guard와 registry 일부는 있음(`tools/teul-cli/src/canon.rs:3951-4109`, `tool/src/gate0_registry.rs:9-60`). SSOT의 확장 구문 프레임 전체를 하나의 registry로 집행하는 제품 근거는 부족함. |
| 37 | NAMING-LINT-01 | 6971-6975 | 조사 단독 금지와 조사 접미사 자동 분리 | 부분구현 | 명시 `~` 조사 tokenization은 있음(`tools/teul-cli/src/lang/lexer.rs:606-641`)이고 call arg josa suffix도 있음(`tools/teul-cli/src/lang/parser.rs:5291-5328`). bare 조사 단독 금지와 자동 분리/canon 삽입은 닫히지 않음. |
| 38 | NAMING-LINT 1) 금지(예약) | 6976-6980 | 조사 단독 identifier는 예약/금지 | 미구현 | bare `을/를/이/가` 등 조사 단독 identifier 금지 진단 구현을 찾지 못함: `rg -n '조사 단독|JOSA.*reserved|E_.*JOSA' tools/teul-cli/src/lang tools/teul-cli/src/cli`는 해당 진단 근거 없음. |
| 39 | NAMING-LINT 2) 해석 우선순위 | 6981-6993 | 조사 접미사 해석 우선순위를 결정적으로 적용 | 부분구현 | 명시 `~`와 `:pin`, `@unit` 접미 체인은 구현됨(`tools/teul-cli/src/lang/lexer.rs:606-641`, `tools/teul-cli/src/lang/parser.rs:5255-5350`, `:5856-5926`). `~` 없는 조사 접미 자동 split 우선순위는 미착륙. |
| 40 | NAMING-LINT 3) 정본화 | 6994-7056 | 자동 분리된 조사 접미사는 canon에서 `~`로 표시 | 미구현 | 명시 `~` 처리 근거는 있으나 자동 split 후 canon 삽입 경로를 찾지 못함: `rg -n 'JOSA-SPLIT|조사 접미|자동 분리|canon.*~' tools/teul-cli/src`는 해당 정본화 근거 없음. |
| 41 | SEMSI-RESULT-01 | 7057-7091 | 셈씨 결과칸/결과 이름 대입 표준 | 미구현 | `돌려주기`/`Return`은 씨앗 안 반환으로 구현됨(`tools/teul-cli/src/lang/ast.rs:141`, `tools/teul-cli/src/runtime/eval.rs:982-984`). 하지만 셈씨 결과칸/결과 이름 대입 전용 parser/runtime 경로는 확인되지 않음: `rg -n '결과칸|SEMSI|결과 이름' tools/teul-cli/src` = NO_MATCH. |
| 42 | FILE-META-01 | 7092-7400 | 파일 선두 메타는 `설정 {}`만 허용, legacy header 금지 | 구현됨 | frontdoor/canon guard가 legacy header를 `E_FRONTDOOR_LEGACY_HEADER_FORBIDDEN`으로 거부하고 file-leading `설정 {}`를 허용하는 테스트가 있음(`tools/teul-cli/src/canon.rs:4032`, `:7899-7971`, `tools/teul-cli/src/cli/frontdoor_input.rs:33-51`, `tools/teul-cli/src/cli/run.rs:136-140`). |
| 43 | SPACE2D-DRAWLIST-01 | 7401-7480 | space2d drawlist primitive와 detbin/hash | 부분구현 | drawlist model/build/detbin/hash는 구현됨(`tools/teul-cli/src/core/bogae.rs:91`, `:454-505`, `:1547`, `:1727`, `:1921`)이고 WASM view meta도 space2d를 파생함(`tool/src/wasm_api.rs:888-891`, `:2573-2611`). primitive/스키마 전체 검증은 부분 착지. |
| 44 | GRAPH-KIND-01 | 7481-7495 | graph 보개 종류 `graph_kind` v0 표준 | 부분구현 | graph metadata를 쓰는 pack과 overlay 검증 pack은 있음(`pack/age2_math_calculus_v1/input.ddn:2-13`, `pack/seamgrim_overlay_param_compare_v0/golden.jsonl:5-21`). 제품 graph_kind enum 표면 검증/스키마 집행은 완전 착지로 보기 어려움. |
| 45 | GRAPH-AXIS-META-01 | 7496-8151 | graph 축 meta `x_kind/unit/label`, `y_kind/unit/label` 표준 | 부분구현 | overlay compare pack은 axis kind/unit mismatch를 검사함(`pack/seamgrim_overlay_param_compare_v0/README.md:35-76`, `pack/seamgrim_overlay_param_compare_v0/golden.jsonl:43`). DDN graph 표면에서 축 메타 schema를 일괄 강제하는 제품 경로는 부분 착지. |

## 요약

| 판정 | 건수 |
|---|---:|
| 구현됨 | 6 |
| 부분구현 | 28 |
| 미구현 | 11 |
| 확인불가 | 0 |
| 합계 | 45 |

## 심각도가 높은 미구현 후보

- `해서` 파이프 계열 §P1 전체는 제품 parser/runtime에서 표면 자체가 없다.
- 말결 `$...` token과 부정/중복 결합 규칙은 `말결값` builtin 일부와 분리되어 미착륙이다.
- AI Gym 표면 함수 `끝내`, `지금관찰`/`눈떠`, `보상`은 CLI backend와 달리 DDN stdlib 함수로 닫히지 않았다.
- 조사 단독 금지/자동 분리/canon `~` 삽입은 명시 `~` 처리만 있고 자동 lint 계층이 없다.
- 셈씨 결과칸 `SEMSI-RESULT-01`은 `돌려주기` 구현과 별개로 제품 경로가 확인되지 않는다.
