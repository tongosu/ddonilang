# BRIEF: 자-3/자-5 UI 배선 (고아 모듈 → 실제 마운트)

> 작성: Claude (2026-07-06) / 실행: Codex / 리뷰: Claude
> 근거: `GANADA_MATRIX_CORRECTED_20260706.md` — 자줄기 백엔드(자-1,2,4)는 진짜닫힘, 자-3/5는 화면 진입점 부재로 FAIL.
> 성격: 이미 존재하는 로직(`seulgi_proposal_ui.js`, `seulgi_replay_safe_workflow.js`, 각 226줄)을 기존 UI 관례에 맞춰 마운트하는 통합 작업. 새 기능 설계 아님 — 기존 패턴을 따를 것.

## 확인된 현재 상태

- `solutions/seamgrim_ui_mvp/ui/seulgi_proposal_ui.js`, `seulgi_replay_safe_workflow.js` 둘 다 실재하나 `index.html`/`app.js` 어디서도 참조되지 않는 고아 모듈.
- 실패 체커가 요구하는 것:
  - `tests/run_roadmap_v2_ja3_seulgi_proposal_ui_check.py`: `index.html`에 `id="seulgi-proposal-ui"` + `data-seulgi-proposal-ui`, `styles.css`에 `.seulgi-proposal-ui`
  - `tests/run_roadmap_v2_ja5_replay_safe_ai_workflow_check.py`: `index.html`에 `id="seulgi-replay-safe-workflow"` + `data-seulgi-replay-safe-workflow`, `styles.css`에 `.seulgi-replay-safe-workflow`

## 작업

1. **기존 패턴 확인 먼저**: 이미 잘 마운트된 비슷한 기능(예: 다른 `STUDIO_*_EXPORT_ACTION` 계열이나 Run 화면 Mirror 탭 패널들)이 `index.html`/`app.js`/`styles.css`에서 어떤 구조로 마운트되는지 확인하라(컨테이너 element, data 속성, JS import/초기화 호출 방식). **그 패턴을 그대로 따라라 — 새 마운트 방식을 발명하지 마라.**
2. `index.html`에 두 모듈을 위한 컨테이너 element를 (1)에서 확인한 패턴대로 추가한다(`id="seulgi-proposal-ui"`/`data-seulgi-proposal-ui`, `id="seulgi-replay-safe-workflow"`/`data-seulgi-replay-safe-workflow`).
3. `app.js`에서 두 JS 모듈을 import하고, 기존 패턴대로 초기화 호출을 추가한다.
4. `styles.css`에 `.seulgi-proposal-ui`, `.seulgi-replay-safe-workflow` 클래스를 추가한다(기존 유사 패널 스타일 관례를 따를 것).
5. 두 JS 모듈(`seulgi_proposal_ui.js`, `seulgi_replay_safe_workflow.js`) 자체의 로직은 이미 완성되어 있다고 가정하고 **수정하지 않는다** — 마운트 배선만 한다. 단, 마운트 후 실행 중 오류가 나면 그 오류 내용을 보고하고, 로직 수정이 필요해 보이면 실행하지 말고 진단만 남겨라(범위 판단은 Claude가 함).

## 검증

- `python tests/run_roadmap_v2_ja3_seulgi_proposal_ui_check.py` PASS
- `python tests/run_roadmap_v2_ja5_replay_safe_ai_workflow_check.py` PASS
- Playwright(이미 프로젝트에 있음)로 `index.html`을 헤드리스 로드해 두 패널이 실제로 렌더되는지, 콘솔 에러가 없는지 확인(Q16과 같은 방법)
- `python tests/run_ci_sanity_gate.py --profile core_lang` PASS(회귀 없음)

## 수용 기준

- [ ] 두 체커 PASS
- [ ] 브라우저 실행 확인(콘솔 에러 0, 두 패널 DOM에 실제 존재)
- [ ] 기존 마운트 패턴을 따랐다는 근거(어떤 기존 기능을 참고했는지) 보고에 명시
- [ ] `seulgi_proposal_ui.js`/`seulgi_replay_safe_workflow.js` 내부 로직 수정 없음(배선만)
- [ ] core_lang sanity gate 회귀 없음

## 금지 사항

새 마운트 방식 발명 / JS 모듈 내부 로직 수정 / 범위 밖 UI 변경 / main 커밋. `codex/queue-20260706` 브랜치에 커밋 1개(또는 index.html/app.js/styles.css 각각 나누어도 무방).

## 보고 형식

이 파일 하단 `## 실행 보고`: 참고한 기존 패턴, 변경 파일별 diff 요약, 검증 결과.

## 실행 보고

- 실행일: 2026-07-06
- 참고한 기존 패턴: `solutions/seamgrim_ui_mvp/ui/index.html`의 Run inspector `run-classroom-report-export`/`run-local-package-export`/`run-student-result-review` 정적 컨테이너 패턴, `solutions/seamgrim_ui_mvp/ui/app.js`의 제품 화면 초기화 흐름(`browseScreen.init()`/`editorScreen.init()`/`runScreen.init()` 뒤 초기화), `solutions/seamgrim_ui_mvp/ui/styles.css`의 Run export 패널 스타일 배치. `window.__SEULGI_*` 계측 payload는 기존 `dev_surfaces.js`의 `setWindowPayload`/`renderSurface` 방식과 같은 키를 사용했다.
- 변경 요약:
  - `solutions/seamgrim_ui_mvp/ui/index.html`: `run-local-package-export` 패널 안에 `id="seulgi-proposal-ui" data-seulgi-proposal-ui`, `id="seulgi-replay-safe-workflow" data-seulgi-replay-safe-workflow` 컨테이너 추가.
  - `solutions/seamgrim_ui_mvp/ui/app.js`: `seulgi_proposal_ui.js`, `seulgi_replay_safe_workflow.js`를 import하고 제품 로드에서 payload build/publish/render 초기화 추가. `?devSurfaces=1` 경로는 기존 dev surface가 담당하도록 분리해 중복 버튼 렌더를 피했다.
  - `solutions/seamgrim_ui_mvp/ui/styles.css`: `.seulgi-proposal-ui`, `.seulgi-replay-safe-workflow`와 두 모듈이 공통으로 쓰는 `.question-card-*` 제품 스타일 추가.
  - `seulgi_proposal_ui.js`, `seulgi_replay_safe_workflow.js` 내부 로직 수정 없음.
- 검증 결과:
  - `python tests/run_roadmap_v2_ja3_seulgi_proposal_ui_check.py` PASS.
  - `python tests/run_roadmap_v2_ja5_replay_safe_ai_workflow_check.py` PASS.
  - Playwright 제품 로드(`index.html`, `devSurfaces` 미사용) PASS: `consoleErrors=0`, `devRoot=false`, 두 패널 status ready, 각 버튼 5개, 전역 schema 확인.
  - `python tests/run_ci_sanity_gate.py --profile core_lang` PASS.
  - `git diff --check` PASS.
