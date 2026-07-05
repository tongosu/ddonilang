# BRIEF: 사용자 지정 씨앗(움직씨/셈씨) 런타임 호출 배선 진단

> 작성: Claude (2026-07-06) / 실행: Codex / 리뷰: Claude
> 성격: **진단 전용 — 코드 수정 금지.** 사실 확인 후 보고만 한다.
> 배경: `pack/lang_kernel_v1_conformance/`의 그룹 1(그린 예상) 3케이스가 전부 `E_RUNTIME_UNDEFINED`로 실패. 실제 저장소 팩을 검색한 결과, `:움직씨`로 선언된 **사용자 지정 이름**(매틱/시작 등 예약 이름이 아닌)을 별도 문장에서 이름으로 호출하는 실제 사례를 찾지 못함(예: `pack/age1_container_resource`, `pack/age2_open_input_record_replay_v1` 등은 전부 `매틱:움직씨`만 사용). SSOT §11.3.2/§V18-00C의 선언·호출 표면 예시가 파서 레벨에서는 유효해도 런타임 실행 경로(`eval.rs` 등)에 실제 배선돼 있는지 확인되지 않았다.

## 질문 (사실만 답할 것 — 추측·수정 금지)

1. `tools/teul-cli/src/runtime/eval.rs` (또는 관련 실행 경로)에서, `이름:움직씨 = {...}` 형태로 선언된 **비예약 이름**의 씨앗이 실행 시점에 심볼 테이블/레지스트리에 등록되는가? 등록된다면 어디서(파일:행)?
2. 프로그램 최상위(top-level, 어떤 이벤트 훅 블록 안도 아닌 위치)에 있는 문장 `이름꼬리.` (예: `돕기.`)가 실제로 실행될 때, 그 이름을 앞서 선언된 씨앗과 연결하는 코드 경로가 존재하는가? 존재한다면 파일:행. 존재하지 않는다면 그 사실을 그대로 보고할 것.
3. `lang/src/parser.rs:5736 resolve_call_tail_candidates`(호출 꼬리 동치 해석)가 실제로 호출되는 지점(콜사이트) 전부를 나열하라 — 이게 파이프(`~기 해서`) 전용인지, 일반 문장 호출에도 쓰이는지.
4. `pack/lang_kernel_v1_conformance/cases/stem_alias_dop_dou.ddn`을 실제로 다시 실행해 `E_RUNTIME_UNDEFINED` 오류의 정확한 발생 지점(스택/코드 위치)을 추적하라. 오류 메시지가 정확히 무엇을 "정의되지 않음"이라고 하는지(씨앗 이름 자체인지, 다른 것인지).
5. 저장소 전체에서 `:움직씨` 또는 `:셈씨`로 선언된 **비예약 이름**이 다른 위치에서 실제로 호출되는 진짜 사례가 있는지 재검색(`grep`) — 있다면 그 파일:행과 호출 방식을 제시.

## 수용 기준

- [ ] 5개 질문 전부에 파일:행 근거 또는 "찾지 못함" 명시로 답함 — 추측 문장 금지
- [ ] `stem_alias_dop_dou.ddn` 재실행 로그(오류 스택/메시지 원문) 첨부
- [ ] 코드 수정 없음, `git status --short` 깨끗
- [ ] 결론 요약 1문단: "비예약 이름의 씨앗을 최상위에서 이름으로 호출하는 것이 (A) 설계상 원래 불가능한 패턴인지 (B) 설계는 가능하나 미구현인지 (C) 다른 호출 방식(예: 이벤트/알림 경유)이 필요한 것인지" 중 어느 쪽인지, 근거와 함께.

## 금지 사항

커널 기능 구현 / 파서·런타임 코드 수정 / golden 갱신 / 이 브리프 범위 밖 조사. main 커밋 금지 — `codex/queue-20260706` 브랜치에 커밋 1개.

## 보고 형식

이 파일 하단 `## 실행 보고`.

## 실행 보고

작성: Codex (2026-07-06)

범위: 진단 전용. 파서/런타임/팩 코드는 수정하지 않았다. Q18 시작 시 `git status --short`는 `M docs/context/briefs/QUEUE_CODEX_20260706.md` 1건이었다(큐에 Q18/Q19가 추가된 상태). Q18 중 저장소 코드/팩 파일 변경은 없고, 이 브리프 실행 보고만 추가했다.

### 결론

판정: **B. 설계는 가능하나 `이름꼬리.` 최상위 bare 호출 표면이 제품 실행 파서에 미구현/미배선된 상태**.

근거:
- 사용자 씨앗 등록은 존재한다. `tools/teul-cli/src/runtime/eval.rs:642`에서 `Stmt::SeedDef`를 모으고, `tools/teul-cli/src/runtime/eval.rs:689`에서 `self.user_seeds`에 등록한다.
- `Expr::Call`로 들어온 호출은 사용자 씨앗으로 dispatch된다. `tools/teul-cli/src/runtime/eval.rs:2215`에서 정확 이름을 찾고, `tools/teul-cli/src/runtime/eval.rs:2220`부터 `하기/기/하고/고/하면/면/하면서/면서` 꼬리를 제거해 씨앗 stem을 찾는다.
- 하지만 `돕기.` 같은 최상위 bare 문장은 `tools/teul-cli/src/lang/parser.rs:5026`부터 식별자를 `parse_path()`로 읽고, `tools/teul-cli/src/lang/parser.rs:5042`에서 `Expr::Path`로 만든다. `tools/teul-cli/src/lang/parser.rs:5967`부터 비루트 경로 앞에 기본 루트 `살림`을 붙이므로 `돕기`는 `살림.돕기` 경로가 된다.
- bare 경로를 zero-arg call로 바꾸는 일반 배선은 없다. bare rewrite는 `tools/teul-cli/src/lang/parser.rs:863`~`864`에서 reset/lifecycle 전용이고, `tools/teul-cli/src/lang/parser.rs:910`~`954`의 허용 이름도 `마당다시/판다시/...` 및 `시작하기/넘어가기/불러오기`에 한정된다.
- 보조 재현에서 동일 씨앗을 `() 돕기.`로 호출하면 PASS했다. 따라서 런타임 씨앗 등록/dispatch 자체가 없어서가 아니라, 그룹1 케이스의 `돕기.` 표면이 호출식으로 파싱되지 않는 것이 직접 원인이다.

### 질문별 답

1. 비예약 `이름:움직씨 = {...}` 씨앗 등록 여부: **등록된다**.
   - `tools/teul-cli/src/runtime/eval.rs:97`: `Evaluator`가 `user_seeds: BTreeMap<String, UserSeed>`를 가진다.
   - `tools/teul-cli/src/runtime/eval.rs:642`: `run_with_ticks_internal`이 top-level `Stmt::SeedDef`를 `seed_defs`로 분리한다.
   - `tools/teul-cli/src/runtime/eval.rs:689`: 분리된 씨앗을 `self.user_seeds.insert(name.clone(), UserSeed { ... })`로 등록한다.
   - `tools/teul-cli/src/runtime/eval.rs:900` 및 `tools/teul-cli/src/runtime/eval.rs:907`: 실행 중 `Stmt::SeedDef`를 만나도 동일하게 `user_seeds`에 등록하는 경로가 있다.

2. top-level `이름꼬리.`가 앞선 씨앗과 연결되는 코드 경로: **`돕기.` 형태에는 없다**.
   - `tools/teul-cli/src/lang/parser.rs:861`~`868`: 일반 bare expression statement는 `parse_expr()` 결과를 statement로 감싼다.
   - `tools/teul-cli/src/lang/parser.rs:5026`~`5043`: 식별자 primary는 호출이 아니라 `Expr::Path`다.
   - `tools/teul-cli/src/lang/parser.rs:5937`~`5979`: `parse_path()`는 비루트 이름에 기본 루트 `살림`을 삽입한다.
   - `tools/teul-cli/src/runtime/eval.rs:2010`~`2027`: `Expr::Path`는 `eval_path()`로 평가되고, 상태에 없으면 `RuntimeError::Undefined`가 난다.
   - 반대로 `() 이름꼬리.` 또는 `(인자=값) 이름꼬리.`처럼 `Expr::Call`로 파싱되는 표면은 `tools/teul-cli/src/lang/parser.rs:887`~`907` 또는 `tools/teul-cli/src/lang/parser.rs:4919`~`4933`/`4963`~`4982`를 타고, 이후 `tools/teul-cli/src/runtime/eval.rs:2194`~`2228`에서 씨앗 dispatch가 가능하다.

3. `lang/src/parser.rs:5736 resolve_call_tail_candidates` 호출지:
   - 직접 호출지는 `lang/src/parser.rs:5713`과 `lang/src/parser.rs:5724` 두 곳뿐이며 둘 다 `resolve_call_target()` 내부다.
   - `resolve_call_target()`은 일반 호출에도 쓰인다. `lang/src/parser.rs:5304`에서 `ExprKind::Call` 일반 경로가 호출한다.
   - 파이프 전용 경로도 있다. `lang/src/parser.rs:5375`에서 `apply_defaults_in_pipe()`가 호출한다.
   - 단, `teul-cli run` 제품 실행은 `tools/teul-cli/src/cli/frontdoor_parse.rs:24`~`33`에서 `tools/teul-cli/src/lang`의 Lexer/Parser를 사용한다. `lang/src/parser.rs` 경로는 `tools/teul-cli/src/cli/run.rs:3979`~`3993`의 `collect_lang_parse_warnings_for_run()`에서 경고 수집용으로만 보인다.

4. `stem_alias_dop_dou.ddn` 재실행 및 오류 발생 지점:
   - 대상 파일:
     - `pack/lang_kernel_v1_conformance/cases/stem_alias_dop_dou.ddn:1`: `돕~도우:움직씨 = {`
     - `pack/lang_kernel_v1_conformance/cases/stem_alias_dop_dou.ddn:5`: `돕기.`
   - 재실행 로그: `I:\home\urihanl\ddn\codex\out\queue-20260706\q18\stem_alias_dop_dou_rerun_clean.txt`
   - 원문:
     ```text
     cmd=I:\home\urihanl\ddn\codex\target\debug\teul-cli.exe run pack/lang_kernel_v1_conformance/cases/stem_alias_dop_dou.ddn
     exit_code=1
     --- stdout ---
     --- stderr ---
     E_RUNTIME_UNDEFINED pack/lang_kernel_v1_conformance/cases/stem_alias_dop_dou.ddn:5:1 정의되지 않은 경로: 살림.돕기
     ```
   - 오류는 씨앗 이름 `돕` 자체가 undefined라는 뜻이 아니라, `돕기.`가 `살림.돕기` 경로로 평가된 뒤 상태 값이 없어 undefined가 된 것이다. 포맷은 `tools/teul-cli/src/cli/run.rs:4841`의 `RuntimeError::Undefined` 메시지다.
   - 보조 재현: 같은 내용을 `() 돕기.`로 바꾼 `I:\home\urihanl\ddn\codex\out\queue-20260706\q18\stem_alias_dop_dou_prefix_call.ddn`는 exit_code=0, stdout `도움`으로 PASS했다.

5. 비예약 `:움직씨`/`:셈씨` 선언 이름이 다른 위치에서 호출되는 실제 사례:
   - 검색 대상: `pack/**/*.ddn`, `tests/**/*.ddn`, `tools/**/*.ddn`, `lang/**/*.ddn`, `core/**/*.ddn`
   - 결과: 호출 후보 28건을 찾았다. 다만 제품 런타임에서 안정적으로 호출식이 되는 사례는 대체로 `()` 또는 `(인자=값)` prefix-call 형태이며, 문제가 된 `이름꼬리.` bare zero-arg 형태는 conformance 그룹1 사례에 집중되어 있다.
   - 대표 사례:
     - `pack/type_runtime_typecheck/input_infer_ok.ddn:1` 선언, `pack/type_runtime_typecheck/input_infer_ok.ddn:5` `(값="글") 통과하기.`
     - `pack/paper1_canonize_agglutinative_ast_v2/cases/c01_ab/input.ddn:1` 선언, `pack/paper1_canonize_agglutinative_ast_v2/cases/c01_ab/input.ddn:6` `(3을, 1에) 더하기.`
     - `pack/lang_kernel_v1_conformance/cases/tail_equiv_gi_hagi.ddn:1` 선언, `pack/lang_kernel_v1_conformance/cases/tail_equiv_gi_hagi.ddn:5` `회복기.`, `:6` `회복하기.`
     - `pack/lang_kernel_v1_conformance/cases/stem_alias_dop_dou.ddn:1` 선언, `:5` `돕기.`
   - 전체 검색 요약:
     ```text
     total_files=1545
     decls_with_calls=28
     pack/type_runtime_typecheck/input_infer_ok.ddn:1 통과:셈씨 -> 통과하기@5: (값="글") 통과하기.
     pack/type_runtime_typecheck/input_int_mismatch.ddn:1 받:셈씨 -> 받기@5: (값=1.5) 받기.
     pack/type_runtime_typecheck/input_num_mismatch.ddn:1 두배:셈씨 -> 두배하기@5: (값="글") 두배하기.
     pack/type_runtime_typecheck/input_num_unit_mismatch.ddn:1 통과:셈씨 -> 통과하기@5: (값=1@m) 통과하기.
     pack/type_runtime_typecheck/input_optional_mismatch.ddn:1 통과:셈씨 -> 통과하기@5: (값=1) 통과하기.
     pack/type_runtime_typecheck/input_optional_ok.ddn:1 통과:셈씨 -> 통과하기@5: () 통과하기.
     pack/paper1_canonize_agglutinative_ast_v2/cases/c01_ab/input.ddn:1 더하:셈씨 -> 더하기@6: (3을, 1에) 더하기.
     pack/paper1_canonize_agglutinative_ast_v2/cases/c02_ba/input.ddn:1 더하:셈씨 -> 더하기@6: (1에, 3을) 더하기.
     pack/paper1_canonize_agglutinative_ast_v2/cases/c03_positional/input.ddn:1 더하:셈씨 -> 더하기@6: (3, 1) 더하기.
     pack/paper1_canonize_agglutinative_ast_v2/cases/c04_alias_acceptance/input.ddn:1 더하:셈씨 -> 더하기@6: (3를, 1에) 더하기.
     pack/paper1_canonize_agglutinative_ast_v2/cases/c05_conflict_warning/input.ddn:1 이동:셈씨 -> 이동하기@6: (100@m:시작~에서) 이동하기.
     pack/lang_kernel_v1_conformance/cases/stem_alias_ambiguous.ddn:1 계산:셈씨 -> 계산하기@9: 계산하기.
     pack/lang_kernel_v1_conformance/cases/stem_alias_ambiguous.ddn:4 계산하:셈씨 -> 계산하기@9: 계산하기.
     pack/lang_kernel_v1_conformance/cases/stem_alias_dop_dou.ddn:1 돕~도우:움직씨 -> 돕기@5: 돕기.
     pack/lang_kernel_v1_conformance/cases/tail_equiv_gi_hagi.ddn:1 회복:움직씨 -> 회복하기@6: 회복하기.; 회복기@5: 회복기.
     ```

### Q18 검증 명령

```text
rg -n "resolve_call_tail_candidates|E_RUNTIME_UNDEFINED|SeedDef|user_seeds|eval_call|eval_path" tools/teul-cli/src/runtime tools/teul-cli/src/cli tools/teul-cli/src/lang lang/src -g "*.rs"
I:\home\urihanl\ddn\codex\target\debug\teul-cli.exe run pack/lang_kernel_v1_conformance/cases/stem_alias_dop_dou.ddn
I:\home\urihanl\ddn\codex\target\debug\teul-cli.exe run I:\home\urihanl\ddn\codex\out\queue-20260706\q18\stem_alias_dop_dou_prefix_call.ddn
git status --short
```

검증 결과:
- `stem_alias_dop_dou.ddn`: FAIL 재현, `E_RUNTIME_UNDEFINED ... 정의되지 않은 경로: 살림.돕기`
- `stem_alias_dop_dou_prefix_call.ddn`: PASS, stdout `도움`
- Q18 진단 중 코드/팩 수정 없음
