#!/usr/bin/env python
from __future__ import annotations

import json
import io
from contextlib import redirect_stderr, redirect_stdout
import importlib
import runpy
import subprocess
import sys
import tempfile
from pathlib import Path

from _ci_age5_combined_heavy_contract import (
    AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
    AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
    build_age5_combined_heavy_policy_origin_trace,
    build_age5_combined_heavy_policy_origin_trace_text,
    build_age4_proof_snapshot,
    build_age4_proof_snapshot_text,
    build_age4_proof_source_snapshot_fields,
    build_age5_close_digest_selftest_default_field,
    build_age5_combined_heavy_child_summary_default_text_transport_fields,
    build_age5_combined_heavy_sanity_contract_fields,
    build_age5_combined_heavy_sync_contract_fields,
)
from _ci_profile_matrix_selftest_lib import (
    build_profile_matrix_snapshot_from_doc,
    build_profile_matrix_triage_payload_from_snapshot,
    PROFILE_MATRIX_SELFTEST_PROFILES,
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC,
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
    PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS,
    expected_profile_matrix_aggregate_summary_contract,
)
from _ci_latest_smoke_contract import (
    LATEST_SMOKE_SKIP_REASON_CI_GATE_RESULT_STATUS_NOT_PASS,
    LATEST_SMOKE_SKIP_REASON_EXPECTED,
    LATEST_SMOKE_SKIP_REASON_FAST_FAIL_PATH,
    LATEST_SMOKE_SKIP_REASON_FLAG_DISABLED,
    LATEST_SMOKE_SKIP_REASON_PENDING_FAILURE_SUMMARY_REGENERATION,
)
from ci_check_error_codes import GATE_REPORT_INDEX_CODES as CODES

REQUIRED_STEPS_COMMON = (
    "ci_profile_split_contract_check",
    "ci_profile_matrix_gate_selftest",
    "age5_close_digest_selftest",
    "ci_fail_and_exit_contract_selftest",
    "ci_sanity_gate",
    "ci_sync_readiness_report_generate",
    "ci_sync_readiness_report_check",
    "ci_emit_artifacts_required_post_summary_check",
    "ci_gate_report_index_selftest",
    "ci_gate_report_index_diagnostics_check",
    "ci_gate_report_index_latest_smoke_check",
)
REQUIRED_STEPS_SEAMGRIM = (
    "seamgrim_ci_gate_seed_meta_step_check",
    "seamgrim_ci_gate_runtime5_passthrough_check",
    "seamgrim_ci_gate_guideblock_step_check",
    "seamgrim_ci_gate_lesson_warning_step_check",
    "seamgrim_ci_gate_stateful_preview_step_check",
    "seamgrim_ci_gate_wasm_web_smoke_step_check",
    "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
    "seamgrim_wasm_cli_diag_parity_check",
)
REQUIRED_STEPS_FULL = REQUIRED_STEPS_COMMON + REQUIRED_STEPS_SEAMGRIM
REQUIRED_STEPS_CORE_LANG = REQUIRED_STEPS_COMMON
AGE5_CHILD_SUMMARY_DEFAULTS = {
    "age5_combined_heavy_full_real_status": "skipped",
    "age5_combined_heavy_runtime_helper_negative_status": "skipped",
    "age5_combined_heavy_group_id_summary_negative_status": "skipped",
}
AGE5_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT = build_age5_combined_heavy_child_summary_default_text_transport_fields()
AGE5_DIGEST_SELFTEST_DEFAULT_FIELD = build_age5_close_digest_selftest_default_field()
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
AGE5_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY = "age5_policy_summary_origin_trace_contract_source_issue"
# split-contract token anchor: age5_policy_summary_origin_trace_contract_compact_reason
# split-contract token anchor: age5_policy_summary_origin_trace_contract_compact_failure_reason
AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY = AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY
AGE5_POLICY_ORIGIN_TRACE_KEY = AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY
AGE4_PROOF_OK_KEY = "age4_proof_ok"
AGE4_PROOF_FAILED_CRITERIA_KEY = "age4_proof_failed_criteria"
AGE4_PROOF_FAILED_PREVIEW_KEY = "age4_proof_failed_preview"
AGE4_PROOF_SUMMARY_HASH_KEY = "age4_proof_summary_hash"
AGE5_W107_PROGRESS_FIXTURE = {
    "age5_full_real_w107_golden_index_selftest_active_cases": "54",
    "age5_full_real_w107_golden_index_selftest_inactive_cases": "1",
    "age5_full_real_w107_golden_index_selftest_index_codes": "34",
    "age5_full_real_w107_golden_index_selftest_current_probe": "-",
    "age5_full_real_w107_golden_index_selftest_last_completed_probe": "validate_pack_pointers",
    "age5_full_real_w107_golden_index_selftest_progress_present": "1",
}
AGE5_W107_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_w107_progress_contract_selftest_completed_checks": "8",
    "age5_full_real_w107_progress_contract_selftest_total_checks": "8",
    "age5_full_real_w107_progress_contract_selftest_checks_text": "golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index",
    "age5_full_real_w107_progress_contract_selftest_current_probe": "-",
    "age5_full_real_w107_progress_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_w107_progress_contract_selftest_progress_present": "1",
}
AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks": "5",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks": "5",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text": "operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe": "-",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe": "proof_operation_family",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present": "1",
}
AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks": "5",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks": "5",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text": "signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe": "-",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe": "signed_contract",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present": "1",
}
AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks": "1",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks": "1",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text": "verify_report_digest_contract",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe": "-",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe": "readme_and_field_contract",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present": "1",
}
AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks": "4",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks": "4",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text": "signed_contract,consumer_contract,promotion,family",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe": "-",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe": "family",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present": "1",
}
AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_proof_certificate_family_contract_selftest_completed_checks": "3",
    "age5_full_real_proof_certificate_family_contract_selftest_total_checks": "3",
    "age5_full_real_proof_certificate_family_contract_selftest_checks_text": "artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family",
    "age5_full_real_proof_certificate_family_contract_selftest_current_probe": "-",
    "age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe": "proof_certificate_family",
    "age5_full_real_proof_certificate_family_contract_selftest_progress_present": "1",
}
AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_proof_family_contract_selftest_completed_checks": "3",
    "age5_full_real_proof_family_contract_selftest_total_checks": "3",
    "age5_full_real_proof_family_contract_selftest_checks_text": "proof_operation_family,proof_certificate_family,proof_family",
    "age5_full_real_proof_family_contract_selftest_current_probe": "-",
    "age5_full_real_proof_family_contract_selftest_last_completed_probe": "proof_family",
    "age5_full_real_proof_family_contract_selftest_progress_present": "1",
}
AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_proof_family_transport_contract_selftest_completed_checks": "9",
    "age5_full_real_proof_family_transport_contract_selftest_total_checks": "9",
    "age5_full_real_proof_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
    "age5_full_real_proof_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_proof_family_transport_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_proof_family_transport_contract_selftest_progress_present": "1",
}
AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_lang_surface_family_contract_selftest_completed_checks": "4",
    "age5_full_real_lang_surface_family_contract_selftest_total_checks": "4",
    "age5_full_real_lang_surface_family_contract_selftest_checks_text": "proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family",
    "age5_full_real_lang_surface_family_contract_selftest_current_probe": "-",
    "age5_full_real_lang_surface_family_contract_selftest_last_completed_probe": "lang_surface_family",
    "age5_full_real_lang_surface_family_contract_selftest_progress_present": "1",
}
AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_lang_runtime_family_contract_selftest_completed_checks": "5",
    "age5_full_real_lang_runtime_family_contract_selftest_total_checks": "5",
    "age5_full_real_lang_runtime_family_contract_selftest_checks_text": "lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family",
    "age5_full_real_lang_runtime_family_contract_selftest_current_probe": "-",
    "age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe": "lang_runtime_family",
    "age5_full_real_lang_runtime_family_contract_selftest_progress_present": "1",
}
AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks": "9",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks": "9",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present": "1",
}
AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_gate0_family_contract_selftest_completed_checks": "5",
    "age5_full_real_gate0_family_contract_selftest_total_checks": "5",
    "age5_full_real_gate0_family_contract_selftest_checks_text": "gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family",
    "age5_full_real_gate0_family_contract_selftest_current_probe": "-",
    "age5_full_real_gate0_family_contract_selftest_last_completed_probe": "gate0_family",
    "age5_full_real_gate0_family_contract_selftest_progress_present": "1",
}
AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_gate0_family_transport_contract_selftest_completed_checks": "9",
    "age5_full_real_gate0_family_transport_contract_selftest_total_checks": "9",
    "age5_full_real_gate0_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
    "age5_full_real_gate0_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_gate0_family_transport_contract_selftest_progress_present": "1",
}
AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks": "9",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks": "9",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present": "1",
}
AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_gate0_transport_family_contract_selftest_completed_checks": "4",
    "age5_full_real_gate0_transport_family_contract_selftest_total_checks": "4",
    "age5_full_real_gate0_transport_family_contract_selftest_checks_text": "lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family",
    "age5_full_real_gate0_transport_family_contract_selftest_current_probe": "-",
    "age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe": "gate0_transport_family",
    "age5_full_real_gate0_transport_family_contract_selftest_progress_present": "1",
}
AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks": "1",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks": "1",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text": "family_contract",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe": "family_contract",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present": "1",
}
AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks": "9",
    "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks": "9",
    "age5_full_real_lang_surface_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
    "age5_full_real_lang_surface_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present": "1",
}
AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks": "9",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks": "9",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present": "1",
}
AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_bogae_alias_family_contract_selftest_completed_checks": "3",
    "age5_full_real_bogae_alias_family_contract_selftest_total_checks": "3",
    "age5_full_real_bogae_alias_family_contract_selftest_checks_text": "shape_alias_contract,alias_family,alias_viewer_family",
    "age5_full_real_bogae_alias_family_contract_selftest_current_probe": "-",
    "age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe": "alias_viewer_family",
    "age5_full_real_bogae_alias_family_contract_selftest_progress_present": "1",
}
AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks": "9",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks": "9",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present": "1",
}

_CI_GATE_REPORT_INDEX_CHECK_MODULE = None
_ENSURED_PARENT_DIRS: set[Path] = set()
_SHARED_CASE_DIR_NAME = "_shared_case"


def fail(msg: str) -> int:
    print(f"[ci-gate-report-index-selftest] fail: {msg}")
    return 1


def check_latest_smoke_skip_reason_contract() -> str | None:
    expected_reason_set = {
        LATEST_SMOKE_SKIP_REASON_FAST_FAIL_PATH,
        LATEST_SMOKE_SKIP_REASON_FLAG_DISABLED,
        LATEST_SMOKE_SKIP_REASON_PENDING_FAILURE_SUMMARY_REGENERATION,
        LATEST_SMOKE_SKIP_REASON_CI_GATE_RESULT_STATUS_NOT_PASS,
    }
    observed_reason_set = set(LATEST_SMOKE_SKIP_REASON_EXPECTED)
    if observed_reason_set != expected_reason_set:
        return (
            "latest-smoke reason contract mismatch: "
            f"expected={sorted(expected_reason_set)} observed={sorted(observed_reason_set)}"
        )
    return None


def ensure_parent_dir(path: Path) -> None:
    parent = path.parent
    if parent in _ENSURED_PARENT_DIRS:
        return
    parent.mkdir(parents=True, exist_ok=True)
    _ENSURED_PARENT_DIRS.add(parent)


def resolve_selftest_root() -> Path:
    root = (
        Path(tempfile.gettempdir())
        / "ddn_ci_gate_report_index_selftest_workspace"
        / "shared"
    )
    root.mkdir(parents=True, exist_ok=True)
    return root


def write_text(path: Path, text: str) -> None:
    ensure_parent_dir(path)
    content = text.rstrip() + "\n"
    if path.exists():
        try:
            if path.read_text(encoding="utf-8") == content:
                return
        except OSError:
            pass
    path.write_text(content, encoding="utf-8")


def write_json(path: Path, payload: dict) -> None:
    ensure_parent_dir(path)
    content = json.dumps(payload, ensure_ascii=False, separators=(",", ":")) + "\n"
    if path.exists():
        try:
            if path.read_text(encoding="utf-8") == content:
                return
        except OSError:
            pass
    path.write_text(content, encoding="utf-8")


def _run_ci_gate_report_index_check_module(
    cmd: list[str],
    argv: list[str],
) -> subprocess.CompletedProcess[str]:
    global _CI_GATE_REPORT_INDEX_CHECK_MODULE
    if _CI_GATE_REPORT_INDEX_CHECK_MODULE is None:
        _CI_GATE_REPORT_INDEX_CHECK_MODULE = importlib.import_module("run_ci_gate_report_index_check")
    module = _CI_GATE_REPORT_INDEX_CHECK_MODULE

    stdout_buf = io.StringIO()
    stderr_buf = io.StringIO()
    old_argv = sys.argv
    returncode = 0
    try:
        sys.argv = argv
        with redirect_stdout(stdout_buf), redirect_stderr(stderr_buf):
            try:
                code = module.main()
                if code is None:
                    returncode = 0
                elif isinstance(code, int):
                    returncode = code
                else:
                    returncode = 1
                    stderr_buf.write(str(code))
            except SystemExit as exc:
                code = exc.code
                if code is None:
                    returncode = 0
                elif isinstance(code, int):
                    returncode = code
                else:
                    returncode = 1
                    stderr_buf.write(str(code))
            except Exception as exc:  # pragma: no cover - defensive fallback
                returncode = 1
                stderr_buf.write(f"{type(exc).__name__}: {exc}")
    finally:
        sys.argv = old_argv
    return subprocess.CompletedProcess(
        args=cmd,
        returncode=returncode,
        stdout=stdout_buf.getvalue(),
        stderr=stderr_buf.getvalue(),
    )


def run_cmd_inprocess(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    if len(cmd) < 2 or not str(cmd[1]).endswith(".py"):
        return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")
    script = str(cmd[1])
    script_norm = script.replace("\\", "/")
    argv = [script, *[str(arg) for arg in cmd[2:]]]
    if script_norm.endswith("tests/run_ci_gate_report_index_check.py"):
        return _run_ci_gate_report_index_check_module(cmd, argv)
    stdout_buf = io.StringIO()
    stderr_buf = io.StringIO()
    old_argv = sys.argv
    returncode = 0
    try:
        sys.argv = argv
        with redirect_stdout(stdout_buf), redirect_stderr(stderr_buf):
            try:
                runpy.run_path(script, run_name="__main__")
            except SystemExit as exc:
                code = exc.code
                if code is None:
                    returncode = 0
                elif isinstance(code, int):
                    returncode = code
                else:
                    returncode = 1
                    stderr_buf.write(str(code))
            except Exception as exc:  # pragma: no cover - defensive fallback
                returncode = 1
                stderr_buf.write(f"{type(exc).__name__}: {exc}")
    finally:
        sys.argv = old_argv
    return subprocess.CompletedProcess(
        args=cmd,
        returncode=returncode,
        stdout=stdout_buf.getvalue(),
        stderr=stderr_buf.getvalue(),
    )


def run_check(
    index: Path,
    required_steps: tuple[str, ...] = (),
    *,
    sanity_profile: str = "full",
    enforce_profile_step_contract: bool = False,
) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_ci_gate_report_index_check.py",
        "--index",
        str(index),
        "--sanity-profile",
        sanity_profile,
    ]
    if enforce_profile_step_contract:
        cmd.append("--enforce-profile-step-contract")
    for step in required_steps:
        cmd.extend(["--required-step", step])
    return run_cmd_inprocess(cmd)


def build_sanity_steps(sanity_profile: str) -> list[dict[str, object]]:
    if sanity_profile == "seamgrim":
        return []
    return [
        {
            "step": "ci_pack_golden_lang_consistency_selftest",
            "ok": True,
            "returncode": 0,
            "cmd": ["python", "tests/run_pack_golden_lang_consistency_selftest.py"],
        }
    ]


def build_index_case(root: Path, case_name: str, sanity_profile: str = "full") -> Path:
    case_dir = root / _SHARED_CASE_DIR_NAME
    case_dir.mkdir(parents=True, exist_ok=True)

    summary = case_dir / "ci_gate_summary.txt"
    summary_line = case_dir / "ci_gate_summary_line.txt"
    aggregate = case_dir / "ci_aggregate_report.detjson"
    final_status_line = case_dir / "ci_gate_final_status_line.txt"
    final_status_parse = case_dir / "ci_gate_final_status_line_parse.detjson"
    result = case_dir / "ci_gate_result.detjson"
    badge = case_dir / "ci_gate_badge.detjson"
    brief = case_dir / "ci_fail_brief.txt"
    triage = case_dir / "ci_fail_triage.detjson"
    sanity = case_dir / "ci_sanity_gate.detjson"
    sync = case_dir / "ci_sync_readiness.detjson"
    parity = case_dir / "seamgrim_wasm_cli_diag_parity_report.detjson"
    fixed64_threeway_inputs = case_dir / "fixed64_threeway_inputs.detjson"
    profile_matrix_selftest = case_dir / "ci_profile_matrix_gate_selftest.detjson"
    index = case_dir / "ci_gate_report_index.detjson"
    aggregate_summary_rows = {}
    age4_proof_ok = "1"
    age4_proof_failed_criteria = "0"
    age4_proof_failed_preview = "-"
    age4_proof_summary_hash = "sha256:age4-proof-report-index-selftest"
    age5_policy_origin_trace = build_age5_combined_heavy_policy_origin_trace()
    age5_policy_origin_trace_text = build_age5_combined_heavy_policy_origin_trace_text(
        age5_policy_origin_trace
    )
    age5_policy_age4_proof_snapshot = build_age4_proof_snapshot()
    age5_policy_age4_proof_snapshot_text = build_age4_proof_snapshot_text(
        age5_policy_age4_proof_snapshot
    )
    age5_policy_age4_proof_source_snapshot = build_age4_proof_source_snapshot_fields(
        top_snapshot=age5_policy_age4_proof_snapshot
    )

    for profile_name in PROFILE_MATRIX_SELFTEST_PROFILES:
        expected_contract = expected_profile_matrix_aggregate_summary_contract(profile_name)
        aggregate_summary_rows[profile_name] = {
            "expected_present": bool(expected_contract["expected_present"]),
            "present": True,
            "status": str(expected_contract["status"]),
            "reason": "ok",
            "expected_profile": str(expected_contract["expected_profile"]),
            "expected_sync_profile": str(expected_contract["expected_sync_profile"]),
            "profile": str(expected_contract["expected_profile"]),
            "sync_profile": str(expected_contract["expected_sync_profile"]),
            "expected_values": dict(expected_contract["values"]),
            "values": dict(expected_contract["values"]),
            "missing_keys": [],
            "mismatched_keys": [],
            "profile_ok": True,
            "sync_profile_ok": True,
            "values_ok": True,
            "gate_marker_expected": bool(expected_contract["gate_marker_expected"]),
            "gate_marker_present": bool(expected_contract["gate_marker_expected"]),
            "gate_marker_ok": True,
            "ok": bool(expected_contract["ok"]),
        }

    write_text(
        summary,
        "\n".join(
            [
                "[ci-gate-summary] PASS",
                "[ci-gate-summary] failed_steps=(none)",
                "[ci-gate-summary] seamgrim_group_id_summary_status=ok",
                "[ci-gate-summary] age5_close_digest_selftest_ok=1",
                *[f"[ci-gate-summary] {key}={value}" for key, value in AGE5_CHILD_SUMMARY_DEFAULTS.items()],
                *[
                    f"[ci-gate-summary] {key}={value}"
                    for key, value in AGE5_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT.items()
                ],
                *[
                    f"[ci-gate-summary] {key}={value}"
                    for key, value in build_age5_combined_heavy_sanity_contract_fields().items()
                ],
                *[
                    f"[ci-gate-summary] {key}={value}"
                    for key, value in build_age5_combined_heavy_sync_contract_fields().items()
                ],
            ]
        ),
    )
    write_text(
        summary_line,
        "status=pass reason=ok failed_steps=0 "
        "age5_w107_active=54 age5_w107_inactive=1 age5_w107_index_codes=34 "
        "age5_w107_current_probe=- age5_w107_last_completed_probe=validate_pack_pointers "
        "age5_w107_progress=1 "
        "age5_w107_contract_completed=8 age5_w107_contract_total=8 "
        "age5_w107_contract_checks_text=golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index "
        "age5_w107_contract_current_probe=- age5_w107_contract_last_completed_probe=report_index "
        "age5_w107_contract_progress=1 "
        "age5_age1_immediate_proof_operation_contract_completed=5 age5_age1_immediate_proof_operation_contract_total=5 "
        "age5_age1_immediate_proof_operation_contract_checks_text=operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family "
        "age5_age1_immediate_proof_operation_contract_current_probe=- age5_age1_immediate_proof_operation_contract_last_completed_probe=proof_operation_family "
        "age5_age1_immediate_proof_operation_contract_progress=1 "
        "age5_proof_certificate_v1_consumer_contract_completed=5 age5_proof_certificate_v1_consumer_contract_total=5 "
        "age5_proof_certificate_v1_consumer_contract_checks_text=signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract "
        "age5_proof_certificate_v1_consumer_contract_current_probe=- age5_proof_certificate_v1_consumer_contract_last_completed_probe=signed_contract "
        "age5_proof_certificate_v1_verify_report_digest_contract_completed=1 age5_proof_certificate_v1_verify_report_digest_contract_total=1 "
        "age5_proof_certificate_v1_verify_report_digest_contract_checks_text=verify_report_digest_contract "
        "age5_proof_certificate_v1_verify_report_digest_contract_current_probe=- age5_proof_certificate_v1_verify_report_digest_contract_last_completed_probe=readme_and_field_contract "
        "age5_proof_certificate_v1_verify_report_digest_contract_progress=1 "
        "age5_proof_certificate_v1_family_contract_completed=4 age5_proof_certificate_v1_family_contract_total=4 "
        "age5_proof_certificate_v1_family_contract_checks_text=signed_contract,consumer_contract,promotion,family "
        "age5_proof_certificate_v1_family_contract_current_probe=- age5_proof_certificate_v1_family_contract_last_completed_probe=family "
        "age5_proof_certificate_v1_family_contract_progress=1 "
        "age5_proof_certificate_family_contract_completed=3 age5_proof_certificate_family_contract_total=3 "
        "age5_proof_certificate_family_contract_checks_text=artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family "
        "age5_proof_certificate_family_contract_current_probe=- age5_proof_certificate_family_contract_last_completed_probe=proof_certificate_family "
        "age5_proof_certificate_family_contract_progress=1 "
        "age5_proof_family_contract_completed=3 age5_proof_family_contract_total=3 "
        "age5_proof_family_contract_checks_text=proof_operation_family,proof_certificate_family,proof_family "
        "age5_proof_family_contract_current_probe=- age5_proof_family_contract_last_completed_probe=proof_family "
        "age5_proof_family_contract_progress=1 "
        "age5_proof_family_transport_contract_completed=9 age5_proof_family_transport_contract_total=9 "
        "age5_proof_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
        "age5_proof_family_transport_contract_current_probe=- age5_proof_family_transport_contract_last_completed_probe=report_index "
        "age5_proof_family_transport_contract_progress=1 "
        "age5_lang_surface_family_contract_completed=4 age5_lang_surface_family_contract_total=4 "
        "age5_lang_surface_family_contract_checks_text=proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family "
        "age5_lang_surface_family_contract_current_probe=- age5_lang_surface_family_contract_last_completed_probe=lang_surface_family "
        "age5_lang_surface_family_contract_progress=1 "
        "age5_lang_runtime_family_contract_completed=5 age5_lang_runtime_family_contract_total=5 "
        "age5_lang_runtime_family_contract_checks_text=lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family "
        "age5_lang_runtime_family_contract_current_probe=- age5_lang_runtime_family_contract_last_completed_probe=lang_runtime_family "
        "age5_lang_runtime_family_contract_progress=1 "
        "age5_gate0_family_contract_completed=5 age5_gate0_family_contract_total=5 "
        "age5_gate0_family_contract_checks_text=gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family "
        "age5_gate0_family_contract_current_probe=- age5_gate0_family_contract_last_completed_probe=gate0_family "
        "age5_gate0_family_contract_progress=1 "
        "age5_gate0_family_transport_contract_completed=9 age5_gate0_family_transport_contract_total=9 "
        "age5_gate0_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
        "age5_gate0_family_transport_contract_current_probe=- age5_gate0_family_transport_contract_last_completed_probe=report_index "
        "age5_gate0_family_transport_contract_progress=1 "
        "age5_gate0_transport_family_contract_completed=4 age5_gate0_transport_family_contract_total=4 "
        "age5_gate0_transport_family_contract_checks_text=lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family "
        "age5_gate0_transport_family_contract_current_probe=- age5_gate0_transport_family_contract_last_completed_probe=gate0_transport_family "
        "age5_gate0_transport_family_contract_progress=1 "
        "age5_lang_runtime_family_transport_contract_completed=9 age5_lang_runtime_family_transport_contract_total=9 "
        "age5_lang_runtime_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
        "age5_lang_runtime_family_transport_contract_current_probe=- age5_lang_runtime_family_transport_contract_last_completed_probe=report_index "
        "age5_lang_runtime_family_transport_contract_progress=1 "
        "age5_gate0_runtime_family_transport_contract_completed=1 age5_gate0_runtime_family_transport_contract_total=1 "
        "age5_gate0_runtime_family_transport_contract_checks_text=family_contract "
        "age5_gate0_runtime_family_transport_contract_current_probe=- age5_gate0_runtime_family_transport_contract_last_completed_probe=family_contract "
        "age5_gate0_runtime_family_transport_contract_progress=1 "
        "age5_lang_surface_family_transport_contract_completed=9 age5_lang_surface_family_transport_contract_total=9 "
        "age5_lang_surface_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
        "age5_lang_surface_family_transport_contract_current_probe=- age5_lang_surface_family_transport_contract_last_completed_probe=report_index "
        "age5_lang_surface_family_transport_contract_progress=1 "
        "age5_proof_certificate_family_transport_contract_completed=9 age5_proof_certificate_family_transport_contract_total=9 "
        "age5_proof_certificate_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
        "age5_proof_certificate_family_transport_contract_current_probe=- age5_proof_certificate_family_transport_contract_last_completed_probe=report_index "
        "age5_proof_certificate_family_transport_contract_progress=1 "
        "age5_bogae_alias_family_contract_completed=3 age5_bogae_alias_family_contract_total=3 "
        "age5_bogae_alias_family_contract_checks_text=shape_alias_contract,alias_family,alias_viewer_family "
        "age5_bogae_alias_family_contract_current_probe=- age5_bogae_alias_family_contract_last_completed_probe=alias_viewer_family "
        "age5_bogae_alias_family_contract_progress=1 "
        "age5_bogae_alias_family_transport_contract_completed=9 age5_bogae_alias_family_transport_contract_total=9 "
        "age5_bogae_alias_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
        "age5_bogae_alias_family_transport_contract_current_probe=- age5_bogae_alias_family_transport_contract_last_completed_probe=report_index "
        "age5_bogae_alias_family_transport_contract_progress=1 "
        "age5_proof_certificate_v1_consumer_contract_progress=1",
    )
    write_text(final_status_line, "schema=ddn.ci.final_status.v1 status=pass reason=ok failed_steps=0 aggregate_status=pass overall_ok=1")
    write_json(
        final_status_parse,
        {
            "schema": "ddn.ci.gate_final_status_line_parse.v1",
            "status_line_path": str(final_status_line),
            "parsed": {
                "status": "pass",
                "reason": "ok",
                "failed_steps": "0",
                "aggregate_status": "pass",
                "overall_ok": "1",
                AGE4_PROOF_OK_KEY: age4_proof_ok,
                AGE4_PROOF_FAILED_CRITERIA_KEY: age4_proof_failed_criteria,
                AGE4_PROOF_FAILED_PREVIEW_KEY: age4_proof_failed_preview,
                **AGE5_W107_PROGRESS_FIXTURE,
                **AGE5_W107_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_FIXTURE,
                **AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            },
        },
    )
    write_json(
        result,
        {
            "schema": "ddn.ci.gate_result.v1",
            "ok": True,
            "status": "pass",
            "reason": "ok",
            "overall_ok": True,
            "aggregate_status": "pass",
            "failed_steps": 0,
            **AGE5_W107_PROGRESS_FIXTURE,
            **AGE5_W107_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            "summary_line_path": str(summary_line),
            "summary_line": summary_line.read_text(encoding="utf-8").strip(),
            "final_status_parse_path": str(final_status_parse),
            "gate_index_path": str(index),
        },
    )
    write_json(
        badge,
        {
            "schema": "ddn.ci.gate_badge.v1",
            "status": "pass",
            "ok": True,
            "label": "ci:pass",
            "result_path": str(result),
        },
    )
    write_text(
        brief,
        " ".join(
            [
                "status=pass",
                "reason=ok",
                "failed_steps_count=0",
                "failed_steps=-",
                f"{AGE4_PROOF_OK_KEY}={age4_proof_ok}",
                f"{AGE4_PROOF_FAILED_CRITERIA_KEY}={age4_proof_failed_criteria}",
                f"{AGE4_PROOF_FAILED_PREVIEW_KEY}={age4_proof_failed_preview}",
                f"{AGE4_PROOF_SUMMARY_HASH_KEY}={age4_proof_summary_hash}",
                "age5_w107_active=54",
                "age5_w107_inactive=1",
                "age5_w107_index_codes=34",
                "age5_w107_current_probe=-",
                "age5_w107_last_completed_probe=validate_pack_pointers",
                "age5_w107_progress=1",
                "age5_w107_contract_completed=8",
                "age5_w107_contract_total=8",
                "age5_w107_contract_checks_text=golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index",
                "age5_w107_contract_current_probe=-",
                "age5_w107_contract_last_completed_probe=report_index",
                "age5_w107_contract_progress=1",
                "age5_age1_immediate_proof_operation_contract_completed=5",
                "age5_age1_immediate_proof_operation_contract_total=5",
                "age5_age1_immediate_proof_operation_contract_checks_text=operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family",
                "age5_age1_immediate_proof_operation_contract_current_probe=-",
                "age5_age1_immediate_proof_operation_contract_last_completed_probe=proof_operation_family",
                "age5_age1_immediate_proof_operation_contract_progress=1",
                "age5_proof_certificate_v1_consumer_contract_completed=5",
                "age5_proof_certificate_v1_consumer_contract_total=5",
                "age5_proof_certificate_v1_consumer_contract_checks_text=signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract",
                "age5_proof_certificate_v1_consumer_contract_current_probe=-",
                "age5_proof_certificate_v1_consumer_contract_last_completed_probe=signed_contract",
                "age5_proof_certificate_v1_consumer_contract_progress=1",
                "age5_proof_certificate_v1_verify_report_digest_contract_completed=1",
                "age5_proof_certificate_v1_verify_report_digest_contract_total=1",
                "age5_proof_certificate_v1_verify_report_digest_contract_checks_text=verify_report_digest_contract",
                "age5_proof_certificate_v1_verify_report_digest_contract_current_probe=-",
                "age5_proof_certificate_v1_verify_report_digest_contract_last_completed_probe=readme_and_field_contract",
                "age5_proof_certificate_v1_verify_report_digest_contract_progress=1",
                "age5_proof_certificate_v1_family_contract_completed=4",
                "age5_proof_certificate_v1_family_contract_total=4",
                "age5_proof_certificate_v1_family_contract_checks_text=signed_contract,consumer_contract,promotion,family",
                "age5_proof_certificate_v1_family_contract_current_probe=-",
                "age5_proof_certificate_v1_family_contract_last_completed_probe=family",
                "age5_proof_certificate_v1_family_contract_progress=1",
                "age5_proof_certificate_family_contract_completed=3",
                "age5_proof_certificate_family_contract_total=3",
                "age5_proof_certificate_family_contract_checks_text=artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family",
                "age5_proof_certificate_family_contract_current_probe=-",
                "age5_proof_certificate_family_contract_last_completed_probe=proof_certificate_family",
                "age5_proof_certificate_family_contract_progress=1",
                "age5_proof_family_contract_completed=3",
                "age5_proof_family_contract_total=3",
                "age5_proof_family_contract_checks_text=proof_operation_family,proof_certificate_family,proof_family",
                "age5_proof_family_contract_current_probe=-",
                "age5_proof_family_contract_last_completed_probe=proof_family",
                "age5_proof_family_contract_progress=1",
                "age5_proof_family_transport_contract_completed=9",
                "age5_proof_family_transport_contract_total=9",
                "age5_proof_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
                "age5_proof_family_transport_contract_current_probe=-",
                "age5_proof_family_transport_contract_last_completed_probe=report_index",
                "age5_proof_family_transport_contract_progress=1",
                "age5_lang_surface_family_contract_completed=4",
                "age5_lang_surface_family_contract_total=4",
                "age5_lang_surface_family_contract_checks_text=proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family",
                "age5_lang_surface_family_contract_current_probe=-",
                "age5_lang_surface_family_contract_last_completed_probe=lang_surface_family",
                "age5_lang_surface_family_contract_progress=1",
                "age5_lang_runtime_family_contract_completed=5",
                "age5_lang_runtime_family_contract_total=5",
                "age5_lang_runtime_family_contract_checks_text=lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family",
                "age5_lang_runtime_family_contract_current_probe=-",
                "age5_lang_runtime_family_contract_last_completed_probe=lang_runtime_family",
                "age5_lang_runtime_family_contract_progress=1",
                "age5_gate0_family_contract_completed=5",
                "age5_gate0_family_contract_total=5",
                "age5_gate0_family_contract_checks_text=gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family",
                "age5_gate0_family_contract_current_probe=-",
                "age5_gate0_family_contract_last_completed_probe=gate0_family",
                "age5_gate0_family_contract_progress=1",
                "age5_gate0_family_transport_contract_completed=9",
                "age5_gate0_family_transport_contract_total=9",
                "age5_gate0_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
                "age5_gate0_family_transport_contract_current_probe=-",
                "age5_gate0_family_transport_contract_last_completed_probe=report_index",
                "age5_gate0_family_transport_contract_progress=1",
                "age5_gate0_surface_family_transport_contract_completed=9",
                "age5_gate0_surface_family_transport_contract_total=9",
                "age5_gate0_surface_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
                "age5_gate0_surface_family_transport_contract_current_probe=-",
                "age5_gate0_surface_family_transport_contract_last_completed_probe=report_index",
                "age5_gate0_surface_family_transport_contract_progress=1",
                "age5_gate0_transport_family_contract_completed=4",
                "age5_gate0_transport_family_contract_total=4",
                "age5_gate0_transport_family_contract_checks_text=lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family",
                "age5_gate0_transport_family_contract_current_probe=-",
                "age5_gate0_transport_family_contract_last_completed_probe=gate0_transport_family",
                "age5_gate0_transport_family_contract_progress=1",
                "age5_lang_runtime_family_transport_contract_completed=9",
                "age5_lang_runtime_family_transport_contract_total=9",
                "age5_lang_runtime_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
                "age5_lang_runtime_family_transport_contract_current_probe=-",
                "age5_lang_runtime_family_transport_contract_last_completed_probe=report_index",
                "age5_lang_runtime_family_transport_contract_progress=1",
                "age5_gate0_runtime_family_transport_contract_completed=1",
                "age5_gate0_runtime_family_transport_contract_total=1",
                "age5_gate0_runtime_family_transport_contract_checks_text=family_contract",
                "age5_gate0_runtime_family_transport_contract_current_probe=-",
                "age5_gate0_runtime_family_transport_contract_last_completed_probe=family_contract",
                "age5_gate0_runtime_family_transport_contract_progress=1",
                "age5_lang_surface_family_transport_contract_completed=9",
                "age5_lang_surface_family_transport_contract_total=9",
                "age5_lang_surface_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
                "age5_lang_surface_family_transport_contract_current_probe=-",
                "age5_lang_surface_family_transport_contract_last_completed_probe=report_index",
                "age5_lang_surface_family_transport_contract_progress=1",
                "age5_proof_certificate_family_transport_contract_completed=9",
                "age5_proof_certificate_family_transport_contract_total=9",
                "age5_proof_certificate_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
                "age5_proof_certificate_family_transport_contract_current_probe=-",
                "age5_proof_certificate_family_transport_contract_last_completed_probe=report_index",
                "age5_proof_certificate_family_transport_contract_progress=1",
                "age5_bogae_alias_family_contract_completed=3",
                "age5_bogae_alias_family_contract_total=3",
                "age5_bogae_alias_family_contract_checks_text=shape_alias_contract,alias_family,alias_viewer_family",
                "age5_bogae_alias_family_contract_current_probe=-",
                "age5_bogae_alias_family_contract_last_completed_probe=alias_viewer_family",
                "age5_bogae_alias_family_contract_progress=1",
                "age5_bogae_alias_family_transport_contract_completed=9",
                "age5_bogae_alias_family_transport_contract_total=9",
                "age5_bogae_alias_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
                "age5_bogae_alias_family_transport_contract_current_probe=-",
                "age5_bogae_alias_family_transport_contract_last_completed_probe=report_index",
                "age5_bogae_alias_family_transport_contract_progress=1",
                "age5_close_digest_selftest_ok=1",
                AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT,
                f"{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}={AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT}",
                f"{AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}={AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT}",
                f'{AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY}={{"age5_close_digest_selftest_ok":"0"}}',
                f"{AGE5_POLICY_REPORT_PATH_KEY}=-",
                f"{AGE5_POLICY_REPORT_EXISTS_KEY}=0",
                f"{AGE5_POLICY_TEXT_PATH_KEY}=-",
                f"{AGE5_POLICY_TEXT_EXISTS_KEY}=0",
                f"{AGE5_POLICY_SUMMARY_PATH_KEY}=-",
                f"{AGE5_POLICY_SUMMARY_EXISTS_KEY}=0",
                f"{AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY}={AGE4_PROOF_SNAPSHOT_FIELDS_TEXT}",
                f"{AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY}={age5_policy_age4_proof_snapshot_text}",
                f"{AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY}={AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT}",
                f"{AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY}={age5_policy_age4_proof_source_snapshot['age4_proof_gate_result_snapshot_present']}",
                f"{AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY}={age5_policy_age4_proof_source_snapshot['age4_proof_gate_result_snapshot_parity']}",
                f"{AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY}={age5_policy_age4_proof_source_snapshot['age4_proof_final_status_parse_snapshot_present']}",
                f"{AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY}={age5_policy_age4_proof_source_snapshot['age4_proof_final_status_parse_snapshot_parity']}",
                f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}=-",
                f"{AGE5_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}=-",
                f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}=-",
                f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY}=-",
                f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}=ok",
                f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}=1",
                f"{AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY}={age5_policy_origin_trace_text}",
                f"{AGE5_POLICY_ORIGIN_TRACE_KEY}="
                + json.dumps(age5_policy_origin_trace, ensure_ascii=False, sort_keys=True, separators=(",", ":")),
                *[f"{key}={value}" for key, value in AGE5_CHILD_SUMMARY_DEFAULTS.items()],
                *[
                    f"{key}={value}"
                    for key, value in AGE5_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT.items()
                ],
            ]
        ),
    )
    profile_matrix_report_payload = {
        "schema": "ddn.ci.profile_matrix_gate_selftest.v1",
        "generated_at_utc": "2026-03-17T00:00:00+00:00",
        "status": "pass",
        "ok": True,
        "selected_real_profiles": ["core_lang", "full", "seamgrim"],
        "skipped_real_profiles": [],
        "step_timeout_defaults_text": PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
        "step_timeout_defaults_sec": dict(PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC),
        "step_timeout_env_keys": dict(PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS),
        "quick": False,
        "dry": False,
        "total_elapsed_ms": 123,
        "aggregate_summary_sanity_ok": True,
        "aggregate_summary_sanity_checked_profiles": list(PROFILE_MATRIX_SELFTEST_PROFILES),
        "aggregate_summary_sanity_failed_profiles": [],
        "aggregate_summary_sanity_skipped_profiles": [],
        "aggregate_summary_sanity_by_profile": aggregate_summary_rows,
        "real_profiles": {
            "core_lang": {
                "selected": True,
                "skipped": False,
                "status": "pass",
                "ok": True,
                "total_elapsed_ms": 11,
                "step_elapsed_ms": 10,
            },
            "full": {
                "selected": True,
                "skipped": False,
                "status": "pass",
                "ok": True,
                "total_elapsed_ms": 22,
                "step_elapsed_ms": 20,
            },
            "seamgrim": {
                "selected": True,
                "skipped": False,
                "status": "pass",
                "ok": True,
                "total_elapsed_ms": 33,
                "step_elapsed_ms": 30,
            },
        },
    }
    profile_matrix_snapshot = build_profile_matrix_snapshot_from_doc(
        profile_matrix_report_payload,
        report_path=str(profile_matrix_selftest),
    )
    if not isinstance(profile_matrix_snapshot, dict):
        raise RuntimeError("profile_matrix snapshot build failed")
    profile_matrix_triage_payload = build_profile_matrix_triage_payload_from_snapshot(
        profile_matrix_snapshot
    )
    write_json(
        triage,
        {
            "schema": "ddn.ci.fail_triage.v1",
            "status": "pass",
            "reason": "ok",
            "failed_steps_count": 0,
            "failed_steps": [],
            "failed_step_detail_rows_count": 0,
            "failed_step_logs_rows_count": 0,
            "failed_step_detail_order": [],
            "failed_step_logs_order": [],
            "summary_report_path_hint_norm": str(summary),
            AGE4_PROOF_OK_KEY: int(age4_proof_ok),
            AGE4_PROOF_FAILED_CRITERIA_KEY: int(age4_proof_failed_criteria),
            AGE4_PROOF_FAILED_PREVIEW_KEY: age4_proof_failed_preview,
            AGE4_PROOF_SUMMARY_HASH_KEY: age4_proof_summary_hash,
            **AGE5_W107_PROGRESS_FIXTURE,
            **AGE5_W107_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            "age5_close_digest_selftest_ok": "1",
            AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
            AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
            AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY: AGE5_DIGEST_SELFTEST_DEFAULT_FIELD,
            AGE5_POLICY_REPORT_PATH_KEY: "-",
            AGE5_POLICY_REPORT_EXISTS_KEY: 0,
            AGE5_POLICY_TEXT_PATH_KEY: "-",
            AGE5_POLICY_TEXT_EXISTS_KEY: 0,
            AGE5_POLICY_SUMMARY_PATH_KEY: "-",
            AGE5_POLICY_SUMMARY_EXISTS_KEY: 0,
            AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY: AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
            AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY: age5_policy_age4_proof_snapshot_text,
            AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY: AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
            AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY: age5_policy_age4_proof_source_snapshot["age4_proof_gate_result_snapshot_present"],
            AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY: age5_policy_age4_proof_source_snapshot["age4_proof_gate_result_snapshot_parity"],
            AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY: age5_policy_age4_proof_source_snapshot["age4_proof_final_status_parse_snapshot_present"],
            AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY: age5_policy_age4_proof_source_snapshot["age4_proof_final_status_parse_snapshot_parity"],
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY: "-",
            AGE5_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY: "-",
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY: "-",
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY: "-",
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY: "ok",
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY: 1,
            AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY: age5_policy_origin_trace_text,
            AGE5_POLICY_ORIGIN_TRACE_KEY: dict(age5_policy_origin_trace),
            "combined_digest_selftest_default_field": AGE5_DIGEST_SELFTEST_DEFAULT_FIELD,
            "profile_matrix_selftest": dict(profile_matrix_triage_payload),
            **AGE5_CHILD_SUMMARY_DEFAULTS,
            **AGE5_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT,
            "artifacts": {
                "summary": {"path": str(summary), "path_norm": str(summary), "exists": True},
                "ci_gate_result_json": {"path": str(result), "path_norm": str(result), "exists": True},
                "ci_gate_badge_json": {"path": str(badge), "path_norm": str(badge), "exists": True},
                "ci_fail_brief_txt": {"path": str(brief), "path_norm": str(brief), "exists": True},
                "ci_fail_triage_json": {"path": str(triage), "path_norm": str(triage), "exists": True},
            },
        },
    )
    write_json(
        sanity,
        {
            "schema": "ddn.ci.sanity_gate.v1",
            "status": "pass",
            "code": "OK",
            "step": "all",
            "profile": sanity_profile,
            "ci_sanity_pipeline_emit_flags_ok": "1" if sanity_profile in {"full", "core_lang"} else "na",
            "ci_sanity_pipeline_emit_flags_selftest_ok": "1" if sanity_profile in {"full", "core_lang"} else "na",
            "ci_sanity_age5_combined_heavy_policy_selftest_ok": "1",
            "ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok": "1",
            **build_age5_combined_heavy_sanity_contract_fields(),
            "steps": build_sanity_steps(sanity_profile),
        },
    )
    write_json(
        sync,
        {
            "schema": "ddn.ci.sync_readiness.v1",
            "status": "pass",
            "ok": True,
            "code": "OK",
            "step": "all",
            "sanity_profile": sanity_profile,
            "ci_sanity_pipeline_emit_flags_ok": "1" if sanity_profile in {"full", "core_lang"} else "na",
            "ci_sanity_pipeline_emit_flags_selftest_ok": "1" if sanity_profile in {"full", "core_lang"} else "na",
            "ci_sanity_age5_combined_heavy_policy_selftest_ok": "1",
            "ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok": "1",
            **build_age5_combined_heavy_sanity_contract_fields(),
            **build_age5_combined_heavy_sync_contract_fields(),
            "steps": [],
        },
    )
    write_json(
        parity,
        {"schema": "ddn.seamgrim.wasm_cli_diag_parity.v1", "status": "pass", "ok": True, "code": "OK", "step": "all", "steps": []},
    )
    write_json(
        fixed64_threeway_inputs,
        {
            "schema": "ddn.fixed64.threeway_inputs.v1",
            "status": "staged",
            "ok": True,
            "reason": "found",
        },
    )
    write_json(
        profile_matrix_selftest,
        profile_matrix_report_payload,
    )

    write_json(
        aggregate,
        {
            "schema": "ddn.ci.aggregate_report.v1",
            "overall_ok": True,
            "age4": {
                "proof_artifact_ok": True,
                "proof_artifact_failed_criteria": [],
                "proof_artifact_failed_preview": age4_proof_failed_preview,
                "proof_artifact_summary_hash": age4_proof_summary_hash,
            },
            "age5": {
                "age5_close_digest_selftest_ok": "1",
                AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
                AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY: AGE5_DIGEST_SELFTEST_DEFAULT_FIELD,
                AGE5_POLICY_REPORT_PATH_KEY: "-",
                AGE5_POLICY_REPORT_EXISTS_KEY: 0,
                AGE5_POLICY_TEXT_PATH_KEY: "-",
                AGE5_POLICY_TEXT_EXISTS_KEY: 0,
                AGE5_POLICY_SUMMARY_PATH_KEY: "-",
                AGE5_POLICY_SUMMARY_EXISTS_KEY: 0,
                AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY: AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
                AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY: age5_policy_age4_proof_snapshot_text,
                AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY: AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
                AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY: age5_policy_age4_proof_source_snapshot["age4_proof_gate_result_snapshot_present"],
                AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY: age5_policy_age4_proof_source_snapshot["age4_proof_gate_result_snapshot_parity"],
                AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY: age5_policy_age4_proof_source_snapshot["age4_proof_final_status_parse_snapshot_present"],
                AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY: age5_policy_age4_proof_source_snapshot["age4_proof_final_status_parse_snapshot_parity"],
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY: "-",
                AGE5_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY: "-",
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY: "-",
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY: "-",
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY: "ok",
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY: 1,
                AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY: age5_policy_origin_trace_text,
                AGE5_POLICY_ORIGIN_TRACE_KEY: dict(age5_policy_origin_trace),
            },
        },
    )
    write_json(
        index,
        {
            "schema": "ddn.ci.aggregate_gate.index.v1",
            "generated_at_utc": "2026-03-04T00:00:00+00:00",
            "report_prefix": "",
            "report_prefix_source": "",
            "report_dir": str(case_dir),
            "ci_sanity_profile": sanity_profile,
            "step_log_dir": "",
            "step_log_failed_only": False,
            "overall_ok": True,
            "reports": {
                "aggregate": str(aggregate),
                "summary": str(summary),
                "summary_line": str(summary_line),
                "final_status_parse_json": str(final_status_parse),
                "ci_gate_result_json": str(result),
                "ci_gate_badge_json": str(badge),
                "ci_fail_brief_txt": str(brief),
                "ci_fail_triage_json": str(triage),
                "ci_profile_matrix_gate_selftest": str(profile_matrix_selftest),
                "ci_sanity_gate": str(sanity),
                "ci_sync_readiness": str(sync),
                "seamgrim_wasm_cli_diag_parity": str(parity),
                "fixed64_threeway_inputs": str(fixed64_threeway_inputs),
            },
            "steps": [
                {
                    "name": "ci_profile_split_contract_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_ci_profile_split_contract_check.py"],
                },
                {
                    "name": "age5_close_digest_selftest",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_age5_close_digest_selftest.py"],
                },
                {
                    "name": "ci_profile_matrix_gate_selftest",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_ci_profile_matrix_gate_selftest.py"],
                },
                {"name": "ci_sanity_gate", "returncode": 0, "ok": True, "cmd": ["python", "tests/run_ci_sanity_gate.py"]},
                {
                    "name": "ci_sync_readiness_report_generate",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_ci_sync_readiness_check.py"],
                },
                {
                    "name": "ci_sync_readiness_report_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_ci_sync_readiness_report_check.py"],
                },
                {
                    "name": "ci_emit_artifacts_required_post_summary_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_ci_emit_artifacts_check.py"],
                },
                {
                    "name": "seamgrim_wasm_cli_diag_parity_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_seamgrim_wasm_cli_diag_parity_check.py"],
                },
                {
                    "name": "seamgrim_ci_gate_seed_meta_step_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_seamgrim_ci_gate_seed_meta_step_check.py"],
                },
                {
                    "name": "seamgrim_ci_gate_runtime5_passthrough_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_seamgrim_ci_gate_runtime5_passthrough_check.py"],
                },
                {
                    "name": "seamgrim_ci_gate_guideblock_step_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_seamgrim_ci_gate_guideblock_step_check.py"],
                },
                {
                    "name": "seamgrim_ci_gate_lesson_warning_step_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_seamgrim_ci_gate_lesson_warning_step_check.py"],
                },
                {
                    "name": "seamgrim_ci_gate_stateful_preview_step_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_seamgrim_ci_gate_stateful_preview_step_check.py"],
                },
                {
                    "name": "seamgrim_ci_gate_wasm_web_smoke_step_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check.py"],
                },
                {
                    "name": "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check_selftest.py"],
                },
                {
                    "name": "ci_fail_and_exit_contract_selftest",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_ci_fail_and_exit_contract_selftest.py"],
                },
                {
                    "name": "ci_gate_report_index_selftest",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_ci_gate_report_index_check_selftest.py"],
                },
                {
                    "name": "ci_gate_report_index_diagnostics_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_ci_gate_report_index_diagnostics_check.py"],
                },
                {
                    "name": "ci_gate_report_index_latest_smoke_check",
                    "returncode": 0,
                    "ok": True,
                    "cmd": ["python", "tests/run_ci_gate_report_index_latest_smoke_check.py"],
                },
            ],
        },
    )
    return index


def configure_case_as_fail(
    index_path: Path,
    *,
    reason: str = "forced_fail",
    failed_step_ids: tuple[str, str] = (
        "ci_emit_artifacts_required_post_summary_check",
        "ci_gate_report_index_diagnostics_check",
    ),
) -> None:
    index_doc = json.loads(index_path.read_text(encoding="utf-8"))
    reports = index_doc.get("reports", {})
    if not isinstance(reports, dict):
        raise RuntimeError("index.reports missing")
    summary_path = Path(str(reports["summary"]))
    summary_line_path = Path(str(reports["summary_line"]))
    final_status_parse_path = Path(str(reports["final_status_parse_json"]))
    result_path = Path(str(reports["ci_gate_result_json"]))
    badge_path = Path(str(reports["ci_gate_badge_json"]))
    brief_path = Path(str(reports["ci_fail_brief_txt"]))
    triage_path = Path(str(reports["ci_fail_triage_json"]))

    steps = index_doc.get("steps")
    if not isinstance(steps, list):
        raise RuntimeError("index.steps missing")
    step_rows: dict[str, dict] = {}
    for row in steps:
        if isinstance(row, dict):
            step_name = str(row.get("name", "")).strip()
            if step_name:
                step_rows[step_name] = row

    missing_steps = [step for step in failed_step_ids if step not in step_rows]
    if missing_steps:
        raise RuntimeError(f"missing step rows: {','.join(missing_steps)}")

    step_log_dir = index_path.parent / "step_logs"
    step_log_dir.mkdir(parents=True, exist_ok=True)
    failed_logs: dict[str, dict[str, str]] = {}
    for step_name in failed_step_ids:
        row = step_rows[step_name]
        stdout_path = step_log_dir / f"{index_path.stem}.{step_name}.stdout.txt"
        stderr_path = step_log_dir / f"{index_path.stem}.{step_name}.stderr.txt"
        write_text(stdout_path, f"[selftest] {step_name} stdout")
        write_text(stderr_path, f"[selftest] {step_name} stderr")
        row["ok"] = False
        row["returncode"] = 1
        row["stdout_log_path"] = str(stdout_path)
        row["stderr_log_path"] = str(stderr_path)
        failed_logs[step_name] = {"stdout": str(stdout_path), "stderr": str(stderr_path)}

    index_doc["overall_ok"] = False
    index_doc["step_log_dir"] = str(step_log_dir)
    index_doc["step_log_failed_only"] = True
    write_json(index_path, index_doc)

    summary_lines = [line.strip() for line in summary_path.read_text(encoding="utf-8").splitlines() if line.strip()]
    updated_summary_lines: list[str] = []
    failed_steps_value = ",".join(failed_step_ids)
    for line in summary_lines:
        if line == "[ci-gate-summary] PASS":
            updated_summary_lines.append("[ci-gate-summary] FAIL")
            continue
        if line.startswith("[ci-gate-summary] failed_steps="):
            updated_summary_lines.append(f"[ci-gate-summary] failed_steps={failed_steps_value}")
            for step_name in failed_step_ids:
                row = step_rows[step_name]
                cmd = row.get("cmd")
                cmd_text = " ".join(str(token) for token in cmd) if isinstance(cmd, list) else "-"
                logs_row = failed_logs[step_name]
                updated_summary_lines.append(
                    f"[ci-gate-summary] failed_step_detail={step_name} rc=1 cmd={cmd_text}"
                )
                updated_summary_lines.append(
                    f"[ci-gate-summary] failed_step_logs={step_name} "
                    f"stdout={logs_row['stdout']} stderr={logs_row['stderr']}"
                )
            continue
        updated_summary_lines.append(line)
    write_text(summary_path, "\n".join(updated_summary_lines))

    summary_line_text = summary_line_path.read_text(encoding="utf-8")
    summary_line_text = summary_line_text.replace("status=pass", "status=fail", 1)
    summary_line_text = summary_line_text.replace("reason=ok", f"reason={reason}", 1)
    summary_line_text = summary_line_text.replace("failed_steps=0", f"failed_steps={len(failed_step_ids)}", 1)
    write_text(summary_line_path, summary_line_text.strip())

    final_status_line_text = (
        "schema=ddn.ci.final_status.v1 "
        f"status=fail reason={reason} failed_steps={len(failed_step_ids)} "
        "aggregate_status=fail overall_ok=0"
    )
    final_status_parse_doc = json.loads(final_status_parse_path.read_text(encoding="utf-8"))
    status_line_path = Path(str(final_status_parse_doc["status_line_path"]))
    write_text(status_line_path, final_status_line_text)

    parsed = final_status_parse_doc.get("parsed")
    if isinstance(parsed, dict):
        parsed["status"] = "fail"
        parsed["reason"] = reason
        parsed["failed_steps"] = str(len(failed_step_ids))
        parsed["aggregate_status"] = "fail"
        parsed["overall_ok"] = "0"
    write_json(final_status_parse_path, final_status_parse_doc)

    result_doc = json.loads(result_path.read_text(encoding="utf-8"))
    result_doc["ok"] = False
    result_doc["status"] = "fail"
    result_doc["reason"] = reason
    result_doc["overall_ok"] = False
    result_doc["aggregate_status"] = "fail"
    result_doc["failed_steps"] = len(failed_step_ids)
    result_doc["summary_line"] = summary_line_text.strip()
    write_json(result_path, result_doc)

    badge_doc = json.loads(badge_path.read_text(encoding="utf-8"))
    badge_doc["status"] = "fail"
    badge_doc["ok"] = False
    badge_doc["label"] = "ci:fail"
    write_json(badge_path, badge_doc)

    brief_text = brief_path.read_text(encoding="utf-8")
    brief_text = brief_text.replace("status=pass", "status=fail", 1)
    brief_text = brief_text.replace("reason=ok", f"reason={reason}", 1)
    brief_text = brief_text.replace("failed_steps_count=0", f"failed_steps_count={len(failed_step_ids)}", 1)
    brief_text = brief_text.replace("failed_steps=-", f"failed_steps={failed_steps_value}", 1)
    write_text(brief_path, brief_text)

    triage_doc = json.loads(triage_path.read_text(encoding="utf-8"))
    triage_doc["status"] = "fail"
    triage_doc["reason"] = reason
    triage_doc["failed_steps_count"] = len(failed_step_ids)
    triage_doc["failed_step_detail_rows_count"] = len(failed_step_ids)
    triage_doc["failed_step_logs_rows_count"] = len(failed_step_ids)
    triage_doc["failed_step_detail_order"] = list(failed_step_ids)
    triage_doc["failed_step_logs_order"] = list(failed_step_ids)
    triage_doc["failed_steps"] = [
        {
            "name": step_name,
            "returncode": 1,
            "cmd": (
                " ".join(str(token) for token in step_rows.get(step_name, {}).get("cmd", []))
                if isinstance(step_rows.get(step_name, {}).get("cmd"), list)
                else "-"
            ),
            "fast_fail_step_detail": (
                "name="
                + step_name
                + " rc=1 cmd="
                + (
                    " ".join(str(token) for token in step_rows.get(step_name, {}).get("cmd", []))
                    if isinstance(step_rows.get(step_name, {}).get("cmd"), list)
                    else "-"
                )
            ),
            "fast_fail_step_logs": (
                f"name={step_name} "
                f"stdout={failed_logs[step_name]['stdout']} stderr={failed_logs[step_name]['stderr']}"
            ),
            "stdout_log_path": failed_logs[step_name]["stdout"],
            "stdout_log_path_norm": failed_logs[step_name]["stdout"].replace("\\", "/"),
            "stderr_log_path": failed_logs[step_name]["stderr"],
            "stderr_log_path_norm": failed_logs[step_name]["stderr"].replace("\\", "/"),
            "brief": f"[selftest] {step_name} failed",
        }
        for step_name in failed_step_ids
    ]
    write_json(triage_path, triage_doc)


def run_triage_mutation_expect_fail(
    root: Path,
    *,
    case_name: str,
    mutator,
    expected_code: str,
    use_fail_fixture: bool = True,
    required_steps=REQUIRED_STEPS_FULL,
    sanity_profile: str = "full",
) -> str | None:
    index = build_index_case(root, case_name)
    if use_fail_fixture:
        configure_case_as_fail(index)
    index_doc = json.loads(index.read_text(encoding="utf-8"))
    triage_report = Path(str(index_doc["reports"]["ci_fail_triage_json"]))
    triage_doc = json.loads(triage_report.read_text(encoding="utf-8"))
    mutator(triage_doc)
    write_json(triage_report, triage_doc)
    proc = run_check(
        index,
        required_steps,
        sanity_profile=sanity_profile,
        enforce_profile_step_contract=True,
    )
    if proc.returncode == 0:
        return f"{case_name} case must fail"
    if f"fail code={expected_code}" not in proc.stderr:
        return f"{case_name} code mismatch: err={proc.stderr}"
    return None


def main() -> int:
    latest_smoke_reason_issue = check_latest_smoke_skip_reason_contract()
    if latest_smoke_reason_issue is not None:
        return fail(latest_smoke_reason_issue)

    root = resolve_selftest_root()

    ok_index = build_index_case(root, "ok")
    ok_proc = run_check(
        ok_index,
        REQUIRED_STEPS_FULL,
        sanity_profile="full",
        enforce_profile_step_contract=True,
    )
    if ok_proc.returncode != 0:
        return fail(f"ok case failed: out={ok_proc.stdout} err={ok_proc.stderr}")

    summary_group_id_missing_index = build_index_case(root, "summary_group_id_missing")
    summary_group_id_missing_doc = json.loads(summary_group_id_missing_index.read_text(encoding="utf-8"))
    summary_group_id_missing_path = Path(str(summary_group_id_missing_doc["reports"]["summary"]))
    filtered_lines = [
        line
        for line in summary_group_id_missing_path.read_text(encoding="utf-8").splitlines()
        if "seamgrim_group_id_summary_status" not in line
    ]
    write_text(summary_group_id_missing_path, "\n".join(filtered_lines))
    summary_group_id_missing_proc = run_check(
        summary_group_id_missing_index,
        REQUIRED_STEPS_FULL,
        sanity_profile="full",
        enforce_profile_step_contract=True,
    )
    if summary_group_id_missing_proc.returncode == 0:
        return fail("summary group_id key missing case must fail")
    if f"fail code={CODES['SUMMARY_KEY_MISSING']}" not in summary_group_id_missing_proc.stderr:
        return fail(
            "summary group_id key missing code mismatch: "
            f"err={summary_group_id_missing_proc.stderr}"
        )

    summary_status_marker_duplicate_index = build_index_case(root, "summary_status_marker_duplicate")
    summary_status_marker_duplicate_doc = json.loads(summary_status_marker_duplicate_index.read_text(encoding="utf-8"))
    summary_status_marker_duplicate_path = Path(str(summary_status_marker_duplicate_doc["reports"]["summary"]))
    summary_status_marker_duplicate_lines = summary_status_marker_duplicate_path.read_text(
        encoding="utf-8"
    ).splitlines()
    summary_status_marker_duplicate_lines.append("[ci-gate-summary] FAIL")
    write_text(summary_status_marker_duplicate_path, "\n".join(summary_status_marker_duplicate_lines))
    summary_status_marker_duplicate_proc = run_check(
        summary_status_marker_duplicate_index,
        REQUIRED_STEPS_FULL,
        sanity_profile="full",
        enforce_profile_step_contract=True,
    )
    if summary_status_marker_duplicate_proc.returncode == 0:
        return fail("summary status marker duplicate case must fail")
    if f"fail code={CODES['SUMMARY_STATUS_MISMATCH']}" not in summary_status_marker_duplicate_proc.stderr:
        return fail(
            "summary status marker duplicate code mismatch: "
            f"err={summary_status_marker_duplicate_proc.stderr}"
        )

    summary_status_marker_not_first_pass_index = build_index_case(root, "summary_status_marker_not_first_pass")
    summary_status_marker_not_first_pass_doc = json.loads(
        summary_status_marker_not_first_pass_index.read_text(encoding="utf-8")
    )
    summary_status_marker_not_first_pass_path = Path(
        str(summary_status_marker_not_first_pass_doc["reports"]["summary"])
    )
    summary_status_marker_not_first_pass_lines = summary_status_marker_not_first_pass_path.read_text(
        encoding="utf-8"
    ).splitlines()
    summary_status_marker_not_first_pass_idx = next(
        (
            idx
            for idx, line in enumerate(summary_status_marker_not_first_pass_lines)
            if line in {"[ci-gate-summary] PASS", "[ci-gate-summary] FAIL"}
        ),
        -1,
    )
    if summary_status_marker_not_first_pass_idx < 0:
        return fail("summary status marker not first pass setup failed")
    summary_status_marker_not_first_pass_line = summary_status_marker_not_first_pass_lines.pop(
        summary_status_marker_not_first_pass_idx
    )
    summary_status_marker_not_first_pass_lines.insert(1, summary_status_marker_not_first_pass_line)
    write_text(
        summary_status_marker_not_first_pass_path,
        "\n".join(summary_status_marker_not_first_pass_lines),
    )
    summary_status_marker_not_first_pass_proc = run_check(
        summary_status_marker_not_first_pass_index,
        REQUIRED_STEPS_FULL,
        sanity_profile="full",
        enforce_profile_step_contract=True,
    )
    if summary_status_marker_not_first_pass_proc.returncode == 0:
        return fail("summary status marker not first pass case must fail")
    if f"fail code={CODES['SUMMARY_STATUS_MISMATCH']}" not in summary_status_marker_not_first_pass_proc.stderr:
        return fail(
            "summary status marker not first pass code mismatch: "
            f"err={summary_status_marker_not_first_pass_proc.stderr}"
        )

    summary_status_marker_not_first_fail_index = build_index_case(root, "summary_status_marker_not_first_fail")
    configure_case_as_fail(
        summary_status_marker_not_first_fail_index,
        reason="status_marker_not_first_fail",
        failed_step_ids=("ci_sync_readiness_report_check",),
    )
    summary_status_marker_not_first_fail_doc = json.loads(
        summary_status_marker_not_first_fail_index.read_text(encoding="utf-8")
    )
    summary_status_marker_not_first_fail_path = Path(
        str(summary_status_marker_not_first_fail_doc["reports"]["summary"])
    )
    summary_status_marker_not_first_fail_lines = summary_status_marker_not_first_fail_path.read_text(
        encoding="utf-8"
    ).splitlines()
    summary_status_marker_not_first_fail_status_idx = next(
        (
            idx
            for idx, line in enumerate(summary_status_marker_not_first_fail_lines)
            if line in {"[ci-gate-summary] PASS", "[ci-gate-summary] FAIL"}
        ),
        -1,
    )
    summary_status_marker_not_first_fail_failed_steps_idx = next(
        (
            idx
            for idx, line in enumerate(summary_status_marker_not_first_fail_lines)
            if line.startswith("[ci-gate-summary] failed_steps=")
        ),
        -1,
    )
    if summary_status_marker_not_first_fail_status_idx < 0 or summary_status_marker_not_first_fail_failed_steps_idx < 0:
        return fail("summary status marker not first fail setup failed")
    summary_status_marker_not_first_fail_failed_steps_line = summary_status_marker_not_first_fail_lines.pop(
        summary_status_marker_not_first_fail_failed_steps_idx
    )
    if summary_status_marker_not_first_fail_failed_steps_idx < summary_status_marker_not_first_fail_status_idx:
        summary_status_marker_not_first_fail_status_idx -= 1
    summary_status_marker_not_first_fail_lines.insert(
        summary_status_marker_not_first_fail_status_idx,
        summary_status_marker_not_first_fail_failed_steps_line,
    )
    write_text(
        summary_status_marker_not_first_fail_path,
        "\n".join(summary_status_marker_not_first_fail_lines),
    )
    summary_status_marker_not_first_fail_proc = run_check(
        summary_status_marker_not_first_fail_index,
        REQUIRED_STEPS_FULL,
        sanity_profile="full",
        enforce_profile_step_contract=True,
    )
    if summary_status_marker_not_first_fail_proc.returncode == 0:
        return fail("summary status marker not first fail case must fail")
    if f"fail code={CODES['SUMMARY_STATUS_MISMATCH']}" not in summary_status_marker_not_first_fail_proc.stderr:
        return fail(
            "summary status marker not first fail code mismatch: "
            f"err={summary_status_marker_not_first_fail_proc.stderr}"
        )

    summary_duplicate_failed_steps_key_index = build_index_case(root, "summary_duplicate_failed_steps_key")
    summary_duplicate_failed_steps_key_doc = json.loads(
        summary_duplicate_failed_steps_key_index.read_text(encoding="utf-8")
    )
    summary_duplicate_failed_steps_key_path = Path(str(summary_duplicate_failed_steps_key_doc["reports"]["summary"]))
    summary_duplicate_failed_steps_key_lines = summary_duplicate_failed_steps_key_path.read_text(
        encoding="utf-8"
    ).splitlines()
    summary_duplicate_failed_steps_key_lines.append("[ci-gate-summary] failed_steps=(none)")
    write_text(summary_duplicate_failed_steps_key_path, "\n".join(summary_duplicate_failed_steps_key_lines))
    summary_duplicate_failed_steps_key_proc = run_check(
        summary_duplicate_failed_steps_key_index,
        REQUIRED_STEPS_FULL,
        sanity_profile="full",
        enforce_profile_step_contract=True,
    )
    if summary_duplicate_failed_steps_key_proc.returncode == 0:
        return fail("summary duplicate failed_steps key case must fail")
    if (
        f"fail code={CODES['SUMMARY_VALUE_INVALID']}"
        not in summary_duplicate_failed_steps_key_proc.stderr
    ):
        return fail(
            "summary duplicate failed_steps key code mismatch: "
            f"err={summary_duplicate_failed_steps_key_proc.stderr}"
        )

    summary_age_close_partial_missing_index = build_index_case(root, "summary_age_close_partial_missing")
    summary_age_close_partial_missing_doc = json.loads(
        summary_age_close_partial_missing_index.read_text(encoding="utf-8")
    )
    summary_age_close_partial_missing_path = Path(str(summary_age_close_partial_missing_doc["reports"]["summary"]))
    summary_age_close_partial_missing_lines = summary_age_close_partial_missing_path.read_text(
        encoding="utf-8"
    ).splitlines()
    summary_age_close_partial_missing_lines.append(
        f"[ci-gate-summary] age2_status={root / 'only_age2_present.detjson'}"
    )
    write_text(summary_age_close_partial_missing_path, "\n".join(summary_age_close_partial_missing_lines))
    summary_age_close_partial_missing_proc = run_check(
        summary_age_close_partial_missing_index,
        REQUIRED_STEPS_FULL,
        sanity_profile="full",
        enforce_profile_step_contract=True,
    )
    if summary_age_close_partial_missing_proc.returncode == 0:
        return fail("summary age close partial missing case must fail")
    if f"fail code={CODES['SUMMARY_KEY_MISSING']}" not in summary_age_close_partial_missing_proc.stderr:
        return fail(
            "summary age close partial missing code mismatch: "
            f"err={summary_age_close_partial_missing_proc.stderr}"
        )

    profile_matrix_aggregate_bad_index = build_index_case(root, "profile_matrix_aggregate_bad")
    profile_matrix_aggregate_bad_doc = json.loads(profile_matrix_aggregate_bad_index.read_text(encoding="utf-8"))
    profile_matrix_aggregate_bad_path = Path(
        str(profile_matrix_aggregate_bad_doc["reports"]["ci_profile_matrix_gate_selftest"])
    )
    profile_matrix_aggregate_bad_report = json.loads(profile_matrix_aggregate_bad_path.read_text(encoding="utf-8"))
    profile_matrix_aggregate_bad_report["aggregate_summary_sanity_by_profile"]["full"]["values"][
        "ci_sanity_pack_golden_metadata_ok"
    ] = "0"
    write_json(profile_matrix_aggregate_bad_path, profile_matrix_aggregate_bad_report)
    profile_matrix_aggregate_bad_triage_path = Path(
        str(profile_matrix_aggregate_bad_doc["reports"]["ci_fail_triage_json"])
    )
    profile_matrix_aggregate_bad_snapshot = build_profile_matrix_snapshot_from_doc(
        profile_matrix_aggregate_bad_report,
        report_path=str(profile_matrix_aggregate_bad_path),
    )
    if not isinstance(profile_matrix_aggregate_bad_snapshot, dict):
        return fail("profile_matrix aggregate bad snapshot build failed")
    profile_matrix_aggregate_bad_triage_doc = json.loads(
        profile_matrix_aggregate_bad_triage_path.read_text(encoding="utf-8")
    )
    profile_matrix_aggregate_bad_triage_doc["profile_matrix_selftest"] = (
        build_profile_matrix_triage_payload_from_snapshot(
            profile_matrix_aggregate_bad_snapshot
        )
    )
    write_json(
        profile_matrix_aggregate_bad_triage_path,
        profile_matrix_aggregate_bad_triage_doc,
    )
    profile_matrix_aggregate_bad_proc = run_check(
        profile_matrix_aggregate_bad_index,
        REQUIRED_STEPS_FULL,
        sanity_profile="full",
        enforce_profile_step_contract=True,
    )
    if profile_matrix_aggregate_bad_proc.returncode == 0:
        return fail("profile_matrix aggregate summary bad case must fail")
    if f"fail code={CODES['ARTIFACT_JSON_INVALID']}" not in profile_matrix_aggregate_bad_proc.stderr:
        return fail(
            "profile_matrix aggregate summary bad code mismatch: "
            f"err={profile_matrix_aggregate_bad_proc.stderr}"
        )

    profile_matrix_timeout_defaults_bad_index = build_index_case(root, "profile_matrix_timeout_defaults_bad")
    profile_matrix_timeout_defaults_bad_doc = json.loads(
        profile_matrix_timeout_defaults_bad_index.read_text(encoding="utf-8")
    )
    profile_matrix_timeout_defaults_bad_path = Path(
        str(profile_matrix_timeout_defaults_bad_doc["reports"]["ci_profile_matrix_gate_selftest"])
    )
    profile_matrix_timeout_defaults_bad_report = json.loads(
        profile_matrix_timeout_defaults_bad_path.read_text(encoding="utf-8")
    )
    profile_matrix_timeout_defaults_bad_report["step_timeout_defaults_sec"] = {"core_lang": "bad"}
    write_json(profile_matrix_timeout_defaults_bad_path, profile_matrix_timeout_defaults_bad_report)
    profile_matrix_timeout_defaults_bad_proc = run_check(
        profile_matrix_timeout_defaults_bad_index,
        REQUIRED_STEPS_FULL,
        sanity_profile="full",
        enforce_profile_step_contract=True,
    )
    if profile_matrix_timeout_defaults_bad_proc.returncode == 0:
        return fail("profile_matrix timeout defaults bad case must fail")
    if f"fail code={CODES['ARTIFACT_JSON_INVALID']}" not in profile_matrix_timeout_defaults_bad_proc.stderr:
        return fail(
            "profile_matrix timeout defaults bad code mismatch: "
            f"err={profile_matrix_timeout_defaults_bad_proc.stderr}"
        )
    else:
        generated_at_missing_index = build_index_case(root, "generated_at_missing")
        generated_at_missing_doc = json.loads(generated_at_missing_index.read_text(encoding="utf-8"))
        generated_at_missing_doc["generated_at_utc"] = ""
        write_json(generated_at_missing_index, generated_at_missing_doc)
        generated_at_missing_proc = run_check(
            generated_at_missing_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if generated_at_missing_proc.returncode == 0:
            return fail("generated_at missing case must fail")
        if f"fail code={CODES['GENERATED_AT_MISSING']}" not in generated_at_missing_proc.stderr:
            return fail(f"generated_at missing code mismatch: err={generated_at_missing_proc.stderr}")

        generated_at_invalid_index = build_index_case(root, "generated_at_invalid")
        generated_at_invalid_doc = json.loads(generated_at_invalid_index.read_text(encoding="utf-8"))
        generated_at_invalid_doc["generated_at_utc"] = "not-a-timestamp"
        write_json(generated_at_invalid_index, generated_at_invalid_doc)
        generated_at_invalid_proc = run_check(
            generated_at_invalid_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if generated_at_invalid_proc.returncode == 0:
            return fail("generated_at invalid case must fail")
        if f"fail code={CODES['GENERATED_AT_INVALID']}" not in generated_at_invalid_proc.stderr:
            return fail(f"generated_at invalid code mismatch: err={generated_at_invalid_proc.stderr}")

        report_dir_missing_index = build_index_case(root, "report_dir_missing")
        report_dir_missing_doc = json.loads(report_dir_missing_index.read_text(encoding="utf-8"))
        report_dir_missing_doc["report_dir"] = ""
        write_json(report_dir_missing_index, report_dir_missing_doc)
        report_dir_missing_proc = run_check(
            report_dir_missing_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if report_dir_missing_proc.returncode == 0:
            return fail("report_dir missing case must fail")
        if f"fail code={CODES['REPORT_DIR_MISSING']}" not in report_dir_missing_proc.stderr:
            return fail(f"report_dir missing code mismatch: err={report_dir_missing_proc.stderr}")

        report_dir_not_found_index = build_index_case(root, "report_dir_not_found")
        report_dir_not_found_doc = json.loads(report_dir_not_found_index.read_text(encoding="utf-8"))
        report_dir_not_found_doc["report_dir"] = str(root / "missing" / "report_dir")
        write_json(report_dir_not_found_index, report_dir_not_found_doc)
        report_dir_not_found_proc = run_check(
            report_dir_not_found_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if report_dir_not_found_proc.returncode == 0:
            return fail("report_dir not found case must fail")
        if f"fail code={CODES['REPORT_DIR_NOT_FOUND']}" not in report_dir_not_found_proc.stderr:
            return fail(f"report_dir not found code mismatch: err={report_dir_not_found_proc.stderr}")

        report_prefix_source_invalid_index = build_index_case(root, "report_prefix_source_invalid")
        report_prefix_source_invalid_doc = json.loads(report_prefix_source_invalid_index.read_text(encoding="utf-8"))
        report_prefix_source_invalid_doc["report_prefix"] = "demo"
        report_prefix_source_invalid_doc["report_prefix_source"] = "manual"
        write_json(report_prefix_source_invalid_index, report_prefix_source_invalid_doc)
        report_prefix_source_invalid_proc = run_check(
            report_prefix_source_invalid_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if report_prefix_source_invalid_proc.returncode == 0:
            return fail("report_prefix_source invalid case must fail")
        if f"fail code={CODES['REPORT_PREFIX_SOURCE_INVALID']}" not in report_prefix_source_invalid_proc.stderr:
            return fail(f"report_prefix_source invalid code mismatch: err={report_prefix_source_invalid_proc.stderr}")

        report_prefix_source_mismatch_index = build_index_case(root, "report_prefix_source_mismatch")
        report_prefix_source_mismatch_doc = json.loads(report_prefix_source_mismatch_index.read_text(encoding="utf-8"))
        report_prefix_source_mismatch_doc["report_prefix"] = "demo"
        report_prefix_source_mismatch_doc["report_prefix_source"] = ""
        write_json(report_prefix_source_mismatch_index, report_prefix_source_mismatch_doc)
        report_prefix_source_mismatch_proc = run_check(
            report_prefix_source_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if report_prefix_source_mismatch_proc.returncode == 0:
            return fail("report_prefix_source mismatch case must fail")
        if (
            f"fail code={CODES['REPORT_PREFIX_SOURCE_MISMATCH']}"
            not in report_prefix_source_mismatch_proc.stderr
        ):
            return fail(f"report_prefix_source mismatch code mismatch: err={report_prefix_source_mismatch_proc.stderr}")

        report_prefix_source_env_empty_index = build_index_case(root, "report_prefix_source_env_empty")
        report_prefix_source_env_empty_doc = json.loads(report_prefix_source_env_empty_index.read_text(encoding="utf-8"))
        report_prefix_source_env_empty_doc["report_prefix"] = "demo"
        report_prefix_source_env_empty_doc["report_prefix_source"] = "env:"
        write_json(report_prefix_source_env_empty_index, report_prefix_source_env_empty_doc)
        report_prefix_source_env_empty_proc = run_check(
            report_prefix_source_env_empty_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if report_prefix_source_env_empty_proc.returncode == 0:
            return fail("report_prefix_source env-empty case must fail")
        if f"fail code={CODES['REPORT_PREFIX_SOURCE_INVALID']}" not in report_prefix_source_env_empty_proc.stderr:
            return fail(
                f"report_prefix_source env-empty code mismatch: err={report_prefix_source_env_empty_proc.stderr}"
            )

        report_prefix_source_env_ok_index = build_index_case(root, "report_prefix_source_env_ok")
        report_prefix_source_env_ok_doc = json.loads(report_prefix_source_env_ok_index.read_text(encoding="utf-8"))
        report_prefix_source_env_ok_doc["report_prefix"] = "demo"
        report_prefix_source_env_ok_doc["report_prefix_source"] = "env:CI_REPORT_PREFIX"
        write_json(report_prefix_source_env_ok_index, report_prefix_source_env_ok_doc)
        report_prefix_source_env_ok_proc = run_check(
            report_prefix_source_env_ok_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if report_prefix_source_env_ok_proc.returncode != 0:
            return fail(
                "report_prefix_source env-ok case should pass: "
                f"out={report_prefix_source_env_ok_proc.stdout} err={report_prefix_source_env_ok_proc.stderr}"
            )

        step_log_dir_type_index = build_index_case(root, "step_log_dir_type")
        step_log_dir_type_doc = json.loads(step_log_dir_type_index.read_text(encoding="utf-8"))
        step_log_dir_type_doc["step_log_dir"] = {"bad": "type"}
        write_json(step_log_dir_type_index, step_log_dir_type_doc)
        step_log_dir_type_proc = run_check(
            step_log_dir_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if step_log_dir_type_proc.returncode == 0:
            return fail("step_log_dir type case must fail")
        if f"fail code={CODES['STEP_LOG_DIR_TYPE']}" not in step_log_dir_type_proc.stderr:
            return fail(f"step_log_dir type code mismatch: err={step_log_dir_type_proc.stderr}")

        step_log_failed_only_type_index = build_index_case(root, "step_log_failed_only_type")
        step_log_failed_only_type_doc = json.loads(step_log_failed_only_type_index.read_text(encoding="utf-8"))
        step_log_failed_only_type_doc["step_log_failed_only"] = "1"
        write_json(step_log_failed_only_type_index, step_log_failed_only_type_doc)
        step_log_failed_only_type_proc = run_check(
            step_log_failed_only_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if step_log_failed_only_type_proc.returncode == 0:
            return fail("step_log_failed_only type case must fail")
        if f"fail code={CODES['STEP_LOG_FAILED_ONLY_TYPE']}" not in step_log_failed_only_type_proc.stderr:
            return fail(
                f"step_log_failed_only type code mismatch: err={step_log_failed_only_type_proc.stderr}"
            )

        step_log_config_mismatch_index = build_index_case(root, "step_log_config_mismatch")
        step_log_config_mismatch_doc = json.loads(step_log_config_mismatch_index.read_text(encoding="utf-8"))
        step_log_config_mismatch_doc["step_log_failed_only"] = True
        step_log_config_mismatch_doc["step_log_dir"] = ""
        write_json(step_log_config_mismatch_index, step_log_config_mismatch_doc)
        step_log_config_mismatch_proc = run_check(
            step_log_config_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if step_log_config_mismatch_proc.returncode == 0:
            return fail("step_log config mismatch case must fail")
        if f"fail code={CODES['STEP_LOG_CONFIG_MISMATCH']}" not in step_log_config_mismatch_proc.stderr:
            return fail(
                f"step_log config mismatch code mismatch: err={step_log_config_mismatch_proc.stderr}"
            )

        step_log_dir_not_found_index = build_index_case(root, "step_log_dir_not_found")
        step_log_dir_not_found_doc = json.loads(step_log_dir_not_found_index.read_text(encoding="utf-8"))
        step_log_dir_not_found_doc["step_log_dir"] = str(root / "missing" / "step_logs")
        write_json(step_log_dir_not_found_index, step_log_dir_not_found_doc)
        step_log_dir_not_found_proc = run_check(
            step_log_dir_not_found_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if step_log_dir_not_found_proc.returncode == 0:
            return fail("step_log_dir not found case must fail")
        if f"fail code={CODES['STEP_LOG_DIR_NOT_FOUND']}" not in step_log_dir_not_found_proc.stderr:
            return fail(
                f"step_log_dir not found code mismatch: err={step_log_dir_not_found_proc.stderr}"
            )

        missing_key_index = build_index_case(root, "missing_key")
        missing_key_doc = json.loads(missing_key_index.read_text(encoding="utf-8"))
        missing_key_doc["reports"].pop("seamgrim_wasm_cli_diag_parity", None)
        write_json(missing_key_index, missing_key_doc)
        missing_key_proc = run_check(
            missing_key_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if missing_key_proc.returncode == 0:
            return fail("missing key case must fail")
        if f"fail code={CODES['REPORT_KEY_MISSING']}" not in missing_key_proc.stderr:
            return fail(f"missing key code mismatch: err={missing_key_proc.stderr}")

        missing_fixed64_key_index = build_index_case(root, "missing_fixed64_key")
        missing_fixed64_key_doc = json.loads(missing_fixed64_key_index.read_text(encoding="utf-8"))
        missing_fixed64_key_doc["reports"].pop("fixed64_threeway_inputs", None)
        write_json(missing_fixed64_key_index, missing_fixed64_key_doc)
        missing_fixed64_key_proc = run_check(
            missing_fixed64_key_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if missing_fixed64_key_proc.returncode == 0:
            return fail("missing fixed64 key case must fail")
        if f"fail code={CODES['REPORT_KEY_MISSING']}" not in missing_fixed64_key_proc.stderr:
            return fail(f"missing fixed64 key code mismatch: err={missing_fixed64_key_proc.stderr}")

        missing_path_index = build_index_case(root, "missing_path")
        missing_path_doc = json.loads(missing_path_index.read_text(encoding="utf-8"))
        missing_path_doc["reports"]["seamgrim_wasm_cli_diag_parity"] = str(root / "missing" / "parity.detjson")
        write_json(missing_path_index, missing_path_doc)
        missing_path_proc = run_check(
            missing_path_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if missing_path_proc.returncode == 0:
            return fail("missing path case must fail")
        if f"fail code={CODES['REPORT_PATH_MISSING']}" not in missing_path_proc.stderr:
            return fail(f"missing path code mismatch: err={missing_path_proc.stderr}")

        bad_schema_index = build_index_case(root, "bad_schema")
        bad_schema_doc = json.loads(bad_schema_index.read_text(encoding="utf-8"))
        parity_path = Path(str(bad_schema_doc["reports"]["seamgrim_wasm_cli_diag_parity"]))
        write_json(parity_path, {"schema": "wrong.schema"})
        bad_schema_proc = run_check(
            bad_schema_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if bad_schema_proc.returncode == 0:
            return fail("bad schema case must fail")
        if f"fail code={CODES['ARTIFACT_SCHEMA_MISMATCH']}" not in bad_schema_proc.stderr:
            return fail(f"bad schema code mismatch: err={bad_schema_proc.stderr}")

        final_parse_parsed_missing_index = build_index_case(root, "final_parse_parsed_missing")
        final_parse_parsed_missing_doc = json.loads(final_parse_parsed_missing_index.read_text(encoding="utf-8"))
        final_parse_parsed_missing_report = Path(str(final_parse_parsed_missing_doc["reports"]["final_status_parse_json"]))
        final_parse_parsed_missing_payload = json.loads(final_parse_parsed_missing_report.read_text(encoding="utf-8"))
        final_parse_parsed_missing_payload["parsed"] = []
        write_json(final_parse_parsed_missing_report, final_parse_parsed_missing_payload)
        final_parse_parsed_missing_proc = run_check(
            final_parse_parsed_missing_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_parsed_missing_proc.returncode == 0:
            return fail("final parse parsed missing case must fail")
        if f"fail code={CODES['FINAL_PARSE_PARSED_MISSING']}" not in final_parse_parsed_missing_proc.stderr:
            return fail(f"final parse parsed missing code mismatch: err={final_parse_parsed_missing_proc.stderr}")

        final_parse_status_line_path_missing_index = build_index_case(root, "final_parse_status_line_path_missing")
        final_parse_status_line_path_missing_doc = json.loads(
            final_parse_status_line_path_missing_index.read_text(encoding="utf-8")
        )
        final_parse_status_line_path_missing_report = Path(
            str(final_parse_status_line_path_missing_doc["reports"]["final_status_parse_json"])
        )
        final_parse_status_line_path_missing_payload = json.loads(
            final_parse_status_line_path_missing_report.read_text(encoding="utf-8")
        )
        final_parse_status_line_path_missing_payload["status_line_path"] = ""
        write_json(final_parse_status_line_path_missing_report, final_parse_status_line_path_missing_payload)
        final_parse_status_line_path_missing_proc = run_check(
            final_parse_status_line_path_missing_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_status_line_path_missing_proc.returncode == 0:
            return fail("final parse status_line_path missing case must fail")
        if (
            f"fail code={CODES['FINAL_PARSE_STATUS_LINE_PATH_MISSING']}"
            not in final_parse_status_line_path_missing_proc.stderr
        ):
            return fail(
                "final parse status_line_path missing code mismatch: "
                f"err={final_parse_status_line_path_missing_proc.stderr}"
            )

        final_parse_status_line_path_not_found_index = build_index_case(root, "final_parse_status_line_path_not_found")
        final_parse_status_line_path_not_found_doc = json.loads(
            final_parse_status_line_path_not_found_index.read_text(encoding="utf-8")
        )
        final_parse_status_line_path_not_found_report = Path(
            str(final_parse_status_line_path_not_found_doc["reports"]["final_status_parse_json"])
        )
        final_parse_status_line_path_not_found_payload = json.loads(
            final_parse_status_line_path_not_found_report.read_text(encoding="utf-8")
        )
        final_parse_status_line_path_not_found_payload["status_line_path"] = str(root / "missing" / "ci_gate_final_status_line.txt")
        write_json(final_parse_status_line_path_not_found_report, final_parse_status_line_path_not_found_payload)
        final_parse_status_line_path_not_found_proc = run_check(
            final_parse_status_line_path_not_found_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_status_line_path_not_found_proc.returncode == 0:
            return fail("final parse status_line_path not found case must fail")
        if (
            f"fail code={CODES['FINAL_PARSE_STATUS_LINE_PATH_NOT_FOUND']}"
            not in final_parse_status_line_path_not_found_proc.stderr
        ):
            return fail(
                "final parse status_line_path not found code mismatch: "
                f"err={final_parse_status_line_path_not_found_proc.stderr}"
            )

        final_parse_status_mismatch_index = build_index_case(root, "final_parse_status_mismatch")
        final_parse_status_mismatch_doc = json.loads(final_parse_status_mismatch_index.read_text(encoding="utf-8"))
        final_parse_status_mismatch_report = Path(str(final_parse_status_mismatch_doc["reports"]["final_status_parse_json"]))
        final_parse_status_mismatch_payload = json.loads(final_parse_status_mismatch_report.read_text(encoding="utf-8"))
        final_parse_status_mismatch_payload["parsed"]["status"] = "fail"
        write_json(final_parse_status_mismatch_report, final_parse_status_mismatch_payload)
        final_parse_status_mismatch_proc = run_check(
            final_parse_status_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_status_mismatch_proc.returncode == 0:
            return fail("final parse status mismatch case must fail")
        if f"fail code={CODES['FINAL_PARSE_STATUS_MISMATCH']}" not in final_parse_status_mismatch_proc.stderr:
            return fail(f"final parse status mismatch code mismatch: err={final_parse_status_mismatch_proc.stderr}")

        final_parse_overall_ok_invalid_index = build_index_case(root, "final_parse_overall_ok_invalid")
        final_parse_overall_ok_invalid_doc = json.loads(final_parse_overall_ok_invalid_index.read_text(encoding="utf-8"))
        final_parse_overall_ok_invalid_report = Path(
            str(final_parse_overall_ok_invalid_doc["reports"]["final_status_parse_json"])
        )
        final_parse_overall_ok_invalid_payload = json.loads(final_parse_overall_ok_invalid_report.read_text(encoding="utf-8"))
        final_parse_overall_ok_invalid_payload["parsed"]["overall_ok"] = "true"
        write_json(final_parse_overall_ok_invalid_report, final_parse_overall_ok_invalid_payload)
        final_parse_overall_ok_invalid_proc = run_check(
            final_parse_overall_ok_invalid_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_overall_ok_invalid_proc.returncode == 0:
            return fail("final parse overall_ok invalid case must fail")
        if f"fail code={CODES['FINAL_PARSE_OVERALL_OK_INVALID']}" not in final_parse_overall_ok_invalid_proc.stderr:
            return fail(f"final parse overall_ok invalid code mismatch: err={final_parse_overall_ok_invalid_proc.stderr}")

        final_parse_overall_ok_mismatch_index = build_index_case(root, "final_parse_overall_ok_mismatch")
        final_parse_overall_ok_mismatch_doc = json.loads(final_parse_overall_ok_mismatch_index.read_text(encoding="utf-8"))
        final_parse_overall_ok_mismatch_report = Path(
            str(final_parse_overall_ok_mismatch_doc["reports"]["final_status_parse_json"])
        )
        final_parse_overall_ok_mismatch_payload = json.loads(
            final_parse_overall_ok_mismatch_report.read_text(encoding="utf-8")
        )
        final_parse_overall_ok_mismatch_payload["parsed"]["overall_ok"] = "0"
        write_json(final_parse_overall_ok_mismatch_report, final_parse_overall_ok_mismatch_payload)
        final_parse_overall_ok_mismatch_proc = run_check(
            final_parse_overall_ok_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_overall_ok_mismatch_proc.returncode == 0:
            return fail("final parse overall_ok mismatch case must fail")
        if (
            f"fail code={CODES['FINAL_PARSE_OVERALL_OK_MISMATCH']}"
            not in final_parse_overall_ok_mismatch_proc.stderr
        ):
            return fail(
                f"final parse overall_ok mismatch code mismatch: err={final_parse_overall_ok_mismatch_proc.stderr}"
            )

        final_parse_aggregate_status_invalid_index = build_index_case(root, "final_parse_aggregate_status_invalid")
        final_parse_aggregate_status_invalid_doc = json.loads(
            final_parse_aggregate_status_invalid_index.read_text(encoding="utf-8")
        )
        final_parse_aggregate_status_invalid_report = Path(
            str(final_parse_aggregate_status_invalid_doc["reports"]["final_status_parse_json"])
        )
        final_parse_aggregate_status_invalid_payload = json.loads(
            final_parse_aggregate_status_invalid_report.read_text(encoding="utf-8")
        )
        final_parse_aggregate_status_invalid_payload["parsed"]["aggregate_status"] = "unknown"
        write_json(final_parse_aggregate_status_invalid_report, final_parse_aggregate_status_invalid_payload)
        final_parse_aggregate_status_invalid_proc = run_check(
            final_parse_aggregate_status_invalid_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_aggregate_status_invalid_proc.returncode == 0:
            return fail("final parse aggregate_status invalid case must fail")
        if (
            f"fail code={CODES['FINAL_PARSE_AGGREGATE_STATUS_INVALID']}"
            not in final_parse_aggregate_status_invalid_proc.stderr
        ):
            return fail(
                "final parse aggregate_status invalid code mismatch: "
                f"err={final_parse_aggregate_status_invalid_proc.stderr}"
            )

        final_parse_failed_steps_type_index = build_index_case(root, "final_parse_failed_steps_type")
        final_parse_failed_steps_type_doc = json.loads(final_parse_failed_steps_type_index.read_text(encoding="utf-8"))
        final_parse_failed_steps_type_report = Path(
            str(final_parse_failed_steps_type_doc["reports"]["final_status_parse_json"])
        )
        final_parse_failed_steps_type_payload = json.loads(final_parse_failed_steps_type_report.read_text(encoding="utf-8"))
        final_parse_failed_steps_type_payload["parsed"]["failed_steps"] = "x"
        write_json(final_parse_failed_steps_type_report, final_parse_failed_steps_type_payload)
        final_parse_failed_steps_type_proc = run_check(
            final_parse_failed_steps_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_failed_steps_type_proc.returncode == 0:
            return fail("final parse failed_steps type case must fail")
        if f"fail code={CODES['FINAL_PARSE_FAILED_STEPS_TYPE']}" not in final_parse_failed_steps_type_proc.stderr:
            return fail(f"final parse failed_steps type code mismatch: err={final_parse_failed_steps_type_proc.stderr}")

        final_parse_failed_steps_mismatch_index = build_index_case(root, "final_parse_failed_steps_mismatch")
        final_parse_failed_steps_mismatch_doc = json.loads(
            final_parse_failed_steps_mismatch_index.read_text(encoding="utf-8")
        )
        final_parse_failed_steps_mismatch_report = Path(
            str(final_parse_failed_steps_mismatch_doc["reports"]["final_status_parse_json"])
        )
        final_parse_failed_steps_mismatch_payload = json.loads(
            final_parse_failed_steps_mismatch_report.read_text(encoding="utf-8")
        )
        final_parse_failed_steps_mismatch_payload["parsed"]["failed_steps"] = "1"
        write_json(final_parse_failed_steps_mismatch_report, final_parse_failed_steps_mismatch_payload)
        final_parse_failed_steps_mismatch_proc = run_check(
            final_parse_failed_steps_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_failed_steps_mismatch_proc.returncode == 0:
            return fail("final parse failed_steps mismatch case must fail")
        if (
            f"fail code={CODES['FINAL_PARSE_FAILED_STEPS_MISMATCH']}"
            not in final_parse_failed_steps_mismatch_proc.stderr
        ):
            return fail(
                f"final parse failed_steps mismatch code mismatch: err={final_parse_failed_steps_mismatch_proc.stderr}"
            )

        missing_required_step_index = build_index_case(root, "missing_required_step")
        missing_required_step_doc = json.loads(missing_required_step_index.read_text(encoding="utf-8"))
        missing_required_step_doc["steps"] = [
            row for row in missing_required_step_doc["steps"] if row.get("name") != "ci_sync_readiness_report_check"
        ]
        write_json(missing_required_step_index, missing_required_step_doc)
        missing_required_step_proc = run_check(
            missing_required_step_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if missing_required_step_proc.returncode == 0:
            return fail("missing required step case must fail")
        if f"fail code={CODES['REQUIRED_STEP_MISSING']}" not in missing_required_step_proc.stderr:
            return fail(f"missing required step code mismatch: err={missing_required_step_proc.stderr}")

        bad_step_shape_index = build_index_case(root, "bad_step_shape")
        bad_step_shape_doc = json.loads(bad_step_shape_index.read_text(encoding="utf-8"))
        bad_step_shape_doc["steps"][0] = "oops"
        write_json(bad_step_shape_index, bad_step_shape_doc)
        bad_step_shape_proc = run_check(
            bad_step_shape_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if bad_step_shape_proc.returncode == 0:
            return fail("bad step shape case must fail")
        if f"fail code={CODES['STEP_ROW_TYPE']}" not in bad_step_shape_proc.stderr:
            return fail(f"bad step shape code mismatch: err={bad_step_shape_proc.stderr}")

        bad_profile_index = build_index_case(root, "bad_profile")
        bad_profile_doc = json.loads(bad_profile_index.read_text(encoding="utf-8"))
        bad_profile_doc["ci_sanity_profile"] = "unknown_profile"
        write_json(bad_profile_index, bad_profile_doc)
        bad_profile_proc = run_check(
            bad_profile_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if bad_profile_proc.returncode == 0:
            return fail("bad profile case must fail")
        if f"fail code={CODES['PROFILE_INVALID']}" not in bad_profile_proc.stderr:
            return fail(f"bad profile code mismatch: err={bad_profile_proc.stderr}")

        profile_mismatch_index = build_index_case(root, "profile_mismatch", sanity_profile="seamgrim")
        profile_mismatch_proc = run_check(
            profile_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if profile_mismatch_proc.returncode == 0:
            return fail("profile mismatch case must fail")
        if f"fail code={CODES['PROFILE_MISMATCH']}" not in profile_mismatch_proc.stderr:
            return fail(f"profile mismatch code mismatch: err={profile_mismatch_proc.stderr}")

        index_overall_ok_type_index = build_index_case(root, "index_overall_ok_type")
        index_overall_ok_type_doc = json.loads(index_overall_ok_type_index.read_text(encoding="utf-8"))
        index_overall_ok_type_doc["overall_ok"] = "true"
        write_json(index_overall_ok_type_index, index_overall_ok_type_doc)
        index_overall_ok_type_proc = run_check(
            index_overall_ok_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if index_overall_ok_type_proc.returncode == 0:
            return fail("index overall_ok type case must fail")
        if f"fail code={CODES['INDEX_OVERALL_OK_TYPE']}" not in index_overall_ok_type_proc.stderr:
            return fail(f"index overall_ok type code mismatch: err={index_overall_ok_type_proc.stderr}")

        index_overall_ok_steps_mismatch_index = build_index_case(root, "index_overall_ok_steps_mismatch")
        index_overall_ok_steps_mismatch_doc = json.loads(index_overall_ok_steps_mismatch_index.read_text(encoding="utf-8"))
        index_overall_ok_steps_mismatch_doc["overall_ok"] = False
        write_json(index_overall_ok_steps_mismatch_index, index_overall_ok_steps_mismatch_doc)
        index_overall_ok_steps_mismatch_proc = run_check(
            index_overall_ok_steps_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if index_overall_ok_steps_mismatch_proc.returncode == 0:
            return fail("index overall_ok steps mismatch case must fail")
        if f"fail code={CODES['INDEX_OVERALL_OK_STEPS_MISMATCH']}" not in index_overall_ok_steps_mismatch_proc.stderr:
            return fail(f"index overall_ok steps mismatch code mismatch: err={index_overall_ok_steps_mismatch_proc.stderr}")

        sanity_profile_mismatch_index = build_index_case(root, "sanity_profile_mismatch")
        sanity_profile_mismatch_doc = json.loads(sanity_profile_mismatch_index.read_text(encoding="utf-8"))
        sanity_report = Path(str(sanity_profile_mismatch_doc["reports"]["ci_sanity_gate"]))
        sanity_report_doc = json.loads(sanity_report.read_text(encoding="utf-8"))
        sanity_report_doc["profile"] = "seamgrim"
        write_json(sanity_report, sanity_report_doc)
        sanity_profile_mismatch_proc = run_check(
            sanity_profile_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if sanity_profile_mismatch_proc.returncode == 0:
            return fail("sanity profile mismatch case must fail")
        if f"fail code={CODES['SANITY_PROFILE_MISMATCH']}" not in sanity_profile_mismatch_proc.stderr:
            return fail(f"sanity profile mismatch code mismatch: err={sanity_profile_mismatch_proc.stderr}")

        sanity_required_step_missing_index = build_index_case(root, "sanity_required_step_missing")
        sanity_required_step_missing_doc = json.loads(sanity_required_step_missing_index.read_text(encoding="utf-8"))
        sanity_required_step_missing_report = Path(str(sanity_required_step_missing_doc["reports"]["ci_sanity_gate"]))
        sanity_required_step_missing_payload = json.loads(sanity_required_step_missing_report.read_text(encoding="utf-8"))
        sanity_required_step_missing_payload["steps"] = []
        write_json(sanity_required_step_missing_report, sanity_required_step_missing_payload)
        sanity_required_step_missing_proc = run_check(
            sanity_required_step_missing_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if sanity_required_step_missing_proc.returncode == 0:
            return fail("sanity required lang consistency step missing case must fail")
        if f"fail code={CODES['REQUIRED_STEP_MISSING']}" not in sanity_required_step_missing_proc.stderr:
            return fail(
                "sanity required lang consistency step missing code mismatch: "
                f"err={sanity_required_step_missing_proc.stderr}"
            )

        sanity_required_step_fail_index = build_index_case(root, "sanity_required_step_fail")
        sanity_required_step_fail_doc = json.loads(sanity_required_step_fail_index.read_text(encoding="utf-8"))
        sanity_required_step_fail_report = Path(str(sanity_required_step_fail_doc["reports"]["ci_sanity_gate"]))
        sanity_required_step_fail_payload = json.loads(sanity_required_step_fail_report.read_text(encoding="utf-8"))
        if not isinstance(sanity_required_step_fail_payload.get("steps"), list):
            return fail("sanity required lang consistency step fail case invalid setup")
        sanity_required_step_fail_payload["steps"][0]["ok"] = False
        sanity_required_step_fail_payload["steps"][0]["returncode"] = 1
        write_json(sanity_required_step_fail_report, sanity_required_step_fail_payload)
        sanity_required_step_fail_proc = run_check(
            sanity_required_step_fail_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if sanity_required_step_fail_proc.returncode == 0:
            return fail("sanity required lang consistency step fail case must fail")
        if f"fail code={CODES['REQUIRED_STEP_MISSING']}" not in sanity_required_step_fail_proc.stderr:
            return fail(
                "sanity required lang consistency step fail code mismatch: "
                f"err={sanity_required_step_fail_proc.stderr}"
            )

        core_lang_sanity_required_step_missing_index = build_index_case(
            root,
            "core_lang_sanity_required_step_missing",
            sanity_profile="core_lang",
        )
        core_lang_sanity_required_step_missing_doc = json.loads(
            core_lang_sanity_required_step_missing_index.read_text(encoding="utf-8")
        )
        core_lang_sanity_required_step_missing_report = Path(
            str(core_lang_sanity_required_step_missing_doc["reports"]["ci_sanity_gate"])
        )
        core_lang_sanity_required_step_missing_payload = json.loads(
            core_lang_sanity_required_step_missing_report.read_text(encoding="utf-8")
        )
        core_lang_sanity_required_step_missing_payload["steps"] = []
        write_json(core_lang_sanity_required_step_missing_report, core_lang_sanity_required_step_missing_payload)
        core_lang_sanity_required_step_missing_proc = run_check(
            core_lang_sanity_required_step_missing_index,
            REQUIRED_STEPS_CORE_LANG,
            sanity_profile="core_lang",
            enforce_profile_step_contract=True,
        )
        if core_lang_sanity_required_step_missing_proc.returncode == 0:
            return fail("core_lang profile missing lang consistency sanity step case must fail")
        if f"fail code={CODES['REQUIRED_STEP_MISSING']}" not in core_lang_sanity_required_step_missing_proc.stderr:
            return fail(
                "core_lang profile missing lang consistency sanity step code mismatch: "
                f"err={core_lang_sanity_required_step_missing_proc.stderr}"
            )

        sync_profile_mismatch_index = build_index_case(root, "sync_profile_mismatch")
        sync_profile_mismatch_doc = json.loads(sync_profile_mismatch_index.read_text(encoding="utf-8"))
        sync_report = Path(str(sync_profile_mismatch_doc["reports"]["ci_sync_readiness"]))
        sync_report_doc = json.loads(sync_report.read_text(encoding="utf-8"))
        sync_report_doc["sanity_profile"] = "seamgrim"
        write_json(sync_report, sync_report_doc)
        sync_profile_mismatch_proc = run_check(
            sync_profile_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if sync_profile_mismatch_proc.returncode == 0:
            return fail("sync profile mismatch case must fail")
        if f"fail code={CODES['SYNC_PROFILE_MISMATCH']}" not in sync_profile_mismatch_proc.stderr:
            return fail(f"sync profile mismatch code mismatch: err={sync_profile_mismatch_proc.stderr}")

        result_overall_ok_type_index = build_index_case(root, "result_overall_ok_type")
        result_overall_ok_type_doc = json.loads(result_overall_ok_type_index.read_text(encoding="utf-8"))
        result_overall_ok_type_report = Path(str(result_overall_ok_type_doc["reports"]["ci_gate_result_json"]))
        result_overall_ok_type_result = json.loads(result_overall_ok_type_report.read_text(encoding="utf-8"))
        result_overall_ok_type_result["overall_ok"] = "true"
        write_json(result_overall_ok_type_report, result_overall_ok_type_result)
        result_overall_ok_type_proc = run_check(
            result_overall_ok_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_overall_ok_type_proc.returncode == 0:
            return fail("result overall_ok type case must fail")
        if f"fail code={CODES['RESULT_OVERALL_OK_TYPE']}" not in result_overall_ok_type_proc.stderr:
            return fail(f"result overall_ok type code mismatch: err={result_overall_ok_type_proc.stderr}")

        result_overall_ok_mismatch_index = build_index_case(root, "result_overall_ok_mismatch")
        result_overall_ok_mismatch_doc = json.loads(result_overall_ok_mismatch_index.read_text(encoding="utf-8"))
        result_overall_ok_mismatch_report = Path(str(result_overall_ok_mismatch_doc["reports"]["ci_gate_result_json"]))
        result_overall_ok_mismatch_result = json.loads(result_overall_ok_mismatch_report.read_text(encoding="utf-8"))
        result_overall_ok_mismatch_result["overall_ok"] = False
        write_json(result_overall_ok_mismatch_report, result_overall_ok_mismatch_result)
        result_overall_ok_mismatch_proc = run_check(
            result_overall_ok_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_overall_ok_mismatch_proc.returncode == 0:
            return fail("result overall_ok mismatch case must fail")
        if f"fail code={CODES['RESULT_OVERALL_OK_MISMATCH']}" not in result_overall_ok_mismatch_proc.stderr:
            return fail(f"result overall_ok mismatch code mismatch: err={result_overall_ok_mismatch_proc.stderr}")

        result_ok_type_index = build_index_case(root, "result_ok_type")
        result_ok_type_doc = json.loads(result_ok_type_index.read_text(encoding="utf-8"))
        result_ok_type_report = Path(str(result_ok_type_doc["reports"]["ci_gate_result_json"]))
        result_ok_type_result = json.loads(result_ok_type_report.read_text(encoding="utf-8"))
        result_ok_type_result["ok"] = "1"
        write_json(result_ok_type_report, result_ok_type_result)
        result_ok_type_proc = run_check(
            result_ok_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_ok_type_proc.returncode == 0:
            return fail("result ok type case must fail")
        if f"fail code={CODES['RESULT_OK_TYPE']}" not in result_ok_type_proc.stderr:
            return fail(f"result ok type code mismatch: err={result_ok_type_proc.stderr}")

        result_failed_steps_type_index = build_index_case(root, "result_failed_steps_type")
        result_failed_steps_type_doc = json.loads(result_failed_steps_type_index.read_text(encoding="utf-8"))
        result_failed_steps_type_report = Path(str(result_failed_steps_type_doc["reports"]["ci_gate_result_json"]))
        result_failed_steps_type_result = json.loads(result_failed_steps_type_report.read_text(encoding="utf-8"))
        result_failed_steps_type_result["failed_steps"] = "0"
        write_json(result_failed_steps_type_report, result_failed_steps_type_result)
        result_failed_steps_type_proc = run_check(
            result_failed_steps_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_failed_steps_type_proc.returncode == 0:
            return fail("result failed_steps type case must fail")
        if f"fail code={CODES['RESULT_FAILED_STEPS_TYPE']}" not in result_failed_steps_type_proc.stderr:
            return fail(f"result failed_steps type code mismatch: err={result_failed_steps_type_proc.stderr}")

        result_failed_steps_mismatch_index = build_index_case(root, "result_failed_steps_mismatch")
        result_failed_steps_mismatch_doc = json.loads(result_failed_steps_mismatch_index.read_text(encoding="utf-8"))
        result_failed_steps_mismatch_report = Path(str(result_failed_steps_mismatch_doc["reports"]["ci_gate_result_json"]))
        result_failed_steps_mismatch_result = json.loads(result_failed_steps_mismatch_report.read_text(encoding="utf-8"))
        result_failed_steps_mismatch_result["failed_steps"] = 1
        write_json(result_failed_steps_mismatch_report, result_failed_steps_mismatch_result)
        result_failed_steps_mismatch_proc = run_check(
            result_failed_steps_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_failed_steps_mismatch_proc.returncode == 0:
            return fail("result failed_steps mismatch case must fail")
        if f"fail code={CODES['RESULT_FAILED_STEPS_MISMATCH']}" not in result_failed_steps_mismatch_proc.stderr:
            return fail(f"result failed_steps mismatch code mismatch: err={result_failed_steps_mismatch_proc.stderr}")

        result_status_mismatch_index = build_index_case(root, "result_status_mismatch")
        result_status_mismatch_doc = json.loads(result_status_mismatch_index.read_text(encoding="utf-8"))
        result_status_mismatch_report = Path(str(result_status_mismatch_doc["reports"]["ci_gate_result_json"]))
        result_status_mismatch_result = json.loads(result_status_mismatch_report.read_text(encoding="utf-8"))
        result_status_mismatch_result["status"] = "fail"
        write_json(result_status_mismatch_report, result_status_mismatch_result)
        result_status_mismatch_proc = run_check(
            result_status_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_status_mismatch_proc.returncode == 0:
            return fail("result status mismatch case must fail")
        if f"fail code={CODES['RESULT_STATUS_MISMATCH']}" not in result_status_mismatch_proc.stderr:
            return fail(f"result status mismatch code mismatch: err={result_status_mismatch_proc.stderr}")

        result_aggregate_status_invalid_index = build_index_case(root, "result_aggregate_status_invalid")
        result_aggregate_status_invalid_doc = json.loads(result_aggregate_status_invalid_index.read_text(encoding="utf-8"))
        result_aggregate_status_invalid_report = Path(
            str(result_aggregate_status_invalid_doc["reports"]["ci_gate_result_json"])
        )
        result_aggregate_status_invalid_result = json.loads(result_aggregate_status_invalid_report.read_text(encoding="utf-8"))
        result_aggregate_status_invalid_result["aggregate_status"] = "unknown"
        write_json(result_aggregate_status_invalid_report, result_aggregate_status_invalid_result)
        result_aggregate_status_invalid_proc = run_check(
            result_aggregate_status_invalid_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_aggregate_status_invalid_proc.returncode == 0:
            return fail("result aggregate_status invalid case must fail")
        if (
            f"fail code={CODES['RESULT_AGGREGATE_STATUS_INVALID']}"
            not in result_aggregate_status_invalid_proc.stderr
        ):
            return fail(
                f"result aggregate_status invalid code mismatch: err={result_aggregate_status_invalid_proc.stderr}"
            )

        result_aggregate_status_mismatch_index = build_index_case(root, "result_aggregate_status_mismatch")
        result_aggregate_status_mismatch_doc = json.loads(
            result_aggregate_status_mismatch_index.read_text(encoding="utf-8")
        )
        result_aggregate_status_mismatch_report = Path(
            str(result_aggregate_status_mismatch_doc["reports"]["ci_gate_result_json"])
        )
        result_aggregate_status_mismatch_result = json.loads(
            result_aggregate_status_mismatch_report.read_text(encoding="utf-8")
        )
        result_aggregate_status_mismatch_result["aggregate_status"] = "fail"
        write_json(result_aggregate_status_mismatch_report, result_aggregate_status_mismatch_result)
        result_aggregate_status_mismatch_proc = run_check(
            result_aggregate_status_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_aggregate_status_mismatch_proc.returncode == 0:
            return fail("result aggregate_status mismatch case must fail")
        if (
            f"fail code={CODES['RESULT_AGGREGATE_STATUS_MISMATCH']}"
            not in result_aggregate_status_mismatch_proc.stderr
        ):
            return fail(
                f"result aggregate_status mismatch code mismatch: err={result_aggregate_status_mismatch_proc.stderr}"
            )

        result_ok_mismatch_index = build_index_case(root, "result_ok_mismatch")
        result_ok_mismatch_doc = json.loads(result_ok_mismatch_index.read_text(encoding="utf-8"))
        result_ok_mismatch_report = Path(str(result_ok_mismatch_doc["reports"]["ci_gate_result_json"]))
        result_ok_mismatch_result = json.loads(result_ok_mismatch_report.read_text(encoding="utf-8"))
        result_ok_mismatch_result["ok"] = False
        write_json(result_ok_mismatch_report, result_ok_mismatch_result)
        result_ok_mismatch_proc = run_check(
            result_ok_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_ok_mismatch_proc.returncode == 0:
            return fail("result ok mismatch case must fail")
        if f"fail code={CODES['RESULT_OK_MISMATCH']}" not in result_ok_mismatch_proc.stderr:
            return fail(f"result ok mismatch code mismatch: err={result_ok_mismatch_proc.stderr}")

        result_summary_line_path_mismatch_index = build_index_case(root, "result_summary_line_path_mismatch")
        result_summary_line_path_mismatch_doc = json.loads(result_summary_line_path_mismatch_index.read_text(encoding="utf-8"))
        result_summary_line_path_mismatch_report = Path(
            str(result_summary_line_path_mismatch_doc["reports"]["ci_gate_result_json"])
        )
        result_summary_line_path_mismatch_result = json.loads(
            result_summary_line_path_mismatch_report.read_text(encoding="utf-8")
        )
        result_summary_line_path_mismatch_result["summary_line_path"] = str(root / "mismatch" / "summary_line.txt")
        write_json(result_summary_line_path_mismatch_report, result_summary_line_path_mismatch_result)
        result_summary_line_path_mismatch_proc = run_check(
            result_summary_line_path_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_summary_line_path_mismatch_proc.returncode == 0:
            return fail("result summary_line_path mismatch case must fail")
        if f"fail code={CODES['RESULT_SUMMARY_LINE_PATH_MISMATCH']}" not in result_summary_line_path_mismatch_proc.stderr:
            return fail(
                f"result summary_line_path mismatch code mismatch: err={result_summary_line_path_mismatch_proc.stderr}"
            )

        result_summary_line_mismatch_index = build_index_case(root, "result_summary_line_mismatch")
        result_summary_line_mismatch_doc = json.loads(result_summary_line_mismatch_index.read_text(encoding="utf-8"))
        result_summary_line_mismatch_report = Path(str(result_summary_line_mismatch_doc["reports"]["ci_gate_result_json"]))
        result_summary_line_mismatch_result = json.loads(result_summary_line_mismatch_report.read_text(encoding="utf-8"))
        result_summary_line_mismatch_result["summary_line"] = "status=fail reason=mismatch failed_steps=1"
        write_json(result_summary_line_mismatch_report, result_summary_line_mismatch_result)
        result_summary_line_mismatch_proc = run_check(
            result_summary_line_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_summary_line_mismatch_proc.returncode == 0:
            return fail("result summary_line mismatch case must fail")
        if f"fail code={CODES['RESULT_SUMMARY_LINE_MISMATCH']}" not in result_summary_line_mismatch_proc.stderr:
            return fail(f"result summary_line mismatch code mismatch: err={result_summary_line_mismatch_proc.stderr}")

        result_gate_index_path_mismatch_index = build_index_case(root, "result_gate_index_path_mismatch")
        result_gate_index_path_mismatch_doc = json.loads(result_gate_index_path_mismatch_index.read_text(encoding="utf-8"))
        result_gate_index_path_mismatch_report = Path(
            str(result_gate_index_path_mismatch_doc["reports"]["ci_gate_result_json"])
        )
        result_gate_index_path_mismatch_result = json.loads(result_gate_index_path_mismatch_report.read_text(encoding="utf-8"))
        result_gate_index_path_mismatch_result["gate_index_path"] = str(root / "mismatch" / "ci_gate_report_index.detjson")
        write_json(result_gate_index_path_mismatch_report, result_gate_index_path_mismatch_result)
        result_gate_index_path_mismatch_proc = run_check(
            result_gate_index_path_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_gate_index_path_mismatch_proc.returncode == 0:
            return fail("result gate_index_path mismatch case must fail")
        if f"fail code={CODES['RESULT_GATE_INDEX_PATH_MISMATCH']}" not in result_gate_index_path_mismatch_proc.stderr:
            return fail(
                f"result gate_index_path mismatch code mismatch: err={result_gate_index_path_mismatch_proc.stderr}"
            )

        result_final_status_parse_path_mismatch_index = build_index_case(root, "result_final_status_parse_path_mismatch")
        result_final_status_parse_path_mismatch_doc = json.loads(
            result_final_status_parse_path_mismatch_index.read_text(encoding="utf-8")
        )
        result_final_status_parse_path_mismatch_report = Path(
            str(result_final_status_parse_path_mismatch_doc["reports"]["ci_gate_result_json"])
        )
        result_final_status_parse_path_mismatch_result = json.loads(
            result_final_status_parse_path_mismatch_report.read_text(encoding="utf-8")
        )
        result_final_status_parse_path_mismatch_result["final_status_parse_path"] = str(
            root / "mismatch" / "ci_gate_final_status_line_parse.detjson"
        )
        write_json(result_final_status_parse_path_mismatch_report, result_final_status_parse_path_mismatch_result)
        result_final_status_parse_path_mismatch_proc = run_check(
            result_final_status_parse_path_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_final_status_parse_path_mismatch_proc.returncode == 0:
            return fail("result final_status_parse_path mismatch case must fail")
        if (
            f"fail code={CODES['RESULT_FINAL_STATUS_PARSE_PATH_MISMATCH']}"
            not in result_final_status_parse_path_mismatch_proc.stderr
        ):
            return fail(
                "result final_status_parse_path mismatch code mismatch: "
                f"err={result_final_status_parse_path_mismatch_proc.stderr}"
            )

        final_parse_w107_mismatch_index = build_index_case(root, "final_parse_w107_mismatch")
        final_parse_w107_mismatch_doc = json.loads(final_parse_w107_mismatch_index.read_text(encoding="utf-8"))
        final_parse_w107_mismatch_report = Path(
            str(final_parse_w107_mismatch_doc["reports"]["final_status_parse_json"])
        )
        final_parse_w107_mismatch_payload = json.loads(final_parse_w107_mismatch_report.read_text(encoding="utf-8"))
        final_parse_w107_mismatch_payload["parsed"][
            "age5_full_real_w107_golden_index_selftest_active_cases"
        ] = "999"
        write_json(final_parse_w107_mismatch_report, final_parse_w107_mismatch_payload)
        final_parse_w107_mismatch_proc = run_check(
            final_parse_w107_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_w107_mismatch_proc.returncode == 0:
            return fail("final_parse w107 mismatch case must fail")
        if f"fail code={CODES['ARTIFACT_JSON_INVALID']}" not in final_parse_w107_mismatch_proc.stderr:
            return fail(
                "final_parse w107 mismatch code mismatch: "
                f"err={final_parse_w107_mismatch_proc.stderr}"
            )

        final_parse_w107_contract_mismatch_index = build_index_case(root, "final_parse_w107_contract_mismatch")
        final_parse_w107_contract_mismatch_doc = json.loads(final_parse_w107_contract_mismatch_index.read_text(encoding="utf-8"))
        final_parse_w107_contract_mismatch_report = Path(
            str(final_parse_w107_contract_mismatch_doc["reports"]["final_status_parse_json"])
        )
        final_parse_w107_contract_mismatch_payload = json.loads(
            final_parse_w107_contract_mismatch_report.read_text(encoding="utf-8")
        )
        final_parse_w107_contract_mismatch_payload["parsed"][
            "age5_full_real_w107_progress_contract_selftest_completed_checks"
        ] = "999"
        write_json(final_parse_w107_contract_mismatch_report, final_parse_w107_contract_mismatch_payload)
        final_parse_w107_contract_mismatch_proc = run_check(
            final_parse_w107_contract_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_w107_contract_mismatch_proc.returncode == 0:
            return fail("final_parse w107 contract mismatch case must fail")
        if f"fail code={CODES['ARTIFACT_JSON_INVALID']}" not in final_parse_w107_contract_mismatch_proc.stderr:
            return fail(
                "final_parse w107 contract mismatch code mismatch: "
                f"err={final_parse_w107_contract_mismatch_proc.stderr}"
            )

        result_w107_mismatch_index = build_index_case(root, "result_w107_mismatch")
        result_w107_mismatch_doc = json.loads(result_w107_mismatch_index.read_text(encoding="utf-8"))
        result_w107_mismatch_report = Path(str(result_w107_mismatch_doc["reports"]["ci_gate_result_json"]))
        result_w107_mismatch_payload = json.loads(result_w107_mismatch_report.read_text(encoding="utf-8"))
        result_w107_mismatch_payload["age5_full_real_w107_golden_index_selftest_active_cases"] = "999"
        write_json(result_w107_mismatch_report, result_w107_mismatch_payload)
        result_w107_mismatch_proc = run_check(
            result_w107_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_w107_mismatch_proc.returncode == 0:
            return fail("result w107 mismatch case must fail")
        if f"fail code={CODES['ARTIFACT_JSON_INVALID']}" not in result_w107_mismatch_proc.stderr:
            return fail(
                "result w107 mismatch code mismatch: "
                f"err={result_w107_mismatch_proc.stderr}"
            )

        result_w107_contract_mismatch_index = build_index_case(root, "result_w107_contract_mismatch")
        result_w107_contract_mismatch_doc = json.loads(result_w107_contract_mismatch_index.read_text(encoding="utf-8"))
        result_w107_contract_mismatch_report = Path(
            str(result_w107_contract_mismatch_doc["reports"]["ci_gate_result_json"])
        )
        result_w107_contract_mismatch_payload = json.loads(
            result_w107_contract_mismatch_report.read_text(encoding="utf-8")
        )
        result_w107_contract_mismatch_payload[
            "age5_full_real_w107_progress_contract_selftest_completed_checks"
        ] = "999"
        write_json(result_w107_contract_mismatch_report, result_w107_contract_mismatch_payload)
        result_w107_contract_mismatch_proc = run_check(
            result_w107_contract_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_w107_contract_mismatch_proc.returncode == 0:
            return fail("result w107 contract mismatch case must fail")
        if f"fail code={CODES['ARTIFACT_JSON_INVALID']}" not in result_w107_contract_mismatch_proc.stderr:
            return fail(
                "result w107 contract mismatch code mismatch: "
                f"err={result_w107_contract_mismatch_proc.stderr}"
            )

        final_parse_age1_immediate_proof_operation_contract_mismatch_index = build_index_case(
            root, "final_parse_age1_immediate_proof_operation_contract_mismatch"
        )
        final_parse_age1_immediate_proof_operation_contract_mismatch_doc = json.loads(
            final_parse_age1_immediate_proof_operation_contract_mismatch_index.read_text(encoding="utf-8")
        )
        final_parse_age1_immediate_proof_operation_contract_mismatch_report = Path(
            str(final_parse_age1_immediate_proof_operation_contract_mismatch_doc["reports"]["final_status_parse_json"])
        )
        final_parse_age1_immediate_proof_operation_contract_mismatch_payload = json.loads(
            final_parse_age1_immediate_proof_operation_contract_mismatch_report.read_text(encoding="utf-8")
        )
        final_parse_age1_immediate_proof_operation_contract_mismatch_payload["parsed"][
            "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks"
        ] = "999"
        write_json(
            final_parse_age1_immediate_proof_operation_contract_mismatch_report,
            final_parse_age1_immediate_proof_operation_contract_mismatch_payload,
        )
        final_parse_age1_immediate_proof_operation_contract_mismatch_proc = run_check(
            final_parse_age1_immediate_proof_operation_contract_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if final_parse_age1_immediate_proof_operation_contract_mismatch_proc.returncode == 0:
            return fail("final_parse age1 immediate proof operation contract mismatch case must fail")
        if (
            f"fail code={CODES['ARTIFACT_JSON_INVALID']}"
            not in final_parse_age1_immediate_proof_operation_contract_mismatch_proc.stderr
        ):
            return fail(
                "final_parse age1 immediate proof operation contract mismatch code mismatch: "
                f"err={final_parse_age1_immediate_proof_operation_contract_mismatch_proc.stderr}"
            )

        result_age1_immediate_proof_operation_contract_mismatch_index = build_index_case(
            root, "result_age1_immediate_proof_operation_contract_mismatch"
        )
        result_age1_immediate_proof_operation_contract_mismatch_doc = json.loads(
            result_age1_immediate_proof_operation_contract_mismatch_index.read_text(encoding="utf-8")
        )
        result_age1_immediate_proof_operation_contract_mismatch_report = Path(
            str(result_age1_immediate_proof_operation_contract_mismatch_doc["reports"]["ci_gate_result_json"])
        )
        result_age1_immediate_proof_operation_contract_mismatch_payload = json.loads(
            result_age1_immediate_proof_operation_contract_mismatch_report.read_text(encoding="utf-8")
        )
        result_age1_immediate_proof_operation_contract_mismatch_payload[
            "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks"
        ] = "999"
        write_json(
            result_age1_immediate_proof_operation_contract_mismatch_report,
            result_age1_immediate_proof_operation_contract_mismatch_payload,
        )
        result_age1_immediate_proof_operation_contract_mismatch_proc = run_check(
            result_age1_immediate_proof_operation_contract_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if result_age1_immediate_proof_operation_contract_mismatch_proc.returncode == 0:
            return fail("result age1 immediate proof operation contract mismatch case must fail")
        if (
            f"fail code={CODES['ARTIFACT_JSON_INVALID']}"
            not in result_age1_immediate_proof_operation_contract_mismatch_proc.stderr
        ):
            return fail(
                "result age1 immediate proof operation contract mismatch code mismatch: "
                f"err={result_age1_immediate_proof_operation_contract_mismatch_proc.stderr}"
            )

        triage_w107_mismatch_index = build_index_case(root, "triage_w107_mismatch")
        triage_w107_mismatch_doc = json.loads(triage_w107_mismatch_index.read_text(encoding="utf-8"))
        triage_w107_mismatch_report = Path(str(triage_w107_mismatch_doc["reports"]["ci_fail_triage_json"]))
        triage_w107_mismatch_payload = json.loads(triage_w107_mismatch_report.read_text(encoding="utf-8"))
        triage_w107_mismatch_payload["age5_full_real_w107_golden_index_selftest_active_cases"] = "999"
        write_json(triage_w107_mismatch_report, triage_w107_mismatch_payload)
        triage_w107_mismatch_proc = run_check(
            triage_w107_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_w107_mismatch_proc.returncode == 0:
            return fail("triage w107 mismatch case must fail")
        if f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}" not in triage_w107_mismatch_proc.stderr:
            return fail(
                "triage w107 mismatch code mismatch: "
                f"err={triage_w107_mismatch_proc.stderr}"
            )

        triage_w107_contract_mismatch_index = build_index_case(root, "triage_w107_contract_mismatch")
        triage_w107_contract_mismatch_doc = json.loads(triage_w107_contract_mismatch_index.read_text(encoding="utf-8"))
        triage_w107_contract_mismatch_report = Path(
            str(triage_w107_contract_mismatch_doc["reports"]["ci_fail_triage_json"])
        )
        triage_w107_contract_mismatch_payload = json.loads(
            triage_w107_contract_mismatch_report.read_text(encoding="utf-8")
        )
        triage_w107_contract_mismatch_payload[
            "age5_full_real_w107_progress_contract_selftest_completed_checks"
        ] = "999"
        write_json(triage_w107_contract_mismatch_report, triage_w107_contract_mismatch_payload)
        triage_w107_contract_mismatch_proc = run_check(
            triage_w107_contract_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_w107_contract_mismatch_proc.returncode == 0:
            return fail("triage w107 contract mismatch case must fail")
        if f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}" not in triage_w107_contract_mismatch_proc.stderr:
            return fail(
                "triage w107 contract mismatch code mismatch: "
                f"err={triage_w107_contract_mismatch_proc.stderr}"
            )

        triage_age1_immediate_proof_operation_contract_mismatch_index = build_index_case(
            root, "triage_age1_immediate_proof_operation_contract_mismatch"
        )
        triage_age1_immediate_proof_operation_contract_mismatch_doc = json.loads(
            triage_age1_immediate_proof_operation_contract_mismatch_index.read_text(encoding="utf-8")
        )
        triage_age1_immediate_proof_operation_contract_mismatch_report = Path(
            str(triage_age1_immediate_proof_operation_contract_mismatch_doc["reports"]["ci_fail_triage_json"])
        )
        triage_age1_immediate_proof_operation_contract_mismatch_payload = json.loads(
            triage_age1_immediate_proof_operation_contract_mismatch_report.read_text(encoding="utf-8")
        )
        triage_age1_immediate_proof_operation_contract_mismatch_payload[
            "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks"
        ] = "999"
        write_json(
            triage_age1_immediate_proof_operation_contract_mismatch_report,
            triage_age1_immediate_proof_operation_contract_mismatch_payload,
        )
        triage_age1_immediate_proof_operation_contract_mismatch_proc = run_check(
            triage_age1_immediate_proof_operation_contract_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_age1_immediate_proof_operation_contract_mismatch_proc.returncode == 0:
            return fail("triage age1 immediate proof operation contract mismatch case must fail")
        if (
            f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}"
            not in triage_age1_immediate_proof_operation_contract_mismatch_proc.stderr
        ):
            return fail(
                "triage age1 immediate proof operation contract mismatch code mismatch: "
                f"err={triage_age1_immediate_proof_operation_contract_mismatch_proc.stderr}"
            )

        badge_status_mismatch_index = build_index_case(root, "badge_status_mismatch")
        badge_status_mismatch_doc = json.loads(badge_status_mismatch_index.read_text(encoding="utf-8"))
        badge_status_mismatch_report = Path(str(badge_status_mismatch_doc["reports"]["ci_gate_badge_json"]))
        badge_status_mismatch_badge = json.loads(badge_status_mismatch_report.read_text(encoding="utf-8"))
        badge_status_mismatch_badge["status"] = "fail"
        write_json(badge_status_mismatch_report, badge_status_mismatch_badge)
        badge_status_mismatch_proc = run_check(
            badge_status_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if badge_status_mismatch_proc.returncode == 0:
            return fail("badge status mismatch case must fail")
        if f"fail code={CODES['BADGE_STATUS_MISMATCH']}" not in badge_status_mismatch_proc.stderr:
            return fail(f"badge status mismatch code mismatch: err={badge_status_mismatch_proc.stderr}")

        badge_ok_type_index = build_index_case(root, "badge_ok_type")
        badge_ok_type_doc = json.loads(badge_ok_type_index.read_text(encoding="utf-8"))
        badge_ok_type_report = Path(str(badge_ok_type_doc["reports"]["ci_gate_badge_json"]))
        badge_ok_type_badge = json.loads(badge_ok_type_report.read_text(encoding="utf-8"))
        badge_ok_type_badge["ok"] = "1"
        write_json(badge_ok_type_report, badge_ok_type_badge)
        badge_ok_type_proc = run_check(
            badge_ok_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if badge_ok_type_proc.returncode == 0:
            return fail("badge ok type case must fail")
        if f"fail code={CODES['BADGE_OK_TYPE']}" not in badge_ok_type_proc.stderr:
            return fail(f"badge ok type code mismatch: err={badge_ok_type_proc.stderr}")

        badge_ok_mismatch_index = build_index_case(root, "badge_ok_mismatch")
        badge_ok_mismatch_doc = json.loads(badge_ok_mismatch_index.read_text(encoding="utf-8"))
        badge_ok_mismatch_report = Path(str(badge_ok_mismatch_doc["reports"]["ci_gate_badge_json"]))
        badge_ok_mismatch_badge = json.loads(badge_ok_mismatch_report.read_text(encoding="utf-8"))
        badge_ok_mismatch_badge["ok"] = False
        write_json(badge_ok_mismatch_report, badge_ok_mismatch_badge)
        badge_ok_mismatch_proc = run_check(
            badge_ok_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if badge_ok_mismatch_proc.returncode == 0:
            return fail("badge ok mismatch case must fail")
        if f"fail code={CODES['BADGE_OK_MISMATCH']}" not in badge_ok_mismatch_proc.stderr:
            return fail(f"badge ok mismatch code mismatch: err={badge_ok_mismatch_proc.stderr}")

        badge_result_path_mismatch_index = build_index_case(root, "badge_result_path_mismatch")
        badge_result_path_mismatch_doc = json.loads(badge_result_path_mismatch_index.read_text(encoding="utf-8"))
        badge_result_path_mismatch_report = Path(str(badge_result_path_mismatch_doc["reports"]["ci_gate_badge_json"]))
        badge_result_path_mismatch_badge = json.loads(badge_result_path_mismatch_report.read_text(encoding="utf-8"))
        badge_result_path_mismatch_badge["result_path"] = str(root / "mismatch" / "ci_gate_result.detjson")
        write_json(badge_result_path_mismatch_report, badge_result_path_mismatch_badge)
        badge_result_path_mismatch_proc = run_check(
            badge_result_path_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if badge_result_path_mismatch_proc.returncode == 0:
            return fail("badge result_path mismatch case must fail")
        if f"fail code={CODES['BADGE_RESULT_PATH_MISMATCH']}" not in badge_result_path_mismatch_proc.stderr:
            return fail(f"badge result_path mismatch code mismatch: err={badge_result_path_mismatch_proc.stderr}")

        triage_mutation_cases: tuple[tuple[str, object, object, str, bool], ...] = (
            # triage status mismatch case must fail
            ("triage_status_mismatch", "status", "fail", CODES["TRIAGE_STATUS_MISMATCH"], False),
            # triage reason mismatch case must fail
            ("triage_reason_mismatch", "reason", "different_reason", CODES["TRIAGE_REASON_MISMATCH"], False),
            ("triage_failed_steps_count_missing", "failed_steps_count", None, CODES["TRIAGE_ARTIFACTS_MISSING"], True),
            ("triage_failed_steps_type", "failed_steps", {}, CODES["TRIAGE_ARTIFACTS_MISSING"], True),
            ("triage_failed_steps_count_mismatch", "failed_steps_count", 1, CODES["TRIAGE_ARTIFACTS_MISSING"], True),
            (
                "triage_failed_step_detail_rows_count_type",
                "failed_step_detail_rows_count",
                "bad",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
                True,
            ),
            (
                "triage_failed_step_logs_rows_count_type",
                "failed_step_logs_rows_count",
                "bad",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
                True,
            ),
            (
                "triage_failed_step_detail_order_type",
                "failed_step_detail_order",
                "bad",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
                True,
            ),
            (
                "triage_failed_step_logs_order_type",
                "failed_step_logs_order",
                "bad",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
                True,
            ),
        )
        for case_name, field_name, field_value, expected_code, use_fail_fixture in triage_mutation_cases:
            if field_value is None:
                error = run_triage_mutation_expect_fail(
                    root,
                    case_name=case_name,
                    mutator=lambda triage_doc, key=field_name: triage_doc.pop(str(key), None),
                    expected_code=str(expected_code),
                    use_fail_fixture=use_fail_fixture,
                )
            else:
                error = run_triage_mutation_expect_fail(
                    root,
                    case_name=case_name,
                    mutator=lambda triage_doc, key=field_name, value=field_value: triage_doc.__setitem__(str(key), value),
                    expected_code=str(expected_code),
                    use_fail_fixture=use_fail_fixture,
                )
            if error is not None:
                return fail(error)

        fail_summary_contract_ok_index = build_index_case(root, "fail_summary_contract_ok")
        configure_case_as_fail(fail_summary_contract_ok_index)
        fail_summary_contract_ok_proc = run_check(
            fail_summary_contract_ok_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if fail_summary_contract_ok_proc.returncode != 0:
            return fail(
                "fail summary contract baseline case must pass: "
                f"out={fail_summary_contract_ok_proc.stdout} err={fail_summary_contract_ok_proc.stderr}"
            )

        summary_failed_step_detail_missing_index = build_index_case(root, "summary_failed_step_detail_missing")
        configure_case_as_fail(summary_failed_step_detail_missing_index)
        summary_failed_step_detail_missing_doc = json.loads(
            summary_failed_step_detail_missing_index.read_text(encoding="utf-8")
        )
        summary_failed_step_detail_missing_path = Path(
            str(summary_failed_step_detail_missing_doc["reports"]["summary"])
        )
        detail_removed = False
        filtered_summary_lines: list[str] = []
        for line in summary_failed_step_detail_missing_path.read_text(encoding="utf-8").splitlines():
            if (not detail_removed) and "failed_step_detail=" in line:
                detail_removed = True
                continue
            filtered_summary_lines.append(line)
        write_text(summary_failed_step_detail_missing_path, "\n".join(filtered_summary_lines))
        summary_failed_step_detail_missing_proc = run_check(
            summary_failed_step_detail_missing_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if summary_failed_step_detail_missing_proc.returncode == 0:
            return fail("summary failed_step_detail missing case must fail")
        if (
            f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}"
            not in summary_failed_step_detail_missing_proc.stderr
        ):
            return fail(
                "summary failed_step_detail missing code mismatch: "
                f"err={summary_failed_step_detail_missing_proc.stderr}"
            )

        summary_failed_step_logs_missing_index = build_index_case(root, "summary_failed_step_logs_missing")
        configure_case_as_fail(summary_failed_step_logs_missing_index)
        summary_failed_step_logs_missing_doc = json.loads(
            summary_failed_step_logs_missing_index.read_text(encoding="utf-8")
        )
        summary_failed_step_logs_missing_path = Path(
            str(summary_failed_step_logs_missing_doc["reports"]["summary"])
        )
        logs_removed = False
        filtered_summary_lines = []
        for line in summary_failed_step_logs_missing_path.read_text(encoding="utf-8").splitlines():
            if (not logs_removed) and "failed_step_logs=" in line:
                logs_removed = True
                continue
            filtered_summary_lines.append(line)
        write_text(summary_failed_step_logs_missing_path, "\n".join(filtered_summary_lines))
        summary_failed_step_logs_missing_proc = run_check(
            summary_failed_step_logs_missing_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if summary_failed_step_logs_missing_proc.returncode == 0:
            return fail("summary failed_step_logs missing case must fail")
        if (
            f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}"
            not in summary_failed_step_logs_missing_proc.stderr
        ):
            return fail(
                "summary failed_step_logs missing code mismatch: "
                f"err={summary_failed_step_logs_missing_proc.stderr}"
            )

        summary_failed_step_detail_order_mismatch_index = build_index_case(
            root, "summary_failed_step_detail_order_mismatch"
        )
        configure_case_as_fail(summary_failed_step_detail_order_mismatch_index)
        summary_failed_step_detail_order_mismatch_doc = json.loads(
            summary_failed_step_detail_order_mismatch_index.read_text(encoding="utf-8")
        )
        summary_failed_step_detail_order_mismatch_path = Path(
            str(summary_failed_step_detail_order_mismatch_doc["reports"]["summary"])
        )
        summary_failed_step_detail_order_lines = summary_failed_step_detail_order_mismatch_path.read_text(
            encoding="utf-8"
        ).splitlines()
        detail_line_indexes = [
            idx for idx, line in enumerate(summary_failed_step_detail_order_lines) if "failed_step_detail=" in line
        ]
        log_line_indexes = [
            idx for idx, line in enumerate(summary_failed_step_detail_order_lines) if "failed_step_logs=" in line
        ]
        if len(detail_line_indexes) < 2:
            return fail("summary failed_step_detail order mismatch fixture requires >=2 detail rows")
        if len(log_line_indexes) < 2:
            return fail("summary failed_step_detail order mismatch fixture requires >=2 log rows")
        first_detail_idx = detail_line_indexes[0]
        second_detail_idx = detail_line_indexes[1]
        first_log_idx = log_line_indexes[0]
        second_log_idx = log_line_indexes[1]
        if first_log_idx != first_detail_idx + 1 or second_log_idx != second_detail_idx + 1:
            return fail("summary failed_step_detail order mismatch fixture expects detail/log adjacency")
        prefix = summary_failed_step_detail_order_lines[:first_detail_idx]
        block1 = summary_failed_step_detail_order_lines[first_detail_idx : first_log_idx + 1]
        middle = summary_failed_step_detail_order_lines[first_log_idx + 1 : second_detail_idx]
        block2 = summary_failed_step_detail_order_lines[second_detail_idx : second_log_idx + 1]
        suffix = summary_failed_step_detail_order_lines[second_log_idx + 1 :]
        summary_failed_step_detail_order_lines = (
            prefix
            + block2
            + middle
            + block1
            + suffix
        )
        write_text(
            summary_failed_step_detail_order_mismatch_path,
            "\n".join(summary_failed_step_detail_order_lines),
        )
        summary_failed_step_detail_order_mismatch_proc = run_check(
            summary_failed_step_detail_order_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if summary_failed_step_detail_order_mismatch_proc.returncode == 0:
            return fail("summary failed_step_detail order mismatch case must fail")
        if (
            f"fail code={CODES['SUMMARY_VALUE_INVALID']}"
            not in summary_failed_step_detail_order_mismatch_proc.stderr
        ):
            return fail(
                "summary failed_step_detail order mismatch code mismatch: "
                f"err={summary_failed_step_detail_order_mismatch_proc.stderr}"
            )

        summary_failed_step_logs_order_mismatch_index = build_index_case(
            root, "summary_failed_step_logs_order_mismatch"
        )
        configure_case_as_fail(summary_failed_step_logs_order_mismatch_index)
        summary_failed_step_logs_order_mismatch_doc = json.loads(
            summary_failed_step_logs_order_mismatch_index.read_text(encoding="utf-8")
        )
        summary_failed_step_logs_order_mismatch_path = Path(
            str(summary_failed_step_logs_order_mismatch_doc["reports"]["summary"])
        )
        summary_failed_step_logs_order_lines = summary_failed_step_logs_order_mismatch_path.read_text(
            encoding="utf-8"
        ).splitlines()
        log_line_indexes = [
            idx for idx, line in enumerate(summary_failed_step_logs_order_lines) if "failed_step_logs=" in line
        ]
        if len(log_line_indexes) < 2:
            return fail("summary failed_step_logs order mismatch fixture requires >=2 log rows")
        first_log_idx = log_line_indexes[0]
        second_log_idx = log_line_indexes[1]
        first_log_line = summary_failed_step_logs_order_lines.pop(first_log_idx)
        summary_failed_step_logs_order_lines.insert(second_log_idx, first_log_line)
        write_text(
            summary_failed_step_logs_order_mismatch_path,
            "\n".join(summary_failed_step_logs_order_lines),
        )
        summary_failed_step_logs_order_mismatch_proc = run_check(
            summary_failed_step_logs_order_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if summary_failed_step_logs_order_mismatch_proc.returncode == 0:
            return fail("summary failed_step_logs order mismatch case must fail")
        if (
            f"fail code={CODES['SUMMARY_VALUE_INVALID']}"
            not in summary_failed_step_logs_order_mismatch_proc.stderr
        ):
            return fail(
                "summary failed_step_logs order mismatch code mismatch: "
                f"err={summary_failed_step_logs_order_mismatch_proc.stderr}"
            )

        summary_failed_step_rows_interleaved_index = build_index_case(
            root, "summary_failed_step_rows_interleaved"
        )
        configure_case_as_fail(summary_failed_step_rows_interleaved_index)
        summary_failed_step_rows_interleaved_doc = json.loads(
            summary_failed_step_rows_interleaved_index.read_text(encoding="utf-8")
        )
        summary_failed_step_rows_interleaved_path = Path(
            str(summary_failed_step_rows_interleaved_doc["reports"]["summary"])
        )
        summary_failed_step_rows_interleaved_lines = summary_failed_step_rows_interleaved_path.read_text(
            encoding="utf-8"
        ).splitlines()
        detail_line_indexes = [
            idx for idx, line in enumerate(summary_failed_step_rows_interleaved_lines) if "failed_step_detail=" in line
        ]
        if len(detail_line_indexes) < 2:
            return fail("summary failed_step rows interleaved fixture requires >=2 detail rows")
        first_detail_idx = detail_line_indexes[0]
        second_detail_idx = detail_line_indexes[1]
        second_detail_line = summary_failed_step_rows_interleaved_lines.pop(second_detail_idx)
        summary_failed_step_rows_interleaved_lines.insert(first_detail_idx + 1, second_detail_line)
        write_text(
            summary_failed_step_rows_interleaved_path,
            "\n".join(summary_failed_step_rows_interleaved_lines),
        )
        summary_failed_step_rows_interleaved_proc = run_check(
            summary_failed_step_rows_interleaved_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if summary_failed_step_rows_interleaved_proc.returncode == 0:
            return fail("summary failed_step rows interleaved case must fail")
        if (
            f"fail code={CODES['SUMMARY_VALUE_INVALID']}"
            not in summary_failed_step_rows_interleaved_proc.stderr
        ):
            return fail(
                "summary failed_step rows interleaved code mismatch: "
                f"err={summary_failed_step_rows_interleaved_proc.stderr}"
            )

        summary_failed_step_logs_before_detail_index = build_index_case(
            root, "summary_failed_step_logs_before_detail"
        )
        configure_case_as_fail(summary_failed_step_logs_before_detail_index)
        summary_failed_step_logs_before_detail_doc = json.loads(
            summary_failed_step_logs_before_detail_index.read_text(encoding="utf-8")
        )
        summary_failed_step_logs_before_detail_path = Path(
            str(summary_failed_step_logs_before_detail_doc["reports"]["summary"])
        )
        summary_failed_step_logs_before_detail_lines = summary_failed_step_logs_before_detail_path.read_text(
            encoding="utf-8"
        ).splitlines()
        first_detail_idx = next(
            (idx for idx, line in enumerate(summary_failed_step_logs_before_detail_lines) if "failed_step_detail=" in line),
            -1,
        )
        first_log_idx = next(
            (idx for idx, line in enumerate(summary_failed_step_logs_before_detail_lines) if "failed_step_logs=" in line),
            -1,
        )
        if first_detail_idx < 0 or first_log_idx < 0:
            return fail("summary failed_step_logs before detail fixture requires detail/log rows")
        summary_failed_step_logs_before_detail_lines[first_detail_idx], summary_failed_step_logs_before_detail_lines[
            first_log_idx
        ] = (
            summary_failed_step_logs_before_detail_lines[first_log_idx],
            summary_failed_step_logs_before_detail_lines[first_detail_idx],
        )
        write_text(
            summary_failed_step_logs_before_detail_path,
            "\n".join(summary_failed_step_logs_before_detail_lines),
        )
        summary_failed_step_logs_before_detail_proc = run_check(
            summary_failed_step_logs_before_detail_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if summary_failed_step_logs_before_detail_proc.returncode == 0:
            return fail("summary failed_step_logs before detail case must fail")
        if (
            f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}"
            not in summary_failed_step_logs_before_detail_proc.stderr
        ):
            return fail(
                "summary failed_step_logs before detail code mismatch: "
                f"err={summary_failed_step_logs_before_detail_proc.stderr}"
            )

        summary_failed_step_detail_rc_mismatch_index = build_index_case(root, "summary_failed_step_detail_rc_mismatch")
        configure_case_as_fail(summary_failed_step_detail_rc_mismatch_index)
        summary_failed_step_detail_rc_mismatch_doc = json.loads(
            summary_failed_step_detail_rc_mismatch_index.read_text(encoding="utf-8")
        )
        summary_failed_step_detail_rc_mismatch_triage_path = Path(
            str(summary_failed_step_detail_rc_mismatch_doc["reports"]["ci_fail_triage_json"])
        )
        summary_failed_step_detail_rc_mismatch_triage = json.loads(
            summary_failed_step_detail_rc_mismatch_triage_path.read_text(encoding="utf-8")
        )
        summary_failed_step_detail_rc_mismatch_triage["failed_steps"][0]["returncode"] = 2
        write_json(
            summary_failed_step_detail_rc_mismatch_triage_path,
            summary_failed_step_detail_rc_mismatch_triage,
        )
        summary_failed_step_detail_rc_mismatch_proc = run_check(
            summary_failed_step_detail_rc_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if summary_failed_step_detail_rc_mismatch_proc.returncode == 0:
            return fail("summary failed_step_detail rc mismatch case must fail")
        if (
            f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}"
            not in summary_failed_step_detail_rc_mismatch_proc.stderr
        ):
            return fail(
                "summary failed_step_detail rc mismatch code mismatch: "
                f"err={summary_failed_step_detail_rc_mismatch_proc.stderr}"
            )

        summary_failed_steps_missing_on_fail_index = build_index_case(root, "summary_failed_steps_missing_on_fail")
        configure_case_as_fail(summary_failed_steps_missing_on_fail_index)
        summary_failed_steps_missing_on_fail_doc = json.loads(
            summary_failed_steps_missing_on_fail_index.read_text(encoding="utf-8")
        )
        summary_failed_steps_missing_on_fail_path = Path(
            str(summary_failed_steps_missing_on_fail_doc["reports"]["summary"])
        )
        rewritten_summary_lines: list[str] = []
        for line in summary_failed_steps_missing_on_fail_path.read_text(encoding="utf-8").splitlines():
            if line.startswith("[ci-gate-summary] failed_steps="):
                rewritten_summary_lines.append("[ci-gate-summary] failed_steps=(none)")
                continue
            rewritten_summary_lines.append(line)
        write_text(summary_failed_steps_missing_on_fail_path, "\n".join(rewritten_summary_lines))
        summary_failed_steps_missing_on_fail_proc = run_check(
            summary_failed_steps_missing_on_fail_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if summary_failed_steps_missing_on_fail_proc.returncode == 0:
            return fail("summary failed_steps missing on fail case must fail")
        if (
            f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}"
            not in summary_failed_steps_missing_on_fail_proc.stderr
        ):
            return fail(
                "summary failed_steps missing on fail code mismatch: "
                f"err={summary_failed_steps_missing_on_fail_proc.stderr}"
            )

        summary_failed_steps_after_detail_on_fail_index = build_index_case(
            root, "summary_failed_steps_after_detail_on_fail"
        )
        configure_case_as_fail(summary_failed_steps_after_detail_on_fail_index)
        summary_failed_steps_after_detail_on_fail_doc = json.loads(
            summary_failed_steps_after_detail_on_fail_index.read_text(encoding="utf-8")
        )
        summary_failed_steps_after_detail_on_fail_path = Path(
            str(summary_failed_steps_after_detail_on_fail_doc["reports"]["summary"])
        )
        summary_failed_steps_after_detail_lines = summary_failed_steps_after_detail_on_fail_path.read_text(
            encoding="utf-8"
        ).splitlines()
        failed_steps_line_idx = next(
            (idx for idx, line in enumerate(summary_failed_steps_after_detail_lines) if line.startswith("[ci-gate-summary] failed_steps=")),
            -1,
        )
        first_detail_idx = next(
            (idx for idx, line in enumerate(summary_failed_steps_after_detail_lines) if "failed_step_detail=" in line),
            -1,
        )
        if failed_steps_line_idx < 0 or first_detail_idx < 0:
            return fail("summary failed_steps after detail fixture requires failed_steps/detail rows")
        failed_steps_line = summary_failed_steps_after_detail_lines.pop(failed_steps_line_idx)
        if failed_steps_line_idx < first_detail_idx:
            first_detail_idx -= 1
        summary_failed_steps_after_detail_lines.insert(first_detail_idx + 1, failed_steps_line)
        write_text(
            summary_failed_steps_after_detail_on_fail_path,
            "\n".join(summary_failed_steps_after_detail_lines),
        )
        summary_failed_steps_after_detail_on_fail_proc = run_check(
            summary_failed_steps_after_detail_on_fail_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if summary_failed_steps_after_detail_on_fail_proc.returncode == 0:
            return fail("summary failed_steps after detail on fail case must fail")
        if (
            f"fail code={CODES['SUMMARY_VALUE_INVALID']}"
            not in summary_failed_steps_after_detail_on_fail_proc.stderr
        ):
            return fail(
                "summary failed_steps after detail on fail code mismatch: "
                f"err={summary_failed_steps_after_detail_on_fail_proc.stderr}"
            )

        summary_failed_steps_duplicate_on_fail_index = build_index_case(
            root, "summary_failed_steps_duplicate_on_fail"
        )
        configure_case_as_fail(summary_failed_steps_duplicate_on_fail_index)
        summary_failed_steps_duplicate_on_fail_doc = json.loads(
            summary_failed_steps_duplicate_on_fail_index.read_text(encoding="utf-8")
        )
        summary_failed_steps_duplicate_on_fail_path = Path(
            str(summary_failed_steps_duplicate_on_fail_doc["reports"]["summary"])
        )
        summary_failed_steps_duplicate_lines: list[str] = []
        replaced_failed_steps_line = False
        for line in summary_failed_steps_duplicate_on_fail_path.read_text(encoding="utf-8").splitlines():
            if (not replaced_failed_steps_line) and line.startswith("[ci-gate-summary] failed_steps="):
                summary_failed_steps_duplicate_lines.append(
                    "[ci-gate-summary] failed_steps=ci_sanity_gate,ci_sanity_gate"
                )
                replaced_failed_steps_line = True
                continue
            summary_failed_steps_duplicate_lines.append(line)
        write_text(
            summary_failed_steps_duplicate_on_fail_path,
            "\n".join(summary_failed_steps_duplicate_lines),
        )
        summary_failed_steps_duplicate_on_fail_proc = run_check(
            summary_failed_steps_duplicate_on_fail_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if summary_failed_steps_duplicate_on_fail_proc.returncode == 0:
            return fail("summary failed_steps duplicate on fail case must fail")
        if (
            f"fail code={CODES['SUMMARY_VALUE_INVALID']}"
            not in summary_failed_steps_duplicate_on_fail_proc.stderr
        ):
            return fail(
                "summary failed_steps duplicate on fail code mismatch: "
                f"err={summary_failed_steps_duplicate_on_fail_proc.stderr}"
            )

        summary_failed_steps_order_mismatch_on_fail_index = build_index_case(
            root, "summary_failed_steps_order_mismatch_on_fail"
        )
        configure_case_as_fail(summary_failed_steps_order_mismatch_on_fail_index)
        summary_failed_steps_order_mismatch_on_fail_doc = json.loads(
            summary_failed_steps_order_mismatch_on_fail_index.read_text(encoding="utf-8")
        )
        summary_failed_steps_order_mismatch_on_fail_path = Path(
            str(summary_failed_steps_order_mismatch_on_fail_doc["reports"]["summary"])
        )
        summary_failed_steps_order_mismatch_lines: list[str] = []
        replaced_failed_steps_line = False
        for line in summary_failed_steps_order_mismatch_on_fail_path.read_text(encoding="utf-8").splitlines():
            if (not replaced_failed_steps_line) and line.startswith("[ci-gate-summary] failed_steps="):
                summary_failed_steps_order_mismatch_lines.append(
                    "[ci-gate-summary] failed_steps=ci_gate_report_index_diagnostics_check,ci_emit_artifacts_required_post_summary_check"
                )
                replaced_failed_steps_line = True
                continue
            summary_failed_steps_order_mismatch_lines.append(line)
        write_text(
            summary_failed_steps_order_mismatch_on_fail_path,
            "\n".join(summary_failed_steps_order_mismatch_lines),
        )
        summary_failed_steps_order_mismatch_on_fail_proc = run_check(
            summary_failed_steps_order_mismatch_on_fail_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if summary_failed_steps_order_mismatch_on_fail_proc.returncode == 0:
            return fail("summary failed_steps order mismatch on fail case must fail")
        if (
            f"fail code={CODES['SUMMARY_VALUE_INVALID']}"
            not in summary_failed_steps_order_mismatch_on_fail_proc.stderr
        ):
            return fail(
                "summary failed_steps order mismatch on fail code mismatch: "
                f"err={summary_failed_steps_order_mismatch_on_fail_proc.stderr}"
            )

        failed_steps_order_mismatch_from_index_on_fail_index = build_index_case(
            root, "failed_steps_order_mismatch_from_index_on_fail"
        )
        configure_case_as_fail(
            failed_steps_order_mismatch_from_index_on_fail_index,
            failed_step_ids=(
                "ci_gate_report_index_diagnostics_check",
                "ci_emit_artifacts_required_post_summary_check",
            ),
        )
        failed_steps_order_mismatch_from_index_on_fail_proc = run_check(
            failed_steps_order_mismatch_from_index_on_fail_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if failed_steps_order_mismatch_from_index_on_fail_proc.returncode == 0:
            return fail("failed_steps order mismatch from index on fail case must fail")
        if (
            f"fail code={CODES['SUMMARY_VALUE_INVALID']}"
            not in failed_steps_order_mismatch_from_index_on_fail_proc.stderr
        ):
            return fail(
                "failed_steps order mismatch from index on fail code mismatch: "
                f"err={failed_steps_order_mismatch_from_index_on_fail_proc.stderr}"
            )

        triage_failed_steps_order_mismatch_on_fail_index = build_index_case(
            root, "triage_failed_steps_order_mismatch_on_fail"
        )
        configure_case_as_fail(triage_failed_steps_order_mismatch_on_fail_index)
        triage_failed_steps_order_mismatch_on_fail_doc = json.loads(
            triage_failed_steps_order_mismatch_on_fail_index.read_text(encoding="utf-8")
        )
        triage_failed_steps_order_mismatch_on_fail_path = Path(
            str(triage_failed_steps_order_mismatch_on_fail_doc["reports"]["ci_fail_triage_json"])
        )
        triage_failed_steps_order_mismatch_on_fail_payload = json.loads(
            triage_failed_steps_order_mismatch_on_fail_path.read_text(encoding="utf-8")
        )
        triage_failed_steps_rows = triage_failed_steps_order_mismatch_on_fail_payload.get("failed_steps", [])
        if not isinstance(triage_failed_steps_rows, list) or len(triage_failed_steps_rows) < 2:
            return fail("triage failed_steps order mismatch fixture requires >=2 triage failed_steps rows")
        triage_failed_steps_rows[0], triage_failed_steps_rows[1] = (
            triage_failed_steps_rows[1],
            triage_failed_steps_rows[0],
        )
        triage_failed_steps_order_mismatch_on_fail_payload["failed_steps"] = triage_failed_steps_rows
        write_json(
            triage_failed_steps_order_mismatch_on_fail_path,
            triage_failed_steps_order_mismatch_on_fail_payload,
        )
        triage_failed_steps_order_mismatch_on_fail_proc = run_check(
            triage_failed_steps_order_mismatch_on_fail_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_failed_steps_order_mismatch_on_fail_proc.returncode == 0:
            return fail("triage failed_steps order mismatch on fail case must fail")
        if (
            f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}"
            not in triage_failed_steps_order_mismatch_on_fail_proc.stderr
        ):
            return fail(
                "triage failed_steps order mismatch on fail code mismatch: "
                f"err={triage_failed_steps_order_mismatch_on_fail_proc.stderr}"
            )

        brief_failed_steps_present_on_pass_index = build_index_case(root, "brief_failed_steps_present_on_pass")
        brief_failed_steps_present_on_pass_doc = json.loads(
            brief_failed_steps_present_on_pass_index.read_text(encoding="utf-8")
        )
        brief_failed_steps_present_on_pass_path = Path(
            str(brief_failed_steps_present_on_pass_doc["reports"]["ci_fail_brief_txt"])
        )
        brief_failed_steps_present_on_pass_text = brief_failed_steps_present_on_pass_path.read_text(encoding="utf-8")
        brief_failed_steps_present_on_pass_text = brief_failed_steps_present_on_pass_text.replace(
            "failed_steps_count=0", "failed_steps_count=1", 1
        )
        brief_failed_steps_present_on_pass_text = brief_failed_steps_present_on_pass_text.replace(
            "failed_steps=-", "failed_steps=ci_sanity_gate", 1
        )
        write_text(brief_failed_steps_present_on_pass_path, brief_failed_steps_present_on_pass_text)
        brief_failed_steps_present_on_pass_proc = run_check(
            brief_failed_steps_present_on_pass_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if brief_failed_steps_present_on_pass_proc.returncode == 0:
            return fail("brief failed_steps present on pass case must fail")
        if (
            f"fail code={CODES['SUMMARY_VALUE_INVALID']}"
            not in brief_failed_steps_present_on_pass_proc.stderr
        ):
            return fail(
                "brief failed_steps present on pass code mismatch: "
                f"err={brief_failed_steps_present_on_pass_proc.stderr}"
            )

        brief_failed_steps_missing_on_fail_index = build_index_case(root, "brief_failed_steps_missing_on_fail")
        configure_case_as_fail(brief_failed_steps_missing_on_fail_index)
        brief_failed_steps_missing_on_fail_doc = json.loads(
            brief_failed_steps_missing_on_fail_index.read_text(encoding="utf-8")
        )
        brief_failed_steps_missing_on_fail_path = Path(
            str(brief_failed_steps_missing_on_fail_doc["reports"]["ci_fail_brief_txt"])
        )
        brief_failed_steps_missing_on_fail_tokens = (
            brief_failed_steps_missing_on_fail_path.read_text(encoding="utf-8").split()
        )
        rewritten_brief_tokens: list[str] = []
        replaced_failed_steps_token = False
        for token in brief_failed_steps_missing_on_fail_tokens:
            if (not replaced_failed_steps_token) and token.startswith("failed_steps="):
                rewritten_brief_tokens.append("failed_steps=-")
                replaced_failed_steps_token = True
                continue
            rewritten_brief_tokens.append(token)
        brief_failed_steps_missing_on_fail_text = " ".join(rewritten_brief_tokens)
        write_text(brief_failed_steps_missing_on_fail_path, brief_failed_steps_missing_on_fail_text)
        brief_failed_steps_missing_on_fail_proc = run_check(
            brief_failed_steps_missing_on_fail_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if brief_failed_steps_missing_on_fail_proc.returncode == 0:
            return fail("brief failed_steps missing on fail case must fail")
        if (
            f"fail code={CODES['SUMMARY_VALUE_INVALID']}"
            not in brief_failed_steps_missing_on_fail_proc.stderr
        ):
            return fail(
                "brief failed_steps missing on fail code mismatch: "
                f"err={brief_failed_steps_missing_on_fail_proc.stderr}"
            )

        brief_duplicate_key_on_fail_index = build_index_case(root, "brief_duplicate_key_on_fail")
        configure_case_as_fail(brief_duplicate_key_on_fail_index)
        brief_duplicate_key_on_fail_doc = json.loads(
            brief_duplicate_key_on_fail_index.read_text(encoding="utf-8")
        )
        brief_duplicate_key_on_fail_path = Path(
            str(brief_duplicate_key_on_fail_doc["reports"]["ci_fail_brief_txt"])
        )
        brief_duplicate_key_on_fail_text = brief_duplicate_key_on_fail_path.read_text(encoding="utf-8").strip()
        brief_duplicate_key_on_fail_text += (
            " failed_steps=ci_emit_artifacts_required_post_summary_check,ci_gate_report_index_diagnostics_check"
        )
        write_text(brief_duplicate_key_on_fail_path, brief_duplicate_key_on_fail_text)
        brief_duplicate_key_on_fail_proc = run_check(
            brief_duplicate_key_on_fail_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if brief_duplicate_key_on_fail_proc.returncode == 0:
            return fail("brief duplicate key on fail case must fail")
        if (
            f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}"
            not in brief_duplicate_key_on_fail_proc.stderr
        ):
            return fail(
                "brief duplicate key on fail code mismatch: "
                f"err={brief_duplicate_key_on_fail_proc.stderr}"
            )

        brief_failed_steps_order_mismatch_on_fail_index = build_index_case(
            root, "brief_failed_steps_order_mismatch_on_fail"
        )
        configure_case_as_fail(brief_failed_steps_order_mismatch_on_fail_index)
        brief_failed_steps_order_mismatch_on_fail_doc = json.loads(
            brief_failed_steps_order_mismatch_on_fail_index.read_text(encoding="utf-8")
        )
        brief_failed_steps_order_mismatch_on_fail_path = Path(
            str(brief_failed_steps_order_mismatch_on_fail_doc["reports"]["ci_fail_brief_txt"])
        )
        brief_failed_steps_order_mismatch_tokens = (
            brief_failed_steps_order_mismatch_on_fail_path.read_text(encoding="utf-8").split()
        )
        rewritten_brief_tokens = []
        replaced_failed_steps_token = False
        for token in brief_failed_steps_order_mismatch_tokens:
            if (not replaced_failed_steps_token) and token.startswith("failed_steps="):
                rewritten_brief_tokens.append(
                    "failed_steps=ci_gate_report_index_diagnostics_check,ci_emit_artifacts_required_post_summary_check"
                )
                replaced_failed_steps_token = True
                continue
            rewritten_brief_tokens.append(token)
        brief_failed_steps_order_mismatch_on_fail_text = " ".join(rewritten_brief_tokens)
        write_text(
            brief_failed_steps_order_mismatch_on_fail_path,
            brief_failed_steps_order_mismatch_on_fail_text,
        )
        brief_failed_steps_order_mismatch_on_fail_proc = run_check(
            brief_failed_steps_order_mismatch_on_fail_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if brief_failed_steps_order_mismatch_on_fail_proc.returncode == 0:
            return fail("brief failed_steps order mismatch on fail case must fail")
        if (
            f"fail code={CODES['SUMMARY_VALUE_INVALID']}"
            not in brief_failed_steps_order_mismatch_on_fail_proc.stderr
        ):
            return fail(
                "brief failed_steps order mismatch on fail code mismatch: "
                f"err={brief_failed_steps_order_mismatch_on_fail_proc.stderr}"
            )

        brief_failed_steps_count_mismatch_on_fail_index = build_index_case(
            root, "brief_failed_steps_count_mismatch_on_fail"
        )
        configure_case_as_fail(brief_failed_steps_count_mismatch_on_fail_index)
        brief_failed_steps_count_mismatch_on_fail_doc = json.loads(
            brief_failed_steps_count_mismatch_on_fail_index.read_text(encoding="utf-8")
        )
        brief_failed_steps_count_mismatch_on_fail_path = Path(
            str(brief_failed_steps_count_mismatch_on_fail_doc["reports"]["ci_fail_brief_txt"])
        )
        brief_failed_steps_count_mismatch_tokens = (
            brief_failed_steps_count_mismatch_on_fail_path.read_text(encoding="utf-8").split()
        )
        rewritten_brief_tokens = []
        replaced_count_token = False
        for token in brief_failed_steps_count_mismatch_tokens:
            if (not replaced_count_token) and token.startswith("failed_steps_count="):
                rewritten_brief_tokens.append("failed_steps_count=999")
                replaced_count_token = True
                continue
            rewritten_brief_tokens.append(token)
        brief_failed_steps_count_mismatch_text = " ".join(rewritten_brief_tokens)
        write_text(brief_failed_steps_count_mismatch_on_fail_path, brief_failed_steps_count_mismatch_text)
        brief_failed_steps_count_mismatch_on_fail_proc = run_check(
            brief_failed_steps_count_mismatch_on_fail_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if brief_failed_steps_count_mismatch_on_fail_proc.returncode == 0:
            return fail("brief failed_steps_count mismatch on fail case must fail")
        if (
            f"fail code={CODES['SUMMARY_VALUE_INVALID']}"
            not in brief_failed_steps_count_mismatch_on_fail_proc.stderr
        ):
            return fail(
                "brief failed_steps_count mismatch on fail code mismatch: "
                f"err={brief_failed_steps_count_mismatch_on_fail_proc.stderr}"
            )

        brief_failed_steps_duplicate_on_fail_index = build_index_case(root, "brief_failed_steps_duplicate_on_fail")
        configure_case_as_fail(brief_failed_steps_duplicate_on_fail_index)
        brief_failed_steps_duplicate_on_fail_doc = json.loads(
            brief_failed_steps_duplicate_on_fail_index.read_text(encoding="utf-8")
        )
        brief_failed_steps_duplicate_on_fail_path = Path(
            str(brief_failed_steps_duplicate_on_fail_doc["reports"]["ci_fail_brief_txt"])
        )
        brief_failed_steps_duplicate_tokens = (
            brief_failed_steps_duplicate_on_fail_path.read_text(encoding="utf-8").split()
        )
        rewritten_brief_tokens = []
        replaced_count_token = False
        replaced_failed_steps_token = False
        for token in brief_failed_steps_duplicate_tokens:
            if (not replaced_count_token) and token.startswith("failed_steps_count="):
                rewritten_brief_tokens.append("failed_steps_count=2")
                replaced_count_token = True
                continue
            if (not replaced_failed_steps_token) and token.startswith("failed_steps="):
                rewritten_brief_tokens.append("failed_steps=ci_sanity_gate,ci_sanity_gate")
                replaced_failed_steps_token = True
                continue
            rewritten_brief_tokens.append(token)
        brief_failed_steps_duplicate_text = " ".join(rewritten_brief_tokens)
        write_text(brief_failed_steps_duplicate_on_fail_path, brief_failed_steps_duplicate_text)
        brief_failed_steps_duplicate_on_fail_proc = run_check(
            brief_failed_steps_duplicate_on_fail_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if brief_failed_steps_duplicate_on_fail_proc.returncode == 0:
            return fail("brief failed_steps duplicate on fail case must fail")
        if (
            f"fail code={CODES['SUMMARY_VALUE_INVALID']}"
            not in brief_failed_steps_duplicate_on_fail_proc.stderr
        ):
            return fail(
                "brief failed_steps duplicate on fail code mismatch: "
                f"err={brief_failed_steps_duplicate_on_fail_proc.stderr}"
            )

        brief_failed_steps_mismatch_on_fail_index = build_index_case(root, "brief_failed_steps_mismatch_on_fail")
        configure_case_as_fail(brief_failed_steps_mismatch_on_fail_index)
        brief_failed_steps_mismatch_on_fail_doc = json.loads(
            brief_failed_steps_mismatch_on_fail_index.read_text(encoding="utf-8")
        )
        brief_failed_steps_mismatch_on_fail_path = Path(
            str(brief_failed_steps_mismatch_on_fail_doc["reports"]["ci_fail_brief_txt"])
        )
        brief_failed_steps_mismatch_tokens = (
            brief_failed_steps_mismatch_on_fail_path.read_text(encoding="utf-8").split()
        )
        rewritten_brief_tokens = []
        replaced_failed_steps_token = False
        for token in brief_failed_steps_mismatch_tokens:
            if (not replaced_failed_steps_token) and token.startswith("failed_steps="):
                rewritten_brief_tokens.append("failed_steps=__bogus_failed_step__")
                replaced_failed_steps_token = True
                continue
            rewritten_brief_tokens.append(token)
        brief_failed_steps_mismatch_text = " ".join(rewritten_brief_tokens)
        write_text(brief_failed_steps_mismatch_on_fail_path, brief_failed_steps_mismatch_text)
        brief_failed_steps_mismatch_on_fail_proc = run_check(
            brief_failed_steps_mismatch_on_fail_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if brief_failed_steps_mismatch_on_fail_proc.returncode == 0:
            return fail("brief failed_steps mismatch on fail case must fail")
        if (
            f"fail code={CODES['SUMMARY_VALUE_INVALID']}"
            not in brief_failed_steps_mismatch_on_fail_proc.stderr
        ):
            return fail(
                "brief failed_steps mismatch on fail code mismatch: "
                f"err={brief_failed_steps_mismatch_on_fail_proc.stderr}"
            )

        triage_failed_step_returncode_zero_index = build_index_case(root, "triage_failed_step_returncode_zero")
        configure_case_as_fail(triage_failed_step_returncode_zero_index)
        triage_failed_step_returncode_zero_doc = json.loads(
            triage_failed_step_returncode_zero_index.read_text(encoding="utf-8")
        )
        triage_failed_step_returncode_zero_triage_path = Path(
            str(triage_failed_step_returncode_zero_doc["reports"]["ci_fail_triage_json"])
        )
        triage_failed_step_returncode_zero_triage = json.loads(
            triage_failed_step_returncode_zero_triage_path.read_text(encoding="utf-8")
        )
        triage_failed_step_returncode_zero_triage["failed_steps"][0]["returncode"] = 0
        write_json(
            triage_failed_step_returncode_zero_triage_path,
            triage_failed_step_returncode_zero_triage,
        )
        triage_failed_step_returncode_zero_summary_path = Path(
            str(triage_failed_step_returncode_zero_doc["reports"]["summary"])
        )
        updated_summary_lines = []
        detail_row_rewritten = False
        for line in triage_failed_step_returncode_zero_summary_path.read_text(encoding="utf-8").splitlines():
            if (not detail_row_rewritten) and "failed_step_detail=" in line:
                updated_summary_lines.append(line.replace(" rc=1 ", " rc=0 ", 1))
                detail_row_rewritten = True
                continue
            updated_summary_lines.append(line)
        write_text(triage_failed_step_returncode_zero_summary_path, "\n".join(updated_summary_lines))
        triage_failed_step_returncode_zero_proc = run_check(
            triage_failed_step_returncode_zero_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_failed_step_returncode_zero_proc.returncode == 0:
            return fail("triage failed_step returncode zero case must fail")
        if (
            f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}"
            not in triage_failed_step_returncode_zero_proc.stderr
        ):
            return fail(
                "triage failed_step returncode zero code mismatch: "
                f"err={triage_failed_step_returncode_zero_proc.stderr}"
            )

        triage_failed_step_returncode_index_mismatch_index = build_index_case(
            root, "triage_failed_step_returncode_index_mismatch"
        )
        configure_case_as_fail(triage_failed_step_returncode_index_mismatch_index)
        triage_failed_step_returncode_index_mismatch_doc = json.loads(
            triage_failed_step_returncode_index_mismatch_index.read_text(encoding="utf-8")
        )
        triage_failed_step_returncode_index_mismatch_triage_path = Path(
            str(triage_failed_step_returncode_index_mismatch_doc["reports"]["ci_fail_triage_json"])
        )
        triage_failed_step_returncode_index_mismatch_triage = json.loads(
            triage_failed_step_returncode_index_mismatch_triage_path.read_text(encoding="utf-8")
        )
        triage_failed_step_returncode_index_mismatch_triage["failed_steps"][0]["returncode"] = 9
        write_json(
            triage_failed_step_returncode_index_mismatch_triage_path,
            triage_failed_step_returncode_index_mismatch_triage,
        )
        triage_failed_step_returncode_index_mismatch_summary_path = Path(
            str(triage_failed_step_returncode_index_mismatch_doc["reports"]["summary"])
        )
        updated_summary_lines = []
        detail_row_rewritten = False
        for line in triage_failed_step_returncode_index_mismatch_summary_path.read_text(encoding="utf-8").splitlines():
            if (not detail_row_rewritten) and "failed_step_detail=" in line:
                updated_summary_lines.append(line.replace(" rc=1 ", " rc=9 ", 1))
                detail_row_rewritten = True
                continue
            updated_summary_lines.append(line)
        write_text(
            triage_failed_step_returncode_index_mismatch_summary_path,
            "\n".join(updated_summary_lines),
        )
        triage_failed_step_returncode_index_mismatch_proc = run_check(
            triage_failed_step_returncode_index_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_failed_step_returncode_index_mismatch_proc.returncode == 0:
            return fail("triage failed_step returncode/index mismatch case must fail")
        if (
            f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}"
            not in triage_failed_step_returncode_index_mismatch_proc.stderr
        ):
            return fail(
                "triage failed_step returncode/index mismatch code mismatch: "
                f"err={triage_failed_step_returncode_index_mismatch_proc.stderr}"
            )

        triage_age5_child_mismatch_index = build_index_case(root, "triage_age5_child_mismatch")
        triage_age5_child_mismatch_doc = json.loads(triage_age5_child_mismatch_index.read_text(encoding="utf-8"))
        triage_age5_child_mismatch_report = Path(
            str(triage_age5_child_mismatch_doc["reports"]["ci_fail_triage_json"])
        )
        triage_age5_child_mismatch_triage = json.loads(
            triage_age5_child_mismatch_report.read_text(encoding="utf-8")
        )
        mismatch_key = AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS[1]
        triage_age5_child_mismatch_triage[mismatch_key] = "fail"
        write_json(triage_age5_child_mismatch_report, triage_age5_child_mismatch_triage)
        triage_age5_child_mismatch_proc = run_check(
            triage_age5_child_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_age5_child_mismatch_proc.returncode == 0:
            return fail("triage age5 child mismatch case must fail")
        if f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}" not in triage_age5_child_mismatch_proc.stderr:
            return fail(
                "triage age5 child mismatch code mismatch: "
                f"err={triage_age5_child_mismatch_proc.stderr}"
            )

        triage_age5_default_transport_mismatch_index = build_index_case(root, "triage_age5_default_transport_mismatch")
        triage_age5_default_transport_mismatch_doc = json.loads(
            triage_age5_default_transport_mismatch_index.read_text(encoding="utf-8")
        )
        triage_age5_default_transport_mismatch_report = Path(
            str(triage_age5_default_transport_mismatch_doc["reports"]["ci_fail_triage_json"])
        )
        triage_age5_default_transport_mismatch_triage = json.loads(
            triage_age5_default_transport_mismatch_report.read_text(encoding="utf-8")
        )
        triage_age5_default_transport_mismatch_triage[
            "ci_sanity_age5_combined_heavy_child_summary_default_fields"
        ] = "BROKEN"
        triage_age5_default_transport_mismatch_triage[
            AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY
        ] = "BROKEN"
        triage_age5_default_transport_mismatch_triage["combined_digest_selftest_default_field"] = {
            "broken": "1"
        }
        write_json(
            triage_age5_default_transport_mismatch_report,
            triage_age5_default_transport_mismatch_triage,
        )
        triage_age5_default_transport_mismatch_proc = run_check(
            triage_age5_default_transport_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_age5_default_transport_mismatch_proc.returncode == 0:
            return fail("triage age5 default transport mismatch case must fail")
        if f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}" not in triage_age5_default_transport_mismatch_proc.stderr:
            return fail(
                "triage age5 default transport mismatch code mismatch: "
                f"err={triage_age5_default_transport_mismatch_proc.stderr}"
            )

        triage_profile_matrix_missing_index = build_index_case(root, "triage_profile_matrix_missing")
        triage_profile_matrix_missing_doc = json.loads(
            triage_profile_matrix_missing_index.read_text(encoding="utf-8")
        )
        triage_profile_matrix_missing_report = Path(
            str(triage_profile_matrix_missing_doc["reports"]["ci_fail_triage_json"])
        )
        triage_profile_matrix_missing_payload = json.loads(
            triage_profile_matrix_missing_report.read_text(encoding="utf-8")
        )
        triage_profile_matrix_missing_payload.pop("profile_matrix_selftest", None)
        write_json(triage_profile_matrix_missing_report, triage_profile_matrix_missing_payload)
        triage_profile_matrix_missing_proc = run_check(
            triage_profile_matrix_missing_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_profile_matrix_missing_proc.returncode == 0:
            return fail("triage profile_matrix missing case must fail")
        if f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}" not in triage_profile_matrix_missing_proc.stderr:
            return fail(
                "triage profile_matrix missing code mismatch: "
                f"err={triage_profile_matrix_missing_proc.stderr}"
            )

        triage_profile_matrix_timeout_mismatch_index = build_index_case(
            root, "triage_profile_matrix_timeout_mismatch"
        )
        triage_profile_matrix_timeout_mismatch_doc = json.loads(
            triage_profile_matrix_timeout_mismatch_index.read_text(encoding="utf-8")
        )
        triage_profile_matrix_timeout_mismatch_report = Path(
            str(triage_profile_matrix_timeout_mismatch_doc["reports"]["ci_fail_triage_json"])
        )
        triage_profile_matrix_timeout_mismatch_payload = json.loads(
            triage_profile_matrix_timeout_mismatch_report.read_text(encoding="utf-8")
        )
        triage_profile_matrix_block = triage_profile_matrix_timeout_mismatch_payload.get(
            "profile_matrix_selftest"
        )
        if not isinstance(triage_profile_matrix_block, dict):
            return fail("triage profile_matrix timeout mismatch fixture missing profile_matrix_selftest")
        triage_profile_matrix_block["step_timeout_defaults_sec"] = {
            "core_lang": 111.0,
            "full": 222.0,
            "seamgrim": 333.0,
        }
        triage_profile_matrix_timeout_mismatch_payload[
            "profile_matrix_selftest"
        ] = triage_profile_matrix_block
        write_json(
            triage_profile_matrix_timeout_mismatch_report,
            triage_profile_matrix_timeout_mismatch_payload,
        )
        triage_profile_matrix_timeout_mismatch_proc = run_check(
            triage_profile_matrix_timeout_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_profile_matrix_timeout_mismatch_proc.returncode == 0:
            return fail("triage profile_matrix timeout mismatch case must fail")
        if (
            f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}"
            not in triage_profile_matrix_timeout_mismatch_proc.stderr
        ):
            return fail(
                "triage profile_matrix timeout mismatch code mismatch: "
                f"err={triage_profile_matrix_timeout_mismatch_proc.stderr}"
            )

        summary_age5_child_mismatch_index = build_index_case(root, "summary_age5_child_mismatch")
        summary_age5_child_mismatch_doc = json.loads(summary_age5_child_mismatch_index.read_text(encoding="utf-8"))
        summary_age5_child_mismatch_path = Path(str(summary_age5_child_mismatch_doc["reports"]["summary"]))
        summary_age5_child_mismatch_text = summary_age5_child_mismatch_path.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] age5_combined_heavy_runtime_helper_negative_status=skipped",
            "[ci-gate-summary] age5_combined_heavy_runtime_helper_negative_status=fail",
        )
        write_text(summary_age5_child_mismatch_path, summary_age5_child_mismatch_text)
        summary_age5_child_mismatch_proc = run_check(
            summary_age5_child_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if summary_age5_child_mismatch_proc.returncode == 0:
            return fail("summary age5 child mismatch case must fail")
        if f"fail code={CODES['SUMMARY_VALUE_INVALID']}" not in summary_age5_child_mismatch_proc.stderr:
            return fail(
                "summary age5 child mismatch code mismatch: "
                f"err={summary_age5_child_mismatch_proc.stderr}"
            )

        triage_summary_hint_norm_mismatch_index = build_index_case(root, "triage_summary_hint_norm_mismatch")
        triage_summary_hint_norm_mismatch_doc = json.loads(
            triage_summary_hint_norm_mismatch_index.read_text(encoding="utf-8")
        )
        triage_summary_hint_norm_mismatch_report = Path(
            str(triage_summary_hint_norm_mismatch_doc["reports"]["ci_fail_triage_json"])
        )
        triage_summary_hint_norm_mismatch_triage = json.loads(
            triage_summary_hint_norm_mismatch_report.read_text(encoding="utf-8")
        )
        triage_summary_hint_norm_mismatch_triage["summary_report_path_hint_norm"] = str(
            root / "mismatch" / "ci_gate_summary.txt"
        )
        write_json(triage_summary_hint_norm_mismatch_report, triage_summary_hint_norm_mismatch_triage)
        triage_summary_hint_norm_mismatch_proc = run_check(
            triage_summary_hint_norm_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_summary_hint_norm_mismatch_proc.returncode == 0:
            return fail("triage summary_hint_norm mismatch case must fail")
        if (
            f"fail code={CODES['TRIAGE_SUMMARY_HINT_NORM_MISMATCH']}"
            not in triage_summary_hint_norm_mismatch_proc.stderr
        ):
            return fail(
                "triage summary_hint_norm mismatch code mismatch: "
                f"err={triage_summary_hint_norm_mismatch_proc.stderr}"
            )

        triage_artifacts_missing_index = build_index_case(root, "triage_artifacts_missing")
        triage_artifacts_missing_doc = json.loads(triage_artifacts_missing_index.read_text(encoding="utf-8"))
        triage_artifacts_missing_report = Path(str(triage_artifacts_missing_doc["reports"]["ci_fail_triage_json"]))
        triage_artifacts_missing_triage = json.loads(triage_artifacts_missing_report.read_text(encoding="utf-8"))
        triage_artifacts_missing_triage.pop("artifacts", None)
        write_json(triage_artifacts_missing_report, triage_artifacts_missing_triage)
        triage_artifacts_missing_proc = run_check(
            triage_artifacts_missing_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_artifacts_missing_proc.returncode == 0:
            return fail("triage artifacts missing case must fail")
        if f"fail code={CODES['TRIAGE_ARTIFACTS_MISSING']}" not in triage_artifacts_missing_proc.stderr:
            return fail(f"triage artifacts missing code mismatch: err={triage_artifacts_missing_proc.stderr}")

        triage_artifact_path_norm_mismatch_index = build_index_case(root, "triage_artifact_path_norm_mismatch")
        triage_artifact_path_norm_mismatch_doc = json.loads(
            triage_artifact_path_norm_mismatch_index.read_text(encoding="utf-8")
        )
        triage_artifact_path_norm_mismatch_report = Path(
            str(triage_artifact_path_norm_mismatch_doc["reports"]["ci_fail_triage_json"])
        )
        triage_artifact_path_norm_mismatch_triage = json.loads(
            triage_artifact_path_norm_mismatch_report.read_text(encoding="utf-8")
        )
        triage_artifact_path_norm_mismatch_triage["artifacts"]["summary"]["path_norm"] = str(
            root / "mismatch" / "ci_gate_summary.txt"
        )
        write_json(triage_artifact_path_norm_mismatch_report, triage_artifact_path_norm_mismatch_triage)
        triage_artifact_path_norm_mismatch_proc = run_check(
            triage_artifact_path_norm_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_artifact_path_norm_mismatch_proc.returncode == 0:
            return fail("triage artifact path_norm mismatch case must fail")
        if (
            f"fail code={CODES['TRIAGE_ARTIFACT_PATH_NORM_MISMATCH']}"
            not in triage_artifact_path_norm_mismatch_proc.stderr
        ):
            return fail(
                "triage artifact path_norm mismatch code mismatch: "
                f"err={triage_artifact_path_norm_mismatch_proc.stderr}"
            )

        triage_artifact_path_mismatch_index = build_index_case(root, "triage_artifact_path_mismatch")
        triage_artifact_path_mismatch_doc = json.loads(triage_artifact_path_mismatch_index.read_text(encoding="utf-8"))
        triage_artifact_path_mismatch_report = Path(str(triage_artifact_path_mismatch_doc["reports"]["ci_fail_triage_json"]))
        triage_artifact_path_mismatch_triage = json.loads(triage_artifact_path_mismatch_report.read_text(encoding="utf-8"))
        triage_artifact_path_mismatch_triage["artifacts"]["summary"]["path"] = str(root / "mismatch" / "ci_gate_summary.txt")
        write_json(triage_artifact_path_mismatch_report, triage_artifact_path_mismatch_triage)
        triage_artifact_path_mismatch_proc = run_check(
            triage_artifact_path_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_artifact_path_mismatch_proc.returncode == 0:
            return fail("triage artifact path mismatch case must fail")
        if f"fail code={CODES['TRIAGE_ARTIFACT_PATH_MISMATCH']}" not in triage_artifact_path_mismatch_proc.stderr:
            return fail(f"triage artifact path mismatch code mismatch: err={triage_artifact_path_mismatch_proc.stderr}")

        triage_artifact_exists_mismatch_index = build_index_case(root, "triage_artifact_exists_mismatch")
        triage_artifact_exists_mismatch_doc = json.loads(
            triage_artifact_exists_mismatch_index.read_text(encoding="utf-8")
        )
        triage_artifact_exists_mismatch_report = Path(
            str(triage_artifact_exists_mismatch_doc["reports"]["ci_fail_triage_json"])
        )
        triage_artifact_exists_mismatch_triage = json.loads(
            triage_artifact_exists_mismatch_report.read_text(encoding="utf-8")
        )
        triage_artifact_exists_mismatch_triage["artifacts"]["summary"]["exists"] = False
        write_json(triage_artifact_exists_mismatch_report, triage_artifact_exists_mismatch_triage)
        triage_artifact_exists_mismatch_proc = run_check(
            triage_artifact_exists_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if triage_artifact_exists_mismatch_proc.returncode == 0:
            return fail("triage artifact exists mismatch case must fail")
        if (
            f"fail code={CODES['TRIAGE_ARTIFACT_EXISTS_MISMATCH']}"
            not in triage_artifact_exists_mismatch_proc.stderr
        ):
            return fail(
                "triage artifact exists mismatch code mismatch: "
                f"err={triage_artifact_exists_mismatch_proc.stderr}"
            )

        cmd_empty_index = build_index_case(root, "cmd_empty")
        cmd_empty_doc = json.loads(cmd_empty_index.read_text(encoding="utf-8"))
        cmd_empty_doc["steps"][0]["cmd"] = []
        write_json(cmd_empty_index, cmd_empty_doc)
        cmd_empty_proc = run_check(
            cmd_empty_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if cmd_empty_proc.returncode == 0:
            return fail("cmd empty case must fail")
        if f"fail code={CODES['STEP_CMD_EMPTY']}" not in cmd_empty_proc.stderr:
            return fail(f"cmd empty code mismatch: err={cmd_empty_proc.stderr}")

        cmd_item_type_index = build_index_case(root, "cmd_item_type")
        cmd_item_type_doc = json.loads(cmd_item_type_index.read_text(encoding="utf-8"))
        cmd_item_type_doc["steps"][0]["cmd"] = ["python", ""]
        write_json(cmd_item_type_index, cmd_item_type_doc)
        cmd_item_type_proc = run_check(
            cmd_item_type_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if cmd_item_type_proc.returncode == 0:
            return fail("cmd item type case must fail")
        if f"fail code={CODES['STEP_CMD_ITEM_TYPE']}" not in cmd_item_type_proc.stderr:
            return fail(f"cmd item type code mismatch: err={cmd_item_type_proc.stderr}")

        ok_rc_mismatch_index = build_index_case(root, "ok_rc_mismatch")
        ok_rc_mismatch_doc = json.loads(ok_rc_mismatch_index.read_text(encoding="utf-8"))
        ok_rc_mismatch_doc["steps"][0]["ok"] = True
        ok_rc_mismatch_doc["steps"][0]["returncode"] = 1
        write_json(ok_rc_mismatch_index, ok_rc_mismatch_doc)
        ok_rc_mismatch_proc = run_check(
            ok_rc_mismatch_index,
            REQUIRED_STEPS_FULL,
            sanity_profile="full",
            enforce_profile_step_contract=True,
        )
        if ok_rc_mismatch_proc.returncode == 0:
            return fail("ok/rc mismatch case must fail")
        if f"fail code={CODES['STEP_OK_RC_MISMATCH']}" not in ok_rc_mismatch_proc.stderr:
            return fail(f"ok/rc mismatch code mismatch: err={ok_rc_mismatch_proc.stderr}")

        core_lang_missing_seamgrim_index = build_index_case(
            root,
            "core_lang_missing_seamgrim_steps",
            sanity_profile="core_lang",
        )
        core_lang_doc = json.loads(core_lang_missing_seamgrim_index.read_text(encoding="utf-8"))
        core_lang_doc["steps"] = [
            row
            for row in core_lang_doc["steps"]
            if str(row.get("name", "")) not in set(REQUIRED_STEPS_SEAMGRIM)
        ]
        write_json(core_lang_missing_seamgrim_index, core_lang_doc)
        core_lang_proc = run_check(
            core_lang_missing_seamgrim_index,
            REQUIRED_STEPS_CORE_LANG,
            sanity_profile="core_lang",
            enforce_profile_step_contract=True,
        )
        if core_lang_proc.returncode != 0:
            return fail(f"core_lang profile should allow missing seamgrim steps: out={core_lang_proc.stdout} err={core_lang_proc.stderr}")

        seamgrim_sanity_step_missing_ok_index = build_index_case(
            root,
            "seamgrim_sanity_step_missing_ok",
            sanity_profile="seamgrim",
        )
        seamgrim_sanity_step_missing_ok_proc = run_check(
            seamgrim_sanity_step_missing_ok_index,
            (),
            sanity_profile="seamgrim",
            enforce_profile_step_contract=True,
        )
        if seamgrim_sanity_step_missing_ok_proc.returncode != 0:
            return fail(
                "seamgrim profile should allow missing lang consistency sanity step: "
                f"out={seamgrim_sanity_step_missing_ok_proc.stdout} "
                f"err={seamgrim_sanity_step_missing_ok_proc.stderr}"
            )

        seamgrim_missing_index = build_index_case(
            root,
            "seamgrim_missing_step",
            sanity_profile="seamgrim",
        )
        seamgrim_doc = json.loads(seamgrim_missing_index.read_text(encoding="utf-8"))
        seamgrim_doc["steps"] = [
            row for row in seamgrim_doc["steps"] if str(row.get("name", "")) != "seamgrim_wasm_cli_diag_parity_check"
        ]
        write_json(seamgrim_missing_index, seamgrim_doc)
        seamgrim_missing_proc = run_check(
            seamgrim_missing_index,
            (),
            sanity_profile="seamgrim",
            enforce_profile_step_contract=True,
        )
        if seamgrim_missing_proc.returncode == 0:
            return fail("seamgrim profile missing parity step case must fail")
        if f"fail code={CODES['REQUIRED_STEP_MISSING']}" not in seamgrim_missing_proc.stderr:
            return fail(f"seamgrim profile missing parity step code mismatch: err={seamgrim_missing_proc.stderr}")

    print("[ci-gate-report-index-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
