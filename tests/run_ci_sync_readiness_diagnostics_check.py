#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from _ci_seamgrim_step_contract import (
    SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS,
    SEAMGRIM_PLATFORM_SANITY_SUMMARY_STEP_FIELDS,
    SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES,
    SEAMGRIM_FEATURED_SYNC_READINESS_VALIDATE_ONLY_CASES,
)


REQUIRED_SYNC_CHECK_TOKENS = [
    "from _ci_seamgrim_step_contract import (",
    "SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS,",
    "merge_step_names,",
    "SANITY_REQUIRED_PASS_STEPS",
    "SYNC_READINESS_OK",
    "SYNC_READINESS_STEP_FAIL",
    "SYNC_READINESS_SANITY_CONTRACT_FAIL",
    "SYNC_READINESS_VALIDATE_ONLY_PATH_MISSING",
    "--sanity-profile",
    "--ci-sanity-profile",
    "sanity_profile",
    "backup_hygiene_selftest",
    "pipeline_emit_flags_check",
    "pipeline_emit_flags_selftest",
    "ci_emit_artifacts_sanity_contract_selftest",
    "age5_combined_heavy_policy_selftest",
    "profile_matrix_full_real_smoke_policy_selftest",
    "profile_matrix_full_real_smoke_check_selftest",
    "age2_close_selftest",
    "age3_close_selftest",
    "fixed64_darwin_real_report_readiness_check_selftest",
    "ci_profile_split_contract_check",
    "ci_profile_matrix_lightweight_contract_selftest",
    "ci_profile_matrix_snapshot_helper_selftest",
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
    "seamgrim_v2_task_batch_check",
    "seamgrim_ci_gate_wasm_web_smoke_step_check",
    "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
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
    "w92_aot_pack_check",
    "w93_universe_pack_check",
    "w94_social_pack_check",
    "w95_cert_pack_check",
    "w96_somssi_pack_check",
    "w97_self_heal_pack_check",
    "seamgrim_wasm_cli_diag_parity_check",
    "SANITY_REQUIRED_PASS_STEPS = merge_step_names(",
    "SANITY_REQUIRED_PASS_STEPS_SEAMGRIM = merge_step_names(",
    "validate_sanity_contract",
    "build_default_sanity_summary_fields",
    "SANITY_SUMMARY_STEP_FIELDS",
    "VALID_SANITY_SUMMARY_VALUES",
    '"ci_sanity_pipeline_emit_flags_ok"',
    '"ci_sanity_pipeline_emit_flags_selftest_ok"',
    '"ci_sanity_emit_artifacts_sanity_contract_selftest_ok"',
    '"ci_sanity_pack_golden_graph_export_ok"',
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
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_ok"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_path"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_schema"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes"',
    '"ci_sync_readiness_ci_sanity_age2_close_ok"',
    '"ci_sync_readiness_ci_sanity_age2_close_selftest_ok"',
    '"ci_sync_readiness_ci_sanity_age3_close_ok"',
    '"ci_sync_readiness_ci_sanity_age3_close_selftest_ok"',
    '"ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok"',
    '"ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok"',
    '"ci_sanity_age5_combined_heavy_policy_selftest_ok"',
    '"ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok"',
    '"ci_sanity_age5_combined_heavy_report_schema"',
    '"ci_sanity_age5_combined_heavy_required_reports"',
    '"ci_sanity_age5_combined_heavy_required_criteria"',
    '"ci_sanity_age5_combined_heavy_child_summary_default_fields"',
    '"ci_sanity_age5_combined_heavy_combined_contract_summary_fields"',
    '"ci_sanity_age5_combined_heavy_full_summary_contract_fields"',
    '"name": "sanity_gate_contract"',
    "--validate-only-sanity-json",
    "code=",
    "step=",
    "msg=",
    "ci_sync_readiness_status=",
]
REQUIRED_SYNC_CHECK_TOKENS.extend(
    [
        "SEAMGRIM_BLOCKER_STEP_CONTRACT_STEPS",
        "*SEAMGRIM_BLOCKER_STEP_CONTRACT_STEPS,",
        "SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS",
        "for summary_key, step_name in SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS",
        "SEAMGRIM_PLATFORM_SANITY_SUMMARY_STEP_FIELDS",
        "for summary_key, step_name in SEAMGRIM_PLATFORM_SANITY_SUMMARY_STEP_FIELDS",
    ]
)
REQUIRED_SYNC_CHECK_TOKENS.extend(
    [
        "from _ci_age3_completion_gate_contract import (",
        "AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS,",
        "AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS,",
        'AGE3_COMPLETION_GATE_CRITERIA_ENABLED_PROFILES = {"full", "core_lang", "seamgrim"}',
        "age3_criteria_enabled = profile in AGE3_COMPLETION_GATE_CRITERIA_ENABLED_PROFILES",
        "for sanity_key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS:",
        "for sanity_key, sync_key in AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS:",
    ]
)

REQUIRED_SYNC_SELFTEST_TOKENS = [
    "steps_count_quick_should_be_1",
    "custom_steps_count_should_be_1",
    "sanity_profile_should_be_full",
    "custom_sanity_profile_should_be_full",
    "validate_only_sanity_profile_should_be_full",
    '"sanity_gate_contract",',
    "sanity_gate_contract_should_be_ok",
    "validate_only_ok_should_pass",
    "validate_only_bad_should_fail",
    "validate_only_missing_parity_should_fail",
    "validate_missing_parity_msg_should_mention_step",
    "validate_only_missing_v2_task_batch_should_fail",
    "validate_missing_v2_task_batch_msg_should_mention_step",
    "validate_only_failed_v2_task_batch_should_fail",
    "validate_failed_v2_task_batch_msg_should_mention_step",
    "validate_only_missing_wasm_web_smoke_selftest_should_fail",
    "validate_missing_wasm_web_smoke_selftest_msg_should_mention_step",
    "SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES",
    "for (",
    ") in SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES:",
    "SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES",
    ") in SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES:",
    "SEAMGRIM_FEATURED_SYNC_READINESS_VALIDATE_ONLY_CASES",
    ") in SEAMGRIM_FEATURED_SYNC_READINESS_VALIDATE_ONLY_CASES:",
    "SEAMGRIM_FEATURED_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES",
    ") in SEAMGRIM_FEATURED_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES:",
    "validate_only_missing_age2_close_selftest_should_fail",
    "validate_missing_age2_close_selftest_msg_should_mention_step",
    "validate_only_missing_age3_close_selftest_should_fail",
    "validate_missing_age3_close_selftest_msg_should_mention_step",
    "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
    "pipeline_flags_ok_should_be_1",
    "pipeline_flags_selftest_ok_should_be_1",
    "validate_only_profile_matrix_full_real_smoke_policy_selftest_ok_should_be_1",
    "validate_only_age5_combined_heavy_policy_selftest_ok_should_be_1",
    "age3_bogae_geoul_visibility_smoke_ok_should_be_1",
    "custom_age3_bogae_geoul_visibility_smoke_ok_should_be_1",
    "validate_only_age3_bogae_geoul_visibility_smoke_ok_should_be_1",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes",
    "validate_only_missing_summary_should_fail",
    "validate_bad_summary_msg_should_mention_summary_key",
    "seamgrim_v2_task_batch_check",
    "seamgrim_wasm_cli_diag_parity_check",
    "validate_only_missing_should_fail",
    "E_SYNC_READINESS_SANITY_CONTRACT_FAIL",
    "E_SYNC_READINESS_VALIDATE_ONLY_PATH_MISSING",
]
REQUIRED_SYNC_SELFTEST_TOKENS.extend(
    [
        "from _ci_age3_completion_gate_contract import AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS",
        'summary_fields.update({key: "1" for key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS})',
    ]
)
REQUIRED_SYNC_SELFTEST_TOKENS.extend(
    [
        "from _ci_seamgrim_step_contract import (",
        "SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES,",
        "SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES,",
        "SEAMGRIM_FEATURED_SYNC_READINESS_VALIDATE_ONLY_CASES,",
        "SEAMGRIM_FEATURED_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES,",
    ]
)

REQUIRED_REPORT_CHECK_TOKENS = [
    "ddn.ci.sync_readiness.v1",
    "--require-pass",
    "--sanity-profile",
    "from ci_check_error_codes import SYNC_READINESS_REPORT_CODES as CODES",
    'CODES["PASS_STATUS_FIELDS"]',
    'CODES["MISSING_CONTRACT_ROW"]',
    'CODES["VALIDATE_ONLY_SHAPE"]',
    'CODES["STATUS_OK_MISMATCH"]',
    "sanity_gate_contract",
    "sanity_profile",
    '"ci_sanity_pipeline_emit_flags_ok"',
    '"ci_sanity_pipeline_emit_flags_selftest_ok"',
    '"ci_sanity_emit_artifacts_sanity_contract_selftest_ok"',
    '"ci_sanity_pack_golden_graph_export_ok"',
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
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_ok"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_path"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_schema"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes"',
    '"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes"',
    '"ci_sync_readiness_ci_sanity_age2_close_ok"',
    '"ci_sync_readiness_ci_sanity_age2_close_selftest_ok"',
    '"ci_sync_readiness_ci_sanity_age3_close_ok"',
    '"ci_sync_readiness_ci_sanity_age3_close_selftest_ok"',
    '"ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok"',
    '"ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok"',
    '"ci_sanity_age5_combined_heavy_policy_selftest_ok"',
    '"ci_sanity_age5_combined_heavy_report_schema"',
    '"ci_sanity_age5_combined_heavy_required_reports"',
    '"ci_sanity_age5_combined_heavy_required_criteria"',
    '"ci_sanity_age5_combined_heavy_child_summary_default_fields"',
    '"ci_sanity_age5_combined_heavy_combined_contract_summary_fields"',
    '"ci_sanity_age5_combined_heavy_full_summary_contract_fields"',
    '"ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok"',
    '"ci_sync_readiness_ci_sanity_age5_combined_heavy_report_schema"',
    '"ci_sync_readiness_ci_sanity_age5_combined_heavy_required_reports"',
    '"ci_sync_readiness_ci_sanity_age5_combined_heavy_required_criteria"',
    '"ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"',
    '"ci_sync_readiness_ci_sanity_age5_combined_heavy_combined_contract_summary_fields"',
    '"ci_sync_readiness_ci_sanity_age5_combined_heavy_full_summary_contract_fields"',
    'CODES["SANITY_SUMMARY_KEY_MISSING"]',
    'CODES["SANITY_SUMMARY_VALUE_INVALID"]',
]
REQUIRED_REPORT_CHECK_TOKENS.extend(
    [
        "SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS",
        "for summary_key, _step_name in SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS",
        "SEAMGRIM_PLATFORM_SANITY_SUMMARY_STEP_FIELDS",
        "for summary_key, _step_name in SEAMGRIM_PLATFORM_SANITY_SUMMARY_STEP_FIELDS",
    ]
)
REQUIRED_REPORT_CHECK_TOKENS.extend(
    [
        "from _ci_age3_completion_gate_contract import (",
        "AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS,",
        "AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS,",
        'AGE3_COMPLETION_GATE_CRITERIA_ENABLED_PROFILES = {"full", "core_lang", "seamgrim"}',
        "for sanity_key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS:",
        "for sanity_key, sync_key in AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS:",
        "sanity age3 criteria key missing",
        "sanity age3 criteria value invalid",
        "pass sanity age3 criteria invalid",
        "sync age3 criteria mismatch",
        "def sync_mirror_key(sanity_key: str) -> str:",
        "sync sanity summary key missing",
        "sync sanity summary value invalid",
        "sync sanity summary mismatch",
    ]
)

REQUIRED_REPORT_SELFTEST_TOKENS = [
    "tests/run_ci_sync_readiness_report_check.py",
    "SYNC_READINESS_REPORT_CODES as CODES",
    "report_check_should_pass",
    "validate_only_report_check_should_pass",
    "report_profile_mismatch_should_fail",
    "bad_code_should_fail",
    "bad_contract_row_should_fail",
    "bad_validate_shape_should_fail",
    "bad_summary_should_fail",
    "bad_emit_artifacts_sanity_should_fail",
    "bad_emit_artifacts_sanity_fail_code_should_match",
    "bad_emit_artifacts_sync_mirror_should_fail",
    "bad_emit_artifacts_sync_mirror_fail_code_should_match",
    "seamgrim_bad_blocker_summary_should_fail:",
    "seamgrim_bad_blocker_summary_fail_code_should_match:",
    "seamgrim_bad_blocker_sync_mirror_should_fail:",
    "seamgrim_bad_blocker_sync_mirror_fail_code_should_match:",
    'CODES["SANITY_SUMMARY_VALUE_INVALID"]',
]
REQUIRED_REPORT_SELFTEST_TOKENS.extend(
    [
        "from _ci_age3_completion_gate_contract import AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS",
        'summary_fields.update({key: "1" for key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS})',
    ]
)


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    sync_check = root / "tests" / "run_ci_sync_readiness_check.py"
    sync_selftest = root / "tests" / "run_ci_sync_readiness_check_selftest.py"
    sync_report_check = root / "tests" / "run_ci_sync_readiness_report_check.py"
    sync_report_selftest = root / "tests" / "run_ci_sync_readiness_report_check_selftest.py"
    for target in (sync_check, sync_selftest, sync_report_check, sync_report_selftest):
        if not target.exists():
            print(f"missing target: {target}")
            return 1

    sync_text = sync_check.read_text(encoding="utf-8")
    selftest_text = sync_selftest.read_text(encoding="utf-8")
    report_check_text = sync_report_check.read_text(encoding="utf-8")
    report_selftest_text = sync_report_selftest.read_text(encoding="utf-8")

    missing: list[str] = []
    missing.extend([f"sync_check:{token}" for token in REQUIRED_SYNC_CHECK_TOKENS if token not in sync_text])
    missing.extend([f"sync_selftest:{token}" for token in REQUIRED_SYNC_SELFTEST_TOKENS if token not in selftest_text])
    missing.extend([f"report_check:{token}" for token in REQUIRED_REPORT_CHECK_TOKENS if token not in report_check_text])
    missing.extend(
        [f"report_selftest:{token}" for token in REQUIRED_REPORT_SELFTEST_TOKENS if token not in report_selftest_text]
    )

    if missing:
        print("ci sync readiness diagnostics check failed:")
        for token in missing[:16]:
            print(f" - missing token: {token}")
        return 1

    print("ci sync readiness diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
