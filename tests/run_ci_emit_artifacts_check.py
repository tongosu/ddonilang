#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path

from _ci_age3_completion_gate_contract import (
    AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS,
    AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS,
)
from _ci_age5_combined_heavy_contract import (
    AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY,
    AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY,
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY,
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY,
    AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
    AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS,
    AGE5_COMBINED_HEAVY_SANITY_CONTRACT_SUMMARY_FIELDS,
    AGE5_COMBINED_HEAVY_SYNC_CONTRACT_SUMMARY_FIELDS,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
    build_age4_proof_snapshot,
    build_age4_proof_snapshot_text,
    build_age5_combined_heavy_policy_origin_trace,
    build_age5_combined_heavy_policy_origin_trace_text,
    build_age5_close_digest_selftest_default_field,
    build_age5_combined_heavy_child_summary_default_text_transport_fields,
)
from _ci_profile_matrix_selftest_lib import (
    PROFILE_MATRIX_BRIEF_KEYS,
    PROFILE_MATRIX_SELFTEST_PROFILES,
    PROFILE_MATRIX_SELFTEST_SCHEMA,
    build_profile_matrix_brief_payload_from_snapshot,
    build_profile_matrix_snapshot_from_doc,
    build_profile_matrix_triage_payload_from_snapshot,
)
from ci_check_error_codes import EMIT_ARTIFACTS_CODES as CODES
from ci_check_error_codes import SUMMARY_VERIFY_CODES

INDEX_SCHEMA = "ddn.ci.aggregate_gate.index.v1"
TRIAGE_SCHEMA = "ddn.ci.fail_triage.v1"
TOKEN_RE = re.compile(r'([A-Za-z0-9_]+)=("([^"\\]|\\.)*"|[^ \t]+)')
SUMMARY_FAILED_STEP_DETAIL_RE = re.compile(r"^failed_step_detail=([^ ]+) rc=([-]?\d+) cmd=(.+)$")
SUMMARY_VERIFY_CODES_SET = set(SUMMARY_VERIFY_CODES.values())
SANITY_REQUIRED_PASS_STEPS = (
    "backup_hygiene_selftest",
    "pipeline_emit_flags_check",
    "pipeline_emit_flags_selftest",
    "ci_emit_artifacts_sanity_contract_selftest",
    "age5_combined_heavy_policy_selftest",
    "profile_matrix_full_real_smoke_policy_selftest",
    "profile_matrix_full_real_smoke_check_selftest",
    "age2_completion_gate",
    "age2_completion_gate_selftest",
    "age2_close_selftest",
    "age2_close",
    "age3_completion_gate",
    "age3_completion_gate_selftest",
    "age3_close_selftest",
    "age3_close",
    "fixed64_darwin_real_report_contract_check",
    "fixed64_darwin_real_report_live_check",
    "fixed64_darwin_real_report_readiness_check_selftest",
    "ci_profile_split_contract_check",
    "ci_profile_matrix_lightweight_contract_selftest",
    "ci_profile_matrix_snapshot_helper_selftest",
    "ci_sanity_dynamic_source_profile_split_selftest",
    "contract_tier_unsupported_check",
    "contract_tier_age3_min_enforcement_check",
    "map_access_contract_check",
    "gaji_registry_strict_audit_check",
    "gaji_registry_defaults_check",
    "stdlib_catalog_check",
    "stdlib_catalog_check_selftest",
    "tensor_v0_pack_check",
    "tensor_v0_cli_check",
    "seamgrim_ci_gate_sam_seulgi_family_step_check",
    "seamgrim_ci_gate_seed_meta_step_check",
    "seamgrim_ci_gate_runtime5_passthrough_check",
    "seamgrim_ci_gate_lesson_warning_step_check",
    "seamgrim_ci_gate_stateful_preview_step_check",
    "seamgrim_ci_gate_wasm_web_smoke_step_check",
    "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
    "seamgrim_ci_gate_pack_evidence_tier_step_check",
    "seamgrim_ci_gate_pack_evidence_tier_step_check_selftest",
    "seamgrim_ci_gate_pack_evidence_tier_runner_check",
    "seamgrim_ci_gate_pack_evidence_tier_report_check",
    "seamgrim_ci_gate_pack_evidence_tier_report_check_selftest",
    "seamgrim_interface_boundary_contract_check",
    "seamgrim_overlay_session_wired_consistency_check",
    "seamgrim_overlay_session_diag_parity_check",
    "seamgrim_overlay_compare_diag_parity_check",
    "age5_close_pack_contract_selftest",
    "ci_pack_golden_age5_surface_selftest",
    "ci_pack_golden_guideblock_selftest",
    "ci_pack_golden_exec_policy_selftest",
    "ci_pack_golden_jjaim_flatten_selftest",
    "ci_pack_golden_event_model_selftest",
    "ci_pack_golden_lang_consistency_selftest",
    "ci_pack_golden_metadata_selftest",
    "ci_pack_golden_graph_export_selftest",
    "ci_canon_ast_dpack_selftest",
    "w92_aot_pack_check",
    "w93_universe_pack_check",
    "w94_social_pack_check",
    "w95_cert_pack_check",
    "w96_somssi_pack_check",
    "w97_self_heal_pack_check",
    "seamgrim_wasm_cli_diag_parity_check",
)
SANITY_REQUIRED_PASS_STEPS_CORE_LANG = (
    "backup_hygiene_selftest",
    "pipeline_emit_flags_check",
    "pipeline_emit_flags_selftest",
    "ci_emit_artifacts_sanity_contract_selftest",
    "age5_combined_heavy_policy_selftest",
    "profile_matrix_full_real_smoke_policy_selftest",
    "profile_matrix_full_real_smoke_check_selftest",
    "age2_completion_gate",
    "age2_completion_gate_selftest",
    "age2_close_selftest",
    "age2_close",
    "age3_completion_gate",
    "age3_completion_gate_selftest",
    "age3_close_selftest",
    "fixed64_darwin_real_report_contract_check",
    "fixed64_darwin_real_report_live_check",
    "fixed64_darwin_real_report_readiness_check_selftest",
    "ci_profile_split_contract_check",
    "ci_profile_matrix_lightweight_contract_selftest",
    "ci_profile_matrix_snapshot_helper_selftest",
    "ci_sanity_dynamic_source_profile_split_selftest",
    "contract_tier_unsupported_check",
    "contract_tier_age3_min_enforcement_check",
    "map_access_contract_check",
    "gaji_registry_strict_audit_check",
    "gaji_registry_defaults_check",
    "stdlib_catalog_check",
    "stdlib_catalog_check_selftest",
    "tensor_v0_pack_check",
    "tensor_v0_cli_check",
    "age5_close_pack_contract_selftest",
    "ci_pack_golden_age5_surface_selftest",
    "ci_pack_golden_guideblock_selftest",
    "ci_pack_golden_exec_policy_selftest",
    "ci_pack_golden_jjaim_flatten_selftest",
    "ci_pack_golden_event_model_selftest",
    "ci_pack_golden_lang_consistency_selftest",
    "ci_pack_golden_metadata_selftest",
    "ci_pack_golden_graph_export_selftest",
    "ci_canon_ast_dpack_selftest",
    "w92_aot_pack_check",
    "w93_universe_pack_check",
    "w94_social_pack_check",
    "w95_cert_pack_check",
    "w96_somssi_pack_check",
    "w97_self_heal_pack_check",
)
SANITY_REQUIRED_PASS_STEPS_SEAMGRIM = (
    "age5_combined_heavy_policy_selftest",
    "fixed64_darwin_real_report_contract_check",
    "fixed64_darwin_real_report_live_check",
    "fixed64_darwin_real_report_readiness_check_selftest",
    "profile_matrix_full_real_smoke_policy_selftest",
    "profile_matrix_full_real_smoke_check_selftest",
    "ci_emit_artifacts_sanity_contract_selftest",
    "age2_completion_gate",
    "age2_completion_gate_selftest",
    "age2_close_selftest",
    "age2_close",
    "age3_completion_gate",
    "age3_completion_gate_selftest",
    "age3_close_selftest",
    "age3_close",
    "ci_profile_split_contract_check",
    "ci_profile_matrix_lightweight_contract_selftest",
    "ci_profile_matrix_snapshot_helper_selftest",
    "ci_sanity_dynamic_source_profile_split_selftest",
    "seamgrim_ci_gate_sam_seulgi_family_step_check",
    "seamgrim_ci_gate_seed_meta_step_check",
    "seamgrim_ci_gate_runtime5_passthrough_check",
    "seamgrim_ci_gate_lesson_warning_step_check",
    "seamgrim_ci_gate_stateful_preview_step_check",
    "seamgrim_ci_gate_wasm_web_smoke_step_check",
    "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
    "seamgrim_ci_gate_pack_evidence_tier_step_check",
    "seamgrim_ci_gate_pack_evidence_tier_step_check_selftest",
    "seamgrim_ci_gate_pack_evidence_tier_runner_check",
    "seamgrim_ci_gate_pack_evidence_tier_report_check",
    "seamgrim_ci_gate_pack_evidence_tier_report_check_selftest",
    "seamgrim_interface_boundary_contract_check",
    "seamgrim_overlay_session_wired_consistency_check",
    "seamgrim_overlay_session_diag_parity_check",
    "seamgrim_overlay_compare_diag_parity_check",
    "seamgrim_wasm_cli_diag_parity_check",
)
VALID_SANITY_PROFILES = set(PROFILE_MATRIX_SELFTEST_PROFILES)
RUNTIME5_SUMMARY_REQUIRED_KEYS = (
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
VALID_RUNTIME5_ITEM_STATUS = {"ok", "failed", "not_executed", "missing_report", "items_missing"}
PROFILE_MATRIX_SELFTEST_SUMMARY_REQUIRED_KEYS = (
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
)
PROFILE_MATRIX_BRIEF_REQUIRED_KEYS = PROFILE_MATRIX_BRIEF_KEYS
SEAMGRIM_FOCUS_SUMMARY_REQUIRED_KEYS = (
    "seamgrim_group_id_summary_status",
)
VALID_SEAMGRIM_FOCUS_SUMMARY_STATUS = {"ok", "failed", "missing_report"}
AGE_CLOSE_STATUS_SUMMARY_SPECS = (
    ("age2_status", "age2_close", "ddn.age2_close_report.v1"),
    ("age3_status", "age3_close_status_json", "ddn.seamgrim.age3_close_status.v1"),
    ("age4_status", "age4_close", "ddn.age4_close_report.v1"),
    ("age5_status", "age5_close", "ddn.age5_close_report.v1"),
)
AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA = "ddn.bogae_geoul_visibility_smoke.v1"
AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_PROFILES = {"full", "core_lang", "seamgrim"}
SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA = "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1"
SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES = 20
SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES = {"seamgrim"}
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA = "ddn.pack_evidence_tier_runner_check.v1"
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES = {"seamgrim"}
SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA = "ddn.numeric_factor_route_diag_contract.v1"
SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES = {"full", "seamgrim"}
SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS: dict[str, int] = {
    "bit_limit": 512,
    "pollard_iters": 200000,
    "pollard_c_seeds": 64,
    "pollard_x0_seeds": 6,
    "fallback_limit": 1000000,
    "small_prime_max": 101,
}
AGE3_COMPLETION_GATE_CRITERIA_PROFILES = {"full", "core_lang", "seamgrim"}
SANITY_RUNTIME_HELPER_SUMMARY_FIELDS = (
    ("ci_sanity_pipeline_emit_flags_ok", {"full", "core_lang"}),
    ("ci_sanity_pipeline_emit_flags_selftest_ok", {"full", "core_lang"}),
    ("ci_sanity_emit_artifacts_sanity_contract_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age2_completion_gate_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age2_completion_gate_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age2_close_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age2_close_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_completion_gate_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_completion_gate_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_close_ok", {"full", "seamgrim"}),
    ("ci_sanity_age3_close_selftest_ok", {"full", "core_lang", "seamgrim"}),
    (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_ok",
        AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_PROFILES,
    ),
    (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists",
        AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_PROFILES,
    ),
    (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok",
        AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_PROFILES,
    ),
    (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok",
        AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_PROFILES,
    ),
    (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes",
        AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_PROFILES,
    ),
    (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes",
        AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_wasm_web_step_check_ok",
        SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_wasm_web_step_check_report_exists",
        SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_pack_evidence_tier_runner_ok",
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists",
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_ok",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_report_exists",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    ("ci_sanity_fixed64_darwin_real_report_live_report_exists", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age5_combined_heavy_policy_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_dynamic_source_profile_split_selftest_ok", {"full", "core_lang", "seamgrim"}),
)
SANITY_RUNTIME_HELPER_SUMMARY_FIELDS = SANITY_RUNTIME_HELPER_SUMMARY_FIELDS + tuple(
    (key, AGE3_COMPLETION_GATE_CRITERIA_PROFILES)
    for key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS
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
        "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path",
        "path",
        AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_PROFILES,
    ),
    (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_schema",
        "schema",
        AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_PROFILES,
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
        "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path",
        "step_path",
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
        "ci_sanity_seamgrim_numeric_factor_policy_report_path",
        "numeric_factor_policy_path",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_schema",
        "numeric_factor_policy_schema",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_text",
        "numeric_factor_policy_text",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_bit_limit",
        "numeric_factor_policy_value",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_pollard_iters",
        "numeric_factor_policy_value",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds",
        "numeric_factor_policy_value",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds",
        "numeric_factor_policy_value",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_fallback_limit",
        "numeric_factor_policy_value",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_small_prime_max",
        "numeric_factor_policy_value",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
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
SYNC_RUNTIME_HELPER_SUMMARY_FIELDS = (
    ("ci_sync_readiness_ci_sanity_pipeline_emit_flags_ok", "ci_sanity_pipeline_emit_flags_ok", {"full", "core_lang"}),
    (
        "ci_sync_readiness_ci_sanity_pipeline_emit_flags_selftest_ok",
        "ci_sanity_pipeline_emit_flags_selftest_ok",
        {"full", "core_lang"},
    ),
    (
        "ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok",
        "ci_sanity_emit_artifacts_sanity_contract_selftest_ok",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok",
        "ci_sanity_pack_golden_graph_export_ok",
        {"full", "core_lang"},
    ),
    (
        "ci_sync_readiness_ci_sanity_age2_completion_gate_ok",
        "ci_sanity_age2_completion_gate_ok",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_age2_completion_gate_selftest_ok",
        "ci_sanity_age2_completion_gate_selftest_ok",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_age2_close_ok",
        "ci_sanity_age2_close_ok",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_age2_close_selftest_ok",
        "ci_sanity_age2_close_selftest_ok",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_completion_gate_ok",
        "ci_sanity_age3_completion_gate_ok",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_completion_gate_selftest_ok",
        "ci_sanity_age3_completion_gate_selftest_ok",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_close_ok",
        "ci_sanity_age3_close_ok",
        {"full", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_close_selftest_ok",
        "ci_sanity_age3_close_selftest_ok",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_ok",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_ok",
        AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists",
        AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok",
        AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok",
        AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes",
        AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes",
        AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_ok",
        "ci_sanity_seamgrim_wasm_web_step_check_ok",
        SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_exists",
        "ci_sanity_seamgrim_wasm_web_step_check_report_exists",
        SEAMGRIM_WASM_WEB_STEP_CHECK_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok",
        "ci_sanity_seamgrim_pack_evidence_tier_runner_ok",
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists",
        "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists",
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok",
        "ci_sanity_seamgrim_numeric_factor_policy_ok",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_report_exists",
        "ci_sanity_seamgrim_numeric_factor_policy_report_exists",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_report_exists",
        "ci_sanity_fixed64_darwin_real_report_live_report_exists",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_age5_combined_heavy_policy_selftest_ok",
        "ci_sanity_age5_combined_heavy_policy_selftest_ok",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok",
        "ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sync_readiness_ci_sanity_dynamic_source_profile_split_selftest_ok",
        "ci_sanity_dynamic_source_profile_split_selftest_ok",
        {"full", "core_lang", "seamgrim"},
    ),
)
SYNC_RUNTIME_HELPER_SUMMARY_FIELDS = SYNC_RUNTIME_HELPER_SUMMARY_FIELDS + tuple(
    (sync_key, sanity_key, AGE3_COMPLETION_GATE_CRITERIA_PROFILES)
    for sanity_key, sync_key in AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS
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
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_path",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path",
        "path",
        AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_schema",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_schema",
        "schema",
        AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_PROFILES,
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
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_path",
        "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path",
        "step_path",
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
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_report_path",
        "ci_sanity_seamgrim_numeric_factor_policy_report_path",
        "numeric_factor_policy_path",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_schema",
        "ci_sanity_seamgrim_numeric_factor_policy_schema",
        "numeric_factor_policy_schema",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_text",
        "ci_sanity_seamgrim_numeric_factor_policy_text",
        "numeric_factor_policy_text",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_bit_limit",
        "ci_sanity_seamgrim_numeric_factor_policy_bit_limit",
        "numeric_factor_policy_value",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_iters",
        "ci_sanity_seamgrim_numeric_factor_policy_pollard_iters",
        "numeric_factor_policy_value",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds",
        "ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds",
        "numeric_factor_policy_value",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds",
        "ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds",
        "numeric_factor_policy_value",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_fallback_limit",
        "ci_sanity_seamgrim_numeric_factor_policy_fallback_limit",
        "numeric_factor_policy_value",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_small_prime_max",
        "ci_sanity_seamgrim_numeric_factor_policy_small_prime_max",
        "numeric_factor_policy_value",
        SEAMGRIM_NUMERIC_FACTOR_POLICY_PROFILES,
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
VALID_RUNTIME_HELPER_SUMMARY_VALUES = {"1", "0", "na", "pending"}
FAILURE_CODE_PATTERN = re.compile(r"[EW]_[A-Z0-9_]+")
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
PACK_GOLDEN_GRAPH_EXPORT_SYNC_SUMMARY_KEY = "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok"
PACK_GOLDEN_GRAPH_EXPORT_SANITY_SOURCE_KEY = "ci_sanity_pack_golden_graph_export_ok"
PACK_GOLDEN_GRAPH_EXPORT_REQUIRED_PROFILES = {"full", "core_lang"}
FAILURE_CODE_PAIR_KEYS = (
    (
        "ci_sanity_age2_completion_gate_failure_codes",
        "ci_sanity_age2_completion_gate_failure_code_count",
    ),
    (
        "ci_sanity_age3_completion_gate_failure_codes",
        "ci_sanity_age3_completion_gate_failure_code_count",
    ),
)
SYNC_FAILURE_CODE_PAIR_KEYS = (
    (
        "ci_sync_readiness_ci_sanity_age2_completion_gate_failure_codes",
        "ci_sync_readiness_ci_sanity_age2_completion_gate_failure_code_count",
        "ci_sanity_age2_completion_gate_failure_codes",
        "ci_sanity_age2_completion_gate_failure_code_count",
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_completion_gate_failure_codes",
        "ci_sync_readiness_ci_sanity_age3_completion_gate_failure_code_count",
        "ci_sanity_age3_completion_gate_failure_codes",
        "ci_sanity_age3_completion_gate_failure_code_count",
    ),
)
SANITY_RUNTIME_HELPER_CONTRACT_FIELDS = AGE5_COMBINED_HEAVY_SANITY_CONTRACT_SUMMARY_FIELDS
SYNC_RUNTIME_HELPER_CONTRACT_FIELDS = AGE5_COMBINED_HEAVY_SYNC_CONTRACT_SUMMARY_FIELDS
AGE5_COMBINED_HEAVY_SANITY_CONTRACT_SUMMARY_KEYS = (
    "ci_sanity_age5_combined_heavy_report_schema",
    "ci_sanity_age5_combined_heavy_required_reports",
    "ci_sanity_age5_combined_heavy_required_criteria",
    "ci_sanity_age5_combined_heavy_child_summary_default_fields",
    "ci_sanity_age5_combined_heavy_combined_contract_summary_fields",
    "ci_sanity_age5_combined_heavy_full_summary_contract_fields",
)
AGE5_COMBINED_HEAVY_SYNC_CONTRACT_SUMMARY_KEYS = (
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_report_schema",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_required_reports",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_required_criteria",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_combined_contract_summary_fields",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_full_summary_contract_fields",
)
VALID_AGE5_CHILD_SUMMARY_STATUS = {"pass", "fail", "skipped"}
AGE4_PROOF_OK_KEY = "age4_proof_ok"
AGE4_PROOF_FAILED_CRITERIA_KEY = "age4_proof_failed_criteria"
AGE4_PROOF_FAILED_PREVIEW_KEY = "age4_proof_failed_preview"
AGE4_PROOF_SUMMARY_HASH_KEY = "age4_proof_summary_hash"
AGE5_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS = (
    build_age5_combined_heavy_child_summary_default_text_transport_fields()
)
AGE5_DIGEST_SELFTEST_SUMMARY_KEY = "age5_close_digest_selftest_ok"
AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY = "age5_policy_combined_digest_selftest_default_field_text"
AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY = "age5_policy_combined_digest_selftest_default_field"
AGE5_POLICY_REPORT_PATH_KEY = "age5_combined_heavy_policy_report_path"
AGE5_POLICY_REPORT_EXISTS_KEY = "age5_combined_heavy_policy_report_exists"
AGE5_POLICY_TEXT_PATH_KEY = "age5_combined_heavy_policy_text_path"
AGE5_POLICY_TEXT_EXISTS_KEY = "age5_combined_heavy_policy_text_exists"
AGE5_POLICY_SUMMARY_PATH_KEY = "age5_combined_heavy_policy_summary_path"
AGE5_POLICY_SUMMARY_EXISTS_KEY = "age5_combined_heavy_policy_summary_exists"
AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY = "age5_policy_age4_proof_snapshot_fields_text"
AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY = "age5_policy_age4_proof_snapshot_text"
AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY = "age5_policy_age4_proof_source_snapshot_fields_text"
AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY = "age5_policy_age4_proof_gate_result_present"
AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY = "age5_policy_age4_proof_gate_result_parity"
AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY = "age5_policy_age4_proof_final_status_parse_present"
AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY = "age5_policy_age4_proof_final_status_parse_parity"
AGE5_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY = "age5_policy_summary_origin_trace_contract_issue"
AGE5_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY = "age5_policy_summary_origin_trace_contract_source_issue"
AGE5_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY = (
    "age5_policy_summary_origin_trace_contract_compact_reason"
)
AGE5_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY = (
    "age5_policy_summary_origin_trace_contract_compact_failure_reason"
)
AGE5_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY = "age5_policy_summary_origin_trace_contract_status"
AGE5_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY = "age5_policy_summary_origin_trace_contract_ok"
AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY = AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY
AGE5_POLICY_ORIGIN_TRACE_KEY = AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY
# split-contract token anchor: combined_digest_selftest_default_field_text / combined_digest_selftest_default_field
AGE5_DIGEST_SELFTEST_DEFAULT_FIELD = build_age5_close_digest_selftest_default_field()

def resolve_required_sanity_steps(profile: str) -> tuple[str, ...]:
    if profile == "core_lang":
        return SANITY_REQUIRED_PASS_STEPS_CORE_LANG
    if profile == "seamgrim":
        return SANITY_REQUIRED_PASS_STEPS_SEAMGRIM
    return SANITY_REQUIRED_PASS_STEPS


def fail(msg: str, code: str = "E_CHECK") -> int:
    print(f"[ci-emit-artifacts-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8-sig"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def load_line(path: Path) -> str:
    if not path.exists():
        return ""
    try:
        return path.read_text(encoding="utf-8-sig").strip()
    except Exception:
        return ""


def normalize_path_text(raw: str) -> str:
    text = str(raw).strip()
    return text.replace("\\", "/")


def resolve_path(raw: str) -> Path | None:
    text = str(raw).strip()
    if not text:
        return None
    return Path(text.replace("\\", "/"))


def parse_tokens(line: str) -> dict[str, str] | None:
    text = str(line).strip()
    if not text:
        return None
    out: dict[str, str] = {}
    pos = 0
    for match in TOKEN_RE.finditer(text):
        if text[pos : match.start()].strip():
            return None
        key = match.group(1)
        raw = match.group(2)
        if raw.startswith('"'):
            try:
                value = json.loads(raw)
            except Exception:
                return None
        else:
            value = raw
        out[key] = str(value)
        pos = match.end()
    if text[pos:].strip():
        return None
    return out


def validate_age5_child_summary_tokens(tokens: dict[str, object], *, source: str) -> str | None:
    for key in AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS:
        value = str(tokens.get(key, "")).strip()
        if not value:
            return f"{source} missing {key}"
        if value not in VALID_AGE5_CHILD_SUMMARY_STATUS:
            return f"{source} invalid {key}: {value}"
    return None


def validate_age5_child_summary_default_transport_tokens(tokens: dict[str, object], *, source: str) -> str | None:
    for key, expected in AGE5_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS.items():
        value = str(tokens.get(key, "")).strip()
        if not value:
            return f"{source} missing {key}"
        if value != expected:
            return f"{source} invalid {key}: {value}"
    return None


def resolve_age5_digest_selftest_expected(summary_kv: dict[str, str], index_doc: dict) -> str:
    summary_value = str(summary_kv.get(AGE5_DIGEST_SELFTEST_SUMMARY_KEY, "")).strip()
    if summary_value in {"0", "1"}:
        return summary_value
    step_ok = read_step_ok(index_doc, "age5_close_digest_selftest")
    if step_ok is None:
        return "0"
    return "1" if step_ok else "0"


def clip(text: str, limit: int) -> str:
    value = str(text).strip()
    if len(value) <= limit:
        return value
    return value[: max(0, limit - 3)] + "..."


def expected_final_line_candidates(index_doc: dict, summary_line_text: str, limit: int) -> list[str]:
    candidates: list[str] = []

    def add(raw: str) -> None:
        text = str(raw).strip()
        if not text:
            return
        compact = clip(text, limit)
        if compact and compact not in candidates:
            candidates.append(compact)

    add(summary_line_text)
    for key in ("summary_line", "ci_gate_result_line", "final_status_line", "aggregate_status_line"):
        path = artifact_path(index_doc, key)
        if path is None:
            continue
        add(load_line(path))
    return candidates


def validate_runtime5_elapsed_text(value: str) -> bool:
    text = str(value).strip()
    if text == "-":
        return True
    try:
        return int(text) >= 0
    except Exception:
        return False


def expected_sync_runtime_helper_summary_value(
    summary_key: str,
    profile: str,
    valid_profiles: set[str],
) -> str:
    if summary_key == PACK_GOLDEN_GRAPH_EXPORT_SYNC_SUMMARY_KEY:
        return "1" if profile in PACK_GOLDEN_GRAPH_EXPORT_REQUIRED_PROFILES else "0"
    return "1" if profile in valid_profiles else "na"


def validate_failure_code_field_value(key: str, value: str, value_kind: str) -> str | None:
    text = str(value).strip()
    if not text:
        return f"{key} is empty"
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
    if value_kind == "numeric_factor_policy_schema":
        if text != SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA:
            return f"{key} schema mismatch: {text}"
        return None
    if value_kind == "numeric_factor_policy_path":
        if text == "-":
            return f"{key} is '-'"
        path = Path(text)
        if not path.exists():
            return f"{key} path does not exist: {path}"
        return None
    if value_kind == "numeric_factor_policy_text":
        if text in {"-", "na", "pending"}:
            return f"{key} invalid policy text: {text}"
        parsed: dict[str, int] = {}
        for token in text.split(";"):
            token = token.strip()
            if not token:
                continue
            if "=" not in token:
                return f"{key} malformed token: {token}"
            name, raw = token.split("=", 1)
            name = name.strip()
            raw = raw.strip()
            if name not in SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS:
                return f"{key} unknown token key: {name}"
            try:
                parsed[name] = int(raw)
            except Exception:
                return f"{key} token must be int: {token}"
        expected_keys = set(SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS.keys())
        if set(parsed.keys()) != expected_keys:
            return f"{key} token keys mismatch: {sorted(parsed.keys())}"
        return None
    if value_kind == "numeric_factor_policy_value":
        policy_key = None
        for candidate in SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS:
            if key.endswith(f"numeric_factor_policy_{candidate}"):
                policy_key = candidate
                break
        if policy_key is None:
            return f"{key} unknown policy key"
        try:
            value_num = int(text)
        except Exception:
            return f"{key} is not an integer: {text}"
        expected = int(SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS[policy_key])
        if value_num != expected:
            return f"{key} mismatch expected={expected} actual={value_num}"
        return None
    return f"{key} unsupported failure-code kind: {value_kind}"


def load_runtime5_checklist_snapshot(report_path: Path) -> dict | None:
    doc = load_json(report_path)
    if not isinstance(doc, dict):
        return None
    if str(doc.get("schema", "")).strip() != "seamgrim.runtime_5min_checklist.v1":
        return None
    items = doc.get("items")
    if not isinstance(items, list):
        return None

    rewrite_row = None
    moyang_row = None
    showcase_row = None
    for row in items:
        if not isinstance(row, dict):
            continue
        row_name = str(row.get("name", "")).strip()
        if row_name == "rewrite_motion_projectile_fallback":
            rewrite_row = row
        elif row_name == "moyang_view_boundary_pack_check":
            moyang_row = row
        elif row_name == "pendulum_tetris_showcase_check":
            showcase_row = row

    def summarize_row(row: object) -> tuple[str, str, str]:
        if not isinstance(row, dict):
            return ("na", "-", "not_executed")
        elapsed_raw = row.get("elapsed_ms")
        try:
            elapsed_text = str(max(0, int(elapsed_raw)))
        except Exception:
            elapsed_text = "-"
        ok = bool(row.get("ok", False))
        return ("1" if ok else "0", elapsed_text, "ok" if ok else "failed")

    rewrite_ok, rewrite_elapsed, rewrite_status = summarize_row(rewrite_row)
    moyang_ok, moyang_elapsed, moyang_status = summarize_row(moyang_row)
    showcase_ok, showcase_elapsed, showcase_status = summarize_row(showcase_row)
    return {
        "path": str(report_path),
        "ok": "1" if bool(doc.get("ok", False)) else "0",
        "rewrite_ok": rewrite_ok,
        "rewrite_elapsed_ms": rewrite_elapsed,
        "rewrite_status": rewrite_status,
        "moyang_ok": moyang_ok,
        "moyang_elapsed_ms": moyang_elapsed,
        "moyang_status": moyang_status,
        "showcase_ok": showcase_ok,
        "showcase_elapsed_ms": showcase_elapsed,
        "showcase_status": showcase_status,
    }


def load_profile_matrix_selftest_snapshot(report_path: Path) -> dict | None:
    doc = load_json(report_path)
    return build_profile_matrix_snapshot_from_doc(doc, report_path=str(report_path))


def parse_summary_report(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        lines = path.read_text(encoding="utf-8-sig").splitlines()
    except Exception:
        return None
    status = ""
    failed_steps: list[str] = []
    failed_step_details: dict[str, str] = {}
    failed_step_logs: dict[str, dict[str, str]] = {}
    failed_step_detail_order: list[str] = []
    failed_step_logs_order: list[str] = []
    kv: dict[str, str] = {}
    prefix = "[ci-gate-summary] "
    for raw in lines:
        line = str(raw).strip()
        if not line.startswith(prefix):
            continue
        body = line[len(prefix) :].strip()
        if body == "PASS":
            status = "pass"
            continue
        if body == "FAIL":
            status = "fail"
            continue
        if body.startswith("failed_steps="):
            payload = body[len("failed_steps=") :].strip()
            if payload in ("", "-", "(none)"):
                failed_steps = []
            else:
                failed_steps = [token.strip() for token in payload.split(",") if token.strip()]
            continue
        if body.startswith("failed_step_detail="):
            payload = body[len("failed_step_detail=") :]
            step_id = payload.split(" ", 1)[0].strip()
            if step_id:
                if step_id not in failed_step_details:
                    failed_step_detail_order.append(step_id)
                failed_step_details[step_id] = body
            continue
        if body.startswith("failed_step_logs="):
            payload = body[len("failed_step_logs=") :]
            parts = payload.split(" ", 1)
            step_id = parts[0].strip()
            if not step_id:
                continue
            row = {"stdout": "", "stderr": ""}
            if len(parts) > 1 and parts[1].strip():
                tokens = parse_tokens(parts[1].strip())
                if isinstance(tokens, dict):
                    row["stdout"] = str(tokens.get("stdout", "")).strip()
                    row["stderr"] = str(tokens.get("stderr", "")).strip()
            if step_id not in failed_step_logs:
                failed_step_logs_order.append(step_id)
            failed_step_logs[step_id] = row
            continue
        if "=" in body:
            key, value = body.split("=", 1)
            key = key.strip()
            value = value.strip()
            if key:
                kv[key] = value
    return {
        "status": status,
        "failed_steps": failed_steps,
        "failed_step_details": failed_step_details,
        "failed_step_logs": failed_step_logs,
        "failed_step_detail_order": failed_step_detail_order,
        "failed_step_logs_order": failed_step_logs_order,
        "kv": kv,
    }


def read_step_ok(index_doc: dict, step_name: str) -> bool | None:
    steps = index_doc.get("steps")
    if not isinstance(steps, list):
        return None
    for row in steps:
        if not isinstance(row, dict):
            continue
        if str(row.get("name", "")).strip() != step_name:
            continue
        if "ok" in row:
            return bool(row.get("ok", False))
        try:
            return int(row.get("returncode", 1)) == 0
        except Exception:
            return False
    return None


def select_latest_index(report_dir: Path, pattern: str, prefix: str) -> tuple[Path | None, dict | None]:
    candidates = sorted(
        report_dir.glob(pattern),
        key=lambda p: (p.stat().st_mtime_ns, str(p)),
        reverse=True,
    )
    for path in candidates:
        doc = load_json(path)
        if not isinstance(doc, dict):
            continue
        if str(doc.get("schema", "")).strip() != INDEX_SCHEMA:
            continue
        if prefix and str(doc.get("report_prefix", "")).strip() != prefix:
            continue
        return path, doc
    return None, None


def artifact_path(index_doc: dict, key: str) -> Path | None:
    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return None
    text = str(reports.get(key, "")).strip()
    if not text or text == "-":
        return None
    return resolve_path(text)


def artifact_path_text(index_doc: dict, key: str) -> str:
    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return ""
    return str(reports.get(key, "")).strip()


def load_age5_policy_snapshot(index_doc: dict) -> dict[str, object]:
    snapshot: dict[str, object] = {
        AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
        AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY: dict(AGE5_DIGEST_SELFTEST_DEFAULT_FIELD),
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY: AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY: build_age4_proof_snapshot_text(
            build_age4_proof_snapshot()
        ),
        AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY: AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY: "0",
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY: "0",
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY: "0",
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY: "0",
        AGE5_POLICY_REPORT_PATH_KEY: "-",
        AGE5_POLICY_REPORT_EXISTS_KEY: 0,
        AGE5_POLICY_TEXT_PATH_KEY: "-",
        AGE5_POLICY_TEXT_EXISTS_KEY: 0,
        AGE5_POLICY_SUMMARY_PATH_KEY: "-",
        AGE5_POLICY_SUMMARY_EXISTS_KEY: 0,
        AGE5_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY: "-",
        AGE5_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY: "-",
        AGE5_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY: "-",
        AGE5_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY: "-",
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY: "ok",
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY: 0,
        AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY: build_age5_combined_heavy_policy_origin_trace_text(),
        AGE5_POLICY_ORIGIN_TRACE_KEY: build_age5_combined_heavy_policy_origin_trace(),
    }
    aggregate_path = artifact_path(index_doc, "aggregate")
    if aggregate_path is None or not aggregate_path.exists():
        return snapshot
    aggregate_doc = load_json(aggregate_path)
    if not isinstance(aggregate_doc, dict):
        return snapshot
    age5_doc = aggregate_doc.get("age5")
    if not isinstance(age5_doc, dict):
        return snapshot
    text_value = str(age5_doc.get(AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip()
    if text_value:
        snapshot[AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY] = text_value
    field_value = age5_doc.get(AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY)
    if isinstance(field_value, dict) and field_value:
        snapshot[AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY] = {
            str(key): str(value) for key, value in field_value.items()
        }
    age4_snapshot_fields_text = str(
        age5_doc.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY, "")
    ).strip()
    if age4_snapshot_fields_text:
        snapshot[AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY] = age4_snapshot_fields_text
    age4_snapshot_text = str(age5_doc.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY, "")).strip()
    if age4_snapshot_text:
        snapshot[AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY] = age4_snapshot_text
    age4_source_snapshot_fields_text = str(
        age5_doc.get(AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY, "")
    ).strip()
    if age4_source_snapshot_fields_text:
        snapshot[AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY] = age4_source_snapshot_fields_text
    for aggregate_key, snapshot_key in (
        (AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY, AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY),
        (AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY, AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY),
        (AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY, AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY),
        (AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY, AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY),
    ):
        value = str(age5_doc.get(aggregate_key, "")).strip()
        if value:
            snapshot[snapshot_key] = value
    report_path = str(age5_doc.get(AGE5_POLICY_REPORT_PATH_KEY, "")).strip()
    if report_path:
        snapshot[AGE5_POLICY_REPORT_PATH_KEY] = report_path
    snapshot[AGE5_POLICY_REPORT_EXISTS_KEY] = int(bool(age5_doc.get(AGE5_POLICY_REPORT_EXISTS_KEY, False)))
    text_path = str(age5_doc.get(AGE5_POLICY_TEXT_PATH_KEY, "")).strip()
    if text_path:
        snapshot[AGE5_POLICY_TEXT_PATH_KEY] = text_path
    snapshot[AGE5_POLICY_TEXT_EXISTS_KEY] = int(bool(age5_doc.get(AGE5_POLICY_TEXT_EXISTS_KEY, False)))
    summary_path = str(age5_doc.get(AGE5_POLICY_SUMMARY_PATH_KEY, "")).strip()
    if summary_path:
        snapshot[AGE5_POLICY_SUMMARY_PATH_KEY] = summary_path
    snapshot[AGE5_POLICY_SUMMARY_EXISTS_KEY] = int(bool(age5_doc.get(AGE5_POLICY_SUMMARY_EXISTS_KEY, False)))
    contract_issue = str(
        age5_doc.get(AGE5_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY, "")
    ).strip()
    if contract_issue:
        snapshot[AGE5_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY] = contract_issue
    source_contract_issue = str(
        age5_doc.get(AGE5_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY, "")
    ).strip()
    if source_contract_issue:
        snapshot[AGE5_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY] = source_contract_issue
    compact_reason = str(
        age5_doc.get(AGE5_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY, "")
    ).strip()
    if compact_reason:
        snapshot[AGE5_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY] = compact_reason
    compact_failure_reason = str(
        age5_doc.get(AGE5_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY, "")
    ).strip()
    if compact_failure_reason:
        snapshot[AGE5_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY] = compact_failure_reason
    contract_status = str(age5_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY, "")).strip()
    if contract_status:
        snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY] = contract_status
    snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY] = int(
        bool(age5_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY, False))
    )
    origin_trace = age5_doc.get(AGE5_POLICY_ORIGIN_TRACE_KEY)
    if isinstance(origin_trace, dict) and origin_trace:
        snapshot[AGE5_POLICY_ORIGIN_TRACE_KEY] = {
            str(key): str(value) for key, value in origin_trace.items()
        }
    else:
        snapshot[AGE5_POLICY_ORIGIN_TRACE_KEY] = build_age5_combined_heavy_policy_origin_trace(
            report_path=str(snapshot[AGE5_POLICY_REPORT_PATH_KEY]),
            report_exists=snapshot[AGE5_POLICY_REPORT_EXISTS_KEY],
            text_path=str(snapshot[AGE5_POLICY_TEXT_PATH_KEY]),
            text_exists=snapshot[AGE5_POLICY_TEXT_EXISTS_KEY],
            summary_path=str(snapshot[AGE5_POLICY_SUMMARY_PATH_KEY]),
            summary_exists=snapshot[AGE5_POLICY_SUMMARY_EXISTS_KEY],
        )
    origin_trace_text = str(age5_doc.get(AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY, "")).strip()
    if origin_trace_text:
        snapshot[AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY] = origin_trace_text
    else:
        snapshot[AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY] = build_age5_combined_heavy_policy_origin_trace_text(
            snapshot[AGE5_POLICY_ORIGIN_TRACE_KEY]
        )
    return snapshot


def load_age4_proof_snapshot(index_doc: dict) -> dict[str, str]:
    snapshot = {
        AGE4_PROOF_OK_KEY: "0",
        AGE4_PROOF_FAILED_CRITERIA_KEY: "-1",
        AGE4_PROOF_FAILED_PREVIEW_KEY: "-",
        AGE4_PROOF_SUMMARY_HASH_KEY: "-",
    }
    aggregate_path = artifact_path(index_doc, "aggregate")
    if aggregate_path is None or not aggregate_path.exists():
        return snapshot
    aggregate_doc = load_json(aggregate_path)
    if not isinstance(aggregate_doc, dict):
        return snapshot
    age4_doc = aggregate_doc.get("age4")
    if not isinstance(age4_doc, dict):
        return snapshot
    snapshot[AGE4_PROOF_OK_KEY] = "1" if bool(age4_doc.get("proof_artifact_ok", False)) else "0"
    failed = age4_doc.get("proof_artifact_failed_criteria")
    if isinstance(failed, list):
        snapshot[AGE4_PROOF_FAILED_CRITERIA_KEY] = str(len(failed))
    preview = str(age4_doc.get("proof_artifact_failed_preview", "")).strip()
    if preview:
        snapshot[AGE4_PROOF_FAILED_PREVIEW_KEY] = preview
    summary_hash = str(age4_doc.get("proof_artifact_summary_hash", "")).strip()
    if summary_hash:
        snapshot[AGE4_PROOF_SUMMARY_HASH_KEY] = summary_hash
    return snapshot


def validate_triage_artifact_row(name: str, row: dict, allow_exists_upgrade: bool = False) -> str | None:
    path_text = str(row.get("path", "")).strip()
    path_norm = str(row.get("path_norm", "")).strip()
    exists_value = row.get("exists")
    if not path_text:
        return f"triage artifacts.{name}.path missing"
    expected_norm = normalize_path_text(path_text)
    if path_norm != expected_norm:
        return f"triage artifacts.{name}.path_norm mismatch triage={path_norm} expected={expected_norm}"
    if not isinstance(exists_value, bool):
        return f"triage artifacts.{name}.exists must be bool"
    resolved = resolve_path(path_text)
    if resolved is None:
        return f"triage artifacts.{name}.path resolve failed"
    expected_exists = bool(resolved.exists())
    if exists_value != expected_exists:
        if allow_exists_upgrade and (not bool(exists_value)) and expected_exists:
            return None
        return (
            f"triage artifacts.{name}.exists mismatch triage={int(exists_value)} "
            f"actual={int(expected_exists)} path={resolved}"
        )
    return None


def default_report_dir() -> str:
    preferred = Path("I:/home/urihanl/ddn/codex/build/reports")
    if os.name == "nt":
        try:
            preferred.mkdir(parents=True, exist_ok=True)
        except OSError:
            pass
        return str(preferred)
    return "build/reports"


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate emitter outputs (brief/triage) against report index")
    parser.add_argument("--report-dir", default=default_report_dir(), help="report directory")
    parser.add_argument("--index-pattern", default="*ci_gate_report_index.detjson", help="index file glob")
    parser.add_argument("--prefix", default="", help="optional report prefix")
    parser.add_argument("--require-brief", action="store_true", help="require ci_fail_brief_txt artifact")
    parser.add_argument("--require-triage", action="store_true", help="require ci_fail_triage_json artifact")
    parser.add_argument(
        "--allow-triage-exists-upgrade",
        action="store_true",
        help="allow triage artifacts.exists=false while the artifact exists now (late-emitted artifact)",
    )
    args = parser.parse_args()

    report_dir = Path(args.report_dir)
    if not report_dir.exists():
        return fail(f"missing report-dir: {report_dir}", code=CODES["REPORT_DIR_MISSING"])
    index_path, index_doc = select_latest_index(report_dir, args.index_pattern, args.prefix.strip())
    if index_path is None or not isinstance(index_doc, dict):
        return fail(
            f"index not found in {report_dir} pattern={args.index_pattern} prefix={args.prefix.strip() or '-'}",
            code=CODES["INDEX_NOT_FOUND"],
        )
    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return fail("index.reports missing", code=CODES["INDEX_REPORTS_MISSING"])

    result_path = artifact_path(index_doc, "ci_gate_result_json")
    if result_path is None:
        return fail("index missing reports.ci_gate_result_json", code=CODES["INDEX_RESULT_PATH_MISSING"])
    result_doc = load_json(result_path)
    if not isinstance(result_doc, dict):
        return fail(f"invalid result json: {result_path}", code=CODES["RESULT_JSON_INVALID"])
    if str(result_doc.get("schema", "")).strip() != "ddn.ci.gate_result.v1":
        return fail(f"result schema mismatch: {result_doc.get('schema')}", code=CODES["RESULT_SCHEMA_MISMATCH"])
    result_status = str(result_doc.get("status", "")).strip() or "unknown"
    result_reason = str(result_doc.get("reason", "-")).strip() or "-"
    try:
        result_failed_steps = int(result_doc.get("failed_steps", 0))
    except Exception:
        return fail("result failed_steps must be int", code=CODES["RESULT_FAILED_STEPS_TYPE"])
    if result_failed_steps < 0:
        return fail(f"result failed_steps invalid: {result_failed_steps}", code=CODES["RESULT_FAILED_STEPS_NEGATIVE"])
    index_prefix = str(index_doc.get("report_prefix", "")).strip()
    summary_line_path = artifact_path(index_doc, "summary_line")
    summary_line_text = load_line(summary_line_path) if summary_line_path is not None else ""
    summary_path = artifact_path(index_doc, "summary")
    if summary_path is None:
        return fail("index missing reports.summary", code=CODES["INDEX_SUMMARY_PATH_MISSING"])
    summary_status = ""
    summary_failed_steps: list[str] = []
    summary_failed_step_details: dict[str, str] = {}
    summary_failed_step_logs: dict[str, dict[str, str]] = {}
    summary_failed_step_detail_order: list[str] = []
    summary_failed_step_logs_order: list[str] = []
    summary_kv: dict[str, str] = {}
    summary_report_exists = bool(summary_path.exists())
    if summary_report_exists:
        summary_report = parse_summary_report(summary_path)
        if not isinstance(summary_report, dict):
            return fail(f"invalid summary report: {summary_path}")
        summary_status = str(summary_report.get("status", "")).strip()
        parsed_failed_steps = summary_report.get("failed_steps")
        parsed_failed_step_details = summary_report.get("failed_step_details")
        parsed_failed_step_logs = summary_report.get("failed_step_logs")
        parsed_failed_step_detail_order = summary_report.get("failed_step_detail_order")
        parsed_failed_step_logs_order = summary_report.get("failed_step_logs_order")
        if not isinstance(parsed_failed_steps, list):
            return fail("summary failed_steps must be list")
        if not isinstance(parsed_failed_step_details, dict):
            return fail("summary failed_step_details must be object")
        if not isinstance(parsed_failed_step_logs, dict):
            return fail("summary failed_step_logs must be object")
        if not isinstance(parsed_failed_step_detail_order, list):
            return fail("summary failed_step_detail_order must be list")
        if not isinstance(parsed_failed_step_logs_order, list):
            return fail("summary failed_step_logs_order must be list")
        summary_failed_steps = [str(step).strip() for step in parsed_failed_steps if str(step).strip()]
        summary_failed_step_details = {str(k).strip(): str(v) for k, v in parsed_failed_step_details.items() if str(k).strip()}
        summary_failed_step_logs = {
            str(k).strip(): dict(v) for k, v in parsed_failed_step_logs.items() if str(k).strip() and isinstance(v, dict)
        }
        summary_failed_step_detail_order = [
            str(step).strip() for step in parsed_failed_step_detail_order if str(step).strip()
        ]
        summary_failed_step_logs_order = [
            str(step).strip() for step in parsed_failed_step_logs_order if str(step).strip()
        ]
        parsed_kv = summary_report.get("kv")
        if isinstance(parsed_kv, dict):
            summary_kv = {
                str(k).strip(): str(v).strip()
                for k, v in parsed_kv.items()
                if str(k).strip()
            }
    if result_status not in ("pass", "fail"):
        return fail(f"unsupported result status: {result_status}", code=CODES["RESULT_STATUS_UNSUPPORTED"])
    if result_status == "pass" and result_failed_steps != 0:
        return fail(
            f"pass result must have failed_steps=0, got {result_failed_steps}",
            code=CODES["RESULT_PASS_FAILED_STEPS"],
        )
    if result_status == "fail" and result_failed_steps <= 0:
        return fail(
            f"fail result must have failed_steps>0, got {result_failed_steps}",
            code=CODES["RESULT_FAIL_FAILED_STEPS"],
        )
    summary_status_known = summary_status in ("pass", "fail")
    if summary_report_exists and summary_status_known:
        if summary_status != result_status:
            return fail(
                f"summary status mismatch summary={summary_status or '-'} result={result_status}",
                code=CODES["SUMMARY_STATUS_MISMATCH"],
            )
        if result_status == "pass":
            if summary_failed_steps:
                return fail(f"pass summary must have empty failed_steps, got {','.join(summary_failed_steps)}")
            if summary_failed_step_details:
                return fail("pass summary must not contain failed_step_detail rows")
            if summary_failed_step_logs:
                return fail("pass summary must not contain failed_step_logs rows")
            summary_profile = str(summary_kv.get("ci_sanity_gate_profile", "")).strip() or "full"
            if summary_profile not in VALID_SANITY_PROFILES:
                summary_profile = "full"
            selftest_summary_keys = {
                "ci_profile_matrix_gate_selftest_ok": (
                    "ci_profile_matrix_gate_selftest",
                    {"full", "core_lang", "seamgrim"},
                ),
                "age5_close_digest_selftest_ok": (
                    "age5_close_digest_selftest",
                    {"full", "core_lang", "seamgrim"},
                ),
                "ci_pack_golden_overlay_compare_selftest_ok": (
                    "ci_pack_golden_overlay_compare_selftest",
                    {"full", "core_lang", "seamgrim"},
                ),
                "ci_pack_golden_overlay_session_selftest_ok": (
                    "ci_pack_golden_overlay_session_selftest",
                    {"full", "core_lang", "seamgrim"},
                ),
                "ci_sanity_pack_golden_lang_consistency_ok": (
                    "ci_pack_golden_lang_consistency_selftest",
                    {"full", "core_lang"},
                ),
                "ci_sanity_pack_golden_metadata_ok": (
                    "ci_pack_golden_metadata_selftest",
                    {"full", "core_lang"},
                ),
                "ci_sanity_pack_golden_graph_export_ok": (
                    "ci_pack_golden_graph_export_selftest",
                    {"full", "core_lang"},
                ),
                "ci_sanity_canon_ast_dpack_ok": (
                    "ci_canon_ast_dpack_selftest",
                    {"full", "core_lang"},
                ),
                "ci_sanity_contract_tier_unsupported_ok": (
                    "contract_tier_unsupported_check",
                    {"full", "core_lang"},
                ),
                "ci_sanity_contract_tier_age3_min_enforcement_ok": (
                    "contract_tier_age3_min_enforcement_check",
                    {"full", "core_lang"},
                ),
                "ci_sanity_stdlib_catalog_ok": (
                    "stdlib_catalog_check",
                    {"full", "core_lang"},
                ),
                "ci_sanity_stdlib_catalog_selftest_ok": (
                    "stdlib_catalog_check_selftest",
                    {"full", "core_lang"},
                ),
                "ci_sanity_tensor_v0_pack_ok": (
                    "tensor_v0_pack_check",
                    {"full", "core_lang"},
                ),
                "ci_sanity_tensor_v0_cli_ok": (
                    "tensor_v0_cli_check",
                    {"full", "core_lang"},
                ),
                "ci_sanity_fixed64_darwin_real_report_contract_ok": (
                    "fixed64_darwin_real_report_contract_check",
                    {"full", "core_lang", "seamgrim"},
                ),
                "ci_sanity_fixed64_darwin_real_report_live_ok": (
                    "fixed64_darwin_real_report_live_check",
                    {"full", "core_lang", "seamgrim"},
                ),
                "ci_sanity_fixed64_darwin_real_report_readiness_selftest_ok": (
                    "fixed64_darwin_real_report_readiness_check_selftest",
                    {"full", "core_lang", "seamgrim"},
                ),
                "ci_sanity_map_access_contract_ok": (
                    "map_access_contract_check",
                    {"full", "core_lang"},
                ),
                "ci_sanity_registry_strict_audit_ok": (
                    "gaji_registry_strict_audit_check",
                    {"full", "core_lang"},
                ),
                "ci_sanity_registry_defaults_ok": (
                    "gaji_registry_defaults_check",
                    {"full", "core_lang"},
                ),
                "ci_sanity_dynamic_source_profile_split_selftest_ok": (
                    "ci_sanity_dynamic_source_profile_split_selftest",
                    {"full", "core_lang", "seamgrim"},
                ),
            }
            for key, valid_profiles in SANITY_RUNTIME_HELPER_SUMMARY_FIELDS:
                value = str(summary_kv.get(key, "")).strip()
                if not value:
                    if key in OPTIONAL_FIXED64_LIVE_SANITY_SUMMARY_KEYS:
                        continue
                    return fail(f"pass summary missing key: {key}", code=CODES["SUMMARY_SELFTEST_KEY_MISSING"])
                if value not in VALID_RUNTIME_HELPER_SUMMARY_VALUES:
                    return fail(
                        f"pass summary invalid {key}: {value}",
                        code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
                    )
                expected_value = "1" if summary_profile in valid_profiles else "na"
                if value != expected_value:
                    return fail(
                        f"pass summary requires {key}={expected_value}, got {value}",
                        code=CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
                    )
            for key, value_kind, valid_profiles in SANITY_RUNTIME_HELPER_TEXT_FIELDS:
                value = str(summary_kv.get(key, "")).strip()
                if not value:
                    if key in OPTIONAL_FIXED64_LIVE_SANITY_SUMMARY_KEYS:
                        continue
                    return fail(f"pass summary missing key: {key}", code=CODES["SUMMARY_SELFTEST_KEY_MISSING"])
                if summary_profile in valid_profiles:
                    if value_kind == "path":
                        resolved = resolve_path(value)
                        if value == "-" or resolved is None or not resolved.exists():
                            return fail(
                                f"pass summary invalid {key}: {value}",
                                code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
                            )
                    elif value_kind == "schema":
                        if value != AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA:
                            return fail(
                                f"pass summary requires {key}={AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA}, got {value}",
                                code=CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
                            )
                    elif value_kind == "step_path":
                        resolved = resolve_path(value)
                        if value == "-" or resolved is None or not resolved.exists():
                            return fail(
                                f"pass summary invalid {key}: {value}",
                                code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
                            )
                    elif value_kind == "step_schema":
                        if value != SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA:
                            return fail(
                                f"pass summary requires {key}={SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA}, got {value}",
                                code=CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
                            )
                    elif value_kind == "pack_evidence_schema":
                        if value != SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA:
                            return fail(
                                f"pass summary requires {key}={SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA}, got {value}",
                                code=CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
                            )
                    elif value_kind == "step_checked_files":
                        try:
                            files_num = int(value)
                        except Exception:
                            return fail(
                                f"pass summary invalid {key}: {value}",
                                code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
                            )
                        if files_num < SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES:
                            return fail(
                                f"pass summary requires {key}>={SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES}, got {files_num}",
                                code=CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
                            )
                    elif value_kind == "step_missing_count":
                        try:
                            missing_num = int(value)
                        except Exception:
                            return fail(
                                f"pass summary invalid {key}: {value}",
                                code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
                            )
                        if missing_num != 0:
                            return fail(
                                f"pass summary requires {key}=0, got {missing_num}",
                                code=CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
                            )
                    elif value_kind == "pack_evidence_docs_issue_count":
                        try:
                            issue_num = int(value)
                        except Exception:
                            return fail(
                                f"pass summary invalid {key}: {value}",
                                code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
                            )
                        if issue_num < 0 or issue_num > 10:
                            return fail(
                                f"pass summary requires 0<={key}<=10, got {issue_num}",
                                code=CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
                            )
                    elif value_kind == "pack_evidence_repo_issue_count":
                        try:
                            issue_num = int(value)
                        except Exception:
                            return fail(
                                f"pass summary invalid {key}: {value}",
                                code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
                            )
                        if issue_num != 0:
                            return fail(
                                f"pass summary requires {key}=0, got {issue_num}",
                                code=CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
                            )
                    elif value_kind in {
                        "codes",
                        "count",
                        "fixed64_live_path",
                        "fixed64_live_status",
                        "fixed64_live_resolved_status",
                        "fixed64_live_resolved_source",
                        "fixed64_live_invalid_count",
                        "fixed64_live_zip",
                        "numeric_factor_policy_schema",
                        "numeric_factor_policy_path",
                        "numeric_factor_policy_text",
                        "numeric_factor_policy_value",
                    }:
                        error = validate_failure_code_field_value(key, value, value_kind)
                        if error is not None:
                            return fail(error, code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"])
                    else:
                        return fail(
                            f"pass summary unsupported value kind {value_kind} for {key}",
                            code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
                        )
                elif value != "-":
                    return fail(
                        f"pass summary requires {key}=- for profile={summary_profile}, got {value}",
                        code=CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
                    )
            for code_key, count_key in FAILURE_CODE_PAIR_KEYS:
                code_value = str(summary_kv.get(code_key, "")).strip()
                count_value = str(summary_kv.get(count_key, "")).strip()
                try:
                    count_num = int(count_value)
                except Exception:
                    return fail(
                        f"pass summary {count_key} is not an integer: {count_value}",
                        code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
                    )
                if code_value == "-" and count_num != 0:
                    return fail(
                        f"pass summary {count_key} must be 0 when {code_key}=-, got {count_num}",
                        code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
                    )
                if code_value != "-":
                    code_items = [token.strip() for token in code_value.split(",") if token.strip()]
                    if len(code_items) != count_num:
                        return fail(
                            f"pass summary {count_key} mismatch {code_key}: count={count_num} codes={len(code_items)}",
                            code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
                        )
            for key, cfg in selftest_summary_keys.items():
                step_name, valid_profiles = cfg
                if summary_profile not in valid_profiles:
                    continue
                value = str(summary_kv.get(key, "")).strip()
                if not value:
                    return fail(f"pass summary missing key: {key}", code=CODES["SUMMARY_SELFTEST_KEY_MISSING"])
                if value not in {"0", "1"}:
                    return fail(
                        f"pass summary invalid {key}: {value}",
                        code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
                    )
                if value != "1":
                    return fail(
                        f"pass summary requires {key}=1, got {value}",
                        code=CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
                    )
                step_ok = read_step_ok(index_doc, step_name)
                if step_ok is not None and int(value) != int(step_ok):
                    return fail(
                        f"summary/index selftest mismatch key={key} summary={value} step_ok={int(step_ok)}",
                        code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                    )
            profile_matrix_selftest_path = artifact_path(index_doc, "ci_profile_matrix_gate_selftest")
            if profile_matrix_selftest_path is None:
                return fail(
                    "index missing reports.ci_profile_matrix_gate_selftest",
                    code=CODES["INDEX_REPORT_KEY_MISSING"],
                )
            profile_matrix_snap = load_profile_matrix_selftest_snapshot(profile_matrix_selftest_path)
            if not isinstance(profile_matrix_snap, dict):
                return fail(
                    f"invalid ci_profile_matrix_gate_selftest json: {profile_matrix_selftest_path}",
                    code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                )
            for key in PROFILE_MATRIX_SELFTEST_SUMMARY_REQUIRED_KEYS:
                value = str(summary_kv.get(key, "")).strip()
                if not value:
                    return fail(
                        f"pass summary missing key: {key}",
                        code=CODES["SUMMARY_SELFTEST_KEY_MISSING"],
                    )
            for key in SEAMGRIM_FOCUS_SUMMARY_REQUIRED_KEYS:
                value = str(summary_kv.get(key, "")).strip()
                if not value:
                    return fail(
                        f"pass summary missing key: {key}",
                        code=CODES["SUMMARY_SELFTEST_KEY_MISSING"],
                    )
                if value not in VALID_SEAMGRIM_FOCUS_SUMMARY_STATUS:
                    return fail(
                        f"pass summary invalid {key}: {value}",
                        code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
                    )
            if summary_kv.get("seamgrim_group_id_summary_status") != "ok":
                return fail(
                    "pass summary requires seamgrim_group_id_summary_status=ok",
                    code=CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
                )
            age_close_values = {
                summary_key: str(summary_kv.get(summary_key, "")).strip()
                for summary_key, _report_key, _expected_schema in AGE_CLOSE_STATUS_SUMMARY_SPECS
            }
            age_close_missing = [key for key, value in age_close_values.items() if not value]
            # Aggregate gate는 emit-artifacts baseline/required 단계에서 preview summary를 먼저 생성한다.
            # preview 단계에서는 age{2,3,4,5}_status 키를 아직 쓰지 않으므로 전체 미존재를 허용한다.
            # 단, 일부만 존재하면 요약 계약이 깨진 상태로 간주한다.
            age_close_preview_mode = bool(age_close_missing) and len(age_close_missing) == len(AGE_CLOSE_STATUS_SUMMARY_SPECS)
            if age_close_missing and not age_close_preview_mode:
                return fail(
                    "pass summary age close status keys are partially missing: "
                    + ",".join(age_close_missing),
                    code=CODES["SUMMARY_SELFTEST_KEY_MISSING"],
                )
            if not age_close_preview_mode:
                for summary_key, report_key, expected_schema in AGE_CLOSE_STATUS_SUMMARY_SPECS:
                    summary_value = age_close_values[summary_key]
                    report_path = artifact_path(index_doc, report_key)
                    if report_path is None:
                        return fail(
                            f"index missing reports.{report_key}",
                            code=CODES["INDEX_REPORT_KEY_MISSING"],
                        )
                    report_text = str(report_path)
                    if summary_value != report_text:
                        return fail(
                            f"summary/{summary_key} mismatch summary={summary_value} report={report_text}",
                            code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                        )
                    if not report_path.exists():
                        return fail(
                            f"pass summary target missing for {summary_key}: {report_path}",
                            code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                        )
                    report_doc = load_json(report_path)
                    if not isinstance(report_doc, dict):
                        return fail(
                            f"invalid json for {summary_key}: {report_path}",
                            code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                        )
                    actual_schema = str(report_doc.get("schema", "")).strip()
                    if actual_schema != expected_schema:
                        return fail(
                            f"schema mismatch for {summary_key}: {actual_schema} expected={expected_schema}",
                            code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                        )
            age5_child_summary_error = validate_age5_child_summary_tokens(summary_kv, source="summary")
            if age5_child_summary_error is not None:
                return fail(age5_child_summary_error, code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"])
            for key in (
                "ci_profile_matrix_gate_selftest_total_elapsed_ms",
                "ci_profile_matrix_gate_selftest_core_lang_elapsed_ms",
                "ci_profile_matrix_gate_selftest_full_elapsed_ms",
                "ci_profile_matrix_gate_selftest_seamgrim_elapsed_ms",
            ):
                value = str(summary_kv.get(key, "")).strip()
                if not validate_runtime5_elapsed_text(value):
                    return fail(
                        f"pass summary invalid {key}: {value}",
                        code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
                    )
            if summary_kv.get("ci_profile_matrix_gate_selftest_status") != "pass":
                return fail(
                    "pass summary requires ci_profile_matrix_gate_selftest_status=pass",
                    code=CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
                )
            if summary_kv.get("ci_profile_matrix_gate_selftest_ok") != "1":
                return fail(
                    "pass summary requires ci_profile_matrix_gate_selftest_ok=1",
                    code=CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
                )
            for key, expected in (
                ("ci_profile_matrix_gate_selftest_report", profile_matrix_snap["path"]),
                ("ci_profile_matrix_gate_selftest_status", profile_matrix_snap["status"]),
                ("ci_profile_matrix_gate_selftest_ok", profile_matrix_snap["ok"]),
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
            ):
                actual = str(summary_kv.get(key, "")).strip()
                if actual != expected:
                    return fail(
                        f"summary/profile_matrix_selftest mismatch key={key} summary={actual} report={expected}",
                        code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                    )
        if result_status == "fail" and not summary_failed_steps:
            return fail("fail summary missing failed_steps")

    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return fail("index.reports missing", code=CODES["INDEX_REPORTS_MISSING"])
    for key in ("ci_fail_brief_txt", "ci_fail_triage_json"):
        if not str(reports.get(key, "")).strip():
            return fail(f"index missing reports.{key}", code=CODES["INDEX_REPORT_KEY_MISSING"])
    runtime5_checklist_path = artifact_path(index_doc, "seamgrim_5min_checklist")
    if result_status == "pass" and runtime5_checklist_path is not None and runtime5_checklist_path.exists():
        runtime5_snap = load_runtime5_checklist_snapshot(runtime5_checklist_path)
        if not isinstance(runtime5_snap, dict):
            return fail(
                f"invalid runtime5 checklist json: {runtime5_checklist_path}",
                code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
            )
        for key in RUNTIME5_SUMMARY_REQUIRED_KEYS:
            value = str(summary_kv.get(key, "")).strip()
            if not value:
                return fail(f"pass summary missing key: {key}", code=CODES["SUMMARY_SELFTEST_KEY_MISSING"])
        if summary_kv.get("seamgrim_5min_checklist") != runtime5_snap["path"]:
            return fail(
                "summary/runtime5 checklist path mismatch "
                f"summary={summary_kv.get('seamgrim_5min_checklist')} report={runtime5_snap['path']}",
                code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
            )
        summary_checklist_ok = str(summary_kv.get("seamgrim_5min_checklist_ok", "")).strip()
        if summary_checklist_ok not in {"0", "1"}:
            return fail(
                f"pass summary invalid seamgrim_5min_checklist_ok: {summary_checklist_ok}",
                code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
            )
        for key in (
            "seamgrim_runtime_5min_rewrite_motion_projectile",
            "seamgrim_runtime_5min_moyang_view_boundary",
            "seamgrim_runtime_5min_pendulum_tetris_showcase",
        ):
            value = str(summary_kv.get(key, "")).strip()
            if value not in {"0", "1", "na"}:
                return fail(
                    f"pass summary invalid {key}: {value}",
                    code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
                )
        for key in (
            "seamgrim_runtime_5min_rewrite_status",
            "seamgrim_runtime_5min_moyang_status",
            "seamgrim_runtime_5min_pendulum_tetris_showcase_status",
        ):
            value = str(summary_kv.get(key, "")).strip()
            if value not in VALID_RUNTIME5_ITEM_STATUS:
                return fail(
                    f"pass summary invalid {key}: {value}",
                    code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
                )
        for key in (
            "seamgrim_runtime_5min_rewrite_elapsed_ms",
            "seamgrim_runtime_5min_moyang_elapsed_ms",
            "seamgrim_runtime_5min_pendulum_tetris_showcase_elapsed_ms",
        ):
            value = str(summary_kv.get(key, "")).strip()
            if not validate_runtime5_elapsed_text(value):
                return fail(
                    f"pass summary invalid {key}: {value}",
                    code=CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
                )
        if runtime5_snap["ok"] != "1":
            return fail(
                f"runtime5 checklist pass artifact must be ok=1, got {runtime5_snap['ok']}",
                code=CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
            )
        for key, expected in (
            ("seamgrim_5min_checklist_ok", runtime5_snap["ok"]),
            ("seamgrim_runtime_5min_rewrite_motion_projectile", runtime5_snap["rewrite_ok"]),
            ("seamgrim_runtime_5min_rewrite_elapsed_ms", runtime5_snap["rewrite_elapsed_ms"]),
            ("seamgrim_runtime_5min_rewrite_status", runtime5_snap["rewrite_status"]),
            ("seamgrim_runtime_5min_moyang_view_boundary", runtime5_snap["moyang_ok"]),
            ("seamgrim_runtime_5min_moyang_elapsed_ms", runtime5_snap["moyang_elapsed_ms"]),
            ("seamgrim_runtime_5min_moyang_status", runtime5_snap["moyang_status"]),
            ("seamgrim_runtime_5min_pendulum_tetris_showcase", runtime5_snap["showcase_ok"]),
            (
                "seamgrim_runtime_5min_pendulum_tetris_showcase_elapsed_ms",
                runtime5_snap["showcase_elapsed_ms"],
            ),
            ("seamgrim_runtime_5min_pendulum_tetris_showcase_status", runtime5_snap["showcase_status"]),
        ):
            actual = str(summary_kv.get(key, "")).strip()
            if actual != expected:
                return fail(
                    f"summary/runtime5 mismatch key={key} summary={actual} report={expected}",
                    code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                )
        if runtime5_snap["rewrite_ok"] != "1" or runtime5_snap["rewrite_status"] != "ok":
            return fail(
                "runtime5 checklist pass artifact requires rewrite item ok=1 status=ok",
                code=CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
            )
        if runtime5_snap["moyang_ok"] != "1" or runtime5_snap["moyang_status"] != "ok":
            return fail(
                "runtime5 checklist pass artifact requires moyang item ok=1 status=ok",
                code=CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
            )
        if runtime5_snap["showcase_ok"] != "1" or runtime5_snap["showcase_status"] != "ok":
            return fail(
                "runtime5 checklist pass artifact requires showcase item ok=1 status=ok",
                code=CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
            )

    sanity_path = artifact_path(index_doc, "ci_sanity_gate")
    if sanity_path is None:
        return fail("index missing reports.ci_sanity_gate", code=CODES["SANITY_PATH_MISSING"])
    sanity_doc = load_json(sanity_path)
    if not isinstance(sanity_doc, dict):
        return fail(f"invalid ci_sanity_gate json: {sanity_path}", code=CODES["SANITY_JSON_INVALID"])
    if str(sanity_doc.get("schema", "")).strip() != "ddn.ci.sanity_gate.v1":
        return fail(
            f"ci_sanity_gate schema mismatch: {sanity_doc.get('schema')}",
            code=CODES["SANITY_SCHEMA_MISMATCH"],
        )
    sanity_status = str(sanity_doc.get("status", "")).strip() or "unknown"
    if sanity_status not in ("pass", "fail"):
        return fail(f"unsupported ci_sanity_gate status: {sanity_status}", code=CODES["SANITY_STATUS_UNSUPPORTED"])
    sanity_profile = str(sanity_doc.get("profile", "")).strip() or "full"
    if sanity_profile not in VALID_SANITY_PROFILES:
        return fail(f"unsupported ci_sanity_gate profile: {sanity_profile}", code=CODES["SANITY_STATUS_UNSUPPORTED"])
    sanity_steps = sanity_doc.get("steps")
    if not isinstance(sanity_steps, list):
        return fail("ci_sanity_gate steps must be list", code=CODES["SANITY_STEPS_TYPE"])
    sanity_failed_steps = 0
    sanity_step_index: dict[str, dict] = {}
    for idx, row in enumerate(sanity_steps):
        if not isinstance(row, dict):
            return fail(f"ci_sanity_gate steps[{idx}] must be object", code=CODES["SANITY_STEPS_TYPE"])
        step_name = str(row.get("step", "")).strip()
        if step_name:
            sanity_step_index[step_name] = row
        if not bool(row.get("ok", False)):
            sanity_failed_steps += 1
    if sanity_status == "pass":
        if sanity_failed_steps != 0:
            return fail(
                f"ci_sanity_gate pass requires failed_steps=0, got {sanity_failed_steps}",
                code=CODES["SANITY_PASS_FAILED_STEPS"],
            )
        sanity_code = str(sanity_doc.get("code", "")).strip()
        if sanity_code != "OK":
            return fail(
                f"ci_sanity_gate pass requires code=OK, got {sanity_code}",
                code=CODES["SANITY_STATUS_MISMATCH"],
            )
        sanity_step = str(sanity_doc.get("step", "")).strip()
        if sanity_step != "all":
            return fail(
                f"ci_sanity_gate pass requires step=all, got {sanity_step}",
                code=CODES["SANITY_STATUS_MISMATCH"],
            )
        for key, valid_profiles in SANITY_RUNTIME_HELPER_SUMMARY_FIELDS:
            value = str(sanity_doc.get(key, "")).strip()
            if not value:
                if key in OPTIONAL_FIXED64_LIVE_SANITY_SUMMARY_KEYS:
                    continue
                return fail(f"ci_sanity_gate missing summary key: {key}", code=CODES["SANITY_REQUIRED_STEP_MISSING"])
            if value not in VALID_RUNTIME_HELPER_SUMMARY_VALUES:
                return fail(f"ci_sanity_gate invalid summary value: {key}={value}", code=CODES["SANITY_REQUIRED_STEP_FAILED"])
            expected_value = "1" if sanity_profile in valid_profiles else "na"
            if value != expected_value:
                return fail(
                    f"ci_sanity_gate summary mismatch {key}: expected={expected_value} actual={value}",
                    code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                )
            summary_value = str(summary_kv.get(key, "")).strip()
            if summary_report_exists and summary_status_known and result_status == "pass":
                if not summary_value and key in OPTIONAL_FIXED64_LIVE_SANITY_SUMMARY_KEYS:
                    continue
                if summary_value != value:
                    return fail(
                        f"summary/ci_sanity_gate mismatch key={key} summary={summary_value} report={value}",
                        code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                    )
        for key, value_kind, valid_profiles in SANITY_RUNTIME_HELPER_TEXT_FIELDS:
            value = str(sanity_doc.get(key, "")).strip()
            if not value:
                if key in OPTIONAL_FIXED64_LIVE_SANITY_SUMMARY_KEYS:
                    continue
                return fail(f"ci_sanity_gate missing summary key: {key}", code=CODES["SANITY_REQUIRED_STEP_MISSING"])
            if sanity_profile in valid_profiles:
                if value_kind == "path":
                    resolved = resolve_path(value)
                    if value == "-" or resolved is None or not resolved.exists():
                        return fail(
                            f"ci_sanity_gate invalid summary value: {key}={value}",
                            code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                        )
                elif value_kind == "schema":
                    if value != AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA:
                        return fail(
                            "ci_sanity_gate summary mismatch "
                            f"{key}: expected={AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA} actual={value}",
                            code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                        )
                elif value_kind == "step_path":
                    resolved = resolve_path(value)
                    if value == "-" or resolved is None or not resolved.exists():
                        return fail(
                            f"ci_sanity_gate invalid summary value: {key}={value}",
                            code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                        )
                elif value_kind == "step_schema":
                    if value != SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA:
                        return fail(
                            "ci_sanity_gate summary mismatch "
                            f"{key}: expected={SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA} actual={value}",
                            code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                        )
                elif value_kind == "pack_evidence_schema":
                    if value != SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA:
                        return fail(
                            "ci_sanity_gate summary mismatch "
                            f"{key}: expected={SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA} actual={value}",
                            code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                        )
                elif value_kind == "step_checked_files":
                    try:
                        checked_files_num = int(value)
                    except Exception:
                        return fail(
                            f"ci_sanity_gate invalid summary value: {key}={value}",
                            code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                        )
                    if checked_files_num < SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES:
                        return fail(
                            "ci_sanity_gate summary mismatch "
                            f"{key}: expected>={SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES} actual={checked_files_num}",
                            code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                        )
                elif value_kind == "step_missing_count":
                    try:
                        missing_count_num = int(value)
                    except Exception:
                        return fail(
                            f"ci_sanity_gate invalid summary value: {key}={value}",
                            code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                        )
                    if missing_count_num != 0:
                        return fail(
                            "ci_sanity_gate summary mismatch "
                            f"{key}: expected=0 actual={missing_count_num}",
                            code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                        )
                elif value_kind == "pack_evidence_docs_issue_count":
                    try:
                        issue_count_num = int(value)
                    except Exception:
                        return fail(
                            f"ci_sanity_gate invalid summary value: {key}={value}",
                            code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                        )
                    if issue_count_num < 0 or issue_count_num > 10:
                        return fail(
                            "ci_sanity_gate summary mismatch "
                            f"{key}: expected within 0..10 actual={issue_count_num}",
                            code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                        )
                elif value_kind == "pack_evidence_repo_issue_count":
                    try:
                        issue_count_num = int(value)
                    except Exception:
                        return fail(
                            f"ci_sanity_gate invalid summary value: {key}={value}",
                            code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                        )
                    if issue_count_num != 0:
                        return fail(
                            "ci_sanity_gate summary mismatch "
                            f"{key}: expected=0 actual={issue_count_num}",
                            code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                        )
                elif value_kind in {
                    "codes",
                    "count",
                    "fixed64_live_path",
                    "fixed64_live_status",
                    "fixed64_live_resolved_status",
                    "fixed64_live_resolved_source",
                    "fixed64_live_invalid_count",
                    "fixed64_live_zip",
                    "numeric_factor_policy_schema",
                    "numeric_factor_policy_path",
                    "numeric_factor_policy_text",
                    "numeric_factor_policy_value",
                }:
                    error = validate_failure_code_field_value(key, value, value_kind)
                    if error is not None:
                        return fail(error, code=CODES["SANITY_REQUIRED_STEP_FAILED"])
                else:
                    return fail(
                        f"ci_sanity_gate unsupported value kind {value_kind} for {key}",
                        code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                    )
            elif value != "-":
                return fail(
                    f"ci_sanity_gate summary mismatch {key}: expected=- actual={value}",
                    code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                )
            summary_value = str(summary_kv.get(key, "")).strip()
            if summary_report_exists and summary_status_known and result_status == "pass":
                if not summary_value and key in OPTIONAL_FIXED64_LIVE_SANITY_SUMMARY_KEYS:
                    continue
                if summary_value != value:
                    return fail(
                        f"summary/ci_sanity_gate mismatch key={key} summary={summary_value} report={value}",
                        code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                    )
        for code_key, count_key in FAILURE_CODE_PAIR_KEYS:
            code_value = str(sanity_doc.get(code_key, "")).strip()
            count_value = str(sanity_doc.get(count_key, "")).strip()
            try:
                count_num = int(count_value)
            except Exception:
                return fail(
                    f"ci_sanity_gate {count_key} is not an integer: {count_value}",
                    code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                )
            if code_value == "-" and count_num != 0:
                return fail(
                    f"ci_sanity_gate {count_key} must be 0 when {code_key}=-, got {count_num}",
                    code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                )
            if code_value != "-":
                code_items = [token.strip() for token in code_value.split(",") if token.strip()]
                if len(code_items) != count_num:
                    return fail(
                        f"ci_sanity_gate {count_key} mismatch {code_key}: count={count_num} codes={len(code_items)}",
                        code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                    )
        for key, expected_value in SANITY_RUNTIME_HELPER_CONTRACT_FIELDS:
            value = str(sanity_doc.get(key, "")).strip()
            if value != expected_value:
                return fail(
                    f"ci_sanity_gate contract mismatch {key}: expected={expected_value} actual={value}",
                    code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                )
            summary_value = str(summary_kv.get(key, "")).strip()
            if summary_report_exists and summary_status_known and result_status == "pass" and summary_value != value:
                return fail(
                    f"summary/ci_sanity_gate contract mismatch key={key} summary={summary_value} report={value}",
                    code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                )
        for required_step in resolve_required_sanity_steps(sanity_profile):
            row = sanity_step_index.get(required_step)
            if row is None:
                return fail(
                    f"ci_sanity_gate pass missing required step: {required_step}",
                    code=CODES["SANITY_REQUIRED_STEP_MISSING"],
                )
            if not bool(row.get("ok", False)):
                return fail(
                    f"ci_sanity_gate pass step must be ok=1: {required_step}",
                    code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                )
            try:
                step_rc = int(row.get("returncode", -1))
            except Exception:
                step_rc = -1
            if step_rc != 0:
                return fail(
                    f"ci_sanity_gate pass step returncode must be 0: {required_step} rc={row.get('returncode')}",
                    code=CODES["SANITY_REQUIRED_STEP_FAILED"],
                )
    else:
        if sanity_failed_steps <= 0:
            return fail(
                "ci_sanity_gate fail requires at least one failed step",
                code=CODES["SANITY_FAIL_FAILED_STEPS"],
            )
    if result_status == "pass" and sanity_status != "pass":
        return fail(
            f"ci_sanity_gate status mismatch result={result_status} sanity={sanity_status}",
            code=CODES["SANITY_STATUS_MISMATCH"],
        )

    sync_readiness_path = artifact_path(index_doc, "ci_sync_readiness")
    if sync_readiness_path is None:
        return fail("index missing reports.ci_sync_readiness", code=CODES["SYNC_READINESS_PATH_MISSING"])
    sync_readiness_doc = load_json(sync_readiness_path)
    if not isinstance(sync_readiness_doc, dict):
        return fail(f"invalid ci_sync_readiness json: {sync_readiness_path}", code=CODES["SYNC_READINESS_JSON_INVALID"])
    if str(sync_readiness_doc.get("schema", "")).strip() != "ddn.ci.sync_readiness.v1":
        return fail(
            f"ci_sync_readiness schema mismatch: {sync_readiness_doc.get('schema')}",
            code=CODES["SYNC_READINESS_SCHEMA_MISMATCH"],
        )
    sync_readiness_status = str(sync_readiness_doc.get("status", "")).strip() or "unknown"
    if sync_readiness_status not in ("pass", "fail"):
        return fail(
            f"unsupported ci_sync_readiness status: {sync_readiness_status}",
            code=CODES["SYNC_READINESS_STATUS_UNSUPPORTED"],
        )
    if result_status == "pass" and sync_readiness_status != "pass":
        return fail(
            f"ci_sync_readiness status mismatch result={result_status} sync={sync_readiness_status}",
            code=CODES["SYNC_READINESS_STATUS_MISMATCH"],
        )
    if sync_readiness_status == "pass":
        sync_code = str(sync_readiness_doc.get("code", "")).strip()
        sync_step = str(sync_readiness_doc.get("step", "")).strip()
        if sync_code != "OK" or sync_step != "all":
            return fail(
                f"ci_sync_readiness pass fields invalid code={sync_code} step={sync_step}",
                code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
            )
        sync_sanity_profile = str(sync_readiness_doc.get("sanity_profile", "")).strip() or "full"
        if sync_sanity_profile not in VALID_SANITY_PROFILES:
            return fail(
                f"ci_sync_readiness sanity_profile invalid: {sync_sanity_profile}",
                code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
            )
        if sync_sanity_profile != sanity_profile:
            return fail(
                "ci_sync_readiness sanity_profile mismatch "
                f"sync={sync_sanity_profile} sanity={sanity_profile}",
                code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
            )
        for summary_key, source_key, valid_profiles in SYNC_RUNTIME_HELPER_SUMMARY_FIELDS:
            value = str(sync_readiness_doc.get(source_key, "")).strip()
            if not value:
                if source_key in OPTIONAL_FIXED64_LIVE_SANITY_SUMMARY_KEYS:
                    continue
                return fail(
                    f"ci_sync_readiness missing summary key: {source_key}",
                    code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                )
            if value not in VALID_RUNTIME_HELPER_SUMMARY_VALUES:
                return fail(
                    f"ci_sync_readiness invalid summary value: {source_key}={value}",
                    code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                )
            expected_value = expected_sync_runtime_helper_summary_value(
                summary_key,
                sync_sanity_profile,
                valid_profiles,
            )
            if value != expected_value:
                return fail(
                    f"ci_sync_readiness summary mismatch {source_key}: expected={expected_value} actual={value}",
                    code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                )
            if value != str(sanity_doc.get(source_key, "")).strip():
                return fail(
                    f"ci_sync_readiness/ci_sanity_gate summary mismatch key={source_key}",
                    code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                )
            summary_value = str(summary_kv.get(summary_key, "")).strip()
            if summary_report_exists and summary_status_known and result_status == "pass":
                if not summary_value and summary_key in OPTIONAL_FIXED64_LIVE_SYNC_SUMMARY_KEYS:
                    continue
                if summary_value != value:
                    return fail(
                        f"summary/ci_sync_readiness mismatch key={summary_key} summary={summary_value} report={value}",
                        code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                    )
        for sanity_key, sync_key in AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS:
            sanity_value = str(sync_readiness_doc.get(sanity_key, "")).strip()
            sync_value = str(sync_readiness_doc.get(sync_key, "")).strip()
            if not sync_value:
                return fail(
                    f"ci_sync_readiness missing criteria sync key: {sync_key}",
                    code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                )
            if sync_value != sanity_value:
                return fail(
                    f"ci_sync_readiness criteria sync mismatch {sync_key}: expected={sanity_value} actual={sync_value}",
                    code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                )
        for summary_key, source_key, value_kind, valid_profiles in SYNC_RUNTIME_HELPER_TEXT_FIELDS:
            value = str(sync_readiness_doc.get(source_key, "")).strip()
            if not value:
                if source_key in OPTIONAL_FIXED64_LIVE_SANITY_SUMMARY_KEYS:
                    continue
                return fail(
                    f"ci_sync_readiness missing summary key: {source_key}",
                    code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                )
            if sync_sanity_profile in valid_profiles:
                if value_kind == "path":
                    resolved = resolve_path(value)
                    if value == "-" or resolved is None or not resolved.exists():
                        return fail(
                            f"ci_sync_readiness invalid summary value: {source_key}={value}",
                            code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                        )
                elif value_kind == "schema":
                    if value != AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA:
                        return fail(
                            "ci_sync_readiness summary mismatch "
                            f"{source_key}: expected={AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA} actual={value}",
                            code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                        )
                elif value_kind == "step_path":
                    resolved = resolve_path(value)
                    if value == "-" or resolved is None or not resolved.exists():
                        return fail(
                            f"ci_sync_readiness invalid summary value: {source_key}={value}",
                            code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                        )
                elif value_kind == "step_schema":
                    if value != SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA:
                        return fail(
                            "ci_sync_readiness summary mismatch "
                            f"{source_key}: expected={SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA} actual={value}",
                            code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                        )
                elif value_kind == "pack_evidence_schema":
                    if value != SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA:
                        return fail(
                            "ci_sync_readiness summary mismatch "
                            f"{source_key}: expected={SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA} actual={value}",
                            code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                        )
                elif value_kind == "step_checked_files":
                    try:
                        checked_files_num = int(value)
                    except Exception:
                        return fail(
                            f"ci_sync_readiness invalid summary value: {source_key}={value}",
                            code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                        )
                    if checked_files_num < SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES:
                        return fail(
                            "ci_sync_readiness summary mismatch "
                            f"{source_key}: expected>={SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES} actual={checked_files_num}",
                            code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                        )
                elif value_kind == "step_missing_count":
                    try:
                        missing_count_num = int(value)
                    except Exception:
                        return fail(
                            f"ci_sync_readiness invalid summary value: {source_key}={value}",
                            code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                        )
                    if missing_count_num != 0:
                        return fail(
                            "ci_sync_readiness summary mismatch "
                            f"{source_key}: expected=0 actual={missing_count_num}",
                            code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                        )
                elif value_kind == "pack_evidence_docs_issue_count":
                    try:
                        issue_count_num = int(value)
                    except Exception:
                        return fail(
                            f"ci_sync_readiness invalid summary value: {source_key}={value}",
                            code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                        )
                    if issue_count_num < 0 or issue_count_num > 10:
                        return fail(
                            "ci_sync_readiness summary mismatch "
                            f"{source_key}: expected within 0..10 actual={issue_count_num}",
                            code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                        )
                elif value_kind == "pack_evidence_repo_issue_count":
                    try:
                        issue_count_num = int(value)
                    except Exception:
                        return fail(
                            f"ci_sync_readiness invalid summary value: {source_key}={value}",
                            code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                        )
                    if issue_count_num != 0:
                        return fail(
                            "ci_sync_readiness summary mismatch "
                            f"{source_key}: expected=0 actual={issue_count_num}",
                            code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                        )
                elif value_kind in {
                    "codes",
                    "count",
                    "fixed64_live_path",
                    "fixed64_live_status",
                    "fixed64_live_resolved_status",
                    "fixed64_live_resolved_source",
                    "fixed64_live_invalid_count",
                    "fixed64_live_zip",
                    "numeric_factor_policy_schema",
                    "numeric_factor_policy_path",
                    "numeric_factor_policy_text",
                    "numeric_factor_policy_value",
                }:
                    error = validate_failure_code_field_value(source_key, value, value_kind)
                    if error is not None:
                        return fail(error, code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"])
                else:
                    return fail(
                        f"ci_sync_readiness unsupported value kind {value_kind} for {source_key}",
                        code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                    )
            elif value != "-":
                return fail(
                    f"ci_sync_readiness summary mismatch {source_key}: expected=- actual={value}",
                    code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                )
            if value != str(sanity_doc.get(source_key, "")).strip():
                return fail(
                    f"ci_sync_readiness/ci_sanity_gate summary mismatch key={source_key}",
                    code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                )
            summary_value = str(summary_kv.get(summary_key, "")).strip()
            if summary_report_exists and summary_status_known and result_status == "pass":
                if not summary_value and summary_key in OPTIONAL_FIXED64_LIVE_SYNC_SUMMARY_KEYS:
                    continue
                if summary_value != value:
                    return fail(
                        f"summary/ci_sync_readiness mismatch key={summary_key} summary={summary_value} report={value}",
                        code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                    )
        for summary_code_key, summary_count_key, source_code_key, source_count_key in SYNC_FAILURE_CODE_PAIR_KEYS:
            source_code_value = str(sync_readiness_doc.get(source_code_key, "")).strip()
            source_count_value = str(sync_readiness_doc.get(source_count_key, "")).strip()
            try:
                source_count_num = int(source_count_value)
            except Exception:
                return fail(
                    f"ci_sync_readiness {source_count_key} is not an integer: {source_count_value}",
                    code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                )
            if source_code_value == "-" and source_count_num != 0:
                return fail(
                    f"ci_sync_readiness {source_count_key} must be 0 when {source_code_key}=-, got {source_count_num}",
                    code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                )
            if source_code_value != "-":
                source_code_items = [token.strip() for token in source_code_value.split(",") if token.strip()]
                if len(source_code_items) != source_count_num:
                    return fail(
                        "ci_sync_readiness failure-code pair mismatch "
                        f"{source_count_key} count={source_count_num} codes={len(source_code_items)}",
                        code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                    )
            summary_code_value = str(summary_kv.get(summary_code_key, "")).strip()
            summary_count_value = str(summary_kv.get(summary_count_key, "")).strip()
            try:
                summary_count_num = int(summary_count_value)
            except Exception:
                return fail(
                    f"summary {summary_count_key} is not an integer: {summary_count_value}",
                    code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                )
            if summary_code_value == "-" and summary_count_num != 0:
                return fail(
                    f"summary {summary_count_key} must be 0 when {summary_code_key}=-, got {summary_count_num}",
                    code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                )
            if summary_code_value != "-":
                summary_code_items = [token.strip() for token in summary_code_value.split(",") if token.strip()]
                if len(summary_code_items) != summary_count_num:
                    return fail(
                        f"summary failure-code pair mismatch {summary_count_key}: count={summary_count_num} "
                        f"codes={len(summary_code_items)}",
                        code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                    )
        for key, expected_value in SYNC_RUNTIME_HELPER_CONTRACT_FIELDS:
            value = str(sync_readiness_doc.get(key, "")).strip()
            if value != expected_value:
                return fail(
                    f"ci_sync_readiness contract mismatch {key}: expected={expected_value} actual={value}",
                    code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                )
            sanity_key = key.replace("ci_sync_readiness_", "", 1)
            if value != str(sanity_doc.get(sanity_key, "")).strip():
                return fail(
                    f"ci_sync_readiness/ci_sanity_gate contract mismatch key={key}",
                    code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
                )
            summary_value = str(summary_kv.get(key, "")).strip()
            if summary_report_exists and summary_status_known and result_status == "pass" and summary_value != value:
                return fail(
                    f"summary/ci_sync_readiness contract mismatch key={key} summary={summary_value} report={value}",
                    code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                )

    age4_proof_snapshot = load_age4_proof_snapshot(index_doc)
    if summary_report_exists and summary_status_known and result_status == "pass":
        for key in (
            AGE4_PROOF_OK_KEY,
            AGE4_PROOF_FAILED_CRITERIA_KEY,
            AGE4_PROOF_FAILED_PREVIEW_KEY,
            AGE4_PROOF_SUMMARY_HASH_KEY,
        ):
            summary_value = str(summary_kv.get(key, "")).strip()
            expected_value = str(age4_proof_snapshot.get(key, "")).strip()
            if not summary_value:
                return fail(f"summary missing {key}")
            if summary_value != expected_value:
                return fail(f"summary invalid {key}: {summary_value}")
    age5_policy_snapshot = load_age5_policy_snapshot(index_doc)
    brief_path = artifact_path(index_doc, "ci_fail_brief_txt")
    brief_required = bool(args.require_brief)
    if brief_path is None:
        if brief_required:
            return fail("index missing reports.ci_fail_brief_txt", code=CODES["INDEX_BRIEF_PATH_MISSING"])
    elif brief_required or brief_path.exists():
        brief_line = load_line(brief_path)
        if not brief_line:
            return fail(f"missing/empty brief file: {brief_path}", code=CODES["BRIEF_REQUIRED_MISSING"])
        brief_tokens = parse_tokens(brief_line)
        if brief_tokens is None:
            return fail("brief token format invalid")
        brief_status = str(brief_tokens.get("status", "")).strip()
        if brief_status and brief_status != result_status:
            return fail(f"brief status mismatch brief={brief_status} result={result_status}")
        brief_reason = str(brief_tokens.get("reason", "")).strip() or "-"
        if brief_reason != result_reason:
            return fail(f"brief reason mismatch brief={brief_reason} result={result_reason}")
        if "failed_steps_count" not in brief_tokens:
            return fail("brief missing failed_steps_count")
        try:
            brief_failed_steps_count = int(str(brief_tokens.get("failed_steps_count", "-1")))
        except Exception:
            return fail("brief failed_steps_count must be int")
        if brief_failed_steps_count < 0:
            return fail(f"brief failed_steps_count invalid: {brief_failed_steps_count}")
        if brief_failed_steps_count != result_failed_steps:
            return fail(
                f"brief failed_steps_count mismatch brief={brief_failed_steps_count} result={result_failed_steps}"
            )
        for key in ("failed_steps", "top_step", "top_message", "final_line"):
            if key not in brief_tokens:
                return fail(f"brief missing {key}")
        if summary_line_text:
            brief_final_line = str(brief_tokens.get("final_line", "")).strip()
            expected_brief_final_line_candidates = expected_final_line_candidates(
                index_doc,
                summary_line_text,
                220,
            )
            if not expected_brief_final_line_candidates:
                expected_brief_final_line_candidates = ["-"]
            if brief_final_line not in expected_brief_final_line_candidates:
                return fail(
                    "brief final_line mismatch "
                    f"brief={brief_final_line} expected_candidates={','.join(expected_brief_final_line_candidates)}"
                )
        profile_matrix_selftest_path = artifact_path(index_doc, "ci_profile_matrix_gate_selftest")
        if profile_matrix_selftest_path is not None and profile_matrix_selftest_path.exists():
            profile_matrix_snap = load_profile_matrix_selftest_snapshot(profile_matrix_selftest_path)
            if not isinstance(profile_matrix_snap, dict):
                return fail(f"invalid profile_matrix selftest json: {profile_matrix_selftest_path}")
            for key in PROFILE_MATRIX_BRIEF_REQUIRED_KEYS:
                if key not in brief_tokens:
                    return fail(f"brief missing {key}")
            for key in (
                "profile_matrix_total_elapsed_ms",
                "profile_matrix_core_lang_elapsed_ms",
                "profile_matrix_full_elapsed_ms",
                "profile_matrix_seamgrim_elapsed_ms",
            ):
                value = str(brief_tokens.get(key, "")).strip()
                if not validate_runtime5_elapsed_text(value):
                    return fail(f"brief {key} invalid: {value}")
            brief_compare_map = build_profile_matrix_brief_payload_from_snapshot(profile_matrix_snap)
            for key, expected in brief_compare_map.items():
                actual = str(brief_tokens.get(key, "")).strip()
                if actual != expected:
                    return fail(
                        f"brief/profile_matrix mismatch key={key} brief={actual} report={expected}"
                    )
        for key in (
            AGE4_PROOF_OK_KEY,
            AGE4_PROOF_FAILED_CRITERIA_KEY,
            AGE4_PROOF_FAILED_PREVIEW_KEY,
            AGE4_PROOF_SUMMARY_HASH_KEY,
        ):
            brief_value = str(brief_tokens.get(key, "")).strip()
            expected_value = str(age4_proof_snapshot.get(key, "")).strip()
            if not brief_value:
                return fail(f"brief missing {key}")
            if brief_value != expected_value:
                return fail(f"brief invalid {key}: {brief_value}")
        age5_child_summary_error = validate_age5_child_summary_tokens(brief_tokens, source="brief")
        if age5_child_summary_error is not None:
            return fail(age5_child_summary_error)
        age5_child_default_transport_error = validate_age5_child_summary_default_transport_tokens(
            brief_tokens, source="brief"
        )
        if age5_child_default_transport_error is not None:
            return fail(age5_child_default_transport_error)
        for key in AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS:
            summary_value = str(summary_kv.get(key, "")).strip()
            brief_value = str(brief_tokens.get(key, "")).strip()
            if summary_value != brief_value:
                return fail(
                    f"summary/brief age5 child summary mismatch key={key} summary={summary_value} brief={brief_value}"
                )
        for key, expected in AGE5_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS.items():
            summary_value = str(summary_kv.get(key, "")).strip() or expected
            brief_value = str(brief_tokens.get(key, "")).strip()
            if summary_value != brief_value:
                return fail(
                    f"summary/brief age5 child default transport mismatch key={key} "
                    f"summary={summary_value} brief={brief_value}"
                )
        brief_default_text = str(brief_tokens.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip()
        if not brief_default_text:
            return fail(f"brief missing {AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}")
        if brief_default_text != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
            return fail(
                f"brief invalid {AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}: {brief_default_text}"
            )
        brief_default_field = str(brief_tokens.get("combined_digest_selftest_default_field", "")).strip()
        if not brief_default_field:
            return fail("brief missing combined_digest_selftest_default_field")
        expected_brief_default_field = AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT.split("=", 1)[1]
        if brief_default_field != expected_brief_default_field:
            return fail(f"brief invalid combined_digest_selftest_default_field: {brief_default_field}")
        brief_policy_default_text = str(
            brief_tokens.get(AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")
        ).strip()
        if not brief_policy_default_text:
            return fail(f"brief missing {AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}")
        if brief_policy_default_text != str(
            age5_policy_snapshot[AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY]
        ).strip():
            return fail(
                f"brief invalid {AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}: {brief_policy_default_text}"
            )
        brief_policy_default_field = str(
            brief_tokens.get(AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY, "")
        ).strip()
        if not brief_policy_default_field:
            return fail(f"brief missing {AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY}")
        expected_brief_policy_default_field = json.dumps(
            dict(age5_policy_snapshot[AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY]),
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        )
        if brief_policy_default_field != expected_brief_policy_default_field:
            return fail(
                f"brief invalid {AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY}: {brief_policy_default_field}"
            )
        brief_policy_age4_snapshot_fields_text = str(
            brief_tokens.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY, "")
        ).strip()
        if not brief_policy_age4_snapshot_fields_text:
            return fail(f"brief missing {AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY}")
        if brief_policy_age4_snapshot_fields_text != str(
            age5_policy_snapshot[AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY]
        ).strip():
            return fail(
                f"brief invalid {AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY}: {brief_policy_age4_snapshot_fields_text}"
            )
        brief_policy_age4_snapshot_text = str(
            brief_tokens.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY, "")
        ).strip()
        if not brief_policy_age4_snapshot_text:
            return fail(f"brief missing {AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY}")
        if brief_policy_age4_snapshot_text != str(
            age5_policy_snapshot[AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY]
        ).strip():
            return fail(
                f"brief invalid {AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY}: {brief_policy_age4_snapshot_text}"
            )
        for key in (
            AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY,
            AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY,
            AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY,
            AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY,
            AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY,
        ):
            brief_value = str(brief_tokens.get(key, "")).strip()
            expected_value = str(age5_policy_snapshot[key]).strip()
            if not brief_value:
                return fail(f"brief missing {key}")
            if brief_value != expected_value:
                return fail(f"brief invalid {key}: {brief_value}")
        brief_policy_origin_trace_text = str(brief_tokens.get(AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY, "")).strip()
        if not brief_policy_origin_trace_text:
            return fail(f"brief missing {AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY}")
        expected_brief_policy_origin_trace_text = str(
            age5_policy_snapshot[AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY]
        ).strip()
        if brief_policy_origin_trace_text != expected_brief_policy_origin_trace_text:
            return fail(
                f"brief invalid {AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY}: {brief_policy_origin_trace_text}"
            )
        brief_policy_origin_trace = str(brief_tokens.get(AGE5_POLICY_ORIGIN_TRACE_KEY, "")).strip()
        if not brief_policy_origin_trace:
            return fail(f"brief missing {AGE5_POLICY_ORIGIN_TRACE_KEY}")
        expected_brief_policy_origin_trace = json.dumps(
            dict(age5_policy_snapshot[AGE5_POLICY_ORIGIN_TRACE_KEY]),
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        )
        if brief_policy_origin_trace != expected_brief_policy_origin_trace:
            return fail(
                f"brief invalid {AGE5_POLICY_ORIGIN_TRACE_KEY}: {brief_policy_origin_trace}"
            )
        for key in (
            AGE5_POLICY_REPORT_PATH_KEY,
            AGE5_POLICY_REPORT_EXISTS_KEY,
            AGE5_POLICY_TEXT_PATH_KEY,
            AGE5_POLICY_TEXT_EXISTS_KEY,
            AGE5_POLICY_SUMMARY_PATH_KEY,
            AGE5_POLICY_SUMMARY_EXISTS_KEY,
        AGE5_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY,
        AGE5_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY,
        AGE5_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY,
        AGE5_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY,
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY,
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY,
        ):
            brief_value = str(brief_tokens.get(key, "")).strip()
            expected_value = str(age5_policy_snapshot[key]).strip()
            if not brief_value:
                return fail(f"brief missing {key}")
            if brief_value != expected_value:
                return fail(f"brief invalid {key}: {brief_value}")
    elif brief_required:
        return fail(f"missing brief file: {brief_path}", code=CODES["BRIEF_REQUIRED_MISSING"])

    triage_path = artifact_path(index_doc, "ci_fail_triage_json")
    triage_doc: dict | None = None
    triage_required = bool(args.require_triage)
    if triage_path is None:
        if triage_required:
            return fail("index missing reports.ci_fail_triage_json", code=CODES["INDEX_TRIAGE_PATH_MISSING"])
    elif triage_required or triage_path.exists():
        triage_doc = load_json(triage_path)
        if not isinstance(triage_doc, dict):
            return fail(f"invalid triage json: {triage_path}", code=CODES["TRIAGE_REQUIRED_MISSING"])
        if not summary_report_exists:
            return fail(f"triage exists but summary report missing: {summary_path}")
        if str(triage_doc.get("schema", "")).strip() != TRIAGE_SCHEMA:
            return fail(f"triage schema mismatch: {triage_doc.get('schema')}")
        triage_status = str(triage_doc.get("status", "")).strip() or "unknown"
        if triage_status != result_status:
            return fail(f"triage status mismatch triage={triage_status} result={result_status}")
        triage_reason = str(triage_doc.get("reason", "-")).strip() or "-"
        if triage_reason != result_reason:
            return fail(f"triage reason mismatch triage={triage_reason} result={result_reason}")
        triage_final_line = str(triage_doc.get("final_line", "")).strip()
        if summary_line_text:
            expected_triage_final_line_candidates = expected_final_line_candidates(
                index_doc,
                summary_line_text,
                360,
            )
            if not expected_triage_final_line_candidates:
                expected_triage_final_line_candidates = ["-"]
            if triage_final_line not in expected_triage_final_line_candidates:
                return fail(
                    "triage final_line mismatch "
                    f"triage={triage_final_line} expected_candidates={','.join(expected_triage_final_line_candidates)}"
                )
        triage_prefix = str(triage_doc.get("report_prefix", "")).strip()
        if index_prefix and triage_prefix != index_prefix:
            return fail(f"triage report_prefix mismatch triage={triage_prefix} index={index_prefix}")
        age5_child_summary_error = validate_age5_child_summary_tokens(triage_doc, source="triage")
        if age5_child_summary_error is not None:
            return fail(age5_child_summary_error)
        age5_child_default_transport_error = validate_age5_child_summary_default_transport_tokens(
            triage_doc, source="triage"
        )
        if age5_child_default_transport_error is not None:
            return fail(age5_child_default_transport_error)
        for key in AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS:
            summary_value = str(summary_kv.get(key, "")).strip()
            triage_value = str(triage_doc.get(key, "")).strip()
            if summary_value != triage_value:
                return fail(
                    f"summary/triage age5 child summary mismatch key={key} summary={summary_value} triage={triage_value}"
                )
        for key, expected in AGE5_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS.items():
            summary_value = str(summary_kv.get(key, "")).strip() or expected
            triage_value = str(triage_doc.get(key, "")).strip()
            if summary_value != triage_value:
                return fail(
                    f"summary/triage age5 child default transport mismatch key={key} "
                    f"summary={summary_value} triage={triage_value}"
                )
        triage_default_text = str(triage_doc.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip()
        if not triage_default_text:
            return fail(f"triage missing {AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}")
        if triage_default_text != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
            return fail(
                f"triage invalid {AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}: {triage_default_text}"
            )
        triage_default_field = triage_doc.get("combined_digest_selftest_default_field")
        if not isinstance(triage_default_field, dict):
            return fail("triage combined_digest_selftest_default_field must be object")
        if dict(triage_default_field) != AGE5_DIGEST_SELFTEST_DEFAULT_FIELD:
            return fail(
                "triage invalid combined_digest_selftest_default_field: "
                f"{triage_default_field}"
            )
        triage_default_field_text = json.dumps(
            dict(triage_default_field), ensure_ascii=False, sort_keys=True, separators=(",", ":")
        )
        if brief_default_field != triage_default_field_text:
            return fail(
                "brief/triage combined_digest_selftest_default_field mismatch "
                f"brief={brief_default_field} triage={triage_default_field_text}"
            )
        triage_policy_default_text = str(
            triage_doc.get(AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")
        ).strip()
        if not triage_policy_default_text:
            return fail(f"triage missing {AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}")
        expected_triage_policy_default_text = str(
            age5_policy_snapshot[AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY]
        ).strip()
        if triage_policy_default_text != expected_triage_policy_default_text:
            return fail(
                f"triage invalid {AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}: {triage_policy_default_text}"
            )
        if brief_policy_default_text != triage_policy_default_text:
            return fail(
                f"brief/triage mismatch key={AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY} "
                f"brief={brief_policy_default_text} triage={triage_policy_default_text}"
            )
        triage_policy_default_field = triage_doc.get(AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY)
        if not isinstance(triage_policy_default_field, dict):
            return fail(f"triage {AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY} must be object")
        expected_triage_policy_default_field = dict(
            age5_policy_snapshot[AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY]
        )
        if dict(triage_policy_default_field) != expected_triage_policy_default_field:
            return fail(
                f"triage invalid {AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY}: "
                f"{triage_policy_default_field}"
            )
        triage_policy_default_field_text = json.dumps(
            dict(triage_policy_default_field), ensure_ascii=False, sort_keys=True, separators=(",", ":")
        )
        if brief_policy_default_field != triage_policy_default_field_text:
            return fail(
                f"brief/triage mismatch key={AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY} "
                f"brief={brief_policy_default_field} triage={triage_policy_default_field_text}"
            )
        triage_policy_age4_snapshot_fields_text = str(
            triage_doc.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY, "")
        ).strip()
        if not triage_policy_age4_snapshot_fields_text:
            return fail(f"triage missing {AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY}")
        if triage_policy_age4_snapshot_fields_text != str(
            age5_policy_snapshot[AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY]
        ).strip():
            return fail(
                f"triage invalid {AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY}: {triage_policy_age4_snapshot_fields_text}"
            )
        if brief_policy_age4_snapshot_fields_text != triage_policy_age4_snapshot_fields_text:
            return fail(
                f"brief/triage mismatch key={AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY} "
                f"brief={brief_policy_age4_snapshot_fields_text} triage={triage_policy_age4_snapshot_fields_text}"
            )
        triage_policy_age4_snapshot_text = str(
            triage_doc.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY, "")
        ).strip()
        if not triage_policy_age4_snapshot_text:
            return fail(f"triage missing {AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY}")
        if triage_policy_age4_snapshot_text != str(
            age5_policy_snapshot[AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY]
        ).strip():
            return fail(
                f"triage invalid {AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY}: {triage_policy_age4_snapshot_text}"
            )
        if brief_policy_age4_snapshot_text != triage_policy_age4_snapshot_text:
            return fail(
                f"brief/triage mismatch key={AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY} "
                f"brief={brief_policy_age4_snapshot_text} triage={triage_policy_age4_snapshot_text}"
            )
        for key in (
            AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY,
            AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY,
            AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY,
            AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY,
            AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY,
        ):
            triage_value = str(triage_doc.get(key, "")).strip()
            expected_value = str(age5_policy_snapshot[key]).strip()
            if not triage_value:
                return fail(f"triage missing {key}")
            if triage_value != expected_value:
                return fail(f"triage invalid {key}: {triage_value}")
            brief_value = str(brief_tokens.get(key, "")).strip()
            if brief_value != triage_value:
                return fail(
                    f"brief/triage mismatch key={key} brief={brief_value} triage={triage_value}"
                )
        triage_policy_origin_trace_text = str(
            triage_doc.get(AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY, "")
        ).strip()
        if not triage_policy_origin_trace_text:
            return fail(f"triage missing {AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY}")
        expected_triage_policy_origin_trace_text = str(
            age5_policy_snapshot[AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY]
        ).strip()
        if triage_policy_origin_trace_text != expected_triage_policy_origin_trace_text:
            return fail(
                f"triage invalid {AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY}: {triage_policy_origin_trace_text}"
            )
        if brief_policy_origin_trace_text != triage_policy_origin_trace_text:
            return fail(
                f"brief/triage mismatch key={AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY} "
                f"brief={brief_policy_origin_trace_text} triage={triage_policy_origin_trace_text}"
            )
        triage_policy_origin_trace = triage_doc.get(AGE5_POLICY_ORIGIN_TRACE_KEY)
        if not isinstance(triage_policy_origin_trace, dict):
            return fail(f"triage {AGE5_POLICY_ORIGIN_TRACE_KEY} must be object")
        expected_triage_policy_origin_trace = dict(
            age5_policy_snapshot[AGE5_POLICY_ORIGIN_TRACE_KEY]
        )
        if dict(triage_policy_origin_trace) != expected_triage_policy_origin_trace:
            return fail(
                f"triage invalid {AGE5_POLICY_ORIGIN_TRACE_KEY}: {triage_policy_origin_trace}"
            )
        triage_policy_origin_trace_text_json = json.dumps(
            dict(triage_policy_origin_trace),
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        )
        if brief_policy_origin_trace != triage_policy_origin_trace_text_json:
            return fail(
                f"brief/triage mismatch key={AGE5_POLICY_ORIGIN_TRACE_KEY} "
                f"brief={brief_policy_origin_trace} triage={triage_policy_origin_trace_text_json}"
            )
        for key in (
            AGE5_POLICY_REPORT_PATH_KEY,
            AGE5_POLICY_REPORT_EXISTS_KEY,
            AGE5_POLICY_TEXT_PATH_KEY,
            AGE5_POLICY_TEXT_EXISTS_KEY,
            AGE5_POLICY_SUMMARY_PATH_KEY,
            AGE5_POLICY_SUMMARY_EXISTS_KEY,
        AGE5_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY,
        AGE5_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY,
        AGE5_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY,
        AGE5_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY,
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY,
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY,
        ):
            triage_value = str(triage_doc.get(key, "")).strip()
            expected_value = str(age5_policy_snapshot[key]).strip()
            if not triage_value:
                return fail(f"triage missing {key}")
            if triage_value != expected_value:
                return fail(f"triage invalid {key}: {triage_value}")
            brief_value = str(brief_tokens.get(key, "")).strip()
            if brief_value != triage_value:
                return fail(
                    f"brief/triage mismatch key={key} brief={brief_value} triage={triage_value}"
                )
        if not isinstance(triage_doc.get("summary_verify_ok"), bool):
            return fail("triage summary_verify_ok must be bool")
        summary_verify_ok = bool(triage_doc.get("summary_verify_ok"))
        summary_verify_issues = triage_doc.get("summary_verify_issues")
        if not isinstance(summary_verify_issues, list):
            return fail("triage summary_verify_issues must be list")
        summary_verify_top_issue = str(triage_doc.get("summary_verify_top_issue", "")).strip()
        if not summary_verify_top_issue:
            return fail("triage summary_verify_top_issue missing")
        try:
            summary_verify_issues_count = int(triage_doc.get("summary_verify_issues_count", -1))
        except Exception:
            return fail("triage summary_verify_issues_count must be int")
        if summary_verify_issues_count != len(summary_verify_issues):
            return fail(
                f"triage summary_verify_issues_count mismatch triage={summary_verify_issues_count} "
                f"actual={len(summary_verify_issues)}"
            )
        parsed_summary_verify_issues: list[str] = []
        for idx, item in enumerate(summary_verify_issues):
            code = str(item).strip()
            if not code:
                return fail(f"triage summary_verify_issues[{idx}] empty")
            if code not in SUMMARY_VERIFY_CODES_SET:
                return fail(f"triage summary_verify_issues[{idx}] invalid code: {code}")
            parsed_summary_verify_issues.append(code)
        if summary_verify_ok and parsed_summary_verify_issues:
            return fail("triage summary_verify_ok=1 requires empty summary_verify_issues")
        if summary_verify_ok and summary_verify_top_issue != "-":
            return fail(f"triage summary_verify_ok=1 requires summary_verify_top_issue='-', got={summary_verify_top_issue}")
        if (not summary_verify_ok) and not parsed_summary_verify_issues:
            return fail("triage summary_verify_ok=0 requires non-empty summary_verify_issues")
        if (not summary_verify_ok) and summary_verify_top_issue not in parsed_summary_verify_issues:
            return fail(
                f"triage summary_verify_top_issue must be one of summary_verify_issues "
                f"top={summary_verify_top_issue}"
            )
        if summary_verify_top_issue != "-" and summary_verify_top_issue not in SUMMARY_VERIFY_CODES_SET:
            return fail(f"triage summary_verify_top_issue invalid code: {summary_verify_top_issue}")

        summary_hint = str(triage_doc.get("summary_report_path_hint", "")).strip()
        summary_norm = str(triage_doc.get("summary_report_path_hint_norm", "")).strip()
        expected_summary = str(summary_path)
        expected_summary_norm = normalize_path_text(expected_summary)
        if expected_summary and summary_hint != expected_summary:
            return fail(f"triage summary_report_path_hint mismatch triage={summary_hint} index={expected_summary}")
        if expected_summary_norm and summary_norm != expected_summary_norm:
            return fail(
                f"triage summary_report_path_hint_norm mismatch triage={summary_norm} index={expected_summary_norm}"
            )
        profile_matrix_selftest_path = artifact_path(index_doc, "ci_profile_matrix_gate_selftest")
        if profile_matrix_selftest_path is not None and profile_matrix_selftest_path.exists():
            profile_matrix_snap = load_profile_matrix_selftest_snapshot(profile_matrix_selftest_path)
            if not isinstance(profile_matrix_snap, dict):
                return fail(f"invalid profile_matrix selftest json: {profile_matrix_selftest_path}")
            triage_profile_matrix = triage_doc.get("profile_matrix_selftest")
            if not isinstance(triage_profile_matrix, dict):
                return fail("triage profile_matrix_selftest missing")
            triage_compare_map = build_profile_matrix_triage_payload_from_snapshot(profile_matrix_snap)
            for key, expected in triage_compare_map.items():
                actual = triage_profile_matrix.get(key)
                if actual != expected:
                    return fail(
                        f"triage/profile_matrix mismatch key={key} triage={actual} report={expected}"
                    )
        for key in (
            AGE4_PROOF_OK_KEY,
            AGE4_PROOF_FAILED_CRITERIA_KEY,
            AGE4_PROOF_FAILED_PREVIEW_KEY,
            AGE4_PROOF_SUMMARY_HASH_KEY,
        ):
            triage_value = str(triage_doc.get(key, "")).strip()
            expected_value = str(age4_proof_snapshot.get(key, "")).strip()
            if not triage_value:
                return fail(f"triage missing {key}")
            if triage_value != expected_value:
                return fail(f"triage invalid {key}: {triage_value}")
            brief_value = str(brief_tokens.get(key, "")).strip()
            if brief_value != triage_value:
                return fail(
                    f"brief/triage mismatch key={key} brief={brief_value} triage={triage_value}"
                )

        failed_steps = triage_doc.get("failed_steps")
        try:
            failed_steps_count = int(triage_doc.get("failed_steps_count", -1))
        except Exception:
            return fail("triage failed_steps_count must be int")
        if not isinstance(failed_steps, list):
            return fail("triage failed_steps must be list")
        if failed_steps_count != len(failed_steps):
            return fail(f"triage failed_steps_count mismatch triage={failed_steps_count} actual={len(failed_steps)}")
        if result_status == "pass":
            if failed_steps_count != 0:
                return fail(f"pass triage failed_steps_count must be 0, got {failed_steps_count}")
            if failed_steps:
                return fail("pass triage must have empty failed_steps")
        if result_status == "fail" and failed_steps_count <= 0:
            return fail("fail triage failed_steps_count must be >0")
        if brief_path is not None and brief_path.exists():
            brief_line = load_line(brief_path)
            brief_tokens = parse_tokens(brief_line) if brief_line else None
            if isinstance(brief_tokens, dict):
                try:
                    brief_failed_steps_count = int(str(brief_tokens.get("failed_steps_count", "-1")))
                except Exception:
                    return fail("brief failed_steps_count parse failed")
                if brief_failed_steps_count != failed_steps_count:
                    return fail(
                        f"brief/triage failed_steps_count mismatch brief={brief_failed_steps_count} triage={failed_steps_count}"
                    )
                expected_digest_selftest = resolve_age5_digest_selftest_expected(summary_kv, index_doc)
                brief_digest_selftest = str(brief_tokens.get(AGE5_DIGEST_SELFTEST_SUMMARY_KEY, "")).strip()
                if brief_digest_selftest not in {"0", "1"}:
                    return fail(
                        f"brief invalid {AGE5_DIGEST_SELFTEST_SUMMARY_KEY}: {brief_digest_selftest}"
                    )
                triage_digest_selftest = str(triage_doc.get(AGE5_DIGEST_SELFTEST_SUMMARY_KEY, "")).strip()
                if triage_digest_selftest not in {"0", "1"}:
                    return fail(
                        f"triage invalid {AGE5_DIGEST_SELFTEST_SUMMARY_KEY}: {triage_digest_selftest}"
                    )
                if brief_digest_selftest != expected_digest_selftest:
                    return fail(
                        "brief/index age5 digest selftest mismatch "
                        f"brief={brief_digest_selftest} expected={expected_digest_selftest}"
                    )
                if triage_digest_selftest != expected_digest_selftest:
                    return fail(
                        "triage/index age5 digest selftest mismatch "
                        f"triage={triage_digest_selftest} expected={expected_digest_selftest}"
                    )
                if brief_digest_selftest != triage_digest_selftest:
                    return fail(
                        "brief/triage age5 digest selftest mismatch "
                        f"brief={brief_digest_selftest} triage={triage_digest_selftest}"
                    )
                brief_default_text = str(
                    brief_tokens.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")
                ).strip()
                if not brief_default_text:
                    return fail(
                        f"brief missing {AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}"
                    )
                brief_default_field = str(
                    brief_tokens.get("combined_digest_selftest_default_field", "")
                ).strip()
                if not brief_default_field:
                    return fail("brief missing combined_digest_selftest_default_field")
                triage_default_text = str(
                    triage_doc.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")
                ).strip()
                if not triage_default_text:
                    return fail(
                        f"triage missing {AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}"
                    )
                if brief_default_text != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
                    return fail(
                        "brief invalid combined digest selftest default text "
                        f"brief={brief_default_text}"
                    )
                expected_brief_default_field = AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT.split("=", 1)[1]
                if brief_default_field != expected_brief_default_field:
                    return fail(
                        "brief invalid combined digest selftest default field "
                        f"brief={brief_default_field}"
                    )
                if triage_default_text != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
                    return fail(
                        "triage invalid combined digest selftest default text "
                        f"triage={triage_default_text}"
                    )
                if brief_default_text != triage_default_text:
                    return fail(
                        "brief/triage combined digest selftest default text mismatch "
                        f"brief={brief_default_text} triage={triage_default_text}"
                    )
                triage_default_field = triage_doc.get("combined_digest_selftest_default_field")
                if not isinstance(triage_default_field, dict):
                    return fail("triage combined_digest_selftest_default_field must be object")
                if dict(triage_default_field) != AGE5_DIGEST_SELFTEST_DEFAULT_FIELD:
                    return fail(
                        "triage invalid combined_digest_selftest_default_field "
                        f"triage={triage_default_field}"
                    )
                triage_default_field_text = json.dumps(
                    dict(triage_default_field), ensure_ascii=False, sort_keys=True, separators=(",", ":")
                )
                if brief_default_field != triage_default_field_text:
                    return fail(
                        "brief/triage combined digest selftest default field mismatch "
                        f"brief={brief_default_field} triage={triage_default_field_text}"
                    )
                for key in AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS:
                    brief_value = str(brief_tokens.get(key, "")).strip()
                    triage_value = str(triage_doc.get(key, "")).strip()
                    if brief_value != triage_value:
                        return fail(
                            f"brief/triage age5 child summary mismatch key={key} brief={brief_value} triage={triage_value}"
                        )
        triage_step_ids: list[str] = []
        triage_step_logs: dict[str, dict[str, str]] = {}
        for idx, row in enumerate(failed_steps):
            if not isinstance(row, dict):
                return fail(f"triage failed_steps[{idx}] must be object")
            step_id = str(row.get("step_id", "")).strip() or str(row.get("name", "")).strip()
            if not step_id:
                return fail(f"triage failed_steps[{idx}].step_id missing")
            if step_id in triage_step_logs:
                return fail(f"triage failed_steps duplicate step_id: {step_id}")
            triage_step_ids.append(step_id)
            step_cmd = str(row.get("cmd", "")).strip()
            if result_status == "fail" and not step_cmd:
                return fail(f"triage failed_steps[{idx}].cmd missing")
            stdout_path = str(row.get("stdout_log_path", "")).strip()
            stderr_path = str(row.get("stderr_log_path", "")).strip()
            stdout_norm = str(row.get("stdout_log_path_norm", "")).strip()
            stderr_norm = str(row.get("stderr_log_path_norm", "")).strip()
            fast_fail_step_detail = str(row.get("fast_fail_step_detail", "")).strip()
            if result_status == "fail":
                if not fast_fail_step_detail:
                    return fail(f"triage failed_steps[{idx}].fast_fail_step_detail missing")
                if f"name={step_id}" not in fast_fail_step_detail:
                    return fail(
                        f"triage failed_steps[{idx}].fast_fail_step_detail step mismatch "
                        f"detail={fast_fail_step_detail}"
                    )
            fast_fail_step_logs = str(row.get("fast_fail_step_logs", "")).strip()
            if result_status == "fail":
                if not fast_fail_step_logs:
                    return fail(f"triage failed_steps[{idx}].fast_fail_step_logs missing")
                if f"name={step_id}" not in fast_fail_step_logs:
                    return fail(
                        f"triage failed_steps[{idx}].fast_fail_step_logs step mismatch "
                        f"logs={fast_fail_step_logs}"
                    )
            triage_step_logs[step_id] = {"stdout": stdout_path, "stderr": stderr_path}
            if stdout_path and stdout_norm != normalize_path_text(stdout_path):
                return fail(
                    f"triage failed_steps[{idx}].stdout_log_path_norm mismatch "
                    f"triage={stdout_norm} expected={normalize_path_text(stdout_path)}"
                )
            if stderr_path and stderr_norm != normalize_path_text(stderr_path):
                return fail(
                    f"triage failed_steps[{idx}].stderr_log_path_norm mismatch "
                    f"triage={stderr_norm} expected={normalize_path_text(stderr_path)}"
                )
            for label, path_text in (("stdout", stdout_path), ("stderr", stderr_path)):
                if not path_text:
                    continue
                resolved = resolve_path(path_text)
                if resolved is None:
                    return fail(f"triage failed_steps[{idx}] {label} path resolve failed")
                if not resolved.exists():
                    return fail(f"triage failed_steps[{idx}] {label} path missing: {resolved}")
        if result_status == "fail" and summary_status == "fail":
            try:
                triage_detail_rows_count = int(triage_doc.get("failed_step_detail_rows_count", -1))
            except Exception:
                return fail(
                    "triage failed_step_detail_rows_count must be int",
                    code=CODES["TRIAGE_REQUIRED_MISSING"],
                )
            if triage_detail_rows_count != len(summary_failed_step_details):
                return fail(
                    "triage/summary failed_step_detail_rows_count mismatch "
                    f"triage={triage_detail_rows_count} summary={len(summary_failed_step_details)}",
                    code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                )
            try:
                triage_logs_rows_count = int(triage_doc.get("failed_step_logs_rows_count", -1))
            except Exception:
                return fail(
                    "triage failed_step_logs_rows_count must be int",
                    code=CODES["TRIAGE_REQUIRED_MISSING"],
                )
            if triage_logs_rows_count != len(summary_failed_step_logs):
                return fail(
                    "triage/summary failed_step_logs_rows_count mismatch "
                    f"triage={triage_logs_rows_count} summary={len(summary_failed_step_logs)}",
                    code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                )
            triage_detail_order_raw = triage_doc.get("failed_step_detail_order")
            if not isinstance(triage_detail_order_raw, list):
                return fail(
                    "triage failed_step_detail_order must be list",
                    code=CODES["TRIAGE_REQUIRED_MISSING"],
                )
            triage_detail_order = [str(step).strip() for step in triage_detail_order_raw if str(step).strip()]
            if triage_detail_order != summary_failed_step_detail_order:
                return fail(
                    "triage/summary failed_step_detail_order mismatch "
                    f"triage={','.join(triage_detail_order) or '-'} "
                    f"summary={','.join(summary_failed_step_detail_order) or '-'}",
                    code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                )
            triage_logs_order_raw = triage_doc.get("failed_step_logs_order")
            if not isinstance(triage_logs_order_raw, list):
                return fail(
                    "triage failed_step_logs_order must be list",
                    code=CODES["TRIAGE_REQUIRED_MISSING"],
                )
            triage_logs_order = [str(step).strip() for step in triage_logs_order_raw if str(step).strip()]
            if triage_logs_order != summary_failed_step_logs_order:
                return fail(
                    "triage/summary failed_step_logs_order mismatch "
                    f"triage={','.join(triage_logs_order) or '-'} "
                    f"summary={','.join(summary_failed_step_logs_order) or '-'}",
                    code=CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
                )
            summary_step_set = set(str(step).strip() for step in summary_failed_steps if str(step).strip())
            triage_step_set = set(triage_step_ids)
            if not triage_step_set:
                return fail("fail triage missing failed step ids")
            if triage_step_set != summary_step_set:
                return fail(
                    f"triage/summary failed_steps mismatch triage={','.join(sorted(triage_step_set))} "
                    f"summary={','.join(sorted(summary_step_set))}"
                )
            for step_id in triage_step_ids:
                if step_id not in summary_failed_step_details:
                    return fail(f"summary missing failed_step_detail for step={step_id}")
                summary_detail_text = str(summary_failed_step_details.get(step_id, "")).strip()
                summary_detail_match = SUMMARY_FAILED_STEP_DETAIL_RE.match(summary_detail_text)
                if summary_detail_match is None:
                    return fail(f"summary invalid failed_step_detail format for step={step_id}")
                summary_cmd = str(summary_detail_match.group(3)).strip()
                triage_step_row = next(
                    (item for item in failed_steps if isinstance(item, dict) and (str(item.get("step_id", "")).strip() or str(item.get("name", "")).strip()) == step_id),
                    None,
                )
                triage_step_cmd = str((triage_step_row or {}).get("cmd", "")).strip()
                if not triage_step_cmd:
                    return fail(f"triage missing cmd for step={step_id}")
                if triage_step_cmd != summary_cmd:
                    return fail(
                        f"triage/summary cmd mismatch step={step_id} "
                        f"triage={triage_step_cmd} summary={summary_cmd}"
                    )
                summary_logs_row = summary_failed_step_logs.get(step_id)
                if not isinstance(summary_logs_row, dict):
                    return fail(f"summary missing failed_step_logs for step={step_id}")
                triage_logs_row = triage_step_logs.get(step_id, {})
                for label in ("stdout", "stderr"):
                    triage_log = str(triage_logs_row.get(label, "")).strip()
                    summary_log = str(summary_logs_row.get(label, "")).strip()
                    if triage_log and not summary_log:
                        return fail(f"summary missing {label} log for step={step_id}")
                    if triage_log and summary_log and triage_log != summary_log:
                        return fail(
                            f"triage/summary {label} log mismatch step={step_id} triage={triage_log} summary={summary_log}"
                        )

        artifacts = triage_doc.get("artifacts")
        if not isinstance(artifacts, dict):
            return fail("triage artifacts missing")
        for key in ("summary", "summary_line", "ci_gate_result_json", "ci_fail_brief_txt", "ci_fail_triage_json"):
            row = artifacts.get(key)
            if not isinstance(row, dict):
                return fail(f"triage artifacts missing key={key}")
        for key, row in artifacts.items():
            if not isinstance(row, dict):
                return fail(f"triage artifacts.{key} must be object")
            issue = validate_triage_artifact_row(
                str(key),
                row,
                allow_exists_upgrade=bool(args.allow_triage_exists_upgrade),
            )
            if issue:
                return fail(issue)
            expected_path = artifact_path_text(index_doc, str(key))
            if expected_path:
                row_path = str(row.get("path", "")).strip()
                if row_path != expected_path:
                    return fail(
                        f"triage artifacts.{key}.path mismatch triage={row_path} index={expected_path}"
                    )
                expected_norm = normalize_path_text(expected_path)
                row_norm = str(row.get("path_norm", "")).strip()
                if row_norm != expected_norm:
                    return fail(
                        f"triage artifacts.{key}.path_norm mismatch triage={row_norm} index={expected_norm}"
                    )
    elif triage_required:
        return fail(f"missing triage json: {triage_path}", code=CODES["TRIAGE_REQUIRED_MISSING"])

    print(
        f"[ci-emit-artifacts-check] ok index={index_path} status={result_status} "
        f"require_brief={int(bool(args.require_brief))} require_triage={int(bool(args.require_triage))}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
