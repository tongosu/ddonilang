# 관찰자/변경자 경계 실측 조사 V1

- 실행일: 2026-07-06
- 브랜치: `codex/queue-20260706`
- 범위: `solutions/seamgrim_ui_mvp/ui/**/*.js`, `tool/src/wasm_api.rs`, `core/src/platform.rs`, `tools/teul-cli/src/**`
- 성격: 정적 분석 전용. 코드, pack, golden 수정 없음.

## 커버리지

- UI `.js` 파일 전체 수: 134개(`rg --files solutions/seamgrim_ui_mvp/ui | *.js`).
- WASM/관찰 함수명 검색에 걸린 파일: 12개.
  - 실제 호출/바인딩/재수출 파일: 10개.
  - 오탐 또는 단순 표면명 파일: `free_lab_first_run.js`는 `required_surface: "ddn_preset_parameters"` 문자열만 있음, `runtime/index.js`는 re-export만 있음.
- `seulgi_proposal_ui.js`, `seulgi_replay_safe_workflow.js`에 대해 `setParam/resetParsed/stepOne/runTicks/restoreState/injectAi/DdnWasmVm/wasm_/getState` 검색 결과는 없음.
- 제품 소스에서 `입력원천/사람/밖일/일정/이어전달/펼침실행/InputSource/input_source/source_kind/origin`을 검색했다. 6원천 enum/분류기는 발견되지 않았고, `source_kind`/`sourceId`는 문서·번들·UI 출처 메타데이터로만 확인됐다.

## 표1: D40 호출부 지도

| 호출부 파일 | 호출 함수 | 드라이버/관찰자 분류 | 근거 |
|---|---|---|---|
| `solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool.js` | `set_param*`, `reset`, `restore_state`, `run_ticks`, `set_input`, `step_one*`, `inject_ai_action`, `clear_ai_injections`, `get_state_hash`, `get_state_json`, `wasm_canon_*`, `wasm_preprocess_source` | WASM raw 표면(변경/관찰 혼합) | generated binding이 raw export를 그대로 노출한다. 예: class/export 시작 `:3`, 변경 함수 `:129`, `:186`, `:205`, `:226`, `:254`, `:278`, `:300`, `:322`, `:357`, `:380`, 관찰 함수 `:102`, `:121`, canon 함수 `:463` 이후. |
| `solutions/seamgrim_ui_mvp/ui/wasm_ddn_wrapper.js` | `updateLogic*`, `setInput`, `setParam*`, `injectAiAction`, `resetParsed`, `restoreStateParsed`, `stepOne*`, `runTicksParsed`, `getStateHash`, `getStateParsed` | 공용 래퍼(변경/관찰 혼합) | `DdnWasmVmClient`가 raw API를 얇게 감싼다(`:14`). 변경 함수는 `:21`, `:36`, `:63`, `:70`, `:77`, `:100`, `:116`, `:125`, `:135`, `:140`, `:159`; 관찰 함수는 `:166`, `:170`. |
| `solutions/seamgrim_ui_mvp/ui/wasm_page_common.js` | `DdnWasmVm`, `DdnWasmVmClient`, `wasm_preprocess_source`, `updateLogic*`, `setParam*`, `stepOne*`, `getStateParsed` | 드라이버 공유 헬퍼 | VM 생성/갱신과 파라미터 적용, 한 틱 실행을 수행한다. VM 생성 `:3066`, `:3150`, `:3182`; wrapper 생성 `:3160`, `:3192`; preprocess `:3120`, `:3130`; update `:2796`, `:2800`, `:3205`; param `:2542`-`:2570`; step `:2593`-`:2625`; state read `:2861`-`:2864`. |
| `solutions/seamgrim_ui_mvp/ui/runtime/wasm_vm_runtime.js` | `getStateParsed`, `resetParsed`, `runTicksParsed`, `stepOneWithInputParsed`, `stepOneParsed`, `setParamParsed`, `getStateHash` | 드라이버 런타임 추상화 | VM 실행을 비동기 런타임 핸들로 감싼다. 관찰 read `:207`, `:358`-`:387`; reset/run/step `:227`, `:253`-`:266`; param `:338`-`:347`. |
| `solutions/seamgrim_ui_mvp/ui/screens/run.js` | `resetParsed`, `runTicksParsed`, `getStateParsed`, `getStateHash` | 드라이버 | RunScreen은 제품 시뮬레이션 실행 화면이다. reset/read `:6892`-`:6900`; 초기 run ticks `:12289`-`:12315`; 이후 hash read `:12334`, `:12584`. |
| `solutions/seamgrim_ui_mvp/ui/app.js` | `applyWasmLogicAndDispatchState`, `getStateHash` | 드라이버 진입점 | 제품 앱이 공유 헬퍼를 import하고 실행한다(`:1`, `:3440`); 실행 뒤 hash를 읽는다(`:3445`). 직접 raw WASM export 호출은 없다. |
| `solutions/seamgrim_ui_mvp/ui/playground.js` | `createWasmCanon`, `preprocessSource`, `canonFlatJson`, `canonMaegimPlan`, `canonAlrimPlan`, `canonBlockEditorPlan`, `resetParsed`, `getStateParsed` | 혼합: canon 관찰 + 플레이그라운드 드라이버 | canon 런타임 생성 `:29`, `:526`; canon/preprocess 사용은 `:568`-`:580`; 플레이그라운드 VM reset/read는 `:1191`-`:1195`. |
| `solutions/seamgrim_ui_mvp/ui/runtime/wasm_canon_runtime.js` | `wasm_canon_flat_json`, `wasm_canon_maegim_plan`, `wasm_canon_alrim_plan`, `wasm_canon_block_editor_plan`, `wasm_preprocess_source` | 관찰자/읽기 전용 | 소스 정본화/계획 JSON만 읽는다. 호출 지점은 `:115`-`:127`, `:140`-`:152`, `:165`-`:177`, `:190`-`:203`, `:218`-`:228`. 변경 함수 호출 없음. |
| `solutions/seamgrim_ui_mvp/ui/runtime/lesson_canon_runtime.js` | `canonMaegimPlan`, `canonFlatJson` | 관찰자/읽기 전용 | lesson 메타/매김 계획을 정본 런타임에서 조회한다(`:214`, `:245`). 변경 함수 호출 없음. |
| `solutions/seamgrim_ui_mvp/ui/block_editor/ddn_block_codec.js` | `canonBlockEditorPlan`, `canonAlrimPlan` | 관찰자/읽기 전용 | 블록 편집기 decode용 계획만 요구한다(`:318`-`:327`, `:338`-`:347`). 변경 함수 호출 없음. |
| `solutions/seamgrim_ui_mvp/ui/runtime/index.js` | `createWasmVm`, `createWasmCanon` re-export | 표면 재수출 | `:1`-`:2`에서 런타임 factory만 다시 내보낸다. 직접 WASM 호출 없음. |
| `solutions/seamgrim_ui_mvp/ui/free_lab_first_run.js` | 없음 | 오탐 | 넓은 검색에서 `required_surface: "ddn_preset_parameters"` 문자열(`:53`)만 걸렸다. WASM API 호출 없음. |
| `solutions/seamgrim_ui_mvp/ui/seulgi_proposal_ui.js` | 없음 | 관찰자/제안 UI | Q28 대상 observer. mutation/read WASM API 검색 결과 없음. 현재 필요한 최소 WASM 읽기 함수 집합은 없음. |
| `solutions/seamgrim_ui_mvp/ui/seulgi_replay_safe_workflow.js` | 없음 | 관찰자/안전 워크플로 UI | Q28 대상 observer. mutation/read WASM API 검색 결과 없음. 현재 필요한 최소 WASM 읽기 함수 집합은 없음. |
| 나머지 UI `.js` 122개 | 없음 | 직접 WASM 호출 없음 | 전체 134개 중 실제 호출/바인딩/재수출/오탐 12개를 제외한 파일들이다. |

### 관찰자 모듈의 현재 최소 읽기 표면

| 관찰자 파일/그룹 | 현재 필요한 최소 읽기 함수 |
|---|---|
| `seulgi_proposal_ui.js`, `seulgi_replay_safe_workflow.js` | 없음. UI 로컬 데이터만 사용한다. |
| `runtime/wasm_canon_runtime.js` | raw `wasm_canon_flat_json`, `wasm_canon_maegim_plan`, `wasm_canon_alrim_plan`, `wasm_canon_block_editor_plan`, `wasm_preprocess_source`. |
| `runtime/lesson_canon_runtime.js` | `canonMaegimPlan`, `canonFlatJson`. |
| `block_editor/ddn_block_codec.js` | `canonBlockEditorPlan`, `canonAlrimPlan`. |
| 드라이버 내부 관찰(`screens/run.js`, `runtime/wasm_vm_runtime.js`, `app.js`) | `getStateParsed`, `getStateHash`가 쓰이지만 reset/run/step/setParam와 같은 드라이버 소유 변경 흐름 안에서만 확인된다. |

## 표2: D41 입력원천별 실제 코드 경로 지도

| 입력원천 후보 | 발생 파일:함수 | InputSnapshot 필드 | 현재 코드상 구분 방법 |
|---|---|---|---|
| 사람: WASM 브라우저 입력 | `tool/src/wasm_api.rs:DdnWasmVm::set_input`(`:287`), `step_one_with_input`(`:475`), `step_one` 입력 구성(`:493` 이후) | core `InputSnapshot.keys_pressed`, `last_key_name`, `pointer_x_i32`, `pointer_y_i32`, `dt` | 함수명 `set_input`/`step_one_with_input`; 필드명 `keys_pressed`, `last_key_name`, `pointer_*`. 6원천 이름 `사람`은 없음. |
| 사람: CLI sam/live 키 입력 | `tools/teul-cli/src/cli/sam_live.rs:LiveInput::sample_tick`(`:77`), `tools/teul-cli/src/cli/run.rs` live branch(`:4324`-`:4335`), `apply_input_frame`(`:1147`) | CLI geoul `InputSnapshotV1.held_mask`, `pressed_mask`, `released_mask`; 실행 상태에는 `샘.키보드.*`, `입력상태.키_*` 자원으로 들어감 | `LiveInput`, `OpenInputFrame`, `held/pressed/released`, `샘.키보드`, `입력상태` 이름. 6원천 enum 없음. |
| 사람: sam 계획 입력 | `tools/teul-cli/src/cli/run.rs:SamPlan::frame_for_tick`(`:827`), `SamPlan::apply_frame`(`:836`), sam branch(`:4287`-`:4297`) | geoul `InputSnapshotV1.held_mask`, `pressed_mask`, `released_mask`; 실행 상태 `샘.키보드.*` | `SamPlan`, `sam_path`, `OpenInputFrame`. 6원천 enum 없음. |
| 슬기: core 샘 AI 큐 | `core/src/platform.rs:DetSam::push_async_ai`(`:1128`), `DetSam::begin_tick`(`:1109`-`:1123`) | core `InputSnapshot.ai_injections: Vec<SeulgiPacket>` | `ai_queue`, `ai_injections`, `SeulgiPacket`, `SeulgiIntent`. 6원천 이름 `슬기`는 타입명 일부에만 있고 입력원천 enum은 없음. |
| 슬기: WASM 주입 | `tool/src/wasm_api.rs:DdnWasmVm::inject_ai_action`(`:769`), `pending_ai_injections` drain(`:495`-`:516`) | core `InputSnapshot.ai_injections` | `pending_ai_injections`, `inject_ai_action`. |
| 슬기: CLI intent 파일 | `tools/teul-cli/src/cli/intent.rs:parse_intent_jsonl`(`:76`), `parse_intent`(`:140`-`:155`) | 이 파일 안에서는 직접 `InputSnapshot`에 넣지 않음 | `SeulgiIntent::{None,MoveTo,Attack,Say}`. intent parser는 확인되지만 6원천 분류기는 아님. |
| 밖일: core net event | `core/src/platform.rs:DetSam::push_net_event`(`:1092`), `DetSam::begin_tick`(`:1109`-`:1123`) | core `InputSnapshot.net_events: Vec<NetEvent>` | `net_queue`, `net_events`, `NetEvent`. 6원천 이름 `밖일` 없음. |
| 밖일: CLI detjson/net event | `tools/teul-cli/src/cli/run.rs:read_net_events_detjson`(`:976`), `apply_net_events`(`:1076`), `build_geoul_snapshot`(`:1175`), `read_net_events_from_state`(`:1276`) | geoul `InputSnapshotV1.net_events`; 실행 상태 `샘.네트워크.이벤트_*` | `net_events`, `NetEventDet`, `샘.네트워크`. |
| 밖일: gateway 입출력 | `tools/teul-cli/src/cli/gateway.rs:GatewayNetEvent`(`:38`), `read_gateway_events`(`:198`), socket read `:414` 이후, `send_events_over_tcp`(`:565`), `order_and_dedupe_events`(`:605`) | gateway 자체는 `GatewayNetEvent`; `InputSnapshot` 직접 필드는 아님. run/geoul 경로로 들어가면 `net_events` 후보 | `GatewayNetEvent`, `net_events`, `source_kind` 메타데이터. `밖일` enum 없음. |
| 밖일: 외부 작업 결과 | `tools/teul-cli/src/cli/dultra_replay.rs` | `InputSnapshot` 필드 없음 | `external_result_normalization: "not_runtime_landed"`(`:54`), `external_ode_stub`(`:105`), `backend: "external_stub"`(`:107`)로 preview/stub 상태 확인. 실제 runtime input source로 착륙한 경로는 발견하지 못했다. |
| 일정: 입력 지연/스케줄 | `tools/teul-cli/src/cli/run.rs:InputLatencyQueue::schedule_and_take`(`:692`), sam/live/open branches `:4293`, `:4331`, `:4352`; `core/src/seulgi/latency.rs:LatencySchedule`(`:28`), `ScheduledPacket`(`:35`) | 새 `InputSnapshot` 원천 필드는 없음. 지연 후 기존 key frame 또는 open input frame으로 적용된다. AI packet에는 `accepted_madi`, `target_madi`가 있음 | `latency_madi`, `latency_schedule`, `target_madi`, `deliver_madi`. 6원천 이름 `일정` 없음. |
| 이어전달: record/replay/open input | `tools/teul-cli/src/runtime/open.rs:OpenRuntime::open_input`(`:692`), `parse_open_input_frame`(`:1340`), `tools/teul-cli/src/cli/replay.rs:apply_snapshot` 사용(`:98`), `tools/teul-cli/src/cli/replay_branch.rs:snapshot_from_held_mask`(`:112`) 및 `apply_snapshot`(`:147`) | geoul `InputSnapshotV1.held_mask`, `pressed_mask`, `released_mask`, `net_events`; 이후 `샘.키보드.*`/`샘.네트워크.*` 상태 자원 | `replay`, `replay_branch`, `open_input`, `OpenInputFrame`. 6원천 이름 `이어전달` 또는 relay enum은 없음. |
| 펼침실행 | `tool/src/wasm_api.rs:DdnWasmVm::apply_currentline_cell`(`:616`-`:636`)만 후보로 확인 | `InputSnapshot` 필드 없음. 함수는 currentline 적용 후 `run_ticks(1)` 결과를 돌려주는 변경 API 후보일 뿐 입력원천으로 분리되지 않음 | `apply_currentline_cell` 이름은 있으나 `펼침실행` 문자열/enum/분류기는 발견하지 못했다. 구현 없음 판정. |

## 판정 요약

- D40: 제품 UI에서 mutation-capable WASM 표면은 raw binding, wrapper, shared page helper, VM runtime, RunScreen, Playground 쪽에 존재한다. Q28 관찰자 모듈(`seulgi_proposal_ui.js`, `seulgi_replay_safe_workflow.js`)은 실제로 mutation 함수를 import/call하지 않는다.
- D40: 현재 읽기 전용 관찰자 표면으로 실측되는 것은 canon/preprocess 계열이며, state read(`getStateParsed`, `getStateHash`)는 드라이버 흐름 내부에서만 확인된다.
- D41: 사람/슬기/net event/일정/replay류 경로는 구현되어 있으나 모두 기존 필드명·모듈명으로만 구분된다. 6원천 enum/분류기(`사람/슬기/밖일/일정/이어전달/펼침실행`)는 제품 코드에서 발견되지 않았다.
- D41: `밖일` 중 net event/gateway는 구현 경로가 있으나, 외부 작업 결과는 `dultra_replay`의 `not_runtime_landed` stub 상태로 확인된다. `펼침실행`은 입력원천 구현 없음으로 확인된다.
