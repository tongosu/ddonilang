#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

from _ci_age3_completion_gate_contract import (
    AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS,
    AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS,
)
from _ci_age5_combined_heavy_contract import (
    AGE5_COMBINED_HEAVY_SANITY_CONTRACT_SUMMARY_FIELDS,
    build_age5_combined_heavy_sync_contract_fields,
)
from _ci_seamgrim_step_contract import (
    SEAMGRIM_BLOCKER_STEP_CONTRACT_STEPS,
    SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS,
    SEAMGRIM_PLATFORM_SANITY_SUMMARY_STEP_FIELDS,
    SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS,
    merge_step_names,
)

SYNC_READINESS_OK = "OK"
SYNC_READINESS_STEP_FAIL = "E_SYNC_READINESS_STEP_FAIL"
SYNC_READINESS_SANITY_CONTRACT_FAIL = "E_SYNC_READINESS_SANITY_CONTRACT_FAIL"
SYNC_READINESS_VALIDATE_ONLY_PATH_MISSING = "E_SYNC_READINESS_VALIDATE_ONLY_PATH_MISSING"

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
    "fixed64_darwin_real_report_live_check_selftest",
    "fixed64_darwin_real_report_readiness_check_selftest",
    "fixed64_threeway_inputs_selftest",
    "fixed64_cross_platform_threeway_gate_selftest",
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
    "seamgrim_ci_gate_seed_meta_step_check",
    "seamgrim_ci_gate_worker_env_step_check",
    "seamgrim_ci_gate_featured_seed_catalog_step_check",
    "seamgrim_ci_gate_featured_seed_catalog_autogen_step_check",
    "seamgrim_ci_gate_sam_seulgi_family_step_check",
    "seamgrim_ci_gate_runtime5_passthrough_check",
    "seamgrim_ci_gate_lesson_warning_step_check",
    "seamgrim_ci_gate_stateful_preview_step_check",
    *SEAMGRIM_BLOCKER_STEP_CONTRACT_STEPS,
    "seamgrim_v2_task_batch_check",
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
    "fixed64_darwin_real_report_live_check_selftest",
    "fixed64_darwin_real_report_readiness_check_selftest",
    "fixed64_threeway_inputs_selftest",
    "fixed64_cross_platform_threeway_gate_selftest",
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
    "fixed64_darwin_real_report_live_check_selftest",
    "fixed64_darwin_real_report_readiness_check_selftest",
    "fixed64_threeway_inputs_selftest",
    "fixed64_cross_platform_threeway_gate_selftest",
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
    "seamgrim_ci_gate_seed_meta_step_check",
    "seamgrim_ci_gate_worker_env_step_check",
    "seamgrim_ci_gate_featured_seed_catalog_step_check",
    "seamgrim_ci_gate_featured_seed_catalog_autogen_step_check",
    "seamgrim_ci_gate_sam_seulgi_family_step_check",
    "seamgrim_ci_gate_runtime5_passthrough_check",
    "seamgrim_ci_gate_lesson_warning_step_check",
    "seamgrim_ci_gate_stateful_preview_step_check",
    *SEAMGRIM_BLOCKER_STEP_CONTRACT_STEPS,
    "seamgrim_v2_task_batch_check",
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

SANITY_REQUIRED_PASS_STEPS = merge_step_names(
    SANITY_REQUIRED_PASS_STEPS,
    SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS,
)
SANITY_REQUIRED_PASS_STEPS_SEAMGRIM = merge_step_names(
    SANITY_REQUIRED_PASS_STEPS_SEAMGRIM,
    SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS,
)

VALID_SANITY_PROFILES = ("full", "core_lang", "seamgrim")
PACK_GOLDEN_GRAPH_EXPORT_SUMMARY_KEY = "ci_sanity_pack_golden_graph_export_ok"
PACK_GOLDEN_GRAPH_EXPORT_SYNC_KEY = "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok"
PACK_GOLDEN_GRAPH_EXPORT_REQUIRED_PROFILES = {"full", "core_lang"}
SANITY_SUMMARY_STEP_FIELDS = (
    ("ci_sanity_pipeline_emit_flags_ok", "pipeline_emit_flags_check", {"full", "core_lang"}),
    ("ci_sanity_pipeline_emit_flags_selftest_ok", "pipeline_emit_flags_selftest", {"full", "core_lang"}),
    (
        "ci_sanity_emit_artifacts_sanity_contract_selftest_ok",
        "ci_emit_artifacts_sanity_contract_selftest",
        {"full", "core_lang", "seamgrim"},
    ),
    ("ci_sanity_age2_completion_gate_ok", "age2_completion_gate", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age2_completion_gate_selftest_ok", "age2_completion_gate_selftest", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age2_close_ok", "age2_close", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age2_close_selftest_ok", "age2_close_selftest", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_completion_gate_ok", "age3_completion_gate", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_completion_gate_selftest_ok", "age3_completion_gate_selftest", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_close_ok", "age3_close", {"full", "seamgrim"}),
    ("ci_sanity_age3_close_selftest_ok", "age3_close_selftest", {"full", "core_lang", "seamgrim"}),
    (
        "ci_sanity_age5_combined_heavy_policy_selftest_ok",
        "age5_combined_heavy_policy_selftest",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok",
        "profile_matrix_full_real_smoke_policy_selftest",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_dynamic_source_profile_split_selftest_ok",
        "ci_sanity_dynamic_source_profile_split_selftest",
        {"full", "core_lang", "seamgrim"},
    ),
    *[
        (summary_key, step_name, {"seamgrim"})
        for summary_key, step_name in SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS
    ],
    *[
        (summary_key, step_name, {"seamgrim"})
        for summary_key, step_name in SEAMGRIM_PLATFORM_SANITY_SUMMARY_STEP_FIELDS
    ],
    (
        "ci_sanity_fixed64_darwin_real_report_live_check_selftest_ok",
        "fixed64_darwin_real_report_live_check_selftest",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_fixed64_threeway_inputs_selftest_ok",
        "fixed64_threeway_inputs_selftest",
        {"full", "core_lang", "seamgrim"},
    ),
)
CLOSE_SUMMARY_SYNC_FIELD_PAIRS = (
    (
        "ci_sanity_emit_artifacts_sanity_contract_selftest_ok",
        "ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok",
    ),
    ("ci_sanity_age2_close_ok", "ci_sync_readiness_ci_sanity_age2_close_ok"),
    ("ci_sanity_age2_close_selftest_ok", "ci_sync_readiness_ci_sanity_age2_close_selftest_ok"),
    ("ci_sanity_age3_close_ok", "ci_sync_readiness_ci_sanity_age3_close_ok"),
    ("ci_sanity_age3_close_selftest_ok", "ci_sync_readiness_ci_sanity_age3_close_selftest_ok"),
)
AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA = "ddn.bogae_geoul_visibility_smoke.v1"
AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_ENABLED_PROFILES = {"full", "core_lang", "seamgrim"}
AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_BOOL_FIELDS = (
    "ci_sanity_age3_bogae_geoul_visibility_smoke_ok",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes",
)
AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_TEXT_FIELDS = (
    "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_schema",
)
AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SYNC_FIELD_PAIRS = (
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
)
SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA = "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1"
SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES = 20
SEAMGRIM_WASM_WEB_STEP_CHECK_ENABLED_PROFILES = {"seamgrim"}
SEAMGRIM_WASM_WEB_STEP_CHECK_BOOL_FIELDS = (
    "ci_sanity_seamgrim_wasm_web_step_check_ok",
    "ci_sanity_seamgrim_wasm_web_step_check_report_exists",
)
SEAMGRIM_WASM_WEB_STEP_CHECK_TEXT_FIELDS = (
    "ci_sanity_seamgrim_wasm_web_step_check_report_path",
    "ci_sanity_seamgrim_wasm_web_step_check_schema",
    "ci_sanity_seamgrim_wasm_web_step_check_checked_files",
    "ci_sanity_seamgrim_wasm_web_step_check_missing_count",
)
SEAMGRIM_WASM_WEB_STEP_CHECK_SYNC_FIELD_PAIRS = (
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
)
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA = "ddn.pack_evidence_tier_runner_check.v1"
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_ENABLED_PROFILES = {"seamgrim"}
SEAMGRIM_PACK_EVIDENCE_TIER_MAX_DOCS_ISSUES = 10
SEAMGRIM_PACK_EVIDENCE_TIER_EXPECTED_REPO_ISSUES = 0
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_BOOL_FIELDS = (
    "ci_sanity_seamgrim_pack_evidence_tier_runner_ok",
    "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists",
)
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_TEXT_FIELDS = (
    "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path",
    "ci_sanity_seamgrim_pack_evidence_tier_runner_schema",
    "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count",
    "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count",
)
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SYNC_FIELD_PAIRS = (
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
)
SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA = "ddn.numeric_factor_route_diag_contract.v1"
SEAMGRIM_NUMERIC_FACTOR_POLICY_ENABLED_PROFILES = {"full", "seamgrim"}
SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS: dict[str, int] = {
    "bit_limit": 512,
    "pollard_iters": 200000,
    "pollard_c_seeds": 64,
    "pollard_x0_seeds": 6,
    "fallback_limit": 1000000,
    "small_prime_max": 101,
}
SEAMGRIM_NUMERIC_FACTOR_POLICY_KEYS = (
    "bit_limit",
    "pollard_iters",
    "pollard_c_seeds",
    "pollard_x0_seeds",
    "fallback_limit",
    "small_prime_max",
)
SEAMGRIM_NUMERIC_FACTOR_POLICY_BOOL_FIELDS = (
    "ci_sanity_seamgrim_numeric_factor_policy_ok",
    "ci_sanity_seamgrim_numeric_factor_policy_report_exists",
)
SEAMGRIM_NUMERIC_FACTOR_POLICY_TEXT_FIELDS = (
    "ci_sanity_seamgrim_numeric_factor_policy_report_path",
    "ci_sanity_seamgrim_numeric_factor_policy_schema",
    "ci_sanity_seamgrim_numeric_factor_policy_text",
    "ci_sanity_seamgrim_numeric_factor_policy_bit_limit",
    "ci_sanity_seamgrim_numeric_factor_policy_pollard_iters",
    "ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds",
    "ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds",
    "ci_sanity_seamgrim_numeric_factor_policy_fallback_limit",
    "ci_sanity_seamgrim_numeric_factor_policy_small_prime_max",
)
SEAMGRIM_NUMERIC_FACTOR_POLICY_SYNC_FIELD_PAIRS = (
    (
        "ci_sanity_seamgrim_numeric_factor_policy_ok",
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok",
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_report_path",
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_report_path",
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_report_exists",
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_report_exists",
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_schema",
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_schema",
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_text",
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_text",
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_bit_limit",
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_bit_limit",
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_pollard_iters",
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_iters",
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds",
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds",
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds",
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds",
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_fallback_limit",
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_fallback_limit",
    ),
    (
        "ci_sanity_seamgrim_numeric_factor_policy_small_prime_max",
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_small_prime_max",
    ),
)
FIXED64_DARWIN_REAL_REPORT_LIVE_SUMMARY_ENABLED_PROFILES = {"full", "core_lang", "seamgrim"}
FIXED64_DARWIN_REAL_REPORT_LIVE_BOOL_FIELDS = (
    "ci_sanity_fixed64_darwin_real_report_live_report_exists",
    "ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip",
)
FIXED64_DARWIN_REAL_REPORT_LIVE_TEXT_FIELDS = (
    "ci_sanity_fixed64_darwin_real_report_live_report_path",
    "ci_sanity_fixed64_darwin_real_report_live_status",
    "ci_sanity_fixed64_darwin_real_report_live_resolved_status",
    "ci_sanity_fixed64_darwin_real_report_live_resolved_source",
    "ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count",
)
FIXED64_DARWIN_REAL_REPORT_LIVE_SYNC_FIELD_PAIRS = (
    (
        "ci_sanity_fixed64_darwin_real_report_live_report_path",
        "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_report_path",
    ),
    (
        "ci_sanity_fixed64_darwin_real_report_live_report_exists",
        "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_report_exists",
    ),
    (
        "ci_sanity_fixed64_darwin_real_report_live_status",
        "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_status",
    ),
    (
        "ci_sanity_fixed64_darwin_real_report_live_resolved_status",
        "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_status",
    ),
    (
        "ci_sanity_fixed64_darwin_real_report_live_resolved_source",
        "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_source",
    ),
    (
        "ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count",
        "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count",
    ),
    (
        "ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip",
        "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip",
    ),
)
COMPLETION_FAILURE_CODE_ENABLED_PROFILES = {"full", "core_lang", "seamgrim"}
AGE3_COMPLETION_GATE_CRITERIA_ENABLED_PROFILES = {"full", "core_lang", "seamgrim"}
COMPLETION_FAILURE_CODE_SUMMARY_GROUPS = (
    (
        "age2_completion_gate",
        "ci_sanity_age2_completion_gate_failure_codes",
        "ci_sanity_age2_completion_gate_failure_code_count",
        COMPLETION_FAILURE_CODE_ENABLED_PROFILES,
    ),
    (
        "age3_completion_gate",
        "ci_sanity_age3_completion_gate_failure_codes",
        "ci_sanity_age3_completion_gate_failure_code_count",
        COMPLETION_FAILURE_CODE_ENABLED_PROFILES,
    ),
)
FAILURE_CODE_PATTERN = re.compile(r"[EW]_[A-Z0-9_]+")
VALID_SANITY_SUMMARY_VALUES = {"1", "0", "na", "pending"}
SANITY_CONTRACT_SUMMARY_FIELDS = AGE5_COMBINED_HEAVY_SANITY_CONTRACT_SUMMARY_FIELDS
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


def resolve_required_sanity_steps(profile: str) -> tuple[str, ...]:
    if profile == "core_lang":
        return SANITY_REQUIRED_PASS_STEPS_CORE_LANG
    if profile == "seamgrim":
        return SANITY_REQUIRED_PASS_STEPS_SEAMGRIM
    return SANITY_REQUIRED_PASS_STEPS


def clip(text: str, limit: int = 180) -> str:
    normalized = " ".join(str(text).split())
    if not normalized:
        return "-"
    if len(normalized) <= limit:
        return normalized
    return normalized[: limit - 3] + "..."


def run_step(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8-sig"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def sync_mirror_key(sanity_key: str) -> str:
    return f"ci_sync_readiness_{sanity_key}"


def build_default_sanity_summary_fields(profile: str) -> dict[str, str]:
    out: dict[str, str] = build_age5_combined_heavy_sync_contract_fields()
    out.update(dict(SANITY_CONTRACT_SUMMARY_FIELDS))
    out[PACK_GOLDEN_GRAPH_EXPORT_SUMMARY_KEY] = "0"
    out[PACK_GOLDEN_GRAPH_EXPORT_SYNC_KEY] = out[PACK_GOLDEN_GRAPH_EXPORT_SUMMARY_KEY]
    for key, _step_name, enabled_profiles in SANITY_SUMMARY_STEP_FIELDS:
        out[key] = "pending" if profile in enabled_profiles else "na"
        out[sync_mirror_key(key)] = out[key]
    for sanity_key, sync_key in AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS:
        if profile in AGE3_COMPLETION_GATE_CRITERIA_ENABLED_PROFILES:
            out[sanity_key] = "pending"
        else:
            out[sanity_key] = "na"
        out[sync_key] = out[sanity_key]
    for _step_name, code_key, count_key, enabled_profiles in COMPLETION_FAILURE_CODE_SUMMARY_GROUPS:
        if profile in enabled_profiles:
            out[code_key] = "pending"
            out[count_key] = "pending"
        else:
            out[code_key] = "na"
            out[count_key] = "na"
        out[sync_mirror_key(code_key)] = out[code_key]
        out[sync_mirror_key(count_key)] = out[count_key]
    if profile in AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_ENABLED_PROFILES:
        smoke_defaults = {
            "ci_sanity_age3_bogae_geoul_visibility_smoke_ok": "pending",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path": "-",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists": "0",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_schema": "-",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok": "pending",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok": "pending",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes": "pending",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes": "pending",
        }
    else:
        smoke_defaults = {
            "ci_sanity_age3_bogae_geoul_visibility_smoke_ok": "na",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path": "-",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists": "na",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_schema": "-",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok": "na",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok": "na",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes": "na",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes": "na",
        }
    out.update(smoke_defaults)
    for sanity_key, sync_key in AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SYNC_FIELD_PAIRS:
        out[sync_key] = out[sanity_key]
    if profile in SEAMGRIM_WASM_WEB_STEP_CHECK_ENABLED_PROFILES:
        step_check_defaults = {
            "ci_sanity_seamgrim_wasm_web_step_check_ok": "pending",
            "ci_sanity_seamgrim_wasm_web_step_check_report_path": "-",
            "ci_sanity_seamgrim_wasm_web_step_check_report_exists": "0",
            "ci_sanity_seamgrim_wasm_web_step_check_schema": "-",
            "ci_sanity_seamgrim_wasm_web_step_check_checked_files": "pending",
            "ci_sanity_seamgrim_wasm_web_step_check_missing_count": "pending",
        }
    else:
        step_check_defaults = {
            "ci_sanity_seamgrim_wasm_web_step_check_ok": "na",
            "ci_sanity_seamgrim_wasm_web_step_check_report_path": "-",
            "ci_sanity_seamgrim_wasm_web_step_check_report_exists": "na",
            "ci_sanity_seamgrim_wasm_web_step_check_schema": "-",
            "ci_sanity_seamgrim_wasm_web_step_check_checked_files": "-",
            "ci_sanity_seamgrim_wasm_web_step_check_missing_count": "-",
        }
    out.update(step_check_defaults)
    for sanity_key, sync_key in SEAMGRIM_WASM_WEB_STEP_CHECK_SYNC_FIELD_PAIRS:
        out[sync_key] = out[sanity_key]
    if profile in SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_ENABLED_PROFILES:
        pack_evidence_defaults = {
            "ci_sanity_seamgrim_pack_evidence_tier_runner_ok": "pending",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path": "-",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists": "0",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_schema": "-",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count": "pending",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count": "pending",
        }
    else:
        pack_evidence_defaults = {
            "ci_sanity_seamgrim_pack_evidence_tier_runner_ok": "na",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path": "-",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists": "na",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_schema": "-",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count": "-",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count": "-",
        }
    out.update(pack_evidence_defaults)
    for sanity_key, sync_key in SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SYNC_FIELD_PAIRS:
        out[sync_key] = out[sanity_key]
    if profile in SEAMGRIM_NUMERIC_FACTOR_POLICY_ENABLED_PROFILES:
        numeric_factor_policy_defaults = {
            "ci_sanity_seamgrim_numeric_factor_policy_ok": "pending",
            "ci_sanity_seamgrim_numeric_factor_policy_report_path": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_report_exists": "0",
            "ci_sanity_seamgrim_numeric_factor_policy_schema": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_text": "pending",
            "ci_sanity_seamgrim_numeric_factor_policy_bit_limit": "pending",
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_iters": "pending",
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds": "pending",
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds": "pending",
            "ci_sanity_seamgrim_numeric_factor_policy_fallback_limit": "pending",
            "ci_sanity_seamgrim_numeric_factor_policy_small_prime_max": "pending",
        }
    else:
        numeric_factor_policy_defaults = {
            "ci_sanity_seamgrim_numeric_factor_policy_ok": "na",
            "ci_sanity_seamgrim_numeric_factor_policy_report_path": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_report_exists": "na",
            "ci_sanity_seamgrim_numeric_factor_policy_schema": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_text": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_bit_limit": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_iters": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_fallback_limit": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_small_prime_max": "-",
        }
    out.update(numeric_factor_policy_defaults)
    for sanity_key, sync_key in SEAMGRIM_NUMERIC_FACTOR_POLICY_SYNC_FIELD_PAIRS:
        out[sync_key] = out[sanity_key]
    if profile in FIXED64_DARWIN_REAL_REPORT_LIVE_SUMMARY_ENABLED_PROFILES:
        fixed64_live_defaults = {
            "ci_sanity_fixed64_darwin_real_report_live_report_path": "-",
            "ci_sanity_fixed64_darwin_real_report_live_report_exists": "pending",
            "ci_sanity_fixed64_darwin_real_report_live_status": "pending",
            "ci_sanity_fixed64_darwin_real_report_live_resolved_status": "pending",
            "ci_sanity_fixed64_darwin_real_report_live_resolved_source": "pending",
            "ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count": "pending",
            "ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip": "pending",
        }
    else:
        fixed64_live_defaults = {
            "ci_sanity_fixed64_darwin_real_report_live_report_path": "-",
            "ci_sanity_fixed64_darwin_real_report_live_report_exists": "na",
            "ci_sanity_fixed64_darwin_real_report_live_status": "na",
            "ci_sanity_fixed64_darwin_real_report_live_resolved_status": "na",
            "ci_sanity_fixed64_darwin_real_report_live_resolved_source": "na",
            "ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count": "na",
            "ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip": "na",
        }
    out.update(fixed64_live_defaults)
    for sanity_key, sync_key in FIXED64_DARWIN_REAL_REPORT_LIVE_SYNC_FIELD_PAIRS:
        out[sync_key] = out[sanity_key]
    return out


def validate_pack_evidence_tier_summary_fields(
    doc: dict[str, object],
    profile: str,
    summary_fields: dict[str, str],
) -> tuple[bool, str, dict[str, str]]:
    enabled = profile in SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_ENABLED_PROFILES
    report_path_key = "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path"
    schema_key = "ci_sanity_seamgrim_pack_evidence_tier_runner_schema"
    docs_issue_count_key = "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count"
    repo_issue_count_key = "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count"

    if not enabled:
        for key in SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_BOOL_FIELDS:
            raw_value = str(doc.get(key, "")).strip()
            if raw_value != "na":
                return False, f"sanity pack_evidence summary expected na: {key}={raw_value}", summary_fields
            summary_fields[key] = raw_value
        for key in SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_TEXT_FIELDS:
            raw_value = str(doc.get(key, "")).strip() or "-"
            if key == report_path_key or key == schema_key:
                if raw_value != "-":
                    return False, f"sanity pack_evidence summary expected '-': {key}={raw_value}", summary_fields
            else:
                if raw_value not in {"-", "na"}:
                    return False, f"sanity pack_evidence summary expected '-'/'na': {key}={raw_value}", summary_fields
            summary_fields[key] = raw_value
        for sanity_key, sync_key in SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SYNC_FIELD_PAIRS:
            summary_fields[sync_key] = summary_fields[sanity_key]
        return True, "ok", summary_fields

    for key in SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_BOOL_FIELDS:
        raw_value = str(doc.get(key, "")).strip()
        if not raw_value:
            return False, f"sanity pack_evidence summary key missing: {key}", summary_fields
        if raw_value not in VALID_SANITY_SUMMARY_VALUES:
            return False, f"sanity pack_evidence summary value invalid: {key}={raw_value}", summary_fields
        if raw_value != "1":
            return False, f"sanity pack_evidence summary pass value invalid: {key}={raw_value}", summary_fields
        summary_fields[key] = raw_value

    report_path_text = str(doc.get(report_path_key, "")).strip()
    if not report_path_text or report_path_text == "-":
        return False, f"sanity pack_evidence summary key missing: {report_path_key}", summary_fields
    schema_text = str(doc.get(schema_key, "")).strip()
    if schema_text != SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA:
        return False, f"sanity pack_evidence schema mismatch: {schema_text}", summary_fields

    docs_issue_count_text = str(doc.get(docs_issue_count_key, "")).strip()
    repo_issue_count_text = str(doc.get(repo_issue_count_key, "")).strip()
    try:
        docs_issue_count_num = int(docs_issue_count_text)
    except Exception:
        return (
            False,
            f"sanity pack_evidence docs_issue_count invalid: {docs_issue_count_text}",
            summary_fields,
        )
    try:
        repo_issue_count_num = int(repo_issue_count_text)
    except Exception:
        return (
            False,
            f"sanity pack_evidence repo_issue_count invalid: {repo_issue_count_text}",
            summary_fields,
        )
    if docs_issue_count_num < 0 or docs_issue_count_num > SEAMGRIM_PACK_EVIDENCE_TIER_MAX_DOCS_ISSUES:
        return (
            False,
            f"sanity pack_evidence docs_issue_count out of range: {docs_issue_count_num}",
            summary_fields,
        )
    if repo_issue_count_num != SEAMGRIM_PACK_EVIDENCE_TIER_EXPECTED_REPO_ISSUES:
        return (
            False,
            f"sanity pack_evidence repo_issue_count must be {SEAMGRIM_PACK_EVIDENCE_TIER_EXPECTED_REPO_ISSUES}: "
            f"{repo_issue_count_num}",
            summary_fields,
        )

    report_path = Path(report_path_text)
    if not report_path.exists():
        return False, f"sanity pack_evidence report missing: {report_path}", summary_fields
    report_doc = load_json(report_path)
    if not isinstance(report_doc, dict):
        return False, f"sanity pack_evidence report invalid json: {report_path}", summary_fields
    if str(report_doc.get("schema", "")).strip() != SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA:
        return (
            False,
            f"sanity pack_evidence report schema mismatch: {report_doc.get('schema')}",
            summary_fields,
        )
    if not bool(report_doc.get("ok", False)):
        return False, "sanity pack_evidence report ok must be true", summary_fields
    docs_profile = report_doc.get("docs_profile")
    repo_profile = report_doc.get("repo_profile")
    if not isinstance(docs_profile, dict) or not isinstance(repo_profile, dict):
        return False, "sanity pack_evidence report profile keys missing", summary_fields
    if str(docs_profile.get("name", "")).strip() != "docs_ssot_rep10":
        return False, "sanity pack_evidence docs_profile.name mismatch", summary_fields
    if str(repo_profile.get("name", "")).strip() != "repo_rep10":
        return False, "sanity pack_evidence repo_profile.name mismatch", summary_fields
    try:
        report_docs_issue_count = int(docs_profile.get("issue_count", -1))
        report_repo_issue_count = int(repo_profile.get("issue_count", -1))
    except Exception:
        return False, "sanity pack_evidence report issue_count parse failed", summary_fields
    if report_docs_issue_count != docs_issue_count_num:
        return (
            False,
            "sanity pack_evidence docs_issue_count mismatch "
            f"summary={docs_issue_count_num} report={report_docs_issue_count}",
            summary_fields,
        )
    if report_repo_issue_count != repo_issue_count_num:
        return (
            False,
            "sanity pack_evidence repo_issue_count mismatch "
            f"summary={repo_issue_count_num} report={report_repo_issue_count}",
            summary_fields,
        )

    summary_fields[report_path_key] = report_path_text
    summary_fields[schema_key] = schema_text
    summary_fields[docs_issue_count_key] = str(docs_issue_count_num)
    summary_fields[repo_issue_count_key] = str(repo_issue_count_num)
    for sanity_key, sync_key in SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SYNC_FIELD_PAIRS:
        summary_fields[sync_key] = summary_fields[sanity_key]
    return True, "ok", summary_fields


def validate_numeric_factor_policy_summary_fields(
    doc: dict[str, object],
    profile: str,
    summary_fields: dict[str, str],
) -> tuple[bool, str, dict[str, str]]:
    enabled = profile in SEAMGRIM_NUMERIC_FACTOR_POLICY_ENABLED_PROFILES
    report_path_key = "ci_sanity_seamgrim_numeric_factor_policy_report_path"
    schema_key = "ci_sanity_seamgrim_numeric_factor_policy_schema"
    text_key = "ci_sanity_seamgrim_numeric_factor_policy_text"
    if not enabled:
        for key in SEAMGRIM_NUMERIC_FACTOR_POLICY_BOOL_FIELDS:
            raw_value = str(doc.get(key, "")).strip()
            if raw_value != "na":
                return (
                    False,
                    f"sanity numeric_factor policy summary expected na: {key}={raw_value}",
                    summary_fields,
                )
            summary_fields[key] = raw_value
        for key in SEAMGRIM_NUMERIC_FACTOR_POLICY_TEXT_FIELDS:
            raw_value = str(doc.get(key, "")).strip() or "-"
            if raw_value != "-":
                return (
                    False,
                    f"sanity numeric_factor policy summary expected '-': {key}={raw_value}",
                    summary_fields,
                )
            summary_fields[key] = raw_value
        for sanity_key, sync_key in SEAMGRIM_NUMERIC_FACTOR_POLICY_SYNC_FIELD_PAIRS:
            summary_fields[sync_key] = summary_fields[sanity_key]
        return True, "ok", summary_fields

    for key in SEAMGRIM_NUMERIC_FACTOR_POLICY_BOOL_FIELDS:
        raw_value = str(doc.get(key, "")).strip()
        if not raw_value:
            return False, f"sanity numeric_factor policy summary key missing: {key}", summary_fields
        if raw_value not in VALID_SANITY_SUMMARY_VALUES:
            return False, f"sanity numeric_factor policy summary value invalid: {key}={raw_value}", summary_fields
        if raw_value != "1":
            return False, f"sanity numeric_factor policy pass value invalid: {key}={raw_value}", summary_fields
        summary_fields[key] = raw_value

    report_path_text = str(doc.get(report_path_key, "")).strip()
    if not report_path_text or report_path_text == "-":
        return False, f"sanity numeric_factor policy key missing: {report_path_key}", summary_fields
    schema_text = str(doc.get(schema_key, "")).strip()
    if schema_text != SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA:
        return False, f"sanity numeric_factor policy schema mismatch: {schema_text}", summary_fields
    policy_text = str(doc.get(text_key, "")).strip()
    if not policy_text or policy_text in {"-", "na", "pending"}:
        return False, f"sanity numeric_factor policy text invalid: {policy_text}", summary_fields

    parsed_summary_values: dict[str, int] = {}
    for policy_key in SEAMGRIM_NUMERIC_FACTOR_POLICY_KEYS:
        summary_key = f"ci_sanity_seamgrim_numeric_factor_policy_{policy_key}"
        raw_value = str(doc.get(summary_key, "")).strip()
        if not raw_value:
            return False, f"sanity numeric_factor policy key missing: {summary_key}", summary_fields
        try:
            parsed_value = int(raw_value)
        except Exception:
            return False, f"sanity numeric_factor policy value invalid: {summary_key}={raw_value}", summary_fields
        expected_value = int(SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS[policy_key])
        if parsed_value != expected_value:
            return (
                False,
                f"sanity numeric_factor policy mismatch: {summary_key}={parsed_value} expected={expected_value}",
                summary_fields,
            )
        parsed_summary_values[policy_key] = parsed_value
        summary_fields[summary_key] = str(parsed_value)

    report_path = Path(report_path_text)
    if not report_path.exists():
        return False, f"sanity numeric_factor policy report missing: {report_path}", summary_fields
    report_doc = load_json(report_path)
    if not isinstance(report_doc, dict):
        return False, f"sanity numeric_factor policy report invalid json: {report_path}", summary_fields
    if str(report_doc.get("schema", "")).strip() != SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA:
        return (
            False,
            f"sanity numeric_factor policy report schema mismatch: {report_doc.get('schema')}",
            summary_fields,
        )
    if str(report_doc.get("status", "")).strip() != "pass":
        return (
            False,
            f"sanity numeric_factor policy report status mismatch: {report_doc.get('status')}",
            summary_fields,
        )
    if not bool(report_doc.get("ok", False)):
        return False, "sanity numeric_factor policy report ok must be true", summary_fields
    if str(report_doc.get("code", "")).strip() != "OK":
        return (
            False,
            f"sanity numeric_factor policy report code mismatch: {report_doc.get('code')}",
            summary_fields,
        )
    report_policy_text = str(report_doc.get("numeric_factor_policy_text", "")).strip()
    if report_policy_text != policy_text:
        return (
            False,
            "sanity numeric_factor policy text mismatch "
            f"summary={policy_text} report={report_policy_text}",
            summary_fields,
        )
    report_policy = report_doc.get("numeric_factor_policy")
    if not isinstance(report_policy, dict):
        return False, "sanity numeric_factor policy report map missing", summary_fields
    for policy_key in SEAMGRIM_NUMERIC_FACTOR_POLICY_KEYS:
        expected_value = int(SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS[policy_key])
        try:
            report_value = int(report_policy.get(policy_key))
        except Exception:
            return (
                False,
                f"sanity numeric_factor policy report value invalid: {policy_key}={report_policy.get(policy_key)}",
                summary_fields,
            )
        if report_value != expected_value:
            return (
                False,
                f"sanity numeric_factor policy report mismatch: {policy_key}={report_value} expected={expected_value}",
                summary_fields,
            )
        if report_value != parsed_summary_values[policy_key]:
            return (
                False,
                "sanity numeric_factor policy summary/report mismatch "
                f"{policy_key}: summary={parsed_summary_values[policy_key]} report={report_value}",
                summary_fields,
            )

    summary_fields[report_path_key] = report_path_text
    summary_fields[schema_key] = schema_text
    summary_fields[text_key] = policy_text
    for sanity_key, sync_key in SEAMGRIM_NUMERIC_FACTOR_POLICY_SYNC_FIELD_PAIRS:
        summary_fields[sync_key] = summary_fields[sanity_key]
    return True, "ok", summary_fields


def validate_sanity_contract(path: Path, expected_profile: str = "full") -> tuple[bool, str, str, dict[str, str]]:
    doc = load_json(path)
    if not isinstance(doc, dict):
        fallback_profile = expected_profile if expected_profile in VALID_SANITY_PROFILES else "full"
        return False, f"invalid sanity json: {path}", fallback_profile, build_default_sanity_summary_fields(fallback_profile)
    if str(doc.get("schema", "")).strip() != "ddn.ci.sanity_gate.v1":
        fallback_profile = expected_profile if expected_profile in VALID_SANITY_PROFILES else "full"
        return (
            False,
            f"sanity schema mismatch: {doc.get('schema')}",
            fallback_profile,
            build_default_sanity_summary_fields(fallback_profile),
        )
    if str(doc.get("status", "")).strip() != "pass":
        fallback_profile = expected_profile if expected_profile in VALID_SANITY_PROFILES else "full"
        return (
            False,
            f"sanity status mismatch: {doc.get('status')}",
            fallback_profile,
            build_default_sanity_summary_fields(fallback_profile),
        )
    if str(doc.get("code", "")).strip() != "OK":
        fallback_profile = expected_profile if expected_profile in VALID_SANITY_PROFILES else "full"
        return (
            False,
            f"sanity code mismatch: {doc.get('code')}",
            fallback_profile,
            build_default_sanity_summary_fields(fallback_profile),
        )
    if str(doc.get("step", "")).strip() != "all":
        fallback_profile = expected_profile if expected_profile in VALID_SANITY_PROFILES else "full"
        return (
            False,
            f"sanity step mismatch: {doc.get('step')}",
            fallback_profile,
            build_default_sanity_summary_fields(fallback_profile),
        )

    profile = str(doc.get("profile", "")).strip()
    if not profile:
        profile = expected_profile if expected_profile in VALID_SANITY_PROFILES else "full"
    if profile not in VALID_SANITY_PROFILES:
        return False, f"sanity profile invalid: {profile}", profile, build_default_sanity_summary_fields("full")
    if expected_profile in VALID_SANITY_PROFILES and profile != expected_profile:
        return (
            False,
            f"sanity profile mismatch expected={expected_profile} actual={profile}",
            profile,
            build_default_sanity_summary_fields(profile),
        )

    steps = doc.get("steps")
    if not isinstance(steps, list):
        return False, "sanity steps must be list", profile, build_default_sanity_summary_fields(profile)
    required_steps = resolve_required_sanity_steps(profile)
    if len(steps) < len(required_steps):
        return False, f"sanity step_count too small: {len(steps)} profile={profile}", profile, build_default_sanity_summary_fields(profile)

    step_index: dict[str, dict] = {}
    for row in steps:
        if not isinstance(row, dict):
            return False, "sanity steps contains non-object row", profile, build_default_sanity_summary_fields(profile)
        name = str(row.get("step", "")).strip()
        if name:
            step_index[name] = row

    for required_step in required_steps:
        row = step_index.get(required_step)
        if row is None:
            return False, f"sanity required step missing: {required_step}", profile, build_default_sanity_summary_fields(profile)
        if not bool(row.get("ok", False)):
            return False, f"sanity required step not ok: {required_step}", profile, build_default_sanity_summary_fields(profile)
        try:
            rc = int(row.get("returncode", -1))
        except Exception:
            rc = -1
        if rc != 0:
            return (
                False,
                f"sanity required step rc!=0: {required_step} rc={row.get('returncode')}",
                profile,
                build_default_sanity_summary_fields(profile),
            )

    summary_fields = build_default_sanity_summary_fields(profile)
    for key, expected_value in SANITY_CONTRACT_SUMMARY_FIELDS:
        raw_value = str(doc.get(key, "")).strip()
        if not raw_value:
            return False, f"sanity contract summary key missing: {key}", profile, summary_fields
        if raw_value != expected_value:
            return (
                False,
                f"sanity contract summary mismatch: {key} expected={expected_value} actual={raw_value}",
                profile,
                summary_fields,
            )
        summary_fields[key] = raw_value
    for key, step_name, enabled_profiles in SANITY_SUMMARY_STEP_FIELDS:
        raw_value = str(doc.get(key, "")).strip()
        if not raw_value:
            return False, f"sanity summary key missing: {key}", profile, summary_fields
        if raw_value not in VALID_SANITY_SUMMARY_VALUES:
            return False, f"sanity summary value invalid: {key}={raw_value}", profile, summary_fields
        if profile not in enabled_profiles:
            if raw_value != "na":
                return False, f"sanity summary expected na: {key}={raw_value}", profile, summary_fields
            summary_fields[key] = raw_value
            continue
        row = step_index.get(step_name)
        expected_value = "pending"
        if isinstance(row, dict):
            try:
                rc = int(row.get("returncode", -1))
            except Exception:
                rc = -1
            expected_value = "1" if bool(row.get("ok", False)) and rc == 0 else "0"
        if raw_value != expected_value:
            return (
                False,
                f"sanity summary mismatch: {key} expected={expected_value} actual={raw_value}",
                profile,
                summary_fields,
            )
        summary_fields[key] = raw_value
        summary_fields[sync_mirror_key(key)] = raw_value

    pack_golden_graph_export_value = str(doc.get(PACK_GOLDEN_GRAPH_EXPORT_SUMMARY_KEY, "")).strip()
    if pack_golden_graph_export_value not in {"0", "1"}:
        return (
            False,
            "sanity summary value invalid: "
            f"{PACK_GOLDEN_GRAPH_EXPORT_SUMMARY_KEY}={pack_golden_graph_export_value}",
            profile,
            summary_fields,
        )
    expected_pack_golden_graph_export_value = (
        "1" if profile in PACK_GOLDEN_GRAPH_EXPORT_REQUIRED_PROFILES else "0"
    )
    if pack_golden_graph_export_value != expected_pack_golden_graph_export_value:
        return (
            False,
            "sanity summary mismatch: "
            f"{PACK_GOLDEN_GRAPH_EXPORT_SUMMARY_KEY} expected={expected_pack_golden_graph_export_value} "
            f"actual={pack_golden_graph_export_value}",
            profile,
            summary_fields,
        )
    summary_fields[PACK_GOLDEN_GRAPH_EXPORT_SUMMARY_KEY] = pack_golden_graph_export_value
    summary_fields[PACK_GOLDEN_GRAPH_EXPORT_SYNC_KEY] = pack_golden_graph_export_value

    age3_step_row = step_index.get("age3_completion_gate")
    age3_criteria_enabled = profile in AGE3_COMPLETION_GATE_CRITERIA_ENABLED_PROFILES
    for sanity_key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS:
        raw_value = str(doc.get(sanity_key, "")).strip()
        if not raw_value:
            return False, f"sanity age3 criteria key missing: {sanity_key}", profile, summary_fields
        if not age3_criteria_enabled:
            if raw_value != "na":
                return (
                    False,
                    f"sanity age3 criteria expected na: {sanity_key}={raw_value}",
                    profile,
                    summary_fields,
                )
            summary_fields[sanity_key] = raw_value
            continue
        if age3_step_row is None:
            if raw_value != "pending":
                return (
                    False,
                    f"sanity age3 criteria expected pending: {sanity_key}={raw_value}",
                    profile,
                    summary_fields,
                )
            summary_fields[sanity_key] = raw_value
            continue
        if raw_value not in {"1", "0"}:
            return (
                False,
                f"sanity age3 criteria value invalid: {sanity_key}={raw_value}",
                profile,
                summary_fields,
            )
        summary_fields[sanity_key] = raw_value
    for sanity_key, sync_key in AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS:
        summary_fields[sync_key] = summary_fields[sanity_key]

    for step_name, code_key, count_key, enabled_profiles in COMPLETION_FAILURE_CODE_SUMMARY_GROUPS:
        code_value = str(doc.get(code_key, "")).strip()
        if not code_value:
            return False, f"sanity failure-code key missing: {code_key}", profile, summary_fields
        count_value = str(doc.get(count_key, "")).strip()
        if not count_value:
            return False, f"sanity failure-code key missing: {count_key}", profile, summary_fields
        if profile not in enabled_profiles:
            if code_value != "na":
                return (
                    False,
                    f"sanity failure-code expected na: {code_key}={code_value}",
                    profile,
                    summary_fields,
                )
            if count_value != "na":
                return (
                    False,
                    f"sanity failure-code expected na: {count_key}={count_value}",
                    profile,
                    summary_fields,
                )
            summary_fields[code_key] = code_value
            summary_fields[count_key] = count_value
            summary_fields[sync_mirror_key(code_key)] = code_value
            summary_fields[sync_mirror_key(count_key)] = count_value
            continue
        row = step_index.get(step_name)
        if row is None:
            if code_value != "pending":
                return (
                    False,
                    f"sanity failure-code expected pending: {code_key}={code_value}",
                    profile,
                    summary_fields,
                )
            if count_value != "pending":
                return (
                    False,
                    f"sanity failure-code expected pending: {count_key}={count_value}",
                    profile,
                    summary_fields,
                )
            summary_fields[code_key] = code_value
            summary_fields[count_key] = count_value
            summary_fields[sync_mirror_key(code_key)] = code_value
            summary_fields[sync_mirror_key(count_key)] = count_value
            continue
        if code_value in {"pending", "na"}:
            return (
                False,
                f"sanity failure-code value invalid: {code_key}={code_value}",
                profile,
                summary_fields,
            )
        if count_value in {"pending", "na"}:
            return (
                False,
                f"sanity failure-code value invalid: {count_key}={count_value}",
                profile,
                summary_fields,
            )
        try:
            count_num = int(count_value)
        except Exception:
            return (
                False,
                f"sanity failure-code count invalid: {count_key}={count_value}",
                profile,
                summary_fields,
            )
        if count_num < 0:
            return (
                False,
                f"sanity failure-code count negative: {count_key}={count_num}",
                profile,
                summary_fields,
            )
        if code_value == "-":
            if count_num != 0:
                return (
                    False,
                    f"sanity failure-code/count mismatch: {code_key}={code_value} {count_key}={count_num}",
                    profile,
                    summary_fields,
                )
        else:
            code_items = [token.strip() for token in code_value.split(",") if token.strip()]
            if len(code_items) != count_num:
                return (
                    False,
                    f"sanity failure-code/count mismatch: {code_key}={code_value} {count_key}={count_num}",
                    profile,
                    summary_fields,
                )
            if len(set(code_items)) != len(code_items):
                return (
                    False,
                    f"sanity failure-code duplicated: {code_key}={code_value}",
                    profile,
                    summary_fields,
                )
            for token in code_items:
                if not FAILURE_CODE_PATTERN.fullmatch(token):
                    return (
                        False,
                        f"sanity failure-code token invalid: {code_key}={token}",
                        profile,
                        summary_fields,
                    )
        summary_fields[code_key] = code_value
        summary_fields[count_key] = count_value
        summary_fields[sync_mirror_key(code_key)] = code_value
        summary_fields[sync_mirror_key(count_key)] = count_value

    smoke_enabled = profile in AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_ENABLED_PROFILES
    smoke_report_path_key = "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path"
    smoke_schema_key = "ci_sanity_age3_bogae_geoul_visibility_smoke_schema"
    smoke_sim_state_hash_changes_key = (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes"
    )
    smoke_sim_bogae_hash_changes_key = (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes"
    )
    if not smoke_enabled:
        for key in AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_BOOL_FIELDS:
            raw_value = str(doc.get(key, "")).strip()
            if raw_value != "na":
                return False, f"sanity smoke summary expected na: {key}={raw_value}", profile, summary_fields
            summary_fields[key] = raw_value
        for sanity_key, sync_key in AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SYNC_FIELD_PAIRS:
            summary_fields[sync_key] = summary_fields[sanity_key]
        for key in AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_TEXT_FIELDS:
            raw_value = str(doc.get(key, "")).strip() or "-"
            if raw_value != "-":
                return False, f"sanity smoke summary expected '-': {key}={raw_value}", profile, summary_fields
            summary_fields[key] = raw_value
        for sanity_key, sync_key in AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SYNC_FIELD_PAIRS:
            summary_fields[sync_key] = summary_fields[sanity_key]
        return True, "ok", profile, summary_fields

    for key in AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_BOOL_FIELDS:
        raw_value = str(doc.get(key, "")).strip()
        if not raw_value:
            return False, f"sanity smoke summary key missing: {key}", profile, summary_fields
        if raw_value not in VALID_SANITY_SUMMARY_VALUES:
            return False, f"sanity smoke summary value invalid: {key}={raw_value}", profile, summary_fields
        if raw_value != "1":
            return False, f"sanity smoke summary pass value invalid: {key}={raw_value}", profile, summary_fields
        summary_fields[key] = raw_value

    smoke_report_path_text = str(doc.get(smoke_report_path_key, "")).strip()
    if not smoke_report_path_text or smoke_report_path_text == "-":
        return False, f"sanity smoke summary key missing: {smoke_report_path_key}", profile, summary_fields
    smoke_schema_text = str(doc.get(smoke_schema_key, "")).strip()
    if smoke_schema_text != AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA:
        return (
            False,
            f"sanity smoke schema mismatch: {smoke_schema_text}",
            profile,
            summary_fields,
        )
    smoke_report_path = Path(smoke_report_path_text)
    if not smoke_report_path.exists():
        return (
            False,
            f"sanity smoke report missing: {smoke_report_path}",
            profile,
            summary_fields,
        )
    smoke_doc = load_json(smoke_report_path)
    if not isinstance(smoke_doc, dict):
        return (
            False,
            f"sanity smoke report invalid json: {smoke_report_path}",
            profile,
            summary_fields,
        )
    if str(smoke_doc.get("schema", "")).strip() != AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA:
        return (
            False,
            f"sanity smoke report schema mismatch: {smoke_doc.get('schema')}",
            profile,
            summary_fields,
        )
    if not bool(smoke_doc.get("overall_ok", False)):
        return (
            False,
            "sanity smoke report overall_ok must be true",
            profile,
            summary_fields,
        )
    smoke_checks = smoke_doc.get("checks")
    if not isinstance(smoke_checks, list) or not smoke_checks:
        return (
            False,
            "sanity smoke report checks must be non-empty list",
            profile,
            summary_fields,
        )
    smoke_sim_hash_delta = smoke_doc.get("simulation_hash_delta")
    if not isinstance(smoke_sim_hash_delta, dict):
        return (
            False,
            "sanity smoke report simulation_hash_delta must be object",
            profile,
            summary_fields,
        )
    smoke_sim_state_hash_changes = "1" if bool(smoke_sim_hash_delta.get("state_hash_changes", False)) else "0"
    smoke_sim_bogae_hash_changes = "1" if bool(smoke_sim_hash_delta.get("bogae_hash_changes", False)) else "0"
    if smoke_sim_state_hash_changes != "1":
        return (
            False,
            "sanity smoke report requires simulation_hash_delta.state_hash_changes=true",
            profile,
            summary_fields,
        )
    if smoke_sim_bogae_hash_changes != "1":
        return (
            False,
            "sanity smoke report requires simulation_hash_delta.bogae_hash_changes=true",
            profile,
            summary_fields,
        )
    if summary_fields.get(smoke_sim_state_hash_changes_key, "") != smoke_sim_state_hash_changes:
        return (
            False,
            "sanity smoke summary/report mismatch: sim_state_hash_changes",
            profile,
            summary_fields,
        )
    if summary_fields.get(smoke_sim_bogae_hash_changes_key, "") != smoke_sim_bogae_hash_changes:
        return (
            False,
            "sanity smoke summary/report mismatch: sim_bogae_hash_changes",
            profile,
            summary_fields,
        )
    summary_fields[smoke_report_path_key] = smoke_report_path_text
    summary_fields[smoke_schema_key] = smoke_schema_text
    for sanity_key, sync_key in AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SYNC_FIELD_PAIRS:
        summary_fields[sync_key] = summary_fields[sanity_key]

    fixed64_enabled = profile in FIXED64_DARWIN_REAL_REPORT_LIVE_SUMMARY_ENABLED_PROFILES
    if not fixed64_enabled:
        for key in FIXED64_DARWIN_REAL_REPORT_LIVE_BOOL_FIELDS:
            raw_value = str(doc.get(key, "")).strip()
            if raw_value != "na":
                return False, f"sanity fixed64 live summary expected na: {key}={raw_value}", profile, summary_fields
            summary_fields[key] = raw_value
        for key in FIXED64_DARWIN_REAL_REPORT_LIVE_TEXT_FIELDS:
            raw_value = str(doc.get(key, "")).strip() or "-"
            if raw_value != "-":
                return False, f"sanity fixed64 live summary expected '-': {key}={raw_value}", profile, summary_fields
            summary_fields[key] = raw_value
        for sanity_key, sync_key in FIXED64_DARWIN_REAL_REPORT_LIVE_SYNC_FIELD_PAIRS:
            summary_fields[sync_key] = summary_fields[sanity_key]
    else:
        fixed64_report_exists_key = "ci_sanity_fixed64_darwin_real_report_live_report_exists"
        fixed64_source_zip_key = "ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip"
        fixed64_report_path_key = "ci_sanity_fixed64_darwin_real_report_live_report_path"
        fixed64_status_key = "ci_sanity_fixed64_darwin_real_report_live_status"
        fixed64_resolved_status_key = "ci_sanity_fixed64_darwin_real_report_live_resolved_status"
        fixed64_resolved_source_key = "ci_sanity_fixed64_darwin_real_report_live_resolved_source"
        fixed64_invalid_count_key = "ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count"

        fixed64_report_exists = str(doc.get(fixed64_report_exists_key, "")).strip()
        if fixed64_report_exists not in {"1", "0"}:
            return (
                False,
                f"sanity fixed64 live report_exists invalid: {fixed64_report_exists}",
                profile,
                summary_fields,
            )
        if fixed64_report_exists != "1":
            return (
                False,
                f"sanity fixed64 live report_exists must be 1: {fixed64_report_exists}",
                profile,
                summary_fields,
            )
        summary_fields[fixed64_report_exists_key] = fixed64_report_exists

        fixed64_source_zip = str(doc.get(fixed64_source_zip_key, "")).strip()
        if fixed64_source_zip not in {"0", "1"}:
            return (
                False,
                f"sanity fixed64 live resolved_source_zip invalid: {fixed64_source_zip}",
                profile,
                summary_fields,
            )
        summary_fields[fixed64_source_zip_key] = fixed64_source_zip

        fixed64_report_path_text = str(doc.get(fixed64_report_path_key, "")).strip()
        if not fixed64_report_path_text or fixed64_report_path_text == "-":
            return (
                False,
                f"sanity fixed64 live summary key missing: {fixed64_report_path_key}",
                profile,
                summary_fields,
            )
        fixed64_report_path = Path(fixed64_report_path_text)
        if not fixed64_report_path.exists():
            return (
                False,
                f"sanity fixed64 live report missing: {fixed64_report_path}",
                profile,
                summary_fields,
            )
        summary_fields[fixed64_report_path_key] = fixed64_report_path_text

        fixed64_status = str(doc.get(fixed64_status_key, "")).strip()
        if not fixed64_status or fixed64_status in {"na", "pending"}:
            return False, f"sanity fixed64 live status invalid: {fixed64_status}", profile, summary_fields
        summary_fields[fixed64_status_key] = fixed64_status

        fixed64_resolved_status = str(doc.get(fixed64_resolved_status_key, "")).strip()
        fixed64_resolved_source = str(doc.get(fixed64_resolved_source_key, "")).strip()
        if fixed64_status == "skip_disabled":
            if fixed64_resolved_status != "-":
                return (
                    False,
                    f"sanity fixed64 live resolved_status must be '-' when skip_disabled: {fixed64_resolved_status}",
                    profile,
                    summary_fields,
                )
            if fixed64_resolved_source != "-":
                return (
                    False,
                    f"sanity fixed64 live resolved_source must be '-' when skip_disabled: {fixed64_resolved_source}",
                    profile,
                    summary_fields,
                )
            if fixed64_source_zip != "0":
                return (
                    False,
                    f"sanity fixed64 live resolved_source_zip must be 0 when skip_disabled: {fixed64_source_zip}",
                    profile,
                    summary_fields,
                )
        else:
            if not fixed64_resolved_status or fixed64_resolved_status in {"-", "na", "pending"}:
                return (
                    False,
                    f"sanity fixed64 live resolved_status invalid: {fixed64_resolved_status}",
                    profile,
                    summary_fields,
                )
            if not fixed64_resolved_source or fixed64_resolved_source in {"-", "na", "pending"}:
                return (
                    False,
                    f"sanity fixed64 live resolved_source invalid: {fixed64_resolved_source}",
                    profile,
                    summary_fields,
                )
        summary_fields[fixed64_resolved_status_key] = fixed64_resolved_status
        summary_fields[fixed64_resolved_source_key] = fixed64_resolved_source

        fixed64_invalid_count_text = str(doc.get(fixed64_invalid_count_key, "")).strip()
        try:
            fixed64_invalid_count_num = int(fixed64_invalid_count_text)
        except Exception:
            return (
                False,
                f"sanity fixed64 live invalid_count is not integer: {fixed64_invalid_count_text}",
                profile,
                summary_fields,
            )
        if fixed64_invalid_count_num < 0:
            return (
                False,
                f"sanity fixed64 live invalid_count negative: {fixed64_invalid_count_num}",
                profile,
                summary_fields,
            )
        if fixed64_status == "skip_disabled" and fixed64_invalid_count_num != 0:
            return (
                False,
                f"sanity fixed64 live invalid_count must be 0 when skip_disabled: {fixed64_invalid_count_num}",
                profile,
                summary_fields,
            )
        summary_fields[fixed64_invalid_count_key] = str(fixed64_invalid_count_num)

        for sanity_key, sync_key in FIXED64_DARWIN_REAL_REPORT_LIVE_SYNC_FIELD_PAIRS:
            summary_fields[sync_key] = summary_fields[sanity_key]

    step_enabled = profile in SEAMGRIM_WASM_WEB_STEP_CHECK_ENABLED_PROFILES
    step_report_path_key = "ci_sanity_seamgrim_wasm_web_step_check_report_path"
    step_schema_key = "ci_sanity_seamgrim_wasm_web_step_check_schema"
    step_checked_files_key = "ci_sanity_seamgrim_wasm_web_step_check_checked_files"
    step_missing_count_key = "ci_sanity_seamgrim_wasm_web_step_check_missing_count"
    if not step_enabled:
        for key in SEAMGRIM_WASM_WEB_STEP_CHECK_BOOL_FIELDS:
            raw_value = str(doc.get(key, "")).strip()
            if raw_value != "na":
                return False, f"sanity wasm/web step summary expected na: {key}={raw_value}", profile, summary_fields
            summary_fields[key] = raw_value
        for key in SEAMGRIM_WASM_WEB_STEP_CHECK_TEXT_FIELDS:
            raw_value = str(doc.get(key, "")).strip() or "-"
            if raw_value != "-":
                return False, f"sanity wasm/web step summary expected '-': {key}={raw_value}", profile, summary_fields
            summary_fields[key] = raw_value
        for sanity_key, sync_key in SEAMGRIM_WASM_WEB_STEP_CHECK_SYNC_FIELD_PAIRS:
            summary_fields[sync_key] = summary_fields[sanity_key]
        pack_ok, pack_msg, summary_fields = validate_pack_evidence_tier_summary_fields(
            doc=doc,
            profile=profile,
            summary_fields=summary_fields,
        )
        if not pack_ok:
            return False, pack_msg, profile, summary_fields
        numeric_ok, numeric_msg, summary_fields = validate_numeric_factor_policy_summary_fields(
            doc=doc,
            profile=profile,
            summary_fields=summary_fields,
        )
        if not numeric_ok:
            return False, numeric_msg, profile, summary_fields
        return True, "ok", profile, summary_fields

    for key in SEAMGRIM_WASM_WEB_STEP_CHECK_BOOL_FIELDS:
        raw_value = str(doc.get(key, "")).strip()
        if not raw_value:
            return False, f"sanity wasm/web step summary key missing: {key}", profile, summary_fields
        if raw_value not in VALID_SANITY_SUMMARY_VALUES:
            return False, f"sanity wasm/web step summary value invalid: {key}={raw_value}", profile, summary_fields
        if raw_value != "1":
            return False, f"sanity wasm/web step summary pass value invalid: {key}={raw_value}", profile, summary_fields
        summary_fields[key] = raw_value

    step_report_path_text = str(doc.get(step_report_path_key, "")).strip()
    if not step_report_path_text or step_report_path_text == "-":
        return False, f"sanity wasm/web step summary key missing: {step_report_path_key}", profile, summary_fields
    step_schema_text = str(doc.get(step_schema_key, "")).strip()
    if step_schema_text != SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA:
        return (
            False,
            f"sanity wasm/web step schema mismatch: {step_schema_text}",
            profile,
            summary_fields,
        )
    step_checked_files_text = str(doc.get(step_checked_files_key, "")).strip()
    step_missing_count_text = str(doc.get(step_missing_count_key, "")).strip()
    try:
        step_checked_files_num = int(step_checked_files_text)
    except Exception:
        return (
            False,
            f"sanity wasm/web step checked_files invalid: {step_checked_files_text}",
            profile,
            summary_fields,
        )
    if step_checked_files_num < SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES:
        return (
            False,
            f"sanity wasm/web step checked_files too small: {step_checked_files_num}",
            profile,
            summary_fields,
        )
    try:
        step_missing_count_num = int(step_missing_count_text)
    except Exception:
        return (
            False,
            f"sanity wasm/web step missing_count invalid: {step_missing_count_text}",
            profile,
            summary_fields,
        )
    if step_missing_count_num != 0:
        return (
            False,
            f"sanity wasm/web step missing_count must be 0: {step_missing_count_num}",
            profile,
            summary_fields,
        )

    step_report_path = Path(step_report_path_text)
    if not step_report_path.exists():
        return (
            False,
            f"sanity wasm/web step report missing: {step_report_path}",
            profile,
            summary_fields,
        )
    step_doc = load_json(step_report_path)
    if not isinstance(step_doc, dict):
        return (
            False,
            f"sanity wasm/web step report invalid json: {step_report_path}",
            profile,
            summary_fields,
        )
    if str(step_doc.get("schema", "")).strip() != SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA:
        return (
            False,
            f"sanity wasm/web step report schema mismatch: {step_doc.get('schema')}",
            profile,
            summary_fields,
        )
    if str(step_doc.get("status", "")).strip() != "pass":
        return (
            False,
            f"sanity wasm/web step report status mismatch: {step_doc.get('status')}",
            profile,
            summary_fields,
        )
    if not bool(step_doc.get("ok", False)):
        return (
            False,
            "sanity wasm/web step report ok must be true",
            profile,
            summary_fields,
        )
    if str(step_doc.get("code", "")).strip() != "OK":
        return (
            False,
            f"sanity wasm/web step report code mismatch: {step_doc.get('code')}",
            profile,
            summary_fields,
        )
    try:
        step_doc_checked_files = int(step_doc.get("checked_files", -1))
    except Exception:
        step_doc_checked_files = -1
    if step_doc_checked_files < SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES:
        return (
            False,
            f"sanity wasm/web step report checked_files too small: {step_doc_checked_files}",
            profile,
            summary_fields,
        )
    try:
        step_doc_missing_count = int(step_doc.get("missing_count", -1))
    except Exception:
        step_doc_missing_count = -1
    if step_doc_missing_count != 0:
        return (
            False,
            f"sanity wasm/web step report missing_count must be 0: {step_doc_missing_count}",
            profile,
            summary_fields,
        )
    step_doc_missing = step_doc.get("missing")
    if not isinstance(step_doc_missing, list) or step_doc_missing:
        return (
            False,
            "sanity wasm/web step report missing must be empty list",
            profile,
            summary_fields,
        )
    if step_doc_checked_files != step_checked_files_num:
        return (
            False,
            "sanity wasm/web step checked_files mismatch "
            f"summary={step_checked_files_num} report={step_doc_checked_files}",
            profile,
            summary_fields,
        )
    if step_doc_missing_count != step_missing_count_num:
        return (
            False,
            "sanity wasm/web step missing_count mismatch "
            f"summary={step_missing_count_num} report={step_doc_missing_count}",
            profile,
            summary_fields,
        )
    summary_fields[step_report_path_key] = step_report_path_text
    summary_fields[step_schema_key] = step_schema_text
    summary_fields[step_checked_files_key] = step_checked_files_text
    summary_fields[step_missing_count_key] = step_missing_count_text
    for sanity_key, sync_key in SEAMGRIM_WASM_WEB_STEP_CHECK_SYNC_FIELD_PAIRS:
        summary_fields[sync_key] = summary_fields[sanity_key]

    pack_ok, pack_msg, summary_fields = validate_pack_evidence_tier_summary_fields(
        doc=doc,
        profile=profile,
        summary_fields=summary_fields,
    )
    if not pack_ok:
        return False, pack_msg, profile, summary_fields
    numeric_ok, numeric_msg, summary_fields = validate_numeric_factor_policy_summary_fields(
        doc=doc,
        profile=profile,
        summary_fields=summary_fields,
    )
    if not numeric_ok:
        return False, numeric_msg, profile, summary_fields

    return True, "ok", profile, summary_fields


def first_message(stdout: str, stderr: str) -> str:
    for raw in (stderr.splitlines() + stdout.splitlines()):
        line = str(raw).strip()
        if line:
            return clip(line, 220)
    return "-"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run sync-readiness CI checks (pipeline flags/selftests/sanity/aggregate) in one command"
    )
    parser.add_argument("--report-dir", default="build/reports", help="report directory")
    parser.add_argument("--report-prefix", default="dev_sync_readiness", help="report prefix for aggregate gate")
    parser.add_argument("--json-out", default="", help="optional path for sync-readiness report json")
    parser.add_argument(
        "--sanity-profile",
        choices=VALID_SANITY_PROFILES,
        default="full",
        help="sanity gate profile to validate/propagate (default: full)",
    )
    parser.add_argument(
        "--validate-only-sanity-json",
        default="",
        help="validate-only mode: skip step execution and validate the given ci_sanity_gate json path",
    )
    parser.add_argument(
        "--skip-aggregate",
        action="store_true",
        help="skip aggregate gate run (quick mode)",
    )
    args = parser.parse_args()

    py = sys.executable
    report_dir = Path(args.report_dir)
    report_dir.mkdir(parents=True, exist_ok=True)
    prefix = args.report_prefix.strip() or "dev_sync_readiness"
    sanity_profile = args.sanity_profile.strip() or "full"
    sanity_json = report_dir / f"{prefix}.ci_sanity_gate.detjson"
    validate_only_sanity_json = args.validate_only_sanity_json.strip()

    rows: list[dict[str, object]] = []
    all_ok = True
    status_code = SYNC_READINESS_OK
    status_step = "all"
    status_msg = "-"
    started = datetime.now(timezone.utc).isoformat()
    detected_sanity_profile = sanity_profile
    sanity_summary_fields = build_default_sanity_summary_fields(sanity_profile)
    if validate_only_sanity_json:
        validate_path = Path(validate_only_sanity_json)
        tick = time.perf_counter()
        if not validate_path.exists():
            all_ok = False
            contract_msg = f"missing sanity json path: {validate_path}"
            status_code = SYNC_READINESS_VALIDATE_ONLY_PATH_MISSING
            status_step = "validate_only"
            status_msg = contract_msg
            contract_ok = False
            detected_sanity_profile = sanity_profile
        else:
            contract_ok, contract_msg, detected_sanity_profile, sanity_summary_fields = validate_sanity_contract(
                validate_path,
                expected_profile=sanity_profile,
            )
            if not contract_ok:
                all_ok = False
                status_code = SYNC_READINESS_SANITY_CONTRACT_FAIL
                status_step = "sanity_gate_contract"
                status_msg = contract_msg
        elapsed_ms = int((time.perf_counter() - tick) * 1000)
        rows.append(
            {
                "name": "sanity_gate_contract",
                "ok": bool(contract_ok),
                "returncode": 0 if contract_ok else 1,
                "elapsed_ms": elapsed_ms,
                "cmd": ["internal", "validate_sanity_contract", str(validate_path)],
                "stdout_head": clip(contract_msg, 220),
                "stderr_head": "-" if contract_ok else clip(contract_msg, 220),
            }
        )
    else:
        steps: list[tuple[str, list[str]]] = [
            ("pipeline_emit_flags_check", [py, "tests/run_ci_pipeline_emit_flags_check.py"]),
            ("pipeline_emit_flags_selftest", [py, "tests/run_ci_pipeline_emit_flags_check_selftest.py"]),
            ("sanity_gate_diagnostics_check", [py, "tests/run_ci_sanity_gate_diagnostics_check.py"]),
            (
                "sanity_gate",
                [py, "tests/run_ci_sanity_gate.py", "--profile", sanity_profile, "--json-out", str(sanity_json)],
            ),
        ]
        if not args.skip_aggregate:
            steps.append(
                (
                    "aggregate_gate",
                    [
                        py,
                        "tests/run_ci_aggregate_gate.py",
                        "--report-dir",
                        str(report_dir),
                        "--report-prefix",
                        prefix,
                        "--ci-sanity-profile",
                        sanity_profile,
                        "--skip-core-tests",
                        "--fast-fail",
                        "--backup-hygiene",
                        "--clean-prefixed-reports",
                        "--quiet-success-logs",
                        "--compact-step-logs",
                        "--step-log-dir",
                        str(report_dir),
                        "--step-log-failed-only",
                        "--checklist-skip-seed-cli",
                        "--checklist-skip-ui-common",
                    ],
                )
            )

        for name, cmd in steps:
            tick = time.perf_counter()
            proc = run_step(cmd)
            elapsed_ms = int((time.perf_counter() - tick) * 1000)
            stdout = proc.stdout or ""
            stderr = proc.stderr or ""
            if stdout.strip():
                print(stdout, end="")
            if stderr.strip():
                print(stderr, end="", file=sys.stderr)
            ok = proc.returncode == 0
            rows.append(
                {
                    "name": name,
                    "ok": ok,
                    "returncode": int(proc.returncode),
                    "elapsed_ms": elapsed_ms,
                    "cmd": cmd,
                    "stdout_head": clip(stdout, 220),
                    "stderr_head": clip(stderr, 220),
                }
            )
            if not ok:
                all_ok = False
                status_code = SYNC_READINESS_STEP_FAIL
                status_step = name
                status_msg = first_message(stdout, stderr)
                break

        total_elapsed_ms = sum(int(row.get("elapsed_ms", 0)) for row in rows)
        if all_ok:
            tick = time.perf_counter()
            contract_ok, contract_msg, detected_sanity_profile, sanity_summary_fields = validate_sanity_contract(
                sanity_json,
                expected_profile=sanity_profile,
            )
            contract_elapsed_ms = int((time.perf_counter() - tick) * 1000)
            contract_row = {
                "name": "sanity_gate_contract",
                "ok": bool(contract_ok),
                "returncode": 0 if contract_ok else 1,
                "elapsed_ms": contract_elapsed_ms,
                "cmd": ["internal", "validate_sanity_contract", str(sanity_json)],
                "stdout_head": clip(contract_msg, 220),
                "stderr_head": "-" if contract_ok else clip(contract_msg, 220),
            }
            rows.append(contract_row)
            total_elapsed_ms += contract_elapsed_ms
            if not contract_ok:
                print(contract_msg, file=sys.stderr)
                all_ok = False
                status_code = SYNC_READINESS_SANITY_CONTRACT_FAIL
                status_step = "sanity_gate_contract"
                status_msg = contract_msg
    total_elapsed_ms = sum(int(row.get("elapsed_ms", 0)) for row in rows)

    payload = {
        "schema": "ddn.ci.sync_readiness.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "started_at_utc": started,
        "status": "pass" if all_ok else "fail",
        "ok": all_ok,
        "code": status_code if not all_ok else SYNC_READINESS_OK,
        "step": status_step if not all_ok else "all",
        "msg": status_msg if not all_ok else "-",
        "report_dir": str(report_dir),
        "report_prefix": prefix,
        "sanity_profile": detected_sanity_profile,
        "skip_aggregate": bool(args.skip_aggregate),
        "validate_only_sanity_json": validate_only_sanity_json,
        **sanity_summary_fields,
        "steps": rows,
        "steps_count": len(rows),
        "total_elapsed_ms": total_elapsed_ms,
    }

    out_path = Path(args.json_out) if args.json_out.strip() else (report_dir / f"{prefix}.ci_sync_readiness.detjson")
    write_json(out_path, payload)
    status = "pass" if all_ok else "fail"
    msg_json = json.dumps(clip(status_msg if not all_ok else "-", 220), ensure_ascii=False)
    print(
        f'ci_sync_readiness_status={status} ok={1 if all_ok else 0} '
        f'code={status_code if not all_ok else SYNC_READINESS_OK} '
        f'sanity_profile={detected_sanity_profile} '
        f'step={status_step if not all_ok else "all"} msg={msg_json} '
        f'steps={len(rows)} total_elapsed_ms={total_elapsed_ms} report="{out_path}"'
    )
    return 0 if all_ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
