# BRIEF: DEV_SUMMARY.md 아카이브 분리

> 작성: Claude (2026-07-06) / 실행: Codex / 리뷰: Claude
> 성격: 순수 정리(hygiene). 판단 없음.

## 배경

Q27에서 `docs/context/all/DEV_SUMMARY.md`가 처음 git에 올라갔는데(기존에 미추적 상태로 25,472줄이 쌓여 있었음), 파일 목적("최신 의사결정/검증 결과를 빠르게 확인하는 용도")에 비해 지금 너무 크다.

## 작업

1. `docs/context/all/DEV_SUMMARY.md`를 읽고, 최근 30일(또는 최근 항목 기준 적절한 개수 — 최소 최근 20개 `###` 항목) 이내 항목만 남기고, 나머지는 `docs/context/all/DEV_SUMMARY_ARCHIVE_20260706.md`로 이동한다.
2. 이동 시 항목 순서/내용은 절대 변경하지 않는다 — 잘라서 붙이는 것뿐이다.
3. `DEV_SUMMARY.md` 최상단에 `> 이전 기록은 DEV_SUMMARY_ARCHIVE_20260706.md 참고` 한 줄만 추가한다.
4. 다른 파일이 `DEV_SUMMARY.md`의 특정 옛 항목을 참조하는지 확인한다(`rg` 검색) — 참조가 있으면 그 파일도 새 아카이브 경로를 가리키도록 갱신할지 보고만 하고 수정은 하지 마라(범위 판단은 Claude).

## 검증

- 분리 전/후 두 파일의 총 줄 수 합이 원본과 일치(내용 손실 없음 확인)
- `python tests/run_ci_sanity_gate.py --profile core_lang` PASS

## 수용 기준

- [ ] `DEV_SUMMARY.md`가 최근 항목만 남고 크게 줄어듦
- [ ] `DEV_SUMMARY_ARCHIVE_20260706.md`에 나머지 전부 보존, 내용 손실 없음
- [ ] 참조 파일 있으면 목록만 보고(수정 안 함)
- [ ] sanity gate PASS

## 금지 사항

내용 삭제/수정 없음(이동만). main 직접 커밋 금지.

## 보고 형식

이 파일 하단 `## 실행 보고`: 분리 전/후 줄 수, 남긴 항목 개수, 참조 파일 목록(있으면).

## 실행 보고

- 분리 기준: 원본 순서를 재정렬하지 않고, 첫 2026-06-05 항목(`### [STUDIO][LTS] Benchmark LTS matrix (2026-06-05)`) 직전에서 단일 절단했다.
- 줄 수:
  - 분리 전 `DEV_SUMMARY.md`: 25,472줄
  - 분리 후 `DEV_SUMMARY.md`: 9,084줄
  - 신규 `DEV_SUMMARY_ARCHIVE_20260706.md`: 16,388줄
  - 합계: 25,472줄
- 남긴 `###` 항목 수: 299개
- 아카이브 `###` 항목 수: 519개
- 안내문: `DEV_SUMMARY.md` 최상단 기존 빈 줄 자리에 `> 이전 기록은 DEV_SUMMARY_ARCHIVE_20260706.md 참고`를 넣어 줄 수 합계를 유지했다.
- 참조 검색: `rg -l "DEV_SUMMARY\.md|DEV_SUMMARY|Development Summary|또니랑 Codex 개발 요약" . -g "!docs/context/all/DEV_SUMMARY.md" -g "!docs/context/all/DEV_SUMMARY_ARCHIVE_20260706.md"`
- 참조 파일: 184개(`pack/` 3, `publish/` 1, `tests/` 180). 브리프 지시에 따라 경로 수정은 하지 않았다.

### 참조 파일 목록

- `pack/roadmap_v2_studio_productization_rebase_v1/rebase.detjson`
- `pack/seamgrim_private_productization_consolidation_audit_v1/audit.detjson`
- `pack/studio_long_horizon_completion_audit_v1/audit.detjson`
- `publish/CI_AGGREGATE_GATE_FIX_NOTE_20260223.md`
- `tests/run_lang_connect_lowering_to_seum_check.py`
- `tests/run_lang_connect_seum_lowering_parser_spike_check.py`
- `tests/run_lang_dstrict_dultra_solver_strategy_proposal_check.py`
- `tests/run_lang_dultra_recorded_replay_contract_pack_check.py`
- `tests/run_lang_dultra_replay_artifact_implementation_gate_check.py`
- `tests/run_lang_dultra_replay_artifact_writer_seed_check.py`
- `tests/run_lang_flow_history_alias_migration_plan_check.py`
- `tests/run_lang_flow_type_collision_rename_check.py`
- `tests/run_lang_history_alias_stdlib_bridge_check.py`
- `tests/run_lang_implementation_followup_closure_rebase_check.py`
- `tests/run_lang_implementation_readiness_rebase_check.py`
- `tests/run_lang_language_risk_removal_closure_rebase_check.py`
- `tests/run_lang_owner_inner_seum_parser_boundary_spike_check.py`
- `tests/run_lang_owner_inner_seum_runtime_scope_rebase_check.py`
- `tests/run_lang_owner_inner_seum_structure_check.py`
- `tests/run_lang_owner_state_symbol_table_product_path_check.py`
- `tests/run_lang_prime_derivative_notation_decision_check.py`
- `tests/run_lang_prime_derivative_runtime_semantics_gate_check.py`
- `tests/run_lang_prime_parser_frontdoor_spike_check.py`
- `tests/run_lang_seum_vol3_prime_example_pack_check.py`
- `tests/run_lang_sim_constraint_third_layer_name_check.py`
- `tests/run_lang_tuck_constraint_surface_shape_proposal_check.py`
- `tests/run_lang_velocity_verlet_fixed64_order_pack_check.py`
- `tests/run_lang_velocity_verlet_runtime_gate_rebase_check.py`
- `tests/run_lang_velocity_verlet_stdlib_surface_acceptance_check.py`
- `tests/run_language_design_priority_rebase_check.py`
- `tests/run_linear_inequality_solve_minimum_check.py`
- `tests/run_malblock_authoring_ui_check.py`
- `tests/run_next_work_queue_after_connect_check.py`
- `tests/run_numeric_root_finding_check.py`
- `tests/run_polynomial_solve_minimum_check.py`
- `tests/run_roadmap_v2_a0_nurigym_schema_skeleton_check.py`
- `tests/run_roadmap_v2_a1_nurigym_reset_step_matrix_reconciliation_check.py`
- `tests/run_roadmap_v2_a2_nurigym_representative_environment_matrix_reconciliation_check.py`
- `tests/run_roadmap_v2_a3_nurigym_python_web_parity_check.py`
- `tests/run_roadmap_v2_a4_dataset_registry_check.py`
- `tests/run_roadmap_v2_a5_nurigym_training_workflow_check.py`
- `tests/run_roadmap_v2_da0_math_proof_scope_check.py`
- `tests/run_roadmap_v2_ga3_editor_diagnostic_reconciliation_check.py`
- `tests/run_roadmap_v2_ga4_package_gaji_reconciliation_check.py`
- `tests/run_roadmap_v2_ga5_grammar_lts_behavior_recheck.py`
- `tests/run_roadmap_v2_ga5_grammar_lts_docs_closed_reconciliation_check.py`
- `tests/run_roadmap_v2_geo0_ai_question_card_seed_behavior_reassessment_check.py`
- `tests/run_roadmap_v2_global_4era_plan_check.py`
- `tests/run_roadmap_v2_ha0_education_curriculum_template_matrix_reconciliation_check.py`
- `tests/run_roadmap_v2_ha1_representative_teaching_smoke_check.py`
- `tests/run_roadmap_v2_ja_seulgi_boundary_reconciliation_check.py`
- `tests/run_roadmap_v2_ja0_ai_boundary_behavior_reassessment_check.py`
- `tests/run_roadmap_v2_ja3_seulgi_proposal_ui_check.py`
- `tests/run_roadmap_v2_ja4_model_artifact_share_reconciliation_check.py`
- `tests/run_roadmap_v2_ja5_replay_safe_ai_workflow_check.py`
- `tests/run_roadmap_v2_la0_malblock_design_behavior_reassessment_check.py`
- `tests/run_roadmap_v2_la0_pa0_docs_closed_reconciliation_check.py`
- `tests/run_roadmap_v2_la1_malblock_intro_rail_matrix_reconciliation_check.py`
- `tests/run_roadmap_v2_la3_workbench_integration_reconciliation_check.py`
- `tests/run_roadmap_v2_la4_lesson_package_reconciliation_check.py`
- `tests/run_roadmap_v2_la5_editor_lts_reconciliation_check.py`
- `tests/run_roadmap_v2_ma0_curriculum_catalog_check.py`
- `tests/run_roadmap_v2_ma1_lesson_first_run_reconciliation_check.py`
- `tests/run_roadmap_v2_na0_stdlib_candidate_list_check.py`
- `tests/run_roadmap_v2_na1_post_matrix_frontier_rebase_check.py`
- `tests/run_roadmap_v2_na1_std_core_grid_input_matrix_reconciliation_check.py`
- `tests/run_roadmap_v2_na2_matrix_status_reconciliation_check.py`
- `tests/run_roadmap_v2_na4_stdlib_registry_reconciliation_check.py`
- `tests/run_roadmap_v2_na5_stdlib_lts_reconciliation_check.py`
- `tests/run_roadmap_v2_next_frontier_rebase_check.py`
- `tests/run_roadmap_v2_pa0_social_case_card_behavior_reassessment_check.py`
- `tests/run_roadmap_v2_pa1_baseline_market_first_run_matrix_reconciliation_check.py`
- `tests/run_roadmap_v2_post_a1_frontier_rebase_check.py`
- `tests/run_roadmap_v2_post_ha0_frontier_rebase_check.py`
- `tests/run_roadmap_v2_post_ha1_frontier_rebase_check.py`
- `tests/run_roadmap_v2_post_la1_frontier_rebase_check.py`
- `tests/run_roadmap_v2_post_pa1_frontier_rebase_check.py`
- `tests/run_roadmap_v2_post_sa1_frontier_rebase_check.py`
- `tests/run_roadmap_v2_sa0_bogae_schema_boundary_check.py`
- `tests/run_roadmap_v2_sa1_bogae_graph_space2d_matrix_reconciliation_check.py`
- `tests/run_roadmap_v2_sa2_sprite_grid2d_final_closure_check.py`
- `tests/run_roadmap_v2_sa2_sprite_grid2d_rebase_check.py`
- `tests/run_roadmap_v2_sa3_game_preview_reconciliation_check.py`
- `tests/run_roadmap_v2_sa4_asset_view_share_reconciliation_check.py`
- `tests/run_roadmap_v2_sa5_renderer_hardening_reconciliation_check.py`
- `tests/run_roadmap_v2_studio_productization_rebase_check.py`
- `tests/run_roadmap_v2_ta2_matrix_status_reconciliation_check.py`
- `tests/run_roadmap_v2_ta3_diagnostic_ui_lsp_check.py`
- `tests/run_roadmap_v2_ta4_registry_verification_check.py`
- `tests/run_roadmap_v2_ta5_benchmark_lts_check.py`
- `tests/run_seamgrim_lesson_library_curation_check.py`
- `tests/run_seamgrim_numeric_track_browser_index_check.py`
- `tests/run_seamgrim_numeric_track_consolidation_check.py`
- `tests/run_seamgrim_numeric_track_lesson_preview_check.py`
- `tests/run_seamgrim_numeric_track_report_export_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_export_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_history_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_history_export_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_export_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_export_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_export_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_summary_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_export_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_export_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_summary_check.py`
- `tests/run_seamgrim_numeric_track_result_compare_history_report_table_summary_export_check.py`
- `tests/run_seamgrim_numeric_track_result_history_filter_check.py`
- `tests/run_seamgrim_numeric_track_result_reopen_check.py`
- `tests/run_seamgrim_numeric_track_result_summary_export_check.py`
- `tests/run_seamgrim_numeric_track_result_timeline_view_check.py`
- `tests/run_seamgrim_numeric_track_run_preset_check.py`
- `tests/run_seamgrim_numeric_track_run_result_link_check.py`
- `tests/run_studio_baseline_reassessment_progress_unlock_check.py`
- `tests/run_studio_baseline_rebase_check.py`
- `tests/run_studio_benchmark_baseline_local_snapshot_check.py`
- `tests/run_studio_benchmark_baseline_prep_dry_run_check.py`
- `tests/run_studio_benchmark_lts_matrix_check.py`
- `tests/run_studio_browser_smoke_flake_audit_check.py`
- `tests/run_studio_browser_smoke_matrix_hardening_check.py`
- `tests/run_studio_classroom_mode_check.py`
- `tests/run_studio_classroom_operations_panel_preview_check.py`
- `tests/run_studio_classroom_operations_triage_check.py`
- `tests/run_studio_classroom_report_workflow_check.py`
- `tests/run_studio_diagnostic_fixit_integration_check.py`
- `tests/run_studio_diagnostic_fixit_preview_check.py`
- `tests/run_studio_dirty_baseline_recheck.py`
- `tests/run_studio_doc_index_refresh_check.py`
- `tests/run_studio_education_operations_lts_check.py`
- `tests/run_studio_lesson_authoring_run_integration_check.py`
- `tests/run_studio_lesson_publication_review_surface_check.py`
- `tests/run_studio_local_packaging_consolidation_check.py`
- `tests/run_studio_local_share_and_packaging_check.py`
- `tests/run_studio_ma3_next_development_queue_rebase_check.py`
- `tests/run_studio_ma3_next_queue_coordinate_lock_check.py`
- `tests/run_studio_ma3_regression_gate_matrix_check.py`
- `tests/run_studio_malblock_workbench_integration_check.py`
- `tests/run_studio_next_roadmap_v2_coordinate_lock_check.py`
- `tests/run_studio_numeric_curriculum_track_check.py`
- `tests/run_studio_numeric_report_workflow_consolidation_check.py`
- `tests/run_studio_numeric_result_report_consolidation_check.py`
- `tests/run_studio_operations_preview_stage_closure_check.py`
- `tests/run_studio_post_approval_chain_maintenance_queue_check.py`
- `tests/run_studio_post_release_gate_maintenance_queue_check.py`
- `tests/run_studio_post_super_long_rebase_check.py`
- `tests/run_studio_private_productization_queue_check.py`
- `tests/run_studio_private_productization_rebase_check.py`
- `tests/run_studio_productization_stage_closure_check.py`
- `tests/run_studio_productization_stage_rebase_check.py`
- `tests/run_studio_public_lesson_publication_prep_check.py`
- `tests/run_studio_public_release_approval_recheck_check.py`
- `tests/run_studio_public_release_asset_plan_check.py`
- `tests/run_studio_public_release_execution_gate_check.py`
- `tests/run_studio_public_release_prep_rebase_check.py`
- `tests/run_studio_public_release_smoke_matrix_check.py`
- `tests/run_studio_publication_artifact_dry_run_check.py`
- `tests/run_studio_rc_checker_cost_trim_check.py`
- `tests/run_studio_registry_share_seed_check.py`
- `tests/run_studio_release_approval_chain_closure_check.py`
- `tests/run_studio_release_approval_fast_check.py`
- `tests/run_studio_release_approval_handoff_check.py`
- `tests/run_studio_release_approval_handoff_text_export_check.py`
- `tests/run_studio_release_approval_packet_check.py`
- `tests/run_studio_release_approval_packet_continuity_check.py`
- `tests/run_studio_release_approval_packet_text_export_check.py`
- `tests/run_studio_release_approval_readiness_recheck.py`
- `tests/run_studio_release_approval_status_snapshot_check.py`
- `tests/run_studio_release_approval_wait_state_closure_check.py`
- `tests/run_studio_release_candidate_check.py`
- `tests/run_studio_release_dry_run_text_summary_check.py`
- `tests/run_studio_release_notes_draft_check.py`
- `tests/run_studio_release_notes_text_export_check.py`
- `tests/run_studio_release_pre_execution_dry_run_check.py`
- `tests/run_studio_release_review_packet_dashboard_check.py`
- `tests/run_studio_stale_release_doc_audit_check.py`
- `tests/run_studio_teacher_feedback_loop_seed_check.py`
- `tests/run_studio_teacher_feedback_surface_preview_check.py`

### 검증 결과

- `python tests/run_ci_sanity_gate.py --profile core_lang` PASS