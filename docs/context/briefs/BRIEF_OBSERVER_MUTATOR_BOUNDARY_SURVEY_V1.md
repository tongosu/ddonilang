# BRIEF: 관찰자/변경자 경계 실측 조사 (D40/D41 수리 설계 근거)

> 작성: Claude (2026-07-06) / 실행: Codex / 리뷰: Claude
> 근거: `docs/context/reports/ECOSYSTEM_CONTRACT_VERIFICATION_V1.md`(Q28)
> 성격: **진단/조사 전용. 코드 수정 없음.** Q28이 찾은 D40/D41 미착륙 지점을 실제로 어떻게 고칠지는 Claude가 설계한다 — 이 브리프는 그 설계에 필요한 실측 지도만 만든다.

## 배경

Q28이 두 가지를 확인했다:
- D40: `tool/src/wasm_api.rs`가 노출하는 `set_param/reset/step_one/run_ticks/restore_state/inject_ai_action`(mutation) 함수와, 순수 관찰 목적 UI 모듈이 같은 WASM 표면에 섞여 있고 capability 분리가 없다.
- D41: 입력이 샘 경계는 통과하지만 `입력원천` 6값(`사람/슬기/밖일/일정/이어전달/펼침실행`) enum/분류기가 제품 코드에 없다.

이 두 개를 실제로 어떻게 고칠지(관찰자 전용 API를 어떻게 분리할지, 6원천을 어떤 타입으로 만들지)는 Claude가 설계해야 하는 부분이라 지금 실행 브리프를 줄 수 없다. 대신 설계에 필요한 실측 지도를 먼저 만든다.

## 작업

### 1. D40 근거 — 관찰자/변경자 호출부 지도

`solutions/seamgrim_ui_mvp/ui/` 전체에서 `tool/src/wasm_api.rs`가 노출하는 함수(`set_param*`, `reset`, `step_one*`, `run_ticks`, `restore_state`, `inject_ai_action`, 그리고 순수 읽기 함수들 — 예: 관찰/상태조회 계열 export)를 실제로 호출하는 모든 지점을 찾아라.

- 호출부를 파일별로 분류: (a) RunScreen처럼 시뮬레이션을 실제로 구동해야 하는 "드라이버"(mutation 필요가 정당함) vs (b) `seulgi_proposal_ui.js`류처럼 관찰/제안만 하면 되는 "관찰자"(mutation이 필요 없어야 함).
- (b)로 분류된 모듈 중 실제로 mutation 함수를 import하거나 호출하는 게 있는지 확인(현재는 없다고 Q28이 봤지만, 전체 UI 모듈 기준으로 다시 전수 확인).
- 모든 관찰자 모듈이 실제로 필요로 하는 최소 읽기 함수 집합을 표로 만들어라(이게 나중에 "관찰자 전용 API"의 실제 표면이 된다).

### 2. D41 근거 — 입력원천별 실제 코드 경로 지도

현재 구현된 입력 경로들(키보드, sam-live, replay, net event, 슬기 주입/`ai_injections`, 일정/schedule 관련이 있다면)이 각각:
- 어느 파일/함수에서 발생하는지
- `InputSnapshot`의 어느 필드로 들어가는지
- 현재 코드에 그 입력을 구분하는 어떤 이름(변수명, 상수, 주석)이라도 쓰이고 있는지(6원천 이름과 매핑될 실마리)

를 표로 정리하라. `이어전달`(relay)과 `밖일`(external task result)에 해당하는 실제 구현이 있는지도 확인하라(Q28은 이 둘을 명시적으로 언급하지 않았다 — 존재 여부 자체를 새로 확인).

## 검증 방법

- 정적 분석(코드 읽기 + `rg`)만 수행한다. 코드 수정, golden 갱신, pack 생성 전혀 없음.

## 산출물

`docs/context/reports/OBSERVER_MUTATOR_BOUNDARY_SURVEY_V1.md`:
- 표1(D40): 호출부 파일 | 호출 함수 | 드라이버/관찰자 분류 | 근거
- 표2(D41): 입력원천 후보 | 발생 파일:함수 | InputSnapshot 필드 | 현재 코드상 구분 방법(있으면)

## 수용 기준

- [ ] 표1이 `solutions/seamgrim_ui_mvp/ui/` 전체 모듈을 빠짐없이 훑었음(어떻게 훑었는지 방법 명시 — 예: 전체 `.js` 파일 목록 대비 커버리지)
- [ ] 표2가 6원천 각각에 대해 "구현 있음(파일:함수)" 또는 "구현 없음 확인" 중 하나로 명확히 판정
- [ ] 코드/golden/pack 변경 없음
- [ ] `codex/queue-20260706` 브랜치에 커밋 1개

## 금지 사항

코드 수정 없음. 발견한 문제를 그 자리에서 고치지 마라 — 지도만 만들어라. main 직접 커밋 금지.

## 보고 형식

이 파일 하단 `## 실행 보고`: 표1/표2 커버리지 요약, 산출물 경로.
