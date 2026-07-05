# BRIEF: 생태계 계층 계약(D39~D41) 실코드 검증

> 작성: Claude (2026-07-05) / 실행: Codex / 리뷰: Claude
> 근거: `docs/context/proposals/PROPOSAL_ECOSYSTEM_LAYER_CONTRACT_V1_20260705.md`
> 성격: **진단/조사 전용.** 코드 수정 없음. 이 브리프는 제안 문서의 주장이 실제 코드와 일치하는지 확인하는 것이 유일한 목적이다.

## 배경

`PROPOSAL_ECOSYSTEM_LAYER_CONTRACT_V1_20260705.md`는 3가지를 주장한다:
- D39: 이야기/누리 계층은 오직 DDN 언어 코드만 실행한다(네이티브 훅/플러그인 없음).
- D40: 보개(UI 렌더러)/거울 소비 도구는 누리에 쓰기 접근이 없고 읽기 전용이다.
- D41: 모든 입력원천(사람/슬기/밖일/일정/이어전달/펼침실행)은 반드시 샘 경계를 거치며 세계 상태를 직접 수정하지 않는다.

이 주장이 SSOT 문면상으로는 맞아 보이지만, 이번 세션에서 반복적으로 "문서상 주장과 실제 코드가 다르다"는 걸 발견했다(Q18, Q20). 이 세 주장을 실제 코드로 확인하기 전에는 제안을 확정하지 않는다.

## 작업

### 1. D39 검증 — 이야기/누리 네이티브 훅 여부

- `tools/teul-cli/src/`에서 이야기/누리 실행 경로(엔진 tick 루프, patch 생성/적용 코드 — `core/engine.rs` 및 관련 모듈)를 확인하라.
- 동적 라이브러리 로드(`libloading`, `dlopen` 류), 임의 스크립팅 훅, WASM 사용자 플러그인 실행 지점이 이 경로 안에 있는지 `rg`로 확인하라.
- `--features wasm`의 `wasm_api.rs`가 이야기/누리 실행 자체에 개입하는지, 아니면 순수 진입점(external API surface)일 뿐인지 구분해서 보고하라.

### 2. D40 검증 — 보개 확장의 쓰기 접근 여부

- `solutions/seamgrim_ui_mvp/ui/seulgi_proposal_ui.js`, `seulgi_replay_safe_workflow.js`, 그리고 `run` 화면의 다른 render 모듈들(`app.js`에서 import되는 것들 중 render 계열)을 확인하라.
- 이 모듈들이 WASM 브릿지를 통해 엔진 상태를 변경하는 함수(예: mutation/apply/patch/tick 계열 export)를 호출하는 경로가 있는지 확인하라. 순수하게 읽은 값을 DOM에 그리기만 하는지, 아니면 어딘가 쓰기 호출이 섞여 있는지가 핵심이다.
- `tools/wasm_api.rs`(또는 `tool/`의 wasm 진입점)가 노출하는 함수 중 "읽기 전용"이라 이름 붙었지만 실제로 내부에서 state를 변경하는 것이 있는지도 확인하라.

### 3. D41 검증 — 입력 경로가 샘을 우회하는지

- `tools/teul-cli/src/cli/intent.rs`, `sam_snapshot.rs`, `sam_live.rs`, `core/geoul.rs`와 `InputSnapshot`/`SeulgiIntent`/`ai_injections` 관련 코드를 확인하라(이 파일들은 `grep -rln "InputSnapshot\|SeulgiIntent\|ai_injections"`로 이미 위치를 확인함).
- 6개 입력원천(사람/슬기/밖일/일정/이어전달/펼침실행) 각각이 실제로 샘 경계(tick 시작 시점의 스냅샷/동결)를 거치는지, 혹은 그중 하나라도 이야기/누리에 직접 값을 주입하는 우회 경로가 있는지 확인하라.
- 있다면 어느 입력원천이, 어느 함수에서 우회하는지 구체적으로 보고하라.

## 검증 방법

- 정적 분석(코드 읽기 + `rg`)이 기본이다. 실행이 필요하면 기존 pack/lesson으로 재현하되, 새 pack/golden을 만들지 않는다.
- 코드 수정, golden 갱신, 파일 생성/삭제 전혀 없음 — 순수 조사 보고서만 산출한다.

## 산출물

`docs/context/reports/ECOSYSTEM_CONTRACT_VERIFICATION_V1.md`:
- D39/D40/D41 각각에 대해 "위반 없음(주장과 일치)" 또는 "위반 발견"으로 판정.
- 위반 발견 시: 파일:행 번호, 코드 스니펫, 왜 계약 위반인지 설명.
- 위반 없음 판정도 근거(확인한 파일/함수 목록)를 명시해야 한다 — "확인 안 하고 위반 없음"은 금지.

## 수용 기준

- [ ] D39/D40/D41 셋 다 판정 완료(위반 있음/없음 + 근거)
- [ ] 위반 발견 시 파일:행 명시
- [ ] 코드/golden/pack 변경 없음(`git status --short` 보고서 파일 외 출력 없음)
- [ ] `codex/queue-20260706`(또는 신규 큐) 브랜치에 커밋 1개

## 금지 사항

코드 수정 없음. 위반을 발견해도 그 자리에서 고치지 마라 — 보고만 하라(수리는 별도 브리프로 Claude가 설계). main 직접 커밋 금지.

## 보고 형식

이 파일 하단 `## 실행 보고`: D39/D40/D41 판정 요약, 산출물 경로, 검증 방법.

## 실행 보고

- 실행일: 2026-07-06
- 브랜치: `codex/queue-20260706`
- 산출물: `docs/context/reports/ECOSYSTEM_CONTRACT_VERIFICATION_V1.md`
- 검증 방법: 정적 분석(`rg` 검색 + 코드 읽기). 실행/pack/golden 생성 없음. 코드 수정 없음.
- D39 판정: 위반 없음.
  - `core/src/engine.rs:40`~`48`의 tick 루프가 `Sam.begin_tick -> Iyagi.run_update -> Nuri.apply_patch` 순서로 고정됨을 확인했다.
  - `libloading|dlopen|LoadLibrary|wasmtime|wasmer|rhai|mlua|pyo3|boa_engine|deno_core` 검색 결과 제품 소스 경로에서 0건.
  - `tool/src/wasm_api.rs`는 외부 WASM API 표면이며 `DdnRunner`/`DetNuri`를 감싸는 경로로 확인했다.
- D40 판정: 위반 발견(계약 미착륙).
  - `seulgi_proposal_ui.js`, `seulgi_replay_safe_workflow.js` 자체는 WASM mutation 호출 없이 DOM/clipboard 중심이며 `auto_apply:false`, `file_write:false`, `state_hash_owner:false`를 고정한다.
  - 다만 `tool/src/wasm_api.rs`와 `wasm_ddn_wrapper.js`/`RunScreen`에는 `set_param`, `reset`, `step_one`, `run_ticks`, `restore_state`, `inject_ai_action` 등 쓰기 API가 노출·사용된다. observer 전용 capability 경계는 아직 없다.
- D41 판정: 부분 위반/미착륙.
  - 구현된 키보드/sam-live/replay/net event/슬기 주입은 `샘.*`/`입력상태.*` 또는 `InputSnapshot` 경로로 모이는 것을 확인했다.
  - 하지만 제품 소스에서 `입력원천`, `사람`, `밖일`, `일정`, `이어전달`, `펼침실행` 6원천 분류기/enum은 찾지 못했다. 제안서의 강한 6원천 계약은 아직 코드로 강제되지 않는다.
