# ECOSYSTEM_CONTRACT_VERIFICATION_V1

> 작성: Codex (2026-07-06)
> 범위: `PROPOSAL_ECOSYSTEM_LAYER_CONTRACT_V1_20260705.md`의 D39/D40/D41 실코드 대조
> 성격: 진단 전용. 코드, pack, golden 수정 없음.

## 요약

| 항목 | 판정 | 핵심 근거 |
|---|---|---|
| D39 이야기/누리 순수 DDN | 위반 없음 | 실행 루프는 `Sam.begin_tick -> Iyagi.run_update -> Nuri.apply_patch`이고, 제품 소스에서 동적 로더/임의 스크립팅 런타임 검색 결과 없음. WASM은 외부 API 표면으로 DDN runner를 감싼다. |
| D40 보개/거울 관찰자 읽기 전용 | 위반 발견(계약 미착륙) | Q26의 `seulgi_*` 두 UI 모듈 자체는 읽기 전용이나, 동일 UI/WASM 표면에 `set_param`, `reset`, `step_one`, `run_ticks`, `restore_state`, `inject_ai_action` 등 쓰기 API가 노출되어 있고 RunScreen에서 사용한다. 관찰자 전용 capability 경계가 없다. |
| D41 입력원천 샘 경유 | 부분 위반/미착륙 | 구현된 키보드/라이브/리플레이/넷 이벤트/슬기 주입은 샘 네임스페이스 또는 `InputSnapshot`으로 모이지만, 제안서의 6원천(`사람/슬기/밖일/일정/이어전달/펼침실행`) enum/분류기는 제품 코드에 없다. |

## 검증 방법

- 정적 분석만 수행했다. 실행, 코드 수정, golden 갱신, pack 생성은 하지 않았다.
- 주요 검색:
  - `rg -n 'libloading|dlopen|LoadLibrary|wasmtime|wasmer|rhai|mlua|pyo3|boa_engine|deno_core' --glob Cargo.toml --glob '*.rs' -- tools/teul-cli/src core/src lang/src tool/src`
  - 결과: `NO_MATCH dynamic-loader-or-scripting-runtime in tools/teul-cli/src core/src lang/src tool/src`
  - `rg -n '입력원천|사람|밖일|일정|이어전달|펼침실행' -- tools/teul-cli/src core/src tool/src lang/src`
  - 결과: `NO_MATCH six-input-origin-labels in product source paths`

## D39 — 이야기/누리 네이티브 훅 여부

판정: **위반 없음.**

확인한 실행 경로:

- `core/src/engine.rs:40`의 `tick_once`는 먼저 `self.sam.begin_tick(tick_id)`로 입력을 동결한다.
- `core/src/engine.rs:45`는 `self.iyagi.run_update(self.nuri.world(), &snapshot)`만 호출한다.
- `core/src/engine.rs:48`은 생성된 patch를 `self.nuri.apply_patch(...)`로 적용한다.
- `core/src/platform.rs:1021`~`1042`의 trait 경계도 `Sam`, `Nuri`, `Iyagi`로 나뉘어 있고, `Iyagi::run_update`는 `&NuriWorld`와 `&InputSnapshot`을 받는다.
- `core/src/platform.rs:1108`~`1123`의 `DetSam::begin_tick`은 AI/네트워크 큐를 정렬한 뒤 `InputSnapshot`에 담는다.

제품 실행 frontdoor:

- `tools/teul-cli/src/cli/run.rs:4204`~`4212`는 메모리 실행에서 `Evaluator::with_state_seed_open(..., OpenRuntime::deny(), ...)` 후 `run_with_ticks`를 호출한다.
- `tools/teul-cli/src/cli/run.rs:4259`~`4312`는 open/sam 경로에서도 `Evaluator`의 tick 실행 API를 사용한다.
- `tool/src/ddn_runtime.rs:1713`~`1718`은 `DdnProgram::from_source`가 DDN source를 파싱하는 진입점이고, `tool/src/ddn_runtime.rs:2655` 이후 `DdnRunner`가 프로그램을 실행한다.

네이티브 훅/플러그인 검색:

- `libloading`, `dlopen`, `LoadLibrary`, `wasmtime`, `wasmer`, `rhai`, `mlua`, `pyo3`, `boa_engine`, `deno_core` 검색 결과는 제품 소스 경로에서 0건이었다.
- `Command::new`은 `tools/teul-cli/src/cli/test.rs`, `cli/patch.rs`, `cli/view.rs`, `cli/run.rs`의 브라우저 열기/테스트 실행 보조에만 나타났고, 이야기/누리 update 루프 안의 사용자 플러그인 로더로 쓰이지 않는다.

WASM 구분:

- `tool/src/wasm_api.rs:11`은 `wasm_bindgen`을 사용하고, `tool/src/wasm_api.rs:134`의 `DdnWasmVm`이 외부 JS API 표면이다.
- `tool/src/wasm_api.rs:157`과 `187`의 생성자는 source를 `parse_program_for_wasm`으로 파싱하고 `DdnRunner::new(...)`를 만든다.
- `tool/src/wasm_api.rs:493`~`528`의 `step_one`은 `InputSnapshot`을 만들고 `runner.run_update(...)` 후 `world.apply_patch(...)`를 수행한다. 이는 외부 API 표면이지, DDN 밖 사용자 플러그인을 이야기/누리 내부에 끼우는 경로는 아니다.

## D40 — 보개 확장의 쓰기 접근 여부

판정: **위반 발견(계약 미착륙).**

현재 관찰 모듈 중 Q26 대상은 읽기 전용에 가깝다:

- `solutions/seamgrim_ui_mvp/ui/app.js:32`~`39`는 `seulgi_proposal_ui.js`와 `seulgi_replay_safe_workflow.js`의 build/render 함수만 import한다.
- `solutions/seamgrim_ui_mvp/ui/app.js:297`과 `306`은 각각 `renderSeulgiProposalUi(...)`, `renderSeulgiReplaySafeWorkflow(...)`만 호출한다.
- `solutions/seamgrim_ui_mvp/ui/seulgi_proposal_ui.js:62`~`65`, `102`~`105`는 `auto_apply:false`, `file_write:false`, `runtime_ast_persisted:false`, `state_hash_owner:false`를 데이터로 고정한다.
- `solutions/seamgrim_ui_mvp/ui/seulgi_replay_safe_workflow.js:62`~`65`, `102`~`105`도 같은 부정 claim을 고정한다.
- 두 모듈의 상호작용은 row 재렌더와 clipboard 복사뿐이다(`seulgi_proposal_ui.js:213`~`220`, `seulgi_replay_safe_workflow.js:213`~`220`).

하지만 UI/WASM 표면 전체는 읽기 전용으로 격리되어 있지 않다:

- `tool/src/wasm_api.rs:336` `set_param`, `438` `reset`, `493` `step_one`, `584` `run_ticks`, `691` `restore_state`, `769` `inject_ai_action`이 모두 외부 JS에 노출된다.
- `tool/src/wasm_api.rs:528`과 `609`는 WASM VM 내부의 `DetNuri`에 patch를 적용한다.
- `solutions/seamgrim_ui_mvp/ui/wasm_ddn_wrapper.js:63`~`67`, `100`~`104`, `122`, `129`, `136`, `144`, `161`은 위 mutation-capable WASM 함수를 JS client 메서드로 감싼다.
- `solutions/seamgrim_ui_mvp/ui/runtime/wasm_vm_runtime.js:218`~`227`, `253`~`266`, `338`~`347`은 reset/run/step/setParam을 실제로 호출한다.
- `solutions/seamgrim_ui_mvp/ui/screens/run.js:6892`~`6899`, `12289`~`12300`, `12561`~`12586`도 제품 RunScreen에서 reset/runTicks/stepFrame 경로를 사용한다.

해석:

- 현재 `seulgi_*` 두 관찰 UI는 D40 원칙을 위반하지 않는다.
- 그러나 "보개 확장/거울 소비 도구는 누리에 쓰기 접근이 없다"는 주장은 아직 코드 경계로 강제되지 않는다. 같은 UI bundle에서 observer 모듈이 mutation-capable client를 import하거나 전달받는 것을 막는 SDK/capability 분리가 없다.
- 따라서 D40은 "현재 일부 모듈 관례로는 충족"이지만 "계약으로 landed"라고 볼 수 없다.

## D41 — 입력 경로가 샘을 우회하는지

판정: **부분 위반/미착륙. 구현된 입력 경로는 대체로 샘 경계를 통과하지만, 6원천 계약은 코드에 없다.**

구현된 경로:

- Core 엔진은 `core/src/engine.rs:42`에서 `Sam.begin_tick`을 먼저 호출한 뒤 `Iyagi.run_update`로 들어간다.
- `core/src/platform.rs:894`~`903`의 `InputSnapshot`은 tick, 키, 포인터, `ai_injections`, `net_events`, rng seed를 담는다.
- `core/src/platform.rs:1092`~`1102`는 `DetSam::push_net_event`, `1108`~`1123`은 `begin_tick`의 정렬/동결, `1128`~`1141`은 `push_async_ai`의 큐 적재 경로다.
- CLI sam tape 경로는 `tools/teul-cli/src/cli/run.rs:827`~`844`에서 `OpenInputFrame`을 만들고 `샘.키보드.*`/`입력상태.*`에 반영한다.
- live input 경로는 `tools/teul-cli/src/cli/sam_live.rs:77`~`95`에서 tick별 held/pressed/released를 샘플링한다.
- `tools/teul-cli/src/cli/run.rs:4286`~`4297`, `4323`~`4335`, `4347`~`4356`은 sam/live/open replay frame을 tick 시작 전 `apply_frame`/`apply_input_frame`에 넣는다.
- `tools/teul-cli/src/runtime/eval.rs:736`~`746`은 매 tick `before_tick`을 먼저 실행한 뒤 본문 평가로 들어간다.
- `tools/teul-cli/src/cli/sam_snapshot.rs:9`~`17`은 거울 snapshot replay를 `샘`/`입력상태` 키로 되살린다.
- `tools/teul-cli/src/core/geoul.rs:76`~`85`는 `InputSnapshotV1` 포맷을 갖고, `145`~`153`은 geoul state hash에서 샘 키를 제거한다.
- WASM 경로는 `tool/src/wasm_api.rs:475`~`528`에서 input/AI를 `InputSnapshot`에 담은 뒤 DDN runner를 실행하고 patch를 적용한다. `tool/src/wasm_api.rs:769`~`775`의 AI 주입도 다음 `step_one`에서 `SeulgiPacket`으로 바뀐다.

문제점:

- 제품 소스에서 `입력원천`, `사람`, `밖일`, `일정`, `이어전달`, `펼침실행` 문자열 검색 결과가 0건이다. 즉 제안서가 말하는 6원천 enum/분류기는 코드에 없다.
- 구현된 구조는 키보드/라이브/리플레이/넷 이벤트/슬기 주입을 다루지만, 각각을 6원천 스키마로 기록하거나 검증하는 표준 manifest/타입은 찾지 못했다.
- `tools/teul-cli/src/cli/run.rs:1147`~`1155`의 `apply_input_frame`은 evaluator `State`를 직접 변경한다. 변경 대상은 샘 네임스페이스라 현재 구현상 경계 역할을 하지만, core의 `InputSnapshot` trait 경계와 동일한 타입 시스템 경계는 아니다.

해석:

- "구현된 입력이 비샘 세계 자원으로 직접 들어가는 우회"는 확인하지 못했다.
- 그러나 D41의 강한 주장인 "6개 입력원천이 반드시 샘 경계를 거친다"는 아직 제품 코드의 타입/enum/검사로 존재하지 않는다.
- 따라서 D41은 원칙 일부는 구현되어 있으나, 계약 확정 전에는 "이미 사실"이 아니라 "정식 분류기와 검사 추가 필요"로 분류하는 편이 맞다.

## 결론

- D39는 현재 코드와 일치한다.
- D40은 Q26 observer 모듈만 보면 맞지만, 제품 SDK/권한 경계로는 미착륙이다. mutation-capable WASM client와 observer module을 분리하는 계약/API가 필요하다.
- D41은 샘 경계 관례와 일부 구현은 있으나 6원천 스키마가 제품 코드에 없으므로 미착륙이다. 특히 "사람/슬기/밖일/일정/이어전달/펼침실행"을 검증 가능한 타입/manifest로 올려야 제안 문구를 확정할 수 있다.
