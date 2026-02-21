# Seamgrim UI MVP

정적 HTML/CSS/JS로 구성된 셈그림 UI MVP 스켈레톤입니다.

## 사용
- 로컬 브리지 실행: `python solutions/seamgrim_ui_mvp/tools/bridge_server.py`
- 브라우저에서 `http://localhost:8787/` 접속 (또는 파일 직접 열고 브리지 URL 입력)
- WASM 스모크 페이지: `http://localhost:8787/wasm_smoke.html`
- AGE3 교과 스키마 게이트: `python tests/run_seamgrim_lesson_schema_gate.py`
- AGE3 승격 완료까지 강제: `python tests/run_seamgrim_lesson_schema_gate.py --require-promoted`
- AGE3 UI 게이트(R3-A/B/C 최소 기능): `python tests/run_seamgrim_ui_age3_gate.py`
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
- 검증/인스펙터: DDN 메타, 해시/bridge_check, 스키마 요약
- 샘/거울 파일 로드(요약 카드)
- 보개 확장: 그래프/2D/표/글/구조 뷰 로드 및 표시

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
- 브리지 서버를 켜지 않으면 “DDN 실행”이 실패합니다. (로컬 미리보기는 사용하지 않습니다.)
