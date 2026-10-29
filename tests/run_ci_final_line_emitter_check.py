#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from _ci_age5_combined_heavy_contract import (
    AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_FRAGMENT,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
    build_age5_combined_heavy_policy_origin_trace_contract_compact_failure_reason,
    build_age5_combined_heavy_policy_origin_trace,
    build_age5_combined_heavy_policy_origin_trace_text,
    build_age4_proof_snapshot,
    build_age4_proof_snapshot_text,
    build_age5_combined_heavy_child_summary_default_text_transport_fields,
)
from _ci_profile_matrix_selftest_lib import (
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC,
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
    PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS,
    expected_profile_matrix_aggregate_summary_contract,
)
from ci_check_error_codes import SUMMARY_VERIFY_CODES

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
AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY = AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY
AGE5_POLICY_ORIGIN_TRACE_KEY = AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY
AGE4_PROOF_OK_KEY = "age4_proof_ok"
AGE4_PROOF_FAILED_CRITERIA_KEY = "age4_proof_failed_criteria"
AGE4_PROOF_FAILED_PREVIEW_KEY = "age4_proof_failed_preview"
AGE4_PROOF_SUMMARY_HASH_KEY = "age4_proof_summary_hash"
SEAMGRIM_PROFILE_MATRIX_VALUES = str(
    expected_profile_matrix_aggregate_summary_contract("seamgrim")["values_text"]
).strip()
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
AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_gate0_transport_family_contract_selftest_completed_checks": "4",
    "age5_full_real_gate0_transport_family_contract_selftest_total_checks": "4",
    "age5_full_real_gate0_transport_family_contract_selftest_checks_text": "lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family",
    "age5_full_real_gate0_transport_family_contract_selftest_current_probe": "-",
    "age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe": "gate0_transport_family",
    "age5_full_real_gate0_transport_family_contract_selftest_progress_present": "1",
}
AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE = {
    "age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks": "9",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks": "9",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe": "-",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe": "report_index",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present": "1",
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


def fail(msg: str) -> int:
    print(f"[ci-final-emitter-check] fail: {msg}")
    return 1


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text.rstrip() + "\n", encoding="utf-8")


def run_emit(report_dir: Path, *extra: str) -> subprocess.CompletedProcess[str]:
    cmd = [sys.executable, "tools/scripts/emit_ci_final_line.py", "--report-dir", str(report_dir), *extra]
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def build_case(
    report_dir: Path,
    prefix: str,
    status: str,
    reason: str,
    with_digest: bool,
    broken_summary: bool = False,
    extra_failed_step_count: int = 0,
) -> None:
    index_path = report_dir / f"{prefix}.ci_gate_report_index.detjson"
    summary_path = report_dir / f"{prefix}.ci_gate_summary_line.txt"
    summary_report_path = report_dir / f"{prefix}.ci_gate_summary.txt"
    result_path = report_dir / f"{prefix}.ci_gate_result.detjson"
    aggregate_path = report_dir / f"{prefix}.ci_aggregate_report.detjson"
    profile_matrix_path = report_dir / f"{prefix}.ci_profile_matrix_gate_selftest.detjson"
    policy_report_path = report_dir / f"{prefix}.age5_combined_heavy_policy.detjson"
    policy_text_path = report_dir / f"{prefix}.age5_combined_heavy_policy.txt"
    policy_summary_path = report_dir / f"{prefix}.age5_combined_heavy_policy_summary.txt"
    seamgrim_stdout = report_dir / f"{prefix}.seamgrim.stdout.txt"
    seamgrim_stderr = report_dir / f"{prefix}.seamgrim.stderr.txt"
    oi_stdout = report_dir / f"{prefix}.oi.stdout.txt"
    oi_stderr = report_dir / f"{prefix}.oi.stderr.txt"
    write_text(seamgrim_stdout, "sg out 1\nsg out 2\nsg out 3")
    write_text(seamgrim_stderr, "sg err 1\nsg err 2\nsg err 3")
    write_text(oi_stdout, "oi out 1\noi out 2\noi out 3")
    write_text(oi_stderr, "oi err 1\noi err 2\noi err 3")
    failed_step_rows: list[tuple[str, Path, Path, str]] = [
        (
            "seamgrim_ci_gate",
            seamgrim_stdout,
            seamgrim_stderr,
            "python tests/run_seamgrim_ci_gate.py",
        ),
        (
            "oi405_406_close",
            oi_stdout,
            oi_stderr,
            "python tests/run_oi405_406_close.py",
        ),
    ]
    for idx in range(max(0, int(extra_failed_step_count))):
        step_id = f"extra_fail_{idx + 1:02d}"
        step_stdout = report_dir / f"{prefix}.{step_id}.stdout.txt"
        step_stderr = report_dir / f"{prefix}.{step_id}.stderr.txt"
        write_text(step_stdout, f"{step_id} out 1\n{step_id} out 2")
        write_text(step_stderr, f"{step_id} err 1\n{step_id} err 2")
        failed_step_rows.append(
            (
                step_id,
                step_stdout,
                step_stderr,
                f"python tests/run_{step_id}.py",
            )
        )
    failed_step_total = 0 if status == "pass" else len(failed_step_rows)
    write_json(
        policy_report_path,
        {
            "schema": "ddn.ci.age5_combined_heavy_policy.v1",
            AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
            "combined_digest_selftest_default_field": {"age5_close_digest_selftest_ok": "0"},
        },
    )
    write_text(
        policy_text_path,
        "combined_digest_selftest_default_field={\"age5_close_digest_selftest_ok\":\"0\"} "
        "combined_digest_selftest_default_field_text=age5_close_digest_selftest_ok=0",
    )
    write_text(
        policy_summary_path,
        f"[age5-combined-heavy-policy] {AGE5_POLICY_REPORT_PATH_KEY}={policy_report_path} "
        f"{AGE5_POLICY_TEXT_PATH_KEY}={policy_text_path}",
    )
    policy_origin_trace = build_age5_combined_heavy_policy_origin_trace(
        report_path=str(policy_report_path),
        report_exists=True,
        text_path=str(policy_text_path),
        text_exists=True,
        summary_path=str(policy_summary_path),
        summary_exists=True,
    )
    age5_policy_age4_proof_snapshot = build_age4_proof_snapshot()
    age5_policy_age4_proof_snapshot_text = build_age4_proof_snapshot_text(
        age5_policy_age4_proof_snapshot
    )
    write_text(
        summary_path,
        f"ci_gate_result_status={status} ok={1 if status == 'pass' else 0} "
        f"overall_ok={1 if status == 'pass' else 0} failed_steps={failed_step_total} "
        f"aggregate_status={status} "
        f"age5_w107_active={AGE5_W107_PROGRESS_FIXTURE['age5_full_real_w107_golden_index_selftest_active_cases']} "
        f"age5_w107_inactive={AGE5_W107_PROGRESS_FIXTURE['age5_full_real_w107_golden_index_selftest_inactive_cases']} "
        f"age5_w107_index_codes={AGE5_W107_PROGRESS_FIXTURE['age5_full_real_w107_golden_index_selftest_index_codes']} "
        f"age5_w107_current_probe={AGE5_W107_PROGRESS_FIXTURE['age5_full_real_w107_golden_index_selftest_current_probe']} "
        f"age5_w107_last_completed_probe={AGE5_W107_PROGRESS_FIXTURE['age5_full_real_w107_golden_index_selftest_last_completed_probe']} "
        f"age5_w107_progress={AGE5_W107_PROGRESS_FIXTURE['age5_full_real_w107_golden_index_selftest_progress_present']} "
        f"age5_w107_contract_completed={AGE5_W107_CONTRACT_PROGRESS_FIXTURE['age5_full_real_w107_progress_contract_selftest_completed_checks']} "
        f"age5_w107_contract_total={AGE5_W107_CONTRACT_PROGRESS_FIXTURE['age5_full_real_w107_progress_contract_selftest_total_checks']} "
        f"age5_w107_contract_checks_text={AGE5_W107_CONTRACT_PROGRESS_FIXTURE['age5_full_real_w107_progress_contract_selftest_checks_text']} "
        f"age5_w107_contract_current_probe={AGE5_W107_CONTRACT_PROGRESS_FIXTURE['age5_full_real_w107_progress_contract_selftest_current_probe']} "
        f"age5_w107_contract_last_completed_probe={AGE5_W107_CONTRACT_PROGRESS_FIXTURE['age5_full_real_w107_progress_contract_selftest_last_completed_probe']} "
        f"age5_w107_contract_progress={AGE5_W107_CONTRACT_PROGRESS_FIXTURE['age5_full_real_w107_progress_contract_selftest_progress_present']} "
        f"age5_age1_immediate_proof_operation_contract_completed={AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE['age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks']} "
        f"age5_age1_immediate_proof_operation_contract_total={AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE['age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks']} "
        f"age5_age1_immediate_proof_operation_contract_checks_text={AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE['age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text']} "
        f"age5_age1_immediate_proof_operation_contract_current_probe={AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE['age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe']} "
        f"age5_age1_immediate_proof_operation_contract_last_completed_probe={AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE['age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe']} "
        f"age5_age1_immediate_proof_operation_contract_progress={AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_FIXTURE['age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present']} "
        f"age5_proof_certificate_v1_consumer_contract_completed={AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks']} "
        f"age5_proof_certificate_v1_consumer_contract_total={AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks']} "
        f"age5_proof_certificate_v1_consumer_contract_checks_text={AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text']} "
        f"age5_proof_certificate_v1_consumer_contract_current_probe={AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe']} "
        f"age5_proof_certificate_v1_consumer_contract_last_completed_probe={AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe']} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_completed={AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks']} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_total={AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks']} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_checks_text={AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text']} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_current_probe={AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe']} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_last_completed_probe={AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe']} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_progress={AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present']} "
        f"age5_proof_certificate_v1_family_contract_completed={AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks']} "
        f"age5_proof_certificate_v1_family_contract_total={AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks']} "
        f"age5_proof_certificate_v1_family_contract_checks_text={AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text']} "
        f"age5_proof_certificate_v1_family_contract_current_probe={AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe']} "
        f"age5_proof_certificate_v1_family_contract_last_completed_probe={AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe']} "
        f"age5_proof_certificate_v1_family_contract_progress={AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present']} "
        f"age5_proof_certificate_family_contract_completed={AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_family_contract_selftest_completed_checks']} "
        f"age5_proof_certificate_family_contract_total={AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_family_contract_selftest_total_checks']} "
        f"age5_proof_certificate_family_contract_checks_text={AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_family_contract_selftest_checks_text']} "
        f"age5_proof_certificate_family_contract_current_probe={AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_family_contract_selftest_current_probe']} "
        f"age5_proof_certificate_family_contract_last_completed_probe={AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe']} "
        f"age5_proof_certificate_family_contract_progress={AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_family_contract_selftest_progress_present']} "
        f"age5_proof_family_contract_completed={AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_family_contract_selftest_completed_checks']} "
        f"age5_proof_family_contract_total={AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_family_contract_selftest_total_checks']} "
        f"age5_proof_family_contract_checks_text={AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_family_contract_selftest_checks_text']} "
        f"age5_proof_family_contract_current_probe={AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_family_contract_selftest_current_probe']} "
        f"age5_proof_family_contract_last_completed_probe={AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_family_contract_selftest_last_completed_probe']} "
        f"age5_proof_family_contract_progress={AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_family_contract_selftest_progress_present']} "
        f"age5_proof_family_transport_contract_completed={AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_family_transport_contract_selftest_completed_checks']} "
        f"age5_proof_family_transport_contract_total={AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_family_transport_contract_selftest_total_checks']} "
        f"age5_proof_family_transport_contract_checks_text={AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_family_transport_contract_selftest_checks_text']} "
        f"age5_proof_family_transport_contract_current_probe={AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_family_transport_contract_selftest_current_probe']} "
        f"age5_proof_family_transport_contract_last_completed_probe={AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_family_transport_contract_selftest_last_completed_probe']} "
        f"age5_proof_family_transport_contract_progress={AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_family_transport_contract_selftest_progress_present']} "
        f"age5_lang_surface_family_contract_completed={AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_surface_family_contract_selftest_completed_checks']} "
        f"age5_lang_surface_family_contract_total={AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_surface_family_contract_selftest_total_checks']} "
        f"age5_lang_surface_family_contract_checks_text={AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_surface_family_contract_selftest_checks_text']} "
        f"age5_lang_surface_family_contract_current_probe={AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_surface_family_contract_selftest_current_probe']} "
        f"age5_lang_surface_family_contract_last_completed_probe={AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_surface_family_contract_selftest_last_completed_probe']} "
        f"age5_lang_surface_family_contract_progress={AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_surface_family_contract_selftest_progress_present']} "
        f"age5_lang_runtime_family_contract_completed={AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_runtime_family_contract_selftest_completed_checks']} "
        f"age5_lang_runtime_family_contract_total={AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_runtime_family_contract_selftest_total_checks']} "
        f"age5_lang_runtime_family_contract_checks_text={AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_runtime_family_contract_selftest_checks_text']} "
        f"age5_lang_runtime_family_contract_current_probe={AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_runtime_family_contract_selftest_current_probe']} "
        f"age5_lang_runtime_family_contract_last_completed_probe={AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe']} "
        f"age5_lang_runtime_family_contract_progress={AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_runtime_family_contract_selftest_progress_present']} "
        f"age5_gate0_family_contract_completed={AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_family_contract_selftest_completed_checks']} "
        f"age5_gate0_family_contract_total={AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_family_contract_selftest_total_checks']} "
        f"age5_gate0_family_contract_checks_text={AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_family_contract_selftest_checks_text']} "
        f"age5_gate0_family_contract_current_probe={AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_family_contract_selftest_current_probe']} "
        f"age5_gate0_family_contract_last_completed_probe={AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_family_contract_selftest_last_completed_probe']} "
        f"age5_gate0_family_contract_progress={AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_family_contract_selftest_progress_present']} "
        f"age5_gate0_family_transport_contract_completed={AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_family_transport_contract_selftest_completed_checks']} "
        f"age5_gate0_family_transport_contract_total={AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_family_transport_contract_selftest_total_checks']} "
        f"age5_gate0_family_transport_contract_checks_text={AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_family_transport_contract_selftest_checks_text']} "
        f"age5_gate0_family_transport_contract_current_probe={AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_family_transport_contract_selftest_current_probe']} "
        f"age5_gate0_family_transport_contract_last_completed_probe={AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe']} "
        f"age5_gate0_family_transport_contract_progress={AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_family_transport_contract_selftest_progress_present']} "
        f"age5_gate0_transport_family_contract_completed={AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_transport_family_contract_selftest_completed_checks']} "
        f"age5_gate0_transport_family_contract_total={AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_transport_family_contract_selftest_total_checks']} "
        f"age5_gate0_transport_family_contract_checks_text={AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_transport_family_contract_selftest_checks_text']} "
        f"age5_gate0_transport_family_contract_current_probe={AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_transport_family_contract_selftest_current_probe']} "
        f"age5_gate0_transport_family_contract_last_completed_probe={AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe']} "
        f"age5_gate0_transport_family_contract_progress={AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_transport_family_contract_selftest_progress_present']} "
        f"age5_gate0_transport_family_transport_contract_completed={AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks']} "
        f"age5_gate0_transport_family_transport_contract_total={AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks']} "
        f"age5_gate0_transport_family_transport_contract_checks_text={AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text']} "
        f"age5_gate0_transport_family_transport_contract_current_probe={AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe']} "
        f"age5_gate0_transport_family_transport_contract_last_completed_probe={AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe']} "
        f"age5_gate0_transport_family_transport_contract_progress={AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present']} "
        f"age5_lang_runtime_family_transport_contract_completed={AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks']} "
        f"age5_lang_runtime_family_transport_contract_total={AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks']} "
        f"age5_lang_runtime_family_transport_contract_checks_text={AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text']} "
        f"age5_lang_runtime_family_transport_contract_current_probe={AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe']} "
        f"age5_lang_runtime_family_transport_contract_last_completed_probe={AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe']} "
        f"age5_lang_runtime_family_transport_contract_progress={AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present']} "
        f"age5_gate0_runtime_family_transport_contract_completed={AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks']} "
        f"age5_gate0_runtime_family_transport_contract_total={AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks']} "
        f"age5_gate0_runtime_family_transport_contract_checks_text={AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text']} "
        f"age5_gate0_runtime_family_transport_contract_current_probe={AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe']} "
        f"age5_gate0_runtime_family_transport_contract_last_completed_probe={AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe']} "
        f"age5_gate0_runtime_family_transport_contract_progress={AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present']} "
        f"age5_lang_surface_family_transport_contract_completed={AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks']} "
        f"age5_lang_surface_family_transport_contract_total={AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_surface_family_transport_contract_selftest_total_checks']} "
        f"age5_lang_surface_family_transport_contract_checks_text={AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_surface_family_transport_contract_selftest_checks_text']} "
        f"age5_lang_surface_family_transport_contract_current_probe={AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_surface_family_transport_contract_selftest_current_probe']} "
        f"age5_lang_surface_family_transport_contract_last_completed_probe={AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe']} "
        f"age5_lang_surface_family_transport_contract_progress={AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_lang_surface_family_transport_contract_selftest_progress_present']} "
        f"age5_proof_certificate_family_transport_contract_completed={AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks']} "
        f"age5_proof_certificate_family_transport_contract_total={AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks']} "
        f"age5_proof_certificate_family_transport_contract_checks_text={AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text']} "
        f"age5_proof_certificate_family_transport_contract_current_probe={AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe']} "
        f"age5_proof_certificate_family_transport_contract_last_completed_probe={AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe']} "
        f"age5_proof_certificate_family_transport_contract_progress={AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present']} "
        f"age5_bogae_alias_family_contract_completed={AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_bogae_alias_family_contract_selftest_completed_checks']} "
        f"age5_bogae_alias_family_contract_total={AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_bogae_alias_family_contract_selftest_total_checks']} "
        f"age5_bogae_alias_family_contract_checks_text={AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_bogae_alias_family_contract_selftest_checks_text']} "
        f"age5_bogae_alias_family_contract_current_probe={AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_bogae_alias_family_contract_selftest_current_probe']} "
        f"age5_bogae_alias_family_contract_last_completed_probe={AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe']} "
        f"age5_bogae_alias_family_contract_progress={AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_FIXTURE['age5_full_real_bogae_alias_family_contract_selftest_progress_present']} "
        f"age5_bogae_alias_family_transport_contract_completed={AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks']} "
        f"age5_bogae_alias_family_transport_contract_total={AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks']} "
        f"age5_bogae_alias_family_transport_contract_checks_text={AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text']} "
        f"age5_bogae_alias_family_transport_contract_current_probe={AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe']} "
        f"age5_bogae_alias_family_transport_contract_last_completed_probe={AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe']} "
        f"age5_bogae_alias_family_transport_contract_progress={AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present']} "
        f"age5_proof_certificate_v1_consumer_contract_progress={AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_FIXTURE['age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present']} "
        f"reason={reason}",
    )
    if status == "pass":
        write_text(
            summary_report_path,
            "\n".join(
                [
                    "[ci-gate-summary] PASS",
                    "[ci-gate-summary] failed_steps=(none)",
                    f"[ci-gate-summary] report_index={index_path}",
                    f"[ci-gate-summary] summary_line={summary_path}",
                    f"[ci-gate-summary] ci_gate_result={result_path}",
                    f"[ci-gate-summary] ci_gate_badge={report_dir / f'{prefix}.ci_gate_badge.detjson'}",
                    f"[ci-gate-summary] ci_fail_brief_hint={report_dir / f'{prefix}.ci_fail_brief.txt'}",
                    "[ci-gate-summary] ci_fail_brief_exists=0",
                    "[ci-gate-summary] age5_close_digest_selftest_ok=1",
                    f"[ci-gate-summary] age3_status={report_dir / f'{prefix}.age3_close_status.detjson'}",
                    f"[ci-gate-summary] age4_status={report_dir / f'{prefix}.age4_close_report.detjson'}",
                ]
            ),
        )
    else:
        fail_lines = [
            "[ci-gate-summary] FAIL",
            "[ci-gate-summary] failed_steps=" + ",".join(step_id for step_id, _, _, _ in failed_step_rows),
        ]
        for step_id, step_stdout, step_stderr, step_cmd in failed_step_rows:
            fail_lines.append(f"[ci-gate-summary] failed_step_detail={step_id} rc=1 cmd={step_cmd}")
            fail_lines.append(
                f"[ci-gate-summary] failed_step_logs={step_id} stdout={step_stdout} stderr={step_stderr}"
            )
        fail_lines.extend(
            [
                f"[ci-gate-summary] report_index={index_path}",
                f"[ci-gate-summary] summary_line={summary_path}",
                f"[ci-gate-summary] ci_gate_result={result_path}",
            ]
        )
        if broken_summary:
            fail_lines[1] = "[ci-gate-summary] failed_steps=unknown_step_only"
            fail_lines = [line for line in fail_lines if "failed_step_detail=oi405_406_close" not in line]
        write_text(summary_report_path, "\n".join(fail_lines))
    write_json(
        result_path,
        {
            "schema": "ddn.ci.gate_result.v1",
            "status": status,
            "ok": status == "pass",
            "reason": reason,
            "failed_steps": failed_step_total,
            "aggregate_status": status,
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
            **AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_FIXTURE,
            **AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_FIXTURE,
        },
    )
    digest = ["step failed: seamgrim_ci_gate", "pack failed: dotbogi_write_forbidden"] if with_digest else []
    age5_child_status = {
        "age5_combined_heavy_full_real_status": "skipped" if status == "pass" else "pass",
        "age5_combined_heavy_runtime_helper_negative_status": "skipped" if status == "pass" else "fail",
        "age5_combined_heavy_group_id_summary_negative_status": "skipped" if status == "pass" else "fail",
    }
    age5_child_summary_default_transport = build_age5_combined_heavy_child_summary_default_text_transport_fields()
    write_json(
        aggregate_path,
        {
            "schema": "ddn.ci.aggregate_report.v1",
            "overall_ok": status == "pass",
            "age4": {
                "ok": status == "pass",
                "failed_criteria": [] if status == "pass" else ["age4_close_pending"],
                "proof_artifact_ok": status == "pass",
                "proof_artifact_failed_criteria": [] if status == "pass" else ["proof_runtime_error_statehash_preserved"],
                "proof_artifact_failed_preview": "-" if status == "pass" else "proof_runtime_error_statehash_preserved",
                "proof_artifact_summary_hash": "sha256:age4-proof-pass" if status == "pass" else "sha256:age4-proof-fail",
            },
            "age5": {
                **dict(age5_child_status),
                **age5_child_summary_default_transport,
                AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
                AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY: {"age5_close_digest_selftest_ok": "0"},
                AGE5_POLICY_REPORT_PATH_KEY: str(policy_report_path),
                AGE5_POLICY_REPORT_EXISTS_KEY: True,
                AGE5_POLICY_TEXT_PATH_KEY: str(policy_text_path),
                AGE5_POLICY_TEXT_EXISTS_KEY: True,
                AGE5_POLICY_SUMMARY_PATH_KEY: str(policy_summary_path),
                AGE5_POLICY_SUMMARY_EXISTS_KEY: True,
                AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY: AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
                AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY: age5_policy_age4_proof_snapshot_text,
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY: (
                    "-" if status == "pass" else "policy_summary_origin_trace_contract_mismatch"
                ),
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY: (
                    "-" if status == "pass" else "BROKEN"
                ),
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY: (
                    "-" if status == "pass" else "issue=policy_summary_origin_trace_contract_mismatch|source=BROKEN"
                ),
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY: (
                    build_age5_combined_heavy_policy_origin_trace_contract_compact_failure_reason(
                        "-" if status == "pass" else "policy_summary_origin_trace_contract_mismatch",
                        "-" if status == "pass" else "BROKEN",
                    )
                ),
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY: (
                    "ok" if status == "pass" else "mismatch"
                ),
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY: status == "pass",
                AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY: build_age5_combined_heavy_policy_origin_trace_text(
                    policy_origin_trace
                ),
                AGE5_POLICY_ORIGIN_TRACE_KEY: dict(policy_origin_trace),
            },
            "failure_digest": digest,
        },
    )
    write_json(
        profile_matrix_path,
        {
            "schema": "ddn.ci.profile_matrix_gate_selftest.v1",
            "status": "pass",
            "ok": True,
            "selected_real_profiles": ["core_lang", "full", "seamgrim"],
            "skipped_real_profiles": [],
            "total_elapsed_ms": 666,
            "aggregate_summary_sanity_ok": True,
            "aggregate_summary_sanity_checked_profiles": ["core_lang", "full", "seamgrim"],
            "aggregate_summary_sanity_failed_profiles": [],
            "aggregate_summary_sanity_skipped_profiles": [],
            "aggregate_summary_sanity_by_profile": {
                name: {
                    "expected_present": True,
                    "present": True,
                    "status": "pass",
                    "expected_profile": str(expected_profile_matrix_aggregate_summary_contract(name)["expected_profile"]),
                    "expected_sync_profile": str(
                        expected_profile_matrix_aggregate_summary_contract(name)["expected_sync_profile"]
                    ),
                    "profile": name,
                    "sync_profile": name,
                    "expected_values": dict(expected_profile_matrix_aggregate_summary_contract(name)["values"]),
                    "values": dict(expected_profile_matrix_aggregate_summary_contract(name)["values"]),
                    "missing_keys": [],
                    "mismatched_keys": [],
                    "profile_ok": True,
                    "sync_profile_ok": True,
                    "values_ok": True,
                    "gate_marker_expected": bool(
                        expected_profile_matrix_aggregate_summary_contract(name)["gate_marker_expected"]
                    ),
                    "gate_marker_present": bool(
                        expected_profile_matrix_aggregate_summary_contract(name)["gate_marker_expected"]
                    ),
                    "gate_marker_ok": True,
                    "ok": True,
                }
                for name in ("core_lang", "full", "seamgrim")
            },
            "real_profiles": {
                "core_lang": {"total_elapsed_ms": 111},
                "full": {"total_elapsed_ms": 222},
                "seamgrim": {"total_elapsed_ms": 333},
            },
        },
    )
    write_json(
        index_path,
        {
            "schema": "ddn.ci.aggregate_gate.index.v1",
            "report_prefix": prefix,
            "reports": {
                "summary": str(summary_report_path),
                "summary_line": str(summary_path),
                "ci_gate_result_json": str(result_path),
                "aggregate": str(aggregate_path),
                "ci_profile_matrix_gate_selftest": str(profile_matrix_path),
                "age4_close": str(report_dir / f"{prefix}.age4_close_report.detjson"),
            },
            "steps": [
                {
                    "name": "age5_close_digest_selftest",
                    "ok": True,
                    "stdout_log_path": str(report_dir / f"{prefix}.age5_digest.stdout.txt"),
                    "stderr_log_path": str(report_dir / f"{prefix}.age5_digest.stderr.txt"),
                },
            ]
            + [
                {
                    "name": step_id,
                    "ok": status == "pass",
                    "stdout_log_path": str(step_stdout),
                    "stderr_log_path": str(step_stderr),
                }
                for step_id, step_stdout, step_stderr, _ in failed_step_rows
            ],
        },
    )


def ensure_contains(text: str, needle: str) -> bool:
    return needle in text


def main() -> int:
    summary_verify_codes = set(SUMMARY_VERIFY_CODES.values())
    expected_default_transport = build_age5_combined_heavy_child_summary_default_text_transport_fields()
    expected_policy_age4_proof_snapshot_text = build_age4_proof_snapshot_text(
        build_age4_proof_snapshot()
    )
    with tempfile.TemporaryDirectory(prefix="ci_final_emit_check_") as tmp:
        report_dir = Path(tmp)
        brief_tpl = report_dir / "__PREFIX__.ci_fail_brief.txt"
        triage_tpl = report_dir / "__PREFIX__.ci_fail_triage.detjson"
        pass_policy_summary_path = report_dir / "passcase.age5_combined_heavy_policy_summary.txt"
        fail_policy_summary_path = report_dir / "failcase.age5_combined_heavy_policy_summary.txt"
        pass_policy_origin_trace = build_age5_combined_heavy_policy_origin_trace(
            report_path=str(report_dir / "passcase.age5_combined_heavy_policy.detjson"),
            report_exists=True,
            text_path=str(report_dir / "passcase.age5_combined_heavy_policy.txt"),
            text_exists=True,
            summary_path=str(pass_policy_summary_path),
            summary_exists=True,
        )
        pass_policy_origin_trace_text = build_age5_combined_heavy_policy_origin_trace_text(
            pass_policy_origin_trace
        )
        fail_policy_origin_trace = build_age5_combined_heavy_policy_origin_trace(
            report_path=str(report_dir / "failcase.age5_combined_heavy_policy.detjson"),
            report_exists=True,
            text_path=str(report_dir / "failcase.age5_combined_heavy_policy.txt"),
            text_exists=True,
            summary_path=str(fail_policy_summary_path),
            summary_exists=True,
        )
        fail_policy_origin_trace_text = build_age5_combined_heavy_policy_origin_trace_text(
            fail_policy_origin_trace
        )

        build_case(report_dir, "passcase", status="pass", reason="-", with_digest=False)
        proc_pass = run_emit(
            report_dir,
            "--prefix",
            "passcase",
            "--print-artifacts",
            "--print-failure-digest",
            "5",
            "--failure-brief-out",
            str(brief_tpl),
            "--triage-json-out",
            str(triage_tpl),
            "--require-final-line",
        )
        if proc_pass.returncode != 0:
            return fail(f"passcase returncode={proc_pass.returncode}")
        if not ensure_contains(proc_pass.stdout, "[ci-final] ci_gate_result_status=pass"):
            return fail("passcase final line missing")
        if not ensure_contains(proc_pass.stdout, "profile_matrix_total_elapsed_ms=666"):
            return fail("passcase final line profile_matrix_total_elapsed_ms missing")
        if not ensure_contains(proc_pass.stdout, "selected_real_profiles=core_lang,full,seamgrim"):
            return fail("passcase final line selected_real_profiles missing")
        if not ensure_contains(proc_pass.stdout, "profile_matrix_status=pass"):
            return fail("passcase final line profile_matrix_status missing")
        if not ensure_contains(proc_pass.stdout, "profile_matrix_ok=1"):
            return fail("passcase final line profile_matrix_ok missing")
        if not ensure_contains(proc_pass.stdout, "age5_close_digest_selftest_ok=1"):
            return fail("passcase final line age5 digest selftest status missing")
        if not ensure_contains(proc_pass.stdout, "age5_w107_active=54"):
            return fail("passcase final line age5_w107_active missing")
        if not ensure_contains(proc_pass.stdout, "age5_w107_progress=1"):
            return fail("passcase final line age5_w107_progress missing")
        if not ensure_contains(proc_pass.stdout, "age5_w107_contract_completed=8"):
            return fail("passcase final line age5_w107_contract_completed missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_w107_contract_checks_text=golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index",
        ):
            return fail("passcase final line age5_w107_contract_checks_text missing")
        if not ensure_contains(proc_pass.stdout, "age5_w107_contract_progress=1"):
            return fail("passcase final line age5_w107_contract_progress missing")
        if not ensure_contains(proc_pass.stdout, "age5_age1_immediate_proof_operation_contract_completed=5"):
            return fail("passcase final line age5_age1_immediate_proof_operation_contract_completed missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_age1_immediate_proof_operation_contract_checks_text=operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family",
        ):
            return fail("passcase final line age5_age1_immediate_proof_operation_contract_checks_text missing")
        if not ensure_contains(proc_pass.stdout, "age5_age1_immediate_proof_operation_contract_progress=1"):
            return fail("passcase final line age5_age1_immediate_proof_operation_contract_progress missing")
        if not ensure_contains(proc_pass.stdout, "age5_proof_certificate_v1_consumer_contract_completed=5"):
            return fail("passcase final line age5_proof_certificate_v1_consumer_contract_completed missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_proof_certificate_v1_consumer_contract_checks_text=signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract",
        ):
            return fail("passcase final line age5_proof_certificate_v1_consumer_contract_checks_text missing")
        if (
            "age5_proof_certificate_v1_verify_report_digest_contract_checks_text=verify_report_digest_contract"
            not in proc_pass.stdout
        ):
            return fail("passcase final line age5_proof_certificate_v1_verify_report_digest_contract_checks_text missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_proof_certificate_v1_family_contract_checks_text=signed_contract,consumer_contract,promotion,family",
        ):
            return fail("passcase final line age5_proof_certificate_v1_family_contract_checks_text missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_proof_certificate_family_contract_checks_text=artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family",
        ):
            return fail("passcase final line age5_proof_certificate_family_contract_checks_text missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_proof_family_contract_checks_text=proof_operation_family,proof_certificate_family,proof_family",
        ):
            return fail("passcase final line age5_proof_family_contract_checks_text missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_lang_surface_family_contract_checks_text=proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family",
        ):
            return fail("passcase final line age5_lang_surface_family_contract_checks_text missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_lang_runtime_family_contract_checks_text=lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family",
        ):
            return fail("passcase final line age5_lang_runtime_family_contract_checks_text missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_lang_runtime_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("passcase final line age5_lang_runtime_family_transport_contract_checks_text missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_gate0_family_contract_checks_text=gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family",
        ):
            return fail("passcase final line age5_gate0_family_contract_checks_text missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_gate0_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("passcase final line age5_gate0_family_transport_contract_checks_text missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_gate0_transport_family_contract_checks_text=lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family",
        ):
            return fail("passcase final line age5_gate0_transport_family_contract_checks_text missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_gate0_transport_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("passcase final line age5_gate0_transport_family_transport_contract_checks_text missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_gate0_runtime_family_transport_contract_checks_text=family_contract",
        ):
            return fail("passcase final line age5_gate0_runtime_family_transport_contract_checks_text missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_lang_surface_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("passcase final line age5_lang_surface_family_transport_contract_checks_text missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_proof_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("passcase final line age5_proof_family_transport_contract_checks_text missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_bogae_alias_family_contract_checks_text=shape_alias_contract,alias_family,alias_viewer_family",
        ):
            return fail("passcase final line age5_bogae_alias_family_contract_checks_text missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_bogae_alias_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("passcase final line age5_bogae_alias_family_transport_contract_checks_text missing")
        if not ensure_contains(
            proc_pass.stdout,
            "age5_proof_certificate_v1_consumer_contract_last_completed_probe=signed_contract",
        ):
            return fail("passcase final line age5_proof_certificate_v1_consumer_contract_last_completed_probe missing")
        if not ensure_contains(proc_pass.stdout, "age5_proof_certificate_v1_consumer_contract_progress=1"):
            return fail("passcase final line age5_proof_certificate_v1_consumer_contract_progress missing")
        if not ensure_contains(proc_pass.stdout, AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT):
            return fail("passcase final line age5 digest selftest default field missing")
        if not ensure_contains(proc_pass.stdout, AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_FRAGMENT):
            return fail("passcase final line age5 digest selftest default contract missing")
        if not ensure_contains(proc_pass.stdout, f"{AGE4_PROOF_OK_KEY}=1"):
            return fail("passcase final line age4 proof ok missing")
        if not ensure_contains(proc_pass.stdout, f"{AGE4_PROOF_FAILED_CRITERIA_KEY}=0"):
            return fail("passcase final line age4 proof failed criteria missing")
        if not ensure_contains(proc_pass.stdout, f"{AGE4_PROOF_SUMMARY_HASH_KEY}=sha256:age4-proof-pass"):
            return fail("passcase final line age4 proof summary hash missing")
        if not ensure_contains(proc_pass.stdout, "age5_combined_heavy_full_real_status=skipped"):
            return fail("passcase final line age5 full_real status missing")
        if not ensure_contains(proc_pass.stdout, "age5_combined_heavy_runtime_helper_negative_status=skipped"):
            return fail("passcase final line age5 runtime_helper_negative status missing")
        if not ensure_contains(proc_pass.stdout, "age5_combined_heavy_group_id_summary_negative_status=skipped"):
            return fail("passcase final line age5 group_id_summary_negative status missing")
        if not ensure_contains(
            proc_pass.stdout,
            "ci_sanity_age5_combined_heavy_child_summary_default_fields="
            + expected_default_transport["ci_sanity_age5_combined_heavy_child_summary_default_fields"],
        ):
            return fail("passcase final line child_summary_default transport missing")
        if not ensure_contains(
            proc_pass.stdout,
            "ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields="
            + expected_default_transport["ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"],
        ):
            return fail("passcase final line sync child_summary_default transport missing")
        if not ensure_contains(
            proc_pass.stdout,
            f"{AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}=age5_close_digest_selftest_ok=0",
        ):
            return fail("passcase final line age5 policy default text missing")
        if not ensure_contains(
            proc_pass.stdout,
            f"{AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY}={AGE4_PROOF_SNAPSHOT_FIELDS_TEXT}",
        ):
            return fail("passcase final line age5 policy age4 proof snapshot fields missing")
        if not ensure_contains(
            proc_pass.stdout,
            f"{AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY}={expected_policy_age4_proof_snapshot_text}",
        ):
            return fail("passcase final line age5 policy age4 proof snapshot text missing")
        if not ensure_contains(
            proc_pass.stdout,
            f'{AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY}={{"age5_close_digest_selftest_ok":"0"}}',
        ):
            return fail("passcase final line age5 policy default field missing")
        if not ensure_contains(proc_pass.stdout, f"{AGE5_POLICY_REPORT_EXISTS_KEY}=1"):
            return fail("passcase final line age5 policy report exists missing")
        if not ensure_contains(proc_pass.stdout, f"{AGE5_POLICY_TEXT_EXISTS_KEY}=1"):
            return fail("passcase final line age5 policy text exists missing")
        if not ensure_contains(proc_pass.stdout, f"{AGE5_POLICY_SUMMARY_PATH_KEY}={pass_policy_summary_path}"):
            return fail("passcase final line age5 policy summary path missing")
        if not ensure_contains(proc_pass.stdout, f"{AGE5_POLICY_SUMMARY_EXISTS_KEY}=1"):
            return fail("passcase final line age5 policy summary exists missing")
        if not ensure_contains(
            proc_pass.stdout,
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}=-",
        ):
            return fail("passcase final line age5 policy origin trace contract issue missing")
        if not ensure_contains(
            proc_pass.stdout,
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}=-",
        ):
            return fail("passcase final line age5 policy origin trace source issue missing")
        if not ensure_contains(
            proc_pass.stdout,
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}=-",
        ):
            return fail("passcase final line age5 policy origin trace compact reason missing")
        if not ensure_contains(proc_pass.stdout, f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}=ok"):
            return fail("passcase final line age5 policy origin trace contract status missing")
        if not ensure_contains(proc_pass.stdout, f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}=1"):
            return fail("passcase final line age5 policy origin trace contract ok missing")
        if not ensure_contains(proc_pass.stdout, f"{AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY}={pass_policy_origin_trace_text}"):
            return fail("passcase final line age5 policy origin trace text missing")
        if not ensure_contains(
            proc_pass.stdout,
            f"{AGE5_POLICY_ORIGIN_TRACE_KEY}="
            + json.dumps(pass_policy_origin_trace, ensure_ascii=False, sort_keys=True, separators=(",", ":")),
        ):
            return fail("passcase final line age5 policy origin trace missing")
        if not ensure_contains(proc_pass.stdout, "[ci-artifact] key=summary exists=1"):
            return fail("passcase summary artifact line missing")
        if ensure_contains(proc_pass.stdout, "[ci-fail]"):
            return fail("passcase must not print ci-fail block")
        pass_brief = report_dir / "passcase.ci_fail_brief.txt"
        if not pass_brief.exists():
            return fail("passcase brief file missing")
        pass_brief_line = pass_brief.read_text(encoding="utf-8").strip()
        if not ensure_contains(pass_brief_line, "status=pass"):
            return fail("passcase brief status missing")
        if not ensure_contains(pass_brief_line, "profile_matrix_total_elapsed_ms=666"):
            return fail("passcase brief profile_matrix_total_elapsed_ms missing")
        if not ensure_contains(pass_brief_line, 'profile_matrix_selected_real_profiles="core_lang,full,seamgrim"'):
            return fail("passcase brief profile_matrix_selected_real_profiles missing")
        if not ensure_contains(pass_brief_line, "age5_close_digest_selftest_ok=1"):
            return fail("passcase brief age5 digest selftest status missing")
        if not ensure_contains(pass_brief_line, AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT):
            return fail("passcase brief age5 digest selftest default field missing")
        if not ensure_contains(pass_brief_line, AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_FRAGMENT):
            return fail("passcase brief age5 digest selftest default contract missing")
        if not ensure_contains(pass_brief_line, f"{AGE4_PROOF_OK_KEY}=1"):
            return fail("passcase brief age4 proof ok missing")
        if not ensure_contains(pass_brief_line, f"{AGE4_PROOF_FAILED_CRITERIA_KEY}=0"):
            return fail("passcase brief age4 proof failed criteria missing")
        if not ensure_contains(pass_brief_line, f"{AGE4_PROOF_SUMMARY_HASH_KEY}=sha256:age4-proof-pass"):
            return fail("passcase brief age4 proof summary hash missing")
        if not ensure_contains(pass_brief_line, "age5_w107_active=54"):
            return fail("passcase brief age5_w107_active missing")
        if not ensure_contains(pass_brief_line, "age5_w107_progress=1"):
            return fail("passcase brief age5_w107_progress missing")
        if not ensure_contains(pass_brief_line, "age5_w107_contract_completed=8"):
            return fail("passcase brief age5_w107_contract_completed missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_w107_contract_checks_text=golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index",
        ):
            return fail("passcase brief age5_w107_contract_checks_text missing")
        if not ensure_contains(pass_brief_line, "age5_w107_contract_last_completed_probe=report_index"):
            return fail("passcase brief age5_w107_contract_last_completed_probe missing")
        if not ensure_contains(pass_brief_line, "age5_age1_immediate_proof_operation_contract_completed=5"):
            return fail("passcase brief age5_age1_immediate_proof_operation_contract_completed missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_age1_immediate_proof_operation_contract_checks_text=operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family",
        ):
            return fail("passcase brief age5_age1_immediate_proof_operation_contract_checks_text missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_age1_immediate_proof_operation_contract_last_completed_probe=proof_operation_family",
        ):
            return fail("passcase brief age5_age1_immediate_proof_operation_contract_last_completed_probe missing")
        if not ensure_contains(pass_brief_line, "age5_proof_certificate_v1_consumer_contract_completed=5"):
            return fail("passcase brief age5_proof_certificate_v1_consumer_contract_completed missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_proof_certificate_v1_consumer_contract_checks_text=signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract",
        ):
            return fail("passcase brief age5_proof_certificate_v1_consumer_contract_checks_text missing")
        if (
            "age5_proof_certificate_v1_verify_report_digest_contract_checks_text=verify_report_digest_contract"
            not in pass_brief.read_text(encoding="utf-8")
        ):
            return fail("passcase brief age5_proof_certificate_v1_verify_report_digest_contract_checks_text missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_proof_certificate_v1_family_contract_checks_text=signed_contract,consumer_contract,promotion,family",
        ):
            return fail("passcase brief age5_proof_certificate_v1_family_contract_checks_text missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_proof_certificate_family_contract_checks_text=artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family",
        ):
            return fail("passcase brief age5_proof_certificate_family_contract_checks_text missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_proof_family_contract_checks_text=proof_operation_family,proof_certificate_family,proof_family",
        ):
            return fail("passcase brief age5_proof_family_contract_checks_text missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_lang_surface_family_contract_checks_text=proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family",
        ):
            return fail("passcase brief age5_lang_surface_family_contract_checks_text missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_lang_runtime_family_contract_checks_text=lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family",
        ):
            return fail("passcase brief age5_lang_runtime_family_contract_checks_text missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_lang_runtime_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("passcase brief age5_lang_runtime_family_transport_contract_checks_text missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_gate0_family_contract_checks_text=gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family",
        ):
            return fail("passcase brief age5_gate0_family_contract_checks_text missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_gate0_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("passcase brief age5_gate0_family_transport_contract_checks_text missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_gate0_transport_family_contract_checks_text=lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family",
        ):
            return fail("passcase brief age5_gate0_transport_family_contract_checks_text missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_gate0_transport_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("passcase brief age5_gate0_transport_family_transport_contract_checks_text missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_gate0_runtime_family_transport_contract_checks_text=family_contract",
        ):
            return fail("passcase brief age5_gate0_runtime_family_transport_contract_checks_text missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_lang_surface_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("passcase brief age5_lang_surface_family_transport_contract_checks_text missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_proof_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("passcase brief age5_proof_family_transport_contract_checks_text missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_bogae_alias_family_contract_checks_text=shape_alias_contract,alias_family,alias_viewer_family",
        ):
            return fail("passcase brief age5_bogae_alias_family_contract_checks_text missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_bogae_alias_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("passcase brief age5_bogae_alias_family_transport_contract_checks_text missing")
        if not ensure_contains(
            pass_brief_line,
            "age5_proof_certificate_v1_consumer_contract_last_completed_probe=signed_contract",
        ):
            return fail("passcase brief age5_proof_certificate_v1_consumer_contract_last_completed_probe missing")
        if not ensure_contains(pass_brief_line, "age5_proof_certificate_v1_consumer_contract_progress=1"):
            return fail("passcase brief age5_proof_certificate_v1_consumer_contract_progress missing")
        if not ensure_contains(pass_brief_line, "age5_combined_heavy_full_real_status=skipped"):
            return fail("passcase brief age5 full_real status missing")
        if not ensure_contains(pass_brief_line, "age5_combined_heavy_runtime_helper_negative_status=skipped"):
            return fail("passcase brief age5 runtime_helper_negative status missing")
        if not ensure_contains(pass_brief_line, "age5_combined_heavy_group_id_summary_negative_status=skipped"):
            return fail("passcase brief age5 group_id_summary_negative status missing")
        if not ensure_contains(
            pass_brief_line,
            "ci_sanity_age5_combined_heavy_child_summary_default_fields="
            + expected_default_transport["ci_sanity_age5_combined_heavy_child_summary_default_fields"],
        ):
            return fail("passcase brief child_summary_default transport missing")
        if not ensure_contains(
            pass_brief_line,
            "ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields="
            + expected_default_transport["ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"],
        ):
            return fail("passcase brief sync child_summary_default transport missing")
        if not ensure_contains(
            pass_brief_line,
            f"{AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}=age5_close_digest_selftest_ok=0",
        ):
            return fail("passcase brief age5 policy default text missing")
        if not ensure_contains(
            pass_brief_line,
            f'{AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY}={{"age5_close_digest_selftest_ok":"0"}}',
        ):
            return fail("passcase brief age5 policy default field missing")
        if not ensure_contains(
            pass_brief_line,
            f"{AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY}={AGE4_PROOF_SNAPSHOT_FIELDS_TEXT}",
        ):
            return fail("passcase brief age5 policy age4 proof snapshot fields missing")
        if not ensure_contains(
            pass_brief_line,
            f"{AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY}={expected_policy_age4_proof_snapshot_text}",
        ):
            return fail("passcase brief age5 policy age4 proof snapshot text missing")
        if not ensure_contains(pass_brief_line, f"{AGE5_POLICY_REPORT_EXISTS_KEY}=1"):
            return fail("passcase brief age5 policy report exists missing")
        if not ensure_contains(pass_brief_line, f"{AGE5_POLICY_TEXT_EXISTS_KEY}=1"):
            return fail("passcase brief age5 policy text exists missing")
        if not ensure_contains(pass_brief_line, f"{AGE5_POLICY_SUMMARY_PATH_KEY}={pass_policy_summary_path}"):
            return fail("passcase brief age5 policy summary path missing")
        if not ensure_contains(pass_brief_line, f"{AGE5_POLICY_SUMMARY_EXISTS_KEY}=1"):
            return fail("passcase brief age5 policy summary exists missing")
        if not ensure_contains(
            pass_brief_line,
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}=-",
        ):
            return fail("passcase brief age5 policy origin trace contract issue missing")
        if not ensure_contains(
            pass_brief_line,
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}=-",
        ):
            return fail("passcase brief age5 policy origin trace source issue missing")
        if not ensure_contains(
            pass_brief_line,
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}=-",
        ):
            return fail("passcase brief age5 policy origin trace compact reason missing")
        if not ensure_contains(pass_brief_line, f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}=ok"):
            return fail("passcase brief age5 policy origin trace contract status missing")
        if not ensure_contains(pass_brief_line, f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}=1"):
            return fail("passcase brief age5 policy origin trace contract ok missing")
        if not ensure_contains(pass_brief_line, f"{AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY}={pass_policy_origin_trace_text}"):
            return fail("passcase brief age5 policy origin trace text missing")
        if not ensure_contains(
            pass_brief_line,
            f"{AGE5_POLICY_ORIGIN_TRACE_KEY}="
            + json.dumps(pass_policy_origin_trace, ensure_ascii=False, sort_keys=True, separators=(",", ":")),
        ):
            return fail("passcase brief age5 policy origin trace missing")
        pass_triage = report_dir / "passcase.ci_fail_triage.detjson"
        if not pass_triage.exists():
            return fail("passcase triage file missing")
        pass_triage_doc = json.loads(pass_triage.read_text(encoding="utf-8"))
        if str(pass_triage_doc.get("schema", "")) != "ddn.ci.fail_triage.v1":
            return fail("passcase triage schema mismatch")
        if str(pass_triage_doc.get("status", "")) != "pass":
            return fail("passcase triage status mismatch")
        if int(pass_triage_doc.get("failed_step_detail_rows_count", -1)) != 0:
            return fail("passcase triage failed_step_detail_rows_count mismatch")
        if int(pass_triage_doc.get("failed_step_logs_rows_count", -1)) != 0:
            return fail("passcase triage failed_step_logs_rows_count mismatch")
        if list(pass_triage_doc.get("failed_step_detail_order", [])) != []:
            return fail("passcase triage failed_step_detail_order mismatch")
        if list(pass_triage_doc.get("failed_step_logs_order", [])) != []:
            return fail("passcase triage failed_step_logs_order mismatch")
        if int(pass_triage_doc.get("summary_verify_issues_count", -1)) != 0:
            return fail("passcase triage summary_verify_issues_count mismatch")
        if str(pass_triage_doc.get("summary_verify_top_issue", "")).strip() != "-":
            return fail("passcase triage summary_verify_top_issue mismatch")
        pass_profile_matrix = pass_triage_doc.get("profile_matrix_selftest")
        if not isinstance(pass_profile_matrix, dict):
            return fail("passcase triage profile_matrix_selftest missing")
        if int(pass_profile_matrix.get("total_elapsed_ms", -1)) != 666:
            return fail("passcase triage profile_matrix_selftest total_elapsed_ms mismatch")
        if str(pass_profile_matrix.get("step_timeout_defaults_text", "")).strip() != PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT:
            return fail("passcase triage profile_matrix_selftest timeout defaults mismatch")
        if dict(pass_profile_matrix.get("step_timeout_defaults_sec", {})) != dict(PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC):
            return fail("passcase triage profile_matrix_selftest timeout defaults sec mismatch")
        if dict(pass_profile_matrix.get("step_timeout_env_keys", {})) != dict(PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS):
            return fail("passcase triage profile_matrix_selftest timeout env keys mismatch")
        if bool(pass_profile_matrix.get("aggregate_summary_sanity_ok", False)) is not True:
            return fail("passcase triage profile_matrix aggregate_summary_sanity_ok mismatch")
        if int(pass_triage_doc.get(AGE4_PROOF_OK_KEY, 0)) != 1:
            return fail("passcase triage age4 proof ok mismatch")
        if int(pass_triage_doc.get(AGE4_PROOF_FAILED_CRITERIA_KEY, -1)) != 0:
            return fail("passcase triage age4 proof failed criteria mismatch")
        if str(pass_triage_doc.get(AGE4_PROOF_SUMMARY_HASH_KEY, "")).strip() != "sha256:age4-proof-pass":
            return fail("passcase triage age4 proof summary hash mismatch")
        if str(pass_triage_doc.get("age5_close_digest_selftest_ok", "")).strip() != "1":
            return fail("passcase triage age5 digest selftest status mismatch")
        if str(pass_triage_doc.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip() != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
            return fail("passcase triage age5 digest selftest default contract text mismatch")
        if dict(pass_triage_doc.get("combined_digest_selftest_default_field", {})) != {"age5_close_digest_selftest_ok": "0"}:
            return fail("passcase triage age5 digest selftest default contract dict mismatch")
        if str(pass_triage_doc.get("age5_combined_heavy_full_real_status", "")).strip() != "skipped":
            return fail("passcase triage age5 full_real status mismatch")
        if str(pass_triage_doc.get("age5_combined_heavy_runtime_helper_negative_status", "")).strip() != "skipped":
            return fail("passcase triage age5 runtime_helper_negative status mismatch")
        if str(pass_triage_doc.get("age5_combined_heavy_group_id_summary_negative_status", "")).strip() != "skipped":
            return fail("passcase triage age5 group_id_summary_negative status mismatch")
        if (
            str(pass_triage_doc.get("ci_sanity_age5_combined_heavy_child_summary_default_fields", "")).strip()
            != expected_default_transport["ci_sanity_age5_combined_heavy_child_summary_default_fields"]
        ):
            return fail("passcase triage child_summary_default transport mismatch")
        if (
            str(
                pass_triage_doc.get(
                    "ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields", ""
                )
            ).strip()
            != expected_default_transport[
                "ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"
            ]
        ):
            return fail("passcase triage sync child_summary_default transport mismatch")
        if str(pass_triage_doc.get(AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip() != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
            return fail("passcase triage age5 policy default text mismatch")
        if dict(pass_triage_doc.get(AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY, {})) != {"age5_close_digest_selftest_ok": "0"}:
            return fail("passcase triage age5 policy default field mismatch")
        if str(pass_triage_doc.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY, "")).strip() != AGE4_PROOF_SNAPSHOT_FIELDS_TEXT:
            return fail("passcase triage age5 policy age4 proof snapshot fields mismatch")
        if str(pass_triage_doc.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY, "")).strip() != expected_policy_age4_proof_snapshot_text:
            return fail("passcase triage age5 policy age4 proof snapshot text mismatch")
        if int(pass_triage_doc.get(AGE5_POLICY_REPORT_EXISTS_KEY, 0)) != 1:
            return fail("passcase triage age5 policy report exists mismatch")
        if int(pass_triage_doc.get(AGE5_POLICY_TEXT_EXISTS_KEY, 0)) != 1:
            return fail("passcase triage age5 policy text exists mismatch")
        if str(pass_triage_doc.get(AGE5_POLICY_SUMMARY_PATH_KEY, "")).strip() != str(pass_policy_summary_path):
            return fail("passcase triage age5 policy summary path mismatch")
        if int(pass_triage_doc.get(AGE5_POLICY_SUMMARY_EXISTS_KEY, 0)) != 1:
            return fail("passcase triage age5 policy summary exists mismatch")
        if str(pass_triage_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY, "")).strip() != "-":
            return fail("passcase triage age5 policy origin trace contract issue mismatch")
        if str(pass_triage_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY, "")).strip() != "-":
            return fail("passcase triage age5 policy origin trace source issue mismatch")
        if str(pass_triage_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY, "")).strip() != "-":
            return fail("passcase triage age5 policy origin trace compact reason mismatch")
        if (
            str(
                pass_triage_doc.get(
                    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY, ""
                )
            ).strip()
            != "-"
        ):
            return fail("passcase triage age5 policy origin trace compact failure reason mismatch")
        if str(pass_triage_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY, "")).strip() != "ok":
            return fail("passcase triage age5 policy origin trace contract status mismatch")
        if int(pass_triage_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY, 0)) != 1:
            return fail("passcase triage age5 policy origin trace contract ok mismatch")
        if str(pass_triage_doc.get(AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY, "")).strip() != pass_policy_origin_trace_text:
            return fail("passcase triage age5 policy origin trace text mismatch")
        if dict(pass_triage_doc.get(AGE5_POLICY_ORIGIN_TRACE_KEY, {})) != pass_policy_origin_trace:
            return fail("passcase triage age5 policy origin trace mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks", "")).strip() != "5":
            return fail("passcase triage age5_proof_certificate_v1_consumer_contract_completed mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text", "")).strip() != "signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract":
            return fail("passcase triage age5_proof_certificate_v1_consumer_contract_checks_text mismatch")
        if (
            str(
                pass_triage_doc.get(
                    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text",
                    "",
                )
            ).strip()
            != "verify_report_digest_contract"
        ):
            return fail("passcase triage age5_proof_certificate_v1_verify_report_digest_contract_checks_text mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks", "")).strip() != "4":
            return fail("passcase triage age5_proof_certificate_v1_family_contract_completed mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text", "")).strip() != "signed_contract,consumer_contract,promotion,family":
            return fail("passcase triage age5_proof_certificate_v1_family_contract_checks_text mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe", "")).strip() != "family":
            return fail("passcase triage age5_proof_certificate_v1_family_contract_last_completed_probe mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present", "")).strip() != "1":
            return fail("passcase triage age5_proof_certificate_v1_family_contract_progress mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_certificate_family_contract_selftest_completed_checks", "")).strip() != "3":
            return fail("passcase triage age5_proof_certificate_family_contract_completed mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_certificate_family_contract_selftest_checks_text", "")).strip() != "artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family":
            return fail("passcase triage age5_proof_certificate_family_contract_checks_text mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe", "")).strip() != "proof_certificate_family":
            return fail("passcase triage age5_proof_certificate_family_contract_last_completed_probe mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_certificate_family_contract_selftest_progress_present", "")).strip() != "1":
            return fail("passcase triage age5_proof_certificate_family_contract_progress mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_family_contract_selftest_completed_checks", "")).strip() != "3":
            return fail("passcase triage age5_proof_family_contract_completed mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_family_contract_selftest_checks_text", "")).strip() != "proof_operation_family,proof_certificate_family,proof_family":
            return fail("passcase triage age5_proof_family_contract_checks_text mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_family_contract_selftest_last_completed_probe", "")).strip() != "proof_family":
            return fail("passcase triage age5_proof_family_contract_last_completed_probe mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_family_contract_selftest_progress_present", "")).strip() != "1":
            return fail("passcase triage age5_proof_family_contract_progress mismatch")
        if str(pass_triage_doc.get("age5_full_real_lang_surface_family_contract_selftest_completed_checks", "")).strip() != "4":
            return fail("passcase triage age5_lang_surface_family_contract_completed mismatch")
        if str(pass_triage_doc.get("age5_full_real_lang_surface_family_contract_selftest_checks_text", "")).strip() != "proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family":
            return fail("passcase triage age5_lang_surface_family_contract_checks_text mismatch")
        if str(pass_triage_doc.get("age5_full_real_lang_surface_family_contract_selftest_last_completed_probe", "")).strip() != "lang_surface_family":
            return fail("passcase triage age5_lang_surface_family_contract_last_completed_probe mismatch")
        if str(pass_triage_doc.get("age5_full_real_lang_surface_family_contract_selftest_progress_present", "")).strip() != "1":
            return fail("passcase triage age5_lang_surface_family_contract_progress mismatch")
        if str(pass_triage_doc.get("age5_full_real_lang_runtime_family_contract_selftest_completed_checks", "")).strip() != "5":
            return fail("passcase triage age5_lang_runtime_family_contract_completed mismatch")
        if str(pass_triage_doc.get("age5_full_real_lang_runtime_family_contract_selftest_checks_text", "")).strip() != "lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family":
            return fail("passcase triage age5_lang_runtime_family_contract_checks_text mismatch")
        if str(pass_triage_doc.get("age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe", "")).strip() != "lang_runtime_family":
            return fail("passcase triage age5_lang_runtime_family_contract_last_completed_probe mismatch")
        if str(pass_triage_doc.get("age5_full_real_lang_runtime_family_contract_selftest_progress_present", "")).strip() != "1":
            return fail("passcase triage age5_lang_runtime_family_contract_progress mismatch")
        if str(pass_triage_doc.get("age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks", "")).strip() != "9":
            return fail("passcase triage age5_lang_runtime_family_transport_contract_completed mismatch")
        if str(pass_triage_doc.get("age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index":
            return fail("passcase triage age5_lang_runtime_family_transport_contract_checks_text mismatch")
        if str(pass_triage_doc.get("age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe", "")).strip() != "report_index":
            return fail("passcase triage age5_lang_runtime_family_transport_contract_last_completed_probe mismatch")
        if str(pass_triage_doc.get("age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("passcase triage age5_lang_runtime_family_transport_contract_progress mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_family_contract_selftest_completed_checks", "")).strip() != "5":
            return fail("passcase triage age5_gate0_family_contract_completed mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_family_contract_selftest_checks_text", "")).strip() != "gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family":
            return fail("passcase triage age5_gate0_family_contract_checks_text mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_family_contract_selftest_last_completed_probe", "")).strip() != "gate0_family":
            return fail("passcase triage age5_gate0_family_contract_last_completed_probe mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_family_contract_selftest_progress_present", "")).strip() != "1":
            return fail("passcase triage age5_gate0_family_contract_progress mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_family_transport_contract_selftest_completed_checks", "")).strip() != "9":
            return fail("passcase triage age5_gate0_family_transport_contract_completed mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index":
            return fail("passcase triage age5_gate0_family_transport_contract_checks_text mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe", "")).strip() != "report_index":
            return fail("passcase triage age5_gate0_family_transport_contract_last_completed_probe mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_family_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("passcase triage age5_gate0_family_transport_contract_progress mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_transport_family_contract_selftest_completed_checks", "")).strip() != "4":
            return fail("passcase triage age5_gate0_transport_family_contract_completed mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_transport_family_contract_selftest_checks_text", "")).strip() != "lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family":
            return fail("passcase triage age5_gate0_transport_family_contract_checks_text mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe", "")).strip() != "gate0_transport_family":
            return fail("passcase triage age5_gate0_transport_family_contract_last_completed_probe mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_transport_family_contract_selftest_progress_present", "")).strip() != "1":
            return fail("passcase triage age5_gate0_transport_family_contract_progress mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks", "")).strip() != "9":
            return fail("passcase triage age5_gate0_transport_family_transport_contract_completed mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index":
            return fail("passcase triage age5_gate0_transport_family_transport_contract_checks_text mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe", "")).strip() != "report_index":
            return fail("passcase triage age5_gate0_transport_family_transport_contract_last_completed_probe mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("passcase triage age5_gate0_transport_family_transport_contract_progress mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks", "")).strip() != "1":
            return fail("passcase triage age5_gate0_runtime_family_transport_contract_completed mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract":
            return fail("passcase triage age5_gate0_runtime_family_transport_contract_checks_text mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe", "")).strip() != "family_contract":
            return fail("passcase triage age5_gate0_runtime_family_transport_contract_last_completed_probe mismatch")
        if str(pass_triage_doc.get("age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("passcase triage age5_gate0_runtime_family_transport_contract_progress mismatch")
        if str(pass_triage_doc.get("age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks", "")).strip() != "9":
            return fail("passcase triage age5_lang_surface_family_transport_contract_completed mismatch")
        if str(pass_triage_doc.get("age5_full_real_lang_surface_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index":
            return fail("passcase triage age5_lang_surface_family_transport_contract_checks_text mismatch")
        if str(pass_triage_doc.get("age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe", "")).strip() != "report_index":
            return fail("passcase triage age5_lang_surface_family_transport_contract_last_completed_probe mismatch")
        if str(pass_triage_doc.get("age5_full_real_lang_surface_family_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("passcase triage age5_lang_surface_family_transport_contract_progress mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_family_transport_contract_selftest_completed_checks", "")).strip() != "9":
            return fail("passcase triage age5_proof_family_transport_contract_completed mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index":
            return fail("passcase triage age5_proof_family_transport_contract_checks_text mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_family_transport_contract_selftest_last_completed_probe", "")).strip() != "report_index":
            return fail("passcase triage age5_proof_family_transport_contract_last_completed_probe mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_family_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("passcase triage age5_proof_family_transport_contract_progress mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index":
            return fail("passcase triage age5_proof_certificate_family_transport_contract_checks_text mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("passcase triage age5_proof_certificate_family_transport_contract_progress mismatch")
        if str(pass_triage_doc.get("age5_full_real_bogae_alias_family_contract_selftest_completed_checks", "")).strip() != "3":
            return fail("passcase triage age5_bogae_alias_family_contract_completed mismatch")
        if str(pass_triage_doc.get("age5_full_real_bogae_alias_family_contract_selftest_checks_text", "")).strip() != "shape_alias_contract,alias_family,alias_viewer_family":
            return fail("passcase triage age5_bogae_alias_family_contract_checks_text mismatch")
        if str(pass_triage_doc.get("age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe", "")).strip() != "alias_viewer_family":
            return fail("passcase triage age5_bogae_alias_family_contract_last_completed_probe mismatch")
        if str(pass_triage_doc.get("age5_full_real_bogae_alias_family_contract_selftest_progress_present", "")).strip() != "1":
            return fail("passcase triage age5_bogae_alias_family_contract_progress mismatch")
        if str(pass_triage_doc.get("age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks", "")).strip() != "9":
            return fail("passcase triage age5_bogae_alias_family_transport_contract_completed mismatch")
        if str(pass_triage_doc.get("age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index":
            return fail("passcase triage age5_bogae_alias_family_transport_contract_checks_text mismatch")
        if str(pass_triage_doc.get("age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe", "")).strip() != "report_index":
            return fail("passcase triage age5_bogae_alias_family_transport_contract_last_completed_probe mismatch")
        if str(pass_triage_doc.get("age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("passcase triage age5_bogae_alias_family_transport_contract_progress mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe", "")).strip() != "signed_contract":
            return fail("passcase triage age5_proof_certificate_v1_consumer_contract_last_completed_probe mismatch")
        if str(pass_triage_doc.get("age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("passcase triage age5_proof_certificate_v1_consumer_contract_progress mismatch")
        pass_artifacts = pass_triage_doc.get("artifacts")
        if not isinstance(pass_artifacts, dict) or "summary" not in pass_artifacts:
            return fail("passcase triage artifacts missing summary")
        for key in ("ci_fail_brief_txt", "ci_fail_triage_json"):
            row = pass_artifacts.get(key)
            if not isinstance(row, dict):
                return fail(f"passcase triage artifacts missing {key}")
            if not bool(row.get("exists", False)):
                return fail(f"passcase triage artifacts {key} exists mismatch")

        build_case(report_dir, "failcase", status="fail", reason="aggregate_failed", with_digest=True)
        proc_fail = run_emit(
            report_dir,
            "--prefix",
            "failcase",
            "--print-artifacts",
            "--print-failure-digest",
            "5",
            "--print-failure-tail-lines",
            "2",
            "--failure-brief-out",
            str(brief_tpl),
            "--triage-json-out",
            str(triage_tpl),
            "--require-final-line",
        )
        if proc_fail.returncode != 0:
            return fail(f"failcase returncode={proc_fail.returncode}")
        if not ensure_contains(proc_fail.stdout, "[ci-final] ci_gate_result_status=fail"):
            return fail("failcase final line missing")
        if not ensure_contains(proc_fail.stdout, "profile_matrix_total_elapsed_ms=666"):
            return fail("failcase final line profile_matrix_total_elapsed_ms missing")
        if not ensure_contains(proc_fail.stdout, "selected_real_profiles=core_lang,full,seamgrim"):
            return fail("failcase final line selected_real_profiles missing")
        if not ensure_contains(proc_fail.stdout, "profile_matrix_status=pass"):
            return fail("failcase final line profile_matrix_status missing")
        if not ensure_contains(proc_fail.stdout, "profile_matrix_ok=1"):
            return fail("failcase final line profile_matrix_ok missing")
        if not ensure_contains(proc_fail.stdout, "age5_close_digest_selftest_ok=1"):
            return fail("failcase final line age5 digest selftest status missing")
        if not ensure_contains(proc_fail.stdout, AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT):
            return fail("failcase final line age5 digest selftest default field missing")
        if not ensure_contains(proc_fail.stdout, AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_FRAGMENT):
            return fail("failcase final line age5 digest selftest default contract missing")
        if not ensure_contains(proc_fail.stdout, f"{AGE4_PROOF_OK_KEY}=0"):
            return fail("failcase final line age4 proof ok missing")
        if not ensure_contains(proc_fail.stdout, f"{AGE4_PROOF_FAILED_CRITERIA_KEY}=1"):
            return fail("failcase final line age4 proof failed criteria missing")
        if not ensure_contains(proc_fail.stdout, f"{AGE4_PROOF_SUMMARY_HASH_KEY}=sha256:age4-proof-fail"):
            return fail("failcase final line age4 proof summary hash missing")
        if not ensure_contains(proc_fail.stdout, "age5_proof_certificate_v1_consumer_contract_completed=5"):
            return fail("failcase final line age5_proof_certificate_v1_consumer_contract_completed missing")
        if not ensure_contains(
            proc_fail.stdout,
            "age5_proof_certificate_v1_consumer_contract_checks_text=signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract",
        ):
            return fail("failcase final line age5_proof_certificate_v1_consumer_contract_checks_text missing")
        if (
            "age5_proof_certificate_v1_verify_report_digest_contract_checks_text=verify_report_digest_contract"
            not in proc_fail.stdout
        ):
            return fail("failcase final line age5_proof_certificate_v1_verify_report_digest_contract_checks_text missing")
        if not ensure_contains(
            proc_fail.stdout,
            "age5_proof_certificate_v1_family_contract_checks_text=signed_contract,consumer_contract,promotion,family",
        ):
            return fail("failcase final line age5_proof_certificate_v1_family_contract_checks_text missing")
        if not ensure_contains(
            proc_fail.stdout,
            "age5_proof_family_contract_checks_text=proof_operation_family,proof_certificate_family,proof_family",
        ):
            return fail("failcase final line age5_proof_family_contract_checks_text missing")
        if not ensure_contains(
            proc_fail.stdout,
            "age5_lang_surface_family_contract_checks_text=proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family",
        ):
            return fail("failcase final line age5_lang_surface_family_contract_checks_text missing")
        if not ensure_contains(
            proc_fail.stdout,
            "age5_lang_runtime_family_contract_checks_text=lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family",
        ):
            return fail("failcase final line age5_lang_runtime_family_contract_checks_text missing")
        if not ensure_contains(
            proc_fail.stdout,
            "age5_lang_runtime_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("failcase final line age5_lang_runtime_family_transport_contract_checks_text missing")
        if not ensure_contains(
            proc_fail.stdout,
            "age5_gate0_family_contract_checks_text=gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family",
        ):
            return fail("failcase final line age5_gate0_family_contract_checks_text missing")
        if not ensure_contains(
            proc_fail.stdout,
            "age5_gate0_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("failcase final line age5_gate0_family_transport_contract_checks_text missing")
        if not ensure_contains(
            proc_fail.stdout,
            "age5_gate0_transport_family_contract_checks_text=lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family",
        ):
            return fail("failcase final line age5_gate0_transport_family_contract_checks_text missing")
        if not ensure_contains(
            proc_fail.stdout,
            "age5_gate0_transport_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("failcase final line age5_gate0_transport_family_transport_contract_checks_text missing")
        if not ensure_contains(
            proc_fail.stdout,
            "age5_gate0_runtime_family_transport_contract_checks_text=family_contract",
        ):
            return fail("failcase final line age5_gate0_runtime_family_transport_contract_checks_text missing")
        if not ensure_contains(
            proc_fail.stdout,
            "age5_lang_surface_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("failcase final line age5_lang_surface_family_transport_contract_checks_text missing")
        if not ensure_contains(
            proc_fail.stdout,
            "age5_proof_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("failcase final line age5_proof_family_transport_contract_checks_text missing")
        if not ensure_contains(
            proc_fail.stdout,
            "age5_bogae_alias_family_contract_checks_text=shape_alias_contract,alias_family,alias_viewer_family",
        ):
            return fail("failcase final line age5_bogae_alias_family_contract_checks_text missing")
        if not ensure_contains(
            proc_fail.stdout,
            "age5_bogae_alias_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("failcase final line age5_bogae_alias_family_transport_contract_checks_text missing")
        if not ensure_contains(
            proc_fail.stdout,
            "age5_proof_certificate_v1_consumer_contract_last_completed_probe=signed_contract",
        ):
            return fail("failcase final line age5_proof_certificate_v1_consumer_contract_last_completed_probe missing")
        if not ensure_contains(proc_fail.stdout, "age5_proof_certificate_v1_consumer_contract_progress=1"):
            return fail("failcase final line age5_proof_certificate_v1_consumer_contract_progress missing")
        if not ensure_contains(proc_fail.stdout, "age5_combined_heavy_full_real_status=pass"):
            return fail("failcase final line age5 full_real status missing")
        if not ensure_contains(proc_fail.stdout, "age5_combined_heavy_runtime_helper_negative_status=fail"):
            return fail("failcase final line age5 runtime_helper_negative status missing")
        if not ensure_contains(proc_fail.stdout, "age5_combined_heavy_group_id_summary_negative_status=fail"):
            return fail("failcase final line age5 group_id_summary_negative status missing")
        if not ensure_contains(
            proc_fail.stdout,
            "ci_sanity_age5_combined_heavy_child_summary_default_fields="
            + expected_default_transport["ci_sanity_age5_combined_heavy_child_summary_default_fields"],
        ):
            return fail("failcase final line child_summary_default transport missing")
        if not ensure_contains(
            proc_fail.stdout,
            "ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields="
            + expected_default_transport["ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"],
        ):
            return fail("failcase final line sync child_summary_default transport missing")
        if not ensure_contains(
            proc_fail.stdout,
            f"{AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}=age5_close_digest_selftest_ok=0",
        ):
            return fail("failcase final line age5 policy default text missing")
        if not ensure_contains(
            proc_fail.stdout,
            f"{AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY}={AGE4_PROOF_SNAPSHOT_FIELDS_TEXT}",
        ):
            return fail("failcase final line age5 policy age4 proof snapshot fields missing")
        if not ensure_contains(
            proc_fail.stdout,
            f"{AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY}={expected_policy_age4_proof_snapshot_text}",
        ):
            return fail("failcase final line age5 policy age4 proof snapshot text missing")
        if not ensure_contains(
            proc_fail.stdout,
            f'{AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY}={{"age5_close_digest_selftest_ok":"0"}}',
        ):
            return fail("failcase final line age5 policy default field missing")
        if not ensure_contains(proc_fail.stdout, f"{AGE5_POLICY_REPORT_EXISTS_KEY}=1"):
            return fail("failcase final line age5 policy report exists missing")
        if not ensure_contains(proc_fail.stdout, f"{AGE5_POLICY_TEXT_EXISTS_KEY}=1"):
            return fail("failcase final line age5 policy text exists missing")
        if not ensure_contains(proc_fail.stdout, f"{AGE5_POLICY_SUMMARY_PATH_KEY}={fail_policy_summary_path}"):
            return fail("failcase final line age5 policy summary path missing")
        if not ensure_contains(proc_fail.stdout, f"{AGE5_POLICY_SUMMARY_EXISTS_KEY}=1"):
            return fail("failcase final line age5 policy summary exists missing")
        if not ensure_contains(
            proc_fail.stdout,
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}=policy_summary_origin_trace_contract_mismatch",
        ):
            return fail("failcase final line age5 policy origin trace contract issue missing")
        if not ensure_contains(
            proc_fail.stdout,
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}=BROKEN",
        ):
            return fail("failcase final line age5 policy origin trace source issue missing")
        if not ensure_contains(
            proc_fail.stdout,
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}=issue=policy_summary_origin_trace_contract_mismatch|source=BROKEN",
        ):
            return fail("failcase final line age5 policy origin trace compact reason missing")
        if not ensure_contains(
            proc_fail.stdout,
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY}=policy_summary_origin_trace_contract_mismatch",
        ):
            return fail("failcase final line age5 policy origin trace compact failure reason missing")
        if not ensure_contains(proc_fail.stdout, f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}=mismatch"):
            return fail("failcase final line age5 policy origin trace contract status missing")
        if not ensure_contains(proc_fail.stdout, f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}=0"):
            return fail("failcase final line age5 policy origin trace contract ok missing")
        if not ensure_contains(proc_fail.stdout, f"{AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY}={fail_policy_origin_trace_text}"):
            return fail("failcase final line age5 policy origin trace text missing")
        if not ensure_contains(
            proc_fail.stdout,
            f"{AGE5_POLICY_ORIGIN_TRACE_KEY}="
            + json.dumps(fail_policy_origin_trace, ensure_ascii=False, sort_keys=True, separators=(",", ":")),
        ):
            return fail("failcase final line age5 policy origin trace missing")
        if not ensure_contains(proc_fail.stdout, "[ci-artifact] key=summary exists=1"):
            return fail("failcase summary artifact line missing")
        if not ensure_contains(proc_fail.stdout, "[ci-fail] status=fail"):
            return fail("failcase ci-fail status missing")
        if not ensure_contains(proc_fail.stdout, "age5_w107_active=54"):
            return fail("failcase final line age5_w107_active missing")
        if not ensure_contains(proc_fail.stdout, "[ci-fail] failed_steps=seamgrim_ci_gate,oi405_406_close"):
            return fail("failcase failed_steps priority order missing")
        if not ensure_contains(proc_fail.stdout, "[ci-fail] digest="):
            return fail("failcase digest missing")
        if not ensure_contains(proc_fail.stdout, "[ci-fail] step_logs=seamgrim_ci_gate"):
            return fail("failcase step log path missing")
        if not ensure_contains(proc_fail.stdout, "[ci-fail-brief] step=seamgrim_ci_gate"):
            return fail("failcase brief message missing")
        if not ensure_contains(proc_fail.stdout, "[ci-fail-tail] step=seamgrim_ci_gate stream=stderr"):
            return fail("failcase tail header missing")
        if not ensure_contains(proc_fail.stdout, "[ci-fail-tail] sg err 3"):
            return fail("failcase tail content missing")
        if not ensure_contains(proc_fail.stdout, "[ci-fail-verify] summary=ok"):
            return fail("failcase summary verify missing")
        fail_brief = report_dir / "failcase.ci_fail_brief.txt"
        if not fail_brief.exists():
            return fail("failcase brief file missing")
        fail_brief_line = fail_brief.read_text(encoding="utf-8").strip()
        if not ensure_contains(fail_brief_line, "status=fail"):
            return fail("failcase brief status missing")
        if not ensure_contains(fail_brief_line, "top_step=seamgrim_ci_gate"):
            return fail("failcase brief top_step missing")
        if not ensure_contains(fail_brief_line, "top_step_rc=1"):
            return fail("failcase brief top_step_rc missing")
        if not ensure_contains(fail_brief_line, 'top_step_cmd="python tests/run_seamgrim_ci_gate.py"'):
            return fail("failcase brief top_step_cmd missing")
        if not ensure_contains(fail_brief_line, "profile_matrix_seamgrim_elapsed_ms=333"):
            return fail("failcase brief profile_matrix_seamgrim_elapsed_ms missing")
        if not ensure_contains(fail_brief_line, "age5_close_digest_selftest_ok=1"):
            return fail("failcase brief age5 digest selftest status missing")
        if not ensure_contains(fail_brief_line, AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT):
            return fail("failcase brief age5 digest selftest default field missing")
        if not ensure_contains(fail_brief_line, AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_FRAGMENT):
            return fail("failcase brief age5 digest selftest default contract missing")
        if not ensure_contains(fail_brief_line, f"{AGE4_PROOF_OK_KEY}=0"):
            return fail("failcase brief age4 proof ok missing")
        if not ensure_contains(fail_brief_line, f"{AGE4_PROOF_FAILED_CRITERIA_KEY}=1"):
            return fail("failcase brief age4 proof failed criteria missing")
        if not ensure_contains(fail_brief_line, f"{AGE4_PROOF_SUMMARY_HASH_KEY}=sha256:age4-proof-fail"):
            return fail("failcase brief age4 proof summary hash missing")
        if not ensure_contains(fail_brief_line, "age5_w107_active=54"):
            return fail("failcase brief age5_w107_active missing")
        if not ensure_contains(fail_brief_line, "age5_w107_last_completed_probe=validate_pack_pointers"):
            return fail("failcase brief age5_w107_last_completed_probe missing")
        if not ensure_contains(fail_brief_line, "age5_w107_progress=1"):
            return fail("failcase brief age5_w107_progress missing")
        if not ensure_contains(fail_brief_line, "age5_w107_contract_completed=8"):
            return fail("failcase brief age5_w107_contract_completed missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_w107_contract_checks_text=golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index",
        ):
            return fail("failcase brief age5_w107_contract_checks_text missing")
        if not ensure_contains(fail_brief_line, "age5_w107_contract_last_completed_probe=report_index"):
            return fail("failcase brief age5_w107_contract_last_completed_probe missing")
        if not ensure_contains(fail_brief_line, "age5_age1_immediate_proof_operation_contract_completed=5"):
            return fail("failcase brief age5_age1_immediate_proof_operation_contract_completed missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_age1_immediate_proof_operation_contract_checks_text=operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family",
        ):
            return fail("failcase brief age5_age1_immediate_proof_operation_contract_checks_text missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_age1_immediate_proof_operation_contract_last_completed_probe=proof_operation_family",
        ):
            return fail("failcase brief age5_age1_immediate_proof_operation_contract_last_completed_probe missing")
        if not ensure_contains(fail_brief_line, "age5_proof_certificate_v1_consumer_contract_completed=5"):
            return fail("failcase brief age5_proof_certificate_v1_consumer_contract_completed missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_proof_certificate_v1_consumer_contract_checks_text=signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract",
        ):
            return fail("failcase brief age5_proof_certificate_v1_consumer_contract_checks_text missing")
        if (
            "age5_proof_certificate_v1_verify_report_digest_contract_checks_text=verify_report_digest_contract"
            not in fail_brief.read_text(encoding="utf-8")
        ):
            return fail("failcase brief age5_proof_certificate_v1_verify_report_digest_contract_checks_text missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_proof_certificate_v1_family_contract_checks_text=signed_contract,consumer_contract,promotion,family",
        ):
            return fail("failcase brief age5_proof_certificate_v1_family_contract_checks_text missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_proof_family_contract_checks_text=proof_operation_family,proof_certificate_family,proof_family",
        ):
            return fail("failcase brief age5_proof_family_contract_checks_text missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_lang_surface_family_contract_checks_text=proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family",
        ):
            return fail("failcase brief age5_lang_surface_family_contract_checks_text missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_lang_runtime_family_contract_checks_text=lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family",
        ):
            return fail("failcase brief age5_lang_runtime_family_contract_checks_text missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_lang_runtime_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("failcase brief age5_lang_runtime_family_transport_contract_checks_text missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_gate0_family_contract_checks_text=gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family",
        ):
            return fail("failcase brief age5_gate0_family_contract_checks_text missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_gate0_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("failcase brief age5_gate0_family_transport_contract_checks_text missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_gate0_transport_family_contract_checks_text=lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family",
        ):
            return fail("failcase brief age5_gate0_transport_family_contract_checks_text missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_gate0_transport_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("failcase brief age5_gate0_transport_family_transport_contract_checks_text missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_gate0_runtime_family_transport_contract_checks_text=family_contract",
        ):
            return fail("failcase brief age5_gate0_runtime_family_transport_contract_checks_text missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_lang_surface_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("failcase brief age5_lang_surface_family_transport_contract_checks_text missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_proof_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("failcase brief age5_proof_family_transport_contract_checks_text missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_bogae_alias_family_contract_checks_text=shape_alias_contract,alias_family,alias_viewer_family",
        ):
            return fail("failcase brief age5_bogae_alias_family_contract_checks_text missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_bogae_alias_family_transport_contract_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
        ):
            return fail("failcase brief age5_bogae_alias_family_transport_contract_checks_text missing")
        if not ensure_contains(
            fail_brief_line,
            "age5_proof_certificate_v1_consumer_contract_last_completed_probe=signed_contract",
        ):
            return fail("failcase brief age5_proof_certificate_v1_consumer_contract_last_completed_probe missing")
        if not ensure_contains(fail_brief_line, "age5_proof_certificate_v1_consumer_contract_progress=1"):
            return fail("failcase brief age5_proof_certificate_v1_consumer_contract_progress missing")
        if not ensure_contains(fail_brief_line, "age5_combined_heavy_full_real_status=pass"):
            return fail("failcase brief age5 full_real status missing")
        if not ensure_contains(fail_brief_line, "age5_combined_heavy_runtime_helper_negative_status=fail"):
            return fail("failcase brief age5 runtime_helper_negative status missing")
        if not ensure_contains(fail_brief_line, "age5_combined_heavy_group_id_summary_negative_status=fail"):
            return fail("failcase brief age5 group_id_summary_negative status missing")
        if not ensure_contains(
            fail_brief_line,
            "ci_sanity_age5_combined_heavy_child_summary_default_fields="
            + expected_default_transport["ci_sanity_age5_combined_heavy_child_summary_default_fields"],
        ):
            return fail("failcase brief child_summary_default transport missing")
        if not ensure_contains(
            fail_brief_line,
            "ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields="
            + expected_default_transport["ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"],
        ):
            return fail("failcase brief sync child_summary_default transport missing")
        if not ensure_contains(
            fail_brief_line,
            f"{AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}=age5_close_digest_selftest_ok=0",
        ):
            return fail("failcase brief age5 policy default text missing")
        if not ensure_contains(
            fail_brief_line,
            f'{AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY}={{"age5_close_digest_selftest_ok":"0"}}',
        ):
            return fail("failcase brief age5 policy default field missing")
        if not ensure_contains(
            fail_brief_line,
            f"{AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY}={AGE4_PROOF_SNAPSHOT_FIELDS_TEXT}",
        ):
            return fail("failcase brief age5 policy age4 proof snapshot fields missing")
        if not ensure_contains(
            fail_brief_line,
            f"{AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY}={expected_policy_age4_proof_snapshot_text}",
        ):
            return fail("failcase brief age5 policy age4 proof snapshot text missing")
        if not ensure_contains(fail_brief_line, f"{AGE5_POLICY_REPORT_EXISTS_KEY}=1"):
            return fail("failcase brief age5 policy report exists missing")
        if not ensure_contains(fail_brief_line, f"{AGE5_POLICY_TEXT_EXISTS_KEY}=1"):
            return fail("failcase brief age5 policy text exists missing")
        if not ensure_contains(fail_brief_line, f"{AGE5_POLICY_SUMMARY_PATH_KEY}={fail_policy_summary_path}"):
            return fail("failcase brief age5 policy summary path missing")
        if not ensure_contains(fail_brief_line, f"{AGE5_POLICY_SUMMARY_EXISTS_KEY}=1"):
            return fail("failcase brief age5 policy summary exists missing")
        if not ensure_contains(
            fail_brief_line,
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}=policy_summary_origin_trace_contract_mismatch",
        ):
            return fail("failcase brief age5 policy origin trace contract issue missing")
        if not ensure_contains(
            fail_brief_line,
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}=BROKEN",
        ):
            return fail("failcase brief age5 policy origin trace source issue missing")
        if not ensure_contains(
            fail_brief_line,
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}=issue=policy_summary_origin_trace_contract_mismatch|source=BROKEN",
        ):
            return fail("failcase brief age5 policy origin trace compact reason missing")
        if not ensure_contains(
            fail_brief_line,
            f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY}=policy_summary_origin_trace_contract_mismatch",
        ):
            return fail("failcase brief age5 policy origin trace compact failure reason missing")
        if not ensure_contains(fail_brief_line, f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}=mismatch"):
            return fail("failcase brief age5 policy origin trace contract status missing")
        if not ensure_contains(fail_brief_line, f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}=0"):
            return fail("failcase brief age5 policy origin trace contract ok missing")
        if not ensure_contains(fail_brief_line, f"{AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY}={fail_policy_origin_trace_text}"):
            return fail("failcase brief age5 policy origin trace text missing")
        if not ensure_contains(
            fail_brief_line,
            f"{AGE5_POLICY_ORIGIN_TRACE_KEY}="
            + json.dumps(fail_policy_origin_trace, ensure_ascii=False, sort_keys=True, separators=(",", ":")),
        ):
            return fail("failcase brief age5 policy origin trace missing")
        fail_triage = report_dir / "failcase.ci_fail_triage.detjson"
        if not fail_triage.exists():
            return fail("failcase triage file missing")
        fail_triage_doc = json.loads(fail_triage.read_text(encoding="utf-8"))
        if str(fail_triage_doc.get("schema", "")) != "ddn.ci.fail_triage.v1":
            return fail("failcase triage schema mismatch")
        if str(fail_triage_doc.get("status", "")) != "fail":
            return fail("failcase triage status mismatch")
        if not bool(fail_triage_doc.get("summary_verify_ok", False)):
            return fail("failcase triage summary_verify_ok mismatch")
        if int(fail_triage_doc.get("summary_verify_issues_count", -1)) != 0:
            return fail("failcase triage summary_verify_issues_count mismatch")
        if str(fail_triage_doc.get("summary_verify_top_issue", "")).strip() != "-":
            return fail("failcase triage summary_verify_top_issue mismatch")
        if int(fail_triage_doc.get("failed_steps_count", 0)) <= 0:
            return fail("failcase triage failed_steps_count mismatch")
        if int(fail_triage_doc.get("failed_step_detail_rows_count", -1)) != 2:
            return fail("failcase triage failed_step_detail_rows_count mismatch")
        if int(fail_triage_doc.get("failed_step_logs_rows_count", -1)) != 2:
            return fail("failcase triage failed_step_logs_rows_count mismatch")
        if list(fail_triage_doc.get("failed_step_detail_order", [])) != [
            "seamgrim_ci_gate",
            "oi405_406_close",
        ]:
            return fail("failcase triage failed_step_detail_order mismatch")
        if list(fail_triage_doc.get("failed_step_logs_order", [])) != [
            "seamgrim_ci_gate",
            "oi405_406_close",
        ]:
            return fail("failcase triage failed_step_logs_order mismatch")
        fail_profile_matrix = fail_triage_doc.get("profile_matrix_selftest")
        if not isinstance(fail_profile_matrix, dict):
            return fail("failcase triage profile_matrix_selftest missing")
        if int(fail_profile_matrix.get("seamgrim_elapsed_ms", -1)) != 333:
            return fail("failcase triage profile_matrix_selftest seamgrim_elapsed_ms mismatch")
        if str(fail_profile_matrix.get("step_timeout_defaults_text", "")).strip() != PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT:
            return fail("failcase triage profile_matrix_selftest timeout defaults mismatch")
        if dict(fail_profile_matrix.get("step_timeout_defaults_sec", {})) != dict(PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC):
            return fail("failcase triage profile_matrix_selftest timeout defaults sec mismatch")
        if dict(fail_profile_matrix.get("step_timeout_env_keys", {})) != dict(PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS):
            return fail("failcase triage profile_matrix_selftest timeout env keys mismatch")
        if str(fail_profile_matrix.get("seamgrim_aggregate_summary_values", "")).strip() != SEAMGRIM_PROFILE_MATRIX_VALUES:
            return fail("failcase triage profile_matrix aggregate summary values mismatch")
        if int(fail_triage_doc.get(AGE4_PROOF_OK_KEY, 1)) != 0:
            return fail("failcase triage age4 proof ok mismatch")
        if int(fail_triage_doc.get(AGE4_PROOF_FAILED_CRITERIA_KEY, -1)) != 1:
            return fail("failcase triage age4 proof failed criteria mismatch")
        if str(fail_triage_doc.get(AGE4_PROOF_SUMMARY_HASH_KEY, "")).strip() != "sha256:age4-proof-fail":
            return fail("failcase triage age4 proof summary hash mismatch")
        if str(fail_triage_doc.get("age5_full_real_w107_golden_index_selftest_active_cases", "")).strip() != "54":
            return fail("failcase triage age5_w107_active mismatch")
        if str(fail_triage_doc.get("age5_full_real_w107_golden_index_selftest_last_completed_probe", "")).strip() != "validate_pack_pointers":
            return fail("failcase triage age5_w107_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_w107_golden_index_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_w107_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_w107_progress_contract_selftest_completed_checks", "")).strip() != "8":
            return fail("failcase triage age5_w107_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_w107_progress_contract_selftest_checks_text", "")).strip() != "golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index":
            return fail("failcase triage age5_w107_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_w107_progress_contract_selftest_last_completed_probe", "")).strip() != "report_index":
            return fail("failcase triage age5_w107_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_w107_progress_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_w107_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks", "")).strip() != "5":
            return fail("failcase triage age5_age1_immediate_proof_operation_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text", "")).strip() != "operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family":
            return fail("failcase triage age5_age1_immediate_proof_operation_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe", "")).strip() != "proof_operation_family":
            return fail("failcase triage age5_age1_immediate_proof_operation_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_age1_immediate_proof_operation_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks", "")).strip() != "5":
            return fail("failcase triage age5_proof_certificate_v1_consumer_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text", "")).strip() != "signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract":
            return fail("failcase triage age5_proof_certificate_v1_consumer_contract_checks_text mismatch")
        if (
            str(
                fail_triage_doc.get(
                    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text",
                    "",
                )
            ).strip()
            != "verify_report_digest_contract"
        ):
            return fail("failcase triage age5_proof_certificate_v1_verify_report_digest_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks", "")).strip() != "4":
            return fail("failcase triage age5_proof_certificate_v1_family_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text", "")).strip() != "signed_contract,consumer_contract,promotion,family":
            return fail("failcase triage age5_proof_certificate_v1_family_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe", "")).strip() != "family":
            return fail("failcase triage age5_proof_certificate_v1_family_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_proof_certificate_v1_family_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_certificate_family_contract_selftest_completed_checks", "")).strip() != "3":
            return fail("failcase triage age5_proof_certificate_family_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_certificate_family_contract_selftest_checks_text", "")).strip() != "artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family":
            return fail("failcase triage age5_proof_certificate_family_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe", "")).strip() != "proof_certificate_family":
            return fail("failcase triage age5_proof_certificate_family_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_certificate_family_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_proof_certificate_family_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_family_contract_selftest_completed_checks", "")).strip() != "3":
            return fail("failcase triage age5_proof_family_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_family_contract_selftest_checks_text", "")).strip() != "proof_operation_family,proof_certificate_family,proof_family":
            return fail("failcase triage age5_proof_family_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_family_contract_selftest_last_completed_probe", "")).strip() != "proof_family":
            return fail("failcase triage age5_proof_family_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_family_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_proof_family_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_lang_surface_family_contract_selftest_completed_checks", "")).strip() != "4":
            return fail("failcase triage age5_lang_surface_family_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_lang_surface_family_contract_selftest_checks_text", "")).strip() != "proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family":
            return fail("failcase triage age5_lang_surface_family_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_lang_surface_family_contract_selftest_last_completed_probe", "")).strip() != "lang_surface_family":
            return fail("failcase triage age5_lang_surface_family_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_lang_surface_family_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_lang_surface_family_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_lang_runtime_family_contract_selftest_completed_checks", "")).strip() != "5":
            return fail("failcase triage age5_lang_runtime_family_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_lang_runtime_family_contract_selftest_checks_text", "")).strip() != "lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family":
            return fail("failcase triage age5_lang_runtime_family_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe", "")).strip() != "lang_runtime_family":
            return fail("failcase triage age5_lang_runtime_family_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_lang_runtime_family_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_lang_runtime_family_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_family_contract_selftest_completed_checks", "")).strip() != "5":
            return fail("failcase triage age5_gate0_family_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_family_contract_selftest_checks_text", "")).strip() != "gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family":
            return fail("failcase triage age5_gate0_family_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_family_contract_selftest_last_completed_probe", "")).strip() != "gate0_family":
            return fail("failcase triage age5_gate0_family_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_family_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_gate0_family_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_family_transport_contract_selftest_completed_checks", "")).strip() != "9":
            return fail("failcase triage age5_gate0_family_transport_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index":
            return fail("failcase triage age5_gate0_family_transport_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe", "")).strip() != "report_index":
            return fail("failcase triage age5_gate0_family_transport_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_family_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_gate0_family_transport_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_transport_family_contract_selftest_completed_checks", "")).strip() != "4":
            return fail("failcase triage age5_gate0_transport_family_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_transport_family_contract_selftest_checks_text", "")).strip() != "lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family":
            return fail("failcase triage age5_gate0_transport_family_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe", "")).strip() != "gate0_transport_family":
            return fail("failcase triage age5_gate0_transport_family_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_transport_family_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_gate0_transport_family_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks", "")).strip() != "9":
            return fail("failcase triage age5_gate0_transport_family_transport_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index":
            return fail("failcase triage age5_gate0_transport_family_transport_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe", "")).strip() != "report_index":
            return fail("failcase triage age5_gate0_transport_family_transport_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_gate0_transport_family_transport_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks", "")).strip() != "9":
            return fail("failcase triage age5_lang_runtime_family_transport_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index":
            return fail("failcase triage age5_lang_runtime_family_transport_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe", "")).strip() != "report_index":
            return fail("failcase triage age5_lang_runtime_family_transport_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_lang_runtime_family_transport_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks", "")).strip() != "1":
            return fail("failcase triage age5_gate0_runtime_family_transport_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract":
            return fail("failcase triage age5_gate0_runtime_family_transport_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe", "")).strip() != "family_contract":
            return fail("failcase triage age5_gate0_runtime_family_transport_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_gate0_runtime_family_transport_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks", "")).strip() != "9":
            return fail("failcase triage age5_lang_surface_family_transport_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_lang_surface_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index":
            return fail("failcase triage age5_lang_surface_family_transport_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe", "")).strip() != "report_index":
            return fail("failcase triage age5_lang_surface_family_transport_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_lang_surface_family_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_lang_surface_family_transport_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_family_transport_contract_selftest_completed_checks", "")).strip() != "9":
            return fail("failcase triage age5_proof_family_transport_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index":
            return fail("failcase triage age5_proof_family_transport_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_family_transport_contract_selftest_last_completed_probe", "")).strip() != "report_index":
            return fail("failcase triage age5_proof_family_transport_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_family_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_proof_family_transport_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index":
            return fail("failcase triage age5_proof_certificate_family_transport_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_proof_certificate_family_transport_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_bogae_alias_family_contract_selftest_completed_checks", "")).strip() != "3":
            return fail("failcase triage age5_bogae_alias_family_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_bogae_alias_family_contract_selftest_checks_text", "")).strip() != "shape_alias_contract,alias_family,alias_viewer_family":
            return fail("failcase triage age5_bogae_alias_family_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe", "")).strip() != "alias_viewer_family":
            return fail("failcase triage age5_bogae_alias_family_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_bogae_alias_family_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_bogae_alias_family_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks", "")).strip() != "9":
            return fail("failcase triage age5_bogae_alias_family_transport_contract_completed mismatch")
        if str(fail_triage_doc.get("age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text", "")).strip() != "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index":
            return fail("failcase triage age5_bogae_alias_family_transport_contract_checks_text mismatch")
        if str(fail_triage_doc.get("age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe", "")).strip() != "report_index":
            return fail("failcase triage age5_bogae_alias_family_transport_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_bogae_alias_family_transport_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe", "")).strip() != "signed_contract":
            return fail("failcase triage age5_proof_certificate_v1_consumer_contract_last_completed_probe mismatch")
        if str(fail_triage_doc.get("age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present", "")).strip() != "1":
            return fail("failcase triage age5_proof_certificate_v1_consumer_contract_progress mismatch")
        if str(fail_triage_doc.get("age5_close_digest_selftest_ok", "")).strip() != "1":
            return fail("failcase triage age5 digest selftest status mismatch")
        if str(fail_triage_doc.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip() != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
            return fail("failcase triage age5 digest selftest default contract text mismatch")
        if dict(fail_triage_doc.get("combined_digest_selftest_default_field", {})) != {"age5_close_digest_selftest_ok": "0"}:
            return fail("failcase triage age5 digest selftest default contract dict mismatch")
        if str(fail_triage_doc.get("age5_combined_heavy_full_real_status", "")).strip() != "pass":
            return fail("failcase triage age5 full_real status mismatch")
        if str(fail_triage_doc.get("age5_combined_heavy_runtime_helper_negative_status", "")).strip() != "fail":
            return fail("failcase triage age5 runtime_helper_negative status mismatch")
        if str(fail_triage_doc.get("age5_combined_heavy_group_id_summary_negative_status", "")).strip() != "fail":
            return fail("failcase triage age5 group_id_summary_negative status mismatch")
        if (
            str(fail_triage_doc.get("ci_sanity_age5_combined_heavy_child_summary_default_fields", "")).strip()
            != expected_default_transport["ci_sanity_age5_combined_heavy_child_summary_default_fields"]
        ):
            return fail("failcase triage child_summary_default transport mismatch")
        if (
            str(
                fail_triage_doc.get(
                    "ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields", ""
                )
            ).strip()
            != expected_default_transport[
                "ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"
            ]
        ):
            return fail("failcase triage sync child_summary_default transport mismatch")
        if str(fail_triage_doc.get(AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip() != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
            return fail("failcase triage age5 policy default text mismatch")
        if dict(fail_triage_doc.get(AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY, {})) != {"age5_close_digest_selftest_ok": "0"}:
            return fail("failcase triage age5 policy default field mismatch")
        if str(fail_triage_doc.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY, "")).strip() != AGE4_PROOF_SNAPSHOT_FIELDS_TEXT:
            return fail("failcase triage age5 policy age4 proof snapshot fields mismatch")
        if str(fail_triage_doc.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY, "")).strip() != expected_policy_age4_proof_snapshot_text:
            return fail("failcase triage age5 policy age4 proof snapshot text mismatch")
        if int(fail_triage_doc.get(AGE5_POLICY_REPORT_EXISTS_KEY, 0)) != 1:
            return fail("failcase triage age5 policy report exists mismatch")
        if int(fail_triage_doc.get(AGE5_POLICY_TEXT_EXISTS_KEY, 0)) != 1:
            return fail("failcase triage age5 policy text exists mismatch")
        if str(fail_triage_doc.get(AGE5_POLICY_SUMMARY_PATH_KEY, "")).strip() != str(fail_policy_summary_path):
            return fail("failcase triage age5 policy summary path mismatch")
        if int(fail_triage_doc.get(AGE5_POLICY_SUMMARY_EXISTS_KEY, 0)) != 1:
            return fail("failcase triage age5 policy summary exists mismatch")
        if (
            str(fail_triage_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY, "")).strip()
            != "policy_summary_origin_trace_contract_mismatch"
        ):
            return fail("failcase triage age5 policy origin trace contract issue mismatch")
        if str(fail_triage_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY, "")).strip() != "BROKEN":
            return fail("failcase triage age5 policy origin trace source issue mismatch")
        if (
            str(fail_triage_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY, "")).strip()
            != "issue=policy_summary_origin_trace_contract_mismatch|source=BROKEN"
        ):
            return fail("failcase triage age5 policy origin trace compact reason mismatch")
        if (
            str(
                fail_triage_doc.get(
                    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY, ""
                )
            ).strip()
            != "policy_summary_origin_trace_contract_mismatch"
        ):
            return fail("failcase triage age5 policy origin trace compact failure reason mismatch")
        if str(fail_triage_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY, "")).strip() != "mismatch":
            return fail("failcase triage age5 policy origin trace contract status mismatch")
        if int(fail_triage_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY, 1)) != 0:
            return fail("failcase triage age5 policy origin trace contract ok mismatch")
        if str(fail_triage_doc.get(AGE5_POLICY_ORIGIN_TRACE_TEXT_KEY, "")).strip() != fail_policy_origin_trace_text:
            return fail("failcase triage age5 policy origin trace text mismatch")
        if dict(fail_triage_doc.get(AGE5_POLICY_ORIGIN_TRACE_KEY, {})) != fail_policy_origin_trace:
            return fail("failcase triage age5 policy origin trace mismatch")
        fail_artifacts = fail_triage_doc.get("artifacts")
        if not isinstance(fail_artifacts, dict):
            return fail("failcase triage artifacts missing")
        for key in ("ci_fail_brief_txt", "ci_fail_triage_json"):
            row = fail_artifacts.get(key)
            if not isinstance(row, dict):
                return fail(f"failcase triage artifacts missing {key}")
            if not bool(row.get("exists", False)):
                return fail(f"failcase triage artifacts {key} exists mismatch")
        fail_steps = fail_triage_doc.get("failed_steps")
        if not isinstance(fail_steps, list) or not fail_steps:
            return fail("failcase triage failed_steps missing")
        first_row = fail_steps[0]
        if not isinstance(first_row, dict):
            return fail("failcase triage failed_steps row invalid")
        if str(first_row.get("cmd", "")).strip() != "python tests/run_seamgrim_ci_gate.py":
            return fail("failcase triage first failed_step cmd mismatch")
        if str(first_row.get("fast_fail_step_detail", "")).strip() != (
            "name=seamgrim_ci_gate rc=1 cmd=python tests/run_seamgrim_ci_gate.py"
        ):
            return fail("failcase triage first failed_step fast_fail_step_detail mismatch")
        if "name=seamgrim_ci_gate " not in str(first_row.get("fast_fail_step_logs", "")).strip():
            return fail("failcase triage first failed_step fast_fail_step_logs missing")
        if "stderr_log_path_norm" not in first_row:
            return fail("failcase triage normalized stderr path missing")

        build_case(
            report_dir,
            "manyfail",
            status="fail",
            reason="aggregate_failed",
            with_digest=True,
            extra_failed_step_count=9,
        )
        proc_manyfail = run_emit(
            report_dir,
            "--prefix",
            "manyfail",
            "--triage-json-out",
            str(triage_tpl),
            "--require-final-line",
        )
        if proc_manyfail.returncode != 0:
            return fail(f"manyfail returncode={proc_manyfail.returncode}")
        manyfail_triage = report_dir / "manyfail.ci_fail_triage.detjson"
        if not manyfail_triage.exists():
            return fail("manyfail triage file missing")
        manyfail_triage_doc = json.loads(manyfail_triage.read_text(encoding="utf-8"))
        manyfail_expected_order = [
            "seamgrim_ci_gate",
            "oi405_406_close",
            "extra_fail_01",
            "extra_fail_02",
            "extra_fail_03",
            "extra_fail_04",
            "extra_fail_05",
            "extra_fail_06",
            "extra_fail_07",
            "extra_fail_08",
            "extra_fail_09",
        ]
        if int(manyfail_triage_doc.get("failed_step_detail_rows_count", -1)) != len(manyfail_expected_order):
            return fail("manyfail triage failed_step_detail_rows_count mismatch")
        if int(manyfail_triage_doc.get("failed_step_logs_rows_count", -1)) != len(manyfail_expected_order):
            return fail("manyfail triage failed_step_logs_rows_count mismatch")
        if list(manyfail_triage_doc.get("failed_step_detail_order", [])) != manyfail_expected_order:
            return fail("manyfail triage failed_step_detail_order mismatch")
        if list(manyfail_triage_doc.get("failed_step_logs_order", [])) != manyfail_expected_order:
            return fail("manyfail triage failed_step_logs_order mismatch")
        if int(manyfail_triage_doc.get("failed_steps_count", -1)) != 8:
            return fail("manyfail triage failed_steps_count (max_steps) mismatch")

        build_case(
            report_dir,
            "failsummary",
            status="fail",
            reason="aggregate_failed",
            with_digest=True,
            broken_summary=True,
        )
        proc_fail_summary = run_emit(
            report_dir,
            "--prefix",
            "failsummary",
            "--print-failure-digest",
            "4",
            "--fail-on-summary-verify-error",
            "--triage-json-out",
            str(triage_tpl),
            "--require-final-line",
        )
        if proc_fail_summary.returncode == 0:
            return fail("failsummary must fail when summary verify option is enabled")
        if not ensure_contains(proc_fail_summary.stdout, "[ci-fail-verify] summary=fail"):
            return fail("failsummary verify fail line missing")
        fail_summary_triage = report_dir / "failsummary.ci_fail_triage.detjson"
        if not fail_summary_triage.exists():
            return fail("failsummary triage file missing")
        fail_summary_triage_doc = json.loads(fail_summary_triage.read_text(encoding="utf-8"))
        if str(fail_summary_triage_doc.get("schema", "")) != "ddn.ci.fail_triage.v1":
            return fail("failsummary triage schema mismatch")
        if bool(fail_summary_triage_doc.get("summary_verify_ok", True)):
            return fail("failsummary triage summary_verify_ok mismatch")
        verify_issues = fail_summary_triage_doc.get("summary_verify_issues")
        if not isinstance(verify_issues, list) or not verify_issues:
            return fail("failsummary triage summary_verify_issues missing")
        if int(fail_summary_triage_doc.get("summary_verify_issues_count", -1)) != len(verify_issues):
            return fail("failsummary triage summary_verify_issues_count mismatch")
        top_issue = str(fail_summary_triage_doc.get("summary_verify_top_issue", "")).strip()
        if not top_issue:
            return fail("failsummary triage summary_verify_top_issue missing")
        first_issue = str(verify_issues[0]).strip()
        if first_issue not in summary_verify_codes:
            return fail(f"failsummary triage summary_verify_issues invalid: {first_issue}")
        if top_issue != first_issue:
            return fail("failsummary triage summary_verify_top_issue mismatch")

        empty_dir = report_dir / "empty"
        empty_dir.mkdir(parents=True, exist_ok=True)
        proc_empty = run_emit(empty_dir, "--require-final-line")
        if proc_empty.returncode == 0:
            return fail("empty case must fail when --require-final-line is set")
        if not ensure_contains(proc_empty.stdout, "status=unknown reason=final_line_missing"):
            return fail("empty case missing unknown status line")

    print("[ci-final-emitter-check] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
