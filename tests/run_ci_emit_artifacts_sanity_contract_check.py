#!/usr/bin/env python
from __future__ import annotations

import argparse
from pathlib import Path
import re


REQUIRED_EMIT_CHECK_TOKENS = [
    "SANITY_REQUIRED_PASS_STEPS",
    "SANITY_REQUIRED_PASS_STEPS_CORE_LANG",
    "SANITY_REQUIRED_PASS_STEPS_SEAMGRIM",
    "resolve_required_sanity_steps(",
    "VALID_SANITY_PROFILES",
    "sanity_profile",
    "expected_final_line_candidates(",
    '"ci_gate_result_line"',
    "allow-triage-exists-upgrade",
    '"ci_profile_split_contract_check"',
    '"ci_profile_matrix_gate_selftest"',
    '"ci_profile_matrix_gate_selftest_report"',
    '"ci_profile_matrix_gate_selftest_status"',
    '"ci_profile_matrix_gate_selftest_total_elapsed_ms"',
    '"ci_profile_matrix_gate_selftest_selected_real_profiles"',
    '"ci_profile_matrix_gate_selftest_core_lang_elapsed_ms"',
    '"ci_profile_matrix_gate_selftest_seamgrim_elapsed_ms"',
    "PROFILE_MATRIX_SELFTEST_SUMMARY_REQUIRED_KEYS",
    "PROFILE_MATRIX_BRIEF_REQUIRED_KEYS",
    "from _ci_profile_matrix_selftest_lib import (",
    "PROFILE_MATRIX_BRIEF_KEYS",
    "PROFILE_MATRIX_SELFTEST_PROFILES",
    "PROFILE_MATRIX_SELFTEST_SCHEMA",
    "build_profile_matrix_brief_payload_from_snapshot(",
    "build_profile_matrix_snapshot_from_doc(",
    "build_profile_matrix_triage_payload_from_snapshot(",
    "load_profile_matrix_selftest_snapshot(",
    '"profile_matrix_total_elapsed_ms"',
    '"profile_matrix_seamgrim_elapsed_ms"',
    '"triage profile_matrix_selftest missing"',
    '"contract_tier_unsupported_check"',
    '"contract_tier_age3_min_enforcement_check"',
    '"map_access_contract_check"',
    '"gaji_registry_strict_audit_check"',
    '"gaji_registry_defaults_check"',
    '"stdlib_catalog_check"',
    '"stdlib_catalog_check_selftest"',
    '"tensor_v0_pack_check"',
    '"tensor_v0_cli_check"',
    '"profile_matrix_full_real_smoke_policy_selftest"',
    '"profile_matrix_full_real_smoke_check_selftest"',
    '"age2_close_selftest"',
    '"age2_close"',
    '"age3_close_selftest"',
    '"age3_close"',
    '"age5_combined_heavy_policy_selftest"',
    '"fixed64_darwin_real_report_live_check"',
    '"fixed64_darwin_real_report_readiness_check_selftest"',
    '"ci_profile_matrix_lightweight_contract_selftest"',
    '"ci_profile_matrix_snapshot_helper_selftest"',
    '"seamgrim_ci_gate_lesson_warning_step_check"',
    '"seamgrim_ci_gate_stateful_preview_step_check"',
    '"seamgrim_ci_gate_wasm_web_smoke_step_check"',
    '"seamgrim_ci_gate_wasm_web_smoke_step_check_selftest"',
    '"seamgrim_overlay_session_diag_parity_check"',
    '"seamgrim_overlay_compare_diag_parity_check"',
    '"seamgrim_overlay_session_wired_consistency_check"',
    '"seamgrim_interface_boundary_contract_check"',
    "SEAMGRIM_FOCUS_SUMMARY_REQUIRED_KEYS",
    "VALID_SEAMGRIM_FOCUS_SUMMARY_STATUS",
    '"seamgrim_group_id_summary_status"',
    '"age5_close_pack_contract_selftest"',
    '"ci_pack_golden_age5_surface_selftest"',
    '"ci_pack_golden_guideblock_selftest"',
    '"ci_pack_golden_exec_policy_selftest"',
    '"ci_pack_golden_jjaim_flatten_selftest"',
    '"ci_pack_golden_event_model_selftest"',
    '"ci_pack_golden_lang_consistency_selftest"',
    '"ci_pack_golden_metadata_selftest"',
    '"ci_pack_golden_graph_export_selftest"',
    '"ci_canon_ast_dpack_selftest"',
    '"ci_sanity_pack_golden_lang_consistency_ok"',
    '"ci_sanity_pack_golden_metadata_ok"',
    '"ci_sanity_pack_golden_graph_export_ok"',
    '"ci_sanity_canon_ast_dpack_ok"',
    '"ci_sanity_contract_tier_unsupported_ok"',
    '"ci_sanity_contract_tier_age3_min_enforcement_ok"',
    '"ci_sanity_map_access_contract_ok"',
    '"ci_sanity_stdlib_catalog_ok"',
    '"ci_sanity_stdlib_catalog_selftest_ok"',
    '"ci_sanity_tensor_v0_pack_ok"',
    '"ci_sanity_tensor_v0_cli_ok"',
    '"ci_sanity_fixed64_darwin_real_report_contract_ok"',
    '"ci_sanity_fixed64_darwin_real_report_live_ok"',
    '"ci_sanity_fixed64_darwin_real_report_readiness_selftest_ok"',
    '"ci_sanity_fixed64_darwin_real_report_live_report_path"',
    '"ci_sanity_fixed64_darwin_real_report_live_report_exists"',
    '"ci_sanity_fixed64_darwin_real_report_live_status"',
    '"ci_sanity_fixed64_darwin_real_report_live_resolved_status"',
    '"ci_sanity_fixed64_darwin_real_report_live_resolved_source"',
    '"ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count"',
    '"ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip"',
    '"ci_sanity_pipeline_emit_flags_ok"',
    '"ci_sanity_pipeline_emit_flags_selftest_ok"',
    '"ci_sanity_age2_close_ok"',
    '"ci_sanity_age2_close_selftest_ok"',
    '"ci_sanity_age3_close_ok"',
    '"ci_sanity_age3_close_selftest_ok"',
    '"ci_sanity_age3_bogae_geoul_visibility_smoke_ok"',
    '"ci_sanity_age3_bogae_geoul_visibility_smoke_report_path"',
    '"ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists"',
    '"ci_sanity_age3_bogae_geoul_visibility_smoke_schema"',
    '"ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok"',
    '"ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok"',
    '"ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes"',
    '"ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes"',
    '"ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok"',
    '"ci_sanity_age5_combined_heavy_policy_selftest_ok"',
    '"ci_sanity_age5_combined_heavy_report_schema"',
    '"ci_sanity_age5_combined_heavy_required_reports"',
    '"ci_sanity_age5_combined_heavy_required_criteria"',
    '"ci_sanity_age5_combined_heavy_child_summary_default_fields"',
    '"ci_sanity_age5_combined_heavy_combined_contract_summary_fields"',
    '"ci_sanity_age5_combined_heavy_full_summary_contract_fields"',
    '"ci_sanity_registry_defaults_ok"',
    '"ci_sync_readiness_ci_sanity_pipeline_emit_flags_ok"',
    '"ci_sync_readiness_ci_sanity_pipeline_emit_flags_selftest_ok"',
    '"ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok"',
    '"ci_sync_readiness_ci_sanity_age2_close_ok"',
    '"ci_sync_readiness_ci_sanity_age2_close_selftest_ok"',
    '"ci_sync_readiness_ci_sanity_age3_close_ok"',
    '"ci_sync_readiness_ci_sanity_age3_close_selftest_ok"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_ok"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_path"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_schema"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes"',
    '"ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_report_path"',
    '"ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_report_exists"',
    '"ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_status"',
    '"ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_status"',
    '"ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_source"',
    '"ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count"',
    '"ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip"',
    '"ci_sync_readiness_ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok"',
    '"ci_sync_readiness_ci_sanity_age5_combined_heavy_policy_selftest_ok"',
    '"ci_sync_readiness_ci_sanity_age5_combined_heavy_report_schema"',
    '"ci_sync_readiness_ci_sanity_age5_combined_heavy_required_reports"',
    '"ci_sync_readiness_ci_sanity_age5_combined_heavy_required_criteria"',
    '"ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"',
    '"ci_sync_readiness_ci_sanity_age5_combined_heavy_combined_contract_summary_fields"',
    '"ci_sync_readiness_ci_sanity_age5_combined_heavy_full_summary_contract_fields"',
    "SANITY_RUNTIME_HELPER_SUMMARY_FIELDS",
    "SANITY_RUNTIME_HELPER_TEXT_FIELDS",
    "SYNC_RUNTIME_HELPER_SUMMARY_FIELDS",
    "SYNC_RUNTIME_HELPER_TEXT_FIELDS",
    "VALID_RUNTIME_HELPER_SUMMARY_VALUES",
    "ddn.bogae_geoul_visibility_smoke.v1",
    '"seamgrim_5min_checklist"',
    '"seamgrim_runtime_5min_rewrite_motion_projectile"',
    '"seamgrim_runtime_5min_moyang_view_boundary"',
    "RUNTIME5_SUMMARY_REQUIRED_KEYS",
    "VALID_RUNTIME5_ITEM_STATUS",
    "load_runtime5_checklist_snapshot(",
    '"w92_aot_pack_check"',
    '"w93_universe_pack_check"',
    '"w94_social_pack_check"',
    '"w95_cert_pack_check"',
    '"w96_somssi_pack_check"',
    '"w97_self_heal_pack_check"',
    '"seamgrim_wasm_cli_diag_parity_check"',
    '"ci_sync_readiness"',
    "ddn.ci.sync_readiness.v1",
    'code=CODES["SYNC_READINESS_PATH_MISSING"]',
    'code=CODES["SYNC_READINESS_SCHEMA_MISMATCH"]',
    'code=CODES["SYNC_READINESS_STATUS_UNSUPPORTED"]',
    'code=CODES["SYNC_READINESS_STATUS_MISMATCH"]',
    'code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"]',
    'code=CODES["SANITY_REQUIRED_STEP_MISSING"]',
    'code=CODES["SANITY_REQUIRED_STEP_FAILED"]',
]
REQUIRED_EMIT_CHECK_TOKENS.extend(
    [
        "from _ci_age3_completion_gate_contract import (",
        "AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS,",
        "AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS,",
        'AGE3_COMPLETION_GATE_CRITERIA_PROFILES = {"full", "core_lang", "seamgrim"}',
        "for key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS",
        "for sanity_key, sync_key in AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS",
        "ci_sync_readiness missing criteria sync key",
    ]
)

REQUIRED_EMIT_SELFTEST_TOKENS = [
    "broken_sanity_compare_step_missing",
    "broken_sanity_compare_step_failed",
    "badsanitycomparemissing",
    "badsanitycomparefailed",
    "broken_sanity_wasm_web_selftest_step_missing",
    "broken_sanity_wasm_web_selftest_step_failed",
    "badsanitywasmwebselftestmissing",
    "badsanitywasmwebselftestfailed",
    "broken_sanity_wired_step_missing",
    "broken_sanity_wired_step_failed",
    "badsanitywiredmissing",
    "badsanitywiredfailed",
    "broken_sanity_required_step_missing",
    "broken_sanity_required_step_failed",
    "ci_profile_split_contract_check",
    "ci_profile_matrix_gate_selftest",
    "contract_tier_unsupported_check",
    "contract_tier_age3_min_enforcement_check",
    "map_access_contract_check",
    "gaji_registry_strict_audit_check",
    "gaji_registry_defaults_check",
    "stdlib_catalog_check",
    "stdlib_catalog_check_selftest",
    "tensor_v0_pack_check",
    "tensor_v0_cli_check",
    "profile_matrix_full_real_smoke_policy_selftest",
    "profile_matrix_full_real_smoke_check_selftest",
    "age2_close_selftest",
    "age2_close",
    "age3_close_selftest",
    "age3_close",
    "age5_combined_heavy_policy_selftest",
    "fixed64_darwin_real_report_live_check",
    "fixed64_darwin_real_report_readiness_check_selftest",
    "ci_profile_matrix_lightweight_contract_selftest",
    "ci_profile_matrix_snapshot_helper_selftest",
    "ci_pack_golden_age5_surface_selftest",
    "ci_pack_golden_guideblock_selftest",
    "ci_pack_golden_exec_policy_selftest",
    "ci_pack_golden_jjaim_flatten_selftest",
    "ci_pack_golden_event_model_selftest",
    "ci_pack_golden_lang_consistency_selftest",
    "ci_pack_golden_metadata_selftest",
    "ci_pack_golden_graph_export_selftest",
    "ci_canon_ast_dpack_selftest",
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
    "ci_sanity_pipeline_emit_flags_ok",
    "ci_sanity_pipeline_emit_flags_selftest_ok",
    "ci_sanity_age2_close_ok",
    "ci_sanity_age2_close_selftest_ok",
    "ci_sanity_age3_close_ok",
    "ci_sanity_age3_close_selftest_ok",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_ok",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_schema",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes",
    "ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok",
    "ci_sanity_age5_combined_heavy_policy_selftest_ok",
    "ci_sanity_age5_combined_heavy_report_schema",
    "ci_sanity_age5_combined_heavy_required_reports",
    "ci_sanity_age5_combined_heavy_required_criteria",
    "ci_sanity_age5_combined_heavy_child_summary_default_fields",
    "ci_sanity_age5_combined_heavy_combined_contract_summary_fields",
    "ci_sanity_age5_combined_heavy_full_summary_contract_fields",
    "ci_sanity_registry_strict_audit_ok",
    "ci_sanity_registry_defaults_ok",
    "ci_sync_readiness_ci_sanity_pipeline_emit_flags_ok",
    "ci_sync_readiness_ci_sanity_pipeline_emit_flags_selftest_ok",
    "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok",
    "ci_sync_readiness_ci_sanity_age2_close_ok",
    "ci_sync_readiness_ci_sanity_age2_close_selftest_ok",
    "ci_sync_readiness_ci_sanity_age3_close_ok",
    "ci_sync_readiness_ci_sanity_age3_close_selftest_ok",
    "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_ok",
    "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_path",
    "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists",
    "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_schema",
    "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok",
    "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok",
    "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes",
    "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes",
    "ci_sync_readiness_ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_policy_selftest_ok",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_report_schema",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_required_reports",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_required_criteria",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_combined_contract_summary_fields",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_full_summary_contract_fields",
    "age3_bogae_geoul_visibility_smoke_report_path",
    "ddn.bogae_geoul_visibility_smoke.v1",
    "ci_profile_matrix_gate_selftest_ok",
    "broken_profile_matrix_summary_missing",
    "broken_profile_matrix_summary_value",
    "broken_profile_matrix_report_mismatch",
    "broken_profile_matrix_brief_missing",
    "broken_profile_matrix_brief_value",
    "broken_profile_matrix_triage_mismatch",
    "aggregate_summary_sanity_ok",
    "aggregate_summary_sanity_checked_profiles",
    "aggregate_summary_sanity_failed_profiles",
    "aggregate_summary_sanity_skipped_profiles",
    'f"{profile_name}_aggregate_summary_status"',
    'f"{profile_name}_aggregate_summary_values"',
    "ci_profile_matrix_gate_selftest.detjson",
    "missprofilematrixkey",
    "badprofilematrixvalue",
    "profilematrixmismatch",
    "missprofilematrixbrief",
    "badprofilematrixbrief",
    "badprofilematrixtriage",
    "with_runtime5_checklist",
    "broken_runtime5_summary_missing",
    "broken_runtime5_summary_value",
    "broken_runtime5_report_mismatch",
    "seamgrim_5min_checklist_report.detjson",
    "okruntime5off",
    "missruntime5key",
    "badruntime5value",
    "runtime5mismatch",
    "w92_aot_pack_check",
    "w93_universe_pack_check",
    "w94_social_pack_check",
    "w95_cert_pack_check",
    "w96_somssi_pack_check",
    "w97_self_heal_pack_check",
    "seamgrim_ci_gate_lesson_warning_step_check",
    "seamgrim_ci_gate_stateful_preview_step_check",
    "seamgrim_ci_gate_wasm_web_smoke_step_check",
    "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
    "seamgrim_interface_boundary_contract_check",
    "seamgrim_wasm_cli_diag_parity_check",
    "seamgrim_group_id_summary_status=ok",
    "with_sync_readiness",
    "broken_sync_readiness_schema",
    "broken_sync_readiness_status_unsupported",
    "broken_sync_readiness_status_mismatch",
    "broken_sync_readiness_pass_fields",
    'sanity_profile: str = "full"',
    '"profile": sanity_profile',
    '"sanity_profile": sanity_profile',
    "okseamgrim",
    "okfinallinefallback",
    "ci_gate_result_line",
    "existupgrade",
    "--allow-triage-exists-upgrade",
    "exists mismatch",
    "misssync",
    "badsyncschema",
    "badsyncstatus",
    "badsyncmismatch",
    "badsyncpassfields",
]
REQUIRED_EMIT_SELFTEST_TOKENS.extend(
    [
        "from _ci_age3_completion_gate_contract import (",
        "AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS,",
        "AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS,",
        "for sanity_key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS:",
        "for _sanity_key, sync_key in AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS:",
    ]
)

REQUIRED_CODE_MAP_TOKENS = [
    '"SANITY_REQUIRED_STEP_MISSING": "E_SANITY_REQUIRED_STEP_MISSING"',
    '"SANITY_REQUIRED_STEP_FAILED": "E_SANITY_REQUIRED_STEP_FAILED"',
    '"SYNC_READINESS_PATH_MISSING": "E_SYNC_READINESS_PATH_MISSING"',
    '"SYNC_READINESS_SCHEMA_MISMATCH": "E_SYNC_READINESS_SCHEMA_MISMATCH"',
    '"SYNC_READINESS_STATUS_UNSUPPORTED": "E_SYNC_READINESS_STATUS_UNSUPPORTED"',
    '"SYNC_READINESS_STATUS_MISMATCH": "E_SYNC_READINESS_STATUS_MISMATCH"',
    '"SYNC_READINESS_PASS_STATUS_FIELDS": "E_SYNC_READINESS_PASS_STATUS_FIELDS"',
]

EMIT_CODE_KEY_PATTERN = re.compile(r"""CODES\[['"]([A-Z0-9_]+)['"]\]""")
SELFTEST_FAIL_CODE_KEY_PATTERN = re.compile(r"""fail code=\{CODES\[['"]([A-Z0-9_]+)['"]\]\}""")
EMIT_CODE_MAP_BLOCK_PATTERN = re.compile(
    r"""EMIT_ARTIFACTS_CODES\s*=\s*\{(?P<body>.*?)^\}""",
    re.DOTALL | re.MULTILINE,
)
EMIT_CODE_MAP_KEY_PATTERN = re.compile(r'''"([A-Z0-9_]+)"\s*:\s*"E_[A-Z0-9_]+"''')


def extract_emit_code_map_keys(code_map_text: str) -> set[str]:
    block = EMIT_CODE_MAP_BLOCK_PATTERN.search(code_map_text)
    if block is None:
        return set()
    body = block.group("body")
    return set(EMIT_CODE_MAP_KEY_PATTERN.findall(body))


def resolve_target_path(root: Path, raw: str) -> Path:
    path = Path(raw)
    if path.is_absolute():
        return path
    return root / path


def main() -> int:
    parser = argparse.ArgumentParser(description="Check emit-artifacts check/selftest/code-map contract alignment")
    parser.add_argument("--emit-check", default="tests/run_ci_emit_artifacts_check.py")
    parser.add_argument("--emit-selftest", default="tests/run_ci_emit_artifacts_check_selftest.py")
    parser.add_argument("--code-map", default="tests/ci_check_error_codes.py")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    emit_check = resolve_target_path(root, str(args.emit_check))
    emit_selftest = resolve_target_path(root, str(args.emit_selftest))
    code_map = resolve_target_path(root, str(args.code_map))

    for target in (emit_check, emit_selftest, code_map):
        if not target.exists():
            print(f"missing target: {target}")
            return 1

    emit_check_text = emit_check.read_text(encoding="utf-8")
    emit_selftest_text = emit_selftest.read_text(encoding="utf-8")
    code_map_text = code_map.read_text(encoding="utf-8")

    missing: list[str] = []
    missing.extend([f"emit_check:{token}" for token in REQUIRED_EMIT_CHECK_TOKENS if token not in emit_check_text])
    missing.extend([f"emit_selftest:{token}" for token in REQUIRED_EMIT_SELFTEST_TOKENS if token not in emit_selftest_text])
    missing.extend([f"code_map:{token}" for token in REQUIRED_CODE_MAP_TOKENS if token not in code_map_text])
    emit_check_code_keys = set(EMIT_CODE_KEY_PATTERN.findall(emit_check_text))
    emit_selftest_fail_code_keys = set(SELFTEST_FAIL_CODE_KEY_PATTERN.findall(emit_selftest_text))
    emit_code_map_keys = extract_emit_code_map_keys(code_map_text)
    if not emit_code_map_keys:
        missing.append("code_map:EMIT_ARTIFACTS_CODES block missing")
    missing.extend(
        [
            f"emit_selftest:fail_code_missing:{key}"
            for key in sorted(emit_check_code_keys - emit_selftest_fail_code_keys)
        ]
    )
    missing.extend(
        [
            f"emit_selftest:fail_code_unknown:{key}"
            for key in sorted(emit_selftest_fail_code_keys - emit_check_code_keys)
        ]
    )
    missing.extend(
        [
            f"code_map:emit_code_missing:{key}"
            for key in sorted(emit_check_code_keys - emit_code_map_keys)
        ]
    )
    missing.extend(
        [
            f"code_map:emit_code_unused:{key}"
            for key in sorted(emit_code_map_keys - emit_check_code_keys)
        ]
    )

    if missing:
        print("ci emit artifacts sanity contract check failed:")
        for token in missing[:16]:
            print(f" - missing token: {token}")
        return 1

    print(
        "ci emit artifacts sanity contract check ok "
        f"(emit_codes={len(emit_check_code_keys)} selftest_fail_codes={len(emit_selftest_fail_code_keys)})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
