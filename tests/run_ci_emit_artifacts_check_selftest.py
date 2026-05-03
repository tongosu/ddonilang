#!/usr/bin/env python
from __future__ import annotations

import json
import io
from contextlib import contextmanager, redirect_stderr, redirect_stdout
import importlib
import runpy
import subprocess
import sys
import tempfile
from pathlib import Path

from _ci_age3_completion_gate_contract import (
    AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS,
    AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS,
)
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
    build_profile_matrix_brief_payload_from_snapshot,
    PROFILE_MATRIX_SELFTEST_PROFILES,
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC,
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
    PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS,
    build_profile_matrix_snapshot_from_doc,
    expected_profile_matrix_aggregate_summary_contract,
)
from _ci_seamgrim_step_contract import (
    SEAMGRIM_BLOCKER_STEP_CONTRACT_STEPS,
    SEAMGRIM_FEATURED_SEED_STEP_CONTRACT_STEPS,
    SEAMGRIM_PLATFORM_STEP_CONTRACT_STEPS,
)
from ci_check_error_codes import EMIT_ARTIFACTS_CODES as CODES

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
SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA = "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1"
SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES = 20
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA = "ddn.pack_evidence_tier_runner_check.v1"
SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA = "ddn.numeric_factor_route_diag_contract.v1"
SEAMGRIM_NUMERIC_FACTOR_POLICY_KEYS = (
    "bit_limit",
    "pollard_iters",
    "pollard_c_seeds",
    "pollard_x0_seeds",
    "fallback_limit",
    "small_prime_max",
)
SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS: dict[str, int] = {
    "bit_limit": 512,
    "pollard_iters": 200000,
    "pollard_c_seeds": 64,
    "pollard_x0_seeds": 6,
    "fallback_limit": 1000000,
    "small_prime_max": 101,
}
_MODULE_CACHE: dict[str, object] = {}
_ENSURED_PARENT_DIRS: set[Path] = set()
CaseKwargs = dict[str, object]
PassCase = tuple[str, CaseKwargs]
FailCase = tuple[str, str | None, CaseKwargs]
StrictFailCase = tuple[str, str, CaseKwargs]
MutatedFailCase = tuple[str, str, object]
ResultContractFailCase = tuple[str, CaseKwargs, str]
FlagFailRow = tuple[str, str | None, str]

def fail(msg: str) -> int:
    print(f"[ci-emit-artifacts-check-selftest] fail: {msg}")
    return 1


# Keep literal fail-code tokens for sanity contract regex checks.
SELFTEST_FAIL_CODE_LITERAL_TOKENS = (
    "fail code={CODES['BRIEF_REQUIRED_MISSING']}",
    "fail code={CODES['INDEX_BRIEF_PATH_MISSING']}",
    "fail code={CODES['INDEX_NOT_FOUND']}",
    "fail code={CODES['INDEX_REPORTS_MISSING']}",
    "fail code={CODES['INDEX_REPORT_KEY_MISSING']}",
    "fail code={CODES['INDEX_RESULT_PATH_MISSING']}",
    "fail code={CODES['INDEX_SUMMARY_PATH_MISSING']}",
    "fail code={CODES['INDEX_TRIAGE_PATH_MISSING']}",
    "fail code={CODES['REPORT_DIR_MISSING']}",
    "fail code={CODES['RESULT_FAILED_STEPS_NEGATIVE']}",
    "fail code={CODES['RESULT_FAILED_STEPS_TYPE']}",
    "fail code={CODES['RESULT_FAIL_FAILED_STEPS']}",
    "fail code={CODES['RESULT_JSON_INVALID']}",
    "fail code={CODES['RESULT_PASS_FAILED_STEPS']}",
    "fail code={CODES['RESULT_SCHEMA_MISMATCH']}",
    "fail code={CODES['RESULT_STATUS_UNSUPPORTED']}",
    "fail code={CODES['SANITY_FAIL_FAILED_STEPS']}",
    "fail code={CODES['SANITY_JSON_INVALID']}",
    "fail code={CODES['SANITY_PASS_FAILED_STEPS']}",
    "fail code={CODES['TRIAGE_REQUIRED_MISSING']}",
    "fail code={CODES['SANITY_PATH_MISSING']}",
    "fail code={CODES['SANITY_REQUIRED_STEP_MISSING']}",
    "fail code={CODES['SANITY_REQUIRED_STEP_FAILED']}",
    "fail code={CODES['SANITY_SCHEMA_MISMATCH']}",
    "fail code={CODES['SANITY_STEPS_TYPE']}",
    "fail code={CODES['SANITY_STATUS_MISMATCH']}",
    "fail code={CODES['SANITY_STATUS_UNSUPPORTED']}",
    "fail code={CODES['SUMMARY_SELFTEST_EXPECT_PASS']}",
    "fail code={CODES['SUMMARY_SELFTEST_KEY_MISSING']}",
    "fail code={CODES['SUMMARY_SELFTEST_STEP_MISMATCH']}",
    "fail code={CODES['SUMMARY_SELFTEST_VALUE_INVALID']}",
    "fail code={CODES['SUMMARY_STATUS_MISMATCH']}",
    "fail code={CODES['SYNC_READINESS_JSON_INVALID']}",
    "fail code={CODES['SYNC_READINESS_PATH_MISSING']}",
    "fail code={CODES['SYNC_READINESS_SCHEMA_MISMATCH']}",
    "fail code={CODES['SYNC_READINESS_STATUS_UNSUPPORTED']}",
    "fail code={CODES['SYNC_READINESS_STATUS_MISMATCH']}",
    "fail code={CODES['SYNC_READINESS_PASS_STATUS_FIELDS']}",
)


def ensure_parent_dir(path: Path) -> None:
    parent = path.parent
    if parent in _ENSURED_PARENT_DIRS:
        return
    parent.mkdir(parents=True, exist_ok=True)
    _ENSURED_PARENT_DIRS.add(parent)


def write_json(path: Path, payload: dict) -> None:
    ensure_parent_dir(path)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def write_text(path: Path, text: str) -> None:
    ensure_parent_dir(path)
    path.write_text(text.rstrip() + "\n", encoding="utf-8")


def _run_module_main(
    module_name: str,
    cmd: list[str],
    argv: list[str],
) -> subprocess.CompletedProcess[str]:
    module = _MODULE_CACHE.get(module_name)
    if module is None:
        module = importlib.import_module(module_name)
        _MODULE_CACHE[module_name] = module

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
    if script_norm.endswith("tests/run_ci_emit_artifacts_check.py"):
        return _run_module_main("run_ci_emit_artifacts_check", cmd, argv)
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


def run_check(report_dir: Path, *extra: str) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_ci_emit_artifacts_check.py",
        "--report-dir",
        str(report_dir),
        *extra,
    ]
    return run_cmd_inprocess(cmd)


@contextmanager
def persistent_tmpdir(prefix: str):
    # Selftest speedup: skip TemporaryDirectory cleanup(rmtree) cost.
    yield tempfile.mkdtemp(prefix=prefix)


def build_case(
    report_dir: Path,
    prefix: str,
    status: str,
    with_brief: bool,
    with_triage: bool,
    sanity_profile: str = "full",
    with_sanity: bool = True,
    with_sync_readiness: bool = True,
    broken_norm: bool = False,
    broken_brief: bool = False,
    broken_triage_final: bool = False,
    broken_triage_failed_step_detail_rows_count_mismatch: bool = False,
    broken_triage_failed_step_logs_rows_count_mismatch: bool = False,
    broken_triage_failed_step_detail_order_mismatch: bool = False,
    broken_triage_failed_step_logs_order_mismatch: bool = False,
    broken_triage_failed_step_detail_rows_count_type: bool = False,
    broken_triage_failed_step_logs_rows_count_type: bool = False,
    broken_triage_failed_step_detail_order_type: bool = False,
    broken_triage_failed_step_logs_order_type: bool = False,
    broken_artifact_ref: bool = False,
    broken_summary: bool = False,
    broken_verify_issue: bool = False,
    broken_sanity_schema: bool = False,
    broken_sanity_status: bool = False,
    broken_sanity_required_step_missing: bool = False,
    broken_sanity_required_step_failed: bool = False,
    broken_sanity_product_blocker_step_missing: bool = False,
    broken_sanity_product_blocker_step_failed: bool = False,
    broken_sanity_observe_output_contract_step_missing: bool = False,
    broken_sanity_observe_output_contract_step_failed: bool = False,
    broken_sanity_runtime_view_source_strict_step_missing: bool = False,
    broken_sanity_runtime_view_source_strict_step_failed: bool = False,
    broken_sanity_run_legacy_autofix_step_missing: bool = False,
    broken_sanity_run_legacy_autofix_step_failed: bool = False,
    broken_sanity_wired_step_missing: bool = False,
    broken_sanity_wired_step_failed: bool = False,
    broken_sanity_compare_step_missing: bool = False,
    broken_sanity_compare_step_failed: bool = False,
    broken_sanity_wasm_web_selftest_step_missing: bool = False,
    broken_sanity_wasm_web_selftest_step_failed: bool = False,
    broken_sanity_seamgrim_pack_schema: bool = False,
    broken_sanity_seamgrim_pack_schema_non_seamgrim: bool = False,
    broken_sanity_seamgrim_pack_docs_issue_count: bool = False,
    broken_sanity_seamgrim_pack_docs_issue_count_non_seamgrim: bool = False,
    broken_sanity_seamgrim_pack_repo_issue_count: bool = False,
    broken_sanity_seamgrim_wasm_checked_files: bool = False,
    broken_sanity_seamgrim_wasm_checked_files_non_seamgrim: bool = False,
    broken_sanity_seamgrim_pack_docs_issue_count_summary_mismatch: bool = False,
    broken_sanity_seamgrim_wasm_checked_files_summary_mismatch: bool = False,
    broken_summary_selftest_missing: bool = False,
    broken_summary_selftest_value: bool = False,
    broken_summary_selftest_step_mismatch: bool = False,
    broken_profile_matrix_summary_missing: bool = False,
    broken_profile_matrix_summary_value: bool = False,
    broken_profile_matrix_report_mismatch: bool = False,
    broken_profile_matrix_brief_missing: bool = False,
    broken_profile_matrix_brief_value: bool = False,
    broken_profile_matrix_triage_mismatch: bool = False,
    with_runtime5_checklist: bool = True,
    broken_runtime5_summary_missing: bool = False,
    broken_runtime5_summary_value: bool = False,
    broken_runtime5_report_mismatch: bool = False,
    broken_sync_readiness_schema: bool = False,
    broken_sync_readiness_status_unsupported: bool = False,
    broken_sync_readiness_status_mismatch: bool = False,
    broken_sync_readiness_pass_fields: bool = False,
    broken_sync_readiness_seamgrim_pack_schema: bool = False,
    broken_sync_readiness_seamgrim_pack_schema_non_seamgrim: bool = False,
    broken_sync_readiness_seamgrim_pack_docs_issue_count: bool = False,
    broken_sync_readiness_seamgrim_pack_docs_issue_count_non_seamgrim: bool = False,
    broken_sync_readiness_seamgrim_pack_docs_issue_count_summary_mismatch: bool = False,
    broken_sync_readiness_seamgrim_wasm_checked_files: bool = False,
    broken_sync_readiness_seamgrim_wasm_checked_files_non_seamgrim: bool = False,
    broken_sync_readiness_seamgrim_wasm_checked_files_summary_mismatch: bool = False,
    broken_age5_child_summary_summary_mismatch: bool = False,
    broken_age5_child_summary_triage_mismatch: bool = False,
    broken_age5_child_summary_default_triage_mismatch: bool = False,
    force_artifact_exists_false: tuple[str, ...] = (),
    omit_age_close_status_summary_keys: tuple[str, ...] = (),
    triage_fail_step_use_name_field: bool = False,
) -> None:
    if sanity_profile not in {"full", "core_lang", "seamgrim"}:
        sanity_profile = "full"

    index_path = report_dir / f"{prefix}.ci_gate_report_index.detjson"
    aggregate_path = report_dir / f"{prefix}.ci_aggregate_report.detjson"
    result_path = report_dir / f"{prefix}.ci_gate_result.detjson"
    summary_path = report_dir / f"{prefix}.ci_gate_summary.txt"
    summary_line_path = report_dir / f"{prefix}.ci_gate_summary_line.txt"
    ci_gate_result_line_path = report_dir / f"{prefix}.ci_gate_result_line.txt"
    brief_path = report_dir / f"{prefix}.ci_fail_brief.txt"
    triage_path = report_dir / f"{prefix}.ci_fail_triage.detjson"
    sanity_path = report_dir / f"{prefix}.ci_sanity_gate.detjson"
    sync_readiness_path = report_dir / f"{prefix}.ci_sync_readiness.detjson"
    runtime5_checklist_path = report_dir / f"{prefix}.seamgrim_5min_checklist_report.detjson"
    profile_matrix_selftest_path = report_dir / f"{prefix}.ci_profile_matrix_gate_selftest.detjson"
    age2_close_report_path = report_dir / f"{prefix}.age2_close_report.detjson"
    age3_close_status_report_path = report_dir / f"{prefix}.age3_close_status.detjson"
    age4_close_report_path = report_dir / f"{prefix}.age4_close_report.detjson"
    age5_close_report_path = report_dir / f"{prefix}.age5_close_report.detjson"
    profile_matrix_total_elapsed_ms = 999 if broken_profile_matrix_report_mismatch else 666
    profile_matrix_selected_real_profiles = "core_lang,full,seamgrim"
    profile_matrix_core_lang_elapsed_ms = 111
    profile_matrix_full_elapsed_ms = 222
    profile_matrix_seamgrim_elapsed_ms = 333
    profile_matrix_checked_profiles = profile_matrix_selected_real_profiles
    profile_matrix_failed_profiles = "-"
    profile_matrix_skipped_profiles = "-"
    include_core_lang_keys = sanity_profile in {"full", "core_lang"}
    pipeline_emit_flags_ok = "1" if include_core_lang_keys else "na"
    pipeline_emit_flags_selftest_ok = "1" if include_core_lang_keys else "na"
    emit_artifacts_sanity_contract_selftest_ok = "1"
    age2_completion_gate_ok = "1"
    age2_completion_gate_selftest_ok = "1"
    age2_close_ok = "1"
    age2_close_selftest_ok = "1"
    age2_completion_gate_failure_codes = "-"
    age2_completion_gate_failure_code_count = "0"
    age3_completion_gate_ok = "1"
    age3_completion_gate_selftest_ok = "1"
    age3_close_ok = "1" if sanity_profile in {"full", "seamgrim"} else "na"
    age3_close_selftest_ok = "1"
    age3_completion_gate_failure_codes = "-"
    age3_completion_gate_failure_code_count = "0"
    age3_bogae_geoul_visibility_smoke_ok = "1"
    age3_bogae_geoul_visibility_smoke_overall_ok = "1"
    age3_bogae_geoul_visibility_smoke_checks_ok = "1"
    age3_bogae_geoul_visibility_smoke_sim_state_hash_changes = "1"
    age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes = "1"
    age3_bogae_geoul_visibility_smoke_report_exists = "1"
    age3_bogae_geoul_visibility_smoke_schema = "ddn.bogae_geoul_visibility_smoke.v1"
    age3_bogae_geoul_visibility_smoke_report_path = (
        report_dir / f"{prefix}.age3_completion_gate.bogae_geoul_visibility_smoke.detjson"
    )
    seamgrim_wasm_web_step_check_enabled = sanity_profile == "seamgrim"
    seamgrim_wasm_web_step_check_ok = "1" if seamgrim_wasm_web_step_check_enabled else "na"
    seamgrim_wasm_web_step_check_report_path = (
        report_dir / f"{prefix}.seamgrim_wasm_web_step_check.detjson"
    )
    seamgrim_wasm_web_step_check_report_path_text = (
        str(seamgrim_wasm_web_step_check_report_path) if seamgrim_wasm_web_step_check_enabled else "-"
    )
    seamgrim_wasm_web_step_check_report_exists = "1" if seamgrim_wasm_web_step_check_enabled else "na"
    seamgrim_wasm_web_step_check_schema = (
        SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA if seamgrim_wasm_web_step_check_enabled else "-"
    )
    seamgrim_wasm_web_step_check_checked_files = (
        str(SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES) if seamgrim_wasm_web_step_check_enabled else "-"
    )
    seamgrim_wasm_web_step_check_missing_count = "0" if seamgrim_wasm_web_step_check_enabled else "-"
    seamgrim_pack_evidence_tier_runner_enabled = sanity_profile == "seamgrim"
    seamgrim_pack_evidence_tier_runner_ok = "1" if seamgrim_pack_evidence_tier_runner_enabled else "na"
    seamgrim_pack_evidence_tier_runner_report_path = (
        report_dir / f"{prefix}.seamgrim_pack_evidence_tier_runner_check.detjson"
    )
    seamgrim_pack_evidence_tier_runner_report_path_text = (
        str(seamgrim_pack_evidence_tier_runner_report_path)
        if seamgrim_pack_evidence_tier_runner_enabled
        else "-"
    )
    seamgrim_pack_evidence_tier_runner_report_exists = (
        "1" if seamgrim_pack_evidence_tier_runner_enabled else "na"
    )
    seamgrim_pack_evidence_tier_runner_schema = (
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA
        if seamgrim_pack_evidence_tier_runner_enabled
        else "-"
    )
    seamgrim_pack_evidence_tier_runner_docs_issue_count = (
        "0" if seamgrim_pack_evidence_tier_runner_enabled else "-"
    )
    seamgrim_pack_evidence_tier_runner_repo_issue_count = (
        "0" if seamgrim_pack_evidence_tier_runner_enabled else "-"
    )
    seamgrim_numeric_factor_policy_enabled = sanity_profile in {"full", "seamgrim"}
    seamgrim_numeric_factor_policy_report_path = (
        report_dir / f"{prefix}.seamgrim_numeric_factor_policy.detjson"
    )
    seamgrim_numeric_factor_policy_report_path_text = (
        str(seamgrim_numeric_factor_policy_report_path)
        if seamgrim_numeric_factor_policy_enabled
        else "-"
    )
    seamgrim_numeric_factor_policy_ok = "1" if seamgrim_numeric_factor_policy_enabled else "na"
    seamgrim_numeric_factor_policy_report_exists = "1" if seamgrim_numeric_factor_policy_enabled else "na"
    seamgrim_numeric_factor_policy_schema = (
        SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA if seamgrim_numeric_factor_policy_enabled else "-"
    )
    seamgrim_numeric_factor_policy_text = (
        ";".join(
            f"{key}={SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS[key]}"
            for key in SEAMGRIM_NUMERIC_FACTOR_POLICY_KEYS
        )
        if seamgrim_numeric_factor_policy_enabled
        else "-"
    )
    seamgrim_numeric_factor_policy_values = {
        key: (
            str(SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS[key])
            if seamgrim_numeric_factor_policy_enabled
            else "-"
        )
        for key in SEAMGRIM_NUMERIC_FACTOR_POLICY_KEYS
    }
    sanity_seamgrim_pack_evidence_tier_runner_schema = seamgrim_pack_evidence_tier_runner_schema
    sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count = (
        seamgrim_pack_evidence_tier_runner_docs_issue_count
    )
    sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count = (
        seamgrim_pack_evidence_tier_runner_repo_issue_count
    )
    sanity_seamgrim_wasm_web_step_check_checked_files = seamgrim_wasm_web_step_check_checked_files
    if seamgrim_pack_evidence_tier_runner_enabled and broken_sanity_seamgrim_pack_schema:
        sanity_seamgrim_pack_evidence_tier_runner_schema = "ddn.pack_evidence_tier_runner_check.v0.broken"
    if (not seamgrim_pack_evidence_tier_runner_enabled) and broken_sanity_seamgrim_pack_schema_non_seamgrim:
        sanity_seamgrim_pack_evidence_tier_runner_schema = SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA
    if seamgrim_pack_evidence_tier_runner_enabled and broken_sanity_seamgrim_pack_docs_issue_count:
        sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count = "11"
    if (
        (not seamgrim_pack_evidence_tier_runner_enabled)
        and broken_sanity_seamgrim_pack_docs_issue_count_non_seamgrim
    ):
        sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count = "0"
    if seamgrim_pack_evidence_tier_runner_enabled and broken_sanity_seamgrim_pack_docs_issue_count_summary_mismatch:
        sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count = "1"
    if seamgrim_pack_evidence_tier_runner_enabled and broken_sanity_seamgrim_pack_repo_issue_count:
        sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count = "1"
    if seamgrim_wasm_web_step_check_enabled and broken_sanity_seamgrim_wasm_checked_files:
        sanity_seamgrim_wasm_web_step_check_checked_files = str(
            max(0, SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES - 10)
        )
    if (
        (not seamgrim_wasm_web_step_check_enabled)
        and broken_sanity_seamgrim_wasm_checked_files_non_seamgrim
    ):
        sanity_seamgrim_wasm_web_step_check_checked_files = str(SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES)
    if seamgrim_wasm_web_step_check_enabled and broken_sanity_seamgrim_wasm_checked_files_summary_mismatch:
        sanity_seamgrim_wasm_web_step_check_checked_files = str(
            SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES + 1
        )
    sync_seamgrim_pack_evidence_tier_runner_schema = seamgrim_pack_evidence_tier_runner_schema
    sync_seamgrim_pack_evidence_tier_runner_docs_issue_count = (
        seamgrim_pack_evidence_tier_runner_docs_issue_count
    )
    sync_seamgrim_wasm_web_step_check_checked_files = seamgrim_wasm_web_step_check_checked_files
    if seamgrim_pack_evidence_tier_runner_enabled and broken_sync_readiness_seamgrim_pack_schema:
        sync_seamgrim_pack_evidence_tier_runner_schema = "ddn.pack_evidence_tier_runner_check.v0.broken"
    if (not seamgrim_pack_evidence_tier_runner_enabled) and broken_sync_readiness_seamgrim_pack_schema_non_seamgrim:
        sync_seamgrim_pack_evidence_tier_runner_schema = SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA
    if seamgrim_pack_evidence_tier_runner_enabled and broken_sync_readiness_seamgrim_pack_docs_issue_count:
        sync_seamgrim_pack_evidence_tier_runner_docs_issue_count = "11"
    if (
        (not seamgrim_pack_evidence_tier_runner_enabled)
        and broken_sync_readiness_seamgrim_pack_docs_issue_count_non_seamgrim
    ):
        sync_seamgrim_pack_evidence_tier_runner_docs_issue_count = "0"
    if (
        seamgrim_pack_evidence_tier_runner_enabled
        and broken_sync_readiness_seamgrim_pack_docs_issue_count_summary_mismatch
    ):
        sync_seamgrim_pack_evidence_tier_runner_docs_issue_count = "1"
    if seamgrim_wasm_web_step_check_enabled and broken_sync_readiness_seamgrim_wasm_checked_files:
        sync_seamgrim_wasm_web_step_check_checked_files = str(
            max(0, SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES - 10)
        )
    if (
        (not seamgrim_wasm_web_step_check_enabled)
        and broken_sync_readiness_seamgrim_wasm_checked_files_non_seamgrim
    ):
        sync_seamgrim_wasm_web_step_check_checked_files = str(SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES)
    if (
        seamgrim_wasm_web_step_check_enabled
        and broken_sync_readiness_seamgrim_wasm_checked_files_summary_mismatch
    ):
        sync_seamgrim_wasm_web_step_check_checked_files = str(
            SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES + 1
        )
    age5_combined_heavy_policy_ok = "1"
    profile_matrix_policy_ok = "1"
    dynamic_source_profile_split_selftest_ok = "1"
    age5_child_summary_fields = {
        "age5_combined_heavy_full_real_status": "skipped",
        "age5_combined_heavy_runtime_helper_negative_status": "skipped",
        "age5_combined_heavy_group_id_summary_negative_status": "skipped",
    }
    age5_digest_selftest_ok = "1"
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
    age5_child_summary_default_transport = build_age5_combined_heavy_child_summary_default_text_transport_fields()
    age4_proof_ok = "1" if status == "pass" else "0"
    age4_proof_failed_criteria = "0" if status == "pass" else "1"
    age4_proof_failed_preview = "-" if status == "pass" else "proof_runtime_error_statehash_preserved"
    age4_proof_summary_hash = "sha256:age4-proof-okcase" if status == "pass" else "sha256:age4-proof-failcase"
    profile_matrix_expected_contracts = {
        profile_name: expected_profile_matrix_aggregate_summary_contract(profile_name)
        for profile_name in PROFILE_MATRIX_SELFTEST_PROFILES
    }
    sanity_contract_fields = build_age5_combined_heavy_sanity_contract_fields()
    sync_contract_fields = build_age5_combined_heavy_sync_contract_fields()
    write_json(
        age3_bogae_geoul_visibility_smoke_report_path,
        {
            "schema": age3_bogae_geoul_visibility_smoke_schema,
            "generated_at_utc": "2026-03-24T00:00:00+00:00",
            "overall_ok": True,
            "checks": [
                {"id": "smoke_probe_open", "ok": True},
                {"id": "smoke_probe_mirror", "ok": True},
            ],
            "simulation_hash_delta": {
                "state_hash_changes": True,
                "bogae_hash_changes": True,
            },
            "failed_checks": [],
        },
    )
    if seamgrim_wasm_web_step_check_enabled:
        write_json(
            seamgrim_wasm_web_step_check_report_path,
            {
                "schema": SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA,
                "status": "pass",
                "ok": True,
                "code": "OK",
                "checked_files": SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES,
                "missing_count": 0,
                "missing": [],
            },
        )
    if seamgrim_pack_evidence_tier_runner_enabled:
        write_json(
            seamgrim_pack_evidence_tier_runner_report_path,
            {
                "schema": SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA,
                "status": "pass",
                "ok": True,
                "docs_profile": {
                    "name": "docs_ssot_rep10",
                    "issue_count": 0,
                },
                "repo_profile": {
                    "name": "repo_rep10",
                    "issue_count": 0,
                },
            },
        )
    if seamgrim_numeric_factor_policy_enabled:
        write_json(
            seamgrim_numeric_factor_policy_report_path,
            {
                "schema": SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA,
                "status": "pass",
                "ok": True,
                "code": "OK",
                "numeric_factor_policy_text": seamgrim_numeric_factor_policy_text,
                "numeric_factor_policy": {
                    key: int(SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS[key])
                    for key in SEAMGRIM_NUMERIC_FACTOR_POLICY_KEYS
                },
            },
        )
    write_json(
        age2_close_report_path,
        {
            "schema": "ddn.age2_close_report.v1",
            "overall_ok": status == "pass",
            "criteria": [],
            "failure_digest": [],
            "failure_codes": [],
        },
    )
    write_json(
        age3_close_status_report_path,
        {
            "schema": "ddn.seamgrim.age3_close_status.v1",
            "status": "pass" if status == "pass" else "fail",
            "ok": status == "pass",
        },
    )
    write_json(
        age4_close_report_path,
        {
            "schema": "ddn.age4_close_report.v1",
            "overall_ok": status == "pass",
            "criteria": [],
            "failure_digest": [],
        },
    )
    write_json(
        age5_close_report_path,
        {
            "schema": "ddn.age5_close_report.v1",
            "overall_ok": status == "pass",
            "criteria": [],
            "failure_digest": [],
        },
    )

    failed_steps_count = 0 if status == "pass" else 1
    sample_step_id = "sample_fail"
    sample_stdout_path = report_dir / f"{prefix}.ci_gate_step_{sample_step_id}.stdout.txt"
    sample_stderr_path = report_dir / f"{prefix}.ci_gate_step_{sample_step_id}.stderr.txt"
    if failed_steps_count > 0:
        write_text(sample_stdout_path, "[sample-fail] stdout")
        write_text(sample_stderr_path, "[sample-fail] stderr")
    if status == "pass" or broken_summary:
        summary_lines = [
            "[ci-gate-summary] PASS",
            "[ci-gate-summary] failed_steps=(none)",
        ]
        if with_runtime5_checklist:
            runtime5_checklist_ok = "1"
            runtime5_rewrite_ok = "1"
            runtime5_rewrite_elapsed_ms = "321"
            runtime5_rewrite_status = "ok"
            runtime5_moyang_ok = "1"
            runtime5_moyang_elapsed_ms = "654"
            runtime5_moyang_status = "ok"
            runtime5_showcase_ok = "1"
            runtime5_showcase_elapsed_ms = "777"
            runtime5_showcase_status = "bad" if broken_runtime5_summary_value else "ok"
            summary_lines.append(f"[ci-gate-summary] seamgrim_5min_checklist={runtime5_checklist_path}")
            summary_lines.append(f"[ci-gate-summary] seamgrim_5min_checklist_ok={runtime5_checklist_ok}")
            summary_lines.append(
                f"[ci-gate-summary] seamgrim_runtime_5min_rewrite_motion_projectile={runtime5_rewrite_ok}"
            )
            summary_lines.append(f"[ci-gate-summary] seamgrim_runtime_5min_rewrite_elapsed_ms={runtime5_rewrite_elapsed_ms}")
            summary_lines.append(f"[ci-gate-summary] seamgrim_runtime_5min_rewrite_status={runtime5_rewrite_status}")
            if not broken_runtime5_summary_missing:
                summary_lines.append(
                    f"[ci-gate-summary] seamgrim_runtime_5min_moyang_view_boundary={runtime5_moyang_ok}"
                )
            summary_lines.append(f"[ci-gate-summary] seamgrim_runtime_5min_moyang_elapsed_ms={runtime5_moyang_elapsed_ms}")
            summary_lines.append(f"[ci-gate-summary] seamgrim_runtime_5min_moyang_status={runtime5_moyang_status}")
            summary_lines.append(
                f"[ci-gate-summary] seamgrim_runtime_5min_pendulum_tetris_showcase={runtime5_showcase_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] seamgrim_runtime_5min_pendulum_tetris_showcase_elapsed_ms="
                f"{runtime5_showcase_elapsed_ms}"
            )
            if not broken_runtime5_summary_missing:
                summary_lines.append(
                    "[ci-gate-summary] seamgrim_runtime_5min_pendulum_tetris_showcase_status="
                    f"{runtime5_showcase_status}"
                )
        if not broken_summary_selftest_missing:
            compare_ok = "0" if broken_summary_selftest_value else "1"
            session_ok = "1"
            lang_consistency_ok = "1" if include_core_lang_keys else "0"
            pack_golden_metadata_ok = "1" if include_core_lang_keys else "0"
            pack_golden_graph_export_ok = "1" if include_core_lang_keys else "0"
            sync_pack_golden_graph_export_ok = "1" if include_core_lang_keys else "0"
            canon_ast_dpack_ok = "1" if include_core_lang_keys else "0"
            contract_tier_unsupported_ok = "1" if include_core_lang_keys else "0"
            contract_tier_age3_min_enforcement_ok = "1" if include_core_lang_keys else "0"
            stdlib_catalog_ok = "1" if include_core_lang_keys else "0"
            stdlib_catalog_selftest_ok = "1" if include_core_lang_keys else "0"
            tensor_v0_pack_ok = "1" if include_core_lang_keys else "0"
            tensor_v0_cli_ok = "1" if include_core_lang_keys else "0"
            fixed64_darwin_real_report_contract_ok = "1"
            fixed64_darwin_real_report_live_ok = "1"
            fixed64_darwin_real_report_readiness_selftest_ok = "1"
            map_access_contract_ok = "1" if include_core_lang_keys else "0"
            registry_strict_audit_ok = "1" if include_core_lang_keys else "0"
            registry_defaults_ok = "1" if include_core_lang_keys else "0"
            profile_matrix_status = "bad" if broken_profile_matrix_summary_value else "pass"
            profile_matrix_total_elapsed_ms_summary = "bad" if broken_profile_matrix_summary_value else "666"
            profile_matrix_report_value = "wrong.detjson" if broken_profile_matrix_report_mismatch else str(profile_matrix_selftest_path)
            summary_lines.append("[ci-gate-summary] ci_profile_matrix_gate_selftest_ok=1")
            if not broken_profile_matrix_summary_missing:
                summary_lines.append(
                    f"[ci-gate-summary] ci_profile_matrix_gate_selftest_report={profile_matrix_report_value}"
                )
                summary_lines.append(
                    f"[ci-gate-summary] ci_profile_matrix_gate_selftest_status={profile_matrix_status}"
                )
                summary_lines.append(
                    "[ci-gate-summary] ci_profile_matrix_gate_selftest_total_elapsed_ms="
                    f"{profile_matrix_total_elapsed_ms_summary}"
                )
                summary_lines.append(
                    "[ci-gate-summary] ci_profile_matrix_gate_selftest_selected_real_profiles="
                    "core_lang,full,seamgrim"
                )
                summary_lines.append("[ci-gate-summary] ci_profile_matrix_gate_selftest_skipped_real_profiles=-")
                summary_lines.append(
                    "[ci-gate-summary] ci_profile_matrix_gate_selftest_step_timeout_defaults="
                    "core_lang:900,full:1200,seamgrim:1500"
                )
                summary_lines.append("[ci-gate-summary] ci_profile_matrix_gate_selftest_core_lang_elapsed_ms=111")
                summary_lines.append("[ci-gate-summary] ci_profile_matrix_gate_selftest_full_elapsed_ms=222")
            summary_lines.append("[ci-gate-summary] ci_profile_matrix_gate_selftest_seamgrim_elapsed_ms=333")
            summary_lines.append("[ci-gate-summary] age5_close_digest_selftest_ok=1")
            summary_lines.append(f"[ci-gate-summary] {AGE4_PROOF_OK_KEY}={age4_proof_ok}")
            summary_lines.append(f"[ci-gate-summary] {AGE4_PROOF_FAILED_CRITERIA_KEY}={age4_proof_failed_criteria}")
            summary_lines.append(f"[ci-gate-summary] {AGE4_PROOF_FAILED_PREVIEW_KEY}={age4_proof_failed_preview}")
            summary_lines.append(f"[ci-gate-summary] {AGE4_PROOF_SUMMARY_HASH_KEY}={age4_proof_summary_hash}")
            omit_age_close_keys = {str(key).strip() for key in omit_age_close_status_summary_keys if str(key).strip()}
            if "age2_status" not in omit_age_close_keys:
                summary_lines.append(f"[ci-gate-summary] age2_status={age2_close_report_path}")
            if "age3_status" not in omit_age_close_keys:
                summary_lines.append(f"[ci-gate-summary] age3_status={age3_close_status_report_path}")
            if "age4_status" not in omit_age_close_keys:
                summary_lines.append(f"[ci-gate-summary] age4_status={age4_close_report_path}")
            if "age5_status" not in omit_age_close_keys:
                summary_lines.append(f"[ci-gate-summary] age5_status={age5_close_report_path}")
            summary_lines.append(f"[ci-gate-summary] ci_pack_golden_overlay_compare_selftest_ok={compare_ok}")
            summary_lines.append(f"[ci-gate-summary] ci_pack_golden_overlay_session_selftest_ok={session_ok}")
            summary_lines.append("[ci-gate-summary] seamgrim_group_id_summary_status=ok")
            for key in AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS:
                value = age5_child_summary_fields[key]
                if broken_age5_child_summary_summary_mismatch and key == "age5_combined_heavy_runtime_helper_negative_status":
                    value = "fail"
                summary_lines.append(f"[ci-gate-summary] {key}={value}")
            summary_lines.append(f"[ci-gate-summary] ci_sanity_gate_profile={sanity_profile}")
            summary_lines.append(f"[ci-gate-summary] ci_sanity_pipeline_emit_flags_ok={pipeline_emit_flags_ok}")
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_pipeline_emit_flags_selftest_ok="
                f"{pipeline_emit_flags_selftest_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_emit_artifacts_sanity_contract_selftest_ok="
                f"{emit_artifacts_sanity_contract_selftest_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_emit_artifacts_sanity_contract_selftest_ok="
                f"{emit_artifacts_sanity_contract_selftest_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age2_completion_gate_ok="
                f"{age2_completion_gate_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age2_completion_gate_selftest_ok="
                f"{age2_completion_gate_selftest_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age2_close_ok="
                f"{age2_close_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age2_close_selftest_ok="
                f"{age2_close_selftest_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age2_completion_gate_failure_codes="
                f"{age2_completion_gate_failure_codes}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age2_completion_gate_failure_code_count="
                f"{age2_completion_gate_failure_code_count}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age3_completion_gate_ok="
                f"{age3_completion_gate_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age3_completion_gate_selftest_ok="
                f"{age3_completion_gate_selftest_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age3_close_ok="
                f"{age3_close_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age3_close_selftest_ok="
                f"{age3_close_selftest_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age3_completion_gate_failure_codes="
                f"{age3_completion_gate_failure_codes}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age3_completion_gate_failure_code_count="
                f"{age3_completion_gate_failure_code_count}"
            )
            for sanity_key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS:
                summary_lines.append(f"[ci-gate-summary] {sanity_key}=1")
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age3_bogae_geoul_visibility_smoke_ok="
                f"{age3_bogae_geoul_visibility_smoke_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age3_bogae_geoul_visibility_smoke_report_path="
                f"{age3_bogae_geoul_visibility_smoke_report_path}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists="
                f"{age3_bogae_geoul_visibility_smoke_report_exists}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age3_bogae_geoul_visibility_smoke_schema="
                f"{age3_bogae_geoul_visibility_smoke_schema}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok="
                f"{age3_bogae_geoul_visibility_smoke_overall_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok="
                f"{age3_bogae_geoul_visibility_smoke_checks_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes="
                f"{age3_bogae_geoul_visibility_smoke_sim_state_hash_changes}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes="
                f"{age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_ok="
                f"{seamgrim_pack_evidence_tier_runner_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_report_path="
                f"{seamgrim_pack_evidence_tier_runner_report_path_text}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists="
                f"{seamgrim_pack_evidence_tier_runner_report_exists}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_schema="
                f"{seamgrim_pack_evidence_tier_runner_schema}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count="
                f"{seamgrim_pack_evidence_tier_runner_docs_issue_count}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count="
                f"{seamgrim_pack_evidence_tier_runner_repo_issue_count}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_ok="
                f"{seamgrim_numeric_factor_policy_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_report_path="
                f"{seamgrim_numeric_factor_policy_report_path_text}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_report_exists="
                f"{seamgrim_numeric_factor_policy_report_exists}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_schema="
                f"{seamgrim_numeric_factor_policy_schema}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_text="
                f"{seamgrim_numeric_factor_policy_text}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_bit_limit="
                f"{seamgrim_numeric_factor_policy_values['bit_limit']}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_pollard_iters="
                f"{seamgrim_numeric_factor_policy_values['pollard_iters']}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds="
                f"{seamgrim_numeric_factor_policy_values['pollard_c_seeds']}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds="
                f"{seamgrim_numeric_factor_policy_values['pollard_x0_seeds']}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_fallback_limit="
                f"{seamgrim_numeric_factor_policy_values['fallback_limit']}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_numeric_factor_policy_small_prime_max="
                f"{seamgrim_numeric_factor_policy_values['small_prime_max']}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_ok="
                f"{seamgrim_wasm_web_step_check_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_report_path="
                f"{seamgrim_wasm_web_step_check_report_path_text}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_report_exists="
                f"{seamgrim_wasm_web_step_check_report_exists}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_schema="
                f"{seamgrim_wasm_web_step_check_schema}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_checked_files="
                f"{seamgrim_wasm_web_step_check_checked_files}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_missing_count="
                f"{seamgrim_wasm_web_step_check_missing_count}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_age5_combined_heavy_policy_selftest_ok="
                f"{age5_combined_heavy_policy_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok="
                f"{profile_matrix_policy_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_dynamic_source_profile_split_selftest_ok="
                f"{dynamic_source_profile_split_selftest_ok}"
            )
            for key, value in sanity_contract_fields.items():
                summary_lines.append(f"[ci-gate-summary] {key}={value}")
            summary_lines.append(
                f"[ci-gate-summary] ci_sanity_pack_golden_lang_consistency_ok={lang_consistency_ok}"
            )
            summary_lines.append(
                f"[ci-gate-summary] ci_sanity_pack_golden_metadata_ok={pack_golden_metadata_ok}"
            )
            summary_lines.append(
                f"[ci-gate-summary] ci_sanity_pack_golden_graph_export_ok={pack_golden_graph_export_ok}"
            )
            summary_lines.append(
                f"[ci-gate-summary] ci_sanity_canon_ast_dpack_ok={canon_ast_dpack_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_contract_tier_unsupported_ok="
                f"{contract_tier_unsupported_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_contract_tier_age3_min_enforcement_ok="
                f"{contract_tier_age3_min_enforcement_ok}"
            )
            summary_lines.append(
                f"[ci-gate-summary] ci_sanity_stdlib_catalog_ok={stdlib_catalog_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_stdlib_catalog_selftest_ok="
                f"{stdlib_catalog_selftest_ok}"
            )
            summary_lines.append(
                f"[ci-gate-summary] ci_sanity_tensor_v0_pack_ok={tensor_v0_pack_ok}"
            )
            summary_lines.append(
                f"[ci-gate-summary] ci_sanity_tensor_v0_cli_ok={tensor_v0_cli_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_contract_ok="
                f"{fixed64_darwin_real_report_contract_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_live_ok="
                f"{fixed64_darwin_real_report_live_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_readiness_selftest_ok="
                f"{fixed64_darwin_real_report_readiness_selftest_ok}"
            )
            summary_lines.append(
                f"[ci-gate-summary] ci_sanity_map_access_contract_ok={map_access_contract_ok}"
            )
            summary_lines.append(
                f"[ci-gate-summary] ci_sanity_registry_strict_audit_ok={registry_strict_audit_ok}"
            )
            summary_lines.append(
                f"[ci-gate-summary] ci_sanity_registry_defaults_ok={registry_defaults_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_pipeline_emit_flags_ok="
                f"{pipeline_emit_flags_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_pipeline_emit_flags_selftest_ok="
                f"{pipeline_emit_flags_selftest_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok="
                f"{emit_artifacts_sanity_contract_selftest_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok="
                f"{sync_pack_golden_graph_export_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok="
                f"{emit_artifacts_sanity_contract_selftest_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age2_completion_gate_ok="
                f"{age2_completion_gate_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age2_completion_gate_selftest_ok="
                f"{age2_completion_gate_selftest_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age2_close_ok="
                f"{age2_close_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age2_close_selftest_ok="
                f"{age2_close_selftest_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age2_completion_gate_failure_codes="
                f"{age2_completion_gate_failure_codes}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age2_completion_gate_failure_code_count="
                f"{age2_completion_gate_failure_code_count}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_completion_gate_ok="
                f"{age3_completion_gate_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_completion_gate_selftest_ok="
                f"{age3_completion_gate_selftest_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_close_ok="
                f"{age3_close_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_close_selftest_ok="
                f"{age3_close_selftest_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_completion_gate_failure_codes="
                f"{age3_completion_gate_failure_codes}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_completion_gate_failure_code_count="
                f"{age3_completion_gate_failure_code_count}"
            )
            for _sanity_key, sync_key in AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS:
                summary_lines.append(f"[ci-gate-summary] {sync_key}=1")
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_ok="
                f"{age3_bogae_geoul_visibility_smoke_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_path="
                f"{age3_bogae_geoul_visibility_smoke_report_path}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists="
                f"{age3_bogae_geoul_visibility_smoke_report_exists}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_schema="
                f"{age3_bogae_geoul_visibility_smoke_schema}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok="
                f"{age3_bogae_geoul_visibility_smoke_overall_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok="
                f"{age3_bogae_geoul_visibility_smoke_checks_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes="
                f"{age3_bogae_geoul_visibility_smoke_sim_state_hash_changes}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes="
                f"{age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok="
                f"{seamgrim_pack_evidence_tier_runner_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_path="
                f"{seamgrim_pack_evidence_tier_runner_report_path_text}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists="
                f"{seamgrim_pack_evidence_tier_runner_report_exists}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_schema="
                f"{seamgrim_pack_evidence_tier_runner_schema}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count="
                f"{seamgrim_pack_evidence_tier_runner_docs_issue_count}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count="
                f"{seamgrim_pack_evidence_tier_runner_repo_issue_count}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok="
                f"{seamgrim_numeric_factor_policy_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_report_path="
                f"{seamgrim_numeric_factor_policy_report_path_text}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_report_exists="
                f"{seamgrim_numeric_factor_policy_report_exists}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_schema="
                f"{seamgrim_numeric_factor_policy_schema}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_text="
                f"{seamgrim_numeric_factor_policy_text}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_bit_limit="
                f"{seamgrim_numeric_factor_policy_values['bit_limit']}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_iters="
                f"{seamgrim_numeric_factor_policy_values['pollard_iters']}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds="
                f"{seamgrim_numeric_factor_policy_values['pollard_c_seeds']}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds="
                f"{seamgrim_numeric_factor_policy_values['pollard_x0_seeds']}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_fallback_limit="
                f"{seamgrim_numeric_factor_policy_values['fallback_limit']}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_small_prime_max="
                f"{seamgrim_numeric_factor_policy_values['small_prime_max']}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_ok="
                f"{seamgrim_wasm_web_step_check_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_path="
                f"{seamgrim_wasm_web_step_check_report_path_text}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_exists="
                f"{seamgrim_wasm_web_step_check_report_exists}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_schema="
                f"{seamgrim_wasm_web_step_check_schema}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_checked_files="
                f"{seamgrim_wasm_web_step_check_checked_files}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_missing_count="
                f"{seamgrim_wasm_web_step_check_missing_count}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_age5_combined_heavy_policy_selftest_ok="
                f"{age5_combined_heavy_policy_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok="
                f"{profile_matrix_policy_ok}"
            )
            summary_lines.append(
                "[ci-gate-summary] ci_sync_readiness_ci_sanity_dynamic_source_profile_split_selftest_ok="
                f"{dynamic_source_profile_split_selftest_ok}"
            )
            for key, value in sync_contract_fields.items():
                summary_lines.append(f"[ci-gate-summary] {key}={value}")
        write_text(summary_path, "\n".join(summary_lines))
    else:
        write_text(
            summary_path,
            "\n".join(
                [
                    "[ci-gate-summary] FAIL",
                    f"[ci-gate-summary] failed_steps={sample_step_id}",
                    f"[ci-gate-summary] failed_step_detail={sample_step_id} rc=1 cmd=python tests/run_sample_fail.py",
                    f"[ci-gate-summary] failed_step_logs={sample_step_id} stdout={sample_stdout_path} stderr={sample_stderr_path}",
                    "[ci-gate-summary] seamgrim_group_id_summary_status=ok",
                    *[f"[ci-gate-summary] {key}={value}" for key, value in age5_child_summary_fields.items()],
                    *[
                        f"[ci-gate-summary] {key}={value}"
                        for key, value in age5_child_summary_default_transport.items()
                    ],
                ]
            ),
        )
    summary_line = (
        f"ci_gate_result_status={status} ok={1 if status == 'pass' else 0} "
        f"overall_ok={1 if status == 'pass' else 0} failed_steps={failed_steps_count} "
        f"aggregate_status={status} reason=-"
    )
    write_text(summary_line_path, summary_line)
    write_text(ci_gate_result_line_path, summary_line)
    write_json(
        result_path,
        {
            "schema": "ddn.ci.gate_result.v1",
            "status": status,
            "ok": status == "pass",
            "reason": "-",
            "failed_steps": failed_steps_count,
        },
    )
    if with_runtime5_checklist:
        runtime5_showcase_elapsed_ms = 999 if broken_runtime5_report_mismatch else 777
        write_json(
            runtime5_checklist_path,
            {
                "schema": "seamgrim.runtime_5min_checklist.v1",
                "generated_at_utc": "2026-03-07T00:00:00+00:00",
                "ok": True,
                "runtime_report_path": "build/reports/sample_runtime.detjson",
                "browse_report_path": "build/reports/sample_browse.detjson",
                "base_url": "http://127.0.0.1:8787",
                "items": [
                    {
                        "name": "rewrite_motion_projectile_fallback",
                        "label": "Rewrite 운동/포물선 보개 폴백 점검",
                        "ok": True,
                        "elapsed_ms": 321,
                        "returncode": 0,
                    },
                    {
                        "name": "moyang_view_boundary_pack_check",
                        "label": "모양 view boundary 팩 점검",
                        "ok": True,
                        "elapsed_ms": 654,
                        "returncode": 0,
                    },
                    {
                        "name": "pendulum_tetris_showcase_check",
                        "label": "진자+테트리스 쇼케이스 점검",
                        "ok": True,
                        "elapsed_ms": runtime5_showcase_elapsed_ms,
                        "returncode": 0,
                    },
                ],
                "failed_items": [],
            },
        )
    if with_sanity:
        sanity_status = "pass" if status == "pass" else "fail"
        if broken_sanity_status:
            sanity_status = "fail" if sanity_status == "pass" else "pass"
        if sanity_status == "pass":
            sanity_code = "OK"
            sanity_step = "all"
            sanity_steps = [
                {"step": "backup_hygiene_selftest", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {"step": "pipeline_emit_flags_check", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {"step": "pipeline_emit_flags_selftest", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {
                    "step": "ci_emit_artifacts_sanity_contract_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "age5_combined_heavy_policy_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "profile_matrix_full_real_smoke_policy_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "profile_matrix_full_real_smoke_check_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "age2_completion_gate",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "age2_completion_gate_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "age2_close_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "age2_close",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "age3_completion_gate",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "age3_completion_gate_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "age3_close_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "age3_close",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "fixed64_darwin_real_report_contract_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "fixed64_darwin_real_report_live_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "fixed64_darwin_real_report_readiness_check_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {"step": "ci_profile_split_contract_check", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {
                    "step": "ci_profile_matrix_lightweight_contract_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "ci_profile_matrix_snapshot_helper_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "ci_sanity_dynamic_source_profile_split_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {"step": "contract_tier_unsupported_check", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {
                    "step": "contract_tier_age3_min_enforcement_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {"step": "map_access_contract_check", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {"step": "gaji_registry_strict_audit_check", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {"step": "gaji_registry_defaults_check", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {"step": "stdlib_catalog_check", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {"step": "stdlib_catalog_check_selftest", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {"step": "tensor_v0_pack_check", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {"step": "tensor_v0_cli_check", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {
                    "step": "seamgrim_ci_gate_sam_seulgi_family_step_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {"step": "seamgrim_ci_gate_seed_meta_step_check", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {
                    "step": "seamgrim_ci_gate_runtime5_passthrough_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_ci_gate_lesson_warning_step_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_ci_gate_stateful_preview_step_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_ci_gate_wasm_web_smoke_step_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_ci_gate_pack_evidence_tier_step_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_ci_gate_pack_evidence_tier_step_check_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_ci_gate_pack_evidence_tier_runner_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_ci_gate_pack_evidence_tier_report_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_ci_gate_pack_evidence_tier_report_check_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_interface_boundary_contract_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_overlay_session_wired_consistency_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_overlay_session_diag_parity_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_overlay_compare_diag_parity_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "age5_close_pack_contract_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "ci_pack_golden_age5_surface_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "ci_pack_golden_guideblock_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "ci_pack_golden_exec_policy_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "ci_pack_golden_jjaim_flatten_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "ci_pack_golden_event_model_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "ci_pack_golden_lang_consistency_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "ci_pack_golden_metadata_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "ci_pack_golden_graph_export_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "ci_canon_ast_dpack_selftest",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "w92_aot_pack_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "w93_universe_pack_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "w94_social_pack_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "w95_cert_pack_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "w96_somssi_pack_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "w97_self_heal_pack_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
                {
                    "step": "seamgrim_wasm_cli_diag_parity_check",
                    "ok": True,
                    "returncode": 0,
                    "cmd": ["python", "x.py"],
                },
            ]
            for featured_step in SEAMGRIM_FEATURED_SEED_STEP_CONTRACT_STEPS:
                sanity_steps.append(
                    {
                        "step": featured_step,
                        "ok": True,
                        "returncode": 0,
                        "cmd": ["python", "x.py"],
                    }
                )
                if not any(row.get("step") == featured_step for row in sanity_steps):
                    return fail(f"internal fixture error: missing featured step {featured_step}")
            for blocker_step in SEAMGRIM_BLOCKER_STEP_CONTRACT_STEPS:
                sanity_steps.append(
                    {
                        "step": blocker_step,
                        "ok": True,
                        "returncode": 0,
                        "cmd": ["python", "x.py"],
                    }
                )
                if not any(row.get("step") == blocker_step for row in sanity_steps):
                    return fail(f"internal fixture error: missing blocker step {blocker_step}")
            for platform_step in SEAMGRIM_PLATFORM_STEP_CONTRACT_STEPS:
                sanity_steps.append(
                    {
                        "step": platform_step,
                        "ok": True,
                        "returncode": 0,
                        "cmd": ["python", "x.py"],
                    }
                )
                if not any(row.get("step") == platform_step for row in sanity_steps):
                    return fail(f"internal fixture error: missing platform step {platform_step}")
            if not any(row.get("step") == "seamgrim_v2_task_batch_check" for row in sanity_steps):
                sanity_steps.append(
                    {
                        "step": "seamgrim_v2_task_batch_check",
                        "ok": True,
                        "returncode": 0,
                        "cmd": ["python", "x.py"],
                    }
                )
            if broken_sanity_required_step_missing:
                sanity_steps = [row for row in sanity_steps if row.get("step") != "age5_close_pack_contract_selftest"]
            if broken_sanity_required_step_failed:
                for row in sanity_steps:
                    if row.get("step") == "seamgrim_overlay_session_diag_parity_check":
                        row["ok"] = True
                        row["returncode"] = 1
                        break
            if broken_sanity_product_blocker_step_missing:
                sanity_steps = [row for row in sanity_steps if row.get("step") != "seamgrim_product_blocker_bundle_check"]
            if broken_sanity_product_blocker_step_failed:
                for row in sanity_steps:
                    if row.get("step") == "seamgrim_product_blocker_bundle_check":
                        row["ok"] = True
                        row["returncode"] = 1
                        break
            if broken_sanity_observe_output_contract_step_missing:
                sanity_steps = [row for row in sanity_steps if row.get("step") != "seamgrim_observe_output_contract_check"]
            if broken_sanity_observe_output_contract_step_failed:
                for row in sanity_steps:
                    if row.get("step") == "seamgrim_observe_output_contract_check":
                        row["ok"] = True
                        row["returncode"] = 1
                        break
            if broken_sanity_runtime_view_source_strict_step_missing:
                sanity_steps = [row for row in sanity_steps if row.get("step") != "seamgrim_runtime_view_source_strict_check"]
            if broken_sanity_runtime_view_source_strict_step_failed:
                for row in sanity_steps:
                    if row.get("step") == "seamgrim_runtime_view_source_strict_check":
                        row["ok"] = True
                        row["returncode"] = 1
                        break
            if broken_sanity_run_legacy_autofix_step_missing:
                sanity_steps = [row for row in sanity_steps if row.get("step") != "seamgrim_run_legacy_autofix_check"]
            if broken_sanity_run_legacy_autofix_step_failed:
                for row in sanity_steps:
                    if row.get("step") == "seamgrim_run_legacy_autofix_check":
                        row["ok"] = True
                        row["returncode"] = 1
                        break
            if broken_sanity_wired_step_missing:
                sanity_steps = [row for row in sanity_steps if row.get("step") != "seamgrim_overlay_session_wired_consistency_check"]
            if broken_sanity_wired_step_failed:
                for row in sanity_steps:
                    if row.get("step") == "seamgrim_overlay_session_wired_consistency_check":
                        row["ok"] = True
                        row["returncode"] = 1
                        break
            if broken_sanity_compare_step_missing:
                sanity_steps = [row for row in sanity_steps if row.get("step") != "seamgrim_overlay_compare_diag_parity_check"]
            if broken_sanity_compare_step_failed:
                for row in sanity_steps:
                    if row.get("step") == "seamgrim_overlay_compare_diag_parity_check":
                        row["ok"] = True
                        row["returncode"] = 1
                        break
            if broken_sanity_wasm_web_selftest_step_missing:
                sanity_steps = [
                    row for row in sanity_steps if row.get("step") != "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest"
                ]
            if broken_sanity_wasm_web_selftest_step_failed:
                for row in sanity_steps:
                    if row.get("step") == "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest":
                        row["ok"] = True
                        row["returncode"] = 1
                        break
        else:
            sanity_code = "E_CI_SANITY_SAMPLE_FAIL"
            sanity_step = "pipeline_emit_flags_check"
            sanity_steps = [
                {"step": "backup_hygiene_selftest", "ok": True, "returncode": 0, "cmd": ["python", "x.py"]},
                {"step": "pipeline_emit_flags_check", "ok": False, "returncode": 1, "cmd": ["python", "x.py"]},
            ]
        write_json(
            sanity_path,
            {
                "schema": "ddn.ci.sanity_gate.v1" if not broken_sanity_schema else "broken.schema",
                "generated_at_utc": "2026-03-02T00:00:00+00:00",
                "status": sanity_status,
                "code": sanity_code,
                "step": sanity_step,
                "profile": sanity_profile,
                "ci_sanity_pipeline_emit_flags_ok": pipeline_emit_flags_ok,
                "ci_sanity_pipeline_emit_flags_selftest_ok": pipeline_emit_flags_selftest_ok,
                "ci_sanity_emit_artifacts_sanity_contract_selftest_ok": emit_artifacts_sanity_contract_selftest_ok,
                "ci_sanity_pack_golden_graph_export_ok": "1" if include_core_lang_keys else "0",
                "ci_sanity_age2_completion_gate_ok": age2_completion_gate_ok,
                "ci_sanity_age2_completion_gate_selftest_ok": age2_completion_gate_selftest_ok,
                "ci_sanity_age2_close_ok": age2_close_ok,
                "ci_sanity_age2_close_selftest_ok": age2_close_selftest_ok,
                "ci_sanity_age2_completion_gate_failure_codes": age2_completion_gate_failure_codes,
                "ci_sanity_age2_completion_gate_failure_code_count": age2_completion_gate_failure_code_count,
                "ci_sanity_age3_completion_gate_ok": age3_completion_gate_ok,
                "ci_sanity_age3_completion_gate_selftest_ok": age3_completion_gate_selftest_ok,
                "ci_sanity_age3_close_ok": age3_close_ok,
                "ci_sanity_age3_close_selftest_ok": age3_close_selftest_ok,
                "ci_sanity_age3_completion_gate_failure_codes": age3_completion_gate_failure_codes,
                "ci_sanity_age3_completion_gate_failure_code_count": age3_completion_gate_failure_code_count,
                **{key: "1" for key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS},
                "ci_sanity_age3_bogae_geoul_visibility_smoke_ok": age3_bogae_geoul_visibility_smoke_ok,
                "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path": str(
                    age3_bogae_geoul_visibility_smoke_report_path
                ),
                "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists": (
                    age3_bogae_geoul_visibility_smoke_report_exists
                ),
                "ci_sanity_age3_bogae_geoul_visibility_smoke_schema": (
                    age3_bogae_geoul_visibility_smoke_schema
                ),
                "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok": (
                    age3_bogae_geoul_visibility_smoke_overall_ok
                ),
                "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok": (
                    age3_bogae_geoul_visibility_smoke_checks_ok
                ),
                "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes": (
                    age3_bogae_geoul_visibility_smoke_sim_state_hash_changes
                ),
                "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes": (
                    age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes
                ),
                "ci_sanity_seamgrim_pack_evidence_tier_runner_ok": seamgrim_pack_evidence_tier_runner_ok,
                "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path": (
                    seamgrim_pack_evidence_tier_runner_report_path_text
                ),
                "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists": (
                    seamgrim_pack_evidence_tier_runner_report_exists
                ),
                "ci_sanity_seamgrim_pack_evidence_tier_runner_schema": (
                    sanity_seamgrim_pack_evidence_tier_runner_schema
                ),
                "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count": (
                    sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count
                ),
                "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count": (
                    sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count
                ),
                "ci_sanity_seamgrim_numeric_factor_policy_ok": seamgrim_numeric_factor_policy_ok,
                "ci_sanity_seamgrim_numeric_factor_policy_report_path": (
                    seamgrim_numeric_factor_policy_report_path_text
                ),
                "ci_sanity_seamgrim_numeric_factor_policy_report_exists": (
                    seamgrim_numeric_factor_policy_report_exists
                ),
                "ci_sanity_seamgrim_numeric_factor_policy_schema": seamgrim_numeric_factor_policy_schema,
                "ci_sanity_seamgrim_numeric_factor_policy_text": seamgrim_numeric_factor_policy_text,
                "ci_sanity_seamgrim_numeric_factor_policy_bit_limit": seamgrim_numeric_factor_policy_values[
                    "bit_limit"
                ],
                "ci_sanity_seamgrim_numeric_factor_policy_pollard_iters": seamgrim_numeric_factor_policy_values[
                    "pollard_iters"
                ],
                "ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds": seamgrim_numeric_factor_policy_values[
                    "pollard_c_seeds"
                ],
                "ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds": seamgrim_numeric_factor_policy_values[
                    "pollard_x0_seeds"
                ],
                "ci_sanity_seamgrim_numeric_factor_policy_fallback_limit": seamgrim_numeric_factor_policy_values[
                    "fallback_limit"
                ],
                "ci_sanity_seamgrim_numeric_factor_policy_small_prime_max": seamgrim_numeric_factor_policy_values[
                    "small_prime_max"
                ],
                "ci_sanity_seamgrim_wasm_web_step_check_ok": seamgrim_wasm_web_step_check_ok,
                "ci_sanity_seamgrim_wasm_web_step_check_report_path": (
                    seamgrim_wasm_web_step_check_report_path_text
                ),
                "ci_sanity_seamgrim_wasm_web_step_check_report_exists": (
                    seamgrim_wasm_web_step_check_report_exists
                ),
                "ci_sanity_seamgrim_wasm_web_step_check_schema": (
                    seamgrim_wasm_web_step_check_schema
                ),
                "ci_sanity_seamgrim_wasm_web_step_check_checked_files": (
                    sanity_seamgrim_wasm_web_step_check_checked_files
                ),
                "ci_sanity_seamgrim_wasm_web_step_check_missing_count": (
                    seamgrim_wasm_web_step_check_missing_count
                ),
                "ci_sanity_age5_combined_heavy_policy_selftest_ok": age5_combined_heavy_policy_ok,
                "ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok": profile_matrix_policy_ok,
                "ci_sanity_dynamic_source_profile_split_selftest_ok": dynamic_source_profile_split_selftest_ok,
                **sanity_contract_fields,
                "msg": "-",
                "steps": sanity_steps,
            },
        )
    if with_brief:
        brief_status = "fail" if broken_brief else status
        brief_reason = "bad_reason" if broken_brief else "-"
        brief_failed_steps = 99 if broken_brief else failed_steps_count
        brief_final_line = "-" if broken_brief else summary_line
        profile_matrix_report_doc = {
            "schema": "ddn.ci.profile_matrix_gate_selftest.v1",
            "status": "pass",
            "ok": True,
            "selected_real_profiles": ["core_lang", "full", "seamgrim"],
            "skipped_real_profiles": [],
            "step_timeout_defaults_text": PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
            "step_timeout_defaults_sec": dict(PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC),
            "step_timeout_env_keys": dict(PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS),
            "total_elapsed_ms": profile_matrix_total_elapsed_ms,
            "aggregate_summary_sanity_ok": True,
            "aggregate_summary_sanity_checked_profiles": list(PROFILE_MATRIX_SELFTEST_PROFILES),
            "aggregate_summary_sanity_failed_profiles": [],
            "aggregate_summary_sanity_skipped_profiles": [],
            "aggregate_summary_sanity_by_profile": {},
            "real_profiles": {
                "core_lang": {
                    "selected": True,
                    "skipped": False,
                    "status": "pass",
                    "ok": True,
                    "total_elapsed_ms": profile_matrix_core_lang_elapsed_ms,
                },
                "full": {
                    "selected": True,
                    "skipped": False,
                    "status": "pass",
                    "ok": True,
                    "total_elapsed_ms": profile_matrix_full_elapsed_ms,
                },
                "seamgrim": {
                    "selected": True,
                    "skipped": False,
                    "status": "pass",
                    "ok": True,
                    "total_elapsed_ms": profile_matrix_seamgrim_elapsed_ms,
                },
            },
        }
        profile_matrix_snapshot = build_profile_matrix_snapshot_from_doc(
            profile_matrix_report_doc,
            report_path=str(profile_matrix_selftest_path),
        )
        brief_parts = [
            f"status={brief_status}",
            f'reason="{brief_reason}"',
            f"failed_steps_count={brief_failed_steps}",
            'failed_steps="-"',
            "top_step=-",
            'top_message="-"',
            f'final_line="{brief_final_line}"',
            f"age5_close_digest_selftest_ok={age5_digest_selftest_ok}",
            AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT,
            f"{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}={AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT}",
            f"{AGE4_PROOF_OK_KEY}={age4_proof_ok}",
            f"{AGE4_PROOF_FAILED_CRITERIA_KEY}={age4_proof_failed_criteria}",
            f"{AGE4_PROOF_FAILED_PREVIEW_KEY}={age4_proof_failed_preview}",
            f"{AGE4_PROOF_SUMMARY_HASH_KEY}={age4_proof_summary_hash}",
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
        ]
        brief_parts.extend(
            f"{key}={value}" for key, value in age5_child_summary_fields.items()
        )
        brief_parts.extend(
            f"{key}={value}" for key, value in age5_child_summary_default_transport.items()
        )
        if not broken_profile_matrix_brief_missing:
            if profile_matrix_snapshot is None:
                return fail("profile_matrix_snapshot build failed")
            brief_profile_payload = build_profile_matrix_brief_payload_from_snapshot(profile_matrix_snapshot)
            if broken_profile_matrix_brief_value:
                brief_profile_payload["profile_matrix_total_elapsed_ms"] = "bad"
            brief_parts.extend(
                [
                    f'profile_matrix_total_elapsed_ms={brief_profile_payload["profile_matrix_total_elapsed_ms"]}',
                    f'profile_matrix_selected_real_profiles="{brief_profile_payload["profile_matrix_selected_real_profiles"]}"',
                    f'profile_matrix_core_lang_elapsed_ms={brief_profile_payload["profile_matrix_core_lang_elapsed_ms"]}',
                    f'profile_matrix_full_elapsed_ms={brief_profile_payload["profile_matrix_full_elapsed_ms"]}',
                    f'profile_matrix_seamgrim_elapsed_ms={brief_profile_payload["profile_matrix_seamgrim_elapsed_ms"]}',
                ]
            )
        write_text(brief_path, " ".join(brief_parts))
    if with_triage:
        triage_final_line = "-" if broken_triage_final else summary_line
        failed_steps_rows: list[dict[str, object]] = []
        if failed_steps_count > 0:
            step_cmd_text = "python tests/run_sample_fail.py"
            row: dict[str, object] = {
                "message": "sample failure",
                "returncode": 1,
                "cmd": step_cmd_text,
                "fast_fail_step_detail": f"name={sample_step_id} rc=1 cmd={step_cmd_text}",
                "fast_fail_step_logs": (
                    f"name={sample_step_id} "
                    f"stdout={sample_stdout_path} stderr={sample_stderr_path}"
                ),
                "stdout_log_path": str(sample_stdout_path),
                "stdout_log_path_norm": str(sample_stdout_path).replace("\\", "/"),
                "stderr_log_path": str(sample_stderr_path),
                "stderr_log_path_norm": str(sample_stderr_path).replace("\\", "/"),
            }
            if triage_fail_step_use_name_field:
                row["name"] = sample_step_id
            else:
                row["step_id"] = sample_step_id
            failed_steps_rows.append(row)
        triage_summary_line_path = summary_line_path
        triage_summary_line_norm = str(summary_line_path).replace("\\", "/")
        if broken_artifact_ref:
            alt_summary_line_path = report_dir / f"{prefix}.alt_summary_line.txt"
            write_text(alt_summary_line_path, "ci_gate_result_status=pass ok=1 overall_ok=1 failed_steps=0")
            triage_summary_line_path = alt_summary_line_path
            triage_summary_line_norm = str(alt_summary_line_path).replace("\\", "/")
        force_false_keys = {str(key).strip() for key in force_artifact_exists_false if str(key).strip()}
        triage_failed_step_detail_rows_count = failed_steps_count
        if broken_triage_failed_step_detail_rows_count_mismatch:
            triage_failed_step_detail_rows_count += 1
        triage_failed_step_logs_rows_count = failed_steps_count
        if broken_triage_failed_step_logs_rows_count_mismatch:
            triage_failed_step_logs_rows_count += 1
        triage_failed_step_detail_rows_count_value: object = triage_failed_step_detail_rows_count
        triage_failed_step_logs_rows_count_value: object = triage_failed_step_logs_rows_count
        if broken_triage_failed_step_detail_rows_count_type:
            triage_failed_step_detail_rows_count_value = "bad"
        if broken_triage_failed_step_logs_rows_count_type:
            triage_failed_step_logs_rows_count_value = "bad"
        triage_failed_step_detail_order = [sample_step_id] if failed_steps_count > 0 else []
        if broken_triage_failed_step_detail_order_mismatch and failed_steps_count > 0:
            triage_failed_step_detail_order = [f"{sample_step_id}_broken"]
        triage_failed_step_logs_order = [sample_step_id] if failed_steps_count > 0 else []
        if broken_triage_failed_step_logs_order_mismatch and failed_steps_count > 0:
            triage_failed_step_logs_order = [f"{sample_step_id}_broken"]
        triage_failed_step_detail_order_value: object = triage_failed_step_detail_order
        triage_failed_step_logs_order_value: object = triage_failed_step_logs_order
        if broken_triage_failed_step_detail_order_type:
            triage_failed_step_detail_order_value = "bad"
        if broken_triage_failed_step_logs_order_type:
            triage_failed_step_logs_order_value = "bad"
        write_json(
            triage_path,
            {
                "schema": "ddn.ci.fail_triage.v1",
                "generated_at_utc": "2026-02-21T00:00:00+00:00",
                "status": status,
                "reason": "-",
                "report_prefix": prefix,
                "final_line": triage_final_line,
                "summary_verify_ok": False if broken_verify_issue else True,
                "summary_verify_issues": ["bad_issue_token"] if broken_verify_issue else [],
                "summary_verify_issues_count": 1 if broken_verify_issue else 0,
                "summary_verify_top_issue": "bad_issue_token" if broken_verify_issue else "-",
                "failed_steps": failed_steps_rows,
                "failed_steps_count": failed_steps_count,
                "failed_step_detail_rows_count": triage_failed_step_detail_rows_count_value,
                "failed_step_logs_rows_count": triage_failed_step_logs_rows_count_value,
                "failed_step_detail_order": triage_failed_step_detail_order_value,
                "failed_step_logs_order": triage_failed_step_logs_order_value,
                "aggregate_digest": [],
                "aggregate_digest_count": 0,
                "summary_report_path_hint": str(summary_path),
                "summary_report_path_hint_norm": (
                    "BROKEN/PATH" if broken_norm else str(summary_path).replace("\\", "/")
                ),
                AGE4_PROOF_OK_KEY: int(age4_proof_ok),
                AGE4_PROOF_FAILED_CRITERIA_KEY: int(age4_proof_failed_criteria),
                AGE4_PROOF_FAILED_PREVIEW_KEY: age4_proof_failed_preview,
                AGE4_PROOF_SUMMARY_HASH_KEY: age4_proof_summary_hash,
                "age5_close_digest_selftest_ok": age5_digest_selftest_ok,
                AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: (
                    "BROKEN"
                    if broken_age5_child_summary_default_triage_mismatch
                    else AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT
                ),
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
                "combined_digest_selftest_default_field": (
                    {"broken": "1"}
                    if broken_age5_child_summary_default_triage_mismatch
                    else AGE5_DIGEST_SELFTEST_DEFAULT_FIELD
                ),
                **(
                    {
                        **age5_child_summary_fields,
                        "age5_combined_heavy_runtime_helper_negative_status": "fail",
                    }
                    if broken_age5_child_summary_triage_mismatch
                    else age5_child_summary_fields
                ),
                **(
                    {
                        **age5_child_summary_default_transport,
                        "ci_sanity_age5_combined_heavy_child_summary_default_fields": "BROKEN",
                    }
                    if broken_age5_child_summary_default_triage_mismatch
                    else age5_child_summary_default_transport
                ),
                "profile_matrix_selftest": {
                    "report_path": str(profile_matrix_selftest_path),
                    "status": "pass",
                    "ok": True,
                    "total_elapsed_ms": (
                        1001 if broken_profile_matrix_triage_mismatch else profile_matrix_total_elapsed_ms
                    ),
                    "selected_real_profiles": profile_matrix_selected_real_profiles.split(","),
                    "skipped_real_profiles": [],
                    "step_timeout_defaults_text": PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
                    "step_timeout_defaults_sec": dict(PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC),
                    "step_timeout_env_keys": dict(PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS),
                    "core_lang_elapsed_ms": profile_matrix_core_lang_elapsed_ms,
                    "full_elapsed_ms": profile_matrix_full_elapsed_ms,
                    "seamgrim_elapsed_ms": profile_matrix_seamgrim_elapsed_ms,
                    "aggregate_summary_sanity_ok": True,
                    "aggregate_summary_sanity_checked_profiles": profile_matrix_checked_profiles.split(","),
                    "aggregate_summary_sanity_failed_profiles": [],
                    "aggregate_summary_sanity_skipped_profiles": [],
                    **{
                        f"{profile_name}_aggregate_summary_status": str(contract["status"])
                        for profile_name, contract in profile_matrix_expected_contracts.items()
                    },
                    **{
                        f"{profile_name}_aggregate_summary_ok": bool(contract["ok"])
                        for profile_name, contract in profile_matrix_expected_contracts.items()
                    },
                    **{
                        f"{profile_name}_aggregate_summary_values": str(contract["values_text"])
                        for profile_name, contract in profile_matrix_expected_contracts.items()
                    },
                },
                "artifacts": {
                    "summary": {
                        "path": str(summary_path),
                        "path_norm": "BROKEN/PATH" if broken_norm else str(summary_path).replace("\\", "/"),
                        "exists": False if "summary" in force_false_keys else True,
                    },
                    "summary_line": {
                        "path": str(triage_summary_line_path),
                        "path_norm": triage_summary_line_norm,
                        "exists": False if "summary_line" in force_false_keys else True,
                    },
                    "ci_gate_result_json": {
                        "path": str(result_path),
                        "path_norm": str(result_path).replace("\\", "/"),
                        "exists": False if "ci_gate_result_json" in force_false_keys else True,
                    },
                    "ci_fail_brief_txt": {
                        "path": str(brief_path),
                        "path_norm": str(brief_path).replace("\\", "/"),
                        "exists": (
                            False
                            if "ci_fail_brief_txt" in force_false_keys
                            else bool(with_brief)
                        ),
                    },
                    "ci_fail_triage_json": {
                        "path": str(triage_path),
                        "path_norm": str(triage_path).replace("\\", "/"),
                        "exists": False if "ci_fail_triage_json" in force_false_keys else True,
                    },
                },
            },
        )

    if with_sync_readiness:
        sync_status = "pass" if status == "pass" else "fail"
        if broken_sync_readiness_status_unsupported:
            sync_status = "unknown"
        elif broken_sync_readiness_status_mismatch:
            sync_status = "fail" if sync_status == "pass" else "pass"
        sync_code = "OK" if sync_status == "pass" else "E_SYNC_READINESS_STEP_FAIL"
        sync_step = "all" if sync_status == "pass" else "aggregate_gate"
        if broken_sync_readiness_pass_fields and sync_status == "pass":
            sync_code = "BROKEN"
        write_json(
            sync_readiness_path,
            {
                "schema": "ddn.ci.sync_readiness.v1" if not broken_sync_readiness_schema else "broken.schema",
                "generated_at_utc": "2026-03-02T00:00:00+00:00",
                "status": sync_status,
                "ok": sync_status == "pass",
                "code": sync_code,
                "step": sync_step,
                "sanity_profile": sanity_profile,
                "ci_sanity_pipeline_emit_flags_ok": pipeline_emit_flags_ok,
                "ci_sanity_pipeline_emit_flags_selftest_ok": pipeline_emit_flags_selftest_ok,
                "ci_sanity_emit_artifacts_sanity_contract_selftest_ok": emit_artifacts_sanity_contract_selftest_ok,
                "ci_sanity_pack_golden_graph_export_ok": "1" if include_core_lang_keys else "0",
                "ci_sanity_age2_completion_gate_ok": age2_completion_gate_ok,
                "ci_sanity_age2_completion_gate_selftest_ok": age2_completion_gate_selftest_ok,
                "ci_sanity_age2_close_ok": age2_close_ok,
                "ci_sanity_age2_close_selftest_ok": age2_close_selftest_ok,
                "ci_sanity_age2_completion_gate_failure_codes": age2_completion_gate_failure_codes,
                "ci_sanity_age2_completion_gate_failure_code_count": age2_completion_gate_failure_code_count,
                "ci_sanity_age3_completion_gate_ok": age3_completion_gate_ok,
                "ci_sanity_age3_completion_gate_selftest_ok": age3_completion_gate_selftest_ok,
                "ci_sanity_age3_close_ok": age3_close_ok,
                "ci_sanity_age3_close_selftest_ok": age3_close_selftest_ok,
                "ci_sanity_age3_completion_gate_failure_codes": age3_completion_gate_failure_codes,
                "ci_sanity_age3_completion_gate_failure_code_count": age3_completion_gate_failure_code_count,
                **{key: "1" for key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS},
                **{
                    sync_key: "1"
                    for _sanity_key, sync_key in AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS
                },
                "ci_sanity_age3_bogae_geoul_visibility_smoke_ok": age3_bogae_geoul_visibility_smoke_ok,
                "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path": str(
                    age3_bogae_geoul_visibility_smoke_report_path
                ),
                "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists": (
                    age3_bogae_geoul_visibility_smoke_report_exists
                ),
                "ci_sanity_age3_bogae_geoul_visibility_smoke_schema": (
                    age3_bogae_geoul_visibility_smoke_schema
                ),
                "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok": (
                    age3_bogae_geoul_visibility_smoke_overall_ok
                ),
                "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok": (
                    age3_bogae_geoul_visibility_smoke_checks_ok
                ),
                "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes": (
                    age3_bogae_geoul_visibility_smoke_sim_state_hash_changes
                ),
                "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes": (
                    age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes
                ),
                "ci_sanity_seamgrim_pack_evidence_tier_runner_ok": seamgrim_pack_evidence_tier_runner_ok,
                "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path": (
                    seamgrim_pack_evidence_tier_runner_report_path_text
                ),
                "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists": (
                    seamgrim_pack_evidence_tier_runner_report_exists
                ),
                "ci_sanity_seamgrim_pack_evidence_tier_runner_schema": (
                    sync_seamgrim_pack_evidence_tier_runner_schema
                ),
                "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count": (
                    sync_seamgrim_pack_evidence_tier_runner_docs_issue_count
                ),
                "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count": (
                    seamgrim_pack_evidence_tier_runner_repo_issue_count
                ),
                "ci_sanity_seamgrim_numeric_factor_policy_ok": seamgrim_numeric_factor_policy_ok,
                "ci_sanity_seamgrim_numeric_factor_policy_report_path": (
                    seamgrim_numeric_factor_policy_report_path_text
                ),
                "ci_sanity_seamgrim_numeric_factor_policy_report_exists": (
                    seamgrim_numeric_factor_policy_report_exists
                ),
                "ci_sanity_seamgrim_numeric_factor_policy_schema": seamgrim_numeric_factor_policy_schema,
                "ci_sanity_seamgrim_numeric_factor_policy_text": seamgrim_numeric_factor_policy_text,
                "ci_sanity_seamgrim_numeric_factor_policy_bit_limit": seamgrim_numeric_factor_policy_values[
                    "bit_limit"
                ],
                "ci_sanity_seamgrim_numeric_factor_policy_pollard_iters": seamgrim_numeric_factor_policy_values[
                    "pollard_iters"
                ],
                "ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds": seamgrim_numeric_factor_policy_values[
                    "pollard_c_seeds"
                ],
                "ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds": seamgrim_numeric_factor_policy_values[
                    "pollard_x0_seeds"
                ],
                "ci_sanity_seamgrim_numeric_factor_policy_fallback_limit": seamgrim_numeric_factor_policy_values[
                    "fallback_limit"
                ],
                "ci_sanity_seamgrim_numeric_factor_policy_small_prime_max": seamgrim_numeric_factor_policy_values[
                    "small_prime_max"
                ],
                "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok": (
                    seamgrim_pack_evidence_tier_runner_ok
                ),
                "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_path": (
                    seamgrim_pack_evidence_tier_runner_report_path_text
                ),
                "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists": (
                    seamgrim_pack_evidence_tier_runner_report_exists
                ),
                "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_schema": (
                    sync_seamgrim_pack_evidence_tier_runner_schema
                ),
                "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count": (
                    sync_seamgrim_pack_evidence_tier_runner_docs_issue_count
                ),
                "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count": (
                    seamgrim_pack_evidence_tier_runner_repo_issue_count
                ),
                "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok": (
                    seamgrim_numeric_factor_policy_ok
                ),
                "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_report_path": (
                    seamgrim_numeric_factor_policy_report_path_text
                ),
                "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_report_exists": (
                    seamgrim_numeric_factor_policy_report_exists
                ),
                "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_schema": (
                    seamgrim_numeric_factor_policy_schema
                ),
                "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_text": (
                    seamgrim_numeric_factor_policy_text
                ),
                "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_bit_limit": (
                    seamgrim_numeric_factor_policy_values["bit_limit"]
                ),
                "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_iters": (
                    seamgrim_numeric_factor_policy_values["pollard_iters"]
                ),
                "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds": (
                    seamgrim_numeric_factor_policy_values["pollard_c_seeds"]
                ),
                "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds": (
                    seamgrim_numeric_factor_policy_values["pollard_x0_seeds"]
                ),
                "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_fallback_limit": (
                    seamgrim_numeric_factor_policy_values["fallback_limit"]
                ),
                "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_small_prime_max": (
                    seamgrim_numeric_factor_policy_values["small_prime_max"]
                ),
                "ci_sanity_seamgrim_wasm_web_step_check_ok": seamgrim_wasm_web_step_check_ok,
                "ci_sanity_seamgrim_wasm_web_step_check_report_path": (
                    seamgrim_wasm_web_step_check_report_path_text
                ),
                "ci_sanity_seamgrim_wasm_web_step_check_report_exists": (
                    seamgrim_wasm_web_step_check_report_exists
                ),
                "ci_sanity_seamgrim_wasm_web_step_check_schema": (
                    seamgrim_wasm_web_step_check_schema
                ),
                "ci_sanity_seamgrim_wasm_web_step_check_checked_files": (
                    sync_seamgrim_wasm_web_step_check_checked_files
                ),
                "ci_sanity_seamgrim_wasm_web_step_check_missing_count": (
                    seamgrim_wasm_web_step_check_missing_count
                ),
                "ci_sanity_age5_combined_heavy_policy_selftest_ok": age5_combined_heavy_policy_ok,
                "ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok": profile_matrix_policy_ok,
                "ci_sanity_dynamic_source_profile_split_selftest_ok": dynamic_source_profile_split_selftest_ok,
                "ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok": (
                    emit_artifacts_sanity_contract_selftest_ok
                ),
                "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok": (
                    "1" if include_core_lang_keys else "0"
                ),
                **sanity_contract_fields,
                **sync_contract_fields,
                "msg": "-",
                "steps": [],
                "steps_count": 0,
            },
        )

    write_json(
        profile_matrix_selftest_path,
        {
            "schema": "ddn.ci.profile_matrix_gate_selftest.v1",
            "generated_at_utc": "2026-03-07T00:00:00+00:00",
            "status": "pass",
            "ok": True,
            "selected_real_profiles": ["core_lang", "full", "seamgrim"],
            "skipped_real_profiles": [],
            "step_timeout_defaults_text": PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
            "step_timeout_defaults_sec": dict(PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC),
            "step_timeout_env_keys": dict(PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS),
            "total_elapsed_ms": profile_matrix_total_elapsed_ms,
            "aggregate_summary_sanity_ok": True,
            "aggregate_summary_sanity_checked_profiles": list(PROFILE_MATRIX_SELFTEST_PROFILES),
            "aggregate_summary_sanity_failed_profiles": [],
            "aggregate_summary_sanity_skipped_profiles": [],
            "aggregate_summary_sanity_by_profile": {
                profile_name: {
                    "expected_present": bool(contract["expected_present"]),
                    "present": True,
                    "status": str(contract["status"]),
                    "reason": "ok",
                    "expected_profile": str(contract["expected_profile"]),
                    "expected_sync_profile": str(contract["expected_sync_profile"]),
                    "profile": str(contract["expected_profile"]),
                    "sync_profile": str(contract["expected_sync_profile"]),
                    "expected_values": dict(contract["values"]),
                    "values": dict(contract["values"]),
                    "missing_keys": [],
                    "mismatched_keys": [],
                    "profile_ok": True,
                    "sync_profile_ok": True,
                    "values_ok": True,
                    "gate_marker_expected": bool(contract["gate_marker_expected"]),
                    "gate_marker_present": bool(contract["gate_marker_expected"]),
                    "gate_marker_ok": True,
                    "ok": bool(contract["ok"]),
                }
                for profile_name, contract in profile_matrix_expected_contracts.items()
            },
            "real_profiles": {
                "core_lang": {
                    "selected": True,
                    "skipped": False,
                    "status": "pass",
                    "ok": True,
                    "total_elapsed_ms": profile_matrix_core_lang_elapsed_ms,
                },
                "full": {
                    "selected": True,
                    "skipped": False,
                    "status": "pass",
                    "ok": True,
                    "total_elapsed_ms": profile_matrix_full_elapsed_ms,
                },
                "seamgrim": {
                    "selected": True,
                    "skipped": False,
                    "status": "pass",
                    "ok": True,
                    "total_elapsed_ms": profile_matrix_seamgrim_elapsed_ms,
                },
            },
        },
    )
    write_json(
        aggregate_path,
        {
            "schema": "ddn.ci.aggregate_report.v1",
            "overall_ok": status == "pass",
            "age4": {
                "proof_artifact_ok": status == "pass",
                "proof_artifact_failed_criteria": [] if status == "pass" else ["proof_runtime_error_statehash_preserved"],
                "proof_artifact_failed_preview": age4_proof_failed_preview,
                "proof_artifact_summary_hash": age4_proof_summary_hash,
            },
            "age5": {
                "age5_close_digest_selftest_ok": age5_digest_selftest_ok,
                AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
                AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY: dict(AGE5_DIGEST_SELFTEST_DEFAULT_FIELD),
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
        index_path,
        {
            "schema": "ddn.ci.aggregate_gate.index.v1",
            "report_prefix": prefix,
            "reports": {
                "aggregate": str(aggregate_path),
                "summary": str(summary_path),
                "summary_line": str(summary_line_path),
                "ci_gate_result_line": str(ci_gate_result_line_path),
                "ci_gate_result_json": str(result_path),
                "ci_fail_brief_txt": str(brief_path),
                "ci_fail_triage_json": str(triage_path),
                "ci_profile_matrix_gate_selftest": str(profile_matrix_selftest_path),
                "age2_close": str(age2_close_report_path),
                "age3_close_status_json": str(age3_close_status_report_path),
                "age4_close": str(age4_close_report_path),
                "age5_close": str(age5_close_report_path),
                "seamgrim_5min_checklist": str(runtime5_checklist_path),
                **({"ci_sanity_gate": str(sanity_path)} if with_sanity else {}),
                **({"ci_sync_readiness": str(sync_readiness_path)} if with_sync_readiness else {}),
            },
            "steps": [
                {
                    "name": "age5_close_digest_selftest",
                    "returncode": 0,
                    "ok": True,
                },
                {
                    "name": "ci_profile_matrix_gate_selftest",
                    "returncode": 0,
                    "ok": True,
                },
                {
                    "name": "ci_pack_golden_overlay_compare_selftest",
                    "returncode": 1 if broken_summary_selftest_step_mismatch else 0,
                    "ok": False if broken_summary_selftest_step_mismatch else True,
                },
                {
                    "name": "ci_pack_golden_overlay_session_selftest",
                    "returncode": 0,
                    "ok": True,
                },
                {
                    "name": "ci_pack_golden_lang_consistency_selftest",
                    "returncode": 0,
                    "ok": True,
                },
                {
                    "name": "gaji_registry_strict_audit_check",
                    "returncode": 0,
                    "ok": True,
                },
                {
                    "name": "gaji_registry_defaults_check",
                    "returncode": 0,
                    "ok": True,
                },
                {
                    "name": "tensor_v0_cli_check",
                    "returncode": 0,
                    "ok": True,
                },
            ],
        },
    )


def main() -> int:
    with persistent_tmpdir(prefix="ci_emit_artifacts_selftest_") as tmp:
        report_dir = Path(tmp)

        def run_case_expect_fail(
            prefix: str,
            *,
            expected_code: str | None = None,
            **build_kwargs: object,
        ) -> str | None:
            case_with_brief = bool(build_kwargs.pop("with_brief", True))
            case_with_triage = bool(build_kwargs.pop("with_triage", True))
            build_case(
                report_dir,
                prefix,
                with_brief=case_with_brief,
                with_triage=case_with_triage,
                **build_kwargs,
            )
            proc = run_check(
                report_dir,
                "--prefix",
                prefix,
                "--require-brief",
                "--require-triage",
            )
            if proc.returncode == 0:
                return f"{prefix} must fail"
            if expected_code is not None and f"fail code={expected_code}" not in proc.stderr:
                return f"{prefix} error code missing: err={proc.stderr}"
            return None

        def run_case_expect_pass(
            prefix: str,
            **build_kwargs: object,
        ) -> str | None:
            case_with_brief = bool(build_kwargs.pop("with_brief", True))
            case_with_triage = bool(build_kwargs.pop("with_triage", True))
            build_case(
                report_dir,
                prefix,
                with_brief=case_with_brief,
                with_triage=case_with_triage,
                **build_kwargs,
            )
            proc = run_check(
                report_dir,
                "--prefix",
                prefix,
                "--require-brief",
                "--require-triage",
            )
            if proc.returncode != 0:
                return f"{prefix} failed rc={proc.returncode} out={proc.stdout} err={proc.stderr}"
            return None

        def run_check_expect_fail(
            target_report_dir: Path,
            prefix: str,
            *,
            expected_code: str | None = None,
            expected_substring: str | None = None,
            extra_args: tuple[str, ...] = (),
        ) -> str | None:
            proc = run_check(target_report_dir, "--prefix", prefix, *extra_args)
            if proc.returncode == 0:
                return f"{prefix} must fail"
            if expected_code is not None and f"fail code={expected_code}" not in proc.stderr:
                return f"{prefix} error code missing: err={proc.stderr}"
            if expected_substring is not None and expected_substring not in proc.stderr:
                return f"{prefix} expected message missing: needle={expected_substring} err={proc.stderr}"
            return None

        def run_check_expect_pass(
            target_report_dir: Path,
            prefix: str,
            *,
            extra_args: tuple[str, ...] = (),
        ) -> str | None:
            proc = run_check(target_report_dir, "--prefix", prefix, *extra_args)
            if proc.returncode != 0:
                return f"{prefix} failed rc={proc.returncode} out={proc.stdout} err={proc.stderr}"
            return None

        def run_mutated_case_expect_pass(
            prefix: str,
            *,
            mutate=None,
            **build_kwargs: object,
        ) -> str | None:
            case_with_brief = bool(build_kwargs.pop("with_brief", True))
            case_with_triage = bool(build_kwargs.pop("with_triage", True))
            build_case(
                report_dir,
                prefix,
                with_brief=case_with_brief,
                with_triage=case_with_triage,
                **build_kwargs,
            )
            if mutate is not None:
                mutate(prefix)
            proc = run_check(
                report_dir,
                "--prefix",
                prefix,
                "--require-brief",
                "--require-triage",
            )
            if proc.returncode != 0:
                return f"{prefix} failed rc={proc.returncode} out={proc.stdout} err={proc.stderr}"
            return None

        def run_mutated_case_expect_fail(
            prefix: str,
            *,
            expected_code: str | None = None,
            mutate=None,
            **build_kwargs: object,
        ) -> str | None:
            case_with_brief = bool(build_kwargs.pop("with_brief", True))
            case_with_triage = bool(build_kwargs.pop("with_triage", True))
            build_case(
                report_dir,
                prefix,
                with_brief=case_with_brief,
                with_triage=case_with_triage,
                **build_kwargs,
            )
            if mutate is not None:
                mutate(prefix)
            return run_check_expect_fail(
                report_dir,
                prefix,
                expected_code=expected_code,
                extra_args=("--require-brief", "--require-triage"),
            )

        def run_mutated_case_fail_matrix(
            cases: tuple[MutatedFailCase, ...],
            **build_kwargs: object,
        ) -> str | None:
            for case_name, expected_code, mutate in cases:
                error = run_mutated_case_expect_fail(
                    case_name,
                    expected_code=expected_code,
                    mutate=mutate,
                    **build_kwargs,
                )
                if error:
                    return error
            return None

        def run_case_pass_matrix(
            cases: tuple[PassCase, ...],
        ) -> str | None:
            for case_name, case_kwargs in cases:
                error = run_case_expect_pass(case_name, **case_kwargs)
                if error:
                    return error
            return None

        def run_case_fail_matrix(
            cases: tuple[FailCase, ...],
        ) -> str | None:
            for case_name, expected_code, case_kwargs in cases:
                error = run_case_expect_fail(
                    case_name,
                    expected_code=expected_code,
                    **case_kwargs,
                )
                if error:
                    return error
            return None

        def kw_pass(**kwargs: object) -> dict[str, object]:
            return {"status": "pass", **kwargs}

        def kw_fail(**kwargs: object) -> dict[str, object]:
            return {"status": "fail", **kwargs}

        def pass_case(name: str, **kwargs: object) -> tuple[str, dict[str, object]]:
            return (name, kw_pass(**kwargs))

        def fail_case(
            name: str,
            expected_code: str | None = None,
            **kwargs: object,
        ) -> tuple[str, str | None, dict[str, object]]:
            case_kwargs = dict(kwargs)
            if "status" not in case_kwargs:
                case_kwargs = kw_pass(**case_kwargs)
            return (name, expected_code, case_kwargs)

        def flag_fail_cases(
            rows: tuple[FlagFailRow, ...],
            *,
            status: str = "pass",
            **shared_kwargs: object,
        ) -> tuple[FailCase, ...]:
            cases: list[FailCase] = []
            for case_name, expected_code, flag_name in rows:
                kwargs: dict[str, object] = {flag_name: True, **shared_kwargs}
                if status != "pass":
                    kwargs["status"] = status
                cases.append(fail_case(case_name, expected_code, **kwargs))
            return tuple(cases)

        def run_fail_flag_rows(
            rows: tuple[FlagFailRow, ...],
            *,
            status: str = "pass",
            **shared_kwargs: object,
        ) -> str | None:
            return run_case_fail_matrix(
                flag_fail_cases(rows, status=status, **shared_kwargs)
            )

        def rows_with_code(
            rows: tuple[tuple[str, str], ...],
            expected_code: str | None,
        ) -> tuple[FlagFailRow, ...]:
            return tuple((case_name, expected_code, flag_name) for case_name, flag_name in rows)

        def write_index_report(prefix: str, reports: dict[str, str]) -> None:
            write_json(
                report_dir / f"{prefix}.ci_gate_report_index.detjson",
                {
                    "schema": "ddn.ci.aggregate_gate.index.v1",
                    "report_prefix": prefix,
                    "reports": reports,
                },
            )

        def write_result_and_index(prefix: str, result_payload: dict[str, object]) -> Path:
            result_path = report_dir / f"{prefix}.ci_gate_result.detjson"
            write_json(result_path, result_payload)
            write_index_report(
                prefix,
                {
                    "ci_gate_result_json": str(result_path),
                    "summary": str(report_dir / f"{prefix}.ci_gate_summary.txt"),
                },
            )
            return result_path

        missing_report_dir = report_dir / "_missing_report_dir_"
        error = run_check_expect_fail(
            missing_report_dir,
            "missingreportdir",
            expected_code=CODES["REPORT_DIR_MISSING"],
        )
        if error:
            return fail(error)

        error = run_check_expect_fail(
            report_dir,
            "missingindex",
            expected_code=CODES["INDEX_NOT_FOUND"],
        )
        if error:
            return fail(error)

        bad_index_reports_missing = report_dir / "badindexreportsmissing.ci_gate_report_index.detjson"
        write_json(
            bad_index_reports_missing,
            {
                "schema": "ddn.ci.aggregate_gate.index.v1",
                "report_prefix": "badindexreportsmissing",
            },
        )
        error = run_check_expect_fail(
            report_dir,
            "badindexreportsmissing",
            expected_code=CODES["INDEX_REPORTS_MISSING"],
        )
        if error:
            return fail(error)

        bad_index_result_path_missing = report_dir / "badindexresultpathmissing.ci_gate_report_index.detjson"
        write_json(
            bad_index_result_path_missing,
            {
                "schema": "ddn.ci.aggregate_gate.index.v1",
                "report_prefix": "badindexresultpathmissing",
                "reports": {},
            },
        )
        error = run_check_expect_fail(
            report_dir,
            "badindexresultpathmissing",
            expected_code=CODES["INDEX_RESULT_PATH_MISSING"],
        )
        if error:
            return fail(error)

        bad_result_json_invalid_path = report_dir / "badresultjsoninvalid.ci_gate_result.detjson"
        write_text(bad_result_json_invalid_path, "{not-json}")
        write_index_report(
            "badresultjsoninvalid",
            {
                "ci_gate_result_json": str(bad_result_json_invalid_path),
                "summary": str(report_dir / "badresultjsoninvalid.ci_gate_summary.txt"),
            },
        )
        error = run_check_expect_fail(
            report_dir,
            "badresultjsoninvalid",
            expected_code=CODES["RESULT_JSON_INVALID"],
        )
        if error:
            return fail(error)

        result_contract_fail_cases: tuple[ResultContractFailCase, ...] = (
            (
                "badresultschemamismatch",
                {"schema": "broken.schema", "status": "pass", "failed_steps": 0},
                CODES["RESULT_SCHEMA_MISMATCH"],
            ),
            (
                "badresultfailedstepstype",
                {"schema": "ddn.ci.gate_result.v1", "status": "pass", "failed_steps": "bad"},
                CODES["RESULT_FAILED_STEPS_TYPE"],
            ),
            (
                "badresultfailedstepsnegative",
                {"schema": "ddn.ci.gate_result.v1", "status": "pass", "failed_steps": -1},
                CODES["RESULT_FAILED_STEPS_NEGATIVE"],
            ),
            (
                "badresultstatusunsupported",
                {"schema": "ddn.ci.gate_result.v1", "status": "weird", "failed_steps": 0},
                CODES["RESULT_STATUS_UNSUPPORTED"],
            ),
            (
                "badresultpassfailedsteps",
                {"schema": "ddn.ci.gate_result.v1", "status": "pass", "failed_steps": 1},
                CODES["RESULT_PASS_FAILED_STEPS"],
            ),
            (
                "badresultfailfailedsteps",
                {"schema": "ddn.ci.gate_result.v1", "status": "fail", "failed_steps": 0},
                CODES["RESULT_FAIL_FAILED_STEPS"],
            ),
        )
        for case_name, result_payload, expected_code in result_contract_fail_cases:
            write_result_and_index(case_name, result_payload)
            error = run_check_expect_fail(
                report_dir,
                case_name,
                expected_code=expected_code,
            )
            if error:
                return fail(error)

        bad_index_summary_path_missing_result = write_result_and_index(
            "badindexsummarypathmissing",
            {"schema": "ddn.ci.gate_result.v1", "status": "fail", "failed_steps": 1},
        )
        write_index_report(
            "badindexsummarypathmissing",
            {"ci_gate_result_json": str(bad_index_summary_path_missing_result)},
        )
        error = run_check_expect_fail(
            report_dir,
            "badindexsummarypathmissing",
            expected_code=CODES["INDEX_SUMMARY_PATH_MISSING"],
        )
        if error:
            return fail(error)

        bad_index_report_key_missing_result = write_result_and_index(
            "badindexreportkeymissing",
            {"schema": "ddn.ci.gate_result.v1", "status": "fail", "failed_steps": 1},
        )
        write_index_report(
            "badindexreportkeymissing",
            {
                "ci_gate_result_json": str(bad_index_report_key_missing_result),
                "summary": str(report_dir / "badindexreportkeymissing.ci_gate_summary.txt"),
            },
        )
        error = run_check_expect_fail(
            report_dir,
            "badindexreportkeymissing",
            expected_code=CODES["INDEX_REPORT_KEY_MISSING"],
        )
        if error:
            return fail(error)

        error = run_case_expect_pass("okcase", status="pass")
        if error:
            return fail(error)

        def mutate_summary_status(prefix: str) -> None:
            path = report_dir / f"{prefix}.ci_gate_summary.txt"
            text = path.read_text(encoding="utf-8")
            path.write_text(text.replace("[ci-gate-summary] PASS", "[ci-gate-summary] FAIL", 1), encoding="utf-8")

        def mutate_sanity_json_invalid(prefix: str) -> None:
            write_text(report_dir / f"{prefix}.ci_sanity_gate.detjson", "{broken-json}")

        def mutate_sanity_status_unsupported(prefix: str) -> None:
            path = report_dir / f"{prefix}.ci_sanity_gate.detjson"
            doc = json.loads(path.read_text(encoding="utf-8"))
            doc["status"] = "unknown"
            write_json(path, doc)

        def mutate_sanity_steps_type(prefix: str) -> None:
            path = report_dir / f"{prefix}.ci_sanity_gate.detjson"
            doc = json.loads(path.read_text(encoding="utf-8"))
            doc["steps"] = "bad"
            write_json(path, doc)

        def mutate_sanity_pass_failed_steps(prefix: str) -> None:
            path = report_dir / f"{prefix}.ci_sanity_gate.detjson"
            doc = json.loads(path.read_text(encoding="utf-8"))
            if isinstance(doc.get("steps"), list) and doc["steps"]:
                doc["steps"][0]["ok"] = False
            write_json(path, doc)

        def mutate_sanity_fail_failed_steps(prefix: str) -> None:
            path = report_dir / f"{prefix}.ci_sanity_gate.detjson"
            doc = json.loads(path.read_text(encoding="utf-8"))
            doc["status"] = "fail"
            doc["code"] = "E_CI_SANITY_SAMPLE_FAIL"
            doc["step"] = "pipeline_emit_flags_check"
            if isinstance(doc.get("steps"), list):
                for row in doc["steps"]:
                    if isinstance(row, dict):
                        row["ok"] = True
                        row["returncode"] = 0
            write_json(path, doc)

        def mutate_sync_readiness_json_invalid(prefix: str) -> None:
            write_text(report_dir / f"{prefix}.ci_sync_readiness.detjson", "{broken-json}")

        def mutate_index_brief_path_missing(prefix: str) -> None:
            path = report_dir / f"{prefix}.ci_gate_report_index.detjson"
            doc = json.loads(path.read_text(encoding="utf-8"))
            doc["reports"]["ci_fail_brief_txt"] = "-"
            write_json(path, doc)

        def mutate_index_triage_path_missing(prefix: str) -> None:
            path = report_dir / f"{prefix}.ci_gate_report_index.detjson"
            doc = json.loads(path.read_text(encoding="utf-8"))
            doc["reports"]["ci_fail_triage_json"] = "-"
            write_json(path, doc)

        mutated_fail_cases: tuple[MutatedFailCase, ...] = (
            ("badsummarystatusmismatch", CODES["SUMMARY_STATUS_MISMATCH"], mutate_summary_status),
            ("badsanityjsoninvalid", CODES["SANITY_JSON_INVALID"], mutate_sanity_json_invalid),
            ("badsanitystatusunsupported", CODES["SANITY_STATUS_UNSUPPORTED"], mutate_sanity_status_unsupported),
            ("badsanitysteptype", CODES["SANITY_STEPS_TYPE"], mutate_sanity_steps_type),
            ("badsanitypassfailedsteps", CODES["SANITY_PASS_FAILED_STEPS"], mutate_sanity_pass_failed_steps),
            ("badsanityfailfailedsteps", CODES["SANITY_FAIL_FAILED_STEPS"], mutate_sanity_fail_failed_steps),
            ("badsyncreadinessjsoninvalid", CODES["SYNC_READINESS_JSON_INVALID"], mutate_sync_readiness_json_invalid),
            ("badindexbriefpathmissing", CODES["INDEX_BRIEF_PATH_MISSING"], mutate_index_brief_path_missing),
            ("badindextriagepathmissing", CODES["INDEX_TRIAGE_PATH_MISSING"], mutate_index_triage_path_missing),
        )
        error = run_mutated_case_fail_matrix(mutated_fail_cases, status="pass")
        if error:
            return fail(error)

        ok_case_matrix: tuple[PassCase, ...] = (
            pass_case("okageclosepreview", omit_age_close_status_summary_keys=("age2_status", "age3_status", "age4_status", "age5_status")),
            pass_case("okruntime5off", with_runtime5_checklist=False),
            pass_case("okseamgrim", sanity_profile="seamgrim"),
        )
        error = run_case_pass_matrix(ok_case_matrix)
        if error:
            return fail(error)

        def mutate_summary_line_fallback(prefix: str) -> None:
            write_text(
                report_dir / f"{prefix}.ci_gate_summary_line.txt",
                "ci_gate_status=pass overall_ok=1 failed_steps=0 aggregate_status=pass reason=-",
            )

        error = run_mutated_case_expect_pass(
            "okfinallinefallback",
            status="pass",
            mutate=mutate_summary_line_fallback,
        )
        if error:
            return fail(error)

        ageclose_and_seamgrim_fail_cases: tuple[StrictFailCase, ...] = (
            fail_case("badageclosepartial", CODES["SUMMARY_SELFTEST_KEY_MISSING"], omit_age_close_status_summary_keys=("age2_status",)),
            fail_case("badsanityseamgrimpackschema", CODES["SANITY_REQUIRED_STEP_FAILED"], sanity_profile="seamgrim", broken_sanity_seamgrim_pack_schema=True),
            fail_case("badsanityseamgrimpackrepo", CODES["SANITY_REQUIRED_STEP_FAILED"], sanity_profile="seamgrim", broken_sanity_seamgrim_pack_repo_issue_count=True),
        )
        error = run_case_fail_matrix(ageclose_and_seamgrim_fail_cases)
        if error:
            return fail(error)

        seamgrim_flag_rows: tuple[FlagFailRow, ...] = (
            rows_with_code(
                (
                    ("badsanityseamgrimpackdocsmismatch", "broken_sanity_seamgrim_pack_docs_issue_count_summary_mismatch"),
                    ("badsanityseamgrimwasmcheckedmismatch", "broken_sanity_seamgrim_wasm_checked_files_summary_mismatch"),
                ),
                CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
            )
            + rows_with_code(
                (
                    ("badsyncseamgrimpackdocsmismatch", "broken_sync_readiness_seamgrim_pack_docs_issue_count_summary_mismatch"),
                    ("badsyncseamgrimwasmcheckedmismatch", "broken_sync_readiness_seamgrim_wasm_checked_files_summary_mismatch"),
                    ("badsyncseamgrimpackschema", "broken_sync_readiness_seamgrim_pack_schema"),
                    ("badsyncseamgrimpackdocs", "broken_sync_readiness_seamgrim_pack_docs_issue_count"),
                    ("badsyncseamgrimwasmchecked", "broken_sync_readiness_seamgrim_wasm_checked_files"),
                ),
                CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
            )
            + rows_with_code(
                (
                    ("badsanityseamgrimpackdocs", "broken_sanity_seamgrim_pack_docs_issue_count"),
                    ("badsanityseamgrimwasmchecked", "broken_sanity_seamgrim_wasm_checked_files"),
                ),
                CODES["SANITY_REQUIRED_STEP_FAILED"],
            )
        )
        full_profile_flag_rows: tuple[FlagFailRow, ...] = (
            rows_with_code(
                (
                    ("badsanityfullseamgrimschema", "broken_sanity_seamgrim_pack_schema_non_seamgrim"),
                    ("badsanityfullseamgrimdocs", "broken_sanity_seamgrim_pack_docs_issue_count_non_seamgrim"),
                    ("badsanityfullseamgrimwasmchecked", "broken_sanity_seamgrim_wasm_checked_files_non_seamgrim"),
                ),
                CODES["SANITY_REQUIRED_STEP_FAILED"],
            )
            + rows_with_code(
                (
                    ("badsyncfullseamgrimschema", "broken_sync_readiness_seamgrim_pack_schema_non_seamgrim"),
                    ("badsyncfullseamgrimdocs", "broken_sync_readiness_seamgrim_pack_docs_issue_count_non_seamgrim"),
                    ("badsyncfullseamgrimwasmchecked", "broken_sync_readiness_seamgrim_wasm_checked_files_non_seamgrim"),
                ),
                CODES["SYNC_READINESS_PASS_STATUS_FIELDS"],
            )
        )
        seamgrim_profile_fail_cases: tuple[FailCase, ...] = (
            flag_fail_cases(seamgrim_flag_rows, sanity_profile="seamgrim")
            + flag_fail_cases(full_profile_flag_rows, sanity_profile="full")
        )
        error = run_case_fail_matrix(seamgrim_profile_fail_cases)
        if error:
            return fail(error)

        build_case(
            report_dir,
            "existupgrade",
            status="pass",
            with_brief=True,
            with_triage=True,
            force_artifact_exists_false=("summary_line",),
        )
        error = run_check_expect_fail(
            report_dir,
            "existupgrade",
            expected_substring="exists mismatch",
            extra_args=("--require-brief", "--require-triage"),
        )
        if error:
            return fail(error)
        error = run_check_expect_pass(
            report_dir,
            "existupgrade",
            extra_args=("--require-brief", "--require-triage", "--allow-triage-exists-upgrade"),
        )
        if error:
            return fail(error)

        fail_status_pass_cases: tuple[PassCase, ...] = (
            pass_case("failcase", status="fail"),
            pass_case("failcasenamefallback", status="fail", triage_fail_step_use_name_field=True),
        )
        error = run_case_pass_matrix(fail_status_pass_cases)
        if error:
            return fail(error)

        summary_runtime_profile_flag_rows: tuple[FlagFailRow, ...] = (
            rows_with_code(
                (
                    ("missselftestkey", "broken_summary_selftest_missing"),
                    ("missruntime5key", "broken_runtime5_summary_missing"),
                    ("missprofilematrixkey", "broken_profile_matrix_summary_missing"),
                ),
                CODES["SUMMARY_SELFTEST_KEY_MISSING"],
            )
            + rows_with_code(
                (
                    ("badruntime5value", "broken_runtime5_summary_value"),
                    ("badprofilematrixvalue", "broken_profile_matrix_summary_value"),
                ),
                CODES["SUMMARY_SELFTEST_VALUE_INVALID"],
            )
            + rows_with_code(
                (
                    ("profilematrixmismatch", "broken_profile_matrix_report_mismatch"),
                    ("runtime5mismatch", "broken_runtime5_report_mismatch"),
                    ("badselfteststep", "broken_summary_selftest_step_mismatch"),
                ),
                CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
            )
            + rows_with_code(
                (("badselftestvalue", "broken_summary_selftest_value"),),
                CODES["SUMMARY_SELFTEST_EXPECT_PASS"],
            )
        )
        error = run_fail_flag_rows(summary_runtime_profile_flag_rows)
        if error:
            return fail(error)

        profile_matrix_fail_cases: tuple[FailCase, ...] = (
            fail_case("missprofilematrixbrief", None, broken_profile_matrix_brief_missing=True),
            fail_case("badprofilematrixbrief", None, broken_profile_matrix_brief_value=True),
            fail_case("badprofilematrixtriage", None, broken_profile_matrix_triage_mismatch=True),
        )
        error = run_case_fail_matrix(profile_matrix_fail_cases)
        if error:
            return fail(error)

        sync_sanity_path_cases: tuple[StrictFailCase, ...] = (
            fail_case("missbrief", CODES["BRIEF_REQUIRED_MISSING"], with_brief=False, with_triage=True),
            fail_case("misstriage", CODES["TRIAGE_REQUIRED_MISSING"], with_brief=True, with_triage=False),
            fail_case("misssanity", CODES["SANITY_PATH_MISSING"], with_sanity=False),
            fail_case("misssync", CODES["SYNC_READINESS_PATH_MISSING"], with_sync_readiness=False),
        ) + flag_fail_cases(
            rows_with_code((("badsyncschema", "broken_sync_readiness_schema"),), CODES["SYNC_READINESS_SCHEMA_MISMATCH"])
            + rows_with_code((("badsyncstatus", "broken_sync_readiness_status_unsupported"),), CODES["SYNC_READINESS_STATUS_UNSUPPORTED"])
            + rows_with_code((("badsyncmismatch", "broken_sync_readiness_status_mismatch"),), CODES["SYNC_READINESS_STATUS_MISMATCH"])
            + rows_with_code((("badsyncpassfields", "broken_sync_readiness_pass_fields"),), CODES["SYNC_READINESS_PASS_STATUS_FIELDS"])
            + rows_with_code((("badsanityschema", "broken_sanity_schema"),), CODES["SANITY_SCHEMA_MISMATCH"])
            + rows_with_code((("badsanitystatus", "broken_sanity_status"),), CODES["SANITY_STATUS_MISMATCH"])
        )
        error = run_case_fail_matrix(sync_sanity_path_cases)
        if error:
            return fail(error)

        bad_sanity_required_step_flag_rows: tuple[FlagFailRow, ...] = (
            rows_with_code(
                (
                    ("badsanitystepmissing", "broken_sanity_required_step_missing"),
                    ("badsanityproductblockermissing", "broken_sanity_product_blocker_step_missing"),
                    ("badsanityobserveoutputmissing", "broken_sanity_observe_output_contract_step_missing"),
                    ("badsanityviewsourcestrictmissing", "broken_sanity_runtime_view_source_strict_step_missing"),
                    ("badsanitylegacyautofixmissing", "broken_sanity_run_legacy_autofix_step_missing"),
                    ("badsanitywiredmissing", "broken_sanity_wired_step_missing"),
                    ("badsanitycomparemissing", "broken_sanity_compare_step_missing"),
                    ("badsanitywasmwebselftestmissing", "broken_sanity_wasm_web_selftest_step_missing"),
                ),
                CODES["SANITY_REQUIRED_STEP_MISSING"],
            )
            + rows_with_code(
                (
                    ("badsanitystepfailed", "broken_sanity_required_step_failed"),
                    ("badsanityproductblockerfailed", "broken_sanity_product_blocker_step_failed"),
                    ("badsanityobserveoutputfailed", "broken_sanity_observe_output_contract_step_failed"),
                    ("badsanityviewsourcestrictfailed", "broken_sanity_runtime_view_source_strict_step_failed"),
                    ("badsanitylegacyautofixfailed", "broken_sanity_run_legacy_autofix_step_failed"),
                    ("badsanitywiredfailed", "broken_sanity_wired_step_failed"),
                    ("badsanitycomparefailed", "broken_sanity_compare_step_failed"),
                    ("badsanitywasmwebselftestfailed", "broken_sanity_wasm_web_selftest_step_failed"),
                ),
                CODES["SANITY_REQUIRED_STEP_FAILED"],
            )
        )
        error = run_fail_flag_rows(bad_sanity_required_step_flag_rows)
        if error:
            return fail(error)

        basic_fail_flag_rows: tuple[FlagFailRow, ...] = (
            ("badnorm", None, "broken_norm"),
            ("badbrief", None, "broken_brief"),
            ("badtriagefinal", None, "broken_triage_final"),
        )
        error = run_fail_flag_rows(basic_fail_flag_rows)
        if error:
            return fail(error)

        artifact_summary_fail_cases: tuple[FailCase, ...] = (
            fail_case("badartifactref", None, broken_artifact_ref=True),
            fail_case("badsummary", CODES["SUMMARY_STATUS_MISMATCH"], status="fail", broken_summary=True),
        )
        error = run_case_fail_matrix(artifact_summary_fail_cases)
        if error:
            return fail(error)

        triage_fail_flag_rows: tuple[FlagFailRow, ...] = (
            rows_with_code(
                (
                    ("badtriagedetailrowscount", "broken_triage_failed_step_detail_rows_count_mismatch"),
                    ("badtriagelogsrowscount", "broken_triage_failed_step_logs_rows_count_mismatch"),
                    ("badtriagedetailorder", "broken_triage_failed_step_detail_order_mismatch"),
                    ("badtriagelogsorder", "broken_triage_failed_step_logs_order_mismatch"),
                ),
                CODES["SUMMARY_SELFTEST_STEP_MISMATCH"],
            )
            + rows_with_code(
                (
                    ("badtriagedetailrowscounttype", "broken_triage_failed_step_detail_rows_count_type"),
                    ("badtriagelogsrowscounttype", "broken_triage_failed_step_logs_rows_count_type"),
                    ("badtriagedetailordertype", "broken_triage_failed_step_detail_order_type"),
                    ("badtriagelogsordertype", "broken_triage_failed_step_logs_order_type"),
                ),
                CODES["TRIAGE_REQUIRED_MISSING"],
            )
        )
        error = run_fail_flag_rows(triage_fail_flag_rows, status="fail")
        if error:
            return fail(error)

        age5_fail_flag_rows: tuple[FlagFailRow, ...] = (
            ("badverifyissue", None, "broken_verify_issue"),
            ("badage5childsummary", None, "broken_age5_child_summary_triage_mismatch"),
            ("badage5childsummarydefault", None, "broken_age5_child_summary_default_triage_mismatch"),
            ("badage5childsummarysummary", None, "broken_age5_child_summary_summary_mismatch"),
        )
        error = run_fail_flag_rows(age5_fail_flag_rows)
        if error:
            return fail(error)

    print("[ci-emit-artifacts-check-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
