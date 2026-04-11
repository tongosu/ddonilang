#!/usr/bin/env python
from __future__ import annotations

import argparse
from datetime import datetime
import json
import re
import sys
from pathlib import Path

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
    build_profile_matrix_snapshot_from_doc,
    build_profile_matrix_triage_payload_from_snapshot,
    profile_matrix_triage_missing_keys,
    PROFILE_MATRIX_SELFTEST_PROFILES,
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC,
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
    PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS,
    PROFILE_MATRIX_SUMMARY_VALUE_KEYS,
    expected_profile_matrix_aggregate_summary_contract,
)
from ci_check_error_codes import GATE_REPORT_INDEX_CODES as CODES

INDEX_SCHEMA = "ddn.ci.aggregate_gate.index.v1"
TOKEN_RE = re.compile(r'([A-Za-z0-9_]+)=("([^"\\]|\\.)*"|[^ \t]+)')
SUMMARY_FAILED_STEP_DETAIL_RE = re.compile(r"^failed_step_detail=([^ ]+) rc=([-]?\d+) cmd=(.+)$")
SUMMARY_FAILED_STEP_LOGS_RE = re.compile(r"^failed_step_logs=([^ ]+) stdout=([^ ]+) stderr=([^ ]+)$")

REQUIRED_REPORT_PATH_KEYS = (
    "summary",
    "summary_line",
    "final_status_parse_json",
    "ci_gate_result_json",
    "ci_gate_badge_json",
    "ci_fail_brief_txt",
    "ci_fail_triage_json",
    "ci_profile_matrix_gate_selftest",
    "ci_sanity_gate",
    "ci_sync_readiness",
    "seamgrim_wasm_cli_diag_parity",
    "fixed64_threeway_inputs",
)

ARTIFACT_SCHEMA_MAP = {
    "final_status_parse_json": (
        "ddn.ci.gate_final_status_line_parse.v1",
        "ddn.ci.status_line.parse.v1",
    ),
    "ci_gate_result_json": "ddn.ci.gate_result.v1",
    "ci_gate_badge_json": "ddn.ci.gate_badge.v1",
    "ci_fail_triage_json": "ddn.ci.fail_triage.v1",
    "ci_profile_matrix_gate_selftest": "ddn.ci.profile_matrix_gate_selftest.v1",
    "ci_sanity_gate": "ddn.ci.sanity_gate.v1",
    "ci_sync_readiness": "ddn.ci.sync_readiness.v1",
    "seamgrim_wasm_cli_diag_parity": "ddn.seamgrim.wasm_cli_diag_parity.v1",
    "fixed64_threeway_inputs": "ddn.fixed64.threeway_inputs.v1",
}

VALID_SANITY_PROFILES = PROFILE_MATRIX_SELFTEST_PROFILES
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
PROFILE_REQUIRED_STEPS_COMMON = (
    "ci_profile_split_contract_check",
    "ci_profile_matrix_gate_selftest",
    "ci_fail_and_exit_contract_selftest",
    "ci_sanity_gate",
    "ci_sync_readiness_report_generate",
    "ci_sync_readiness_report_check",
    "ci_emit_artifacts_required_post_summary_check",
    "ci_gate_report_index_selftest",
    "ci_gate_report_index_diagnostics_check",
    "ci_gate_report_index_latest_smoke_check",
)
PROFILE_REQUIRED_STEPS_CORE_LANG = ()
PROFILE_REQUIRED_STEPS_SEAMGRIM = (
    "seamgrim_ci_gate_seed_meta_step_check",
    "seamgrim_ci_gate_sam_seulgi_family_step_check",
    "seamgrim_ci_gate_runtime5_passthrough_check",
    "seamgrim_ci_gate_guideblock_step_check",
    "seamgrim_ci_gate_lesson_warning_step_check",
    "seamgrim_ci_gate_stateful_preview_step_check",
    "seamgrim_ci_gate_wasm_web_smoke_step_check",
    "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
    "seamgrim_wasm_cli_diag_parity_check",
)
SANITY_REQUIRED_PASS_STEPS_FULL_CORE_LANG = (
    "ci_pack_golden_lang_consistency_selftest",
)
SANITY_REQUIRED_PASS_STEPS_SEAMGRIM = ()
PROFILE_MATRIX_AGGREGATE_SUMMARY_KEYS = PROFILE_MATRIX_SUMMARY_VALUE_KEYS
SANITY_RUNTIME_HELPER_SUMMARY_FIELDS = (
    ("ci_sanity_pipeline_emit_flags_ok", {"full", "core_lang"}),
    ("ci_sanity_pipeline_emit_flags_selftest_ok", {"full", "core_lang"}),
    ("ci_sanity_age5_combined_heavy_policy_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok", {"full", "core_lang", "seamgrim"}),
)
SANITY_RUNTIME_HELPER_CONTRACT_FIELDS = AGE5_COMBINED_HEAVY_SANITY_CONTRACT_SUMMARY_FIELDS
SYNC_RUNTIME_HELPER_CONTRACT_FIELDS = AGE5_COMBINED_HEAVY_SYNC_CONTRACT_SUMMARY_FIELDS
VALID_AGE5_CHILD_SUMMARY_STATUS = {"pass", "fail", "skipped"}
AGE4_PROOF_OK_KEY = "age4_proof_ok"
AGE4_PROOF_FAILED_CRITERIA_KEY = "age4_proof_failed_criteria"
AGE4_PROOF_FAILED_PREVIEW_KEY = "age4_proof_failed_preview"
AGE4_PROOF_SUMMARY_HASH_KEY = "age4_proof_summary_hash"
AGE5_W107_PROGRESS_KEYS = (
    "age5_full_real_w107_golden_index_selftest_active_cases",
    "age5_full_real_w107_golden_index_selftest_inactive_cases",
    "age5_full_real_w107_golden_index_selftest_index_codes",
    "age5_full_real_w107_golden_index_selftest_current_probe",
    "age5_full_real_w107_golden_index_selftest_last_completed_probe",
    "age5_full_real_w107_golden_index_selftest_progress_present",
)
AGE5_W107_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_w107_progress_contract_selftest_completed_checks",
    "age5_full_real_w107_progress_contract_selftest_total_checks",
    "age5_full_real_w107_progress_contract_selftest_checks_text",
    "age5_full_real_w107_progress_contract_selftest_current_probe",
    "age5_full_real_w107_progress_contract_selftest_last_completed_probe",
    "age5_full_real_w107_progress_contract_selftest_progress_present",
)
AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present",
)
AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present",
)
AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present",
)
AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present",
)
AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_proof_certificate_family_contract_selftest_completed_checks",
    "age5_full_real_proof_certificate_family_contract_selftest_total_checks",
    "age5_full_real_proof_certificate_family_contract_selftest_checks_text",
    "age5_full_real_proof_certificate_family_contract_selftest_current_probe",
    "age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe",
    "age5_full_real_proof_certificate_family_contract_selftest_progress_present",
)
AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_proof_family_contract_selftest_completed_checks",
    "age5_full_real_proof_family_contract_selftest_total_checks",
    "age5_full_real_proof_family_contract_selftest_checks_text",
    "age5_full_real_proof_family_contract_selftest_current_probe",
    "age5_full_real_proof_family_contract_selftest_last_completed_probe",
    "age5_full_real_proof_family_contract_selftest_progress_present",
)
AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_proof_family_transport_contract_selftest_completed_checks",
    "age5_full_real_proof_family_transport_contract_selftest_total_checks",
    "age5_full_real_proof_family_transport_contract_selftest_checks_text",
    "age5_full_real_proof_family_transport_contract_selftest_current_probe",
    "age5_full_real_proof_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_proof_family_transport_contract_selftest_progress_present",
)
AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_lang_surface_family_contract_selftest_completed_checks",
    "age5_full_real_lang_surface_family_contract_selftest_total_checks",
    "age5_full_real_lang_surface_family_contract_selftest_checks_text",
    "age5_full_real_lang_surface_family_contract_selftest_current_probe",
    "age5_full_real_lang_surface_family_contract_selftest_last_completed_probe",
    "age5_full_real_lang_surface_family_contract_selftest_progress_present",
)
AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_lang_runtime_family_contract_selftest_completed_checks",
    "age5_full_real_lang_runtime_family_contract_selftest_total_checks",
    "age5_full_real_lang_runtime_family_contract_selftest_checks_text",
    "age5_full_real_lang_runtime_family_contract_selftest_current_probe",
    "age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe",
    "age5_full_real_lang_runtime_family_contract_selftest_progress_present",
)
AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks",
    "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks",
    "age5_full_real_lang_surface_family_transport_contract_selftest_checks_text",
    "age5_full_real_lang_surface_family_transport_contract_selftest_current_probe",
    "age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present",
)
AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present",
)
AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present",
)
AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_gate0_family_transport_contract_selftest_completed_checks",
    "age5_full_real_gate0_family_transport_contract_selftest_total_checks",
    "age5_full_real_gate0_family_transport_contract_selftest_checks_text",
    "age5_full_real_gate0_family_transport_contract_selftest_current_probe",
    "age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_family_transport_contract_selftest_progress_present",
)
AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_gate0_transport_family_contract_selftest_completed_checks",
    "age5_full_real_gate0_transport_family_contract_selftest_total_checks",
    "age5_full_real_gate0_transport_family_contract_selftest_checks_text",
    "age5_full_real_gate0_transport_family_contract_selftest_current_probe",
    "age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_transport_family_contract_selftest_progress_present",
)
AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present",
)
AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present",
)
AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present",
)
AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_bogae_alias_family_contract_selftest_completed_checks",
    "age5_full_real_bogae_alias_family_contract_selftest_total_checks",
    "age5_full_real_bogae_alias_family_contract_selftest_checks_text",
    "age5_full_real_bogae_alias_family_contract_selftest_current_probe",
    "age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe",
    "age5_full_real_bogae_alias_family_contract_selftest_progress_present",
)
AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present",
)
AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present",
)
AGE5_W107_BRIEF_KEY_MAP = (
    ("age5_full_real_w107_golden_index_selftest_active_cases", "age5_w107_active"),
    ("age5_full_real_w107_golden_index_selftest_inactive_cases", "age5_w107_inactive"),
    ("age5_full_real_w107_golden_index_selftest_index_codes", "age5_w107_index_codes"),
    ("age5_full_real_w107_golden_index_selftest_current_probe", "age5_w107_current_probe"),
    ("age5_full_real_w107_golden_index_selftest_last_completed_probe", "age5_w107_last_completed_probe"),
    ("age5_full_real_w107_golden_index_selftest_progress_present", "age5_w107_progress"),
)
AGE5_W107_CONTRACT_BRIEF_KEY_MAP = (
    ("age5_full_real_w107_progress_contract_selftest_completed_checks", "age5_w107_contract_completed"),
    ("age5_full_real_w107_progress_contract_selftest_total_checks", "age5_w107_contract_total"),
    ("age5_full_real_w107_progress_contract_selftest_checks_text", "age5_w107_contract_checks_text"),
    ("age5_full_real_w107_progress_contract_selftest_current_probe", "age5_w107_contract_current_probe"),
    ("age5_full_real_w107_progress_contract_selftest_last_completed_probe", "age5_w107_contract_last_completed_probe"),
    ("age5_full_real_w107_progress_contract_selftest_progress_present", "age5_w107_contract_progress"),
)
AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks",
        "age5_age1_immediate_proof_operation_contract_completed",
    ),
    (
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks",
        "age5_age1_immediate_proof_operation_contract_total",
    ),
    (
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text",
        "age5_age1_immediate_proof_operation_contract_checks_text",
    ),
    (
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe",
        "age5_age1_immediate_proof_operation_contract_current_probe",
    ),
    (
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe",
        "age5_age1_immediate_proof_operation_contract_last_completed_probe",
    ),
    (
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present",
        "age5_age1_immediate_proof_operation_contract_progress",
    ),
)
AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks",
        "age5_proof_certificate_v1_consumer_contract_completed",
    ),
    (
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks",
        "age5_proof_certificate_v1_consumer_contract_total",
    ),
    (
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text",
        "age5_proof_certificate_v1_consumer_contract_checks_text",
    ),
    (
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe",
        "age5_proof_certificate_v1_consumer_contract_current_probe",
    ),
    (
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe",
        "age5_proof_certificate_v1_consumer_contract_last_completed_probe",
    ),
    (
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present",
        "age5_proof_certificate_v1_consumer_contract_progress",
    ),
)
AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks",
        "age5_proof_certificate_v1_verify_report_digest_contract_completed",
    ),
    (
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks",
        "age5_proof_certificate_v1_verify_report_digest_contract_total",
    ),
    (
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text",
        "age5_proof_certificate_v1_verify_report_digest_contract_checks_text",
    ),
    (
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe",
        "age5_proof_certificate_v1_verify_report_digest_contract_current_probe",
    ),
    (
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe",
        "age5_proof_certificate_v1_verify_report_digest_contract_last_completed_probe",
    ),
    (
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present",
        "age5_proof_certificate_v1_verify_report_digest_contract_progress",
    ),
)
AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks",
        "age5_proof_certificate_v1_family_contract_completed",
    ),
    (
        "age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks",
        "age5_proof_certificate_v1_family_contract_total",
    ),
    (
        "age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text",
        "age5_proof_certificate_v1_family_contract_checks_text",
    ),
    (
        "age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe",
        "age5_proof_certificate_v1_family_contract_current_probe",
    ),
    (
        "age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe",
        "age5_proof_certificate_v1_family_contract_last_completed_probe",
    ),
    (
        "age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present",
        "age5_proof_certificate_v1_family_contract_progress",
    ),
)
AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_proof_certificate_family_contract_selftest_completed_checks",
        "age5_proof_certificate_family_contract_completed",
    ),
    (
        "age5_full_real_proof_certificate_family_contract_selftest_total_checks",
        "age5_proof_certificate_family_contract_total",
    ),
    (
        "age5_full_real_proof_certificate_family_contract_selftest_checks_text",
        "age5_proof_certificate_family_contract_checks_text",
    ),
    (
        "age5_full_real_proof_certificate_family_contract_selftest_current_probe",
        "age5_proof_certificate_family_contract_current_probe",
    ),
    (
        "age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe",
        "age5_proof_certificate_family_contract_last_completed_probe",
    ),
    (
        "age5_full_real_proof_certificate_family_contract_selftest_progress_present",
        "age5_proof_certificate_family_contract_progress",
    ),
)
AGE5_PROOF_FAMILY_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_proof_family_contract_selftest_completed_checks",
        "age5_proof_family_contract_completed",
    ),
    (
        "age5_full_real_proof_family_contract_selftest_total_checks",
        "age5_proof_family_contract_total",
    ),
    (
        "age5_full_real_proof_family_contract_selftest_checks_text",
        "age5_proof_family_contract_checks_text",
    ),
    (
        "age5_full_real_proof_family_contract_selftest_current_probe",
        "age5_proof_family_contract_current_probe",
    ),
    (
        "age5_full_real_proof_family_contract_selftest_last_completed_probe",
        "age5_proof_family_contract_last_completed_probe",
    ),
    (
        "age5_full_real_proof_family_contract_selftest_progress_present",
        "age5_proof_family_contract_progress",
    ),
)
AGE5_LANG_SURFACE_FAMILY_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_lang_surface_family_contract_selftest_completed_checks",
        "age5_lang_surface_family_contract_completed",
    ),
    (
        "age5_full_real_lang_surface_family_contract_selftest_total_checks",
        "age5_lang_surface_family_contract_total",
    ),
    (
        "age5_full_real_lang_surface_family_contract_selftest_checks_text",
        "age5_lang_surface_family_contract_checks_text",
    ),
    (
        "age5_full_real_lang_surface_family_contract_selftest_current_probe",
        "age5_lang_surface_family_contract_current_probe",
    ),
    (
        "age5_full_real_lang_surface_family_contract_selftest_last_completed_probe",
        "age5_lang_surface_family_contract_last_completed_probe",
    ),
    (
        "age5_full_real_lang_surface_family_contract_selftest_progress_present",
        "age5_lang_surface_family_contract_progress",
    ),
)
AGE5_LANG_RUNTIME_FAMILY_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_lang_runtime_family_contract_selftest_completed_checks",
        "age5_lang_runtime_family_contract_completed",
    ),
    (
        "age5_full_real_lang_runtime_family_contract_selftest_total_checks",
        "age5_lang_runtime_family_contract_total",
    ),
    (
        "age5_full_real_lang_runtime_family_contract_selftest_checks_text",
        "age5_lang_runtime_family_contract_checks_text",
    ),
    (
        "age5_full_real_lang_runtime_family_contract_selftest_current_probe",
        "age5_lang_runtime_family_contract_current_probe",
    ),
    (
        "age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe",
        "age5_lang_runtime_family_contract_last_completed_probe",
    ),
    (
        "age5_full_real_lang_runtime_family_contract_selftest_progress_present",
        "age5_lang_runtime_family_contract_progress",
    ),
)
AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks",
        "age5_lang_surface_family_transport_contract_completed",
    ),
    (
        "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks",
        "age5_lang_surface_family_transport_contract_total",
    ),
    (
        "age5_full_real_lang_surface_family_transport_contract_selftest_checks_text",
        "age5_lang_surface_family_transport_contract_checks_text",
    ),
    (
        "age5_full_real_lang_surface_family_transport_contract_selftest_current_probe",
        "age5_lang_surface_family_transport_contract_current_probe",
    ),
    (
        "age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe",
        "age5_lang_surface_family_transport_contract_last_completed_probe",
    ),
    (
        "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present",
        "age5_lang_surface_family_transport_contract_progress",
    ),
)
AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks",
        "age5_lang_runtime_family_transport_contract_completed",
    ),
    (
        "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks",
        "age5_lang_runtime_family_transport_contract_total",
    ),
    (
        "age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text",
        "age5_lang_runtime_family_transport_contract_checks_text",
    ),
    (
        "age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe",
        "age5_lang_runtime_family_transport_contract_current_probe",
    ),
    (
        "age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe",
        "age5_lang_runtime_family_transport_contract_last_completed_probe",
    ),
    (
        "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present",
        "age5_lang_runtime_family_transport_contract_progress",
    ),
)
AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks",
        "age5_gate0_runtime_family_transport_contract_completed",
    ),
    (
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks",
        "age5_gate0_runtime_family_transport_contract_total",
    ),
    (
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text",
        "age5_gate0_runtime_family_transport_contract_checks_text",
    ),
    (
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe",
        "age5_gate0_runtime_family_transport_contract_current_probe",
    ),
    (
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe",
        "age5_gate0_runtime_family_transport_contract_last_completed_probe",
    ),
    (
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present",
        "age5_gate0_runtime_family_transport_contract_progress",
    ),
)
AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_gate0_family_transport_contract_selftest_completed_checks",
        "age5_gate0_family_transport_contract_completed",
    ),
    (
        "age5_full_real_gate0_family_transport_contract_selftest_total_checks",
        "age5_gate0_family_transport_contract_total",
    ),
    (
        "age5_full_real_gate0_family_transport_contract_selftest_checks_text",
        "age5_gate0_family_transport_contract_checks_text",
    ),
    (
        "age5_full_real_gate0_family_transport_contract_selftest_current_probe",
        "age5_gate0_family_transport_contract_current_probe",
    ),
    (
        "age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe",
        "age5_gate0_family_transport_contract_last_completed_probe",
    ),
    (
        "age5_full_real_gate0_family_transport_contract_selftest_progress_present",
        "age5_gate0_family_transport_contract_progress",
    ),
)
AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_gate0_transport_family_contract_selftest_completed_checks",
        "age5_gate0_transport_family_contract_completed",
    ),
    (
        "age5_full_real_gate0_transport_family_contract_selftest_total_checks",
        "age5_gate0_transport_family_contract_total",
    ),
    (
        "age5_full_real_gate0_transport_family_contract_selftest_checks_text",
        "age5_gate0_transport_family_contract_checks_text",
    ),
    (
        "age5_full_real_gate0_transport_family_contract_selftest_current_probe",
        "age5_gate0_transport_family_contract_current_probe",
    ),
    (
        "age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe",
        "age5_gate0_transport_family_contract_last_completed_probe",
    ),
    (
        "age5_full_real_gate0_transport_family_contract_selftest_progress_present",
        "age5_gate0_transport_family_contract_progress",
    ),
)
AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks",
        "age5_gate0_surface_family_transport_contract_completed",
    ),
    (
        "age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks",
        "age5_gate0_surface_family_transport_contract_total",
    ),
    (
        "age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text",
        "age5_gate0_surface_family_transport_contract_checks_text",
    ),
    (
        "age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe",
        "age5_gate0_surface_family_transport_contract_current_probe",
    ),
    (
        "age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe",
        "age5_gate0_surface_family_transport_contract_last_completed_probe",
    ),
    (
        "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present",
        "age5_gate0_surface_family_transport_contract_progress",
    ),
)
AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_proof_family_transport_contract_selftest_completed_checks",
        "age5_proof_family_transport_contract_completed",
    ),
    (
        "age5_full_real_proof_family_transport_contract_selftest_total_checks",
        "age5_proof_family_transport_contract_total",
    ),
    (
        "age5_full_real_proof_family_transport_contract_selftest_checks_text",
        "age5_proof_family_transport_contract_checks_text",
    ),
    (
        "age5_full_real_proof_family_transport_contract_selftest_current_probe",
        "age5_proof_family_transport_contract_current_probe",
    ),
    (
        "age5_full_real_proof_family_transport_contract_selftest_last_completed_probe",
        "age5_proof_family_transport_contract_last_completed_probe",
    ),
    (
        "age5_full_real_proof_family_transport_contract_selftest_progress_present",
        "age5_proof_family_transport_contract_progress",
    ),
)
AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks",
        "age5_proof_certificate_family_transport_contract_completed",
    ),
    (
        "age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks",
        "age5_proof_certificate_family_transport_contract_total",
    ),
    (
        "age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text",
        "age5_proof_certificate_family_transport_contract_checks_text",
    ),
    (
        "age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe",
        "age5_proof_certificate_family_transport_contract_current_probe",
    ),
    (
        "age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe",
        "age5_proof_certificate_family_transport_contract_last_completed_probe",
    ),
    (
        "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present",
        "age5_proof_certificate_family_transport_contract_progress",
    ),
)
AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_bogae_alias_family_contract_selftest_completed_checks",
        "age5_bogae_alias_family_contract_completed",
    ),
    (
        "age5_full_real_bogae_alias_family_contract_selftest_total_checks",
        "age5_bogae_alias_family_contract_total",
    ),
    (
        "age5_full_real_bogae_alias_family_contract_selftest_checks_text",
        "age5_bogae_alias_family_contract_checks_text",
    ),
    (
        "age5_full_real_bogae_alias_family_contract_selftest_current_probe",
        "age5_bogae_alias_family_contract_current_probe",
    ),
    (
        "age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe",
        "age5_bogae_alias_family_contract_last_completed_probe",
    ),
    (
        "age5_full_real_bogae_alias_family_contract_selftest_progress_present",
        "age5_bogae_alias_family_contract_progress",
    ),
)
AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks",
        "age5_bogae_alias_family_transport_contract_completed",
    ),
    (
        "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks",
        "age5_bogae_alias_family_transport_contract_total",
    ),
    (
        "age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text",
        "age5_bogae_alias_family_transport_contract_checks_text",
    ),
    (
        "age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe",
        "age5_bogae_alias_family_transport_contract_current_probe",
    ),
    (
        "age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe",
        "age5_bogae_alias_family_transport_contract_last_completed_probe",
    ),
    (
        "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present",
        "age5_bogae_alias_family_transport_contract_progress",
    ),
)
AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP = (
    (
        "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks",
        "age5_bogae_alias_family_transport_contract_completed",
    ),
    (
        "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks",
        "age5_bogae_alias_family_transport_contract_total",
    ),
    (
        "age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text",
        "age5_bogae_alias_family_transport_contract_checks_text",
    ),
    (
        "age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe",
        "age5_bogae_alias_family_transport_contract_current_probe",
    ),
    (
        "age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe",
        "age5_bogae_alias_family_transport_contract_last_completed_probe",
    ),
    (
        "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present",
        "age5_bogae_alias_family_transport_contract_progress",
    ),
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
AGE5_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS = (
    build_age5_combined_heavy_child_summary_default_text_transport_fields()
)
# split-contract token anchor: combined_digest_selftest_default_field_text / combined_digest_selftest_default_field
AGE5_DIGEST_SELFTEST_DEFAULT_FIELD = build_age5_close_digest_selftest_default_field()
_JSON_DISK_CACHE: dict[str, tuple[int | None, int | None, dict | None]] = {}
_TEXT_DISK_CACHE: dict[str, tuple[int | None, int | None, str]] = {}


def _cache_key(path: Path) -> str:
    # NOTE:
    # resolve()/realpath adds significant overhead on Windows selftests
    # because it repeatedly triggers nt._getfinalpathname.
    return str(path).replace("\\", "/").lower()


def _path_sig(path: Path) -> tuple[int | None, int | None]:
    try:
        st = path.stat()
    except OSError:
        return None, None
    return st.st_mtime_ns, st.st_size


def _read_json_from_disk(path: Path) -> dict | None:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    if not isinstance(data, dict):
        return None
    return data


def _load_json_disk_cached(path: Path) -> dict | None:
    cache_key = _cache_key(path)
    mtime_ns, size = _path_sig(path)
    cached = _JSON_DISK_CACHE.get(cache_key)
    if cached is not None and cached[0] == mtime_ns and cached[1] == size:
        return cached[2]
    doc = _read_json_from_disk(path) if mtime_ns is not None else None
    _JSON_DISK_CACHE[cache_key] = (mtime_ns, size, doc)
    return doc


def _read_text_from_disk(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8").strip()
    except Exception:
        return ""


def _load_text_disk_cached(path: Path) -> str:
    cache_key = _cache_key(path)
    mtime_ns, size = _path_sig(path)
    cached = _TEXT_DISK_CACHE.get(cache_key)
    if cached is not None and cached[0] == mtime_ns and cached[1] == size:
        return cached[2]
    text = _read_text_from_disk(path) if mtime_ns is not None else ""
    _TEXT_DISK_CACHE[cache_key] = (mtime_ns, size, text)
    return text


def parse_summary(path: Path) -> tuple[str | None, dict[str, str], list[str]]:
    text = _load_text_disk_cached(path)
    lines = [line.strip() for line in text.splitlines() if line.strip()]
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


def count_summary_key(summary_lines: list[str], key: str) -> int:
    prefix = f"[ci-gate-summary] {key}="
    return sum(1 for raw_line in summary_lines if str(raw_line).strip().startswith(prefix))


def count_summary_status_markers(summary_lines: list[str]) -> int:
    markers = {"[ci-gate-summary] PASS", "[ci-gate-summary] FAIL"}
    return sum(1 for raw_line in summary_lines if str(raw_line).strip() in markers)


def first_summary_line_index(summary_lines: list[str], prefix: str) -> int:
    for idx, raw_line in enumerate(summary_lines):
        if str(raw_line).strip().startswith(prefix):
            return idx
    return -1


def first_summary_status_marker_index(summary_lines: list[str]) -> int:
    markers = {"[ci-gate-summary] PASS", "[ci-gate-summary] FAIL"}
    for idx, raw_line in enumerate(summary_lines):
        if str(raw_line).strip() in markers:
            return idx
    return -1


def fail(msg: str, code: str) -> int:
    print(f"[ci-gate-report-index-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_json(
    path: Path,
    *,
    cache: dict[Path, dict | None] | None = None,
) -> dict | None:
    if cache is not None and path in cache:
        return cache[path]
    doc = _load_json_disk_cached(path)
    if cache is not None:
        cache[path] = doc
    return doc


def resolve_report_path(index_doc: dict, key: str) -> Path | None:
    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return None
    raw = str(reports.get(key, "")).strip()
    if not raw:
        return None
    return Path(raw.replace("\\", "/"))


def load_age5_policy_snapshot(
    index_doc: dict,
    *,
    aggregate_doc: dict | None = None,
    json_cache: dict[Path, dict | None] | None = None,
) -> dict[str, object]:
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
    if not isinstance(aggregate_doc, dict):
        aggregate_path = resolve_report_path(index_doc, "aggregate")
        if aggregate_path is None:
            return snapshot
        aggregate_doc = load_json(aggregate_path, cache=json_cache)
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


def load_age4_proof_snapshot(
    index_doc: dict,
    *,
    aggregate_doc: dict | None = None,
    json_cache: dict[Path, dict | None] | None = None,
) -> dict[str, str]:
    snapshot = {
        AGE4_PROOF_OK_KEY: "0",
        AGE4_PROOF_FAILED_CRITERIA_KEY: "-1",
        AGE4_PROOF_FAILED_PREVIEW_KEY: "-",
        AGE4_PROOF_SUMMARY_HASH_KEY: "-",
    }
    if not isinstance(aggregate_doc, dict):
        aggregate_path = resolve_report_path(index_doc, "aggregate")
        if aggregate_path is None:
            return snapshot
        aggregate_doc = load_json(aggregate_path, cache=json_cache)
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


def load_age5_w107_progress_snapshot(doc: dict | None) -> dict[str, str]:
    snapshot = {
        AGE5_W107_PROGRESS_KEYS[0]: "-",
        AGE5_W107_PROGRESS_KEYS[1]: "-",
        AGE5_W107_PROGRESS_KEYS[2]: "-",
        AGE5_W107_PROGRESS_KEYS[3]: "-",
        AGE5_W107_PROGRESS_KEYS[4]: "-",
        AGE5_W107_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_w107_contract_progress_snapshot(doc: dict | None) -> dict[str, str]:
    snapshot = {
        AGE5_W107_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_age1_immediate_proof_operation_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_proof_certificate_v1_family_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_proof_certificate_family_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_proof_family_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_lang_surface_family_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_lang_runtime_family_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_lang_surface_family_transport_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_lang_runtime_family_transport_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_gate0_runtime_family_transport_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_gate0_family_transport_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_gate0_transport_family_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_gate0_transport_family_transport_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_gate0_surface_family_transport_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_proof_family_transport_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_proof_certificate_family_transport_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_bogae_alias_family_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_bogae_alias_family_transport_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_bogae_alias_family_transport_contract_progress_snapshot(
    doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(doc.get(key, fallback)).strip() or fallback
    return snapshot


def read_text(path: Path) -> str:
    return _load_text_disk_cached(path)


def normalize_path_text(raw: str) -> str:
    value = str(raw).strip()
    if not value:
        return ""
    return str(Path(value.replace("\\", "/")))


def parse_status_line_tokens(line: str) -> dict[str, str]:
    out: dict[str, str] = {}
    for part in str(line).strip().split():
        if "=" not in part:
            continue
        key, value = part.split("=", 1)
        key = key.strip()
        value = value.strip()
        if not key:
            continue
        if key in {"ci_gate_status", "ci_gate_result_status"}:
            out["status"] = value
            continue
        out[key] = value
    return out


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
            if not isinstance(value, str):
                value = str(value)
        else:
            value = raw
        if key in out:
            return None
        out[key] = value
        pos = match.end()
    if text[pos:].strip():
        return None
    return out


def parse_failed_steps_value(raw: str) -> list[str]:
    value = str(raw).strip()
    if not value or value in {"-", "(none)", "(None)"}:
        return []
    return [token.strip() for token in value.split(",") if token.strip()]


def parse_summary_failed_step_rows(
    summary_lines: list[str],
) -> tuple[
    dict[str, dict[str, object]],
    dict[str, dict[str, str]],
    list[str],
    list[str],
    list[tuple[str, str]],
    str | None,
]:
    details: dict[str, dict[str, object]] = {}
    logs: dict[str, dict[str, str]] = {}
    detail_order: list[str] = []
    log_order: list[str] = []
    step_row_sequence: list[tuple[str, str]] = []
    prefix = "[ci-gate-summary] "
    for raw_line in summary_lines:
        line = str(raw_line).strip()
        if not line.startswith(prefix):
            continue
        body = line[len(prefix) :]
        if body.startswith("failed_step_detail="):
            match = SUMMARY_FAILED_STEP_DETAIL_RE.match(body)
            if not match:
                return {}, {}, [], [], [], "summary failed_step_detail format invalid"
            step_id = str(match.group(1)).strip()
            if not step_id:
                return {}, {}, [], [], [], "summary failed_step_detail step id missing"
            if step_id in details:
                return {}, {}, [], [], [], f"summary duplicate failed_step_detail row: {step_id}"
            details[step_id] = {
                "rc": int(match.group(2)),
                "cmd": str(match.group(3)).strip(),
            }
            detail_order.append(step_id)
            step_row_sequence.append(("detail", step_id))
            continue
        if body.startswith("failed_step_logs="):
            match = SUMMARY_FAILED_STEP_LOGS_RE.match(body)
            if not match:
                return {}, {}, [], [], [], "summary failed_step_logs format invalid"
            step_id = str(match.group(1)).strip()
            if not step_id:
                return {}, {}, [], [], [], "summary failed_step_logs step id missing"
            if step_id not in details:
                return (
                    {},
                    {},
                    [],
                    [],
                    [],
                    f"summary failed_step_logs appeared before failed_step_detail: {step_id}",
                )
            if step_id in logs:
                return {}, {}, [], [], [], f"summary duplicate failed_step_logs row: {step_id}"
            stdout_path = str(match.group(2)).strip()
            stderr_path = str(match.group(3)).strip()
            logs[step_id] = {
                "stdout": "" if stdout_path == "-" else stdout_path,
                "stderr": "" if stderr_path == "-" else stderr_path,
            }
            log_order.append(step_id)
            step_row_sequence.append(("logs", step_id))
    return details, logs, detail_order, log_order, step_row_sequence, None


def is_compatible_summary_line(result_summary_line: str, expected_summary_line: str) -> bool:
    result_tokens = parse_status_line_tokens(result_summary_line)
    expected_tokens = parse_status_line_tokens(expected_summary_line)
    required_keys = ("status", "overall_ok", "failed_steps", "aggregate_status", "reason")
    for key in required_keys:
        if result_tokens.get(key, "") != expected_tokens.get(key, ""):
            return False
    return True


def resolve_profile_required_steps(profile: str) -> tuple[str, ...]:
    if profile == "core_lang":
        return PROFILE_REQUIRED_STEPS_COMMON + PROFILE_REQUIRED_STEPS_CORE_LANG
    if profile == "seamgrim":
        return PROFILE_REQUIRED_STEPS_COMMON + PROFILE_REQUIRED_STEPS_SEAMGRIM
    return PROFILE_REQUIRED_STEPS_COMMON + PROFILE_REQUIRED_STEPS_SEAMGRIM


def resolve_sanity_required_pass_steps(profile: str) -> tuple[str, ...]:
    if profile == "seamgrim":
        return SANITY_REQUIRED_PASS_STEPS_SEAMGRIM
    return SANITY_REQUIRED_PASS_STEPS_FULL_CORE_LANG


def join_names(raw: object) -> str:
    if not isinstance(raw, list):
        return "-"
    names = [str(item).strip() for item in raw if str(item).strip()]
    return ",".join(names) if names else "-"


def validate_profile_matrix_selftest(doc: dict) -> str | None:
    quick_mode = bool(doc.get("quick", False))
    timeout_defaults_text = str(doc.get("step_timeout_defaults_text", "")).strip()
    if timeout_defaults_text != PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT:
        return "profile_matrix step_timeout_defaults_text mismatch"
    timeout_defaults_sec = doc.get("step_timeout_defaults_sec")
    if not isinstance(timeout_defaults_sec, dict):
        return "profile_matrix step_timeout_defaults_sec must be object"
    timeout_env_keys = doc.get("step_timeout_env_keys")
    if not isinstance(timeout_env_keys, dict):
        return "profile_matrix step_timeout_env_keys must be object"
    for profile in VALID_SANITY_PROFILES:
        expected_timeout_default = float(PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC[profile])
        raw_timeout_default = timeout_defaults_sec.get(profile)
        try:
            timeout_default = float(raw_timeout_default)
        except Exception:
            return f"profile_matrix step_timeout_defaults_sec invalid: {profile}"
        if timeout_default != expected_timeout_default:
            return f"profile_matrix step_timeout_defaults_sec mismatch: {profile}"
        timeout_env_key = str(timeout_env_keys.get(profile, "")).strip()
        expected_timeout_env_key = str(PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS[profile]).strip()
        if timeout_env_key != expected_timeout_env_key:
            return f"profile_matrix step_timeout_env_keys mismatch: {profile}"
    selected_profiles = doc.get("selected_real_profiles")
    skipped_profiles = doc.get("skipped_real_profiles")
    if not isinstance(selected_profiles, list):
        return "profile_matrix selected_real_profiles must be list"
    if not isinstance(skipped_profiles, list):
        return "profile_matrix skipped_real_profiles must be list"
    real_profiles = doc.get("real_profiles")
    if not isinstance(real_profiles, dict):
        return "profile_matrix real_profiles must be object"
    aggregate_ok = doc.get("aggregate_summary_sanity_ok")
    if not isinstance(aggregate_ok, bool):
        return "profile_matrix aggregate_summary_sanity_ok must be bool"
    checked_profiles = doc.get("aggregate_summary_sanity_checked_profiles")
    failed_profiles = doc.get("aggregate_summary_sanity_failed_profiles")
    aggregate_skipped_profiles = doc.get("aggregate_summary_sanity_skipped_profiles")
    if not isinstance(checked_profiles, list):
        return "profile_matrix aggregate_summary_sanity_checked_profiles must be list"
    if not isinstance(failed_profiles, list):
        return "profile_matrix aggregate_summary_sanity_failed_profiles must be list"
    if not isinstance(aggregate_skipped_profiles, list):
        return "profile_matrix aggregate_summary_sanity_skipped_profiles must be list"
    aggregate_by_profile = doc.get("aggregate_summary_sanity_by_profile")
    if not isinstance(aggregate_by_profile, dict):
        return "profile_matrix aggregate_summary_sanity_by_profile must be object"
    for profile in VALID_SANITY_PROFILES:
        expected_contract = expected_profile_matrix_aggregate_summary_contract(profile)
        expected_present_contract = bool(expected_contract.get("expected_present", True))
        if quick_mode:
            expected_present_contract = False
        row = real_profiles.get(profile)
        if not isinstance(row, dict):
            return f"profile_matrix real_profiles row invalid: {profile}"
        aggregate_row = aggregate_by_profile.get(profile)
        if not isinstance(aggregate_row, dict):
            return f"profile_matrix aggregate_summary row invalid: {profile}"
        if not isinstance(aggregate_row.get("expected_present"), bool):
            return f"profile_matrix aggregate_summary expected_present invalid: {profile}"
        if bool(aggregate_row.get("expected_present", False)) != expected_present_contract:
            return f"profile_matrix aggregate_summary expected_present mismatch: {profile}"
        if str(aggregate_row.get("expected_profile", "")).strip() != str(expected_contract["expected_profile"]):
            return f"profile_matrix aggregate_summary expected_profile mismatch: {profile}"
        if str(aggregate_row.get("expected_sync_profile", "")).strip() != str(
            expected_contract["expected_sync_profile"]
        ):
            return f"profile_matrix aggregate_summary expected_sync_profile mismatch: {profile}"
        if not isinstance(aggregate_row.get("ok"), bool):
            return f"profile_matrix aggregate_summary ok invalid: {profile}"
        if not isinstance(aggregate_row.get("status"), str):
            return f"profile_matrix aggregate_summary status invalid: {profile}"
        expected_values = aggregate_row.get("expected_values")
        if not isinstance(expected_values, dict):
            return f"profile_matrix aggregate_summary expected_values invalid: {profile}"
        values = aggregate_row.get("values")
        if not isinstance(values, dict):
            return f"profile_matrix aggregate_summary values invalid: {profile}"
        expected_gate_marker = bool(expected_contract["gate_marker_expected"]) and expected_present_contract
        if bool(aggregate_row.get("gate_marker_expected", False)) != expected_gate_marker:
            return f"profile_matrix aggregate_summary gate_marker_expected mismatch: {profile}"
        if bool(aggregate_row.get("expected_present", False)):
            for key, expected in dict(expected_contract["values"]).items():
                expected_value = str(expected_values.get(key, "")).strip()
                expected_contract_value = str(expected).strip()
                if not expected_contract_value:
                    return f"profile_matrix aggregate_summary value invalid: {profile}:{key}"
                if expected_value != expected_contract_value:
                    return f"profile_matrix aggregate_summary expected_value invalid: {profile}:{key}"
                value = str(values.get(key, "")).strip()
                if value != expected_contract_value:
                    return f"profile_matrix aggregate_summary value mismatch: {profile}:{key}"
    if bool(aggregate_ok) != (join_names(failed_profiles) == "-"):
        return "profile_matrix aggregate_summary_sanity_ok mismatch"
    return None


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate aggregate gate report-index schema and report paths")
    parser.add_argument("--index", required=True, help="path to ci_gate_report_index.detjson")
    parser.add_argument(
        "--required-step",
        action="append",
        default=[],
        help="required step name in index.steps (can be repeated)",
    )
    parser.add_argument(
        "--sanity-profile",
        choices=VALID_SANITY_PROFILES,
        default="full",
        help="sanity profile for implicit required-step contract",
    )
    parser.add_argument(
        "--enforce-profile-step-contract",
        action="store_true",
        help="enforce implicit required steps by --sanity-profile",
    )
    args = parser.parse_args()

    index_path = Path(args.index)
    if not index_path.exists():
        return fail(f"missing index file: {index_path}", CODES["INDEX_MISSING"])
    json_cache: dict[Path, dict | None] = {}
    index_doc = load_json(index_path, cache=json_cache)
    if not isinstance(index_doc, dict):
        return fail(f"invalid index json: {index_path}", CODES["INDEX_JSON_INVALID"])
    if str(index_doc.get("schema", "")).strip() != INDEX_SCHEMA:
        return fail(
            f"index schema mismatch: {index_doc.get('schema')}",
            CODES["INDEX_SCHEMA"],
        )
    generated_at_utc = str(index_doc.get("generated_at_utc", "")).strip()
    if not generated_at_utc:
        return fail("index.generated_at_utc is missing", CODES["GENERATED_AT_MISSING"])
    try:
        dt_text = generated_at_utc[:-1] + "+00:00" if generated_at_utc.endswith("Z") else generated_at_utc
        datetime.fromisoformat(dt_text)
    except Exception:
        return fail(
            f"index.generated_at_utc invalid isoformat: {generated_at_utc}",
            CODES["GENERATED_AT_INVALID"],
        )
    report_dir_raw = str(index_doc.get("report_dir", "")).strip()
    if not report_dir_raw:
        return fail("index.report_dir is missing", CODES["REPORT_DIR_MISSING"])
    report_dir_path = Path(report_dir_raw.replace("\\", "/"))
    if not report_dir_path.exists():
        return fail(f"index.report_dir not found: {report_dir_path}", CODES["REPORT_DIR_NOT_FOUND"])
    report_prefix = str(index_doc.get("report_prefix", "")).strip()
    report_prefix_source = str(index_doc.get("report_prefix_source", "")).strip()
    if report_prefix:
        if not report_prefix_source:
            return fail(
                "index.report_prefix_source missing while report_prefix is set",
                CODES["REPORT_PREFIX_SOURCE_MISMATCH"],
            )
        if report_prefix_source != "arg" and not report_prefix_source.startswith("env:"):
            return fail(
                f"index.report_prefix_source invalid: {report_prefix_source}",
                CODES["REPORT_PREFIX_SOURCE_INVALID"],
            )
        if report_prefix_source.startswith("env:") and not report_prefix_source[4:].strip():
            return fail(
                f"index.report_prefix_source invalid: {report_prefix_source}",
                CODES["REPORT_PREFIX_SOURCE_INVALID"],
            )
    elif report_prefix_source:
        return fail(
            "index.report_prefix_source must be empty when report_prefix is empty",
            CODES["REPORT_PREFIX_SOURCE_MISMATCH"],
        )
    step_log_dir_raw = index_doc.get("step_log_dir", "")
    if not isinstance(step_log_dir_raw, str):
        return fail("index.step_log_dir must be string", CODES["STEP_LOG_DIR_TYPE"])
    step_log_dir = step_log_dir_raw.strip()
    if step_log_dir:
        step_log_dir_path = Path(step_log_dir.replace("\\", "/"))
        if not step_log_dir_path.exists():
            return fail(
                f"index.step_log_dir not found: {step_log_dir_path}",
                CODES["STEP_LOG_DIR_NOT_FOUND"],
            )
    step_log_failed_only = index_doc.get("step_log_failed_only")
    if not isinstance(step_log_failed_only, bool):
        return fail(
            "index.step_log_failed_only must be bool",
            CODES["STEP_LOG_FAILED_ONLY_TYPE"],
        )
    if step_log_failed_only and not step_log_dir:
        return fail(
            "index.step_log_failed_only=1 requires non-empty step_log_dir",
            CODES["STEP_LOG_CONFIG_MISMATCH"],
        )
    index_profile = str(index_doc.get("ci_sanity_profile", "")).strip()
    if index_profile not in VALID_SANITY_PROFILES:
        return fail(f"invalid ci_sanity_profile: {index_profile}", CODES["PROFILE_INVALID"])
    expected_profile = str(args.sanity_profile).strip()
    if expected_profile in VALID_SANITY_PROFILES and index_profile != expected_profile:
        return fail(
            f"ci_sanity_profile mismatch expected={expected_profile} actual={index_profile}",
            CODES["PROFILE_MISMATCH"],
        )

    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return fail("index.reports is missing", CODES["INDEX_REPORTS_MISSING"])

    steps = index_doc.get("steps")
    if steps is None:
        return fail("index.steps is missing", CODES["STEPS_MISSING"])
    if not isinstance(steps, list):
        return fail("index.steps must be list", CODES["STEPS_TYPE"])
    index_overall_ok = index_doc.get("overall_ok")
    if not isinstance(index_overall_ok, bool):
        return fail("index.overall_ok must be bool", CODES["INDEX_OVERALL_OK_TYPE"])
    seen_step_names: set[str] = set()
    step_ok_by_name: dict[str, bool] = {}
    step_rc_by_name: dict[str, int] = {}
    failed_step_count = 0
    for idx, row in enumerate(steps):
        if not isinstance(row, dict):
            return fail(f"index.steps[{idx}] must be object", CODES["STEP_ROW_TYPE"])
        step_name = str(row.get("name", "")).strip()
        if not step_name:
            return fail(f"index.steps[{idx}].name missing", CODES["STEP_NAME"])
        if step_name in seen_step_names:
            return fail(f"index.steps duplicate name: {step_name}", CODES["STEP_DUP"])
        seen_step_names.add(step_name)
        ok_value = row.get("ok")
        if not isinstance(ok_value, bool):
            return fail(f"index.steps[{idx}].ok must be bool", CODES["STEP_OK_TYPE"])
        step_ok_by_name[step_name] = bool(ok_value)
        if not bool(ok_value):
            failed_step_count += 1
        rc = None
        try:
            rc = int(row.get("returncode"))
        except Exception:
            return fail(f"index.steps[{idx}].returncode must be int", CODES["STEP_RC_TYPE"])
        step_rc_by_name[step_name] = rc
        if bool(ok_value) != (rc == 0):
            return fail(
                f"index.steps[{idx}] ok/returncode mismatch ok={ok_value} returncode={rc}",
                CODES["STEP_OK_RC_MISMATCH"],
            )
        cmd_value = row.get("cmd")
        if not isinstance(cmd_value, list):
            return fail(f"index.steps[{idx}].cmd must be list", CODES["STEP_CMD_TYPE"])
        if not cmd_value:
            return fail(f"index.steps[{idx}].cmd must not be empty", CODES["STEP_CMD_EMPTY"])
        for part in cmd_value:
            if not isinstance(part, str) or not part.strip():
                return fail(
                    f"index.steps[{idx}].cmd[*] must be non-empty string",
                    CODES["STEP_CMD_ITEM_TYPE"],
                )
    expected_index_overall_ok = failed_step_count == 0
    if index_overall_ok != expected_index_overall_ok:
        return fail(
            f"index.overall_ok mismatch expected={expected_index_overall_ok} from steps failed_step_count={failed_step_count}",
            CODES["INDEX_OVERALL_OK_STEPS_MISMATCH"],
        )

    resolved_report_paths: dict[str, Path] = {}
    for key in REQUIRED_REPORT_PATH_KEYS:
        path = resolve_report_path(index_doc, key)
        if path is None:
            return fail(f"missing index reports key/path: {key}", CODES["REPORT_KEY_MISSING"])
        if not path.exists():
            return fail(f"missing report path for {key}: {path}", CODES["REPORT_PATH_MISSING"])
        resolved_report_paths[key] = path

    aggregate_doc: dict | None = None
    aggregate_path = resolve_report_path(index_doc, "aggregate")
    if aggregate_path is not None:
        loaded_aggregate = load_json(aggregate_path, cache=json_cache)
        if isinstance(loaded_aggregate, dict):
            aggregate_doc = loaded_aggregate

    artifact_docs: dict[str, dict] = {}
    for key, expected_schema in ARTIFACT_SCHEMA_MAP.items():
        artifact_path = resolved_report_paths.get(key)
        if artifact_path is None:
            return fail(f"missing artifact path key: {key}", CODES["REPORT_KEY_MISSING"])
        artifact_doc = load_json(artifact_path, cache=json_cache)
        if not isinstance(artifact_doc, dict):
            return fail(
                f"artifact json invalid key={key} path={artifact_path}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
        actual_schema = str(artifact_doc.get("schema", "")).strip()
        if isinstance(expected_schema, tuple):
            expected_schemas = expected_schema
        else:
            expected_schemas = (expected_schema,)
        if actual_schema not in expected_schemas:
            return fail(
                "artifact schema mismatch "
                f"key={key} schema={actual_schema} expected={','.join(expected_schemas)}",
                CODES["ARTIFACT_SCHEMA_MISMATCH"],
            )
        artifact_docs[key] = artifact_doc

    final_parse_doc = artifact_docs["final_status_parse_json"]
    final_parse_parsed = final_parse_doc.get("parsed")
    if not isinstance(final_parse_parsed, dict):
        return fail("final_status_parse parsed missing", CODES["FINAL_PARSE_PARSED_MISSING"])
    final_parse_status_line_path_raw = str(final_parse_doc.get("status_line_path", "")).strip()
    if not final_parse_status_line_path_raw:
        return fail("final_status_parse status_line_path missing", CODES["FINAL_PARSE_STATUS_LINE_PATH_MISSING"])
    final_parse_status_line_path = Path(final_parse_status_line_path_raw.replace("\\", "/"))
    if not final_parse_status_line_path.exists():
        return fail(
            f"final_status_parse status_line_path not found: {final_parse_status_line_path}",
            CODES["FINAL_PARSE_STATUS_LINE_PATH_NOT_FOUND"],
        )
    expected_final_parse_status = "pass" if index_overall_ok else "fail"
    final_parse_status = str(final_parse_parsed.get("status", "")).strip()
    if final_parse_status != expected_final_parse_status:
        return fail(
            f"final_status_parse status mismatch expected={expected_final_parse_status} actual={final_parse_status}",
            CODES["FINAL_PARSE_STATUS_MISMATCH"],
        )
    final_parse_overall_ok_raw = str(final_parse_parsed.get("overall_ok", "")).strip()
    if final_parse_overall_ok_raw not in {"0", "1"}:
        return fail(
            f"final_status_parse overall_ok invalid: {final_parse_overall_ok_raw}",
            CODES["FINAL_PARSE_OVERALL_OK_INVALID"],
        )
    final_parse_overall_ok = final_parse_overall_ok_raw == "1"
    if final_parse_overall_ok != index_overall_ok:
        return fail(
            f"final_status_parse overall_ok mismatch expected={int(index_overall_ok)} actual={int(final_parse_overall_ok)}",
            CODES["FINAL_PARSE_OVERALL_OK_MISMATCH"],
        )
    final_parse_aggregate_status = str(final_parse_parsed.get("aggregate_status", "")).strip()
    if final_parse_aggregate_status not in {"pass", "fail"}:
        return fail(
            f"final_status_parse aggregate_status invalid: {final_parse_aggregate_status}",
            CODES["FINAL_PARSE_AGGREGATE_STATUS_INVALID"],
        )
    final_parse_failed_steps_raw = str(final_parse_parsed.get("failed_steps", "")).strip()
    try:
        final_parse_failed_steps = int(final_parse_failed_steps_raw)
    except Exception:
        return fail(
            "final_status_parse failed_steps must be int string",
            CODES["FINAL_PARSE_FAILED_STEPS_TYPE"],
        )
    if final_parse_failed_steps != failed_step_count:
        return fail(
            f"final_status_parse failed_steps mismatch expected={failed_step_count} actual={final_parse_failed_steps}",
            CODES["FINAL_PARSE_FAILED_STEPS_MISMATCH"],
        )
    age4_proof_snapshot = load_age4_proof_snapshot(
        index_doc,
        aggregate_doc=aggregate_doc,
        json_cache=json_cache,
    )
    final_parse_age4_proof_ok = str(final_parse_parsed.get(AGE4_PROOF_OK_KEY, "")).strip()
    if final_parse_age4_proof_ok not in {"0", "1"}:
        return fail(
            f"final_status_parse {AGE4_PROOF_OK_KEY} invalid: {final_parse_age4_proof_ok}",
            CODES["FINAL_PARSE_OVERALL_OK_INVALID"],
        )
    if final_parse_age4_proof_ok != age4_proof_snapshot[AGE4_PROOF_OK_KEY]:
        return fail(
            f"final_status_parse {AGE4_PROOF_OK_KEY} mismatch expected={age4_proof_snapshot[AGE4_PROOF_OK_KEY]} actual={final_parse_age4_proof_ok}",
            CODES["FINAL_PARSE_OVERALL_OK_MISMATCH"],
        )
    final_parse_age4_proof_failed_raw = str(final_parse_parsed.get(AGE4_PROOF_FAILED_CRITERIA_KEY, "")).strip()
    try:
        final_parse_age4_proof_failed = int(final_parse_age4_proof_failed_raw)
    except Exception:
        return fail(
            f"final_status_parse {AGE4_PROOF_FAILED_CRITERIA_KEY} must be int string",
            CODES["FINAL_PARSE_FAILED_STEPS_TYPE"],
        )
    if final_parse_age4_proof_failed != int(age4_proof_snapshot[AGE4_PROOF_FAILED_CRITERIA_KEY]):
        return fail(
            f"final_status_parse {AGE4_PROOF_FAILED_CRITERIA_KEY} mismatch expected={age4_proof_snapshot[AGE4_PROOF_FAILED_CRITERIA_KEY]} actual={final_parse_age4_proof_failed}",
            CODES["FINAL_PARSE_FAILED_STEPS_MISMATCH"],
        )
    final_parse_age4_proof_preview = str(final_parse_parsed.get(AGE4_PROOF_FAILED_PREVIEW_KEY, "")).strip()
    if not final_parse_age4_proof_preview:
        return fail(
            f"final_status_parse {AGE4_PROOF_FAILED_PREVIEW_KEY} missing",
            CODES["FINAL_PARSE_FAILED_STEPS_MISMATCH"],
        )
    if final_parse_age4_proof_preview != age4_proof_snapshot[AGE4_PROOF_FAILED_PREVIEW_KEY]:
        return fail(
            f"final_status_parse {AGE4_PROOF_FAILED_PREVIEW_KEY} mismatch expected={age4_proof_snapshot[AGE4_PROOF_FAILED_PREVIEW_KEY]} actual={final_parse_age4_proof_preview}",
            CODES["FINAL_PARSE_FAILED_STEPS_MISMATCH"],
        )
    final_parse_age5_w107_progress = load_age5_w107_progress_snapshot(final_parse_parsed)
    final_parse_age5_w107_contract_progress = load_age5_w107_contract_progress_snapshot(final_parse_parsed)
    final_parse_age5_age1_immediate_proof_operation_contract_progress = (
        load_age5_age1_immediate_proof_operation_contract_progress_snapshot(final_parse_parsed)
    )
    final_parse_age5_proof_certificate_v1_consumer_transport_contract_progress = (
        load_age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot(final_parse_parsed)
    )
    final_parse_age5_proof_certificate_v1_verify_report_digest_contract_progress = (
        load_age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot(
            final_parse_parsed
        )
    )
    final_parse_age5_proof_certificate_v1_family_contract_progress = (
        load_age5_proof_certificate_v1_family_contract_progress_snapshot(final_parse_parsed)
    )
    final_parse_age5_proof_certificate_family_contract_progress = (
        load_age5_proof_certificate_family_contract_progress_snapshot(final_parse_parsed)
    )
    final_parse_age5_proof_family_contract_progress = (
        load_age5_proof_family_contract_progress_snapshot(final_parse_parsed)
    )
    final_parse_age5_lang_surface_family_contract_progress = (
        load_age5_lang_surface_family_contract_progress_snapshot(final_parse_parsed)
    )
    final_parse_age5_lang_runtime_family_contract_progress = (
        load_age5_lang_runtime_family_contract_progress_snapshot(final_parse_parsed)
    )
    final_parse_age5_lang_runtime_family_transport_contract_progress = (
        load_age5_lang_runtime_family_transport_contract_progress_snapshot(final_parse_parsed)
    )
    final_parse_age5_gate0_family_transport_contract_progress = (
        load_age5_gate0_family_transport_contract_progress_snapshot(final_parse_parsed)
    )
    final_parse_age5_gate0_transport_family_contract_progress = (
        load_age5_gate0_transport_family_contract_progress_snapshot(final_parse_parsed)
    )
    final_parse_age5_gate0_transport_family_transport_contract_progress = (
        load_age5_gate0_transport_family_transport_contract_progress_snapshot(final_parse_parsed)
    )
    final_parse_age5_gate0_surface_family_transport_contract_progress = (
        load_age5_gate0_surface_family_transport_contract_progress_snapshot(final_parse_parsed)
    )
    final_parse_age5_gate0_runtime_family_transport_contract_progress = (
        load_age5_gate0_runtime_family_transport_contract_progress_snapshot(final_parse_parsed)
    )
    final_parse_age5_lang_surface_family_transport_contract_progress = (
        load_age5_lang_surface_family_transport_contract_progress_snapshot(final_parse_parsed)
    )
    final_parse_age5_proof_family_transport_contract_progress = (
        load_age5_proof_family_transport_contract_progress_snapshot(final_parse_parsed)
    )
    final_parse_age5_proof_certificate_family_transport_contract_progress = (
        load_age5_proof_certificate_family_transport_contract_progress_snapshot(final_parse_parsed)
    )
    final_parse_age5_bogae_alias_family_contract_progress = (
        load_age5_bogae_alias_family_contract_progress_snapshot(final_parse_parsed)
    )
    final_parse_age5_bogae_alias_family_transport_contract_progress = (
        load_age5_bogae_alias_family_transport_contract_progress_snapshot(final_parse_parsed)
    )
    if final_parse_age5_w107_progress[AGE5_W107_PROGRESS_KEYS[5]] not in {"0", "1"}:
        return fail(
            "final_status_parse age5_full_real_w107_golden_index_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_W107_PROGRESS_KEYS[:3]:
        value = final_parse_age5_w107_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if final_parse_age5_w107_contract_progress[AGE5_W107_CONTRACT_PROGRESS_KEYS[5]] not in {"0", "1"}:
        return fail(
            "final_status_parse age5_full_real_w107_progress_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_W107_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_w107_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        final_parse_age5_age1_immediate_proof_operation_contract_progress[
            AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "final_status_parse age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_age1_immediate_proof_operation_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        final_parse_age5_proof_certificate_v1_consumer_transport_contract_progress[
            AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "final_status_parse age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_proof_certificate_v1_consumer_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        final_parse_age5_proof_certificate_v1_verify_report_digest_contract_progress[
            AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "final_status_parse age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_proof_certificate_v1_verify_report_digest_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        final_parse_age5_proof_certificate_v1_family_contract_progress[
            AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "final_status_parse age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_proof_certificate_v1_family_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        final_parse_age5_proof_certificate_family_contract_progress[
            AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "final_status_parse age5_full_real_proof_certificate_family_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_proof_certificate_family_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        final_parse_age5_proof_family_contract_progress[
            AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "final_status_parse age5_full_real_proof_family_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_proof_family_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        final_parse_age5_lang_surface_family_contract_progress[
            AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "final_status_parse age5_full_real_lang_surface_family_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_lang_surface_family_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        final_parse_age5_lang_runtime_family_contract_progress[
            AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "final_status_parse age5_full_real_lang_runtime_family_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_lang_runtime_family_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        final_parse_age5_gate0_family_transport_contract_progress[
            AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "final_status_parse age5_full_real_gate0_family_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_gate0_family_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        final_parse_age5_gate0_transport_family_contract_progress[
            AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "final_status_parse age5_full_real_gate0_transport_family_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_gate0_transport_family_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        final_parse_age5_gate0_transport_family_transport_contract_progress[
            AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "final_status_parse age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_gate0_transport_family_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        final_parse_age5_gate0_surface_family_transport_contract_progress[
            AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "final_status_parse age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_gate0_surface_family_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        final_parse_age5_gate0_runtime_family_transport_contract_progress[
            AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "final_status_parse age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_gate0_runtime_family_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        final_parse_age5_lang_surface_family_transport_contract_progress[
            AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "final_status_parse age5_full_real_lang_surface_family_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_lang_surface_family_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        final_parse_age5_proof_family_transport_contract_progress[
            AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "final_status_parse age5_full_real_proof_family_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_proof_family_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        final_parse_age5_proof_certificate_family_transport_contract_progress[
            AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "final_status_parse age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_proof_certificate_family_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if final_parse_age5_bogae_alias_family_contract_progress[
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5]
    ] not in {"0", "1"}:
        return fail(
            "final_status_parse age5_full_real_bogae_alias_family_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_bogae_alias_family_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if final_parse_age5_bogae_alias_family_transport_contract_progress[
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
    ] not in {"0", "1"}:
        return fail(
            "final_status_parse age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = final_parse_age5_bogae_alias_family_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"final_status_parse {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )

    sanity_profile = str(artifact_docs["ci_sanity_gate"].get("profile", "")).strip()
    if sanity_profile not in VALID_SANITY_PROFILES:
        return fail(f"invalid sanity profile in ci_sanity_gate: {sanity_profile}", CODES["SANITY_PROFILE_INVALID"])
    if sanity_profile != index_profile:
        return fail(
            f"ci_sanity_gate profile mismatch index={index_profile} actual={sanity_profile}",
            CODES["SANITY_PROFILE_MISMATCH"],
        )
    sanity_steps_raw = artifact_docs["ci_sanity_gate"].get("steps")
    if not isinstance(sanity_steps_raw, list):
        return fail("ci_sanity_gate steps must be list", CODES["ARTIFACT_JSON_INVALID"])
    sanity_step_index: dict[str, dict] = {}
    for row in sanity_steps_raw:
        if not isinstance(row, dict):
            continue
        row_step = str(row.get("step", "")).strip()
        if row_step:
            sanity_step_index[row_step] = row
    for required_step in resolve_sanity_required_pass_steps(sanity_profile):
        row = sanity_step_index.get(required_step)
        if row is None:
            return fail(
                f"ci_sanity_gate required pass step missing: {required_step}",
                CODES["REQUIRED_STEP_MISSING"],
            )
        if not bool(row.get("ok", False)):
            return fail(
                f"ci_sanity_gate required pass step not ok: {required_step}",
                CODES["REQUIRED_STEP_MISSING"],
            )
        try:
            row_rc = int(row.get("returncode", -1))
        except Exception:
            row_rc = -1
        if row_rc != 0:
            return fail(
                f"ci_sanity_gate required pass step rc!=0: {required_step} rc={row.get('returncode')}",
                CODES["REQUIRED_STEP_MISSING"],
            )

    sync_profile = str(artifact_docs["ci_sync_readiness"].get("sanity_profile", "")).strip()
    if sync_profile not in VALID_SANITY_PROFILES:
        return fail(
            f"invalid sanity_profile in ci_sync_readiness: {sync_profile}",
            CODES["SYNC_PROFILE_INVALID"],
        )
    if sync_profile != index_profile:
        return fail(
            f"ci_sync_readiness sanity_profile mismatch index={index_profile} actual={sync_profile}",
            CODES["SYNC_PROFILE_MISMATCH"],
        )
    summary_path = resolved_report_paths["summary"]
    if not summary_path.exists():
        return fail(f"missing summary file: {summary_path}", CODES["SUMMARY_MISSING"])
    summary_status, summary_kv, summary_lines = parse_summary(summary_path)
    if not summary_lines:
        return fail("summary file is empty", CODES["SUMMARY_EMPTY"])
    if count_summary_status_markers(summary_lines) != 1:
        return fail("summary status marker must appear exactly once", CODES["SUMMARY_STATUS_MISMATCH"])
    first_summary_idx = first_summary_line_index(summary_lines, "[ci-gate-summary] ")
    first_status_idx = first_summary_status_marker_index(summary_lines)
    if first_summary_idx < 0 or first_status_idx < 0 or first_status_idx != first_summary_idx:
        return fail("summary status marker must be the first summary line", CODES["SUMMARY_STATUS_MISMATCH"])
    if count_summary_key(summary_lines, "failed_steps") != 1:
        return fail("summary failed_steps key must appear exactly once", CODES["SUMMARY_VALUE_INVALID"])
    if summary_status not in {"pass", "fail"}:
        return fail("summary status missing or invalid", CODES["SUMMARY_STATUS_MISMATCH"])
    if summary_status == "fail":
        failed_steps_line_idx = first_summary_line_index(summary_lines, "[ci-gate-summary] failed_steps=")
        first_detail_idx = first_summary_line_index(summary_lines, "[ci-gate-summary] failed_step_detail=")
        first_logs_idx = first_summary_line_index(summary_lines, "[ci-gate-summary] failed_step_logs=")
        first_step_row_idx = -1
        if first_detail_idx >= 0 and first_logs_idx >= 0:
            first_step_row_idx = min(first_detail_idx, first_logs_idx)
        elif first_detail_idx >= 0:
            first_step_row_idx = first_detail_idx
        elif first_logs_idx >= 0:
            first_step_row_idx = first_logs_idx
        if first_step_row_idx >= 0 and failed_steps_line_idx > first_step_row_idx:
            return fail(
                "summary failed_steps must appear before failed_step_detail/failed_step_logs rows",
                CODES["SUMMARY_VALUE_INVALID"],
            )
    sanity_doc = artifact_docs["ci_sanity_gate"]
    sync_doc = artifact_docs["ci_sync_readiness"]
    for key, valid_profiles in SANITY_RUNTIME_HELPER_SUMMARY_FIELDS:
        sanity_value = str(sanity_doc.get(key, "")).strip()
        sync_value = str(sync_doc.get(key, "")).strip()
        if sanity_value not in {"1", "na"}:
            return fail(f"invalid ci_sanity_gate summary value {key}={sanity_value}", CODES["ARTIFACT_JSON_INVALID"])
        if sync_value not in {"1", "na"}:
            return fail(f"invalid ci_sync_readiness summary value {key}={sync_value}", CODES["ARTIFACT_JSON_INVALID"])
        expected_value = "1" if index_profile in valid_profiles else "na"
        if sanity_value != expected_value:
            return fail(
                f"ci_sanity_gate summary mismatch {key} expected={expected_value} actual={sanity_value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
        if sync_value != expected_value:
            return fail(
                f"ci_sync_readiness summary mismatch {key} expected={expected_value} actual={sync_value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
        if sync_value != sanity_value:
            return fail(
                f"ci_sync_readiness/ci_sanity_gate summary mismatch {key} sync={sync_value} sanity={sanity_value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key, expected_value in SANITY_RUNTIME_HELPER_CONTRACT_FIELDS:
        sanity_value = str(sanity_doc.get(key, "")).strip()
        summary_value = str(summary_kv.get(key, "")).strip()
        if sanity_value != expected_value:
            return fail(
                f"ci_sanity_gate contract mismatch {key} expected={expected_value} actual={sanity_value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
        if summary_value != expected_value:
            return fail(
                f"summary contract mismatch {key} expected={expected_value} actual={summary_value}",
                CODES["SUMMARY_VALUE_INVALID"],
            )
    for key, expected_value in SYNC_RUNTIME_HELPER_CONTRACT_FIELDS:
        sync_value = str(sync_doc.get(key, "")).strip()
        summary_value = str(summary_kv.get(key, "")).strip()
        if sync_value != expected_value:
            return fail(
                f"ci_sync_readiness contract mismatch {key} expected={expected_value} actual={sync_value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
        if summary_value != expected_value:
            return fail(
                f"summary contract mismatch {key} expected={expected_value} actual={summary_value}",
                CODES["SUMMARY_VALUE_INVALID"],
            )

    result_doc = artifact_docs["ci_gate_result_json"]
    result_ok = result_doc.get("ok")
    if not isinstance(result_ok, bool):
        return fail("ci_gate_result ok must be bool", CODES["RESULT_OK_TYPE"])
    result_overall_ok = result_doc.get("overall_ok")
    if not isinstance(result_overall_ok, bool):
        return fail("ci_gate_result overall_ok must be bool", CODES["RESULT_OVERALL_OK_TYPE"])
    if result_overall_ok != index_overall_ok:
        return fail(
            f"ci_gate_result overall_ok mismatch index={index_overall_ok} actual={result_overall_ok}",
            CODES["RESULT_OVERALL_OK_MISMATCH"],
        )
    result_failed_steps_raw = result_doc.get("failed_steps")
    if not isinstance(result_failed_steps_raw, int) or isinstance(result_failed_steps_raw, bool):
        return fail("ci_gate_result failed_steps must be int", CODES["RESULT_FAILED_STEPS_TYPE"])
    result_failed_steps = int(result_failed_steps_raw)
    if result_failed_steps != failed_step_count:
        return fail(
            f"ci_gate_result failed_steps mismatch expected={failed_step_count} actual={result_failed_steps}",
            CODES["RESULT_FAILED_STEPS_MISMATCH"],
        )
    result_status = str(result_doc.get("status", "")).strip()
    expected_result_status = "pass" if index_overall_ok else "fail"
    if result_status != expected_result_status:
        return fail(
            f"ci_gate_result status mismatch expected={expected_result_status} actual={result_status}",
            CODES["RESULT_STATUS_MISMATCH"],
        )
    if summary_status != result_status:
        return fail(
            f"summary status mismatch expected={result_status} actual={summary_status}",
            CODES["SUMMARY_STATUS_MISMATCH"],
        )
    if summary_status == "pass":
        for key in SEAMGRIM_FOCUS_SUMMARY_REQUIRED_KEYS:
            value = str(summary_kv.get(key, "")).strip()
            if not value:
                return fail(f"summary missing key: {key}", CODES["SUMMARY_KEY_MISSING"])
            if value not in VALID_SEAMGRIM_FOCUS_SUMMARY_STATUS:
                return fail(f"summary invalid {key}: {value}", CODES["SUMMARY_VALUE_INVALID"])
        if str(summary_kv.get("seamgrim_group_id_summary_status", "")).strip() != "ok":
            return fail(
                "summary requires seamgrim_group_id_summary_status=ok in pass report",
                CODES["SUMMARY_STATUS_MISMATCH"],
            )
        for key in AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS:
            summary_value = str(summary_kv.get(key, "")).strip()
            if not summary_value:
                return fail(f"summary missing key: {key}", CODES["SUMMARY_KEY_MISSING"])
            if summary_value not in VALID_AGE5_CHILD_SUMMARY_STATUS:
                return fail(f"summary invalid {key}: {summary_value}", CODES["SUMMARY_VALUE_INVALID"])
        age5_close_digest_selftest_ok = str(summary_kv.get("age5_close_digest_selftest_ok", "")).strip()
        if age5_close_digest_selftest_ok not in {"0", "1"}:
            return fail(
                f"summary invalid age5_close_digest_selftest_ok: {age5_close_digest_selftest_ok}",
                CODES["SUMMARY_VALUE_INVALID"],
            )
        if age5_close_digest_selftest_ok != "1":
            return fail(
                "summary requires age5_close_digest_selftest_ok=1 in pass report",
                CODES["SUMMARY_STATUS_MISMATCH"],
            )
        if "age5_close_digest_selftest" not in step_ok_by_name:
            return fail(
                "index.steps missing age5_close_digest_selftest",
                CODES["REQUIRED_STEP_MISSING"],
            )
        if int(age5_close_digest_selftest_ok) != int(step_ok_by_name["age5_close_digest_selftest"]):
            return fail(
                "summary/index mismatch for age5_close_digest_selftest_ok",
                CODES["SUMMARY_STATUS_MISMATCH"],
            )
        age_close_values = {
            summary_key: str(summary_kv.get(summary_key, "")).strip()
            for summary_key, _report_key, _expected_schema in AGE_CLOSE_STATUS_SUMMARY_SPECS
        }
        age_close_missing = [key for key, value in age_close_values.items() if not value]
        # preview summary에서는 age close status 키가 아직 채워지지 않을 수 있다.
        # 단, 일부만 누락되면 계약 위반으로 본다.
        age_close_preview_mode = bool(age_close_missing) and len(age_close_missing) == len(AGE_CLOSE_STATUS_SUMMARY_SPECS)
        if age_close_missing and not age_close_preview_mode:
            return fail(
                "summary age close status keys are partially missing: " + ",".join(age_close_missing),
                CODES["SUMMARY_KEY_MISSING"],
            )
        if not age_close_preview_mode:
            for summary_key, report_key, expected_schema in AGE_CLOSE_STATUS_SUMMARY_SPECS:
                summary_value_norm = normalize_path_text(age_close_values[summary_key])
                report_path = resolve_report_path(index_doc, report_key)
                if report_path is None:
                    return fail(f"missing index reports key/path: {report_key}", CODES["REPORT_KEY_MISSING"])
                report_path_norm = normalize_path_text(str(report_path))
                if summary_value_norm != report_path_norm:
                    return fail(
                        f"summary/{summary_key} mismatch summary={summary_value_norm} report={report_path_norm}",
                        CODES["SUMMARY_VALUE_INVALID"],
                    )
                if not report_path.exists():
                    return fail(f"missing report path for {report_key}: {report_path}", CODES["REPORT_PATH_MISSING"])
                report_doc = load_json(report_path, cache=json_cache)
                if not isinstance(report_doc, dict):
                    return fail(
                        f"artifact json invalid key={report_key} path={report_path}",
                        CODES["ARTIFACT_JSON_INVALID"],
                    )
                actual_schema = str(report_doc.get("schema", "")).strip()
                if actual_schema != expected_schema:
                    return fail(
                        f"artifact schema mismatch key={report_key} schema={actual_schema} expected={expected_schema}",
                        CODES["ARTIFACT_SCHEMA_MISMATCH"],
                    )
    result_aggregate_status = str(result_doc.get("aggregate_status", "")).strip()
    if result_aggregate_status not in {"pass", "fail"}:
        return fail(
            f"ci_gate_result aggregate_status invalid: {result_aggregate_status}",
            CODES["RESULT_AGGREGATE_STATUS_INVALID"],
        )
    if result_aggregate_status != final_parse_aggregate_status:
        return fail(
            f"ci_gate_result aggregate_status mismatch expected={final_parse_aggregate_status} actual={result_aggregate_status}",
            CODES["RESULT_AGGREGATE_STATUS_MISMATCH"],
        )
    expected_result_ok = (
        result_status == "pass"
        and result_overall_ok
        and result_aggregate_status == "pass"
        and result_failed_steps == 0
    )
    if result_ok != expected_result_ok:
        return fail(
            f"ci_gate_result ok mismatch expected={int(expected_result_ok)} actual={int(result_ok)}",
            CODES["RESULT_OK_MISMATCH"],
        )
    result_summary_line_path = normalize_path_text(str(result_doc.get("summary_line_path", "")).strip())
    expected_summary_line_path = str(resolved_report_paths["summary_line"])
    if result_summary_line_path != expected_summary_line_path:
        return fail(
            f"ci_gate_result summary_line_path mismatch expected={expected_summary_line_path} actual={result_summary_line_path}",
            CODES["RESULT_SUMMARY_LINE_PATH_MISMATCH"],
        )
    expected_summary_line = read_text(resolved_report_paths["summary_line"])
    result_summary_line = str(result_doc.get("summary_line", "")).strip()
    if result_summary_line != expected_summary_line and not is_compatible_summary_line(
        result_summary_line,
        expected_summary_line,
    ):
        return fail(
            "ci_gate_result summary_line mismatch",
            CODES["RESULT_SUMMARY_LINE_MISMATCH"],
        )
    result_gate_index_path = normalize_path_text(str(result_doc.get("gate_index_path", "")).strip())
    expected_gate_index_path = str(index_path)
    if result_gate_index_path != expected_gate_index_path:
        return fail(
            f"ci_gate_result gate_index_path mismatch expected={expected_gate_index_path} actual={result_gate_index_path}",
            CODES["RESULT_GATE_INDEX_PATH_MISMATCH"],
        )
    result_final_status_parse_path = normalize_path_text(str(result_doc.get("final_status_parse_path", "")).strip())
    expected_final_status_parse_path = str(resolved_report_paths["final_status_parse_json"])
    if result_final_status_parse_path != expected_final_status_parse_path:
        return fail(
            "ci_gate_result final_status_parse_path mismatch",
            CODES["RESULT_FINAL_STATUS_PARSE_PATH_MISMATCH"],
        )
    result_age5_w107_progress = load_age5_w107_progress_snapshot(result_doc)
    result_age5_w107_contract_progress = load_age5_w107_contract_progress_snapshot(result_doc)
    result_age5_age1_immediate_proof_operation_contract_progress = (
        load_age5_age1_immediate_proof_operation_contract_progress_snapshot(result_doc)
    )
    result_age5_proof_certificate_v1_consumer_transport_contract_progress = (
        load_age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot(result_doc)
    )
    result_age5_proof_certificate_v1_verify_report_digest_contract_progress = (
        load_age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot(result_doc)
    )
    result_age5_proof_certificate_v1_family_contract_progress = (
        load_age5_proof_certificate_v1_family_contract_progress_snapshot(result_doc)
    )
    result_age5_proof_certificate_family_contract_progress = (
        load_age5_proof_certificate_family_contract_progress_snapshot(result_doc)
    )
    result_age5_proof_family_contract_progress = (
        load_age5_proof_family_contract_progress_snapshot(result_doc)
    )
    result_age5_lang_surface_family_contract_progress = (
        load_age5_lang_surface_family_contract_progress_snapshot(result_doc)
    )
    result_age5_lang_runtime_family_contract_progress = (
        load_age5_lang_runtime_family_contract_progress_snapshot(result_doc)
    )
    result_age5_lang_runtime_family_transport_contract_progress = (
        load_age5_lang_runtime_family_transport_contract_progress_snapshot(result_doc)
    )
    result_age5_gate0_family_transport_contract_progress = (
        load_age5_gate0_family_transport_contract_progress_snapshot(result_doc)
    )
    result_age5_gate0_transport_family_contract_progress = (
        load_age5_gate0_transport_family_contract_progress_snapshot(result_doc)
    )
    result_age5_gate0_transport_family_transport_contract_progress = (
        load_age5_gate0_transport_family_transport_contract_progress_snapshot(result_doc)
    )
    result_age5_gate0_surface_family_transport_contract_progress = (
        load_age5_gate0_surface_family_transport_contract_progress_snapshot(result_doc)
    )
    result_age5_gate0_runtime_family_transport_contract_progress = (
        load_age5_gate0_runtime_family_transport_contract_progress_snapshot(result_doc)
    )
    result_age5_lang_surface_family_transport_contract_progress = (
        load_age5_lang_surface_family_transport_contract_progress_snapshot(result_doc)
    )
    result_age5_proof_family_transport_contract_progress = (
        load_age5_proof_family_transport_contract_progress_snapshot(result_doc)
    )
    result_age5_proof_certificate_family_transport_contract_progress = (
        load_age5_proof_certificate_family_transport_contract_progress_snapshot(result_doc)
    )
    result_age5_bogae_alias_family_contract_progress = (
        load_age5_bogae_alias_family_contract_progress_snapshot(result_doc)
    )
    result_age5_bogae_alias_family_transport_contract_progress = (
        load_age5_bogae_alias_family_transport_contract_progress_snapshot(result_doc)
    )
    if result_age5_w107_progress[AGE5_W107_PROGRESS_KEYS[5]] not in {"0", "1"}:
        return fail(
            "ci_gate_result age5_full_real_w107_golden_index_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_W107_PROGRESS_KEYS[:3]:
        value = result_age5_w107_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if result_age5_w107_contract_progress[AGE5_W107_CONTRACT_PROGRESS_KEYS[5]] not in {"0", "1"}:
        return fail(
            "ci_gate_result age5_full_real_w107_progress_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_W107_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_w107_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        result_age5_age1_immediate_proof_operation_contract_progress[
            AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "ci_gate_result age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_age1_immediate_proof_operation_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        result_age5_proof_certificate_v1_consumer_transport_contract_progress[
            AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "ci_gate_result age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_proof_certificate_v1_consumer_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        result_age5_proof_certificate_v1_verify_report_digest_contract_progress[
            AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "ci_gate_result age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_proof_certificate_v1_verify_report_digest_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        result_age5_proof_certificate_v1_family_contract_progress[
            AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "ci_gate_result age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_proof_certificate_v1_family_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        result_age5_proof_certificate_family_contract_progress[
            AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "ci_gate_result age5_full_real_proof_certificate_family_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_proof_certificate_family_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        result_age5_proof_family_contract_progress[
            AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "ci_gate_result age5_full_real_proof_family_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_proof_family_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        result_age5_lang_surface_family_contract_progress[
            AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "ci_gate_result age5_full_real_lang_surface_family_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_lang_surface_family_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        result_age5_lang_runtime_family_contract_progress[
            AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "ci_gate_result age5_full_real_lang_runtime_family_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_lang_runtime_family_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        result_age5_gate0_family_transport_contract_progress[
            AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "ci_gate_result age5_full_real_gate0_family_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_gate0_family_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        result_age5_gate0_transport_family_contract_progress[
            AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "ci_gate_result age5_full_real_gate0_transport_family_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_gate0_transport_family_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        result_age5_gate0_transport_family_transport_contract_progress[
            AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "ci_gate_result age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_gate0_transport_family_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        result_age5_gate0_surface_family_transport_contract_progress[
            AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "ci_gate_result age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_gate0_surface_family_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        result_age5_gate0_runtime_family_transport_contract_progress[
            AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "ci_gate_result age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_gate0_runtime_family_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        result_age5_lang_surface_family_transport_contract_progress[
            AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "ci_gate_result age5_full_real_lang_surface_family_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_lang_surface_family_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        result_age5_proof_family_transport_contract_progress[
            AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "ci_gate_result age5_full_real_proof_family_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_proof_family_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if (
        result_age5_proof_certificate_family_transport_contract_progress[
            AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
        ]
        not in {"0", "1"}
    ):
        return fail(
            "ci_gate_result age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_proof_certificate_family_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if result_age5_bogae_alias_family_contract_progress[
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5]
    ] not in {"0", "1"}:
        return fail(
            "ci_gate_result age5_full_real_bogae_alias_family_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_bogae_alias_family_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    if result_age5_bogae_alias_family_transport_contract_progress[
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]
    ] not in {"0", "1"}:
        return fail(
            "ci_gate_result age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present invalid",
            CODES["ARTIFACT_JSON_INVALID"],
        )
    for key in AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = result_age5_bogae_alias_family_transport_contract_progress[key]
        if value == "-":
            continue
        try:
            int(value)
        except Exception:
            return fail(
                f"ci_gate_result {key} invalid: {value}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_W107_PROGRESS_KEYS:
        if result_age5_w107_progress[key] != final_parse_age5_w107_progress[key]:
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_w107_progress[key]} final={final_parse_age5_w107_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_W107_CONTRACT_PROGRESS_KEYS:
        if result_age5_w107_contract_progress[key] != final_parse_age5_w107_contract_progress[key]:
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_w107_contract_progress[key]} final={final_parse_age5_w107_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_age1_immediate_proof_operation_contract_progress[key]
            != final_parse_age5_age1_immediate_proof_operation_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_age1_immediate_proof_operation_contract_progress[key]} final={final_parse_age5_age1_immediate_proof_operation_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_proof_certificate_v1_consumer_transport_contract_progress[key]
            != final_parse_age5_proof_certificate_v1_consumer_transport_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_proof_certificate_v1_consumer_transport_contract_progress[key]} final={final_parse_age5_proof_certificate_v1_consumer_transport_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_proof_certificate_v1_verify_report_digest_contract_progress[key]
            != final_parse_age5_proof_certificate_v1_verify_report_digest_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_proof_certificate_v1_verify_report_digest_contract_progress[key]} final={final_parse_age5_proof_certificate_v1_verify_report_digest_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_proof_certificate_v1_family_contract_progress[key]
            != final_parse_age5_proof_certificate_v1_family_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_proof_certificate_v1_family_contract_progress[key]} final={final_parse_age5_proof_certificate_v1_family_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_proof_certificate_family_contract_progress[key]
            != final_parse_age5_proof_certificate_family_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_proof_certificate_family_contract_progress[key]} final={final_parse_age5_proof_certificate_family_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_proof_family_contract_progress[key]
            != final_parse_age5_proof_family_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_proof_family_contract_progress[key]} final={final_parse_age5_proof_family_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_lang_surface_family_contract_progress[key]
            != final_parse_age5_lang_surface_family_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_lang_surface_family_contract_progress[key]} final={final_parse_age5_lang_surface_family_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_lang_runtime_family_contract_progress[key]
            != final_parse_age5_lang_runtime_family_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_lang_runtime_family_contract_progress[key]} final={final_parse_age5_lang_runtime_family_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_lang_runtime_family_transport_contract_progress[key]
            != final_parse_age5_lang_runtime_family_transport_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_lang_runtime_family_transport_contract_progress[key]} final={final_parse_age5_lang_runtime_family_transport_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_gate0_runtime_family_transport_contract_progress[key]
            != final_parse_age5_gate0_runtime_family_transport_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_gate0_runtime_family_transport_contract_progress[key]} final={final_parse_age5_gate0_runtime_family_transport_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_gate0_family_transport_contract_progress[key]
            != final_parse_age5_gate0_family_transport_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_gate0_family_transport_contract_progress[key]} final={final_parse_age5_gate0_family_transport_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_gate0_transport_family_contract_progress[key]
            != final_parse_age5_gate0_transport_family_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_gate0_transport_family_contract_progress[key]} final={final_parse_age5_gate0_transport_family_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_gate0_transport_family_transport_contract_progress[key]
            != final_parse_age5_gate0_transport_family_transport_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_gate0_transport_family_transport_contract_progress[key]} final={final_parse_age5_gate0_transport_family_transport_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_gate0_surface_family_transport_contract_progress[key]
            != final_parse_age5_gate0_surface_family_transport_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_gate0_surface_family_transport_contract_progress[key]} final={final_parse_age5_gate0_surface_family_transport_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_lang_surface_family_transport_contract_progress[key]
            != final_parse_age5_lang_surface_family_transport_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_lang_surface_family_transport_contract_progress[key]} final={final_parse_age5_lang_surface_family_transport_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_proof_family_transport_contract_progress[key]
            != final_parse_age5_proof_family_transport_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_proof_family_transport_contract_progress[key]} final={final_parse_age5_proof_family_transport_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_proof_certificate_family_transport_contract_progress[key]
            != final_parse_age5_proof_certificate_family_transport_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_proof_certificate_family_transport_contract_progress[key]} final={final_parse_age5_proof_certificate_family_transport_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_bogae_alias_family_contract_progress[key]
            != final_parse_age5_bogae_alias_family_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_bogae_alias_family_contract_progress[key]} final={final_parse_age5_bogae_alias_family_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
    for key in AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if (
            result_age5_bogae_alias_family_transport_contract_progress[key]
            != final_parse_age5_bogae_alias_family_transport_contract_progress[key]
        ):
            return fail(
                f"ci_gate_result/final_status_parse mismatch key={key} result={result_age5_bogae_alias_family_transport_contract_progress[key]} final={final_parse_age5_bogae_alias_family_transport_contract_progress[key]}",
                CODES["ARTIFACT_JSON_INVALID"],
            )

    result_reason = str(result_doc.get("reason", "")).strip() or "-"

    badge_doc = artifact_docs["ci_gate_badge_json"]
    badge_status = str(badge_doc.get("status", "")).strip()
    if badge_status != result_status:
        return fail(
            f"ci_gate_badge status mismatch expected={result_status} actual={badge_status}",
            CODES["BADGE_STATUS_MISMATCH"],
        )
    badge_ok = badge_doc.get("ok")
    if not isinstance(badge_ok, bool):
        return fail("ci_gate_badge ok must be bool", CODES["BADGE_OK_TYPE"])
    if bool(badge_ok) != bool(expected_result_ok):
        return fail(
            f"ci_gate_badge ok mismatch expected={int(bool(expected_result_ok))} actual={int(bool(badge_ok))}",
            CODES["BADGE_OK_MISMATCH"],
        )
    badge_result_path = normalize_path_text(str(badge_doc.get("result_path", "")).strip())
    expected_badge_result_path = str(resolved_report_paths["ci_gate_result_json"])
    if badge_result_path != expected_badge_result_path:
        return fail(
            f"ci_gate_badge result_path mismatch expected={expected_badge_result_path} actual={badge_result_path}",
            CODES["BADGE_RESULT_PATH_MISMATCH"],
        )

    triage_doc = artifact_docs["ci_fail_triage_json"]
    triage_status = str(triage_doc.get("status", "")).strip()
    if triage_status != result_status:
        return fail(
            f"ci_fail_triage status mismatch expected={result_status} actual={triage_status}",
            CODES["TRIAGE_STATUS_MISMATCH"],
        )
    triage_reason = str(triage_doc.get("reason", "")).strip() or "-"
    if triage_reason != result_reason:
        return fail(
            f"ci_fail_triage reason mismatch expected={result_reason} actual={triage_reason}",
            CODES["TRIAGE_REASON_MISMATCH"],
        )
    profile_matrix_doc = artifact_docs["ci_profile_matrix_gate_selftest"]
    profile_matrix_snapshot = build_profile_matrix_snapshot_from_doc(
        profile_matrix_doc,
        report_path=str(resolved_report_paths["ci_profile_matrix_gate_selftest"]),
    )
    if not isinstance(profile_matrix_snapshot, dict):
        return fail(
            "ci_fail_triage profile_matrix snapshot build failed",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    expected_profile_matrix_triage = build_profile_matrix_triage_payload_from_snapshot(
        profile_matrix_snapshot
    )
    triage_profile_matrix = triage_doc.get("profile_matrix_selftest")
    if not isinstance(triage_profile_matrix, dict):
        return fail(
            "ci_fail_triage profile_matrix_selftest missing",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    missing_profile_matrix_keys = profile_matrix_triage_missing_keys(triage_profile_matrix)
    if missing_profile_matrix_keys:
        return fail(
            "ci_fail_triage profile_matrix_selftest missing keys="
            + ",".join(missing_profile_matrix_keys),
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    for key, expected_value in expected_profile_matrix_triage.items():
        actual_value = triage_profile_matrix.get(key)
        if isinstance(expected_value, (dict, list)):
            expected_text = json.dumps(
                expected_value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
            )
            actual_text = json.dumps(
                actual_value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
            )
            if actual_text != expected_text:
                return fail(
                    f"ci_fail_triage profile_matrix_selftest mismatch key={key}",
                    CODES["TRIAGE_ARTIFACTS_MISSING"],
                )
            continue
        if actual_value != expected_value:
            return fail(
                f"ci_fail_triage profile_matrix_selftest mismatch key={key}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    brief_text = read_text(resolved_report_paths["ci_fail_brief_txt"])
    brief_tokens = parse_tokens(brief_text)
    if brief_tokens is None:
        return fail("ci_fail_brief token format invalid", CODES["TRIAGE_ARTIFACTS_MISSING"])
    triage_failed_steps = triage_doc.get("failed_steps")
    if not isinstance(triage_failed_steps, list):
        return fail("ci_fail_triage failed_steps must be list", CODES["TRIAGE_ARTIFACTS_MISSING"])
    triage_failed_steps_count_raw = triage_doc.get("failed_steps_count")
    if not isinstance(triage_failed_steps_count_raw, int) or isinstance(triage_failed_steps_count_raw, bool):
        return fail("ci_fail_triage failed_steps_count must be int", CODES["TRIAGE_ARTIFACTS_MISSING"])
    triage_failed_steps_count = int(triage_failed_steps_count_raw)
    if triage_failed_steps_count != len(triage_failed_steps):
        return fail(
            "ci_fail_triage failed_steps_count mismatch "
            f"count={triage_failed_steps_count} len={len(triage_failed_steps)}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    if triage_failed_steps_count != result_failed_steps:
        return fail(
            "ci_fail_triage/result failed_steps_count mismatch "
            f"triage={triage_failed_steps_count} result={result_failed_steps}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    brief_failed_steps_count_text = str(brief_tokens.get("failed_steps_count", "")).strip()
    if not brief_failed_steps_count_text:
        return fail("ci_fail_brief missing key: failed_steps_count", CODES["SUMMARY_KEY_MISSING"])
    try:
        brief_failed_steps_count = int(brief_failed_steps_count_text)
    except Exception:
        return fail("ci_fail_brief failed_steps_count must be int", CODES["SUMMARY_VALUE_INVALID"])
    if brief_failed_steps_count != result_failed_steps:
        return fail(
            "ci_fail_brief/result failed_steps_count mismatch "
            f"brief={brief_failed_steps_count} result={result_failed_steps}",
            CODES["SUMMARY_VALUE_INVALID"],
        )
    if brief_failed_steps_count != triage_failed_steps_count:
        return fail(
            "ci_fail_brief/ci_fail_triage failed_steps_count mismatch "
            f"brief={brief_failed_steps_count} triage={triage_failed_steps_count}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    (
        summary_failed_step_details,
        summary_failed_step_logs,
        summary_failed_step_detail_order,
        summary_failed_step_log_order,
        summary_failed_step_row_sequence,
        summary_failed_step_rows_error,
    ) = (
        parse_summary_failed_step_rows(summary_lines)
    )
    if summary_failed_step_rows_error is not None:
        return fail(summary_failed_step_rows_error, CODES["TRIAGE_ARTIFACTS_MISSING"])
    triage_detail_rows_count_raw = triage_doc.get("failed_step_detail_rows_count")
    if not isinstance(triage_detail_rows_count_raw, int) or isinstance(triage_detail_rows_count_raw, bool):
        return fail("ci_fail_triage failed_step_detail_rows_count must be int", CODES["TRIAGE_ARTIFACTS_MISSING"])
    triage_detail_rows_count = int(triage_detail_rows_count_raw)
    if triage_detail_rows_count != len(summary_failed_step_details):
        return fail(
            "summary/ci_fail_triage failed_step_detail_rows_count mismatch "
            f"summary={len(summary_failed_step_details)} triage={triage_detail_rows_count}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    triage_logs_rows_count_raw = triage_doc.get("failed_step_logs_rows_count")
    if not isinstance(triage_logs_rows_count_raw, int) or isinstance(triage_logs_rows_count_raw, bool):
        return fail("ci_fail_triage failed_step_logs_rows_count must be int", CODES["TRIAGE_ARTIFACTS_MISSING"])
    triage_logs_rows_count = int(triage_logs_rows_count_raw)
    if triage_logs_rows_count != len(summary_failed_step_logs):
        return fail(
            "summary/ci_fail_triage failed_step_logs_rows_count mismatch "
            f"summary={len(summary_failed_step_logs)} triage={triage_logs_rows_count}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    triage_detail_order_raw = triage_doc.get("failed_step_detail_order")
    if not isinstance(triage_detail_order_raw, list):
        return fail("ci_fail_triage failed_step_detail_order must be list", CODES["TRIAGE_ARTIFACTS_MISSING"])
    triage_detail_order: list[str] = []
    for idx, item in enumerate(triage_detail_order_raw):
        step_id = str(item).strip()
        if not step_id:
            return fail(
                f"ci_fail_triage failed_step_detail_order[{idx}] empty",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        triage_detail_order.append(step_id)
    if triage_detail_order != summary_failed_step_detail_order:
        return fail(
            "summary/ci_fail_triage failed_step_detail_order mismatch "
            f"summary={','.join(summary_failed_step_detail_order) or '-'} "
            f"triage={','.join(triage_detail_order) or '-'}",
            CODES["SUMMARY_VALUE_INVALID"],
        )
    triage_logs_order_raw = triage_doc.get("failed_step_logs_order")
    if not isinstance(triage_logs_order_raw, list):
        return fail("ci_fail_triage failed_step_logs_order must be list", CODES["TRIAGE_ARTIFACTS_MISSING"])
    triage_logs_order: list[str] = []
    for idx, item in enumerate(triage_logs_order_raw):
        step_id = str(item).strip()
        if not step_id:
            return fail(
                f"ci_fail_triage failed_step_logs_order[{idx}] empty",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        triage_logs_order.append(step_id)
    if triage_logs_order != summary_failed_step_log_order:
        return fail(
            "summary/ci_fail_triage failed_step_logs_order mismatch "
            f"summary={','.join(summary_failed_step_log_order) or '-'} "
            f"triage={','.join(triage_logs_order) or '-'}",
            CODES["SUMMARY_VALUE_INVALID"],
        )
    triage_step_ids: list[str] = []
    triage_step_rc_by_id: dict[str, int] = {}
    triage_step_log_by_id: dict[str, dict[str, str]] = {}
    triage_step_cmd_by_id: dict[str, str] = {}
    for idx, row in enumerate(triage_failed_steps):
        if not isinstance(row, dict):
            return fail(f"ci_fail_triage failed_steps[{idx}] must be object", CODES["TRIAGE_ARTIFACTS_MISSING"])
        step_id = str(row.get("step_id", "")).strip() or str(row.get("name", "")).strip()
        if not step_id:
            return fail(
                f"ci_fail_triage failed_steps[{idx}] step_id/name missing",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        if step_id in triage_step_log_by_id:
            return fail(
                f"ci_fail_triage failed_steps duplicate step id: {step_id}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        triage_step_ids.append(step_id)
        returncode_raw = row.get("returncode")
        if not isinstance(returncode_raw, int) or isinstance(returncode_raw, bool):
            return fail(
                f"ci_fail_triage failed_steps[{idx}] returncode must be int",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        triage_returncode = int(returncode_raw)
        if triage_returncode == 0:
            return fail(
                f"ci_fail_triage failed_steps[{idx}] returncode must be non-zero",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        expected_index_rc = step_rc_by_name.get(step_id)
        if expected_index_rc is None:
            return fail(
                f"ci_fail_triage failed_steps[{idx}] step not found in index.steps: {step_id}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        if triage_returncode != int(expected_index_rc):
            return fail(
                "ci_fail_triage/index returncode mismatch "
                f"step={step_id} triage={triage_returncode} index={expected_index_rc}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        triage_step_rc_by_id[step_id] = triage_returncode
        triage_cmd = str(row.get("cmd", "")).strip()
        if not triage_cmd:
            return fail(
                f"ci_fail_triage failed_steps[{idx}] cmd missing",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        triage_step_cmd_by_id[step_id] = triage_cmd
        stdout_path = str(row.get("stdout_log_path", "")).strip()
        stderr_path = str(row.get("stderr_log_path", "")).strip()
        triage_step_log_by_id[step_id] = {"stdout": stdout_path, "stderr": stderr_path}
        fast_fail_step_detail = str(row.get("fast_fail_step_detail", "")).strip()
        expected_fast_fail_step_detail = f"name={step_id} rc={triage_returncode} cmd={triage_cmd}"
        if fast_fail_step_detail != expected_fast_fail_step_detail:
            return fail(
                "ci_fail_triage fast_fail_step_detail mismatch "
                f"step={step_id} triage={fast_fail_step_detail} expected={expected_fast_fail_step_detail}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        fast_fail_step_logs = str(row.get("fast_fail_step_logs", "")).strip()
        expected_fast_fail_step_logs = f"name={step_id} stdout={stdout_path or '-'} stderr={stderr_path or '-'}"
        if fast_fail_step_logs != expected_fast_fail_step_logs:
            return fail(
                "ci_fail_triage fast_fail_step_logs mismatch "
                f"step={step_id} triage={fast_fail_step_logs} expected={expected_fast_fail_step_logs}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        stdout_norm = str(row.get("stdout_log_path_norm", "")).strip()
        stderr_norm = str(row.get("stderr_log_path_norm", "")).strip()
        if stdout_path and stdout_norm != stdout_path.replace("\\", "/"):
            return fail(
                f"ci_fail_triage failed_steps[{idx}] stdout_log_path_norm mismatch",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        if stderr_path and stderr_norm != stderr_path.replace("\\", "/"):
            return fail(
                f"ci_fail_triage failed_steps[{idx}] stderr_log_path_norm mismatch",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        for label, path_text in (("stdout", stdout_path), ("stderr", stderr_path)):
            if not path_text:
                continue
            resolved_log = Path(path_text.replace("\\", "/"))
            if not resolved_log.exists():
                return fail(
                    f"ci_fail_triage failed_steps[{idx}] {label} log missing: {resolved_log}",
                    CODES["TRIAGE_ARTIFACTS_MISSING"],
                )
    expected_failed_step_order = [name for name, ok in step_ok_by_name.items() if not ok]
    expected_failed_step_ids = sorted(expected_failed_step_order)
    triage_failed_step_ids_sorted = sorted(set(triage_step_ids))
    if triage_failed_step_ids_sorted != expected_failed_step_ids:
        return fail(
            "ci_fail_triage/index failed step ids mismatch "
            f"triage={','.join(triage_failed_step_ids_sorted) or '-'} "
            f"index={','.join(expected_failed_step_ids) or '-'}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    summary_failed_steps = parse_failed_steps_value(summary_kv.get("failed_steps", ""))
    brief_failed_steps = parse_failed_steps_value(brief_tokens.get("failed_steps", ""))
    triage_failed_steps_with_logs = sorted(
        step_id
        for step_id, row in triage_step_log_by_id.items()
        if str(row.get("stdout", "")).strip() or str(row.get("stderr", "")).strip()
    )
    summary_detail_step_ids_sorted = sorted(summary_failed_step_details.keys())
    summary_log_step_ids_sorted = sorted(summary_failed_step_logs.keys())
    if brief_failed_steps_count != len(brief_failed_steps):
        return fail(
            "ci_fail_brief failed_steps_count/list mismatch "
            f"count={brief_failed_steps_count} list_len={len(brief_failed_steps)}",
            CODES["SUMMARY_VALUE_INVALID"],
        )
    if len(set(summary_failed_steps)) != len(summary_failed_steps):
        return fail("summary failed_steps contains duplicates", CODES["SUMMARY_VALUE_INVALID"])
    if len(set(brief_failed_steps)) != len(brief_failed_steps):
        return fail("ci_fail_brief failed_steps contains duplicates", CODES["SUMMARY_VALUE_INVALID"])
    if result_status == "pass":
        if triage_failed_steps_count != 0 or triage_step_ids:
            return fail(
                "ci_fail_triage pass report must have no failed steps",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        if summary_failed_steps:
            return fail("summary pass report must have no failed_steps", CODES["SUMMARY_STATUS_MISMATCH"])
        if brief_failed_steps:
            return fail("ci_fail_brief pass report must have no failed_steps", CODES["SUMMARY_VALUE_INVALID"])
        if summary_failed_step_details:
            return fail("summary pass report must have no failed_step_detail rows", CODES["SUMMARY_STATUS_MISMATCH"])
        if summary_failed_step_logs:
            return fail("summary pass report must have no failed_step_logs rows", CODES["SUMMARY_STATUS_MISMATCH"])
    else:
        if triage_failed_steps_count <= 0 or not triage_step_ids:
            return fail(
                "ci_fail_triage fail report must include failed steps",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        if not summary_failed_steps:
            return fail(
                "summary fail report must include failed_steps",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        if sorted(set(summary_failed_steps)) != triage_failed_step_ids_sorted:
            return fail(
                "summary/ci_fail_triage failed_steps mismatch "
                f"summary={','.join(sorted(set(summary_failed_steps)))} "
                f"triage={','.join(triage_failed_step_ids_sorted)}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        if summary_failed_steps != expected_failed_step_order:
            return fail(
                "summary/index failed_steps order mismatch "
                f"summary={','.join(summary_failed_steps) or '-'} "
                f"index={','.join(expected_failed_step_order) or '-'}",
                CODES["SUMMARY_VALUE_INVALID"],
            )
        if triage_step_ids != expected_failed_step_order:
            return fail(
                "ci_fail_triage/index failed_steps order mismatch "
                f"triage={','.join(triage_step_ids) or '-'} "
                f"index={','.join(expected_failed_step_order) or '-'}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        if triage_step_ids != summary_failed_steps:
            return fail(
                "summary/ci_fail_triage failed_steps order mismatch "
                f"summary={','.join(summary_failed_steps) or '-'} "
                f"triage={','.join(triage_step_ids) or '-'}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        if not brief_failed_steps:
            return fail(
                "ci_fail_brief fail report must include failed_steps",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        if sorted(set(brief_failed_steps)) != triage_failed_step_ids_sorted:
            return fail(
                "ci_fail_brief/ci_fail_triage failed_steps mismatch "
                f"brief={','.join(sorted(set(brief_failed_steps)))} "
                f"triage={','.join(triage_failed_step_ids_sorted)}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        if brief_failed_steps != expected_failed_step_order:
            return fail(
                "ci_fail_brief/index failed_steps order mismatch "
                f"brief={','.join(brief_failed_steps) or '-'} "
                f"index={','.join(expected_failed_step_order) or '-'}",
                CODES["SUMMARY_VALUE_INVALID"],
            )
        if brief_failed_steps != summary_failed_steps:
            return fail(
                "summary/ci_fail_brief failed_steps order mismatch "
                f"summary={','.join(summary_failed_steps) or '-'} "
                f"brief={','.join(brief_failed_steps) or '-'}",
                CODES["SUMMARY_VALUE_INVALID"],
            )
        if summary_detail_step_ids_sorted != triage_failed_step_ids_sorted:
            return fail(
                "summary/ci_fail_triage failed_step_detail ids mismatch "
                f"summary={','.join(summary_detail_step_ids_sorted) or '-'} "
                f"triage={','.join(triage_failed_step_ids_sorted) or '-'}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        if summary_failed_step_detail_order != summary_failed_steps:
            return fail(
                "summary failed_step_detail order mismatch "
                f"summary_failed_steps={','.join(summary_failed_steps) or '-'} "
                f"detail_order={','.join(summary_failed_step_detail_order) or '-'}",
                CODES["SUMMARY_VALUE_INVALID"],
            )
        if summary_log_step_ids_sorted != triage_failed_steps_with_logs:
            return fail(
                "summary/ci_fail_triage failed_step_logs ids mismatch "
                f"summary={','.join(summary_log_step_ids_sorted) or '-'} "
                f"triage={','.join(triage_failed_steps_with_logs) or '-'}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        expected_summary_log_order = [step_id for step_id in summary_failed_steps if step_id in summary_failed_step_logs]
        if summary_failed_step_log_order != expected_summary_log_order:
            return fail(
                "summary failed_step_logs order mismatch "
                f"summary_failed_steps={','.join(summary_failed_steps) or '-'} "
                f"log_order={','.join(summary_failed_step_log_order) or '-'}",
                CODES["SUMMARY_VALUE_INVALID"],
            )
        expected_summary_step_row_sequence: list[tuple[str, str]] = []
        for step_id in summary_failed_steps:
            expected_summary_step_row_sequence.append(("detail", step_id))
            if step_id in triage_failed_steps_with_logs:
                expected_summary_step_row_sequence.append(("logs", step_id))
        if summary_failed_step_row_sequence != expected_summary_step_row_sequence:
            expected_text = ",".join(f"{kind}:{step}" for kind, step in expected_summary_step_row_sequence) or "-"
            actual_text = ",".join(f"{kind}:{step}" for kind, step in summary_failed_step_row_sequence) or "-"
            return fail(
                "summary failed_step row sequence mismatch "
                f"expected={expected_text} actual={actual_text}",
                CODES["SUMMARY_VALUE_INVALID"],
            )
        for step_id in triage_failed_step_ids_sorted:
            summary_detail_row = summary_failed_step_details.get(step_id)
            if not isinstance(summary_detail_row, dict):
                return fail(
                    f"summary missing failed_step_detail for step={step_id}",
                    CODES["TRIAGE_ARTIFACTS_MISSING"],
                )
            summary_rc_raw = summary_detail_row.get("rc")
            if not isinstance(summary_rc_raw, int) or isinstance(summary_rc_raw, bool):
                return fail(
                    f"summary invalid failed_step_detail rc for step={step_id}",
                    CODES["TRIAGE_ARTIFACTS_MISSING"],
                )
            summary_rc = int(summary_rc_raw)
            if summary_rc == 0:
                return fail(
                    f"summary invalid failed_step_detail rc=0 for step={step_id}",
                    CODES["TRIAGE_ARTIFACTS_MISSING"],
                )
            triage_rc = int(triage_step_rc_by_id.get(step_id, -1))
            if summary_rc != triage_rc:
                return fail(
                    "summary/ci_fail_triage failed_step_detail rc mismatch "
                    f"step={step_id} summary={summary_rc} triage={triage_rc}",
                    CODES["TRIAGE_ARTIFACTS_MISSING"],
                )
            summary_cmd = str(summary_detail_row.get("cmd", "")).strip()
            if not summary_cmd:
                return fail(
                    f"summary empty failed_step_detail cmd for step={step_id}",
                    CODES["TRIAGE_ARTIFACTS_MISSING"],
                )
            triage_cmd = str(triage_step_cmd_by_id.get(step_id, "")).strip()
            if summary_cmd != triage_cmd:
                return fail(
                    "summary/ci_fail_triage failed_step_detail cmd mismatch "
                    f"step={step_id} summary={summary_cmd} triage={triage_cmd}",
                    CODES["TRIAGE_ARTIFACTS_MISSING"],
                )
            triage_log_row = triage_step_log_by_id.get(step_id, {})
            summary_log_row = summary_failed_step_logs.get(step_id, {})
            for label in ("stdout", "stderr"):
                triage_log = str(triage_log_row.get(label, "")).strip()
                summary_log = str(summary_log_row.get(label, "")).strip()
                if triage_log and not summary_log:
                    return fail(
                        f"summary missing failed_step_logs {label} for step={step_id}",
                        CODES["TRIAGE_ARTIFACTS_MISSING"],
                    )
                if not triage_log and summary_log:
                    return fail(
                        f"summary unexpected failed_step_logs {label} for step={step_id}",
                        CODES["TRIAGE_ARTIFACTS_MISSING"],
                    )
                if triage_log and normalize_path_text(summary_log) != normalize_path_text(triage_log):
                    return fail(
                        "summary/ci_fail_triage failed_step_logs mismatch "
                        f"step={step_id} label={label} summary={summary_log} triage={triage_log}",
                        CODES["TRIAGE_ARTIFACTS_MISSING"],
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
            return fail(f"ci_fail_brief missing key: {key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_W107_BRIEF_KEY_MAP:
        expected_value = result_age5_w107_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_W107_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_w107_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_age1_immediate_proof_operation_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_proof_certificate_v1_consumer_transport_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_proof_certificate_v1_verify_report_digest_contract_progress[
            triage_key
        ]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_proof_certificate_v1_family_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_proof_certificate_family_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_PROOF_FAMILY_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_proof_family_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_LANG_SURFACE_FAMILY_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_lang_surface_family_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_LANG_RUNTIME_FAMILY_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_lang_runtime_family_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_lang_runtime_family_transport_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_gate0_runtime_family_transport_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_gate0_family_transport_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_gate0_surface_family_transport_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_gate0_transport_family_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_lang_surface_family_transport_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_proof_family_transport_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_proof_certificate_family_transport_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_bogae_alias_family_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for triage_key, brief_key in AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP:
        expected_value = result_age5_bogae_alias_family_transport_contract_progress[triage_key]
        brief_value = str(brief_tokens.get(brief_key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {brief_key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {brief_key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(triage_key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {triage_key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {triage_key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={triage_key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    age5_policy_snapshot = load_age5_policy_snapshot(
        index_doc,
        aggregate_doc=aggregate_doc,
        json_cache=json_cache,
    )
    expected_digest_selftest = str(summary_kv.get(AGE5_DIGEST_SELFTEST_SUMMARY_KEY, "")).strip()
    if expected_digest_selftest not in {"0", "1"}:
        expected_digest_selftest = "1" if bool(step_ok_by_name.get("age5_close_digest_selftest", False)) else "0"
    brief_digest_selftest = str(brief_tokens.get(AGE5_DIGEST_SELFTEST_SUMMARY_KEY, "")).strip()
    if not brief_digest_selftest:
        return fail(f"ci_fail_brief missing key: {AGE5_DIGEST_SELFTEST_SUMMARY_KEY}", CODES["SUMMARY_KEY_MISSING"])
    if brief_digest_selftest not in {"0", "1"}:
        return fail(
            f"ci_fail_brief invalid {AGE5_DIGEST_SELFTEST_SUMMARY_KEY}: {brief_digest_selftest}",
            CODES["SUMMARY_VALUE_INVALID"],
        )
    triage_digest_selftest = str(triage_doc.get(AGE5_DIGEST_SELFTEST_SUMMARY_KEY, "")).strip()
    if not triage_digest_selftest:
        return fail(
            f"ci_fail_triage missing key: {AGE5_DIGEST_SELFTEST_SUMMARY_KEY}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    if triage_digest_selftest not in {"0", "1"}:
        return fail(
            f"ci_fail_triage invalid {AGE5_DIGEST_SELFTEST_SUMMARY_KEY}: {triage_digest_selftest}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    if brief_digest_selftest != expected_digest_selftest:
        return fail(
            "summary/ci_fail_brief mismatch key=age5_close_digest_selftest_ok "
            f"summary={expected_digest_selftest} brief={brief_digest_selftest}",
            CODES["SUMMARY_VALUE_INVALID"],
        )
    if triage_digest_selftest != expected_digest_selftest:
        return fail(
            "summary/ci_fail_triage mismatch key=age5_close_digest_selftest_ok "
            f"summary={expected_digest_selftest} triage={triage_digest_selftest}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    if triage_digest_selftest != brief_digest_selftest:
        return fail(
            "ci_fail_triage/ci_fail_brief mismatch key=age5_close_digest_selftest_ok "
            f"brief={brief_digest_selftest} triage={triage_digest_selftest}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    brief_default_text = str(brief_tokens.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip()
    if not brief_default_text:
        return fail(
            f"ci_fail_brief missing key: {AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}",
            CODES["SUMMARY_KEY_MISSING"],
        )
    if brief_default_text != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
        return fail(
            f"ci_fail_brief invalid {AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}: {brief_default_text}",
            CODES["SUMMARY_VALUE_INVALID"],
        )
    brief_default_field = str(brief_tokens.get("combined_digest_selftest_default_field", "")).strip()
    if not brief_default_field:
        return fail(
            "ci_fail_brief missing key: combined_digest_selftest_default_field",
            CODES["SUMMARY_KEY_MISSING"],
        )
    expected_brief_default_field = AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT.split("=", 1)[1]
    if brief_default_field != expected_brief_default_field:
        return fail(
            f"ci_fail_brief invalid combined_digest_selftest_default_field: {brief_default_field}",
            CODES["SUMMARY_VALUE_INVALID"],
        )
    brief_policy_default_text = str(
        brief_tokens.get(AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")
    ).strip()
    if not brief_policy_default_text:
        return fail(
            f"ci_fail_brief missing key: {AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}",
            CODES["SUMMARY_KEY_MISSING"],
        )
    expected_brief_policy_default_text = str(
        age5_policy_snapshot[AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY]
    ).strip()
    if brief_policy_default_text != expected_brief_policy_default_text:
        return fail(
            f"ci_fail_brief invalid {AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}: {brief_policy_default_text}",
            CODES["SUMMARY_VALUE_INVALID"],
        )
    brief_policy_default_field = str(
        brief_tokens.get(AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY, "")
    ).strip()
    if not brief_policy_default_field:
        return fail(
            f"ci_fail_brief missing key: {AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY}",
            CODES["SUMMARY_KEY_MISSING"],
        )
    expected_brief_policy_default_field = json.dumps(
        dict(age5_policy_snapshot[AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY]),
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    )
    if brief_policy_default_field != expected_brief_policy_default_field:
        return fail(
            f"ci_fail_brief invalid {AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY}: {brief_policy_default_field}",
            CODES["SUMMARY_VALUE_INVALID"],
        )
    brief_policy_age4_snapshot_fields_text = str(
        brief_tokens.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY, "")
    ).strip()
    if not brief_policy_age4_snapshot_fields_text:
        return fail(
            f"ci_fail_brief missing key: {AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY}",
            CODES["SUMMARY_KEY_MISSING"],
        )
    expected_brief_policy_age4_snapshot_fields_text = str(
        age5_policy_snapshot[AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY]
    ).strip()
    if brief_policy_age4_snapshot_fields_text != expected_brief_policy_age4_snapshot_fields_text:
        return fail(
            f"ci_fail_brief invalid {AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY}: {brief_policy_age4_snapshot_fields_text}",
            CODES["SUMMARY_VALUE_INVALID"],
        )
    brief_policy_age4_snapshot_text = str(
        brief_tokens.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY, "")
    ).strip()
    if not brief_policy_age4_snapshot_text:
        return fail(
            f"ci_fail_brief missing key: {AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY}",
            CODES["SUMMARY_KEY_MISSING"],
        )
    expected_brief_policy_age4_snapshot_text = str(
        age5_policy_snapshot[AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY]
    ).strip()
    if brief_policy_age4_snapshot_text != expected_brief_policy_age4_snapshot_text:
        return fail(
            f"ci_fail_brief invalid {AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY}: {brief_policy_age4_snapshot_text}",
            CODES["SUMMARY_VALUE_INVALID"],
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
            return fail(f"ci_fail_brief missing key: {key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
    brief_policy_origin_trace_text = str(brief_tokens.get(AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY, "")).strip()
    if not brief_policy_origin_trace_text:
        return fail(
            f"ci_fail_brief missing key: {AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY}",
            CODES["SUMMARY_KEY_MISSING"],
        )
    expected_brief_policy_origin_trace_text = str(
        age5_policy_snapshot[AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY]
    ).strip()
    if brief_policy_origin_trace_text != expected_brief_policy_origin_trace_text:
        return fail(
            f"ci_fail_brief invalid {AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY}: {brief_policy_origin_trace_text}",
            CODES["SUMMARY_VALUE_INVALID"],
        )
    brief_policy_origin_trace = str(brief_tokens.get(AGE5_POLICY_ORIGIN_TRACE_KEY, "")).strip()
    if not brief_policy_origin_trace:
        return fail(
            f"ci_fail_brief missing key: {AGE5_POLICY_ORIGIN_TRACE_KEY}",
            CODES["SUMMARY_KEY_MISSING"],
        )
    expected_brief_policy_origin_trace = json.dumps(
        dict(age5_policy_snapshot[AGE5_POLICY_ORIGIN_TRACE_KEY]),
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    )
    if brief_policy_origin_trace != expected_brief_policy_origin_trace:
        return fail(
            f"ci_fail_brief invalid {AGE5_POLICY_ORIGIN_TRACE_KEY}: {brief_policy_origin_trace}",
            CODES["SUMMARY_VALUE_INVALID"],
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
            return fail(f"ci_fail_brief missing key: {key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected_value:
            return fail(f"ci_fail_brief invalid {key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
    triage_default_text = str(triage_doc.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip()
    if not triage_default_text:
        return fail(
            f"ci_fail_triage missing key: {AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    if triage_default_text != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
        return fail(
            f"ci_fail_triage invalid {AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}: {triage_default_text}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    if brief_default_text != triage_default_text:
        return fail(
            f"ci_fail_triage/ci_fail_brief mismatch key={AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY} "
            f"brief={brief_default_text} triage={triage_default_text}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    triage_default_field = triage_doc.get("combined_digest_selftest_default_field")
    if not isinstance(triage_default_field, dict):
        return fail(
            "ci_fail_triage combined_digest_selftest_default_field missing",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    if dict(triage_default_field) != AGE5_DIGEST_SELFTEST_DEFAULT_FIELD:
        return fail(
            "ci_fail_triage invalid combined_digest_selftest_default_field",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    triage_default_field_text = json.dumps(
        dict(triage_default_field), ensure_ascii=False, sort_keys=True, separators=(",", ":")
    )
    if brief_default_field != triage_default_field_text:
        return fail(
            "ci_fail_triage/ci_fail_brief mismatch key=combined_digest_selftest_default_field "
            f"brief={brief_default_field} triage={triage_default_field_text}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    triage_policy_default_text = str(
        triage_doc.get(AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")
    ).strip()
    if not triage_policy_default_text:
        return fail(
            f"ci_fail_triage missing key: {AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    if triage_policy_default_text != expected_brief_policy_default_text:
        return fail(
            f"ci_fail_triage invalid {AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}: {triage_policy_default_text}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    if brief_policy_default_text != triage_policy_default_text:
        return fail(
            f"ci_fail_triage/ci_fail_brief mismatch key={AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY} "
            f"brief={brief_policy_default_text} triage={triage_policy_default_text}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    triage_policy_default_field = triage_doc.get(AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY)
    if not isinstance(triage_policy_default_field, dict):
        return fail(
            f"ci_fail_triage {AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY} missing",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    if dict(triage_policy_default_field) != dict(age5_policy_snapshot[AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY]):
        return fail(
            f"ci_fail_triage invalid {AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    triage_policy_default_field_text = json.dumps(
        dict(triage_policy_default_field), ensure_ascii=False, sort_keys=True, separators=(",", ":")
    )
    if brief_policy_default_field != triage_policy_default_field_text:
        return fail(
            f"ci_fail_triage/ci_fail_brief mismatch key={AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY} "
            f"brief={brief_policy_default_field} triage={triage_policy_default_field_text}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    triage_policy_age4_snapshot_fields_text = str(
        triage_doc.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY, "")
    ).strip()
    if not triage_policy_age4_snapshot_fields_text:
        return fail(
            f"ci_fail_triage missing key: {AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    if triage_policy_age4_snapshot_fields_text != expected_brief_policy_age4_snapshot_fields_text:
        return fail(
            f"ci_fail_triage invalid {AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY}: {triage_policy_age4_snapshot_fields_text}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    if brief_policy_age4_snapshot_fields_text != triage_policy_age4_snapshot_fields_text:
        return fail(
            f"ci_fail_triage/ci_fail_brief mismatch key={AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY} "
            f"brief={brief_policy_age4_snapshot_fields_text} triage={triage_policy_age4_snapshot_fields_text}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    triage_policy_age4_snapshot_text = str(
        triage_doc.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY, "")
    ).strip()
    if not triage_policy_age4_snapshot_text:
        return fail(
            f"ci_fail_triage missing key: {AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    if triage_policy_age4_snapshot_text != expected_brief_policy_age4_snapshot_text:
        return fail(
            f"ci_fail_triage invalid {AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY}: {triage_policy_age4_snapshot_text}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    if brief_policy_age4_snapshot_text != triage_policy_age4_snapshot_text:
        return fail(
            f"ci_fail_triage/ci_fail_brief mismatch key={AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY} "
            f"brief={brief_policy_age4_snapshot_text} triage={triage_policy_age4_snapshot_text}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
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
            return fail(f"ci_fail_triage missing key: {key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        brief_value = str(brief_tokens.get(key, "")).strip()
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    triage_policy_origin_trace_text = str(
        triage_doc.get(AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY, "")
    ).strip()
    if not triage_policy_origin_trace_text:
        return fail(
            f"ci_fail_triage missing key: {AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    if triage_policy_origin_trace_text != expected_brief_policy_origin_trace_text:
        return fail(
            f"ci_fail_triage invalid {AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY}: {triage_policy_origin_trace_text}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    if brief_policy_origin_trace_text != triage_policy_origin_trace_text:
        return fail(
            f"ci_fail_triage/ci_fail_brief mismatch key={AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY} "
            f"brief={brief_policy_origin_trace_text} triage={triage_policy_origin_trace_text}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    triage_policy_origin_trace = triage_doc.get(AGE5_POLICY_ORIGIN_TRACE_KEY)
    if not isinstance(triage_policy_origin_trace, dict):
        return fail(
            f"ci_fail_triage {AGE5_POLICY_ORIGIN_TRACE_KEY} missing",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    if dict(triage_policy_origin_trace) != dict(age5_policy_snapshot[AGE5_POLICY_ORIGIN_TRACE_KEY]):
        return fail(
            f"ci_fail_triage invalid {AGE5_POLICY_ORIGIN_TRACE_KEY}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
        )
    triage_policy_origin_trace_text_json = json.dumps(
        dict(triage_policy_origin_trace),
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    )
    if brief_policy_origin_trace != triage_policy_origin_trace_text_json:
        return fail(
            f"ci_fail_triage/ci_fail_brief mismatch key={AGE5_POLICY_ORIGIN_TRACE_KEY} "
            f"brief={brief_policy_origin_trace} triage={triage_policy_origin_trace_text_json}",
            CODES["TRIAGE_ARTIFACTS_MISSING"],
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
            return fail(f"ci_fail_triage missing key: {key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected_value:
            return fail(f"ci_fail_triage invalid {key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        brief_value = str(brief_tokens.get(key, "")).strip()
        if brief_value != triage_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for key in AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS:
        summary_value = str(summary_kv.get(key, "")).strip()
        if not summary_value:
            return fail(f"summary missing key: {key}", CODES["SUMMARY_KEY_MISSING"])
        if summary_value not in VALID_AGE5_CHILD_SUMMARY_STATUS:
            return fail(f"summary invalid {key}: {summary_value}", CODES["SUMMARY_VALUE_INVALID"])
        brief_value = str(brief_tokens.get(key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value not in VALID_AGE5_CHILD_SUMMARY_STATUS:
            return fail(f"ci_fail_brief invalid {key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value not in VALID_AGE5_CHILD_SUMMARY_STATUS:
            return fail(f"ci_fail_triage invalid {key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if summary_value != brief_value:
            return fail(
                f"summary/ci_fail_brief mismatch key={key} summary={summary_value} brief={brief_value}",
                CODES["SUMMARY_VALUE_INVALID"],
            )
        if summary_value != triage_value:
            return fail(
                f"summary/ci_fail_triage mismatch key={key} summary={summary_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        if triage_value != brief_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    for key, expected in AGE5_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS.items():
        summary_value = str(summary_kv.get(key, "")).strip() or expected
        brief_value = str(brief_tokens.get(key, "")).strip()
        if not brief_value:
            return fail(f"ci_fail_brief missing key: {key}", CODES["SUMMARY_KEY_MISSING"])
        if brief_value != expected:
            return fail(f"ci_fail_brief invalid {key}: {brief_value}", CODES["SUMMARY_VALUE_INVALID"])
        triage_value = str(triage_doc.get(key, "")).strip()
        if not triage_value:
            return fail(f"ci_fail_triage missing key: {key}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if triage_value != expected:
            return fail(f"ci_fail_triage invalid {key}: {triage_value}", CODES["TRIAGE_ARTIFACTS_MISSING"])
        if summary_value != brief_value:
            return fail(
                f"summary/ci_fail_brief mismatch key={key} summary={summary_value} brief={brief_value}",
                CODES["SUMMARY_VALUE_INVALID"],
            )
        if summary_value != triage_value:
            return fail(
                f"summary/ci_fail_triage mismatch key={key} summary={summary_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        if triage_value != brief_value:
            return fail(
                f"ci_fail_triage/ci_fail_brief mismatch key={key} brief={brief_value} triage={triage_value}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
    triage_summary_hint_norm = normalize_path_text(str(triage_doc.get("summary_report_path_hint_norm", "")).strip())
    expected_summary_hint_norm = normalize_path_text(str(resolved_report_paths["summary"]))
    if triage_summary_hint_norm != expected_summary_hint_norm:
        return fail(
            "ci_fail_triage summary_report_path_hint_norm mismatch",
            CODES["TRIAGE_SUMMARY_HINT_NORM_MISMATCH"],
        )
    triage_artifacts = triage_doc.get("artifacts")
    if not isinstance(triage_artifacts, dict):
        return fail("ci_fail_triage artifacts missing", CODES["TRIAGE_ARTIFACTS_MISSING"])
    triage_artifact_required_keys = (
        "summary",
        "ci_gate_result_json",
        "ci_gate_badge_json",
        "ci_fail_brief_txt",
        "ci_fail_triage_json",
    )
    for artifact_key in triage_artifact_required_keys:
        artifact_row = triage_artifacts.get(artifact_key)
        if not isinstance(artifact_row, dict):
            return fail(
                f"ci_fail_triage artifacts missing row key={artifact_key}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        artifact_path = normalize_path_text(str(artifact_row.get("path", "")).strip())
        expected_artifact_path = str(resolved_report_paths[artifact_key])
        if artifact_path != expected_artifact_path:
            return fail(
                f"ci_fail_triage artifacts path mismatch key={artifact_key}",
                CODES["TRIAGE_ARTIFACT_PATH_MISMATCH"],
            )
        artifact_path_norm = normalize_path_text(str(artifact_row.get("path_norm", "")).strip())
        expected_artifact_path_norm = str(resolved_report_paths[artifact_key])
        if artifact_path_norm != expected_artifact_path_norm:
            return fail(
                f"ci_fail_triage artifacts path_norm mismatch key={artifact_key}",
                CODES["TRIAGE_ARTIFACT_PATH_NORM_MISMATCH"],
            )
        artifact_exists = artifact_row.get("exists")
        if not isinstance(artifact_exists, bool) or not artifact_exists:
            return fail(
                f"ci_fail_triage artifacts exists mismatch key={artifact_key}",
                CODES["TRIAGE_ARTIFACT_EXISTS_MISMATCH"],
            )

    required_steps: list[str] = []
    if bool(args.enforce_profile_step_contract):
        required_steps.extend(resolve_profile_required_steps(str(args.sanity_profile).strip()))
    required_steps.extend([str(item).strip() for item in args.required_step if str(item).strip()])
    deduped_required_steps: list[str] = []
    seen_required_steps: set[str] = set()
    for step_name in required_steps:
        if not step_name or step_name in seen_required_steps:
            continue
        seen_required_steps.add(step_name)
        deduped_required_steps.append(step_name)
    missing_required_steps = [name for name in deduped_required_steps if name not in seen_step_names]
    if missing_required_steps:
        return fail(
            f"missing required index step(s): {','.join(missing_required_steps)}",
            CODES["REQUIRED_STEP_MISSING"],
        )

    profile_matrix_error = validate_profile_matrix_selftest(profile_matrix_doc)
    if profile_matrix_error is not None:
        return fail(profile_matrix_error, CODES["ARTIFACT_JSON_INVALID"])

    print(f"[ci-gate-report-index-check] ok index={index_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
