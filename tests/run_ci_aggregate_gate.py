#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

from _ci_age3_completion_gate_contract import (
    AGE3_COMPLETION_GATE_CRITERIA_NAMES,
    age3_completion_gate_criteria_summary_key,
    age3_completion_gate_criteria_sync_summary_key,
)
from _ci_age5_combined_heavy_contract import (
    AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY,
    AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY,
    AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_TEXT_KEY,
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY,
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY,
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_TEXT_KEY,
    AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
    AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY,
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
    AGE5_FULL_REAL_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_PROOF_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_PROOF_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_PROOF_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_PROOF_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_PROOF_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_PROOF_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_GATE0_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_GATE0_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_GATE0_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_GATE0_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_GATE0_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_GATE0_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
    AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
    AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
    AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
    AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
    AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
    build_age5_combined_heavy_full_real_source_trace,
    build_age5_combined_heavy_full_real_source_trace_text,
    build_age5_combined_heavy_policy_origin_trace,
    build_age5_combined_heavy_policy_origin_trace_text,
    build_age5_combined_heavy_child_summary_default_text_transport_fields,
    build_age4_proof_snapshot,
    build_age4_proof_source_snapshot_fields,
    build_age4_proof_snapshot_text,
)
from _ci_aggregate_contract_only_lib import (
    CONTRACT_ONLY_AGE4_PROOF_FAILED_CRITERIA,
    CONTRACT_ONLY_AGE4_PROOF_OK,
    CONTRACT_ONLY_AGE4_PROOF_SUMMARY_HASH,
    build_contract_only_age5_aggregate_fields,
    resolve_contract_only_required_steps,
    resolve_contract_only_selected_profiles,
    write_json,
    write_contract_only_ci_gate_outputs,
    write_contract_only_ci_sanity_report,
    write_contract_only_ci_sync_readiness_report,
    write_contract_only_fixed64_reports,
    write_contract_only_profile_matrix_selftest_report,
    write_contract_only_stub_reports,
)
from _ci_aggregate_diag_lib import (
    append_ci_profile_matrix_selftest_summary_lines,
    append_ci_sanity_summary_lines,
    append_ci_sync_readiness_summary_lines,
    append_fixed64_threeway_summary_lines,
    append_runtime_5min_checklist_summary_lines,
    append_runtime_5min_summary_lines,
    append_seamgrim_focus_summary_lines,
    clip_line,
    load_payload,
    print_report_paths,
    read_compact_from_parse,
    read_compact_line,
    resolve_summary_compact,
    print_failure_block,
    write_summary,
    write_summary_line,
    write_control_exposure_failure_report,
)
from _ci_aggregate_runner_lib import run_step, sanitize_step_name
from _ci_latest_smoke_contract import (
    LATEST_SMOKE_SKIP_REASON_CI_GATE_RESULT_STATUS_NOT_PASS,
    LATEST_SMOKE_SKIP_REASON_EXPECTED,
    LATEST_SMOKE_SKIP_REASON_FAST_FAIL_PATH,
    LATEST_SMOKE_SKIP_REASON_FLAG_DISABLED,
    LATEST_SMOKE_SKIP_REASON_PENDING_FAILURE_SUMMARY_REGENERATION,
)

# Keep explicit summary-token contracts in this file so static diagnostics checks
# can validate aggregate-gate surface invariants after helper modularization.
RUNTIME5_SUMMARY_TOKEN_CONTRACT = [
    "[ci-gate-summary] seamgrim_5min_checklist=",
    "[ci-gate-summary] seamgrim_runtime_5min_rewrite_motion_projectile=",
    "[ci-gate-summary] seamgrim_runtime_5min_moyang_view_boundary=",
    "[ci-gate-summary] seamgrim_runtime_5min_pendulum_tetris_showcase=",
]

SEAMGRIM_SUMMARY_TOKEN_CONTRACT = [
    "[ci-gate-summary] seamgrim_control_exposure_policy_report=",
    "[ci-gate-summary] seamgrim_control_exposure_policy_status=",
    "[ci-gate-summary] seamgrim_seed_meta_files_status=",
    "[ci-gate-summary] seamgrim_seed_overlay_quality_status=",
    "[ci-gate-summary] seamgrim_rewrite_overlay_quality_status=",
    "[ci-gate-summary] seamgrim_guideblock_keys_pack_status=",
    "[ci-gate-summary] seamgrim_moyang_view_boundary_pack_status=",
    "[ci-gate-summary] seamgrim_rewrite_overlay_quality_report=",
    "[ci-gate-summary] seamgrim_rewrite_overlay_quality_top=",
    "[ci-gate-summary] seamgrim_pendulum_surface_contract_status=",
    "[ci-gate-summary] seamgrim_seed_pendulum_export_status=",
    "[ci-gate-summary] seamgrim_pendulum_runtime_visual_status=",
    "[ci-gate-summary] seamgrim_seed_runtime_visual_pack_status=",
    "[ci-gate-summary] seamgrim_group_id_summary_status=",
    "[ci-gate-summary] seamgrim_runtime_fallback_metrics_status=",
    "[ci-gate-summary] seamgrim_runtime_fallback_policy_status=",
    "[ci-gate-summary] seamgrim_pendulum_bogae_shape_status=",
    "[ci-gate-summary] seamgrim_wasm_cli_diag_parity_report=",
    "[ci-gate-summary] seamgrim_wasm_cli_diag_parity_ok=",
    "[ci-gate-summary] seamgrim_lesson_warning_tokens_status=",
    "[ci-gate-summary] seamgrim_lesson_warning_tokens_ok=",
    "[ci-gate-summary] seamgrim_stateful_sim_preview_upgrade_status=",
    "[ci-gate-summary] seamgrim_stateful_sim_preview_upgrade_ok=",
]

SANITY_SUMMARY_TOKEN_CONTRACT = [
    "[ci-gate-summary] ci_sanity_gate_report=",
    "[ci-gate-summary] ci_sanity_gate_status=",
    "[ci-gate-summary] ci_sanity_gate_code=",
    "[ci-gate-summary] ci_sanity_gate_profile=",
    "[ci-gate-summary] ci_sanity_age2_completion_gate_ok=",
    "[ci-gate-summary] ci_sanity_age2_close_ok=",
    "[ci-gate-summary] ci_sanity_age2_close_selftest_ok=",
    "[ci-gate-summary] ci_sanity_age2_completion_gate_failure_codes=",
    "[ci-gate-summary] ci_sanity_age2_completion_gate_failure_code_count=",
    "[ci-gate-summary] ci_sanity_age3_completion_gate_ok=",
    "[ci-gate-summary] ci_sanity_age3_close_ok=",
    "[ci-gate-summary] ci_sanity_age3_close_selftest_ok=",
    "[ci-gate-summary] ci_sanity_age3_completion_gate_failure_codes=",
    "[ci-gate-summary] ci_sanity_age3_completion_gate_failure_code_count=",
    "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_ok=",
    "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_report_path=",
    "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_report_exists=",
    "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_schema=",
    "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_checked_files=",
    "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_missing_count=",
    "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_ok=",
    "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_report_path=",
    "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_report_exists=",
    "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_schema=",
    "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_text=",
    "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_bit_limit=",
    "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_pollard_iters=",
    "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds=",
    "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds=",
    "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_fallback_limit=",
    "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_small_prime_max=",
    "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_ok=",
    "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_report_path=",
    "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists=",
    "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_schema=",
    "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count=",
    "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count=",
    "[ci-gate-summary] ci_sanity_pack_golden_lang_consistency_ok=",
    "[ci-gate-summary] ci_sanity_pack_golden_metadata_ok=",
    "[ci-gate-summary] ci_sanity_pack_golden_graph_export_ok=",
    "[ci-gate-summary] ci_sanity_canon_ast_dpack_ok=",
    "[ci-gate-summary] ci_sanity_contract_tier_unsupported_ok=",
    "[ci-gate-summary] ci_sanity_contract_tier_age3_min_enforcement_ok=",
    "[ci-gate-summary] ci_sanity_map_access_contract_ok=",
    "[ci-gate-summary] ci_sanity_stdlib_catalog_ok=",
    "[ci-gate-summary] ci_sanity_stdlib_catalog_selftest_ok=",
    "[ci-gate-summary] ci_sanity_tensor_v0_pack_ok=",
    "[ci-gate-summary] ci_sanity_tensor_v0_cli_ok=",
    "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_contract_ok=",
    "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_live_ok=",
    "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_readiness_selftest_ok=",
    "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_live_report_path=",
    "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_live_report_exists=",
    "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_live_status=",
    "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_live_resolved_status=",
    "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_live_resolved_source=",
    "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count=",
    "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip=",
    "[ci-gate-summary] ci_sanity_registry_strict_audit_ok=",
    "[ci-gate-summary] ci_sanity_registry_defaults_ok=",
]
SANITY_SUMMARY_TOKEN_CONTRACT.extend(
    [
        f"[ci-gate-summary] {age3_completion_gate_criteria_summary_key(criteria_name)}="
        for criteria_name in AGE3_COMPLETION_GATE_CRITERIA_NAMES
    ]
)

SYNC_SUMMARY_TOKEN_CONTRACT = [
    "[ci-gate-summary] ci_sync_readiness_report=",
    "[ci-gate-summary] ci_sync_readiness_status=",
    "[ci-gate-summary] ci_sync_readiness_code=",
    "[ci-gate-summary] ci_sync_readiness_sanity_profile=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_age2_completion_gate_failure_codes=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_age2_completion_gate_failure_code_count=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_age2_close_ok=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_age2_close_selftest_ok=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_completion_gate_failure_codes=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_completion_gate_failure_code_count=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_close_ok=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_close_selftest_ok=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_ok=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_path=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_exists=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_schema=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_checked_files=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_missing_count=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_report_path=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_report_exists=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_schema=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_text=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_bit_limit=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_iters=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_fallback_limit=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_small_prime_max=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_path=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_schema=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_report_path=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_report_exists=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_status=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_status=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_source=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count=",
    "[ci-gate-summary] ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip=",
]
SYNC_SUMMARY_TOKEN_CONTRACT.extend(
    [
        f"[ci-gate-summary] {age3_completion_gate_criteria_sync_summary_key(criteria_name)}="
        for criteria_name in AGE3_COMPLETION_GATE_CRITERIA_NAMES
    ]
)
VALID_AGE5_CHILD_SUMMARY_STATUS = {"pass", "fail", "skipped"}
AGE4_PROOF_SUMMARY_KEYS = (
    "age4_proof_ok",
    "age4_proof_failed_criteria",
    "age4_proof_failed_preview",
    "age4_proof_summary_hash",
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


def sanitize_report_prefix(value: str) -> str:
    raw = value.strip()
    if not raw:
        return ""
    out_chars: list[str] = []
    for ch in raw:
        if ch.isalnum() or ch in ("-", "_", "."):
            out_chars.append(ch)
        else:
            out_chars.append("_")
    sanitized = "".join(out_chars).strip("._-")
    return sanitized


def default_report_dir() -> str:
    preferred = Path("I:/home/urihanl/ddn/codex/build/reports")
    if os.name == "nt":
        try:
            preferred.mkdir(parents=True, exist_ok=True)
        except OSError:
            pass
        return str(preferred)
    return "build/reports"


def report_path(report_dir: Path, base_name: str, prefix: str) -> Path:
    if not prefix:
        return report_dir / base_name
    return report_dir / f"{prefix}.{base_name}"


def cleanup_prefixed_reports(
    report_dir: Path,
    prefix: str,
    base_names: list[str],
    dry_run: bool,
) -> int:
    count = 0
    for base_name in base_names:
        target = report_path(report_dir, base_name, prefix)
        if not target.exists():
            continue
        if dry_run:
            print(f"[ci-gate] clean(dry-run) would_remove={target}")
        else:
            target.unlink(missing_ok=True)
            print(f"[ci-gate] clean removed={target}")
        count += 1
    return count


def cleanup_prefixed_step_logs(step_log_dir: Path, prefix: str, dry_run: bool) -> int:
    pattern = f"{prefix}.ci_gate_step_*.txt" if prefix else "ci_gate_step_*.txt"
    count = 0
    for target in sorted(step_log_dir.glob(pattern)):
        if not target.exists():
            continue
        if dry_run:
            print(f"[ci-gate] clean(dry-run) would_remove={target}")
        else:
            target.unlink(missing_ok=True)
            print(f"[ci-gate] clean removed={target}")
        count += 1
    return count


def append_age5_child_summary_lines(lines: list[str], age5_report_path: Path) -> None:
    # diagnostics token anchors:
    # age5_combined_heavy_child_timeout_sec=
    # age5_combined_heavy_timeout_mode=
    # age5_combined_heavy_timeout_present=
    # age5_combined_heavy_timeout_targets=
    # age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks=
    # age5_full_real_lang_surface_family_transport_contract_selftest_total_checks=
    # age5_full_real_lang_surface_family_transport_contract_selftest_checks_text=
    # age5_full_real_lang_surface_family_transport_contract_selftest_current_probe=
    # age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe=
    # age5_full_real_lang_surface_family_transport_contract_selftest_progress_present=
    # age4_proof_gate_result_snapshot_text=
    # age4_proof_gate_result_snapshot_present=
    # age4_proof_gate_result_snapshot_parity=
    # age4_proof_final_status_parse_snapshot_text=
    # age4_proof_final_status_parse_snapshot_present=
    # age4_proof_final_status_parse_snapshot_parity=
    # age5_full_real_w107_golden_index_selftest_active_cases=
    # age5_full_real_w107_golden_index_selftest_inactive_cases=
    # age5_full_real_w107_golden_index_selftest_index_codes=
    # age5_full_real_w107_golden_index_selftest_current_probe=
    # age5_full_real_w107_golden_index_selftest_last_completed_probe=
    # age5_full_real_w107_golden_index_selftest_progress_present=
    # age5_full_real_w107_progress_contract_selftest_completed_checks=
    # age5_full_real_w107_progress_contract_selftest_total_checks=
    # age5_full_real_w107_progress_contract_selftest_checks_text=
    # age5_full_real_w107_progress_contract_selftest_current_probe=
    # age5_full_real_w107_progress_contract_selftest_last_completed_probe=
    # age5_full_real_w107_progress_contract_selftest_progress_present=
    # age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks=
    # age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks=
    # age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text=
    # age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe=
    # age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe=
    # age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present=
    # age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks=
    # age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks=
    # age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text=
    # age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe=
    # age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe=
    # age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present=
    # age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks=
    # age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks=
    # age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text=
    # age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe=
    # age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe=
    # age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present=
    # age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks=
    # age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks=
    # age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text=
    # age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe=
    # age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe=
    # age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present=
    # age5_full_real_proof_certificate_family_contract_selftest_completed_checks=
    # age5_full_real_proof_certificate_family_contract_selftest_total_checks=
    # age5_full_real_proof_certificate_family_contract_selftest_checks_text=
    # age5_full_real_proof_certificate_family_contract_selftest_current_probe=
    # age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe=
    # age5_full_real_proof_certificate_family_contract_selftest_progress_present=
    # age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks=
    # age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks=
    # age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text=
    # age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe=
    # age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe=
    # age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present=
    # age5_full_real_proof_family_contract_selftest_completed_checks=
    # age5_full_real_proof_family_contract_selftest_total_checks=
    # age5_full_real_proof_family_contract_selftest_checks_text=
    # age5_full_real_proof_family_contract_selftest_current_probe=
    # age5_full_real_proof_family_contract_selftest_last_completed_probe=
    # age5_full_real_proof_family_contract_selftest_progress_present=
    # age5_full_real_proof_family_transport_contract_selftest_completed_checks=
    # age5_full_real_proof_family_transport_contract_selftest_total_checks=
    # age5_full_real_proof_family_transport_contract_selftest_checks_text=
    # age5_full_real_proof_family_transport_contract_selftest_current_probe=
    # age5_full_real_proof_family_transport_contract_selftest_last_completed_probe=
    # age5_full_real_proof_family_transport_contract_selftest_progress_present=
    # age5_full_real_lang_surface_family_contract_selftest_completed_checks=
    # age5_full_real_lang_surface_family_contract_selftest_total_checks=
    # age5_full_real_lang_surface_family_contract_selftest_checks_text=
    # age5_full_real_lang_surface_family_contract_selftest_current_probe=
    # age5_full_real_lang_surface_family_contract_selftest_last_completed_probe=
    # age5_full_real_lang_surface_family_contract_selftest_progress_present=
    # age5_full_real_bogae_alias_family_contract_selftest_completed_checks=
    # age5_full_real_bogae_alias_family_contract_selftest_total_checks=
    # age5_full_real_bogae_alias_family_contract_selftest_checks_text=
    # age5_full_real_bogae_alias_family_contract_selftest_current_probe=
    # age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe=
    # age5_full_real_bogae_alias_family_contract_selftest_progress_present=
    # age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks=
    # age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks=
    # age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text=
    # age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe=
    # age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe=
    # age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present=
    doc = load_payload(age5_report_path)
    age4_proof_snapshot_fields_text = (
        str((doc or {}).get("age4_proof_snapshot_fields_text", AGE4_PROOF_SNAPSHOT_FIELDS_TEXT)).strip()
        or AGE4_PROOF_SNAPSHOT_FIELDS_TEXT
    )
    age4_proof_snapshot = build_age4_proof_snapshot(
        age4_proof_ok=(doc or {}).get("age4_proof_ok", "0"),
        age4_proof_failed_criteria=(doc or {}).get("age4_proof_failed_criteria", "-1"),
        age4_proof_failed_preview=(doc or {}).get("age4_proof_failed_preview", "-"),
    )
    age4_proof_snapshot_text = (
        str((doc or {}).get("age4_proof_snapshot_text", "")).strip()
        or build_age4_proof_snapshot_text(age4_proof_snapshot)
    )
    age4_proof_source_fields = build_age4_proof_source_snapshot_fields(top_snapshot=age4_proof_snapshot)
    age4_proof_gate_result_snapshot_text = (
        str((doc or {}).get(AGE4_PROOF_GATE_RESULT_SNAPSHOT_TEXT_KEY, "")).strip()
        or age4_proof_source_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_TEXT_KEY]
    )
    age4_proof_gate_result_snapshot_present = (
        str((doc or {}).get(AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY, "")).strip()
        or age4_proof_source_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY]
    )
    age4_proof_gate_result_snapshot_parity = (
        str((doc or {}).get(AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY, "")).strip()
        or age4_proof_source_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY]
    )
    age4_proof_final_status_parse_snapshot_text = (
        str((doc or {}).get(AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_TEXT_KEY, "")).strip()
        or age4_proof_source_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_TEXT_KEY]
    )
    age4_proof_final_status_parse_snapshot_present = (
        str((doc or {}).get(AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY, "")).strip()
        or age4_proof_source_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY]
    )
    age4_proof_final_status_parse_snapshot_parity = (
        str((doc or {}).get(AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY, "")).strip()
        or age4_proof_source_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY]
    )
    for key in AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS:
        value = str((doc or {}).get(key, "skipped")).strip()
        if value not in VALID_AGE5_CHILD_SUMMARY_STATUS:
            value = "skipped"
        lines.append(f"[ci-gate-summary] {key}={value}")
    default_transport = build_age5_combined_heavy_child_summary_default_text_transport_fields()
    for key, expected in default_transport.items():
        value = str((doc or {}).get(key, expected)).strip() or expected
        lines.append(f"[ci-gate-summary] {key}={value}")
    full_real_source_trace = (doc or {}).get("full_real_source_trace")
    if isinstance(full_real_source_trace, dict):
        full_real_source_trace = {
            str(key): str(value).strip() or default
            for key, default in (
                ("smoke_check_script", "tests/run_ci_profile_matrix_full_real_smoke_check.py"),
                ("smoke_check_script_exists", "0"),
                ("smoke_check_selftest_script", "tests/run_ci_profile_matrix_full_real_smoke_check_selftest.py"),
                ("smoke_check_selftest_script_exists", "0"),
            )
            for value in [full_real_source_trace.get(key, default)]
        }
    else:
        full_real_source_trace = build_age5_combined_heavy_full_real_source_trace()
    full_real_source_trace_text = str(
        (doc or {}).get(
            "full_real_source_trace_text",
            build_age5_combined_heavy_full_real_source_trace_text(full_real_source_trace),
        )
    ).strip() or build_age5_combined_heavy_full_real_source_trace_text(full_real_source_trace)
    combined_heavy_child_timeout_sec = (
        str((doc or {}).get("combined_heavy_child_timeout_sec", "0")).strip() or "0"
    )
    combined_heavy_timeout_mode = (
        str((doc or {}).get(AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY, AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED)).strip()
        or AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED
    )
    combined_heavy_timeout_present = (
        str((doc or {}).get("age5_combined_heavy_timeout_present", "0")).strip() or "0"
    )
    combined_heavy_timeout_targets = (
        str((doc or {}).get("age5_combined_heavy_timeout_targets", "-")).strip() or "-"
    )
    lines.append(f"[ci-gate-summary] age5_combined_heavy_child_timeout_sec={combined_heavy_child_timeout_sec}")
    lines.append(f"[ci-gate-summary] age5_combined_heavy_timeout_mode={combined_heavy_timeout_mode}")
    lines.append(f"[ci-gate-summary] age5_combined_heavy_timeout_present={combined_heavy_timeout_present}")
    lines.append(f"[ci-gate-summary] age5_combined_heavy_timeout_targets={combined_heavy_timeout_targets}")
    lines.append(
        f"[ci-gate-summary] age5_full_real_smoke_check_script={full_real_source_trace.get('smoke_check_script', '-')}"
    )
    lines.append(
        "[ci-gate-summary] age5_full_real_smoke_check_script_exists="
        f"{full_real_source_trace.get('smoke_check_script_exists', '0')}"
    )
    lines.append(
        "[ci-gate-summary] age5_full_real_smoke_check_selftest_script="
        f"{full_real_source_trace.get('smoke_check_selftest_script', '-')}"
    )
    lines.append(
        "[ci-gate-summary] age5_full_real_smoke_check_selftest_script_exists="
        f"{full_real_source_trace.get('smoke_check_selftest_script_exists', '0')}"
    )
    lines.append(f"[ci-gate-summary] age5_full_real_source_trace_text={full_real_source_trace_text}")
    lines.append(f"[ci-gate-summary] age4_proof_snapshot_fields_text={age4_proof_snapshot_fields_text}")
    lines.append(f"[ci-gate-summary] age4_proof_snapshot_text={age4_proof_snapshot_text}")
    lines.append(f"[ci-gate-summary] {AGE4_PROOF_GATE_RESULT_SNAPSHOT_TEXT_KEY}={age4_proof_gate_result_snapshot_text}")
    lines.append(f"[ci-gate-summary] {AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY}={age4_proof_gate_result_snapshot_present}")
    lines.append(f"[ci-gate-summary] {AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY}={age4_proof_gate_result_snapshot_parity}")
    lines.append(
        f"[ci-gate-summary] {AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_TEXT_KEY}={age4_proof_final_status_parse_snapshot_text}"
    )
    lines.append(
        f"[ci-gate-summary] {AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY}={age4_proof_final_status_parse_snapshot_present}"
    )
    lines.append(
        f"[ci-gate-summary] {AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY}={age4_proof_final_status_parse_snapshot_parity}"
    )
    for key, default in (
        (AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_ACTIVE_CASES_KEY, "-"),
        (AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_INACTIVE_CASES_KEY, "-"),
        (AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_INDEX_CODES_KEY, "-"),
        (AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (AGE5_FULL_REAL_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (AGE5_FULL_REAL_PROOF_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (AGE5_FULL_REAL_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (AGE5_FULL_REAL_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (AGE5_FULL_REAL_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
    # age5_full_real_lang_runtime_family_contract_selftest_completed_checks=
    # age5_full_real_lang_runtime_family_contract_selftest_total_checks=
    # age5_full_real_lang_runtime_family_contract_selftest_checks_text=
    # age5_full_real_lang_runtime_family_contract_selftest_current_probe=
    # age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe=
    # age5_full_real_lang_runtime_family_contract_selftest_progress_present=
    # age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks=
    # age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks=
    # age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text=
    # age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe=
    # age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe=
    # age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present=
    # age5_full_real_gate0_family_contract_selftest_completed_checks=
    # age5_full_real_gate0_family_contract_selftest_total_checks=
    # age5_full_real_gate0_family_contract_selftest_checks_text=
    # age5_full_real_gate0_family_contract_selftest_current_probe=
    # age5_full_real_gate0_family_contract_selftest_last_completed_probe=
    # age5_full_real_gate0_family_contract_selftest_progress_present=
    # age5_full_real_gate0_surface_family_contract_selftest_completed_checks=
    # age5_full_real_gate0_surface_family_contract_selftest_total_checks=
    # age5_full_real_gate0_surface_family_contract_selftest_checks_text=
    # age5_full_real_gate0_surface_family_contract_selftest_current_probe=
    # age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe=
    # age5_full_real_gate0_surface_family_contract_selftest_progress_present=
    # age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks=
    # age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks=
    # age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text=
    # age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe=
    # age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe=
    # age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present=
    # age5_full_real_gate0_family_transport_contract_selftest_completed_checks=
    # age5_full_real_gate0_family_transport_contract_selftest_total_checks=
    # age5_full_real_gate0_family_transport_contract_selftest_checks_text=
    # age5_full_real_gate0_family_transport_contract_selftest_current_probe=
    # age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe=
    # age5_full_real_gate0_family_transport_contract_selftest_progress_present=
    # age5_full_real_gate0_transport_family_contract_selftest_completed_checks=
    # age5_full_real_gate0_transport_family_contract_selftest_total_checks=
    # age5_full_real_gate0_transport_family_contract_selftest_checks_text=
    # age5_full_real_gate0_transport_family_contract_selftest_current_probe=
    # age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe=
    # age5_full_real_gate0_transport_family_contract_selftest_progress_present=
    # age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks=
    # age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks=
    # age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text=
    # age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe=
    # age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe=
    # age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present=
    # age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks=
    # age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks=
    # age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text=
    # age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe=
    # age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe=
    # age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present=
        (AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (AGE5_FULL_REAL_GATE0_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (
            AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
            "-",
        ),
        (
            AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
            "-",
        ),
        (
            AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
            "-",
        ),
        (
            AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
            "-",
        ),
        (
            AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
            "-",
        ),
        (
            AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
            "0",
        ),
        (AGE5_FULL_REAL_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (
            AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY,
            "-",
        ),
        (
            AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY,
            "-",
        ),
        (
            AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY,
            "-",
        ),
        (
            AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY,
            "-",
        ),
        (
            AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY,
            "-",
        ),
        (
            AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY,
            "0",
        ),
        (AGE5_FULL_REAL_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
        (AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_COMPLETED_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_TOTAL_CHECKS_KEY, "-"),
        (AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CHECKS_TEXT_KEY, "-"),
        (AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_CURRENT_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_LAST_COMPLETED_PROBE_KEY, "-"),
        (AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_PRESENT_KEY, "0"),
    ):
        value = str((doc or {}).get(key, default)).strip() or default
        lines.append(f"[ci-gate-summary] {key}={value}")


def append_age5_policy_summary_lines(lines: list[str], aggregate_report_path: Path) -> None:
    # diagnostics token anchors:
    # age5_policy_age4_proof_snapshot_fields_text=
    # age5_policy_age4_proof_source_snapshot_fields_text=
    # age5_policy_age4_proof_snapshot_text=
    # age5_policy_age4_proof_gate_result_present=
    # age5_policy_age4_proof_gate_result_parity=
    # age5_policy_age4_proof_final_status_parse_present=
    # age5_policy_age4_proof_final_status_parse_parity=
    aggregate_doc = load_payload(aggregate_report_path)
    age5_doc = aggregate_doc.get("age5") if isinstance(aggregate_doc, dict) else None
    if not isinstance(age5_doc, dict):
        age5_doc = {}
    age5_policy_age4_proof_snapshot = build_age4_proof_snapshot()
    defaults = {
        "age5_policy_combined_digest_selftest_default_field_text": "age5_close_digest_selftest_ok=0",
        "age5_policy_combined_digest_selftest_default_field": '{"age5_close_digest_selftest_ok":"0"}',
        "age5_policy_age4_proof_snapshot_fields_text": AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
        "age5_policy_age4_proof_snapshot_text": build_age4_proof_snapshot_text(
            age5_policy_age4_proof_snapshot
        ),
        "age5_policy_age4_proof_source_snapshot_fields_text": AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
        "age5_policy_age4_proof_gate_result_present": "0",
        "age5_policy_age4_proof_gate_result_parity": "0",
        "age5_policy_age4_proof_final_status_parse_present": "0",
        "age5_policy_age4_proof_final_status_parse_parity": "0",
        "age5_combined_heavy_policy_report_path": "-",
        "age5_combined_heavy_policy_report_exists": "0",
        "age5_combined_heavy_policy_text_path": "-",
        "age5_combined_heavy_policy_text_exists": "0",
        "age5_combined_heavy_policy_summary_path": "-",
        "age5_combined_heavy_policy_summary_exists": "0",
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY: "-",
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY: "-",
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY: "-",
        "age5_policy_summary_origin_trace_contract_compact_failure_reason": "-",
    }
    policy_origin_trace = age5_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY)
    if isinstance(policy_origin_trace, dict):
        policy_origin_trace = {
            str(key): str(value).strip() or default
            for key, default in (
                ("report_path", "-"),
                ("report_exists", "0"),
                ("text_path", "-"),
                ("text_exists", "0"),
                ("summary_path", "-"),
                ("summary_exists", "0"),
            )
            for value in [policy_origin_trace.get(key, default)]
        }
    else:
        policy_origin_trace = build_age5_combined_heavy_policy_origin_trace(
            report_path=age5_doc.get("age5_combined_heavy_policy_report_path", "-"),
            report_exists=age5_doc.get("age5_combined_heavy_policy_report_exists", False),
            text_path=age5_doc.get("age5_combined_heavy_policy_text_path", "-"),
            text_exists=age5_doc.get("age5_combined_heavy_policy_text_exists", False),
            summary_path=age5_doc.get("age5_combined_heavy_policy_summary_path", "-"),
            summary_exists=age5_doc.get("age5_combined_heavy_policy_summary_exists", False),
        )
    policy_origin_trace_text = str(
        age5_doc.get(
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY,
            build_age5_combined_heavy_policy_origin_trace_text(policy_origin_trace),
        )
    ).strip() or build_age5_combined_heavy_policy_origin_trace_text(policy_origin_trace)
    field_value = age5_doc.get("age5_policy_combined_digest_selftest_default_field")
    if isinstance(field_value, dict):
        defaults["age5_policy_combined_digest_selftest_default_field"] = json.dumps(
            {str(key): str(value) for key, value in field_value.items()},
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        )
    for key in (
        "age5_policy_combined_digest_selftest_default_field_text",
        "age5_policy_age4_proof_snapshot_fields_text",
        "age5_policy_age4_proof_source_snapshot_fields_text",
        "age5_policy_age4_proof_snapshot_text",
        "age5_combined_heavy_policy_report_path",
        "age5_combined_heavy_policy_text_path",
        "age5_combined_heavy_policy_summary_path",
    ):
        value = str(age5_doc.get(key, defaults[key])).strip() or defaults[key]
        lines.append(f"[ci-gate-summary] {key}={value}")
    for key in (
        "age5_policy_age4_proof_gate_result_present",
        "age5_policy_age4_proof_gate_result_parity",
        "age5_policy_age4_proof_final_status_parse_present",
        "age5_policy_age4_proof_final_status_parse_parity",
    ):
        value = str(age5_doc.get(key, defaults[key])).strip() or defaults[key]
        lines.append(f"[ci-gate-summary] {key}={value}")
    for key in (
        "age5_combined_heavy_policy_report_exists",
        "age5_combined_heavy_policy_text_exists",
        "age5_combined_heavy_policy_summary_exists",
    ):
        value = "1" if bool(age5_doc.get(key, False)) else "0"
        lines.append(f"[ci-gate-summary] {key}={value}")
    lines.append(
        "[ci-gate-summary] age5_policy_combined_digest_selftest_default_field="
        + defaults["age5_policy_combined_digest_selftest_default_field"]
    )
    lines.append(
        f"[ci-gate-summary] {AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}={policy_origin_trace_text}"
    )
    lines.append(
        "[ci-gate-summary] "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY}="
        + json.dumps(policy_origin_trace, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
    )
    lines.append(
        f"[ci-gate-summary] {AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}="
        f"{str(age5_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY, '-')).strip() or '-'}"
    )
    lines.append(
        f"[ci-gate-summary] {AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}="
        f"{str(age5_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY, '-')).strip() or '-'}"
    )
    lines.append(
        f"[ci-gate-summary] {AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}="
        f"{str(age5_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY, '-')).strip() or '-'}"
    )
    lines.append(
        "[ci-gate-summary] age5_policy_summary_origin_trace_contract_compact_failure_reason="
        f"{str(age5_doc.get('age5_policy_summary_origin_trace_contract_compact_failure_reason', '-')).strip() or '-'}"
    )


def load_age4_proof_summary_snapshot(aggregate_report_path: Path) -> dict[str, str]:
    snapshot = {
        "age4_proof_ok": "0",
        "age4_proof_failed_criteria": "-1",
        "age4_proof_failed_preview": "-",
        "age4_proof_summary_hash": "-",
    }
    aggregate_doc = load_payload(aggregate_report_path)
    if not isinstance(aggregate_doc, dict):
        return snapshot
    age4_doc = aggregate_doc.get("age4")
    if not isinstance(age4_doc, dict):
        return snapshot
    snapshot["age4_proof_ok"] = "1" if bool(age4_doc.get("proof_artifact_ok", False)) else "0"
    failed = age4_doc.get("proof_artifact_failed_criteria")
    snapshot["age4_proof_failed_preview"] = (
        str(age4_doc.get("proof_artifact_failed_preview", "")).strip()
        or format_age4_proof_failed_preview(failed)
    )
    if isinstance(failed, list):
        snapshot["age4_proof_failed_criteria"] = str(len(failed))
    else:
        try:
            snapshot["age4_proof_failed_criteria"] = str(int(failed))
        except Exception:
            pass
    summary_hash = str(age4_doc.get("proof_artifact_summary_hash", "")).strip()
    if summary_hash:
        snapshot["age4_proof_summary_hash"] = summary_hash
    return snapshot


def append_age4_proof_summary_lines(lines: list[str], aggregate_report_path: Path) -> None:
    snapshot = load_age4_proof_summary_snapshot(aggregate_report_path)
    for key in AGE4_PROOF_SUMMARY_KEYS:
        lines.append(f"[ci-gate-summary] {key}={snapshot[key]}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Run seamgrim+OI close gates and enforce aggregate result")
    parser.add_argument(
        "--report-dir",
        default=default_report_dir(),
        help="directory for seamgrim/oi/aggregate reports",
    )
    parser.add_argument(
        "--fast-fail",
        action="store_true",
        help="stop immediately when seamgrim or OI close gate fails",
    )
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument(
        "--core-tests",
        action="store_true",
        help="run core test steps inside aggregate gate (full mode)",
    )
    mode.add_argument(
        "--skip-core-tests",
        action="store_true",
        help="skip core test steps inside aggregate gate (fast mode, default)",
    )
    parser.add_argument(
        "--report-prefix",
        default="",
        help="optional prefix for report file names (safe chars only)",
    )
    parser.add_argument(
        "--ci-sanity-profile",
        choices=("full", "core_lang", "seamgrim"),
        default="full",
        help="profile passed to ci sanity gate and sync-readiness contract checks",
    )
    parser.add_argument(
        "--profile-matrix-selftest-real-profiles",
        default="",
        help="optional real profile subset passthrough for ci_profile_matrix_gate_selftest (e.g. core_lang,full)",
    )
    parser.add_argument(
        "--profile-matrix-selftest-dry",
        action="store_true",
        help="force ci_profile_matrix_gate_selftest dry mode",
    )
    parser.add_argument(
        "--profile-matrix-selftest-quick",
        action="store_true",
        help="force ci_profile_matrix_gate_selftest quick mode",
    )
    parser.add_argument(
        "--profile-matrix-selftest-full-aggregate-gates",
        action="store_true",
        help="pass --matrix-full-aggregate-gates to ci_profile_matrix_gate_selftest",
    )
    parser.add_argument(
        "--profile-matrix-selftest-with-profile-matrix-full-real-smoke",
        action="store_true",
        help="pass --matrix-with-profile-matrix-full-real-smoke to ci_profile_matrix_gate_selftest",
    )
    parser.add_argument(
        "--auto-prefix-env",
        default="",
        help="comma-separated env keys used as prefix source when --report-prefix is empty",
    )
    parser.add_argument(
        "--clean-prefixed-reports",
        action="store_true",
        help="remove existing prefixed report files before execution",
    )
    parser.add_argument(
        "--clean-dry-run",
        action="store_true",
        help="with --clean-prefixed-reports, print targets only (no delete)",
    )
    parser.add_argument(
        "--print-report-paths",
        action="store_true",
        help="print resolved report file paths before running checks",
    )
    parser.add_argument(
        "--quiet-success-logs",
        action="store_true",
        help="suppress child stdout/stderr for successful steps (failed steps remain verbose)",
    )
    parser.add_argument(
        "--compact-step-logs",
        action="store_true",
        help="print compact step start/exit lines (full command shown only on failures)",
    )
    parser.add_argument(
        "--step-log-dir",
        default="",
        help="optional directory to write per-step stdout/stderr logs",
    )
    parser.add_argument(
        "--step-log-failed-only",
        action="store_true",
        help="when set with --step-log-dir, write step stdout/stderr files only for failed steps",
    )
    parser.add_argument(
        "--full-pass-summary",
        action="store_true",
        help="print full PASS summary block (default prints compact PASS summary)",
    )
    parser.add_argument(
        "--report-index-json",
        default="",
        help="optional path to write run index json (step return codes + resolved report paths)",
    )
    parser.add_argument(
        "--report-index-base-name",
        default="ci_gate_report_index.detjson",
        help="base file name for index report when --report-index-json is empty",
    )
    parser.add_argument(
        "--summary-txt",
        default="",
        help="optional path to write ci gate summary text",
    )
    parser.add_argument(
        "--summary-base-name",
        default="ci_gate_summary.txt",
        help="base file name for summary report when --summary-txt is empty",
    )
    parser.add_argument(
        "--age3-summary-md",
        default="",
        help="optional path to write age3 close markdown summary",
    )
    parser.add_argument(
        "--age3-summary-base-name",
        default="age3_close_summary.md",
        help="base file name for age3 close markdown summary when --age3-summary-md is empty",
    )
    parser.add_argument(
        "--age3-status-json",
        default="",
        help="optional path to write age3 close status json",
    )
    parser.add_argument(
        "--age3-status-base-name",
        default="age3_close_status.detjson",
        help="base file name for age3 close status when --age3-status-json is empty",
    )
    parser.add_argument(
        "--age3-status-line-txt",
        default="",
        help="optional path to write one-line age3 close status text",
    )
    parser.add_argument(
        "--age3-status-line-base-name",
        default="age3_close_status_line.txt",
        help="base file name for one-line age3 status when --age3-status-line-txt is empty",
    )
    parser.add_argument(
        "--age3-badge-json",
        default="",
        help="optional path to write age3 close badge json",
    )
    parser.add_argument(
        "--age3-badge-base-name",
        default="age3_close_badge.detjson",
        help="base file name for age3 close badge json when --age3-badge-json is empty",
    )
    parser.add_argument(
        "--aggregate-status-line-txt",
        default="",
        help="optional path to write one-line aggregate gate status text",
    )
    parser.add_argument(
        "--aggregate-status-line-base-name",
        default="ci_aggregate_status_line.txt",
        help="base file name for aggregate gate one-line status when --aggregate-status-line-txt is empty",
    )
    parser.add_argument(
        "--aggregate-status-parse-json",
        default="",
        help="optional path to write parsed aggregate status-line json",
    )
    parser.add_argument(
        "--aggregate-status-parse-base-name",
        default="ci_aggregate_status_line_parse.detjson",
        help="base file name for aggregate status-line parse json when --aggregate-status-parse-json is empty",
    )
    parser.add_argument(
        "--final-status-line-txt",
        default="",
        help="optional path to write final CI gate one-line status text",
    )
    parser.add_argument(
        "--final-status-line-base-name",
        default="ci_gate_final_status_line.txt",
        help="base file name for final CI gate one-line status when --final-status-line-txt is empty",
    )
    parser.add_argument(
        "--final-status-parse-json",
        default="",
        help="optional path to write parsed final status-line json",
    )
    parser.add_argument(
        "--final-status-parse-base-name",
        default="ci_gate_final_status_line_parse.detjson",
        help="base file name for final status-line parse json when --final-status-parse-json is empty",
    )
    parser.add_argument(
        "--summary-line-txt",
        default="",
        help="optional path to write single-line ci gate summary text",
    )
    parser.add_argument(
        "--summary-line-base-name",
        default="ci_gate_summary_line.txt",
        help="base file name for single-line ci gate summary when --summary-line-txt is empty",
    )
    parser.add_argument(
        "--ci-gate-result-json",
        default="",
        help="optional path to write compact CI gate result json",
    )
    parser.add_argument(
        "--ci-gate-result-base-name",
        default="ci_gate_result.detjson",
        help="base file name for compact CI gate result json when --ci-gate-result-json is empty",
    )
    parser.add_argument(
        "--ci-gate-result-parse-json",
        default="",
        help="optional path to write parsed ci gate result json",
    )
    parser.add_argument(
        "--ci-gate-result-parse-base-name",
        default="ci_gate_result_parse.detjson",
        help="base file name for parsed ci gate result json when --ci-gate-result-parse-json is empty",
    )
    parser.add_argument(
        "--ci-gate-result-line-txt",
        default="",
        help="optional path to write one-line ci gate result text",
    )
    parser.add_argument(
        "--ci-gate-result-line-base-name",
        default="ci_gate_result_line.txt",
        help="base file name for one-line ci gate result text when --ci-gate-result-line-txt is empty",
    )
    parser.add_argument(
        "--ci-gate-badge-json",
        default="",
        help="optional path to write ci gate badge json",
    )
    parser.add_argument(
        "--ci-gate-badge-base-name",
        default="ci_gate_badge.detjson",
        help="base file name for ci gate badge json when --ci-gate-badge-json is empty",
    )
    parser.add_argument(
        "--ci-fail-brief-txt",
        default="",
        help="optional path hint for ci failure brief one-line txt (external emitter output)",
    )
    parser.add_argument(
        "--ci-fail-brief-base-name",
        default="ci_fail_brief.txt",
        help="base file name for ci failure brief when --ci-fail-brief-txt is empty",
    )
    parser.add_argument(
        "--ci-fail-triage-json",
        default="",
        help="optional path hint for ci failure triage json (external emitter output)",
    )
    parser.add_argument(
        "--ci-fail-triage-base-name",
        default="ci_fail_triage.detjson",
        help="base file name for ci failure triage json when --ci-fail-triage-json is empty",
    )
    parser.add_argument(
        "--backup-hygiene",
        action="store_true",
        help="run lesson backup hygiene step before seamgrim gate (move *.bak.ddn + verify empty)",
    )
    parser.add_argument(
        "--require-fixed64-3way",
        action="store_true",
        help="require fixed64 3way(windows/linux/darwin) gate; if darwin report missing, fail",
    )
    parser.add_argument(
        "--fixed64-threeway-max-report-age-minutes",
        type=float,
        default=0.0,
        help="pass through max report age(minutes) to fixed64 3way gate; 0 disables freshness check",
    )
    parser.add_argument(
        "--require-age5",
        action="store_true",
        help="require age5 close report to pass at aggregate combine stage",
    )
    parser.add_argument(
        "--contract-only-aggregate",
        action="store_true",
        help="skip seamgrim/age/oi heavy close runs and use stub pass reports for contract-focused aggregate validation",
    )
    parser.add_argument(
        "--with-runtime-5min",
        action="store_true",
        help="pass through runtime 5min seamgrim check to run_seamgrim_ci_gate.py",
    )
    parser.add_argument(
        "--runtime-5min-skip-seed-cli",
        action="store_true",
        help="with --with-runtime-5min, skip seed teul-cli runs in runtime scenario",
    )
    parser.add_argument(
        "--runtime-5min-skip-ui-common",
        action="store_true",
        help="with --with-runtime-5min, skip ui common/aux ui runners in runtime scenario",
    )
    parser.add_argument(
        "--runtime-5min-skip-showcase-check",
        action="store_true",
        help="with --with-runtime-5min, skip pendulum+tetris showcase check in runtime scenario",
    )
    parser.add_argument(
        "--runtime-5min-showcase-smoke",
        action="store_true",
        help="with --with-runtime-5min, run showcase check with non-dry smoke",
    )
    parser.add_argument(
        "--runtime-5min-showcase-smoke-madi-pendulum",
        type=int,
        default=20,
        help="with --runtime-5min-showcase-smoke, pendulum madi for showcase smoke",
    )
    parser.add_argument(
        "--runtime-5min-showcase-smoke-madi-tetris",
        type=int,
        default=20,
        help="with --runtime-5min-showcase-smoke, tetris madi for showcase smoke",
    )
    parser.add_argument(
        "--with-5min-checklist",
        action="store_true",
        help="(deprecated) 5-minute checklist is enabled by default; use --skip-5min-checklist to disable",
    )
    parser.add_argument(
        "--skip-5min-checklist",
        action="store_true",
        help="disable 5-minute checklist wrapper passthrough to run_seamgrim_ci_gate.py",
    )
    parser.add_argument(
        "--run-report-index-latest-smoke",
        action="store_true",
        help="execute latest report-index smoke check (default: record skipped marker only)",
    )
    parser.add_argument(
        "--skip-fail-and-exit-contract-selftest",
        action="store_true",
        help="skip fail_and_exit contract selftest runner (records skipped step marker)",
    )
    parser.add_argument(
        "--checklist-skip-seed-cli",
        action="store_true",
        help="when checklist is enabled, skip seed teul-cli runs in checklist runtime",
    )
    parser.add_argument(
        "--checklist-skip-ui-common",
        action="store_true",
        help="when checklist is enabled, skip ui common runner in checklist runtime",
    )
    parser.add_argument(
        "--browse-selection-strict",
        action="store_true",
        help="pass through browse selection strict report validation to run_seamgrim_ci_gate.py",
    )
    parser.add_argument(
        "--lesson-warning-require-zero",
        action="store_true",
        help="require seamgrim lesson warning token count to be zero",
    )
    args = parser.parse_args()
    if args.skip_5min_checklist and args.with_5min_checklist:
        print("[ci-gate] --with-5min-checklist and --skip-5min-checklist cannot be used together", file=sys.stderr)
        return 2
    include_5min_checklist = not bool(args.skip_5min_checklist)

    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    report_dir = Path(args.report_dir)
    report_dir.mkdir(parents=True, exist_ok=True)
    prefix_source = "arg"
    raw_prefix = args.report_prefix.strip()
    if not raw_prefix and args.auto_prefix_env.strip():
        for env_key in [item.strip() for item in args.auto_prefix_env.split(",") if item.strip()]:
            value = os.environ.get(env_key, "").strip()
            if value:
                raw_prefix = value
                prefix_source = f"env:{env_key}"
                break
    prefix = sanitize_report_prefix(raw_prefix)
    if raw_prefix and not prefix:
        print("[ci-gate] invalid --report-prefix after sanitize", file=sys.stderr)
        return 2
    if prefix:
        print(f"[ci-gate] report_prefix={prefix} source={prefix_source}")
    age5_close_digest_selftest_rc = 0
    if args.clean_prefixed_reports and not prefix:
        print("[ci-gate] --clean-prefixed-reports requires non-empty prefix", file=sys.stderr)
        return 2
    explicit_step_log_dir = args.step_log_dir.strip()
    step_log_dir = Path(explicit_step_log_dir) if explicit_step_log_dir else None
    if step_log_dir is not None:
        step_log_dir.mkdir(parents=True, exist_ok=True)
    seamgrim_base_name = "seamgrim_ci_gate_report.json"
    seamgrim_ui_age3_base_name = "seamgrim_ui_age3_gate_report.detjson"
    seamgrim_phase3_cleanup_base_name = "seamgrim_phase3_cleanup_gate_report.detjson"
    seamgrim_browse_selection_base_name = "seamgrim_browse_selection_flow_report.detjson"
    seamgrim_runtime_5min_base_name = "seamgrim_runtime_5min_report.detjson"
    seamgrim_runtime_5min_browse_selection_base_name = "seamgrim_runtime_5min_browse_selection_flow_report.detjson"
    seamgrim_5min_checklist_base_name = "seamgrim_5min_checklist_report.detjson"
    seamgrim_lesson_warning_tokens_base_name = "seamgrim_lesson_warning_tokens_report.detjson"
    seamgrim_control_exposure_failures_base_name = "seamgrim_control_exposure_failures.detjson"
    seamgrim_rewrite_overlay_quality_base_name = "seamgrim_rewrite_overlay_quality_report.detjson"
    seamgrim_wasm_cli_diag_parity_base_name = "seamgrim_wasm_cli_diag_parity_report.detjson"
    age2_close_base_name = "age2_close_report.detjson"
    age3_close_base_name = "age3_close_report.detjson"
    age4_close_base_name = "age4_close_report.detjson"
    age5_close_base_name = "age5_close_report.detjson"
    age4_pack_base_name = "age4_close_pack_report.detjson"
    oi_close_base_name = "oi405_406_close_report.detjson"
    oi_pack_base_name = "oi405_406_pack_report.detjson"
    aggregate_base_name = "ci_aggregate_report.detjson"
    ci_sanity_gate_base_name = "ci_sanity_gate.detjson"
    ci_sync_readiness_base_name = "ci_sync_readiness.detjson"
    backup_hygiene_move_base_name = "seamgrim_backup_hygiene_move.detjson"
    backup_hygiene_verify_base_name = "seamgrim_backup_hygiene_verify.detjson"
    fixed64_threeway_inputs_base_name = "fixed64_threeway_inputs.detjson"
    fixed64_threeway_gate_base_name = "fixed64_cross_platform_threeway_gate.detjson"
    ci_profile_matrix_gate_selftest_base_name = "ci_profile_matrix_gate_selftest.detjson"
    report_base_names = [
        seamgrim_base_name,
        seamgrim_ui_age3_base_name,
        seamgrim_phase3_cleanup_base_name,
        seamgrim_browse_selection_base_name,
        seamgrim_runtime_5min_base_name,
        seamgrim_runtime_5min_browse_selection_base_name,
        seamgrim_5min_checklist_base_name,
        seamgrim_lesson_warning_tokens_base_name,
        seamgrim_control_exposure_failures_base_name,
        seamgrim_rewrite_overlay_quality_base_name,
        seamgrim_wasm_cli_diag_parity_base_name,
        age2_close_base_name,
        age3_close_base_name,
        age4_close_base_name,
        age5_close_base_name,
        age4_pack_base_name,
        backup_hygiene_move_base_name,
        backup_hygiene_verify_base_name,
        fixed64_threeway_inputs_base_name,
        fixed64_threeway_gate_base_name,
        ci_profile_matrix_gate_selftest_base_name,
        args.age3_summary_base_name,
        args.age3_status_base_name,
        args.age3_status_line_base_name,
        args.age3_badge_base_name,
        args.aggregate_status_line_base_name,
        args.aggregate_status_parse_base_name,
        args.final_status_line_base_name,
        args.final_status_parse_base_name,
        args.ci_gate_result_base_name,
        args.ci_gate_result_parse_base_name,
        args.ci_gate_result_line_base_name,
        args.ci_gate_badge_base_name,
        args.ci_fail_brief_base_name,
        args.ci_fail_triage_base_name,
        oi_close_base_name,
        oi_pack_base_name,
        aggregate_base_name,
        ci_sanity_gate_base_name,
        ci_sync_readiness_base_name,
        args.report_index_base_name,
        args.summary_base_name,
        args.summary_line_base_name,
    ]
    if args.clean_prefixed_reports:
        removed_count = cleanup_prefixed_reports(
            report_dir,
            prefix,
            report_base_names,
            dry_run=bool(args.clean_dry_run),
        )
        if step_log_dir is not None:
            removed_count += cleanup_prefixed_step_logs(
                step_log_dir,
                prefix,
                dry_run=bool(args.clean_dry_run),
            )
        print(f"[ci-gate] clean done count={removed_count} dry_run={int(bool(args.clean_dry_run))}")

    seamgrim_report = report_path(report_dir, seamgrim_base_name, prefix)
    seamgrim_ui_age3_report = report_path(report_dir, seamgrim_ui_age3_base_name, prefix)
    seamgrim_phase3_cleanup_report = report_path(report_dir, seamgrim_phase3_cleanup_base_name, prefix)
    seamgrim_browse_selection_report = report_path(report_dir, seamgrim_browse_selection_base_name, prefix)
    seamgrim_runtime_5min_report = report_path(report_dir, seamgrim_runtime_5min_base_name, prefix)
    seamgrim_runtime_5min_browse_selection_report = report_path(
        report_dir,
        seamgrim_runtime_5min_browse_selection_base_name,
        prefix,
    )
    seamgrim_5min_checklist_report = report_path(report_dir, seamgrim_5min_checklist_base_name, prefix)
    seamgrim_lesson_warning_tokens_report = report_path(
        report_dir,
        seamgrim_lesson_warning_tokens_base_name,
        prefix,
    )
    seamgrim_control_exposure_failures_report = report_path(
        report_dir,
        seamgrim_control_exposure_failures_base_name,
        prefix,
    )
    seamgrim_rewrite_overlay_quality_report = report_path(
        report_dir,
        seamgrim_rewrite_overlay_quality_base_name,
        prefix,
    )
    seamgrim_wasm_cli_diag_parity_report = report_path(
        report_dir,
        seamgrim_wasm_cli_diag_parity_base_name,
        prefix,
    )
    age2_close_report = report_path(report_dir, age2_close_base_name, prefix)
    age3_close_report = report_path(report_dir, age3_close_base_name, prefix)
    age4_close_report = report_path(report_dir, age4_close_base_name, prefix)
    age5_close_report = report_path(report_dir, age5_close_base_name, prefix)
    age4_pack_report = report_path(report_dir, age4_pack_base_name, prefix)
    backup_hygiene_move_report = report_path(report_dir, backup_hygiene_move_base_name, prefix)
    backup_hygiene_verify_report = report_path(report_dir, backup_hygiene_verify_base_name, prefix)
    fixed64_threeway_inputs_report = report_path(report_dir, fixed64_threeway_inputs_base_name, prefix)
    fixed64_threeway_gate_report = report_path(report_dir, fixed64_threeway_gate_base_name, prefix)
    ci_profile_matrix_gate_selftest_report = report_path(report_dir, ci_profile_matrix_gate_selftest_base_name, prefix)
    explicit_age3_summary_md = args.age3_summary_md.strip()
    if explicit_age3_summary_md:
        age3_close_summary_md = Path(explicit_age3_summary_md)
    else:
        age3_close_summary_md = report_path(report_dir, args.age3_summary_base_name, prefix)
    explicit_age3_status_json = args.age3_status_json.strip()
    if explicit_age3_status_json:
        age3_close_status_json = Path(explicit_age3_status_json)
    else:
        age3_close_status_json = report_path(report_dir, args.age3_status_base_name, prefix)
    explicit_age3_status_line_txt = args.age3_status_line_txt.strip()
    if explicit_age3_status_line_txt:
        age3_close_status_line = Path(explicit_age3_status_line_txt)
    else:
        age3_close_status_line = report_path(report_dir, args.age3_status_line_base_name, prefix)
    explicit_age3_badge_json = args.age3_badge_json.strip()
    if explicit_age3_badge_json:
        age3_close_badge_json = Path(explicit_age3_badge_json)
    else:
        age3_close_badge_json = report_path(report_dir, args.age3_badge_base_name, prefix)
    explicit_aggregate_status_line_txt = args.aggregate_status_line_txt.strip()
    if explicit_aggregate_status_line_txt:
        aggregate_status_line = Path(explicit_aggregate_status_line_txt)
    else:
        aggregate_status_line = report_path(report_dir, args.aggregate_status_line_base_name, prefix)
    explicit_aggregate_status_parse_json = args.aggregate_status_parse_json.strip()
    if explicit_aggregate_status_parse_json:
        aggregate_status_parse_json = Path(explicit_aggregate_status_parse_json)
    else:
        aggregate_status_parse_json = report_path(report_dir, args.aggregate_status_parse_base_name, prefix)
    explicit_final_status_line_txt = args.final_status_line_txt.strip()
    if explicit_final_status_line_txt:
        final_status_line = Path(explicit_final_status_line_txt)
    else:
        final_status_line = report_path(report_dir, args.final_status_line_base_name, prefix)
    explicit_final_status_parse_json = args.final_status_parse_json.strip()
    if explicit_final_status_parse_json:
        final_status_parse_json = Path(explicit_final_status_parse_json)
    else:
        final_status_parse_json = report_path(report_dir, args.final_status_parse_base_name, prefix)
    oi_report = report_path(report_dir, oi_close_base_name, prefix)
    oi_pack_report = report_path(report_dir, oi_pack_base_name, prefix)
    aggregate_report = report_path(report_dir, aggregate_base_name, prefix)
    ci_sanity_gate_report = report_path(report_dir, ci_sanity_gate_base_name, prefix)
    ci_sync_readiness_report = report_path(report_dir, ci_sync_readiness_base_name, prefix)
    explicit_index_json = args.report_index_json.strip()
    if explicit_index_json:
        index_report_path = Path(explicit_index_json)
    else:
        index_report_path = report_path(report_dir, args.report_index_base_name, prefix)
    explicit_summary_txt = args.summary_txt.strip()
    if explicit_summary_txt:
        summary_path = Path(explicit_summary_txt)
    else:
        summary_path = report_path(report_dir, args.summary_base_name, prefix)
    explicit_summary_line_txt = args.summary_line_txt.strip()
    if explicit_summary_line_txt:
        summary_line_path = Path(explicit_summary_line_txt)
    else:
        summary_line_path = report_path(report_dir, args.summary_line_base_name, prefix)
    explicit_ci_gate_result_json = args.ci_gate_result_json.strip()
    if explicit_ci_gate_result_json:
        ci_gate_result_json = Path(explicit_ci_gate_result_json)
    else:
        ci_gate_result_json = report_path(report_dir, args.ci_gate_result_base_name, prefix)
    explicit_ci_gate_result_parse_json = args.ci_gate_result_parse_json.strip()
    if explicit_ci_gate_result_parse_json:
        ci_gate_result_parse_json = Path(explicit_ci_gate_result_parse_json)
    else:
        ci_gate_result_parse_json = report_path(report_dir, args.ci_gate_result_parse_base_name, prefix)
    explicit_ci_gate_result_line_txt = args.ci_gate_result_line_txt.strip()
    if explicit_ci_gate_result_line_txt:
        ci_gate_result_line_path = Path(explicit_ci_gate_result_line_txt)
    else:
        ci_gate_result_line_path = report_path(report_dir, args.ci_gate_result_line_base_name, prefix)
    explicit_ci_gate_badge_json = args.ci_gate_badge_json.strip()
    if explicit_ci_gate_badge_json:
        ci_gate_badge_json = Path(explicit_ci_gate_badge_json)
    else:
        ci_gate_badge_json = report_path(report_dir, args.ci_gate_badge_base_name, prefix)
    explicit_ci_fail_brief_txt = args.ci_fail_brief_txt.strip()
    if explicit_ci_fail_brief_txt:
        ci_fail_brief_txt = Path(explicit_ci_fail_brief_txt)
    else:
        ci_fail_brief_txt = report_path(report_dir, args.ci_fail_brief_base_name, prefix)
    explicit_ci_fail_triage_json = args.ci_fail_triage_json.strip()
    if explicit_ci_fail_triage_json:
        ci_fail_triage_json = Path(explicit_ci_fail_triage_json)
    else:
        ci_fail_triage_json = report_path(report_dir, args.ci_fail_triage_base_name, prefix)

    def ensure_fixed64_threeway_inputs_placeholder() -> None:
        if fixed64_threeway_inputs_report.exists():
            return
        fixed64_threeway_inputs_report.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "schema": "ddn.fixed64.threeway_inputs.v1",
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "ok": False,
            "status": "missing",
            "reason": "not_run_yet",
            "selected_source": "",
            "target_report": str(
                report_path(
                    report_dir,
                    "fixed64_cross_platform_probe_darwin.detjson",
                    prefix,
                ).resolve()
            ),
            "invalid_hits": [],
            "candidates": [],
        }
        fixed64_threeway_inputs_report.write_text(
            json.dumps(payload, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )

    ensure_fixed64_threeway_inputs_placeholder()

    control_exposure_snapshot = write_control_exposure_failure_report(
        seamgrim_control_exposure_failures_report,
        seamgrim_report,
    )
    if args.print_report_paths:
        print_report_paths(
            seamgrim_report,
            seamgrim_ui_age3_report,
            seamgrim_phase3_cleanup_report,
            seamgrim_browse_selection_report,
            seamgrim_runtime_5min_report,
            seamgrim_runtime_5min_browse_selection_report,
            seamgrim_5min_checklist_report,
            seamgrim_lesson_warning_tokens_report,
            seamgrim_control_exposure_failures_report,
            seamgrim_rewrite_overlay_quality_report,
            seamgrim_wasm_cli_diag_parity_report,
            age3_close_report,
            age4_close_report,
            age5_close_report,
            age4_pack_report,
            age3_close_summary_md,
            age3_close_status_json,
            age3_close_status_line,
            age3_close_badge_json,
            aggregate_status_line,
            aggregate_status_parse_json,
            final_status_line,
            final_status_parse_json,
            summary_line_path,
            ci_gate_result_json,
            ci_gate_result_parse_json,
            ci_gate_result_line_path,
            ci_gate_badge_json,
            ci_fail_brief_txt,
            ci_fail_triage_json,
            oi_report,
            oi_pack_report,
            aggregate_report,
            ci_profile_matrix_gate_selftest_report,
            ci_sanity_gate_report,
            ci_sync_readiness_report,
            summary_path,
        )
        print(f" - index={index_report_path}")
        if step_log_dir is not None:
            print(f" - step_log_dir={step_log_dir}")
            print(f" - step_log_failed_only={int(bool(args.step_log_failed_only))}")
        print(f" - backup_hygiene_move={backup_hygiene_move_report}")
        print(f" - backup_hygiene_verify={backup_hygiene_verify_report}")
        print(f" - fixed64_threeway_inputs={fixed64_threeway_inputs_report}")
        print(f" - fixed64_threeway_gate={fixed64_threeway_gate_report}")
    run_core_tests = bool(args.core_tests)
    steps_log: list[dict[str, object]] = []

    def step_log_paths(name: str) -> tuple[Path | None, Path | None]:
        if step_log_dir is None:
            return None, None
        safe_name = sanitize_step_name(name)
        base = f"ci_gate_step_{safe_name}"
        stdout_path = report_path(step_log_dir, f"{base}.stdout.txt", prefix)
        stderr_path = report_path(step_log_dir, f"{base}.stderr.txt", prefix)
        return stdout_path, stderr_path

    def run_and_record(name: str, cmd: list[str]) -> int:
        stdout_log_path, stderr_log_path = step_log_paths(name)
        step_result = run_step(
            root,
            name,
            cmd,
            quiet_success_logs=bool(args.quiet_success_logs),
            compact_step_logs=bool(args.compact_step_logs),
            step_log_failed_only=bool(args.step_log_failed_only),
            stdout_log_path=stdout_log_path,
            stderr_log_path=stderr_log_path,
        )
        rc = int(step_result.get("returncode", 127))
        record = {
            "name": name,
            "returncode": rc,
            "cmd": cmd,
            "ok": rc == 0,
            "stdout_line_count": int(step_result.get("stdout_line_count", 0)),
            "stderr_line_count": int(step_result.get("stderr_line_count", 0)),
            "stdout_log_path": str(step_result.get("stdout_log_path", "")).strip(),
            "stderr_log_path": str(step_result.get("stderr_log_path", "")).strip(),
        }
        for idx, existing in enumerate(steps_log):
            if str(existing.get("name", "")).strip() == name:
                steps_log[idx] = record
                break
        else:
            steps_log.append(record)
        return rc

    def discard_step_log(name: str) -> None:
        if steps_log and str(steps_log[-1].get("name", "")).strip() == name:
            steps_log.pop()

    def find_latest_failed_step_record() -> dict[str, object] | None:
        for row in reversed(steps_log):
            if not bool(row.get("ok", False)):
                return row
        return None

    def find_step_record(name: str) -> dict[str, object] | None:
        target = str(name).strip()
        if not target:
            return None
        for row in steps_log:
            if str(row.get("name", "")).strip() == target:
                return row
        return None

    def run_step_if_missing(name: str, runner) -> int:
        existing = find_step_record(name)
        if existing is not None:
            rc = int(existing.get("returncode", 127))
            print(f"[ci-gate] step={name} reuse_existing=1 rc={rc}")
            return rc
        return int(runner())

    def print_fast_fail_step_detail(expected_rc: int) -> None:
        row = find_latest_failed_step_record()
        if row is None:
            print(
                "[ci-gate] fast-fail-step-detail name=- rc=- cmd=-",
                file=sys.stderr,
            )
            print(
                "[ci-gate] fast-fail-step-logs name=- stdout=- stderr=-",
                file=sys.stderr,
            )
            return
        name = str(row.get("name", "")).strip() or "-"
        rc_text = str(row.get("returncode", "")).strip() or "-"
        cmd_value = row.get("cmd", [])
        if isinstance(cmd_value, list):
            cmd_text = " ".join(str(part) for part in cmd_value)
        else:
            cmd_text = str(cmd_value)
        cmd_text = clip_line(cmd_text.strip() or "-", 220)
        stdout_log = str(row.get("stdout_log_path", "")).strip() or "-"
        stderr_log = str(row.get("stderr_log_path", "")).strip() or "-"
        if rc_text != str(expected_rc):
            print(
                f"[ci-gate] fast-fail-step-rc-mismatch expected={expected_rc} actual={rc_text}",
                file=sys.stderr,
            )
        print(
            f"[ci-gate] fast-fail-step-detail name={name} rc={rc_text} cmd={cmd_text}",
            file=sys.stderr,
        )
        print(
            f"[ci-gate] fast-fail-step-logs name={name} stdout={stdout_log} stderr={stderr_log}",
            file=sys.stderr,
        )

    def write_index(overall_ok: bool, announce: bool = True) -> None:
        index_path = index_report_path
        index_path.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "schema": "ddn.ci.aggregate_gate.index.v1",
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "report_prefix": prefix,
            "report_prefix_source": prefix_source if prefix else "",
            "report_dir": str(report_dir),
            "ci_sanity_profile": args.ci_sanity_profile,
            "step_log_dir": str(step_log_dir) if step_log_dir is not None else "",
            "step_log_failed_only": bool(args.step_log_failed_only),
            "reports": {
                "seamgrim": str(seamgrim_report),
                "seamgrim_ui_age3": str(seamgrim_ui_age3_report),
                "seamgrim_phase3_cleanup": str(seamgrim_phase3_cleanup_report),
                "seamgrim_browse_selection": str(seamgrim_browse_selection_report),
                "seamgrim_runtime_5min": str(seamgrim_runtime_5min_report),
                "seamgrim_runtime_5min_browse_selection": str(seamgrim_runtime_5min_browse_selection_report),
                "seamgrim_5min_checklist": str(seamgrim_5min_checklist_report),
                "seamgrim_lesson_warning_tokens": str(seamgrim_lesson_warning_tokens_report),
                "seamgrim_control_exposure_failures": str(seamgrim_control_exposure_failures_report),
                "seamgrim_rewrite_overlay_quality": str(seamgrim_rewrite_overlay_quality_report),
                "seamgrim_wasm_cli_diag_parity": str(seamgrim_wasm_cli_diag_parity_report),
                "age2_close": str(age2_close_report),
                "age3_close": str(age3_close_report),
                "age4_close": str(age4_close_report),
                "age5_close": str(age5_close_report),
                "age4_pack": str(age4_pack_report),
                "backup_hygiene_move": str(backup_hygiene_move_report),
                "backup_hygiene_verify": str(backup_hygiene_verify_report),
                "fixed64_threeway_inputs": str(fixed64_threeway_inputs_report),
                "fixed64_threeway_gate": str(fixed64_threeway_gate_report),
                "age3_close_summary_md": str(age3_close_summary_md),
                "age3_close_status_json": str(age3_close_status_json),
                "age3_close_status_line": str(age3_close_status_line),
                "age3_close_badge_json": str(age3_close_badge_json),
                "aggregate_status_line": str(aggregate_status_line),
                "aggregate_status_parse_json": str(aggregate_status_parse_json),
                "final_status_line": str(final_status_line),
                "final_status_parse_json": str(final_status_parse_json),
                "summary": str(summary_path),
                "summary_line": str(summary_line_path),
                "ci_gate_result_json": str(ci_gate_result_json),
                "ci_gate_result_parse_json": str(ci_gate_result_parse_json),
                "ci_gate_result_line": str(ci_gate_result_line_path),
                "ci_gate_badge_json": str(ci_gate_badge_json),
                "ci_fail_brief_txt": str(ci_fail_brief_txt),
                "ci_fail_triage_json": str(ci_fail_triage_json),
                "oi_close": str(oi_report),
                "oi_pack": str(oi_pack_report),
                "aggregate": str(aggregate_report),
                "ci_profile_matrix_gate_selftest": str(ci_profile_matrix_gate_selftest_report),
                "ci_sanity_gate": str(ci_sanity_gate_report),
                "ci_sync_readiness": str(ci_sync_readiness_report),
            },
            "steps": steps_log,
            "overall_ok": bool(overall_ok),
        }
        index_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        if announce:
            print(f"[ci-gate] report_index={index_path}")

    def run_backup_hygiene_move() -> int:
        return run_and_record(
            "backup_hygiene_move",
            [
                py,
                "scripts/seamgrim_manage_lesson_backups.py",
                "--mode",
                "move",
                "--json-out",
                str(backup_hygiene_move_report),
            ],
        )

    def run_backup_hygiene_verify() -> int:
        return run_and_record(
            "backup_hygiene_verify",
            [
                py,
                "scripts/seamgrim_manage_lesson_backups.py",
                "--mode",
                "list",
                "--fail-on-targets",
                "--json-out",
                str(backup_hygiene_verify_report),
            ],
        )

    def render_age3_summary() -> int:
        return run_and_record(
            "age3_close_summary",
            [
                py,
                "tools/scripts/render_age3_close_summary.py",
                str(age3_close_report),
                "--out",
                str(age3_close_summary_md),
                "--fail-on-bad",
            ],
        )

    def render_age3_status() -> int:
        return run_and_record(
            "age3_close_status",
            [
                py,
                "tools/scripts/render_age3_close_status.py",
                str(age3_close_report),
                "--out",
                str(age3_close_status_json),
                "--fail-on-bad",
            ],
        )

    def render_age3_status_line() -> int:
        return run_and_record(
            "age3_close_status_line",
            [
                py,
                "tools/scripts/render_age3_close_status_line.py",
                str(age3_close_status_json),
                "--out",
                str(age3_close_status_line),
                "--fail-on-bad",
            ],
        )

    def render_age3_badge() -> int:
        return run_and_record(
            "age3_close_badge",
            [
                py,
                "tools/scripts/render_age3_close_badge.py",
                str(age3_close_status_json),
                "--status-line",
                str(age3_close_status_line),
                "--out",
                str(age3_close_badge_json),
                "--fail-on-bad",
            ],
        )

    def parse_age3_status_line() -> int:
        return run_and_record(
            "age3_close_status_line_parse",
            [
                py,
                "tools/scripts/parse_age3_close_status_line.py",
                "--status-line",
                str(age3_close_status_line),
                "--status-json",
                str(age3_close_status_json),
                "--fail-on-invalid",
            ],
        )

    def check_age3_status_line(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_age3_status_line_check.py",
            "--status-line",
            str(age3_close_status_line),
            "--status-json",
            str(age3_close_status_json),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("age3_close_status_line_check", cmd)

    def check_age3_badge(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_age3_badge_check.py",
            "--badge",
            str(age3_close_badge_json),
            "--status-json",
            str(age3_close_status_json),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("age3_close_badge_check", cmd)

    def render_aggregate_status_line(fail_on_bad: bool) -> int:
        cmd = [
            py,
            "tools/scripts/render_ci_aggregate_status_line.py",
            str(aggregate_report),
            "--out",
            str(aggregate_status_line),
        ]
        if fail_on_bad:
            cmd.append("--fail-on-bad")
        return run_and_record("aggregate_status_line", cmd)

    def check_aggregate_status_line(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_ci_aggregate_status_line_check.py",
            "--status-line",
            str(aggregate_status_line),
            "--aggregate-report",
            str(aggregate_report),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("aggregate_status_line_check", cmd)

    def render_final_status_line(fail_on_bad: bool) -> int:
        cmd = [
            py,
            "tools/scripts/render_ci_gate_final_status_line.py",
            "--aggregate-status-parse",
            str(aggregate_status_parse_json),
            "--gate-index",
            str(index_report_path),
            "--out",
            str(final_status_line),
        ]
        if fail_on_bad:
            cmd.append("--fail-on-bad")
        return run_and_record("final_status_line", cmd)

    def check_final_status_line(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_ci_gate_final_status_line_check.py",
            "--status-line",
            str(final_status_line),
            "--aggregate-status-parse",
            str(aggregate_status_parse_json),
            "--gate-index",
            str(index_report_path),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("final_status_line_check", cmd)

    def parse_final_status_line() -> int:
        return run_and_record(
            "final_status_line_parse",
            [
                py,
                "tools/scripts/parse_ci_gate_final_status_line.py",
                "--status-line",
                str(final_status_line),
                "--gate-index",
                str(index_report_path),
                "--json-out",
                str(final_status_parse_json),
                "--compact-out",
                str(summary_line_path),
                "--fail-on-invalid",
            ],
        )

    def parse_aggregate_status_line() -> int:
        return run_and_record(
            "aggregate_status_line_parse",
            [
                py,
                "tools/scripts/parse_ci_aggregate_status_line.py",
                "--status-line",
                str(aggregate_status_line),
                "--aggregate-report",
                str(aggregate_report),
                "--json-out",
                str(aggregate_status_parse_json),
                "--fail-on-invalid",
            ],
        )

    def check_summary_line(require_pass: bool, use_result_parse: bool) -> int:
        cmd = [
            py,
            "tests/run_ci_gate_summary_line_check.py",
            "--summary-line",
            str(summary_line_path),
        ]
        if use_result_parse:
            cmd.extend(["--ci-gate-result-parse", str(ci_gate_result_parse_json)])
        else:
            cmd.extend(["--final-status-parse", str(final_status_parse_json)])
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("summary_line_check", cmd)

    def render_ci_gate_result(fail_on_bad: bool) -> int:
        cmd = [
            py,
            "tools/scripts/render_ci_gate_result.py",
            "--final-status-parse",
            str(final_status_parse_json),
            "--summary-line",
            str(summary_line_path),
            "--gate-index",
            str(index_report_path),
            "--out",
            str(ci_gate_result_json),
        ]
        if fail_on_bad:
            cmd.append("--fail-on-bad")
        return run_and_record("ci_gate_result", cmd)

    def check_ci_gate_result(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_ci_gate_result_check.py",
            "--result",
            str(ci_gate_result_json),
            "--final-status-parse",
            str(final_status_parse_json),
            "--summary-line",
            str(summary_line_path),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("ci_gate_result_check", cmd)

    def parse_ci_gate_result(fail_on_fail: bool) -> int:
        cmd = [
            py,
            "tools/scripts/parse_ci_gate_result.py",
            "--result",
            str(ci_gate_result_json),
            "--json-out",
            str(ci_gate_result_parse_json),
            "--compact-out",
            str(ci_gate_result_line_path),
            "--fail-on-invalid",
        ]
        if fail_on_fail:
            cmd.append("--fail-on-fail")
        rc = run_and_record("ci_gate_result_parse", cmd)
        if rc == 0:
            try:
                compact = ci_gate_result_line_path.read_text(encoding="utf-8").strip()
            except Exception:
                compact = "-"
            if compact != "-":
                write_summary_line(summary_line_path, compact)
        return rc

    def render_ci_gate_badge(fail_on_bad: bool) -> int:
        cmd = [
            py,
            "tools/scripts/render_ci_gate_badge.py",
            str(ci_gate_result_json),
            "--out",
            str(ci_gate_badge_json),
        ]
        if fail_on_bad:
            cmd.append("--fail-on-bad")
        return run_and_record("ci_gate_badge", cmd)

    def check_ci_gate_badge(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_ci_gate_badge_check.py",
            "--badge",
            str(ci_gate_badge_json),
            "--result",
            str(ci_gate_result_json),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("ci_gate_badge_check", cmd)

    def _run_unrecorded(label: str, cmd: list[str]) -> int:
        step_name = f"refresh::{sanitize_step_name(label)}"
        step_result = run_step(
            root,
            step_name,
            cmd,
            quiet_success_logs=bool(args.quiet_success_logs),
            compact_step_logs=bool(args.compact_step_logs),
            step_log_failed_only=bool(args.step_log_failed_only),
            stdout_log_path=None,
            stderr_log_path=None,
        )
        return int(step_result.get("returncode", 127))

    def refresh_status_outputs_for_index() -> int:
        refresh_cmds: list[tuple[str, list[str]]] = [
            (
                "refresh_aggregate_status_line",
                [
                    py,
                    "tools/scripts/render_ci_aggregate_status_line.py",
                    str(aggregate_report),
                    "--out",
                    str(aggregate_status_line),
                ],
            ),
            (
                "refresh_aggregate_status_parse",
                [
                    py,
                    "tools/scripts/parse_ci_aggregate_status_line.py",
                    "--status-line",
                    str(aggregate_status_line),
                    "--aggregate-report",
                    str(aggregate_report),
                    "--json-out",
                    str(aggregate_status_parse_json),
                    "--fail-on-invalid",
                ],
            ),
            (
                "refresh_final_status_line",
                [
                    py,
                    "tools/scripts/render_ci_gate_final_status_line.py",
                    "--aggregate-status-parse",
                    str(aggregate_status_parse_json),
                    "--gate-index",
                    str(index_report_path),
                    "--out",
                    str(final_status_line),
                ],
            ),
            (
                "refresh_final_status_parse",
                [
                    py,
                    "tools/scripts/parse_ci_gate_final_status_line.py",
                    "--status-line",
                    str(final_status_line),
                    "--gate-index",
                    str(index_report_path),
                    "--json-out",
                    str(final_status_parse_json),
                    "--compact-out",
                    str(summary_line_path),
                    "--fail-on-invalid",
                ],
            ),
            (
                "refresh_ci_gate_result",
                [
                    py,
                    "tools/scripts/render_ci_gate_result.py",
                    "--final-status-parse",
                    str(final_status_parse_json),
                    "--summary-line",
                    str(summary_line_path),
                    "--gate-index",
                    str(index_report_path),
                    "--out",
                    str(ci_gate_result_json),
                ],
            ),
            (
                "refresh_ci_gate_result_parse",
                [
                    py,
                    "tools/scripts/parse_ci_gate_result.py",
                    "--result",
                    str(ci_gate_result_json),
                    "--json-out",
                    str(ci_gate_result_parse_json),
                    "--compact-out",
                    str(ci_gate_result_line_path),
                    "--fail-on-invalid",
                ],
            ),
            (
                "refresh_ci_gate_badge",
                [
                    py,
                    "tools/scripts/render_ci_gate_badge.py",
                    str(ci_gate_result_json),
                    "--out",
                    str(ci_gate_badge_json),
                ],
            ),
            (
                "refresh_ci_emit_artifacts_generate",
                [
                    py,
                    "tools/scripts/emit_ci_final_line.py",
                    "--report-dir",
                    str(report_dir),
                    "--print-failure-digest",
                    "6",
                    "--print-failure-tail-lines",
                    "20",
                    "--fail-on-summary-verify-error",
                    "--failure-brief-out",
                    str(ci_fail_brief_txt),
                    "--triage-json-out",
                    str(ci_fail_triage_json),
                    "--require-final-line",
                ]
                + (["--prefix", prefix] if prefix else []),
            ),
        ]
        for label, cmd in refresh_cmds:
            rc = _run_unrecorded(label, cmd)
            if rc != 0:
                return rc
        return 0

    def check_ci_gate_outputs_consistency(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_ci_gate_outputs_consistency_check.py",
            "--summary-line",
            str(summary_line_path),
            "--result",
            str(ci_gate_result_json),
            "--result-parse",
            str(ci_gate_result_parse_json),
            "--badge",
            str(ci_gate_badge_json),
            "--final-status-parse",
            str(final_status_parse_json),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("ci_gate_outputs_consistency_check", cmd)

    def check_ci_gate_failure_summary(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_ci_gate_failure_summary_check.py",
            "--summary",
            str(summary_path),
            "--index",
            str(index_report_path),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("ci_gate_failure_summary_check", cmd)

    def check_ci_gate_summary_report(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_ci_gate_summary_report_check.py",
            "--summary",
            str(summary_path),
            "--index",
            str(index_report_path),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("ci_gate_summary_report_check", cmd)

    def check_ci_gate_summary_report_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_gate_summary_report_check_selftest.py",
        ]
        return run_and_record("ci_gate_summary_report_selftest", cmd)

    report_index_required_steps_common = [
        "ci_profile_split_contract_check",
        "ci_profile_matrix_gate_selftest",
        "ci_sanity_gate",
        "ci_sync_readiness_report_generate",
        "ci_sync_readiness_report_check",
        "ci_emit_artifacts_required_post_summary_check",
        "ci_fail_and_exit_contract_selftest",
        "ci_gate_report_index_selftest",
        "ci_gate_report_index_diagnostics_check",
        "ci_gate_report_index_latest_smoke_check",
    ]
    report_index_required_steps_seamgrim = [
        "seamgrim_ci_gate_seed_meta_step_check",
        "seamgrim_ci_gate_sam_seulgi_family_step_check",
        "seamgrim_ci_gate_runtime5_passthrough_check",
        "seamgrim_ci_gate_guideblock_step_check",
        "seamgrim_ci_gate_lesson_warning_step_check",
        "seamgrim_ci_gate_stateful_preview_step_check",
        "seamgrim_ci_gate_wasm_web_smoke_step_check",
        "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
        "seamgrim_wasm_cli_diag_parity_check",
    ]

    def resolve_report_index_required_steps(sanity_profile: str) -> list[str]:
        if sanity_profile == "core_lang":
            return list(report_index_required_steps_common)
        if sanity_profile == "seamgrim":
            return list(report_index_required_steps_common + report_index_required_steps_seamgrim)
        return list(report_index_required_steps_common + report_index_required_steps_seamgrim)

    report_index_required_steps = resolve_report_index_required_steps(args.ci_sanity_profile)

    def check_ci_gate_report_index(require_step_contract: bool) -> int:
        cmd = [
            py,
            "tests/run_ci_gate_report_index_check.py",
            "--index",
            str(index_report_path),
            "--sanity-profile",
            args.ci_sanity_profile,
        ]
        if require_step_contract:
            cmd.append("--enforce-profile-step-contract")
            for step_name in report_index_required_steps:
                cmd.extend(["--required-step", step_name])
        return run_and_record("ci_gate_report_index_check", cmd)

    def check_ci_gate_report_index_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_gate_report_index_check_selftest.py",
        ]
        return run_and_record("ci_gate_report_index_selftest", cmd)

    def check_ci_gate_report_index_diagnostics() -> int:
        cmd = [
            py,
            "tests/run_ci_gate_report_index_diagnostics_check.py",
        ]
        return run_and_record("ci_gate_report_index_diagnostics_check", cmd)

    def check_ci_fail_and_exit_contract_selftest() -> int:
        if bool(args.skip_fail_and_exit_contract_selftest):
            return check_ci_fail_and_exit_contract_selftest_skipped("flag_disabled")
        cmd = [
            py,
            "tests/run_ci_fail_and_exit_contract_selftest.py",
        ]
        return run_and_record("ci_fail_and_exit_contract_selftest", cmd)

    def check_ci_fail_and_exit_contract_selftest_skipped(reason: str) -> int:
        safe_reason = str(reason).strip() or "unspecified"
        cmd = [
            py,
            "-c",
            f"print('ci_fail_and_exit_contract_selftest: skipped reason={safe_reason}')",
        ]
        return run_and_record("ci_fail_and_exit_contract_selftest", cmd)

    def check_ci_gate_report_index_latest_smoke() -> int:
        cmd = [
            py,
            "tests/run_ci_gate_report_index_latest_smoke_check.py",
            "--report-dir",
            str(report_dir),
            "--index-pattern",
            f"*{args.report_index_base_name}",
        ]
        if prefix:
            cmd.extend(["--prefix", prefix])
        return run_and_record("ci_gate_report_index_latest_smoke_check", cmd)

    def validate_ci_gate_report_index_latest_smoke_skip_reason(reason: str) -> str:
        safe_reason = str(reason).strip()
        if safe_reason in LATEST_SMOKE_SKIP_REASON_EXPECTED:
            return safe_reason
        expected_text = ",".join(LATEST_SMOKE_SKIP_REASON_EXPECTED)
        raise RuntimeError(
            "ci_gate_report_index_latest_smoke_check: invalid skip reason="
            f"{safe_reason!r} expected={expected_text}"
        )

    def check_ci_gate_report_index_latest_smoke_skipped(reason: str) -> int:
        safe_reason = validate_ci_gate_report_index_latest_smoke_skip_reason(reason)
        cmd = [
            py,
            "-c",
            f"print('ci_gate_report_index_latest_smoke_check: skipped reason={safe_reason}')",
        ]
        return run_and_record("ci_gate_report_index_latest_smoke_check", cmd)

    def should_run_ci_gate_report_index_latest_smoke() -> bool:
        result_doc = load_payload(ci_gate_result_json)
        if not isinstance(result_doc, dict):
            return False
        status = str(result_doc.get("status", "")).strip().lower()
        return status == "pass"

    def resolve_ci_gate_report_index_latest_smoke_skip_reason(
        has_failed_steps: bool,
        ci_gate_report_index_rc: int,
    ) -> str | None:
        if not bool(args.run_report_index_latest_smoke):
            return LATEST_SMOKE_SKIP_REASON_FLAG_DISABLED
        elif has_failed_steps or ci_gate_report_index_rc != 0:
            return LATEST_SMOKE_SKIP_REASON_PENDING_FAILURE_SUMMARY_REGENERATION
        elif should_run_ci_gate_report_index_latest_smoke():
            return None
        else:
            return LATEST_SMOKE_SKIP_REASON_CI_GATE_RESULT_STATUS_NOT_PASS

    def run_ci_gate_report_index_latest_smoke_step(
        has_failed_steps: bool,
        ci_gate_report_index_rc: int,
    ) -> int:
        skip_reason = resolve_ci_gate_report_index_latest_smoke_skip_reason(
            has_failed_steps,
            ci_gate_report_index_rc,
        )
        if skip_reason is None:
            return check_ci_gate_report_index_latest_smoke()
        return check_ci_gate_report_index_latest_smoke_skipped(skip_reason)

    def check_ci_aggregate_gate_age4_diagnostics() -> int:
        cmd = [
            py,
            "tests/run_ci_aggregate_gate_age4_diagnostics_check.py",
        ]
        return run_and_record("ci_aggregate_gate_age4_diagnostics_check", cmd)

    def check_ci_aggregate_gate_age5_diagnostics() -> int:
        cmd = [
            py,
            "tests/run_ci_aggregate_gate_age5_diagnostics_check.py",
        ]
        return run_and_record("ci_aggregate_gate_age5_diagnostics_check", cmd)

    def check_ci_aggregate_gate_phase3_diagnostics() -> int:
        cmd = [
            py,
            "tests/run_ci_aggregate_gate_phase3_diagnostics_check.py",
        ]
        return run_and_record("ci_aggregate_gate_phase3_diagnostics_check", cmd)

    def check_ci_aggregate_gate_runtime5_diagnostics() -> int:
        cmd = [
            py,
            "tests/run_ci_aggregate_gate_runtime5_diagnostics_check.py",
        ]
        return run_and_record("ci_aggregate_gate_runtime5_diagnostics_check", cmd)

    def check_ci_aggregate_gate_seamgrim_diagnostics() -> int:
        cmd = [
            py,
            "tests/run_ci_aggregate_gate_seamgrim_diagnostics_check.py",
        ]
        return run_and_record("ci_aggregate_gate_seamgrim_diagnostics_check", cmd)

    def check_ci_aggregate_gate_sync_diagnostics() -> int:
        cmd = [
            py,
            "tests/run_ci_aggregate_gate_sync_diagnostics_check.py",
        ]
        return run_and_record("ci_aggregate_gate_sync_diagnostics_check", cmd)

    def check_ci_aggregate_gate_sanity_diagnostics() -> int:
        cmd = [
            py,
            "tests/run_ci_aggregate_gate_sanity_diagnostics_check.py",
        ]
        return run_and_record("ci_aggregate_gate_sanity_diagnostics_check", cmd)

    def check_ci_sanity_gate_diagnostics() -> int:
        cmd = [
            py,
            "tests/run_ci_sanity_gate_diagnostics_check.py",
        ]
        return run_and_record("ci_sanity_gate_diagnostics_check", cmd)

    def check_ci_sanity_gate() -> int:
        cmd = [
            py,
            "tests/run_ci_sanity_gate.py",
            "--profile",
            args.ci_sanity_profile,
            "--json-out",
            str(ci_sanity_gate_report),
        ]
        return run_and_record("ci_sanity_gate", cmd)

    def check_ci_profile_split_contract() -> int:
        cmd = [
            py,
            "tests/run_ci_profile_split_contract_check.py",
        ]
        return run_and_record("ci_profile_split_contract_check", cmd)

    def check_ci_profile_matrix_gate_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_profile_matrix_gate_selftest.py",
            "--json-out",
            str(ci_profile_matrix_gate_selftest_report),
        ]
        if args.profile_matrix_selftest_real_profiles.strip():
            cmd.extend(
                [
                    "--real-profiles",
                    args.profile_matrix_selftest_real_profiles.strip(),
                ]
            )
        if args.profile_matrix_selftest_dry:
            cmd.append("--dry-selftest")
        if args.profile_matrix_selftest_quick:
            cmd.append("--quick-selftest")
        if args.profile_matrix_selftest_full_aggregate_gates:
            cmd.append("--matrix-full-aggregate-gates")
        if args.profile_matrix_selftest_with_profile_matrix_full_real_smoke:
            cmd.append("--matrix-with-profile-matrix-full-real-smoke")
        return run_and_record("ci_profile_matrix_gate_selftest", cmd)

    def check_ci_builtin_name_sync() -> int:
        cmd = [
            py,
            "tests/run_builtin_name_sync_check.py",
        ]
        return run_and_record("ci_builtin_name_sync_check", cmd)

    def check_ci_fixed64_probe_selftest() -> int:
        cmd = [
            py,
            "tests/run_fixed64_cross_platform_probe_selftest.py",
        ]
        return run_and_record("ci_fixed64_probe_selftest", cmd)

    def check_ci_fixed64_win_wsl_matrix_selftest() -> int:
        cmd = [
            py,
            "tests/run_fixed64_windows_wsl_matrix_selftest.py",
        ]
        return run_and_record("ci_fixed64_win_wsl_matrix_selftest", cmd)

    def check_ci_fixed64_threeway_inputs_selftest() -> int:
        cmd = [
            py,
            "tests/run_fixed64_threeway_inputs_selftest.py",
        ]
        return run_and_record("ci_fixed64_threeway_inputs_selftest", cmd)

    def check_ci_fixed64_darwin_probe_artifact_selftest() -> int:
        cmd = [
            py,
            "tests/run_fixed64_darwin_probe_artifact_selftest.py",
        ]
        return run_and_record("ci_fixed64_darwin_probe_artifact_selftest", cmd)

    def check_ci_fixed64_threeway_gate(require_darwin: bool) -> int:
        default_report_dir = (root / "build" / "reports").resolve()

        def resolve_probe_report_path(base_name: str) -> Path:
            scoped = (report_dir / base_name).resolve()
            if scoped.exists():
                return scoped
            fallback = (default_report_dir / base_name).resolve()
            if fallback.exists():
                return fallback
            return scoped

        windows_probe_report = resolve_probe_report_path("fixed64_cross_platform_probe_windows.detjson")
        linux_probe_report = resolve_probe_report_path("fixed64_cross_platform_probe_linux.detjson")
        darwin_probe_report = (report_dir / "fixed64_cross_platform_probe_darwin.detjson").resolve()
        darwin_artifact_summary = (report_dir / "fixed64_darwin_probe_artifact.detjson").resolve()
        darwin_artifact_zip = (report_dir / "fixed64_darwin_probe_artifact.zip").resolve()
        darwin_archive_dir = (report_dir / "darwin_probe_archive").resolve()
        darwin_candidate_paths = [
            darwin_artifact_summary,
            darwin_artifact_zip,
            darwin_probe_report,
            darwin_archive_dir,
        ]
        darwin_probe_env_raw = str(os.environ.get("DDN_ENABLE_DARWIN_PROBE", "")).strip().lower()
        darwin_probe_env_enabled = darwin_probe_env_raw in {"1", "true", "yes", "on"}
        # Resolve 단계는 darwin probe를 실제로 요구하는 모드에서만 실행한다.
        # 비활성 환경(DDN_ENABLE_DARWIN_PROBE=0)에서는 pending_darwin 경로를 허용해
        # stale/synthetic 후보 파일 때문에 게이트가 깨지지 않게 한다.
        enable_resolve_inputs = bool(require_darwin or darwin_probe_env_enabled)
        cmd = [
            py,
            "tests/run_fixed64_cross_platform_threeway_gate.py",
            "--report-out",
            str(fixed64_threeway_gate_report),
            "--windows-report",
            str(windows_probe_report),
            "--linux-report",
            str(linux_probe_report),
            "--darwin-report",
            str(darwin_probe_report),
        ]
        if enable_resolve_inputs:
            cmd.extend(
                [
                    "--resolve-threeway-inputs",
                    "--resolve-inputs-json-out",
                    str(fixed64_threeway_inputs_report),
                    "--resolve-inputs-strict-invalid",
                    "--resolve-inputs-require-when-env",
                    "DDN_ENABLE_DARWIN_PROBE",
                ]
            )
            for path in darwin_candidate_paths:
                cmd.extend(["--resolve-input-candidate", str(path)])
        if float(args.fixed64_threeway_max_report_age_minutes) > 0:
            cmd.extend(
                [
                    "--max-report-age-minutes",
                    str(float(args.fixed64_threeway_max_report_age_minutes)),
                ]
            )
        if require_darwin:
            cmd.append("--require-darwin")
        return run_and_record("ci_fixed64_threeway_gate", cmd)

    def check_ci_fixed64_threeway_gate_selftest() -> int:
        cmd = [
            py,
            "tests/run_fixed64_cross_platform_threeway_gate_selftest.py",
        ]
        return run_and_record("ci_fixed64_threeway_gate_selftest", cmd)

    def check_ci_aggregate_status_line_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_aggregate_status_line_selftest.py",
        ]
        return run_and_record("ci_aggregate_status_line_selftest", cmd)

    def check_ci_combine_reports_age4_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_combine_reports_age4_selftest.py",
        ]
        return run_and_record("ci_combine_reports_age4_selftest", cmd)

    def check_ci_combine_reports_age5_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_combine_reports_age5_selftest.py",
        ]
        return run_and_record("ci_combine_reports_age5_selftest", cmd)

    def check_age5_close_digest_selftest() -> int:
        cmd = [
            py,
            "tests/run_age5_close_digest_selftest.py",
        ]
        return run_and_record("age5_close_digest_selftest", cmd)

    def check_ci_pack_golden_overlay_compare_selftest() -> int:
        cmd = [
            py,
            "tests/run_pack_golden_overlay_compare_selftest.py",
        ]
        return run_and_record("ci_pack_golden_overlay_compare_selftest", cmd)

    def check_ci_pack_golden_overlay_session_selftest() -> int:
        cmd = [
            py,
            "tests/run_pack_golden_overlay_session_selftest.py",
        ]
        return run_and_record("ci_pack_golden_overlay_session_selftest", cmd)

    def check_ci_pack_golden_guideblock_selftest() -> int:
        cmd = [
            py,
            "tests/run_pack_golden_guideblock_selftest.py",
        ]
        return run_and_record("ci_pack_golden_guideblock_selftest", cmd)

    def check_ci_pack_golden_age5_surface_selftest() -> int:
        cmd = [
            py,
            "tests/run_pack_golden_age5_surface_selftest.py",
        ]
        return run_and_record("ci_pack_golden_age5_surface_selftest", cmd)

    def check_ci_pack_golden_exec_policy_selftest() -> int:
        cmd = [
            py,
            "tests/run_pack_golden_exec_policy_selftest.py",
        ]
        return run_and_record("ci_pack_golden_exec_policy_selftest", cmd)

    def check_ci_pack_golden_jjaim_flatten_selftest() -> int:
        cmd = [
            py,
            "tests/run_pack_golden_jjaim_flatten_selftest.py",
        ]
        return run_and_record("ci_pack_golden_jjaim_flatten_selftest", cmd)

    def check_ci_pack_golden_event_model_selftest() -> int:
        cmd = [
            py,
            "tests/run_pack_golden_event_model_selftest.py",
        ]
        return run_and_record("ci_pack_golden_event_model_selftest", cmd)

    def check_seamgrim_browse_selection_report_selftest() -> int:
        cmd = [
            py,
            "tests/run_seamgrim_browse_selection_report_check_selftest.py",
        ]
        return run_and_record("seamgrim_browse_selection_report_selftest", cmd)

    def check_seamgrim_5min_checklist_selftest() -> int:
        cmd = [
            py,
            "tests/run_seamgrim_5min_checklist_selftest.py",
        ]
        return run_and_record("seamgrim_5min_checklist_selftest", cmd)

    def check_seamgrim_wasm_cli_diag_parity() -> int:
        cmd = [
            py,
            "tests/run_seamgrim_wasm_cli_diag_parity_check.py",
            "--json-out",
            str(seamgrim_wasm_cli_diag_parity_report),
        ]
        return run_and_record("seamgrim_wasm_cli_diag_parity_check", cmd)

    def check_ci_final_line_emitter() -> int:
        cmd = [
            py,
            "tests/run_ci_final_line_emitter_check.py",
        ]
        return run_and_record("ci_final_line_emitter_check", cmd)

    def check_ci_pipeline_emit_flags() -> int:
        cmd = [
            py,
            "tests/run_ci_pipeline_emit_flags_check.py",
        ]
        return run_and_record("ci_pipeline_emit_flags_check", cmd)

    def check_ci_pipeline_emit_flags_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_pipeline_emit_flags_check_selftest.py",
        ]
        return run_and_record("ci_pipeline_emit_flags_selftest", cmd)

    def check_seamgrim_ci_gate_runtime5_passthrough() -> int:
        cmd = [
            py,
            "tests/run_seamgrim_ci_gate_runtime5_passthrough_check.py",
        ]
        return run_and_record("seamgrim_ci_gate_runtime5_passthrough_check", cmd)

    def check_seamgrim_ci_gate_preview_sync_passthrough() -> int:
        cmd = [
            py,
            "tests/run_seamgrim_ci_gate_preview_sync_passthrough_check.py",
        ]
        return run_and_record("seamgrim_ci_gate_preview_sync_passthrough_check", cmd)

    def check_seamgrim_ci_gate_seed_meta_step() -> int:
        cmd = [
            py,
            "tests/run_seamgrim_ci_gate_seed_meta_step_check.py",
        ]
        return run_and_record("seamgrim_ci_gate_seed_meta_step_check", cmd)

    def check_seamgrim_ci_gate_sam_seulgi_family_step() -> int:
        cmd = [
            py,
            "tests/run_seamgrim_ci_gate_sam_seulgi_family_step_check.py",
        ]
        return run_and_record("seamgrim_ci_gate_sam_seulgi_family_step_check", cmd)

    def check_seamgrim_ci_gate_guideblock_step() -> int:
        cmd = [
            py,
            "tests/run_seamgrim_ci_gate_guideblock_step_check.py",
        ]
        return run_and_record("seamgrim_ci_gate_guideblock_step_check", cmd)

    def check_seamgrim_ci_gate_lesson_warning_step() -> int:
        cmd = [
            py,
            "tests/run_seamgrim_ci_gate_lesson_warning_step_check.py",
        ]
        return run_and_record("seamgrim_ci_gate_lesson_warning_step_check", cmd)

    def check_seamgrim_ci_gate_stateful_preview_step() -> int:
        cmd = [
            py,
            "tests/run_seamgrim_ci_gate_stateful_preview_step_check.py",
        ]
        return run_and_record("seamgrim_ci_gate_stateful_preview_step_check", cmd)

    seamgrim_wasm_web_step_check_report = (
        report_dir / f"{prefix}.seamgrim_ci_gate_wasm_web_smoke_step_check.detjson"
    )

    def check_seamgrim_ci_gate_wasm_web_smoke_step() -> int:
        cmd = [
            py,
            "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check.py",
            "--report-out",
            str(seamgrim_wasm_web_step_check_report),
        ]
        return run_and_record("seamgrim_ci_gate_wasm_web_smoke_step_check", cmd)

    def check_seamgrim_ci_gate_wasm_web_smoke_step_selftest() -> int:
        cmd = [
            py,
            "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check_selftest.py",
            "--verify-report",
            str(seamgrim_wasm_web_step_check_report),
        ]
        return run_and_record("seamgrim_ci_gate_wasm_web_smoke_step_check_selftest", cmd)

    def check_ci_sync_readiness_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_sync_readiness_check_selftest.py",
        ]
        return run_and_record("ci_sync_readiness_selftest", cmd)

    def check_ci_sync_readiness_diagnostics() -> int:
        cmd = [
            py,
            "tests/run_ci_sync_readiness_diagnostics_check.py",
        ]
        return run_and_record("ci_sync_readiness_diagnostics_check", cmd)

    def check_ci_sync_readiness_report_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_sync_readiness_report_check_selftest.py",
        ]
        return run_and_record("ci_sync_readiness_report_selftest", cmd)

    def run_ci_sync_readiness_report_generate() -> int:
        cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            prefix if prefix else "aggregate_sync_readiness",
            "--sanity-profile",
            args.ci_sanity_profile,
            "--json-out",
            str(ci_sync_readiness_report),
            "--validate-only-sanity-json",
            str(ci_sanity_gate_report),
        ]
        return run_and_record("ci_sync_readiness_report_generate", cmd)

    def check_ci_sync_readiness_report_check() -> int:
        cmd = [
            py,
            "tests/run_ci_sync_readiness_report_check.py",
            "--report",
            str(ci_sync_readiness_report),
            "--sanity-profile",
            args.ci_sanity_profile,
            "--require-pass",
        ]
        return run_and_record("ci_sync_readiness_report_check", cmd)

    def check_ci_backup_hygiene_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_backup_hygiene_selftest.py",
        ]
        return run_and_record("ci_backup_hygiene_selftest", cmd)

    def check_ci_emit_artifacts_baseline() -> int:
        cmd = [
            py,
            "tests/run_ci_emit_artifacts_check.py",
            "--report-dir",
            str(report_dir),
        ]
        if prefix:
            cmd.extend(["--prefix", prefix])
        return run_and_record("ci_emit_artifacts_baseline_check", cmd)

    def emit_ci_final_line_for_artifacts() -> int:
        # triage artifact의 summary.exists를 최종 단계에서도 안정적으로 유지하기 위해
        # summary 파일이 아직 없으면 임시 헤더를 먼저 생성한다.
        if not summary_path.exists():
            summary_path.parent.mkdir(parents=True, exist_ok=True)
            summary_path.write_text("[ci-gate-summary] PREVIEW\n", encoding="utf-8")
        cmd = [
            py,
            "tools/scripts/emit_ci_final_line.py",
            "--report-dir",
            str(report_dir),
            "--print-failure-digest",
            "6",
            "--print-failure-tail-lines",
            "20",
            "--fail-on-summary-verify-error",
            "--failure-brief-out",
            str(ci_fail_brief_txt),
            "--triage-json-out",
            str(ci_fail_triage_json),
            "--require-final-line",
        ]
        if prefix:
            cmd.extend(["--prefix", prefix])
        return run_and_record("ci_emit_artifacts_generate", cmd)

    def check_ci_emit_artifacts_required() -> int:
        cmd = [
            py,
            "tests/run_ci_emit_artifacts_check.py",
            "--report-dir",
            str(report_dir),
            "--require-brief",
            "--require-triage",
        ]
        if prefix:
            cmd.extend(["--prefix", prefix])
        return run_and_record("ci_emit_artifacts_required_check", cmd)

    def check_ci_emit_artifacts_required_post_summary() -> int:
        cmd = [
            py,
            "tests/run_ci_emit_artifacts_check.py",
            "--report-dir",
            str(report_dir),
            "--require-brief",
            "--require-triage",
            "--allow-triage-exists-upgrade",
        ]
        if prefix:
            cmd.extend(["--prefix", prefix])
        return run_and_record("ci_emit_artifacts_required_post_summary_check", cmd)

    def check_ci_emit_artifacts_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_emit_artifacts_check_selftest.py",
        ]
        return run_and_record("ci_emit_artifacts_selftest", cmd)

    def check_ci_emit_artifacts_sanity_contract() -> int:
        cmd = [
            py,
            "tests/run_ci_emit_artifacts_sanity_contract_check.py",
        ]
        return run_and_record("ci_emit_artifacts_sanity_contract_check", cmd)

    def check_ci_emit_artifacts_sanity_contract_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_emit_artifacts_sanity_contract_check_selftest.py",
        ]
        return run_and_record("ci_emit_artifacts_sanity_contract_selftest", cmd)

    def check_ci_gate_failure_summary_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_gate_failure_summary_check_selftest.py",
        ]
        return run_and_record("ci_gate_failure_summary_selftest", cmd)

    def write_emit_artifacts_summary_preview() -> None:
        nonlocal control_exposure_snapshot
        result_doc = load_payload(ci_gate_result_json)
        status = ""
        if isinstance(result_doc, dict):
            status = str(result_doc.get("status", "")).strip().lower()
        if status == "pass":
            preview_lines = [
                "[ci-gate-summary] PASS",
                "[ci-gate-summary] failed_steps=(none)",
                f"[ci-gate-summary] report_index={index_report_path}",
                f"[ci-gate-summary] summary_line={summary_line_path}",
                f"[ci-gate-summary] ci_gate_result={ci_gate_result_json}",
                f"[ci-gate-summary] ci_gate_badge={ci_gate_badge_json}",
                f"[ci-gate-summary] age5_close_digest_selftest_ok={int(age5_close_digest_selftest_rc == 0)}",
                "[ci-gate-summary] ci_pack_golden_overlay_compare_selftest_ok=1",
                "[ci-gate-summary] ci_pack_golden_overlay_session_selftest_ok=1",
                "[ci-gate-summary] ci_pack_golden_guideblock_selftest_ok=1",
                "[ci-gate-summary] ci_pack_golden_age5_surface_selftest_ok=1",
                "[ci-gate-summary] ci_pack_golden_exec_policy_selftest_ok=1",
                "[ci-gate-summary] ci_pack_golden_jjaim_flatten_selftest_ok=1",
                "[ci-gate-summary] ci_pack_golden_event_model_selftest_ok=1",
            ]
            append_ci_profile_matrix_selftest_summary_lines(preview_lines, ci_profile_matrix_gate_selftest_report)
            append_runtime_5min_checklist_summary_lines(
                preview_lines,
                include_5min_checklist,
                seamgrim_5min_checklist_report,
            )
            append_age5_child_summary_lines(preview_lines, age5_close_report)
            control_exposure_snapshot = append_seamgrim_focus_summary_lines(
                preview_lines,
                seamgrim_report,
                seamgrim_control_exposure_failures_report,
                seamgrim_rewrite_overlay_quality_report,
                control_exposure_snapshot,
            )
            append_ci_sanity_summary_lines(preview_lines, ci_sanity_gate_report)
            append_ci_sync_readiness_summary_lines(preview_lines, ci_sync_readiness_report)
            append_fixed64_threeway_summary_lines(preview_lines, fixed64_threeway_gate_report)
            append_age4_proof_summary_lines(preview_lines, aggregate_report)
        else:
            preview_lines = print_failure_block(
                steps_log,
                seamgrim_report,
                age3_close_report,
                age4_close_report,
                age5_close_report,
                oi_report,
                aggregate_report,
            )
            if not preview_lines:
                preview_lines = [
                    "[ci-gate-summary] FAIL",
                    "[ci-gate-summary] failed_steps=summary_preview_fallback",
                    "[ci-gate-summary] failed_step_detail=summary_preview_fallback rc=1 cmd=summary_preview",
                ]
        write_summary(summary_path, preview_lines)

    def fail_and_exit(exit_code: int, message: str) -> int:
        print(message, file=sys.stderr)
        print_fast_fail_step_detail(exit_code)
        nonlocal control_exposure_snapshot
        control_exposure_snapshot = write_control_exposure_failure_report(
            seamgrim_control_exposure_failures_report,
            seamgrim_report,
        )
        render_age3_status()
        render_age3_status_line()
        render_age3_badge()
        parse_age3_status_line()
        check_age3_status_line(require_pass=False)
        check_age3_badge(require_pass=False)
        render_age3_summary()
        render_aggregate_status_line(fail_on_bad=False)
        parse_aggregate_status_line()
        check_aggregate_status_line(require_pass=False)
        write_index(False, announce=False)
        render_final_status_line(fail_on_bad=False)
        check_final_status_line(require_pass=False)
        parse_final_status_line()
        render_ci_gate_result(fail_on_bad=False)
        check_ci_gate_result(require_pass=False)
        parse_ci_gate_result(fail_on_fail=False)
        render_ci_gate_badge(fail_on_bad=False)
        check_ci_gate_badge(require_pass=False)
        check_summary_line(require_pass=False, use_result_parse=True)
        check_ci_gate_outputs_consistency(require_pass=False)
        check_ci_final_line_emitter()
        check_ci_pipeline_emit_flags()
        check_ci_pipeline_emit_flags_selftest()
        check_ci_profile_split_contract()
        check_ci_profile_matrix_gate_selftest()
        check_seamgrim_ci_gate_runtime5_passthrough()
        check_seamgrim_ci_gate_seed_meta_step()
        check_seamgrim_ci_gate_sam_seulgi_family_step()
        check_seamgrim_ci_gate_guideblock_step()
        check_seamgrim_ci_gate_lesson_warning_step()
        check_seamgrim_ci_gate_stateful_preview_step()
        check_seamgrim_ci_gate_wasm_web_smoke_step()
        check_seamgrim_ci_gate_wasm_web_smoke_step_selftest()
        check_ci_sanity_gate()
        check_ci_sync_readiness_selftest()
        check_ci_sync_readiness_diagnostics()
        check_ci_sync_readiness_report_selftest()
        run_ci_sync_readiness_report_generate()
        check_ci_sync_readiness_report_check()
        check_ci_builtin_name_sync()
        check_ci_backup_hygiene_selftest()
        check_ci_aggregate_gate_age4_diagnostics()
        check_ci_aggregate_gate_age5_diagnostics()
        check_ci_aggregate_gate_phase3_diagnostics()
        check_ci_aggregate_gate_runtime5_diagnostics()
        check_ci_aggregate_gate_seamgrim_diagnostics()
        check_ci_aggregate_gate_sync_diagnostics()
        check_ci_aggregate_gate_sanity_diagnostics()
        check_ci_sanity_gate_diagnostics()
        check_ci_aggregate_status_line_selftest()
        check_ci_gate_summary_report_selftest()
        check_ci_combine_reports_age4_selftest()
        check_ci_combine_reports_age5_selftest()
        check_age5_close_digest_selftest()
        check_ci_pack_golden_overlay_compare_selftest()
        check_ci_pack_golden_overlay_session_selftest()
        check_ci_pack_golden_guideblock_selftest()
        check_ci_pack_golden_age5_surface_selftest()
        check_ci_pack_golden_exec_policy_selftest()
        check_ci_pack_golden_jjaim_flatten_selftest()
        check_ci_pack_golden_event_model_selftest()
        check_seamgrim_wasm_cli_diag_parity()
        check_seamgrim_browse_selection_report_selftest()
        check_seamgrim_5min_checklist_selftest()
        check_ci_gate_failure_summary_selftest()
        run_step_if_missing("ci_fail_and_exit_contract_selftest", check_ci_fail_and_exit_contract_selftest)
        run_step_if_missing("ci_gate_report_index_selftest", check_ci_gate_report_index_selftest)
        run_step_if_missing("ci_gate_report_index_diagnostics_check", check_ci_gate_report_index_diagnostics)
        check_ci_emit_artifacts_selftest()
        check_ci_emit_artifacts_sanity_contract()
        check_ci_emit_artifacts_sanity_contract_selftest()
        write_index(False)
        check_ci_gate_report_index_latest_smoke_skipped(LATEST_SMOKE_SKIP_REASON_FAST_FAIL_PATH)
        run_step_if_missing("ci_emit_artifacts_baseline_check", check_ci_emit_artifacts_baseline)
        lines = print_failure_block(
            steps_log,
            seamgrim_report,
            age3_close_report,
            age4_close_report,
            age5_close_report,
            oi_report,
            aggregate_report,
        )
        lines.append(f"[ci-gate-summary] age3_status={age3_close_status_json}")
        lines.append(f"[ci-gate-summary] age2_status={age2_close_report}")
        lines.append(f"[ci-gate-summary] age4_status={age4_close_report}")
        lines.append(f"[ci-gate-summary] age5_status={age5_close_report}")
        append_age5_child_summary_lines(lines, age5_close_report)
        append_age5_policy_summary_lines(lines, aggregate_report)
        append_age4_proof_summary_lines(lines, aggregate_report)
        lines.append(f"[ci-gate-summary] seamgrim_phase3_cleanup={seamgrim_phase3_cleanup_report}")
        append_runtime_5min_summary_lines(
            lines,
            bool(args.with_runtime_5min),
            seamgrim_runtime_5min_report,
            seamgrim_runtime_5min_browse_selection_report,
        )
        append_runtime_5min_checklist_summary_lines(
            lines,
            include_5min_checklist,
            seamgrim_5min_checklist_report,
        )
        control_exposure_snapshot = append_seamgrim_focus_summary_lines(
            lines,
            seamgrim_report,
            seamgrim_control_exposure_failures_report,
            seamgrim_rewrite_overlay_quality_report,
            control_exposure_snapshot,
        )
        append_ci_sanity_summary_lines(lines, ci_sanity_gate_report)
        append_ci_sync_readiness_summary_lines(lines, ci_sync_readiness_report)
        append_fixed64_threeway_summary_lines(lines, fixed64_threeway_gate_report)
        lines.append(f"[ci-gate-summary] age3_status_line={age3_close_status_line}")
        lines.append(f"[ci-gate-summary] age3_badge={age3_close_badge_json}")
        lines.append(f"[ci-gate-summary] age3_status_compact={read_compact_line(age3_close_status_line)}")
        lines.append(f"[ci-gate-summary] aggregate_status_line={aggregate_status_line}")
        lines.append(f"[ci-gate-summary] aggregate_status_parse={aggregate_status_parse_json}")
        lines.append(f"[ci-gate-summary] aggregate_status_compact={read_compact_line(aggregate_status_line)}")
        lines.append(f"[ci-gate-summary] final_status_line={final_status_line}")
        lines.append(f"[ci-gate-summary] final_status_parse={final_status_parse_json}")
        lines.append(f"[ci-gate-summary] summary_line={summary_line_path}")
        lines.append(f"[ci-gate-summary] ci_gate_result={ci_gate_result_json}")
        lines.append(f"[ci-gate-summary] ci_gate_result_parse={ci_gate_result_parse_json}")
        lines.append(f"[ci-gate-summary] ci_gate_result_line={ci_gate_result_line_path}")
        lines.append(f"[ci-gate-summary] ci_gate_badge={ci_gate_badge_json}")
        lines.append(f"[ci-gate-summary] ci_fail_brief_hint={ci_fail_brief_txt}")
        lines.append(f"[ci-gate-summary] ci_fail_brief_exists={int(ci_fail_brief_txt.exists())}")
        lines.append(f"[ci-gate-summary] ci_fail_triage_hint={ci_fail_triage_json}")
        lines.append(f"[ci-gate-summary] ci_fail_triage_exists={int(ci_fail_triage_json.exists())}")
        lines.append(f"[ci-gate-summary] final_status_compact={read_compact_line(final_status_line)}")
        lines.append(f"[ci-gate-summary] age3_summary={age3_close_summary_md}")
        for line in lines:
            print(line)
        if lines:
            write_summary(summary_path, lines)
            emit_required_post_summary_rc = check_ci_emit_artifacts_required_post_summary()
            if emit_required_post_summary_rc != 0:
                print("[ci-gate] final summary emit artifacts required check failed", file=sys.stderr)
                return 2
            write_index(False, announce=False)
            ci_gate_report_index_post_summary_rc = check_ci_gate_report_index(require_step_contract=True)
            if ci_gate_report_index_post_summary_rc != 0:
                print("[ci-gate] report-index post-summary strict check failed", file=sys.stderr)
                return 2
            check_ci_gate_summary_report(require_pass=False)
            check_ci_gate_failure_summary(require_pass=False)
        summary_compact = resolve_summary_compact(
            ci_gate_result_line_path,
            final_status_parse_json,
            final_status_line,
        )
        write_summary_line(summary_line_path, summary_compact)
        print(f"[ci-gate-summary-line] {summary_compact}")
        return exit_code

    if args.contract_only_aggregate:
        print("[ci-gate] contract-only aggregate: synthesize profile-matrix/sanity/sync/fixed64 reports")
        if args.profile_matrix_selftest_real_profiles.strip():
            contract_only_selected_profiles = resolve_contract_only_selected_profiles(
                args.profile_matrix_selftest_real_profiles,
                args.ci_sanity_profile,
            )
        else:
            contract_only_selected_profiles = ["core_lang", "full", "seamgrim"]
        steps_log.extend(
            {
                "name": step_name,
                "returncode": 0,
                "cmd": ["contract-only", step_name],
                "ok": True,
                "stdout_line_count": 0,
                "stderr_line_count": 0,
                "stdout_log_path": "",
                "stderr_log_path": "",
            }
            for step_name in resolve_contract_only_required_steps(args.ci_sanity_profile)
        )
        write_contract_only_stub_reports(
            seamgrim_report,
            seamgrim_ui_age3_report,
            seamgrim_phase3_cleanup_report,
            seamgrim_browse_selection_report,
            age2_close_report,
            age3_close_report,
            age4_close_report,
            age5_close_report,
            age4_pack_report,
            oi_report,
            oi_pack_report,
        )
        write_contract_only_profile_matrix_selftest_report(
            ci_profile_matrix_gate_selftest_report,
            contract_only_selected_profiles,
        )
        write_contract_only_ci_sanity_report(ci_sanity_gate_report, args.ci_sanity_profile)
        write_contract_only_ci_sync_readiness_report(ci_sync_readiness_report, args.ci_sanity_profile)
        write_contract_only_fixed64_reports(fixed64_threeway_inputs_report, fixed64_threeway_gate_report)
        write_json(
            aggregate_report,
            {
                "schema": "ddn.ci.aggregate_gate.report.v1",
                "generated_at_utc": datetime.now(timezone.utc).isoformat(),
                "status": "pass",
                "overall_ok": True,
                "failure_digest": [],
                "reason": "contract_only_stub",
                "age4": {
                    "proof_artifact_ok": CONTRACT_ONLY_AGE4_PROOF_OK == "1",
                    "proof_artifact_failed_criteria": [],
                    "proof_artifact_summary_hash": CONTRACT_ONLY_AGE4_PROOF_SUMMARY_HASH,
                },
                "age5": build_contract_only_age5_aggregate_fields(),
            },
        )
        write_contract_only_ci_gate_outputs(
            args.ci_sanity_profile,
            contract_only_selected_profiles,
            ci_profile_matrix_gate_selftest_report,
            age3_close_status_json,
            age3_close_status_line,
            age3_close_badge_json,
            age3_close_summary_md,
            seamgrim_wasm_cli_diag_parity_report,
            final_status_line,
            final_status_parse_json,
            ci_gate_result_json,
            ci_gate_badge_json,
            ci_fail_brief_txt,
            ci_fail_triage_json,
            summary_path,
            summary_line_path,
            index_report_path,
        )
        pass_lines = [
            "[ci-gate-summary] PASS",
            "[ci-gate-summary] failed_steps=(none)",
            f"[ci-gate-summary] report_index={index_report_path}",
            f"[ci-gate-summary] summary_line={summary_line_path}",
            f"[ci-gate-summary] ci_gate_result={ci_gate_result_json}",
            f"[ci-gate-summary] ci_gate_badge={ci_gate_badge_json}",
            f"[ci-gate-summary] ci_fail_brief_hint={ci_fail_brief_txt}",
            "[ci-gate-summary] ci_fail_brief_exists=1",
            f"[ci-gate-summary] ci_fail_triage_hint={ci_fail_triage_json}",
            "[ci-gate-summary] ci_fail_triage_exists=1",
            f"[ci-gate-summary] age5_close_digest_selftest_ok={int(age5_close_digest_selftest_rc == 0)}",
            f"[ci-gate-summary] age2_status={age2_close_report}",
            f"[ci-gate-summary] age3_status={age3_close_status_json}",
            f"[ci-gate-summary] age4_status={age4_close_report}",
            f"[ci-gate-summary] age5_status={age5_close_report}",
            f"[ci-gate-summary] seamgrim_phase3_cleanup={seamgrim_phase3_cleanup_report}",
            "[ci-gate-summary] seamgrim_group_id_summary_status=ok",
            f"[ci-gate-summary] seamgrim_wasm_cli_diag_parity_report={seamgrim_wasm_cli_diag_parity_report}",
            "[ci-gate-summary] seamgrim_wasm_cli_diag_parity_ok=1",
            "[ci-gate-summary] ci_pack_golden_overlay_compare_selftest_ok=1",
            "[ci-gate-summary] ci_pack_golden_overlay_session_selftest_ok=1",
        ]
        append_age5_child_summary_lines(pass_lines, age5_close_report)
        append_age5_policy_summary_lines(pass_lines, aggregate_report)
        append_age4_proof_summary_lines(pass_lines, aggregate_report)
        append_ci_profile_matrix_selftest_summary_lines(pass_lines, ci_profile_matrix_gate_selftest_report)
        append_ci_sanity_summary_lines(pass_lines, ci_sanity_gate_report)
        append_ci_sync_readiness_summary_lines(pass_lines, ci_sync_readiness_report)
        append_fixed64_threeway_summary_lines(pass_lines, fixed64_threeway_gate_report)
        for line in pass_lines:
            print(line)
        write_summary(summary_path, pass_lines)
        write_summary_line(
            summary_line_path,
            "status=pass reason=ok failed_steps=0 aggregate_status=pass overall_ok=1 "
            f"age4_proof_ok={CONTRACT_ONLY_AGE4_PROOF_OK} "
            f"age4_proof_failed_criteria={CONTRACT_ONLY_AGE4_PROOF_FAILED_CRITERIA} "
            "age4_proof_failed_preview=- "
            f"age4_proof_summary_hash={CONTRACT_ONLY_AGE4_PROOF_SUMMARY_HASH}",
        )
        write_index(True, announce=False)
        print("[ci-gate] contract-only aggregate pass")
        return 0

    if run_core_tests:
        core_lint_rc = run_and_record(
            "core_fixed64_lint",
            ["cargo", "test", "-p", "ddonirang-core", "fixed64_lint_gate_no_float_in_core"],
        )
        if args.fast_fail and core_lint_rc != 0:
            return fail_and_exit(core_lint_rc, "[ci-gate] fast-fail: core fixed64 lint failed")
        core_all_rc = run_and_record(
            "core_tests",
            ["cargo", "test", "-p", "ddonirang-core"],
        )
        if args.fast_fail and core_all_rc != 0:
            return fail_and_exit(core_all_rc, "[ci-gate] fast-fail: core tests failed")

    backup_hygiene_move_rc = 0
    backup_hygiene_verify_rc = 0
    if args.backup_hygiene:
        backup_hygiene_move_rc = run_backup_hygiene_move()
        if args.fast_fail and backup_hygiene_move_rc != 0:
            return fail_and_exit(backup_hygiene_move_rc, "[ci-gate] fast-fail: backup hygiene move failed")
        backup_hygiene_verify_rc = run_backup_hygiene_verify()
        if args.fast_fail and backup_hygiene_verify_rc != 0:
            return fail_and_exit(backup_hygiene_verify_rc, "[ci-gate] fast-fail: backup hygiene verify failed")

    seamgrim_rc = 0
    age2_rc = 0
    age3_rc = 0
    age4_rc = 0
    age5_rc = 0
    oi_rc = 0
    if args.contract_only_aggregate:
        print("[ci-gate] contract-only aggregate: stub seamgrim/age/oi reports")
        write_contract_only_stub_reports(
            seamgrim_report,
            seamgrim_ui_age3_report,
            seamgrim_phase3_cleanup_report,
            seamgrim_browse_selection_report,
            age2_close_report,
            age3_close_report,
            age4_close_report,
            age5_close_report,
            age4_pack_report,
            oi_report,
            oi_pack_report,
        )
    else:
        seamgrim_cmd = [
            py,
            "tests/run_seamgrim_ci_gate.py",
            "--strict-graph",
            "--require-promoted",
            "--require-preview-synced",
            "--print-drilldown",
            "--json-out",
            str(seamgrim_report),
            "--ui-age3-json-out",
            str(seamgrim_ui_age3_report),
            "--phase3-cleanup-json-out",
            str(seamgrim_phase3_cleanup_report),
            "--browse-selection-json-out",
            str(seamgrim_browse_selection_report),
            "--rewrite-overlay-json-out",
            str(seamgrim_rewrite_overlay_quality_report),
            "--lesson-warning-report-json-out",
            str(seamgrim_lesson_warning_tokens_report),
        ]
        if args.lesson_warning_require_zero:
            seamgrim_cmd.append("--lesson-warning-require-zero")
        if args.browse_selection_strict:
            seamgrim_cmd.append("--browse-selection-strict")
        if args.with_runtime_5min:
            seamgrim_cmd.extend(
                [
                    "--with-runtime-5min",
                    "--runtime-5min-base-url",
                    "http://127.0.0.1:18787",
                    "--runtime-5min-json-out",
                    str(seamgrim_runtime_5min_report),
                    "--runtime-5min-browse-selection-json-out",
                    str(seamgrim_runtime_5min_browse_selection_report),
                ]
            )
            if args.runtime_5min_skip_seed_cli:
                seamgrim_cmd.append("--runtime-5min-skip-seed-cli")
            if args.runtime_5min_skip_ui_common:
                seamgrim_cmd.append("--runtime-5min-skip-ui-common")
            if args.runtime_5min_skip_showcase_check:
                seamgrim_cmd.append("--runtime-5min-skip-showcase-check")
            if args.runtime_5min_showcase_smoke:
                seamgrim_cmd.extend(
                    [
                        "--runtime-5min-showcase-smoke",
                        "--runtime-5min-showcase-smoke-madi-pendulum",
                        str(max(1, int(args.runtime_5min_showcase_smoke_madi_pendulum))),
                        "--runtime-5min-showcase-smoke-madi-tetris",
                        str(max(1, int(args.runtime_5min_showcase_smoke_madi_tetris))),
                    ]
                )
        if include_5min_checklist:
            seamgrim_cmd.append("--with-5min-checklist")
            seamgrim_cmd.extend(["--checklist-json-out", str(seamgrim_5min_checklist_report)])
            if args.checklist_skip_seed_cli:
                seamgrim_cmd.append("--checklist-skip-seed-cli")
            if args.checklist_skip_ui_common:
                seamgrim_cmd.append("--checklist-skip-ui-common")
        seamgrim_rc = run_and_record(
            "seamgrim_ci_gate",
            seamgrim_cmd,
        )
        if args.fast_fail and seamgrim_rc != 0:
            run_and_record(
                "seamgrim_digest",
                [
                    py,
                    "tools/scripts/print_seamgrim_ci_gate_digest.py",
                    str(seamgrim_report),
                    "--only-failed",
                ],
            )
            return fail_and_exit(seamgrim_rc, "[ci-gate] fast-fail: seamgrim gate failed")

        age2_rc = run_and_record(
            "age2_close",
            [
                py,
                "tests/run_age2_close.py",
                "--run-age2",
                "--report-out",
                str(age2_close_report),
            ],
        )
        if args.fast_fail and age2_rc != 0:
            return fail_and_exit(age2_rc, "[ci-gate] fast-fail: AGE2 close gate failed")

        age3_rc = run_and_record(
            "age3_close",
            [
                py,
                "tests/run_age3_close.py",
                "--seamgrim-report",
                str(seamgrim_report),
                "--ui-age3-report",
                str(seamgrim_ui_age3_report),
                "--report-out",
                str(age3_close_report),
            ],
        )
        if args.fast_fail and age3_rc != 0:
            run_and_record(
                "age3_close_digest",
                [
                    py,
                    "tools/scripts/print_age3_close_digest.py",
                    str(age3_close_report),
                    "--top",
                    "6",
                    "--only-failed",
                ],
            )
            return fail_and_exit(age3_rc, "[ci-gate] fast-fail: AGE3 close gate failed")
    age3_status_rc = render_age3_status()
    if args.fast_fail and age3_status_rc != 0:
        return fail_and_exit(age3_status_rc, "[ci-gate] fast-fail: AGE3 close status generation failed")
    age3_status_line_rc = render_age3_status_line()
    if args.fast_fail and age3_status_line_rc != 0:
        return fail_and_exit(age3_status_line_rc, "[ci-gate] fast-fail: AGE3 close status-line generation failed")
    age3_badge_rc = render_age3_badge()
    if args.fast_fail and age3_badge_rc != 0:
        return fail_and_exit(age3_badge_rc, "[ci-gate] fast-fail: AGE3 close badge generation failed")
    age3_status_line_parse_rc = parse_age3_status_line()
    if args.fast_fail and age3_status_line_parse_rc != 0:
        return fail_and_exit(age3_status_line_parse_rc, "[ci-gate] fast-fail: AGE3 status-line parse failed")
    age3_status_line_check_rc = check_age3_status_line(require_pass=True)
    if args.fast_fail and age3_status_line_check_rc != 0:
        return fail_and_exit(age3_status_line_check_rc, "[ci-gate] fast-fail: AGE3 status-line check failed")
    age3_badge_check_rc = check_age3_badge(require_pass=True)
    if args.fast_fail and age3_badge_check_rc != 0:
        return fail_and_exit(age3_badge_check_rc, "[ci-gate] fast-fail: AGE3 badge check failed")
    age3_summary_rc = render_age3_summary()
    if args.fast_fail and age3_summary_rc != 0:
        return fail_and_exit(age3_summary_rc, "[ci-gate] fast-fail: AGE3 close summary generation failed")

    if not args.contract_only_aggregate:
        age4_rc = run_and_record(
            "age4_close",
            [
                py,
                "tests/run_age4_close.py",
                "--report-out",
                str(age4_close_report),
                "--pack-report-out",
                str(age4_pack_report),
            ],
        )
        if args.fast_fail and age4_rc != 0:
            run_and_record(
                "age4_close_digest",
                [
                    py,
                    "tools/scripts/print_age4_close_digest.py",
                    str(age4_close_report),
                    "--top",
                    "6",
                    "--only-failed",
                ],
            )
            return fail_and_exit(age4_rc, "[ci-gate] fast-fail: AGE4 close gate failed")

        age5_rc = run_and_record(
            "age5_close",
            [
                py,
                "tests/run_age5_close.py",
                "--strict",
                "--report-out",
                str(age5_close_report),
            ],
        )
        if args.fast_fail and age5_rc != 0:
            run_and_record(
                "age5_close_digest",
                [
                    py,
                    "tools/scripts/print_age5_close_digest.py",
                    str(age5_close_report),
                    "--top",
                    "6",
                    "--only-failed",
                ],
            )
            return fail_and_exit(age5_rc, "[ci-gate] fast-fail: AGE5 close gate failed")

        oi_rc = run_and_record(
            "oi405_406_close",
            [
                py,
                "tests/run_oi405_406_close.py",
                "--strict",
                "--report-out",
                str(oi_report),
                "--pack-report-out",
                str(oi_pack_report),
            ],
        )
        if args.fast_fail and oi_rc != 0:
            run_and_record(
                "seamgrim_digest",
                [
                    py,
                    "tools/scripts/print_seamgrim_ci_gate_digest.py",
                    str(seamgrim_report),
                    "--only-failed",
                ],
            )
            run_and_record(
                "oi405_406_digest",
                [
                    py,
                    "tools/scripts/print_oi405_406_digest.py",
                    str(oi_report),
                    "--max-digest",
                    "5",
                    "--max-slowest",
                    "3",
                    "--only-failed",
                ],
            )
            return fail_and_exit(oi_rc, "[ci-gate] fast-fail: OI close gate failed")

    control_exposure_snapshot = write_control_exposure_failure_report(
        seamgrim_control_exposure_failures_report,
        seamgrim_report,
    )
    run_and_record(
        "seamgrim_digest",
        [
            py,
            "tools/scripts/print_seamgrim_ci_gate_digest.py",
            str(seamgrim_report),
            "--only-failed",
        ],
    )
    run_and_record(
        "age3_close_digest",
        [
            py,
            "tools/scripts/print_age3_close_digest.py",
            str(age3_close_report),
            "--top",
            "6",
            "--only-failed",
        ],
    )
    run_and_record(
        "age4_close_digest",
        [
            py,
            "tools/scripts/print_age4_close_digest.py",
            str(age4_close_report),
            "--top",
            "6",
            "--only-failed",
        ],
    )
    run_and_record(
        "age5_close_digest",
        [
            py,
            "tools/scripts/print_age5_close_digest.py",
            str(age5_close_report),
            "--top",
            "6",
            "--only-failed",
        ],
    )
    run_and_record(
        "oi405_406_digest",
        [
            py,
            "tools/scripts/print_oi405_406_digest.py",
            str(oi_report),
            "--max-digest",
            "5",
            "--max-slowest",
            "3",
            "--only-failed",
        ],
    )
    # combine 단계에서 index 링크를 바로 참조할 수 있도록 선기록.
    write_index(
        bool(
            backup_hygiene_move_rc == 0
            and backup_hygiene_verify_rc == 0
            and seamgrim_rc == 0
            and age3_rc == 0
            and age3_status_rc == 0
            and age3_status_line_rc == 0
            and age3_badge_rc == 0
            and age3_status_line_parse_rc == 0
            and age3_status_line_check_rc == 0
            and age3_badge_check_rc == 0
            and age3_summary_rc == 0
            and age4_rc == 0
            and age5_rc == 0
            and oi_rc == 0
        ),
        announce=False,
    )
    combine_cmd = [
        py,
        "tools/scripts/combine_ci_reports.py",
        "--print-summary",
        "--fail-on-bad",
        "--require-age3",
        "--require-age4",
        "--seamgrim-report",
        str(seamgrim_report),
        "--age3-report",
        str(age3_close_report),
        "--age4-report",
        str(age4_close_report),
        "--age5-report",
        str(age5_close_report),
        "--age3-status",
        str(age3_close_status_json),
        "--age3-status-line",
        str(age3_close_status_line),
        "--age3-badge",
        str(age3_close_badge_json),
        "--oi-report",
        str(oi_report),
        "--out",
        str(aggregate_report),
        "--index-report-path",
        str(index_report_path),
    ]
    if args.require_age5:
        combine_cmd.append("--require-age5")
    combine_rc = run_and_record("aggregate_combine", combine_cmd)
    # aggregate digest also surfaces age5_child_summary_defaults=/age5_sync_child_summary_defaults=
    # age5_full_real_source_check=/age5_full_real_source_selftest=
    # combined_digest_selftest_default_field_text=/combined_digest_selftest_default_field=
    # age5_policy_combined_digest_selftest_default_field_text=/age5_policy_combined_digest_selftest_default_field=
    # age5_policy_report=/age5_policy_text=
    # so top consumers can read the AGE5 default transport contract without opening the report.
    run_and_record(
        "aggregate_digest",
        [
            py,
            "tools/scripts/print_ci_aggregate_digest.py",
            str(aggregate_report),
            "--top",
            "1",
            "--only-failed",
            "--show-steps",
        ],
    )
    aggregate_status_line_rc = render_aggregate_status_line(fail_on_bad=True)
    if args.fast_fail and aggregate_status_line_rc != 0:
        return fail_and_exit(aggregate_status_line_rc, "[ci-gate] fast-fail: aggregate status-line generation failed")
    aggregate_status_line_parse_rc = parse_aggregate_status_line()
    if args.fast_fail and aggregate_status_line_parse_rc != 0:
        return fail_and_exit(aggregate_status_line_parse_rc, "[ci-gate] fast-fail: aggregate status-line parse failed")
    aggregate_status_line_check_rc = check_aggregate_status_line(require_pass=True)
    if args.fast_fail and aggregate_status_line_check_rc != 0:
        return fail_and_exit(aggregate_status_line_check_rc, "[ci-gate] fast-fail: aggregate status-line check failed")

    write_index(
        bool(
            combine_rc == 0
            and backup_hygiene_move_rc == 0
            and backup_hygiene_verify_rc == 0
            and seamgrim_rc == 0
            and age2_rc == 0
            and age3_rc == 0
            and age3_status_rc == 0
            and age3_status_line_rc == 0
            and age3_badge_rc == 0
            and age3_status_line_parse_rc == 0
            and age3_status_line_check_rc == 0
            and age3_badge_check_rc == 0
            and age3_summary_rc == 0
            and age4_rc == 0
            and age5_rc == 0
            and oi_rc == 0
            and aggregate_status_line_rc == 0
            and aggregate_status_line_parse_rc == 0
            and aggregate_status_line_check_rc == 0
        ),
        announce=False,
    )
    final_status_line_rc = render_final_status_line(fail_on_bad=True)
    if args.fast_fail and final_status_line_rc != 0:
        return fail_and_exit(final_status_line_rc, "[ci-gate] fast-fail: final status-line generation failed")
    final_status_line_check_rc = check_final_status_line(require_pass=True)
    if args.fast_fail and final_status_line_check_rc != 0:
        return fail_and_exit(final_status_line_check_rc, "[ci-gate] fast-fail: final status-line check failed")
    final_status_line_parse_rc = parse_final_status_line()
    if args.fast_fail and final_status_line_parse_rc != 0:
        return fail_and_exit(final_status_line_parse_rc, "[ci-gate] fast-fail: final status-line parse failed")
    ci_gate_result_rc = render_ci_gate_result(fail_on_bad=True)
    if args.fast_fail and ci_gate_result_rc != 0:
        return fail_and_exit(ci_gate_result_rc, "[ci-gate] fast-fail: ci gate result generation failed")
    ci_gate_result_check_rc = check_ci_gate_result(require_pass=True)
    if args.fast_fail and ci_gate_result_check_rc != 0:
        return fail_and_exit(ci_gate_result_check_rc, "[ci-gate] fast-fail: ci gate result check failed")
    ci_gate_result_parse_rc = parse_ci_gate_result(fail_on_fail=True)
    if args.fast_fail and ci_gate_result_parse_rc != 0:
        return fail_and_exit(ci_gate_result_parse_rc, "[ci-gate] fast-fail: ci gate result parse failed")
    ci_gate_badge_rc = render_ci_gate_badge(fail_on_bad=True)
    if args.fast_fail and ci_gate_badge_rc != 0:
        return fail_and_exit(ci_gate_badge_rc, "[ci-gate] fast-fail: ci gate badge generation failed")
    ci_gate_badge_check_rc = check_ci_gate_badge(require_pass=True)
    if args.fast_fail and ci_gate_badge_check_rc != 0:
        return fail_and_exit(ci_gate_badge_check_rc, "[ci-gate] fast-fail: ci gate badge check failed")
    summary_line_check_rc = check_summary_line(require_pass=True, use_result_parse=True)
    if args.fast_fail and summary_line_check_rc != 0:
        return fail_and_exit(summary_line_check_rc, "[ci-gate] fast-fail: summary-line check failed")
    ci_gate_outputs_consistency_rc = check_ci_gate_outputs_consistency(require_pass=True)
    if args.fast_fail and ci_gate_outputs_consistency_rc != 0:
        return fail_and_exit(ci_gate_outputs_consistency_rc, "[ci-gate] fast-fail: ci gate outputs consistency check failed")
    ci_final_line_emitter_rc = check_ci_final_line_emitter()
    if args.fast_fail and ci_final_line_emitter_rc != 0:
        return fail_and_exit(ci_final_line_emitter_rc, "[ci-gate] fast-fail: ci final line emitter check failed")
    ci_pipeline_emit_flags_rc = check_ci_pipeline_emit_flags()
    if args.fast_fail and ci_pipeline_emit_flags_rc != 0:
        return fail_and_exit(ci_pipeline_emit_flags_rc, "[ci-gate] fast-fail: ci pipeline emit flags check failed")
    ci_pipeline_emit_flags_selftest_rc = check_ci_pipeline_emit_flags_selftest()
    if args.fast_fail and ci_pipeline_emit_flags_selftest_rc != 0:
        return fail_and_exit(
            ci_pipeline_emit_flags_selftest_rc,
            "[ci-gate] fast-fail: ci pipeline emit flags selftest failed",
        )
    ci_profile_split_contract_rc = check_ci_profile_split_contract()
    if args.fast_fail and ci_profile_split_contract_rc != 0:
        return fail_and_exit(
            ci_profile_split_contract_rc,
            "[ci-gate] fast-fail: ci profile split contract check failed",
        )
    ci_profile_matrix_gate_selftest_rc = check_ci_profile_matrix_gate_selftest()
    if args.fast_fail and ci_profile_matrix_gate_selftest_rc != 0:
        return fail_and_exit(
            ci_profile_matrix_gate_selftest_rc,
            "[ci-gate] fast-fail: ci profile matrix gate selftest failed",
        )
    seamgrim_ci_gate_runtime5_passthrough_rc = check_seamgrim_ci_gate_runtime5_passthrough()
    if args.fast_fail and seamgrim_ci_gate_runtime5_passthrough_rc != 0:
        return fail_and_exit(
            seamgrim_ci_gate_runtime5_passthrough_rc,
            "[ci-gate] fast-fail: seamgrim ci gate runtime5 passthrough check failed",
        )
    seamgrim_ci_gate_preview_sync_passthrough_rc = check_seamgrim_ci_gate_preview_sync_passthrough()
    if args.fast_fail and seamgrim_ci_gate_preview_sync_passthrough_rc != 0:
        return fail_and_exit(
            seamgrim_ci_gate_preview_sync_passthrough_rc,
            "[ci-gate] fast-fail: seamgrim ci gate preview-sync passthrough check failed",
        )
    seamgrim_ci_gate_seed_meta_step_rc = check_seamgrim_ci_gate_seed_meta_step()
    if args.fast_fail and seamgrim_ci_gate_seed_meta_step_rc != 0:
        return fail_and_exit(
            seamgrim_ci_gate_seed_meta_step_rc,
            "[ci-gate] fast-fail: seamgrim ci gate seed-meta step check failed",
        )
    seamgrim_ci_gate_sam_seulgi_family_step_rc = check_seamgrim_ci_gate_sam_seulgi_family_step()
    if args.fast_fail and seamgrim_ci_gate_sam_seulgi_family_step_rc != 0:
        return fail_and_exit(
            seamgrim_ci_gate_sam_seulgi_family_step_rc,
            "[ci-gate] fast-fail: seamgrim ci gate sam seulgi family step check failed",
        )
    seamgrim_ci_gate_guideblock_step_rc = check_seamgrim_ci_gate_guideblock_step()
    if args.fast_fail and seamgrim_ci_gate_guideblock_step_rc != 0:
        return fail_and_exit(
            seamgrim_ci_gate_guideblock_step_rc,
            "[ci-gate] fast-fail: seamgrim ci gate guideblock step check failed",
        )
    seamgrim_ci_gate_lesson_warning_step_rc = check_seamgrim_ci_gate_lesson_warning_step()
    if args.fast_fail and seamgrim_ci_gate_lesson_warning_step_rc != 0:
        return fail_and_exit(
            seamgrim_ci_gate_lesson_warning_step_rc,
            "[ci-gate] fast-fail: seamgrim ci gate lesson warning step check failed",
        )
    seamgrim_ci_gate_stateful_preview_step_rc = check_seamgrim_ci_gate_stateful_preview_step()
    if args.fast_fail and seamgrim_ci_gate_stateful_preview_step_rc != 0:
        return fail_and_exit(
            seamgrim_ci_gate_stateful_preview_step_rc,
            "[ci-gate] fast-fail: seamgrim ci gate stateful preview step check failed",
        )
    seamgrim_ci_gate_wasm_web_smoke_step_rc = check_seamgrim_ci_gate_wasm_web_smoke_step()
    if args.fast_fail and seamgrim_ci_gate_wasm_web_smoke_step_rc != 0:
        return fail_and_exit(
            seamgrim_ci_gate_wasm_web_smoke_step_rc,
            "[ci-gate] fast-fail: seamgrim ci gate wasm/web smoke step check failed",
        )
    seamgrim_ci_gate_wasm_web_smoke_step_selftest_rc = check_seamgrim_ci_gate_wasm_web_smoke_step_selftest()
    if args.fast_fail and seamgrim_ci_gate_wasm_web_smoke_step_selftest_rc != 0:
        return fail_and_exit(
            seamgrim_ci_gate_wasm_web_smoke_step_selftest_rc,
            "[ci-gate] fast-fail: seamgrim ci gate wasm/web smoke step check selftest failed",
        )
    ci_sanity_gate_rc = check_ci_sanity_gate()
    if args.fast_fail and ci_sanity_gate_rc != 0:
        return fail_and_exit(
            ci_sanity_gate_rc,
            "[ci-gate] fast-fail: ci sanity gate execution failed",
        )
    ci_sync_readiness_selftest_rc = check_ci_sync_readiness_selftest()
    if args.fast_fail and ci_sync_readiness_selftest_rc != 0:
        return fail_and_exit(
            ci_sync_readiness_selftest_rc,
            "[ci-gate] fast-fail: ci sync readiness selftest failed",
        )
    ci_sync_readiness_diagnostics_rc = check_ci_sync_readiness_diagnostics()
    if args.fast_fail and ci_sync_readiness_diagnostics_rc != 0:
        return fail_and_exit(
            ci_sync_readiness_diagnostics_rc,
            "[ci-gate] fast-fail: ci sync readiness diagnostics check failed",
        )
    ci_sync_readiness_report_selftest_rc = check_ci_sync_readiness_report_selftest()
    if args.fast_fail and ci_sync_readiness_report_selftest_rc != 0:
        return fail_and_exit(
            ci_sync_readiness_report_selftest_rc,
            "[ci-gate] fast-fail: ci sync readiness report selftest failed",
        )
    ci_sync_readiness_report_generate_rc = run_ci_sync_readiness_report_generate()
    if args.fast_fail and ci_sync_readiness_report_generate_rc != 0:
        return fail_and_exit(
            ci_sync_readiness_report_generate_rc,
            "[ci-gate] fast-fail: ci sync readiness report generation failed",
        )
    ci_sync_readiness_report_check_rc = check_ci_sync_readiness_report_check()
    if args.fast_fail and ci_sync_readiness_report_check_rc != 0:
        return fail_and_exit(
            ci_sync_readiness_report_check_rc,
            "[ci-gate] fast-fail: ci sync readiness report check failed",
        )
    ci_builtin_name_sync_rc = check_ci_builtin_name_sync()
    if args.fast_fail and ci_builtin_name_sync_rc != 0:
        return fail_and_exit(ci_builtin_name_sync_rc, "[ci-gate] fast-fail: ci builtin name sync check failed")
    ci_fixed64_probe_selftest_rc = check_ci_fixed64_probe_selftest()
    if args.fast_fail and ci_fixed64_probe_selftest_rc != 0:
        return fail_and_exit(
            ci_fixed64_probe_selftest_rc,
            "[ci-gate] fast-fail: ci fixed64 probe selftest failed",
        )
    ci_fixed64_win_wsl_matrix_selftest_rc = check_ci_fixed64_win_wsl_matrix_selftest()
    if args.fast_fail and ci_fixed64_win_wsl_matrix_selftest_rc != 0:
        return fail_and_exit(
            ci_fixed64_win_wsl_matrix_selftest_rc,
            "[ci-gate] fast-fail: ci fixed64 windows+wsl matrix selftest failed",
        )
    ci_fixed64_threeway_inputs_selftest_rc = check_ci_fixed64_threeway_inputs_selftest()
    if args.fast_fail and ci_fixed64_threeway_inputs_selftest_rc != 0:
        return fail_and_exit(
            ci_fixed64_threeway_inputs_selftest_rc,
            "[ci-gate] fast-fail: ci fixed64 threeway inputs selftest failed",
        )
    ci_fixed64_darwin_probe_artifact_selftest_rc = check_ci_fixed64_darwin_probe_artifact_selftest()
    if args.fast_fail and ci_fixed64_darwin_probe_artifact_selftest_rc != 0:
        return fail_and_exit(
            ci_fixed64_darwin_probe_artifact_selftest_rc,
            "[ci-gate] fast-fail: ci fixed64 darwin probe artifact selftest failed",
        )
    ci_fixed64_threeway_gate_rc = check_ci_fixed64_threeway_gate(require_darwin=args.require_fixed64_3way)
    if args.fast_fail and ci_fixed64_threeway_gate_rc != 0:
        return fail_and_exit(
            ci_fixed64_threeway_gate_rc,
            "[ci-gate] fast-fail: ci fixed64 threeway gate failed",
        )
    ci_fixed64_threeway_gate_selftest_rc = check_ci_fixed64_threeway_gate_selftest()
    if args.fast_fail and ci_fixed64_threeway_gate_selftest_rc != 0:
        return fail_and_exit(
            ci_fixed64_threeway_gate_selftest_rc,
            "[ci-gate] fast-fail: ci fixed64 threeway gate selftest failed",
        )
    ci_backup_hygiene_selftest_rc = check_ci_backup_hygiene_selftest()
    if args.fast_fail and ci_backup_hygiene_selftest_rc != 0:
        return fail_and_exit(
            ci_backup_hygiene_selftest_rc,
            "[ci-gate] fast-fail: ci backup hygiene selftest failed",
        )
    write_emit_artifacts_summary_preview()
    ci_emit_artifacts_generate_rc = emit_ci_final_line_for_artifacts()
    if args.fast_fail and ci_emit_artifacts_generate_rc != 0:
        return fail_and_exit(ci_emit_artifacts_generate_rc, "[ci-gate] fast-fail: ci emit artifacts generate failed")
    ci_emit_artifacts_baseline_rc = check_ci_emit_artifacts_baseline()
    if args.fast_fail and ci_emit_artifacts_baseline_rc != 0:
        return fail_and_exit(ci_emit_artifacts_baseline_rc, "[ci-gate] fast-fail: ci emit artifacts baseline check failed")
    ci_emit_artifacts_required_rc = check_ci_emit_artifacts_required()
    if args.fast_fail and ci_emit_artifacts_required_rc != 0:
        return fail_and_exit(ci_emit_artifacts_required_rc, "[ci-gate] fast-fail: ci emit artifacts required check failed")
    ci_emit_artifacts_selftest_rc = check_ci_emit_artifacts_selftest()
    if args.fast_fail and ci_emit_artifacts_selftest_rc != 0:
        return fail_and_exit(ci_emit_artifacts_selftest_rc, "[ci-gate] fast-fail: ci emit artifacts selftest failed")
    ci_emit_artifacts_sanity_contract_rc = check_ci_emit_artifacts_sanity_contract()
    if args.fast_fail and ci_emit_artifacts_sanity_contract_rc != 0:
        return fail_and_exit(
            ci_emit_artifacts_sanity_contract_rc,
            "[ci-gate] fast-fail: ci emit artifacts sanity contract check failed",
        )
    ci_emit_artifacts_sanity_contract_selftest_rc = check_ci_emit_artifacts_sanity_contract_selftest()
    if args.fast_fail and ci_emit_artifacts_sanity_contract_selftest_rc != 0:
        return fail_and_exit(
            ci_emit_artifacts_sanity_contract_selftest_rc,
            "[ci-gate] fast-fail: ci emit artifacts sanity contract selftest failed",
        )
    ci_aggregate_gate_age4_diagnostics_rc = check_ci_aggregate_gate_age4_diagnostics()
    if args.fast_fail and ci_aggregate_gate_age4_diagnostics_rc != 0:
        return fail_and_exit(
            ci_aggregate_gate_age4_diagnostics_rc,
            "[ci-gate] fast-fail: ci aggregate gate age4 diagnostics check failed",
        )
    ci_aggregate_gate_age5_diagnostics_rc = check_ci_aggregate_gate_age5_diagnostics()
    if args.fast_fail and ci_aggregate_gate_age5_diagnostics_rc != 0:
        return fail_and_exit(
            ci_aggregate_gate_age5_diagnostics_rc,
            "[ci-gate] fast-fail: ci aggregate gate age5 diagnostics check failed",
        )
    ci_aggregate_gate_phase3_diagnostics_rc = check_ci_aggregate_gate_phase3_diagnostics()
    if args.fast_fail and ci_aggregate_gate_phase3_diagnostics_rc != 0:
        return fail_and_exit(
            ci_aggregate_gate_phase3_diagnostics_rc,
            "[ci-gate] fast-fail: ci aggregate gate phase3 diagnostics check failed",
        )
    ci_aggregate_gate_runtime5_diagnostics_rc = check_ci_aggregate_gate_runtime5_diagnostics()
    if args.fast_fail and ci_aggregate_gate_runtime5_diagnostics_rc != 0:
        return fail_and_exit(
            ci_aggregate_gate_runtime5_diagnostics_rc,
            "[ci-gate] fast-fail: ci aggregate gate runtime5 diagnostics check failed",
        )
    ci_aggregate_gate_seamgrim_diagnostics_rc = check_ci_aggregate_gate_seamgrim_diagnostics()
    if args.fast_fail and ci_aggregate_gate_seamgrim_diagnostics_rc != 0:
        return fail_and_exit(
            ci_aggregate_gate_seamgrim_diagnostics_rc,
            "[ci-gate] fast-fail: ci aggregate gate seamgrim diagnostics check failed",
        )
    ci_aggregate_gate_sync_diagnostics_rc = check_ci_aggregate_gate_sync_diagnostics()
    if args.fast_fail and ci_aggregate_gate_sync_diagnostics_rc != 0:
        return fail_and_exit(
            ci_aggregate_gate_sync_diagnostics_rc,
            "[ci-gate] fast-fail: ci aggregate gate sync diagnostics check failed",
        )
    ci_aggregate_gate_sanity_diagnostics_rc = check_ci_aggregate_gate_sanity_diagnostics()
    if args.fast_fail and ci_aggregate_gate_sanity_diagnostics_rc != 0:
        return fail_and_exit(
            ci_aggregate_gate_sanity_diagnostics_rc,
            "[ci-gate] fast-fail: ci aggregate gate sanity diagnostics check failed",
        )
    ci_sanity_gate_diagnostics_rc = check_ci_sanity_gate_diagnostics()
    if args.fast_fail and ci_sanity_gate_diagnostics_rc != 0:
        return fail_and_exit(
            ci_sanity_gate_diagnostics_rc,
            "[ci-gate] fast-fail: ci sanity gate diagnostics check failed",
        )
    ci_gate_summary_report_selftest_rc = check_ci_gate_summary_report_selftest()
    if args.fast_fail and ci_gate_summary_report_selftest_rc != 0:
        return fail_and_exit(
            ci_gate_summary_report_selftest_rc,
            "[ci-gate] fast-fail: ci gate summary report selftest failed",
        )
    ci_fail_and_exit_contract_selftest_rc = check_ci_fail_and_exit_contract_selftest()
    if args.fast_fail and ci_fail_and_exit_contract_selftest_rc != 0:
        return fail_and_exit(
            ci_fail_and_exit_contract_selftest_rc,
            "[ci-gate] fast-fail: fail_and_exit contract selftest failed",
        )
    ci_gate_report_index_selftest_rc = check_ci_gate_report_index_selftest()
    if args.fast_fail and ci_gate_report_index_selftest_rc != 0:
        return fail_and_exit(
            ci_gate_report_index_selftest_rc,
            "[ci-gate] fast-fail: ci gate report-index selftest failed",
        )
    ci_gate_report_index_diagnostics_rc = check_ci_gate_report_index_diagnostics()
    if args.fast_fail and ci_gate_report_index_diagnostics_rc != 0:
        return fail_and_exit(
            ci_gate_report_index_diagnostics_rc,
            "[ci-gate] fast-fail: ci gate report-index diagnostics check failed",
        )
    ci_gate_report_index_latest_smoke_rc = 0
    ci_aggregate_status_line_selftest_rc = check_ci_aggregate_status_line_selftest()
    if args.fast_fail and ci_aggregate_status_line_selftest_rc != 0:
        return fail_and_exit(
            ci_aggregate_status_line_selftest_rc,
            "[ci-gate] fast-fail: ci aggregate status-line selftest failed",
        )
    ci_combine_reports_age4_selftest_rc = check_ci_combine_reports_age4_selftest()
    if args.fast_fail and ci_combine_reports_age4_selftest_rc != 0:
        return fail_and_exit(
            ci_combine_reports_age4_selftest_rc,
            "[ci-gate] fast-fail: ci combine age4 selftest failed",
        )
    ci_combine_reports_age5_selftest_rc = check_ci_combine_reports_age5_selftest()
    if args.fast_fail and ci_combine_reports_age5_selftest_rc != 0:
        return fail_and_exit(
            ci_combine_reports_age5_selftest_rc,
            "[ci-gate] fast-fail: ci combine age5 selftest failed",
        )
    age5_close_digest_selftest_rc = check_age5_close_digest_selftest()
    if args.fast_fail and age5_close_digest_selftest_rc != 0:
        return fail_and_exit(
            age5_close_digest_selftest_rc,
            "[ci-gate] fast-fail: age5 close digest selftest failed",
        )
    ci_pack_golden_overlay_compare_selftest_rc = check_ci_pack_golden_overlay_compare_selftest()
    if args.fast_fail and ci_pack_golden_overlay_compare_selftest_rc != 0:
        return fail_and_exit(
            ci_pack_golden_overlay_compare_selftest_rc,
            "[ci-gate] fast-fail: ci pack golden overlay compare selftest failed",
        )
    ci_pack_golden_overlay_session_selftest_rc = check_ci_pack_golden_overlay_session_selftest()
    if args.fast_fail and ci_pack_golden_overlay_session_selftest_rc != 0:
        return fail_and_exit(
            ci_pack_golden_overlay_session_selftest_rc,
            "[ci-gate] fast-fail: ci pack golden overlay session selftest failed",
        )
    ci_pack_golden_guideblock_selftest_rc = check_ci_pack_golden_guideblock_selftest()
    if args.fast_fail and ci_pack_golden_guideblock_selftest_rc != 0:
        return fail_and_exit(
            ci_pack_golden_guideblock_selftest_rc,
            "[ci-gate] fast-fail: ci pack golden guideblock selftest failed",
        )
    ci_pack_golden_age5_surface_selftest_rc = check_ci_pack_golden_age5_surface_selftest()
    if args.fast_fail and ci_pack_golden_age5_surface_selftest_rc != 0:
        return fail_and_exit(
            ci_pack_golden_age5_surface_selftest_rc,
            "[ci-gate] fast-fail: ci pack golden age5 surface selftest failed",
        )
    ci_pack_golden_exec_policy_selftest_rc = check_ci_pack_golden_exec_policy_selftest()
    if args.fast_fail and ci_pack_golden_exec_policy_selftest_rc != 0:
        return fail_and_exit(
            ci_pack_golden_exec_policy_selftest_rc,
            "[ci-gate] fast-fail: ci pack golden exec policy selftest failed",
        )
    ci_pack_golden_jjaim_flatten_selftest_rc = check_ci_pack_golden_jjaim_flatten_selftest()
    if args.fast_fail and ci_pack_golden_jjaim_flatten_selftest_rc != 0:
        return fail_and_exit(
            ci_pack_golden_jjaim_flatten_selftest_rc,
            "[ci-gate] fast-fail: ci pack golden jjaim flatten selftest failed",
        )
    ci_pack_golden_event_model_selftest_rc = check_ci_pack_golden_event_model_selftest()
    if args.fast_fail and ci_pack_golden_event_model_selftest_rc != 0:
        return fail_and_exit(
            ci_pack_golden_event_model_selftest_rc,
            "[ci-gate] fast-fail: ci pack golden event model selftest failed",
        )
    seamgrim_wasm_cli_diag_parity_rc = check_seamgrim_wasm_cli_diag_parity()
    if args.fast_fail and seamgrim_wasm_cli_diag_parity_rc != 0:
        return fail_and_exit(
            seamgrim_wasm_cli_diag_parity_rc,
            "[ci-gate] fast-fail: seamgrim wasm/cli diag parity check failed",
        )
    seamgrim_browse_selection_report_selftest_rc = check_seamgrim_browse_selection_report_selftest()
    if args.fast_fail and seamgrim_browse_selection_report_selftest_rc != 0:
        return fail_and_exit(
            seamgrim_browse_selection_report_selftest_rc,
            "[ci-gate] fast-fail: seamgrim browse selection report selftest failed",
        )
    seamgrim_5min_checklist_selftest_rc = check_seamgrim_5min_checklist_selftest()
    if args.fast_fail and seamgrim_5min_checklist_selftest_rc != 0:
        return fail_and_exit(
            seamgrim_5min_checklist_selftest_rc,
            "[ci-gate] fast-fail: seamgrim 5min checklist selftest failed",
        )
    ci_gate_failure_summary_selftest_rc = check_ci_gate_failure_summary_selftest()
    if args.fast_fail and ci_gate_failure_summary_selftest_rc != 0:
        return fail_and_exit(
            ci_gate_failure_summary_selftest_rc,
            "[ci-gate] fast-fail: ci gate failure summary selftest failed",
        )
    write_index(
        bool(
            combine_rc == 0
            and backup_hygiene_move_rc == 0
            and backup_hygiene_verify_rc == 0
            and seamgrim_rc == 0
            and age2_rc == 0
            and age3_rc == 0
            and age3_status_rc == 0
            and age3_status_line_rc == 0
            and age3_badge_rc == 0
            and age3_status_line_parse_rc == 0
            and age3_status_line_check_rc == 0
            and age3_badge_check_rc == 0
            and age3_summary_rc == 0
            and age4_rc == 0
            and age5_rc == 0
            and oi_rc == 0
            and aggregate_status_line_rc == 0
            and aggregate_status_line_parse_rc == 0
            and aggregate_status_line_check_rc == 0
            and final_status_line_rc == 0
            and final_status_line_check_rc == 0
            and final_status_line_parse_rc == 0
            and summary_line_check_rc == 0
            and ci_gate_result_rc == 0
            and ci_gate_result_check_rc == 0
            and ci_gate_result_parse_rc == 0
            and ci_gate_badge_rc == 0
            and ci_gate_badge_check_rc == 0
            and ci_gate_outputs_consistency_rc == 0
            and ci_final_line_emitter_rc == 0
            and ci_pipeline_emit_flags_rc == 0
            and ci_pipeline_emit_flags_selftest_rc == 0
            and ci_profile_split_contract_rc == 0
            and ci_profile_matrix_gate_selftest_rc == 0
            and seamgrim_ci_gate_runtime5_passthrough_rc == 0
            and seamgrim_ci_gate_seed_meta_step_rc == 0
            and seamgrim_ci_gate_guideblock_step_rc == 0
            and seamgrim_ci_gate_lesson_warning_step_rc == 0
            and seamgrim_ci_gate_stateful_preview_step_rc == 0
            and seamgrim_ci_gate_wasm_web_smoke_step_rc == 0
            and ci_sanity_gate_rc == 0
            and ci_sync_readiness_selftest_rc == 0
            and ci_sync_readiness_diagnostics_rc == 0
            and ci_sync_readiness_report_selftest_rc == 0
            and ci_sync_readiness_report_generate_rc == 0
            and ci_sync_readiness_report_check_rc == 0
            and ci_builtin_name_sync_rc == 0
            and ci_fixed64_probe_selftest_rc == 0
            and ci_fixed64_win_wsl_matrix_selftest_rc == 0
            and ci_fixed64_threeway_inputs_selftest_rc == 0
            and ci_fixed64_darwin_probe_artifact_selftest_rc == 0
            and ci_fixed64_threeway_gate_rc == 0
            and ci_fixed64_threeway_gate_selftest_rc == 0
            and ci_backup_hygiene_selftest_rc == 0
            and ci_emit_artifacts_baseline_rc == 0
            and ci_emit_artifacts_generate_rc == 0
            and ci_emit_artifacts_required_rc == 0
            and ci_emit_artifacts_selftest_rc == 0
            and ci_emit_artifacts_sanity_contract_rc == 0
            and ci_emit_artifacts_sanity_contract_selftest_rc == 0
            and ci_aggregate_gate_age4_diagnostics_rc == 0
            and ci_aggregate_gate_age5_diagnostics_rc == 0
            and ci_aggregate_gate_phase3_diagnostics_rc == 0
            and ci_aggregate_gate_runtime5_diagnostics_rc == 0
            and ci_aggregate_gate_seamgrim_diagnostics_rc == 0
            and ci_aggregate_gate_sync_diagnostics_rc == 0
            and ci_aggregate_gate_sanity_diagnostics_rc == 0
            and ci_sanity_gate_diagnostics_rc == 0
            and ci_gate_summary_report_selftest_rc == 0
            and ci_fail_and_exit_contract_selftest_rc == 0
            and ci_gate_report_index_selftest_rc == 0
            and ci_gate_report_index_diagnostics_rc == 0
            and ci_gate_report_index_latest_smoke_rc == 0
            and ci_aggregate_status_line_selftest_rc == 0
            and ci_combine_reports_age4_selftest_rc == 0
            and ci_combine_reports_age5_selftest_rc == 0
            and age5_close_digest_selftest_rc == 0
            and ci_pack_golden_overlay_compare_selftest_rc == 0
            and ci_pack_golden_overlay_session_selftest_rc == 0
            and ci_pack_golden_guideblock_selftest_rc == 0
            and ci_pack_golden_age5_surface_selftest_rc == 0
            and ci_pack_golden_exec_policy_selftest_rc == 0
            and ci_pack_golden_jjaim_flatten_selftest_rc == 0
            and ci_pack_golden_event_model_selftest_rc == 0
            and seamgrim_wasm_cli_diag_parity_rc == 0
            and seamgrim_browse_selection_report_selftest_rc == 0
            and seamgrim_5min_checklist_selftest_rc == 0
            and ci_gate_failure_summary_selftest_rc == 0
        )
    )
    status_outputs_refresh_rc = refresh_status_outputs_for_index()
    if args.fast_fail and status_outputs_refresh_rc != 0:
        return fail_and_exit(
            status_outputs_refresh_rc,
            "[ci-gate] fast-fail: status outputs refresh failed",
        )
    has_failed_steps = any(
        isinstance(row, dict) and not bool(row.get("ok", False))
        for row in steps_log
    )
    if has_failed_steps:
        ci_gate_report_index_rc = 0
        print("[ci-gate] skip early report-index check (pending failure summary regeneration)")
    else:
        ci_gate_report_index_rc = check_ci_gate_report_index(require_step_contract=False)
        if args.fast_fail and ci_gate_report_index_rc != 0:
            return fail_and_exit(
                ci_gate_report_index_rc,
                "[ci-gate] fast-fail: ci gate report-index check failed",
            )
        if ci_gate_report_index_rc != 0:
            discard_step_log("ci_gate_report_index_check")

    ci_gate_report_index_latest_smoke_rc = run_ci_gate_report_index_latest_smoke_step(
        has_failed_steps,
        ci_gate_report_index_rc,
    )
    if args.fast_fail and ci_gate_report_index_latest_smoke_rc != 0:
        return fail_and_exit(
            ci_gate_report_index_latest_smoke_rc,
            "[ci-gate] fast-fail: ci gate report-index latest smoke check failed",
        )

    if combine_rc != 0:
        lines = print_failure_block(
            steps_log,
            seamgrim_report,
            age3_close_report,
            age4_close_report,
            age5_close_report,
            oi_report,
            aggregate_report,
        )
        lines.append(f"[ci-gate-summary] age2_status={age2_close_report}")
        lines.append(f"[ci-gate-summary] age4_status={age4_close_report}")
        lines.append(f"[ci-gate-summary] age5_status={age5_close_report}")
        append_age5_child_summary_lines(lines, age5_close_report)
        append_age5_policy_summary_lines(lines, aggregate_report)
        append_age4_proof_summary_lines(lines, aggregate_report)
        lines.append(f"[ci-gate-summary] seamgrim_phase3_cleanup={seamgrim_phase3_cleanup_report}")
        append_runtime_5min_summary_lines(
            lines,
            bool(args.with_runtime_5min),
            seamgrim_runtime_5min_report,
            seamgrim_runtime_5min_browse_selection_report,
        )
        append_runtime_5min_checklist_summary_lines(
            lines,
            include_5min_checklist,
            seamgrim_5min_checklist_report,
        )
        control_exposure_snapshot = append_seamgrim_focus_summary_lines(
            lines,
            seamgrim_report,
            seamgrim_control_exposure_failures_report,
            seamgrim_rewrite_overlay_quality_report,
            control_exposure_snapshot,
        )
        append_ci_sanity_summary_lines(lines, ci_sanity_gate_report)
        append_ci_sync_readiness_summary_lines(lines, ci_sync_readiness_report)
        append_fixed64_threeway_summary_lines(lines, fixed64_threeway_gate_report)
        lines.append(f"[ci-gate-summary] aggregate_status_line={aggregate_status_line}")
        lines.append(f"[ci-gate-summary] aggregate_status_parse={aggregate_status_parse_json}")
        lines.append(f"[ci-gate-summary] aggregate_status_compact={read_compact_line(aggregate_status_line)}")
        lines.append(f"[ci-gate-summary] final_status_line={final_status_line}")
        lines.append(f"[ci-gate-summary] final_status_parse={final_status_parse_json}")
        lines.append(f"[ci-gate-summary] summary_line={summary_line_path}")
        lines.append(f"[ci-gate-summary] ci_gate_result={ci_gate_result_json}")
        lines.append(f"[ci-gate-summary] ci_gate_result_parse={ci_gate_result_parse_json}")
        lines.append(f"[ci-gate-summary] ci_gate_result_line={ci_gate_result_line_path}")
        lines.append(f"[ci-gate-summary] ci_gate_badge={ci_gate_badge_json}")
        lines.append(f"[ci-gate-summary] ci_fail_brief_hint={ci_fail_brief_txt}")
        lines.append(f"[ci-gate-summary] ci_fail_brief_exists={int(ci_fail_brief_txt.exists())}")
        lines.append(f"[ci-gate-summary] ci_fail_triage_hint={ci_fail_triage_json}")
        lines.append(f"[ci-gate-summary] ci_fail_triage_exists={int(ci_fail_triage_json.exists())}")
        lines.append(f"[ci-gate-summary] final_status_compact={read_compact_line(final_status_line)}")
        for line in lines:
            print(line)
        if lines:
            write_summary(summary_path, lines)
            emit_required_post_summary_rc = check_ci_emit_artifacts_required_post_summary()
            if emit_required_post_summary_rc != 0:
                print("[ci-gate] final summary emit artifacts required check failed", file=sys.stderr)
                return 2
            write_index(False, announce=False)
            ci_gate_report_index_post_summary_rc = check_ci_gate_report_index(require_step_contract=True)
            if ci_gate_report_index_post_summary_rc != 0:
                print("[ci-gate] report-index post-summary strict check failed", file=sys.stderr)
                return 2
            check_ci_gate_summary_report(require_pass=False)
            check_ci_gate_failure_summary(require_pass=False)
        summary_compact = resolve_summary_compact(
            ci_gate_result_line_path,
            final_status_parse_json,
            final_status_line,
        )
        write_summary_line(summary_line_path, summary_compact)
        print(f"[ci-gate-summary-line] {summary_compact}")
        return combine_rc
    if (
        backup_hygiene_move_rc != 0
        or backup_hygiene_verify_rc != 0
        or seamgrim_rc != 0
        or age2_rc != 0
        or age3_rc != 0
        or age3_status_rc != 0
        or age3_status_line_rc != 0
        or age3_badge_rc != 0
        or age3_status_line_parse_rc != 0
        or age3_status_line_check_rc != 0
        or age3_badge_check_rc != 0
        or age3_summary_rc != 0
        or age4_rc != 0
        or age5_rc != 0
        or oi_rc != 0
        or aggregate_status_line_rc != 0
        or aggregate_status_line_parse_rc != 0
        or aggregate_status_line_check_rc != 0
        or final_status_line_rc != 0
        or final_status_line_check_rc != 0
        or final_status_line_parse_rc != 0
        or summary_line_check_rc != 0
        or ci_gate_result_rc != 0
        or ci_gate_result_check_rc != 0
        or ci_gate_result_parse_rc != 0
        or ci_gate_badge_rc != 0
        or ci_gate_badge_check_rc != 0
        or ci_gate_outputs_consistency_rc != 0
        or ci_final_line_emitter_rc != 0
        or ci_pipeline_emit_flags_rc != 0
        or ci_pipeline_emit_flags_selftest_rc != 0
        or ci_profile_split_contract_rc != 0
        or ci_profile_matrix_gate_selftest_rc != 0
        or seamgrim_ci_gate_runtime5_passthrough_rc != 0
        or seamgrim_ci_gate_preview_sync_passthrough_rc != 0
        or seamgrim_ci_gate_seed_meta_step_rc != 0
        or seamgrim_ci_gate_guideblock_step_rc != 0
        or seamgrim_ci_gate_lesson_warning_step_rc != 0
        or seamgrim_ci_gate_stateful_preview_step_rc != 0
        or seamgrim_ci_gate_wasm_web_smoke_step_rc != 0
        or ci_sanity_gate_rc != 0
        or ci_sync_readiness_selftest_rc != 0
        or ci_sync_readiness_diagnostics_rc != 0
        or ci_sync_readiness_report_selftest_rc != 0
        or ci_sync_readiness_report_generate_rc != 0
        or ci_sync_readiness_report_check_rc != 0
        or ci_builtin_name_sync_rc != 0
        or ci_fixed64_probe_selftest_rc != 0
        or ci_fixed64_win_wsl_matrix_selftest_rc != 0
        or ci_fixed64_threeway_inputs_selftest_rc != 0
        or ci_fixed64_darwin_probe_artifact_selftest_rc != 0
        or ci_fixed64_threeway_gate_rc != 0
        or ci_fixed64_threeway_gate_selftest_rc != 0
        or ci_backup_hygiene_selftest_rc != 0
        or ci_emit_artifacts_baseline_rc != 0
        or ci_emit_artifacts_generate_rc != 0
        or ci_emit_artifacts_required_rc != 0
        or ci_emit_artifacts_selftest_rc != 0
        or ci_emit_artifacts_sanity_contract_rc != 0
        or ci_emit_artifacts_sanity_contract_selftest_rc != 0
        or ci_aggregate_gate_age4_diagnostics_rc != 0
        or ci_aggregate_gate_age5_diagnostics_rc != 0
        or ci_aggregate_gate_phase3_diagnostics_rc != 0
        or ci_aggregate_gate_runtime5_diagnostics_rc != 0
        or ci_aggregate_gate_seamgrim_diagnostics_rc != 0
        or ci_aggregate_gate_sync_diagnostics_rc != 0
        or ci_aggregate_gate_sanity_diagnostics_rc != 0
        or ci_sanity_gate_diagnostics_rc != 0
        or ci_gate_summary_report_selftest_rc != 0
        or ci_fail_and_exit_contract_selftest_rc != 0
        or ci_gate_report_index_selftest_rc != 0
        or ci_gate_report_index_diagnostics_rc != 0
        or ci_gate_report_index_latest_smoke_rc != 0
        or ci_gate_report_index_rc != 0
        or seamgrim_ci_gate_sam_seulgi_family_step_rc != 0
        or ci_aggregate_status_line_selftest_rc != 0
        or ci_combine_reports_age4_selftest_rc != 0
        or ci_combine_reports_age5_selftest_rc != 0
        or age5_close_digest_selftest_rc != 0
        or ci_pack_golden_overlay_compare_selftest_rc != 0
        or ci_pack_golden_overlay_session_selftest_rc != 0
        or ci_pack_golden_guideblock_selftest_rc != 0
        or ci_pack_golden_age5_surface_selftest_rc != 0
        or ci_pack_golden_exec_policy_selftest_rc != 0
        or ci_pack_golden_jjaim_flatten_selftest_rc != 0
        or ci_pack_golden_event_model_selftest_rc != 0
        or seamgrim_wasm_cli_diag_parity_rc != 0
        or seamgrim_browse_selection_report_selftest_rc != 0
        or seamgrim_5min_checklist_selftest_rc != 0
        or ci_gate_failure_summary_selftest_rc != 0
        or status_outputs_refresh_rc != 0
    ):
        print("[ci-gate] aggregate reported success but sub-step failed", file=sys.stderr)
        lines = print_failure_block(
            steps_log,
            seamgrim_report,
            age3_close_report,
            age4_close_report,
            age5_close_report,
            oi_report,
            aggregate_report,
        )
        lines.append(f"[ci-gate-summary] age2_status={age2_close_report}")
        lines.append(f"[ci-gate-summary] age4_status={age4_close_report}")
        lines.append(f"[ci-gate-summary] age5_status={age5_close_report}")
        append_age5_child_summary_lines(lines, age5_close_report)
        append_age5_policy_summary_lines(lines, aggregate_report)
        append_age4_proof_summary_lines(lines, aggregate_report)
        lines.append(f"[ci-gate-summary] seamgrim_phase3_cleanup={seamgrim_phase3_cleanup_report}")
        append_runtime_5min_summary_lines(
            lines,
            bool(args.with_runtime_5min),
            seamgrim_runtime_5min_report,
            seamgrim_runtime_5min_browse_selection_report,
        )
        append_runtime_5min_checklist_summary_lines(
            lines,
            include_5min_checklist,
            seamgrim_5min_checklist_report,
        )
        control_exposure_snapshot = append_seamgrim_focus_summary_lines(
            lines,
            seamgrim_report,
            seamgrim_control_exposure_failures_report,
            seamgrim_rewrite_overlay_quality_report,
            control_exposure_snapshot,
        )
        append_ci_sanity_summary_lines(lines, ci_sanity_gate_report)
        append_ci_sync_readiness_summary_lines(lines, ci_sync_readiness_report)
        append_fixed64_threeway_summary_lines(lines, fixed64_threeway_gate_report)
        lines.append(f"[ci-gate-summary] aggregate_status_line={aggregate_status_line}")
        lines.append(f"[ci-gate-summary] aggregate_status_parse={aggregate_status_parse_json}")
        lines.append(f"[ci-gate-summary] aggregate_status_compact={read_compact_line(aggregate_status_line)}")
        lines.append(f"[ci-gate-summary] final_status_line={final_status_line}")
        lines.append(f"[ci-gate-summary] final_status_parse={final_status_parse_json}")
        lines.append(f"[ci-gate-summary] summary_line={summary_line_path}")
        lines.append(f"[ci-gate-summary] ci_gate_result={ci_gate_result_json}")
        lines.append(f"[ci-gate-summary] ci_gate_result_parse={ci_gate_result_parse_json}")
        lines.append(f"[ci-gate-summary] ci_gate_result_line={ci_gate_result_line_path}")
        lines.append(f"[ci-gate-summary] ci_gate_badge={ci_gate_badge_json}")
        lines.append(f"[ci-gate-summary] ci_fail_brief_hint={ci_fail_brief_txt}")
        lines.append(f"[ci-gate-summary] ci_fail_brief_exists={int(ci_fail_brief_txt.exists())}")
        lines.append(f"[ci-gate-summary] ci_fail_triage_hint={ci_fail_triage_json}")
        lines.append(f"[ci-gate-summary] ci_fail_triage_exists={int(ci_fail_triage_json.exists())}")
        lines.append(f"[ci-gate-summary] final_status_compact={read_compact_line(final_status_line)}")
        for line in lines:
            print(line)
        if lines:
            write_summary(summary_path, lines)
            check_ci_gate_summary_report(require_pass=False)
            check_ci_gate_failure_summary(require_pass=False)
        summary_compact = resolve_summary_compact(
            ci_gate_result_line_path,
            final_status_parse_json,
            final_status_line,
        )
        write_summary_line(summary_line_path, summary_compact)
        print(f"[ci-gate-summary-line] {summary_compact}")
        post_summary_refresh_rc = refresh_status_outputs_for_index()
        if post_summary_refresh_rc != 0:
            print("[ci-gate] status outputs refresh failed after failure-summary rewrite", file=sys.stderr)
            return 2
        emit_required_post_summary_rc = run_and_record(
            "ci_emit_artifacts_required_post_summary_check",
            [
                py,
                "-c",
                "print('ci_emit_artifacts_required_post_summary_check: skipped strict triage-step validation on failure path')",
            ],
        )
        if emit_required_post_summary_rc != 0:
            print("[ci-gate] final summary emit artifacts required check failed", file=sys.stderr)
            return 2
        write_index(False, announce=False)
        ci_gate_report_index_post_summary_rc = check_ci_gate_report_index(require_step_contract=True)
        if ci_gate_report_index_post_summary_rc != 0:
            print("[ci-gate] report-index post-summary strict check failed", file=sys.stderr)
            return 2
        return 2
    if args.full_pass_summary:
        pass_lines = [
            "[ci-gate-summary] PASS",
            "[ci-gate-summary] failed_steps=(none)",
            f"[ci-gate-summary] report_index={index_report_path}",
            f"[ci-gate-summary] age2_status={age2_close_report}",
            f"[ci-gate-summary] age3_status={age3_close_status_json}",
            f"[ci-gate-summary] age4_status={age4_close_report}",
            f"[ci-gate-summary] age5_status={age5_close_report}",
            f"[ci-gate-summary] seamgrim_phase3_cleanup={seamgrim_phase3_cleanup_report}",
            f"[ci-gate-summary] age3_status_line={age3_close_status_line}",
            f"[ci-gate-summary] age3_badge={age3_close_badge_json}",
            f"[ci-gate-summary] age3_status_compact={read_compact_line(age3_close_status_line)}",
            f"[ci-gate-summary] aggregate_status_line={aggregate_status_line}",
            f"[ci-gate-summary] aggregate_status_parse={aggregate_status_parse_json}",
            f"[ci-gate-summary] aggregate_status_compact={read_compact_line(aggregate_status_line)}",
            f"[ci-gate-summary] final_status_line={final_status_line}",
            f"[ci-gate-summary] final_status_parse={final_status_parse_json}",
            f"[ci-gate-summary] summary_line={summary_line_path}",
            f"[ci-gate-summary] ci_gate_result={ci_gate_result_json}",
            f"[ci-gate-summary] ci_gate_result_parse={ci_gate_result_parse_json}",
            f"[ci-gate-summary] ci_gate_result_line={ci_gate_result_line_path}",
            f"[ci-gate-summary] ci_gate_badge={ci_gate_badge_json}",
            f"[ci-gate-summary] ci_fail_brief_hint={ci_fail_brief_txt}",
            f"[ci-gate-summary] ci_fail_brief_exists={int(ci_fail_brief_txt.exists())}",
            f"[ci-gate-summary] ci_fail_triage_hint={ci_fail_triage_json}",
            f"[ci-gate-summary] ci_fail_triage_exists={int(ci_fail_triage_json.exists())}",
            f"[ci-gate-summary] age5_close_digest_selftest_ok={int(age5_close_digest_selftest_rc == 0)}",
            f"[ci-gate-summary] ci_pack_golden_overlay_compare_selftest_ok={int(ci_pack_golden_overlay_compare_selftest_rc == 0)}",
            f"[ci-gate-summary] ci_pack_golden_overlay_session_selftest_ok={int(ci_pack_golden_overlay_session_selftest_rc == 0)}",
            f"[ci-gate-summary] ci_pack_golden_guideblock_selftest_ok={int(ci_pack_golden_guideblock_selftest_rc == 0)}",
            f"[ci-gate-summary] ci_pack_golden_age5_surface_selftest_ok={int(ci_pack_golden_age5_surface_selftest_rc == 0)}",
            f"[ci-gate-summary] ci_pack_golden_exec_policy_selftest_ok={int(ci_pack_golden_exec_policy_selftest_rc == 0)}",
            f"[ci-gate-summary] ci_pack_golden_jjaim_flatten_selftest_ok={int(ci_pack_golden_jjaim_flatten_selftest_rc == 0)}",
            f"[ci-gate-summary] ci_pack_golden_event_model_selftest_ok={int(ci_pack_golden_event_model_selftest_rc == 0)}",
            f"[ci-gate-summary] seamgrim_wasm_cli_diag_parity_report={seamgrim_wasm_cli_diag_parity_report}",
            f"[ci-gate-summary] seamgrim_wasm_cli_diag_parity_ok={int(seamgrim_wasm_cli_diag_parity_rc == 0)}",
            f"[ci-gate-summary] final_status_compact={read_compact_line(final_status_line)}",
            f"[ci-gate-summary] age3_summary={age3_close_summary_md}",
        ]
        append_age5_child_summary_lines(pass_lines, age5_close_report)
        append_age5_policy_summary_lines(pass_lines, aggregate_report)
        append_age4_proof_summary_lines(pass_lines, aggregate_report)
        append_ci_profile_matrix_selftest_summary_lines(pass_lines, ci_profile_matrix_gate_selftest_report)
        append_runtime_5min_summary_lines(
            pass_lines,
            bool(args.with_runtime_5min),
            seamgrim_runtime_5min_report,
            seamgrim_runtime_5min_browse_selection_report,
        )
        append_runtime_5min_checklist_summary_lines(
            pass_lines,
            include_5min_checklist,
            seamgrim_5min_checklist_report,
        )
    else:
        pass_lines = [
            "[ci-gate-summary] PASS",
            "[ci-gate-summary] failed_steps=(none)",
            f"[ci-gate-summary] report_index={index_report_path}",
            f"[ci-gate-summary] summary_line={summary_line_path}",
            f"[ci-gate-summary] ci_gate_result={ci_gate_result_json}",
            f"[ci-gate-summary] ci_gate_badge={ci_gate_badge_json}",
            f"[ci-gate-summary] ci_fail_brief_hint={ci_fail_brief_txt}",
            f"[ci-gate-summary] ci_fail_brief_exists={int(ci_fail_brief_txt.exists())}",
            f"[ci-gate-summary] ci_fail_triage_hint={ci_fail_triage_json}",
            f"[ci-gate-summary] ci_fail_triage_exists={int(ci_fail_triage_json.exists())}",
            f"[ci-gate-summary] age5_close_digest_selftest_ok={int(age5_close_digest_selftest_rc == 0)}",
            f"[ci-gate-summary] ci_pack_golden_overlay_compare_selftest_ok={int(ci_pack_golden_overlay_compare_selftest_rc == 0)}",
            f"[ci-gate-summary] ci_pack_golden_overlay_session_selftest_ok={int(ci_pack_golden_overlay_session_selftest_rc == 0)}",
            f"[ci-gate-summary] ci_pack_golden_guideblock_selftest_ok={int(ci_pack_golden_guideblock_selftest_rc == 0)}",
            f"[ci-gate-summary] ci_pack_golden_age5_surface_selftest_ok={int(ci_pack_golden_age5_surface_selftest_rc == 0)}",
            f"[ci-gate-summary] ci_pack_golden_exec_policy_selftest_ok={int(ci_pack_golden_exec_policy_selftest_rc == 0)}",
            f"[ci-gate-summary] ci_pack_golden_jjaim_flatten_selftest_ok={int(ci_pack_golden_jjaim_flatten_selftest_rc == 0)}",
            f"[ci-gate-summary] ci_pack_golden_event_model_selftest_ok={int(ci_pack_golden_event_model_selftest_rc == 0)}",
            f"[ci-gate-summary] seamgrim_wasm_cli_diag_parity_report={seamgrim_wasm_cli_diag_parity_report}",
            f"[ci-gate-summary] seamgrim_wasm_cli_diag_parity_ok={int(seamgrim_wasm_cli_diag_parity_rc == 0)}",
            f"[ci-gate-summary] age2_status={age2_close_report}",
            f"[ci-gate-summary] age3_status={age3_close_status_json}",
            f"[ci-gate-summary] age4_status={age4_close_report}",
            f"[ci-gate-summary] age5_status={age5_close_report}",
            f"[ci-gate-summary] seamgrim_phase3_cleanup={seamgrim_phase3_cleanup_report}",
        ]
        append_age5_child_summary_lines(pass_lines, age5_close_report)
        append_age5_policy_summary_lines(pass_lines, aggregate_report)
        append_age4_proof_summary_lines(pass_lines, aggregate_report)
        append_ci_profile_matrix_selftest_summary_lines(pass_lines, ci_profile_matrix_gate_selftest_report)
        append_runtime_5min_summary_lines(
            pass_lines,
            bool(args.with_runtime_5min),
            seamgrim_runtime_5min_report,
            seamgrim_runtime_5min_browse_selection_report,
        )
        append_runtime_5min_checklist_summary_lines(
            pass_lines,
            include_5min_checklist,
            seamgrim_5min_checklist_report,
        )
    control_exposure_snapshot = append_seamgrim_focus_summary_lines(
        pass_lines,
        seamgrim_report,
        seamgrim_control_exposure_failures_report,
        seamgrim_rewrite_overlay_quality_report,
        control_exposure_snapshot,
    )
    append_ci_sanity_summary_lines(pass_lines, ci_sanity_gate_report)
    append_ci_sync_readiness_summary_lines(pass_lines, ci_sync_readiness_report)
    append_fixed64_threeway_summary_lines(pass_lines, fixed64_threeway_gate_report)
    for line in pass_lines:
        print(line)
    write_summary(summary_path, pass_lines)
    emit_required_post_summary_rc = check_ci_emit_artifacts_required_post_summary()
    if emit_required_post_summary_rc != 0:
        print("[ci-gate] final summary emit artifacts required check failed", file=sys.stderr)
        return 2
    write_index(True, announce=False)
    ci_gate_report_index_post_summary_rc = check_ci_gate_report_index(require_step_contract=True)
    if ci_gate_report_index_post_summary_rc != 0:
        print("[ci-gate] report-index post-summary strict check failed", file=sys.stderr)
        return 2
    summary_report_check_rc = check_ci_gate_summary_report(require_pass=True)
    if summary_report_check_rc != 0:
        return 2
    summary_failure_check_rc = check_ci_gate_failure_summary(require_pass=True)
    if summary_failure_check_rc != 0:
        return 2
    summary_compact = resolve_summary_compact(
        ci_gate_result_line_path,
        final_status_parse_json,
        final_status_line,
    )
    write_summary_line(summary_line_path, summary_compact)
    print(f"[ci-gate-summary-line] {summary_compact}")
    print("[ci-gate] all checks passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

