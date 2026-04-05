#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from _ci_age5_combined_heavy_contract import (
    AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
    AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_GUARDED,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY,
    AGE5_FULL_REAL_CORE_LANG_SANITY_ELAPSED_FIELDS_TEXT,
    AGE5_FULL_REAL_ELAPSED_FIELDS_TEXT,
    AGE5_FULL_REAL_FIXED64_DARWIN_REAL_REPORT_READINESS_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROBE_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_ELAPSED_MAP_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_STATUS_MAP_FIELDS_TEXT,
    AGE5_FULL_REAL_TIMEOUT_BREAKDOWN_FIELDS_TEXT,
    build_age4_proof_snapshot,
    build_age4_proof_source_snapshot_fields,
    build_age4_proof_snapshot_text,
    build_age5_combined_heavy_full_real_source_trace,
    build_age5_close_digest_selftest_default_field,
    build_age5_combined_heavy_child_summary_default_text_transport_fields,
    build_age5_full_real_core_lang_sanity_elapsed_summary,
    build_age5_full_real_elapsed_summary,
    build_age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress,
    build_age5_full_real_pipeline_emit_flags_selftest_progress,
    build_age5_full_real_pipeline_emit_flags_selftest_probe,
    build_age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress,
    build_age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress,
    build_age5_full_real_profile_elapsed_map,
    build_age5_full_real_profile_status_map,
    build_age5_full_real_timeout_breakdown,
)


def fail(detail: str) -> int:
    print(f"[age5-close-digest-selftest] fail: {detail}")
    return 1


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def run_digest(report: Path, *extra: str) -> subprocess.CompletedProcess[str]:
    cmd = [sys.executable, "tools/scripts/print_age5_close_digest.py", str(report), *extra]
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def main() -> int:
    expected_default_transport = build_age5_combined_heavy_child_summary_default_text_transport_fields()
    expected_digest_default_field = build_age5_close_digest_selftest_default_field()
    expected_full_real_source_trace = build_age5_combined_heavy_full_real_source_trace(
        smoke_check_script_exists=True,
        smoke_check_selftest_script_exists=True,
    )
    expected_age4_proof_pass = build_age4_proof_snapshot(
        age4_proof_ok="1",
        age4_proof_failed_criteria="0",
        age4_proof_failed_preview="-",
    )
    expected_age4_proof_pass_source = build_age4_proof_source_snapshot_fields(
        top_snapshot=expected_age4_proof_pass,
        gate_result_snapshot=expected_age4_proof_pass,
        gate_result_present=True,
        final_status_parse_snapshot=expected_age4_proof_pass,
        final_status_parse_present=True,
    )
    expected_age4_proof_fail = build_age4_proof_snapshot(
        age4_proof_ok="0",
        age4_proof_failed_criteria="2",
        age4_proof_failed_preview="proof_runtime_error_statehash_preserved",
    )
    expected_age4_proof_fail_source = build_age4_proof_source_snapshot_fields(
        top_snapshot=expected_age4_proof_fail,
        gate_result_snapshot=expected_age4_proof_fail,
        gate_result_present=True,
        final_status_parse_snapshot=expected_age4_proof_fail,
        final_status_parse_present=True,
    )
    expected_policy_age4_proof = build_age4_proof_snapshot()
    expected_policy_age4_proof_source = build_age4_proof_source_snapshot_fields(
        top_snapshot=expected_policy_age4_proof
    )
    expected_full_real_elapsed_pass = build_age5_full_real_elapsed_summary()
    expected_full_real_core_lang_sanity_elapsed_pass = build_age5_full_real_core_lang_sanity_elapsed_summary()
    expected_full_real_profile_elapsed_map_pass = build_age5_full_real_profile_elapsed_map()
    expected_full_real_profile_status_map_pass = build_age5_full_real_profile_status_map()
    expected_full_real_pipeline_emit_flags_selftest_progress_pass = (
        build_age5_full_real_pipeline_emit_flags_selftest_progress()
    )
    expected_full_real_pipeline_emit_flags_selftest_probe_pass = (
        build_age5_full_real_pipeline_emit_flags_selftest_probe()
    )
    expected_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_pass = (
        build_age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress()
    )
    expected_full_real_profile_matrix_full_real_smoke_check_selftest_progress_pass = (
        build_age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress()
    )
    expected_full_real_elapsed_fail = build_age5_full_real_elapsed_summary(
        age5_full_real_total_elapsed_ms="10000",
        age5_full_real_slowest_profile="core_lang",
        age5_full_real_slowest_elapsed_ms="10000",
        age5_full_real_elapsed_present=True,
    )
    expected_full_real_core_lang_sanity_elapsed_fail = build_age5_full_real_core_lang_sanity_elapsed_summary(
        age5_full_real_core_lang_sanity_total_elapsed_ms="10000",
        age5_full_real_core_lang_sanity_slowest_step="pipeline_emit_flags_check",
        age5_full_real_core_lang_sanity_slowest_elapsed_ms="3000",
        age5_full_real_core_lang_sanity_elapsed_present=True,
    )
    expected_full_real_profile_elapsed_map_fail = build_age5_full_real_profile_elapsed_map(
        age5_full_real_profile_elapsed_map="core_lang:10000,full:-,seamgrim:-",
        age5_full_real_profile_elapsed_map_present=True,
    )
    expected_full_real_profile_status_map_fail = build_age5_full_real_profile_status_map(
        age5_full_real_profile_status_map="core_lang:fail,full:-,seamgrim:-",
        age5_full_real_profile_status_map_present=True,
    )
    expected_full_real_pipeline_emit_flags_selftest_progress_fail = (
        build_age5_full_real_pipeline_emit_flags_selftest_progress(
            age5_full_real_pipeline_emit_flags_selftest_current_case="broken_policy_helper_scope_should_fail",
            age5_full_real_pipeline_emit_flags_selftest_last_completed_case=(
                "missing_featured_seed_catalog_autogen_should_fail"
            ),
            age5_full_real_pipeline_emit_flags_selftest_total_elapsed_ms="2876",
            age5_full_real_pipeline_emit_flags_selftest_progress_present=True,
        )
    )
    expected_full_real_pipeline_emit_flags_selftest_probe_fail = (
        build_age5_full_real_pipeline_emit_flags_selftest_probe(
            age5_full_real_pipeline_emit_flags_selftest_current_probe="wait_runtime_ui_check",
            age5_full_real_pipeline_emit_flags_selftest_last_completed_probe="spawn_runtime_ui_check",
            age5_full_real_pipeline_emit_flags_selftest_probe_present=True,
        )
    )
    expected_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_fail = (
        build_age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress(
            age5_full_real_profile_matrix_full_real_smoke_policy_selftest_current_case="gitlab_default_off",
            age5_full_real_profile_matrix_full_real_smoke_policy_selftest_last_completed_case="-",
            age5_full_real_profile_matrix_full_real_smoke_policy_selftest_current_format="shell",
            age5_full_real_profile_matrix_full_real_smoke_policy_selftest_last_completed_format="json",
            age5_full_real_profile_matrix_full_real_smoke_policy_selftest_total_elapsed_ms="902",
            age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_present=True,
        )
    )
    expected_full_real_profile_matrix_full_real_smoke_check_selftest_progress_fail = (
        build_age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress(
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_case="override_ok_should_pass",
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_case="explicit_optin_should_fail",
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_total_elapsed_ms="311",
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_probe="run_smoke",
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_probe="validate_explicit_optin_failure",
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_present=True,
        )
    )
    expected_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_pass = (
        build_age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress()
    )
    expected_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_fail = (
        build_age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress(
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_case=(
                "pass_3way_should_pass"
            ),
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_case=(
                "pass_contract_only_should_pass"
            ),
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_total_elapsed_ms="517",
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_probe=(
                "validate_pass_3way"
            ),
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_probe=(
                "validate_contract_only"
            ),
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_present=True,
        )
    )
    expected_full_real_timeout_breakdown_pass = build_age5_full_real_timeout_breakdown()
    expected_full_real_timeout_breakdown_fail = build_age5_full_real_timeout_breakdown(
        age5_full_real_timeout_step="core_lang",
        age5_full_real_timeout_profiles="core_lang,full,seamgrim",
        age5_full_real_timeout_present=True,
    )

    with tempfile.TemporaryDirectory(prefix="age5_close_digest_selftest_") as td:
        root = Path(td)
        pass_report = root / "age5_close_pass.detjson"
        fail_report = root / "age5_close_fail.detjson"

        write_json(
            pass_report,
            {
                "schema": "ddn.age5_close_report.v1",
                "overall_ok": True,
                "criteria": [],
                "failure_digest": [],
                "age5_close_digest_selftest_ok": 1,
                AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
                "combined_digest_selftest_default_field": expected_digest_default_field,
                "age5_combined_heavy_full_real_status": "skipped",
                "age5_combined_heavy_runtime_helper_negative_status": "skipped",
                "age5_combined_heavy_group_id_summary_negative_status": "skipped",
                AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY: AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED,
                "full_real_source_trace": expected_full_real_source_trace,
                "age5_full_real_core_lang_sanity_elapsed_fields_text": AGE5_FULL_REAL_CORE_LANG_SANITY_ELAPSED_FIELDS_TEXT,
                "age5_full_real_elapsed_fields_text": AGE5_FULL_REAL_ELAPSED_FIELDS_TEXT,
                "age5_full_real_pipeline_emit_flags_selftest_progress_fields_text": (
                    AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROGRESS_FIELDS_TEXT
                ),
                "age5_full_real_pipeline_emit_flags_selftest_probe_fields_text": (
                    AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROBE_FIELDS_TEXT
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
                "age5_full_real_profile_elapsed_map_fields_text": AGE5_FULL_REAL_PROFILE_ELAPSED_MAP_FIELDS_TEXT,
                "age5_full_real_profile_status_map_fields_text": AGE5_FULL_REAL_PROFILE_STATUS_MAP_FIELDS_TEXT,
                "age5_full_real_timeout_breakdown_fields_text": AGE5_FULL_REAL_TIMEOUT_BREAKDOWN_FIELDS_TEXT,
                "age4_proof_snapshot_fields_text": AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
                "age4_proof_snapshot_text": build_age4_proof_snapshot_text(expected_age4_proof_pass),
                **expected_age4_proof_pass,
                **expected_age4_proof_pass_source,
                "policy_contract": {
                    "age4_proof_snapshot_fields_text": AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
                    "age4_proof_source_snapshot_fields_text": AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
                    "age4_proof_snapshot_text": build_age4_proof_snapshot_text(expected_policy_age4_proof),
                    **expected_policy_age4_proof,
                    **expected_policy_age4_proof_source,
                },
                **expected_default_transport,
                **expected_full_real_core_lang_sanity_elapsed_pass,
                **expected_full_real_elapsed_pass,
                **expected_full_real_pipeline_emit_flags_selftest_progress_pass,
                **expected_full_real_pipeline_emit_flags_selftest_probe_pass,
                **expected_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_pass,
                **expected_full_real_profile_matrix_full_real_smoke_check_selftest_progress_pass,
                **expected_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_pass,
                **expected_full_real_profile_elapsed_map_pass,
                **expected_full_real_profile_status_map_pass,
                **expected_full_real_timeout_breakdown_pass,
            },
        )
        pass_proc = run_digest(pass_report, "--top", "4", "--only-failed")
        if pass_proc.returncode != 0:
            return fail(f"pass_digest_rc={pass_proc.returncode}")
        pass_stdout = str(pass_proc.stdout or "")
        if "[age5-close] overall_ok=1" not in pass_stdout:
            return fail("pass_digest_overall_ok_missing")
        if "age5_full_real=skipped" not in pass_stdout:
            return fail("pass_digest_full_real_missing")
        if "age5_runtime_helper_negative=skipped" not in pass_stdout:
            return fail("pass_digest_runtime_helper_missing")
        if "age5_group_id_summary_negative=skipped" not in pass_stdout:
            return fail("pass_digest_group_id_missing")
        if "age5_combined_heavy_child_timeout_sec=0" not in pass_stdout:
            return fail("pass_digest_timeout_sec_missing")
        if f"age5_combined_heavy_timeout_mode={AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED}" not in pass_stdout:
            return fail("pass_digest_timeout_mode_missing")
        if "age5_combined_heavy_timeout_present=0" not in pass_stdout:
            return fail("pass_digest_timeout_present_missing")
        if "age5_combined_heavy_timeout_targets=-" not in pass_stdout:
            return fail("pass_digest_timeout_targets_missing")
        if (
            f"age5_full_real_core_lang_sanity_elapsed_fields_text={AGE5_FULL_REAL_CORE_LANG_SANITY_ELAPSED_FIELDS_TEXT}"
            not in pass_stdout
        ):
            return fail("pass_digest_full_real_core_lang_sanity_elapsed_fields_text_missing")
        if "age5_full_real_core_lang_sanity_total_elapsed_ms=-" not in pass_stdout:
            return fail("pass_digest_full_real_core_lang_sanity_total_elapsed_ms_missing")
        if "age5_full_real_core_lang_sanity_slowest_step=-" not in pass_stdout:
            return fail("pass_digest_full_real_core_lang_sanity_slowest_step_missing")
        if "age5_full_real_core_lang_sanity_slowest_elapsed_ms=-" not in pass_stdout:
            return fail("pass_digest_full_real_core_lang_sanity_slowest_elapsed_ms_missing")
        if "age5_full_real_core_lang_sanity_elapsed_present=0" not in pass_stdout:
            return fail("pass_digest_full_real_core_lang_sanity_elapsed_present_missing")
        if (
            f"age5_full_real_elapsed_fields_text={AGE5_FULL_REAL_ELAPSED_FIELDS_TEXT}"
            not in pass_stdout
        ):
            return fail("pass_digest_full_real_elapsed_fields_text_missing")
        if "age5_full_real_total_elapsed_ms=-" not in pass_stdout:
            return fail("pass_digest_full_real_total_elapsed_ms_missing")
        if "age5_full_real_slowest_profile=-" not in pass_stdout:
            return fail("pass_digest_full_real_slowest_profile_missing")
        if "age5_full_real_slowest_elapsed_ms=-" not in pass_stdout:
            return fail("pass_digest_full_real_slowest_elapsed_ms_missing")
        if "age5_full_real_elapsed_present=0" not in pass_stdout:
            return fail("pass_digest_full_real_elapsed_present_missing")
        if (
            "age5_full_real_pipeline_emit_flags_selftest_progress_fields_text="
            + AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROGRESS_FIELDS_TEXT
            not in pass_stdout
        ):
            return fail("pass_digest_pipeline_emit_flags_selftest_fields_text_missing")
        if "age5_full_real_pipeline_emit_flags_selftest_current_case=-" not in pass_stdout:
            return fail("pass_digest_pipeline_emit_flags_selftest_current_case_missing")
        if "age5_full_real_pipeline_emit_flags_selftest_last_completed_case=-" not in pass_stdout:
            return fail("pass_digest_pipeline_emit_flags_selftest_last_completed_case_missing")
        if "age5_full_real_pipeline_emit_flags_selftest_total_elapsed_ms=-" not in pass_stdout:
            return fail("pass_digest_pipeline_emit_flags_selftest_total_elapsed_ms_missing")
        if "age5_full_real_pipeline_emit_flags_selftest_progress_present=0" not in pass_stdout:
            return fail("pass_digest_pipeline_emit_flags_selftest_present_missing")
        if (
            "age5_full_real_pipeline_emit_flags_selftest_probe_fields_text="
            + AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROBE_FIELDS_TEXT
            not in pass_stdout
        ):
            return fail("pass_digest_pipeline_emit_flags_selftest_probe_fields_text_missing")
        if (
            "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_fields_text="
            + AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SELFTEST_PROGRESS_FIELDS_TEXT
            not in pass_stdout
        ):
            return fail("pass_digest_profile_matrix_full_real_smoke_policy_selftest_fields_text_missing")
        if (
            "age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_fields_text="
            + AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT
            not in pass_stdout
        ):
            return fail("pass_digest_profile_matrix_full_real_smoke_check_selftest_fields_text_missing")
        if "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_current_case=-" not in pass_stdout:
            return fail("pass_digest_profile_matrix_full_real_smoke_policy_selftest_current_case_missing")
        if "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_last_completed_case=-" not in pass_stdout:
            return fail("pass_digest_profile_matrix_full_real_smoke_policy_selftest_last_completed_case_missing")
        if "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_current_format=-" not in pass_stdout:
            return fail("pass_digest_profile_matrix_full_real_smoke_policy_selftest_current_format_missing")
        if "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_last_completed_format=-" not in pass_stdout:
            return fail("pass_digest_profile_matrix_full_real_smoke_policy_selftest_last_completed_format_missing")
        if "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_total_elapsed_ms=-" not in pass_stdout:
            return fail("pass_digest_profile_matrix_full_real_smoke_policy_selftest_total_elapsed_ms_missing")
        if "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_present=0" not in pass_stdout:
            return fail("pass_digest_profile_matrix_full_real_smoke_policy_selftest_present_missing")
        if "age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_case=-" not in pass_stdout:
            return fail("pass_digest_profile_matrix_full_real_smoke_check_selftest_current_case_missing")
        if "age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_case=-" not in pass_stdout:
            return fail("pass_digest_profile_matrix_full_real_smoke_check_selftest_last_completed_case_missing")
        if "age5_full_real_profile_matrix_full_real_smoke_check_selftest_total_elapsed_ms=-" not in pass_stdout:
            return fail("pass_digest_profile_matrix_full_real_smoke_check_selftest_total_elapsed_ms_missing")
        if "age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_probe=-" not in pass_stdout:
            return fail("pass_digest_profile_matrix_full_real_smoke_check_selftest_current_probe_missing")
        if "age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_probe=-" not in pass_stdout:
            return fail("pass_digest_profile_matrix_full_real_smoke_check_selftest_last_completed_probe_missing")
        if "age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_present=0" not in pass_stdout:
            return fail("pass_digest_profile_matrix_full_real_smoke_check_selftest_present_missing")
        if (
            "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_fields_text="
            + AGE5_FULL_REAL_FIXED64_DARWIN_REAL_REPORT_READINESS_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT
            not in pass_stdout
        ):
            return fail("pass_digest_fixed64_readiness_fields_text_missing")
        if "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_case=-" not in pass_stdout:
            return fail("pass_digest_fixed64_readiness_current_case_missing")
        if (
            "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_case=-"
            not in pass_stdout
        ):
            return fail("pass_digest_fixed64_readiness_last_completed_case_missing")
        if (
            "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_total_elapsed_ms=-"
            not in pass_stdout
        ):
            return fail("pass_digest_fixed64_readiness_total_elapsed_ms_missing")
        if "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_probe=-" not in pass_stdout:
            return fail("pass_digest_fixed64_readiness_current_probe_missing")
        if (
            "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_probe=-"
            not in pass_stdout
        ):
            return fail("pass_digest_fixed64_readiness_last_completed_probe_missing")
        if (
            "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_present=0"
            not in pass_stdout
        ):
            return fail("pass_digest_fixed64_readiness_present_missing")
        if "age5_full_real_pipeline_emit_flags_selftest_current_probe=-" not in pass_stdout:
            return fail("pass_digest_pipeline_emit_flags_selftest_current_probe_missing")
        if "age5_full_real_pipeline_emit_flags_selftest_last_completed_probe=-" not in pass_stdout:
            return fail("pass_digest_pipeline_emit_flags_selftest_last_completed_probe_missing")
        if "age5_full_real_pipeline_emit_flags_selftest_probe_present=0" not in pass_stdout:
            return fail("pass_digest_pipeline_emit_flags_selftest_probe_present_missing")
        if (
            f"age5_full_real_profile_elapsed_map_fields_text={AGE5_FULL_REAL_PROFILE_ELAPSED_MAP_FIELDS_TEXT}"
            not in pass_stdout
        ):
            return fail("pass_digest_full_real_profile_elapsed_map_fields_text_missing")
        if "age5_full_real_profile_elapsed_map=-" not in pass_stdout:
            return fail("pass_digest_full_real_profile_elapsed_map_missing")
        if "age5_full_real_profile_elapsed_map_present=0" not in pass_stdout:
            return fail("pass_digest_full_real_profile_elapsed_map_present_missing")
        if (
            f"age5_full_real_profile_status_map_fields_text={AGE5_FULL_REAL_PROFILE_STATUS_MAP_FIELDS_TEXT}"
            not in pass_stdout
        ):
            return fail("pass_digest_full_real_profile_status_map_fields_text_missing")
        if "age5_full_real_profile_status_map=-" not in pass_stdout:
            return fail("pass_digest_full_real_profile_status_map_missing")
        if "age5_full_real_profile_status_map_present=0" not in pass_stdout:
            return fail("pass_digest_full_real_profile_status_map_present_missing")
        if (
            f"age5_full_real_timeout_breakdown_fields_text={AGE5_FULL_REAL_TIMEOUT_BREAKDOWN_FIELDS_TEXT}"
            not in pass_stdout
        ):
            return fail("pass_digest_full_real_timeout_breakdown_fields_text_missing")
        if "age5_full_real_timeout_step=-" not in pass_stdout:
            return fail("pass_digest_full_real_timeout_step_missing")
        if "age5_full_real_timeout_profiles=-" not in pass_stdout:
            return fail("pass_digest_full_real_timeout_profiles_missing")
        if "age5_full_real_timeout_present=0" not in pass_stdout:
            return fail("pass_digest_full_real_timeout_present_missing")
        if "age5_full_real_source_check=1" not in pass_stdout:
            return fail("pass_digest_full_real_source_check_missing")
        if "age5_full_real_source_selftest=1" not in pass_stdout:
            return fail("pass_digest_full_real_source_selftest_missing")
        if f"age4_proof_snapshot_fields_text={AGE4_PROOF_SNAPSHOT_FIELDS_TEXT}" not in pass_stdout:
            return fail("pass_digest_age4_proof_snapshot_fields_text_missing")
        if (
            "age4_proof_snapshot_text="
            + build_age4_proof_snapshot_text(expected_age4_proof_pass)
            not in pass_stdout
        ):
            return fail("pass_digest_age4_proof_snapshot_text_missing")
        if "age4_proof_gate_result_present=1" not in pass_stdout:
            return fail("pass_digest_age4_proof_gate_result_present_missing")
        if "age4_proof_gate_result_parity=1" not in pass_stdout:
            return fail("pass_digest_age4_proof_gate_result_parity_missing")
        if "age4_proof_final_status_parse_present=1" not in pass_stdout:
            return fail("pass_digest_age4_proof_final_status_parse_present_missing")
        if "age4_proof_final_status_parse_parity=1" not in pass_stdout:
            return fail("pass_digest_age4_proof_final_status_parse_parity_missing")
        if f"age5_policy_age4_proof_snapshot_fields_text={AGE4_PROOF_SNAPSHOT_FIELDS_TEXT}" not in pass_stdout:
            return fail("pass_digest_policy_age4_proof_snapshot_fields_text_missing")
        if f"age5_policy_age4_proof_source_snapshot_fields_text={AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT}" not in pass_stdout:
            return fail("pass_digest_policy_age4_proof_source_snapshot_fields_text_missing")
        if (
            "age5_policy_age4_proof_snapshot_text="
            + build_age4_proof_snapshot_text(expected_policy_age4_proof)
            not in pass_stdout
        ):
            return fail("pass_digest_policy_age4_proof_snapshot_text_missing")
        if "age5_policy_age4_proof_gate_result_present=0" not in pass_stdout:
            return fail("pass_digest_policy_age4_proof_gate_result_present_missing")
        if "age5_policy_age4_proof_gate_result_parity=0" not in pass_stdout:
            return fail("pass_digest_policy_age4_proof_gate_result_parity_missing")
        if "age5_policy_age4_proof_final_status_parse_present=0" not in pass_stdout:
            return fail("pass_digest_policy_age4_proof_final_status_parse_present_missing")
        if "age5_policy_age4_proof_final_status_parse_parity=0" not in pass_stdout:
            return fail("pass_digest_policy_age4_proof_final_status_parse_parity_missing")
        if "age5_close_digest_selftest_ok=1" not in pass_stdout:
            return fail("pass_digest_selftest_ok_missing")
        if f"{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}={AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT}" not in pass_stdout:
            return fail("pass_digest_selftest_default_text_missing")
        if (
            "combined_digest_selftest_default_field="
            + json.dumps(expected_digest_default_field, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            not in pass_stdout
        ):
            return fail("pass_digest_selftest_default_field_missing")
        if (
            "age5_child_summary_defaults="
            + expected_default_transport["ci_sanity_age5_combined_heavy_child_summary_default_fields"]
            not in pass_stdout
        ):
            return fail("pass_digest_child_summary_defaults_missing")
        if (
            "age5_sync_child_summary_defaults="
            + expected_default_transport["ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"]
            not in pass_stdout
        ):
            return fail("pass_digest_sync_child_summary_defaults_missing")
        if "failure_digest=(none)" in pass_stdout:
            return fail("pass_only_failed_should_not_emit_digest_lines")

        write_json(
            fail_report,
            {
                "schema": "ddn.age5_close_report.v1",
                "overall_ok": False,
                "criteria": [
                    {"name": "age5_ci_profile_matrix_full_real_smoke_optin_pass", "ok": False},
                    {"name": "age5_ci_profile_core_lang_runtime_helper_negative_optin_pass", "ok": False},
                ],
                "failure_digest": [
                    "criteria=age5_ci_profile_matrix_full_real_smoke_optin_pass",
                    "criteria=age5_ci_profile_core_lang_runtime_helper_negative_optin_pass",
                ],
                "age5_close_digest_selftest_ok": 1,
                AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
                "combined_digest_selftest_default_field": expected_digest_default_field,
                "age5_combined_heavy_full_real_status": "fail",
                "age5_combined_heavy_runtime_helper_negative_status": "fail",
                "age5_combined_heavy_group_id_summary_negative_status": "skipped",
                "combined_heavy_child_timeout_sec": 3,
                AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY: AGE5_COMBINED_HEAVY_TIMEOUT_MODE_GUARDED,
                "age5_combined_heavy_timeout_present": "1",
                "age5_combined_heavy_timeout_targets": "full_real,runtime_helper_negative",
                "full_real_source_trace": expected_full_real_source_trace,
                "age5_full_real_core_lang_sanity_elapsed_fields_text": AGE5_FULL_REAL_CORE_LANG_SANITY_ELAPSED_FIELDS_TEXT,
                "age5_full_real_elapsed_fields_text": AGE5_FULL_REAL_ELAPSED_FIELDS_TEXT,
                "age5_full_real_pipeline_emit_flags_selftest_progress_fields_text": (
                    AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROGRESS_FIELDS_TEXT
                ),
                "age5_full_real_pipeline_emit_flags_selftest_probe_fields_text": (
                    AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROBE_FIELDS_TEXT
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
                "age5_full_real_profile_elapsed_map_fields_text": AGE5_FULL_REAL_PROFILE_ELAPSED_MAP_FIELDS_TEXT,
                "age5_full_real_profile_status_map_fields_text": AGE5_FULL_REAL_PROFILE_STATUS_MAP_FIELDS_TEXT,
                "age5_full_real_timeout_breakdown_fields_text": AGE5_FULL_REAL_TIMEOUT_BREAKDOWN_FIELDS_TEXT,
                "age4_proof_snapshot_fields_text": AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
                "age4_proof_snapshot_text": build_age4_proof_snapshot_text(expected_age4_proof_fail),
                **expected_age4_proof_fail,
                **expected_age4_proof_fail_source,
                "policy_contract": {
                    "age4_proof_snapshot_fields_text": AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
                    "age4_proof_source_snapshot_fields_text": AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
                    "age4_proof_snapshot_text": build_age4_proof_snapshot_text(expected_policy_age4_proof),
                    **expected_policy_age4_proof,
                    **expected_policy_age4_proof_source,
                },
                **expected_default_transport,
                **expected_full_real_core_lang_sanity_elapsed_fail,
                **expected_full_real_elapsed_fail,
                **expected_full_real_pipeline_emit_flags_selftest_progress_fail,
                **expected_full_real_pipeline_emit_flags_selftest_probe_fail,
                **expected_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_fail,
                **expected_full_real_profile_matrix_full_real_smoke_check_selftest_progress_fail,
                **expected_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_fail,
                **expected_full_real_profile_elapsed_map_fail,
                **expected_full_real_profile_status_map_fail,
                **expected_full_real_timeout_breakdown_fail,
            },
        )
        fail_proc = run_digest(fail_report, "--top", "2", "--only-failed")
        if fail_proc.returncode != 0:
            return fail(f"fail_digest_rc={fail_proc.returncode}")
        fail_stdout = str(fail_proc.stdout or "")
        if "[age5-close] overall_ok=0 criteria=2 failed=2" not in fail_stdout:
            return fail("fail_digest_overall_summary_missing")
        if "age5_full_real=fail" not in fail_stdout:
            return fail("fail_digest_full_real_missing")
        if "age5_runtime_helper_negative=fail" not in fail_stdout:
            return fail("fail_digest_runtime_helper_missing")
        if "age5_group_id_summary_negative=skipped" not in fail_stdout:
            return fail("fail_digest_group_id_missing")
        if "age5_combined_heavy_child_timeout_sec=3" not in fail_stdout:
            return fail("fail_digest_timeout_sec_missing")
        if (
            "age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_case="
            + expected_full_real_profile_matrix_full_real_smoke_check_selftest_progress_fail[
                "age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_case"
            ]
            not in fail_stdout
        ):
            return fail("fail_digest_profile_matrix_full_real_smoke_check_selftest_current_case_missing")
        if (
            "age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_case="
            + expected_full_real_profile_matrix_full_real_smoke_check_selftest_progress_fail[
                "age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_case"
            ]
            not in fail_stdout
        ):
            return fail("fail_digest_profile_matrix_full_real_smoke_check_selftest_last_completed_case_missing")
        if (
            "age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_probe="
            + expected_full_real_profile_matrix_full_real_smoke_check_selftest_progress_fail[
                "age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_probe"
            ]
            not in fail_stdout
        ):
            return fail("fail_digest_profile_matrix_full_real_smoke_check_selftest_current_probe_missing")
        if (
            "age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_probe="
            + expected_full_real_profile_matrix_full_real_smoke_check_selftest_progress_fail[
                "age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_probe"
            ]
            not in fail_stdout
        ):
            return fail("fail_digest_profile_matrix_full_real_smoke_check_selftest_last_completed_probe_missing")
        if (
            "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_case="
            + expected_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_fail[
                "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_case"
            ]
            not in fail_stdout
        ):
            return fail("fail_digest_fixed64_readiness_current_case_missing")
        if (
            "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_case="
            + expected_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_fail[
                "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_case"
            ]
            not in fail_stdout
        ):
            return fail("fail_digest_fixed64_readiness_last_completed_case_missing")
        if (
            "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_probe="
            + expected_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_fail[
                "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_probe"
            ]
            not in fail_stdout
        ):
            return fail("fail_digest_fixed64_readiness_current_probe_missing")
        if (
            "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_probe="
            + expected_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_fail[
                "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_probe"
            ]
            not in fail_stdout
        ):
            return fail("fail_digest_fixed64_readiness_last_completed_probe_missing")
        if (
            "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_total_elapsed_ms=517"
            not in fail_stdout
        ):
            return fail("fail_digest_fixed64_readiness_total_elapsed_ms_missing")
        if (
            "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_present=1"
            not in fail_stdout
        ):
            return fail("fail_digest_fixed64_readiness_present_missing")
        if f"age5_combined_heavy_timeout_mode={AGE5_COMBINED_HEAVY_TIMEOUT_MODE_GUARDED}" not in fail_stdout:
            return fail("fail_digest_timeout_mode_missing")
        if "age5_combined_heavy_timeout_present=1" not in fail_stdout:
            return fail("fail_digest_timeout_present_missing")
        if "age5_combined_heavy_timeout_targets=full_real,runtime_helper_negative" not in fail_stdout:
            return fail("fail_digest_timeout_targets_missing")
        if (
            f"age5_full_real_core_lang_sanity_elapsed_fields_text={AGE5_FULL_REAL_CORE_LANG_SANITY_ELAPSED_FIELDS_TEXT}"
            not in fail_stdout
        ):
            return fail("fail_digest_full_real_core_lang_sanity_elapsed_fields_text_missing")
        if "age5_full_real_core_lang_sanity_total_elapsed_ms=10000" not in fail_stdout:
            return fail("fail_digest_full_real_core_lang_sanity_total_elapsed_ms_missing")
        if "age5_full_real_core_lang_sanity_slowest_step=pipeline_emit_flags_check" not in fail_stdout:
            return fail("fail_digest_full_real_core_lang_sanity_slowest_step_missing")
        if "age5_full_real_core_lang_sanity_slowest_elapsed_ms=3000" not in fail_stdout:
            return fail("fail_digest_full_real_core_lang_sanity_slowest_elapsed_ms_missing")
        if "age5_full_real_core_lang_sanity_elapsed_present=1" not in fail_stdout:
            return fail("fail_digest_full_real_core_lang_sanity_elapsed_present_missing")
        if (
            f"age5_full_real_elapsed_fields_text={AGE5_FULL_REAL_ELAPSED_FIELDS_TEXT}"
            not in fail_stdout
        ):
            return fail("fail_digest_full_real_elapsed_fields_text_missing")
        if "age5_full_real_total_elapsed_ms=10000" not in fail_stdout:
            return fail("fail_digest_full_real_total_elapsed_ms_missing")
        if "age5_full_real_slowest_profile=core_lang" not in fail_stdout:
            return fail("fail_digest_full_real_slowest_profile_missing")
        if "age5_full_real_slowest_elapsed_ms=10000" not in fail_stdout:
            return fail("fail_digest_full_real_slowest_elapsed_ms_missing")
        if "age5_full_real_elapsed_present=1" not in fail_stdout:
            return fail("fail_digest_full_real_elapsed_present_missing")
        if (
            "age5_full_real_pipeline_emit_flags_selftest_progress_fields_text="
            + AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROGRESS_FIELDS_TEXT
            not in fail_stdout
        ):
            return fail("fail_digest_pipeline_emit_flags_selftest_fields_text_missing")
        if (
            "age5_full_real_pipeline_emit_flags_selftest_current_case=broken_policy_helper_scope_should_fail"
            not in fail_stdout
        ):
            return fail("fail_digest_pipeline_emit_flags_selftest_current_case_missing")
        if (
            "age5_full_real_pipeline_emit_flags_selftest_last_completed_case=missing_featured_seed_catalog_autogen_should_fail"
            not in fail_stdout
        ):
            return fail("fail_digest_pipeline_emit_flags_selftest_last_completed_case_missing")
        if "age5_full_real_pipeline_emit_flags_selftest_total_elapsed_ms=2876" not in fail_stdout:
            return fail("fail_digest_pipeline_emit_flags_selftest_total_elapsed_ms_missing")
        if "age5_full_real_pipeline_emit_flags_selftest_progress_present=1" not in fail_stdout:
            return fail("fail_digest_pipeline_emit_flags_selftest_present_missing")
        if (
            "age5_full_real_pipeline_emit_flags_selftest_probe_fields_text="
            + AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROBE_FIELDS_TEXT
            not in fail_stdout
        ):
            return fail("fail_digest_pipeline_emit_flags_selftest_probe_fields_text_missing")
        if (
            "age5_full_real_pipeline_emit_flags_selftest_current_probe=wait_runtime_ui_check"
            not in fail_stdout
        ):
            return fail("fail_digest_pipeline_emit_flags_selftest_current_probe_missing")
        if (
            "age5_full_real_pipeline_emit_flags_selftest_last_completed_probe=spawn_runtime_ui_check"
            not in fail_stdout
        ):
            return fail("fail_digest_pipeline_emit_flags_selftest_last_completed_probe_missing")
        if "age5_full_real_pipeline_emit_flags_selftest_probe_present=1" not in fail_stdout:
            return fail("fail_digest_pipeline_emit_flags_selftest_probe_present_missing")
        if (
            f"age5_full_real_profile_elapsed_map_fields_text={AGE5_FULL_REAL_PROFILE_ELAPSED_MAP_FIELDS_TEXT}"
            not in fail_stdout
        ):
            return fail("fail_digest_full_real_profile_elapsed_map_fields_text_missing")
        if "age5_full_real_profile_elapsed_map=core_lang:10000,full:-,seamgrim:-" not in fail_stdout:
            return fail("fail_digest_full_real_profile_elapsed_map_missing")
        if "age5_full_real_profile_elapsed_map_present=1" not in fail_stdout:
            return fail("fail_digest_full_real_profile_elapsed_map_present_missing")
        if (
            f"age5_full_real_profile_status_map_fields_text={AGE5_FULL_REAL_PROFILE_STATUS_MAP_FIELDS_TEXT}"
            not in fail_stdout
        ):
            return fail("fail_digest_full_real_profile_status_map_fields_text_missing")
        if "age5_full_real_profile_status_map=core_lang:fail,full:-,seamgrim:-" not in fail_stdout:
            return fail("fail_digest_full_real_profile_status_map_missing")
        if "age5_full_real_profile_status_map_present=1" not in fail_stdout:
            return fail("fail_digest_full_real_profile_status_map_present_missing")
        if (
            f"age5_full_real_timeout_breakdown_fields_text={AGE5_FULL_REAL_TIMEOUT_BREAKDOWN_FIELDS_TEXT}"
            not in fail_stdout
        ):
            return fail("fail_digest_full_real_timeout_breakdown_fields_text_missing")
        if "age5_full_real_timeout_step=core_lang" not in fail_stdout:
            return fail("fail_digest_full_real_timeout_step_missing")
        if "age5_full_real_timeout_profiles=core_lang,full,seamgrim" not in fail_stdout:
            return fail("fail_digest_full_real_timeout_profiles_missing")
        if "age5_full_real_timeout_present=1" not in fail_stdout:
            return fail("fail_digest_full_real_timeout_present_missing")
        if "age5_full_real_source_check=1" not in fail_stdout:
            return fail("fail_digest_full_real_source_check_missing")
        if "age5_full_real_source_selftest=1" not in fail_stdout:
            return fail("fail_digest_full_real_source_selftest_missing")
        if f"age4_proof_snapshot_fields_text={AGE4_PROOF_SNAPSHOT_FIELDS_TEXT}" not in fail_stdout:
            return fail("fail_digest_age4_proof_snapshot_fields_text_missing")
        if (
            "age4_proof_snapshot_text="
            + build_age4_proof_snapshot_text(expected_age4_proof_fail)
            not in fail_stdout
        ):
            return fail("fail_digest_age4_proof_snapshot_text_missing")
        if "age4_proof_gate_result_present=1" not in fail_stdout:
            return fail("fail_digest_age4_proof_gate_result_present_missing")
        if "age4_proof_gate_result_parity=1" not in fail_stdout:
            return fail("fail_digest_age4_proof_gate_result_parity_missing")
        if "age4_proof_final_status_parse_present=1" not in fail_stdout:
            return fail("fail_digest_age4_proof_final_status_parse_present_missing")
        if "age4_proof_final_status_parse_parity=1" not in fail_stdout:
            return fail("fail_digest_age4_proof_final_status_parse_parity_missing")
        if f"age5_policy_age4_proof_snapshot_fields_text={AGE4_PROOF_SNAPSHOT_FIELDS_TEXT}" not in fail_stdout:
            return fail("fail_digest_policy_age4_proof_snapshot_fields_text_missing")
        if f"age5_policy_age4_proof_source_snapshot_fields_text={AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT}" not in fail_stdout:
            return fail("fail_digest_policy_age4_proof_source_snapshot_fields_text_missing")
        if (
            "age5_policy_age4_proof_snapshot_text="
            + build_age4_proof_snapshot_text(expected_policy_age4_proof)
            not in fail_stdout
        ):
            return fail("fail_digest_policy_age4_proof_snapshot_text_missing")
        if "age5_policy_age4_proof_gate_result_present=0" not in fail_stdout:
            return fail("fail_digest_policy_age4_proof_gate_result_present_missing")
        if "age5_policy_age4_proof_gate_result_parity=0" not in fail_stdout:
            return fail("fail_digest_policy_age4_proof_gate_result_parity_missing")
        if "age5_policy_age4_proof_final_status_parse_present=0" not in fail_stdout:
            return fail("fail_digest_policy_age4_proof_final_status_parse_present_missing")
        if "age5_policy_age4_proof_final_status_parse_parity=0" not in fail_stdout:
            return fail("fail_digest_policy_age4_proof_final_status_parse_parity_missing")
        if "age5_close_digest_selftest_ok=1" not in fail_stdout:
            return fail("fail_digest_selftest_ok_missing")
        if f"{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}={AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT}" not in fail_stdout:
            return fail("fail_digest_selftest_default_text_missing")
        if (
            "combined_digest_selftest_default_field="
            + json.dumps(expected_digest_default_field, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
            not in fail_stdout
        ):
            return fail("fail_digest_selftest_default_field_missing")
        if " - criteria=age5_ci_profile_matrix_full_real_smoke_optin_pass" not in fail_stdout:
            return fail("fail_digest_line_1_missing")
        if " - criteria=age5_ci_profile_core_lang_runtime_helper_negative_optin_pass" not in fail_stdout:
            return fail("fail_digest_line_2_missing")
        if (
            "age5_child_summary_defaults="
            + expected_default_transport["ci_sanity_age5_combined_heavy_child_summary_default_fields"]
            not in fail_stdout
        ):
            return fail("fail_digest_child_summary_defaults_missing")
        if (
            "age5_sync_child_summary_defaults="
            + expected_default_transport["ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"]
            not in fail_stdout
        ):
            return fail("fail_digest_sync_child_summary_defaults_missing")

    print("[age5-close-digest-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
