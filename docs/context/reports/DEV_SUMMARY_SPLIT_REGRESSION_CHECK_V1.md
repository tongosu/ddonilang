# DEV_SUMMARY split regression check v1

## Scope

- 목적: Q32의 `docs/context/all/DEV_SUMMARY.md` / `DEV_SUMMARY_ARCHIVE_20260706.md` 분리 이후, 기존 `DEV_SUMMARY` 참조 체커가 새 FAIL로 회귀했는지 실측한다.
- 기준선: `26d8d2b` (Q31, DEV_SUMMARY 분리 직전)
- 현 지점: `d05c7ff` (`codex/queue-20260706`)
- 참조 수집: `rg -l "DEV_SUMMARY\.md|DEV_SUMMARY|Development Summary|또니랑 Codex 개발 요약" . -g "!docs/context/all/DEV_SUMMARY.md" -g "!docs/context/all/DEV_SUMMARY_ARCHIVE_20260706.md"`
- 실행 방식: 참조 파일 184개 중 `tests/run_*.py` 180개를 기준선 worktree와 현 worktree에서 각각 개별 실행했다. 나머지 4개는 실행 파일이 아니므로 표에서 N/A로 표시한다.

## Summary

- 전체 참조 파일: 184개
- 실행 체커: 180개
- 비실행 참조: 4개
- 동일 결과: 167개
- 달라진 결과: 13개
- 새 FAIL: 0개
- 기준선 상태: PASS 5 / FAIL 175 / TIMEOUT 0
- 현 상태: PASS 18 / FAIL 162 / TIMEOUT 0
- `python tests/run_ci_sanity_gate.py --profile core_lang`: PASS (`ci_sanity_status=pass code=OK step=all msg="-" profile=core_lang`)
- 원시 실행 로그: `out/goal-a-dev-summary-split-regression/dev_summary_split_regression_results.json`, `out/goal-a-dev-summary-split-regression/core_lang.log`

## Changed Results

| file | pre-split | post-split | note |
|---|---:|---:|---|
| `tests/run_roadmap_v2_ga4_package_gaji_reconciliation_check.py` | FAIL | PASS | 개선: 기준선 FAIL -> 현 PASS |
| `tests/run_roadmap_v2_global_4era_plan_check.py` | FAIL | PASS | 개선: 기준선 FAIL -> 현 PASS |
| `tests/run_roadmap_v2_ja0_ai_boundary_behavior_reassessment_check.py` | FAIL | PASS | 개선: 기준선 FAIL -> 현 PASS |
| `tests/run_roadmap_v2_ja3_seulgi_proposal_ui_check.py` | FAIL | PASS | 개선: 기준선 FAIL -> 현 PASS |
| `tests/run_roadmap_v2_ja4_model_artifact_share_reconciliation_check.py` | FAIL | PASS | 개선: 기준선 FAIL -> 현 PASS |
| `tests/run_roadmap_v2_ja5_replay_safe_ai_workflow_check.py` | FAIL | PASS | 개선: 기준선 FAIL -> 현 PASS |
| `tests/run_roadmap_v2_ja_seulgi_boundary_reconciliation_check.py` | FAIL | PASS | 개선: 기준선 FAIL -> 현 PASS |
| `tests/run_roadmap_v2_la0_pa0_docs_closed_reconciliation_check.py` | FAIL | PASS | 개선: 기준선 FAIL -> 현 PASS |
| `tests/run_roadmap_v2_na4_stdlib_registry_reconciliation_check.py` | FAIL | PASS | 개선: 기준선 FAIL -> 현 PASS |
| `tests/run_roadmap_v2_pa0_social_case_card_behavior_reassessment_check.py` | FAIL | PASS | 개선: 기준선 FAIL -> 현 PASS |
| `tests/run_roadmap_v2_post_ha0_frontier_rebase_check.py` | FAIL | PASS | 개선: 기준선 FAIL -> 현 PASS |
| `tests/run_roadmap_v2_post_ha1_frontier_rebase_check.py` | FAIL | PASS | 개선: 기준선 FAIL -> 현 PASS |
| `tests/run_studio_publication_artifact_dry_run_check.py` | FAIL | PASS | 개선: 기준선 FAIL -> 현 PASS |

## Regression Judgment

- DEV_SUMMARY 분리로 인한 새 FAIL은 0개다.
- 13개 체커는 기준선에서 FAIL이었고 현 지점에서 PASS로 바뀌었다. 이 변경은 Q32 이후의 현 브랜치 상태를 반영한 개선이며, DEV_SUMMARY 분리 회귀 후보가 아니다.
- 따라서 `DEV_SUMMARY.md` 본문/아카이브 이동 토큰 때문에 새로 깨진 체커는 발견되지 않았다.

## Full Reference Table

| file | pre-split estimated status | post-split measured status | same |
|---|---:|---:|---:|
| `tests/run_lang_connect_lowering_to_seum_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_connect_seum_lowering_parser_spike_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_dstrict_dultra_solver_strategy_proposal_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_dultra_recorded_replay_contract_pack_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_dultra_replay_artifact_implementation_gate_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_dultra_replay_artifact_writer_seed_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_flow_history_alias_migration_plan_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_flow_type_collision_rename_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_history_alias_stdlib_bridge_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_implementation_followup_closure_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_implementation_readiness_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_language_risk_removal_closure_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_owner_inner_seum_parser_boundary_spike_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_owner_inner_seum_runtime_scope_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_owner_inner_seum_structure_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_owner_state_symbol_table_product_path_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_prime_derivative_notation_decision_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_prime_derivative_runtime_semantics_gate_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_prime_parser_frontdoor_spike_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_seum_vol3_prime_example_pack_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_sim_constraint_third_layer_name_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_tuck_constraint_surface_shape_proposal_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_velocity_verlet_fixed64_order_pack_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_velocity_verlet_runtime_gate_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_lang_velocity_verlet_stdlib_surface_acceptance_check.py` | FAIL | FAIL | yes |
| `tests/run_language_design_priority_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_linear_inequality_solve_minimum_check.py` | FAIL | FAIL | yes |
| `tests/run_malblock_authoring_ui_check.py` | FAIL | FAIL | yes |
| `tests/run_next_work_queue_after_connect_check.py` | FAIL | FAIL | yes |
| `tests/run_numeric_root_finding_check.py` | FAIL | FAIL | yes |
| `tests/run_polynomial_solve_minimum_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_a0_nurigym_schema_skeleton_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_a1_nurigym_reset_step_matrix_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_a2_nurigym_representative_environment_matrix_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_a3_nurigym_python_web_parity_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_a4_dataset_registry_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_a5_nurigym_training_workflow_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_da0_math_proof_scope_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_ga3_editor_diagnostic_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_ga4_package_gaji_reconciliation_check.py` | FAIL | PASS | no |
| `tests/run_roadmap_v2_ga5_grammar_lts_behavior_recheck.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_ga5_grammar_lts_docs_closed_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_geo0_ai_question_card_seed_behavior_reassessment_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_global_4era_plan_check.py` | FAIL | PASS | no |
| `tests/run_roadmap_v2_ha0_education_curriculum_template_matrix_reconciliation_check.py` | PASS | PASS | yes |
| `tests/run_roadmap_v2_ha1_representative_teaching_smoke_check.py` | PASS | PASS | yes |
| `tests/run_roadmap_v2_ja0_ai_boundary_behavior_reassessment_check.py` | FAIL | PASS | no |
| `tests/run_roadmap_v2_ja3_seulgi_proposal_ui_check.py` | FAIL | PASS | no |
| `tests/run_roadmap_v2_ja4_model_artifact_share_reconciliation_check.py` | FAIL | PASS | no |
| `tests/run_roadmap_v2_ja5_replay_safe_ai_workflow_check.py` | FAIL | PASS | no |
| `tests/run_roadmap_v2_ja_seulgi_boundary_reconciliation_check.py` | FAIL | PASS | no |
| `tests/run_roadmap_v2_la0_malblock_design_behavior_reassessment_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_la0_pa0_docs_closed_reconciliation_check.py` | FAIL | PASS | no |
| `tests/run_roadmap_v2_la1_malblock_intro_rail_matrix_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_la3_workbench_integration_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_la4_lesson_package_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_la5_editor_lts_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_ma0_curriculum_catalog_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_ma1_lesson_first_run_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_na0_stdlib_candidate_list_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_na1_post_matrix_frontier_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_na1_std_core_grid_input_matrix_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_na2_matrix_status_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_na4_stdlib_registry_reconciliation_check.py` | FAIL | PASS | no |
| `tests/run_roadmap_v2_na5_stdlib_lts_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_next_frontier_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_pa0_social_case_card_behavior_reassessment_check.py` | FAIL | PASS | no |
| `tests/run_roadmap_v2_pa1_baseline_market_first_run_matrix_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_post_a1_frontier_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_post_ha0_frontier_rebase_check.py` | FAIL | PASS | no |
| `tests/run_roadmap_v2_post_ha1_frontier_rebase_check.py` | FAIL | PASS | no |
| `tests/run_roadmap_v2_post_la1_frontier_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_post_pa1_frontier_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_post_sa1_frontier_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_sa0_bogae_schema_boundary_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_sa1_bogae_graph_space2d_matrix_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_sa2_sprite_grid2d_final_closure_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_sa2_sprite_grid2d_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_sa3_game_preview_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_sa4_asset_view_share_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_sa5_renderer_hardening_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_studio_productization_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_ta2_matrix_status_reconciliation_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_ta3_diagnostic_ui_lsp_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_ta4_registry_verification_check.py` | FAIL | FAIL | yes |
| `tests/run_roadmap_v2_ta5_benchmark_lts_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_lesson_library_curation_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_browser_index_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_consolidation_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_lesson_preview_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_report_export_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_export_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_history_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_history_export_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_history_report_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_history_report_export_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_history_report_table_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_history_report_table_export_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_export_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_summary_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_badge_export_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_history_report_table_status_export_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_history_report_table_summary_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_compare_history_report_table_summary_export_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_history_filter_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_reopen_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_summary_export_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_result_timeline_view_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_run_preset_check.py` | FAIL | FAIL | yes |
| `tests/run_seamgrim_numeric_track_run_result_link_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_baseline_reassessment_progress_unlock_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_baseline_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_benchmark_baseline_local_snapshot_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_benchmark_baseline_prep_dry_run_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_benchmark_lts_matrix_check.py` | PASS | PASS | yes |
| `tests/run_studio_browser_smoke_flake_audit_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_browser_smoke_matrix_hardening_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_classroom_mode_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_classroom_operations_panel_preview_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_classroom_operations_triage_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_classroom_report_workflow_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_diagnostic_fixit_integration_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_diagnostic_fixit_preview_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_dirty_baseline_recheck.py` | FAIL | FAIL | yes |
| `tests/run_studio_doc_index_refresh_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_education_operations_lts_check.py` | PASS | PASS | yes |
| `tests/run_studio_lesson_authoring_run_integration_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_lesson_publication_review_surface_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_local_packaging_consolidation_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_local_share_and_packaging_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_ma3_next_development_queue_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_ma3_next_queue_coordinate_lock_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_ma3_regression_gate_matrix_check.py` | PASS | PASS | yes |
| `tests/run_studio_malblock_workbench_integration_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_next_roadmap_v2_coordinate_lock_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_numeric_curriculum_track_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_numeric_report_workflow_consolidation_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_numeric_result_report_consolidation_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_operations_preview_stage_closure_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_post_approval_chain_maintenance_queue_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_post_release_gate_maintenance_queue_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_post_super_long_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_private_productization_queue_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_private_productization_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_productization_stage_closure_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_productization_stage_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_public_lesson_publication_prep_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_public_release_approval_recheck_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_public_release_asset_plan_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_public_release_execution_gate_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_public_release_prep_rebase_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_public_release_smoke_matrix_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_publication_artifact_dry_run_check.py` | FAIL | PASS | no |
| `tests/run_studio_rc_checker_cost_trim_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_registry_share_seed_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_release_approval_chain_closure_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_release_approval_fast_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_release_approval_handoff_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_release_approval_handoff_text_export_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_release_approval_packet_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_release_approval_packet_continuity_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_release_approval_packet_text_export_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_release_approval_readiness_recheck.py` | FAIL | FAIL | yes |
| `tests/run_studio_release_approval_status_snapshot_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_release_approval_wait_state_closure_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_release_candidate_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_release_dry_run_text_summary_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_release_notes_draft_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_release_notes_text_export_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_release_pre_execution_dry_run_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_release_review_packet_dashboard_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_stale_release_doc_audit_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_teacher_feedback_loop_seed_check.py` | FAIL | FAIL | yes |
| `tests/run_studio_teacher_feedback_surface_preview_check.py` | FAIL | FAIL | yes |
| `pack/roadmap_v2_studio_productization_rebase_v1/rebase.detjson` | N/A (non-executable reference) | N/A (non-executable reference) | N/A |
| `pack/seamgrim_private_productization_consolidation_audit_v1/audit.detjson` | N/A (non-executable reference) | N/A (non-executable reference) | N/A |
| `pack/studio_long_horizon_completion_audit_v1/audit.detjson` | N/A (non-executable reference) | N/A (non-executable reference) | N/A |
| `publish/CI_AGGREGATE_GATE_FIX_NOTE_20260223.md` | N/A (non-executable reference) | N/A (non-executable reference) | N/A |

