# UI_RUNNER_DEPENDENCY_MAP_V1

## 범위

- 미션 단계 D 산출물이다.
- `solutions/seamgrim_ui_mvp/ui/*.js` 최상위 JS 102개와 `tests/*.mjs` 러너 191개를 정적 추적했다.
- 제품 도달 기준은 `solutions/seamgrim_ui_mvp/ui/index.html`의 script-src와 `app.js`에서의 정적 도달이다.
- `ui/screens/*.js`는 실제 화면 엔트리로 쓰이는 제품 시작점 가정에 포함했다.
- 정적 import/export를 우선 추적했고, `import("...")` 및 `dev_surfaces.js`의 `module: "./*.js"` 문자열 레지스트리는 `동적의심`으로 표기했다.
- 수정·삭제는 하지 않았다.

## 요약

- UI 최상위 JS: 102개 (제품 도달 또는 제품 시작점 가정 91, 러너전용 10, 고아 1)
- 제품 연결 중 동적의심: 54개
- tests/*.mjs 러너: 191개 (러너전용 189, 고아 2)
- 전체 표 행 수: 293개

## 표

| 모듈/러너 | 제품 연결(index.html/app.js/screens에서 도달) | 참조자 목록 | 분류(제품/러너전용/고아) |
|---|---|---|---|
| `solutions/seamgrim_ui_mvp/ui/app.js` | 예 | `solutions/seamgrim_ui_mvp/ui/index.html`, `tests/education_assessment_pack_runner.mjs`, `tests/education_classroom_ui_pack_runner.mjs`, `tests/education_operations_lts_runner.mjs`, `tests/education_publication_pack_runner.mjs`, `tests/free_lab_experiment_report_runner.mjs`, `tests/free_lab_first_run_runner.mjs`, `tests/free_lab_research_workflow_runner.mjs`, `tests/free_lab_share_pack_runner.mjs`, `tests/free_lab_ui_pack_runner.mjs`, `tests/question_card_author_tool_share_runner.mjs`, `tests/question_card_dev_assist_runner.mjs`, ... 외 102개 | 제품 |
| `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/app.js`, `tests/studio_lesson_publication_review_surface_runner.mjs`, `tests/studio_local_package_export_action_runner.mjs`, `tests/studio_release_review_packet_dashboard_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/display_label_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/components/table_preview.js`, `solutions/seamgrim_ui_mvp/ui/inspector_contract.js`, `solutions/seamgrim_ui_mvp/ui/playground.js`, `solutions/seamgrim_ui_mvp/ui/run_observe_summary_contract.js`, `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/seamgrim_korean_display_label_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/education_assessment_pack.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/education_assessment_pack_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/education_classroom_ui_pack.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/education_classroom_ui_pack_runner.mjs`, `tests/education_publication_pack_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/education_operations_lts.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/education_operations_lts_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/education_publication_pack.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/education_operations_lts_runner.mjs`, `tests/education_publication_pack_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/featured_seed_catalog.js` | 예 | `solutions/seamgrim_ui_mvp/ui/app.js`, `solutions/seamgrim_ui_mvp/ui/screens/browse.js`, `tests/seamgrim_featured_seed_quick_launch_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/featured_seed_quick_launch.js` | 예 | `solutions/seamgrim_ui_mvp/ui/app.js`, `tests/seamgrim_featured_seed_quick_launch_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/first_run_catalog.js` | 예 | `solutions/seamgrim_ui_mvp/ui/run_action_rail_contract.js`, `solutions/seamgrim_ui_mvp/ui/screens/browse.js`, `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/seamgrim_first_run_catalog_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/formula_sugar.js` | 아니오 | `tests/seamgrim_formula_sugar_runner.mjs` | 러너전용 |
| `solutions/seamgrim_ui_mvp/ui/free_lab_experiment_report.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/free_lab_experiment_report_runner.mjs`, `tests/free_lab_ui_pack_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/free_lab_first_run.js` | 아니오 | `tests/free_lab_experiment_report_runner.mjs`, `tests/free_lab_first_run_runner.mjs`, `tests/free_lab_ui_pack_runner.mjs` | 러너전용 |
| `solutions/seamgrim_ui_mvp/ui/free_lab_research_workflow.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/free_lab_research_workflow_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/free_lab_share_pack.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/free_lab_research_workflow_runner.mjs`, `tests/free_lab_share_pack_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/free_lab_ui_pack.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/free_lab_share_pack_runner.mjs`, `tests/free_lab_ui_pack_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/graph_autorender.js` | 아니오 | `tests/seamgrim_bogae_graph_prefix_runner.mjs`, `tests/seamgrim_bogae_madi_graph_ui_runner.mjs`, `tests/seamgrim_graph_autorender_runner.mjs` | 러너전용 |
| `solutions/seamgrim_ui_mvp/ui/input_registry.js` | 예 | `solutions/seamgrim_ui_mvp/ui/app.js`, `solutions/seamgrim_ui_mvp/ui/scene_summary_contract.js`, `solutions/seamgrim_ui_mvp/ui/snapshot_session_contract.js`, `tests/seamgrim_input_registry_runner.mjs`, `tests/seamgrim_scene_summary_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/inspector_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/seamgrim_inspector_contract_runner.mjs`, `tests/seamgrim_korean_display_label_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/legacy_warning_guide.js` | 예 | `solutions/seamgrim_ui_mvp/ui/app.js`, `tests/seamgrim_legacy_warning_guide_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/lesson_library_curation.js` | 예 | `solutions/seamgrim_ui_mvp/ui/app.js`, `tests/seamgrim_lesson_library_curation_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/lesson_loader_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/app.js`, `solutions/seamgrim_ui_mvp/ui/playground.js`, `tests/seamgrim_education_curriculum_template_runner.mjs`, `tests/seamgrim_lesson_authoring_flow_browser_runner.mjs`, `tests/seamgrim_lesson_library_curation_runner.mjs`, `tests/seamgrim_lesson_loader_runner.mjs`, `tests/seamgrim_workbench_shell_browser_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/numeric_curriculum_track.js` | 예 | `solutions/seamgrim_ui_mvp/ui/app.js`, `solutions/seamgrim_ui_mvp/ui/screens/browse.js`, `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/seamgrim_numeric_track_browser_index_runner.mjs`, `tests/seamgrim_numeric_track_lesson_preview_runner.mjs`, `tests/seamgrim_numeric_track_report_export_runner.mjs`, `tests/seamgrim_numeric_track_result_compare_export_runner.mjs`, `tests/seamgrim_numeric_track_result_compare_history_export_runner.mjs`, `tests/seamgrim_numeric_track_result_compare_history_report_export_runner.mjs`, `tests/seamgrim_numeric_track_result_compare_history_report_runner.mjs`, `tests/seamgrim_numeric_track_result_compare_history_report_table_export_runner.mjs`, `tests/seamgrim_numeric_track_result_compare_history_report_table_runner.mjs`, ... 외 21개 | 제품 |
| `solutions/seamgrim_ui_mvp/ui/nurigym_python_web_parity.js` | 아니오 | `tests/nurigym_python_web_parity_runner.mjs` | 러너전용 |
| `solutions/seamgrim_ui_mvp/ui/overlay_session_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/app.js`, `solutions/seamgrim_ui_mvp/ui/snapshot_session_contract.js`, `tests/seamgrim_overlay_session_contract_runner.mjs`, `tests/seamgrim_overlay_session_pack_runner.mjs`, `tests/seamgrim_scene_summary_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/platform_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/app.js`, `solutions/seamgrim_ui_mvp/ui/platform_mock_adapter_contract.js`, `tests/seamgrim_platform_mock_adapter_roundtrip_runner.mjs`, `tests/seamgrim_platform_mock_payload_snapshot_runner.mjs`, `tests/seamgrim_platform_server_adapter_integration_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/platform_mock_adapter_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/app.js`, `tests/seamgrim_platform_mock_adapter_roundtrip_runner.mjs`, `tests/seamgrim_platform_mock_payload_snapshot_runner.mjs`, `tests/seamgrim_platform_server_adapter_integration_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/platform_server_adapter_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/app.js`, `tests/seamgrim_platform_server_action_rail_runner.mjs`, `tests/seamgrim_platform_server_adapter_contract_runner.mjs`, `tests/seamgrim_platform_server_adapter_integration_runner.mjs`, `tests/seamgrim_run_warning_message_map_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/play_diagnostic_contract.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/playground.js`, `solutions/seamgrim_ui_mvp/ui/studio_diagnostic_fixit_preview.js`, `tests/seamgrim_playground_diagnostic_contract_runner.mjs`, `tests/studio_diagnostic_fixit_editor_apply_runner.mjs`, `tests/studio_diagnostic_fixit_integration_runner.mjs`, `tests/studio_diagnostic_fixit_preview_browser_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/play_output_contract.js` | 아니오 | `tests/seamgrim_play_output_contract_runner.mjs` | 러너전용 |
| `solutions/seamgrim_ui_mvp/ui/play_source_contract.js` | 아니오 | `tests/seamgrim_play_source_contract_runner.mjs` | 러너전용 |
| `solutions/seamgrim_ui_mvp/ui/playground.js` | 아니오 | 없음(정적 검색) | 고아 |
| `solutions/seamgrim_ui_mvp/ui/preview_payload_loader.js` | 예 | `solutions/seamgrim_ui_mvp/ui/preview_session.js`, `tests/seamgrim_preview_payload_loader_runner.mjs`, `tests/studio_local_package_export_action_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/preview_result_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/playground.js`, `solutions/seamgrim_ui_mvp/ui/preview_session.js`, `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/seamgrim_preview_result_contract_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/preview_session.js` | 예 | `solutions/seamgrim_ui_mvp/ui/screens/browse.js`, `tests/seamgrim_preview_session_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/preview_view_model.js` | 예 | `solutions/seamgrim_ui_mvp/ui/playground.js`, `solutions/seamgrim_ui_mvp/ui/preview_session.js`, `solutions/seamgrim_ui_mvp/ui/screens/browse.js`, `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/seamgrim_preview_view_model_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/question_card_author_tool_share.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/question_card_author_tool_share_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/question_card_dev_assist.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/question_card_dev_assist_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/question_card_smoke.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/question_card_smoke_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/question_card_validation.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/question_card_validation_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/question_card_workflow_hardening.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/question_card_workflow_hardening_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/rpg_box_authoring_ui.js` | 아니오 | `tests/rpg_box_authoring_ui_runner.mjs`, `tests/rpg_story_package_runner.mjs` | 러너전용 |
| `solutions/seamgrim_ui_mvp/ui/rpg_engine_adapter_lts.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/rpg_engine_adapter_lts_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/rpg_story_package.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/rpg_engine_adapter_lts_runner.mjs`, `tests/rpg_story_package_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/run_action_rail_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/seamgrim_run_warning_message_map_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/run_exec_status_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/seamgrim_run_warning_message_map_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/run_observe_action_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/seamgrim_pendulum_bogae_runner.mjs`, `tests/seamgrim_run_warning_message_map_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/run_observe_family_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/seamgrim_observe_output_contract_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/run_observe_summary_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/seamgrim_korean_display_label_runner.mjs`, `tests/seamgrim_observe_output_contract_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/run_runtime_hint_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/seamgrim_run_warning_message_map_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/run_warning_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/run_action_rail_contract.js`, `solutions/seamgrim_ui_mvp/ui/run_exec_status_contract.js`, `solutions/seamgrim_ui_mvp/ui/run_warning_panel_contract.js`, `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/seamgrim_run_warning_message_map_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/run_warning_panel_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/seamgrim_run_warning_message_map_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/scene_summary_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/app.js`, `tests/seamgrim_scene_summary_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/seamgrim_numeric_track_consolidation.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/seamgrim_numeric_track_consolidation_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js` | 예 | `solutions/seamgrim_ui_mvp/ui/components/structure_preview.js`, `solutions/seamgrim_ui_mvp/ui/graph_autorender.js`, `solutions/seamgrim_ui_mvp/ui/playground.js`, `solutions/seamgrim_ui_mvp/ui/preview_result_contract.js`, `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `solutions/seamgrim_ui_mvp/ui/wasm_ddn_wrapper.js`, `tests/seamgrim_bogae_graph_prefix_runner.mjs`, `tests/seamgrim_bogae_madi_graph_ui_runner.mjs`, `tests/seamgrim_editor_run_handoff_runner.mjs`, `tests/seamgrim_editor_run_transaction_runner.mjs`, `tests/seamgrim_observe_output_contract_runner.mjs`, `tests/seamgrim_playground_smoke_runner.mjs`, ... 외 6개 | 제품 |
| `solutions/seamgrim_ui_mvp/ui/seulgi_proposal_ui.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/seulgi_proposal_ui_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/seulgi_replay_safe_workflow.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/seulgi_replay_safe_workflow_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/snapshot_session_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/app.js`, `tests/seamgrim_range_split_runner.mjs`, `tests/seamgrim_update_tick_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/social_world_bridge_pack.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/social_world_bridge_pack_runner.mjs`, `tests/social_world_policy_ghost_ui_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/social_world_lts_readiness.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/education_assessment_pack_runner.mjs`, `tests/social_world_lts_readiness_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/social_world_policy_ghost_ui.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/social_world_policy_ghost_ui_runner.mjs`, `tests/social_world_template_registry_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/social_world_template_registry.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/social_world_lts_readiness_runner.mjs`, `tests/social_world_template_registry_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_benchmark_baseline_local_snapshot.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_benchmark_baseline_local_snapshot_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_benchmark_baseline_prep_dry_run.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_benchmark_baseline_prep_dry_run_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_classroom_mode.js` | 예 | `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/studio_classroom_mode_browser_runner.mjs`, `tests/studio_classroom_report_export_action_runner.mjs`, `tests/studio_classroom_report_workflow_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_classroom_operations_panel_preview.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_classroom_operations_panel_preview_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_classroom_operations_triage.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_classroom_operations_triage_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_diagnostic_fixit_integration.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/toolchain_diagnostic_ui_lsp.js`, `tests/studio_diagnostic_fixit_integration_runner.mjs`, `tests/toolchain_diagnostic_ui_lsp_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_diagnostic_fixit_preview.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/studio_diagnostic_fixit_integration.js`, `tests/studio_diagnostic_fixit_editor_apply_runner.mjs`, `tests/studio_diagnostic_fixit_integration_runner.mjs`, `tests/studio_diagnostic_fixit_preview_browser_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_edit_run_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/app.js`, `solutions/seamgrim_ui_mvp/ui/run_warning_panel_contract.js`, `tests/seamgrim_lesson_authoring_flow_browser_runner.mjs`, `tests/seamgrim_run_warning_message_map_runner.mjs`, `tests/seamgrim_workbench_shell_browser_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_lesson_authoring_run_integration.js` | 아니오 | `tests/studio_lesson_authoring_run_integration_runner.mjs` | 러너전용 |
| `solutions/seamgrim_ui_mvp/ui/studio_lesson_publication_review_surface.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/studio_lesson_publication_review_surface_runner.mjs`, `tests/studio_publication_prep_export_action_runner.mjs`, `tests/studio_registry_share_seed_export_action_runner.mjs`, `tests/studio_release_approval_continuity_export_action_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_local_release_rehearsal_check.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_local_release_rehearsal_check_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_local_share_package.js` | 예 | `solutions/seamgrim_ui_mvp/ui/app.js`, `solutions/seamgrim_ui_mvp/ui/screens/browse.js`, `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/studio_local_package_export_action_runner.mjs`, `tests/studio_local_share_and_packaging_browser_runner.mjs`, `tests/studio_publication_prep_export_action_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_ma3_next_development_queue_rebase.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_ma3_next_development_queue_rebase_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_ma3_next_queue_coordinate_lock.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_ma3_next_queue_coordinate_lock_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_ma3_regression_gate_matrix.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_ma3_regression_gate_matrix_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_malblock_workbench_integration.js` | 아니오 | `tests/studio_malblock_workbench_integration_runner.mjs` | 러너전용 |
| `solutions/seamgrim_ui_mvp/ui/studio_next_roadmap_v2_coordinate_lock.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_next_roadmap_v2_coordinate_lock_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_numeric_report_workflow_stage.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_numeric_report_stage_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_numeric_result_report_stage.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_numeric_result_stage_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_operations_preview_stage_closure.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_operations_preview_stage_closure_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_post_super_long_rebase.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_post_super_long_rebase_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_productization_stage_closure.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_productization_stage_closure_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_productization_stage_rebase.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_productization_stage_rebase_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_public_release_approval_recheck.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_public_release_approval_recheck_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_publication_artifact_dry_run.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_publication_artifact_dry_run_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_release_review_packet_dashboard.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_release_review_packet_dashboard_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_teacher_feedback_loop_seed.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_teacher_feedback_loop_seed_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/studio_teacher_feedback_surface_preview.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/studio_teacher_feedback_surface_preview_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/subpanel_tab_policy.js` | 예 | `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/seamgrim_subpanel_graph_mode_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/toolchain_benchmark_lts.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/toolchain_benchmark_lts_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/toolchain_diagnostic_ui_lsp.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/toolchain_benchmark_lts_runner.mjs`, `tests/toolchain_diagnostic_ui_lsp_runner.mjs`, `tests/toolchain_registry_verification_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/toolchain_registry_verification.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/toolchain_benchmark_lts_runner.mjs`, `tests/toolchain_registry_verification_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/ttonimaru_platform_hardening.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/ttonimaru_platform_hardening_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/ttonimaru_project_share_ui.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/ttonimaru_project_share_ui_runner.mjs`, `tests/ttonimaru_public_registry_seed_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/ttonimaru_public_registry_seed.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/ttonimaru_platform_hardening_runner.mjs`, `tests/ttonimaru_public_registry_seed_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/ttonimaru_publication_read_api.js` | 예(동적의심) | `solutions/seamgrim_ui_mvp/ui/dev_surfaces.js`, `tests/ttonimaru_project_share_ui_runner.mjs`, `tests/ttonimaru_publication_read_api_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/update_tick_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/overlay_session_contract.js`, `solutions/seamgrim_ui_mvp/ui/scene_summary_contract.js`, `solutions/seamgrim_ui_mvp/ui/snapshot_session_contract.js`, `tests/seamgrim_update_tick_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/view_family_contract.js` | 예 | `solutions/seamgrim_ui_mvp/ui/app.js`, `solutions/seamgrim_ui_mvp/ui/input_registry.js`, `solutions/seamgrim_ui_mvp/ui/inspector_contract.js`, `solutions/seamgrim_ui_mvp/ui/lesson_loader_contract.js`, `solutions/seamgrim_ui_mvp/ui/playground.js`, `solutions/seamgrim_ui_mvp/ui/preview_session.js`, `solutions/seamgrim_ui_mvp/ui/scene_summary_contract.js`, `solutions/seamgrim_ui_mvp/ui/screens/browse.js`, `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js`, `solutions/seamgrim_ui_mvp/ui/snapshot_session_contract.js`, `tests/bogae_cell_grid_primitive_runner.mjs` | 제품 |
| `solutions/seamgrim_ui_mvp/ui/wasm_ddn_wrapper.js` | 아니오 | `tests/seamgrim_bogae_madi_graph_ui_runner.mjs`, `tests/seamgrim_editor_run_transaction_runner.mjs`, `tests/seamgrim_playground_smoke_runner.mjs`, `tests/seamgrim_sample_grid_space_runner.mjs`, `tests/seamgrim_stdlib_1_wasm_runner.mjs`, `tests/seamgrim_studio_draft_runtime_runner.mjs`, `tests/seamgrim_wasm_cli_runtime_parity_runner.mjs`, `tests/seamgrim_wasm_pack_runner.mjs`, `tests/seamgrim_wasm_wrapper_runner.mjs` | 러너전용 |
| `solutions/seamgrim_ui_mvp/ui/wasm_page_common.js` | 예 | `solutions/seamgrim_ui_mvp/ui/app.js`, `solutions/seamgrim_ui_mvp/ui/components/bogae.js`, `solutions/seamgrim_ui_mvp/ui/components/dotbogi.js`, `solutions/seamgrim_ui_mvp/ui/playground.js`, `solutions/seamgrim_ui_mvp/ui/runtime/wasm_vm_runtime.js`, `solutions/seamgrim_ui_mvp/ui/screens/run.js`, `tests/seamgrim_space2d_primitive_source_runner.mjs`, `tests/seamgrim_ui_common_runner.mjs`, `tests/seamgrim_wasm_loader_diag_runner.mjs`, `tests/seamgrim_wasm_pack_runner.mjs` | 제품 |
| `tests/block_editor_roundtrip_runner.mjs` | 아니오 | `tests/run_block_editor_roundtrip_check.py`, `tests/run_block_editor_roundtrip_expected_refresh_check.py`, `tests/run_seamgrim_malblock_roundtrip_subset_check.py` | 러너전용 |
| `tests/bogae_cell_grid_primitive_runner.mjs` | 아니오 | `tests/run_bogae_cell_grid_primitive_check.py` | 러너전용 |
| `tests/education_assessment_pack_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ha2_education_assessment_pack_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/education_classroom_ui_pack_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ha3_classroom_ui_pack_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/education_operations_lts_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ha5_education_operations_lts_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/education_publication_pack_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ha4_public_course_publication_pack_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/free_lab_experiment_report_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ba2_free_lab_experiment_report_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/free_lab_first_run_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ba1_free_lab_first_run_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/free_lab_research_workflow_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ba5_free_lab_research_workflow_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/free_lab_share_pack_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ba4_free_lab_share_pack_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/free_lab_ui_pack_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ba3_free_lab_ui_pack_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/malblock_authoring_ui_browser_runner.mjs` | 아니오 | `tests/run_malblock_authoring_ui_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/nurigym_python_web_parity_runner.mjs` | 아니오 | `tests/run_roadmap_v2_a3_nurigym_python_web_parity_check.py` | 러너전용 |
| `tests/nurimaker_grid_runner.mjs` | 아니오 | `tests/run_nurimaker_grid_smoke_check.py` | 러너전용 |
| `tests/question_card_author_tool_share_runner.mjs` | 아니오 | `tests/run_roadmap_v2_geo0_ai_question_card_seed_behavior_reassessment_check.py`, `tests/run_roadmap_v2_geo4_author_tool_share_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/question_card_dev_assist_runner.mjs` | 아니오 | `tests/run_roadmap_v2_geo0_ai_question_card_seed_behavior_reassessment_check.py`, `tests/run_roadmap_v2_geo3_dev_assist_ui_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/question_card_smoke_runner.mjs` | 아니오 | `tests/run_roadmap_v2_geo0_ai_question_card_seed_behavior_reassessment_check.py`, `tests/run_roadmap_v2_geo1_question_card_smoke_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/question_card_validation_runner.mjs` | 아니오 | `tests/run_roadmap_v2_geo0_ai_question_card_seed_behavior_reassessment_check.py`, `tests/run_roadmap_v2_geo2_ai_output_validation_pack_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/question_card_workflow_hardening_runner.mjs` | 아니오 | `tests/run_roadmap_v2_geo0_ai_question_card_seed_behavior_reassessment_check.py`, `tests/run_roadmap_v2_geo5_ai_workflow_hardening_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/rpg_box_authoring_ui_runner.mjs` | 아니오 | `tests/run_roadmap_v2_cha3_rpg_box_authoring_ui_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/rpg_engine_adapter_lts_runner.mjs` | 아니오 | `tests/run_roadmap_v2_cha5_rpg_engine_adapter_lts_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/rpg_story_package_runner.mjs` | 아니오 | `tests/run_roadmap_v2_cha4_rpg_story_package_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/rpgbox_block_runner.mjs` | 아니오 | `tests/run_rpgbox_block_smoke_check.py` | 러너전용 |
| `tests/seamgrim_auth_save_surface_runner.mjs` | 아니오 | `tests/run_roadmap_v2_na4_stdlib_registry_reconciliation_check.py`, `tests/run_seamgrim_auth_save_surface_check.py`, `tests/run_seamgrim_platform_mock_interface_contract_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_block_editor_runner.mjs` | 아니오 | `tests/run_roadmap_v2_la1_malblock_intro_rail_matrix_reconciliation_check.py`, `tests/run_roadmap_v2_na1_post_matrix_frontier_rebase_check.py`, `tests/run_seamgrim_block_editor_smoke_check.py`, `tests/run_seamgrim_intro_exec_rail_check.py`, `tests/run_seamgrim_vol4_runtime_track_check.py` | 러너전용 |
| `tests/seamgrim_bogae_console_grid_runner.mjs` | 아니오 | 없음(정적 검색) | 고아 |
| `tests/seamgrim_bogae_graph_prefix_runner.mjs` | 아니오 | `tests/run_bogae_graph_prefix_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_bogae_madi_graph_ui_runner.mjs` | 아니오 | `tests/run_seamgrim_bogae_madi_graph_ui_check.py` | 러너전용 |
| `tests/seamgrim_browse_selection_runner.mjs` | 아니오 | `tests/run_seamgrim_browse_selection_flow_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_console_rich_markup_runner.mjs` | 아니오 | `tests/run_seamgrim_console_rich_markup_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_control_exposure_policy_runner.mjs` | 아니오 | `tests/run_seamgrim_control_exposure_policy_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_editor_run_handoff_runner.mjs` | 아니오 | `tests/run_seamgrim_editor_run_handoff_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_editor_run_transaction_runner.mjs` | 아니오 | `tests/run_seamgrim_editor_run_transaction_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_editor_selection_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_education_curriculum_template_runner.mjs` | 아니오 | `tests/run_seamgrim_education_curriculum_template_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_featured_seed_quick_launch_runner.mjs` | 아니오 | `tests/run_seamgrim_featured_seed_quick_launch_check.py`, `tests/run_seamgrim_product_stabilization_smoke_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_first_run_catalog_runner.mjs` | 아니오 | `tests/run_seamgrim_first_run_onboarding_pack_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_formula_sugar_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_graph_autorender_runner.mjs` | 아니오 | `tests/run_seamgrim_graph_autorender.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_guideblock_keys_pack_runner.mjs` | 아니오 | `tests/run_seamgrim_guideblock_keys_pack_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_input_registry_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_inspector_contract_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_korean_display_label_runner.mjs` | 아니오 | `tests/run_seamgrim_product_stabilization_smoke_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_legacy_warning_guide_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_lesson_authoring_flow_browser_runner.mjs` | 아니오 | `tests/run_seamgrim_lesson_authoring_flow_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_lesson_library_curation_runner.mjs` | 아니오 | `tests/run_seamgrim_lesson_library_curation_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_lesson_loader_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ga5_grammar_lts_docs_closed_reconciliation_check.py`, `tests/run_seamgrim_lesson_authoring_flow_check.py`, `tests/run_studio_baseline_rebase_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_lesson_run_preset_rail_runner.mjs` | 아니오 | `tests/run_seamgrim_lesson_run_preset_rail_check.py`, `tests/run_seamgrim_numeric_track_run_preset_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_live_repl_runner.mjs` | 아니오 | `tests/run_seamgrim_live_repl_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_kernel_ui_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_kernel_ui_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_browser_index_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_browser_index_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_consolidation_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_consolidation_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_lesson_preview_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_lesson_preview_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_report_export_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_report_export_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_export_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_export_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_history_export_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_history_export_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_history_report_export_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_history_report_export_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_history_report_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_history_report_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_history_report_table_export_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_history_report_table_export_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_history_report_table_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_history_report_table_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_export_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_export_check.py`, `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_check.py`, `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_export_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_check.py`, `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_summary_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_summary_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_summary_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_check.py`, `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_history_report_table_status_badge_export_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_check.py`, `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_export_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_history_report_table_status_badge_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_check.py`, `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_export_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_history_report_table_status_export_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_check.py`, `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_export_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_history_report_table_status_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_check.py`, `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_export_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_history_report_table_summary_export_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_check.py`, `tests/run_seamgrim_numeric_track_result_compare_history_report_table_summary_export_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_history_report_table_summary_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_history_report_table_summary_check.py`, `tests/run_seamgrim_numeric_track_result_compare_history_report_table_summary_export_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_history_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_history_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_compare_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_compare_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_history_filter_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_history_filter_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_reopen_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_reopen_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_summary_export_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_summary_export_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_result_timeline_view_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_result_timeline_view_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_run_preset_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_run_preset_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_numeric_track_run_result_link_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_run_result_link_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_object_revision_surface_runner.mjs` | 아니오 | `tests/run_seamgrim_object_revision_surface_check.py`, `tests/run_seamgrim_platform_mock_interface_contract_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_observe_output_contract_runner.mjs` | 아니오 | `tests/run_seamgrim_observe_output_contract_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_overlay_compare_pack_runner.mjs` | 아니오 | `tests/run_seamgrim_group_id_summary_check.py`, `tests/run_seamgrim_overlay_compare_pack.py`, `tests/run_seamgrim_overlay_group_id_report_selftest.py` | 러너전용 |
| `tests/seamgrim_overlay_session_contract_runner.mjs` | 아니오 | `tests/run_seamgrim_overlay_session_contract.py`, `tests/run_seamgrim_overlay_session_wired_consistency_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_overlay_session_pack_runner.mjs` | 아니오 | `tests/run_seamgrim_group_id_summary_check.py`, `tests/run_seamgrim_overlay_group_id_report_selftest.py`, `tests/run_seamgrim_overlay_session_pack.py`, `tests/run_seamgrim_overlay_session_wired_consistency_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_package_registry_surface_runner.mjs` | 아니오 | `tests/run_seamgrim_package_registry_surface_check.py`, `tests/run_seamgrim_platform_mock_interface_contract_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_pendulum_bogae_runner.mjs` | 아니오 | `tests/run_seamgrim_first_run_onboarding_pack_check.py`, `tests/run_seamgrim_pendulum_bogae_shape_check.py`, `tests/run_seamgrim_product_stabilization_smoke_check.py`, `tests/run_seamgrim_runtime_5min_check.py`, `tests/run_seamgrim_wasm_smoke.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_pendulum_runtime_visual_runner.mjs` | 아니오 | `tests/run_seamgrim_pendulum_runtime_visual_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_platform_mock_adapter_roundtrip_runner.mjs` | 아니오 | `tests/run_seamgrim_platform_mock_adapter_roundtrip_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_platform_mock_menu_mode_runner.mjs` | 아니오 | `tests/run_seamgrim_platform_mock_interface_contract_check.py`, `tests/run_seamgrim_platform_mock_menu_mode_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_platform_mock_payload_snapshot_runner.mjs` | 아니오 | `tests/run_seamgrim_platform_mock_payload_snapshot_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_platform_route_precedence_runner.mjs` | 아니오 | `tests/run_seamgrim_platform_route_precedence_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_platform_server_action_rail_runner.mjs` | 아니오 | `tests/run_seamgrim_platform_server_action_rail_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_platform_server_adapter_contract_runner.mjs` | 아니오 | `tests/run_seamgrim_platform_server_adapter_contract_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_platform_server_adapter_integration_runner.mjs` | 아니오 | `tests/run_seamgrim_platform_server_adapter_contract_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_play_output_contract_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_play_source_contract_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_playground_diagnostic_contract_runner.mjs` | 아니오 | `tests/run_seamgrim_playground_smoke_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_playground_smoke_runner.mjs` | 아니오 | `tests/run_seamgrim_playground_smoke_check.py` | 러너전용 |
| `tests/seamgrim_preview_component_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_preview_payload_loader_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_preview_result_contract_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_preview_session_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_preview_view_model_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_publication_snapshot_surface_runner.mjs` | 아니오 | `tests/run_seamgrim_publication_snapshot_surface_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_range_split_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_review_workflow_surface_runner.mjs` | 아니오 | `tests/run_seamgrim_review_workflow_surface_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_run_history_comparison_rail_runner.mjs` | 아니오 | `tests/run_seamgrim_run_history_comparison_rail_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_run_history_export_summary_runner.mjs` | 아니오 | `tests/run_seamgrim_run_history_export_summary_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_run_legacy_autofix_runner.mjs` | 아니오 | `tests/run_seamgrim_run_legacy_autofix_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_run_manager_compare_runner.mjs` | 아니오 | `tests/run_seamgrim_product_stabilization_smoke_check.py`, `tests/run_seamgrim_run_manager_compare_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_run_toolbar_compact_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_run_warning_message_map_runner.mjs` | 아니오 | `tests/run_seamgrim_pendulum_bogae_shape_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_runtime_view_stack_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_sample_grid_space_runner.mjs` | 아니오 | `tests/run_seamgrim_product_stabilization_smoke_check.py` | 러너전용 |
| `tests/seamgrim_scene_summary_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_seed_runtime_visual_pack_runner.mjs` | 아니오 | `tests/run_seamgrim_group_id_synthesis_report_selftest.py`, `tests/run_seamgrim_seed_runtime_visual_pack_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_sharing_publishing_surface_runner.mjs` | 아니오 | `tests/run_seamgrim_platform_mock_interface_contract_check.py`, `tests/run_seamgrim_sharing_publishing_surface_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_source_management_surface_runner.mjs` | 아니오 | `tests/run_seamgrim_source_management_surface_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_space2d_primitive_source_runner.mjs` | 아니오 | `tests/run_seamgrim_space2d_primitive_source_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_stdlib_1_wasm_runner.mjs` | 아니오 | `tests/run_seamgrim_stdlib_1_wasm_check.py` | 러너전용 |
| `tests/seamgrim_studio_draft_runtime_runner.mjs` | 아니오 | 없음(정적 검색) | 고아 |
| `tests/seamgrim_studio_layout_contract_runner.mjs` | 아니오 | `tests/run_seamgrim_playground_smoke_check.py`, `tests/run_seamgrim_product_stabilization_smoke_check.py`, `tests/run_seamgrim_workbench_shell_check.py`, `tests/run_studio_baseline_rebase_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_subpanel_graph_mode_runner.mjs` | 아니오 | `tests/run_seamgrim_subpanel_graph_mode_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_ui_common_runner.mjs` | 아니오 | `tests/run_seamgrim_product_stabilization_smoke_check.py`, `tests/run_seamgrim_runtime_5min_check.py`, `tests/run_seamgrim_wasm_smoke.py`, `tests/run_seamgrim_workbench_shell_check.py`, `tests/run_studio_baseline_rebase_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_update_tick_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_view_dock_time_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_view_only_state_hash_invariant_runner.mjs` | 아니오 | `tests/run_seamgrim_view_only_state_hash_invariant_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_wasm_cli_runtime_parity_runner.mjs` | 아니오 | `tests/run_connect_wasm_cli_parity_check.py`, `tests/run_ddonirang_book_bundle_examples_canon_check.py`, `tests/run_ddonirang_vol1_bundle_cli_wasm_parity_check.py`, `tests/run_ddonirang_vol2_bundle_cli_wasm_parity_check.py`, `tests/run_ddonirang_vol3_bundle_cli_wasm_parity_check.py`, `tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py`, `tests/run_lang_core_0_check.py`, `tests/run_lang_text_escape_check.py`, `tests/run_relation_solve_wasm_cli_parity_check.py`, `tests/run_relation_solve_wasm_cli_parity_v2_check.py`, `tests/run_runtime_support_integrity_audit_check.py`, `tests/run_seamgrim_intro_exec_blocky_check.py`, ... 외 2개 | 러너전용 |
| `tests/seamgrim_wasm_lesson_canon_runner.mjs` | 아니오 | `tests/run_seamgrim_wasm_smoke.py` | 러너전용 |
| `tests/seamgrim_wasm_loader_diag_runner.mjs` | 아니오 | `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_wasm_pack_runner.mjs` | 아니오 | `tests/run_seamgrim_wasm_smoke.py` | 러너전용 |
| `tests/seamgrim_wasm_vm_runtime_runner.mjs` | 아니오 | `tests/run_age5_close.py`, `tests/run_ci_aggregate_gate_age5_diagnostics_check.py`, `tests/run_seamgrim_runtime_5min_check.py`, `tests/run_seamgrim_wasm_cli_diag_parity_check.py`, `tests/run_seamgrim_wasm_smoke.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_wasm_wrapper_runner.mjs` | 아니오 | `tests/run_age5_close.py`, `tests/run_ci_aggregate_gate_age5_diagnostics_check.py`, `tests/run_seamgrim_wasm_cli_diag_parity_check.py`, `tests/run_seamgrim_wasm_smoke.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_workbench_polish_runner.mjs` | 아니오 | `tests/run_seamgrim_lesson_library_curation_check.py`, `tests/run_seamgrim_workbench_polish_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seamgrim_workbench_shell_browser_runner.mjs` | 아니오 | `tests/run_seamgrim_workbench_shell_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seulgi_proposal_ui_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ja0_ai_boundary_behavior_reassessment_check.py`, `tests/run_roadmap_v2_ja3_seulgi_proposal_ui_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/seulgi_replay_safe_workflow_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ja0_ai_boundary_behavior_reassessment_check.py`, `tests/run_roadmap_v2_ja5_replay_safe_ai_workflow_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/social_world_bridge_pack_runner.mjs` | 아니오 | `tests/run_roadmap_v2_pa0_social_case_card_behavior_reassessment_check.py`, `tests/run_roadmap_v2_pa2_social_bridge_pack_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/social_world_lts_readiness_runner.mjs` | 아니오 | `tests/run_roadmap_v2_pa0_social_case_card_behavior_reassessment_check.py`, `tests/run_roadmap_v2_pa5_social_world_lts_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/social_world_policy_ghost_ui_runner.mjs` | 아니오 | `tests/run_roadmap_v2_pa0_social_case_card_behavior_reassessment_check.py`, `tests/run_roadmap_v2_pa3_policy_ghost_ui_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/social_world_template_registry_runner.mjs` | 아니오 | `tests/run_roadmap_v2_pa0_social_case_card_behavior_reassessment_check.py`, `tests/run_roadmap_v2_pa4_social_template_registry_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/std_grid_game_bogae_browser_dom_smoke_runner.mjs` | 아니오 | `tests/run_std_grid_game_bogae_browser_dom_smoke_check.py` | 러너전용 |
| `tests/std_grid_game_bogae_browser_input_delivery_runner.mjs` | 아니오 | `tests/run_std_grid_game_bogae_browser_input_delivery_check.py` | 러너전용 |
| `tests/std_grid_game_bogae_finite_live_loop_runner.mjs` | 아니오 | `tests/run_std_grid_game_bogae_finite_live_loop_check.py` | 러너전용 |
| `tests/std_grid_game_bogae_viewer_js_dom_runner.mjs` | 아니오 | `tests/run_std_grid_game_bogae_viewer_js_dom_check.py` | 러너전용 |
| `tests/studio_benchmark_baseline_local_snapshot_runner.mjs` | 아니오 | `tests/run_studio_benchmark_baseline_local_snapshot_check.py`, `tests/run_studio_ma3_regression_gate_matrix_check.py`, `tests/run_studio_operations_preview_stage_closure_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_benchmark_baseline_prep_dry_run_runner.mjs` | 아니오 | `tests/run_studio_benchmark_baseline_prep_dry_run_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_benchmark_lts_matrix_export_action_runner.mjs` | 아니오 | `tests/run_studio_benchmark_lts_matrix_export_action_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_classroom_mode_browser_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ma3_seamgrim_curriculum_3_classroom_ui_pack_closure_check.py`, `tests/run_roadmap_v2_ma3_studio_classroom_workbench_prereq_rebase_check.py`, `tests/run_studio_classroom_mode_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_classroom_mode_switch_runner.mjs` | 아니오 | `tests/run_studio_classroom_mode_switch_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_classroom_operations_panel_preview_runner.mjs` | 아니오 | `tests/run_studio_classroom_operations_panel_preview_check.py`, `tests/run_studio_ma3_regression_gate_matrix_check.py`, `tests/run_studio_operations_preview_stage_closure_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_classroom_operations_triage_runner.mjs` | 아니오 | `tests/run_studio_classroom_operations_triage_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_classroom_report_export_action_runner.mjs` | 아니오 | `tests/run_studio_classroom_report_export_action_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_classroom_report_workflow_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ma3_seamgrim_curriculum_3_classroom_ui_pack_closure_check.py`, `tests/run_roadmap_v2_ma3_studio_classroom_workbench_prereq_rebase_check.py`, `tests/run_studio_classroom_report_workflow_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_diagnostic_fixit_editor_apply_runner.mjs` | 아니오 | `tests/run_studio_diagnostic_fixit_editor_apply_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_diagnostic_fixit_integration_runner.mjs` | 아니오 | `tests/run_studio_diagnostic_fixit_integration_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_diagnostic_fixit_preview_browser_runner.mjs` | 아니오 | `tests/run_studio_diagnostic_fixit_preview_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_education_operations_lts_export_action_runner.mjs` | 아니오 | `tests/run_studio_education_operations_lts_export_action_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_lesson_authoring_run_integration_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ma3_seamgrim_curriculum_3_classroom_ui_pack_closure_check.py`, `tests/run_roadmap_v2_ma3_studio_classroom_workbench_prereq_rebase_check.py`, `tests/run_studio_lesson_authoring_run_integration_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_lesson_publication_review_surface_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ma4_public_lesson_publication_prereq_rebase_check.py`, `tests/run_roadmap_v2_ma4_seamgrim_curriculum_4_publication_pack_closure_check.py`, `tests/run_studio_lesson_publication_review_surface_check.py`, `tests/run_studio_ma3_regression_gate_matrix_check.py`, `tests/run_studio_operations_preview_stage_closure_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_local_package_export_action_runner.mjs` | 아니오 | `tests/run_studio_publication_prep_export_action_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_local_release_rehearsal_check_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ma5_curriculum_lts_prereq_rebase_check.py`, `tests/run_roadmap_v2_ma5_lts_candidate_progress_boundary_check.py`, `tests/run_roadmap_v2_ma5_seamgrim_curriculum_5_lts_pack_closure_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_local_share_and_packaging_browser_runner.mjs` | 아니오 | `tests/run_studio_local_packaging_consolidation_check.py`, `tests/run_studio_local_share_and_packaging_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_ma3_next_development_queue_rebase_runner.mjs` | 아니오 | `tests/run_studio_ma3_next_development_queue_rebase_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_ma3_next_queue_coordinate_lock_runner.mjs` | 아니오 | `tests/run_studio_ma3_next_queue_coordinate_lock_check.py`, `tests/run_studio_operations_preview_stage_closure_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_ma3_regression_gate_matrix_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ma5_lts_candidate_progress_boundary_check.py`, `tests/run_roadmap_v2_ma5_seamgrim_curriculum_5_lts_pack_closure_check.py`, `tests/run_studio_ma3_regression_gate_matrix_check.py`, `tests/run_studio_operations_preview_stage_closure_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_malblock_workbench_integration_runner.mjs` | 아니오 | `tests/run_studio_malblock_workbench_integration_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_next_roadmap_v2_coordinate_lock_runner.mjs` | 아니오 | `tests/run_studio_next_roadmap_v2_coordinate_lock_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_numeric_report_stage_runner.mjs` | 아니오 | `tests/run_studio_numeric_report_workflow_consolidation_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_numeric_report_workflow_consolidation_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_consolidation_check.py`, `tests/run_studio_numeric_report_workflow_consolidation_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_numeric_result_report_consolidation_runner.mjs` | 아니오 | `tests/run_seamgrim_numeric_track_consolidation_check.py`, `tests/run_studio_numeric_result_report_consolidation_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_numeric_result_stage_runner.mjs` | 아니오 | `tests/run_studio_numeric_result_report_consolidation_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_operations_preview_stage_closure_runner.mjs` | 아니오 | `tests/run_studio_operations_preview_stage_closure_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_post_super_long_rebase_runner.mjs` | 아니오 | `tests/run_studio_post_super_long_rebase_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_productization_stage_closure_runner.mjs` | 아니오 | `tests/run_studio_productization_stage_closure_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_productization_stage_rebase_runner.mjs` | 아니오 | `tests/run_studio_productization_stage_rebase_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_public_release_approval_recheck_runner.mjs` | 아니오 | `tests/run_studio_public_release_approval_recheck_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_publication_artifact_dry_run_runner.mjs` | 아니오 | `tests/run_studio_publication_artifact_dry_run_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_publication_prep_export_action_runner.mjs` | 아니오 | `tests/run_studio_publication_prep_export_action_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_registry_share_seed_export_action_runner.mjs` | 아니오 | `tests/run_studio_registry_share_seed_export_action_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_release_approval_continuity_export_action_runner.mjs` | 아니오 | `tests/run_studio_release_approval_continuity_export_action_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_release_review_packet_dashboard_runner.mjs` | 아니오 | `tests/run_studio_ma3_regression_gate_matrix_check.py`, `tests/run_studio_operations_preview_stage_closure_check.py`, `tests/run_studio_release_review_packet_dashboard_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_teacher_feedback_loop_seed_runner.mjs` | 아니오 | `tests/run_studio_teacher_feedback_loop_seed_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/studio_teacher_feedback_surface_preview_runner.mjs` | 아니오 | `tests/run_studio_ma3_regression_gate_matrix_check.py`, `tests/run_studio_operations_preview_stage_closure_check.py`, `tests/run_studio_teacher_feedback_surface_preview_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/toolchain_benchmark_lts_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ta5_benchmark_lts_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/toolchain_diagnostic_ui_lsp_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ta3_diagnostic_ui_lsp_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/toolchain_registry_verification_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ta4_registry_verification_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/ttonimaru_platform_hardening_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ka5_platform_hardening_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/ttonimaru_project_share_ui_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ka3_project_share_ui_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/ttonimaru_public_registry_seed_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ka4_public_registry_seed_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |
| `tests/ttonimaru_publication_read_api_runner.mjs` | 아니오 | `tests/run_roadmap_v2_ka2_publication_read_api_check.py`, `solutions/seamgrim_ui_mvp/ui` | 러너전용 |

## 검증

- `Get-ChildItem solutions/seamgrim_ui_mvp/ui -File -Filter *.js`: 102개.
- `Get-ChildItem tests -File -Filter *.mjs`: 191개.
- 표 행 수 293개로 대상 합계와 일치한다.
- 정적 추적만 수행했으며 파일 수정·삭제는 하지 않았다.
