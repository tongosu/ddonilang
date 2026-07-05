# BRIEF: D40 수리 — 관찰자 전용 JS 클라이언트 분리

> 작성: Claude (2026-07-06) / 실행: Codex / 리뷰: Claude
> 근거: `docs/context/reports/ECOSYSTEM_CONTRACT_VERIFICATION_V1.md`(Q28), `docs/context/reports/OBSERVER_MUTATOR_BOUNDARY_SURVEY_V1.md`(Q29)
> 성격: **실제 코드 구현.** 설계는 이 브리프로 확정됨 — 추가 설계 판단 없이 그대로 구현.

## 배경

Q29가 확인한 것: `tool/src/wasm_api.rs`의 `get_state_hash`/`get_state_json`은 **이미 `&self`(읽기 전용) 메서드**다 — Rust 쪽 mutation 여부 문제가 아니다. 문제는 JS 쪽에 있다: `wasm_ddn_wrapper.js`/`wasm_page_common.js`가 mutation 함수(`setParam*`, `reset`, `stepOne*`, `runTicks`, `restoreState`, `injectAiAction`)와 읽기 함수를 **한 파일에 묶어서** 내보내고, "관찰만 하면 되는" 모듈이 가져다 쓸 **얇은 읽기 전용 JS 클라이언트가 없다.**

같은 문제를 canon 쪽은 이미 풀어놓았다: `runtime/wasm_canon_runtime.js`는 canon 함수만 내보내는 별도 파일이고, 실제로 `lesson_canon_runtime.js`/`ddn_block_codec.js` 같은 순수 관찰자들이 이 파일만 가져다 쓴다(Q29 표1 확인). **이 패턴을 state 읽기 쪽에도 그대로 복제하면 된다.** Rust 코드/WASM 빌드는 건드릴 필요 없다.

## 작업

### 1. 얇은 관찰자 클라이언트 신설

`solutions/seamgrim_ui_mvp/ui/runtime/wasm_state_observer_client.js`(신규 파일, `wasm_canon_runtime.js`와 동일한 구조/스타일 규칙을 따를 것 — 그 파일을 먼저 읽고 그대로 모사):

- 기존 `DdnWasmVm`/`DdnWasmVmClient` 인스턴스(이미 생성된 것)를 인자로 받아, **오직 다음만** 노출하는 뷰 객체/함수를 만든다: `getStateHash(vm)`, `getStateParsed(vm)`(내부적으로 `get_state_hash`/`get_state_json`만 호출).
- `setParam*`, `reset`, `stepOne*`, `runTicks`, `restoreState`, `injectAiAction` 등 mutation 계열 함수는 이 파일 안에 **이름조차 등장하면 안 된다** — import도, re-export도, 문자열 참조도 없어야 한다.
- 이 파일은 VM을 **생성하지 않는다** — 생성/구동은 여전히 드라이버(`wasm_page_common.js`, `screens/run.js`)의 몫이다. 이 클라이언트는 이미 존재하는 VM 인스턴스를 읽기만 하는 뷰다.

### 2. 회귀 가드 테스트

`tests/run_wasm_state_observer_client_capability_check.py`(신규):
- `runtime/wasm_state_observer_client.js` 파일 텍스트에 `set_param|setParam|reset|step_one|stepOne|run_ticks|runTicks|restore_state|restoreState|inject_ai_action|injectAiAction` 패턴이 **하나도 없음**을 정적으로 확인한다(정규식 grep 방식, `run_ci_sanity_gate.py`의 기존 체커들과 같은 스타일).
- 실패 시 `E_OBSERVER_CLIENT_MUTATION_LEAK`로 실패한다.
- 이 체커를 `tests/run_ci_sanity_gate.py`의 `core_lang` 프로파일에 스텝으로 등록한다(기존 스텝 등록 패턴을 그대로 따를 것 — 다른 체커가 어떻게 등록됐는지 먼저 확인).

### 3. 향후 사용처 연결(현재는 필요 없음, 확인만)

Q29 확인 결과 `seulgi_proposal_ui.js`/`seulgi_replay_safe_workflow.js`는 현재 WASM state를 전혀 읽지 않는다 — 그러니 이번 브리프에서 이 두 파일을 수정할 필요는 없다. 새 클라이언트가 존재한다는 것 자체가 "앞으로 관찰자가 필요하면 이걸 쓴다"는 계약이 되는 것으로 충분하다. **기존 드라이버 파일(`screens/run.js`, `wasm_page_common.js`, `playground.js`)은 건드리지 않는다** — 이들은 정당하게 mutation이 필요한 드라이버다(Q29 분류 기준).

## 검증

- `python tests/run_wasm_state_observer_client_capability_check.py` PASS
- `python tests/run_ci_sanity_gate.py --profile core_lang` PASS(회귀 없음, 새 스텝 포함)
- Playwright 제품 로드(기존 방식과 동일)로 콘솔 에러 0 확인 — 새 파일이 아무 데도 import 안 됐어도 이 자체로는 문제 없음(다음 소비자가 생길 때 연결).
- `git diff --check` PASS

## 수용 기준

- [ ] `runtime/wasm_state_observer_client.js` 신설, mutation 함수명 0건
- [ ] 회귀 가드 체커 신설 + `core_lang` 프로파일 등록
- [ ] 기존 드라이버 파일 수정 없음
- [ ] `wasm_canon_runtime.js`와 구조/네이밍 일관성 유지(리뷰 시 비교 확인)

## 금지 사항

Rust/WASM 빌드 변경 없음(이번 브리프는 순수 JS 신규 파일 + 테스트). 기존 드라이버 파일 로직 변경 없음. main 직접 커밋 금지, `codex/queue-20260706` 브랜치에 커밋.

## 보고 형식

이 파일 하단 `## 실행 보고`: 신규 파일 경로, 노출 함수 목록, 회귀 가드 통과 여부, sanity gate 결과.

## 실행 보고

- 실행일: 2026-07-06
- 브랜치: `codex/queue-20260706`
- 신규 파일: `solutions/seamgrim_ui_mvp/ui/runtime/wasm_state_observer_client.js`
- 노출 함수: `getStateHash(target)`, `getStateParsed(target)`, `createWasmStateObserverClient(target)`.
- 회귀 가드: `tests/run_wasm_state_observer_client_capability_check.py` 신규 추가, `tests/run_ci_sanity_gate.py`의 `core_lang` 프로파일에 `wasm_state_observer_client_capability_check` 등록.
- 금지 토큰 확인: `rg -n "set_param|setParam|reset|step_one|stepOne|run_ticks|runTicks|restore_state|restoreState|inject_ai_action|injectAiAction" solutions/seamgrim_ui_mvp/ui/runtime/wasm_state_observer_client.js` 결과 0건.
- 검증:
  - `python tests/run_wasm_state_observer_client_capability_check.py` PASS.
  - `python tests/run_ci_sanity_gate.py --profile core_lang --only-step wasm_state_observer_client_capability_check` PASS.
  - `node tests/seulgi_proposal_ui_runner.mjs` PASS.
  - `node tests/seulgi_replay_safe_workflow_runner.mjs` PASS.
  - `python tests/run_ci_sanity_gate.py --profile core_lang` PASS.
  - `git diff --check` PASS.
- 비고: `core_lang` 실행 중 갱신된 기존 open 로그 2개는 Q30 산출물이 아니므로 원복했다. Rust/WASM 빌드/기존 드라이버 파일 변경 없음.
