# UI_RUNTIME_LOAD_CHECK_V1

## 범위

- Q16 산출물이다.
- 입력은 `docs/context/reports/UI_RUNNER_DEPENDENCY_MAP_V1.md`의 제품 분류 91개 모듈이다.
- 정적분류 `예(동적의심)` 54개는 별도 카운트로 재확인했다.
- 로드는 로컬 루프백 HTTP 서버에서 `solutions/seamgrim_ui_mvp/ui/index.html?devSurfaces=1`을 Playwright/Chromium headless로 열어 확인했다.
- UI 코드 수정·삭제는 하지 않았다.

## 실행 증빙

- 실행 URL: `http://127.0.0.1:10484/solutions/seamgrim_ui_mvp/ui/index.html?devSurfaces=1`
- JSON 로그: `I:\home\urihanl\ddn\codex\out\playwright\queue-20260706-q16\ui-runtime-load-check.json`
- 스크린샷: `I:\home\urihanl\ddn\codex\out\playwright\queue-20260706-q16\ui-runtime-load-check.png`
- DOM 확인: `hasApp=True`, `hasLessonGrid=True`, `hasDevSurfaceRoot=True`, `devSurfaceSections=52`
- 제품 경로에서 로드된 UI JS 모듈 수: 117개
- 콘솔 경고/에러: 0건
- pageerror: 0건
- requestfailed: 0건
- 4xx/5xx 응답: 0건

## 요약

- 제품 분류 모듈: 91개
- 제품 분류 중 동적의심: 54개
- 제품 모듈 런타임 로드 성공: 91개
- 제품 모듈 런타임 로드 실패: 0개
- 제품 모듈 미도달: 0개
- 동적의심 런타임 로드 성공: 54개
- 동적의심 런타임 로드 실패: 0개
- 동적의심 미도달: 0개

## 해석

- Q16 실행에서는 제품 91개가 모두 실제 `index.html?devSurfaces=1` 제품 경로에서 요청됐고, 각 모듈의 브라우저 `import()`도 모두 성공했다.
- 동적의심 54개도 모두 `dev_surfaces.js` 경로에서 실제 요청됐으므로, 이번 실행 조건에서는 죽은 코드 후보로 남는 항목이 없다.
- 콘솔 경고/에러, pageerror, requestfailed, 4xx/5xx 응답은 모두 0건이었다.

## 표

| 모듈 | 정적분류(기존) | 런타임 로드 확인(성공/실패/미도달) | 콘솔 에러 내용 |
|---|---|---|---|
| `solutions/seamgrim_ui_mvp/ui/app.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/display_label_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/education_assessment_pack.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/education_classroom_ui_pack.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/education_operations_lts.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/education_publication_pack.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/featured_seed_catalog.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/featured_seed_quick_launch.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/first_run_catalog.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/free_lab_experiment_report.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/free_lab_research_workflow.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/free_lab_share_pack.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/free_lab_ui_pack.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/input_registry.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/inspector_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/legacy_warning_guide.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/lesson_library_curation.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/lesson_loader_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/numeric_curriculum_track.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/overlay_session_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/platform_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/platform_mock_adapter_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/platform_server_adapter_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/play_diagnostic_contract.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/preview_payload_loader.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/preview_result_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/preview_session.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/preview_view_model.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/question_card_author_tool_share.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/question_card_dev_assist.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/question_card_smoke.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/question_card_validation.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/question_card_workflow_hardening.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/rpg_engine_adapter_lts.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/rpg_story_package.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/run_action_rail_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/run_exec_status_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/run_observe_action_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/run_observe_family_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/run_observe_summary_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/run_runtime_hint_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/run_warning_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/run_warning_panel_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/scene_summary_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/seamgrim_numeric_track_consolidation.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/seulgi_proposal_ui.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/seulgi_replay_safe_workflow.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/snapshot_session_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/social_world_bridge_pack.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/social_world_lts_readiness.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/social_world_policy_ghost_ui.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/social_world_template_registry.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_benchmark_baseline_local_snapshot.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_benchmark_baseline_prep_dry_run.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_classroom_mode.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_classroom_operations_panel_preview.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_classroom_operations_triage.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_diagnostic_fixit_integration.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_diagnostic_fixit_preview.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_edit_run_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_lesson_publication_review_surface.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_local_release_rehearsal_check.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_local_share_package.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_ma3_next_development_queue_rebase.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_ma3_next_queue_coordinate_lock.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_ma3_regression_gate_matrix.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_next_roadmap_v2_coordinate_lock.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_numeric_report_workflow_stage.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_numeric_result_report_stage.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_operations_preview_stage_closure.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_post_super_long_rebase.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_productization_stage_closure.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_productization_stage_rebase.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_public_release_approval_recheck.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_publication_artifact_dry_run.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_release_review_packet_dashboard.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_teacher_feedback_loop_seed.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/studio_teacher_feedback_surface_preview.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/subpanel_tab_policy.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/toolchain_benchmark_lts.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/toolchain_diagnostic_ui_lsp.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/toolchain_registry_verification.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/ttonimaru_platform_hardening.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/ttonimaru_project_share_ui.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/ttonimaru_public_registry_seed.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/ttonimaru_publication_read_api.js` | 제품(동적의심) | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/update_tick_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/view_family_contract.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |
| `solutions/seamgrim_ui_mvp/ui/wasm_page_common.js` | 제품 | 성공 | 없음; 제품/devSurfaces 경로 요청 + 직접 import PASS |

## 자기 검증

- `UI_RUNNER_DEPENDENCY_MAP_V1.md`에서 제품 행 91개, 동적의심 54개를 파싱했다.
- Playwright 실행 결과: `Q16 runtime PASS product=91 dynamic=54`.
- 상태 집계: `{"성공": 91}`
- 동적의심 상태 집계: `{"성공": 54}`
- UI 코드 수정 없음.
