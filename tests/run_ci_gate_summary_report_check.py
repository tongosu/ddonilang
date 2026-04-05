#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

from _ci_age3_completion_gate_contract import (
    AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS,
    AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS,
)
from _ci_age5_combined_heavy_contract import (
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY,
    AGE5_COMBINED_HEAVY_SANITY_CONTRACT_SUMMARY_FIELDS,
    AGE5_COMBINED_HEAVY_SYNC_CONTRACT_SUMMARY_FIELDS,
    AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_ACTIVE_CASES_KEY,
    AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_INACTIVE_CASES_KEY,
    AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_INDEX_CODES_KEY,
    AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    build_age5_combined_heavy_policy_origin_trace,
    build_age5_combined_heavy_policy_origin_trace_text,
)
from _ci_profile_matrix_selftest_lib import (
    PROFILE_MATRIX_SELFTEST_PROFILES,
    PROFILE_MATRIX_SELFTEST_SCHEMA,
    build_profile_matrix_snapshot_from_doc,
    expected_profile_matrix_aggregate_summary_contract,
    format_profile_matrix_summary_values,
)
from ci_check_error_codes import SUMMARY_REPORT_CODES as CODES

PASS_REQUIRED_KEYS = (
    "report_index",
    "summary_line",
    "ci_gate_result",
    "ci_gate_badge",
    "ci_fail_brief_hint",
    "ci_fail_brief_exists",
    "ci_fail_triage_hint",
    "ci_fail_triage_exists",
    "ci_profile_matrix_gate_selftest_report",
    "ci_profile_matrix_gate_selftest_status",
    "ci_profile_matrix_gate_selftest_ok",
    "ci_profile_matrix_gate_selftest_total_elapsed_ms",
    "ci_profile_matrix_gate_selftest_selected_real_profiles",
    "ci_profile_matrix_gate_selftest_skipped_real_profiles",
    "ci_profile_matrix_gate_selftest_step_timeout_defaults",
    "ci_profile_matrix_gate_selftest_core_lang_elapsed_ms",
    "ci_profile_matrix_gate_selftest_full_elapsed_ms",
    "ci_profile_matrix_gate_selftest_seamgrim_elapsed_ms",
    "ci_profile_matrix_gate_selftest_aggregate_summary_sanity_ok",
    "ci_profile_matrix_gate_selftest_aggregate_summary_checked_profiles",
    "ci_profile_matrix_gate_selftest_aggregate_summary_failed_profiles",
    "ci_profile_matrix_gate_selftest_aggregate_summary_skipped_profiles",
    "ci_profile_matrix_gate_selftest_core_lang_aggregate_summary_status",
    "ci_profile_matrix_gate_selftest_core_lang_aggregate_summary_ok",
    "ci_profile_matrix_gate_selftest_core_lang_aggregate_summary_values",
    "ci_profile_matrix_gate_selftest_full_aggregate_summary_status",
    "ci_profile_matrix_gate_selftest_full_aggregate_summary_ok",
    "ci_profile_matrix_gate_selftest_full_aggregate_summary_values",
    "ci_profile_matrix_gate_selftest_seamgrim_aggregate_summary_status",
    "ci_profile_matrix_gate_selftest_seamgrim_aggregate_summary_ok",
    "ci_profile_matrix_gate_selftest_seamgrim_aggregate_summary_values",
    "age2_status",
    "age3_status",
    "age4_status",
    "age4_proof_ok",
    "age4_proof_failed_criteria",
    "age4_proof_failed_preview",
    "age4_proof_summary_hash",
    "age5_status",
    "age5_close_digest_selftest_ok",
    "age5_policy_combined_digest_selftest_default_field_text",
    "age5_policy_combined_digest_selftest_default_field",
    "age5_combined_heavy_policy_report_path",
    "age5_combined_heavy_policy_report_exists",
    "age5_combined_heavy_policy_text_path",
    "age5_combined_heavy_policy_text_exists",
    "age5_combined_heavy_policy_summary_path",
    "age5_combined_heavy_policy_summary_exists",
    "age5_combined_heavy_policy_origin_trace_text",
    "age5_combined_heavy_policy_origin_trace",
    "age5_policy_summary_origin_trace_contract_issue",
    "age5_policy_summary_origin_trace_contract_source_issue",
    "age5_policy_summary_origin_trace_contract_compact_reason",
    "age5_policy_summary_origin_trace_contract_compact_failure_reason",
    "age5_combined_heavy_full_real_status",
    "age5_combined_heavy_runtime_helper_negative_status",
    "age5_combined_heavy_group_id_summary_negative_status",
    "age5_full_real_w107_golden_index_selftest_active_cases",
    "age5_full_real_w107_golden_index_selftest_inactive_cases",
    "age5_full_real_w107_golden_index_selftest_index_codes",
    "age5_full_real_w107_golden_index_selftest_current_probe",
    "age5_full_real_w107_golden_index_selftest_last_completed_probe",
    "age5_full_real_w107_golden_index_selftest_progress_present",
    "age5_full_real_w107_progress_contract_selftest_completed_checks",
    "age5_full_real_w107_progress_contract_selftest_total_checks",
    "age5_full_real_w107_progress_contract_selftest_checks_text",
    "age5_full_real_w107_progress_contract_selftest_current_probe",
    "age5_full_real_w107_progress_contract_selftest_last_completed_probe",
    "age5_full_real_w107_progress_contract_selftest_progress_present",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present",
    "age5_full_real_proof_certificate_family_contract_selftest_completed_checks",
    "age5_full_real_proof_certificate_family_contract_selftest_total_checks",
    "age5_full_real_proof_certificate_family_contract_selftest_checks_text",
    "age5_full_real_proof_certificate_family_contract_selftest_current_probe",
    "age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe",
    "age5_full_real_proof_certificate_family_contract_selftest_progress_present",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present",
    "age5_full_real_proof_family_contract_selftest_completed_checks",
    "age5_full_real_proof_family_contract_selftest_total_checks",
    "age5_full_real_proof_family_contract_selftest_checks_text",
    "age5_full_real_proof_family_contract_selftest_current_probe",
    "age5_full_real_proof_family_contract_selftest_last_completed_probe",
    "age5_full_real_proof_family_contract_selftest_progress_present",
    "age5_full_real_proof_family_transport_contract_selftest_completed_checks",
    "age5_full_real_proof_family_transport_contract_selftest_total_checks",
    "age5_full_real_proof_family_transport_contract_selftest_checks_text",
    "age5_full_real_proof_family_transport_contract_selftest_current_probe",
    "age5_full_real_proof_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_proof_family_transport_contract_selftest_progress_present",
    "age5_full_real_lang_surface_family_contract_selftest_completed_checks",
    "age5_full_real_lang_surface_family_contract_selftest_total_checks",
    "age5_full_real_lang_surface_family_contract_selftest_checks_text",
    "age5_full_real_lang_surface_family_contract_selftest_current_probe",
    "age5_full_real_lang_surface_family_contract_selftest_last_completed_probe",
    "age5_full_real_lang_surface_family_contract_selftest_progress_present",
    "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks",
    "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks",
    "age5_full_real_lang_surface_family_transport_contract_selftest_checks_text",
    "age5_full_real_lang_surface_family_transport_contract_selftest_current_probe",
    "age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present",
    "age5_full_real_lang_runtime_family_contract_selftest_completed_checks",
    "age5_full_real_lang_runtime_family_contract_selftest_total_checks",
    "age5_full_real_lang_runtime_family_contract_selftest_checks_text",
    "age5_full_real_lang_runtime_family_contract_selftest_current_probe",
    "age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe",
    "age5_full_real_lang_runtime_family_contract_selftest_progress_present",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present",
    "age5_full_real_gate0_family_contract_selftest_completed_checks",
    "age5_full_real_gate0_family_contract_selftest_total_checks",
    "age5_full_real_gate0_family_contract_selftest_checks_text",
    "age5_full_real_gate0_family_contract_selftest_current_probe",
    "age5_full_real_gate0_family_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_family_contract_selftest_progress_present",
    "age5_full_real_gate0_surface_family_contract_selftest_completed_checks",
    "age5_full_real_gate0_surface_family_contract_selftest_total_checks",
    "age5_full_real_gate0_surface_family_contract_selftest_checks_text",
    "age5_full_real_gate0_surface_family_contract_selftest_current_probe",
    "age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_surface_family_contract_selftest_progress_present",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present",
    "age5_full_real_gate0_family_transport_contract_selftest_completed_checks",
    "age5_full_real_gate0_family_transport_contract_selftest_total_checks",
    "age5_full_real_gate0_family_transport_contract_selftest_checks_text",
    "age5_full_real_gate0_family_transport_contract_selftest_current_probe",
    "age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_family_transport_contract_selftest_progress_present",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present",
    "age5_full_real_gate0_transport_family_contract_selftest_completed_checks",
    "age5_full_real_gate0_transport_family_contract_selftest_total_checks",
    "age5_full_real_gate0_transport_family_contract_selftest_checks_text",
    "age5_full_real_gate0_transport_family_contract_selftest_current_probe",
    "age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_transport_family_contract_selftest_progress_present",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present",
    "age5_full_real_bogae_alias_family_contract_selftest_completed_checks",
    "age5_full_real_bogae_alias_family_contract_selftest_total_checks",
    "age5_full_real_bogae_alias_family_contract_selftest_checks_text",
    "age5_full_real_bogae_alias_family_contract_selftest_current_probe",
    "age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe",
    "age5_full_real_bogae_alias_family_contract_selftest_progress_present",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present",
    "seamgrim_phase3_cleanup",
    "seamgrim_wasm_cli_diag_parity_report",
    "seamgrim_wasm_cli_diag_parity_ok",
    "seamgrim_group_id_summary_status",
    "ci_sanity_gate_report",
    "ci_sanity_gate_status",
    "ci_sanity_gate_ok",
    "ci_sanity_gate_code",
    "ci_sanity_gate_step",
    "ci_sanity_gate_profile",
    "ci_sanity_gate_msg",
    "ci_sanity_gate_step_count",
    "ci_sanity_gate_failed_steps",
    "ci_sanity_pipeline_emit_flags_ok",
    "ci_sanity_pipeline_emit_flags_selftest_ok",
    "ci_sanity_emit_artifacts_sanity_contract_selftest_ok",
    "ci_sanity_age2_completion_gate_ok",
    "ci_sanity_age2_completion_gate_selftest_ok",
    "ci_sanity_age3_completion_gate_ok",
    "ci_sanity_age3_completion_gate_selftest_ok",
    "ci_sanity_age2_completion_gate_failure_codes",
    "ci_sanity_age2_completion_gate_failure_code_count",
    "ci_sanity_age3_completion_gate_failure_codes",
    "ci_sanity_age3_completion_gate_failure_code_count",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_ok",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_schema",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes",
    "ci_sanity_seamgrim_pack_evidence_tier_runner_ok",
    "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path",
    "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists",
    "ci_sanity_seamgrim_pack_evidence_tier_runner_schema",
    "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count",
    "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count",
    "ci_sanity_seamgrim_wasm_web_step_check_ok",
    "ci_sanity_seamgrim_wasm_web_step_check_report_path",
    "ci_sanity_seamgrim_wasm_web_step_check_report_exists",
    "ci_sanity_seamgrim_wasm_web_step_check_schema",
    "ci_sanity_seamgrim_wasm_web_step_check_checked_files",
    "ci_sanity_seamgrim_wasm_web_step_check_missing_count",
    "ci_sanity_age5_combined_heavy_policy_selftest_ok",
    "ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok",
    "ci_sanity_dynamic_source_profile_split_selftest_ok",
    "ci_sanity_age5_combined_heavy_report_schema",
    "ci_sanity_age5_combined_heavy_required_reports",
    "ci_sanity_age5_combined_heavy_required_criteria",
    "ci_sanity_age5_combined_heavy_child_summary_default_fields",
    "ci_sanity_age5_combined_heavy_combined_contract_summary_fields",
    "ci_sanity_age5_combined_heavy_full_summary_contract_fields",
    "ci_sanity_seamgrim_interface_boundary_ok",
    "ci_sanity_overlay_session_wired_consistency_ok",
    "ci_sanity_overlay_session_diag_parity_ok",
    "ci_sanity_overlay_compare_diag_parity_ok",
    "ci_sanity_pack_golden_lang_consistency_ok",
    "ci_sanity_pack_golden_metadata_ok",
    "ci_sanity_pack_golden_graph_export_ok",
    "ci_sanity_canon_ast_dpack_ok",
    "ci_sanity_contract_tier_unsupported_ok",
    "ci_sanity_contract_tier_age3_min_enforcement_ok",
    "ci_sanity_map_access_contract_ok",
    "ci_sanity_stdlib_catalog_ok",
    "ci_sanity_stdlib_catalog_selftest_ok",
    "ci_sanity_tensor_v0_pack_ok",
    "ci_sanity_tensor_v0_cli_ok",
    "ci_sanity_fixed64_darwin_real_report_contract_ok",
    "ci_sanity_fixed64_darwin_real_report_live_ok",
    "ci_sanity_fixed64_darwin_real_report_readiness_selftest_ok",
    "ci_sanity_registry_strict_audit_ok",
    "ci_sanity_registry_defaults_ok",
    "ci_sync_readiness_report",
    "ci_sync_readiness_status",
    "ci_sync_readiness_ok",
    "ci_sync_readiness_code",
    "ci_sync_readiness_step",
    "ci_sync_readiness_sanity_profile",
    "ci_sync_readiness_msg",
    "ci_sync_readiness_step_count",
    "ci_sync_readiness_ci_sanity_pipeline_emit_flags_ok",
    "ci_sync_readiness_ci_sanity_pipeline_emit_flags_selftest_ok",
    "ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok",
    "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok",
    "ci_sync_readiness_ci_sanity_age2_completion_gate_ok",
    "ci_sync_readiness_ci_sanity_age2_completion_gate_selftest_ok",
    "ci_sync_readiness_ci_sanity_age3_completion_gate_ok",
    "ci_sync_readiness_ci_sanity_age3_completion_gate_selftest_ok",
    "ci_sync_readiness_ci_sanity_age2_completion_gate_failure_codes",
    "ci_sync_readiness_ci_sanity_age2_completion_gate_failure_code_count",
    "ci_sync_readiness_ci_sanity_age3_completion_gate_failure_codes",
    "ci_sync_readiness_ci_sanity_age3_completion_gate_failure_code_count",
    "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_ok",
    "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_path",
    "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists",
    "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_schema",
    "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok",
    "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok",
    "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes",
    "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes",
    "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok",
    "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_path",
    "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists",
    "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_schema",
    "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count",
    "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count",
    "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_ok",
    "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_path",
    "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_exists",
    "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_schema",
    "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_checked_files",
    "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_missing_count",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_policy_selftest_ok",
    "ci_sync_readiness_ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok",
    "ci_sync_readiness_ci_sanity_dynamic_source_profile_split_selftest_ok",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_report_schema",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_required_reports",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_required_criteria",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_combined_contract_summary_fields",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_full_summary_contract_fields",
    "fixed64_threeway_report",
    "fixed64_threeway_status",
    "fixed64_threeway_ok",
    "ci_pack_golden_overlay_compare_selftest_ok",
    "ci_pack_golden_overlay_session_selftest_ok",
)
PASS_REQUIRED_KEYS = PASS_REQUIRED_KEYS + tuple(AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS) + tuple(
    sync_key for _sanity_key, sync_key in AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS
)
RUNTIME5_REQUIRED_KEYS = (
    "seamgrim_5min_checklist",
    "seamgrim_5min_checklist_ok",
    "seamgrim_runtime_5min_rewrite_motion_projectile",
    "seamgrim_runtime_5min_rewrite_elapsed_ms",
    "seamgrim_runtime_5min_rewrite_status",
    "seamgrim_runtime_5min_moyang_view_boundary",
    "seamgrim_runtime_5min_moyang_elapsed_ms",
    "seamgrim_runtime_5min_moyang_status",
    "seamgrim_runtime_5min_pendulum_tetris_showcase",
    "seamgrim_runtime_5min_pendulum_tetris_showcase_elapsed_ms",
    "seamgrim_runtime_5min_pendulum_tetris_showcase_status",
)
MIN_REQUIRED_CI_SANITY_STEPS = 14
VALID_SANITY_PROFILES = set(PROFILE_MATRIX_SELFTEST_PROFILES)
AGE3_COMPLETION_GATE_CRITERIA_ENABLED_PROFILES = {"full", "core_lang", "seamgrim"}
VALID_RUNTIME5_ITEM_STATUS = {"ok", "failed", "not_executed", "missing_report", "items_missing"}
VALID_AGE5_CHILD_SUMMARY_STATUS = {"pass", "fail", "skipped"}
AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA = "ddn.bogae_geoul_visibility_smoke.v1"
SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA = "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1"
SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES = 20
SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES = {"seamgrim"}
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA = "ddn.pack_evidence_tier_runner_check.v1"
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_MAX_DOCS_ISSUES = 10
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_EXPECTED_REPO_ISSUES = 0
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES = {"seamgrim"}
OPTIONAL_FIXED64_LIVE_SANITY_SUMMARY_KEYS = {
    "ci_sanity_fixed64_darwin_real_report_live_report_exists",
    "ci_sanity_fixed64_darwin_real_report_live_report_path",
    "ci_sanity_fixed64_darwin_real_report_live_status",
    "ci_sanity_fixed64_darwin_real_report_live_resolved_status",
    "ci_sanity_fixed64_darwin_real_report_live_resolved_source",
    "ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count",
    "ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip",
}
OPTIONAL_FIXED64_LIVE_SYNC_SUMMARY_KEYS = {
    "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_report_exists",
    "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_report_path",
    "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_status",
    "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_status",
    "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_source",
    "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count",
    "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip",
}
PACK_GOLDEN_GRAPH_EXPORT_SANITY_KEY = "ci_sanity_pack_golden_graph_export_ok"
PACK_GOLDEN_GRAPH_EXPORT_SYNC_KEY = "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok"
PACK_GOLDEN_GRAPH_EXPORT_REQUIRED_PROFILES = {"full", "core_lang"}
PROFILE_MATRIX_AGGREGATE_SUMMARY_EXPECTED = {
    profile_name: expected_profile_matrix_aggregate_summary_contract(profile_name)
    for profile_name in PROFILE_MATRIX_SELFTEST_PROFILES
}
SANITY_RUNTIME_HELPER_SUMMARY_FIELDS = (
    ("ci_sanity_pipeline_emit_flags_ok", {"full", "core_lang"}),
    ("ci_sanity_pipeline_emit_flags_selftest_ok", {"full", "core_lang"}),
    ("ci_sanity_emit_artifacts_sanity_contract_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age2_completion_gate_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age2_completion_gate_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_completion_gate_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_completion_gate_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_bogae_geoul_visibility_smoke_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok", {"full", "core_lang", "seamgrim"}),
    (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes",
        {"full", "core_lang", "seamgrim"},
    ),
    ("ci_sanity_seamgrim_pack_evidence_tier_runner_ok", SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES),
    (
        "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists",
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES,
    ),
    ("ci_sanity_seamgrim_wasm_web_step_check_ok", SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES),
    ("ci_sanity_seamgrim_wasm_web_step_check_report_exists", SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES),
    ("ci_sanity_fixed64_darwin_real_report_live_report_exists", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_pack_golden_graph_export_ok", {"full", "core_lang"}),
    ("ci_sanity_age5_combined_heavy_policy_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_dynamic_source_profile_split_selftest_ok", {"full", "core_lang", "seamgrim"}),
)
SANITY_RUNTIME_HELPER_SUMMARY_FIELDS = SANITY_RUNTIME_HELPER_SUMMARY_FIELDS + tuple(
    (key, AGE3_COMPLETION_GATE_CRITERIA_ENABLED_PROFILES)
    for key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS
)
SYNC_RUNTIME_HELPER_SUMMARY_FIELDS = (
    ("ci_sync_readiness_ci_sanity_pipeline_emit_flags_ok", {"full", "core_lang"}),
    ("ci_sync_readiness_ci_sanity_pipeline_emit_flags_selftest_ok", {"full", "core_lang"}),
    ("ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sync_readiness_ci_sanity_age2_completion_gate_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sync_readiness_ci_sanity_age2_completion_gate_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sync_readiness_ci_sanity_age3_completion_gate_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sync_readiness_ci_sanity_age3_completion_gate_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_ok", {"full", "core_lang", "seamgrim"}),
    (
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok",
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists",
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_ok",
        SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_exists",
        SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES,
    ),
    ("ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_report_exists", {"full", "core_lang", "seamgrim"}),
    ("ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok", {"full", "core_lang"}),
    ("ci_sync_readiness_ci_sanity_age5_combined_heavy_policy_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sync_readiness_ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sync_readiness_ci_sanity_dynamic_source_profile_split_selftest_ok", {"full", "core_lang", "seamgrim"}),
)
SYNC_RUNTIME_HELPER_SUMMARY_FIELDS = SYNC_RUNTIME_HELPER_SUMMARY_FIELDS + tuple(
    (sync_key, AGE3_COMPLETION_GATE_CRITERIA_ENABLED_PROFILES)
    for _sanity_key, sync_key in AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS
)
SANITY_RUNTIME_HELPER_TEXT_FIELDS = (
    (
        "ci_sanity_age2_completion_gate_failure_codes",
        "codes",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_age2_completion_gate_failure_code_count",
        "count",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_age3_completion_gate_failure_codes",
        "codes",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_age3_completion_gate_failure_code_count",
        "count",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path",
        "pack_evidence_path",
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_pack_evidence_tier_runner_schema",
        "pack_evidence_schema",
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count",
        "pack_evidence_docs_issue_count",
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count",
        "pack_evidence_repo_issue_count",
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_wasm_web_step_check_report_path",
        "step_path",
        SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_wasm_web_step_check_schema",
        "step_schema",
        SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_wasm_web_step_check_checked_files",
        "step_checked_files",
        SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_wasm_web_step_check_missing_count",
        "step_missing_count",
        SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES,
    ),
    (
        "ci_sanity_fixed64_darwin_real_report_live_report_path",
        "fixed64_live_path",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_fixed64_darwin_real_report_live_status",
        "fixed64_live_status",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_fixed64_darwin_real_report_live_resolved_status",
        "fixed64_live_resolved_status",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_fixed64_darwin_real_report_live_resolved_source",
        "fixed64_live_resolved_source",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count",
        "fixed64_live_invalid_count",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip",
        "fixed64_live_zip",
        {"full", "core_lang", "seamgrim"},
    ),
)
SYNC_RUNTIME_HELPER_TEXT_FIELDS = (
    (
        "ci_sync_readiness_ci_sanity_age2_completion_gate_failure_codes",
        "ci_sanity_age2_completion_gate_failure_codes",
        "codes",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_age2_completion_gate_failure_code_count",
        "ci_sanity_age2_completion_gate_failure_code_count",
        "count",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_completion_gate_failure_codes",
        "ci_sanity_age3_completion_gate_failure_codes",
        "codes",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_completion_gate_failure_code_count",
        "ci_sanity_age3_completion_gate_failure_code_count",
        "count",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_path",
        "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path",
        "pack_evidence_path",
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_schema",
        "ci_sanity_seamgrim_pack_evidence_tier_runner_schema",
        "pack_evidence_schema",
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count",
        "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count",
        "pack_evidence_docs_issue_count",
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count",
        "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count",
        "pack_evidence_repo_issue_count",
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_path",
        "ci_sanity_seamgrim_wasm_web_step_check_report_path",
        "step_path",
        SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_schema",
        "ci_sanity_seamgrim_wasm_web_step_check_schema",
        "step_schema",
        SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_checked_files",
        "ci_sanity_seamgrim_wasm_web_step_check_checked_files",
        "step_checked_files",
        SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_missing_count",
        "ci_sanity_seamgrim_wasm_web_step_check_missing_count",
        "step_missing_count",
        SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_report_path",
        "ci_sanity_fixed64_darwin_real_report_live_report_path",
        "fixed64_live_path",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_status",
        "ci_sanity_fixed64_darwin_real_report_live_status",
        "fixed64_live_status",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_status",
        "ci_sanity_fixed64_darwin_real_report_live_resolved_status",
        "fixed64_live_resolved_status",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_source",
        "ci_sanity_fixed64_darwin_real_report_live_resolved_source",
        "fixed64_live_resolved_source",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count",
        "ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count",
        "fixed64_live_invalid_count",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip",
        "ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip",
        "fixed64_live_zip",
        {"full", "core_lang", "seamgrim"},
    ),
)
FAILURE_CODE_PAIR_KEYS = (
    (
        "ci_sanity_age2_completion_gate_failure_codes",
        "ci_sanity_age2_completion_gate_failure_code_count",
    ),
    (
        "ci_sanity_age3_completion_gate_failure_codes",
        "ci_sanity_age3_completion_gate_failure_code_count",
    ),
    (
        "ci_sync_readiness_ci_sanity_age2_completion_gate_failure_codes",
        "ci_sync_readiness_ci_sanity_age2_completion_gate_failure_code_count",
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_completion_gate_failure_codes",
        "ci_sync_readiness_ci_sanity_age3_completion_gate_failure_code_count",
    ),
)
FAILURE_CODE_PATTERN = re.compile(r"[EW]_[A-Z0-9_]+")
SANITY_RUNTIME_HELPER_CONTRACT_FIELDS = AGE5_COMBINED_HEAVY_SANITY_CONTRACT_SUMMARY_FIELDS
SYNC_RUNTIME_HELPER_CONTRACT_FIELDS = AGE5_COMBINED_HEAVY_SYNC_CONTRACT_SUMMARY_FIELDS
AGE5_POLICY_SUMMARY_KEYS = (
    "age5_policy_combined_digest_selftest_default_field_text",
    "age5_policy_combined_digest_selftest_default_field",
    "age5_combined_heavy_policy_report_path",
    "age5_combined_heavy_policy_report_exists",
    "age5_combined_heavy_policy_text_path",
    "age5_combined_heavy_policy_text_exists",
    "age5_combined_heavy_policy_summary_path",
    "age5_combined_heavy_policy_summary_exists",
    "age5_combined_heavy_policy_origin_trace_text",
    "age5_combined_heavy_policy_origin_trace",
    "age5_policy_summary_origin_trace_contract_issue",
    "age5_policy_summary_origin_trace_contract_source_issue",
    "age5_policy_summary_origin_trace_contract_compact_reason",
    "age5_policy_summary_origin_trace_contract_compact_failure_reason",
)
AGE4_PROOF_SUMMARY_KEYS = (
    "age4_proof_ok",
    "age4_proof_failed_criteria",
    "age4_proof_failed_preview",
    "age4_proof_summary_hash",
)
AGE5_W107_PROGRESS_SUMMARY_KEYS = (
    AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_ACTIVE_CASES_KEY,
    AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_INACTIVE_CASES_KEY,
    AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_INDEX_CODES_KEY,
    AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
)


def format_age4_proof_failed_preview(failed: object) -> str:
    if isinstance(failed, list):
        items = [str(item).strip() for item in failed if str(item).strip()]
        if not items:
            return "-"
        preview = items[:2]
        if len(items) > 2:
            preview.append(f"+{len(items) - 2}more")
        return ",".join(preview)
    try:
        count = int(failed)
    except Exception:
        return "-"
    if count <= 0:
        return "-"
    return f"count:{count}"


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def parse_summary(path: Path) -> tuple[str | None, dict[str, str], list[str]]:
    lines = [line.strip() for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
    status: str | None = None
    kv: dict[str, str] = {}
    for line in lines:
        if not line.startswith("[ci-gate-summary] "):
            continue
        body = line[len("[ci-gate-summary] ") :]
        if body in {"PASS", "FAIL"}:
            status = body.lower()
            continue
        if "=" not in body:
            continue
        key, value = body.split("=", 1)
        key = key.strip()
        value = value.strip()
        if key:
            kv[key] = value
    return status, kv, lines


def fail(msg: str, code: str = "E_CHECK") -> int:
    print(f"[ci-gate-summary-report-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def validate_runtime5_elapsed(key: str, value: str) -> str | None:
    text = str(value).strip()
    if not text:
        return f"{key} is empty"
    if text == "-":
        return None
    try:
        elapsed_num = int(text)
    except Exception:
        return f"{key} is not an integer: {text}"
    if elapsed_num < 0:
        return f"{key} must be >= 0: {elapsed_num}"
    return None


def validate_failure_code_field_value(key: str, value: str, value_kind: str) -> str | None:
    text = str(value).strip()
    if not text:
        return f"{key} is empty"
    if value_kind == "step_path":
        if text == "-":
            return f"{key} is '-'"
        path = Path(text)
        if not path.exists():
            return f"{key} path does not exist: {path}"
        return None
    if value_kind == "pack_evidence_path":
        if text == "-":
            return f"{key} is '-'"
        path = Path(text)
        if not path.exists():
            return f"{key} path does not exist: {path}"
        return None
    if value_kind == "step_schema":
        if text != SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA:
            return f"{key} schema mismatch: {text}"
        return None
    if value_kind == "pack_evidence_schema":
        if text != SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA:
            return f"{key} schema mismatch: {text}"
        return None
    if value_kind == "step_checked_files":
        try:
            checked_files_num = int(text)
        except Exception:
            return f"{key} is not an integer: {text}"
        if checked_files_num < SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES:
            return (
                f"{key} must be >= {SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES}: "
                f"{checked_files_num}"
            )
        return None
    if value_kind == "step_missing_count":
        try:
            missing_count_num = int(text)
        except Exception:
            return f"{key} is not an integer: {text}"
        if missing_count_num != 0:
            return f"{key} must be 0: {missing_count_num}"
        return None
    if value_kind == "pack_evidence_docs_issue_count":
        try:
            docs_issue_num = int(text)
        except Exception:
            return f"{key} is not an integer: {text}"
        if docs_issue_num < 0 or docs_issue_num > SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_MAX_DOCS_ISSUES:
            return (
                f"{key} must be within 0..{SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_MAX_DOCS_ISSUES}: "
                f"{docs_issue_num}"
            )
        return None
    if value_kind == "pack_evidence_repo_issue_count":
        try:
            repo_issue_num = int(text)
        except Exception:
            return f"{key} is not an integer: {text}"
        if repo_issue_num != SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_EXPECTED_REPO_ISSUES:
            return (
                f"{key} must be {SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_EXPECTED_REPO_ISSUES}: "
                f"{repo_issue_num}"
            )
        return None
    if value_kind == "count":
        try:
            count_num = int(text)
        except Exception:
            return f"{key} is not an integer: {text}"
        if count_num < 0:
            return f"{key} must be >= 0: {count_num}"
        return None
    if value_kind == "codes":
        if text == "-":
            return None
        code_items = [token.strip() for token in text.split(",") if token.strip()]
        if not code_items:
            return f"{key} has no codes"
        if len(set(code_items)) != len(code_items):
            return f"{key} has duplicated codes: {text}"
        for token in code_items:
            if not FAILURE_CODE_PATTERN.fullmatch(token):
                return f"{key} invalid code token: {token}"
        return None
    if value_kind == "fixed64_live_path":
        if text == "-":
            return f"{key} is '-'"
        path = Path(text)
        if not path.exists():
            return f"{key} path does not exist: {path}"
        return None
    if value_kind == "fixed64_live_status":
        if text in {"-", "na", "pending"}:
            return f"{key} invalid status text: {text}"
        return None
    if value_kind == "fixed64_live_resolved_status":
        if text in {"na", "pending"}:
            return f"{key} invalid resolved_status text: {text}"
        return None
    if value_kind == "fixed64_live_resolved_source":
        if text in {"na", "pending"}:
            return f"{key} invalid resolved_source text: {text}"
        return None
    if value_kind == "fixed64_live_invalid_count":
        try:
            invalid_count_num = int(text)
        except Exception:
            return f"{key} is not an integer: {text}"
        if invalid_count_num < 0:
            return f"{key} must be >= 0: {invalid_count_num}"
        return None
    if value_kind == "fixed64_live_zip":
        if text not in {"0", "1"}:
            return f"{key} must be 0|1: {text}"
        return None
    return f"{key} unsupported failure-code kind: {value_kind}"


def expected_runtime_helper_summary_value(key: str, profile: str, valid_profiles: set[str]) -> str:
    if key in {PACK_GOLDEN_GRAPH_EXPORT_SANITY_KEY, PACK_GOLDEN_GRAPH_EXPORT_SYNC_KEY}:
        return "1" if profile in PACK_GOLDEN_GRAPH_EXPORT_REQUIRED_PROFILES else "0"
    return "1" if profile in valid_profiles else "na"


def valid_runtime_helper_summary_values(expected_value: str) -> set[str]:
    if expected_value == "0":
        return {"0", "1"}
    return {"1", "na"}


def load_profile_matrix_selftest_snapshot(report_path: Path) -> dict[str, str] | None:
    doc = load_json(report_path)
    return build_profile_matrix_snapshot_from_doc(doc)


def load_age5_policy_summary_snapshot(aggregate_report_path: Path) -> dict[str, str] | None:
    aggregate_doc = load_json(aggregate_report_path)
    if not isinstance(aggregate_doc, dict):
        return None
    age5_doc = aggregate_doc.get("age5")
    if not isinstance(age5_doc, dict):
        return None
    default_field = age5_doc.get("age5_policy_combined_digest_selftest_default_field")
    default_field_text = '{"age5_close_digest_selftest_ok":"0"}'
    if isinstance(default_field, dict):
        default_field_text = json.dumps(
            {str(key): str(value) for key, value in default_field.items()},
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        )
    origin_trace = age5_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY)
    if isinstance(origin_trace, dict):
        origin_trace_dict = build_age5_combined_heavy_policy_origin_trace(
            report_path=origin_trace.get("report_path", "-"),
            report_exists=origin_trace.get("report_exists", "0") == "1",
            text_path=origin_trace.get("text_path", "-"),
            text_exists=origin_trace.get("text_exists", "0") == "1",
            summary_path=origin_trace.get("summary_path", "-"),
            summary_exists=origin_trace.get("summary_exists", "0") == "1",
        )
    else:
        origin_trace_dict = build_age5_combined_heavy_policy_origin_trace(
            report_path=age5_doc.get("age5_combined_heavy_policy_report_path", "-"),
            report_exists=age5_doc.get("age5_combined_heavy_policy_report_exists", False),
            text_path=age5_doc.get("age5_combined_heavy_policy_text_path", "-"),
            text_exists=age5_doc.get("age5_combined_heavy_policy_text_exists", False),
            summary_path=age5_doc.get("age5_combined_heavy_policy_summary_path", "-"),
            summary_exists=age5_doc.get("age5_combined_heavy_policy_summary_exists", False),
        )
    origin_trace_text = str(
        age5_doc.get(
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY,
            build_age5_combined_heavy_policy_origin_trace_text(origin_trace_dict),
        )
    ).strip() or build_age5_combined_heavy_policy_origin_trace_text(origin_trace_dict)
    return {
        "age5_policy_combined_digest_selftest_default_field_text": (
            str(age5_doc.get("age5_policy_combined_digest_selftest_default_field_text", "age5_close_digest_selftest_ok=0")).strip()
            or "age5_close_digest_selftest_ok=0"
        ),
        "age5_policy_combined_digest_selftest_default_field": default_field_text,
        "age5_combined_heavy_policy_report_path": (
            str(age5_doc.get("age5_combined_heavy_policy_report_path", "-")).strip() or "-"
        ),
        "age5_combined_heavy_policy_report_exists": (
            "1" if bool(age5_doc.get("age5_combined_heavy_policy_report_exists", False)) else "0"
        ),
        "age5_combined_heavy_policy_text_path": (
            str(age5_doc.get("age5_combined_heavy_policy_text_path", "-")).strip() or "-"
        ),
        "age5_combined_heavy_policy_text_exists": (
            "1" if bool(age5_doc.get("age5_combined_heavy_policy_text_exists", False)) else "0"
        ),
        "age5_combined_heavy_policy_summary_path": (
            str(age5_doc.get("age5_combined_heavy_policy_summary_path", "-")).strip() or "-"
        ),
        "age5_combined_heavy_policy_summary_exists": (
            "1" if bool(age5_doc.get("age5_combined_heavy_policy_summary_exists", False)) else "0"
        ),
        "age5_combined_heavy_policy_origin_trace_text": origin_trace_text,
        "age5_combined_heavy_policy_origin_trace": json.dumps(
            origin_trace_dict,
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        ),
        "age5_policy_summary_origin_trace_contract_issue": (
            str(age5_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY, "-")).strip()
            or "-"
        ),
        "age5_policy_summary_origin_trace_contract_source_issue": (
            str(age5_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY, "-")).strip()
            or "-"
        ),
        "age5_policy_summary_origin_trace_contract_compact_reason": (
            str(age5_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY, "-")).strip()
            or "-"
        ),
        "age5_policy_summary_origin_trace_contract_compact_failure_reason": (
            str(age5_doc.get("age5_policy_summary_origin_trace_contract_compact_failure_reason", "-")).strip()
            or "-"
        ),
    }


def load_age4_proof_summary_snapshot(aggregate_report_path: Path) -> dict[str, str] | None:
    aggregate_doc = load_json(aggregate_report_path)
    if not isinstance(aggregate_doc, dict):
        return None
    age4_doc = aggregate_doc.get("age4")
    if not isinstance(age4_doc, dict):
        return None
    failed = age4_doc.get("proof_artifact_failed_criteria")
    if isinstance(failed, list):
        failed_text = str(len(failed))
    else:
        try:
            failed_text = str(int(failed))
        except Exception:
            return None
    summary_hash = str(age4_doc.get("proof_artifact_summary_hash", "")).strip()
    if not summary_hash:
        return None
    return {
        "age4_proof_ok": "1" if bool(age4_doc.get("proof_artifact_ok", False)) else "0",
        "age4_proof_failed_criteria": failed_text,
        "age4_proof_failed_preview": (
            str(age4_doc.get("proof_artifact_failed_preview", "")).strip()
            or format_age4_proof_failed_preview(failed)
        ),
        "age4_proof_summary_hash": summary_hash,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate ci_gate_summary.txt core key/value lines")
    parser.add_argument("--summary", required=True, help="path to ci_gate_summary.txt")
    parser.add_argument("--index", required=True, help="path to ci_gate_report_index.detjson")
    parser.add_argument("--require-pass", action="store_true", help="require summary PASS block")
    args = parser.parse_args()

    summary_path = Path(args.summary)
    index_path = Path(args.index)
    if not summary_path.exists():
        return fail(f"missing summary file: {summary_path}", code=CODES["SUMMARY_MISSING"])
    status, kv, lines = parse_summary(summary_path)
    if not lines:
        return fail("summary file is empty", code=CODES["SUMMARY_EMPTY"])
    if status not in {"pass", "fail"}:
        return fail("missing PASS/FAIL header line", code=CODES["SUMMARY_STATUS_MISSING"])
    if args.require_pass and status != "pass":
        return fail("require-pass set but summary status is not PASS", code=CODES["REQUIRE_PASS"])

    index_doc = load_json(index_path)
    if index_doc is None:
        return fail(f"invalid index json: {index_path}", code=CODES["INDEX_INVALID"])

    if status == "pass":
        for key in PASS_REQUIRED_KEYS:
            value = kv.get(key, "").strip()
            if not value:
                return fail(f"missing summary key: {key}", code=CODES["PASS_KEY_MISSING"])

        if kv.get("report_index") != str(index_path):
            return fail(
                f"report_index mismatch summary={kv.get('report_index')} index={index_path}",
                code=CODES["REPORT_INDEX_MISMATCH"],
            )

        reports = index_doc.get("reports")
        if not isinstance(reports, dict):
            return fail("index.reports is missing", code=CODES["INDEX_REPORTS_MISSING"])
        compare_map = {
            "summary_line": str(reports.get("summary_line", "")).strip(),
            "ci_gate_result": str(reports.get("ci_gate_result_json", "")).strip(),
            "ci_gate_badge": str(reports.get("ci_gate_badge_json", "")).strip(),
            "ci_fail_triage_hint": str(reports.get("ci_fail_triage_json", "")).strip(),
            "ci_profile_matrix_gate_selftest_report": str(reports.get("ci_profile_matrix_gate_selftest", "")).strip(),
            "age2_status": str(reports.get("age2_close", "")).strip(),
            "age3_status": str(reports.get("age3_close_status_json", "")).strip(),
            "age4_status": str(reports.get("age4_close", "")).strip(),
            "age5_status": str(reports.get("age5_close", "")).strip(),
            "seamgrim_phase3_cleanup": str(reports.get("seamgrim_phase3_cleanup", "")).strip(),
            "seamgrim_wasm_cli_diag_parity_report": str(reports.get("seamgrim_wasm_cli_diag_parity", "")).strip(),
            "ci_sanity_gate_report": str(reports.get("ci_sanity_gate", "")).strip(),
            "ci_sync_readiness_report": str(reports.get("ci_sync_readiness", "")).strip(),
            "fixed64_threeway_report": str(reports.get("fixed64_threeway_gate", "")).strip(),
        }
        for key, expected in compare_map.items():
            if not expected:
                return fail(f"index missing expected path for {key}", code=CODES["INDEX_PATH_MISSING"])
            if kv.get(key) != expected:
                return fail(
                    f"{key} mismatch summary={kv.get(key)} index={expected}",
                    code=CODES["SUMMARY_INDEX_PATH_MISMATCH"],
                )
            if not Path(expected).exists():
                return fail(
                    f"PASS summary requires existing path for {key}: {expected}",
                    code=CODES["PASS_KEY_MISSING"],
                )
        age5_report_path = Path(compare_map["age5_status"])
        age5_report_doc = load_json(age5_report_path)
        if age5_report_doc is None:
            return fail(
                f"invalid age5 report json: {age5_report_path}",
                code=CODES["PASS_KEY_MISSING"],
            )
        aggregate_report_path_text = str(reports.get("aggregate", "")).strip()
        if not aggregate_report_path_text:
            return fail(
                "index missing expected path for aggregate",
                code=CODES["INDEX_PATH_MISSING"],
            )
        aggregate_report_path = Path(aggregate_report_path_text)
        if not aggregate_report_path.exists():
            return fail(
                f"PASS summary requires existing aggregate path: {aggregate_report_path}",
                code=CODES["PASS_KEY_MISSING"],
            )
        age5_policy_summary_snapshot = load_age5_policy_summary_snapshot(aggregate_report_path)
        if age5_policy_summary_snapshot is None:
            return fail(
                f"invalid aggregate age5 policy summary snapshot: {aggregate_report_path}",
                code=CODES["PASS_KEY_MISSING"],
            )
        age4_proof_summary_snapshot = load_age4_proof_summary_snapshot(aggregate_report_path)
        if age4_proof_summary_snapshot is None:
            return fail(
                f"invalid aggregate age4 proof summary snapshot: {aggregate_report_path}",
                code=CODES["PASS_KEY_MISSING"],
            )
        for key in AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS:
            value = kv.get(key, "").strip()
            if value not in VALID_AGE5_CHILD_SUMMARY_STATUS:
                return fail(
                    f"{key} invalid: {value}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            expected_value = str(age5_report_doc.get(key, "")).strip()
            if expected_value not in VALID_AGE5_CHILD_SUMMARY_STATUS:
                return fail(
                    f"age5 report missing/invalid {key}: {expected_value}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if value != expected_value:
                return fail(
                    f"{key} mismatch summary={value} age5={expected_value}",
                    code=CODES["SUMMARY_INDEX_PATH_MISMATCH"],
                )
        for key in AGE5_W107_PROGRESS_SUMMARY_KEYS:
            value = kv.get(key, "").strip()
            if not value:
                return fail(f"missing summary key: {key}", code=CODES["PASS_KEY_MISSING"])
            expected_value = str(age5_report_doc.get(key, "")).strip()
            if not expected_value:
                return fail(f"age5 report missing {key}", code=CODES["PASS_KEY_MISSING"])
            if value != expected_value:
                return fail(
                    f"{key} mismatch summary={value} age5={expected_value}",
                    code=CODES["SUMMARY_INDEX_PATH_MISMATCH"],
                )
        for key in AGE5_POLICY_SUMMARY_KEYS:
            value = kv.get(key, "").strip()
            expected_value = age5_policy_summary_snapshot.get(key, "").strip()
            if value != expected_value:
                return fail(
                    f"{key} mismatch summary={value} aggregate={expected_value}",
                    code=CODES["SUMMARY_INDEX_PATH_MISMATCH"],
                )
        for key in AGE4_PROOF_SUMMARY_KEYS:
            value = kv.get(key, "").strip()
            expected_value = age4_proof_summary_snapshot.get(key, "").strip()
            if value != expected_value:
                return fail(
                    f"{key} mismatch summary={value} aggregate={expected_value}",
                    code=CODES["SUMMARY_INDEX_PATH_MISMATCH"],
                )
        runtime5_checklist_path = str(reports.get("seamgrim_5min_checklist", "")).strip()
        if runtime5_checklist_path and Path(runtime5_checklist_path).exists():
            for key in RUNTIME5_REQUIRED_KEYS:
                value = kv.get(key, "").strip()
                if not value:
                    return fail(f"missing summary key: {key}", code=CODES["PASS_KEY_MISSING"])
            if kv.get("seamgrim_5min_checklist") != runtime5_checklist_path:
                return fail(
                    "seamgrim_5min_checklist mismatch "
                    f"summary={kv.get('seamgrim_5min_checklist')} index={runtime5_checklist_path}",
                    code=CODES["SUMMARY_INDEX_PATH_MISMATCH"],
                )
            checklist_ok = kv.get("seamgrim_5min_checklist_ok", "").strip()
            if checklist_ok not in {"0", "1"}:
                return fail(
                    f"seamgrim_5min_checklist_ok invalid: {checklist_ok}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if checklist_ok != "1":
                return fail("PASS summary requires seamgrim_5min_checklist_ok=1", code=CODES["PASS_KEY_MISSING"])
            for item_key, item_status_key, item_elapsed_key in (
                (
                    "seamgrim_runtime_5min_rewrite_motion_projectile",
                    "seamgrim_runtime_5min_rewrite_status",
                    "seamgrim_runtime_5min_rewrite_elapsed_ms",
                ),
                (
                    "seamgrim_runtime_5min_moyang_view_boundary",
                    "seamgrim_runtime_5min_moyang_status",
                    "seamgrim_runtime_5min_moyang_elapsed_ms",
                ),
                (
                    "seamgrim_runtime_5min_pendulum_tetris_showcase",
                    "seamgrim_runtime_5min_pendulum_tetris_showcase_status",
                    "seamgrim_runtime_5min_pendulum_tetris_showcase_elapsed_ms",
                ),
            ):
                item_ok = kv.get(item_key, "").strip()
                if item_ok not in {"0", "1", "na"}:
                    return fail(
                        f"{item_key} invalid: {item_ok}",
                        code=CODES["PASS_KEY_MISSING"],
                    )
                if item_ok != "1":
                    required_ok_message = {
                        "seamgrim_runtime_5min_rewrite_motion_projectile": (
                            "PASS summary requires seamgrim_runtime_5min_rewrite_motion_projectile=1"
                        ),
                        "seamgrim_runtime_5min_moyang_view_boundary": (
                            "PASS summary requires seamgrim_runtime_5min_moyang_view_boundary=1"
                        ),
                    }.get(item_key, f"PASS summary requires {item_key}=1")
                    return fail(
                        required_ok_message,
                        code=CODES["PASS_KEY_MISSING"],
                    )
                item_status = kv.get(item_status_key, "").strip()
                if item_status not in VALID_RUNTIME5_ITEM_STATUS:
                    return fail(
                        f"{item_status_key} invalid: {item_status}",
                        code=CODES["PASS_KEY_MISSING"],
                    )
                if item_status != "ok":
                    return fail(
                        f"PASS summary requires {item_status_key}=ok",
                        code=CODES["PASS_KEY_MISSING"],
                    )
                item_elapsed_error = validate_runtime5_elapsed(item_elapsed_key, kv.get(item_elapsed_key, ""))
                if item_elapsed_error:
                    return fail(item_elapsed_error, code=CODES["PASS_KEY_MISSING"])
        fixed64_ok_text = kv.get("fixed64_threeway_ok", "").strip()
        if fixed64_ok_text not in {"0", "1"}:
            return fail(f"fixed64_threeway_ok invalid: {fixed64_ok_text}", code=CODES["PASS_KEY_MISSING"])
        if fixed64_ok_text != "1":
            return fail("PASS summary requires fixed64_threeway_ok=1", code=CODES["PASS_KEY_MISSING"])
        fixed64_status = kv.get("fixed64_threeway_status", "").strip()
        if not fixed64_status:
            return fail("fixed64_threeway_status is empty", code=CODES["PASS_KEY_MISSING"])

        overlay_compare_selftest_ok = kv.get("ci_pack_golden_overlay_compare_selftest_ok", "").strip()
        if overlay_compare_selftest_ok not in {"0", "1"}:
            return fail(
                f"ci_pack_golden_overlay_compare_selftest_ok invalid: {overlay_compare_selftest_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if overlay_compare_selftest_ok != "1":
            return fail(
                "PASS summary requires ci_pack_golden_overlay_compare_selftest_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )

        age5_close_digest_selftest_ok = kv.get("age5_close_digest_selftest_ok", "").strip()
        if age5_close_digest_selftest_ok not in {"0", "1"}:
            return fail(
                f"age5_close_digest_selftest_ok invalid: {age5_close_digest_selftest_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if age5_close_digest_selftest_ok != "1":
            return fail(
                "PASS summary requires age5_close_digest_selftest_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )

        profile_matrix_selftest_ok = kv.get("ci_profile_matrix_gate_selftest_ok", "").strip()
        if profile_matrix_selftest_ok not in {"0", "1"}:
            return fail(
                f"ci_profile_matrix_gate_selftest_ok invalid: {profile_matrix_selftest_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if profile_matrix_selftest_ok != "1":
            return fail(
                "PASS summary requires ci_profile_matrix_gate_selftest_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        profile_matrix_selftest_status = kv.get("ci_profile_matrix_gate_selftest_status", "").strip()
        if profile_matrix_selftest_status != "pass":
            return fail(
                "PASS summary requires ci_profile_matrix_gate_selftest_status=pass",
                code=CODES["PASS_KEY_MISSING"],
            )
        profile_matrix_aggregate_summary_ok = kv.get(
            "ci_profile_matrix_gate_selftest_aggregate_summary_sanity_ok",
            "",
        ).strip()
        if profile_matrix_aggregate_summary_ok not in {"0", "1"}:
            return fail(
                "ci_profile_matrix_gate_selftest_aggregate_summary_sanity_ok invalid: "
                f"{profile_matrix_aggregate_summary_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if profile_matrix_aggregate_summary_ok != "1":
            return fail(
                "PASS summary requires ci_profile_matrix_gate_selftest_aggregate_summary_sanity_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        profile_matrix_report_path = Path(kv.get("ci_profile_matrix_gate_selftest_report", "").strip())
        profile_matrix_snap = load_profile_matrix_selftest_snapshot(profile_matrix_report_path)
        if profile_matrix_snap is None:
            return fail(
                f"invalid ci_profile_matrix_gate_selftest report: {profile_matrix_report_path}",
                code=CODES["PASS_KEY_MISSING"],
            )
        for key, expected in (
            ("ci_profile_matrix_gate_selftest_ok", profile_matrix_snap["ok"]),
            ("ci_profile_matrix_gate_selftest_status", profile_matrix_snap["status"]),
            ("ci_profile_matrix_gate_selftest_total_elapsed_ms", profile_matrix_snap["total_elapsed_ms"]),
            (
                "ci_profile_matrix_gate_selftest_selected_real_profiles",
                profile_matrix_snap["selected_real_profiles"],
            ),
            (
                "ci_profile_matrix_gate_selftest_skipped_real_profiles",
                profile_matrix_snap["skipped_real_profiles"],
            ),
            (
                "ci_profile_matrix_gate_selftest_step_timeout_defaults",
                profile_matrix_snap["step_timeout_defaults_text"],
            ),
            (
                "ci_profile_matrix_gate_selftest_core_lang_elapsed_ms",
                profile_matrix_snap["core_lang_elapsed_ms"],
            ),
            (
                "ci_profile_matrix_gate_selftest_full_elapsed_ms",
                profile_matrix_snap["full_elapsed_ms"],
            ),
            (
                "ci_profile_matrix_gate_selftest_seamgrim_elapsed_ms",
                profile_matrix_snap["seamgrim_elapsed_ms"],
            ),
            (
                "ci_profile_matrix_gate_selftest_aggregate_summary_sanity_ok",
                profile_matrix_snap["aggregate_summary_sanity_ok"],
            ),
            (
                "ci_profile_matrix_gate_selftest_aggregate_summary_checked_profiles",
                profile_matrix_snap["aggregate_summary_sanity_checked_profiles"],
            ),
            (
                "ci_profile_matrix_gate_selftest_aggregate_summary_failed_profiles",
                profile_matrix_snap["aggregate_summary_sanity_failed_profiles"],
            ),
            (
                "ci_profile_matrix_gate_selftest_aggregate_summary_skipped_profiles",
                profile_matrix_snap["aggregate_summary_sanity_skipped_profiles"],
            ),
            (
                "ci_profile_matrix_gate_selftest_core_lang_aggregate_summary_status",
                profile_matrix_snap["core_lang_aggregate_summary_status"],
            ),
            (
                "ci_profile_matrix_gate_selftest_core_lang_aggregate_summary_ok",
                profile_matrix_snap["core_lang_aggregate_summary_ok"],
            ),
            (
                "ci_profile_matrix_gate_selftest_core_lang_aggregate_summary_values",
                profile_matrix_snap["core_lang_aggregate_summary_values"],
            ),
            (
                "ci_profile_matrix_gate_selftest_full_aggregate_summary_status",
                profile_matrix_snap["full_aggregate_summary_status"],
            ),
            (
                "ci_profile_matrix_gate_selftest_full_aggregate_summary_ok",
                profile_matrix_snap["full_aggregate_summary_ok"],
            ),
            (
                "ci_profile_matrix_gate_selftest_full_aggregate_summary_values",
                profile_matrix_snap["full_aggregate_summary_values"],
            ),
            (
                "ci_profile_matrix_gate_selftest_seamgrim_aggregate_summary_status",
                profile_matrix_snap["seamgrim_aggregate_summary_status"],
            ),
            (
                "ci_profile_matrix_gate_selftest_seamgrim_aggregate_summary_ok",
                profile_matrix_snap["seamgrim_aggregate_summary_ok"],
            ),
            (
                "ci_profile_matrix_gate_selftest_seamgrim_aggregate_summary_values",
                profile_matrix_snap["seamgrim_aggregate_summary_values"],
            ),
        ):
            actual = kv.get(key, "").strip()
            if actual != expected:
                return fail(
                    f"ci_profile_matrix selftest mismatch key={key} summary={actual} report={expected}",
                    code=CODES["PASS_KEY_MISSING"],
                )
        checked_profiles = kv.get("ci_profile_matrix_gate_selftest_aggregate_summary_checked_profiles", "").strip()
        failed_profiles = kv.get("ci_profile_matrix_gate_selftest_aggregate_summary_failed_profiles", "").strip()
        skipped_profiles = kv.get("ci_profile_matrix_gate_selftest_aggregate_summary_skipped_profiles", "").strip()
        expected_checked_profiles = str(profile_matrix_snap["aggregate_summary_sanity_checked_profiles"]).strip()
        expected_failed_profiles = str(profile_matrix_snap["aggregate_summary_sanity_failed_profiles"]).strip()
        expected_skipped_profiles = str(profile_matrix_snap["aggregate_summary_sanity_skipped_profiles"]).strip()
        if checked_profiles != expected_checked_profiles:
            return fail(
                "PASS summary requires ci_profile_matrix_gate_selftest_aggregate_summary_checked_profiles="
                f"{expected_checked_profiles}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if failed_profiles != expected_failed_profiles:
            return fail(
                "PASS summary requires ci_profile_matrix_gate_selftest_aggregate_summary_failed_profiles="
                f"{expected_failed_profiles}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if skipped_profiles != expected_skipped_profiles:
            return fail(
                "PASS summary requires ci_profile_matrix_gate_selftest_aggregate_summary_skipped_profiles="
                f"{expected_skipped_profiles}",
                code=CODES["PASS_KEY_MISSING"],
            )
        # Aggregate-summary 계약은 풀 실행에서만 고정값(pass/values)을 강제한다.
        is_full_aggregate_contract = (
            checked_profiles == "core_lang,full,seamgrim"
            and failed_profiles == "-"
            and skipped_profiles == "-"
        )
        if is_full_aggregate_contract:
            for profile_name, expected_contract in PROFILE_MATRIX_AGGREGATE_SUMMARY_EXPECTED.items():
                status_key = f"ci_profile_matrix_gate_selftest_{profile_name}_aggregate_summary_status"
                ok_key = f"ci_profile_matrix_gate_selftest_{profile_name}_aggregate_summary_ok"
                values_key = f"ci_profile_matrix_gate_selftest_{profile_name}_aggregate_summary_values"
                expected_status = str(expected_contract["status"])
                expected_ok = "1" if bool(expected_contract["ok"]) else "0"
                expected_values = format_profile_matrix_summary_values(expected_contract["values"])
                if kv.get(status_key, "").strip() != expected_status:
                    return fail(
                        f"PASS summary requires {status_key}={expected_status}",
                        code=CODES["PASS_KEY_MISSING"],
                    )
                if kv.get(ok_key, "").strip() != expected_ok:
                    return fail(
                        f"PASS summary requires {ok_key}={expected_ok}",
                        code=CODES["PASS_KEY_MISSING"],
                    )
                if kv.get(values_key, "").strip() != expected_values:
                    return fail(
                        f"PASS summary requires {values_key}={expected_values}",
                        code=CODES["PASS_KEY_MISSING"],
                    )
        for key in (
            "ci_profile_matrix_gate_selftest_total_elapsed_ms",
            "ci_profile_matrix_gate_selftest_core_lang_elapsed_ms",
            "ci_profile_matrix_gate_selftest_full_elapsed_ms",
            "ci_profile_matrix_gate_selftest_seamgrim_elapsed_ms",
        ):
            elapsed_error = validate_runtime5_elapsed(key, kv.get(key, ""))
            if elapsed_error:
                return fail(elapsed_error, code=CODES["PASS_KEY_MISSING"])

        overlay_session_selftest_ok = kv.get("ci_pack_golden_overlay_session_selftest_ok", "").strip()
        if overlay_session_selftest_ok not in {"0", "1"}:
            return fail(
                f"ci_pack_golden_overlay_session_selftest_ok invalid: {overlay_session_selftest_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if overlay_session_selftest_ok != "1":
            return fail(
                "PASS summary requires ci_pack_golden_overlay_session_selftest_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        seamgrim_wasm_cli_diag_parity_ok = kv.get("seamgrim_wasm_cli_diag_parity_ok", "").strip()
        if seamgrim_wasm_cli_diag_parity_ok not in {"0", "1"}:
            return fail(
                "seamgrim_wasm_cli_diag_parity_ok invalid: "
                f"{seamgrim_wasm_cli_diag_parity_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if seamgrim_wasm_cli_diag_parity_ok != "1":
            return fail(
                "PASS summary requires seamgrim_wasm_cli_diag_parity_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )

        ci_sanity_status = kv.get("ci_sanity_gate_status", "").strip()
        if ci_sanity_status != "pass":
            return fail(
                f"PASS summary requires ci_sanity_gate_status=pass, got={ci_sanity_status}",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_ok = kv.get("ci_sanity_gate_ok", "").strip()
        if ci_sanity_ok != "1":
            return fail(
                f"PASS summary requires ci_sanity_gate_ok=1, got={ci_sanity_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_code = kv.get("ci_sanity_gate_code", "").strip()
        if ci_sanity_code != "OK":
            return fail(
                f"PASS summary requires ci_sanity_gate_code=OK, got={ci_sanity_code}",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_profile = kv.get("ci_sanity_gate_profile", "").strip() or "full"
        if ci_sanity_profile not in VALID_SANITY_PROFILES:
            return fail(
                f"invalid ci_sanity_gate_profile={ci_sanity_profile}",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_failed_steps = kv.get("ci_sanity_gate_failed_steps", "").strip()
        if ci_sanity_failed_steps != "0":
            return fail(
                "PASS summary requires ci_sanity_gate_failed_steps=0",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_interface_boundary_ok = kv.get("ci_sanity_seamgrim_interface_boundary_ok", "").strip()
        if ci_sanity_interface_boundary_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_seamgrim_interface_boundary_ok invalid: "
                f"{ci_sanity_interface_boundary_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile in {"full", "seamgrim"} and ci_sanity_interface_boundary_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_seamgrim_interface_boundary_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_wired_ok = kv.get("ci_sanity_overlay_session_wired_consistency_ok", "").strip()
        if ci_sanity_wired_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_overlay_session_wired_consistency_ok invalid: "
                f"{ci_sanity_wired_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile in {"full", "seamgrim"} and ci_sanity_wired_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_overlay_session_wired_consistency_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_session_parity_ok = kv.get("ci_sanity_overlay_session_diag_parity_ok", "").strip()
        if ci_sanity_session_parity_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_overlay_session_diag_parity_ok invalid: "
                f"{ci_sanity_session_parity_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile in {"full", "seamgrim"} and ci_sanity_session_parity_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_overlay_session_diag_parity_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_compare_parity_ok = kv.get("ci_sanity_overlay_compare_diag_parity_ok", "").strip()
        if ci_sanity_compare_parity_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_overlay_compare_diag_parity_ok invalid: "
                f"{ci_sanity_compare_parity_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile in {"full", "seamgrim"} and ci_sanity_compare_parity_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_overlay_compare_diag_parity_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_lang_consistency_ok = kv.get("ci_sanity_pack_golden_lang_consistency_ok", "").strip()
        if ci_sanity_lang_consistency_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_pack_golden_lang_consistency_ok invalid: "
                f"{ci_sanity_lang_consistency_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile in {"full", "core_lang"} and ci_sanity_lang_consistency_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_pack_golden_lang_consistency_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_pack_golden_metadata_ok = kv.get("ci_sanity_pack_golden_metadata_ok", "").strip()
        if ci_sanity_pack_golden_metadata_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_pack_golden_metadata_ok invalid: "
                f"{ci_sanity_pack_golden_metadata_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile in {"full", "core_lang"} and ci_sanity_pack_golden_metadata_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_pack_golden_metadata_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_pack_golden_graph_export_ok = kv.get("ci_sanity_pack_golden_graph_export_ok", "").strip()
        if ci_sanity_pack_golden_graph_export_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_pack_golden_graph_export_ok invalid: "
                f"{ci_sanity_pack_golden_graph_export_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile in {"full", "core_lang"} and ci_sanity_pack_golden_graph_export_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_pack_golden_graph_export_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_canon_ast_dpack_ok = kv.get("ci_sanity_canon_ast_dpack_ok", "").strip()
        if ci_sanity_canon_ast_dpack_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_canon_ast_dpack_ok invalid: "
                f"{ci_sanity_canon_ast_dpack_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile in {"full", "core_lang"} and ci_sanity_canon_ast_dpack_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_canon_ast_dpack_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_contract_tier_unsupported_ok = kv.get("ci_sanity_contract_tier_unsupported_ok", "").strip()
        if ci_sanity_contract_tier_unsupported_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_contract_tier_unsupported_ok invalid: "
                f"{ci_sanity_contract_tier_unsupported_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile in {"full", "core_lang"} and ci_sanity_contract_tier_unsupported_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_contract_tier_unsupported_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_contract_tier_age3_min_enforcement_ok = (
            kv.get("ci_sanity_contract_tier_age3_min_enforcement_ok", "").strip()
        )
        if ci_sanity_contract_tier_age3_min_enforcement_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_contract_tier_age3_min_enforcement_ok invalid: "
                f"{ci_sanity_contract_tier_age3_min_enforcement_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile in {"full", "core_lang"} and ci_sanity_contract_tier_age3_min_enforcement_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_contract_tier_age3_min_enforcement_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_map_access_contract_ok = kv.get("ci_sanity_map_access_contract_ok", "").strip()
        if ci_sanity_map_access_contract_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_map_access_contract_ok invalid: "
                f"{ci_sanity_map_access_contract_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile in {"full", "core_lang"} and ci_sanity_map_access_contract_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_map_access_contract_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_stdlib_catalog_ok = kv.get("ci_sanity_stdlib_catalog_ok", "").strip()
        if ci_sanity_stdlib_catalog_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_stdlib_catalog_ok invalid: "
                f"{ci_sanity_stdlib_catalog_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile in {"full", "core_lang"} and ci_sanity_stdlib_catalog_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_stdlib_catalog_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_stdlib_catalog_selftest_ok = kv.get("ci_sanity_stdlib_catalog_selftest_ok", "").strip()
        if ci_sanity_stdlib_catalog_selftest_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_stdlib_catalog_selftest_ok invalid: "
                f"{ci_sanity_stdlib_catalog_selftest_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile in {"full", "core_lang"} and ci_sanity_stdlib_catalog_selftest_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_stdlib_catalog_selftest_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_tensor_v0_pack_ok = kv.get("ci_sanity_tensor_v0_pack_ok", "").strip()
        if ci_sanity_tensor_v0_pack_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_tensor_v0_pack_ok invalid: "
                f"{ci_sanity_tensor_v0_pack_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile in {"full", "core_lang"} and ci_sanity_tensor_v0_pack_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_tensor_v0_pack_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_tensor_v0_cli_ok = kv.get("ci_sanity_tensor_v0_cli_ok", "").strip()
        if ci_sanity_tensor_v0_cli_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_tensor_v0_cli_ok invalid: "
                f"{ci_sanity_tensor_v0_cli_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile in {"full", "core_lang"} and ci_sanity_tensor_v0_cli_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_tensor_v0_cli_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_fixed64_darwin_real_report_contract_ok = (
            kv.get("ci_sanity_fixed64_darwin_real_report_contract_ok", "").strip()
        )
        if ci_sanity_fixed64_darwin_real_report_contract_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_fixed64_darwin_real_report_contract_ok invalid: "
                f"{ci_sanity_fixed64_darwin_real_report_contract_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_fixed64_darwin_real_report_contract_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_fixed64_darwin_real_report_contract_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_fixed64_darwin_real_report_live_ok = (
            kv.get("ci_sanity_fixed64_darwin_real_report_live_ok", "").strip()
        )
        if ci_sanity_fixed64_darwin_real_report_live_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_fixed64_darwin_real_report_live_ok invalid: "
                f"{ci_sanity_fixed64_darwin_real_report_live_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_fixed64_darwin_real_report_live_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_fixed64_darwin_real_report_live_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_fixed64_darwin_real_report_readiness_selftest_ok = (
            kv.get("ci_sanity_fixed64_darwin_real_report_readiness_selftest_ok", "").strip()
        )
        if ci_sanity_fixed64_darwin_real_report_readiness_selftest_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_fixed64_darwin_real_report_readiness_selftest_ok invalid: "
                f"{ci_sanity_fixed64_darwin_real_report_readiness_selftest_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_fixed64_darwin_real_report_readiness_selftest_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_fixed64_darwin_real_report_readiness_selftest_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_registry_strict_audit_ok = kv.get("ci_sanity_registry_strict_audit_ok", "").strip()
        if ci_sanity_registry_strict_audit_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_registry_strict_audit_ok invalid: "
                f"{ci_sanity_registry_strict_audit_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile in {"full", "core_lang"} and ci_sanity_registry_strict_audit_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_registry_strict_audit_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_registry_defaults_ok = kv.get("ci_sanity_registry_defaults_ok", "").strip()
        if ci_sanity_registry_defaults_ok not in {"0", "1"}:
            return fail(
                "ci_sanity_registry_defaults_ok invalid: "
                f"{ci_sanity_registry_defaults_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile in {"full", "core_lang"} and ci_sanity_registry_defaults_ok != "1":
            return fail(
                "PASS summary requires ci_sanity_registry_defaults_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sanity_step_count = kv.get("ci_sanity_gate_step_count", "").strip()
        try:
            ci_sanity_step_count_num = int(ci_sanity_step_count)
        except Exception:
            return fail("ci_sanity_gate_step_count is not an integer", code=CODES["PASS_KEY_MISSING"])
        if ci_sanity_step_count_num <= 0:
            return fail(
                f"PASS summary requires ci_sanity_gate_step_count>0, got={ci_sanity_step_count_num}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sanity_profile == "full" and ci_sanity_step_count_num < MIN_REQUIRED_CI_SANITY_STEPS:
            return fail(
                "PASS summary requires ci_sanity_gate_step_count>="
                f"{MIN_REQUIRED_CI_SANITY_STEPS}, got={ci_sanity_step_count_num}",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sync_status = kv.get("ci_sync_readiness_status", "").strip()
        if ci_sync_status != "pass":
            return fail(
                f"PASS summary requires ci_sync_readiness_status=pass, got={ci_sync_status}",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sync_ok = kv.get("ci_sync_readiness_ok", "").strip()
        if ci_sync_ok != "1":
            return fail(
                f"PASS summary requires ci_sync_readiness_ok=1, got={ci_sync_ok}",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sync_code = kv.get("ci_sync_readiness_code", "").strip()
        if ci_sync_code != "OK":
            return fail(
                f"PASS summary requires ci_sync_readiness_code=OK, got={ci_sync_code}",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sync_step = kv.get("ci_sync_readiness_step", "").strip()
        if ci_sync_step != "all":
            return fail(
                f"PASS summary requires ci_sync_readiness_step=all, got={ci_sync_step}",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sync_sanity_profile = kv.get("ci_sync_readiness_sanity_profile", "").strip() or "full"
        if ci_sync_sanity_profile not in VALID_SANITY_PROFILES:
            return fail(
                f"ci_sync_readiness_sanity_profile invalid: {ci_sync_sanity_profile}",
                code=CODES["PASS_KEY_MISSING"],
            )
        if ci_sync_sanity_profile != ci_sanity_profile:
            return fail(
                "ci_sync_readiness_sanity_profile mismatch "
                f"summary={ci_sync_sanity_profile} sanity={ci_sanity_profile}",
                code=CODES["PASS_KEY_MISSING"],
            )
        ci_sync_step_count = kv.get("ci_sync_readiness_step_count", "").strip()
        try:
            ci_sync_step_count_num = int(ci_sync_step_count)
        except Exception:
            return fail("ci_sync_readiness_step_count is not an integer", code=CODES["PASS_KEY_MISSING"])
        if ci_sync_step_count_num <= 0:
            return fail(
                f"PASS summary requires ci_sync_readiness_step_count>0, got={ci_sync_step_count_num}",
                code=CODES["PASS_KEY_MISSING"],
            )
        for key, valid_profiles in SANITY_RUNTIME_HELPER_SUMMARY_FIELDS:
            value = kv.get(key, "").strip()
            if not value and key in OPTIONAL_FIXED64_LIVE_SANITY_SUMMARY_KEYS:
                continue
            expected_value = expected_runtime_helper_summary_value(key, ci_sync_sanity_profile, valid_profiles)
            if value not in valid_runtime_helper_summary_values(expected_value):
                return fail(f"{key} invalid: {value}", code=CODES["PASS_KEY_MISSING"])
            if value != expected_value:
                return fail(
                    f"PASS summary requires {key}={expected_value}, got={value}",
                    code=CODES["PASS_KEY_MISSING"],
                )
        for key, value_kind, valid_profiles in SANITY_RUNTIME_HELPER_TEXT_FIELDS:
            value = kv.get(key, "").strip()
            if not value:
                if key in OPTIONAL_FIXED64_LIVE_SANITY_SUMMARY_KEYS:
                    continue
                return fail(f"{key} missing", code=CODES["PASS_KEY_MISSING"])
            if ci_sync_sanity_profile not in valid_profiles:
                if value not in {"na", "-"}:
                    return fail(
                        f"PASS summary requires {key}=na|- , got={value}",
                        code=CODES["PASS_KEY_MISSING"],
                    )
                continue
            error = validate_failure_code_field_value(key, value, value_kind)
            if error is not None:
                return fail(error, code=CODES["PASS_KEY_MISSING"])
        for key, expected_value in SANITY_RUNTIME_HELPER_CONTRACT_FIELDS:
            value = kv.get(key, "").strip()
            if value != expected_value:
                return fail(
                    f"PASS summary requires {key}={expected_value}, got={value}",
                    code=CODES["PASS_KEY_MISSING"],
                )
        for key, valid_profiles in SYNC_RUNTIME_HELPER_SUMMARY_FIELDS:
            value = kv.get(key, "").strip()
            if not value and key in OPTIONAL_FIXED64_LIVE_SYNC_SUMMARY_KEYS:
                continue
            expected_value = expected_runtime_helper_summary_value(key, ci_sync_sanity_profile, valid_profiles)
            if value not in valid_runtime_helper_summary_values(expected_value):
                return fail(f"{key} invalid: {value}", code=CODES["PASS_KEY_MISSING"])
            if value != expected_value:
                return fail(
                    f"PASS summary requires {key}={expected_value}, got={value}",
                    code=CODES["PASS_KEY_MISSING"],
                )
        for sanity_key, sync_key in AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS:
            sanity_value = kv.get(sanity_key, "").strip()
            sync_value = kv.get(sync_key, "").strip()
            if sanity_value != sync_value:
                return fail(
                    f"PASS summary requires {sync_key}={sanity_value}, got={sync_value}",
                    code=CODES["PASS_KEY_MISSING"],
                )
        for key, source_key, value_kind, valid_profiles in SYNC_RUNTIME_HELPER_TEXT_FIELDS:
            value = kv.get(key, "").strip()
            if not value:
                if key in OPTIONAL_FIXED64_LIVE_SYNC_SUMMARY_KEYS:
                    continue
                return fail(f"{key} missing", code=CODES["PASS_KEY_MISSING"])
            if ci_sync_sanity_profile not in valid_profiles:
                if value not in {"na", "-"}:
                    return fail(
                        f"PASS summary requires {key}=na|- , got={value}",
                        code=CODES["PASS_KEY_MISSING"],
                    )
                continue
            error = validate_failure_code_field_value(key, value, value_kind)
            if error is not None:
                return fail(error, code=CODES["PASS_KEY_MISSING"])
            source_value = kv.get(source_key, "").strip()
            if value != source_value:
                return fail(
                    f"PASS summary requires {key}={source_value}, got={value}",
                    code=CODES["PASS_KEY_MISSING"],
                )
        for code_key, count_key in FAILURE_CODE_PAIR_KEYS:
            code_value = kv.get(code_key, "").strip()
            count_value = kv.get(count_key, "").strip()
            if code_value == "na" and count_value == "na":
                continue
            try:
                count_num = int(count_value)
            except Exception:
                return fail(f"{count_key} invalid: {count_value}", code=CODES["PASS_KEY_MISSING"])
            if code_value == "-" and count_num != 0:
                return fail(
                    f"PASS summary failure-code/count mismatch: {code_key}={code_value} {count_key}={count_num}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if code_value != "-":
                code_items = [token.strip() for token in code_value.split(",") if token.strip()]
                if len(code_items) != count_num:
                    return fail(
                        f"PASS summary failure-code/count mismatch: {code_key}={code_value} {count_key}={count_num}",
                        code=CODES["PASS_KEY_MISSING"],
                    )
        for key, expected_value in SYNC_RUNTIME_HELPER_CONTRACT_FIELDS:
            value = kv.get(key, "").strip()
            if value != expected_value:
                return fail(
                    f"PASS summary requires {key}={expected_value}, got={value}",
                    code=CODES["PASS_KEY_MISSING"],
                )
        age3_smoke_report_path_text = (
            kv.get("ci_sanity_age3_bogae_geoul_visibility_smoke_report_path", "").strip()
        )
        if not age3_smoke_report_path_text or age3_smoke_report_path_text == "-":
            return fail(
                "PASS summary requires ci_sanity_age3_bogae_geoul_visibility_smoke_report_path",
                code=CODES["PASS_KEY_MISSING"],
            )
        age3_smoke_report_path = Path(age3_smoke_report_path_text)
        if not age3_smoke_report_path.exists():
            return fail(
                "PASS summary requires existing ci_sanity_age3_bogae_geoul_visibility_smoke_report_path="
                f"{age3_smoke_report_path}",
                code=CODES["PASS_KEY_MISSING"],
            )
        smoke_schema_summary = kv.get("ci_sanity_age3_bogae_geoul_visibility_smoke_schema", "").strip() or "-"
        if smoke_schema_summary != AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA:
            return fail(
                "PASS summary requires ci_sanity_age3_bogae_geoul_visibility_smoke_schema="
                f"{AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA}, got={smoke_schema_summary}",
                code=CODES["PASS_KEY_MISSING"],
            )
        smoke_doc = load_json(age3_smoke_report_path)
        if smoke_doc is None:
            return fail(
                f"invalid smoke report json: {age3_smoke_report_path}",
                code=CODES["PASS_KEY_MISSING"],
            )
        smoke_doc_schema = str(smoke_doc.get("schema", "")).strip()
        if smoke_doc_schema != AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA:
            return fail(
                f"smoke report schema mismatch summary={smoke_schema_summary} report={smoke_doc_schema}",
                code=CODES["PASS_KEY_MISSING"],
            )
        smoke_doc_overall_ok = bool(smoke_doc.get("overall_ok", False))
        smoke_doc_checks = smoke_doc.get("checks")
        smoke_doc_checks_ok = isinstance(smoke_doc_checks, list) and bool(smoke_doc_checks)
        if not smoke_doc_overall_ok:
            return fail(
                "PASS summary requires smoke report overall_ok=true",
                code=CODES["PASS_KEY_MISSING"],
            )
        if not smoke_doc_checks_ok:
            return fail(
                "PASS summary requires smoke report checks to be non-empty list",
                code=CODES["PASS_KEY_MISSING"],
            )
        smoke_doc_sim_hash_delta = smoke_doc.get("simulation_hash_delta")
        if not isinstance(smoke_doc_sim_hash_delta, dict):
            return fail(
                "PASS summary requires smoke report simulation_hash_delta object",
                code=CODES["PASS_KEY_MISSING"],
            )
        smoke_doc_sim_state_hash_changes = "1" if bool(smoke_doc_sim_hash_delta.get("state_hash_changes", False)) else "0"
        smoke_doc_sim_bogae_hash_changes = "1" if bool(smoke_doc_sim_hash_delta.get("bogae_hash_changes", False)) else "0"
        if smoke_doc_sim_state_hash_changes != "1":
            return fail(
                "PASS summary requires smoke report simulation_hash_delta.state_hash_changes=true",
                code=CODES["PASS_KEY_MISSING"],
            )
        if smoke_doc_sim_bogae_hash_changes != "1":
            return fail(
                "PASS summary requires smoke report simulation_hash_delta.bogae_hash_changes=true",
                code=CODES["PASS_KEY_MISSING"],
            )
        for sanity_key, sync_key in (
            (
                "ci_sanity_age3_bogae_geoul_visibility_smoke_ok",
                "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_ok",
            ),
            (
                "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path",
                "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_path",
            ),
            (
                "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists",
                "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists",
            ),
            (
                "ci_sanity_age3_bogae_geoul_visibility_smoke_schema",
                "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_schema",
            ),
            (
                "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok",
                "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok",
            ),
            (
                "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok",
                "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok",
            ),
            (
                "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes",
                "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes",
            ),
            (
                "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes",
                "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes",
            ),
        ):
            sanity_value = kv.get(sanity_key, "").strip()
            sync_value = kv.get(sync_key, "").strip()
            if sanity_value != sync_value:
                return fail(
                    f"{sync_key} mismatch summary={sync_value} sanity={sanity_value}",
                    code=CODES["SUMMARY_INDEX_PATH_MISMATCH"],
                )
        if kv.get("ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists", "").strip() != "1":
            return fail(
                "PASS summary requires ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        if kv.get("ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok", "").strip() != "1":
            return fail(
                "PASS summary requires ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        if kv.get("ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok", "").strip() != "1":
            return fail(
                "PASS summary requires ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        if kv.get("ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes", "").strip() != "1":
            return fail(
                "PASS summary requires ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        if kv.get("ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes", "").strip() != "1":
            return fail(
                "PASS summary requires ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes=1",
                code=CODES["PASS_KEY_MISSING"],
            )
        if kv.get("ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes", "").strip() != (
            smoke_doc_sim_state_hash_changes
        ):
            return fail(
                "PASS summary/report mismatch: smoke sim_state_hash_changes",
                code=CODES["PASS_KEY_MISSING"],
            )
        if kv.get("ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes", "").strip() != (
            smoke_doc_sim_bogae_hash_changes
        ):
            return fail(
                "PASS summary/report mismatch: smoke sim_bogae_hash_changes",
                code=CODES["PASS_KEY_MISSING"],
            )

        if ci_sync_sanity_profile == "seamgrim":
            pack_evidence_report_path_text = kv.get(
                "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path",
                "",
            ).strip()
            if not pack_evidence_report_path_text or pack_evidence_report_path_text == "-":
                return fail(
                    "PASS summary requires ci_sanity_seamgrim_pack_evidence_tier_runner_report_path",
                    code=CODES["PASS_KEY_MISSING"],
                )
            pack_evidence_report_path = Path(pack_evidence_report_path_text)
            if not pack_evidence_report_path.exists():
                return fail(
                    "PASS summary requires existing ci_sanity_seamgrim_pack_evidence_tier_runner_report_path="
                    f"{pack_evidence_report_path}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            pack_evidence_schema_summary = (
                kv.get("ci_sanity_seamgrim_pack_evidence_tier_runner_schema", "").strip() or "-"
            )
            if pack_evidence_schema_summary != SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA:
                return fail(
                    "PASS summary requires ci_sanity_seamgrim_pack_evidence_tier_runner_schema="
                    f"{SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA}, got={pack_evidence_schema_summary}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            pack_evidence_docs_issue_count_text = kv.get(
                "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count",
                "",
            ).strip()
            pack_evidence_repo_issue_count_text = kv.get(
                "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count",
                "",
            ).strip()
            try:
                pack_evidence_docs_issue_count_num = int(pack_evidence_docs_issue_count_text)
            except Exception:
                return fail(
                    "PASS summary requires integer ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count, "
                    f"got={pack_evidence_docs_issue_count_text}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if (
                pack_evidence_docs_issue_count_num < 0
                or pack_evidence_docs_issue_count_num > SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_MAX_DOCS_ISSUES
            ):
                return fail(
                    "PASS summary requires ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count within "
                    f"0..{SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_MAX_DOCS_ISSUES}, "
                    f"got={pack_evidence_docs_issue_count_num}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            try:
                pack_evidence_repo_issue_count_num = int(pack_evidence_repo_issue_count_text)
            except Exception:
                return fail(
                    "PASS summary requires integer ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count, "
                    f"got={pack_evidence_repo_issue_count_text}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if pack_evidence_repo_issue_count_num != SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_EXPECTED_REPO_ISSUES:
                return fail(
                    "PASS summary requires ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count="
                    f"{SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_EXPECTED_REPO_ISSUES}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            pack_evidence_doc = load_json(pack_evidence_report_path)
            if pack_evidence_doc is None:
                return fail(
                    f"invalid pack evidence report json: {pack_evidence_report_path}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if str(pack_evidence_doc.get("schema", "")).strip() != SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA:
                return fail(
                    "pack evidence report schema mismatch "
                    f"summary={pack_evidence_schema_summary} report={pack_evidence_doc.get('schema')}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if str(pack_evidence_doc.get("status", "")).strip() != "pass":
                return fail(
                    "PASS summary requires pack evidence report status=pass",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if not bool(pack_evidence_doc.get("ok", False)):
                return fail(
                    "PASS summary requires pack evidence report ok=true",
                    code=CODES["PASS_KEY_MISSING"],
                )
            docs_profile_doc = pack_evidence_doc.get("docs_profile")
            repo_profile_doc = pack_evidence_doc.get("repo_profile")
            if not isinstance(docs_profile_doc, dict) or not isinstance(repo_profile_doc, dict):
                return fail(
                    "PASS summary requires pack evidence docs_profile/repo_profile",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if str(docs_profile_doc.get("name", "")).strip() != "docs_ssot_rep10":
                return fail(
                    "PASS summary requires pack evidence docs_profile.name=docs_ssot_rep10",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if str(repo_profile_doc.get("name", "")).strip() != "repo_rep10":
                return fail(
                    "PASS summary requires pack evidence repo_profile.name=repo_rep10",
                    code=CODES["PASS_KEY_MISSING"],
                )
            try:
                pack_evidence_doc_docs_issue_count = int(docs_profile_doc.get("issue_count", -1))
                pack_evidence_doc_repo_issue_count = int(repo_profile_doc.get("issue_count", -1))
            except Exception:
                return fail(
                    "PASS summary requires pack evidence issue_count integers",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if pack_evidence_doc_docs_issue_count != pack_evidence_docs_issue_count_num:
                return fail(
                    "PASS summary requires pack evidence docs_issue_count parity "
                    f"summary={pack_evidence_docs_issue_count_num} report={pack_evidence_doc_docs_issue_count}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if pack_evidence_doc_repo_issue_count != pack_evidence_repo_issue_count_num:
                return fail(
                    "PASS summary requires pack evidence repo_issue_count parity "
                    f"summary={pack_evidence_repo_issue_count_num} report={pack_evidence_doc_repo_issue_count}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            for sanity_key, sync_key in (
                (
                    "ci_sanity_seamgrim_pack_evidence_tier_runner_ok",
                    "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok",
                ),
                (
                    "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path",
                    "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_path",
                ),
                (
                    "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists",
                    "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists",
                ),
                (
                    "ci_sanity_seamgrim_pack_evidence_tier_runner_schema",
                    "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_schema",
                ),
                (
                    "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count",
                    "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count",
                ),
                (
                    "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count",
                    "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count",
                ),
            ):
                sanity_value = kv.get(sanity_key, "").strip()
                sync_value = kv.get(sync_key, "").strip()
                if sanity_value != sync_value:
                    return fail(
                        f"{sync_key} mismatch summary={sync_value} sanity={sanity_value}",
                        code=CODES["SUMMARY_INDEX_PATH_MISMATCH"],
                    )
            if kv.get("ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists", "").strip() != "1":
                return fail(
                    "PASS summary requires ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists=1",
                    code=CODES["PASS_KEY_MISSING"],
                )

            step_report_path_text = kv.get("ci_sanity_seamgrim_wasm_web_step_check_report_path", "").strip()
            if not step_report_path_text or step_report_path_text == "-":
                return fail(
                    "PASS summary requires ci_sanity_seamgrim_wasm_web_step_check_report_path",
                    code=CODES["PASS_KEY_MISSING"],
                )
            step_report_path = Path(step_report_path_text)
            if not step_report_path.exists():
                return fail(
                    "PASS summary requires existing ci_sanity_seamgrim_wasm_web_step_check_report_path="
                    f"{step_report_path}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            step_schema_summary = kv.get("ci_sanity_seamgrim_wasm_web_step_check_schema", "").strip() or "-"
            if step_schema_summary != SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA:
                return fail(
                    "PASS summary requires ci_sanity_seamgrim_wasm_web_step_check_schema="
                    f"{SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA}, got={step_schema_summary}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            step_checked_files_text = kv.get("ci_sanity_seamgrim_wasm_web_step_check_checked_files", "").strip()
            step_missing_count_text = kv.get("ci_sanity_seamgrim_wasm_web_step_check_missing_count", "").strip()
            try:
                step_checked_files_num = int(step_checked_files_text)
            except Exception:
                return fail(
                    "PASS summary requires integer ci_sanity_seamgrim_wasm_web_step_check_checked_files, "
                    f"got={step_checked_files_text}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if step_checked_files_num < SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES:
                return fail(
                    "PASS summary requires ci_sanity_seamgrim_wasm_web_step_check_checked_files"
                    f">={SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES}, got={step_checked_files_num}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            try:
                step_missing_count_num = int(step_missing_count_text)
            except Exception:
                return fail(
                    "PASS summary requires integer ci_sanity_seamgrim_wasm_web_step_check_missing_count, "
                    f"got={step_missing_count_text}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if step_missing_count_num != 0:
                return fail(
                    "PASS summary requires ci_sanity_seamgrim_wasm_web_step_check_missing_count=0",
                    code=CODES["PASS_KEY_MISSING"],
                )
            step_doc = load_json(step_report_path)
            if step_doc is None:
                return fail(
                    f"invalid wasm/web step check report json: {step_report_path}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if str(step_doc.get("schema", "")).strip() != SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA:
                return fail(
                    "wasm/web step check report schema mismatch "
                    f"summary={step_schema_summary} report={step_doc.get('schema')}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if str(step_doc.get("status", "")).strip() != "pass":
                return fail(
                    "PASS summary requires wasm/web step check report status=pass",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if not bool(step_doc.get("ok", False)):
                return fail(
                    "PASS summary requires wasm/web step check report ok=true",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if str(step_doc.get("code", "")).strip() != "OK":
                return fail(
                    "PASS summary requires wasm/web step check report code=OK",
                    code=CODES["PASS_KEY_MISSING"],
                )
            try:
                step_doc_checked_files = int(step_doc.get("checked_files", -1))
            except Exception:
                step_doc_checked_files = -1
            try:
                step_doc_missing_count = int(step_doc.get("missing_count", -1))
            except Exception:
                step_doc_missing_count = -1
            if step_doc_checked_files != step_checked_files_num:
                return fail(
                    "PASS summary requires wasm/web step check checked_files parity "
                    f"summary={step_checked_files_num} report={step_doc_checked_files}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            if step_doc_missing_count != step_missing_count_num:
                return fail(
                    "PASS summary requires wasm/web step check missing_count parity "
                    f"summary={step_missing_count_num} report={step_doc_missing_count}",
                    code=CODES["PASS_KEY_MISSING"],
                )
            step_doc_missing = step_doc.get("missing")
            if not isinstance(step_doc_missing, list) or step_doc_missing:
                return fail(
                    "PASS summary requires wasm/web step check report missing=[]",
                    code=CODES["PASS_KEY_MISSING"],
                )
            for sanity_key, sync_key in (
                (
                    "ci_sanity_seamgrim_wasm_web_step_check_ok",
                    "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_ok",
                ),
                (
                    "ci_sanity_seamgrim_wasm_web_step_check_report_path",
                    "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_path",
                ),
                (
                    "ci_sanity_seamgrim_wasm_web_step_check_report_exists",
                    "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_exists",
                ),
                (
                    "ci_sanity_seamgrim_wasm_web_step_check_schema",
                    "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_schema",
                ),
                (
                    "ci_sanity_seamgrim_wasm_web_step_check_checked_files",
                    "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_checked_files",
                ),
                (
                    "ci_sanity_seamgrim_wasm_web_step_check_missing_count",
                    "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_missing_count",
                ),
            ):
                sanity_value = kv.get(sanity_key, "").strip()
                sync_value = kv.get(sync_key, "").strip()
                if sanity_value != sync_value:
                    return fail(
                        f"{sync_key} mismatch summary={sync_value} sanity={sanity_value}",
                        code=CODES["SUMMARY_INDEX_PATH_MISMATCH"],
                    )
            if kv.get("ci_sanity_seamgrim_wasm_web_step_check_report_exists", "").strip() != "1":
                return fail(
                    "PASS summary requires ci_sanity_seamgrim_wasm_web_step_check_report_exists=1",
                    code=CODES["PASS_KEY_MISSING"],
                )

        hint = kv.get("ci_fail_brief_hint", "").strip()
        if not hint:
            return fail("ci_fail_brief_hint is empty", code=CODES["BRIEF_HINT_EMPTY"])
        exists_text = kv.get("ci_fail_brief_exists", "").strip()
        if exists_text not in {"0", "1"}:
            return fail(f"ci_fail_brief_exists invalid: {exists_text}", code=CODES["BRIEF_EXISTS_INVALID"])
        brief_exists_actual = 1 if Path(hint).exists() else 0
        if int(exists_text) != brief_exists_actual:
            return fail(
                f"ci_fail_brief_exists mismatch summary={exists_text} actual={brief_exists_actual}",
                code=CODES["BRIEF_EXISTS_MISMATCH"],
            )
        if brief_exists_actual != 1:
            return fail(f"PASS summary requires ci_fail_brief_exists=1 path={hint}", code=CODES["PASS_BRIEF_REQUIRED"])
        triage_hint = kv.get("ci_fail_triage_hint", "").strip()
        if not triage_hint:
            return fail("ci_fail_triage_hint is empty", code=CODES["TRIAGE_HINT_EMPTY"])
        triage_exists_text = kv.get("ci_fail_triage_exists", "").strip()
        if triage_exists_text not in {"0", "1"}:
            return fail(f"ci_fail_triage_exists invalid: {triage_exists_text}", code=CODES["TRIAGE_EXISTS_INVALID"])
        triage_exists_actual = 1 if Path(triage_hint).exists() else 0
        if int(triage_exists_text) != triage_exists_actual:
            return fail(
                f"ci_fail_triage_exists mismatch summary={triage_exists_text} actual={triage_exists_actual}",
                code=CODES["TRIAGE_EXISTS_MISMATCH"],
            )
        if triage_exists_actual != 1:
            return fail(
                f"PASS summary requires ci_fail_triage_exists=1 path={triage_hint}",
                code=CODES["PASS_TRIAGE_REQUIRED"],
            )

    print(
        f"[ci-gate-summary-report-check] ok status={status} "
        f"summary={summary_path} index={index_path}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
