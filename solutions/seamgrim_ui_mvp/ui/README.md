# Seamgrim UI MVP

정적 HTML/CSS/JS로 구성된 셈그림 UI MVP 스켈레톤입니다.

> 2026-02-23 재건 노트  
> UI는 `index.html` 단일 진입점의 `탐색 -> DDN 편집 -> 실행` 3화면 구조로 재작성되었습니다.  
> `playground.html`, `wasm_smoke.html` 및 비교/세션 오버레이 모듈은 제거되었습니다.
> 아래 문서의 기존 AGE3 practical 상세는 일부 레거시 설명을 포함할 수 있으며, 최신 동선은 `index.html` 기준입니다.

## 사용
- 실행 서버 시작(단일 진입점): `python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py`
- 배포 바인딩 예시: `python solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py --host 0.0.0.0 --port 8787`
- Docker 시작: `docker compose -f solutions/seamgrim_ui_mvp/deploy/docker-compose.yml up --build -d`
- 브라우저에서 `http://localhost:8787/` 접속
- UI는 교과/샘플 경로를 `/...`와 `/solutions/seamgrim_ui_mvp/...` 순서로 자동 폴백합니다(실행 서버/정적 루트 차이 완화).
- 연합 검색은 기본적으로 `/api/lessons/inventory`만 조회합니다(불필요한 `build/reports/*.json` 404 방지).
- 파일 인벤토리 폴백이 필요하면 `index.html` 로드 전에 `window.SEAMGRIM_ENABLE_FEDERATED_FILE_FALLBACK = true;` 와 `window.SEAMGRIM_FEDERATED_FILE_CANDIDATES = ["build/reports/seamgrim_lesson_inventory.json"];` 를 함께 지정합니다.
- 실행 화면은 기본적으로 sim-core 정책(`시뮬 중심 + 축 선택 + 교과 오버레이`)으로 동작합니다. 해제가 필요하면 `window.SEAMGRIM_SIM_CORE_POLICY = false`를 설정합니다.
- 현재 돋보기는 그래프 전용입니다(표/설명 탭 제거). 설명은 보개 오버레이(`#overlay-description`)로만 제공합니다.
- x/y축 선택 UI는 항상 활성화되어 있어 사용자가 바로 바꿀 수 있습니다.
- 보개 범위 입력/프리셋 UI는 제거되었고, 보개 뷰 조작은 키보드 단축키 경로만 유지합니다.
- 고급 Playground 바로 진입: `http://localhost:8787/?advanced=playground`
- 고급 Smoke 바로 진입: `http://localhost:8787/?advanced=smoke`
- AGE3 교과 스키마 게이트: `python tests/run_seamgrim_lesson_schema_gate.py`
- AGE3 승격 완료까지 강제: `python tests/run_seamgrim_lesson_schema_gate.py --require-promoted`
- AGE3 UI 게이트(R3-A/B/C 최소 기능): `python tests/run_seamgrim_ui_age3_gate.py`
- sim-core 계약 게이트: `python tests/run_seamgrim_sim_core_contract_gate.py`
- 5분 검증 체크리스트(사람 읽기형): `python tests/run_seamgrim_5min_checklist.py --base-url http://127.0.0.1:8787 --skip-seed-cli`
- CI 게이트에서 체크리스트 포함 실행: `python tests/run_seamgrim_ci_gate.py --browse-selection-strict --with-5min-checklist --checklist-skip-seed-cli --checklist-skip-ui-common`
- AGE3 완료 리포트 생성: `python tests/run_age3_close.py --run-seamgrim --report-out build/reports/age3_close_report.detjson`
- AGE3 완료 상태 JSON 생성: `python tools/scripts/render_age3_close_status.py build/reports/age3_close_report.detjson --out build/reports/age3_close_status.detjson --fail-on-bad`
- AGE3 완료 상태 1줄 텍스트 생성: `python tools/scripts/render_age3_close_status_line.py build/reports/age3_close_status.detjson --out build/reports/age3_close_status_line.txt --fail-on-bad`
  - 형식(v1): `schema="..." status=pass|fail overall_ok=0|1 criteria_total=... criteria_failed_count=... ...`
- AGE3 완료 배지 JSON 생성: `python tools/scripts/render_age3_close_badge.py build/reports/age3_close_status.detjson --status-line build/reports/age3_close_status_line.txt --out build/reports/age3_close_badge.detjson --fail-on-bad`
- AGE3 완료 상태 1줄 검증: `python tests/run_age3_status_line_check.py --status-line build/reports/age3_close_status_line.txt --status-json build/reports/age3_close_status.detjson --require-pass`
- AGE3 완료 상태 1줄 파싱(대시보드용): `python tools/scripts/parse_age3_close_status_line.py --status-line build/reports/age3_close_status_line.txt --status-json build/reports/age3_close_status.detjson --fail-on-invalid`
- CI 집계 상태 1줄 생성: `python tools/scripts/render_ci_aggregate_status_line.py build/reports/ci_aggregate_report.detjson --out build/reports/ci_aggregate_status_line.txt --fail-on-bad`
- CI 집계 상태 1줄 파싱: `python tools/scripts/parse_ci_aggregate_status_line.py --status-line build/reports/ci_aggregate_status_line.txt --aggregate-report build/reports/ci_aggregate_report.detjson --fail-on-invalid`
  - 파싱 JSON 저장: `python tools/scripts/parse_ci_aggregate_status_line.py --status-line build/reports/ci_aggregate_status_line.txt --aggregate-report build/reports/ci_aggregate_report.detjson --json-out build/reports/ci_aggregate_status_line_parse.detjson --fail-on-invalid`
- CI 집계 상태 1줄 검증: `python tests/run_ci_aggregate_status_line_check.py --status-line build/reports/ci_aggregate_status_line.txt --aggregate-report build/reports/ci_aggregate_report.detjson --require-pass`
- CI 최종 상태 1줄 생성: `python tools/scripts/render_ci_gate_final_status_line.py --aggregate-status-parse build/reports/ci_aggregate_status_line_parse.detjson --gate-index build/reports/ci_gate_report_index.detjson --out build/reports/ci_gate_final_status_line.txt --fail-on-bad`
- CI 최종 상태 1줄 검증: `python tests/run_ci_gate_final_status_line_check.py --status-line build/reports/ci_gate_final_status_line.txt --aggregate-status-parse build/reports/ci_aggregate_status_line_parse.detjson --gate-index build/reports/ci_gate_report_index.detjson --require-pass`
- CI 최종 상태 1줄 파싱: `python tools/scripts/parse_ci_gate_final_status_line.py --status-line build/reports/ci_gate_final_status_line.txt --json-out build/reports/ci_gate_final_status_line_parse.detjson --compact-out build/reports/ci_gate_summary_line.txt --fail-on-invalid`
- CI 최종 요약 1줄 검증: `python tests/run_ci_gate_summary_line_check.py --summary-line build/reports/ci_gate_summary_line.txt --ci-gate-result-parse build/reports/ci_gate_result_parse.detjson --require-pass`
- CI 요약 리포트 핵심 키 검증: `python tests/run_ci_gate_summary_report_check.py --summary build/reports/ci_gate_summary.txt --index build/reports/ci_gate_report_index.detjson --require-pass`
- CI 요약 리포트 실패 상세 블록 검증: `python tests/run_ci_gate_failure_summary_check.py --summary build/reports/ci_gate_summary.txt --index build/reports/ci_gate_report_index.detjson --require-pass`
- CI 게이트 결과 JSON 생성: `python tools/scripts/render_ci_gate_result.py --final-status-parse build/reports/ci_gate_final_status_line_parse.detjson --summary-line build/reports/ci_gate_summary_line.txt --gate-index build/reports/ci_gate_report_index.detjson --out build/reports/ci_gate_result.detjson --fail-on-bad`
- CI 게이트 결과 JSON 검증: `python tests/run_ci_gate_result_check.py --result build/reports/ci_gate_result.detjson --final-status-parse build/reports/ci_gate_final_status_line_parse.detjson --summary-line build/reports/ci_gate_summary_line.txt --require-pass`
- CI 게이트 결과 파싱: `python tools/scripts/parse_ci_gate_result.py --result build/reports/ci_gate_result.detjson --json-out build/reports/ci_gate_result_parse.detjson --compact-out build/reports/ci_gate_result_line.txt --fail-on-invalid --fail-on-fail`
- CI 게이트 배지 JSON 생성: `python tools/scripts/render_ci_gate_badge.py build/reports/ci_gate_result.detjson --out build/reports/ci_gate_badge.detjson --fail-on-bad`
- CI 게이트 배지 검증: `python tests/run_ci_gate_badge_check.py --badge build/reports/ci_gate_badge.detjson --result build/reports/ci_gate_result.detjson --require-pass`
- CI 최종 산출 정합 검증: `python tests/run_ci_gate_outputs_consistency_check.py --summary-line build/reports/ci_gate_summary_line.txt --result build/reports/ci_gate_result.detjson --result-parse build/reports/ci_gate_result_parse.detjson --badge build/reports/ci_gate_badge.detjson --final-status-parse build/reports/ci_gate_final_status_line_parse.detjson --require-pass`
- CI 로그용 최종 1줄/아티팩트 요약 출력: `python tools/scripts/emit_ci_final_line.py --report-dir build/reports --print-artifacts --print-failure-digest 6 --print-failure-tail-lines 20 --failure-brief-out build/reports/__PREFIX__.ci_fail_brief.txt --triage-json-out build/reports/__PREFIX__.ci_fail_triage.detjson --require-final-line`
- CI 실패요약 블록 교차검증까지 엄격 실행: `python tools/scripts/emit_ci_final_line.py --report-dir build/reports --print-artifacts --print-failure-digest 6 --print-failure-tail-lines 20 --fail-on-summary-verify-error --failure-brief-out build/reports/__PREFIX__.ci_fail_brief.txt --triage-json-out build/reports/__PREFIX__.ci_fail_triage.detjson --require-final-line`
- CI 통합 게이트 실행(성공 단계 로그 축약): `python tests/run_ci_aggregate_gate.py --skip-core-tests --fast-fail --report-prefix local --quiet-success-logs --compact-step-logs --step-log-dir build/reports --step-log-failed-only`
- CI 통합 게이트 전체 PASS 요약(상세) 출력: `python tests/run_ci_aggregate_gate.py --skip-core-tests --fast-fail --report-prefix local --quiet-success-logs --compact-step-logs --step-log-dir build/reports --step-log-failed-only --full-pass-summary`
- CI 최종 라인 emitter 셀프체크: `python tests/run_ci_final_line_emitter_check.py`
- CI 파이프라인 플래그 드리프트 체크: `python tests/run_ci_pipeline_emit_flags_check.py`
- CI emitter 산출물(brief/triage) 정합 체크: `python tests/run_ci_emit_artifacts_check.py --report-dir build/reports --require-brief --require-triage`
- CI emitter 산출물 체크 셀프테스트: `python tests/run_ci_emit_artifacts_check_selftest.py`
- 단계별 로그 파일: `<prefix>.ci_gate_step_<step>.(stdout|stderr).txt` (index의 `steps[].*_log_path` 참조, `--step-log-failed-only`이면 실패 단계만 저장)
- 실패 요약 1줄 파일: `<prefix>.ci_fail_brief.txt` (`--failure-brief-out build/reports/__PREFIX__.ci_fail_brief.txt`)
- 실패 트리아지 JSON: `<prefix>.ci_fail_triage.detjson` (`--triage-json-out build/reports/__PREFIX__.ci_fail_triage.detjson`)
- CI에서는 기본 게이트(`--require-promoted` 미사용)로 preview 품질/상태 인덱스/preview 누락을 검증합니다.

## 기능
- 2분할 레이아웃: 좌 작업 패널 + 우 뷰(View Dock)
- 작업 모드 분리: `기본`(교과/DDN/실행) + `고급`(도구/인스펙터/내보내기)
- 교과 탭: lesson 선택 → meta.toml + lesson.ddn 로드, required_views 반영
  - `DDN 상태` 필터(`AGE3 목표형/전환중/레거시/혼합/미확인`) 지원
  - `AGE3 preview 우선 사용` 토글 지원 (`lesson.age3.preview.ddn` 우선 로드)
  - preview 자동실행이 실패하면 `lesson.ddn`으로 1회 자동 폴백하고 모드를 저장합니다.
- DDN 탭: 정본 DDN 실행/불러오기/내보내기 + 프리셋 관리
- View Dock: 표시 범위/줌/팬/그리드·축 토글
- 시간(보기) 컨트롤: t 커서/재생/정지/한칸, 오프라인 샘플링
- run manager: 실행 목록 + 오버레이 토글/solo/강조
- before/after 순차 비교(기준/변형 번갈아 재생)
- 저장물 3종: graph/snapshot/session 내보내기/불러오기
- 미디어 내보내기: WebM/GIF 녹화(그래프/2D/구조 캔버스)
- 입력원 레지스트리: DDN/수식/lesson 입력원 요약 + 세션 저장/복원
- 검증/인스펙터: DDN 메타, 해시/run_server_check, 스키마 요약
- 샘/거울 파일 로드(요약 카드)
- 보개 확장: 그래프/2D/표/글/구조 뷰 로드 및 표시

## 실용 모드(메인 UI)
- 메인 `index.html`은 실용 모드로 운영합니다.
- 기본 동선: `교과 선택 -> DDN 로드 -> WASM 자동 실행`
- 시작 시 마지막 선택 교과를 자동 복원하고, 없으면 우선순위 교과를 자동 선택해 바로 실행합니다.
- 교과 목록에서 practical 우선순위 최상단 후보는 `추천 1순위` 배지로 표시되며, 배지 클릭으로 즉시 실행할 수 있습니다.
- 교과 탭 상단 `추천 실행 (Alt+R)` 버튼으로도 동일하게 추천 교과 즉시 실행이 가능합니다.
- 실행/시간 탭의 `진자 데모 시작 (Alt+1)` 버튼으로 `physics_pendulum_seed_v1`를 즉시 실행할 수 있습니다.
- 진자 데모 시작/프리셋 적용 시 그래프 축(x=0..t_max, y=±theta 범위)과 2D 카메라가 고정 범위로 자동 설정됩니다.
- 실행/시간 탭의 `시나리오 다음 단계 (Alt+4)` 버튼으로 `기본 -> 줄 길이 증가 -> 중력 감소` 순서를 반복 적용할 수 있습니다.
- 실행/시간 탭의 `시나리오 자동재생 시작 (Alt+5)` 버튼으로 단계 시연을 자동 순환할 수 있으며, 간격(ms)을 조절할 수 있습니다.
- 실행/시간 탭의 `프리셋` 버튼(기본/줄 길이 증가/중력 감소)으로 진자 조종값을 즉시 바꿔 재실행할 수 있습니다.
- 단축키 `Alt+R`로 현재 기준 추천 교과를 즉시 실행할 수 있습니다. (입력 필드 포커스 중에는 동작하지 않음)
- 단축키 `Alt+1`로 진자 데모를 어디서든 즉시 시작할 수 있습니다. (입력 필드 포커스 중에는 동작하지 않음)
- 단축키 `Alt+2`/`Alt+3`로 진자 프리셋(줄 길이 증가/중력 감소)을 빠르게 적용할 수 있습니다.
- 단축키 `Alt+4`로 진자 시나리오 다음 단계를 빠르게 진행할 수 있습니다.
- 단축키 `Alt+5`로 진자 시나리오 자동재생 시작/정지를 전환할 수 있습니다.
- 단축키 `Ctrl+K`(macOS는 `Cmd+K`)로 교과 탭 검색창에 즉시 포커스할 수 있습니다.
- 단축키 `Shift+방향키`와 `Shift++/-`로 그래프 pan/zoom을 키보드로 조작할 수 있습니다.
- 단축키 `Ctrl+Shift+방향키`(macOS는 `Cmd+Shift+방향키`)와 `Ctrl+Shift++/-`(`Cmd+Shift++/-`)로 2D 보개 pan/zoom을 키보드로 조작할 수 있습니다.
- 교과 필터(학년/과목/검색/상태/템플릿/Rewrite)는 브라우저 로컬 저장소에 자동 저장되어 재접속 시 복원됩니다.
- 교과 탭 상단 `필터 초기화 (Esc)` 버튼 또는 `Esc` 단축키로 저장된 필터를 기본값으로 즉시 되돌릴 수 있습니다.
- `?` 또는 `F1` 단축키(입력 필드 외), `단축키 (?/F1)` 버튼으로 practical 단축키 도움말 팝오버를 열고 닫을 수 있습니다.
- 도움말 팝오버에는 최근 감지된 단축키와 감지 시각(HH:MM:SS)이 실시간으로 표시됩니다.
- 도움말 팝오버에는 최근 감지 단축키 히스토리 5개가 최신순으로 표시됩니다.
- 도움말 팝오버의 `초기화 (Shift+Esc)` 버튼 또는 `Shift+Esc` 단축키로 최근 감지 단축키/시각 표시를 `-`로 리셋할 수 있습니다.
- `Ctrl`/`Cmd` 수정키 입력이 반복되는데 조합 단축키가 감지되지 않으면 도움말 하단에 `Ctrl/Cmd 미감지` 경고 배지가 표시됩니다.
- `Ctrl/Cmd 미감지` 배지를 클릭하면 점검 가이드가 열리며 `Ctrl/Cmd+K`, `Ctrl/Cmd+Enter`, `Ctrl/Cmd+S`를 순서대로 테스트할 수 있습니다.
- 점검 가이드는 각 단축키 테스트 결과를 자동으로 `대기/성공/실패` 상태와 시각으로 표시합니다.
- 점검 가이드의 `전체 재점검` 버튼으로 3개 체크 상태를 `대기`로 일괄 초기화할 수 있습니다.
- 브라우저/OS 기본 단축키가 우선되는 환경에서는 일부 단축키가 동작하지 않을 수 있습니다(도움말 팝오버에 안내 표시).
- 단축키 도움말 팝오버의 열림/닫힘 상태도 로컬 저장소에 유지되어 재접속 시 복원됩니다.
- 실행 탭과 상단 `고급 메뉴`에서 `Playground`/`Smoke`로 진입할 수 있습니다.
- `playground.html`, `wasm_smoke.html` 레거시 파일은 제거되었고, 동작 진입점은 `index.html` 하나만 유지합니다.
- UI 전처리기(실행 화면 경로)에서 `보개 { ... }`/`모양 { ... }` 블록의 `선(...)`, `원(...)`, `점(...)`을 `space2d.shape` 출력으로 자동 변환합니다.
  - 현재 범위는 블록 내부가 위 3개 프리미티브로만 구성된 경우입니다.
  - 예제: `solutions/seamgrim_ui_mvp/samples/05_pendulum_bogae_block_ui.ddn`
  - 권장 패턴:
    - `채비`는 조절값 기본값/범위 선언만 담당
    - `(시작)할때`는 시뮬 상태(`t`, `theta`, `omega` 등) 초기화만 담당
    - 관찰 출력은 소스에서 `보임 { ... }`로 작성하고, 런타임이 내부적으로 표시 이벤트로 변환
  - `모양`은 "도형 선언 블록"입니다. 사용자는 `선/원/점`만 쓰고, `space2d.shape` 키를 직접 다루지 않아도 됩니다.
  - 즉, 교과/seed 원문에서는 `보여주기`를 직접 쓸 필요가 없습니다(`보임/모양` 사용).

## WASM 매핑
WASM patch 모드에서 `set_resource_fixed64`와 `set_resource_value`를 UI에 직접 반영할 수 있습니다.

### fixed64 매핑 허용 필드
아래 대상만 허용됩니다. (매핑 형식: `target=tag`)
- `graph.axis.x_min`
- `graph.axis.x_max`
- `graph.axis.y_min`
- `graph.axis.y_max`
- `graph.sample.x_min`
- `graph.sample.x_max`
- `graph.sample.step`
- `graph.view.auto`
- `graph.view.x_min`
- `graph.view.x_max`
- `graph.view.y_min`
- `graph.view.y_max`
- `graph.view.pan_x`
- `graph.view.pan_y`
- `graph.view.zoom`
- `space2d.view.auto`
- `space2d.view.x_min`
- `space2d.view.x_max`
- `space2d.view.y_min`
- `space2d.view.y_max`
- `space2d.view.pan_x`
- `space2d.view.pan_y`
- `space2d.view.zoom`

### fixed64 매핑 예시
```txt
graph.axis.x_min=axis_x_min
graph.axis.x_max=axis_x_max
graph.view.zoom=view_zoom
space2d.view.x_min=world_x_min
space2d.view.x_max=world_x_max
```

### 스키마 프리셋 예시
WASM 패널의 “스키마 프리셋”에서 저장/선택할 수 있습니다.
```txt
seamgrim.graph.v0=graph
seamgrim.space2d.v0=space2d
seamgrim.table.v0=table
seamgrim.text.v0=text
seamgrim.structure.v0=structure
```

## 샘플
- graph: `solutions/seamgrim_ui_mvp/samples/graph_v0/sample_graph.json`
- table: `solutions/seamgrim_ui_mvp/samples/table_v0/sample_dataset.json`
- table(csv): `solutions/seamgrim_ui_mvp/samples/table_v0/sample_dataset.csv`
- text: `solutions/seamgrim_ui_mvp/samples/text_v0/lesson_intro.md`
- structure: `solutions/seamgrim_ui_mvp/samples/structure_v0/sample_structure.json`
- space2d: `solutions/seamgrim_ui_mvp/samples/space2d_v0/sample_space2d.json`
  - space2d는 `points`와 함께 `shapes`(line/circle/point)도 지원합니다.

## lesson 자산 자동 로드
- lesson 폴더에 `table.json`/`table.csv`, `text.md`, `structure.json`, `space2d.json`이 있으면 자동 로드합니다.
- `required_views`에 해당하는 자산만 로드합니다.
- 교과 탭의 “뷰 자동 이동” 토글로 로딩 후 뷰 전환을 제어합니다.

## 주의
- 메인 `index.html`은 practical 모드에서 실행 서버 호출 없이 WASM 실행을 우선합니다.
- 실행 서버 의존 기능(비교/내보내기/검증/고급 도구)은 `index.html` 고급 메뉴의 `Smoke` 모드에서 사용합니다.
- 실행 서버 URL은 실행 탭(`run-server-url`)과 도구 탭(`bridge-url`) 입력이 자동 동기화되며, 브라우저 로컬 저장소에 유지됩니다.
