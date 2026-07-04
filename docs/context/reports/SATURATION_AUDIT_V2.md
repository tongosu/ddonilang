# SATURATION_AUDIT_V2

## 범위

- Q7 산출물이다.
- Q6 보정 후 `DDN_SATURATION_AUDIT=1`이면 count가 0이어도 `saturation_audit count=0`을 출력하는 계측으로 826개 추적 pack golden을 재집계했다.
- `python tests/run_pack_golden.py --all`은 기존 Q5와 동일하게 스키마 오류에서 중단될 수 있으므로, runner를 팩별 독립 실행했다.
- 실행 상태 기준선은 Q5의 무계측 팩별 실행 결과를 사용했다. Q6에서 미설정 경로의 비트 동일성을 pack golden 2건과 `core_lang`으로 재확인했기 때문이다.
- count 수집은 runner를 수정하지 않고 build 디렉터리의 임시 monkeypatch harness로 내부 `teul-cli` subprocess stdout/stderr를 팩별 raw 로그에 복제해 파싱했다.
- runner 스키마 단계 또는 runner 내부 검증만으로 `teul-cli`가 미기동한 팩은 `count=0`과 함께 `teul-cli 미기동(...)` 근거를 별도 표기했다. 무근거 0 추정은 사용하지 않았다.
- FAIL/TIMEOUT 팩은 수리하지 않고 `스키마` / `골든 불일치` / `타임아웃` 1차 분류만 기록했다.
- golden 갱신은 하지 않았다.

## 실행 로그

- 실행 worktree: `I:/dev/ddonilang/codex_q7_worktree_20260704_154134`
- 집계 원본: `I:/home/urihanl/ddn/codex/build/q7_saturation_audit/per_pack_results_v2.detjson`
- 실행 로그: `I:/home/urihanl/ddn/codex/build/q7_saturation_audit/per_pack_results_v2.log`
- 팩별 raw 로그: `I:/home/urihanl/ddn/codex/build/q7_saturation_audit/raw_pack_logs/`

## 요약

- 실행 팩 수: 826개
- PASS: 727개
- FAIL: 98개
- TIMEOUT: 1개
- 포화 발생 팩: 0개
- 총 포화 발생 횟수: 0회
- 실패 단계 1차 분류: 스키마 27개 / 골든 불일치 71개 / 타임아웃 1개
- count audit line 확인 팩: 812개
- teul-cli 미기동 count=0 팩: 14개 (teul-cli 미기동(러너 내부 검증만) 4개, teul-cli 미기동(스키마 단계) 10개)

## 표

| 팩 | 실행 상태 | 실패 단계 | 포화 발생 횟수 | count 근거 | 발생 지점(연산 종류) | 추정 원인 | 오류 전환 시 영향(FAIL 전환 여부) |
|---|---|---|---:|---|---|---|---|
| `age1_charim_index` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `age1_immediate_proof_case_analysis_else_solver_open_search_smoke_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age1_immediate_proof_case_analysis_smoke_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age1_immediate_proof_case_analysis_solver_open_search_smoke_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age1_immediate_proof_case_analysis_solver_open_smoke_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age1_immediate_proof_case_analysis_solver_search_smoke_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age1_immediate_proof_case_analysis_solver_search_solve_smoke_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age1_immediate_proof_smoke_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age1_immediate_proof_solver_search_solve_smoke_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age2_dice_seed_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `age2_econ_market_shock_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `age2_game_miniloop_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `age2_kernel_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `age2_math_calculus_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `age2_open_input_record_replay_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age2_phys_pendulum_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `age3_beat_reserve_smoke_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_case_completeness_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=5 | 해당 없음 | count=0 audit line 확인 (audit log lines=5) | 영향 없음(PASS, count=0) |
| `age4_proof_artifact_cert_subject_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `age4_proof_case_analysis_else_solver_open_search_replay_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_case_analysis_exists_solver_open_search_replay_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_case_analysis_forall_solver_open_search_replay_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_case_analysis_solver_open_replay_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_case_analysis_solver_open_search_replay_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_case_analysis_solver_search_replay_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_clock_replay_missing_failure_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_clock_replay_parse_failure_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_clock_replay_tamper_failure_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_detjson_smoke_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_file_read_replay_missing_failure_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_file_read_replay_parse_failure_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_file_read_replay_tamper_failure_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_input_replay_missing_failure_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_input_replay_parse_failure_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_input_replay_tamper_failure_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_solver_deny_failure_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_solver_open_replay_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_solver_replay_missing_failure_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_solver_replay_parse_failure_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_solver_replay_tamper_failure_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_solver_search_replay_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_proof_solver_translation_smoke_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `age4_quantifier_surface_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 영향 없음(PASS, count=0) |
| `ai_det_tier_capability_matrix_v1` | 실행 실패 + count=0 확인 | 스키마 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `bdl2_subpixel_aa_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `benchmark_baseline_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `block_header_no_colon` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=5 | 해당 없음 | count=0 audit line 확인 (audit log lines=5) | 영향 없음(PASS, count=0) |
| `bogae_adapter_v1_smoke` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `bogae_api_catalog_v1_basic` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `bogae_api_catalog_v1_errors` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `bogae_api_catalog_v1_game_hud` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `bogae_api_catalog_v1_graph` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `bogae_asset_manifest_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `bogae_bg_key_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `bogae_bundle_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `bogae_cache_min` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `bogae_canvas_key_precedence_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `bogae_canvas_ssot_key_precedence_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `bogae_drawlist_listkey_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `bogae_drawlist_trait_alias_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `bogae_editor_v0` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `bogae_hash_determinism_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `bogae_mapping_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `bogae_observe_basics` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `bogae_overlay_full_runner_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `bogae_runner_bogae_hash` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `bogae_shape_trait_ssot_alias_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `bogae_web_viewer_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `bounded_equation_solver_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `compound_update_basics` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=6 | 해당 없음 | count=0 audit line 확인 (audit log lines=6) | 영향 없음(PASS, count=0) |
| `connect_endpoint_boundary_range_check_fail_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_boundary_range_check_pass_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_boundary_range_missing_value_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_boundary_range_unit_conflict_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_boundary_range_unit_pass_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_boundary_range_unsupported_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_boundary_value_injection_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_boundary_value_solve_remap_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_boundary_value_unsupported_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_carried_property_forward_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_carried_property_reverse_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_equality_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_explicit_solve_equal_value_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_explicit_solve_flat_set_value_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_explicit_solve_flow_value_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_explicit_solve_range_fail_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_explicit_solve_range_missing_value_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_explicit_solve_range_pass_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_explicit_solve_range_unit_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_explicit_solve_range_unsupported_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_explicit_solve_unsupported_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_flow_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_formula_relation_bridge_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_formula_relation_solve_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_formula_relation_unsupported_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_multi_inner_econ_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_multi_inner_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_relation_flatten_relation_set_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_relation_flatten_single_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_relation_flatten_statement_set_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_relation_normalize_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_reverse_flow_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_check_direct_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_check_fail_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_check_pass_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_check_runner_fail_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_check_runner_pass_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_check_runner_unsupported_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_check_unsupported_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_detail_mixed_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_detail_pass_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_detail_unit_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_detail_unsupported_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_mixed_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_pass_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_summary_direct_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_summary_mixed_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_summary_pass_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_summary_unsupported_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_text_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_unit_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_case_suite_unsupported_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_report_fail_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_report_missing_value_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_report_pass_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_report_unit_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_report_unsupported_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_text_report_fail_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_text_report_missing_value_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_text_report_pass_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_text_report_unit_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_range_text_report_unsupported_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_result_remap_failure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_result_remap_partial_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_result_remap_success_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_solve_result_remap_unsupported_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_statement_append_boundary_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_statement_append_mixed_pair_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_statement_append_same_pair_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_unit_boundary_dim_conflict_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_unit_boundary_explicit_solve_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_unit_boundary_incompatible_unit_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_unit_boundary_injection_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_endpoint_unit_boundary_solve_remap_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_sign_convention_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1c_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1d_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1e_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1f_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1g_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1h_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1i_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1j_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1k_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1l_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1m_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1n_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1o_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1p_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1q_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1r_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1s_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1t_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1u_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_flow_v1v_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_relation_bridge_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_subset_lowering_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `connect_wasm_cli_parity_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `constraint_solve_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `diag_contract_mode_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `diag_fixit_coverage_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=6 | 해당 없음 | count=0 audit line 확인 (audit log lines=6) | 영향 없음(PASS, count=0) |
| `diag_fixit_json_schema_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 영향 없음(PASS, count=0) |
| `dialect_builtin_equiv_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `dotbogi_ddn_interface_v1_event_roundtrip` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `dotbogi_ddn_interface_v1_smoke` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `dotbogi_ddn_interface_v1_write_forbidden` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `dstrict_velocity_verlet_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `eco_abm_spatial_smoke` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=7 | 해당 없음 | count=0 audit line 확인 (audit log lines=7) | 영향 없음(PASS, count=0) |
| `eco_diag_convergence_smoke` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 영향 없음(PASS, count=0) |
| `eco_macro_micro_runner_smoke` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=8 | 해당 없음 | count=0 audit line 확인 (audit log lines=8) | 영향 없음(PASS, count=0) |
| `eco_network_flow_smoke` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=7 | 해당 없음 | count=0 audit line 확인 (audit log lines=7) | 영향 없음(PASS, count=0) |
| `eco_stats_stdlib_smoke` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=13 | 해당 없음 | count=0 audit line 확인 (audit log lines=13) | 영향 없음(PASS, count=0) |
| `econ_age4_ready_baseline_tax_ceiling_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `edu_algo_loop` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `edu_algo_search` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `edu_chem_reaction` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `edu_data_struct` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `edu_ddn_mastercourse/L01` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `edu_ddn_mastercourse/L02` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `edu_ddn_mastercourse/L03` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `edu_math_function_line` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `edu_math_parabola` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `edu_math_stats` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `edu_physics_energy` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `edu_physics_motion` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `education_curriculum_1_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `education_curriculum_2_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `education_curriculum_3_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `education_curriculum_4_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `education_curriculum_5_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `external_intent_boundary_v1` | 실행 실패 + count=0 확인 | 스키마 | 0 | teul-cli 미기동(스키마 단계) | 해당 없음 | count=0 확인 (teul-cli 미기동(스키마 단계)) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `formula_relation_solve_quadratic_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `formula_relation_solve_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 영향 없음(PASS, count=0) |
| `free_lab_1_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `free_lab_2_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `free_lab_3_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `free_lab_4_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `free_lab_5_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gate0_contract_abort_statehash_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 영향 없음(PASS, count=0) |
| `geoul_min_schema_v0` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `geoul_record_query_timeline_minimum_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=7 | 해당 없음 | count=0 audit line 확인 (audit log lines=7) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `geoul_replay_playback_closure_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=6 | 해당 없음 | count=0 audit line 확인 (audit log lines=6) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `gogae5_w45_intent_bundle` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae5_w46_goal_grammar` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae5_w46_goal_parser_dorok` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae5_w47_nurigym_observation_spec` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae5_w48_goap_planner` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae5_w49_latency_L_madi` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae5_w49_latency_madi` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 영향 없음(PASS, count=0) |
| `gogae5_w50_safety_gatekeeper` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae5_w51_llm_bridge_mock` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae5_w52_multi_agent_bus` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae5_w53_dataset_export` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `gogae5_w54_workshop_gui_v0` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `gogae5_w55_smart_errand_integration` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `gogae7_demo_one` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae8_w78_parallel_realms` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=6 | 해당 없음 | count=0 audit line 확인 (audit log lines=6) | 영향 없음(PASS, count=0) |
| `gogae8_w79_gpu_warp` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `gogae8_w81_reward_v1` | 실행 실패 + count=0 확인 | 스키마 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `gogae8_w82_seulgi_train_v2` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae8_w83_edu_accuracy_v1` | 실행 실패 + count=0 확인 | 스키마 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `gogae8_w84_swarm_deterministic_collision` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae8_w85_ondevice_infer_v1` | 실행 실패 + count=0 확인 | 스키마 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `gogae8_w86_imitation_learning_v1` | 실행 실패 + count=0 확인 | 스키마 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `gogae8_w87_eval_suite_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `gogae8_w88_bundle_hash_parity` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae9_w89_self_evolving_code` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae9_w90_meta_universe` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=6 | 해당 없음 | count=0 audit line 확인 (audit log lines=6) | 영향 없음(PASS, count=0) |
| `gogae9_w91_malmoi_docset` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae9_w92_aot_compiler_v2` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae9_w93_universe_gui` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae9_w94_social_sim` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae9_w95_cert` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae9_w96_somssi_hub` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae9_w97_self_heal` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae9_w98_release_gate` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `gogae9_w99_evolving_universe` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `guideblock_keys_basics` | PASS + count=0 확인 | 해당 없음 | 0 | teul-cli 미기동(러너 내부 검증만) | 해당 없음 | count=0 확인 (teul-cli 미기동(러너 내부 검증만)) | 영향 없음(PASS, count=0) |
| `input_key_alias_ko_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `input_key_alias_ko_v2` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `inputkey_missing_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `lang_blocked_ssot_track_parking_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_chaebi_scope_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=5 | 해당 없음 | count=0 audit line 확인 (audit log lines=5) | 영향 없음(PASS, count=0) |
| `lang_connect_lowering_to_seum_check_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_connect_seum_lowering_parser_spike_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_consistency_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=25 | 해당 없음 | count=0 audit line 확인 (audit log lines=25) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `lang_continue_skip_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 영향 없음(PASS, count=0) |
| `lang_ddn_meta_header_basics` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `lang_dialect_smoke_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `lang_dstrict_dultra_solver_strategy_proposal_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_dultra_recorded_replay_contract_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_dultra_replay_artifact_implementation_gate_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_dultra_replay_artifact_writer_seed_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_flow_history_alias_migration_plan_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_flow_history_ssot_acceptance_request_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_flow_type_collision_rename_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_generic_iterable_order_contract_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `lang_generic_iterable_sealed_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `lang_generic_iterable_strict_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `lang_history_alias_stdlib_bridge_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_hook_when_edge_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `lang_hook_while_condition_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `lang_implementation_followup_closure_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_implementation_readiness_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_inline_lambda_function_pin_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `lang_lambda_capture_runtime_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_lambda_return_runtime_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_lambda_store_runtime_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_language_risk_removal_closure_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_maegim_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=5 | 해당 없음 | count=0 audit line 확인 (audit log lines=5) | 영향 없음(PASS, count=0) |
| `lang_non_ssot_track_resume_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_owner_inner_seum_parser_boundary_spike_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_owner_inner_seum_runtime_scope_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_owner_inner_seum_structure_check_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_owner_state_symbol_table_product_path_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_pack_checker_stability_sweep_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_post_ssot_landing_product_gate_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_pragmatism_pack_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `lang_prime_derivative_notation_decision_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_prime_derivative_runtime_semantics_gate_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_prime_parser_frontdoor_spike_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_prime_ssot_acceptance_request_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_product_path_transition_closure_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_range_literal_v0` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_regex_literal_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=9 | 해당 없음 | count=0 audit line 확인 (audit log lines=9) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `lang_settings_header_closure_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `lang_seum_vol3_prime_example_pack_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_sim_constraint_third_layer_name_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_ssot_acceptance_request_closure_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_ssot_landing_coordination_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_ssot_landing_wait_state_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_ssot_owner_landing_audit_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_ssot_owner_landing_handoff_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_surface_family_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `lang_tag_context_uniqueness_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `lang_teulcli_parser_parity_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 영향 없음(PASS, count=0) |
| `lang_tuck_constraint_surface_shape_proposal_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_tuck_ssot_acceptance_handoff_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_tuck_ssot_acceptance_request_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_unit_temp_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=7 | 해당 없음 | count=0 audit line 확인 (audit log lines=7) | 영향 없음(PASS, count=0) |
| `lang_velocity_verlet_fixed64_order_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_velocity_verlet_runtime_gate_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lang_velocity_verlet_stdlib_surface_acceptance_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `language_design_priority_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `lifecycle_core_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=6 | 해당 없음 | count=0 audit line 확인 (audit log lines=6) | 영향 없음(PASS, count=0) |
| `lifecycle_pan_replay_minimum_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `lifecycle_reset_grid_input_minimum_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=5 | 해당 없음 | count=0 audit line 확인 (audit log lines=5) | 영향 없음(PASS, count=0) |
| `linear_inequality_solve_minimum_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `lsp_followon_minimum_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `malblock_authoring_ui_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `malhim_rpg_1_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `malhim_rpg_2_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `malhim_rpg_3_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `malhim_rpg_4_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `malhim_rpg_5_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `math_calculus_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `math_numeric_diff_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `math_numeric_int_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `math_vector_minimum_first_run_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `model_artifact_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `model_artifact_eval_minimum_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `model_artifact_inference_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `module_system_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=8 | 해당 없음 | count=0 audit line 확인 (audit log lines=8) | 영향 없음(PASS, count=0) |
| `numeric_display_policy_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `numeric_exact_numbers_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `numeric_exact_universe_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `numeric_factor_certificate_route_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `numeric_factor_form_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `numeric_factor_job_resume_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `numeric_factor_kernel_unbounded_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `numeric_fraction_normalize_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `numeric_maegim_binding_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 영향 없음(PASS, count=0) |
| `numeric_promotion_rules_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `numeric_root_finding_bisection_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `numeric_sized_variants_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `numeric_solver_capability_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `numeric_type_alias_korean_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=7 | 해당 없음 | count=0 audit line 확인 (audit log lines=7) | 영향 없음(PASS, count=0) |
| `numeric_type_pin_vs_constructor_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 영향 없음(PASS, count=0) |
| `nuri_gym_bandit_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `nuri_gym_canon_contract_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `nuri_gym_cartpole_shared_sync_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `nuri_gym_cartpole_shared_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `nuri_gym_cartpole_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `nuri_gym_dataset_registry_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `nuri_gym_gridmaze_easy_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `nuri_gym_gridmaze_hard_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `nuri_gym_gridmaze_layouts_injected_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `nuri_gym_gridmaze_medium_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `nuri_gym_gridmaze_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `nuri_gym_gridworld_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `nuri_gym_pendulum_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 영향 없음(PASS, count=0) |
| `nurigym_python_web_parity_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `nurigym_shared_sync_action_pipeline_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `nurigym_shared_sync_priority_tiebreak_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `nurigym_training_workflow_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `ode_method_comparison_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `ode_tick_loop_lesson_baseline_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_bundle_artifact` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_clock_record_replay` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_decl_policy` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_decl_policy_warn` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `open_deny_policy` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `open_diag_minimal_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_end_to_end` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `open_ffi_record_replay` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_ffi_schema_invalid` | 실행 실패 + count=0 확인 | 스키마 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `open_file_read_abs_case` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `open_file_read_abs_path` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `open_file_read_key_mismatch` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_file_read_path_normalize` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `open_file_read_record_replay` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_file_read_unc_case` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `open_file_read_unc_dot` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `open_file_read_unc_dot_mix` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `open_file_read_unc_mix` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `open_file_read_unc_space_special` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `open_gpu_record_replay` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_gpu_schema_invalid` | 실행 실패 + count=0 확인 | 스키마 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `open_net_record_replay` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_net_schema_invalid` | 실행 실패 + count=0 확인 | 스키마 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `open_policy_allowlist` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_policy_conflict` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_rand_record_replay` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_replay_hash_mismatch` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_replay_invalid` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_replay_mismatch_diag` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 영향 없음(PASS, count=0) |
| `open_replay_missing` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_replay_schema_mismatch` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_replay_schema_v2_accept` | 실행 실패 + count=0 확인 | 스키마 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `open_replay_schema_v2_extra` | 실행 실패 + count=0 확인 | 스키마 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `open_replay_schema_v2_meta_bool_null` | 실행 실패 + count=0 확인 | 스키마 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `open_replay_schema_v2_meta_list` | 실행 실패 + count=0 확인 | 스키마 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `open_replay_schema_v2_meta_scalar` | 실행 실패 + count=0 확인 | 스키마 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `open_replay_schema_v2_missing` | 실행 실패 + count=0 확인 | 스키마 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `open_replay_schema_v2_nested_meta` | 실행 실패 + count=0 확인 | 스키마 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `open_replay_site_mismatch` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `open_site_id_canon` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `patent_c_single_funnel_nonreentry_queue_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `polynomial_solve_minimum_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `project_age_target_error_code_example` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `project_age_target_precedence_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `project_age_target_precedence_v1_negative` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `proof_alert_continue_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `proof_ddn_jeunggeo_bridge_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `proof_ddn_relation_bridge_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `proof_guard_rollback_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `proof_guard_tick_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `proof_kernel_term_replay_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `proof_numeric_certificate_verify_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `proof_numeric_factor_certificate_strength_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `proof_relation_equivalence_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `proof_relation_solve_consistency_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `proof_runtime_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `proof_seum_bridge_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `proof_seum_runtime_bridge_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `proof_symbolic_rewrite_verify_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `proof_tactic_rewrite_chain_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `proof_tactic_symbolic_eq_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `question_card_author_tool_share_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `question_card_dev_assist_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `question_card_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `question_card_validation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `question_card_workflow_hardening_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `relation_solve_ddn_bridge_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `relation_solve_ddn_bridge_v2` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 영향 없음(PASS, count=0) |
| `relation_solve_system_2x2_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 영향 없음(PASS, count=0) |
| `relation_solve_wasm_cli_parity_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `relation_solve_wasm_cli_parity_v2` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 영향 없음(PASS, count=0) |
| `roadmap_v2_a0_nurigym_schema_skeleton_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_a1_nurigym_reset_step_matrix_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_a2_nurigym_representative_environment_matrix_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ba0_free_lab_seed_reassessment_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ba0_free_lab_seed_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_cha0_rpg_seed_reassessment_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_cha0_rpg_seed_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_da0_math_proof_scope_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_da1_math_first_run_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_da2_symbolic_solve_proof_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_da3_seamgrim_math_view_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_da4_math_package_share_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_da5_math_lts_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ga0_current_line_ledger_matrix_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ga1_core_smoke_matrix_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ga2_matrix_status_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ga3_editor_diagnostic_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ga4_package_gaji_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ga5_grammar_lts_behavior_recheck_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ga5_grammar_lts_docs_closed_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ga5_release_gate_behavior_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ga5_release_gate_blocker_audit_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_geo0_ai_question_card_seed_behavior_reassessment_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_global_4era_plan_v5` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ha0_education_curriculum_template_matrix_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ja0_ai_boundary_behavior_reassessment_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ja4_model_artifact_share_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ja_seulgi_boundary_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ka0_platform_charter_matrix_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ka1_server_mvp_matrix_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_la0_malblock_design_behavior_reassessment_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_la0_pa0_docs_closed_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_la1_malblock_intro_rail_matrix_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_la2_matrix_status_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_la3_workbench_integration_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_la4_lesson_package_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_la5_editor_lts_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ma0_curriculum_catalog_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ma1_lesson_first_run_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ma2_studio_prereq_unlock_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ma3_studio_classroom_workbench_prereq_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ma4_public_lesson_publication_prereq_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ma5_curriculum_lts_prereq_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_na0_stdlib_candidate_list_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_na1_post_matrix_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_na1_std_core_grid_input_matrix_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_na2_matrix_status_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_na3_matrix_status_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_na3_resource_network_policy_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_na4_stdlib_registry_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_na5_stdlib_lts_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_next_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_pa0_social_case_card_behavior_reassessment_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_pa1_baseline_market_first_run_matrix_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_post_a1_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_post_da5_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_post_ga0_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_post_ga1_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_post_ha0_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_post_ha1_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_post_ka0_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_post_ka1_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_post_la1_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_post_la2_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_post_na3_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_post_pa1_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_post_sa1_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_post_ta0_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_post_ta1_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_post_ta2_frontier_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_sa0_bogae_schema_boundary_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_sa1_bogae_graph_space2d_matrix_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_sa2_sprite_grid2d_final_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_sa2_sprite_grid2d_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_sa3_game_preview_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_sa4_asset_view_share_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_sa5_renderer_hardening_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_studio_productization_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ta0_pack_checker_skeleton_matrix_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ta1_pack_runner_basis_matrix_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ta2_guide_status_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `roadmap_v2_ta2_matrix_status_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `rpg_combat_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `rpg_dialogue_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `rpg_dialogue_template_parse_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `rpg_enemy_spawn_signal_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `rpg_invariants_smoke_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `rpg_inventory_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `rpg_item_pickup_signal_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `rpg_level_up_cascade_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `rpg_npc_dialogue_signal_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `rpg_npc_statemachine_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `rpg_nurigym_env_smoke_v1` | 실행 실패 + count=0 확인 | 스키마 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `rpg_player_actor_parse_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `rpg_player_actor_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `rpg_player_damage_signal_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `rpg_player_death_transition_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `rpg_quest_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `rpg_seumvalue_parse_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `rpg_statemachine_parse_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `rpg_world_define_canon_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `sa2_sprite_skin_minimum_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `sam_ai_ordering_v1` | 실행 실패 + count=0 확인 | 스키마 | 0 | teul-cli 미기동(스키마 단계) | 해당 없음 | count=0 확인 (teul-cli 미기동(스키마 단계)) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `sam_inputsnapshot_contract_v1` | 실행 실패 + count=0 확인 | 스키마 | 0 | teul-cli 미기동(스키마 단계) | 해당 없음 | count=0 확인 (teul-cli 미기동(스키마 단계)) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `seamgrim_baseline_stabilization_closure_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_bogae_madang_alias_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `seamgrim_bridge_surface_v0_basics` | 실행 실패 + count=0 확인 | 스키마 | 0 | teul-cli 미기동(스키마 단계) | 해당 없음 | count=0 확인 (teul-cli 미기동(스키마 단계)) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `seamgrim_cli_warning_parity_repair_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_curriculum_2_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_curriculum_3_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_curriculum_4_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_curriculum_5_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_curriculum_batch_smoke_v1` | TIMEOUT + count=0 확인 | 타임아웃 | 0 | audit log lines=402 | 해당 없음 | count=0 audit line 확인 (audit log lines=402) | TIMEOUT + count=0. 실패 단계=타임아웃; 수리 금지, 1차 분류만. |
| `seamgrim_curriculum_rewrite_core_smoke_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=12 | 해당 없음 | count=0 audit line 확인 (audit log lines=12) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `seamgrim_curriculum_rewrite_sample_smoke_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=24 | 해당 없음 | count=0 audit line 확인 (audit log lines=24) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `seamgrim_curriculum_seed_smoke_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `seamgrim_event_model_ir_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `seamgrim_event_surface_canon_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=7 | 해당 없음 | count=0 audit line 확인 (audit log lines=7) | 영향 없음(PASS, count=0) |
| `seamgrim_exec_policy_effect_diag_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=26 | 해당 없음 | count=0 audit line 확인 (audit log lines=26) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `seamgrim_exec_policy_effect_map_v1` | 실행 실패 + count=0 확인 | 스키마 | 0 | audit log lines=6 | 해당 없음 | count=0 audit line 확인 (audit log lines=6) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `seamgrim_first_run_onboarding_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_graph_autorender_v1` | PASS + count=0 확인 | 해당 없음 | 0 | teul-cli 미기동(러너 내부 검증만) | 해당 없음 | count=0 확인 (teul-cli 미기동(러너 내부 검증만)) | 영향 없음(PASS, count=0) |
| `seamgrim_graph_v0_basics` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `seamgrim_guseong_flatten_diag_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=21 | 해당 없음 | count=0 audit line 확인 (audit log lines=21) | 영향 없음(PASS, count=0) |
| `seamgrim_guseong_flatten_ir_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `seamgrim_jjaim_block_stub_canon_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=6 | 해당 없음 | count=0 audit line 확인 (audit log lines=6) | 영향 없음(PASS, count=0) |
| `seamgrim_josa_binding_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `seamgrim_lesson_authoring_flow_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_lesson_library_curation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_lesson_run_preset_rail_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_line_graph` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `seamgrim_malblock_roundtrip_subset_queue_guard_repair_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_moyang_template_instance_view_boundary_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=5 | 해당 없음 | count=0 audit line 확인 (audit log lines=5) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_browser_index_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_consolidation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_lesson_preview_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_report_export_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_export_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_history_export_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_history_report_export_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_history_report_table_export_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_export_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_summary_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_status_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_history_report_table_status_badge_a11y_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_history_report_table_status_badge_export_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_history_report_table_status_badge_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_history_report_table_status_export_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_history_report_table_status_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_history_report_table_summary_export_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_history_report_table_summary_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_history_report_table_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_history_report_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_history_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_compare_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_history_filter_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_reopen_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_summary_export_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_result_timeline_view_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_run_preset_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_numeric_track_run_result_link_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_overlay_param_compare_v0` | PASS + count=0 확인 | 해당 없음 | 0 | teul-cli 미기동(러너 내부 검증만) | 해당 없음 | count=0 확인 (teul-cli 미기동(러너 내부 검증만)) | 영향 없음(PASS, count=0) |
| `seamgrim_overlay_session_roundtrip_v0` | PASS + count=0 확인 | 해당 없음 | 0 | teul-cli 미기동(러너 내부 검증만) | 해당 없음 | count=0 확인 (teul-cli 미기동(러너 내부 검증만)) | 영향 없음(PASS, count=0) |
| `seamgrim_playground_smoke_v1` | 실행 실패 + count=0 확인 | 스키마 | 0 | teul-cli 미기동(스키마 단계) | 해당 없음 | count=0 확인 (teul-cli 미기동(스키마 단계)) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `seamgrim_private_productization_consolidation_audit_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_private_productization_next_queue_recheck_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_registry_publish_install_shell_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_run_history_comparison_rail_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_run_history_export_summary_followup_recheck_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_run_history_export_summary_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_runtime_5min_baseline_repair_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_smoke_golden_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `seamgrim_space2d_v0_basics` | 실행 실패 + count=0 확인 | 스키마 | 0 | teul-cli 미기동(스키마 단계) | 해당 없음 | count=0 확인 (teul-cli 미기동(스키마 단계)) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `seamgrim_state_hash_view_boundary_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `seamgrim_static_run_completion_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_wasm_cli_runtime_parity_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=5 | 해당 없음 | count=0 audit line 확인 (audit log lines=5) | 영향 없음(PASS, count=0) |
| `seamgrim_wasm_tool_golden_baseline_repair_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_workbench_polish_v2` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seamgrim_workbench_shell_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seulgi_gatekeeper_v1` | 실행 실패 + count=0 확인 | 스키마 | 0 | teul-cli 미기동(스키마 단계) | 해당 없음 | count=0 확인 (teul-cli 미기동(스키마 단계)) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `seulgi_proposal_ui_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seulgi_replay_safe_workflow_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seulgi_v1` | 실행 실패 + count=0 확인 | 스키마 | 0 | teul-cli 미기동(스키마 단계) | 해당 없음 | count=0 확인 (teul-cli 미기동(스키마 단계)) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `seum_assertion_block_v1b` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `seumssi_relation_bridge_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `social_world_econ_2_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `social_world_econ_3_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `social_world_econ_4_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `social_world_econ_5_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `state_machine_transition_action_aborted_report_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `state_machine_transition_action_arg_unresolved_report_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `state_machine_transition_action_failure_report_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `state_machine_transition_check_unresolved_report_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `state_machine_transition_failure_report_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `state_machine_transition_guard_rejected_report_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `state_machine_transition_guard_unresolved_report_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `state_machine_transition_report_v1` | 실행 실패 + count=0 확인 | 골든 불일치 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 실행 실패 + count=0. 실패 단계=골든 불일치; 포화 수리 금지, 1차 분류만. |
| `std_block_piece_geometry_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_block_piece_grid_bridge_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_block_piece_minimum_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_core_grid_unit_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_event_minimum_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_falling_piece_state_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_bounds_collision_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_cell_read_write_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_backend_parity_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_bridge_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_browser_dom_smoke_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_browser_input_delivery_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_drawlist_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_finite_live_loop_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_finite_live_loop_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_live_bridge_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_live_input_contract_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_live_input_frame_effect_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_live_web_assets_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_size_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_viewer_js_dom_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_viewer_js_live_input_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_viewer_js_playback_controls_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_web_out_determinism_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_web_playback_assets_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_web_playback_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_web_playback_controls_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_web_showcase_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_bogae_web_viewer_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_ghost_piece_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_hold_queue_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_lock_spawn_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_one_tick_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_playable_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_playable_view_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_rules_minimum_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_score_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_session_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_set_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_simple_wall_kick_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_state_lifecycle_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_state_minimum_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_state_pause_resume_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_step_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_view_overlay_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_view_summary_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_game_view_text_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_line_clear_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_pathfind_blocked_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_pathfind_bounds_diag_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_grid_pathfind_reachable_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_input_map_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_input_map_keyboard_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_input_map_web_snapshot_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_physics_1d_basics_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_random_bag_minimum_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_random_bag_order_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_random_bag_refill_preview_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_resource_network_policy_minimum_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `std_tetromino_catalog_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `stdlib_1_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=5 | 해당 없음 | count=0 audit line 확인 (audit log lines=5) | 영향 없음(PASS, count=0) |
| `stdlib_charim_basics` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `stdlib_examples/01_charim_filter` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `stdlib_examples/02_charim_map` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `stdlib_examples/03_charim_length` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `stdlib_examples/04_string_split_join` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `stdlib_examples/05_string_contains` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `stdlib_examples/06_math_abs` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `stdlib_examples/07_math_minmax_clamp` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `stdlib_examples/08_template_fill` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `stdlib_examples/09_template_match` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `stdlib_examples/10_list_reduce` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `stdlib_l1_filters_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `stdlib_l1_integrators_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `stdlib_l1_interpolations_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `stdlib_map_basics` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `stdlib_math_basics` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `stdlib_missing_coverage_io_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `stdlib_missing_coverage_pure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `stdlib_range_basics` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `stdlib_text_basics` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `studio_baseline_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_benchmark_baseline_local_snapshot_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_benchmark_baseline_prep_dry_run_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_benchmark_lts_matrix_export_action_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_benchmark_lts_matrix_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_browser_smoke_flake_audit_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_browser_smoke_matrix_hardening_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_classroom_mode_switch_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_classroom_mode_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_classroom_operations_panel_preview_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_classroom_operations_triage_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_classroom_report_export_action_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_classroom_report_workflow_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_diagnostic_fixit_editor_apply_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_diagnostic_fixit_integration_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_diagnostic_fixit_preview_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_dirty_baseline_recheck_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_doc_index_refresh_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_education_operations_lts_export_action_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_education_operations_lts_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_historical_progress_ledger_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_lesson_authoring_run_integration_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_lesson_publication_review_surface_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_local_package_export_action_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_local_packaging_consolidation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_local_release_rehearsal_check_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_local_share_and_packaging_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_long_horizon_completion_audit_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_long_horizon_next_jit_reconciliation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_ma3_next_development_queue_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_ma3_next_queue_coordinate_lock_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_ma3_regression_gate_matrix_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_malblock_workbench_integration_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_next_roadmap_v2_coordinate_lock_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_numeric_curriculum_track_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_numeric_report_workflow_consolidation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_numeric_result_report_consolidation_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_operations_preview_stage_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_post_approval_chain_maintenance_queue_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_post_release_gate_maintenance_queue_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_post_super_long_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_private_productization_queue_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_private_productization_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_productization_stage_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_productization_stage_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_progress_claim_boundary_audit_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_public_lesson_publication_prep_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_public_release_approval_recheck_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_public_release_asset_plan_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_public_release_execution_gate_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_public_release_execution_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_public_release_external_publish_handoff_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_public_release_prep_rebase_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_public_release_smoke_matrix_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_publication_artifact_dry_run_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_publication_prep_export_action_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_rc_checker_cost_trim_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_registry_share_seed_export_action_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_registry_share_seed_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_approval_chain_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_approval_continuity_export_action_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_approval_fast_check_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_approval_gate_status_recheck_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_approval_handoff_text_export_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_approval_handoff_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_approval_packet_continuity_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_approval_packet_text_export_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_approval_packet_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_approval_readiness_recheck_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_approval_status_snapshot_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_approval_wait_state_closure_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_candidate_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_dry_run_text_summary_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_notes_draft_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_notes_text_export_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_pre_execution_dry_run_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_release_review_packet_dashboard_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_safe_continuation_queue_recheck_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_stale_release_doc_audit_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_teacher_feedback_loop_seed_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `studio_teacher_feedback_surface_preview_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `symbolic_ddn_cli_parity_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=4 | 해당 없음 | count=0 audit line 확인 (audit log lines=4) | 영향 없음(PASS, count=0) |
| `symbolic_ddn_formula_bridge_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `symbolic_diff_integral_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `symbolic_equivalence_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `symbolic_expand_factor_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `symbolic_mathir_canon_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `symbolic_minimum_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `symbolic_multivar_polynomial_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `symbolic_polynomial_simplify_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `symbolic_rational_expr_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=2 | 해당 없음 | count=0 audit line 확인 (audit log lines=2) | 영향 없음(PASS, count=0) |
| `symbolic_relation_canon_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `tensor_stdlib_phase0` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `tensor_v0_dense` | 실행 실패 + count=0 확인 | 스키마 | 0 | teul-cli 미기동(스키마 단계) | 해당 없음 | count=0 확인 (teul-cli 미기동(스키마 단계)) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `tensor_v0_sparse` | 실행 실패 + count=0 확인 | 스키마 | 0 | teul-cli 미기동(스키마 단계) | 해당 없음 | count=0 확인 (teul-cli 미기동(스키마 단계)) | 실행 실패 + count=0. 실패 단계=스키마; 포화 수리 금지, 1차 분류만. |
| `toolchain_pack_3_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `toolchain_pack_4_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `toolchain_pack_5_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `ttonimaru_registry_2_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `ttonimaru_registry_3_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `ttonimaru_registry_4_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `ttonimaru_registry_5_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `v25_first_ai_production_path_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=3 | 해당 없음 | count=0 audit line 확인 (audit log lines=3) | 영향 없음(PASS, count=0) |
| `view_only_switch_no_statehash_change_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `vol4_event_dispatch_runtime_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `vol4_multi_signal_priority_runtime_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `vol4_resume_isolation_runtime_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `vol4_state_transition_runtime_v1` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |
| `w25_all_block_sugar` | PASS + count=0 확인 | 해당 없음 | 0 | audit log lines=1 | 해당 없음 | count=0 audit line 확인 (audit log lines=1) | 영향 없음(PASS, count=0) |

## 검증

- `git ls-files pack/**/golden.jsonl` 기준 826개 pack을 대상으로 집계했다.
- 기준선 상태 분포는 PASS 727 / FAIL 98 / TIMEOUT 1로 Q7 명세와 일치한다.
- `saturation_count > 0`인 pack은 0개다.
- 무근거 0 추정 문구와 근거는 사용하지 않았다.
- FAIL 98개와 TIMEOUT 1개는 수리하지 않고 실패 단계만 1차 분류했다.
- 본 작업트리의 추적 pack 산출물 변경은 발생하지 않았다. 별도 worktree에 생긴 실행 산출물은 삭제하지 않았다.

