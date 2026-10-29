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
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY,
    build_age5_combined_heavy_policy_origin_trace,
    build_age5_combined_heavy_policy_origin_trace_text,
    build_age5_combined_heavy_sanity_contract_fields,
    build_age5_combined_heavy_sync_contract_fields,
)
from _ci_profile_matrix_selftest_lib import (
    PROFILE_MATRIX_SELFTEST_PROFILES,
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC,
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
    PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS,
    expected_profile_matrix_aggregate_summary_contract,
)
from ci_check_error_codes import SUMMARY_REPORT_CODES as CODES

_MODULE_CACHE: dict[str, object] = {}
_ENSURED_PARENT_DIRS: set[Path] = set()


def fail(msg: str) -> int:
    print(f"[ci-gate-summary-report-selftest] fail: {msg}")
    return 1


def ensure_parent_dir(path: Path) -> None:
    parent = path.parent
    if parent in _ENSURED_PARENT_DIRS:
        return
    parent.mkdir(parents=True, exist_ok=True)
    _ENSURED_PARENT_DIRS.add(parent)


def write_text(path: Path, text: str) -> None:
    ensure_parent_dir(path)
    path.write_text(text.rstrip() + "\n", encoding="utf-8")


def write_json(path: Path, payload: dict) -> None:
    ensure_parent_dir(path)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


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
    if script_norm.endswith("tests/run_ci_gate_summary_report_check.py"):
        return _run_module_main("run_ci_gate_summary_report_check", cmd, argv)
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


def run_check(summary: Path, index: Path, require_pass: bool = False) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_ci_gate_summary_report_check.py",
        "--summary",
        str(summary),
        "--index",
        str(index),
    ]
    if require_pass:
        cmd.append("--require-pass")
    return run_cmd_inprocess(cmd)


def build_pass_case(root: Path, name: str) -> tuple[Path, Path]:
    case_dir = root / name
    case_dir.mkdir(parents=True, exist_ok=True)
    summary_path = case_dir / "ci_gate_summary.txt"
    index_path = case_dir / "ci_gate_report_index.detjson"
    summary_line = case_dir / "ci_gate_summary_line.txt"
    result = case_dir / "ci_gate_result.detjson"
    badge = case_dir / "ci_gate_badge.detjson"
    brief = case_dir / "ci_fail_brief.txt"
    triage = case_dir / "ci_fail_triage.detjson"
    age2_status = case_dir / "age2_close_report.detjson"
    age3_status = case_dir / "age3_close_status.detjson"
    age3_bogae_geoul_visibility_smoke_report = case_dir / "age3_bogae_geoul_visibility_smoke.detjson"
    age4_status = case_dir / "age4_close_report.detjson"
    age5_status = case_dir / "age5_close_report.detjson"
    phase3_cleanup = case_dir / "seamgrim_phase3_cleanup_gate_report.detjson"
    seamgrim_wasm_cli_diag_parity_report = case_dir / "seamgrim_wasm_cli_diag_parity_report.detjson"
    seamgrim_5min_checklist = case_dir / "seamgrim_5min_checklist_report.detjson"
    ci_profile_matrix_gate_selftest_report = case_dir / "ci_profile_matrix_gate_selftest.detjson"
    ci_sanity_gate = case_dir / "ci_sanity_gate.detjson"
    ci_sync_readiness = case_dir / "ci_sync_readiness.detjson"
    fixed64_threeway_report = case_dir / "fixed64_cross_platform_threeway_gate.detjson"
    aggregate_report = case_dir / "ci_aggregate_report.detjson"
    age4_proof_ok = "1"
    age4_proof_failed_criteria = "0"
    age4_proof_failed_preview = "-"
    age4_proof_summary_hash = "sha256:age4-proof-summary-selftest"
    for path in (
        summary_line,
        result,
        badge,
        brief,
        triage,
        age2_status,
        age3_status,
        age4_status,
        phase3_cleanup,
        seamgrim_wasm_cli_diag_parity_report,
        ci_sanity_gate,
        ci_sync_readiness,
        fixed64_threeway_report,
    ):
        write_text(path, "{}")
    write_json(
        age2_status,
        {
            "schema": "ddn.age2_close_report.v1",
            "overall_ok": True,
            "criteria": [],
            "failure_digest": [],
            "failure_codes": [],
        },
    )
    write_json(
        age3_bogae_geoul_visibility_smoke_report,
        {
            "schema": "ddn.bogae_geoul_visibility_smoke.v1",
            "overall_ok": True,
            "checks": [{"name": "artifact_presence", "ok": True}],
            "simulation_hash_delta": {
                "state_hash_changes": True,
                "bogae_hash_changes": True,
            },
        },
    )
    age5_child_summary_fields = {
        "age5_combined_heavy_full_real_status": "skipped",
        "age5_combined_heavy_runtime_helper_negative_status": "skipped",
        "age5_combined_heavy_group_id_summary_negative_status": "skipped",
        "age5_full_real_w107_golden_index_selftest_active_cases": "54",
        "age5_full_real_w107_golden_index_selftest_inactive_cases": "1",
        "age5_full_real_w107_golden_index_selftest_index_codes": "34",
        "age5_full_real_w107_golden_index_selftest_current_probe": "-",
        "age5_full_real_w107_golden_index_selftest_last_completed_probe": "validate_pack_pointers",
        "age5_full_real_w107_golden_index_selftest_progress_present": "1",
        "age5_full_real_w107_progress_contract_selftest_completed_checks": "8",
        "age5_full_real_w107_progress_contract_selftest_total_checks": "8",
        "age5_full_real_w107_progress_contract_selftest_checks_text": "golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index",
        "age5_full_real_w107_progress_contract_selftest_current_probe": "-",
        "age5_full_real_w107_progress_contract_selftest_last_completed_probe": "report_index",
        "age5_full_real_w107_progress_contract_selftest_progress_present": "1",
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks": "5",
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks": "5",
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text": "signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract",
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe": "-",
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe": "signed_contract",
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present": "1",
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks": "1",
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks": "1",
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text": "verify_report_digest_contract",
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe": "-",
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe": "readme_and_field_contract",
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present": "1",
        "age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks": "4",
        "age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks": "4",
        "age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text": "signed_contract,consumer_contract,promotion,family",
        "age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe": "-",
        "age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe": "family",
        "age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present": "1",
        "age5_full_real_proof_certificate_family_contract_selftest_completed_checks": "3",
        "age5_full_real_proof_certificate_family_contract_selftest_total_checks": "3",
        "age5_full_real_proof_certificate_family_contract_selftest_checks_text": "artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family",
        "age5_full_real_proof_certificate_family_contract_selftest_current_probe": "-",
        "age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe": "proof_certificate_family",
        "age5_full_real_proof_certificate_family_contract_selftest_progress_present": "1",
        "age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks": "9",
        "age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks": "9",
        "age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        "age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe": "-",
        "age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe": "report_index",
        "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present": "1",
        "age5_full_real_proof_family_contract_selftest_completed_checks": "3",
        "age5_full_real_proof_family_contract_selftest_total_checks": "3",
        "age5_full_real_proof_family_contract_selftest_checks_text": "proof_operation_family,proof_certificate_family,proof_family",
        "age5_full_real_proof_family_contract_selftest_current_probe": "-",
        "age5_full_real_proof_family_contract_selftest_last_completed_probe": "proof_family",
        "age5_full_real_proof_family_contract_selftest_progress_present": "1",
        "age5_full_real_proof_family_transport_contract_selftest_completed_checks": "9",
        "age5_full_real_proof_family_transport_contract_selftest_total_checks": "9",
        "age5_full_real_proof_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        "age5_full_real_proof_family_transport_contract_selftest_current_probe": "-",
        "age5_full_real_proof_family_transport_contract_selftest_last_completed_probe": "report_index",
        "age5_full_real_proof_family_transport_contract_selftest_progress_present": "1",
        "age5_full_real_lang_surface_family_contract_selftest_completed_checks": "4",
        "age5_full_real_lang_surface_family_contract_selftest_total_checks": "4",
        "age5_full_real_lang_surface_family_contract_selftest_checks_text": "proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family",
        "age5_full_real_lang_surface_family_contract_selftest_current_probe": "-",
        "age5_full_real_lang_surface_family_contract_selftest_last_completed_probe": "lang_surface_family",
        "age5_full_real_lang_surface_family_contract_selftest_progress_present": "1",
        "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks": "9",
        "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks": "9",
        "age5_full_real_lang_surface_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        "age5_full_real_lang_surface_family_transport_contract_selftest_current_probe": "-",
        "age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe": "report_index",
        "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present": "1",
        "age5_full_real_lang_runtime_family_contract_selftest_completed_checks": "5",
        "age5_full_real_lang_runtime_family_contract_selftest_total_checks": "5",
        "age5_full_real_lang_runtime_family_contract_selftest_checks_text": "lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family",
        "age5_full_real_lang_runtime_family_contract_selftest_current_probe": "-",
        "age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe": "lang_runtime_family",
        "age5_full_real_lang_runtime_family_contract_selftest_progress_present": "1",
        "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks": "9",
        "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks": "9",
        "age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        "age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe": "-",
        "age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe": "report_index",
        "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present": "1",
        "age5_full_real_gate0_family_contract_selftest_completed_checks": "5",
        "age5_full_real_gate0_family_contract_selftest_total_checks": "5",
        "age5_full_real_gate0_family_contract_selftest_checks_text": "gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family",
        "age5_full_real_gate0_family_contract_selftest_current_probe": "-",
        "age5_full_real_gate0_family_contract_selftest_last_completed_probe": "gate0_family",
        "age5_full_real_gate0_family_contract_selftest_progress_present": "1",
        "age5_full_real_gate0_surface_family_contract_selftest_completed_checks": "5",
        "age5_full_real_gate0_surface_family_contract_selftest_total_checks": "5",
        "age5_full_real_gate0_surface_family_contract_selftest_checks_text": "lang_surface_family,lang_runtime_family,gate0_runtime_family,gate0_family,gate0_transport_family",
        "age5_full_real_gate0_surface_family_contract_selftest_current_probe": "-",
        "age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe": "gate0_transport_family",
        "age5_full_real_gate0_surface_family_contract_selftest_progress_present": "1",
        "age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks": "9",
        "age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks": "9",
        "age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        "age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe": "-",
        "age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe": "report_index",
        "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present": "1",
        "age5_full_real_gate0_family_transport_contract_selftest_completed_checks": "9",
        "age5_full_real_gate0_family_transport_contract_selftest_total_checks": "9",
        "age5_full_real_gate0_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        "age5_full_real_gate0_family_transport_contract_selftest_current_probe": "-",
        "age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe": "report_index",
        "age5_full_real_gate0_family_transport_contract_selftest_progress_present": "1",
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks": "1",
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks": "1",
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text": "family_contract",
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe": "-",
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe": "family_contract",
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present": "1",
        "age5_full_real_gate0_transport_family_contract_selftest_completed_checks": "4",
        "age5_full_real_gate0_transport_family_contract_selftest_total_checks": "4",
        "age5_full_real_gate0_transport_family_contract_selftest_checks_text": "lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family",
        "age5_full_real_gate0_transport_family_contract_selftest_current_probe": "-",
        "age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe": "gate0_transport_family",
        "age5_full_real_gate0_transport_family_contract_selftest_progress_present": "1",
        "age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks": "9",
        "age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks": "9",
        "age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        "age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe": "-",
        "age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe": "report_index",
        "age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present": "1",
        "age5_full_real_bogae_alias_family_contract_selftest_completed_checks": "3",
        "age5_full_real_bogae_alias_family_contract_selftest_total_checks": "3",
        "age5_full_real_bogae_alias_family_contract_selftest_checks_text": "shape_alias_contract,alias_family,alias_viewer_family",
        "age5_full_real_bogae_alias_family_contract_selftest_current_probe": "-",
        "age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe": "alias_viewer_family",
        "age5_full_real_bogae_alias_family_contract_selftest_progress_present": "1",
        "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks": "9",
        "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks": "9",
        "age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        "age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe": "-",
        "age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe": "report_index",
        "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present": "1",
    }
    write_json(
        age5_status,
        {
            "schema": "ddn.age5.close_report.v1",
            **age5_child_summary_fields,
        },
    )
    age5_policy_summary_fields = {
        "age5_policy_combined_digest_selftest_default_field_text": "age5_close_digest_selftest_ok=0",
        "age5_policy_combined_digest_selftest_default_field": {"age5_close_digest_selftest_ok": "0"},
        "age5_combined_heavy_policy_report_path": "-",
        "age5_combined_heavy_policy_report_exists": False,
        "age5_combined_heavy_policy_text_path": "-",
        "age5_combined_heavy_policy_text_exists": False,
        "age5_combined_heavy_policy_summary_path": "-",
        "age5_combined_heavy_policy_summary_exists": False,
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY: "-",
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY: "-",
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY: "-",
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY: "-",
    }
    age5_policy_origin_trace = build_age5_combined_heavy_policy_origin_trace()
    age5_policy_summary_fields[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY] = age5_policy_origin_trace
    age5_policy_summary_fields[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY] = (
        build_age5_combined_heavy_policy_origin_trace_text(age5_policy_origin_trace)
    )
    write_json(
        aggregate_report,
        {
            "schema": "ddn.ci.aggregate_report.v1",
            "age4": {
                "proof_artifact_ok": True,
                "proof_artifact_failed_criteria": [],
                "proof_artifact_failed_preview": age4_proof_failed_preview,
                "proof_artifact_summary_hash": age4_proof_summary_hash,
            },
            "age5": {
                **age5_child_summary_fields,
                **age5_policy_summary_fields,
            },
        },
    )
    aggregate_summary_rows = {}
    for profile_name in PROFILE_MATRIX_SELFTEST_PROFILES:
        expected_contract = expected_profile_matrix_aggregate_summary_contract(profile_name)
        aggregate_summary_rows[profile_name] = {
            "expected_present": True,
            "ok": bool(expected_contract["ok"]),
            "status": str(expected_contract["status"]),
            "values": dict(expected_contract["values"]),
        }

    write_json(
        ci_profile_matrix_gate_selftest_report,
        {
            "schema": "ddn.ci.profile_matrix_gate_selftest.v1",
            "status": "pass",
            "ok": True,
            "selected_real_profiles": ["core_lang", "full", "seamgrim"],
            "skipped_real_profiles": [],
            "step_timeout_defaults_text": PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
            "step_timeout_defaults_sec": dict(PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC),
            "step_timeout_env_keys": dict(PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS),
            "total_elapsed_ms": 666,
            "aggregate_summary_sanity_ok": True,
            "aggregate_summary_sanity_checked_profiles": list(PROFILE_MATRIX_SELFTEST_PROFILES),
            "aggregate_summary_sanity_failed_profiles": [],
            "aggregate_summary_sanity_skipped_profiles": [],
            "aggregate_summary_sanity_by_profile": aggregate_summary_rows,
            "real_profiles": {
                "core_lang": {"selected": True, "skipped": False, "status": "pass", "ok": True, "total_elapsed_ms": 111},
                "full": {"selected": True, "skipped": False, "status": "pass", "ok": True, "total_elapsed_ms": 222},
                "seamgrim": {"selected": True, "skipped": False, "status": "pass", "ok": True, "total_elapsed_ms": 333},
            },
        },
    )
    sanity_summary = {
        "ci_sanity_pipeline_emit_flags_ok": "1",
        "ci_sanity_pipeline_emit_flags_selftest_ok": "1",
        "ci_sanity_emit_artifacts_sanity_contract_selftest_ok": "1",
        "ci_sanity_age2_completion_gate_ok": "1",
        "ci_sanity_age2_completion_gate_selftest_ok": "1",
        "ci_sanity_age3_completion_gate_ok": "1",
        "ci_sanity_age3_completion_gate_selftest_ok": "1",
        "ci_sanity_age2_completion_gate_failure_codes": "-",
        "ci_sanity_age2_completion_gate_failure_code_count": "0",
        "ci_sanity_age3_completion_gate_failure_codes": "-",
        "ci_sanity_age3_completion_gate_failure_code_count": "0",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_ok": "1",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path": str(age3_bogae_geoul_visibility_smoke_report),
        "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists": "1",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_schema": "ddn.bogae_geoul_visibility_smoke.v1",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok": "1",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok": "1",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes": "1",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes": "1",
        "ci_sanity_seamgrim_pack_evidence_tier_runner_ok": "na",
        "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path": "-",
        "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists": "na",
        "ci_sanity_seamgrim_pack_evidence_tier_runner_schema": "-",
        "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count": "-",
        "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count": "-",
        "ci_sanity_seamgrim_wasm_web_step_check_ok": "na",
        "ci_sanity_seamgrim_wasm_web_step_check_report_path": "-",
        "ci_sanity_seamgrim_wasm_web_step_check_report_exists": "na",
        "ci_sanity_seamgrim_wasm_web_step_check_schema": "-",
        "ci_sanity_seamgrim_wasm_web_step_check_checked_files": "-",
        "ci_sanity_seamgrim_wasm_web_step_check_missing_count": "-",
        "ci_sanity_age5_combined_heavy_policy_selftest_ok": "1",
        "ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok": "1",
        "ci_sanity_dynamic_source_profile_split_selftest_ok": "1",
        **build_age5_combined_heavy_sanity_contract_fields(),
    }
    sanity_summary.update({key: "1" for key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS})
    sync_smoke_summary = {
        "ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok": "1",
        "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok": "1",
        "ci_sync_readiness_ci_sanity_age2_completion_gate_failure_codes": "-",
        "ci_sync_readiness_ci_sanity_age2_completion_gate_failure_code_count": "0",
        "ci_sync_readiness_ci_sanity_age3_completion_gate_failure_codes": "-",
        "ci_sync_readiness_ci_sanity_age3_completion_gate_failure_code_count": "0",
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_ok": "1",
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_path": (
            str(age3_bogae_geoul_visibility_smoke_report)
        ),
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists": "1",
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_schema": (
            "ddn.bogae_geoul_visibility_smoke.v1"
        ),
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok": "1",
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok": "1",
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes": "1",
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes": "1",
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok": "na",
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_path": "-",
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists": "na",
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_schema": "-",
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count": "-",
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count": "-",
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_ok": "na",
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_path": "-",
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_exists": "na",
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_schema": "-",
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_checked_files": "-",
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_missing_count": "-",
    }
    sync_smoke_summary.update(
        {sync_key: "1" for _sanity_key, sync_key in AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS}
    )
    write_json(
        index_path,
        {
            "schema": "ddn.ci.aggregate_gate.index.v1",
            "reports": {
                "summary_line": str(summary_line),
                "ci_gate_result_json": str(result),
                "ci_gate_badge_json": str(badge),
                "ci_fail_triage_json": str(triage),
                "ci_profile_matrix_gate_selftest": str(ci_profile_matrix_gate_selftest_report),
                "age2_close": str(age2_status),
                "age3_close_status_json": str(age3_status),
                "age4_close": str(age4_status),
                "age5_close": str(age5_status),
                "seamgrim_phase3_cleanup": str(phase3_cleanup),
                "seamgrim_wasm_cli_diag_parity": str(seamgrim_wasm_cli_diag_parity_report),
                "seamgrim_5min_checklist": str(seamgrim_5min_checklist),
                "ci_sanity_gate": str(ci_sanity_gate),
                "ci_sync_readiness": str(ci_sync_readiness),
                "fixed64_threeway_gate": str(fixed64_threeway_report),
                "aggregate": str(aggregate_report),
            },
        },
    )
    write_json(
        ci_sanity_gate,
        {
            "schema": "ddn.ci.sanity_gate.v1",
            "status": "pass",
            "code": "OK",
            "step": "all",
            "profile": "full",
            "msg": "-",
            **sanity_summary,
            "steps": [],
        },
    )
    write_json(
        ci_sync_readiness,
        {
            "schema": "ddn.ci.sync_readiness.v1",
            "status": "pass",
            "ok": True,
            "code": "OK",
            "step": "all",
            "sanity_profile": "full",
            "msg": "-",
            **sanity_summary,
            **build_age5_combined_heavy_sync_contract_fields(),
            **sync_smoke_summary,
            "steps": [{"name": "sanity_gate_contract", "ok": True, "returncode": 0}],
            "steps_count": 1,
        },
    )
    write_json(
        seamgrim_5min_checklist,
        {
            "schema": "seamgrim.runtime_5min_checklist.v1",
            "ok": True,
            "items": [
                {"name": "rewrite_motion_projectile_fallback", "ok": True, "elapsed_ms": 321},
                {"name": "moyang_view_boundary_pack_check", "ok": True, "elapsed_ms": 654},
                {"name": "pendulum_tetris_showcase_check", "ok": True, "elapsed_ms": 777},
            ],
        },
    )
    lines = [
        "[ci-gate-summary] PASS",
        "[ci-gate-summary] failed_steps=(none)",
        f"[ci-gate-summary] report_index={index_path}",
        f"[ci-gate-summary] summary_line={summary_line}",
        f"[ci-gate-summary] ci_gate_result={result}",
        f"[ci-gate-summary] ci_gate_badge={badge}",
        f"[ci-gate-summary] ci_fail_brief_hint={brief}",
        "[ci-gate-summary] ci_fail_brief_exists=1",
        f"[ci-gate-summary] ci_fail_triage_hint={triage}",
        "[ci-gate-summary] ci_fail_triage_exists=1",
        f"[ci-gate-summary] ci_profile_matrix_gate_selftest_report={ci_profile_matrix_gate_selftest_report}",
        "[ci-gate-summary] ci_profile_matrix_gate_selftest_status=pass",
        "[ci-gate-summary] ci_profile_matrix_gate_selftest_ok=1",
        "[ci-gate-summary] ci_profile_matrix_gate_selftest_total_elapsed_ms=666",
        "[ci-gate-summary] ci_profile_matrix_gate_selftest_selected_real_profiles=core_lang,full,seamgrim",
        "[ci-gate-summary] ci_profile_matrix_gate_selftest_skipped_real_profiles=-",
        "[ci-gate-summary] ci_profile_matrix_gate_selftest_step_timeout_defaults=core_lang:900,full:1200,seamgrim:1500",
        "[ci-gate-summary] ci_profile_matrix_gate_selftest_core_lang_elapsed_ms=111",
        "[ci-gate-summary] ci_profile_matrix_gate_selftest_full_elapsed_ms=222",
        "[ci-gate-summary] ci_profile_matrix_gate_selftest_seamgrim_elapsed_ms=333",
        "[ci-gate-summary] ci_profile_matrix_gate_selftest_aggregate_summary_sanity_ok=1",
        "[ci-gate-summary] ci_profile_matrix_gate_selftest_aggregate_summary_checked_profiles=core_lang,full,seamgrim",
        "[ci-gate-summary] ci_profile_matrix_gate_selftest_aggregate_summary_failed_profiles=-",
        "[ci-gate-summary] ci_profile_matrix_gate_selftest_aggregate_summary_skipped_profiles=-",
        "[ci-gate-summary] age5_close_digest_selftest_ok=1",
        "[ci-gate-summary] age5_policy_combined_digest_selftest_default_field_text=age5_close_digest_selftest_ok=0",
        '[ci-gate-summary] age5_policy_combined_digest_selftest_default_field={"age5_close_digest_selftest_ok":"0"}',
        "[ci-gate-summary] age5_combined_heavy_policy_report_path=-",
        "[ci-gate-summary] age5_combined_heavy_policy_report_exists=0",
        "[ci-gate-summary] age5_combined_heavy_policy_text_path=-",
        "[ci-gate-summary] age5_combined_heavy_policy_text_exists=0",
        "[ci-gate-summary] age5_combined_heavy_policy_summary_path=-",
        "[ci-gate-summary] age5_combined_heavy_policy_summary_exists=0",
        "[ci-gate-summary] "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}="
        f"{age5_policy_summary_fields[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY]}",
        "[ci-gate-summary] "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY}="
        + json.dumps(age5_policy_origin_trace, ensure_ascii=False, sort_keys=True, separators=(",", ":")),
        f"[ci-gate-summary] {AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}=-",
        f"[ci-gate-summary] {AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}=-",
        f"[ci-gate-summary] {AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}=-",
        f"[ci-gate-summary] {AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY}=-",
        "[ci-gate-summary] ci_pack_golden_overlay_compare_selftest_ok=1",
        "[ci-gate-summary] ci_pack_golden_overlay_session_selftest_ok=1",
        f"[ci-gate-summary] age2_status={age2_status}",
        f"[ci-gate-summary] age3_status={age3_status}",
        f"[ci-gate-summary] age4_status={age4_status}",
        f"[ci-gate-summary] age4_proof_ok={age4_proof_ok}",
        f"[ci-gate-summary] age4_proof_failed_criteria={age4_proof_failed_criteria}",
        f"[ci-gate-summary] age4_proof_failed_preview={age4_proof_failed_preview}",
        f"[ci-gate-summary] age4_proof_summary_hash={age4_proof_summary_hash}",
        f"[ci-gate-summary] age5_status={age5_status}",
        *[f"[ci-gate-summary] {key}={value}" for key, value in age5_child_summary_fields.items()],
        f"[ci-gate-summary] seamgrim_phase3_cleanup={phase3_cleanup}",
        f"[ci-gate-summary] seamgrim_wasm_cli_diag_parity_report={seamgrim_wasm_cli_diag_parity_report}",
        "[ci-gate-summary] seamgrim_wasm_cli_diag_parity_ok=1",
        "[ci-gate-summary] seamgrim_group_id_summary_status=ok",
        f"[ci-gate-summary] seamgrim_5min_checklist={seamgrim_5min_checklist}",
        "[ci-gate-summary] seamgrim_5min_checklist_ok=1",
        "[ci-gate-summary] seamgrim_runtime_5min_rewrite_motion_projectile=1",
        "[ci-gate-summary] seamgrim_runtime_5min_rewrite_elapsed_ms=321",
        "[ci-gate-summary] seamgrim_runtime_5min_rewrite_status=ok",
        "[ci-gate-summary] seamgrim_runtime_5min_moyang_view_boundary=1",
        "[ci-gate-summary] seamgrim_runtime_5min_moyang_elapsed_ms=654",
        "[ci-gate-summary] seamgrim_runtime_5min_moyang_status=ok",
        "[ci-gate-summary] seamgrim_runtime_5min_pendulum_tetris_showcase=1",
        "[ci-gate-summary] seamgrim_runtime_5min_pendulum_tetris_showcase_elapsed_ms=777",
        "[ci-gate-summary] seamgrim_runtime_5min_pendulum_tetris_showcase_status=ok",
        f"[ci-gate-summary] ci_sanity_gate_report={ci_sanity_gate}",
        "[ci-gate-summary] ci_sanity_gate_status=pass",
        "[ci-gate-summary] ci_sanity_gate_ok=1",
        "[ci-gate-summary] ci_sanity_gate_code=OK",
        "[ci-gate-summary] ci_sanity_gate_step=all",
        "[ci-gate-summary] ci_sanity_gate_profile=full",
        "[ci-gate-summary] ci_sanity_gate_msg=-",
        "[ci-gate-summary] ci_sanity_gate_step_count=14",
        "[ci-gate-summary] ci_sanity_gate_failed_steps=0",
        "[ci-gate-summary] ci_sanity_pipeline_emit_flags_ok=1",
        "[ci-gate-summary] ci_sanity_pipeline_emit_flags_selftest_ok=1",
        "[ci-gate-summary] ci_sanity_emit_artifacts_sanity_contract_selftest_ok=1",
        "[ci-gate-summary] ci_sanity_age2_completion_gate_ok=1",
        "[ci-gate-summary] ci_sanity_age2_completion_gate_selftest_ok=1",
        "[ci-gate-summary] ci_sanity_age3_completion_gate_ok=1",
        "[ci-gate-summary] ci_sanity_age3_completion_gate_selftest_ok=1",
        "[ci-gate-summary] ci_sanity_age2_completion_gate_failure_codes=-",
        "[ci-gate-summary] ci_sanity_age2_completion_gate_failure_code_count=0",
        "[ci-gate-summary] ci_sanity_age3_completion_gate_failure_codes=-",
        "[ci-gate-summary] ci_sanity_age3_completion_gate_failure_code_count=0",
        *[f"[ci-gate-summary] {key}=1" for key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS],
        "[ci-gate-summary] ci_sanity_age3_bogae_geoul_visibility_smoke_ok=1",
        "[ci-gate-summary] "
        f"ci_sanity_age3_bogae_geoul_visibility_smoke_report_path={age3_bogae_geoul_visibility_smoke_report}",
        "[ci-gate-summary] ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists=1",
        "[ci-gate-summary] ci_sanity_age3_bogae_geoul_visibility_smoke_schema=ddn.bogae_geoul_visibility_smoke.v1",
        "[ci-gate-summary] ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok=1",
        "[ci-gate-summary] ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok=1",
        "[ci-gate-summary] ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes=1",
        "[ci-gate-summary] ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes=1",
        "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_ok=na",
        "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_report_path=-",
        "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists=na",
        "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_schema=-",
        "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count=-",
        "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count=-",
        "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_ok=na",
        "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_report_path=-",
        "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_report_exists=na",
        "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_schema=-",
        "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_checked_files=-",
        "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_missing_count=-",
        "[ci-gate-summary] ci_sanity_age5_combined_heavy_policy_selftest_ok=1",
        "[ci-gate-summary] ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok=1",
        "[ci-gate-summary] ci_sanity_dynamic_source_profile_split_selftest_ok=1",
        *[
            f"[ci-gate-summary] {key}={value}"
            for key, value in build_age5_combined_heavy_sanity_contract_fields().items()
        ],
        "[ci-gate-summary] ci_sanity_seamgrim_interface_boundary_ok=1",
        "[ci-gate-summary] ci_sanity_overlay_session_wired_consistency_ok=1",
        "[ci-gate-summary] ci_sanity_overlay_session_diag_parity_ok=1",
        "[ci-gate-summary] ci_sanity_overlay_compare_diag_parity_ok=1",
        "[ci-gate-summary] ci_sanity_pack_golden_lang_consistency_ok=1",
        "[ci-gate-summary] ci_sanity_pack_golden_metadata_ok=1",
        "[ci-gate-summary] ci_sanity_pack_golden_graph_export_ok=1",
        "[ci-gate-summary] ci_sanity_canon_ast_dpack_ok=1",
        "[ci-gate-summary] ci_sanity_contract_tier_unsupported_ok=1",
        "[ci-gate-summary] ci_sanity_contract_tier_age3_min_enforcement_ok=1",
        "[ci-gate-summary] ci_sanity_map_access_contract_ok=1",
        "[ci-gate-summary] ci_sanity_stdlib_catalog_ok=1",
        "[ci-gate-summary] ci_sanity_stdlib_catalog_selftest_ok=1",
        "[ci-gate-summary] ci_sanity_tensor_v0_pack_ok=1",
        "[ci-gate-summary] ci_sanity_tensor_v0_cli_ok=1",
        "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_contract_ok=1",
        "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_live_ok=1",
        "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_readiness_selftest_ok=1",
        "[ci-gate-summary] ci_sanity_registry_strict_audit_ok=1",
        "[ci-gate-summary] ci_sanity_registry_defaults_ok=1",
        f"[ci-gate-summary] ci_sync_readiness_report={ci_sync_readiness}",
        "[ci-gate-summary] ci_sync_readiness_status=pass",
        "[ci-gate-summary] ci_sync_readiness_ok=1",
        "[ci-gate-summary] ci_sync_readiness_code=OK",
        "[ci-gate-summary] ci_sync_readiness_step=all",
        "[ci-gate-summary] ci_sync_readiness_sanity_profile=full",
        "[ci-gate-summary] ci_sync_readiness_msg=-",
        "[ci-gate-summary] ci_sync_readiness_step_count=1",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_pipeline_emit_flags_ok=1",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_pipeline_emit_flags_selftest_ok=1",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok=1",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok=1",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_age2_completion_gate_ok=1",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_age2_completion_gate_selftest_ok=1",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_completion_gate_ok=1",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_completion_gate_selftest_ok=1",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_age2_completion_gate_failure_codes=-",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_age2_completion_gate_failure_code_count=0",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_completion_gate_failure_codes=-",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_completion_gate_failure_code_count=0",
        *[
            f"[ci-gate-summary] {sync_key}=1"
            for _sanity_key, sync_key in AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS
        ],
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_ok=1",
        "[ci-gate-summary] "
        f"ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_path={age3_bogae_geoul_visibility_smoke_report}",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists=1",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_schema=ddn.bogae_geoul_visibility_smoke.v1",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok=1",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok=1",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes=1",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes=1",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok=na",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_path=-",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists=na",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_schema=-",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count=-",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count=-",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_ok=na",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_path=-",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_exists=na",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_schema=-",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_checked_files=-",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_missing_count=-",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_age5_combined_heavy_policy_selftest_ok=1",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok=1",
        "[ci-gate-summary] ci_sync_readiness_ci_sanity_dynamic_source_profile_split_selftest_ok=1",
        *[
            f"[ci-gate-summary] {key}={value}"
            for key, value in build_age5_combined_heavy_sync_contract_fields().items()
        ],
        f"[ci-gate-summary] fixed64_threeway_report={fixed64_threeway_report}",
        "[ci-gate-summary] fixed64_threeway_status=pending_darwin",
        "[ci-gate-summary] fixed64_threeway_ok=1",
    ]
    for profile_name in PROFILE_MATRIX_SELFTEST_PROFILES:
        expected_contract = expected_profile_matrix_aggregate_summary_contract(profile_name)
        lines.extend(
            [
                f"[ci-gate-summary] ci_profile_matrix_gate_selftest_{profile_name}_aggregate_summary_status={expected_contract['status']}",
                "[ci-gate-summary] "
                f"ci_profile_matrix_gate_selftest_{profile_name}_aggregate_summary_ok="
                f"{1 if expected_contract['ok'] else 0}",
                "[ci-gate-summary] "
                f"ci_profile_matrix_gate_selftest_{profile_name}_aggregate_summary_values="
                f"{expected_contract['values_text']}",
            ]
        )
    write_text(summary_path, "\n".join(lines))
    return summary_path, index_path


def remove_line_with_prefix(path: Path, prefix: str) -> None:
    lines = path.read_text(encoding="utf-8").splitlines()
    filtered = [line for line in lines if not line.startswith(prefix)]
    write_text(path, "\n".join(filtered))


@contextmanager
def persistent_tmpdir(prefix: str):
    # Selftest speedup: skip TemporaryDirectory cleanup(rmtree) cost.
    yield tempfile.mkdtemp(prefix=prefix)


def main() -> int:
    with persistent_tmpdir(prefix="ci_gate_summary_report_selftest_") as tmp:
        root = Path(tmp)

        summary_ok, index_ok = build_pass_case(root, "ok")
        proc_ok = run_check(summary_ok, index_ok, require_pass=True)
        if proc_ok.returncode != 0:
            return fail(f"ok case failed: out={proc_ok.stdout} err={proc_ok.stderr}")

        summary_ok_seamgrim, index_ok_seamgrim = build_pass_case(root, "ok_seamgrim_pack_evidence")
        seamgrim_case_dir = root / "ok_seamgrim_pack_evidence"
        seamgrim_wasm_report = seamgrim_case_dir / "seamgrim_wasm_web_step_check.detjson"
        seamgrim_pack_evidence_report = seamgrim_case_dir / "seamgrim_pack_evidence_tier_runner_check.detjson"
        write_json(
            seamgrim_wasm_report,
            {
                "schema": "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1",
                "status": "pass",
                "ok": True,
                "code": "OK",
                "checked_files": 20,
                "missing_count": 0,
                "missing": [],
            },
        )
        write_json(
            seamgrim_pack_evidence_report,
            {
                "schema": "ddn.pack_evidence_tier_runner_check.v1",
                "status": "pass",
                "ok": True,
                "docs_profile": {"name": "docs_ssot_rep10", "issue_count": 0},
                "repo_profile": {"name": "repo_rep10", "issue_count": 0},
            },
        )
        seamgrim_summary_text = summary_ok_seamgrim.read_text(encoding="utf-8")
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sanity_gate_profile=full",
            "[ci-gate-summary] ci_sanity_gate_profile=seamgrim",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sync_readiness_sanity_profile=full",
            "[ci-gate-summary] ci_sync_readiness_sanity_profile=seamgrim",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sanity_pipeline_emit_flags_ok=1",
            "[ci-gate-summary] ci_sanity_pipeline_emit_flags_ok=na",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sanity_pipeline_emit_flags_selftest_ok=1",
            "[ci-gate-summary] ci_sanity_pipeline_emit_flags_selftest_ok=na",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_pipeline_emit_flags_ok=1",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_pipeline_emit_flags_ok=na",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_pipeline_emit_flags_selftest_ok=1",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_pipeline_emit_flags_selftest_ok=na",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sanity_pack_golden_graph_export_ok=1",
            "[ci-gate-summary] ci_sanity_pack_golden_graph_export_ok=0",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok=1",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok=0",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_ok=na",
            "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_ok=1",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_report_path=-",
            "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_report_path="
            f"{seamgrim_pack_evidence_report}",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists=na",
            "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists=1",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_schema=-",
            "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_schema="
            "ddn.pack_evidence_tier_runner_check.v1",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count=-",
            "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count=0",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count=-",
            "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count=0",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok=na",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok=1",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_path=-",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_path="
            f"{seamgrim_pack_evidence_report}",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists=na",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists=1",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_schema=-",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_schema="
            "ddn.pack_evidence_tier_runner_check.v1",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count=-",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count=0",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count=-",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count=0",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_ok=na",
            "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_ok=1",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_report_path=-",
            "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_report_path="
            f"{seamgrim_wasm_report}",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_report_exists=na",
            "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_report_exists=1",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_schema=-",
            "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_schema="
            "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_checked_files=-",
            "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_checked_files=20",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_missing_count=-",
            "[ci-gate-summary] ci_sanity_seamgrim_wasm_web_step_check_missing_count=0",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_ok=na",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_ok=1",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_path=-",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_path="
            f"{seamgrim_wasm_report}",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_exists=na",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_exists=1",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_schema=-",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_schema="
            "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_checked_files=-",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_checked_files=20",
        )
        seamgrim_summary_text = seamgrim_summary_text.replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_missing_count=-",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_missing_count=0",
        )
        write_text(summary_ok_seamgrim, seamgrim_summary_text)
        proc_ok_seamgrim = run_check(summary_ok_seamgrim, index_ok_seamgrim, require_pass=True)
        if proc_ok_seamgrim.returncode != 0:
            return fail(
                "ok_seamgrim_pack_evidence case failed: "
                f"out={proc_ok_seamgrim.stdout} err={proc_ok_seamgrim.stderr}"
            )
        seamgrim_summary_base_text = seamgrim_summary_text

        seamgrim_bad_pack_evidence_schema_text = seamgrim_summary_base_text.replace(
            "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_schema="
            "ddn.pack_evidence_tier_runner_check.v1",
            "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_schema="
            "ddn.pack_evidence_tier_runner_check.v0",
        ).replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_schema="
            "ddn.pack_evidence_tier_runner_check.v1",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_schema="
            "ddn.pack_evidence_tier_runner_check.v0",
        )
        write_text(summary_ok_seamgrim, seamgrim_bad_pack_evidence_schema_text)
        proc_seamgrim_bad_pack_evidence_schema = run_check(
            summary_ok_seamgrim,
            index_ok_seamgrim,
            require_pass=True,
        )
        if proc_seamgrim_bad_pack_evidence_schema.returncode == 0:
            return fail("seamgrim pack evidence schema mismatch case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_seamgrim_bad_pack_evidence_schema.stderr:
            return fail(
                "seamgrim pack evidence schema mismatch code mismatch: "
                f"err={proc_seamgrim_bad_pack_evidence_schema.stderr}"
            )

        seamgrim_bad_pack_evidence_repo_issue_text = seamgrim_summary_base_text.replace(
            "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count=0",
            "[ci-gate-summary] ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count=1",
        ).replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count=0",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count=1",
        )
        write_text(summary_ok_seamgrim, seamgrim_bad_pack_evidence_repo_issue_text)
        proc_seamgrim_bad_pack_evidence_repo_issue = run_check(
            summary_ok_seamgrim,
            index_ok_seamgrim,
            require_pass=True,
        )
        if proc_seamgrim_bad_pack_evidence_repo_issue.returncode == 0:
            return fail("seamgrim pack evidence repo issue count mismatch case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_seamgrim_bad_pack_evidence_repo_issue.stderr:
            return fail(
                "seamgrim pack evidence repo issue count mismatch code mismatch: "
                f"err={proc_seamgrim_bad_pack_evidence_repo_issue.stderr}"
            )

        seamgrim_bad_pack_evidence_sync_mirror_text = seamgrim_summary_base_text.replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_path="
            f"{seamgrim_pack_evidence_report}",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_path="
            f"{seamgrim_wasm_report}",
        )
        write_text(summary_ok_seamgrim, seamgrim_bad_pack_evidence_sync_mirror_text)
        proc_seamgrim_bad_pack_evidence_sync_mirror = run_check(
            summary_ok_seamgrim,
            index_ok_seamgrim,
            require_pass=True,
        )
        if proc_seamgrim_bad_pack_evidence_sync_mirror.returncode == 0:
            return fail("seamgrim pack evidence sanity/sync mirror mismatch case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_seamgrim_bad_pack_evidence_sync_mirror.stderr:
            return fail(
                "seamgrim pack evidence sanity/sync mirror mismatch code mismatch: "
                f"err={proc_seamgrim_bad_pack_evidence_sync_mirror.stderr}"
            )

        write_text(summary_ok_seamgrim, seamgrim_summary_base_text)

        summary_missing_age4_proof, index_missing_age4_proof = build_pass_case(root, "missing_age4_proof")
        remove_line_with_prefix(summary_missing_age4_proof, "[ci-gate-summary] age4_proof_ok=")
        proc_missing_age4_proof = run_check(summary_missing_age4_proof, index_missing_age4_proof, require_pass=True)
        if proc_missing_age4_proof.returncode == 0:
            return fail("missing age4_proof_ok must fail")

        summary_missing_age4_preview, index_missing_age4_preview = build_pass_case(root, "missing_age4_preview")
        remove_line_with_prefix(summary_missing_age4_preview, "[ci-gate-summary] age4_proof_failed_preview=")
        proc_missing_age4_preview = run_check(summary_missing_age4_preview, index_missing_age4_preview, require_pass=True)
        if proc_missing_age4_preview.returncode == 0:
            return fail("missing age4_proof_failed_preview must fail")

        summary_ok_without_runtime5, index_ok_without_runtime5 = build_pass_case(root, "ok_without_runtime5")
        checklist_path = root / "ok_without_runtime5" / "seamgrim_5min_checklist_report.detjson"
        if checklist_path.exists():
            checklist_path.unlink()
        remove_line_with_prefix(summary_ok_without_runtime5, "[ci-gate-summary] seamgrim_5min_checklist=")
        remove_line_with_prefix(summary_ok_without_runtime5, "[ci-gate-summary] seamgrim_5min_checklist_ok=")
        remove_line_with_prefix(
            summary_ok_without_runtime5,
            "[ci-gate-summary] seamgrim_runtime_5min_rewrite_motion_projectile=",
        )
        remove_line_with_prefix(
            summary_ok_without_runtime5,
            "[ci-gate-summary] seamgrim_runtime_5min_rewrite_elapsed_ms=",
        )
        remove_line_with_prefix(
            summary_ok_without_runtime5,
            "[ci-gate-summary] seamgrim_runtime_5min_rewrite_status=",
        )
        remove_line_with_prefix(
            summary_ok_without_runtime5,
            "[ci-gate-summary] seamgrim_runtime_5min_moyang_view_boundary=",
        )
        remove_line_with_prefix(
            summary_ok_without_runtime5,
            "[ci-gate-summary] seamgrim_runtime_5min_moyang_elapsed_ms=",
        )
        remove_line_with_prefix(
            summary_ok_without_runtime5,
            "[ci-gate-summary] seamgrim_runtime_5min_moyang_status=",
        )
        remove_line_with_prefix(
            summary_ok_without_runtime5,
            "[ci-gate-summary] seamgrim_runtime_5min_pendulum_tetris_showcase=",
        )
        remove_line_with_prefix(
            summary_ok_without_runtime5,
            "[ci-gate-summary] seamgrim_runtime_5min_pendulum_tetris_showcase_elapsed_ms=",
        )
        remove_line_with_prefix(
            summary_ok_without_runtime5,
            "[ci-gate-summary] seamgrim_runtime_5min_pendulum_tetris_showcase_status=",
        )
        proc_ok_without_runtime5 = run_check(summary_ok_without_runtime5, index_ok_without_runtime5, require_pass=True)
        if proc_ok_without_runtime5.returncode != 0:
            return fail(
                "ok_without_runtime5 case failed: "
                f"out={proc_ok_without_runtime5.stdout} err={proc_ok_without_runtime5.stderr}"
            )

        summary_missing_key, index_missing_key = build_pass_case(root, "missing_key")
        text_missing_key = summary_missing_key.read_text(encoding="utf-8")
        text_missing_key = text_missing_key.replace("[ci-gate-summary] ci_gate_badge=", "[ci-gate-summary] REMOVED=")
        write_text(summary_missing_key, text_missing_key)
        proc_missing_key = run_check(summary_missing_key, index_missing_key, require_pass=True)
        if proc_missing_key.returncode == 0:
            return fail("missing key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_missing_key.stderr:
            return fail(f"missing key code mismatch: err={proc_missing_key.stderr}")

        summary_age5_child_mismatch, index_age5_child_mismatch = build_pass_case(root, "age5_child_mismatch")
        age5_child_mismatch_text = summary_age5_child_mismatch.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] age5_combined_heavy_runtime_helper_negative_status=skipped",
            "[ci-gate-summary] age5_combined_heavy_runtime_helper_negative_status=fail",
        )
        write_text(summary_age5_child_mismatch, age5_child_mismatch_text)
        proc_age5_child_mismatch = run_check(summary_age5_child_mismatch, index_age5_child_mismatch, require_pass=True)
        if proc_age5_child_mismatch.returncode == 0:
            return fail("age5 child summary mismatch case must fail")
        if f"fail code={CODES['SUMMARY_INDEX_PATH_MISMATCH']}" not in proc_age5_child_mismatch.stderr:
            return fail(f"age5 child summary mismatch code mismatch: err={proc_age5_child_mismatch.stderr}")

        summary_age5_policy_mismatch, index_age5_policy_mismatch = build_pass_case(root, "age5_policy_mismatch")
        age5_policy_mismatch_text = summary_age5_policy_mismatch.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] age5_combined_heavy_policy_report_exists=0",
            "[ci-gate-summary] age5_combined_heavy_policy_report_exists=1",
        )
        write_text(summary_age5_policy_mismatch, age5_policy_mismatch_text)
        proc_age5_policy_mismatch = run_check(summary_age5_policy_mismatch, index_age5_policy_mismatch, require_pass=True)
        if proc_age5_policy_mismatch.returncode == 0:
            return fail("age5 policy summary mismatch case must fail")
        if f"fail code={CODES['SUMMARY_INDEX_PATH_MISMATCH']}" not in proc_age5_policy_mismatch.stderr:
            return fail(f"age5 policy summary mismatch code mismatch: err={proc_age5_policy_mismatch.stderr}")

        summary_missing_sanity, index_missing_sanity = build_pass_case(root, "missing_sanity")
        text_missing_sanity = summary_missing_sanity.read_text(encoding="utf-8")
        text_missing_sanity = text_missing_sanity.replace(
            "[ci-gate-summary] ci_sanity_gate_status=pass",
            "[ci-gate-summary] REMOVED_SANITY_STATUS=pass",
        )
        write_text(summary_missing_sanity, text_missing_sanity)
        proc_missing_sanity = run_check(summary_missing_sanity, index_missing_sanity, require_pass=True)
        if proc_missing_sanity.returncode == 0:
            return fail("missing ci_sanity key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_missing_sanity.stderr:
            return fail(f"missing ci_sanity key code mismatch: err={proc_missing_sanity.stderr}")
        summary_missing_sync, index_missing_sync = build_pass_case(root, "missing_sync")
        text_missing_sync = summary_missing_sync.read_text(encoding="utf-8")
        text_missing_sync = text_missing_sync.replace(
            "[ci-gate-summary] ci_sync_readiness_status=pass",
            "[ci-gate-summary] REMOVED_SYNC_STATUS=pass",
        )
        write_text(summary_missing_sync, text_missing_sync)
        proc_missing_sync = run_check(summary_missing_sync, index_missing_sync, require_pass=True)
        if proc_missing_sync.returncode == 0:
            return fail("missing ci_sync key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_missing_sync.stderr:
            return fail(f"missing ci_sync key code mismatch: err={proc_missing_sync.stderr}")

        summary_bad_emit_artifacts_sanity, index_bad_emit_artifacts_sanity = build_pass_case(
            root,
            "bad_emit_artifacts_sanity",
        )
        bad_emit_artifacts_sanity_text = summary_bad_emit_artifacts_sanity.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_sanity_emit_artifacts_sanity_contract_selftest_ok=1",
            "[ci-gate-summary] ci_sanity_emit_artifacts_sanity_contract_selftest_ok=0",
        )
        write_text(summary_bad_emit_artifacts_sanity, bad_emit_artifacts_sanity_text)
        proc_bad_emit_artifacts_sanity = run_check(
            summary_bad_emit_artifacts_sanity,
            index_bad_emit_artifacts_sanity,
            require_pass=True,
        )
        if proc_bad_emit_artifacts_sanity.returncode == 0:
            return fail("ci_sanity_emit_artifacts_sanity_contract_selftest_ok=0 case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_emit_artifacts_sanity.stderr:
            return fail(
                "ci_sanity_emit_artifacts_sanity_contract_selftest_ok code mismatch: "
                f"err={proc_bad_emit_artifacts_sanity.stderr}"
            )

        summary_bad_emit_artifacts_sync_mirror, index_bad_emit_artifacts_sync_mirror = build_pass_case(
            root,
            "bad_emit_artifacts_sync_mirror",
        )
        bad_emit_artifacts_sync_mirror_text = summary_bad_emit_artifacts_sync_mirror.read_text(
            encoding="utf-8"
        ).replace(
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok=1",
            "[ci-gate-summary] ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok=0",
        )
        write_text(summary_bad_emit_artifacts_sync_mirror, bad_emit_artifacts_sync_mirror_text)
        proc_bad_emit_artifacts_sync_mirror = run_check(
            summary_bad_emit_artifacts_sync_mirror,
            index_bad_emit_artifacts_sync_mirror,
            require_pass=True,
        )
        if proc_bad_emit_artifacts_sync_mirror.returncode == 0:
            return fail(
                "ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok=0 case must fail"
            )
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_emit_artifacts_sync_mirror.stderr:
            return fail(
                "ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok code mismatch: "
                f"err={proc_bad_emit_artifacts_sync_mirror.stderr}"
            )

        summary_missing_parity_key, index_missing_parity_key = build_pass_case(root, "missing_parity_key")
        text_missing_parity_key = summary_missing_parity_key.read_text(encoding="utf-8")
        text_missing_parity_key = text_missing_parity_key.replace(
            "[ci-gate-summary] seamgrim_wasm_cli_diag_parity_report=",
            "[ci-gate-summary] REMOVED_PARITY_REPORT=",
        )
        write_text(summary_missing_parity_key, text_missing_parity_key)
        proc_missing_parity_key = run_check(summary_missing_parity_key, index_missing_parity_key, require_pass=True)
        if proc_missing_parity_key.returncode == 0:
            return fail("missing seamgrim_wasm_cli_diag_parity_report case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_missing_parity_key.stderr:
            return fail(f"missing seamgrim_wasm_cli_diag_parity_report code mismatch: err={proc_missing_parity_key.stderr}")

        summary_bad_parity_mismatch, index_bad_parity_mismatch = build_pass_case(root, "bad_parity_mismatch")
        bad_parity_mismatch_text = summary_bad_parity_mismatch.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] seamgrim_wasm_cli_diag_parity_report=",
            "[ci-gate-summary] seamgrim_wasm_cli_diag_parity_report=wrong.detjson #",
        )
        write_text(summary_bad_parity_mismatch, bad_parity_mismatch_text)
        proc_bad_parity_mismatch = run_check(summary_bad_parity_mismatch, index_bad_parity_mismatch, require_pass=True)
        if proc_bad_parity_mismatch.returncode == 0:
            return fail("seamgrim_wasm_cli_diag_parity_report mismatch case must fail")
        if f"fail code={CODES['SUMMARY_INDEX_PATH_MISMATCH']}" not in proc_bad_parity_mismatch.stderr:
            return fail(f"seamgrim_wasm_cli_diag_parity_report mismatch code mismatch: err={proc_bad_parity_mismatch.stderr}")

        summary_bad_parity_ok, index_bad_parity_ok = build_pass_case(root, "bad_parity_ok")
        bad_parity_ok_text = summary_bad_parity_ok.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] seamgrim_wasm_cli_diag_parity_ok=1",
            "[ci-gate-summary] seamgrim_wasm_cli_diag_parity_ok=0",
        )
        write_text(summary_bad_parity_ok, bad_parity_ok_text)
        proc_bad_parity_ok = run_check(summary_bad_parity_ok, index_bad_parity_ok, require_pass=True)
        if proc_bad_parity_ok.returncode == 0:
            return fail("seamgrim_wasm_cli_diag_parity_ok=0 case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_parity_ok.stderr:
            return fail(f"seamgrim_wasm_cli_diag_parity_ok code mismatch: err={proc_bad_parity_ok.stderr}")

        summary_bad_profile_matrix_ok, index_bad_profile_matrix_ok = build_pass_case(root, "bad_profile_matrix_ok")
        bad_profile_matrix_ok_text = summary_bad_profile_matrix_ok.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_profile_matrix_gate_selftest_ok=1",
            "[ci-gate-summary] ci_profile_matrix_gate_selftest_ok=0",
        )
        write_text(summary_bad_profile_matrix_ok, bad_profile_matrix_ok_text)
        proc_bad_profile_matrix_ok = run_check(summary_bad_profile_matrix_ok, index_bad_profile_matrix_ok, require_pass=True)
        if proc_bad_profile_matrix_ok.returncode == 0:
            return fail("ci_profile_matrix_gate_selftest_ok=0 case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_profile_matrix_ok.stderr:
            return fail(f"ci_profile_matrix_gate_selftest_ok code mismatch: err={proc_bad_profile_matrix_ok.stderr}")

        summary_bad_profile_matrix_aggregate_ok, index_bad_profile_matrix_aggregate_ok = build_pass_case(
            root,
            "bad_profile_matrix_aggregate_ok",
        )
        bad_profile_matrix_aggregate_ok_text = summary_bad_profile_matrix_aggregate_ok.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_profile_matrix_gate_selftest_aggregate_summary_sanity_ok=1",
            "[ci-gate-summary] ci_profile_matrix_gate_selftest_aggregate_summary_sanity_ok=0",
        )
        write_text(summary_bad_profile_matrix_aggregate_ok, bad_profile_matrix_aggregate_ok_text)
        proc_bad_profile_matrix_aggregate_ok = run_check(
            summary_bad_profile_matrix_aggregate_ok,
            index_bad_profile_matrix_aggregate_ok,
            require_pass=True,
        )
        if proc_bad_profile_matrix_aggregate_ok.returncode == 0:
            return fail("ci_profile_matrix_gate_selftest_aggregate_summary_sanity_ok=0 case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_profile_matrix_aggregate_ok.stderr:
            return fail(
                "ci_profile_matrix_gate_selftest_aggregate_summary_sanity_ok code mismatch: "
                f"err={proc_bad_profile_matrix_aggregate_ok.stderr}"
            )

        summary_bad_profile_matrix_aggregate_values, index_bad_profile_matrix_aggregate_values = build_pass_case(
            root,
            "bad_profile_matrix_aggregate_values",
        )
        bad_profile_matrix_aggregate_values_report = (
            root / "bad_profile_matrix_aggregate_values" / "ci_profile_matrix_gate_selftest.detjson"
        )
        bad_profile_matrix_aggregate_values_doc = json.loads(
            bad_profile_matrix_aggregate_values_report.read_text(encoding="utf-8")
        )
        bad_profile_matrix_aggregate_values_doc["aggregate_summary_sanity_by_profile"]["full"]["values"][
            "ci_sanity_pack_golden_metadata_ok"
        ] = "0"
        write_json(bad_profile_matrix_aggregate_values_report, bad_profile_matrix_aggregate_values_doc)
        proc_bad_profile_matrix_aggregate_values = run_check(
            summary_bad_profile_matrix_aggregate_values,
            index_bad_profile_matrix_aggregate_values,
            require_pass=True,
        )
        if proc_bad_profile_matrix_aggregate_values.returncode == 0:
            return fail("ci_profile_matrix aggregate summary values mismatch case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_profile_matrix_aggregate_values.stderr:
            return fail(
                "ci_profile_matrix aggregate summary values mismatch code mismatch: "
                f"err={proc_bad_profile_matrix_aggregate_values.stderr}"
            )

        summary_bad_profile_matrix_report, index_bad_profile_matrix_report = build_pass_case(root, "bad_profile_matrix_report")
        bad_profile_matrix_report_text = summary_bad_profile_matrix_report.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_profile_matrix_gate_selftest_report=",
            "[ci-gate-summary] ci_profile_matrix_gate_selftest_report=wrong.detjson #",
        )
        write_text(summary_bad_profile_matrix_report, bad_profile_matrix_report_text)
        proc_bad_profile_matrix_report = run_check(
            summary_bad_profile_matrix_report,
            index_bad_profile_matrix_report,
            require_pass=True,
        )
        if proc_bad_profile_matrix_report.returncode == 0:
            return fail("ci_profile_matrix_gate_selftest_report mismatch case must fail")
        if f"fail code={CODES['SUMMARY_INDEX_PATH_MISMATCH']}" not in proc_bad_profile_matrix_report.stderr:
            return fail(
                "ci_profile_matrix_gate_selftest_report code mismatch: "
                f"err={proc_bad_profile_matrix_report.stderr}"
            )

        summary_bad_profile_matrix_elapsed, index_bad_profile_matrix_elapsed = build_pass_case(root, "bad_profile_matrix_elapsed")
        bad_profile_matrix_elapsed_text = summary_bad_profile_matrix_elapsed.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_profile_matrix_gate_selftest_total_elapsed_ms=666",
            "[ci-gate-summary] ci_profile_matrix_gate_selftest_total_elapsed_ms=-1",
        )
        write_text(summary_bad_profile_matrix_elapsed, bad_profile_matrix_elapsed_text)
        proc_bad_profile_matrix_elapsed = run_check(
            summary_bad_profile_matrix_elapsed,
            index_bad_profile_matrix_elapsed,
            require_pass=True,
        )
        if proc_bad_profile_matrix_elapsed.returncode == 0:
            return fail("ci_profile_matrix_gate_selftest_total_elapsed_ms=-1 case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_profile_matrix_elapsed.stderr:
            return fail(
                "ci_profile_matrix_gate_selftest_total_elapsed_ms code mismatch: "
                f"err={proc_bad_profile_matrix_elapsed.stderr}"
            )

        summary_missing_parity_file, index_missing_parity_file = build_pass_case(root, "missing_parity_file")
        missing_parity_file_text = summary_missing_parity_file.read_text(encoding="utf-8")
        marker = "[ci-gate-summary] seamgrim_wasm_cli_diag_parity_report="
        line = next((raw for raw in missing_parity_file_text.splitlines() if raw.startswith(marker)), "")
        if not line:
            return fail("missing_parity_file case: seamgrim_wasm_cli_diag_parity_report line missing")
        parity_file_path = Path(line[len(marker) :].strip())
        if parity_file_path.exists():
            parity_file_path.unlink()
        proc_missing_parity_file = run_check(summary_missing_parity_file, index_missing_parity_file, require_pass=True)
        if proc_missing_parity_file.returncode == 0:
            return fail("missing seamgrim_wasm_cli_diag_parity_report file case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_missing_parity_file.stderr:
            return fail(
                "missing seamgrim_wasm_cli_diag_parity_report file code mismatch: "
                f"err={proc_missing_parity_file.stderr}"
            )

        summary_missing_sync_file, index_missing_sync_file = build_pass_case(root, "missing_sync_file")
        text_missing_sync_file = summary_missing_sync_file.read_text(encoding="utf-8")
        sync_marker = "[ci-gate-summary] ci_sync_readiness_report="
        sync_line = next((raw for raw in text_missing_sync_file.splitlines() if raw.startswith(sync_marker)), "")
        if not sync_line:
            return fail("missing_sync_file case: ci_sync_readiness_report line missing")
        sync_file_path = Path(sync_line[len(sync_marker) :].strip())
        if sync_file_path.exists():
            sync_file_path.unlink()
        proc_missing_sync_file = run_check(summary_missing_sync_file, index_missing_sync_file, require_pass=True)
        if proc_missing_sync_file.returncode == 0:
            return fail("missing ci_sync_readiness_report file case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_missing_sync_file.stderr:
            return fail(
                "missing ci_sync_readiness_report file code mismatch: "
                f"err={proc_missing_sync_file.stderr}"
            )

        summary_bad_index, index_bad_index = build_pass_case(root, "bad_index")
        bad_text = summary_bad_index.read_text(encoding="utf-8").replace(
            f"[ci-gate-summary] report_index={index_bad_index}",
            "[ci-gate-summary] report_index=wrong.index.detjson",
        )
        write_text(summary_bad_index, bad_text)
        proc_bad_index = run_check(summary_bad_index, index_bad_index, require_pass=True)
        if proc_bad_index.returncode == 0:
            return fail("report_index mismatch case must fail")
        if f"fail code={CODES['REPORT_INDEX_MISMATCH']}" not in proc_bad_index.stderr:
            return fail(f"report_index code mismatch: err={proc_bad_index.stderr}")

        summary_bad_brief, index_bad_brief = build_pass_case(root, "bad_brief")
        bad_brief_text = summary_bad_brief.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_fail_brief_exists=1",
            "[ci-gate-summary] ci_fail_brief_exists=0",
        )
        write_text(summary_bad_brief, bad_brief_text)
        proc_bad_brief = run_check(summary_bad_brief, index_bad_brief, require_pass=True)
        if proc_bad_brief.returncode == 0:
            return fail("brief exists mismatch case must fail")
        if f"fail code={CODES['BRIEF_EXISTS_MISMATCH']}" not in proc_bad_brief.stderr:
            return fail(f"brief exists code mismatch: err={proc_bad_brief.stderr}")

        summary_bad_sanity_steps, index_bad_sanity_steps = build_pass_case(root, "bad_sanity_steps")
        bad_sanity_steps_text = summary_bad_sanity_steps.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_sanity_gate_step_count=14",
            "[ci-gate-summary] ci_sanity_gate_step_count=1",
        )
        write_text(summary_bad_sanity_steps, bad_sanity_steps_text)
        proc_bad_sanity_steps = run_check(summary_bad_sanity_steps, index_bad_sanity_steps, require_pass=True)
        if proc_bad_sanity_steps.returncode == 0:
            return fail("low ci_sanity_gate_step_count case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_steps.stderr:
            return fail(f"low ci_sanity_gate_step_count code mismatch: err={proc_bad_sanity_steps.stderr}")
        summary_bad_sync_steps, index_bad_sync_steps = build_pass_case(root, "bad_sync_steps")
        bad_sync_steps_text = summary_bad_sync_steps.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_sync_readiness_step_count=1",
            "[ci-gate-summary] ci_sync_readiness_step_count=0",
        )
        write_text(summary_bad_sync_steps, bad_sync_steps_text)
        proc_bad_sync_steps = run_check(summary_bad_sync_steps, index_bad_sync_steps, require_pass=True)
        if proc_bad_sync_steps.returncode == 0:
            return fail("low ci_sync_readiness_step_count case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sync_steps.stderr:
            return fail(f"low ci_sync_readiness_step_count code mismatch: err={proc_bad_sync_steps.stderr}")
        summary_bad_sanity_parity, index_bad_sanity_parity = build_pass_case(root, "bad_sanity_parity")
        bad_sanity_parity_text = summary_bad_sanity_parity.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_sanity_overlay_compare_diag_parity_ok=1",
            "[ci-gate-summary] ci_sanity_overlay_compare_diag_parity_ok=0",
        )
        write_text(summary_bad_sanity_parity, bad_sanity_parity_text)
        proc_bad_sanity_parity = run_check(summary_bad_sanity_parity, index_bad_sanity_parity, require_pass=True)
        if proc_bad_sanity_parity.returncode == 0:
            return fail("ci_sanity overlay compare parity key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_parity.stderr:
            return fail(f"ci_sanity overlay compare parity code mismatch: err={proc_bad_sanity_parity.stderr}")
        summary_bad_sanity_lang_consistency, index_bad_sanity_lang_consistency = build_pass_case(
            root,
            "bad_sanity_lang_consistency",
        )
        bad_sanity_lang_consistency_text = summary_bad_sanity_lang_consistency.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_sanity_pack_golden_lang_consistency_ok=1",
            "[ci-gate-summary] ci_sanity_pack_golden_lang_consistency_ok=0",
        )
        write_text(summary_bad_sanity_lang_consistency, bad_sanity_lang_consistency_text)
        proc_bad_sanity_lang_consistency = run_check(
            summary_bad_sanity_lang_consistency,
            index_bad_sanity_lang_consistency,
            require_pass=True,
        )
        if proc_bad_sanity_lang_consistency.returncode == 0:
            return fail("ci_sanity lang consistency key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_lang_consistency.stderr:
            return fail(f"ci_sanity lang consistency code mismatch: err={proc_bad_sanity_lang_consistency.stderr}")

        summary_bad_sanity_pack_metadata, index_bad_sanity_pack_metadata = build_pass_case(
            root,
            "bad_sanity_pack_metadata",
        )
        bad_sanity_pack_metadata_text = summary_bad_sanity_pack_metadata.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_sanity_pack_golden_metadata_ok=1",
            "[ci-gate-summary] ci_sanity_pack_golden_metadata_ok=0",
        )
        write_text(summary_bad_sanity_pack_metadata, bad_sanity_pack_metadata_text)
        proc_bad_sanity_pack_metadata = run_check(
            summary_bad_sanity_pack_metadata,
            index_bad_sanity_pack_metadata,
            require_pass=True,
        )
        if proc_bad_sanity_pack_metadata.returncode == 0:
            return fail("ci_sanity pack metadata key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_pack_metadata.stderr:
            return fail(f"ci_sanity pack metadata code mismatch: err={proc_bad_sanity_pack_metadata.stderr}")

        summary_bad_sanity_pack_graph_export, index_bad_sanity_pack_graph_export = build_pass_case(
            root,
            "bad_sanity_pack_graph_export",
        )
        bad_sanity_pack_graph_export_text = summary_bad_sanity_pack_graph_export.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_sanity_pack_golden_graph_export_ok=1",
            "[ci-gate-summary] ci_sanity_pack_golden_graph_export_ok=0",
        )
        write_text(summary_bad_sanity_pack_graph_export, bad_sanity_pack_graph_export_text)
        proc_bad_sanity_pack_graph_export = run_check(
            summary_bad_sanity_pack_graph_export,
            index_bad_sanity_pack_graph_export,
            require_pass=True,
        )
        if proc_bad_sanity_pack_graph_export.returncode == 0:
            return fail("ci_sanity pack graph export key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_pack_graph_export.stderr:
            return fail(f"ci_sanity pack graph export code mismatch: err={proc_bad_sanity_pack_graph_export.stderr}")

        summary_bad_sanity_canon_ast, index_bad_sanity_canon_ast = build_pass_case(
            root,
            "bad_sanity_canon_ast",
        )
        bad_sanity_canon_ast_text = summary_bad_sanity_canon_ast.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_sanity_canon_ast_dpack_ok=1",
            "[ci-gate-summary] ci_sanity_canon_ast_dpack_ok=0",
        )
        write_text(summary_bad_sanity_canon_ast, bad_sanity_canon_ast_text)
        proc_bad_sanity_canon_ast = run_check(
            summary_bad_sanity_canon_ast,
            index_bad_sanity_canon_ast,
            require_pass=True,
        )
        if proc_bad_sanity_canon_ast.returncode == 0:
            return fail("ci_sanity canon ast dpack key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_canon_ast.stderr:
            return fail(f"ci_sanity canon ast dpack code mismatch: err={proc_bad_sanity_canon_ast.stderr}")

        summary_bad_sanity_contract_tier, index_bad_sanity_contract_tier = build_pass_case(
            root,
            "bad_sanity_contract_tier",
        )
        bad_sanity_contract_tier_text = summary_bad_sanity_contract_tier.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_sanity_contract_tier_unsupported_ok=1",
            "[ci-gate-summary] ci_sanity_contract_tier_unsupported_ok=0",
        )
        write_text(summary_bad_sanity_contract_tier, bad_sanity_contract_tier_text)
        proc_bad_sanity_contract_tier = run_check(
            summary_bad_sanity_contract_tier,
            index_bad_sanity_contract_tier,
            require_pass=True,
        )
        if proc_bad_sanity_contract_tier.returncode == 0:
            return fail("ci_sanity contract tier key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_contract_tier.stderr:
            return fail(f"ci_sanity contract tier code mismatch: err={proc_bad_sanity_contract_tier.stderr}")
        summary_bad_sanity_contract_tier_age3, index_bad_sanity_contract_tier_age3 = build_pass_case(
            root,
            "bad_sanity_contract_tier_age3",
        )
        bad_sanity_contract_tier_age3_text = summary_bad_sanity_contract_tier_age3.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_sanity_contract_tier_age3_min_enforcement_ok=1",
            "[ci-gate-summary] ci_sanity_contract_tier_age3_min_enforcement_ok=0",
        )
        write_text(summary_bad_sanity_contract_tier_age3, bad_sanity_contract_tier_age3_text)
        proc_bad_sanity_contract_tier_age3 = run_check(
            summary_bad_sanity_contract_tier_age3,
            index_bad_sanity_contract_tier_age3,
            require_pass=True,
        )
        if proc_bad_sanity_contract_tier_age3.returncode == 0:
            return fail("ci_sanity contract tier age3 min enforcement key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_contract_tier_age3.stderr:
            return fail(
                "ci_sanity contract tier age3 min enforcement code mismatch: "
                f"err={proc_bad_sanity_contract_tier_age3.stderr}"
            )

        summary_bad_sanity_map_access, index_bad_sanity_map_access = build_pass_case(
            root,
            "bad_sanity_map_access",
        )
        bad_sanity_map_access_text = summary_bad_sanity_map_access.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_sanity_map_access_contract_ok=1",
            "[ci-gate-summary] ci_sanity_map_access_contract_ok=0",
        )
        write_text(summary_bad_sanity_map_access, bad_sanity_map_access_text)
        proc_bad_sanity_map_access = run_check(summary_bad_sanity_map_access, index_bad_sanity_map_access, require_pass=True)
        if proc_bad_sanity_map_access.returncode == 0:
            return fail("ci_sanity map access key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_map_access.stderr:
            return fail(f"ci_sanity map access code mismatch: err={proc_bad_sanity_map_access.stderr}")
        summary_bad_sanity_stdlib_catalog, index_bad_sanity_stdlib_catalog = build_pass_case(
            root,
            "bad_sanity_stdlib_catalog",
        )
        bad_sanity_stdlib_catalog_text = summary_bad_sanity_stdlib_catalog.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_sanity_stdlib_catalog_ok=1",
            "[ci-gate-summary] ci_sanity_stdlib_catalog_ok=0",
        )
        write_text(summary_bad_sanity_stdlib_catalog, bad_sanity_stdlib_catalog_text)
        proc_bad_sanity_stdlib_catalog = run_check(
            summary_bad_sanity_stdlib_catalog,
            index_bad_sanity_stdlib_catalog,
            require_pass=True,
        )
        if proc_bad_sanity_stdlib_catalog.returncode == 0:
            return fail("ci_sanity stdlib catalog key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_stdlib_catalog.stderr:
            return fail(f"ci_sanity stdlib catalog code mismatch: err={proc_bad_sanity_stdlib_catalog.stderr}")
        summary_bad_sanity_stdlib_catalog_selftest, index_bad_sanity_stdlib_catalog_selftest = build_pass_case(
            root,
            "bad_sanity_stdlib_catalog_selftest",
        )
        bad_sanity_stdlib_catalog_selftest_text = summary_bad_sanity_stdlib_catalog_selftest.read_text(
            encoding="utf-8"
        ).replace(
            "[ci-gate-summary] ci_sanity_stdlib_catalog_selftest_ok=1",
            "[ci-gate-summary] ci_sanity_stdlib_catalog_selftest_ok=0",
        )
        write_text(summary_bad_sanity_stdlib_catalog_selftest, bad_sanity_stdlib_catalog_selftest_text)
        proc_bad_sanity_stdlib_catalog_selftest = run_check(
            summary_bad_sanity_stdlib_catalog_selftest,
            index_bad_sanity_stdlib_catalog_selftest,
            require_pass=True,
        )
        if proc_bad_sanity_stdlib_catalog_selftest.returncode == 0:
            return fail("ci_sanity stdlib catalog selftest key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_stdlib_catalog_selftest.stderr:
            return fail(
                "ci_sanity stdlib catalog selftest code mismatch: "
                f"err={proc_bad_sanity_stdlib_catalog_selftest.stderr}"
            )
        summary_bad_sanity_registry_audit, index_bad_sanity_registry_audit = build_pass_case(
            root,
            "bad_sanity_registry_audit",
        )
        bad_sanity_registry_audit_text = summary_bad_sanity_registry_audit.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_sanity_registry_strict_audit_ok=1",
            "[ci-gate-summary] ci_sanity_registry_strict_audit_ok=0",
        )
        write_text(summary_bad_sanity_registry_audit, bad_sanity_registry_audit_text)
        proc_bad_sanity_registry_audit = run_check(
            summary_bad_sanity_registry_audit,
            index_bad_sanity_registry_audit,
            require_pass=True,
        )
        if proc_bad_sanity_registry_audit.returncode == 0:
            return fail("ci_sanity registry strict audit key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_registry_audit.stderr:
            return fail(f"ci_sanity registry strict audit code mismatch: err={proc_bad_sanity_registry_audit.stderr}")
        summary_bad_sanity_fixed64_darwin_real_report, index_bad_sanity_fixed64_darwin_real_report = build_pass_case(
            root,
            "bad_sanity_fixed64_darwin_real_report",
        )
        bad_sanity_fixed64_darwin_real_report_text = summary_bad_sanity_fixed64_darwin_real_report.read_text(
            encoding="utf-8"
        ).replace(
            "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_contract_ok=1",
            "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_contract_ok=0",
        )
        write_text(
            summary_bad_sanity_fixed64_darwin_real_report,
            bad_sanity_fixed64_darwin_real_report_text,
        )
        proc_bad_sanity_fixed64_darwin_real_report = run_check(
            summary_bad_sanity_fixed64_darwin_real_report,
            index_bad_sanity_fixed64_darwin_real_report,
            require_pass=True,
        )
        if proc_bad_sanity_fixed64_darwin_real_report.returncode == 0:
            return fail("ci_sanity fixed64 darwin real report key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_fixed64_darwin_real_report.stderr:
            return fail(
                "ci_sanity fixed64 darwin real report code mismatch: "
                f"err={proc_bad_sanity_fixed64_darwin_real_report.stderr}"
            )
        summary_bad_sanity_fixed64_darwin_live, index_bad_sanity_fixed64_darwin_live = build_pass_case(
            root,
            "bad_sanity_fixed64_darwin_live",
        )
        bad_sanity_fixed64_darwin_live_text = summary_bad_sanity_fixed64_darwin_live.read_text(
            encoding="utf-8"
        ).replace(
            "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_live_ok=1",
            "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_live_ok=0",
        )
        write_text(
            summary_bad_sanity_fixed64_darwin_live,
            bad_sanity_fixed64_darwin_live_text,
        )
        proc_bad_sanity_fixed64_darwin_live = run_check(
            summary_bad_sanity_fixed64_darwin_live,
            index_bad_sanity_fixed64_darwin_live,
            require_pass=True,
        )
        if proc_bad_sanity_fixed64_darwin_live.returncode == 0:
            return fail("ci_sanity fixed64 darwin live key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_fixed64_darwin_live.stderr:
            return fail(
                "ci_sanity fixed64 darwin live code mismatch: "
                f"err={proc_bad_sanity_fixed64_darwin_live.stderr}"
            )
        summary_bad_sanity_fixed64_darwin_readiness_selftest, index_bad_sanity_fixed64_darwin_readiness_selftest = (
            build_pass_case(
                root,
                "bad_sanity_fixed64_darwin_readiness_selftest",
            )
        )
        bad_sanity_fixed64_darwin_readiness_selftest_text = (
            summary_bad_sanity_fixed64_darwin_readiness_selftest.read_text(encoding="utf-8").replace(
                "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_readiness_selftest_ok=1",
                "[ci-gate-summary] ci_sanity_fixed64_darwin_real_report_readiness_selftest_ok=0",
            )
        )
        write_text(
            summary_bad_sanity_fixed64_darwin_readiness_selftest,
            bad_sanity_fixed64_darwin_readiness_selftest_text,
        )
        proc_bad_sanity_fixed64_darwin_readiness_selftest = run_check(
            summary_bad_sanity_fixed64_darwin_readiness_selftest,
            index_bad_sanity_fixed64_darwin_readiness_selftest,
            require_pass=True,
        )
        if proc_bad_sanity_fixed64_darwin_readiness_selftest.returncode == 0:
            return fail("ci_sanity fixed64 darwin readiness selftest key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_fixed64_darwin_readiness_selftest.stderr:
            return fail(
                "ci_sanity fixed64 darwin readiness selftest code mismatch: "
                f"err={proc_bad_sanity_fixed64_darwin_readiness_selftest.stderr}"
            )
        summary_bad_sanity_registry_defaults, index_bad_sanity_registry_defaults = build_pass_case(
            root,
            "bad_sanity_registry_defaults",
        )
        bad_sanity_registry_defaults_text = summary_bad_sanity_registry_defaults.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_sanity_registry_defaults_ok=1",
            "[ci-gate-summary] ci_sanity_registry_defaults_ok=0",
        )
        write_text(summary_bad_sanity_registry_defaults, bad_sanity_registry_defaults_text)
        proc_bad_sanity_registry_defaults = run_check(
            summary_bad_sanity_registry_defaults,
            index_bad_sanity_registry_defaults,
            require_pass=True,
        )
        if proc_bad_sanity_registry_defaults.returncode == 0:
            return fail("ci_sanity registry defaults key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_registry_defaults.stderr:
            return fail(f"ci_sanity registry defaults code mismatch: err={proc_bad_sanity_registry_defaults.stderr}")

        summary_missing_moyang, index_missing_moyang = build_pass_case(root, "missing_moyang")
        remove_line_with_prefix(
            summary_missing_moyang,
            "[ci-gate-summary] seamgrim_runtime_5min_moyang_view_boundary=",
        )
        proc_missing_moyang = run_check(summary_missing_moyang, index_missing_moyang, require_pass=True)
        if proc_missing_moyang.returncode == 0:
            return fail("missing seamgrim_runtime_5min_moyang_view_boundary case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_missing_moyang.stderr:
            return fail(f"missing seamgrim_runtime_5min_moyang_view_boundary code mismatch: err={proc_missing_moyang.stderr}")

        summary_bad_moyang_status, index_bad_moyang_status = build_pass_case(root, "bad_moyang_status")
        bad_moyang_status_text = summary_bad_moyang_status.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] seamgrim_runtime_5min_moyang_status=ok",
            "[ci-gate-summary] seamgrim_runtime_5min_moyang_status=failed",
        )
        write_text(summary_bad_moyang_status, bad_moyang_status_text)
        proc_bad_moyang_status = run_check(summary_bad_moyang_status, index_bad_moyang_status, require_pass=True)
        if proc_bad_moyang_status.returncode == 0:
            return fail("seamgrim_runtime_5min_moyang_status=failed case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_moyang_status.stderr:
            return fail(f"seamgrim_runtime_5min_moyang_status code mismatch: err={proc_bad_moyang_status.stderr}")

        summary_missing_showcase, index_missing_showcase = build_pass_case(root, "missing_showcase")
        remove_line_with_prefix(
            summary_missing_showcase,
            "[ci-gate-summary] seamgrim_runtime_5min_pendulum_tetris_showcase=",
        )
        proc_missing_showcase = run_check(summary_missing_showcase, index_missing_showcase, require_pass=True)
        if proc_missing_showcase.returncode == 0:
            return fail("missing seamgrim_runtime_5min_pendulum_tetris_showcase case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_missing_showcase.stderr:
            return fail(
                "missing seamgrim_runtime_5min_pendulum_tetris_showcase code mismatch: "
                f"err={proc_missing_showcase.stderr}"
            )

        summary_bad_showcase_status, index_bad_showcase_status = build_pass_case(root, "bad_showcase_status")
        bad_showcase_status_text = summary_bad_showcase_status.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] seamgrim_runtime_5min_pendulum_tetris_showcase_status=ok",
            "[ci-gate-summary] seamgrim_runtime_5min_pendulum_tetris_showcase_status=failed",
        )
        write_text(summary_bad_showcase_status, bad_showcase_status_text)
        proc_bad_showcase_status = run_check(summary_bad_showcase_status, index_bad_showcase_status, require_pass=True)
        if proc_bad_showcase_status.returncode == 0:
            return fail("seamgrim_runtime_5min_pendulum_tetris_showcase_status=failed case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_showcase_status.stderr:
            return fail(
                "seamgrim_runtime_5min_pendulum_tetris_showcase_status code mismatch: "
                f"err={proc_bad_showcase_status.stderr}"
            )

    print("[ci-gate-summary-report-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
