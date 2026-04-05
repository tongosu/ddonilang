#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

from _ci_age5_combined_heavy_contract import (
    AGE4_PROOF_FAILED_CRITERIA_KEY,
    AGE4_PROOF_FAILED_PREVIEW_KEY,
    AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY,
    AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY,
    AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_TEXT_KEY,
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY,
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY,
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_TEXT_KEY,
    AGE4_PROOF_OK_KEY,
    AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
    AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_DEFAULT,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_ENV_KEY,
    AGE5_COMBINED_HEAVY_FLAG,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_FULL_REAL_SOURCE_TRACE_TEXT,
    AGE5_COMBINED_HEAVY_MODE,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_REPORT_SCHEMA,
    AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA,
    AGE5_COMBINED_HEAVY_REQUIRED_REPORTS,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT,
    AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_DEFAULT,
    AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_KEY,
    AGE5_FULL_REAL_ELAPSED_FIELDS_TEXT,
    AGE5_FULL_REAL_CORE_LANG_SANITY_ELAPSED_FIELDS_TEXT,
    AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROBE_FIELDS_TEXT,
    AGE5_FULL_REAL_FIXED64_DARWIN_REAL_REPORT_READINESS_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_MAP_ACCESS_CONTRACT_CHECK_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_CI_PACK_GOLDEN_EXEC_POLICY_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_CI_PACK_GOLDEN_AGE5_SURFACE_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_CI_PACK_GOLDEN_GUIDEBLOCK_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_CI_PACK_GOLDEN_JJAIM_FLATTEN_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_CI_PACK_GOLDEN_EVENT_MODEL_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_CI_PACK_GOLDEN_LANG_CONSISTENCY_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROOF_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_GATE0_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_W94_SOCIAL_PACK_CHECK_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_W95_CERT_PACK_CHECK_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_W96_SOMSSI_PACK_CHECK_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_W97_SELF_HEAL_PACK_CHECK_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_TENSOR_V0_CLI_CHECK_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_ELAPSED_MAP_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_STATUS_MAP_FIELDS_TEXT,
    AGE5_FULL_REAL_TIMEOUT_BREAKDOWN_FIELDS_TEXT,
    build_age5_combined_heavy_child_summary_fields,
    build_age5_combined_heavy_child_summary_fields_from_criteria,
    build_age5_combined_heavy_combined_report_contract_fields,
    build_age5_combined_heavy_child_summary_default_text_transport_fields,
    build_age5_combined_heavy_full_real_source_trace,
    build_age5_combined_heavy_full_real_source_trace_text,
    build_age5_combined_heavy_child_summary_default_fields,
    build_age5_combined_heavy_full_summary_contract_fields,
    build_age5_combined_heavy_full_summary_text_transport_fields,
    build_age5_combined_heavy_timeout_policy_fields,
    build_age4_proof_snapshot,
    build_age4_proof_source_snapshot_fields,
    build_age4_proof_snapshot_text,
    build_age5_close_digest_selftest_default_field,
    build_age5_full_real_elapsed_summary,
    build_age5_full_real_core_lang_sanity_elapsed_summary,
    build_age5_full_real_pipeline_emit_flags_progress,
    build_age5_full_real_pipeline_emit_flags_selftest_progress,
    build_age5_full_real_pipeline_emit_flags_selftest_probe,
    build_age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress,
    build_age5_full_real_map_access_contract_check_progress,
    build_age5_full_real_ci_pack_golden_exec_policy_selftest_progress,
    build_age5_full_real_ci_pack_golden_age5_surface_selftest_progress,
    build_age5_full_real_ci_pack_golden_guideblock_selftest_progress,
    build_age5_full_real_ci_pack_golden_jjaim_flatten_selftest_progress,
    build_age5_full_real_ci_pack_golden_event_model_selftest_progress,
    build_age5_full_real_ci_pack_golden_lang_consistency_selftest_progress,
    build_age5_full_real_w107_golden_index_selftest_progress,
    build_age5_full_real_w107_progress_contract_selftest_progress,
    build_age5_full_real_age1_immediate_proof_operation_contract_selftest_progress,
    build_age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress,
    build_age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress,
    build_age5_full_real_proof_certificate_v1_family_contract_selftest_progress,
    build_age5_full_real_proof_certificate_family_contract_selftest_progress,
    build_age5_full_real_proof_certificate_family_transport_contract_selftest_progress,
    build_age5_full_real_proof_family_contract_selftest_progress,
    build_age5_full_real_proof_family_transport_contract_selftest_progress,
    build_age5_full_real_lang_surface_family_contract_selftest_progress,
    build_age5_full_real_lang_surface_family_transport_contract_selftest_progress,
    build_age5_full_real_lang_runtime_family_contract_selftest_progress,
    build_age5_full_real_lang_runtime_family_transport_contract_selftest_progress,
    build_age5_full_real_gate0_runtime_family_transport_contract_selftest_progress,
    build_age5_full_real_gate0_family_contract_selftest_progress,
    build_age5_full_real_gate0_surface_family_contract_selftest_progress,
    build_age5_full_real_gate0_surface_family_transport_contract_selftest_progress,
    build_age5_full_real_gate0_family_transport_contract_selftest_progress,
    build_age5_full_real_gate0_transport_family_contract_selftest_progress,
    build_age5_full_real_gate0_transport_family_transport_contract_selftest_progress,
    build_age5_full_real_bogae_alias_family_contract_selftest_progress,
    build_age5_full_real_bogae_alias_family_transport_contract_selftest_progress,
    build_age5_full_real_w94_social_pack_check_progress,
    build_age5_full_real_w95_cert_pack_check_progress,
    build_age5_full_real_w96_somssi_pack_check_progress,
    build_age5_full_real_w97_self_heal_pack_check_progress,
    build_age5_full_real_tensor_v0_cli_check_progress,
    build_age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress,
    build_age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress,
    build_age5_full_real_profile_elapsed_map,
    build_age5_full_real_profile_status_map,
    build_age5_full_real_timeout_breakdown,
    resolve_age5_combined_heavy_timeout_mode,
)
from _ci_profile_matrix_full_real_smoke_contract import (
    PROFILE_MATRIX_FULL_REAL_SMOKE_ALLOW_FLAG,
    PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SCRIPT,
    PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY,
    PROFILE_MATRIX_FULL_REAL_SMOKE_FLAG,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STATUS_PASS,
)

AGE5_FULL_REAL_CORE_LANG_SANITY_PROGRESS_FIELDS_TEXT = (
    "age5_full_real_core_lang_sanity_current_step=-|"
    "age5_full_real_core_lang_sanity_last_completed_step=-|"
    "age5_full_real_core_lang_sanity_progress_present=0"
)

AGE4_S2_TASK_PATH = Path("docs/context/codex_tasks/TASK_SEAMGRIM_AGE4_S2_PRIMITIVE_RUNTIME_UI_SLOTS_V1.md")
S5_BASELINE_TASK_PATH = Path("docs/context/codex_tasks/S5_OVERLAY_BASELINE_VARIANT.md")
S5_DETAILED_TASK_PATH = Path("docs/context/codex_tasks/S5_OVERLAY_2LAYER_DETAILED.md")
AGE5_SLOT_UI_PATH = Path("solutions/seamgrim_ui_mvp/ui/index.html")
AGE5_APP_UI_PATH = Path("solutions/seamgrim_ui_mvp/ui/app.js")
OVERLAY_SESSION_CONTRACT_PATH = Path("solutions/seamgrim_ui_mvp/ui/overlay_session_contract.js")
SLOT_LABELS = [
    "A 실시간 입력",
    "B 꾸러미 브라우저",
    "C 3D 렌더",
]
PACK_HINT = "pack/seamgrim_overlay_param_compare_v0"
PACK_GOLDEN_PATH = Path("pack/seamgrim_overlay_param_compare_v0/golden.jsonl")
PACK_MIN_CASE_COUNT = 76
S6_SESSION_PACK_HINT = "pack/seamgrim_overlay_session_roundtrip_v0"
S6_SESSION_PACK_GOLDEN_PATH = Path("pack/seamgrim_overlay_session_roundtrip_v0/golden.jsonl")
S6_SESSION_PACK_MIN_CASE_COUNT = 9
S5_BASELINE_DOD_TOKENS = [
    "- [x] 진자 L 비교 가능.",
    "- [x] 축 불일치 차단.",
]
S5_DETAILED_DOD_TOKENS = [
    "- [x] 진자 L=1.0 vs L=2.0 오버레이 가능",
    "- [x] 축 메타 불일치 시 차단",
    "- [x] session 저장/로드 시 variant params/visible/order 보존",
]
S6_SESSION_CONTRACT_APP_TOKENS = [
    "./overlay_session_contract.js",
    "buildOverlaySessionRunsPayload(",
    "buildOverlayCompareSessionPayload(",
    "resolveOverlayCompareFromSession(",
]
S6_SESSION_CONTRACT_MODULE_TOKENS = [
    "export function buildOverlaySessionRunsPayload",
    "export function buildOverlayCompareSessionPayload",
    "export function resolveOverlayCompareFromSession",
]
S6_VIEW_COMBO_CONTRACT_APP_TOKENS = [
    "buildSessionViewComboPayload(",
    "resolveSessionViewComboFromPayload(",
    "view_combo: buildSessionViewComboPayload({",
]
S6_VIEW_COMBO_CONTRACT_MODULE_TOKENS = [
    "export function resolveSessionViewComboFromPayload",
    "export function buildSessionViewComboPayload",
]
S6_SESSION_PACK_CASE_FILES = [
    Path("pack/seamgrim_overlay_session_roundtrip_v0/c01_role_priority_restore_ok/case.detjson"),
    Path("pack/seamgrim_overlay_session_roundtrip_v0/c02_compare_id_fallback_when_role_missing/case.detjson"),
    Path("pack/seamgrim_overlay_session_roundtrip_v0/c03_drop_variant_on_axis_mismatch/case.detjson"),
    Path("pack/seamgrim_overlay_session_roundtrip_v0/c04_disable_on_baseline_missing/case.detjson"),
    Path("pack/seamgrim_overlay_session_roundtrip_v0/c05_ui_layout_restore_run_tools/case.detjson"),
    Path("pack/seamgrim_overlay_session_roundtrip_v0/c06_ui_layout_invalid_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_session_roundtrip_v0/c07_ui_layout_view_combo_cross_restore/case.detjson"),
    Path("pack/seamgrim_overlay_session_roundtrip_v0/c08_ui_layout_view_combo_cross_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_session_roundtrip_v0/c09_compare_disabled_with_runs_and_basic_layout/case.detjson"),
]
PACK_CASE_FILES = [
    Path("pack/seamgrim_overlay_param_compare_v0/c01_pendulum_L_compare/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c02_axis_mismatch_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c03_series_missing_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c04_series_mismatch_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c05_graph_kind_mismatch_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c06_y_unit_mismatch_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c07_graph_missing_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c08_y_kind_mismatch_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c09_series_missing_baseline_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c10_x_kind_mismatch_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c11_graph_kind_normalized_equal_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c12_series_id_normalized_equal_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c13_x_unit_normalized_equal_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c14_y_kind_normalized_equal_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c15_x_kind_normalized_equal_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c16_y_unit_normalized_equal_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c17_graph_kind_meta_kind_fallback_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c18_graph_kind_schema_fallback_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c19_x_kind_axis_kind_fallback_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c20_x_unit_axis_unit_fallback_ok/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c21_x_kind_empty_fallback_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c22_x_unit_empty_fallback_blocked/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c23_graph_missing_priority_over_series_missing/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c24_series_missing_with_meta_kind_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c25_graph_missing_over_axis_mismatch_xkind/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c26_graph_missing_over_axis_mismatch_xunit/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c27_graph_missing_over_series_mismatch_candidate_baseline/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c28_graph_missing_over_series_mismatch_candidate_variant/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c29_graph_missing_over_axis_mismatch_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c30_graph_missing_over_axis_mismatch_reverse_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c31_axis_mismatch_order_graphkind_before_xkind/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c32_axis_mismatch_order_xkind_before_xunit/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c33_axis_mismatch_order_xunit_before_ykind/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c34_axis_mismatch_order_ykind_before_yunit/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c35_series_missing_over_mismatch_candidate_baseline/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c36_series_missing_over_mismatch_candidate_variant/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c37_graph_missing_over_graphkind_mismatch_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c38_graph_missing_over_graphkind_mismatch_reverse_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c39_graphkind_mismatch_over_series_missing_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c40_graphkind_mismatch_over_series_missing_reverse_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c41_xkind_mismatch_over_series_missing_with_axis_kind_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c42_xunit_mismatch_over_series_missing_reverse_with_axis_unit_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c43_ykind_mismatch_over_series_missing_with_y_kind_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c44_yunit_mismatch_over_series_missing_reverse_with_y_unit_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c45_graph_missing_over_ykind_mismatch_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c46_graph_missing_over_yunit_mismatch_reverse_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c47_graph_missing_over_series_mismatch_with_ykind_candidate/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c48_graph_missing_over_series_mismatch_reverse_with_yunit_candidate/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c49_series_missing_over_ykind_mismatch_candidate_baseline/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c50_series_missing_over_yunit_mismatch_candidate_variant/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c51_graphkind_mismatch_over_series_missing_with_ykind_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c52_graphkind_mismatch_over_series_missing_reverse_with_yunit_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c53_xkind_mismatch_over_series_missing_with_ykind_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c54_xunit_mismatch_over_series_missing_reverse_with_yunit_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c55_graph_missing_over_ykind_mismatch_with_series_candidate/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c56_graph_missing_over_yunit_mismatch_reverse_with_series_candidate/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c57_ykind_mismatch_over_series_mismatch_with_y_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c58_yunit_mismatch_over_series_mismatch_reverse_with_y_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c59_graphkind_mismatch_over_series_mismatch_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c60_xkind_mismatch_over_series_mismatch_reverse_with_axis_kind_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c61_graphkind_mismatch_over_series_mismatch_reverse_with_meta_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c62_xunit_mismatch_over_series_mismatch_with_axis_unit_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c63_xunit_mismatch_over_series_mismatch_reverse_with_axis_unit_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c64_ykind_mismatch_over_series_mismatch_reverse_with_y_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c65_yunit_mismatch_over_series_mismatch_with_y_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c66_xkind_mismatch_over_series_mismatch_with_axis_kind_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c67_xunit_mismatch_over_yunit_and_series_mismatch_with_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c68_xunit_mismatch_over_yunit_and_series_mismatch_reverse_with_fallback/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c69_graphkind_mismatch_over_all_axis_and_series_mismatch/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c70_xkind_mismatch_over_remaining_axis_and_series_mismatch/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c71_xunit_mismatch_over_ykind_yunit_and_series_mismatch/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c72_ykind_mismatch_over_yunit_and_series_mismatch/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c73_yunit_mismatch_over_series_mismatch_with_full_fallback_chain/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c74_yunit_mismatch_over_series_mismatch_reverse_with_full_fallback_chain/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c75_graph_missing_over_series_missing_with_full_fallback_chain/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c76_graph_missing_over_series_missing_reverse_with_full_fallback_chain/case.detjson"),
    Path("pack/seamgrim_overlay_param_compare_v0/c77_group_id_normalized_equal_ok/case.detjson"),
]

AGE5_SURFACE_PACK_CONTRACTS = [
    {
        "name": "bogae_madang_alias",
        "golden": Path("pack/seamgrim_bogae_madang_alias_v1/golden.jsonl"),
        "min_cases": 2,
        "tokens": [
            "c01_alias_warn",
            "W_BOGAE_MADANG_ALIAS_DEPRECATED",
            "c02_canonical_no_warn",
        ],
    },
    {
        "name": "moyang_template_instance_view_boundary",
        "golden": Path("pack/seamgrim_moyang_template_instance_view_boundary_v1/golden.jsonl"),
        "min_cases": 5,
        "tokens": [
            "smoke_template_instance_a.v1.json",
            "smoke_template_instance_b.v1.json",
            "smoke_template_instance_c.v1.json",
            "input_template_instance_a.ddn",
            "input_template_instance_b.ddn",
        ],
    },
    {
        "name": "jjaim_block_stub_canon",
        "golden": Path("pack/seamgrim_jjaim_block_stub_canon_v1/golden.jsonl"),
        "min_cases": 6,
        "tokens": [
            "c01_guseong_alias",
            "W_JJAIM_ALIAS_DEPRECATED",
            "c02_invalid_subblock_EXPECT_FAIL",
            "E_JJAIM_SUBBLOCK_INVALID",
            "c06_jjaim_canonical_no_warn",
        ],
    },
    {
        "name": "guseong_flatten_ir",
        "golden": Path("pack/seamgrim_guseong_flatten_ir_v1/golden.jsonl"),
        "min_cases": 2,
        "tokens": [
            "c01_basic",
            "c02_guseong_alias_warn",
            "W_JJAIM_ALIAS_DEPRECATED",
        ],
    },
    {
        "name": "event_surface_canon",
        "golden": Path("pack/seamgrim_event_surface_canon_v1/golden.jsonl"),
        "min_cases": 7,
        "tokens": [
            "c01_canon",
            "c06_kind_noun_alias_EXPECT_FAIL",
            "E_EVENT_SURFACE_ALIAS_FORBIDDEN",
        ],
    },
    {
        "name": "event_model_ir",
        "golden": Path("pack/seamgrim_event_model_ir_v1/golden.jsonl"),
        "min_cases": 2,
        "tokens": [
            "c01_basic",
            "c02_alias_surface_emit_EXPECT_FAIL",
            "E_EVENT_SURFACE_ALIAS_FORBIDDEN",
            "alrim-plan-json",
        ],
    },
    {
        "name": "block_header_no_colon",
        "golden": Path("pack/block_header_no_colon/golden.jsonl"),
        "min_cases": 5,
        "tokens": [
            "c01_decl_colon_warn",
            "c05_mixed_colon_no_colon_warn",
            "W_BLOCK_HEADER_COLON_DEPRECATED",
        ],
    },
    {
        "name": "exec_policy_effect_map",
        "golden": Path("pack/seamgrim_exec_policy_effect_map_v1/golden.jsonl"),
        "min_cases": 6,
        "tokens": [
            "c01_general_allowed",
            "c03_duplicate_exec_policy_blocks",
            "c06_effect_only_allowed_defaults_strict",
        ],
    },
    {
        "name": "exec_policy_effect_diag",
        "golden": Path("pack/seamgrim_exec_policy_effect_diag_v1/golden.jsonl"),
        "min_cases": 26,
        "tokens": [
            "c01_strict_effect_call_EXPECT_FAIL",
            "E_EFFECT_IN_STRICT_MODE",
            "c22_strict_effect_policy_ignored_warn",
            "W_EFFECT_POLICY_IGNORED_IN_STRICT",
            "c26_effect_block_alias_baggat_EXPECT_FAIL",
            "E_EFFECT_SURFACE_ALIAS_FORBIDDEN",
        ],
    },
    {
        "name": "guideblock_keys_basics",
        "golden": Path("pack/guideblock_keys_basics/golden.jsonl"),
        "min_cases": 4,
        "tokens": [
            "c01_hash_header_alias/case.detjson",
            "c02_guideblock_alias/case.detjson",
            "c03_mixed_precedence/case.detjson",
            "c04_canonical_keys/case.detjson",
        ],
    },
]

CI_PROFILE_GATE_SCRIPTS = {
    "split": Path("tests/run_ci_profile_split_contract_check.py"),
    "core_lang": Path("tests/run_ci_profile_core_lang_gate.py"),
    "full": Path("tests/run_ci_profile_full_gate.py"),
    "seamgrim": Path("tests/run_ci_profile_seamgrim_gate.py"),
}
CI_PROFILE_MATRIX_FULL_REAL_SMOKE_SCRIPT = Path(PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SCRIPT)
CI_PROFILE_CORE_LANG_RUNTIME_HELPER_CONTRACT_SELFTEST_SCRIPT = Path(
    "tests/run_ci_profile_core_lang_runtime_helper_contract_selftest.py"
)
CI_PROFILE_CORE_LANG_GROUP_ID_SUMMARY_CONTRACT_SELFTEST_SCRIPT = Path(
    "tests/run_ci_profile_core_lang_group_id_summary_contract_selftest.py"
)
CI_PROFILE_RUNTIME_HELPER_MISMATCH_KEY_ENV = "DDN_CI_PROFILE_GATE_FORCE_RUNTIME_HELPER_SUMMARY_MISMATCH_KEY"
CI_PROFILE_CORE_LANG_RUNTIME_HELPER_MISMATCH_TARGET_KEY = "ci_sanity_age5_combined_heavy_policy_selftest_ok"

CI_PROFILE_CORE_LANG_CHAIN_TOKENS = [
    "tests/run_ci_sanity_gate.py",
    "tests/run_ci_sync_readiness_check.py",
    "tests/run_ci_sync_readiness_report_check.py",
    "RUNTIME_HELPER_SUMMARY_SELFTEST_MISMATCH_ENV",
    "RUNTIME_HELPER_SUMMARY_SELFTEST_MISMATCH_KEY_ENV",
    "RUNTIME_HELPER_SUMMARY_SELFTEST_MARKER_PREFIX",
    "maybe_force_runtime_helper_summary_mismatch(",
    "--quick",
    PROFILE_MATRIX_FULL_REAL_SMOKE_FLAG,
    "--skip-aggregate",
    "--sanity-profile",
    "--require-pass",
    "DDN_CI_PROFILE_GATE_SKIP_AGGREGATE",
    "from _ci_profile_matrix_full_real_smoke_contract import (",
    "PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY",
    "PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SCRIPT",
    "PROFILE_MATRIX_FULL_REAL_SMOKE_STATUS_PASS",
    "[ci-profile-core-lang] aggregate gate skipped by --quick",
    "[ci-profile-core-lang] aggregate gate skipped by DDN_CI_PROFILE_GATE_SKIP_AGGREGATE=1",
    "[ci-profile-core-lang] profile-matrix full-real smoke enabled",
    "ci_sync_readiness_status=pass",
    "sanity_profile=core_lang",
    "ci_profile_core_lang_status=pass",
    "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok",
    "ci_profile_core_lang_status=fail reason=aggregate_summary_sync_pack_golden_graph_export_mismatch",
    "ci_sanity_seamgrim_numeric_factor_policy_ok",
    "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok",
    "ci_profile_core_lang_status=fail reason=aggregate_summary_sanity_numeric_factor_policy_mismatch",
    "ci_profile_core_lang_status=fail reason=aggregate_summary_sync_numeric_factor_policy_mismatch",
    "ci_sanity_age5_combined_heavy_policy_selftest_ok",
]

CI_PROFILE_SEAMGRIM_CHAIN_TOKENS = [
    "tests/run_ci_sanity_gate.py",
    "tests/run_ci_sync_readiness_check.py",
    "tests/run_ci_sync_readiness_report_check.py",
    "--quick",
    PROFILE_MATRIX_FULL_REAL_SMOKE_FLAG,
    "--skip-aggregate",
    "--sanity-profile",
    "--require-pass",
    "DDN_CI_PROFILE_GATE_SKIP_AGGREGATE",
    "from _ci_profile_matrix_full_real_smoke_contract import (",
    "PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY",
    "PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SCRIPT",
    "PROFILE_MATRIX_FULL_REAL_SMOKE_STATUS_PASS",
    "[ci-profile-seamgrim] aggregate gate skipped by --quick",
    "[ci-profile-seamgrim] aggregate gate skipped by DDN_CI_PROFILE_GATE_SKIP_AGGREGATE=1",
    "[ci-profile-seamgrim] profile-matrix full-real smoke enabled",
    "ci_sync_readiness_status=pass",
    "sanity_profile=seamgrim",
    "ci_profile_seamgrim_status=pass",
    "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok",
    "ci_profile_seamgrim_status=fail reason=aggregate_summary_sync_pack_golden_graph_export_mismatch",
    "ci_sanity_seamgrim_numeric_factor_policy_ok",
    "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok",
    "ci_profile_seamgrim_status=fail reason=aggregate_summary_sanity_numeric_factor_policy_mismatch",
    "ci_profile_seamgrim_status=fail reason=aggregate_summary_sync_numeric_factor_policy_mismatch",
]

CI_PROFILE_FULL_CHAIN_TOKENS = [
    "tests/run_ci_sanity_gate.py",
    "tests/run_ci_sync_readiness_check.py",
    "tests/run_ci_sync_readiness_report_check.py",
    "--quick",
    PROFILE_MATRIX_FULL_REAL_SMOKE_FLAG,
    "--skip-aggregate",
    "--sanity-profile",
    "--require-pass",
    "DDN_CI_PROFILE_GATE_SKIP_AGGREGATE",
    "from _ci_profile_matrix_full_real_smoke_contract import (",
    "PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY",
    "PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SCRIPT",
    "PROFILE_MATRIX_FULL_REAL_SMOKE_STATUS_PASS",
    "[ci-profile-full] aggregate gate skipped by --quick",
    "[ci-profile-full] aggregate gate skipped by DDN_CI_PROFILE_GATE_SKIP_AGGREGATE=1",
    "[ci-profile-full] profile-matrix full-real smoke enabled",
    "ci_sync_readiness_status=pass",
    "sanity_profile=full",
    "ci_profile_full_status=pass",
    "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok",
    "ci_profile_full_status=fail reason=aggregate_summary_sync_pack_golden_graph_export_mismatch",
    "ci_sanity_seamgrim_numeric_factor_policy_ok",
    "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok",
    "ci_profile_full_status=fail reason=aggregate_summary_sanity_numeric_factor_policy_mismatch",
    "ci_profile_full_status=fail reason=aggregate_summary_sync_numeric_factor_policy_mismatch",
]

CI_PROFILE_SPLIT_CONTRACT_TOKENS = [
    "tests/run_ci_profile_core_lang_gate.py",
    "tests/run_ci_profile_full_gate.py",
    "tests/run_ci_profile_seamgrim_gate.py",
    "tests/run_ci_profile_core_lang_runtime_helper_contract_selftest.py",
    "tests/run_ci_profile_gate_runtime_helper_contract_selftest.py",
    "PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SCRIPT",
    "tests/run_ci_profile_matrix_gate.py",
    "tests/run_ci_profile_matrix_gate_selftest.py",
    "tests/run_ci_sync_readiness_check.py",
    "tests/run_ci_sync_readiness_report_check.py",
    "--quick",
    "PROFILE_MATRIX_FULL_REAL_SMOKE_FLAG",
    "--quick-gates",
    "--skip-aggregate",
    "--sanity-profile",
    "--require-pass",
    "DDN_CI_PROFILE_GATE_SKIP_AGGREGATE",
    "PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY",
    "PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SCRIPT",
    "PROFILE_MATRIX_FULL_REAL_SMOKE_STATUS_PASS",
    "DDN_CI_PROFILE_MATRIX_QUICK_GATES",
    "quick_enabled_profiles",
    "quick_disabled_profiles",
    "quick_profile_flags",
    "quick_profile_count",
    "quick_profile_flags_complete",
    "quick_enabled_profiles_mismatch",
    "quick_profile_count_mismatch",
    "quick_profile_flags_complete_mismatch",
    "ci_sync_readiness_status=pass",
    "ci_sanity_pack_golden_graph_export_ok",
    "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok",
    "ci_sanity_seamgrim_numeric_factor_policy_ok",
    "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok",
    "sanity_profile=core_lang",
    "sanity_profile=full",
    "sanity_profile=seamgrim",
    "ddn.ci.profile_matrix_gate.v1",
]


def truthy_env(name: str) -> bool:
    return str(os.environ.get(name, "")).strip().lower() in {"1", "true", "yes", "on"}

CI_PROFILE_CORE_LANG_REPORT_PATH_TOKENS = [
    'prefix = args.report_prefix.strip() or "ci_profile_core_lang"',
    'report = report_dir / f"{prefix}.ci_sync_readiness.detjson"',
]

CI_PROFILE_SEAMGRIM_REPORT_PATH_TOKENS = [
    'prefix = args.report_prefix.strip() or "ci_profile_seamgrim"',
    'report = report_dir / f"{prefix}.ci_sync_readiness.detjson"',
]

CI_PROFILE_FULL_REPORT_PATH_TOKENS = [
    'prefix = args.report_prefix.strip() or "ci_profile_full"',
    'report = report_dir / f"{prefix}.ci_sync_readiness.detjson"',
]

CI_SYNC_READINESS_REPORT_PATH_CONTRACT_SCRIPT = Path("tests/run_ci_sync_readiness_check.py")
CI_SYNC_READINESS_REPORT_PATH_CONTRACT_TOKENS = [
    "--report-prefix",
    "--json-out",
    "ci_sync_readiness.detjson",
    "out_path = Path(args.json_out)",
]

CI_GATE_REPORT_INDEX_CONTRACT_SCRIPT = Path("tests/run_ci_aggregate_gate.py")
CI_GATE_REPORT_INDEX_CHECK_SCRIPT = Path("tests/run_ci_gate_report_index_check.py")
CI_GATE_REPORT_INDEX_SELFTEST_SCRIPT = Path("tests/run_ci_gate_report_index_check_selftest.py")
CI_GATE_REPORT_INDEX_DIAGNOSTICS_SCRIPT = Path("tests/run_ci_gate_report_index_diagnostics_check.py")
CI_GATE_REPORT_INDEX_CODE_MAP = Path("tests/ci_check_error_codes.py")
CI_GATE_REPORT_INDEX_CONTRACT_TOKENS = [
    "check_ci_gate_report_index",
    "check_ci_gate_report_index_selftest",
    "check_ci_gate_report_index_diagnostics",
    "check_ci_gate_report_index_latest_smoke",
    "check_ci_emit_artifacts_required_post_summary",
    "ci_emit_artifacts_required_post_summary_check",
    "allow-triage-exists-upgrade",
    "report_index_required_steps_common",
    "report_index_required_steps_seamgrim",
    "resolve_report_index_required_steps",
    "report_index_required_steps",
    "require_step_contract",
    "--sanity-profile",
    "--enforce-profile-step-contract",
    "--required-step",
    "check_ci_gate_report_index(require_step_contract=False)",
    "check_ci_gate_report_index(require_step_contract=True)",
    "report-index post-summary strict check failed",
    "ci_gate_report_index_check",
    "ci_gate_report_index_selftest",
    "ci_gate_report_index_diagnostics_check",
    "ci_gate_report_index_latest_smoke_check",
    "tests/run_ci_gate_report_index_check.py",
    "tests/run_ci_gate_report_index_check_selftest.py",
    "tests/run_ci_gate_report_index_diagnostics_check.py",
    "tests/run_ci_gate_report_index_latest_smoke_check.py",
]
CI_GATE_REPORT_INDEX_CHECK_TOKENS = [
    "INDEX_SCHEMA = \"ddn.ci.aggregate_gate.index.v1\"",
    "VALID_SANITY_PROFILES",
    "PROFILE_REQUIRED_STEPS_COMMON",
    "\"ci_emit_artifacts_required_post_summary_check\"",
    "PROFILE_REQUIRED_STEPS_SEAMGRIM",
    "resolve_profile_required_steps",
    "--sanity-profile",
    "--enforce-profile-step-contract",
    "--required-step",
    "invalid ci_sanity_profile",
    "ci_sanity_profile mismatch",
    "invalid sanity profile in ci_sanity_gate",
    "ci_sanity_gate profile mismatch",
    "invalid sanity_profile in ci_sync_readiness",
    "ci_sync_readiness sanity_profile mismatch",
    "final_status_parse parsed missing",
    "final_status_parse status_line_path missing",
    "final_status_parse status_line_path not found",
    "final_status_parse status mismatch",
    "final_status_parse overall_ok invalid",
    "final_status_parse overall_ok mismatch",
    "final_status_parse aggregate_status invalid",
    "final_status_parse failed_steps must be int string",
    "final_status_parse failed_steps mismatch",
    "AGE4_PROOF_FAILED_PREVIEW_KEY = \"age4_proof_failed_preview\"",
    "final_parse_age4_proof_preview = str(final_parse_parsed.get(AGE4_PROOF_FAILED_PREVIEW_KEY, \"\")).strip()",
    "f\"final_status_parse {AGE4_PROOF_FAILED_PREVIEW_KEY} missing\"",
    "f\"final_status_parse {AGE4_PROOF_FAILED_PREVIEW_KEY} mismatch expected=",
    "index.overall_ok must be bool",
    "index.overall_ok mismatch expected=",
    "ci_gate_result overall_ok must be bool",
    "ci_gate_result overall_ok mismatch",
    "ci_gate_result ok must be bool",
    "ci_gate_result ok mismatch",
    "ci_gate_result failed_steps must be int",
    "ci_gate_result failed_steps mismatch",
    "ci_gate_result status mismatch",
    "ci_gate_result aggregate_status invalid",
    "ci_gate_result aggregate_status mismatch",
    "ci_gate_result summary_line_path mismatch",
    "ci_gate_result summary_line mismatch",
    "ci_gate_result gate_index_path mismatch",
    "ci_gate_result final_status_parse_path mismatch",
    "AGE4_PROOF_FAILED_PREVIEW_KEY,",
    "f\"ci_fail_brief missing key: {key}\"",
    "f\"ci_fail_brief invalid {key}: {brief_value}\"",
    "ci_gate_badge status mismatch",
    "ci_gate_badge ok must be bool",
    "ci_gate_badge ok mismatch",
    "ci_gate_badge result_path mismatch",
    "ci_fail_triage status mismatch",
    "ci_fail_triage reason mismatch",
    "ci_fail_triage summary_report_path_hint_norm mismatch",
    "ci_fail_triage artifacts missing",
    "ci_fail_triage artifacts path mismatch",
    "ci_fail_triage artifacts path_norm mismatch",
    "ci_fail_triage artifacts exists mismatch",
    "index.steps is missing",
    ".ok must be bool",
    "ok/returncode mismatch",
    ".returncode must be int",
    ".cmd must be list",
    ".cmd must not be empty",
    ".cmd[*] must be non-empty string",
    "missing required index step(s)",
    "\"seamgrim_wasm_cli_diag_parity\"",
    "\"ddn.seamgrim.wasm_cli_diag_parity.v1\"",
    "GATE_REPORT_INDEX_CODES",
]
CI_GATE_REPORT_INDEX_CODE_MAP_TOKENS = [
    "GATE_REPORT_INDEX_CODES",
    "\"INDEX_MISSING\": \"E_GATE_INDEX_MISSING\"",
    "\"PROFILE_INVALID\": \"E_GATE_INDEX_PROFILE_INVALID\"",
    "\"PROFILE_MISMATCH\": \"E_GATE_INDEX_PROFILE_MISMATCH\"",
    "\"SANITY_PROFILE_INVALID\": \"E_GATE_INDEX_SANITY_PROFILE_INVALID\"",
    "\"SANITY_PROFILE_MISMATCH\": \"E_GATE_INDEX_SANITY_PROFILE_MISMATCH\"",
    "\"SYNC_PROFILE_INVALID\": \"E_GATE_INDEX_SYNC_PROFILE_INVALID\"",
    "\"SYNC_PROFILE_MISMATCH\": \"E_GATE_INDEX_SYNC_PROFILE_MISMATCH\"",
    "\"FINAL_PARSE_PARSED_MISSING\": \"E_GATE_INDEX_FINAL_PARSE_PARSED_MISSING\"",
    "\"FINAL_PARSE_STATUS_LINE_PATH_MISSING\": \"E_GATE_INDEX_FINAL_PARSE_STATUS_LINE_PATH_MISSING\"",
    "\"FINAL_PARSE_STATUS_LINE_PATH_NOT_FOUND\": \"E_GATE_INDEX_FINAL_PARSE_STATUS_LINE_PATH_NOT_FOUND\"",
    "\"FINAL_PARSE_STATUS_MISMATCH\": \"E_GATE_INDEX_FINAL_PARSE_STATUS_MISMATCH\"",
    "\"FINAL_PARSE_OVERALL_OK_INVALID\": \"E_GATE_INDEX_FINAL_PARSE_OVERALL_OK_INVALID\"",
    "\"FINAL_PARSE_OVERALL_OK_MISMATCH\": \"E_GATE_INDEX_FINAL_PARSE_OVERALL_OK_MISMATCH\"",
    "\"FINAL_PARSE_AGGREGATE_STATUS_INVALID\": \"E_GATE_INDEX_FINAL_PARSE_AGGREGATE_STATUS_INVALID\"",
    "\"FINAL_PARSE_FAILED_STEPS_TYPE\": \"E_GATE_INDEX_FINAL_PARSE_FAILED_STEPS_TYPE\"",
    "\"FINAL_PARSE_FAILED_STEPS_MISMATCH\": \"E_GATE_INDEX_FINAL_PARSE_FAILED_STEPS_MISMATCH\"",
    "\"INDEX_OVERALL_OK_TYPE\": \"E_GATE_INDEX_OVERALL_OK_TYPE\"",
    "\"INDEX_OVERALL_OK_STEPS_MISMATCH\": \"E_GATE_INDEX_OVERALL_OK_STEPS_MISMATCH\"",
    "\"RESULT_OVERALL_OK_TYPE\": \"E_GATE_INDEX_RESULT_OVERALL_OK_TYPE\"",
    "\"RESULT_OVERALL_OK_MISMATCH\": \"E_GATE_INDEX_RESULT_OVERALL_OK_MISMATCH\"",
    "\"RESULT_OK_TYPE\": \"E_GATE_INDEX_RESULT_OK_TYPE\"",
    "\"RESULT_OK_MISMATCH\": \"E_GATE_INDEX_RESULT_OK_MISMATCH\"",
    "\"RESULT_FAILED_STEPS_TYPE\": \"E_GATE_INDEX_RESULT_FAILED_STEPS_TYPE\"",
    "\"RESULT_FAILED_STEPS_MISMATCH\": \"E_GATE_INDEX_RESULT_FAILED_STEPS_MISMATCH\"",
    "\"RESULT_STATUS_MISMATCH\": \"E_GATE_INDEX_RESULT_STATUS_MISMATCH\"",
    "\"RESULT_AGGREGATE_STATUS_INVALID\": \"E_GATE_INDEX_RESULT_AGGREGATE_STATUS_INVALID\"",
    "\"RESULT_AGGREGATE_STATUS_MISMATCH\": \"E_GATE_INDEX_RESULT_AGGREGATE_STATUS_MISMATCH\"",
    "\"RESULT_SUMMARY_LINE_PATH_MISMATCH\": \"E_GATE_INDEX_RESULT_SUMMARY_LINE_PATH_MISMATCH\"",
    "\"RESULT_SUMMARY_LINE_MISMATCH\": \"E_GATE_INDEX_RESULT_SUMMARY_LINE_MISMATCH\"",
    "\"RESULT_GATE_INDEX_PATH_MISMATCH\": \"E_GATE_INDEX_RESULT_GATE_INDEX_PATH_MISMATCH\"",
    "\"RESULT_FINAL_STATUS_PARSE_PATH_MISMATCH\": \"E_GATE_INDEX_RESULT_FINAL_STATUS_PARSE_PATH_MISMATCH\"",
    "\"BADGE_OK_TYPE\": \"E_GATE_INDEX_BADGE_OK_TYPE\"",
    "\"BADGE_OK_MISMATCH\": \"E_GATE_INDEX_BADGE_OK_MISMATCH\"",
    "\"BADGE_STATUS_MISMATCH\": \"E_GATE_INDEX_BADGE_STATUS_MISMATCH\"",
    "\"BADGE_RESULT_PATH_MISMATCH\": \"E_GATE_INDEX_BADGE_RESULT_PATH_MISMATCH\"",
    "\"TRIAGE_STATUS_MISMATCH\": \"E_GATE_INDEX_TRIAGE_STATUS_MISMATCH\"",
    "\"TRIAGE_REASON_MISMATCH\": \"E_GATE_INDEX_TRIAGE_REASON_MISMATCH\"",
    "\"TRIAGE_ARTIFACTS_MISSING\": \"E_GATE_INDEX_TRIAGE_ARTIFACTS_MISSING\"",
    "\"TRIAGE_ARTIFACT_PATH_MISMATCH\": \"E_GATE_INDEX_TRIAGE_ARTIFACT_PATH_MISMATCH\"",
    "\"TRIAGE_ARTIFACT_PATH_NORM_MISMATCH\": \"E_GATE_INDEX_TRIAGE_ARTIFACT_PATH_NORM_MISMATCH\"",
    "\"TRIAGE_ARTIFACT_EXISTS_MISMATCH\": \"E_GATE_INDEX_TRIAGE_ARTIFACT_EXISTS_MISMATCH\"",
    "\"TRIAGE_SUMMARY_HINT_NORM_MISMATCH\": \"E_GATE_INDEX_TRIAGE_SUMMARY_HINT_NORM_MISMATCH\"",
    "\"STEP_ROW_TYPE\": \"E_GATE_INDEX_STEP_ROW_TYPE\"",
    "\"STEP_OK_RC_MISMATCH\": \"E_GATE_INDEX_STEP_OK_RC_MISMATCH\"",
    "\"STEP_CMD_EMPTY\": \"E_GATE_INDEX_STEP_CMD_EMPTY\"",
    "\"STEP_CMD_ITEM_TYPE\": \"E_GATE_INDEX_STEP_CMD_ITEM_TYPE\"",
    "\"REQUIRED_STEP_MISSING\": \"E_GATE_INDEX_REQUIRED_STEP_MISSING\"",
    "\"ARTIFACT_SCHEMA_MISMATCH\": \"E_GATE_INDEX_ARTIFACT_SCHEMA_MISMATCH\"",
]

CI_SEAMGRIM_DIAG_PARITY_SCRIPTS = {
    "wasm_cli": Path("tests/run_seamgrim_wasm_cli_diag_parity_check.py"),
    "overlay_compare": Path("tests/run_seamgrim_overlay_compare_diag_parity_check.py"),
    "overlay_session": Path("tests/run_seamgrim_overlay_session_diag_parity_check.py"),
    "overlay_session_wired": Path("tests/run_seamgrim_overlay_session_wired_consistency_check.py"),
}

CI_SEAMGRIM_WASM_CLI_DIAG_PARITY_TOKENS = [
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


def clip(text: str, limit: int = 140) -> str:
    normalized = " ".join(text.split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


def sample_items(items: list[str], limit: int = 2) -> str:
    if not items:
        return "-"
    return ",".join(items[:limit])


def full_items(items: list[str]) -> str:
    if not items:
        return "-"
    return ",".join(items)


def sample_window(items: list[str], index: int, radius: int = 1) -> str:
    if not items:
        return "-"
    if index < 0:
        return sample_items(items, limit=3)
    start = max(0, index - radius)
    end = min(len(items), index + radius + 1)
    return ",".join(items[start:end])


def build_order_repair_hint(pack_golden_path: Path, expected_refs: list[str]) -> str:
    return (
        f"repair_hint: reorder {pack_golden_path} to PACK_CASE_FILES order. "
        f"first={sample_items(expected_refs, 3)} last={sample_items(expected_refs[-3:], 3)}"
    )


def build_order_repair_cmd(pack_golden_path: Path, expected_refs: list[str]) -> str:
    # command example for deterministic golden.jsonl reordering to PACK_CASE_FILES order
    script = (
        "import json;from pathlib import Path;"
        f"refs={repr(expected_refs)};"
        f"p=Path({repr(str(pack_golden_path).replace('\\\\', '/'))});"
        "p.write_text('\\n'.join(json.dumps({'overlay_compare_case': r}, ensure_ascii=False) for r in refs)+'\\n', encoding='utf-8')"
    )
    return "python -c " + json.dumps(script, ensure_ascii=False)


def build_order_repair_cmd_short(pack_golden_path: Path) -> str:
    script = (
        "import json;from pathlib import Path;"
        f"p=Path({repr(str(pack_golden_path).replace('\\\\', '/'))});"
        "rows=[json.loads(x) for x in p.read_text(encoding='utf-8').splitlines() if x.strip()];"
        "rows.sort(key=lambda r:r.get('overlay_compare_case',''));"
        "p.write_text('\\n'.join(json.dumps(r, ensure_ascii=False) for r in rows)+'\\n', encoding='utf-8')"
    )
    return "python -c " + json.dumps(script, ensure_ascii=False)


def head_tail(items: list[str], size: int = 3) -> str:
    if not items:
        return "head=- tail=-"
    head = ",".join(items[:size])
    tail = ",".join(items[-size:])
    return f"head={head} tail={tail}"


def default_report_path(file_name: str) -> str:
    preferred = Path("I:/home/urihanl/ddn/codex/build/reports")
    if os.name == "nt":
        try:
            preferred.mkdir(parents=True, exist_ok=True)
        except OSError:
            pass
        return str(preferred / file_name)
    return f"build/reports/{file_name}"


def load_text(path: Path) -> str:
    if not path.exists():
        return ""
    try:
        return path.read_text(encoding="utf-8")
    except Exception:
        return ""


def run_text(
    cmd: list[str],
    cwd: Path,
    env: dict[str, str] | None = None,
    timeout_sec: int | None = None,
) -> subprocess.CompletedProcess[str]:
    try:
        return subprocess.run(
            cmd,
            cwd=cwd,
            env=env,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=timeout_sec if timeout_sec and timeout_sec > 0 else None,
        )
    except subprocess.TimeoutExpired as exc:
        stdout = str(exc.stdout or "")
        stderr = str(exc.stderr or "")
        timeout_note = (
            "[age5-close-combined-heavy] child timeout "
            f"timeout_sec={timeout_sec if timeout_sec and timeout_sec > 0 else 0} "
            f"cmd={' '.join(cmd)}"
        )
        merged_stdout = "\n".join(part for part in (stdout.strip(), timeout_note) if part).strip()
        merged_stderr = "\n".join(part for part in (stderr.strip(), "timeout_expired") if part).strip()
        return subprocess.CompletedProcess(
            cmd,
            124,
            stdout=merged_stdout,
            stderr=merged_stderr,
        )


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def cached_age5_close_child_report_ok(path: Path, required_criterion: str) -> bool:
    doc = load_json(path)
    if not isinstance(doc, dict):
        return False
    if str(doc.get("schema", "")).strip() != "ddn.age5_close_report.v1":
        return False
    if not bool(doc.get("overall_ok", False)):
        return False
    criteria = doc.get("criteria")
    if not isinstance(criteria, list):
        return False
    for row in criteria:
        if not isinstance(row, dict):
            continue
        if str(row.get("name", "")).strip() != required_criterion:
            continue
        return bool(row.get("ok", False))
    return False


def child_report_indicates_timeout(path: Path, criterion_name: str) -> bool:
    doc = load_json(path)
    if not isinstance(doc, dict):
        return False
    criteria = doc.get("criteria")
    if not isinstance(criteria, list):
        return False
    for row in criteria:
        if not isinstance(row, dict):
            continue
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        if "rc=124" in detail or "step timeout after" in detail:
            return True
    failure_digest = doc.get("failure_digest")
    if isinstance(failure_digest, list):
        joined = " | ".join(str(item).strip() for item in failure_digest if str(item).strip())
        if "rc=124" in joined or "step timeout after" in joined:
            return True
    return False


def parse_timeout_step_profiles(*texts: str) -> dict[str, str]:
    step = "-"
    profiles = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        if step == "-":
            match = re.search(r"\bstep=([A-Za-z0-9_.-]+)", normalized)
            if match:
                step = match.group(1)
        if profiles == "-":
            match = re.search(r"\bprofiles=([A-Za-z0-9_,.-]+)", normalized)
            if match:
                profiles = match.group(1)
    return build_age5_full_real_timeout_breakdown(
        age5_full_real_timeout_step=step,
        age5_full_real_timeout_profiles=profiles,
        age5_full_real_timeout_present=(step != "-" or profiles != "-"),
    )


def load_report_criterion_detail(path: Path, criterion_name: str) -> tuple[str, str]:
    doc = load_json(path)
    detail = ""
    failure_digest_joined = ""
    if isinstance(doc, dict):
        criteria = doc.get("criteria")
        if isinstance(criteria, list):
            for row in criteria:
                if not isinstance(row, dict):
                    continue
                if str(row.get("name", "")).strip() != criterion_name:
                    continue
                detail = str(row.get("detail", "")).strip()
                break
        failure_digest = doc.get("failure_digest")
        if isinstance(failure_digest, list):
            failure_digest_joined = " | ".join(
                str(item).strip() for item in failure_digest if str(item).strip()
            )
    return detail, failure_digest_joined


def extract_full_real_timeout_breakdown_from_child_report(path: Path, criterion_name: str) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_timeout_step_profiles(detail, failure_digest_joined)


def extract_full_real_timeout_breakdown_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_timeout_step_profiles(detail, failure_digest_joined)


def parse_full_real_elapsed_summary(*texts: str) -> dict[str, str]:
    total_elapsed_ms = "-"
    slowest_profile = "-"
    slowest_elapsed_ms = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        if total_elapsed_ms == "-":
            match = re.search(r"\bci_profile_matrix_full_real_total_elapsed_ms=([0-9-]+)", normalized)
            if match:
                total_elapsed_ms = match.group(1)
        if slowest_profile == "-":
            match = re.search(r"\bci_profile_matrix_full_real_slowest_profile=([A-Za-z0-9_.-]+)", normalized)
            if match:
                slowest_profile = match.group(1)
        if slowest_elapsed_ms == "-":
            match = re.search(r"\bci_profile_matrix_full_real_slowest_elapsed_ms=([0-9-]+)", normalized)
            if match:
                slowest_elapsed_ms = match.group(1)
    return build_age5_full_real_elapsed_summary(
        age5_full_real_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_slowest_profile=slowest_profile,
        age5_full_real_slowest_elapsed_ms=slowest_elapsed_ms,
        age5_full_real_elapsed_present=(
            total_elapsed_ms != "-" or slowest_profile != "-" or slowest_elapsed_ms != "-"
        ),
    )


def extract_full_real_elapsed_summary_from_child_report(path: Path, criterion_name: str) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_elapsed_summary(detail, failure_digest_joined)


def extract_full_real_elapsed_summary_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_elapsed_summary(detail, failure_digest_joined)


def parse_full_real_profile_elapsed_map(*texts: str) -> dict[str, str]:
    profile_map = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        match = re.search(r"\bci_profile_matrix_full_real_profile_elapsed_map=([A-Za-z0-9:,_-]+)", normalized)
        if match:
            profile_map = match.group(1)
            break
    return build_age5_full_real_profile_elapsed_map(
        age5_full_real_profile_elapsed_map=profile_map,
        age5_full_real_profile_elapsed_map_present=(profile_map != "-"),
    )


def parse_full_real_profile_status_map(*texts: str) -> dict[str, str]:
    profile_status_map = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        match = re.search(r"\bci_profile_matrix_full_real_profile_status_map=([A-Za-z0-9:,_-]+)", normalized)
        if match:
            profile_status_map = match.group(1)
            break
    return build_age5_full_real_profile_status_map(
        age5_full_real_profile_status_map=profile_status_map,
        age5_full_real_profile_status_map_present=(profile_status_map != "-"),
    )


def parse_full_real_core_lang_sanity_elapsed_summary(*texts: str) -> dict[str, str]:
    total_elapsed_ms = "-"
    slowest_step = "-"
    slowest_elapsed_ms = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        if total_elapsed_ms == "-":
            match_total = re.search(r"\bci_profile_core_lang_sanity_total_elapsed_ms=([A-Za-z0-9._-]+)", normalized)
            if match_total:
                total_elapsed_ms = match_total.group(1)
        if slowest_step == "-":
            match_step = re.search(r"\bci_profile_core_lang_sanity_slowest_step=([A-Za-z0-9._-]+)", normalized)
            if match_step:
                slowest_step = match_step.group(1)
        if slowest_elapsed_ms == "-":
            match_elapsed = re.search(r"\bci_profile_core_lang_sanity_slowest_elapsed_ms=([A-Za-z0-9._-]+)", normalized)
            if match_elapsed:
                slowest_elapsed_ms = match_elapsed.group(1)
    return build_age5_full_real_core_lang_sanity_elapsed_summary(
        age5_full_real_core_lang_sanity_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_core_lang_sanity_slowest_step=slowest_step,
        age5_full_real_core_lang_sanity_slowest_elapsed_ms=slowest_elapsed_ms,
        age5_full_real_core_lang_sanity_elapsed_present=(
            total_elapsed_ms != "-" or slowest_step != "-" or slowest_elapsed_ms != "-"
        ),
    )


def parse_full_real_core_lang_sanity_progress(*texts: str) -> dict[str, str]:
    current_step = "-"
    last_completed_step = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_matches = re.findall(r"\bci_sanity_current_step=([A-Za-z0-9._-]+)", normalized)
        completed_matches = re.findall(r"\bci_sanity_last_completed_step=([A-Za-z0-9._-]+)", normalized)
        if current_matches:
            current_step = current_matches[-1]
        if completed_matches:
            last_completed_step = completed_matches[-1]
    return {
        "age5_full_real_core_lang_sanity_current_step": current_step,
        "age5_full_real_core_lang_sanity_last_completed_step": last_completed_step,
        "age5_full_real_core_lang_sanity_progress_present": (
            "1" if current_step != "-" or last_completed_step != "-" else "0"
        ),
    }


def parse_full_real_pipeline_emit_flags_progress(*texts: str) -> dict[str, str]:
    current_section = "-"
    last_completed_section = "-"
    total_elapsed_ms = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_matches = re.findall(r"\bci_pipeline_emit_flags_current_section=([A-Za-z0-9._,-]+)", normalized)
        completed_matches = re.findall(
            r"\bci_pipeline_emit_flags_last_completed_section=([A-Za-z0-9._,-]+)", normalized
        )
        elapsed_matches = re.findall(r"\bci_pipeline_emit_flags_total_elapsed_ms=([A-Za-z0-9._-]+)", normalized)
        if current_matches:
            current_section = current_matches[-1]
        if completed_matches:
            last_completed_section = completed_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
    return build_age5_full_real_pipeline_emit_flags_progress(
        age5_full_real_pipeline_emit_flags_current_section=current_section,
        age5_full_real_pipeline_emit_flags_last_completed_section=last_completed_section,
        age5_full_real_pipeline_emit_flags_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_pipeline_emit_flags_progress_present=(
            current_section != "-" or last_completed_section != "-" or total_elapsed_ms != "-"
        ),
    )


def parse_full_real_pipeline_emit_flags_selftest_progress(*texts: str) -> dict[str, str]:
    current_case = "-"
    last_completed_case = "-"
    total_elapsed_ms = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_matches = re.findall(
            r"\bci_pipeline_emit_flags_selftest_current_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        completed_matches = re.findall(
            r"\bci_pipeline_emit_flags_selftest_last_completed_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        elapsed_matches = re.findall(
            r"\bci_pipeline_emit_flags_selftest_total_elapsed_ms=([A-Za-z0-9._-]+)",
            normalized,
        )
        if current_matches:
            current_case = current_matches[-1]
        if completed_matches:
            last_completed_case = completed_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
    return build_age5_full_real_pipeline_emit_flags_selftest_progress(
        age5_full_real_pipeline_emit_flags_selftest_current_case=current_case,
        age5_full_real_pipeline_emit_flags_selftest_last_completed_case=last_completed_case,
        age5_full_real_pipeline_emit_flags_selftest_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_pipeline_emit_flags_selftest_progress_present=(
            current_case != "-" or last_completed_case != "-" or total_elapsed_ms != "-"
        ),
    )


def parse_full_real_pipeline_emit_flags_selftest_probe(*texts: str) -> dict[str, str]:
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_matches = re.findall(
            r"\bci_pipeline_emit_flags_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        completed_matches = re.findall(
            r"\bci_pipeline_emit_flags_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if current_matches:
            current_probe = current_matches[-1]
        if completed_matches:
            last_completed_probe = completed_matches[-1]
    return build_age5_full_real_pipeline_emit_flags_selftest_probe(
        age5_full_real_pipeline_emit_flags_selftest_current_probe=current_probe,
        age5_full_real_pipeline_emit_flags_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_pipeline_emit_flags_selftest_probe_present=(
            current_probe != "-" or last_completed_probe != "-"
        ),
    )


def parse_full_real_age5_combined_policy_selftest_progress(*texts: str) -> dict[str, str]:
    current_case = "-"
    last_completed_case = "-"
    current_format = "-"
    last_completed_format = "-"
    current_probe = "-"
    last_completed_probe = "-"
    total_elapsed_ms = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_case_matches = re.findall(
            r"\bci_age5_combined_heavy_policy_selftest_current_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_case_matches = re.findall(
            r"\bci_age5_combined_heavy_policy_selftest_last_completed_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_format_matches = re.findall(
            r"\bci_age5_combined_heavy_policy_selftest_current_format=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_format_matches = re.findall(
            r"\bci_age5_combined_heavy_policy_selftest_last_completed_format=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bci_age5_combined_heavy_policy_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bci_age5_combined_heavy_policy_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        elapsed_matches = re.findall(
            r"\bci_age5_combined_heavy_policy_selftest_total_elapsed_ms=([A-Za-z0-9._-]+)",
            normalized,
        )
        if current_case_matches:
            current_case = current_case_matches[-1]
        if last_completed_case_matches:
            last_completed_case = last_completed_case_matches[-1]
        if current_format_matches:
            current_format = current_format_matches[-1]
        if last_completed_format_matches:
            last_completed_format = last_completed_format_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
    progress_present = (
        current_case != "-"
        or last_completed_case != "-"
        or current_format != "-"
        or last_completed_format != "-"
        or current_probe != "-"
        or last_completed_probe != "-"
        or total_elapsed_ms != "-"
    )
    return {
        "age5_full_real_age5_combined_policy_selftest_current_case": current_case,
        "age5_full_real_age5_combined_policy_selftest_last_completed_case": last_completed_case,
        "age5_full_real_age5_combined_policy_selftest_current_format": current_format,
        "age5_full_real_age5_combined_policy_selftest_last_completed_format": last_completed_format,
        "age5_full_real_age5_combined_policy_selftest_current_probe": current_probe,
        "age5_full_real_age5_combined_policy_selftest_last_completed_probe": last_completed_probe,
        "age5_full_real_age5_combined_policy_selftest_total_elapsed_ms": total_elapsed_ms,
        "age5_full_real_age5_combined_policy_selftest_progress_present": "1" if progress_present else "0",
    }


def parse_full_real_profile_matrix_full_real_smoke_policy_selftest_progress(*texts: str) -> dict[str, str]:
    current_case = "-"
    last_completed_case = "-"
    current_format = "-"
    last_completed_format = "-"
    total_elapsed_ms = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_case_matches = re.findall(
            r"\bci_profile_matrix_full_real_smoke_policy_selftest_current_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_case_matches = re.findall(
            r"\bci_profile_matrix_full_real_smoke_policy_selftest_last_completed_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_format_matches = re.findall(
            r"\bci_profile_matrix_full_real_smoke_policy_selftest_current_format=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_format_matches = re.findall(
            r"\bci_profile_matrix_full_real_smoke_policy_selftest_last_completed_format=([A-Za-z0-9._,-]+)",
            normalized,
        )
        elapsed_matches = re.findall(
            r"\bci_profile_matrix_full_real_smoke_policy_selftest_total_elapsed_ms=([A-Za-z0-9._-]+)",
            normalized,
        )
        if current_case_matches:
            current_case = current_case_matches[-1]
        if last_completed_case_matches:
            last_completed_case = last_completed_case_matches[-1]
        if current_format_matches:
            current_format = current_format_matches[-1]
        if last_completed_format_matches:
            last_completed_format = last_completed_format_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
    return build_age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress(
        age5_full_real_profile_matrix_full_real_smoke_policy_selftest_current_case=current_case,
        age5_full_real_profile_matrix_full_real_smoke_policy_selftest_last_completed_case=last_completed_case,
        age5_full_real_profile_matrix_full_real_smoke_policy_selftest_current_format=current_format,
        age5_full_real_profile_matrix_full_real_smoke_policy_selftest_last_completed_format=last_completed_format,
        age5_full_real_profile_matrix_full_real_smoke_policy_selftest_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_present=(
            current_case != "-"
            or last_completed_case != "-"
            or current_format != "-"
            or last_completed_format != "-"
            or total_elapsed_ms != "-"
        ),
    )


def parse_full_real_profile_matrix_full_real_smoke_check_selftest_progress(*texts: str) -> dict[str, str]:
    current_case = "-"
    last_completed_case = "-"
    total_elapsed_ms = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_case_matches = re.findall(
            r"\bci_profile_matrix_full_real_smoke_check_selftest_current_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_case_matches = re.findall(
            r"\bci_profile_matrix_full_real_smoke_check_selftest_last_completed_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        elapsed_matches = re.findall(
            r"\bci_profile_matrix_full_real_smoke_check_selftest_total_elapsed_ms=([A-Za-z0-9._-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bci_profile_matrix_full_real_smoke_check_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bci_profile_matrix_full_real_smoke_check_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if current_case_matches:
            current_case = current_case_matches[-1]
        if last_completed_case_matches:
            last_completed_case = last_completed_case_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress(
        age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_case=current_case,
        age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_case=last_completed_case,
        age5_full_real_profile_matrix_full_real_smoke_check_selftest_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_probe=current_probe,
        age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_present=(
            current_case != "-"
            or last_completed_case != "-"
            or total_elapsed_ms != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress(*texts: str) -> dict[str, str]:
    current_case = "-"
    last_completed_case = "-"
    total_elapsed_ms = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_case_matches = re.findall(
            r"\bci_fixed64_darwin_real_report_readiness_check_selftest_current_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_case_matches = re.findall(
            r"\bci_fixed64_darwin_real_report_readiness_check_selftest_last_completed_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        elapsed_matches = re.findall(
            r"\bci_fixed64_darwin_real_report_readiness_check_selftest_total_elapsed_ms=([A-Za-z0-9._-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bci_fixed64_darwin_real_report_readiness_check_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bci_fixed64_darwin_real_report_readiness_check_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if current_case_matches:
            current_case = current_case_matches[-1]
        if last_completed_case_matches:
            last_completed_case = last_completed_case_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress(
        age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_case=current_case,
        age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_case=last_completed_case,
        age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_probe=current_probe,
        age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_present=(
            current_case != "-"
            or last_completed_case != "-"
            or total_elapsed_ms != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_map_access_contract_check_progress(*texts: str) -> dict[str, str]:
    current_case = "-"
    last_completed_case = "-"
    total_elapsed_ms = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_case_matches = re.findall(
            r"\bci_map_access_contract_check_current_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_case_matches = re.findall(
            r"\bci_map_access_contract_check_last_completed_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        elapsed_matches = re.findall(
            r"\bci_map_access_contract_check_total_elapsed_ms=([A-Za-z0-9._-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bci_map_access_contract_check_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bci_map_access_contract_check_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if current_case_matches:
            current_case = current_case_matches[-1]
        if last_completed_case_matches:
            last_completed_case = last_completed_case_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_map_access_contract_check_progress(
        age5_full_real_map_access_contract_check_current_case=current_case,
        age5_full_real_map_access_contract_check_last_completed_case=last_completed_case,
        age5_full_real_map_access_contract_check_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_map_access_contract_check_current_probe=current_probe,
        age5_full_real_map_access_contract_check_last_completed_probe=last_completed_probe,
        age5_full_real_map_access_contract_check_progress_present=(
            current_case != "-"
            or last_completed_case != "-"
            or total_elapsed_ms != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_tensor_v0_cli_check_progress(*texts: str) -> dict[str, str]:
    current_case = "-"
    last_completed_case = "-"
    total_elapsed_ms = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_case_matches = re.findall(
            r"\bci_tensor_v0_cli_check_current_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_case_matches = re.findall(
            r"\bci_tensor_v0_cli_check_last_completed_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        elapsed_matches = re.findall(
            r"\bci_tensor_v0_cli_check_total_elapsed_ms=([A-Za-z0-9._-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bci_tensor_v0_cli_check_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bci_tensor_v0_cli_check_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if current_case_matches:
            current_case = current_case_matches[-1]
        if last_completed_case_matches:
            last_completed_case = last_completed_case_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_tensor_v0_cli_check_progress(
        age5_full_real_tensor_v0_cli_check_current_case=current_case,
        age5_full_real_tensor_v0_cli_check_last_completed_case=last_completed_case,
        age5_full_real_tensor_v0_cli_check_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_tensor_v0_cli_check_current_probe=current_probe,
        age5_full_real_tensor_v0_cli_check_last_completed_probe=last_completed_probe,
        age5_full_real_tensor_v0_cli_check_progress_present=(
            current_case != "-"
            or last_completed_case != "-"
            or total_elapsed_ms != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_ci_pack_golden_age5_surface_selftest_progress(*texts: str) -> dict[str, str]:
    current_case = "-"
    last_completed_case = "-"
    total_elapsed_ms = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_case_matches = re.findall(
            r"\bci_pack_golden_age5_surface_selftest_current_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_case_matches = re.findall(
            r"\bci_pack_golden_age5_surface_selftest_last_completed_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        elapsed_matches = re.findall(
            r"\bci_pack_golden_age5_surface_selftest_total_elapsed_ms=([A-Za-z0-9._-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bci_pack_golden_age5_surface_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bci_pack_golden_age5_surface_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if current_case_matches:
            current_case = current_case_matches[-1]
        if last_completed_case_matches:
            last_completed_case = last_completed_case_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_ci_pack_golden_age5_surface_selftest_progress(
        age5_full_real_ci_pack_golden_age5_surface_selftest_current_case=current_case,
        age5_full_real_ci_pack_golden_age5_surface_selftest_last_completed_case=last_completed_case,
        age5_full_real_ci_pack_golden_age5_surface_selftest_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_ci_pack_golden_age5_surface_selftest_current_probe=current_probe,
        age5_full_real_ci_pack_golden_age5_surface_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_ci_pack_golden_age5_surface_selftest_progress_present=(
            current_case != "-"
            or last_completed_case != "-"
            or total_elapsed_ms != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_ci_pack_golden_guideblock_selftest_progress(*texts: str) -> dict[str, str]:
    current_case = "-"
    last_completed_case = "-"
    total_elapsed_ms = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_case_matches = re.findall(
            r"\bci_pack_golden_guideblock_selftest_current_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_case_matches = re.findall(
            r"\bci_pack_golden_guideblock_selftest_last_completed_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        elapsed_matches = re.findall(
            r"\bci_pack_golden_guideblock_selftest_total_elapsed_ms=([A-Za-z0-9._-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bci_pack_golden_guideblock_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bci_pack_golden_guideblock_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if current_case_matches:
            current_case = current_case_matches[-1]
        if last_completed_case_matches:
            last_completed_case = last_completed_case_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_ci_pack_golden_guideblock_selftest_progress(
        age5_full_real_ci_pack_golden_guideblock_selftest_current_case=current_case,
        age5_full_real_ci_pack_golden_guideblock_selftest_last_completed_case=last_completed_case,
        age5_full_real_ci_pack_golden_guideblock_selftest_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_ci_pack_golden_guideblock_selftest_current_probe=current_probe,
        age5_full_real_ci_pack_golden_guideblock_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_ci_pack_golden_guideblock_selftest_progress_present=(
            current_case != "-"
            or last_completed_case != "-"
            or total_elapsed_ms != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_ci_pack_golden_exec_policy_selftest_progress(*texts: str) -> dict[str, str]:
    current_case = "-"
    last_completed_case = "-"
    total_elapsed_ms = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_case_matches = re.findall(
            r"\bci_pack_golden_exec_policy_selftest_current_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_case_matches = re.findall(
            r"\bci_pack_golden_exec_policy_selftest_last_completed_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        elapsed_matches = re.findall(
            r"\bci_pack_golden_exec_policy_selftest_total_elapsed_ms=([A-Za-z0-9._-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bci_pack_golden_exec_policy_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bci_pack_golden_exec_policy_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if current_case_matches:
            current_case = current_case_matches[-1]
        if last_completed_case_matches:
            last_completed_case = last_completed_case_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_ci_pack_golden_exec_policy_selftest_progress(
        age5_full_real_ci_pack_golden_exec_policy_selftest_current_case=current_case,
        age5_full_real_ci_pack_golden_exec_policy_selftest_last_completed_case=last_completed_case,
        age5_full_real_ci_pack_golden_exec_policy_selftest_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_ci_pack_golden_exec_policy_selftest_current_probe=current_probe,
        age5_full_real_ci_pack_golden_exec_policy_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_ci_pack_golden_exec_policy_selftest_progress_present=(
            current_case != "-"
            or last_completed_case != "-"
            or total_elapsed_ms != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_ci_pack_golden_jjaim_flatten_selftest_progress(*texts: str) -> dict[str, str]:
    current_case = "-"
    last_completed_case = "-"
    total_elapsed_ms = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_case_matches = re.findall(
            r"\bci_pack_golden_jjaim_flatten_selftest_current_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_case_matches = re.findall(
            r"\bci_pack_golden_jjaim_flatten_selftest_last_completed_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        elapsed_matches = re.findall(
            r"\bci_pack_golden_jjaim_flatten_selftest_total_elapsed_ms=([A-Za-z0-9._-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bci_pack_golden_jjaim_flatten_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bci_pack_golden_jjaim_flatten_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if current_case_matches:
            current_case = current_case_matches[-1]
        if last_completed_case_matches:
            last_completed_case = last_completed_case_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_ci_pack_golden_jjaim_flatten_selftest_progress(
        age5_full_real_ci_pack_golden_jjaim_flatten_selftest_current_case=current_case,
        age5_full_real_ci_pack_golden_jjaim_flatten_selftest_last_completed_case=last_completed_case,
        age5_full_real_ci_pack_golden_jjaim_flatten_selftest_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_ci_pack_golden_jjaim_flatten_selftest_current_probe=current_probe,
        age5_full_real_ci_pack_golden_jjaim_flatten_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_ci_pack_golden_jjaim_flatten_selftest_progress_present=(
            current_case != "-"
            or last_completed_case != "-"
            or total_elapsed_ms != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_ci_pack_golden_event_model_selftest_progress(*texts: str) -> dict[str, str]:
    current_case = "-"
    last_completed_case = "-"
    total_elapsed_ms = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_case_matches = re.findall(
            r"\bci_pack_golden_event_model_selftest_current_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_case_matches = re.findall(
            r"\bci_pack_golden_event_model_selftest_last_completed_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        elapsed_matches = re.findall(
            r"\bci_pack_golden_event_model_selftest_total_elapsed_ms=([A-Za-z0-9._-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bci_pack_golden_event_model_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bci_pack_golden_event_model_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if current_case_matches:
            current_case = current_case_matches[-1]
        if last_completed_case_matches:
            last_completed_case = last_completed_case_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_ci_pack_golden_event_model_selftest_progress(
        age5_full_real_ci_pack_golden_event_model_selftest_current_case=current_case,
        age5_full_real_ci_pack_golden_event_model_selftest_last_completed_case=last_completed_case,
        age5_full_real_ci_pack_golden_event_model_selftest_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_ci_pack_golden_event_model_selftest_current_probe=current_probe,
        age5_full_real_ci_pack_golden_event_model_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_ci_pack_golden_event_model_selftest_progress_present=(
            current_case != "-"
            or last_completed_case != "-"
            or total_elapsed_ms != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_ci_pack_golden_lang_consistency_selftest_progress(*texts: str) -> dict[str, str]:
    current_case = "-"
    last_completed_case = "-"
    total_elapsed_ms = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_case_matches = re.findall(
            r"\bci_pack_golden_lang_consistency_selftest_current_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_case_matches = re.findall(
            r"\bci_pack_golden_lang_consistency_selftest_last_completed_case=([A-Za-z0-9._,-]+)",
            normalized,
        )
        elapsed_matches = re.findall(
            r"\bci_pack_golden_lang_consistency_selftest_total_elapsed_ms=([A-Za-z0-9._-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bci_pack_golden_lang_consistency_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bci_pack_golden_lang_consistency_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if current_case_matches:
            current_case = current_case_matches[-1]
        if last_completed_case_matches:
            last_completed_case = last_completed_case_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_ci_pack_golden_lang_consistency_selftest_progress(
        age5_full_real_ci_pack_golden_lang_consistency_selftest_current_case=current_case,
        age5_full_real_ci_pack_golden_lang_consistency_selftest_last_completed_case=last_completed_case,
        age5_full_real_ci_pack_golden_lang_consistency_selftest_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_ci_pack_golden_lang_consistency_selftest_current_probe=current_probe,
        age5_full_real_ci_pack_golden_lang_consistency_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_ci_pack_golden_lang_consistency_selftest_progress_present=(
            current_case != "-"
            or last_completed_case != "-"
            or total_elapsed_ms != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_w107_golden_index_selftest_progress(*texts: str) -> dict[str, str]:
    active_cases = "-"
    inactive_cases = "-"
    index_codes = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        active_matches = re.findall(r"\bw107_golden_index_selftest_active_cases=([A-Za-z0-9._,-]+)", normalized)
        inactive_matches = re.findall(r"\bw107_golden_index_selftest_inactive_cases=([A-Za-z0-9._,-]+)", normalized)
        index_matches = re.findall(r"\bw107_golden_index_selftest_index_codes=([A-Za-z0-9._,-]+)", normalized)
        current_probe_matches = re.findall(
            r"\bw107_golden_index_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bw107_golden_index_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if active_matches:
            active_cases = active_matches[-1]
        if inactive_matches:
            inactive_cases = inactive_matches[-1]
        if index_matches:
            index_codes = index_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_w107_golden_index_selftest_progress(
        age5_full_real_w107_golden_index_selftest_active_cases=active_cases,
        age5_full_real_w107_golden_index_selftest_inactive_cases=inactive_cases,
        age5_full_real_w107_golden_index_selftest_index_codes=index_codes,
        age5_full_real_w107_golden_index_selftest_current_probe=current_probe,
        age5_full_real_w107_golden_index_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_w107_golden_index_selftest_progress_present=(
            active_cases != "-"
            or inactive_cases != "-"
            or index_codes != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_w107_progress_contract_selftest_progress(*texts: str) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bw107_progress_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bw107_progress_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bw107_progress_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bw107_progress_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bw107_progress_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_w107_progress_contract_selftest_progress(
        age5_full_real_w107_progress_contract_selftest_completed_checks=completed_checks,
        age5_full_real_w107_progress_contract_selftest_total_checks=total_checks,
        age5_full_real_w107_progress_contract_selftest_checks_text=checks_text,
        age5_full_real_w107_progress_contract_selftest_current_probe=current_probe,
        age5_full_real_w107_progress_contract_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_w107_progress_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_age1_immediate_proof_operation_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bage1_immediate_proof_operation_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bage1_immediate_proof_operation_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bage1_immediate_proof_operation_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bage1_immediate_proof_operation_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bage1_immediate_proof_operation_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_age1_immediate_proof_operation_contract_selftest_progress(
        age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks=completed_checks,
        age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks=total_checks,
        age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text=checks_text,
        age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe=current_probe,
        age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bproof_certificate_v1_consumer_transport_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bproof_certificate_v1_consumer_transport_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bproof_certificate_v1_consumer_transport_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bproof_certificate_v1_consumer_transport_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bproof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress(
        age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks=completed_checks,
        age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks=total_checks,
        age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text=checks_text,
        age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe=current_probe,
        age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bproof_certificate_v1_verify_report_digest_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bproof_certificate_v1_verify_report_digest_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bproof_certificate_v1_verify_report_digest_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bproof_certificate_v1_verify_report_digest_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bproof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress(
        age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks=completed_checks,
        age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks=total_checks,
        age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text=checks_text,
        age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe=current_probe,
        age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_proof_certificate_v1_family_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bproof_certificate_v1_family_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bproof_certificate_v1_family_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bproof_certificate_v1_family_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bproof_certificate_v1_family_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bproof_certificate_v1_family_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_proof_certificate_v1_family_contract_selftest_progress(
        age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks=completed_checks,
        age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks=total_checks,
        age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text=checks_text,
        age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe=current_probe,
        age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_proof_certificate_family_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bproof_certificate_family_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bproof_certificate_family_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bproof_certificate_family_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bproof_certificate_family_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bproof_certificate_family_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_proof_certificate_family_contract_selftest_progress(
        age5_full_real_proof_certificate_family_contract_selftest_completed_checks=completed_checks,
        age5_full_real_proof_certificate_family_contract_selftest_total_checks=total_checks,
        age5_full_real_proof_certificate_family_contract_selftest_checks_text=checks_text,
        age5_full_real_proof_certificate_family_contract_selftest_current_probe=current_probe,
        age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_proof_certificate_family_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_proof_certificate_family_transport_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bproof_certificate_family_transport_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bproof_certificate_family_transport_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bproof_certificate_family_transport_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bproof_certificate_family_transport_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bproof_certificate_family_transport_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_proof_certificate_family_transport_contract_selftest_progress(
        age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks=completed_checks,
        age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks=total_checks,
        age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text=checks_text,
        age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe=current_probe,
        age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe=(
            last_completed_probe
        ),
        age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_proof_family_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bproof_family_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bproof_family_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bproof_family_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bproof_family_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bproof_family_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_proof_family_contract_selftest_progress(
        age5_full_real_proof_family_contract_selftest_completed_checks=completed_checks,
        age5_full_real_proof_family_contract_selftest_total_checks=total_checks,
        age5_full_real_proof_family_contract_selftest_checks_text=checks_text,
        age5_full_real_proof_family_contract_selftest_current_probe=current_probe,
        age5_full_real_proof_family_contract_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_proof_family_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_proof_family_transport_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bproof_family_transport_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bproof_family_transport_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bproof_family_transport_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bproof_family_transport_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bproof_family_transport_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_proof_family_transport_contract_selftest_progress(
        age5_full_real_proof_family_transport_contract_selftest_completed_checks=completed_checks,
        age5_full_real_proof_family_transport_contract_selftest_total_checks=total_checks,
        age5_full_real_proof_family_transport_contract_selftest_checks_text=checks_text,
        age5_full_real_proof_family_transport_contract_selftest_current_probe=current_probe,
        age5_full_real_proof_family_transport_contract_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_proof_family_transport_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_bogae_alias_family_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bbogae_alias_family_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bbogae_alias_family_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bbogae_alias_family_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bbogae_alias_family_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bbogae_alias_family_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_bogae_alias_family_contract_selftest_progress(
        age5_full_real_bogae_alias_family_contract_selftest_completed_checks=completed_checks,
        age5_full_real_bogae_alias_family_contract_selftest_total_checks=total_checks,
        age5_full_real_bogae_alias_family_contract_selftest_checks_text=checks_text,
        age5_full_real_bogae_alias_family_contract_selftest_current_probe=current_probe,
        age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_bogae_alias_family_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_lang_surface_family_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\blang_surface_family_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\blang_surface_family_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\blang_surface_family_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\blang_surface_family_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\blang_surface_family_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_lang_surface_family_contract_selftest_progress(
        age5_full_real_lang_surface_family_contract_selftest_completed_checks=completed_checks,
        age5_full_real_lang_surface_family_contract_selftest_total_checks=total_checks,
        age5_full_real_lang_surface_family_contract_selftest_checks_text=checks_text,
        age5_full_real_lang_surface_family_contract_selftest_current_probe=current_probe,
        age5_full_real_lang_surface_family_contract_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_lang_surface_family_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_lang_surface_family_transport_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\blang_surface_family_transport_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\blang_surface_family_transport_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\blang_surface_family_transport_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\blang_surface_family_transport_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\blang_surface_family_transport_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_lang_surface_family_transport_contract_selftest_progress(
        age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks=completed_checks,
        age5_full_real_lang_surface_family_transport_contract_selftest_total_checks=total_checks,
        age5_full_real_lang_surface_family_transport_contract_selftest_checks_text=checks_text,
        age5_full_real_lang_surface_family_transport_contract_selftest_current_probe=current_probe,
        age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe=(
            last_completed_probe
        ),
        age5_full_real_lang_surface_family_transport_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_lang_runtime_family_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\blang_runtime_family_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\blang_runtime_family_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\blang_runtime_family_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\blang_runtime_family_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\blang_runtime_family_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_lang_runtime_family_contract_selftest_progress(
        age5_full_real_lang_runtime_family_contract_selftest_completed_checks=completed_checks,
        age5_full_real_lang_runtime_family_contract_selftest_total_checks=total_checks,
        age5_full_real_lang_runtime_family_contract_selftest_checks_text=checks_text,
        age5_full_real_lang_runtime_family_contract_selftest_current_probe=current_probe,
        age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_lang_runtime_family_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_lang_runtime_family_transport_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\blang_runtime_family_transport_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\blang_runtime_family_transport_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\blang_runtime_family_transport_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\blang_runtime_family_transport_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\blang_runtime_family_transport_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_lang_runtime_family_transport_contract_selftest_progress(
        age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks=completed_checks,
        age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks=total_checks,
        age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text=checks_text,
        age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe=current_probe,
        age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe=(
            last_completed_probe
        ),
        age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_gate0_runtime_family_transport_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bgate0_runtime_family_transport_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bgate0_runtime_family_transport_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bgate0_runtime_family_transport_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bgate0_runtime_family_transport_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bgate0_runtime_family_transport_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_gate0_runtime_family_transport_contract_selftest_progress(
        age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks=completed_checks,
        age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks=total_checks,
        age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text=checks_text,
        age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe=current_probe,
        age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe=(
            last_completed_probe
        ),
        age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_gate0_family_transport_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bgate0_family_transport_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bgate0_family_transport_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bgate0_family_transport_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bgate0_family_transport_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bgate0_family_transport_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_gate0_family_transport_contract_selftest_progress(
        age5_full_real_gate0_family_transport_contract_selftest_completed_checks=completed_checks,
        age5_full_real_gate0_family_transport_contract_selftest_total_checks=total_checks,
        age5_full_real_gate0_family_transport_contract_selftest_checks_text=checks_text,
        age5_full_real_gate0_family_transport_contract_selftest_current_probe=current_probe,
        age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe=(
            last_completed_probe
        ),
        age5_full_real_gate0_family_transport_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_gate0_family_contract_selftest_progress(*texts: str) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bgate0_family_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bgate0_family_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bgate0_family_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bgate0_family_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bgate0_family_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_gate0_family_contract_selftest_progress(
        age5_full_real_gate0_family_contract_selftest_completed_checks=completed_checks,
        age5_full_real_gate0_family_contract_selftest_total_checks=total_checks,
        age5_full_real_gate0_family_contract_selftest_checks_text=checks_text,
        age5_full_real_gate0_family_contract_selftest_current_probe=current_probe,
        age5_full_real_gate0_family_contract_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_gate0_family_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_gate0_surface_family_contract_selftest_progress(*texts: str) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bgate0_surface_family_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bgate0_surface_family_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bgate0_surface_family_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bgate0_surface_family_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bgate0_surface_family_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_gate0_surface_family_contract_selftest_progress(
        age5_full_real_gate0_surface_family_contract_selftest_completed_checks=completed_checks,
        age5_full_real_gate0_surface_family_contract_selftest_total_checks=total_checks,
        age5_full_real_gate0_surface_family_contract_selftest_checks_text=checks_text,
        age5_full_real_gate0_surface_family_contract_selftest_current_probe=current_probe,
        age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe=last_completed_probe,
        age5_full_real_gate0_surface_family_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_gate0_surface_family_transport_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bgate0_surface_family_transport_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bgate0_surface_family_transport_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bgate0_surface_family_transport_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bgate0_surface_family_transport_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bgate0_surface_family_transport_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_gate0_surface_family_transport_contract_selftest_progress(
        age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks=(
            completed_checks
        ),
        age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks=total_checks,
        age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text=checks_text,
        age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe=current_probe,
        age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe=(
            last_completed_probe
        ),
        age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_gate0_transport_family_contract_selftest_progress(*texts: str) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bgate0_transport_family_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bgate0_transport_family_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bgate0_transport_family_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bgate0_transport_family_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bgate0_transport_family_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_gate0_transport_family_contract_selftest_progress(
        age5_full_real_gate0_transport_family_contract_selftest_completed_checks=completed_checks,
        age5_full_real_gate0_transport_family_contract_selftest_total_checks=total_checks,
        age5_full_real_gate0_transport_family_contract_selftest_checks_text=checks_text,
        age5_full_real_gate0_transport_family_contract_selftest_current_probe=current_probe,
        age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe=(
            last_completed_probe
        ),
        age5_full_real_gate0_transport_family_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_gate0_transport_family_transport_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bgate0_transport_family_transport_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bgate0_transport_family_transport_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bgate0_transport_family_transport_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bgate0_transport_family_transport_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bgate0_transport_family_transport_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_gate0_transport_family_transport_contract_selftest_progress(
        age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks=completed_checks,
        age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks=total_checks,
        age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text=checks_text,
        age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe=current_probe,
        age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe=(
            last_completed_probe
        ),
        age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_bogae_alias_family_transport_contract_selftest_progress(
    *texts: str,
) -> dict[str, str]:
    completed_checks = "-"
    total_checks = "-"
    checks_text = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        completed_matches = re.findall(
            r"\bbogae_alias_family_transport_contract_selftest_completed_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        total_matches = re.findall(
            r"\bbogae_alias_family_transport_contract_selftest_total_checks=([A-Za-z0-9._,-]+)",
            normalized,
        )
        checks_text_matches = re.findall(
            r"\bbogae_alias_family_transport_contract_selftest_checks_text=([A-Za-z0-9._,-]+)",
            normalized,
        )
        current_probe_matches = re.findall(
            r"\bbogae_alias_family_transport_contract_selftest_current_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        last_completed_probe_matches = re.findall(
            r"\bbogae_alias_family_transport_contract_selftest_last_completed_probe=([A-Za-z0-9._,-]+)",
            normalized,
        )
        if completed_matches:
            completed_checks = completed_matches[-1]
        if total_matches:
            total_checks = total_matches[-1]
        if checks_text_matches:
            checks_text = checks_text_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_bogae_alias_family_transport_contract_selftest_progress(
        age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks=completed_checks,
        age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks=total_checks,
        age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text=checks_text,
        age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe=current_probe,
        age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe=(
            last_completed_probe
        ),
        age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present=(
            completed_checks != "-"
            or total_checks != "-"
            or checks_text != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_w94_social_pack_check_progress(*texts: str) -> dict[str, str]:
    current_case = "-"
    last_completed_case = "-"
    total_elapsed_ms = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_case_matches = re.findall(r"\bw94_social_pack_check_current_case=([A-Za-z0-9._,-]+)", normalized)
        last_completed_case_matches = re.findall(
            r"\bw94_social_pack_check_last_completed_case=([A-Za-z0-9._,-]+)", normalized
        )
        elapsed_matches = re.findall(r"\bw94_social_pack_check_total_elapsed_ms=([A-Za-z0-9._-]+)", normalized)
        current_probe_matches = re.findall(r"\bw94_social_pack_check_current_probe=([A-Za-z0-9._,-]+)", normalized)
        last_completed_probe_matches = re.findall(
            r"\bw94_social_pack_check_last_completed_probe=([A-Za-z0-9._,-]+)", normalized
        )
        if current_case_matches:
            current_case = current_case_matches[-1]
        if last_completed_case_matches:
            last_completed_case = last_completed_case_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_w94_social_pack_check_progress(
        age5_full_real_w94_social_pack_check_current_case=current_case,
        age5_full_real_w94_social_pack_check_last_completed_case=last_completed_case,
        age5_full_real_w94_social_pack_check_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_w94_social_pack_check_current_probe=current_probe,
        age5_full_real_w94_social_pack_check_last_completed_probe=last_completed_probe,
        age5_full_real_w94_social_pack_check_progress_present=(
            current_case != "-"
            or last_completed_case != "-"
            or total_elapsed_ms != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_w95_cert_pack_check_progress(*texts: str) -> dict[str, str]:
    current_case = "-"
    last_completed_case = "-"
    total_elapsed_ms = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_case_matches = re.findall(r"\bw95_cert_pack_check_current_case=([A-Za-z0-9._,-]+)", normalized)
        last_completed_case_matches = re.findall(
            r"\bw95_cert_pack_check_last_completed_case=([A-Za-z0-9._,-]+)", normalized
        )
        elapsed_matches = re.findall(r"\bw95_cert_pack_check_total_elapsed_ms=([A-Za-z0-9._-]+)", normalized)
        current_probe_matches = re.findall(r"\bw95_cert_pack_check_current_probe=([A-Za-z0-9._,-]+)", normalized)
        last_completed_probe_matches = re.findall(
            r"\bw95_cert_pack_check_last_completed_probe=([A-Za-z0-9._,-]+)", normalized
        )
        if current_case_matches:
            current_case = current_case_matches[-1]
        if last_completed_case_matches:
            last_completed_case = last_completed_case_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_w95_cert_pack_check_progress(
        age5_full_real_w95_cert_pack_check_current_case=current_case,
        age5_full_real_w95_cert_pack_check_last_completed_case=last_completed_case,
        age5_full_real_w95_cert_pack_check_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_w95_cert_pack_check_current_probe=current_probe,
        age5_full_real_w95_cert_pack_check_last_completed_probe=last_completed_probe,
        age5_full_real_w95_cert_pack_check_progress_present=(
            current_case != "-"
            or last_completed_case != "-"
            or total_elapsed_ms != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_w96_somssi_pack_check_progress(*texts: str) -> dict[str, str]:
    current_case = "-"
    last_completed_case = "-"
    total_elapsed_ms = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_case_matches = re.findall(r"\bw96_somssi_pack_check_current_case=([A-Za-z0-9._,-]+)", normalized)
        last_completed_case_matches = re.findall(
            r"\bw96_somssi_pack_check_last_completed_case=([A-Za-z0-9._,-]+)", normalized
        )
        elapsed_matches = re.findall(r"\bw96_somssi_pack_check_total_elapsed_ms=([A-Za-z0-9._-]+)", normalized)
        current_probe_matches = re.findall(r"\bw96_somssi_pack_check_current_probe=([A-Za-z0-9._,-]+)", normalized)
        last_completed_probe_matches = re.findall(
            r"\bw96_somssi_pack_check_last_completed_probe=([A-Za-z0-9._,-]+)", normalized
        )
        if current_case_matches:
            current_case = current_case_matches[-1]
        if last_completed_case_matches:
            last_completed_case = last_completed_case_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_w96_somssi_pack_check_progress(
        age5_full_real_w96_somssi_pack_check_current_case=current_case,
        age5_full_real_w96_somssi_pack_check_last_completed_case=last_completed_case,
        age5_full_real_w96_somssi_pack_check_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_w96_somssi_pack_check_current_probe=current_probe,
        age5_full_real_w96_somssi_pack_check_last_completed_probe=last_completed_probe,
        age5_full_real_w96_somssi_pack_check_progress_present=(
            current_case != "-"
            or last_completed_case != "-"
            or total_elapsed_ms != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def parse_full_real_w97_self_heal_pack_check_progress(*texts: str) -> dict[str, str]:
    current_case = "-"
    last_completed_case = "-"
    total_elapsed_ms = "-"
    current_probe = "-"
    last_completed_probe = "-"
    for text in texts:
        normalized = str(text or "").strip()
        if not normalized:
            continue
        current_case_matches = re.findall(
            r"\bw97_self_heal_pack_check_current_case=([A-Za-z0-9._,-]+)", normalized
        )
        last_completed_case_matches = re.findall(
            r"\bw97_self_heal_pack_check_last_completed_case=([A-Za-z0-9._,-]+)", normalized
        )
        elapsed_matches = re.findall(r"\bw97_self_heal_pack_check_total_elapsed_ms=([A-Za-z0-9._-]+)", normalized)
        current_probe_matches = re.findall(
            r"\bw97_self_heal_pack_check_current_probe=([A-Za-z0-9._,-]+)", normalized
        )
        last_completed_probe_matches = re.findall(
            r"\bw97_self_heal_pack_check_last_completed_probe=([A-Za-z0-9._,-]+)", normalized
        )
        if current_case_matches:
            current_case = current_case_matches[-1]
        if last_completed_case_matches:
            last_completed_case = last_completed_case_matches[-1]
        if elapsed_matches:
            total_elapsed_ms = elapsed_matches[-1]
        if current_probe_matches:
            current_probe = current_probe_matches[-1]
        if last_completed_probe_matches:
            last_completed_probe = last_completed_probe_matches[-1]
    return build_age5_full_real_w97_self_heal_pack_check_progress(
        age5_full_real_w97_self_heal_pack_check_current_case=current_case,
        age5_full_real_w97_self_heal_pack_check_last_completed_case=last_completed_case,
        age5_full_real_w97_self_heal_pack_check_total_elapsed_ms=total_elapsed_ms,
        age5_full_real_w97_self_heal_pack_check_current_probe=current_probe,
        age5_full_real_w97_self_heal_pack_check_last_completed_probe=last_completed_probe,
        age5_full_real_w97_self_heal_pack_check_progress_present=(
            current_case != "-"
            or last_completed_case != "-"
            or total_elapsed_ms != "-"
            or current_probe != "-"
            or last_completed_probe != "-"
        ),
    )


def extract_full_real_profile_elapsed_map_from_child_report(path: Path, criterion_name: str) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_profile_elapsed_map(detail, failure_digest_joined)


def extract_full_real_core_lang_sanity_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_core_lang_sanity_progress(detail, failure_digest_joined)


def extract_full_real_pipeline_emit_flags_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_pipeline_emit_flags_progress(detail, failure_digest_joined)


def extract_full_real_pipeline_emit_flags_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_pipeline_emit_flags_selftest_progress(detail, failure_digest_joined)


def extract_full_real_pipeline_emit_flags_selftest_probe_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_pipeline_emit_flags_selftest_probe(detail, failure_digest_joined)


def extract_full_real_age5_combined_policy_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_age5_combined_policy_selftest_progress(detail, failure_digest_joined)


def extract_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_profile_matrix_full_real_smoke_policy_selftest_progress(detail, failure_digest_joined)


def extract_full_real_profile_matrix_full_real_smoke_check_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_profile_matrix_full_real_smoke_check_selftest_progress(detail, failure_digest_joined)


def extract_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_map_access_contract_check_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_map_access_contract_check_progress(detail, failure_digest_joined)


def extract_full_real_tensor_v0_cli_check_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_tensor_v0_cli_check_progress(detail, failure_digest_joined)


def extract_full_real_ci_pack_golden_jjaim_flatten_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_ci_pack_golden_jjaim_flatten_selftest_progress(detail, failure_digest_joined)


def extract_full_real_ci_pack_golden_event_model_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_ci_pack_golden_event_model_selftest_progress(detail, failure_digest_joined)


def extract_full_real_ci_pack_golden_lang_consistency_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_ci_pack_golden_lang_consistency_selftest_progress(detail, failure_digest_joined)


def extract_full_real_w107_golden_index_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_w107_golden_index_selftest_progress(detail, failure_digest_joined)


def extract_full_real_w107_progress_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_w107_progress_contract_selftest_progress(detail, failure_digest_joined)


def extract_full_real_age1_immediate_proof_operation_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_age1_immediate_proof_operation_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_proof_certificate_v1_family_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_proof_certificate_v1_family_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_proof_certificate_family_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_proof_certificate_family_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_proof_certificate_family_transport_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_proof_certificate_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_proof_family_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_proof_family_contract_selftest_progress(detail, failure_digest_joined)


def extract_full_real_proof_family_transport_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_proof_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_lang_surface_family_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_lang_surface_family_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_lang_surface_family_transport_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_lang_surface_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_lang_runtime_family_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_lang_runtime_family_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_lang_runtime_family_transport_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_lang_runtime_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_gate0_runtime_family_transport_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_gate0_runtime_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_gate0_family_transport_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_gate0_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_gate0_family_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_gate0_family_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_gate0_surface_family_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_gate0_surface_family_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_gate0_surface_family_transport_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_gate0_surface_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_gate0_transport_family_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_gate0_transport_family_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_gate0_transport_family_transport_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_gate0_transport_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_bogae_alias_family_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_bogae_alias_family_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_bogae_alias_family_transport_contract_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_bogae_alias_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_w94_social_pack_check_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_w94_social_pack_check_progress(detail, failure_digest_joined)


def extract_full_real_w95_cert_pack_check_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_w95_cert_pack_check_progress(detail, failure_digest_joined)


def extract_full_real_w96_somssi_pack_check_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_w96_somssi_pack_check_progress(detail, failure_digest_joined)


def extract_full_real_w97_self_heal_pack_check_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_w97_self_heal_pack_check_progress(detail, failure_digest_joined)


def extract_full_real_ci_pack_golden_age5_surface_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_ci_pack_golden_age5_surface_selftest_progress(detail, failure_digest_joined)


def extract_full_real_ci_pack_golden_guideblock_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_ci_pack_golden_guideblock_selftest_progress(detail, failure_digest_joined)


def extract_full_real_ci_pack_golden_exec_policy_selftest_progress_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_ci_pack_golden_exec_policy_selftest_progress(detail, failure_digest_joined)


def extract_full_real_profile_elapsed_map_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_profile_elapsed_map(detail, failure_digest_joined)


def extract_full_real_core_lang_sanity_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_core_lang_sanity_progress(detail, failure_digest_joined)


def extract_full_real_pipeline_emit_flags_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_pipeline_emit_flags_progress(detail, failure_digest_joined)


def extract_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_profile_matrix_full_real_smoke_policy_selftest_progress(detail, failure_digest_joined)


def extract_full_real_profile_matrix_full_real_smoke_check_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_profile_matrix_full_real_smoke_check_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_map_access_contract_check_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_map_access_contract_check_progress(detail, failure_digest_joined)


def extract_full_real_tensor_v0_cli_check_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_tensor_v0_cli_check_progress(detail, failure_digest_joined)


def extract_full_real_ci_pack_golden_age5_surface_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_ci_pack_golden_age5_surface_selftest_progress(detail, failure_digest_joined)


def extract_full_real_ci_pack_golden_guideblock_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_ci_pack_golden_guideblock_selftest_progress(detail, failure_digest_joined)


def extract_full_real_ci_pack_golden_exec_policy_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_ci_pack_golden_exec_policy_selftest_progress(detail, failure_digest_joined)


def extract_full_real_ci_pack_golden_jjaim_flatten_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_ci_pack_golden_jjaim_flatten_selftest_progress(detail, failure_digest_joined)


def extract_full_real_ci_pack_golden_event_model_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_ci_pack_golden_event_model_selftest_progress(detail, failure_digest_joined)


def extract_full_real_ci_pack_golden_lang_consistency_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_ci_pack_golden_lang_consistency_selftest_progress(detail, failure_digest_joined)


def extract_full_real_w107_golden_index_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_w107_golden_index_selftest_progress(detail, failure_digest_joined)


def extract_full_real_w107_progress_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_w107_progress_contract_selftest_progress(detail, failure_digest_joined)


def extract_full_real_age1_immediate_proof_operation_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_age1_immediate_proof_operation_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_proof_certificate_v1_family_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_proof_certificate_v1_family_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_proof_certificate_family_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_proof_certificate_family_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_proof_certificate_family_transport_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        if detail:
            break
    failure_digest_joined = " ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_proof_certificate_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_proof_family_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        if detail:
            break
    failure_digest_joined = " ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_proof_family_contract_selftest_progress(detail, failure_digest_joined)


def extract_full_real_proof_family_transport_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        if detail:
            break
    failure_digest_joined = " ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_proof_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_lang_surface_family_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        if detail:
            break
    failure_digest_joined = " ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_lang_surface_family_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_lang_surface_family_transport_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        if detail:
            break
    failure_digest_joined = " ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_lang_surface_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_lang_runtime_family_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        if detail:
            break
    failure_digest_joined = " ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_lang_runtime_family_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_lang_runtime_family_transport_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        if detail:
            break
    failure_digest_joined = " ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_lang_runtime_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_gate0_runtime_family_transport_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        if detail:
            break
    failure_digest_joined = " ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_gate0_runtime_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_gate0_family_transport_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        if detail:
            break
    failure_digest_joined = " ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_gate0_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_gate0_family_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        if detail:
            break
    failure_digest_joined = " ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_gate0_family_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_gate0_surface_family_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_gate0_surface_family_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_gate0_surface_family_transport_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_gate0_surface_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_gate0_transport_family_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_gate0_transport_family_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_gate0_transport_family_transport_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_gate0_transport_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_bogae_alias_family_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        if detail:
            break
    failure_digest_joined = " ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_bogae_alias_family_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_bogae_alias_family_transport_contract_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        if detail:
            break
    failure_digest_joined = " ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_bogae_alias_family_transport_contract_selftest_progress(
        detail, failure_digest_joined
    )


def extract_full_real_w94_social_pack_check_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_w94_social_pack_check_progress(detail, failure_digest_joined)


def extract_full_real_w95_cert_pack_check_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_w95_cert_pack_check_progress(detail, failure_digest_joined)


def extract_full_real_w96_somssi_pack_check_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_w96_somssi_pack_check_progress(detail, failure_digest_joined)


def extract_full_real_w97_self_heal_pack_check_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_w97_self_heal_pack_check_progress(detail, failure_digest_joined)


def extract_full_real_pipeline_emit_flags_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_pipeline_emit_flags_selftest_progress(detail, failure_digest_joined)


def extract_full_real_pipeline_emit_flags_selftest_probe_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_pipeline_emit_flags_selftest_probe(detail, failure_digest_joined)


def extract_full_real_age5_combined_policy_selftest_progress_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_age5_combined_policy_selftest_progress(detail, failure_digest_joined)


def extract_full_real_profile_status_map_from_child_report(path: Path, criterion_name: str) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_profile_status_map(detail, failure_digest_joined)


def extract_full_real_profile_status_map_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_profile_status_map(detail, failure_digest_joined)


def extract_full_real_core_lang_sanity_elapsed_summary_from_child_report(
    path: Path, criterion_name: str
) -> dict[str, str]:
    detail, failure_digest_joined = load_report_criterion_detail(path, criterion_name)
    return parse_full_real_core_lang_sanity_elapsed_summary(detail, failure_digest_joined)


def extract_full_real_core_lang_sanity_elapsed_summary_from_criteria(
    criteria: list[dict[str, object]],
    failure_digest: list[str] | None = None,
    criterion_name: str = "age5_ci_profile_matrix_full_real_smoke_optin_pass",
) -> dict[str, str]:
    detail = ""
    for row in criteria:
        if str(row.get("name", "")).strip() != criterion_name:
            continue
        detail = str(row.get("detail", "")).strip()
        break
    failure_digest_joined = " | ".join(
        str(item).strip() for item in (failure_digest or []) if str(item).strip()
    )
    return parse_full_real_core_lang_sanity_elapsed_summary(detail, failure_digest_joined)


def run_or_reuse_age5_close_child_report(
    *,
    cmd: list[str],
    cwd: Path,
    env: dict[str, str] | None,
    report_path: Path,
    required_criterion: str,
    timeout_sec: int | None = None,
) -> tuple[subprocess.CompletedProcess[str], bool]:
    if cached_age5_close_child_report_ok(report_path, required_criterion):
        proc = subprocess.CompletedProcess(
            cmd,
            0,
            stdout=(
                "[age5-close-combined-heavy] reused child report "
                f"report={report_path} criterion={required_criterion}"
            ),
            stderr="",
        )
        return proc, True
    return run_text(cmd, cwd, env=env, timeout_sec=timeout_sec), False


def load_age4_proof_snapshot_sources(report_dir: Path) -> tuple[dict[str, str], dict[str, str]]:
    gate_result_snapshot = build_age4_proof_snapshot()
    final_status_parse_snapshot = build_age4_proof_snapshot()
    gate_result_present = False
    final_status_parse_present = False
    snapshot = build_age4_proof_snapshot()
    result_doc = load_json(report_dir / "ci_gate_result.detjson")
    if isinstance(result_doc, dict):
        gate_result_present = True
        gate_result_snapshot[AGE4_PROOF_OK_KEY] = "1" if bool(result_doc.get(AGE4_PROOF_OK_KEY, False)) else "0"
        try:
            gate_result_snapshot[AGE4_PROOF_FAILED_CRITERIA_KEY] = str(
                int(result_doc.get(AGE4_PROOF_FAILED_CRITERIA_KEY, -1))
            )
        except Exception:
            pass
        preview = str(result_doc.get(AGE4_PROOF_FAILED_PREVIEW_KEY, "")).strip()
        if preview:
            gate_result_snapshot[AGE4_PROOF_FAILED_PREVIEW_KEY] = preview
        snapshot.update(gate_result_snapshot)
    final_parse_doc = load_json(report_dir / "ci_gate_final_status_line_parse.detjson")
    parsed = final_parse_doc.get("parsed") if isinstance(final_parse_doc, dict) else None
    if isinstance(parsed, dict):
        final_status_parse_present = True
        final_status_parse_snapshot[AGE4_PROOF_OK_KEY] = (
            "1" if str(parsed.get(AGE4_PROOF_OK_KEY, "0")).strip() == "1" else "0"
        )
        try:
            final_status_parse_snapshot[AGE4_PROOF_FAILED_CRITERIA_KEY] = str(
                int(str(parsed.get(AGE4_PROOF_FAILED_CRITERIA_KEY, "-1")).strip())
            )
        except Exception:
            pass
        preview = str(parsed.get(AGE4_PROOF_FAILED_PREVIEW_KEY, "")).strip()
        if preview:
            final_status_parse_snapshot[AGE4_PROOF_FAILED_PREVIEW_KEY] = preview
        snapshot.update(final_status_parse_snapshot)
    source_fields = build_age4_proof_source_snapshot_fields(
        top_snapshot=snapshot,
        gate_result_snapshot=gate_result_snapshot,
        gate_result_present=gate_result_present,
        final_status_parse_snapshot=final_status_parse_snapshot,
        final_status_parse_present=final_status_parse_present,
    )
    return snapshot, source_fields


def build_age5_combined_heavy_optin_report(
    *,
    root: Path,
    strict: bool,
    combined_heavy_env_enabled: bool,
    full_real_cmd: list[str],
    full_real_proc: subprocess.CompletedProcess[str],
    full_real_report: Path,
    runtime_helper_negative_cmd: list[str],
    runtime_helper_negative_proc: subprocess.CompletedProcess[str],
    runtime_helper_negative_report: Path,
    group_id_summary_negative_cmd: list[str],
    group_id_summary_negative_proc: subprocess.CompletedProcess[str],
    group_id_summary_negative_report: Path,
    combined_heavy_child_timeout_sec: int = 0,
    age4_proof_snapshot: dict[str, str] | None = None,
    age4_proof_source_fields: dict[str, str] | None = None,
) -> dict[str, object]:
    smoke_check_script = root / CI_PROFILE_MATRIX_FULL_REAL_SMOKE_SCRIPT
    smoke_check_selftest_script = root / "tests" / "run_ci_profile_matrix_full_real_smoke_check_selftest.py"
    smoke_check_script_text = str(CI_PROFILE_MATRIX_FULL_REAL_SMOKE_SCRIPT).replace("\\", "/")
    smoke_check_selftest_script_text = "tests/run_ci_profile_matrix_full_real_smoke_check_selftest.py"
    full_real_source_trace = build_age5_combined_heavy_full_real_source_trace(
        smoke_check_script=smoke_check_script_text,
        smoke_check_script_exists=smoke_check_script.exists(),
        smoke_check_selftest_script=smoke_check_selftest_script_text,
        smoke_check_selftest_script_exists=smoke_check_selftest_script.exists(),
    )
    full_real_ok = int(full_real_proc.returncode == 0)
    runtime_helper_negative_ok = int(runtime_helper_negative_proc.returncode == 0)
    group_id_summary_negative_ok = int(group_id_summary_negative_proc.returncode == 0)
    overall_ok = bool(full_real_ok and runtime_helper_negative_ok and group_id_summary_negative_ok)
    timeout_targets: list[str] = []
    if int(full_real_proc.returncode) == 124 or child_report_indicates_timeout(
        full_real_report, "age5_ci_profile_matrix_full_real_smoke_optin_pass"
    ):
        timeout_targets.append("full_real")
    if int(runtime_helper_negative_proc.returncode) == 124 or child_report_indicates_timeout(
        runtime_helper_negative_report, "age5_ci_profile_core_lang_runtime_helper_negative_optin_pass"
    ):
        timeout_targets.append("runtime_helper_negative")
    if int(group_id_summary_negative_proc.returncode) == 124 or child_report_indicates_timeout(
        group_id_summary_negative_report, "age5_ci_profile_core_lang_group_id_summary_negative_optin_pass"
    ):
        timeout_targets.append("group_id_summary_negative")
    timeout_mode = resolve_age5_combined_heavy_timeout_mode(combined_heavy_child_timeout_sec)
    full_real_timeout_breakdown = build_age5_full_real_timeout_breakdown()
    full_real_elapsed_summary = build_age5_full_real_elapsed_summary()
    full_real_core_lang_sanity_elapsed_summary = build_age5_full_real_core_lang_sanity_elapsed_summary()
    full_real_core_lang_sanity_progress = parse_full_real_core_lang_sanity_progress()
    full_real_pipeline_emit_flags_progress = build_age5_full_real_pipeline_emit_flags_progress()
    full_real_pipeline_emit_flags_selftest_progress = build_age5_full_real_pipeline_emit_flags_selftest_progress()
    full_real_pipeline_emit_flags_selftest_probe = build_age5_full_real_pipeline_emit_flags_selftest_probe()
    full_real_age5_combined_policy_selftest_progress = parse_full_real_age5_combined_policy_selftest_progress()
    full_real_profile_matrix_full_real_smoke_policy_selftest_progress = (
        build_age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress()
    )
    full_real_profile_matrix_full_real_smoke_check_selftest_progress = (
        build_age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress()
    )
    full_real_fixed64_darwin_real_report_readiness_check_selftest_progress = (
        build_age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress()
    )
    full_real_map_access_contract_check_progress = (
        build_age5_full_real_map_access_contract_check_progress()
    )
    full_real_tensor_v0_cli_check_progress = build_age5_full_real_tensor_v0_cli_check_progress()
    full_real_ci_pack_golden_exec_policy_selftest_progress = (
        build_age5_full_real_ci_pack_golden_exec_policy_selftest_progress()
    )
    full_real_ci_pack_golden_age5_surface_selftest_progress = (
        build_age5_full_real_ci_pack_golden_age5_surface_selftest_progress()
    )
    full_real_ci_pack_golden_guideblock_selftest_progress = (
        build_age5_full_real_ci_pack_golden_guideblock_selftest_progress()
    )
    full_real_ci_pack_golden_jjaim_flatten_selftest_progress = (
        build_age5_full_real_ci_pack_golden_jjaim_flatten_selftest_progress()
    )
    full_real_ci_pack_golden_event_model_selftest_progress = (
        build_age5_full_real_ci_pack_golden_event_model_selftest_progress()
    )
    full_real_ci_pack_golden_lang_consistency_selftest_progress = (
        build_age5_full_real_ci_pack_golden_lang_consistency_selftest_progress()
    )
    full_real_w107_golden_index_selftest_progress = (
        build_age5_full_real_w107_golden_index_selftest_progress()
    )
    full_real_w107_progress_contract_selftest_progress = (
        build_age5_full_real_w107_progress_contract_selftest_progress()
    )
    full_real_age1_immediate_proof_operation_contract_selftest_progress = (
        build_age5_full_real_age1_immediate_proof_operation_contract_selftest_progress()
    )
    full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress = (
        build_age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress()
    )
    full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress = (
        build_age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress()
    )
    full_real_proof_certificate_v1_family_contract_selftest_progress = (
        build_age5_full_real_proof_certificate_v1_family_contract_selftest_progress()
    )
    full_real_proof_certificate_family_contract_selftest_progress = (
        build_age5_full_real_proof_certificate_family_contract_selftest_progress()
    )
    full_real_proof_certificate_family_transport_contract_selftest_progress = (
        build_age5_full_real_proof_certificate_family_transport_contract_selftest_progress()
    )
    full_real_proof_family_contract_selftest_progress = (
        build_age5_full_real_proof_family_contract_selftest_progress()
    )
    full_real_proof_family_transport_contract_selftest_progress = (
        build_age5_full_real_proof_family_transport_contract_selftest_progress()
    )
    full_real_lang_surface_family_contract_selftest_progress = (
        build_age5_full_real_lang_surface_family_contract_selftest_progress()
    )
    full_real_lang_surface_family_transport_contract_selftest_progress = (
        build_age5_full_real_lang_surface_family_transport_contract_selftest_progress()
    )
    full_real_lang_runtime_family_contract_selftest_progress = (
        build_age5_full_real_lang_runtime_family_contract_selftest_progress()
    )
    full_real_lang_runtime_family_transport_contract_selftest_progress = (
        build_age5_full_real_lang_runtime_family_transport_contract_selftest_progress()
    )
    full_real_gate0_runtime_family_transport_contract_selftest_progress = (
        build_age5_full_real_gate0_runtime_family_transport_contract_selftest_progress()
    )
    full_real_gate0_family_transport_contract_selftest_progress = (
        build_age5_full_real_gate0_family_transport_contract_selftest_progress()
    )
    full_real_gate0_transport_family_contract_selftest_progress = (
        build_age5_full_real_gate0_transport_family_contract_selftest_progress()
    )
    full_real_gate0_transport_family_transport_contract_selftest_progress = (
        build_age5_full_real_gate0_transport_family_transport_contract_selftest_progress()
    )
    full_real_gate0_family_contract_selftest_progress = (
        build_age5_full_real_gate0_family_contract_selftest_progress()
    )
    full_real_gate0_surface_family_contract_selftest_progress = (
        build_age5_full_real_gate0_surface_family_contract_selftest_progress()
    )
    full_real_gate0_surface_family_transport_contract_selftest_progress = (
        build_age5_full_real_gate0_surface_family_transport_contract_selftest_progress()
    )
    full_real_bogae_alias_family_contract_selftest_progress = (
        build_age5_full_real_bogae_alias_family_contract_selftest_progress()
    )
    full_real_bogae_alias_family_transport_contract_selftest_progress = (
        build_age5_full_real_bogae_alias_family_transport_contract_selftest_progress()
    )
    full_real_w94_social_pack_check_progress = build_age5_full_real_w94_social_pack_check_progress()
    full_real_w95_cert_pack_check_progress = build_age5_full_real_w95_cert_pack_check_progress()
    full_real_w96_somssi_pack_check_progress = build_age5_full_real_w96_somssi_pack_check_progress()
    full_real_w97_self_heal_pack_check_progress = build_age5_full_real_w97_self_heal_pack_check_progress()
    full_real_profile_elapsed_map = build_age5_full_real_profile_elapsed_map()
    full_real_profile_status_map = build_age5_full_real_profile_status_map()
    if "full_real" in timeout_targets:
        full_real_timeout_breakdown = extract_full_real_timeout_breakdown_from_child_report(
            full_real_report,
            "age5_ci_profile_matrix_full_real_smoke_optin_pass",
        )
    if bool(full_real_ok) or "full_real" in timeout_targets:
        full_real_elapsed_summary = extract_full_real_elapsed_summary_from_child_report(
            full_real_report,
            "age5_ci_profile_matrix_full_real_smoke_optin_pass",
        )
        full_real_core_lang_sanity_elapsed_summary = (
            extract_full_real_core_lang_sanity_elapsed_summary_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_core_lang_sanity_progress = extract_full_real_core_lang_sanity_progress_from_child_report(
            full_real_report,
            "age5_ci_profile_matrix_full_real_smoke_optin_pass",
        )
        full_real_pipeline_emit_flags_progress = extract_full_real_pipeline_emit_flags_progress_from_child_report(
            full_real_report,
            "age5_ci_profile_matrix_full_real_smoke_optin_pass",
        )
        full_real_pipeline_emit_flags_selftest_progress = (
            extract_full_real_pipeline_emit_flags_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_pipeline_emit_flags_selftest_probe = (
            extract_full_real_pipeline_emit_flags_selftest_probe_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_age5_combined_policy_selftest_progress = (
            extract_full_real_age5_combined_policy_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_profile_matrix_full_real_smoke_policy_selftest_progress = (
            extract_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_profile_matrix_full_real_smoke_check_selftest_progress = (
            extract_full_real_profile_matrix_full_real_smoke_check_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_fixed64_darwin_real_report_readiness_check_selftest_progress = (
            extract_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_map_access_contract_check_progress = (
            extract_full_real_map_access_contract_check_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_tensor_v0_cli_check_progress = (
            extract_full_real_tensor_v0_cli_check_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_ci_pack_golden_exec_policy_selftest_progress = (
            extract_full_real_ci_pack_golden_exec_policy_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_ci_pack_golden_age5_surface_selftest_progress = (
            extract_full_real_ci_pack_golden_age5_surface_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_ci_pack_golden_guideblock_selftest_progress = (
            extract_full_real_ci_pack_golden_guideblock_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_ci_pack_golden_jjaim_flatten_selftest_progress = (
            extract_full_real_ci_pack_golden_jjaim_flatten_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_ci_pack_golden_event_model_selftest_progress = (
            extract_full_real_ci_pack_golden_event_model_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_ci_pack_golden_lang_consistency_selftest_progress = (
            extract_full_real_ci_pack_golden_lang_consistency_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_w107_golden_index_selftest_progress = (
            extract_full_real_w107_golden_index_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_w107_progress_contract_selftest_progress = (
            extract_full_real_w107_progress_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_age1_immediate_proof_operation_contract_selftest_progress = (
            extract_full_real_age1_immediate_proof_operation_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress = (
            extract_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress = (
            extract_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_proof_certificate_v1_family_contract_selftest_progress = (
            extract_full_real_proof_certificate_v1_family_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_proof_certificate_family_contract_selftest_progress = (
            extract_full_real_proof_certificate_family_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_proof_certificate_family_transport_contract_selftest_progress = (
            extract_full_real_proof_certificate_family_transport_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_proof_family_contract_selftest_progress = (
            extract_full_real_proof_family_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_proof_family_transport_contract_selftest_progress = (
            extract_full_real_proof_family_transport_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_lang_surface_family_contract_selftest_progress = (
            extract_full_real_lang_surface_family_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_lang_surface_family_transport_contract_selftest_progress = (
            extract_full_real_lang_surface_family_transport_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_lang_runtime_family_contract_selftest_progress = (
            extract_full_real_lang_runtime_family_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_lang_runtime_family_transport_contract_selftest_progress = (
            extract_full_real_lang_runtime_family_transport_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_gate0_runtime_family_transport_contract_selftest_progress = (
            extract_full_real_gate0_runtime_family_transport_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_gate0_family_transport_contract_selftest_progress = (
            extract_full_real_gate0_family_transport_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_gate0_family_contract_selftest_progress = (
            extract_full_real_gate0_family_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_gate0_surface_family_contract_selftest_progress = (
            extract_full_real_gate0_surface_family_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_gate0_surface_family_transport_contract_selftest_progress = (
            extract_full_real_gate0_surface_family_transport_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_gate0_transport_family_contract_selftest_progress = (
            extract_full_real_gate0_transport_family_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_gate0_transport_family_transport_contract_selftest_progress = (
            extract_full_real_gate0_transport_family_transport_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_bogae_alias_family_contract_selftest_progress = (
            extract_full_real_bogae_alias_family_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_bogae_alias_family_transport_contract_selftest_progress = (
            extract_full_real_bogae_alias_family_transport_contract_selftest_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_w94_social_pack_check_progress = extract_full_real_w94_social_pack_check_progress_from_child_report(
            full_real_report,
            "age5_ci_profile_matrix_full_real_smoke_optin_pass",
        )
        full_real_w95_cert_pack_check_progress = extract_full_real_w95_cert_pack_check_progress_from_child_report(
            full_real_report,
            "age5_ci_profile_matrix_full_real_smoke_optin_pass",
        )
        full_real_w96_somssi_pack_check_progress = extract_full_real_w96_somssi_pack_check_progress_from_child_report(
            full_real_report,
            "age5_ci_profile_matrix_full_real_smoke_optin_pass",
        )
        full_real_w97_self_heal_pack_check_progress = (
            extract_full_real_w97_self_heal_pack_check_progress_from_child_report(
                full_real_report,
                "age5_ci_profile_matrix_full_real_smoke_optin_pass",
            )
        )
        full_real_profile_elapsed_map = extract_full_real_profile_elapsed_map_from_child_report(
            full_real_report,
            "age5_ci_profile_matrix_full_real_smoke_optin_pass",
        )
        full_real_profile_status_map = extract_full_real_profile_status_map_from_child_report(
            full_real_report,
            "age5_ci_profile_matrix_full_real_smoke_optin_pass",
        )
    normalized_age4_proof_snapshot = (
        build_age4_proof_snapshot(**age4_proof_snapshot)
        if isinstance(age4_proof_snapshot, dict)
        else build_age4_proof_snapshot()
    )
    normalized_age4_proof_source_fields = (
        dict(age4_proof_source_fields)
        if isinstance(age4_proof_source_fields, dict)
        else build_age4_proof_source_snapshot_fields(top_snapshot=normalized_age4_proof_snapshot)
    )
    report: dict[str, object] = {
        "schema": AGE5_COMBINED_HEAVY_REPORT_SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "strict": bool(strict),
        "with_combined_heavy_runtime_helper_check": True,
        "combined_heavy_env_enabled": bool(combined_heavy_env_enabled),
        "overall_ok": overall_ok,
        AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY: 0,
        AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
        AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_KEY: build_age5_close_digest_selftest_default_field(),
        "age4_proof_snapshot_fields_text": AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
        "age4_proof_snapshot_text": build_age4_proof_snapshot_text(normalized_age4_proof_snapshot),
        "policy_contract": {
            "env_key": AGE5_COMBINED_HEAVY_ENV_KEY,
            "scope": AGE5_COMBINED_HEAVY_MODE,
            "combined_report_schema": AGE5_COMBINED_HEAVY_REPORT_SCHEMA,
            "full_real_source_trace_text": AGE5_COMBINED_HEAVY_FULL_REAL_SOURCE_TRACE_TEXT,
            "full_real_smoke_check_script": smoke_check_script_text,
            "full_real_smoke_check_selftest_script": smoke_check_selftest_script_text,
            "combined_required_reports": list(AGE5_COMBINED_HEAVY_REQUIRED_REPORTS),
            "combined_required_criteria": list(AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA),
            "combined_child_summary_keys": list(AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS),
            "combined_child_summary_default_fields": build_age5_combined_heavy_child_summary_default_fields(),
            "combined_child_summary_default_fields_text": AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT,
            "combined_timeout_policy_fields": build_age5_combined_heavy_timeout_policy_fields(),
            AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_KEY: AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_DEFAULT,
            AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_KEY: AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT,
            AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_DEFAULT,
            AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
            AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_KEY: build_age5_close_digest_selftest_default_field(),
            "combined_child_summary_default_text_transport_fields": (
                build_age5_combined_heavy_child_summary_default_text_transport_fields()
            ),
            "combined_child_summary_default_text_transport_fields_text": (
                AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS_TEXT
            ),
            **build_age4_proof_snapshot(),
            "age4_proof_snapshot_fields_text": AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
            "age4_proof_source_snapshot_fields_text": AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
            "age4_proof_snapshot_text": build_age4_proof_snapshot_text(build_age4_proof_snapshot()),
            **build_age4_proof_source_snapshot_fields(top_snapshot=build_age4_proof_snapshot()),
            "combined_contract_summary_fields": build_age5_combined_heavy_combined_report_contract_fields(),
            "combined_contract_summary_fields_text": AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT,
            "combined_full_summary_contract_fields": build_age5_combined_heavy_full_summary_contract_fields(),
            "combined_full_summary_contract_fields_text": AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT,
            "combined_full_summary_text_transport_fields": build_age5_combined_heavy_full_summary_text_transport_fields(),
            "combined_full_summary_text_transport_fields_text": AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FIELDS_TEXT,
        },
        "criteria": [
            {
                "name": "age5_ci_profile_matrix_full_real_smoke_optin_pass",
                "ok": bool(full_real_ok),
                "detail": "rc={} cmd={} stdout_tail={}".format(
                    full_real_proc.returncode,
                    " ".join(full_real_cmd),
                    clip(" | ".join(line.strip() for line in str(full_real_proc.stdout or "").strip().splitlines()[-6:]), 500),
                ),
            },
            {
                "name": "age5_ci_profile_core_lang_runtime_helper_negative_optin_pass",
                "ok": bool(runtime_helper_negative_ok),
                "detail": "rc={} cmd={} stdout_tail={}".format(
                    runtime_helper_negative_proc.returncode,
                    " ".join(runtime_helper_negative_cmd),
                    clip(
                        " | ".join(
                            line.strip() for line in str(runtime_helper_negative_proc.stdout or "").strip().splitlines()[-6:]
                        ),
                        500,
                    ),
                ),
            },
            {
                "name": "age5_ci_profile_core_lang_group_id_summary_negative_optin_pass",
                "ok": bool(group_id_summary_negative_ok),
                "detail": "rc={} cmd={} stdout_tail={}".format(
                    group_id_summary_negative_proc.returncode,
                    " ".join(group_id_summary_negative_cmd),
                    clip(
                        " | ".join(
                            line.strip() for line in str(group_id_summary_negative_proc.stdout or "").strip().splitlines()[-6:]
                        ),
                        500,
                    ),
                ),
            },
        ],
        "reports": {
            "full_real": str(full_real_report),
            "runtime_helper_negative": str(runtime_helper_negative_report),
            "group_id_summary_negative": str(group_id_summary_negative_report),
        },
        "full_real_source_trace": full_real_source_trace,
        "full_real_source_trace_text": build_age5_combined_heavy_full_real_source_trace_text(full_real_source_trace),
        "age5_combined_heavy_timeout_present": "1" if timeout_targets else "0",
        "age5_combined_heavy_timeout_targets": ",".join(timeout_targets) if timeout_targets else "-",
        AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY: timeout_mode,
        "age5_combined_heavy_timeout_policy_ok": "1",
        AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_KEY: AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT,
        "age5_full_real_elapsed_fields_text": AGE5_FULL_REAL_ELAPSED_FIELDS_TEXT,
        "age5_full_real_core_lang_sanity_elapsed_fields_text": AGE5_FULL_REAL_CORE_LANG_SANITY_ELAPSED_FIELDS_TEXT,
        "age5_full_real_core_lang_sanity_progress_fields_text": AGE5_FULL_REAL_CORE_LANG_SANITY_PROGRESS_FIELDS_TEXT,
        "age5_full_real_pipeline_emit_flags_progress_fields_text": AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_PROGRESS_FIELDS_TEXT,
        "age5_full_real_pipeline_emit_flags_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_pipeline_emit_flags_selftest_probe_fields_text": (
            AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROBE_FIELDS_TEXT
        ),
        "age5_full_real_age5_combined_policy_selftest_progress_fields_text": (
            "age5_full_real_age5_combined_policy_selftest_current_case=-|"
            "age5_full_real_age5_combined_policy_selftest_last_completed_case=-|"
            "age5_full_real_age5_combined_policy_selftest_current_format=-|"
            "age5_full_real_age5_combined_policy_selftest_last_completed_format=-|"
            "age5_full_real_age5_combined_policy_selftest_current_probe=-|"
            "age5_full_real_age5_combined_policy_selftest_last_completed_probe=-|"
            "age5_full_real_age5_combined_policy_selftest_total_elapsed_ms=-|"
            "age5_full_real_age5_combined_policy_selftest_progress_present=0"
        ),
        "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_fields_text": (
            AGE5_FULL_REAL_FIXED64_DARWIN_REAL_REPORT_READINESS_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_map_access_contract_check_progress_fields_text": (
            AGE5_FULL_REAL_MAP_ACCESS_CONTRACT_CHECK_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_tensor_v0_cli_check_progress_fields_text": (
            AGE5_FULL_REAL_TENSOR_V0_CLI_CHECK_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_ci_pack_golden_exec_policy_selftest_progress_fields_text": (
            AGE5_FULL_REAL_CI_PACK_GOLDEN_EXEC_POLICY_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_ci_pack_golden_age5_surface_selftest_progress_fields_text": (
            AGE5_FULL_REAL_CI_PACK_GOLDEN_AGE5_SURFACE_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_ci_pack_golden_guideblock_selftest_progress_fields_text": (
            AGE5_FULL_REAL_CI_PACK_GOLDEN_GUIDEBLOCK_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_ci_pack_golden_jjaim_flatten_selftest_progress_fields_text": (
            AGE5_FULL_REAL_CI_PACK_GOLDEN_JJAIM_FLATTEN_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_ci_pack_golden_event_model_selftest_progress_fields_text": (
            AGE5_FULL_REAL_CI_PACK_GOLDEN_EVENT_MODEL_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_ci_pack_golden_lang_consistency_selftest_progress_fields_text": (
            AGE5_FULL_REAL_CI_PACK_GOLDEN_LANG_CONSISTENCY_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_w107_golden_index_selftest_progress_fields_text": (
            AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_w107_progress_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_proof_certificate_v1_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_proof_certificate_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_proof_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROOF_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_proof_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_lang_surface_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_lang_surface_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_lang_runtime_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_gate0_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_GATE0_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_gate0_surface_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_gate0_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_gate0_transport_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_gate0_transport_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_bogae_alias_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_w94_social_pack_check_progress_fields_text": (
            AGE5_FULL_REAL_W94_SOCIAL_PACK_CHECK_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_w95_cert_pack_check_progress_fields_text": (
            AGE5_FULL_REAL_W95_CERT_PACK_CHECK_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_w96_somssi_pack_check_progress_fields_text": (
            AGE5_FULL_REAL_W96_SOMSSI_PACK_CHECK_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_w97_self_heal_pack_check_progress_fields_text": (
            AGE5_FULL_REAL_W97_SELF_HEAL_PACK_CHECK_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_profile_elapsed_map_fields_text": AGE5_FULL_REAL_PROFILE_ELAPSED_MAP_FIELDS_TEXT,
        "age5_full_real_profile_status_map_fields_text": AGE5_FULL_REAL_PROFILE_STATUS_MAP_FIELDS_TEXT,
        "age5_full_real_timeout_breakdown_fields_text": AGE5_FULL_REAL_TIMEOUT_BREAKDOWN_FIELDS_TEXT,
    }
    report.update(normalized_age4_proof_snapshot)
    report.update(normalized_age4_proof_source_fields)
    report.update(full_real_elapsed_summary)
    report.update(full_real_core_lang_sanity_elapsed_summary)
    report.update(full_real_core_lang_sanity_progress)
    report.update(full_real_pipeline_emit_flags_progress)
    report.update(full_real_pipeline_emit_flags_selftest_progress)
    report.update(full_real_pipeline_emit_flags_selftest_probe)
    report.update(full_real_age5_combined_policy_selftest_progress)
    report.update(full_real_profile_matrix_full_real_smoke_policy_selftest_progress)
    report.update(full_real_profile_matrix_full_real_smoke_check_selftest_progress)
    report.update(full_real_fixed64_darwin_real_report_readiness_check_selftest_progress)
    report.update(full_real_map_access_contract_check_progress)
    report.update(full_real_tensor_v0_cli_check_progress)
    report.update(full_real_ci_pack_golden_exec_policy_selftest_progress)
    report.update(full_real_ci_pack_golden_age5_surface_selftest_progress)
    report.update(full_real_ci_pack_golden_guideblock_selftest_progress)
    report.update(full_real_ci_pack_golden_jjaim_flatten_selftest_progress)
    report.update(full_real_ci_pack_golden_event_model_selftest_progress)
    report.update(full_real_ci_pack_golden_lang_consistency_selftest_progress)
    report.update(full_real_w107_golden_index_selftest_progress)
    report.update(full_real_w107_progress_contract_selftest_progress)
    report.update(full_real_age1_immediate_proof_operation_contract_selftest_progress)
    report.update(full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress)
    report.update(full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress)
    report.update(full_real_proof_certificate_v1_family_contract_selftest_progress)
    report.update(full_real_proof_certificate_family_contract_selftest_progress)
    report.update(full_real_proof_certificate_family_transport_contract_selftest_progress)
    report.update(full_real_proof_family_contract_selftest_progress)
    report.update(full_real_proof_family_transport_contract_selftest_progress)
    report.update(full_real_lang_surface_family_contract_selftest_progress)
    report.update(full_real_lang_surface_family_transport_contract_selftest_progress)
    report.update(full_real_lang_runtime_family_contract_selftest_progress)
    report.update(full_real_lang_runtime_family_transport_contract_selftest_progress)
    report.update(full_real_gate0_family_contract_selftest_progress)
    report.update(full_real_gate0_surface_family_contract_selftest_progress)
    report.update(full_real_gate0_surface_family_transport_contract_selftest_progress)
    report.update(full_real_gate0_family_transport_contract_selftest_progress)
    report.update(full_real_gate0_transport_family_contract_selftest_progress)
    report.update(full_real_gate0_transport_family_transport_contract_selftest_progress)
    report.update(full_real_gate0_runtime_family_transport_contract_selftest_progress)
    report.update(full_real_bogae_alias_family_contract_selftest_progress)
    report.update(full_real_bogae_alias_family_transport_contract_selftest_progress)
    report.update(full_real_w94_social_pack_check_progress)
    report.update(full_real_w95_cert_pack_check_progress)
    report.update(full_real_w96_somssi_pack_check_progress)
    report.update(full_real_w97_self_heal_pack_check_progress)
    report.update(full_real_profile_elapsed_map)
    report.update(full_real_profile_status_map)
    report.update(full_real_timeout_breakdown)
    report.update(
        build_age5_combined_heavy_child_summary_fields(
            full_real_ok=bool(full_real_ok),
            runtime_helper_negative_ok=bool(runtime_helper_negative_ok),
            group_id_summary_negative_ok=bool(group_id_summary_negative_ok),
        )
    )
    report.update(build_age5_combined_heavy_combined_report_contract_fields())
    report.update(build_age5_combined_heavy_full_summary_contract_fields())
    report.update(build_age5_combined_heavy_full_summary_text_transport_fields())
    report.update(build_age5_combined_heavy_child_summary_default_text_transport_fields())
    return report


def build_age5_close_report(
    *,
    strict: bool,
    with_profile_matrix_full_real_smoke_check: bool,
    with_runtime_helper_mismatch_negative_check: bool,
    with_group_id_summary_mismatch_negative_check: bool,
    with_combined_heavy_runtime_helper_check: bool,
    combined_heavy_env_enabled: bool,
    criteria: list[dict[str, object]],
    failure_digest: list[str],
    pending_items: list[str],
    repair: dict[str, object],
    age4_proof_snapshot: dict[str, str] | None = None,
    age4_proof_source_fields: dict[str, str] | None = None,
) -> dict[str, object]:
    overall_ok = all(bool(row.get("ok", False)) for row in criteria)
    normalized_age4_proof_snapshot = (
        build_age4_proof_snapshot(**age4_proof_snapshot)
        if isinstance(age4_proof_snapshot, dict)
        else build_age4_proof_snapshot()
    )
    normalized_age4_proof_source_fields = (
        dict(age4_proof_source_fields)
        if isinstance(age4_proof_source_fields, dict)
        else build_age4_proof_source_snapshot_fields(top_snapshot=normalized_age4_proof_snapshot)
    )
    full_real_elapsed_summary = (
        extract_full_real_elapsed_summary_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_elapsed_summary()
    )
    full_real_core_lang_sanity_elapsed_summary = (
        extract_full_real_core_lang_sanity_elapsed_summary_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_core_lang_sanity_elapsed_summary()
    )
    full_real_core_lang_sanity_progress = (
        extract_full_real_core_lang_sanity_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else parse_full_real_core_lang_sanity_progress()
    )
    full_real_pipeline_emit_flags_progress = (
        extract_full_real_pipeline_emit_flags_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_pipeline_emit_flags_progress()
    )
    full_real_pipeline_emit_flags_selftest_progress = (
        extract_full_real_pipeline_emit_flags_selftest_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_pipeline_emit_flags_selftest_progress()
    )
    full_real_pipeline_emit_flags_selftest_probe = (
        extract_full_real_pipeline_emit_flags_selftest_probe_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_pipeline_emit_flags_selftest_probe()
    )
    full_real_age5_combined_policy_selftest_progress = (
        extract_full_real_age5_combined_policy_selftest_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else parse_full_real_age5_combined_policy_selftest_progress()
    )
    full_real_profile_matrix_full_real_smoke_policy_selftest_progress = (
        extract_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress()
    )
    full_real_profile_matrix_full_real_smoke_check_selftest_progress = (
        extract_full_real_profile_matrix_full_real_smoke_check_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress()
    )
    full_real_fixed64_darwin_real_report_readiness_check_selftest_progress = (
        extract_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress()
    )
    full_real_map_access_contract_check_progress = (
        extract_full_real_map_access_contract_check_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_map_access_contract_check_progress()
    )
    full_real_tensor_v0_cli_check_progress = (
        extract_full_real_tensor_v0_cli_check_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_tensor_v0_cli_check_progress()
    )
    full_real_ci_pack_golden_exec_policy_selftest_progress = (
        extract_full_real_ci_pack_golden_exec_policy_selftest_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_ci_pack_golden_exec_policy_selftest_progress()
    )
    full_real_ci_pack_golden_age5_surface_selftest_progress = (
        extract_full_real_ci_pack_golden_age5_surface_selftest_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_ci_pack_golden_age5_surface_selftest_progress()
    )
    full_real_ci_pack_golden_guideblock_selftest_progress = (
        extract_full_real_ci_pack_golden_guideblock_selftest_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_ci_pack_golden_guideblock_selftest_progress()
    )
    full_real_ci_pack_golden_jjaim_flatten_selftest_progress = (
        extract_full_real_ci_pack_golden_jjaim_flatten_selftest_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_ci_pack_golden_jjaim_flatten_selftest_progress()
    )
    full_real_ci_pack_golden_event_model_selftest_progress = (
        extract_full_real_ci_pack_golden_event_model_selftest_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_ci_pack_golden_event_model_selftest_progress()
    )
    full_real_ci_pack_golden_lang_consistency_selftest_progress = (
        extract_full_real_ci_pack_golden_lang_consistency_selftest_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_ci_pack_golden_lang_consistency_selftest_progress()
    )
    full_real_w107_golden_index_selftest_progress = (
        extract_full_real_w107_golden_index_selftest_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_w107_golden_index_selftest_progress()
    )
    full_real_w107_progress_contract_selftest_progress = (
        extract_full_real_w107_progress_contract_selftest_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_w107_progress_contract_selftest_progress()
    )
    full_real_age1_immediate_proof_operation_contract_selftest_progress = (
        extract_full_real_age1_immediate_proof_operation_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_age1_immediate_proof_operation_contract_selftest_progress()
    )
    full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress = (
        extract_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress()
    )
    full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress = (
        extract_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress()
    )
    full_real_proof_certificate_v1_family_contract_selftest_progress = (
        extract_full_real_proof_certificate_v1_family_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_proof_certificate_v1_family_contract_selftest_progress()
    )
    full_real_proof_certificate_family_contract_selftest_progress = (
        extract_full_real_proof_certificate_family_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_proof_certificate_family_contract_selftest_progress()
    )
    full_real_proof_certificate_family_transport_contract_selftest_progress = (
        extract_full_real_proof_certificate_family_transport_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_proof_certificate_family_transport_contract_selftest_progress()
    )
    full_real_proof_family_contract_selftest_progress = (
        extract_full_real_proof_family_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_proof_family_contract_selftest_progress()
    )
    full_real_proof_family_transport_contract_selftest_progress = (
        extract_full_real_proof_family_transport_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_proof_family_transport_contract_selftest_progress()
    )
    full_real_lang_surface_family_contract_selftest_progress = (
        extract_full_real_lang_surface_family_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_lang_surface_family_contract_selftest_progress()
    )
    full_real_lang_surface_family_transport_contract_selftest_progress = (
        extract_full_real_lang_surface_family_transport_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_lang_surface_family_transport_contract_selftest_progress()
    )
    full_real_lang_runtime_family_contract_selftest_progress = (
        extract_full_real_lang_runtime_family_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_lang_runtime_family_contract_selftest_progress()
    )
    full_real_lang_runtime_family_transport_contract_selftest_progress = (
        extract_full_real_lang_runtime_family_transport_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_lang_runtime_family_transport_contract_selftest_progress()
    )
    full_real_gate0_family_contract_selftest_progress = (
        extract_full_real_gate0_family_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_gate0_family_contract_selftest_progress()
    )
    full_real_gate0_surface_family_contract_selftest_progress = (
        extract_full_real_gate0_surface_family_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_gate0_surface_family_contract_selftest_progress()
    )
    full_real_gate0_surface_family_transport_contract_selftest_progress = (
        extract_full_real_gate0_surface_family_transport_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_gate0_surface_family_transport_contract_selftest_progress()
    )
    full_real_gate0_family_transport_contract_selftest_progress = (
        extract_full_real_gate0_family_transport_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_gate0_family_transport_contract_selftest_progress()
    )
    full_real_gate0_transport_family_contract_selftest_progress = (
        extract_full_real_gate0_transport_family_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_gate0_transport_family_contract_selftest_progress()
    )
    full_real_gate0_transport_family_transport_contract_selftest_progress = (
        extract_full_real_gate0_transport_family_transport_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_gate0_transport_family_transport_contract_selftest_progress()
    )
    full_real_gate0_runtime_family_transport_contract_selftest_progress = (
        extract_full_real_gate0_runtime_family_transport_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_gate0_runtime_family_transport_contract_selftest_progress()
    )
    full_real_bogae_alias_family_contract_selftest_progress = (
        extract_full_real_bogae_alias_family_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_bogae_alias_family_contract_selftest_progress()
    )
    full_real_bogae_alias_family_transport_contract_selftest_progress = (
        extract_full_real_bogae_alias_family_transport_contract_selftest_progress_from_criteria(
            criteria, failure_digest
        )
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_bogae_alias_family_transport_contract_selftest_progress()
    )
    full_real_w94_social_pack_check_progress = (
        extract_full_real_w94_social_pack_check_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_w94_social_pack_check_progress()
    )
    full_real_w95_cert_pack_check_progress = (
        extract_full_real_w95_cert_pack_check_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_w95_cert_pack_check_progress()
    )
    full_real_w96_somssi_pack_check_progress = (
        extract_full_real_w96_somssi_pack_check_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_w96_somssi_pack_check_progress()
    )
    full_real_w97_self_heal_pack_check_progress = (
        extract_full_real_w97_self_heal_pack_check_progress_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_w97_self_heal_pack_check_progress()
    )
    full_real_profile_elapsed_map = (
        extract_full_real_profile_elapsed_map_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_profile_elapsed_map()
    )
    full_real_profile_status_map = (
        extract_full_real_profile_status_map_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_profile_status_map()
    )
    full_real_timeout_breakdown = (
        extract_full_real_timeout_breakdown_from_criteria(criteria, failure_digest)
        if bool(with_profile_matrix_full_real_smoke_check)
        else build_age5_full_real_timeout_breakdown()
    )
    report = {
        "schema": "ddn.age5_close_report.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "strict": bool(strict),
        "with_profile_matrix_full_real_smoke_check": bool(with_profile_matrix_full_real_smoke_check),
        "with_runtime_helper_mismatch_negative_check": bool(with_runtime_helper_mismatch_negative_check),
        "with_group_id_summary_mismatch_negative_check": bool(with_group_id_summary_mismatch_negative_check),
        "with_combined_heavy_runtime_helper_check": bool(with_combined_heavy_runtime_helper_check),
        "combined_heavy_env_enabled": bool(combined_heavy_env_enabled),
        "overall_ok": overall_ok,
        "combined_heavy_child_timeout_sec": 0,
        "age5_combined_heavy_timeout_present": "0",
        "age5_combined_heavy_timeout_targets": "-",
        AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY: resolve_age5_combined_heavy_timeout_mode(0),
        "age5_full_real_elapsed_fields_text": AGE5_FULL_REAL_ELAPSED_FIELDS_TEXT,
        "age5_full_real_core_lang_sanity_elapsed_fields_text": AGE5_FULL_REAL_CORE_LANG_SANITY_ELAPSED_FIELDS_TEXT,
        "age5_full_real_core_lang_sanity_progress_fields_text": AGE5_FULL_REAL_CORE_LANG_SANITY_PROGRESS_FIELDS_TEXT,
        "age5_full_real_pipeline_emit_flags_progress_fields_text": AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_PROGRESS_FIELDS_TEXT,
        "age5_full_real_pipeline_emit_flags_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_pipeline_emit_flags_selftest_probe_fields_text": (
            AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROBE_FIELDS_TEXT
        ),
        "age5_full_real_age5_combined_policy_selftest_progress_fields_text": (
            "age5_full_real_age5_combined_policy_selftest_current_case=-|"
            "age5_full_real_age5_combined_policy_selftest_last_completed_case=-|"
            "age5_full_real_age5_combined_policy_selftest_current_format=-|"
            "age5_full_real_age5_combined_policy_selftest_last_completed_format=-|"
            "age5_full_real_age5_combined_policy_selftest_current_probe=-|"
            "age5_full_real_age5_combined_policy_selftest_last_completed_probe=-|"
            "age5_full_real_age5_combined_policy_selftest_total_elapsed_ms=-|"
            "age5_full_real_age5_combined_policy_selftest_progress_present=0"
        ),
        "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_fields_text": (
            AGE5_FULL_REAL_FIXED64_DARWIN_REAL_REPORT_READINESS_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_map_access_contract_check_progress_fields_text": (
            AGE5_FULL_REAL_MAP_ACCESS_CONTRACT_CHECK_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_tensor_v0_cli_check_progress_fields_text": (
            AGE5_FULL_REAL_TENSOR_V0_CLI_CHECK_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_ci_pack_golden_exec_policy_selftest_progress_fields_text": (
            AGE5_FULL_REAL_CI_PACK_GOLDEN_EXEC_POLICY_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_ci_pack_golden_age5_surface_selftest_progress_fields_text": (
            AGE5_FULL_REAL_CI_PACK_GOLDEN_AGE5_SURFACE_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_ci_pack_golden_guideblock_selftest_progress_fields_text": (
            AGE5_FULL_REAL_CI_PACK_GOLDEN_GUIDEBLOCK_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_ci_pack_golden_jjaim_flatten_selftest_progress_fields_text": (
            AGE5_FULL_REAL_CI_PACK_GOLDEN_JJAIM_FLATTEN_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_ci_pack_golden_event_model_selftest_progress_fields_text": (
            AGE5_FULL_REAL_CI_PACK_GOLDEN_EVENT_MODEL_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_ci_pack_golden_lang_consistency_selftest_progress_fields_text": (
            AGE5_FULL_REAL_CI_PACK_GOLDEN_LANG_CONSISTENCY_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_w107_golden_index_selftest_progress_fields_text": (
            AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_w107_progress_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_proof_certificate_v1_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_proof_certificate_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_proof_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROOF_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_proof_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_lang_surface_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_lang_surface_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_lang_runtime_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_gate0_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_GATE0_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_gate0_surface_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_gate0_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_gate0_transport_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_gate0_transport_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_bogae_alias_family_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_fields_text": (
            AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_w94_social_pack_check_progress_fields_text": (
            AGE5_FULL_REAL_W94_SOCIAL_PACK_CHECK_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_w95_cert_pack_check_progress_fields_text": (
            AGE5_FULL_REAL_W95_CERT_PACK_CHECK_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_w96_somssi_pack_check_progress_fields_text": (
            AGE5_FULL_REAL_W96_SOMSSI_PACK_CHECK_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_w97_self_heal_pack_check_progress_fields_text": (
            AGE5_FULL_REAL_W97_SELF_HEAL_PACK_CHECK_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_profile_elapsed_map_fields_text": AGE5_FULL_REAL_PROFILE_ELAPSED_MAP_FIELDS_TEXT,
        "age5_full_real_profile_status_map_fields_text": AGE5_FULL_REAL_PROFILE_STATUS_MAP_FIELDS_TEXT,
        "age5_full_real_timeout_breakdown_fields_text": AGE5_FULL_REAL_TIMEOUT_BREAKDOWN_FIELDS_TEXT,
        AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY: 0,
        AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
        AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_KEY: build_age5_close_digest_selftest_default_field(),
        "age4_proof_snapshot_fields_text": AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
        "age4_proof_snapshot_text": build_age4_proof_snapshot_text(normalized_age4_proof_snapshot),
        "criteria": criteria,
        "paths": {
            "age4_s2_task": str(AGE4_S2_TASK_PATH),
            "s5_baseline_task": str(S5_BASELINE_TASK_PATH),
            "s5_detailed_task": str(S5_DETAILED_TASK_PATH),
            "slot_ui": str(AGE5_SLOT_UI_PATH),
            "app_ui": str(AGE5_APP_UI_PATH),
            "overlay_session_contract": str(OVERLAY_SESSION_CONTRACT_PATH),
            "pack_hint": PACK_HINT,
            "pack_golden": str(PACK_GOLDEN_PATH),
            "session_pack_hint": S6_SESSION_PACK_HINT,
            "session_pack_golden": str(S6_SESSION_PACK_GOLDEN_PATH),
            "age5_surface_pack_contracts": [str(item.get("golden")) for item in AGE5_SURFACE_PACK_CONTRACTS],
        },
        "failure_digest": failure_digest[:20],
        "pending_items": pending_items,
        "repair": repair,
    }
    report.update(normalized_age4_proof_snapshot)
    report.update(normalized_age4_proof_source_fields)
    report.update(full_real_elapsed_summary)
    report.update(full_real_core_lang_sanity_elapsed_summary)
    report.update(full_real_core_lang_sanity_progress)
    report.update(full_real_pipeline_emit_flags_progress)
    report.update(full_real_pipeline_emit_flags_selftest_progress)
    report.update(full_real_pipeline_emit_flags_selftest_probe)
    report.update(full_real_age5_combined_policy_selftest_progress)
    report.update(full_real_profile_matrix_full_real_smoke_policy_selftest_progress)
    report.update(full_real_profile_matrix_full_real_smoke_check_selftest_progress)
    report.update(full_real_fixed64_darwin_real_report_readiness_check_selftest_progress)
    report.update(full_real_map_access_contract_check_progress)
    report.update(full_real_tensor_v0_cli_check_progress)
    report.update(full_real_ci_pack_golden_exec_policy_selftest_progress)
    report.update(full_real_ci_pack_golden_age5_surface_selftest_progress)
    report.update(full_real_ci_pack_golden_guideblock_selftest_progress)
    report.update(full_real_ci_pack_golden_jjaim_flatten_selftest_progress)
    report.update(full_real_ci_pack_golden_event_model_selftest_progress)
    report.update(full_real_ci_pack_golden_lang_consistency_selftest_progress)
    report.update(full_real_w107_golden_index_selftest_progress)
    report.update(full_real_w107_progress_contract_selftest_progress)
    report.update(full_real_age1_immediate_proof_operation_contract_selftest_progress)
    report.update(full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress)
    report.update(full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress)
    report.update(full_real_proof_certificate_v1_family_contract_selftest_progress)
    report.update(full_real_proof_certificate_family_contract_selftest_progress)
    report.update(full_real_proof_certificate_family_transport_contract_selftest_progress)
    report.update(full_real_proof_family_contract_selftest_progress)
    report.update(full_real_proof_family_transport_contract_selftest_progress)
    report.update(full_real_lang_surface_family_contract_selftest_progress)
    report.update(full_real_lang_surface_family_transport_contract_selftest_progress)
    report.update(full_real_lang_runtime_family_contract_selftest_progress)
    report.update(full_real_lang_runtime_family_transport_contract_selftest_progress)
    report.update(full_real_gate0_family_contract_selftest_progress)
    report.update(full_real_gate0_surface_family_contract_selftest_progress)
    report.update(full_real_gate0_surface_family_transport_contract_selftest_progress)
    report.update(full_real_gate0_family_transport_contract_selftest_progress)
    report.update(full_real_gate0_transport_family_contract_selftest_progress)
    report.update(full_real_gate0_transport_family_transport_contract_selftest_progress)
    report.update(full_real_gate0_runtime_family_transport_contract_selftest_progress)
    report.update(full_real_bogae_alias_family_contract_selftest_progress)
    report.update(full_real_bogae_alias_family_transport_contract_selftest_progress)
    report.update(full_real_w94_social_pack_check_progress)
    report.update(full_real_w95_cert_pack_check_progress)
    report.update(full_real_w96_somssi_pack_check_progress)
    report.update(full_real_w97_self_heal_pack_check_progress)
    report.update(full_real_profile_elapsed_map)
    report.update(full_real_profile_status_map)
    report.update(full_real_timeout_breakdown)
    report.update(
        build_age5_combined_heavy_child_summary_fields_from_criteria(
            criteria=criteria,
            full_real_requested=bool(with_profile_matrix_full_real_smoke_check),
            runtime_helper_negative_requested=bool(with_runtime_helper_mismatch_negative_check),
            group_id_summary_negative_requested=bool(with_group_id_summary_mismatch_negative_check),
        )
    )
    report.update(build_age5_combined_heavy_child_summary_default_text_transport_fields())
    return report


def count_nonempty_lines(text: str) -> int:
    return sum(1 for line in text.splitlines() if line.strip())


def load_case_refs(path: Path, key: str) -> tuple[list[str], list[str]]:
    text = load_text(path)
    refs: list[str] = []
    errors: list[str] = []
    for idx, line in enumerate(text.splitlines(), 1):
        raw = line.strip()
        if not raw:
            continue
        try:
            row = json.loads(raw)
        except Exception:
            errors.append(f"line={idx}: invalid json")
            continue
        rel = row.get(key)
        if not isinstance(rel, str) or not rel.strip():
            errors.append(f"line={idx}: missing {key}")
            continue
        refs.append(rel.strip().replace("\\", "/"))
    return refs, errors


def load_overlay_compare_case_refs(path: Path) -> tuple[list[str], list[str]]:
    return load_case_refs(path, "overlay_compare_case")


def load_overlay_session_case_refs(path: Path) -> tuple[list[str], list[str]]:
    return load_case_refs(path, "overlay_session_case")


def build_criteria(
    root: Path,
    *,
    strict: bool = False,
    with_profile_matrix_full_real_smoke_check: bool = False,
    full_real_smoke_step_timeout_sec: int = 0,
    with_runtime_helper_mismatch_negative_check: bool = False,
    with_group_id_summary_mismatch_negative_check: bool = False,
) -> tuple[list[dict[str, object]], list[str], list[str], dict[str, object]]:
    criteria: list[dict[str, object]] = []
    failure_digest: list[str] = []
    pending_items: list[str] = []
    repair: dict[str, object] = {}

    age4_s2_text = load_text(root / AGE4_S2_TASK_PATH)
    s5_baseline_text = load_text(root / S5_BASELINE_TASK_PATH)
    s5_detailed_text = load_text(root / S5_DETAILED_TASK_PATH)
    slot_ui_text = load_text(root / AGE5_SLOT_UI_PATH)
    app_ui_text = load_text(root / AGE5_APP_UI_PATH)
    overlay_session_contract_text = load_text(root / OVERLAY_SESSION_CONTRACT_PATH)

    slot_labels_present = [label for label in SLOT_LABELS if label in slot_ui_text]
    slots_declared_ok = len(slot_labels_present) == len(SLOT_LABELS)
    criteria.append(
        {
            "name": "age5_slot_placeholders_declared",
            "ok": slots_declared_ok,
            "detail": f"present={len(slot_labels_present)}/{len(SLOT_LABELS)} file={AGE5_SLOT_UI_PATH}",
        }
    )
    if not slots_declared_ok:
        missing = [label for label in SLOT_LABELS if label not in set(slot_labels_present)]
        failure_digest.append(f"age5_slot_placeholders_declared: missing={clip(', '.join(missing), 200)}")
        pending_items.extend([f"AGE5 slot placeholder missing: {label}" for label in missing])

    slot_disabled_ok = bool(slot_labels_present) and slot_ui_text.count("class=\"age-slot\"") >= len(SLOT_LABELS) and "disabled" in slot_ui_text
    criteria.append(
        {
            "name": "age5_slot_placeholders_disabled",
            "ok": slot_disabled_ok,
            "detail": f"file={AGE5_SLOT_UI_PATH}",
        }
    )
    if not slot_disabled_ok:
        failure_digest.append("age5_slot_placeholders_disabled: disabled marker missing around AGE5 slots")
        pending_items.append("AGE5 슬롯 A/B/C를 비활성 placeholder 상태로 유지")

    age4_s2_slot_plan_ok = "AGE5 슬롯 A/B/C" in age4_s2_text
    criteria.append(
        {
            "name": "age4_s2_task_mentions_age5_slots",
            "ok": age4_s2_slot_plan_ok,
            "detail": f"path={AGE4_S2_TASK_PATH}",
        }
    )
    if not age4_s2_slot_plan_ok:
        failure_digest.append("age4_s2_task_mentions_age5_slots: missing token 'AGE5 슬롯 A/B/C'")
        pending_items.append("AGE4 S2 task 문서에 AGE5 슬롯 A/B/C 연결 문구 유지")

    s5_baseline_ok = bool(s5_baseline_text) and "baseline+variant" in s5_baseline_text and "축" in s5_baseline_text
    criteria.append(
        {
            "name": "s5_baseline_scope_doc_ready",
            "ok": s5_baseline_ok,
            "detail": f"path={S5_BASELINE_TASK_PATH}",
        }
    )
    if not s5_baseline_ok:
        failure_digest.append("s5_baseline_scope_doc_ready: missing baseline+variant or axis constraints")
        pending_items.append("S5 baseline variant 문서의 범위/축 제한 문구 유지")

    s5_detailed_ok = bool(s5_detailed_text) and "baseline+variant" in s5_detailed_text and "graph_kind" in s5_detailed_text
    criteria.append(
        {
            "name": "s5_detailed_scope_doc_ready",
            "ok": s5_detailed_ok,
            "detail": f"path={S5_DETAILED_TASK_PATH}",
        }
    )
    if not s5_detailed_ok:
        failure_digest.append("s5_detailed_scope_doc_ready: missing baseline+variant or graph_kind constraints")
        pending_items.append("S5 detailed 문서의 graph_kind/axis 동등성 조건 유지")

    s5_pack_hint_ok = PACK_HINT in s5_baseline_text and PACK_HINT in s5_detailed_text
    criteria.append(
        {
            "name": "s5_pack_hint_declared",
            "ok": s5_pack_hint_ok,
            "detail": f"hint={PACK_HINT}",
        }
    )
    if not s5_pack_hint_ok:
        failure_digest.append(f"s5_pack_hint_declared: missing hint '{PACK_HINT}' in S5 task docs")
        pending_items.append("S5 문서에 overlay compare pack 경로 힌트 유지")

    missing_baseline_dod = [token for token in S5_BASELINE_DOD_TOKENS if token not in s5_baseline_text]
    s5_baseline_dod_ok = len(missing_baseline_dod) == 0
    criteria.append(
        {
            "name": "s5_baseline_dod_checked",
            "ok": s5_baseline_dod_ok,
            "detail": f"missing={len(missing_baseline_dod)} path={S5_BASELINE_TASK_PATH} sample={sample_items(missing_baseline_dod)}",
        }
    )
    if not s5_baseline_dod_ok:
        failure_digest.append(f"s5_baseline_dod_checked: missing={clip(', '.join(missing_baseline_dod), 200)}")
        pending_items.append("S5 baseline 문서 DoD 체크박스를 완료 상태로 유지")

    missing_detailed_dod = [token for token in S5_DETAILED_DOD_TOKENS if token not in s5_detailed_text]
    s5_detailed_dod_ok = len(missing_detailed_dod) == 0
    criteria.append(
        {
            "name": "s5_detailed_dod_checked",
            "ok": s5_detailed_dod_ok,
            "detail": f"missing={len(missing_detailed_dod)} path={S5_DETAILED_TASK_PATH} sample={sample_items(missing_detailed_dod)}",
        }
    )
    if not s5_detailed_dod_ok:
        failure_digest.append(f"s5_detailed_dod_checked: missing={clip(', '.join(missing_detailed_dod), 200)}")
        pending_items.append("S5 detailed 문서 DoD 체크박스를 완료 상태로 유지")

    missing_s6_app_tokens = [token for token in S6_SESSION_CONTRACT_APP_TOKENS if token not in app_ui_text]
    missing_s6_module_tokens = [
        token for token in S6_SESSION_CONTRACT_MODULE_TOKENS if token not in overlay_session_contract_text
    ]
    s6_session_app_wired = len(missing_s6_app_tokens) == 0
    s6_session_app_retired = len(missing_s6_app_tokens) == len(S6_SESSION_CONTRACT_APP_TOKENS)
    s6_session_module_ready = len(missing_s6_module_tokens) == 0
    if strict:
        s6_overlay_session_contract_ok = s6_session_app_wired and s6_session_module_ready
    else:
        s6_overlay_session_contract_ok = (s6_session_app_wired and s6_session_module_ready) or s6_session_app_retired
    s6_session_mode = (
        "wired"
        if s6_session_app_wired and s6_session_module_ready
        else "retired"
        if s6_session_app_retired
        else "partial"
    )
    criteria.append(
        {
            "name": "s6_overlay_session_contract_wired",
            "ok": s6_overlay_session_contract_ok,
            "detail": "mode={} app_missing={} module_missing={} app_path={} module_path={}".format(
                s6_session_mode,
                len(missing_s6_app_tokens),
                len(missing_s6_module_tokens),
                AGE5_APP_UI_PATH,
                OVERLAY_SESSION_CONTRACT_PATH,
            ),
        }
    )
    if not s6_overlay_session_contract_ok:
        failure_digest.append(
            "s6_overlay_session_contract_wired: mode={} app_missing={} module_missing={}".format(
                s6_session_mode,
                sample_items(missing_s6_app_tokens),
                sample_items(missing_s6_module_tokens),
            )
        )
        pending_items.append(
            "S6 overlay session contract: strict면 wired 필수, non-strict면 wired/retired 중 일관성 유지"
        )

    missing_s6_view_combo_app_tokens = [
        token for token in S6_VIEW_COMBO_CONTRACT_APP_TOKENS if token not in app_ui_text
    ]
    missing_s6_view_combo_module_tokens = [
        token for token in S6_VIEW_COMBO_CONTRACT_MODULE_TOKENS if token not in overlay_session_contract_text
    ]
    s6_view_combo_app_wired = len(missing_s6_view_combo_app_tokens) == 0
    s6_view_combo_app_retired = len(missing_s6_view_combo_app_tokens) == len(S6_VIEW_COMBO_CONTRACT_APP_TOKENS)
    s6_view_combo_module_ready = len(missing_s6_view_combo_module_tokens) == 0
    if strict:
        s6_view_combo_contract_ok = s6_view_combo_app_wired and s6_view_combo_module_ready
    else:
        s6_view_combo_contract_ok = (
            (s6_view_combo_app_wired and s6_view_combo_module_ready) or s6_view_combo_app_retired
        )
    s6_view_combo_mode = (
        "wired"
        if s6_view_combo_app_wired and s6_view_combo_module_ready
        else "retired"
        if s6_view_combo_app_retired
        else "partial"
    )
    criteria.append(
        {
            "name": "s6_view_combo_session_contract_wired",
            "ok": s6_view_combo_contract_ok,
            "detail": "mode={} app_missing={} module_missing={} app_path={} module_path={}".format(
                s6_view_combo_mode,
                len(missing_s6_view_combo_app_tokens),
                len(missing_s6_view_combo_module_tokens),
                AGE5_APP_UI_PATH,
                OVERLAY_SESSION_CONTRACT_PATH,
            ),
        }
    )
    if not s6_view_combo_contract_ok:
        failure_digest.append(
            "s6_view_combo_session_contract_wired: mode={} app_missing={} module_missing={}".format(
                s6_view_combo_mode,
                sample_items(missing_s6_view_combo_app_tokens),
                sample_items(missing_s6_view_combo_module_tokens),
            )
        )
        pending_items.append(
            "S6 view_combo session contract: strict면 wired 필수, non-strict면 wired/retired 중 일관성 유지"
        )

    missing_s6_session_pack_cases = [
        str(path) for path in S6_SESSION_PACK_CASE_FILES if not (root / path).exists()
    ]
    s6_session_pack_cases_present_ok = len(missing_s6_session_pack_cases) == 0
    criteria.append(
        {
            "name": "s6_session_pack_cases_present",
            "ok": s6_session_pack_cases_present_ok,
            "detail": f"present={len(S6_SESSION_PACK_CASE_FILES) - len(missing_s6_session_pack_cases)}/{len(S6_SESSION_PACK_CASE_FILES)} root={S6_SESSION_PACK_HINT}",
        }
    )
    if not s6_session_pack_cases_present_ok:
        failure_digest.append(
            f"s6_session_pack_cases_present: missing={clip(', '.join(missing_s6_session_pack_cases), 200)}"
        )
        pending_items.append("S6 session roundtrip pack 케이스(c01~c09) 파일 유지")

    s6_session_pack_golden_text = load_text(root / S6_SESSION_PACK_GOLDEN_PATH)
    s6_session_pack_golden_case_count = count_nonempty_lines(s6_session_pack_golden_text)
    s6_session_pack_golden_min_ok = s6_session_pack_golden_case_count >= S6_SESSION_PACK_MIN_CASE_COUNT
    criteria.append(
        {
            "name": "s6_session_pack_golden_min_cases",
            "ok": s6_session_pack_golden_min_ok,
            "detail": f"count={s6_session_pack_golden_case_count} required>={S6_SESSION_PACK_MIN_CASE_COUNT} path={S6_SESSION_PACK_GOLDEN_PATH}",
        }
    )
    if not s6_session_pack_golden_min_ok:
        failure_digest.append(
            f"s6_session_pack_golden_min_cases: count={s6_session_pack_golden_case_count} required>={S6_SESSION_PACK_MIN_CASE_COUNT}"
        )
        pending_items.append("S6 session roundtrip golden.jsonl 케이스 수를 최소 9개 이상으로 유지")

    s6_session_pack_refs, s6_session_pack_parse_errors = load_overlay_session_case_refs(root / S6_SESSION_PACK_GOLDEN_PATH)
    s6_pack_root = Path(S6_SESSION_PACK_HINT)
    expected_s6_refs = [str(path.relative_to(s6_pack_root)).replace("\\", "/") for path in S6_SESSION_PACK_CASE_FILES]
    s6_repair_hint = build_order_repair_hint(S6_SESSION_PACK_GOLDEN_PATH, expected_s6_refs)
    s6_repair_cmd_short = build_order_repair_cmd_short(S6_SESSION_PACK_GOLDEN_PATH)
    s6_repair_cmd = build_order_repair_cmd(S6_SESSION_PACK_GOLDEN_PATH, expected_s6_refs)
    repair["session_order"] = {
        "hint": s6_repair_hint,
        "repair_cmd_short": s6_repair_cmd_short,
        "repair_cmd": s6_repair_cmd,
        "expected_case_list_path": str(S6_SESSION_PACK_GOLDEN_PATH),
        "expected_case_count": len(expected_s6_refs),
        "expected_case_head_tail": head_tail(expected_s6_refs),
    }
    expected_s6_ref_set = set(expected_s6_refs)
    actual_s6_ref_set = set(s6_session_pack_refs)
    s6_missing_in_golden = sorted(expected_s6_ref_set - actual_s6_ref_set)
    s6_unknown_in_golden = sorted(actual_s6_ref_set - expected_s6_ref_set)
    s6_seen: set[str] = set()
    s6_duplicate_in_golden: set[str] = set()
    for ref in s6_session_pack_refs:
        if ref in s6_seen:
            s6_duplicate_in_golden.add(ref)
        s6_seen.add(ref)
    s6_golden_case_map_ok = (
        not s6_session_pack_parse_errors
        and not s6_missing_in_golden
        and not s6_unknown_in_golden
        and not s6_duplicate_in_golden
    )
    criteria.append(
        {
            "name": "s6_session_pack_golden_case_map_match",
            "ok": s6_golden_case_map_ok,
            "detail": "parse_errors={} missing={} unknown={} duplicate={} parse_sample={} missing_sample={} unknown_sample={} duplicate_sample={}".format(
                len(s6_session_pack_parse_errors),
                len(s6_missing_in_golden),
                len(s6_unknown_in_golden),
                len(s6_duplicate_in_golden),
                sample_items(s6_session_pack_parse_errors),
                sample_items(s6_missing_in_golden),
                sample_items(s6_unknown_in_golden),
                sample_items(sorted(s6_duplicate_in_golden)),
            ),
        }
    )
    if not s6_golden_case_map_ok:
        s6_issues: list[str] = []
        if s6_session_pack_parse_errors:
            s6_issues.append(f"parse={clip(', '.join(s6_session_pack_parse_errors), 200)}")
            failure_digest.append(
                "s6_session_pack_golden_case_map_match.parse: sample={} full={}".format(
                    sample_items(s6_session_pack_parse_errors),
                    clip(full_items(s6_session_pack_parse_errors), 600),
                )
            )
        if s6_missing_in_golden:
            s6_issues.append(f"missing={clip(', '.join(s6_missing_in_golden), 200)}")
            failure_digest.append(
                "s6_session_pack_golden_case_map_match.missing: sample={} full={}".format(
                    sample_items(s6_missing_in_golden),
                    clip(full_items(s6_missing_in_golden), 600),
                )
            )
        if s6_unknown_in_golden:
            s6_issues.append(f"unknown={clip(', '.join(s6_unknown_in_golden), 200)}")
            failure_digest.append(
                "s6_session_pack_golden_case_map_match.unknown: sample={} full={}".format(
                    sample_items(s6_unknown_in_golden),
                    clip(full_items(s6_unknown_in_golden), 600),
                )
            )
        if s6_duplicate_in_golden:
            s6_issues.append(f"duplicate={clip(', '.join(sorted(s6_duplicate_in_golden)), 200)}")
            s6_duplicate_sorted = sorted(s6_duplicate_in_golden)
            failure_digest.append(
                "s6_session_pack_golden_case_map_match.duplicate: sample={} full={}".format(
                    sample_items(s6_duplicate_sorted),
                    clip(full_items(s6_duplicate_sorted), 600),
                )
            )
        failure_digest.append(f"s6_session_pack_golden_case_map_match: {'; '.join(s6_issues)}")
        pending_items.append("S6 session roundtrip golden.jsonl과 CASE_FILES(c01~c09) 매핑 일치 유지")

    s6_order_mismatch_index = -1
    if not s6_session_pack_parse_errors:
        s6_min_len = min(len(expected_s6_refs), len(s6_session_pack_refs))
        for idx in range(s6_min_len):
            if expected_s6_refs[idx] != s6_session_pack_refs[idx]:
                s6_order_mismatch_index = idx
                break
        if s6_order_mismatch_index < 0 and len(expected_s6_refs) != len(s6_session_pack_refs):
            s6_order_mismatch_index = s6_min_len
    s6_order_ok = (not s6_session_pack_parse_errors) and (s6_order_mismatch_index < 0)
    s6_expected_at = (
        expected_s6_refs[s6_order_mismatch_index] if 0 <= s6_order_mismatch_index < len(expected_s6_refs) else "-"
    )
    s6_actual_at = (
        s6_session_pack_refs[s6_order_mismatch_index]
        if 0 <= s6_order_mismatch_index < len(s6_session_pack_refs)
        else "-"
    )
    criteria.append(
        {
            "name": "s6_session_pack_golden_case_order_stable",
            "ok": s6_order_ok,
            "detail": "mismatch_index={} expected_at={} actual_at={} expected_window={} actual_window={}".format(
                (s6_order_mismatch_index + 1) if s6_order_mismatch_index >= 0 else 0,
                s6_expected_at,
                s6_actual_at,
                sample_window(expected_s6_refs, s6_order_mismatch_index),
                sample_window(s6_session_pack_refs, s6_order_mismatch_index),
            ),
        }
    )
    if not s6_order_ok:
        failure_digest.append(
            "s6_session_pack_golden_case_order_stable: mismatch_index={} expected_at={} actual_at={} expected_window={} actual_window={}".format(
                (s6_order_mismatch_index + 1) if s6_order_mismatch_index >= 0 else 0,
                s6_expected_at,
                s6_actual_at,
                sample_window(expected_s6_refs, s6_order_mismatch_index),
                sample_window(s6_session_pack_refs, s6_order_mismatch_index),
            )
        )
        failure_digest.append(
            "s6_session_pack_golden_case_order_stable.head_tail: expected_{} actual_{}".format(
                head_tail(expected_s6_refs),
                head_tail(s6_session_pack_refs),
            )
        )
        failure_digest.append(f"s6_session_pack_golden_case_order_stable.{s6_repair_hint}")
        failure_digest.append(
            f"s6_session_pack_golden_case_order_stable.repair_cmd_short: {clip(s6_repair_cmd_short, 700)}"
        )
        failure_digest.append(f"s6_session_pack_golden_case_order_stable.repair_cmd: {clip(s6_repair_cmd, 700)}")
        pending_items.append("S6 session roundtrip golden.jsonl 케이스 순서를 c01~c09 고정 순서로 유지")

    missing_pack_cases = [str(path) for path in PACK_CASE_FILES if not (root / path).exists()]
    pack_cases_present_ok = len(missing_pack_cases) == 0
    criteria.append(
        {
            "name": "s5_pack_cases_present",
            "ok": pack_cases_present_ok,
            "detail": f"present={len(PACK_CASE_FILES) - len(missing_pack_cases)}/{len(PACK_CASE_FILES)} root={PACK_HINT}",
        }
    )
    if not pack_cases_present_ok:
        failure_digest.append(f"s5_pack_cases_present: missing={clip(', '.join(missing_pack_cases), 200)}")
        pending_items.append("S5 overlay compare pack 케이스(c01~c76) 파일 유지")

    pack_golden_text = load_text(root / PACK_GOLDEN_PATH)
    pack_golden_case_count = count_nonempty_lines(pack_golden_text)
    pack_golden_min_ok = pack_golden_case_count >= PACK_MIN_CASE_COUNT
    criteria.append(
        {
            "name": "s5_pack_golden_min_cases",
            "ok": pack_golden_min_ok,
            "detail": f"count={pack_golden_case_count} required>={PACK_MIN_CASE_COUNT} path={PACK_GOLDEN_PATH}",
        }
    )
    if not pack_golden_min_ok:
        failure_digest.append(
            f"s5_pack_golden_min_cases: count={pack_golden_case_count} required>={PACK_MIN_CASE_COUNT}"
        )
        pending_items.append("S5 overlay compare golden.jsonl 케이스 수를 최소 76개 이상으로 유지")

    pack_golden_refs, pack_golden_parse_errors = load_overlay_compare_case_refs(root / PACK_GOLDEN_PATH)
    pack_root = Path(PACK_HINT)
    expected_refs = [str(path.relative_to(pack_root)).replace("\\", "/") for path in PACK_CASE_FILES]
    repair_hint = build_order_repair_hint(PACK_GOLDEN_PATH, expected_refs)
    repair_cmd_short = build_order_repair_cmd_short(PACK_GOLDEN_PATH)
    repair_cmd = build_order_repair_cmd(PACK_GOLDEN_PATH, expected_refs)
    repair["order"] = {
        "hint": repair_hint,
        "repair_cmd_short": repair_cmd_short,
        "repair_cmd": repair_cmd,
        "expected_case_list_path": str(PACK_GOLDEN_PATH),
        "expected_case_count": len(expected_refs),
        "expected_case_head_tail": head_tail(expected_refs),
    }
    expected_ref_set = set(expected_refs)
    actual_ref_set = set(pack_golden_refs)
    missing_in_golden = sorted(expected_ref_set - actual_ref_set)
    unknown_in_golden = sorted(actual_ref_set - expected_ref_set)
    seen: set[str] = set()
    duplicate_in_golden: set[str] = set()
    for ref in pack_golden_refs:
        if ref in seen:
            duplicate_in_golden.add(ref)
        seen.add(ref)
    golden_case_map_ok = (
        not pack_golden_parse_errors and not missing_in_golden and not unknown_in_golden and not duplicate_in_golden
    )
    criteria.append(
        {
            "name": "s5_pack_golden_case_map_match",
            "ok": golden_case_map_ok,
            "detail": "parse_errors={} missing={} unknown={} duplicate={} parse_sample={} missing_sample={} unknown_sample={} duplicate_sample={}".format(
                len(pack_golden_parse_errors),
                len(missing_in_golden),
                len(unknown_in_golden),
                len(duplicate_in_golden),
                sample_items(pack_golden_parse_errors),
                sample_items(missing_in_golden),
                sample_items(unknown_in_golden),
                sample_items(sorted(duplicate_in_golden)),
            ),
        }
    )
    if not golden_case_map_ok:
        issues: list[str] = []
        if pack_golden_parse_errors:
            issues.append(f"parse={clip(', '.join(pack_golden_parse_errors), 200)}")
            failure_digest.append(
                "s5_pack_golden_case_map_match.parse: sample={} full={}".format(
                    sample_items(pack_golden_parse_errors),
                    clip(full_items(pack_golden_parse_errors), 600),
                )
            )
        if missing_in_golden:
            issues.append(f"missing={clip(', '.join(missing_in_golden), 200)}")
            failure_digest.append(
                "s5_pack_golden_case_map_match.missing: sample={} full={}".format(
                    sample_items(missing_in_golden),
                    clip(full_items(missing_in_golden), 600),
                )
            )
        if unknown_in_golden:
            issues.append(f"unknown={clip(', '.join(unknown_in_golden), 200)}")
            failure_digest.append(
                "s5_pack_golden_case_map_match.unknown: sample={} full={}".format(
                    sample_items(unknown_in_golden),
                    clip(full_items(unknown_in_golden), 600),
                )
            )
        if duplicate_in_golden:
            issues.append(f"duplicate={clip(', '.join(sorted(duplicate_in_golden)), 200)}")
            duplicate_sorted = sorted(duplicate_in_golden)
            failure_digest.append(
                "s5_pack_golden_case_map_match.duplicate: sample={} full={}".format(
                    sample_items(duplicate_sorted),
                    clip(full_items(duplicate_sorted), 600),
                )
            )
        failure_digest.append(f"s5_pack_golden_case_map_match: {'; '.join(issues)}")
        pending_items.append("S5 overlay compare golden.jsonl과 PACK_CASE_FILES(c01~c76) 매핑 일치 유지")

    order_mismatch_index = -1
    if not pack_golden_parse_errors:
        min_len = min(len(expected_refs), len(pack_golden_refs))
        for idx in range(min_len):
            if expected_refs[idx] != pack_golden_refs[idx]:
                order_mismatch_index = idx
                break
        if order_mismatch_index < 0 and len(expected_refs) != len(pack_golden_refs):
            order_mismatch_index = min_len
    order_ok = (not pack_golden_parse_errors) and (order_mismatch_index < 0)
    expected_at = expected_refs[order_mismatch_index] if 0 <= order_mismatch_index < len(expected_refs) else "-"
    actual_at = pack_golden_refs[order_mismatch_index] if 0 <= order_mismatch_index < len(pack_golden_refs) else "-"
    criteria.append(
        {
            "name": "s5_pack_golden_case_order_stable",
            "ok": order_ok,
            "detail": "mismatch_index={} expected_at={} actual_at={} expected_window={} actual_window={}".format(
                (order_mismatch_index + 1) if order_mismatch_index >= 0 else 0,
                expected_at,
                actual_at,
                sample_window(expected_refs, order_mismatch_index),
                sample_window(pack_golden_refs, order_mismatch_index),
            ),
        }
    )
    if not order_ok:
        failure_digest.append(
            "s5_pack_golden_case_order_stable: mismatch_index={} expected_at={} actual_at={} expected_window={} actual_window={}".format(
                (order_mismatch_index + 1) if order_mismatch_index >= 0 else 0,
                expected_at,
                actual_at,
                sample_window(expected_refs, order_mismatch_index),
                sample_window(pack_golden_refs, order_mismatch_index),
            )
        )
        failure_digest.append(
            "s5_pack_golden_case_order_stable.head_tail: expected_{} actual_{}".format(
                head_tail(expected_refs),
                head_tail(pack_golden_refs),
            )
        )
        failure_digest.append(f"s5_pack_golden_case_order_stable.{repair_hint}")
        failure_digest.append(f"s5_pack_golden_case_order_stable.repair_cmd_short: {clip(repair_cmd_short, 700)}")
        failure_digest.append(f"s5_pack_golden_case_order_stable.repair_cmd: {clip(repair_cmd, 700)}")
        pending_items.append("S5 overlay compare golden.jsonl 케이스 순서를 c01~c76 고정 순서로 유지")

    invalid_case_tokens: list[str] = []
    for path in PACK_CASE_FILES:
        text = load_text(root / path)
        if "\"overlay_ok\"" not in text:
            invalid_case_tokens.append(str(path))
    pack_cases_token_ok = len(invalid_case_tokens) == 0
    criteria.append(
        {
            "name": "s5_pack_cases_overlay_ok_token",
            "ok": pack_cases_token_ok,
            "detail": f"invalid={len(invalid_case_tokens)}",
        }
    )
    if not pack_cases_token_ok:
        failure_digest.append(f"s5_pack_cases_overlay_ok_token: missing overlay_ok token in {clip(', '.join(invalid_case_tokens), 200)}")
        pending_items.append("S5 pack 케이스에 overlay_ok 기대값 유지")

    missing_surface_golden_paths: list[str] = []
    underflow_surface_cases: list[str] = []
    missing_surface_tokens: list[str] = []
    for contract in AGE5_SURFACE_PACK_CONTRACTS:
        contract_name = str(contract.get("name", "")).strip() or "unknown"
        golden_path = contract.get("golden")
        if not isinstance(golden_path, Path):
            missing_surface_golden_paths.append(f"{contract_name}:invalid_path")
            continue
        full_golden_path = root / golden_path
        if not full_golden_path.exists():
            missing_surface_golden_paths.append(f"{contract_name}:{golden_path}")
            continue
        golden_text = load_text(full_golden_path)
        case_count = count_nonempty_lines(golden_text)
        min_cases = int(contract.get("min_cases", 0))
        if case_count < min_cases:
            underflow_surface_cases.append(
                f"{contract_name}:{case_count}<{min_cases}:{golden_path}"
            )
        for token in contract.get("tokens", []):
            token_text = str(token)
            if token_text and token_text not in golden_text:
                missing_surface_tokens.append(f"{contract_name}:{token_text}")

    surface_pack_paths_ok = len(missing_surface_golden_paths) == 0
    criteria.append(
        {
            "name": "age5_surface_pack_contract_paths_present",
            "ok": surface_pack_paths_ok,
            "detail": "missing={} sample={}".format(
                len(missing_surface_golden_paths),
                sample_items(missing_surface_golden_paths),
            ),
        }
    )
    if not surface_pack_paths_ok:
        failure_digest.append(
            "age5_surface_pack_contract_paths_present: missing={}".format(
                clip(full_items(missing_surface_golden_paths), 500)
            )
        )
        pending_items.append("AGE5 surface pack golden.jsonl 경로(핵심 8개) 유지")

    surface_pack_min_cases_ok = len(underflow_surface_cases) == 0
    criteria.append(
        {
            "name": "age5_surface_pack_contract_min_cases",
            "ok": surface_pack_min_cases_ok,
            "detail": "underflow={} sample={}".format(
                len(underflow_surface_cases),
                sample_items(underflow_surface_cases),
            ),
        }
    )
    if not surface_pack_min_cases_ok:
        failure_digest.append(
            "age5_surface_pack_contract_min_cases: underflow={}".format(
                clip(full_items(underflow_surface_cases), 500)
            )
        )
        pending_items.append("AGE5 surface pack golden 최소 케이스 수 계약 유지")

    surface_pack_tokens_ok = len(missing_surface_tokens) == 0
    criteria.append(
        {
            "name": "age5_surface_pack_contract_tokens_present",
            "ok": surface_pack_tokens_ok,
            "detail": "missing={} sample={}".format(
                len(missing_surface_tokens),
                sample_items(missing_surface_tokens),
            ),
        }
    )
    if not surface_pack_tokens_ok:
        failure_digest.append(
            "age5_surface_pack_contract_tokens_present: missing={}".format(
                clip(full_items(missing_surface_tokens), 500)
            )
        )
        pending_items.append("AGE5 surface pack 핵심 토큰(별칭/하드컷/정본 무경고) 유지")

    missing_profile_scripts = [
        f"{name}:{path}" for name, path in CI_PROFILE_GATE_SCRIPTS.items() if not (root / path).exists()
    ]
    profile_scripts_ok = len(missing_profile_scripts) == 0
    criteria.append(
        {
            "name": "age5_ci_profile_gate_scripts_present",
            "ok": profile_scripts_ok,
            "detail": "missing={} sample={}".format(
                len(missing_profile_scripts),
                sample_items(missing_profile_scripts),
            ),
        }
    )
    if not profile_scripts_ok:
        failure_digest.append(
            "age5_ci_profile_gate_scripts_present: missing={}".format(
                clip(full_items(missing_profile_scripts), 500)
            )
        )
        pending_items.append("AGE5 CI profile gate 스크립트 4종(split/core_lang/full/seamgrim) 유지")

    core_profile_text = load_text(root / CI_PROFILE_GATE_SCRIPTS["core_lang"])
    full_profile_text = load_text(root / CI_PROFILE_GATE_SCRIPTS["full"])
    seamgrim_profile_text = load_text(root / CI_PROFILE_GATE_SCRIPTS["seamgrim"])
    missing_core_profile_chain_tokens = [
        token for token in CI_PROFILE_CORE_LANG_CHAIN_TOKENS if token not in core_profile_text
    ]
    missing_full_profile_chain_tokens = [
        token for token in CI_PROFILE_FULL_CHAIN_TOKENS if token not in full_profile_text
    ]
    missing_seamgrim_profile_chain_tokens = [
        token for token in CI_PROFILE_SEAMGRIM_CHAIN_TOKENS if token not in seamgrim_profile_text
    ]
    profile_chain_tokens_ok = len(missing_core_profile_chain_tokens) == 0 and len(
        missing_full_profile_chain_tokens
    ) == 0 and len(missing_seamgrim_profile_chain_tokens) == 0
    criteria.append(
        {
            "name": "age5_ci_profile_gate_sync_chain_tokens",
            "ok": profile_chain_tokens_ok,
            "detail": "core_missing={} full_missing={} seamgrim_missing={} core_sample={} full_sample={} seamgrim_sample={}".format(
                len(missing_core_profile_chain_tokens),
                len(missing_full_profile_chain_tokens),
                len(missing_seamgrim_profile_chain_tokens),
                sample_items(missing_core_profile_chain_tokens),
                sample_items(missing_full_profile_chain_tokens),
                sample_items(missing_seamgrim_profile_chain_tokens),
            ),
        }
    )
    if not profile_chain_tokens_ok:
        if missing_core_profile_chain_tokens:
            failure_digest.append(
                "age5_ci_profile_gate_sync_chain_tokens.core: missing={}".format(
                    clip(full_items(missing_core_profile_chain_tokens), 500)
                )
            )
        if missing_full_profile_chain_tokens:
            failure_digest.append(
                "age5_ci_profile_gate_sync_chain_tokens.full: missing={}".format(
                    clip(full_items(missing_full_profile_chain_tokens), 500)
                )
            )
        if missing_seamgrim_profile_chain_tokens:
            failure_digest.append(
                "age5_ci_profile_gate_sync_chain_tokens.seamgrim: missing={}".format(
                    clip(full_items(missing_seamgrim_profile_chain_tokens), 500)
                )
            )
        pending_items.append("AGE5 CI core_lang/full/seamgrim gate에 sync_readiness 연쇄 검증 토큰 유지")

    split_profile_text = load_text(root / CI_PROFILE_GATE_SCRIPTS["split"])
    missing_split_contract_tokens = [
        token for token in CI_PROFILE_SPLIT_CONTRACT_TOKENS if token not in split_profile_text
    ]
    split_contract_tokens_ok = len(missing_split_contract_tokens) == 0
    criteria.append(
        {
            "name": "age5_ci_profile_split_contract_tokens_present",
            "ok": split_contract_tokens_ok,
            "detail": "missing={} sample={}".format(
                len(missing_split_contract_tokens),
                sample_items(missing_split_contract_tokens),
            ),
        }
    )
    if not split_contract_tokens_ok:
        failure_digest.append(
            "age5_ci_profile_split_contract_tokens_present: missing={}".format(
                clip(full_items(missing_split_contract_tokens), 500)
            )
        )
        pending_items.append("AGE5 CI split contract checker에 profile/sync 토큰 계약 유지")

    missing_core_profile_report_path_tokens = [
        token for token in CI_PROFILE_CORE_LANG_REPORT_PATH_TOKENS if token not in core_profile_text
    ]
    missing_full_profile_report_path_tokens = [
        token for token in CI_PROFILE_FULL_REPORT_PATH_TOKENS if token not in full_profile_text
    ]
    missing_seamgrim_profile_report_path_tokens = [
        token for token in CI_PROFILE_SEAMGRIM_REPORT_PATH_TOKENS if token not in seamgrim_profile_text
    ]
    profile_report_path_tokens_ok = (
        len(missing_core_profile_report_path_tokens) == 0
        and len(missing_full_profile_report_path_tokens) == 0
        and len(missing_seamgrim_profile_report_path_tokens) == 0
    )
    criteria.append(
        {
            "name": "age5_ci_profile_gate_report_path_tokens",
            "ok": profile_report_path_tokens_ok,
            "detail": "core_missing={} full_missing={} seamgrim_missing={} core_sample={} full_sample={} seamgrim_sample={}".format(
                len(missing_core_profile_report_path_tokens),
                len(missing_full_profile_report_path_tokens),
                len(missing_seamgrim_profile_report_path_tokens),
                sample_items(missing_core_profile_report_path_tokens),
                sample_items(missing_full_profile_report_path_tokens),
                sample_items(missing_seamgrim_profile_report_path_tokens),
            ),
        }
    )
    if not profile_report_path_tokens_ok:
        if missing_core_profile_report_path_tokens:
            failure_digest.append(
                "age5_ci_profile_gate_report_path_tokens.core: missing={}".format(
                    clip(full_items(missing_core_profile_report_path_tokens), 500)
                )
            )
        if missing_full_profile_report_path_tokens:
            failure_digest.append(
                "age5_ci_profile_gate_report_path_tokens.full: missing={}".format(
                    clip(full_items(missing_full_profile_report_path_tokens), 500)
                )
            )
        if missing_seamgrim_profile_report_path_tokens:
            failure_digest.append(
                "age5_ci_profile_gate_report_path_tokens.seamgrim: missing={}".format(
                    clip(full_items(missing_seamgrim_profile_report_path_tokens), 500)
                )
            )
        pending_items.append("AGE5 CI core_lang/full/seamgrim gate에 sync_readiness report 경로 계약 토큰 유지")

    sync_readiness_contract_text = load_text(root / CI_SYNC_READINESS_REPORT_PATH_CONTRACT_SCRIPT)
    missing_sync_readiness_report_path_contract_tokens = [
        token
        for token in CI_SYNC_READINESS_REPORT_PATH_CONTRACT_TOKENS
        if token not in sync_readiness_contract_text
    ]
    sync_readiness_report_path_contract_ok = len(missing_sync_readiness_report_path_contract_tokens) == 0
    criteria.append(
        {
            "name": "age5_ci_sync_readiness_report_path_contract_tokens",
            "ok": sync_readiness_report_path_contract_ok,
            "detail": "missing={} sample={} path={}".format(
                len(missing_sync_readiness_report_path_contract_tokens),
                sample_items(missing_sync_readiness_report_path_contract_tokens),
                CI_SYNC_READINESS_REPORT_PATH_CONTRACT_SCRIPT,
            ),
        }
    )
    if not sync_readiness_report_path_contract_ok:
        failure_digest.append(
            "age5_ci_sync_readiness_report_path_contract_tokens: missing={}".format(
                clip(full_items(missing_sync_readiness_report_path_contract_tokens), 500)
            )
        )
        pending_items.append("AGE5 CI sync_readiness check의 report 경로 계약 토큰(--report-prefix/--json-out/out_path) 유지")

    gate_report_index_contract_paths = [
        CI_GATE_REPORT_INDEX_CONTRACT_SCRIPT,
        CI_GATE_REPORT_INDEX_CHECK_SCRIPT,
        CI_GATE_REPORT_INDEX_SELFTEST_SCRIPT,
        CI_GATE_REPORT_INDEX_DIAGNOSTICS_SCRIPT,
        CI_GATE_REPORT_INDEX_CODE_MAP,
    ]
    missing_gate_report_index_contract_paths = [
        str(path) for path in gate_report_index_contract_paths if not (root / path).exists()
    ]
    gate_report_index_contract_paths_ok = len(missing_gate_report_index_contract_paths) == 0
    criteria.append(
        {
            "name": "age5_ci_gate_report_index_contract_paths_present",
            "ok": gate_report_index_contract_paths_ok,
            "detail": "missing={} sample={}".format(
                len(missing_gate_report_index_contract_paths),
                sample_items(missing_gate_report_index_contract_paths),
            ),
        }
    )
    if not gate_report_index_contract_paths_ok:
        failure_digest.append(
            "age5_ci_gate_report_index_contract_paths_present: missing={}".format(
                clip(full_items(missing_gate_report_index_contract_paths), 500)
            )
        )
        pending_items.append("AGE5 CI report-index 계약 스크립트/check/selftest/diagnostics/code-map 파일 유지")

    gate_report_index_contract_text = load_text(root / CI_GATE_REPORT_INDEX_CONTRACT_SCRIPT)
    gate_report_index_check_text = load_text(root / CI_GATE_REPORT_INDEX_CHECK_SCRIPT)
    gate_report_index_code_map_text = load_text(root / CI_GATE_REPORT_INDEX_CODE_MAP)
    missing_gate_report_index_contract_tokens = [
        token for token in CI_GATE_REPORT_INDEX_CONTRACT_TOKENS if token not in gate_report_index_contract_text
    ]
    missing_gate_report_index_check_tokens = [
        token for token in CI_GATE_REPORT_INDEX_CHECK_TOKENS if token not in gate_report_index_check_text
    ]
    missing_gate_report_index_code_map_tokens = [
        token for token in CI_GATE_REPORT_INDEX_CODE_MAP_TOKENS if token not in gate_report_index_code_map_text
    ]
    gate_report_index_contract_tokens_ok = (
        len(missing_gate_report_index_contract_tokens) == 0
        and len(missing_gate_report_index_check_tokens) == 0
        and len(missing_gate_report_index_code_map_tokens) == 0
    )
    criteria.append(
        {
            "name": "age5_ci_gate_report_index_contract_tokens_present",
            "ok": gate_report_index_contract_tokens_ok,
            "detail": "aggregate_missing={} check_missing={} code_map_missing={} aggregate_sample={} check_sample={} code_map_sample={}".format(
                len(missing_gate_report_index_contract_tokens),
                len(missing_gate_report_index_check_tokens),
                len(missing_gate_report_index_code_map_tokens),
                sample_items(missing_gate_report_index_contract_tokens),
                sample_items(missing_gate_report_index_check_tokens),
                sample_items(missing_gate_report_index_code_map_tokens),
            ),
        }
    )
    if not gate_report_index_contract_tokens_ok:
        if missing_gate_report_index_contract_tokens:
            failure_digest.append(
                "age5_ci_gate_report_index_contract_tokens_present.aggregate: missing={}".format(
                    clip(full_items(missing_gate_report_index_contract_tokens), 500)
                )
            )
        if missing_gate_report_index_check_tokens:
            failure_digest.append(
                "age5_ci_gate_report_index_contract_tokens_present.check: missing={}".format(
                    clip(full_items(missing_gate_report_index_check_tokens), 500)
                )
            )
        if missing_gate_report_index_code_map_tokens:
            failure_digest.append(
                "age5_ci_gate_report_index_contract_tokens_present.code_map: missing={}".format(
                    clip(full_items(missing_gate_report_index_code_map_tokens), 500)
                )
            )
        pending_items.append("AGE5 CI report-index 계약 토큰(check/selftest/diagnostics + code-map) 유지")

    missing_seamgrim_diag_parity_scripts = [
        f"{name}:{path}"
        for name, path in CI_SEAMGRIM_DIAG_PARITY_SCRIPTS.items()
        if not (root / path).exists()
    ]
    seamgrim_diag_parity_scripts_ok = len(missing_seamgrim_diag_parity_scripts) == 0
    criteria.append(
        {
            "name": "age5_seamgrim_diag_parity_scripts_present",
            "ok": seamgrim_diag_parity_scripts_ok,
            "detail": "missing={} sample={}".format(
                len(missing_seamgrim_diag_parity_scripts),
                sample_items(missing_seamgrim_diag_parity_scripts),
            ),
        }
    )
    if not seamgrim_diag_parity_scripts_ok:
        failure_digest.append(
            "age5_seamgrim_diag_parity_scripts_present: missing={}".format(
                clip(full_items(missing_seamgrim_diag_parity_scripts), 500)
            )
        )
        pending_items.append("AGE5 seamgrim parity 스크립트(wasm/overlay/session/wired) 4종 유지")

    seamgrim_wasm_cli_diag_parity_script = CI_SEAMGRIM_DIAG_PARITY_SCRIPTS["wasm_cli"]
    seamgrim_wasm_cli_diag_parity_text = load_text(root / seamgrim_wasm_cli_diag_parity_script)
    missing_seamgrim_wasm_cli_diag_parity_tokens = [
        token
        for token in CI_SEAMGRIM_WASM_CLI_DIAG_PARITY_TOKENS
        if token not in seamgrim_wasm_cli_diag_parity_text
    ]
    seamgrim_wasm_cli_diag_parity_tokens_ok = len(missing_seamgrim_wasm_cli_diag_parity_tokens) == 0
    criteria.append(
        {
            "name": "age5_seamgrim_wasm_cli_diag_parity_tokens_present",
            "ok": seamgrim_wasm_cli_diag_parity_tokens_ok,
            "detail": "missing={} sample={} path={}".format(
                len(missing_seamgrim_wasm_cli_diag_parity_tokens),
                sample_items(missing_seamgrim_wasm_cli_diag_parity_tokens),
                seamgrim_wasm_cli_diag_parity_script,
            ),
        }
    )
    if not seamgrim_wasm_cli_diag_parity_tokens_ok:
        failure_digest.append(
            "age5_seamgrim_wasm_cli_diag_parity_tokens_present: missing={}".format(
                clip(full_items(missing_seamgrim_wasm_cli_diag_parity_tokens), 500)
            )
        )
        pending_items.append("AGE5 wasm/cli parity 스크립트에 overlay/session 진단 parity 토큰 유지")

    if with_profile_matrix_full_real_smoke_check:
        smoke_script_ok = (root / CI_PROFILE_MATRIX_FULL_REAL_SMOKE_SCRIPT).exists()
        criteria.append(
            {
                "name": "age5_ci_profile_matrix_full_real_smoke_script_present",
                "ok": smoke_script_ok,
                "detail": f"path={CI_PROFILE_MATRIX_FULL_REAL_SMOKE_SCRIPT}",
            }
        )
        if not smoke_script_ok:
            failure_digest.append(
                f"age5_ci_profile_matrix_full_real_smoke_script_present: missing={CI_PROFILE_MATRIX_FULL_REAL_SMOKE_SCRIPT}"
            )
            pending_items.append("AGE5 profile-matrix full-real smoke 스크립트 유지")
        else:
            smoke_cmd = [
                sys.executable,
                str(CI_PROFILE_MATRIX_FULL_REAL_SMOKE_SCRIPT),
                PROFILE_MATRIX_FULL_REAL_SMOKE_ALLOW_FLAG,
            ]
            if int(full_real_smoke_step_timeout_sec or 0) > 0:
                smoke_cmd.extend(["--step-timeout-sec", str(int(full_real_smoke_step_timeout_sec))])
            smoke_proc = run_text(smoke_cmd, root)
            smoke_stdout = str(smoke_proc.stdout or "")
            smoke_ok = smoke_proc.returncode == 0 and PROFILE_MATRIX_FULL_REAL_SMOKE_STATUS_PASS in smoke_stdout
            smoke_timeout_breakdown = parse_timeout_step_profiles(smoke_stdout)
            smoke_elapsed_summary = parse_full_real_elapsed_summary(smoke_stdout)
            smoke_core_lang_sanity_elapsed_summary = parse_full_real_core_lang_sanity_elapsed_summary(smoke_stdout)
            smoke_core_lang_sanity_progress = parse_full_real_core_lang_sanity_progress(smoke_stdout)
            smoke_pipeline_emit_flags_progress = parse_full_real_pipeline_emit_flags_progress(smoke_stdout)
            smoke_pipeline_emit_flags_selftest_progress = parse_full_real_pipeline_emit_flags_selftest_progress(
                smoke_stdout
            )
            smoke_pipeline_emit_flags_selftest_probe = parse_full_real_pipeline_emit_flags_selftest_probe(
                smoke_stdout
            )
            smoke_age5_combined_policy_selftest_progress = parse_full_real_age5_combined_policy_selftest_progress(
                smoke_stdout
            )
            smoke_profile_matrix_full_real_smoke_policy_selftest_progress = (
                parse_full_real_profile_matrix_full_real_smoke_policy_selftest_progress(smoke_stdout)
            )
            smoke_profile_matrix_full_real_smoke_check_selftest_progress = (
                parse_full_real_profile_matrix_full_real_smoke_check_selftest_progress(smoke_stdout)
            )
            smoke_fixed64_darwin_real_report_readiness_check_selftest_progress = (
                parse_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress(smoke_stdout)
            )
            smoke_map_access_contract_check_progress = parse_full_real_map_access_contract_check_progress(
                smoke_stdout
            )
            smoke_tensor_v0_cli_check_progress = parse_full_real_tensor_v0_cli_check_progress(smoke_stdout)
            smoke_ci_pack_golden_exec_policy_selftest_progress = (
                parse_full_real_ci_pack_golden_exec_policy_selftest_progress(smoke_stdout)
            )
            smoke_ci_pack_golden_age5_surface_selftest_progress = (
                parse_full_real_ci_pack_golden_age5_surface_selftest_progress(smoke_stdout)
            )
            smoke_ci_pack_golden_guideblock_selftest_progress = (
                parse_full_real_ci_pack_golden_guideblock_selftest_progress(smoke_stdout)
            )
            smoke_ci_pack_golden_jjaim_flatten_selftest_progress = (
                parse_full_real_ci_pack_golden_jjaim_flatten_selftest_progress(smoke_stdout)
            )
            smoke_ci_pack_golden_event_model_selftest_progress = (
                parse_full_real_ci_pack_golden_event_model_selftest_progress(smoke_stdout)
            )
            smoke_ci_pack_golden_lang_consistency_selftest_progress = (
                parse_full_real_ci_pack_golden_lang_consistency_selftest_progress(smoke_stdout)
            )
            smoke_w107_golden_index_selftest_progress = (
                parse_full_real_w107_golden_index_selftest_progress(smoke_stdout)
            )
            smoke_w107_progress_contract_selftest_progress = (
                parse_full_real_w107_progress_contract_selftest_progress(smoke_stdout)
            )
            smoke_age1_immediate_proof_operation_contract_selftest_progress = (
                parse_full_real_age1_immediate_proof_operation_contract_selftest_progress(smoke_stdout)
            )
            smoke_proof_certificate_v1_consumer_transport_contract_selftest_progress = (
                parse_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress(
                    smoke_stdout
                )
            )
            smoke_proof_certificate_v1_verify_report_digest_contract_selftest_progress = (
                parse_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress(
                    smoke_stdout
                )
            )
            smoke_w94_social_pack_check_progress = parse_full_real_w94_social_pack_check_progress(smoke_stdout)
            smoke_w95_cert_pack_check_progress = parse_full_real_w95_cert_pack_check_progress(smoke_stdout)
            smoke_w96_somssi_pack_check_progress = parse_full_real_w96_somssi_pack_check_progress(smoke_stdout)
            smoke_w97_self_heal_pack_check_progress = parse_full_real_w97_self_heal_pack_check_progress(smoke_stdout)
            smoke_profile_status_map = parse_full_real_profile_status_map(smoke_stdout)
            smoke_summary_tokens: list[str] = []
            if smoke_elapsed_summary["age5_full_real_elapsed_present"] == "1":
                smoke_summary_tokens.extend(
                    [
                        f"ci_profile_matrix_full_real_total_elapsed_ms={smoke_elapsed_summary['age5_full_real_total_elapsed_ms']}",
                        f"ci_profile_matrix_full_real_slowest_profile={smoke_elapsed_summary['age5_full_real_slowest_profile']}",
                        f"ci_profile_matrix_full_real_slowest_elapsed_ms={smoke_elapsed_summary['age5_full_real_slowest_elapsed_ms']}",
                    ]
                )
            if (
                smoke_core_lang_sanity_elapsed_summary["age5_full_real_core_lang_sanity_elapsed_present"]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "ci_profile_core_lang_sanity_total_elapsed_ms="
                        + smoke_core_lang_sanity_elapsed_summary["age5_full_real_core_lang_sanity_total_elapsed_ms"],
                        "ci_profile_core_lang_sanity_slowest_step="
                        + smoke_core_lang_sanity_elapsed_summary["age5_full_real_core_lang_sanity_slowest_step"],
                        "ci_profile_core_lang_sanity_slowest_elapsed_ms="
                        + smoke_core_lang_sanity_elapsed_summary["age5_full_real_core_lang_sanity_slowest_elapsed_ms"],
                    ]
                )
            if smoke_core_lang_sanity_progress["age5_full_real_core_lang_sanity_progress_present"] == "1":
                smoke_summary_tokens.extend(
                    [
                        "ci_sanity_current_step="
                        + smoke_core_lang_sanity_progress["age5_full_real_core_lang_sanity_current_step"],
                        "ci_sanity_last_completed_step="
                        + smoke_core_lang_sanity_progress["age5_full_real_core_lang_sanity_last_completed_step"],
                    ]
                )
            if smoke_pipeline_emit_flags_progress["age5_full_real_pipeline_emit_flags_progress_present"] == "1":
                smoke_summary_tokens.extend(
                    [
                        "ci_pipeline_emit_flags_current_section="
                        + smoke_pipeline_emit_flags_progress["age5_full_real_pipeline_emit_flags_current_section"],
                        "ci_pipeline_emit_flags_last_completed_section="
                        + smoke_pipeline_emit_flags_progress["age5_full_real_pipeline_emit_flags_last_completed_section"],
                        "ci_pipeline_emit_flags_total_elapsed_ms="
                        + smoke_pipeline_emit_flags_progress["age5_full_real_pipeline_emit_flags_total_elapsed_ms"],
                    ]
                )
            if (
                smoke_pipeline_emit_flags_selftest_progress[
                    "age5_full_real_pipeline_emit_flags_selftest_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "ci_pipeline_emit_flags_selftest_current_case="
                        + smoke_pipeline_emit_flags_selftest_progress[
                            "age5_full_real_pipeline_emit_flags_selftest_current_case"
                        ],
                        "ci_pipeline_emit_flags_selftest_last_completed_case="
                        + smoke_pipeline_emit_flags_selftest_progress[
                            "age5_full_real_pipeline_emit_flags_selftest_last_completed_case"
                        ],
                        "ci_pipeline_emit_flags_selftest_total_elapsed_ms="
                        + smoke_pipeline_emit_flags_selftest_progress[
                            "age5_full_real_pipeline_emit_flags_selftest_total_elapsed_ms"
                        ],
                    ]
                )
            if (
                smoke_pipeline_emit_flags_selftest_probe[
                    "age5_full_real_pipeline_emit_flags_selftest_probe_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "ci_pipeline_emit_flags_selftest_current_probe="
                        + smoke_pipeline_emit_flags_selftest_probe[
                            "age5_full_real_pipeline_emit_flags_selftest_current_probe"
                        ],
                        "ci_pipeline_emit_flags_selftest_last_completed_probe="
                        + smoke_pipeline_emit_flags_selftest_probe[
                            "age5_full_real_pipeline_emit_flags_selftest_last_completed_probe"
                        ],
                    ]
                )
            if (
                smoke_age5_combined_policy_selftest_progress[
                    "age5_full_real_age5_combined_policy_selftest_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "ci_age5_combined_heavy_policy_selftest_current_case="
                        + smoke_age5_combined_policy_selftest_progress[
                            "age5_full_real_age5_combined_policy_selftest_current_case"
                        ],
                        "ci_age5_combined_heavy_policy_selftest_last_completed_case="
                        + smoke_age5_combined_policy_selftest_progress[
                            "age5_full_real_age5_combined_policy_selftest_last_completed_case"
                        ],
                        "ci_age5_combined_heavy_policy_selftest_current_format="
                        + smoke_age5_combined_policy_selftest_progress[
                            "age5_full_real_age5_combined_policy_selftest_current_format"
                        ],
                        "ci_age5_combined_heavy_policy_selftest_last_completed_format="
                        + smoke_age5_combined_policy_selftest_progress[
                            "age5_full_real_age5_combined_policy_selftest_last_completed_format"
                        ],
                        "ci_age5_combined_heavy_policy_selftest_current_probe="
                        + smoke_age5_combined_policy_selftest_progress[
                            "age5_full_real_age5_combined_policy_selftest_current_probe"
                        ],
                        "ci_age5_combined_heavy_policy_selftest_last_completed_probe="
                        + smoke_age5_combined_policy_selftest_progress[
                            "age5_full_real_age5_combined_policy_selftest_last_completed_probe"
                        ],
                        "ci_age5_combined_heavy_policy_selftest_total_elapsed_ms="
                        + smoke_age5_combined_policy_selftest_progress[
                            "age5_full_real_age5_combined_policy_selftest_total_elapsed_ms"
                        ],
                    ]
                )
            if (
                smoke_profile_matrix_full_real_smoke_policy_selftest_progress[
                    "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "ci_profile_matrix_full_real_smoke_policy_selftest_current_case="
                        + smoke_profile_matrix_full_real_smoke_policy_selftest_progress[
                            "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_current_case"
                        ],
                        "ci_profile_matrix_full_real_smoke_policy_selftest_last_completed_case="
                        + smoke_profile_matrix_full_real_smoke_policy_selftest_progress[
                            "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_last_completed_case"
                        ],
                        "ci_profile_matrix_full_real_smoke_policy_selftest_current_format="
                        + smoke_profile_matrix_full_real_smoke_policy_selftest_progress[
                            "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_current_format"
                        ],
                        "ci_profile_matrix_full_real_smoke_policy_selftest_last_completed_format="
                        + smoke_profile_matrix_full_real_smoke_policy_selftest_progress[
                            "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_last_completed_format"
                        ],
                        "ci_profile_matrix_full_real_smoke_policy_selftest_total_elapsed_ms="
                        + smoke_profile_matrix_full_real_smoke_policy_selftest_progress[
                            "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_total_elapsed_ms"
                        ],
                    ]
                )
            if (
                smoke_profile_matrix_full_real_smoke_check_selftest_progress[
                    "age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "ci_profile_matrix_full_real_smoke_check_selftest_current_case="
                        + smoke_profile_matrix_full_real_smoke_check_selftest_progress[
                            "age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_case"
                        ],
                        "ci_profile_matrix_full_real_smoke_check_selftest_last_completed_case="
                        + smoke_profile_matrix_full_real_smoke_check_selftest_progress[
                            "age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_case"
                        ],
                        "ci_profile_matrix_full_real_smoke_check_selftest_current_probe="
                        + smoke_profile_matrix_full_real_smoke_check_selftest_progress[
                            "age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_probe"
                        ],
                        "ci_profile_matrix_full_real_smoke_check_selftest_last_completed_probe="
                        + smoke_profile_matrix_full_real_smoke_check_selftest_progress[
                            "age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_probe"
                        ],
                        "ci_profile_matrix_full_real_smoke_check_selftest_total_elapsed_ms="
                        + smoke_profile_matrix_full_real_smoke_check_selftest_progress[
                            "age5_full_real_profile_matrix_full_real_smoke_check_selftest_total_elapsed_ms"
                        ],
                    ]
                )
            if (
                smoke_fixed64_darwin_real_report_readiness_check_selftest_progress[
                    "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "ci_fixed64_darwin_real_report_readiness_check_selftest_current_case="
                        + smoke_fixed64_darwin_real_report_readiness_check_selftest_progress[
                            "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_case"
                        ],
                        "ci_fixed64_darwin_real_report_readiness_check_selftest_last_completed_case="
                        + smoke_fixed64_darwin_real_report_readiness_check_selftest_progress[
                            "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_case"
                        ],
                        "ci_fixed64_darwin_real_report_readiness_check_selftest_current_probe="
                        + smoke_fixed64_darwin_real_report_readiness_check_selftest_progress[
                            "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_probe"
                        ],
                        "ci_fixed64_darwin_real_report_readiness_check_selftest_last_completed_probe="
                        + smoke_fixed64_darwin_real_report_readiness_check_selftest_progress[
                            "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_probe"
                        ],
                        "ci_fixed64_darwin_real_report_readiness_check_selftest_total_elapsed_ms="
                        + smoke_fixed64_darwin_real_report_readiness_check_selftest_progress[
                            "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_total_elapsed_ms"
                        ],
                    ]
                )
            if (
                smoke_map_access_contract_check_progress[
                    "age5_full_real_map_access_contract_check_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "ci_map_access_contract_check_current_case="
                        + smoke_map_access_contract_check_progress[
                            "age5_full_real_map_access_contract_check_current_case"
                        ],
                        "ci_map_access_contract_check_last_completed_case="
                        + smoke_map_access_contract_check_progress[
                            "age5_full_real_map_access_contract_check_last_completed_case"
                        ],
                        "ci_map_access_contract_check_current_probe="
                        + smoke_map_access_contract_check_progress[
                            "age5_full_real_map_access_contract_check_current_probe"
                        ],
                        "ci_map_access_contract_check_last_completed_probe="
                        + smoke_map_access_contract_check_progress[
                            "age5_full_real_map_access_contract_check_last_completed_probe"
                        ],
                        "ci_map_access_contract_check_total_elapsed_ms="
                        + smoke_map_access_contract_check_progress[
                            "age5_full_real_map_access_contract_check_total_elapsed_ms"
                        ],
                    ]
                )
            if (
                smoke_tensor_v0_cli_check_progress[
                    "age5_full_real_tensor_v0_cli_check_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "ci_tensor_v0_cli_check_current_case="
                        + smoke_tensor_v0_cli_check_progress[
                            "age5_full_real_tensor_v0_cli_check_current_case"
                        ],
                        "ci_tensor_v0_cli_check_last_completed_case="
                        + smoke_tensor_v0_cli_check_progress[
                            "age5_full_real_tensor_v0_cli_check_last_completed_case"
                        ],
                        "ci_tensor_v0_cli_check_current_probe="
                        + smoke_tensor_v0_cli_check_progress[
                            "age5_full_real_tensor_v0_cli_check_current_probe"
                        ],
                        "ci_tensor_v0_cli_check_last_completed_probe="
                        + smoke_tensor_v0_cli_check_progress[
                            "age5_full_real_tensor_v0_cli_check_last_completed_probe"
                        ],
                        "ci_tensor_v0_cli_check_total_elapsed_ms="
                        + smoke_tensor_v0_cli_check_progress[
                            "age5_full_real_tensor_v0_cli_check_total_elapsed_ms"
                        ],
                    ]
                )
            if (
                smoke_ci_pack_golden_exec_policy_selftest_progress[
                    "age5_full_real_ci_pack_golden_exec_policy_selftest_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "ci_pack_golden_exec_policy_selftest_current_case="
                        + smoke_ci_pack_golden_exec_policy_selftest_progress[
                            "age5_full_real_ci_pack_golden_exec_policy_selftest_current_case"
                        ],
                        "ci_pack_golden_exec_policy_selftest_last_completed_case="
                        + smoke_ci_pack_golden_exec_policy_selftest_progress[
                            "age5_full_real_ci_pack_golden_exec_policy_selftest_last_completed_case"
                        ],
                        "ci_pack_golden_exec_policy_selftest_current_probe="
                        + smoke_ci_pack_golden_exec_policy_selftest_progress[
                            "age5_full_real_ci_pack_golden_exec_policy_selftest_current_probe"
                        ],
                        "ci_pack_golden_exec_policy_selftest_last_completed_probe="
                        + smoke_ci_pack_golden_exec_policy_selftest_progress[
                            "age5_full_real_ci_pack_golden_exec_policy_selftest_last_completed_probe"
                        ],
                        "ci_pack_golden_exec_policy_selftest_total_elapsed_ms="
                        + smoke_ci_pack_golden_exec_policy_selftest_progress[
                            "age5_full_real_ci_pack_golden_exec_policy_selftest_total_elapsed_ms"
                        ],
                    ]
                )
            if (
                smoke_ci_pack_golden_age5_surface_selftest_progress[
                    "age5_full_real_ci_pack_golden_age5_surface_selftest_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "ci_pack_golden_age5_surface_selftest_current_case="
                        + smoke_ci_pack_golden_age5_surface_selftest_progress[
                            "age5_full_real_ci_pack_golden_age5_surface_selftest_current_case"
                        ],
                        "ci_pack_golden_age5_surface_selftest_last_completed_case="
                        + smoke_ci_pack_golden_age5_surface_selftest_progress[
                            "age5_full_real_ci_pack_golden_age5_surface_selftest_last_completed_case"
                        ],
                        "ci_pack_golden_age5_surface_selftest_current_probe="
                        + smoke_ci_pack_golden_age5_surface_selftest_progress[
                            "age5_full_real_ci_pack_golden_age5_surface_selftest_current_probe"
                        ],
                        "ci_pack_golden_age5_surface_selftest_last_completed_probe="
                        + smoke_ci_pack_golden_age5_surface_selftest_progress[
                            "age5_full_real_ci_pack_golden_age5_surface_selftest_last_completed_probe"
                        ],
                        "ci_pack_golden_age5_surface_selftest_total_elapsed_ms="
                        + smoke_ci_pack_golden_age5_surface_selftest_progress[
                            "age5_full_real_ci_pack_golden_age5_surface_selftest_total_elapsed_ms"
                        ],
                    ]
                )
            if (
                smoke_ci_pack_golden_guideblock_selftest_progress[
                    "age5_full_real_ci_pack_golden_guideblock_selftest_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "ci_pack_golden_guideblock_selftest_current_case="
                        + smoke_ci_pack_golden_guideblock_selftest_progress[
                            "age5_full_real_ci_pack_golden_guideblock_selftest_current_case"
                        ],
                        "ci_pack_golden_guideblock_selftest_last_completed_case="
                        + smoke_ci_pack_golden_guideblock_selftest_progress[
                            "age5_full_real_ci_pack_golden_guideblock_selftest_last_completed_case"
                        ],
                        "ci_pack_golden_guideblock_selftest_current_probe="
                        + smoke_ci_pack_golden_guideblock_selftest_progress[
                            "age5_full_real_ci_pack_golden_guideblock_selftest_current_probe"
                        ],
                        "ci_pack_golden_guideblock_selftest_last_completed_probe="
                        + smoke_ci_pack_golden_guideblock_selftest_progress[
                            "age5_full_real_ci_pack_golden_guideblock_selftest_last_completed_probe"
                        ],
                        "ci_pack_golden_guideblock_selftest_total_elapsed_ms="
                        + smoke_ci_pack_golden_guideblock_selftest_progress[
                            "age5_full_real_ci_pack_golden_guideblock_selftest_total_elapsed_ms"
                        ],
                    ]
                )
            if (
                smoke_ci_pack_golden_jjaim_flatten_selftest_progress[
                    "age5_full_real_ci_pack_golden_jjaim_flatten_selftest_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "ci_pack_golden_jjaim_flatten_selftest_current_case="
                        + smoke_ci_pack_golden_jjaim_flatten_selftest_progress[
                            "age5_full_real_ci_pack_golden_jjaim_flatten_selftest_current_case"
                        ],
                        "ci_pack_golden_jjaim_flatten_selftest_last_completed_case="
                        + smoke_ci_pack_golden_jjaim_flatten_selftest_progress[
                            "age5_full_real_ci_pack_golden_jjaim_flatten_selftest_last_completed_case"
                        ],
                        "ci_pack_golden_jjaim_flatten_selftest_current_probe="
                        + smoke_ci_pack_golden_jjaim_flatten_selftest_progress[
                            "age5_full_real_ci_pack_golden_jjaim_flatten_selftest_current_probe"
                        ],
                        "ci_pack_golden_jjaim_flatten_selftest_last_completed_probe="
                        + smoke_ci_pack_golden_jjaim_flatten_selftest_progress[
                            "age5_full_real_ci_pack_golden_jjaim_flatten_selftest_last_completed_probe"
                        ],
                        "ci_pack_golden_jjaim_flatten_selftest_total_elapsed_ms="
                        + smoke_ci_pack_golden_jjaim_flatten_selftest_progress[
                            "age5_full_real_ci_pack_golden_jjaim_flatten_selftest_total_elapsed_ms"
                        ],
                    ]
                )
            if (
                smoke_ci_pack_golden_event_model_selftest_progress[
                    "age5_full_real_ci_pack_golden_event_model_selftest_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "ci_pack_golden_event_model_selftest_current_case="
                        + smoke_ci_pack_golden_event_model_selftest_progress[
                            "age5_full_real_ci_pack_golden_event_model_selftest_current_case"
                        ],
                        "ci_pack_golden_event_model_selftest_last_completed_case="
                        + smoke_ci_pack_golden_event_model_selftest_progress[
                            "age5_full_real_ci_pack_golden_event_model_selftest_last_completed_case"
                        ],
                        "ci_pack_golden_event_model_selftest_current_probe="
                        + smoke_ci_pack_golden_event_model_selftest_progress[
                            "age5_full_real_ci_pack_golden_event_model_selftest_current_probe"
                        ],
                        "ci_pack_golden_event_model_selftest_last_completed_probe="
                        + smoke_ci_pack_golden_event_model_selftest_progress[
                            "age5_full_real_ci_pack_golden_event_model_selftest_last_completed_probe"
                        ],
                        "ci_pack_golden_event_model_selftest_total_elapsed_ms="
                        + smoke_ci_pack_golden_event_model_selftest_progress[
                            "age5_full_real_ci_pack_golden_event_model_selftest_total_elapsed_ms"
                        ],
                    ]
                )
            if (
                smoke_ci_pack_golden_lang_consistency_selftest_progress[
                    "age5_full_real_ci_pack_golden_lang_consistency_selftest_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "ci_pack_golden_lang_consistency_selftest_current_case="
                        + smoke_ci_pack_golden_lang_consistency_selftest_progress[
                            "age5_full_real_ci_pack_golden_lang_consistency_selftest_current_case"
                        ],
                        "ci_pack_golden_lang_consistency_selftest_last_completed_case="
                        + smoke_ci_pack_golden_lang_consistency_selftest_progress[
                            "age5_full_real_ci_pack_golden_lang_consistency_selftest_last_completed_case"
                        ],
                        "ci_pack_golden_lang_consistency_selftest_current_probe="
                        + smoke_ci_pack_golden_lang_consistency_selftest_progress[
                            "age5_full_real_ci_pack_golden_lang_consistency_selftest_current_probe"
                        ],
                        "ci_pack_golden_lang_consistency_selftest_last_completed_probe="
                        + smoke_ci_pack_golden_lang_consistency_selftest_progress[
                            "age5_full_real_ci_pack_golden_lang_consistency_selftest_last_completed_probe"
                        ],
                        "ci_pack_golden_lang_consistency_selftest_total_elapsed_ms="
                        + smoke_ci_pack_golden_lang_consistency_selftest_progress[
                            "age5_full_real_ci_pack_golden_lang_consistency_selftest_total_elapsed_ms"
                        ],
                    ]
                )
            if (
                smoke_w107_golden_index_selftest_progress[
                    "age5_full_real_w107_golden_index_selftest_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "w107_golden_index_selftest_active_cases="
                        + smoke_w107_golden_index_selftest_progress[
                            "age5_full_real_w107_golden_index_selftest_active_cases"
                        ],
                        "w107_golden_index_selftest_inactive_cases="
                        + smoke_w107_golden_index_selftest_progress[
                            "age5_full_real_w107_golden_index_selftest_inactive_cases"
                        ],
                        "w107_golden_index_selftest_index_codes="
                        + smoke_w107_golden_index_selftest_progress[
                            "age5_full_real_w107_golden_index_selftest_index_codes"
                        ],
                        "w107_golden_index_selftest_current_probe="
                        + smoke_w107_golden_index_selftest_progress[
                            "age5_full_real_w107_golden_index_selftest_current_probe"
                        ],
                        "w107_golden_index_selftest_last_completed_probe="
                        + smoke_w107_golden_index_selftest_progress[
                            "age5_full_real_w107_golden_index_selftest_last_completed_probe"
                        ],
                    ]
                )
            if (
                smoke_w107_progress_contract_selftest_progress[
                    "age5_full_real_w107_progress_contract_selftest_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "w107_progress_contract_selftest_completed_checks="
                        + smoke_w107_progress_contract_selftest_progress[
                            "age5_full_real_w107_progress_contract_selftest_completed_checks"
                        ],
                        "w107_progress_contract_selftest_total_checks="
                        + smoke_w107_progress_contract_selftest_progress[
                            "age5_full_real_w107_progress_contract_selftest_total_checks"
                        ],
                        "w107_progress_contract_selftest_checks_text="
                        + smoke_w107_progress_contract_selftest_progress[
                            "age5_full_real_w107_progress_contract_selftest_checks_text"
                        ],
                        "w107_progress_contract_selftest_current_probe="
                        + smoke_w107_progress_contract_selftest_progress[
                            "age5_full_real_w107_progress_contract_selftest_current_probe"
                        ],
                        "w107_progress_contract_selftest_last_completed_probe="
                        + smoke_w107_progress_contract_selftest_progress[
                            "age5_full_real_w107_progress_contract_selftest_last_completed_probe"
                        ],
                    ]
                )
            if (
                smoke_age1_immediate_proof_operation_contract_selftest_progress[
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "age1_immediate_proof_operation_contract_selftest_completed_checks="
                        + smoke_age1_immediate_proof_operation_contract_selftest_progress[
                            "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks"
                        ],
                        "age1_immediate_proof_operation_contract_selftest_total_checks="
                        + smoke_age1_immediate_proof_operation_contract_selftest_progress[
                            "age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks"
                        ],
                        "age1_immediate_proof_operation_contract_selftest_checks_text="
                        + smoke_age1_immediate_proof_operation_contract_selftest_progress[
                            "age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text"
                        ],
                        "age1_immediate_proof_operation_contract_selftest_current_probe="
                        + smoke_age1_immediate_proof_operation_contract_selftest_progress[
                            "age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe"
                        ],
                        "age1_immediate_proof_operation_contract_selftest_last_completed_probe="
                        + smoke_age1_immediate_proof_operation_contract_selftest_progress[
                            "age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe"
                        ],
                    ]
                )
            if (
                smoke_proof_certificate_v1_consumer_transport_contract_selftest_progress[
                    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "proof_certificate_v1_consumer_transport_contract_selftest_completed_checks="
                        + smoke_proof_certificate_v1_consumer_transport_contract_selftest_progress[
                            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks"
                        ],
                        "proof_certificate_v1_consumer_transport_contract_selftest_total_checks="
                        + smoke_proof_certificate_v1_consumer_transport_contract_selftest_progress[
                            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks"
                        ],
                        "proof_certificate_v1_consumer_transport_contract_selftest_checks_text="
                        + smoke_proof_certificate_v1_consumer_transport_contract_selftest_progress[
                            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text"
                        ],
                        "proof_certificate_v1_consumer_transport_contract_selftest_current_probe="
                        + smoke_proof_certificate_v1_consumer_transport_contract_selftest_progress[
                            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe"
                        ],
                        "proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe="
                        + smoke_proof_certificate_v1_consumer_transport_contract_selftest_progress[
                            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe"
                        ],
                    ]
                )
            if (
                smoke_proof_certificate_v1_verify_report_digest_contract_selftest_progress[
                    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present"
                ]
                == "1"
            ):
                smoke_summary_tokens.extend(
                    [
                        "proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks="
                        + smoke_proof_certificate_v1_verify_report_digest_contract_selftest_progress[
                            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks"
                        ],
                        "proof_certificate_v1_verify_report_digest_contract_selftest_total_checks="
                        + smoke_proof_certificate_v1_verify_report_digest_contract_selftest_progress[
                            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks"
                        ],
                        "proof_certificate_v1_verify_report_digest_contract_selftest_checks_text="
                        + smoke_proof_certificate_v1_verify_report_digest_contract_selftest_progress[
                            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text"
                        ],
                        "proof_certificate_v1_verify_report_digest_contract_selftest_current_probe="
                        + smoke_proof_certificate_v1_verify_report_digest_contract_selftest_progress[
                            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe"
                        ],
                        "proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe="
                        + smoke_proof_certificate_v1_verify_report_digest_contract_selftest_progress[
                            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe"
                        ],
                    ]
                )
            if smoke_w94_social_pack_check_progress["age5_full_real_w94_social_pack_check_progress_present"] == "1":
                smoke_summary_tokens.extend(
                    [
                        "w94_social_pack_check_current_case="
                        + smoke_w94_social_pack_check_progress["age5_full_real_w94_social_pack_check_current_case"],
                        "w94_social_pack_check_last_completed_case="
                        + smoke_w94_social_pack_check_progress[
                            "age5_full_real_w94_social_pack_check_last_completed_case"
                        ],
                        "w94_social_pack_check_current_probe="
                        + smoke_w94_social_pack_check_progress["age5_full_real_w94_social_pack_check_current_probe"],
                        "w94_social_pack_check_last_completed_probe="
                        + smoke_w94_social_pack_check_progress[
                            "age5_full_real_w94_social_pack_check_last_completed_probe"
                        ],
                        "w94_social_pack_check_total_elapsed_ms="
                        + smoke_w94_social_pack_check_progress["age5_full_real_w94_social_pack_check_total_elapsed_ms"],
                    ]
                )
            if smoke_w95_cert_pack_check_progress["age5_full_real_w95_cert_pack_check_progress_present"] == "1":
                smoke_summary_tokens.extend(
                    [
                        "w95_cert_pack_check_current_case="
                        + smoke_w95_cert_pack_check_progress["age5_full_real_w95_cert_pack_check_current_case"],
                        "w95_cert_pack_check_last_completed_case="
                        + smoke_w95_cert_pack_check_progress[
                            "age5_full_real_w95_cert_pack_check_last_completed_case"
                        ],
                        "w95_cert_pack_check_current_probe="
                        + smoke_w95_cert_pack_check_progress["age5_full_real_w95_cert_pack_check_current_probe"],
                        "w95_cert_pack_check_last_completed_probe="
                        + smoke_w95_cert_pack_check_progress[
                            "age5_full_real_w95_cert_pack_check_last_completed_probe"
                        ],
                        "w95_cert_pack_check_total_elapsed_ms="
                        + smoke_w95_cert_pack_check_progress["age5_full_real_w95_cert_pack_check_total_elapsed_ms"],
                    ]
                )
            if smoke_w96_somssi_pack_check_progress["age5_full_real_w96_somssi_pack_check_progress_present"] == "1":
                smoke_summary_tokens.extend(
                    [
                        "w96_somssi_pack_check_current_case="
                        + smoke_w96_somssi_pack_check_progress["age5_full_real_w96_somssi_pack_check_current_case"],
                        "w96_somssi_pack_check_last_completed_case="
                        + smoke_w96_somssi_pack_check_progress[
                            "age5_full_real_w96_somssi_pack_check_last_completed_case"
                        ],
                        "w96_somssi_pack_check_current_probe="
                        + smoke_w96_somssi_pack_check_progress["age5_full_real_w96_somssi_pack_check_current_probe"],
                        "w96_somssi_pack_check_last_completed_probe="
                        + smoke_w96_somssi_pack_check_progress[
                            "age5_full_real_w96_somssi_pack_check_last_completed_probe"
                        ],
                        "w96_somssi_pack_check_total_elapsed_ms="
                        + smoke_w96_somssi_pack_check_progress["age5_full_real_w96_somssi_pack_check_total_elapsed_ms"],
                    ]
                )
            if smoke_w97_self_heal_pack_check_progress["age5_full_real_w97_self_heal_pack_check_progress_present"] == "1":
                smoke_summary_tokens.extend(
                    [
                        "w97_self_heal_pack_check_current_case="
                        + smoke_w97_self_heal_pack_check_progress["age5_full_real_w97_self_heal_pack_check_current_case"],
                        "w97_self_heal_pack_check_last_completed_case="
                        + smoke_w97_self_heal_pack_check_progress[
                            "age5_full_real_w97_self_heal_pack_check_last_completed_case"
                        ],
                        "w97_self_heal_pack_check_current_probe="
                        + smoke_w97_self_heal_pack_check_progress[
                            "age5_full_real_w97_self_heal_pack_check_current_probe"
                        ],
                        "w97_self_heal_pack_check_last_completed_probe="
                        + smoke_w97_self_heal_pack_check_progress[
                            "age5_full_real_w97_self_heal_pack_check_last_completed_probe"
                        ],
                        "w97_self_heal_pack_check_total_elapsed_ms="
                        + smoke_w97_self_heal_pack_check_progress[
                            "age5_full_real_w97_self_heal_pack_check_total_elapsed_ms"
                        ],
                    ]
                )
            if smoke_profile_status_map["age5_full_real_profile_status_map_present"] == "1":
                smoke_summary_tokens.append(
                    "ci_profile_matrix_full_real_profile_status_map="
                    + smoke_profile_status_map["age5_full_real_profile_status_map"]
                )
            if smoke_timeout_breakdown["age5_full_real_timeout_present"] == "1":
                smoke_summary_tokens.extend(
                    [
                        f"step={smoke_timeout_breakdown['age5_full_real_timeout_step']}",
                        f"profiles={smoke_timeout_breakdown['age5_full_real_timeout_profiles']}",
                    ]
                )
            smoke_tail = clip(" | ".join(line.strip() for line in smoke_stdout.strip().splitlines()[-6:]), 500)
            smoke_summary_text = " ".join(smoke_summary_tokens).strip() or "-"
            criteria.append(
                {
                    "name": "age5_ci_profile_matrix_full_real_smoke_optin_pass",
                    "ok": smoke_ok,
                    "detail": "rc={} marker_ok={} cmd={} summary={} stdout_tail={}".format(
                        smoke_proc.returncode,
                        int(PROFILE_MATRIX_FULL_REAL_SMOKE_STATUS_PASS in smoke_stdout),
                        " ".join(smoke_cmd),
                        smoke_summary_text,
                        smoke_tail,
                    ),
                }
            )
            if not smoke_ok:
                failure_digest.append(
                    "age5_ci_profile_matrix_full_real_smoke_optin_pass: rc={} summary={} stdout_tail={}".format(
                        smoke_proc.returncode,
                        smoke_summary_text,
                        smoke_tail,
                    )
                )
                pending_items.append("AGE5 상위 orchestration에서 profile-matrix full-real opt-in heavy smoke PASS 유지")

    if with_runtime_helper_mismatch_negative_check:
        negative_script_ok = (root / CI_PROFILE_CORE_LANG_RUNTIME_HELPER_CONTRACT_SELFTEST_SCRIPT).exists()
        criteria.append(
            {
                "name": "age5_ci_profile_core_lang_runtime_helper_negative_script_present",
                "ok": negative_script_ok,
                "detail": f"path={CI_PROFILE_CORE_LANG_RUNTIME_HELPER_CONTRACT_SELFTEST_SCRIPT}",
            }
        )
        if not negative_script_ok:
            failure_digest.append(
                "age5_ci_profile_core_lang_runtime_helper_negative_script_present: missing={}".format(
                    CI_PROFILE_CORE_LANG_RUNTIME_HELPER_CONTRACT_SELFTEST_SCRIPT
                )
            )
            pending_items.append("AGE5 상위 orchestration에서 core_lang runtime-helper negative selftest 스크립트 유지")
        else:
            negative_cmd = [
                sys.executable,
                str(CI_PROFILE_CORE_LANG_RUNTIME_HELPER_CONTRACT_SELFTEST_SCRIPT),
            ]
            negative_env = dict(os.environ)
            negative_env[CI_PROFILE_RUNTIME_HELPER_MISMATCH_KEY_ENV] = (
                CI_PROFILE_CORE_LANG_RUNTIME_HELPER_MISMATCH_TARGET_KEY
            )
            negative_proc = run_text(negative_cmd, root, env=negative_env)
            negative_stdout = str(negative_proc.stdout or "")
            negative_ok = (
                negative_proc.returncode == 0
                and "[ci-profile-core-lang-runtime-helper-contract-selftest] ok" in negative_stdout
            )
            criteria.append(
                {
                    "name": "age5_ci_profile_core_lang_runtime_helper_negative_optin_pass",
                    "ok": negative_ok,
                    "detail": "rc={} marker_ok={} target_key={} cmd={} stdout_tail={}".format(
                        negative_proc.returncode,
                        int("[ci-profile-core-lang-runtime-helper-contract-selftest] ok" in negative_stdout),
                        CI_PROFILE_CORE_LANG_RUNTIME_HELPER_MISMATCH_TARGET_KEY,
                        " ".join(negative_cmd),
                        clip(" | ".join(line.strip() for line in negative_stdout.strip().splitlines()[-6:]), 500),
                    ),
                }
            )
            if not negative_ok:
                failure_digest.append(
                    "age5_ci_profile_core_lang_runtime_helper_negative_optin_pass: rc={} stdout_tail={}".format(
                        negative_proc.returncode,
                        clip(" | ".join(line.strip() for line in negative_stdout.strip().splitlines()[-6:]), 500),
                    )
                )
                pending_items.append("AGE5 상위 orchestration에서 core_lang runtime-helper mismatch negative opt-in PASS 유지")

    if with_group_id_summary_mismatch_negative_check:
        negative_script_ok = (root / CI_PROFILE_CORE_LANG_GROUP_ID_SUMMARY_CONTRACT_SELFTEST_SCRIPT).exists()
        criteria.append(
            {
                "name": "age5_ci_profile_core_lang_group_id_summary_negative_script_present",
                "ok": negative_script_ok,
                "detail": f"path={CI_PROFILE_CORE_LANG_GROUP_ID_SUMMARY_CONTRACT_SELFTEST_SCRIPT}",
            }
        )
        if not negative_script_ok:
            failure_digest.append(
                "age5_ci_profile_core_lang_group_id_summary_negative_script_present: missing={}".format(
                    CI_PROFILE_CORE_LANG_GROUP_ID_SUMMARY_CONTRACT_SELFTEST_SCRIPT
                )
            )
            pending_items.append("AGE5 상위 orchestration에서 core_lang group_id_summary negative selftest 스크립트 유지")
        else:
            negative_cmd = [
                sys.executable,
                str(CI_PROFILE_CORE_LANG_GROUP_ID_SUMMARY_CONTRACT_SELFTEST_SCRIPT),
            ]
            negative_proc = run_text(negative_cmd, root)
            negative_stdout = str(negative_proc.stdout or "")
            negative_ok = (
                negative_proc.returncode == 0
                and "[ci-profile-core-lang-group-id-summary-contract-selftest] ok" in negative_stdout
            )
            criteria.append(
                {
                    "name": "age5_ci_profile_core_lang_group_id_summary_negative_optin_pass",
                    "ok": negative_ok,
                    "detail": "rc={} marker_ok={} cmd={} stdout_tail={}".format(
                        negative_proc.returncode,
                        int("[ci-profile-core-lang-group-id-summary-contract-selftest] ok" in negative_stdout),
                        " ".join(negative_cmd),
                        clip(" | ".join(line.strip() for line in negative_stdout.strip().splitlines()[-6:]), 500),
                    ),
                }
            )
            if not negative_ok:
                failure_digest.append(
                    "age5_ci_profile_core_lang_group_id_summary_negative_optin_pass: rc={} stdout_tail={}".format(
                        negative_proc.returncode,
                        clip(" | ".join(line.strip() for line in negative_stdout.strip().splitlines()[-6:]), 500),
                    )
                )
                pending_items.append("AGE5 상위 orchestration에서 core_lang group_id_summary mismatch negative opt-in PASS 유지")

    return criteria, failure_digest[:20], pending_items, repair


def main() -> int:
    parser = argparse.ArgumentParser(description="Evaluate AGE5 close-lite criteria from docs + UI slot declarations")
    parser.add_argument(
        "--report-out",
        default=default_report_path("age5_close_report.detjson"),
        help="output age5 close report path",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="require wired mode for S6 contracts (retired not allowed)",
    )
    parser.add_argument(
        "--with-profile-matrix-full-real-smoke-check",
        action="store_true",
        help="run core_lang heavy gate with opt-in profile-matrix full-real smoke",
    )
    parser.add_argument(
        "--with-runtime-helper-mismatch-negative-check",
        action="store_true",
        help="run core_lang runtime-helper mismatch negative selftest through AGE5 orchestration",
    )
    parser.add_argument(
        "--with-group-id-summary-mismatch-negative-check",
        action="store_true",
        help="run core_lang group_id_summary mismatch negative selftest through AGE5 orchestration",
    )
    parser.add_argument(
        AGE5_COMBINED_HEAVY_FLAG,
        action="store_true",
        help="run both profile-matrix full-real heavy smoke and runtime-helper mismatch negative replay",
    )
    parser.add_argument(
        "--combined-heavy-child-timeout-sec",
        type=int,
        default=0,
        help="optional timeout(sec) applied to each combined-heavy child report run; 0 disables timeout",
    )
    parser.add_argument(
        "--full-real-smoke-step-timeout-sec",
        type=int,
        default=0,
        help="optional per-profile timeout(sec) forwarded to full-real smoke check; mainly for guarded preview runs",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    report_out = Path(args.report_out)
    combined_heavy_requested = bool(args.with_combined_heavy_runtime_helper_check or truthy_env(AGE5_COMBINED_HEAVY_ENV_KEY))
    if int(args.combined_heavy_child_timeout_sec or 0) > 0 and not combined_heavy_requested:
        print(
            "[age5-close] --combined-heavy-child-timeout-sec requires combined-heavy opt-in "
            f"({AGE5_COMBINED_HEAVY_FLAG} or env {AGE5_COMBINED_HEAVY_ENV_KEY}=1)",
            file=sys.stderr,
        )
        return 2
    age4_proof_snapshot, age4_proof_source_fields = load_age4_proof_snapshot_sources(report_out.parent)
    if combined_heavy_requested:
        child_env = dict(os.environ)
        child_env.pop(AGE5_COMBINED_HEAVY_ENV_KEY, None)
        child_timeout_sec = max(0, int(args.combined_heavy_child_timeout_sec or 0))
        full_real_report = report_out.with_name(report_out.stem + ".full_real.detjson")
        runtime_helper_negative_report = report_out.with_name(report_out.stem + ".runtime_helper_negative.detjson")
        group_id_summary_negative_report = report_out.with_name(report_out.stem + ".group_id_summary_negative.detjson")
        full_real_cmd = [
            sys.executable,
            str(Path(__file__).resolve()),
            "--report-out",
            str(full_real_report),
            "--with-profile-matrix-full-real-smoke-check",
        ]
        if child_timeout_sec > 0:
            full_real_cmd.extend(
                [
                    "--full-real-smoke-step-timeout-sec",
                    str(max(1, child_timeout_sec // 2)),
                ]
            )
        if args.strict:
            full_real_cmd.append("--strict")
        negative_cmd = [
            sys.executable,
            str(Path(__file__).resolve()),
            "--report-out",
            str(runtime_helper_negative_report),
            "--with-runtime-helper-mismatch-negative-check",
        ]
        if args.strict:
            negative_cmd.append("--strict")
        group_id_negative_cmd = [
            sys.executable,
            str(Path(__file__).resolve()),
            "--report-out",
            str(group_id_summary_negative_report),
            "--with-group-id-summary-mismatch-negative-check",
        ]
        if args.strict:
            group_id_negative_cmd.append("--strict")
        full_real_proc, full_real_reused = run_or_reuse_age5_close_child_report(
            cmd=full_real_cmd,
            cwd=root,
            env=child_env,
            report_path=full_real_report,
            required_criterion="age5_ci_profile_matrix_full_real_smoke_optin_pass",
            timeout_sec=child_timeout_sec,
        )
        negative_proc, negative_reused = run_or_reuse_age5_close_child_report(
            cmd=negative_cmd,
            cwd=root,
            env=child_env,
            report_path=runtime_helper_negative_report,
            required_criterion="age5_ci_profile_core_lang_runtime_helper_negative_optin_pass",
            timeout_sec=child_timeout_sec,
        )
        group_id_negative_proc, group_id_negative_reused = run_or_reuse_age5_close_child_report(
            cmd=group_id_negative_cmd,
            cwd=root,
            env=child_env,
            report_path=group_id_summary_negative_report,
            required_criterion="age5_ci_profile_core_lang_group_id_summary_negative_optin_pass",
            timeout_sec=child_timeout_sec,
        )
        report = build_age5_combined_heavy_optin_report(
            root=root,
            strict=bool(args.strict),
            combined_heavy_env_enabled=truthy_env(AGE5_COMBINED_HEAVY_ENV_KEY),
            full_real_cmd=full_real_cmd,
            full_real_proc=full_real_proc,
            full_real_report=full_real_report,
            runtime_helper_negative_cmd=negative_cmd,
            runtime_helper_negative_proc=negative_proc,
            runtime_helper_negative_report=runtime_helper_negative_report,
            group_id_summary_negative_cmd=group_id_negative_cmd,
            group_id_summary_negative_proc=group_id_negative_proc,
            group_id_summary_negative_report=group_id_summary_negative_report,
            combined_heavy_child_timeout_sec=child_timeout_sec,
            age4_proof_snapshot=age4_proof_snapshot,
            age4_proof_source_fields=age4_proof_source_fields,
        )
        report["reused_child_reports"] = {
            "full_real": bool(full_real_reused),
            "runtime_helper_negative": bool(negative_reused),
            "group_id_summary_negative": bool(group_id_negative_reused),
        }
        report["combined_heavy_child_timeout_sec"] = child_timeout_sec
        report[AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY] = resolve_age5_combined_heavy_timeout_mode(
            child_timeout_sec
        )
        overall_ok = bool(report.get("overall_ok", False))
        report_out.parent.mkdir(parents=True, exist_ok=True)
        report_out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        failed = sum(1 for row in report["criteria"] if not bool(row.get("ok", False)))
        print(
            f"[age5-close] strict={int(bool(args.strict))} combined=1 overall_ok={int(overall_ok)} "
            f"criteria={len(report['criteria'])} failed={failed} report={report_out}"
        )
        for row in report["criteria"]:
            print(f" - {row.get('name')}: ok={int(bool(row.get('ok', False)))}")
        return 0 if overall_ok else 1

    with_profile_matrix_full_real_smoke_check = bool(args.with_profile_matrix_full_real_smoke_check)
    with_runtime_helper_mismatch_negative_check = bool(args.with_runtime_helper_mismatch_negative_check)
    with_group_id_summary_mismatch_negative_check = bool(args.with_group_id_summary_mismatch_negative_check)
    criteria, failure_digest, pending_items, repair = build_criteria(
        root,
        strict=bool(args.strict),
        with_profile_matrix_full_real_smoke_check=with_profile_matrix_full_real_smoke_check,
        full_real_smoke_step_timeout_sec=max(0, int(args.full_real_smoke_step_timeout_sec or 0)),
        with_runtime_helper_mismatch_negative_check=with_runtime_helper_mismatch_negative_check,
        with_group_id_summary_mismatch_negative_check=with_group_id_summary_mismatch_negative_check,
    )

    report = build_age5_close_report(
        strict=bool(args.strict),
        with_profile_matrix_full_real_smoke_check=with_profile_matrix_full_real_smoke_check,
        with_runtime_helper_mismatch_negative_check=with_runtime_helper_mismatch_negative_check,
        with_group_id_summary_mismatch_negative_check=with_group_id_summary_mismatch_negative_check,
        with_combined_heavy_runtime_helper_check=combined_heavy_requested,
        combined_heavy_env_enabled=truthy_env(AGE5_COMBINED_HEAVY_ENV_KEY),
        criteria=criteria,
        failure_digest=failure_digest,
        pending_items=pending_items,
        repair=repair,
        age4_proof_snapshot=age4_proof_snapshot,
        age4_proof_source_fields=age4_proof_source_fields,
    )
    overall_ok = bool(report.get("overall_ok", False))

    report_out.parent.mkdir(parents=True, exist_ok=True)
    report_out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    failed = sum(1 for row in criteria if not bool(row.get("ok", False)))
    print(
        f"[age5-close] strict={int(bool(args.strict))} overall_ok={int(overall_ok)} "
        f"criteria={len(criteria)} failed={failed} report={report_out}"
    )
    for row in criteria:
        print(f" - {row.get('name')}: ok={int(bool(row.get('ok', False)))}")
    if not overall_ok:
        for line in failure_digest[:8]:
            print(f"   {line}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
