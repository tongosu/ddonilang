# BRIEF: 죽은 체커 173건 삭제 실행 (Q3/Q15 근거)

> 작성: Claude (2026-07-05) / 실행: Codex / 리뷰: Claude
> 근거: `docs/context/reports/BROKEN_CHECKS_AUDIT_V1.md`(Q3), `docs/context/reports/BROKEN_CHECKS_ROOT_CAUSE_V1.md`(Q15), `docs/context/reports/CONSOLIDATION_CANDIDATE_DRAFT_V1.md`(Q19)
> 성격: 이미 완료된 감사 결과를 실행에 옮기는 정리(cleanup) 작업. 새 분석/설계 아님 — 이미 확정된 후보를 삭제만 한다.

## 배경

Q3/Q15가 `tests/run_*.py` 1080개를 전수 분석해 173개 체커가 존재하지도 않는 문서(`.md`)를 필수 참조로 요구하고 있음을 확인했다. 176개 고유 참조 문서 전부 `git log --all --oneline -- <파일>` 결과 0행 — 즉 "삭제된 문서"가 아니라 **애초에 한 번도 작성된 적 없는 문서**다. Q15는 이 173개 전부를 "계획후미실행"(생성후삭제 0건)으로 분류했다. 이 체커들은 실제 기능을 검증하지 않고 문서 존재 여부만 검사하며, 가나다 로드맵 재검증(`GANADA_MATRIX_CORRECTED_20260706.md`)에서 FAIL로 잡힌 74칸 중 다수가 바로 이 부류다.

이 173개는 삭제해도 실제 기능 검증 커버리지 손실이 없다(애초에 기능을 검증한 적이 없으므로). 남겨두는 것은 노이즈만 늘릴 뿐이다.

## 작업

1. `docs/context/reports/BROKEN_CHECKS_AUDIT_V1.md`(Q3)의 표에 있는 173개 체크 파일 목록을 그대로 삭제 대상으로 삼는다.
2. **삭제 전 상호참조 확인**: 173개 목록 중 서로를 subprocess/import로 호출하는 경우가 있다(예: `tests/run_lang_flow_type_collision_rename_check.py`가 `tests/run_lang_prime_derivative_notation_decision_check.py`를 subprocess로 호출). 이런 경우 양쪽 다 173개 목록 안에 있으면 함께 삭제하면 된다. **173개 목록 밖의 다른 파일(체커든 코드든)이 이 173개 중 하나를 참조하면 그 파일은 삭제 대상에서 제외하고 별도로 보고하라** — 임의로 참조를 끊거나 수정하지 마라.
3. 각 파일 삭제 전에 `git log --all --oneline -- <체크파일 경로>`로 이 체크 파일 자체의 이력도 확인해 두어라(체크 파일 자체가 아니라 그것이 요구하는 `.md` 문서의 부재가 근거임을 재확인하는 차원).
4. 173개(상호참조로 인해 제외된 것 제외) `.py` 파일을 삭제한다.
5. 삭제 후 이 체커들이 `tests/run_ci_sanity_gate.py` 등 어떤 프로파일/스위트에도 필수 스텝으로 등록되어 있지 않은지 확인한다(등록되어 있다면 삭제하지 말고 보고만 하라 — 범위 밖 판단은 Claude가 함).
6. `docs/context/all/DEV_SUMMARY.md`에 정리 항목 추가(날짜/삭제 건수/근거 문서).

## 검증

- 삭제 전/후 `python tests/run_ci_sanity_gate.py --profile core_lang` PASS (동일)
- 삭제한 173개 파일 각각의 basename이 남은 코드베이스(`tests/`, `tools/`, `solutions/`, `docs/`) 어디에서도 더 이상 참조되지 않음을 `rg` 등으로 확인
- 상호참조로 제외한 파일이 있다면 그 목록과 사유를 실행 보고에 명시
- `git status --short`로 의도한 파일(삭제 173개 - 제외분, DEV_SUMMARY.md)만 변경되었는지 확인

## 수용 기준

- [ ] 173개 중 상호참조 예외를 뺀 나머지 전부 삭제
- [ ] 상호참조로 제외한 파일 있으면 목록+사유 보고
- [ ] core_lang sanity gate 삭제 전/후 동일하게 PASS
- [ ] 삭제 파일의 잔여 참조 0건 확인
- [ ] DEV_SUMMARY.md 갱신
- [ ] 176행 중 173행(Q3 표) 외 나머지(Q19 표의 176 삭제후보 중 Q3 밖 항목)는 **이번 범위 아님** — 손대지 않는다

## 금지 사항

- Q19의 "통합"(199건), "보류"(99건) 항목 손대지 않음 — 별도 브리프 대기
- 173개 목록 밖 파일 수정/삭제 없음(상호참조로 예외 처리한 파일 제외하고는 목록을 벗어나지 않는다)
- `docs/ssot/` 손대지 않음
- main 직접 커밋 금지. `codex/queue-20260706` 브랜치(또는 신규 큐 브랜치)에 커밋 1개.

## 보고 형식

이 파일 하단 `## 실행 보고`: 실제 삭제 건수, 상호참조로 제외한 파일(있다면), 검증 결과, DEV_SUMMARY 갱신 요약.

## 실행 보고

- 실행일: 2026-07-06
- Q3 원천 목록: `docs/context/reports/BROKEN_CHECKS_AUDIT_V1.md` 표에서 `tests/run_*.py` 173개 추출.
- 삭제 전 확인:
  - 대상 173개 모두 현재 파일 존재.
  - 대상 173개 모두 git 추적 중.
  - 체크 파일 자체 git 이력 확인: 173개 모두 `git log --all --name-only --format= -- <대상들>` 결과에 포함.
  - 삭제 전 `python tests/run_ci_sanity_gate.py --profile core_lang` PASS.
- 상호참조/외부 실행 참조 처리:
  - 173개 밖 실행 체크 스크립트가 후보를 호출하는 참조 39건 확인.
  - 외부 호출 대상에서 시작해 후보 간 subprocess/path 의존을 재귀적으로 닫은 결과, 62개는 삭제에서 제외.
  - 제외 사유: 삭제하면 목록 밖 기존 체크 또는 제외되어 남는 체크가 삭제 파일을 subprocess/path로 참조하게 되므로 브리프 예외 규칙 적용.
- 실제 삭제:
  - 삭제 실행: 111개.
  - 제외 보존: 62개.
- 제외 목록:
  - `tests/run_lang_connect_lowering_to_seum_check.py`
  - `tests/run_lang_connect_seum_lowering_parser_spike_check.py`
  - `tests/run_lang_core_2_representative_grammar_pack_check.py`
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
  - `tests/run_malblock_authoring_ui_check.py`
  - `tests/run_next_work_queue_after_connect_check.py`
  - `tests/run_numeric_root_finding_check.py`
  - `tests/run_nurigym_bandit_minimum_pack_check.py`
  - `tests/run_nurigym_cartpole_pendulum_expected_refresh_check.py`
  - `tests/run_nurigym_contract_gridworld_expected_refresh_check.py`
  - `tests/run_nurigym_dataset_hash_expected_refresh_check.py`
  - `tests/run_roadmap_v2_a1_final_closure_check.py`
  - `tests/run_roadmap_v2_a2_bandit_or_final_recheck.py`
  - `tests/run_roadmap_v2_a2_final_closure_check.py`
  - `tests/run_roadmap_v2_a2_nurigym_rebase_check.py`
  - `tests/run_roadmap_v2_ga2_final_closure_check.py`
  - `tests/run_roadmap_v2_ga2_representative_grammar_rebase_check.py`
  - `tests/run_roadmap_v2_ga5_release_gate_behavior_closure.py`
  - `tests/run_roadmap_v2_na2_event_minimum_closure_rebase_check.py`
  - `tests/run_roadmap_v2_na2_matrix_status_reconciliation_check.py`
  - `tests/run_roadmap_v2_na2_unit_random_event_rebase_check.py`
  - `tests/run_roadmap_v2_next_frontier_rebase_check.py`
  - `tests/run_roadmap_v2_sa1_rebase_check.py`
  - `tests/run_roadmap_v2_sa2_sprite_grid2d_final_closure_check.py`
  - `tests/run_roadmap_v2_sa2_sprite_grid2d_rebase_check.py`
  - `tests/run_roadmap_v2_studio_productization_rebase_check.py`
  - `tests/run_roadmap_v2_ta2_matrix_status_reconciliation_check.py`
  - `tests/run_roadmap_v2_ta3_diagnostic_ui_lsp_check.py`
  - `tests/run_roadmap_v2_ta4_registry_verification_check.py`
  - `tests/run_roadmap_v2_ta5_benchmark_lts_check.py`
  - `tests/run_root_low_risk_retire_delete_approval_gate_blocked_check.py`
  - `tests/run_root_low_risk_retire_delete_check.py`
  - `tests/run_root_low_risk_retire_delete_dry_run_plan_check.py`
  - `tests/run_root_low_risk_retire_delete_preflight_check.py`
  - `tests/run_sa2_sprite_skin_minimum_pack_check.py`
  - `tests/run_seamgrim_lesson_library_curation_check.py`
  - `tests/run_seamgrim_malblock_roundtrip_subset_check.py`
  - `tests/run_studio_baseline_rebase_check.py`
  - `tests/run_studio_numeric_curriculum_track_check.py`
- 삭제 후 검증:
  - 삭제 파일명 기준 실행 코드 잔여 참조: `rg -n --fixed-strings -f delete_basenames_closure.txt --glob '*.py' --glob '*.mjs' --glob '*.js' --glob '*.rs' tests tools solutions` 결과 0건.
  - 삭제 후 `python tests/run_ci_sanity_gate.py --profile core_lang` PASS.
  - `docs/context/all/DEV_SUMMARY.md`에 Q27 정리 항목 추가.
- 참고:
  - `docs/context/reports/BROKEN_CHECKS_AUDIT_V1.md`, `BROKEN_CHECKS_ROOT_CAUSE_V1.md`, `CONSOLIDATION_CANDIDATE_DRAFT_V1.md` 및 `DEV_SUMMARY` 과거 로그에는 과거 감사/검증 근거로서 삭제 파일명 참조가 남아 있다. Q27은 173개 밖 파일 수정 금지 조건을 우선해 역사적 보고서의 본문은 수정하지 않았다.
