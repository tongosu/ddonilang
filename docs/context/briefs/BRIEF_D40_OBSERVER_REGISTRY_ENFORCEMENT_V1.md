# BRIEF: D40 완성 — 관찰자 레지스트리 전면 강제

> 작성: Claude (2026-07-06) / 실행: Codex / 리뷰: Claude
> 근거: `docs/context/reports/OBSERVER_MUTATOR_BOUNDARY_SURVEY_V1.md`(Q29), Q30(`wasm_state_observer_client.js` + 회귀 가드)
> 성격: **실제 구현.** 설계는 이 브리프로 확정됨.

## 배경

Q30은 관찰자 전용 클라이언트 파일 1개(`wasm_state_observer_client.js`)를 만들고, "그 파일 안에 mutation 토큰이 없는지"만 검사하는 가드를 추가했다. 하지만 이건 **그 파일 하나만** 지킨다 — 만약 나중에 누군가 `seulgi_proposal_ui.js`에 실수로 `wasm_ddn_wrapper.js`를 import하면, 지금 체계는 그걸 못 잡는다. D40을 진짜로 "계약"으로 만들려면 **관찰자로 등록된 모든 파일**을 검사해야 한다.

Q29가 이미 관찰자로 분류해둔 파일들이 있다:
- `solutions/seamgrim_ui_mvp/ui/seulgi_proposal_ui.js`
- `solutions/seamgrim_ui_mvp/ui/seulgi_replay_safe_workflow.js`
- `solutions/seamgrim_ui_mvp/ui/runtime/wasm_canon_runtime.js`
- `solutions/seamgrim_ui_mvp/ui/runtime/lesson_canon_runtime.js`
- `solutions/seamgrim_ui_mvp/ui/block_editor/ddn_block_codec.js`
- `solutions/seamgrim_ui_mvp/ui/runtime/wasm_state_observer_client.js`(Q30 신설)

## 작업

### 1. 관찰자 레지스트리 파일 신설

`solutions/seamgrim_ui_mvp/ui/OBSERVER_REGISTRY.json`(신규):
```json
{
  "observers": [
    "seulgi_proposal_ui.js",
    "seulgi_replay_safe_workflow.js",
    "runtime/wasm_canon_runtime.js",
    "runtime/lesson_canon_runtime.js",
    "block_editor/ddn_block_codec.js",
    "runtime/wasm_state_observer_client.js"
  ]
}
```
(경로는 `solutions/seamgrim_ui_mvp/ui/` 기준 상대경로.)

### 2. Q30 체커를 레지스트리 기반으로 확장

`tests/run_wasm_state_observer_client_capability_check.py`를 확장(또는 새 파일 `tests/run_observer_registry_capability_check.py`를 만들고 기존 체커는 유지 — 어느 쪽이든 좋으나 중복 로직은 피할 것):

- `OBSERVER_REGISTRY.json`의 각 파일을 읽어, 전부 `set_param|setParam|reset|step_one|stepOne|run_ticks|runTicks|restore_state|restoreState|inject_ai_action|injectAiAction` 패턴이 없음을 확인(Q30과 동일한 금지 패턴).
- 레지스트리에 등록된 파일이 실제로 존재하지 않으면 실패(`E_OBSERVER_REGISTRY_FILE_MISSING`).
- 위반 발견 시 `E_OBSERVER_REGISTRY_MUTATION_LEAK`로 실패, 파일:행 명시.
- 이 체커를 `run_ci_sanity_gate.py`의 `core_lang` 프로파일에 등록(Q30이 등록한 자리 근처).

### 3. 등록 안내 주석

`OBSERVER_REGISTRY.json` 옆에 `OBSERVER_REGISTRY.md`(짧게, 10줄 이내)를 만들어 "새 관찰자 파일을 추가하려면 이 목록에 등록하고, mutation 함수를 import하면 안 된다"는 규칙만 적어라. 새 설계 추가하지 마라.

## 검증

- `python tests/run_ci_sanity_gate.py --profile core_lang` PASS(회귀 없음, 새 스텝 포함)
- 레지스트리에 등록된 6개 파일 전부 검사 대상에 포함됐는지 확인(로그로 남겨라)
- 의도적으로 레지스트리에 없는 파일(예: `screens/run.js`, 드라이버)은 검사 대상이 아님을 확인(오탐 없음)

## 수용 기준

- [ ] `OBSERVER_REGISTRY.json` 신설, Q29가 분류한 6개 관찰자 파일 전부 포함
- [ ] 체커가 레지스트리 전체를 검사(파일 1개가 아니라)
- [ ] `core_lang` 프로파일 등록
- [ ] 드라이버 파일(`screens/run.js` 등) 수정 없음

## 금지 사항

Rust/WASM 빌드 변경 없음. 드라이버 파일 로직 변경 없음. Q29가 "드라이버"로 분류한 파일을 레지스트리에 넣지 마라(정당하게 mutation이 필요한 파일들이다). main 직접 커밋 금지, `codex/queue-20260706` 브랜치에 커밋.

## 보고 형식

이 파일 하단 `## 실행 보고`: 레지스트리 내용, 체커 확장 방식, 검증 결과.
