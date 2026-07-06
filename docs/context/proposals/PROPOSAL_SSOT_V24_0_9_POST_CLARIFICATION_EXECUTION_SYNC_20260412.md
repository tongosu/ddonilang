
SSOT_ALL 버전/해시: **v24.0.8 / `9f345191c9157bbe2af36eddcbc338c3e7e432618c6ce3209e4661d98f288856`**
workspace_bundle 버전/해시: **codex20260329.zip / `c20057d90570a8c4db9dd544dd3a8c2a0971d4fb3e27fac8451fbeb4e70bb75c`**

# PROPOSAL_SSOT_V24_0_9_POST_CLARIFICATION_EXECUTION_SYNC_20260412

날짜: 2026-04-12
상태: proposal draft
분류: SSOT_RULE / PLANS / OPEN_ISSUES / DOCS_SYNC / NO_NEW_KEYWORD / NO_RUNTIME_SEMANTICS_CHANGE

## 0. 목적

`v24.0.8`이 이미 잠근 clarification wording 을 바꾸지 않고,
그 다음 실행 라운드에서 **무엇을 immediate evidence slice로 읽을지** 를 SSOT source-family 문장으로 더 분명히 고정한다.

핵심은 아래 네 줄이다.

1. `lang_pragmatism_pack_v1` 는 계속 pre-v25 **G1 immediate golden target** 이다.
2. `lang_flow_hook_interaction_v1` 는 새 surface 가 아니라 **supporting pack/checker** 다.
3. `채비 {}` first-line closure 는 **lesson canonical / T5 lint / checker** 다.
4. 교재는 stale ref 를 고치고 docs-first `04_전문.md` shell 을 둔다.

## 1. 변경 대상 파일 경로

### source-of-truth
- `docs/ssot/ssot/README_v24.0.9.md`
- `docs/ssot/ssot/SSOT_INDEX_v24.0.9.md`
- `docs/ssot/ssot/SSOT_MASTER_v24.0.9.md`
- `docs/ssot/ssot/SSOT_DECISIONS_v24.0.9.md`
- `docs/ssot/ssot/SSOT_OPEN_ISSUES_v24.0.9.md`
- `docs/ssot/ssot/SSOT_PLANS_v24.0.9.md`
- `docs/ssot/ssot/SSOT_PENDING_v24.0.9.md`
- `docs/ssot/ssot/SSOT_TOOLCHAIN_v24.0.9.md`
- `docs/ssot/walks/ddonirang_grammar_textbook/README.md`
- `docs/ssot/walks/ddonirang_grammar_textbook/04_전문.md`

### proposal / handoff
- `docs/context/proposals/CODEX_TASK_V24_0_8_POST_CLARIFICATION_EVIDENCE_BUNDLE_20260412.md`
- `docs/context/proposals/READY_PATCH_V24_0_9_POST_CLARIFICATION_EXECUTION_SYNC_20260412.md`

## 2. 반영 요지

### 2-1. current-line hold
- `v24.0.8`까지 landed 한 clarification wording 은 그대로 유지한다.
- 새 semantics 승격이나 new canonical pack family 추가는 하지 않는다.

### 2-2. pragmatism G1 유지
- `lang_pragmatism_pack_v1` 를 계속 pre-v25 G1 immediate golden target 으로 명시한다.
- docs-first / runner_fill / golden_closed 구분을 흐리지 않는다.

### 2-3. flow-hook supporting pack 명시
- `lang_flow_hook_interaction_v1` 를 supporting pack/checker immediate target 으로 명시한다.
- 이것은 `ordinary assignment -> 흐름씨 fixed-point -> tail-phase hook` 관계의 **contract/expected 구조를 문서로 고정하는 작업**(docs-first skeleton)이지, 실제 runtime 관계를 evidence로 닫는 작업이 아니다. 실제 제품 실행 검증은 GOAL-B(`FLOW_HOOK_PHASE_SEPARATION_VERIFICATION_V1.md`, 2026-07-06) 참고 — 현재 `<<-`는 제품 파서(`tools/teul-cli/src/lang/lexer.rs`)에 없어 4개 케이스 전부 파싱 단계에서 실패한다.

### 2-4. 채비 lint-first closure 명시
- `채비 {}` top-level-only 규칙은 lesson canonical / T5 lint / checker 에 먼저 반영한다고 적는다.
- optional `lang_chaebi_scope_v1` support pack 가능성은 열어 두되, current-line canonical MUST-close ID 확장으로 읽지 않게 적는다.

### 2-5. textbook current-line sync
- textbook README 의 stale `참조 SSOT: v24.0.7` 를 current line 에 맞춘다.
- docs-first `04_전문.md` shell 을 추가한다.
- `AGE != exposure_level` 설명축이 전문 계층까지 이어지도록 정리한다.

## 3. DoD

- `lang_pragmatism_pack_v1` status wording mismatch 0건
- `lang_flow_hook_interaction_v1` supporting pack/checker PASS
- `채비` canonical/T5 lint/checker PASS
- stale textbook ref 0건
- `04_전문.md` landed
- advanced shell 이 actual closure claim 으로 읽히는 문장 0건

## 4. 비목표

- 새 core keyword
- 새 runtime semantics
- `lang_chaebi_scope_v1` canonical MUST-close 승격
- auth/save/project/package actual platform closure claim
- 슬기/배움틀/proof runtime completion claim
