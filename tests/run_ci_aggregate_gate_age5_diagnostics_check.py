#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from _ci_fail_and_exit_contract import (
    FAST_FAIL_REENTRY_GUARD_TOKENS,
    validate_fail_and_exit_block_contract,
)


AGE5_RUNNER_TOKENS = [
    "age5_close",
    "tests/run_age5_close.py",
    "tools/scripts/print_age5_close_digest.py",
    "tests/run_age5_close_digest_selftest.py",
    "tools/scripts/print_ci_aggregate_digest.py",
    "--require-age5",
    "--age5-report",
    "check_age5_close_digest_selftest",
    "age5_close_digest_selftest",
    "check_ci_aggregate_gate_age5_diagnostics",
    "ci_aggregate_gate_age5_diagnostics_check",
    "tests/run_ci_aggregate_gate_age5_diagnostics_check.py",
    "ci_aggregate_gate_age5_diagnostics_rc",
]

AGE5_AGGREGATE_DIGEST_TOKENS = [
    "age5_age4_proof_snapshot_fields_text=",
    "age5_age4_proof_snapshot_text=",
    "age5_age4_proof_gate_result_present=",
    "age5_age4_proof_gate_result_parity=",
    "age5_age4_proof_final_status_parse_present=",
    "age5_age4_proof_final_status_parse_parity=",
    "age5_combined_heavy_child_timeout_sec=",
    "age5_combined_heavy_timeout_mode=",
    "age5_combined_heavy_timeout_present=",
    "age5_combined_heavy_timeout_targets=",
    "age5_policy_age4_proof_snapshot_fields_text=",
    "age5_policy_age4_proof_source_snapshot_fields_text=",
    "age5_policy_age4_proof_snapshot_text=",
    "age5_policy_age4_proof_gate_result_present=",
    "age5_policy_age4_proof_gate_result_parity=",
    "age5_policy_age4_proof_final_status_parse_present=",
    "age5_policy_age4_proof_final_status_parse_parity=",
]

AGE5_GATE_CHAIN_TOKENS = [
    "check_ci_profile_split_contract",
    "ci_profile_split_contract_check",
    "tests/run_ci_profile_split_contract_check.py",
    "check_ci_profile_matrix_gate_selftest",
    "ci_profile_matrix_gate_selftest",
    "tests/run_ci_profile_matrix_gate_selftest.py",
    "check_ci_sync_readiness_report_check",
    "ci_sync_readiness_report_check",
    "tests/run_ci_sync_readiness_report_check.py",
    "ci_pack_golden_overlay_compare_selftest",
    "tests/run_pack_golden_overlay_compare_selftest.py",
    "ci_pack_golden_overlay_session_selftest",
    "tests/run_pack_golden_overlay_session_selftest.py",
    "ci_pack_golden_age5_surface_selftest",
    "tests/run_pack_golden_age5_surface_selftest.py",
    "ci_pack_golden_guideblock_selftest",
    "tests/run_pack_golden_guideblock_selftest.py",
    "ci_pack_golden_exec_policy_selftest",
    "tests/run_pack_golden_exec_policy_selftest.py",
    "ci_pack_golden_jjaim_flatten_selftest",
    "tests/run_pack_golden_jjaim_flatten_selftest.py",
    "ci_pack_golden_event_model_selftest",
    "tests/run_pack_golden_event_model_selftest.py",
    "ci_gate_report_index_check",
    "tests/run_ci_gate_report_index_check.py",
    "ci_gate_report_index_selftest",
    "tests/run_ci_gate_report_index_check_selftest.py",
    "ci_gate_report_index_diagnostics_check",
    "tests/run_ci_gate_report_index_diagnostics_check.py",
    "ci_fail_and_exit_contract_selftest",
    "tests/run_ci_fail_and_exit_contract_selftest.py",
    "ci_gate_report_index_latest_smoke_check",
    "tests/run_ci_gate_report_index_latest_smoke_check.py",
]

AGE5_SUMMARY_TOKENS = [
    "[ci-gate-summary] ci_pack_golden_overlay_compare_selftest_ok=",
    "[ci-gate-summary] ci_pack_golden_overlay_session_selftest_ok=",
    "[ci-gate-summary] ci_pack_golden_age5_surface_selftest_ok=",
    "[ci-gate-summary] ci_pack_golden_guideblock_selftest_ok=",
    "[ci-gate-summary] ci_pack_golden_exec_policy_selftest_ok=",
    "[ci-gate-summary] ci_pack_golden_jjaim_flatten_selftest_ok=",
    "[ci-gate-summary] ci_pack_golden_event_model_selftest_ok=",
    "[ci-gate-summary] age5_status=",
    "age5_close_report.detjson",
    "append_age5_child_summary_lines(",
    "AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS",
    "build_age5_combined_heavy_child_summary_default_text_transport_fields",
    "age5_child_summary_defaults=",
    "age5_sync_child_summary_defaults=",
    "age5_full_real_source_check=",
    "age5_full_real_source_selftest=",
    "age5_combined_heavy_child_timeout_sec=",
    "age5_combined_heavy_timeout_mode=",
    "age5_combined_heavy_timeout_present=",
    "age5_combined_heavy_timeout_targets=",
    "combined_digest_selftest_default_field_text=",
    "combined_digest_selftest_default_field=",
    "age5_full_real_smoke_check_script=",
    "age5_full_real_smoke_check_script_exists=",
    "age5_full_real_smoke_check_selftest_script=",
    "age5_full_real_smoke_check_selftest_script_exists=",
    "age5_full_real_source_trace_text=",
    "age5_full_real_w107_golden_index_selftest_active_cases=",
    "age5_full_real_w107_golden_index_selftest_inactive_cases=",
    "age5_full_real_w107_golden_index_selftest_index_codes=",
    "age5_full_real_w107_golden_index_selftest_current_probe=",
    "age5_full_real_w107_golden_index_selftest_last_completed_probe=",
    "age5_full_real_w107_golden_index_selftest_progress_present=",
    "age5_full_real_w107_progress_contract_selftest_completed_checks=",
    "age5_full_real_w107_progress_contract_selftest_total_checks=",
    "age5_full_real_w107_progress_contract_selftest_checks_text=",
    "age5_full_real_w107_progress_contract_selftest_current_probe=",
    "age5_full_real_w107_progress_contract_selftest_last_completed_probe=",
    "age5_full_real_w107_progress_contract_selftest_progress_present=",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks=",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks=",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text=",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe=",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe=",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present=",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks=",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks=",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text=",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe=",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe=",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present=",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks=",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks=",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text=",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe=",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe=",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present=",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks=",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks=",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text=",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe=",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe=",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present=",
    "age5_full_real_proof_certificate_family_contract_selftest_completed_checks=",
    "age5_full_real_proof_certificate_family_contract_selftest_total_checks=",
    "age5_full_real_proof_certificate_family_contract_selftest_checks_text=",
    "age5_full_real_proof_certificate_family_contract_selftest_current_probe=",
    "age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe=",
    "age5_full_real_proof_certificate_family_contract_selftest_progress_present=",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks=",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks=",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text=",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe=",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe=",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present=",
    "age5_full_real_proof_family_contract_selftest_completed_checks=",
    "age5_full_real_proof_family_contract_selftest_total_checks=",
    "age5_full_real_proof_family_contract_selftest_checks_text=",
    "age5_full_real_proof_family_contract_selftest_current_probe=",
    "age5_full_real_proof_family_contract_selftest_last_completed_probe=",
    "age5_full_real_proof_family_contract_selftest_progress_present=",
    "age5_full_real_proof_family_transport_contract_selftest_completed_checks=",
    "age5_full_real_proof_family_transport_contract_selftest_total_checks=",
    "age5_full_real_proof_family_transport_contract_selftest_checks_text=",
    "age5_full_real_proof_family_transport_contract_selftest_current_probe=",
    "age5_full_real_proof_family_transport_contract_selftest_last_completed_probe=",
    "age5_full_real_proof_family_transport_contract_selftest_progress_present=",
    "age5_full_real_lang_surface_family_contract_selftest_completed_checks=",
    "age5_full_real_lang_surface_family_contract_selftest_total_checks=",
    "age5_full_real_lang_surface_family_contract_selftest_checks_text=",
    "age5_full_real_lang_surface_family_contract_selftest_current_probe=",
    "age5_full_real_lang_surface_family_contract_selftest_last_completed_probe=",
    "age5_full_real_lang_surface_family_contract_selftest_progress_present=",
    "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks=",
    "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks=",
    "age5_full_real_lang_surface_family_transport_contract_selftest_checks_text=",
    "age5_full_real_lang_surface_family_transport_contract_selftest_current_probe=",
    "age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe=",
    "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present=",
    "age5_full_real_lang_runtime_family_contract_selftest_completed_checks=",
    "age5_full_real_lang_runtime_family_contract_selftest_total_checks=",
    "age5_full_real_lang_runtime_family_contract_selftest_checks_text=",
    "age5_full_real_lang_runtime_family_contract_selftest_current_probe=",
    "age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe=",
    "age5_full_real_lang_runtime_family_contract_selftest_progress_present=",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks=",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks=",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text=",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe=",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe=",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present=",
    "age5_full_real_gate0_family_contract_selftest_completed_checks=",
    "age5_full_real_gate0_family_contract_selftest_total_checks=",
    "age5_full_real_gate0_family_contract_selftest_checks_text=",
    "age5_full_real_gate0_family_contract_selftest_current_probe=",
    "age5_full_real_gate0_family_contract_selftest_last_completed_probe=",
    "age5_full_real_gate0_family_contract_selftest_progress_present=",
    "age5_full_real_gate0_surface_family_contract_selftest_completed_checks=",
    "age5_full_real_gate0_surface_family_contract_selftest_total_checks=",
    "age5_full_real_gate0_surface_family_contract_selftest_checks_text=",
    "age5_full_real_gate0_surface_family_contract_selftest_current_probe=",
    "age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe=",
    "age5_full_real_gate0_surface_family_contract_selftest_progress_present=",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks=",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks=",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text=",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe=",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe=",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present=",
    "age5_full_real_gate0_family_transport_contract_selftest_completed_checks=",
    "age5_full_real_gate0_family_transport_contract_selftest_total_checks=",
    "age5_full_real_gate0_family_transport_contract_selftest_checks_text=",
    "age5_full_real_gate0_family_transport_contract_selftest_current_probe=",
    "age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe=",
    "age5_full_real_gate0_family_transport_contract_selftest_progress_present=",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks=",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks=",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text=",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe=",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe=",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present=",
    "age5_full_real_gate0_transport_family_contract_selftest_completed_checks=",
    "age5_full_real_gate0_transport_family_contract_selftest_total_checks=",
    "age5_full_real_gate0_transport_family_contract_selftest_checks_text=",
    "age5_full_real_gate0_transport_family_contract_selftest_current_probe=",
    "age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe=",
    "age5_full_real_gate0_transport_family_contract_selftest_progress_present=",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks=",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks=",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text=",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe=",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe=",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present=",
    "age5_full_real_bogae_alias_family_contract_selftest_completed_checks=",
    "age5_full_real_bogae_alias_family_contract_selftest_total_checks=",
    "age5_full_real_bogae_alias_family_contract_selftest_checks_text=",
    "age5_full_real_bogae_alias_family_contract_selftest_current_probe=",
    "age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe=",
    "age5_full_real_bogae_alias_family_contract_selftest_progress_present=",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks=",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks=",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text=",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe=",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe=",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present=",
    "age4_proof_snapshot_fields_text=",
    "age4_proof_snapshot_text=",
    "age4_proof_gate_result_snapshot_text=",
    "age4_proof_gate_result_snapshot_present=",
    "age4_proof_gate_result_snapshot_parity=",
    "age4_proof_final_status_parse_snapshot_text=",
    "age4_proof_final_status_parse_snapshot_present=",
    "age4_proof_final_status_parse_snapshot_parity=",
    "age5_policy_combined_digest_selftest_default_field_text=",
    "age5_policy_combined_digest_selftest_default_field=",
    "age5_policy_age4_proof_snapshot_fields_text=",
    "age5_policy_age4_proof_source_snapshot_fields_text=",
    "age5_policy_age4_proof_snapshot_text=",
    "age5_policy_age4_proof_gate_result_present=",
    "age5_policy_age4_proof_gate_result_parity=",
    "age5_policy_age4_proof_final_status_parse_present=",
    "age5_policy_age4_proof_final_status_parse_parity=",
    "age5_policy_report=",
    "age5_policy_text=",
]

REQUIRED_TOKENS = [
    *AGE5_RUNNER_TOKENS,
    *AGE5_GATE_CHAIN_TOKENS,
    *FAST_FAIL_REENTRY_GUARD_TOKENS,
    *AGE5_SUMMARY_TOKENS,
]

AGE5_CLOSE_REQUIRED_CRITERIA_TOKENS = [
    "AGE5_SURFACE_PACK_CONTRACTS",
    "age5_surface_pack_contract_paths_present",
    "age5_surface_pack_contract_min_cases",
    "age5_surface_pack_contract_tokens_present",
    "CI_PROFILE_GATE_SCRIPTS",
    "age5_ci_profile_gate_scripts_present",
    "age5_ci_profile_gate_sync_chain_tokens",
    "age5_ci_profile_split_contract_tokens_present",
    "age5_ci_profile_gate_report_path_tokens",
    "CI_SYNC_READINESS_REPORT_PATH_CONTRACT_SCRIPT",
    "age5_ci_sync_readiness_report_path_contract_tokens",
    "CI_GATE_REPORT_INDEX_CONTRACT_SCRIPT",
    "CI_GATE_REPORT_INDEX_CHECK_SCRIPT",
    "CI_GATE_REPORT_INDEX_SELFTEST_SCRIPT",
    "CI_GATE_REPORT_INDEX_DIAGNOSTICS_SCRIPT",
    "CI_GATE_REPORT_INDEX_CODE_MAP",
    "CI_GATE_REPORT_INDEX_CONTRACT_TOKENS",
    "CI_GATE_REPORT_INDEX_CHECK_TOKENS",
    "CI_GATE_REPORT_INDEX_CODE_MAP_TOKENS",
    "age5_ci_gate_report_index_contract_paths_present",
    "age5_ci_gate_report_index_contract_tokens_present",
    "GATE_REPORT_INDEX_CODES",
    "build_age5_combined_heavy_child_summary_default_text_transport_fields",
    "CI_SEAMGRIM_DIAG_PARITY_SCRIPTS",
    "CI_SEAMGRIM_WASM_CLI_DIAG_PARITY_TOKENS",
    "age5_seamgrim_diag_parity_scripts_present",
    "age5_seamgrim_wasm_cli_diag_parity_tokens_present",
    "guideblock_keys_basics",
    "exec_policy_effect_diag",
]

AGE5_SURFACE_SELFTEST_REQUIRED_PACK_TOKENS = [
    "seamgrim_bogae_madang_alias_v1",
    "seamgrim_moyang_template_instance_view_boundary_v1",
    "seamgrim_jjaim_block_stub_canon_v1",
    "seamgrim_guseong_flatten_ir_v1",
    "seamgrim_guseong_flatten_diag_v1",
    "seamgrim_event_model_ir_v1",
    "seamgrim_event_surface_canon_v1",
    "block_header_no_colon",
]
GUIDEBLOCK_SELFTEST_REQUIRED_PACK_TOKENS = [
    "guideblock_keys_basics",
]

MOYANG_TEMPLATE_PACK_REQUIRED_CASE_TOKENS = [
    "smoke_template_instance_a.v1.json",
    "smoke_template_instance_b.v1.json",
    "smoke_template_instance_c.v1.json",
    "input_template_instance_a.ddn",
    "input_template_instance_b.ddn",
]

GUIDEBLOCK_KEYS_PACK_REQUIRED_CASE_TOKENS = [
    "c01_hash_header_alias/case.detjson",
    "c02_guideblock_alias/case.detjson",
    "c03_mixed_precedence/case.detjson",
    "c04_canonical_keys/case.detjson",
]

EXEC_POLICY_DIAG_PACK_REQUIRED_CASE_TOKENS = [
    "c01_strict_effect_call_EXPECT_FAIL",
    "E_EFFECT_IN_STRICT_MODE",
    "c22_strict_effect_policy_ignored_warn",
    "W_EFFECT_POLICY_IGNORED_IN_STRICT",
    "c26_effect_block_alias_baggat_EXPECT_FAIL",
    "E_EFFECT_SURFACE_ALIAS_FORBIDDEN",
]

JJAIM_FLATTEN_SELFTEST_REQUIRED_PACK_TOKENS = [
    "seamgrim_guseong_flatten_ir_v1",
    "seamgrim_guseong_flatten_diag_v1",
]
EVENT_MODEL_SELFTEST_REQUIRED_PACK_TOKENS = [
    "seamgrim_event_surface_canon_v1",
    "seamgrim_event_model_ir_v1",
]

CI_SEAMGRIM_WASM_CLI_DIAG_PARITY_REQUIRED_TOKENS = [
    "W_BLOCK_HEADER_COLON_DEPRECATED",
    "E_EVENT_SURFACE_ALIAS_FORBIDDEN",
    "BLOCK_HEADER_MIN_CASES = 5",
    "EVENT_SURFACE_MIN_CASES = 7",
    "tests/seamgrim_wasm_wrapper_runner.mjs",
    "tests/seamgrim_wasm_vm_runtime_runner.mjs",
    "tests/run_pack_golden.py",
    "block_header_no_colon",
    "seamgrim_event_surface_canon_v1",
    "tests/run_seamgrim_overlay_compare_diag_parity_check.py",
    "tests/run_seamgrim_overlay_session_diag_parity_check.py",
    "tests/run_seamgrim_overlay_session_wired_consistency_check.py",
    "tests/run_numeric_factor_route_diag_contract_check.py",
    "overlay compare diag parity check ok",
    "overlay session diag parity check ok",
    "overlay session wired consistency check ok",
    "numeric factor route diag contract check ok",
]

EXEC_POLICY_MAP_PACK_REQUIRED_CASE_TOKENS = [
    "c01_general_allowed",
    "c02_strict_forces_isolated",
    "c03_duplicate_exec_policy_blocks",
    "c04_no_policy_defaults",
    "c05_invalid_effect_enum_gate_error",
    "c06_effect_only_allowed_defaults_strict",
]

BLOCK_HEADER_NO_COLON_PACK_REQUIRED_CASE_TOKENS = [
    "c01_decl_colon_warn",
    "c02_repeat_foreach_colon_warn",
    "c03_while_colon_warn",
    "c04_nested_multi_colon_warn",
    "c05_mixed_colon_no_colon_warn",
    "W_BLOCK_HEADER_COLON_DEPRECATED",
]

JJAIM_FLATTEN_IR_PACK_REQUIRED_CASE_TOKENS = [
    "c01_basic",
    "c02_guseong_alias_warn",
    "W_JJAIM_ALIAS_DEPRECATED",
]
JJAIM_FLATTEN_DIAG_PACK_REQUIRED_CASE_TOKENS = [
    "c14_type_schema_conflict_EXPECT_FAIL",
    "E_JJAIM_TYPE_SCHEMA_CONFLICT",
    "c16_type_tag_required_multi_type_EXPECT_FAIL",
    "E_JJAIM_TYPE_TAG_REQUIRED",
    "c17_tuple_named_index_EXPECT_FAIL",
    "c18_tuple_out_of_range_index_EXPECT_FAIL",
    "E_GUSEONG_TUPLE_INDEX_INVALID",
    "c19_tuple_access_on_scalar_output_EXPECT_FAIL",
    "c20_tuple_access_on_scalar_input_EXPECT_FAIL",
    "E_GUSEONG_TUPLE_ACCESS_ON_SCALAR",
    "c21_tuple_projection_valid_success",
]

EVENT_MODEL_IR_PACK_REQUIRED_CASE_TOKENS = [
    "c01_basic",
    "c02_alias_surface_emit_EXPECT_FAIL",
    "E_EVENT_SURFACE_ALIAS_FORBIDDEN",
    "alrim-plan-json",
]

BOGAE_MADANG_ALIAS_PACK_REQUIRED_CASE_TOKENS = [
    "c01_alias_warn",
    "W_BOGAE_MADANG_ALIAS_DEPRECATED",
    "c02_canonical_no_warn",
]

JJAIM_BLOCK_STUB_PACK_REQUIRED_CASE_TOKENS = [
    "c01_guseong_alias",
    "W_JJAIM_ALIAS_DEPRECATED",
    "c02_invalid_subblock_EXPECT_FAIL",
    "E_JJAIM_SUBBLOCK_INVALID",
    "c03_port_decl_name_only_EXPECT_FAIL",
    "E_JJAIM_PORT_DECL_INVALID",
    "c04_port_type_missing_EXPECT_FAIL",
    "E_JJAIM_PORT_TYPE_MISSING",
    "c05_port_duplicate_EXPECT_FAIL",
    "E_JJAIM_PORT_DUP",
    "c06_jjaim_canonical_no_warn",
]


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    target = root / "tests" / "run_ci_aggregate_gate.py"
    if not target.exists():
        print(f"missing target: {target}")
        return 1
    text = target.read_text(encoding="utf-8")

    missing = [token for token in REQUIRED_TOKENS if token not in text]
    if missing:
        print("aggregate gate age5 diagnostics check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1
    fail_and_exit_contract_issues = validate_fail_and_exit_block_contract(text)
    if fail_and_exit_contract_issues:
        print("aggregate gate age5 diagnostics check failed (fail_and_exit contract):")
        for issue in fail_and_exit_contract_issues[:12]:
            print(f" - {issue}")
        return 1

    aggregate_digest_target = root / "tools" / "scripts" / "print_ci_aggregate_digest.py"
    if not aggregate_digest_target.exists():
        print(f"missing target: {aggregate_digest_target}")
        return 1
    aggregate_digest_text = aggregate_digest_target.read_text(encoding="utf-8")
    aggregate_digest_missing = [
        token for token in AGE5_AGGREGATE_DIGEST_TOKENS if token not in aggregate_digest_text
    ]
    if aggregate_digest_missing:
        print("aggregate gate age5 diagnostics check failed (aggregate digest coverage):")
        for token in aggregate_digest_missing[:12]:
            print(f" - missing aggregate-digest token: {token}")
        return 1

    age5_close_target = root / "tests" / "run_age5_close.py"
    if not age5_close_target.exists():
        print(f"missing target: {age5_close_target}")
        return 1
    age5_close_text = age5_close_target.read_text(encoding="utf-8")
    age5_close_missing = [
        token
        for token in AGE5_CLOSE_REQUIRED_CRITERIA_TOKENS
        if token not in age5_close_text
    ]
    if age5_close_missing:
        print("aggregate gate age5 diagnostics check failed (age5 close criteria coverage):")
        for token in age5_close_missing[:12]:
            print(f" - missing age5-close token: {token}")
        return 1

    seamgrim_wasm_cli_diag_parity = root / "tests" / "run_seamgrim_wasm_cli_diag_parity_check.py"
    if not seamgrim_wasm_cli_diag_parity.exists():
        print(f"missing target: {seamgrim_wasm_cli_diag_parity}")
        return 1
    seamgrim_wasm_cli_diag_parity_text = seamgrim_wasm_cli_diag_parity.read_text(encoding="utf-8")
    seamgrim_wasm_cli_diag_parity_missing = [
        token
        for token in CI_SEAMGRIM_WASM_CLI_DIAG_PARITY_REQUIRED_TOKENS
        if token not in seamgrim_wasm_cli_diag_parity_text
    ]
    if seamgrim_wasm_cli_diag_parity_missing:
        print("aggregate gate age5 diagnostics check failed (seamgrim wasm/cli parity coverage):")
        for token in seamgrim_wasm_cli_diag_parity_missing[:12]:
            print(f" - missing seamgrim-wasm-cli-parity token: {token}")
        return 1

    surface_selftest = root / "tests" / "run_pack_golden_age5_surface_selftest.py"
    if not surface_selftest.exists():
        print(f"missing target: {surface_selftest}")
        return 1
    surface_text = surface_selftest.read_text(encoding="utf-8")
    surface_missing = [
        token
        for token in AGE5_SURFACE_SELFTEST_REQUIRED_PACK_TOKENS
        if token not in surface_text
    ]
    if surface_missing:
        print("aggregate gate age5 diagnostics check failed (surface selftest coverage):")
        for token in surface_missing[:12]:
            print(f" - missing surface token: {token}")
        return 1

    guideblock_selftest = root / "tests" / "run_pack_golden_guideblock_selftest.py"
    if not guideblock_selftest.exists():
        print(f"missing target: {guideblock_selftest}")
        return 1
    guideblock_selftest_text = guideblock_selftest.read_text(encoding="utf-8")
    guideblock_selftest_missing = [
        token
        for token in GUIDEBLOCK_SELFTEST_REQUIRED_PACK_TOKENS
        if token not in guideblock_selftest_text
    ]
    if guideblock_selftest_missing:
        print("aggregate gate age5 diagnostics check failed (guideblock selftest coverage):")
        for token in guideblock_selftest_missing[:12]:
            print(f" - missing guideblock-selftest token: {token}")
        return 1

    jjaim_flatten_selftest = root / "tests" / "run_pack_golden_jjaim_flatten_selftest.py"
    if not jjaim_flatten_selftest.exists():
        print(f"missing target: {jjaim_flatten_selftest}")
        return 1
    jjaim_flatten_text = jjaim_flatten_selftest.read_text(encoding="utf-8")
    jjaim_flatten_missing = [
        token
        for token in JJAIM_FLATTEN_SELFTEST_REQUIRED_PACK_TOKENS
        if token not in jjaim_flatten_text
    ]
    if jjaim_flatten_missing:
        print("aggregate gate age5 diagnostics check failed (jjaim flatten selftest coverage):")
        for token in jjaim_flatten_missing[:12]:
            print(f" - missing jjaim flatten token: {token}")
        return 1

    event_model_selftest = root / "tests" / "run_pack_golden_event_model_selftest.py"
    if not event_model_selftest.exists():
        print(f"missing target: {event_model_selftest}")
        return 1
    event_model_selftest_text = event_model_selftest.read_text(encoding="utf-8")
    event_model_selftest_missing = [
        token
        for token in EVENT_MODEL_SELFTEST_REQUIRED_PACK_TOKENS
        if token not in event_model_selftest_text
    ]
    if event_model_selftest_missing:
        print("aggregate gate age5 diagnostics check failed (event model selftest coverage):")
        for token in event_model_selftest_missing[:12]:
            print(f" - missing event-model-selftest token: {token}")
        return 1

    exec_policy_map_pack = root / "pack" / "seamgrim_exec_policy_effect_map_v1" / "golden.jsonl"
    if not exec_policy_map_pack.exists():
        print(f"missing target: {exec_policy_map_pack}")
        return 1
    exec_policy_map_pack_text = exec_policy_map_pack.read_text(encoding="utf-8")
    exec_policy_map_missing = [
        token
        for token in EXEC_POLICY_MAP_PACK_REQUIRED_CASE_TOKENS
        if token not in exec_policy_map_pack_text
    ]
    if exec_policy_map_missing:
        print("aggregate gate age5 diagnostics check failed (exec policy map pack coverage):")
        for token in exec_policy_map_missing[:12]:
            print(f" - missing exec-policy-map token: {token}")
        return 1

    block_header_pack = root / "pack" / "block_header_no_colon" / "golden.jsonl"
    if not block_header_pack.exists():
        print(f"missing target: {block_header_pack}")
        return 1
    block_header_pack_text = block_header_pack.read_text(encoding="utf-8")
    block_header_missing = [
        token
        for token in BLOCK_HEADER_NO_COLON_PACK_REQUIRED_CASE_TOKENS
        if token not in block_header_pack_text
    ]
    if block_header_missing:
        print("aggregate gate age5 diagnostics check failed (block header pack coverage):")
        for token in block_header_missing[:12]:
            print(f" - missing block-header token: {token}")
        return 1

    jjaim_flatten_ir_pack = root / "pack" / "seamgrim_guseong_flatten_ir_v1" / "golden.jsonl"
    if not jjaim_flatten_ir_pack.exists():
        print(f"missing target: {jjaim_flatten_ir_pack}")
        return 1
    jjaim_flatten_ir_pack_text = jjaim_flatten_ir_pack.read_text(encoding="utf-8")
    jjaim_flatten_ir_missing = [
        token
        for token in JJAIM_FLATTEN_IR_PACK_REQUIRED_CASE_TOKENS
        if token not in jjaim_flatten_ir_pack_text
    ]
    if jjaim_flatten_ir_missing:
        print("aggregate gate age5 diagnostics check failed (jjaim flatten ir pack coverage):")
        for token in jjaim_flatten_ir_missing[:12]:
            print(f" - missing jjaim-flatten-ir token: {token}")
        return 1

    jjaim_flatten_diag_pack = root / "pack" / "seamgrim_guseong_flatten_diag_v1" / "golden.jsonl"
    if not jjaim_flatten_diag_pack.exists():
        print(f"missing target: {jjaim_flatten_diag_pack}")
        return 1
    jjaim_flatten_diag_pack_text = jjaim_flatten_diag_pack.read_text(encoding="utf-8")
    jjaim_flatten_diag_missing = [
        token
        for token in JJAIM_FLATTEN_DIAG_PACK_REQUIRED_CASE_TOKENS
        if token not in jjaim_flatten_diag_pack_text
    ]
    if jjaim_flatten_diag_missing:
        print("aggregate gate age5 diagnostics check failed (jjaim flatten diag pack coverage):")
        for token in jjaim_flatten_diag_missing[:12]:
            print(f" - missing jjaim-flatten-diag token: {token}")
        return 1

    event_model_ir_pack = root / "pack" / "seamgrim_event_model_ir_v1" / "golden.jsonl"
    if not event_model_ir_pack.exists():
        print(f"missing target: {event_model_ir_pack}")
        return 1
    event_model_ir_pack_text = event_model_ir_pack.read_text(encoding="utf-8")
    event_model_ir_missing = [
        token
        for token in EVENT_MODEL_IR_PACK_REQUIRED_CASE_TOKENS
        if token not in event_model_ir_pack_text
    ]
    if event_model_ir_missing:
        print("aggregate gate age5 diagnostics check failed (event model ir pack coverage):")
        for token in event_model_ir_missing[:12]:
            print(f" - missing event-model-ir token: {token}")
        return 1

    moyang_template_pack = root / "pack" / "seamgrim_moyang_template_instance_view_boundary_v1" / "golden.jsonl"
    if not moyang_template_pack.exists():
        print(f"missing target: {moyang_template_pack}")
        return 1
    moyang_template_pack_text = moyang_template_pack.read_text(encoding="utf-8")
    moyang_template_missing = [
        token
        for token in MOYANG_TEMPLATE_PACK_REQUIRED_CASE_TOKENS
        if token not in moyang_template_pack_text
    ]
    if moyang_template_missing:
        print("aggregate gate age5 diagnostics check failed (moyang template pack coverage):")
        for token in moyang_template_missing[:12]:
            print(f" - missing moyang-template token: {token}")
        return 1

    guideblock_keys_pack = root / "pack" / "guideblock_keys_basics" / "golden.jsonl"
    if not guideblock_keys_pack.exists():
        print(f"missing target: {guideblock_keys_pack}")
        return 1
    guideblock_keys_pack_text = guideblock_keys_pack.read_text(encoding="utf-8")
    guideblock_keys_missing = [
        token
        for token in GUIDEBLOCK_KEYS_PACK_REQUIRED_CASE_TOKENS
        if token not in guideblock_keys_pack_text
    ]
    if guideblock_keys_missing:
        print("aggregate gate age5 diagnostics check failed (guideblock keys pack coverage):")
        for token in guideblock_keys_missing[:12]:
            print(f" - missing guideblock token: {token}")
        return 1

    exec_policy_diag_pack = root / "pack" / "seamgrim_exec_policy_effect_diag_v1" / "golden.jsonl"
    if not exec_policy_diag_pack.exists():
        print(f"missing target: {exec_policy_diag_pack}")
        return 1
    exec_policy_diag_pack_text = exec_policy_diag_pack.read_text(encoding="utf-8")
    exec_policy_diag_missing = [
        token
        for token in EXEC_POLICY_DIAG_PACK_REQUIRED_CASE_TOKENS
        if token not in exec_policy_diag_pack_text
    ]
    if exec_policy_diag_missing:
        print("aggregate gate age5 diagnostics check failed (exec policy diag pack coverage):")
        for token in exec_policy_diag_missing[:12]:
            print(f" - missing exec-policy-diag token: {token}")
        return 1

    bogae_madang_alias_pack = root / "pack" / "seamgrim_bogae_madang_alias_v1" / "golden.jsonl"
    if not bogae_madang_alias_pack.exists():
        print(f"missing target: {bogae_madang_alias_pack}")
        return 1
    bogae_madang_alias_pack_text = bogae_madang_alias_pack.read_text(encoding="utf-8")
    bogae_madang_alias_missing = [
        token
        for token in BOGAE_MADANG_ALIAS_PACK_REQUIRED_CASE_TOKENS
        if token not in bogae_madang_alias_pack_text
    ]
    if bogae_madang_alias_missing:
        print("aggregate gate age5 diagnostics check failed (bogae-madang alias pack coverage):")
        for token in bogae_madang_alias_missing[:12]:
            print(f" - missing bogae-madang token: {token}")
        return 1

    jjaim_block_stub_pack = root / "pack" / "seamgrim_jjaim_block_stub_canon_v1" / "golden.jsonl"
    if not jjaim_block_stub_pack.exists():
        print(f"missing target: {jjaim_block_stub_pack}")
        return 1
    jjaim_block_stub_pack_text = jjaim_block_stub_pack.read_text(encoding="utf-8")
    jjaim_block_stub_missing = [
        token
        for token in JJAIM_BLOCK_STUB_PACK_REQUIRED_CASE_TOKENS
        if token not in jjaim_block_stub_pack_text
    ]
    if jjaim_block_stub_missing:
        print("aggregate gate age5 diagnostics check failed (jjaim block stub pack coverage):")
        for token in jjaim_block_stub_missing[:12]:
            print(f" - missing jjaim-block token: {token}")
        return 1

    print("ci aggregate gate age5 diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
