# BROKEN_CHECKS_ROOT_CAUSE_V1

## 범위

- Q15 산출물이다.
- 입력은 `docs/context/reports/BROKEN_CHECKS_AUDIT_V1.md`의 Q3 표이며, 기존 보고서는 수정하지 않았다.
- Q3 기준 누락 `.md` 필수 참조 후보 체크 파일 173개를 전수로 다시 분해했다.
- 생성 이력은 `git log --all --diff-filter=A --name-only --format=...`와 `git log --all --oneline -- <파일명>`을 함께 사용해 확인했다.
- 삭제 커밋은 `git log --all --diff-filter=D --name-only --format=...`의 basename 일치로 특정했다.
- 파일 수정·삭제·수리는 실행하지 않았다.

## 요약

- Q3 체크 파일 수: 173개
- 체크-문서 참조 행 수: 225개
- 고유 참조 문서 수: 176개
- 생성 이력 있음 문서 수: 0개
- 생성 이력 없음 문서 수: 176개
- 참조 행 기준 생성후삭제: 0개
- 참조 행 기준 계획후미실행: 225개
- 체크 파일 기준 생성후삭제: 0개
- 체크 파일 기준 계획후미실행: 173개
- 체크 파일 기준 혼합: 0개

## 해석

- `생성후삭제`는 같은 basename의 `.md`가 과거 커밋에서 생성된 이력이 있고 현재 Q3 기준 참조 위치에는 없다는 뜻이다. 루트 문서가 `docs/context/queue/` 등으로 이동된 경우도 이 분류에 포함된다.
- `계획후미실행`은 Q3가 참조한 basename에 대해 생성 이력과 일반 이력 모두 찾지 못한 경우다. 체크 스크립트가 선행 계획 문서를 가정했지만 문서가 실제로 만들어진 적은 없었던 후보로 본다.
- 한 체크 파일이 생성 이력 있는 문서와 없는 문서를 동시에 참조하면 체크 파일 분류는 `혼합`으로 집계했다.

## 표

| 체크 파일 | 참조 문서 | 생성 이력(있음/없음) | 삭제 커밋(있으면) | 분류(계획후미실행/생성후삭제) | 연관 추정 작업 |
|---|---|---|---|---|---|
| `tests/run_age5_ci_frontier_recheck.py` | `AGE5_CI_FRONTIER_RECHECK_V1.md` (DOC L8) | 없음 | - | 계획후미실행 | AGE5/CI 트랙 |
| `tests/run_age5_ci_frontier_recheck.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L9) | 없음 | - | 계획후미실행 | AGE5/CI 트랙 |
| `tests/run_block_editor_roundtrip_expected_refresh_check.py` | `BLOCK_EDITOR_ROUNDTRIP_EXPECTED_REFRESH_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | Block editor 트랙 |
| `tests/run_block_editor_roundtrip_expected_refresh_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L10) | 없음 | - | 계획후미실행 | Block editor 트랙 |
| `tests/run_constraint_solve_rebase_check.py` | `CONSTRAINT_SOLVE_REBASE_V1.md` (DOC L12) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_constraint_solve_rebase_check.py` | `NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md` (ROADMAP L11) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_lang_blocked_ssot_track_parking_rebase_check.py` | `LANG_BLOCKED_SSOT_TRACK_PARKING_REBASE_V1.md` (DOC L11) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_connect_lowering_to_seum_check.py` | `LANG_CONNECT_LOWERING_TO_SEUM_CHECK_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_connect_seum_lowering_parser_spike_check.py` | `LANG_CONNECT_SEUM_LOWERING_PARSER_SPIKE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_core_2_representative_grammar_pack_check.py` | `LANG_CORE_2_REPRESENTATIVE_GRAMMAR_PACK_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_core_2_representative_grammar_pack_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L11) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_lang_dstrict_dultra_solver_strategy_proposal_check.py` | `LANG_DSTRICT_DULTRA_SOLVER_STRATEGY_PROPOSAL_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_dultra_recorded_replay_contract_pack_check.py` | `LANG_DULTRA_RECORDED_REPLAY_CONTRACT_PACK_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_dultra_replay_artifact_implementation_gate_check.py` | `LANG_DULTRA_REPLAY_ARTIFACT_IMPLEMENTATION_GATE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_dultra_replay_artifact_writer_seed_check.py` | `LANG_DULTRA_REPLAY_ARTIFACT_WRITER_SEED_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_flow_history_alias_migration_plan_check.py` | `LANG_FLOW_HISTORY_ALIAS_MIGRATION_PLAN_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_flow_history_ssot_acceptance_request_check.py` | `LANG_FLOW_HISTORY_SSOT_ACCEPTANCE_REQUEST_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_flow_type_collision_rename_check.py` | `LANG_FLOW_TYPE_COLLISION_RENAME_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_history_alias_stdlib_bridge_check.py` | `LANG_HISTORY_ALIAS_STDLIB_BRIDGE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_implementation_followup_closure_rebase_check.py` | `LANG_IMPLEMENTATION_FOLLOWUP_CLOSURE_REBASE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_implementation_readiness_rebase_check.py` | `LANG_IMPLEMENTATION_READINESS_REBASE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_language_risk_removal_closure_rebase_check.py` | `LANG_LANGUAGE_RISK_REMOVAL_CLOSURE_REBASE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_non_ssot_track_resume_rebase_check.py` | `LANG_NON_SSOT_TRACK_RESUME_REBASE_V1.md` (DOC L11) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_owner_inner_seum_parser_boundary_spike_check.py` | `LANG_OWNER_INNER_SEUM_PARSER_BOUNDARY_SPIKE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_owner_inner_seum_runtime_scope_rebase_check.py` | `LANG_OWNER_INNER_SEUM_RUNTIME_SCOPE_REBASE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_owner_inner_seum_structure_check.py` | `LANG_OWNER_INNER_SEUM_STRUCTURE_CHECK_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_owner_state_symbol_table_product_path_check.py` | `LANG_OWNER_STATE_SYMBOL_TABLE_PRODUCT_PATH_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_pack_checker_stability_sweep_check.py` | `LANG_PACK_CHECKER_STABILITY_SWEEP_V1.md` (DOC L11) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_post_ssot_landing_product_gate_rebase_check.py` | `LANG_POST_SSOT_LANDING_PRODUCT_GATE_REBASE_V1.md` (DOC L11) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_prime_derivative_notation_decision_check.py` | `LANG_PRIME_DERIVATIVE_NOTATION_DECISION_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_prime_derivative_runtime_semantics_gate_check.py` | `LANG_PRIME_DERIVATIVE_RUNTIME_SEMANTICS_GATE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_prime_parser_frontdoor_spike_check.py` | `LANG_PRIME_PARSER_FRONTDOOR_SPIKE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_prime_ssot_acceptance_request_check.py` | `LANG_PRIME_SSOT_ACCEPTANCE_REQUEST_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_product_path_transition_closure_rebase_check.py` | `LANG_PRODUCT_PATH_TRANSITION_CLOSURE_REBASE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_seum_vol3_prime_example_pack_check.py` | `LANG_SEUM_VOL3_PRIME_EXAMPLE_PACK_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_sim_constraint_third_layer_name_check.py` | `LANG_SIM_CONSTRAINT_THIRD_LAYER_NAME_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_ssot_acceptance_request_closure_rebase_check.py` | `LANG_SSOT_ACCEPTANCE_REQUEST_CLOSURE_REBASE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_ssot_landing_coordination_rebase_check.py` | `LANG_SSOT_LANDING_COORDINATION_REBASE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_ssot_landing_wait_state_rebase_check.py` | `LANG_SSOT_LANDING_WAIT_STATE_REBASE_V1.md` (DOC L11) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_ssot_owner_landing_audit_rebase_check.py` | `LANG_SSOT_OWNER_LANDING_AUDIT_REBASE_V1.md` (DOC L11) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_ssot_owner_landing_handoff_check.py` | `LANG_SSOT_OWNER_LANDING_HANDOFF_V1.md` (DOC L11) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_tuck_constraint_surface_shape_proposal_check.py` | `LANG_TUCK_CONSTRAINT_SURFACE_SHAPE_PROPOSAL_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_tuck_ssot_acceptance_handoff_check.py` | `LANG_TUCK_SSOT_ACCEPTANCE_HANDOFF_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_tuck_ssot_acceptance_request_check.py` | `LANG_TUCK_SSOT_ACCEPTANCE_REQUEST_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_lang_velocity_verlet_fixed64_order_pack_check.py` | `LANG_VELOCITY_VERLET_FIXED64_ORDER_PACK_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_lang_velocity_verlet_runtime_gate_rebase_check.py` | `LANG_VELOCITY_VERLET_RUNTIME_GATE_REBASE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_lang_velocity_verlet_stdlib_surface_acceptance_check.py` | `LANG_VELOCITY_VERLET_STDLIB_SURFACE_ACCEPTANCE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_language_design_priority_rebase_check.py` | `LANGUAGE_DESIGN_PRIORITY_REBASE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Lang/SSOT 트랙 |
| `tests/run_malblock_authoring_ui_check.py` | `MALBLOCK_AUTHORING_UI_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | Malblock/Authoring 트랙 |
| `tests/run_malblock_authoring_ui_check.py` | `SEAMGRIM_LESSON_AUTHORING_FLOW_V1.md` (PREV L11) | 없음 | - | 계획후미실행 | Malblock/Authoring 트랙 |
| `tests/run_math_calculus_computed_output_refresh_check.py` | `MATH_CALCULUS_COMPUTED_OUTPUT_REFRESH_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_math_calculus_computed_output_refresh_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L11) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_math_vector_minimum_first_run_check.py` | `MATH_VECTOR_MINIMUM_FIRST_RUN_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_math_vector_minimum_first_run_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L11) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_next_work_queue_after_connect_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L9) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_numeric_root_finding_check.py` | `NUMERIC_ROOT_FINDING_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_numeric_root_finding_check.py` | `NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md` (ROADMAP L10) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_numeric_solver_capability_rebase_check.py` | `NUMERIC_SOLVER_CAPABILITY_REBASE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_numeric_solver_capability_rebase_check.py` | `NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md` (ROADMAP L9) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_nurigym_bandit_minimum_pack_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L12) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_nurigym_bandit_minimum_pack_check.py` | `NURIGYM_BANDIT_MINIMUM_PACK_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | NuriGym 트랙 |
| `tests/run_nurigym_cartpole_pendulum_expected_refresh_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L10) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_nurigym_cartpole_pendulum_expected_refresh_check.py` | `NURIGYM_CARTPOLE_PENDULUM_EXPECTED_REFRESH_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | NuriGym 트랙 |
| `tests/run_nurigym_contract_gridworld_expected_refresh_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L11) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_nurigym_contract_gridworld_expected_refresh_check.py` | `NURIGYM_CONTRACT_GRIDWORLD_EXPECTED_REFRESH_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | NuriGym 트랙 |
| `tests/run_nurigym_contract_gridworld_expected_refresh_check.py` | `ROADMAP_V2_A2_NURIGYM_REPRESENTATIVE_ENV_REBASE_V1.md` (REBASE_DOC L10) | 없음 | - | 계획후미실행 | NuriGym 트랙 |
| `tests/run_nurigym_dataset_hash_expected_refresh_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L11) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_nurigym_dataset_hash_expected_refresh_check.py` | `NURIGYM_DATASET_HASH_EXPECTED_REFRESH_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | NuriGym 트랙 |
| `tests/run_nurigym_dataset_hash_expected_refresh_check.py` | `ROADMAP_V2_A1_NURIGYM_REBASE_V1.md` (REBASE_DOC L10) | 없음 | - | 계획후미실행 | NuriGym 트랙 |
| `tests/run_ode_method_comparison_check.py` | `NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md` (ROADMAP L11) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_ode_method_comparison_check.py` | `ODE_METHOD_COMPARISON_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_ode_method_comparison_check.py` | `ODE_TICK_LOOP_LESSON_BASELINE_V1.md` (PREV L10) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_ode_tick_loop_lesson_baseline_check.py` | `NUMERIC_SOLVER_CAPABILITY_REBASE_V1.md` (PREV L10) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_ode_tick_loop_lesson_baseline_check.py` | `NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md` (ROADMAP L11) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_ode_tick_loop_lesson_baseline_check.py` | `ODE_TICK_LOOP_LESSON_BASELINE_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_roadmap_v2_a1_final_closure_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L11) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_roadmap_v2_a1_final_closure_check.py` | `ROADMAP_V2_A1_NURIGYM_FINAL_CLOSURE_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | NuriGym 트랙 |
| `tests/run_roadmap_v2_a2_bandit_or_final_recheck.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L11) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_roadmap_v2_a2_bandit_or_final_recheck.py` | `ROADMAP_V2_A2_NURIGYM_BANDIT_OR_FINAL_RECHECK_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | NuriGym 트랙 |
| `tests/run_roadmap_v2_a2_final_closure_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L11) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_roadmap_v2_a2_final_closure_check.py` | `ROADMAP_V2_A2_NURIGYM_FINAL_CLOSURE_RECHECK_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | NuriGym 트랙 |
| `tests/run_roadmap_v2_a2_nurigym_rebase_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L11) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_roadmap_v2_a2_nurigym_rebase_check.py` | `ROADMAP_V2_A2_NURIGYM_REPRESENTATIVE_ENV_REBASE_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | NuriGym 트랙 |
| `tests/run_roadmap_v2_ba0_free_lab_seed_reassessment_check.py` | `BA0_FREE_LAB_SEED_REASSESSMENT_V1.md` (DOC L14) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ba0_free_lab_seed_rebase_check.py` | `BA0_FREE_LAB_SEED_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_cha0_rpg_seed_reassessment_check.py` | `CHA0_RPG_SEED_REASSESSMENT_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_cha0_rpg_seed_rebase_check.py` | `CHA0_RPG_SEED_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_cha1_rpg_phrase_action_smoke_check.py` | `CHA1_RPG_PHRASE_ACTION_SMOKE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_cha2_rpg_story_pack_closure_check.py` | `CHA2_RPG_STORY_PACK_CLOSURE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_cha3_rpg_box_authoring_ui_check.py` | `CHA3_RPG_BOX_NURIMAKER_AUTHORING_UI_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_cha4_rpg_story_package_check.py` | `CHA4_RPG_STORY_PACKAGE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_cha5_rpg_engine_adapter_lts_check.py` | `CHA5_RPG_ENGINE_ADAPTER_LTS_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_da1_final_closure_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L9) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_roadmap_v2_da1_final_closure_check.py` | `ROADMAP_V2_DA1_FINAL_CLOSURE_V1.md` (DOC L8) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_da1_math_closure_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L9) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_roadmap_v2_da1_math_closure_check.py` | `ROADMAP_V2_DA1_MATH_CLOSURE_V1.md` (DOC L8) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_roadmap_v2_da1_math_first_run_frontier_rebase_check.py` | `ROADMAP_V2_DA1_MATH_FIRST_RUN_FRONTIER_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_roadmap_v2_da1_math_rebase_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L10) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_roadmap_v2_da1_math_rebase_check.py` | `ROADMAP_V2_DA1_MATH_REBASE_V1.md` (REBASE L9) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_roadmap_v2_da2_symbolic_solve_proof_frontier_rebase_check.py` | `ROADMAP_V2_DA2_SYMBOLIC_SOLVE_PROOF_FRONTIER_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_da3_seamgrim_math_view_frontier_rebase_check.py` | `ROADMAP_V2_DA3_SEAMGRIM_MATH_VIEW_FRONTIER_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_roadmap_v2_da4_math_package_share_frontier_rebase_check.py` | `ROADMAP_V2_DA4_MATH_PACKAGE_SHARE_FRONTIER_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_roadmap_v2_da5_math_lts_frontier_rebase_check.py` | `ROADMAP_V2_DA5_MATH_LTS_FRONTIER_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_roadmap_v2_followon_rebase_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L9) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_roadmap_v2_followon_rebase_check.py` | `ROADMAP_V2_FOLLOWON_REBASE_V1.md` (REBASE L8) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ga0_current_line_ledger_matrix_reconciliation_check.py` | `ROADMAP_V2_GA0_CURRENT_LINE_LEDGER_MATRIX_RECONCILIATION_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ga1_core_smoke_matrix_reconciliation_check.py` | `ROADMAP_V2_GA1_CORE_SMOKE_MATRIX_RECONCILIATION_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ga2_final_closure_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L10) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_roadmap_v2_ga2_final_closure_check.py` | `ROADMAP_V2_GA2_FINAL_CLOSURE_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ga2_matrix_status_reconciliation_check.py` | `GA2_MATRIX_STATUS_RECONCILIATION_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ga2_representative_grammar_rebase_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L9) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_roadmap_v2_ga2_representative_grammar_rebase_check.py` | `ROADMAP_V2_GA2_REPRESENTATIVE_GRAMMAR_REBASE_V1.md` (DOC L8) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ga5_release_gate_behavior_closure.py` | `GA5_RELEASE_GATE_BEHAVIOR_CLOSURE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ga5_release_gate_blocker_audit.py` | `GA5_RELEASE_GATE_BLOCKER_AUDIT_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_geo0_ai_question_card_seed_reconciliation_check.py` | `GEO0_AI_QUESTION_CARD_SEED_RECONCILIATION_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_geo1_question_card_smoke_check.py` | `GEO1_QUESTION_CARD_SMOKE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_geo2_ai_output_validation_pack_check.py` | `GEO2_AI_OUTPUT_VALIDATION_PACK_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_geo3_dev_assist_ui_check.py` | `GEO3_DEV_ASSIST_UI_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_geo4_author_tool_share_check.py` | `GEO4_AUTHOR_TOOL_SHARE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_geo5_ai_workflow_hardening_check.py` | `GEO5_AI_WORKFLOW_HARDENING_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ha2_education_assessment_pack_check.py` | `HA2_EDUCATION_ASSESSMENT_PACK_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ha3_classroom_ui_pack_check.py` | `HA3_CLASSROOM_UI_PACK_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ha4_public_course_publication_pack_check.py` | `HA4_PUBLIC_COURSE_PUBLICATION_PACK_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ha5_education_operations_lts_check.py` | `HA5_EDUCATION_OPERATIONS_LTS_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ka0_platform_charter_matrix_reconciliation_check.py` | `ROADMAP_V2_KA0_PLATFORM_CHARTER_MATRIX_RECONCILIATION_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ka1_server_mvp_matrix_reconciliation_check.py` | `ROADMAP_V2_KA1_SERVER_MVP_MATRIX_RECONCILIATION_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ka2_publication_read_api_check.py` | `KA2_PUBLICATION_READ_API_CLOSURE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ka3_project_share_ui_check.py` | `KA3_PROJECT_SHARE_UI_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ka4_public_registry_seed_check.py` | `KA4_PUBLIC_REGISTRY_SEED_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ka5_platform_hardening_check.py` | `KA5_PLATFORM_HARDENING_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_la2_final_closure_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L11) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_roadmap_v2_la2_final_closure_check.py` | `ROADMAP_V2_LA2_FINAL_CLOSURE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_la2_matrix_status_reconciliation_check.py` | `LA2_MATRIX_STATUS_RECONCILIATION_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_la2_subset_roundtrip_rebase_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L10) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_roadmap_v2_la2_subset_roundtrip_rebase_check.py` | `ROADMAP_V2_LA2_SUBSET_ROUNDTRIP_REBASE_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ma2_seamgrim_curriculum_2_pack_closure_check.py` | `MA2_SEAMGRIM_CURRICULUM_2_PACK_CLOSURE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ma2_studio_prereq_unlock_check.py` | `MA2_STUDIO_PREREQ_UNLOCK_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ma3_seamgrim_curriculum_3_classroom_ui_pack_closure_check.py` | `MA3_SEAMGRIM_CURRICULUM_3_CLASSROOM_UI_PACK_CLOSURE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ma3_studio_classroom_workbench_prereq_rebase_check.py` | `MA3_STUDIO_CLASSROOM_WORKBENCH_PREREQ_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ma4_public_lesson_publication_prereq_rebase_check.py` | `MA4_PUBLIC_LESSON_PUBLICATION_PREREQ_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ma4_seamgrim_curriculum_4_publication_pack_closure_check.py` | `MA4_SEAMGRIM_CURRICULUM_4_PUBLICATION_PACK_CLOSURE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ma5_curriculum_lts_prereq_rebase_check.py` | `MA5_CURRICULUM_LTS_PREREQ_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ma5_seamgrim_curriculum_5_lts_pack_closure_check.py` | `MA5_SEAMGRIM_CURRICULUM_5_LTS_PACK_CLOSURE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_na2_event_minimum_closure_rebase_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L10) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_roadmap_v2_na2_event_minimum_closure_rebase_check.py` | `ROADMAP_V2_NA2_EVENT_MINIMUM_CLOSURE_REBASE_V1.md` (REBASE L8) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_na2_event_minimum_closure_rebase_check.py` | `STD_EVENT_MINIMUM_CLOSURE_V1.md` (EVENT_CLOSURE L9) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_na2_matrix_status_reconciliation_check.py` | `ROADMAP_V2_NA2_MATRIX_STATUS_RECONCILIATION_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_na2_unit_random_event_rebase_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L9) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_roadmap_v2_na2_unit_random_event_rebase_check.py` | `ROADMAP_V2_NA2_UNIT_RANDOM_EVENT_REBASE_V1.md` (REBASE L8) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_na3_matrix_status_reconciliation_check.py` | `ROADMAP_V2_NA3_MATRIX_STATUS_RECONCILIATION_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_na3_resource_network_policy_rebase_check.py` | `ROADMAP_V2_NA3_MATRIX_STATUS_RECONCILIATION_V1.md` (NA3_RECONCILIATION_DOC L23) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_na3_resource_network_policy_rebase_check.py` | `ROADMAP_V2_NA3_RESOURCE_NETWORK_POLICY_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_next_frontier_rebase_check.py` | `ROADMAP_V2_NEXT_FRONTIER_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_pa2_social_bridge_pack_check.py` | `PA2_SOCIAL_BRIDGE_PACK_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_pa3_policy_ghost_ui_check.py` | `PA3_POLICY_GHOST_UI_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_pa4_social_template_registry_check.py` | `PA4_SOCIAL_TEMPLATE_REGISTRY_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_pa5_social_world_lts_check.py` | `PA5_SOCIAL_WORLD_LTS_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_post_da5_frontier_rebase_check.py` | `ROADMAP_V2_POST_DA5_FRONTIER_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_post_ga0_frontier_rebase_check.py` | `ROADMAP_V2_POST_GA0_FRONTIER_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_post_ga1_frontier_rebase_check.py` | `ROADMAP_V2_POST_GA1_FRONTIER_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_post_ka0_frontier_rebase_check.py` | `ROADMAP_V2_POST_KA0_FRONTIER_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_post_ka1_frontier_rebase_check.py` | `ROADMAP_V2_POST_KA1_FRONTIER_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_post_la2_frontier_rebase_check.py` | `ROADMAP_V2_POST_LA2_FRONTIER_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_post_na3_frontier_rebase_check.py` | `ROADMAP_V2_KA1_SERVER_MVP_MATRIX_RECONCILIATION_V1.md` (KA1_RECONCILIATION_DOC L22) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_post_na3_frontier_rebase_check.py` | `ROADMAP_V2_POST_NA3_FRONTIER_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_post_priority_rebase_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L9) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_roadmap_v2_post_priority_rebase_check.py` | `ROADMAP_V2_POST_PRIORITY_REBASE_V1.md` (DOC L8) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_post_ta0_frontier_rebase_check.py` | `ROADMAP_V2_POST_TA0_FRONTIER_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_post_ta1_frontier_rebase_check.py` | `ROADMAP_V2_POST_TA1_FRONTIER_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_post_ta2_frontier_rebase_check.py` | `ROADMAP_V2_POST_TA2_FRONTIER_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_sa1_rebase_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L9) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_roadmap_v2_sa1_rebase_check.py` | `ROADMAP_V2_SA1_REBASE_V1.md` (REBASE L8) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_sa2_sprite_grid2d_final_closure_check.py` | `SA2_SPRITE_GRID2D_FINAL_CLOSURE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_sa2_sprite_grid2d_rebase_check.py` | `SA2_SPRITE_GRID2D_CLOSURE_REBASE_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_studio_productization_rebase_check.py` | `NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md` (NUMERIC_ROADMAP L15) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_roadmap_v2_ta0_pack_checker_skeleton_matrix_reconciliation_check.py` | `ROADMAP_V2_TA0_PACK_CHECKER_SKELETON_MATRIX_RECONCILIATION_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ta1_pack_runner_basis_matrix_reconciliation_check.py` | `ROADMAP_V2_TA1_PACK_RUNNER_BASIS_MATRIX_RECONCILIATION_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ta2_guide_status_reconciliation_check.py` | `ROADMAP_V2_TA2_GUIDE_STATUS_RECONCILIATION_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ta2_matrix_status_reconciliation_check.py` | `TA2_MATRIX_STATUS_RECONCILIATION_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ta3_diagnostic_ui_lsp_check.py` | `TA3_DIAGNOSTIC_UI_LSP_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ta4_registry_verification_check.py` | `TA4_REGISTRY_VERIFICATION_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_roadmap_v2_ta5_benchmark_lts_check.py` | `TA5_BENCHMARK_LTS_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_root_low_risk_retire_delete_approval_gate_blocked_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L13) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_root_low_risk_retire_delete_approval_gate_blocked_check.py` | `ROOT_LOW_RISK_RETIRE_APPROVAL_v1.md` (APPROVAL L10) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_root_low_risk_retire_delete_approval_gate_blocked_check.py` | `ROOT_LOW_RISK_RETIRE_DELETE_APPROVAL_GATE_BLOCKED_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_root_low_risk_retire_delete_approval_gate_blocked_check.py` | `ROOT_LOW_RISK_RETIRE_DELETE_DRY_RUN_PLAN_V1.md` (DRY_RUN L12) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_root_low_risk_retire_delete_approval_gate_blocked_check.py` | `ROOT_LOW_RISK_RETIRE_DELETE_PREFLIGHT_V1.md` (PREFLIGHT L11) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_root_low_risk_retire_delete_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L10) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_root_low_risk_retire_delete_check.py` | `ROOT_LOW_RISK_RETIRE_DELETE_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_root_low_risk_retire_delete_dry_run_plan_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L13) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_root_low_risk_retire_delete_dry_run_plan_check.py` | `ROOT_LOW_RISK_RETIRE_DELETE_DRY_RUN_PLAN_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_root_low_risk_retire_delete_dry_run_plan_check.py` | `ROOT_LOW_RISK_RETIRE_DELETE_PREFLIGHT_V1.md` (PREFLIGHT_DOC L11) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_root_low_risk_retire_delete_preflight_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L11) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_root_low_risk_retire_delete_preflight_check.py` | `ROOT_LOW_RISK_RETIRE_APPROVAL_v1.md` (APPROVAL L10) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_root_low_risk_retire_delete_preflight_check.py` | `ROOT_LOW_RISK_RETIRE_DELETE_PREFLIGHT_V1.md` (DOC L9) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_sa2_sprite_skin_minimum_pack_check.py` | `SA2_SPRITE_SKIN_MINIMUM_PACK_V1.md` (DOC L17) | 없음 | - | 계획후미실행 | 기타 |
| `tests/run_seamgrim_baseline_stabilization_closure_rebase_check.py` | `SEAMGRIM_BASELINE_STABILIZATION_CLOSURE_REBASE_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_seamgrim_cli_warning_parity_repair_check.py` | `SEAMGRIM_CLI_WARNING_PARITY_REPAIR_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_seamgrim_lesson_library_curation_check.py` | `SEAMGRIM_LESSON_LIBRARY_CURATION_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_seamgrim_lesson_library_curation_check.py` | `SEAMGRIM_WORKBENCH_POLISH_V2.md` (PREV L12) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_seamgrim_malblock_roundtrip_subset_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L17) | 없음 | - | 계획후미실행 | Malblock/Authoring 트랙 |
| `tests/run_seamgrim_malblock_roundtrip_subset_queue_guard_repair_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (QUEUE L15) | 없음 | - | 계획후미실행 | Malblock/Authoring 트랙 |
| `tests/run_seamgrim_malblock_roundtrip_subset_queue_guard_repair_check.py` | `SEAMGRIM_MALBLOCK_ROUNDTRIP_SUBSET_QUEUE_GUARD_REPAIR_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | Malblock/Authoring 트랙 |
| `tests/run_seamgrim_private_productization_consolidation_audit.py` | `SEAMGRIM_PRIVATE_PRODUCTIZATION_CONSOLIDATION_AUDIT_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_seamgrim_private_productization_next_queue_recheck.py` | `SEAMGRIM_PRIVATE_PRODUCTIZATION_NEXT_QUEUE_RECHECK_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_seamgrim_run_history_export_summary_followup_recheck.py` | `SEAMGRIM_RUN_HISTORY_EXPORT_SUMMARY_FOLLOWUP_RECHECK_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_seamgrim_runtime_5min_baseline_repair_check.py` | `SEAMGRIM_RUNTIME_5MIN_BASELINE_REPAIR_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_seamgrim_wasm_tool_golden_baseline_repair_check.py` | `SEAMGRIM_WASM_TOOL_GOLDEN_BASELINE_REPAIR_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_std_resource_network_policy_minimum_check.py` | `ROADMAP_V2_NA3_MATRIX_STATUS_RECONCILIATION_V1.md` (NA3_RECONCILIATION_DOC L23) | 없음 | - | 계획후미실행 | Roadmap V2 재정렬 트랙 |
| `tests/run_std_resource_network_policy_minimum_check.py` | `STD_RESOURCE_NETWORK_POLICY_MINIMUM_V1.md` (DOC L13) | 없음 | - | 계획후미실행 | 기타 |
| `tests/run_studio_baseline_rebase_check.py` | `CONNECT_ENDPOINT_SOLVE_RANGE_CASE_SUITE_CHECK_RUNNER_V1V.md` (RETIRED_ROOT_DOCS L12) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_studio_baseline_rebase_check.py` | `NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md` (RETIRED_ROOT_DOCS L12) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_studio_baseline_rebase_check.py` | `ROOT_LOW_RISK_RETIRE_DELETE_V1.md` (RETIRED_ROOT_DOCS L12) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |
| `tests/run_studio_classroom_mode_switch_check.py` | `STUDIO_CLASSROOM_MODE_SWITCH_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_studio_classroom_report_export_action_check.py` | `STUDIO_CLASSROOM_REPORT_EXPORT_ACTION_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_studio_diagnostic_fixit_editor_apply_check.py` | `STUDIO_DIAGNOSTIC_FIXIT_EDITOR_APPLY_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_studio_historical_progress_ledger_reconciliation_check.py` | `STUDIO_HISTORICAL_PROGRESS_LEDGER_RECONCILIATION_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_studio_long_horizon_completion_audit_check.py` | `STUDIO_LONG_HORIZON_COMPLETION_AUDIT_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_studio_long_horizon_next_jit_reconciliation_check.py` | `STUDIO_LONG_HORIZON_NEXT_JIT_RECONCILIATION_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_studio_numeric_curriculum_track_check.py` | `NUMERIC_SOLVER_LONG_HORIZON_ROADMAP_V1.md` (ROADMAP L14) | 없음 | - | 계획후미실행 | Numeric/Math 트랙 |
| `tests/run_studio_progress_claim_boundary_audit_check.py` | `STUDIO_PROGRESS_CLAIM_BOUNDARY_AUDIT_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_studio_public_release_execution_check.py` | `STUDIO_PUBLIC_RELEASE_EXECUTION_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_studio_public_release_external_publish_handoff_check.py` | `STUDIO_PUBLIC_RELEASE_EXTERNAL_PUBLISH_HANDOFF_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_studio_release_approval_gate_status_recheck.py` | `STUDIO_RELEASE_APPROVAL_GATE_STATUS_RECHECK_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Studio/Seamgrim 트랙 |
| `tests/run_studio_safe_continuation_queue_recheck.py` | `STUDIO_SAFE_CONTINUATION_QUEUE_RECHECK_V1.md` (DOC L10) | 없음 | - | 계획후미실행 | Queue/Root cleanup 트랙 |

## 자기 검증

- Q3 표에서 파싱한 체크 파일 수 = 173개 (요구 173개)
- Q3 표에서 파싱한 고유 `.md` 문서 수 = 176개 (Q3 요약 176개)
- Q3 표에서 파싱한 체크-문서 참조 행 수 = 225개
- 사용 명령: `git log --all --diff-filter=A --name-only --format=COMMIT\t%H\t%s`, `git log --all --diff-filter=D --name-only --format=COMMIT\t%H\t%s`, `git log --all --oneline -- <파일명>`
- 삭제·수리·체크 스크립트 수정 없음
