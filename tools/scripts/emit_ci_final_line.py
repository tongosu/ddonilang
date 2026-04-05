#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

from ci_verify_codes import SUMMARY_VERIFY_CODES as VERIFY_CODES

ROOT = Path(__file__).resolve().parents[2]
TESTS_DIR = ROOT / "tests"
if str(TESTS_DIR) not in sys.path:
    sys.path.insert(0, str(TESTS_DIR))

from _ci_age5_combined_heavy_contract import (  # type: ignore
    AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY,
    AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY,
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY,
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY,
    AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
    AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
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
    build_age4_proof_snapshot,
    build_age4_proof_snapshot_text,
    build_age5_combined_heavy_policy_origin_trace_contract_compact_failure_reason,
    build_age5_combined_heavy_policy_origin_trace_contract_compact_reason,
    build_age5_combined_heavy_policy_origin_trace,
    build_age5_combined_heavy_policy_origin_trace_text,
    build_age5_close_digest_selftest_default_field,
    build_age5_combined_heavy_child_summary_default_text_transport_fields,
)
from _ci_profile_matrix_selftest_lib import (  # type: ignore
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC,
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
    PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS,
    PROFILE_MATRIX_SUMMARY_VALUE_KEYS,
)

INDEX_SCHEMA = "ddn.ci.aggregate_gate.index.v1"
PROFILE_MATRIX_SELFTEST_SCHEMA = "ddn.ci.profile_matrix_gate_selftest.v1"
SUMMARY_PATTERNS = (
    "*.ci_gate_summary_line.txt",
    "*.ci_gate_result_line.txt",
    "*.ci_gate_final_status_line.txt",
    "*.ci_aggregate_status_line.txt",
)
ARTIFACT_KEYS = (
    "summary",
    "summary_line",
    "ci_gate_result_json",
    "ci_gate_result_parse_json",
    "ci_gate_badge_json",
    "final_status_line",
    "final_status_parse_json",
    "aggregate_status_line",
    "aggregate_status_parse_json",
    "age3_close_status_json",
    "age3_close_status_line",
    "age3_close_badge_json",
    "ci_fail_brief_txt",
    "ci_fail_triage_json",
)
FAILED_STEP_PRIORITY = (
    "seamgrim_ci_gate",
    "age3_close",
    "oi405_406_close",
    "aggregate_combine",
    "aggregate_status_line_check",
    "final_status_line_check",
    "ci_gate_result_check",
    "summary_line_check",
    "ci_gate_outputs_consistency_check",
)
FAILED_STEP_PRIORITY_MAP = {name: idx for idx, name in enumerate(FAILED_STEP_PRIORITY)}
SUMMARY_DETAIL_RE = re.compile(r"^failed_step_detail=([^ ]+) rc=([-]?\d+) cmd=(.+)$")
SUMMARY_LOGS_RE = re.compile(r"^failed_step_logs=([^ ]+) stdout=([^ ]+) stderr=([^ ]+)$")
PROFILE_MATRIX_STDOUT_TOKEN_SPECS = (
    ("profile_matrix_total_elapsed_ms", "total_elapsed_ms", False, 0),
    ("selected_real_profiles", "selected_real_profiles", False, 0),
    ("profile_matrix_status", "status", False, 0),
    ("profile_matrix_ok", "ok", False, 0),
)
PROFILE_MATRIX_BRIEF_TOKEN_SPECS = (
    ("profile_matrix_total_elapsed_ms", "total_elapsed_ms", False, 0),
    ("profile_matrix_selected_real_profiles", "selected_real_profiles", True, 120),
    ("profile_matrix_core_lang_elapsed_ms", "core_lang_elapsed_ms", False, 0),
    ("profile_matrix_full_elapsed_ms", "full_elapsed_ms", False, 0),
    ("profile_matrix_seamgrim_elapsed_ms", "seamgrim_elapsed_ms", False, 0),
)
PROFILE_MATRIX_AGGREGATE_SUMMARY_VALUE_KEYS = PROFILE_MATRIX_SUMMARY_VALUE_KEYS
PROFILE_MATRIX_SNAPSHOT_FIELD_SPECS = (
    ("report_path", "report_path", "", ""),
    ("status", "text", "status", "missing_report"),
    ("ok", "bool", "ok", False),
    ("total_elapsed_ms", "int", "total_elapsed_ms", None),
    ("selected_real_profiles", "names", "selected_real_profiles", []),
    ("skipped_real_profiles", "names", "skipped_real_profiles", []),
    ("step_timeout_defaults_text", "text", "step_timeout_defaults_text", PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT),
    ("step_timeout_defaults_sec", "dict_float", "step_timeout_defaults_sec", dict(PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC)),
    ("step_timeout_env_keys", "dict_text", "step_timeout_env_keys", dict(PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS)),
    ("core_lang_elapsed_ms", "elapsed", "core_lang", None),
    ("full_elapsed_ms", "elapsed", "full", None),
    ("seamgrim_elapsed_ms", "elapsed", "seamgrim", None),
)
AGE5_CHILD_SUMMARY_KEYS = (
    "age5_combined_heavy_full_real_status",
    "age5_combined_heavy_runtime_helper_negative_status",
    "age5_combined_heavy_group_id_summary_negative_status",
)
AGE5_CHILD_SUMMARY_VALUES = {"pass", "fail", "skipped"}
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
AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_gate0_family_contract_selftest_completed_checks",
    "age5_full_real_gate0_family_contract_selftest_total_checks",
    "age5_full_real_gate0_family_contract_selftest_checks_text",
    "age5_full_real_gate0_family_contract_selftest_current_probe",
    "age5_full_real_gate0_family_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_family_contract_selftest_progress_present",
)
AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_gate0_surface_family_contract_selftest_completed_checks",
    "age5_full_real_gate0_surface_family_contract_selftest_total_checks",
    "age5_full_real_gate0_surface_family_contract_selftest_checks_text",
    "age5_full_real_gate0_surface_family_contract_selftest_current_probe",
    "age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_surface_family_contract_selftest_progress_present",
)
AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present",
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
AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present",
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
AGE5_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS = (
    build_age5_combined_heavy_child_summary_default_text_transport_fields()
)
AGE5_DIGEST_SELFTEST_DEFAULT_FIELD = build_age5_close_digest_selftest_default_field()


def clip(text: str, limit: int = 240) -> str:
    s = str(text).strip()
    if len(s) <= limit:
        return s
    return s[: max(0, limit - 3)] + "..."


def load_json(path: Path) -> dict | None:
    try:
        data = json.loads(path.read_text(encoding="utf-8-sig"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def load_line(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8-sig").strip()
    except Exception:
        return ""


def read_tail_lines(path: Path, line_count: int) -> list[str]:
    if line_count <= 0:
        return []
    try:
        content = path.read_text(encoding="utf-8-sig", errors="replace")
    except Exception:
        return []
    lines = content.splitlines()
    if not lines:
        return []
    tail = lines[-line_count:]
    out: list[str] = []
    for line in tail:
        stripped = str(line).rstrip()
        if stripped:
            out.append(stripped)
    return out


def first_nonempty_line(path: Path, prefer_errorish: bool) -> str:
    try:
        content = path.read_text(encoding="utf-8-sig", errors="replace")
    except Exception:
        return ""
    lines = [str(line).strip() for line in content.splitlines() if str(line).strip()]
    if not lines:
        return ""
    if not prefer_errorish:
        return lines[0]
    error_tokens = ("error", "failed", "fail", "exception", "traceback", "panic", "launch_error")
    lowered_lines = [line.lower() for line in lines]
    for idx, lowered in enumerate(lowered_lines):
        if any(token in lowered for token in error_tokens):
            return lines[idx]
    return lines[0]


def sorted_failed_rows(steps: list[dict]) -> list[dict]:
    indexed_rows = []
    for idx, row in enumerate(steps):
        if not isinstance(row, dict):
            continue
        if bool(row.get("ok", False)):
            continue
        name = str(row.get("name", "-")).strip()
        priority = FAILED_STEP_PRIORITY_MAP.get(name, len(FAILED_STEP_PRIORITY))
        indexed_rows.append((priority, idx, row))
    indexed_rows.sort(key=lambda item: (item[0], item[1]))
    return [row for _, __, row in indexed_rows]


def quote_token(text: str) -> str:
    escaped = str(text).replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def normalize_path(raw_path: str) -> Path:
    return Path(str(raw_path).replace("\\", "/"))


def normalize_path_text(raw_path: str) -> str:
    text = str(raw_path).strip()
    if not text:
        return ""
    return text.replace("\\", "/")


def failed_row_details(row: dict) -> tuple[str, str, str, str]:
    name = str(row.get("name", "-")).strip() or "-"
    stdout_log = str(row.get("stdout_log_path", "")).strip()
    stderr_log = str(row.get("stderr_log_path", "")).strip()
    stderr_path = normalize_path(stderr_log) if stderr_log else None
    stdout_path = normalize_path(stdout_log) if stdout_log else None
    brief = ""
    if stderr_path is not None and stderr_path.exists():
        brief = first_nonempty_line(stderr_path, prefer_errorish=True)
    if not brief and stdout_path is not None and stdout_path.exists():
        brief = first_nonempty_line(stdout_path, prefer_errorish=True)
    return name, stdout_log, stderr_log, brief


def load_summary_failed_step_rows(
    index_doc: dict | None,
) -> tuple[dict[str, dict[str, object]], dict[str, dict[str, str]], list[str], list[str]]:
    detail_rows: dict[str, dict[str, object]] = {}
    log_rows: dict[str, dict[str, str]] = {}
    detail_order: list[str] = []
    log_order: list[str] = []
    if not isinstance(index_doc, dict):
        return detail_rows, log_rows, detail_order, log_order
    summary_path = artifact_path(index_doc, "summary")
    if summary_path is None or not summary_path.exists():
        return detail_rows, log_rows, detail_order, log_order
    _, _, rows = parse_summary_report(summary_path)
    for key, value in rows:
        if key == "failed_step_detail":
            match = SUMMARY_DETAIL_RE.match(f"failed_step_detail={value}")
            if not match:
                continue
            step_id = str(match.group(1)).strip()
            if not step_id:
                continue
            try:
                rc_value = int(match.group(2))
            except Exception:
                continue
            cmd_value = str(match.group(3)).strip()
            if step_id not in detail_rows:
                detail_order.append(step_id)
            detail_rows[step_id] = {
                "rc": rc_value,
                "cmd": cmd_value,
                "raw": str(value).strip(),
            }
            continue
        if key == "failed_step_logs":
            match = SUMMARY_LOGS_RE.match(f"failed_step_logs={value}")
            if not match:
                continue
            step_id = str(match.group(1)).strip()
            if not step_id:
                continue
            stdout_value = str(match.group(2)).strip()
            stderr_value = str(match.group(3)).strip()
            if step_id not in log_rows:
                log_order.append(step_id)
            log_rows[step_id] = {
                "stdout": "" if stdout_value == "-" else stdout_value,
                "stderr": "" if stderr_value == "-" else stderr_value,
                "raw": str(value).strip(),
            }
    return detail_rows, log_rows, detail_order, log_order


def build_failure_brief_line(
    index_doc: dict | None,
    result_doc: dict | None,
    final_line: str,
    limit: int,
) -> str:
    status = str(result_doc.get("status", "")).strip() if isinstance(result_doc, dict) else ""
    reason = str(result_doc.get("reason", "-")).strip() if isinstance(result_doc, dict) else "-"
    if not status:
        status = "unknown"
    if not reason:
        reason = "-"
    failed_steps: list[str] = []
    top_step = "-"
    top_message = "-"
    top_step_rc = "-"
    top_step_cmd = "-"
    if isinstance(index_doc, dict):
        steps = index_doc.get("steps")
        if isinstance(steps, list):
            failed_rows = sorted_failed_rows(steps)
            failed_steps = [str(row.get("name", "-")).strip() or "-" for row in failed_rows]
            if failed_rows:
                top_payload_rows = failed_steps_payload(index_doc, limit=1)
                if top_payload_rows and isinstance(top_payload_rows[0], dict):
                    top_payload = top_payload_rows[0]
                    top_step = str(top_payload.get("step_id", "")).strip() or str(top_payload.get("name", "-")).strip() or "-"
                    top_message = str(top_payload.get("brief", "")).strip() or top_message
                    top_step_cmd = str(top_payload.get("cmd", "")).strip() or top_step_cmd
                    try:
                        top_step_rc = str(int(top_payload.get("returncode", -1)))
                    except Exception:
                        top_step_rc = "-"
                else:
                    name, _, _, brief = failed_row_details(failed_rows[0])
                    top_step = name
                    if brief:
                        top_message = brief
    failed_steps_count = len(failed_steps)
    failed_steps_joined = ",".join(failed_steps[: max(1, limit)]) if failed_steps else "-"
    compact = clip(final_line, 220) if final_line else "-"
    age4_proof_snapshot = load_age4_proof_snapshot(index_doc)
    profile_matrix_snapshot = load_profile_matrix_selftest_snapshot(index_doc)
    age5_policy_snapshot = load_age5_policy_snapshot(index_doc)
    age5_w107_progress_snapshot = load_age5_w107_progress_snapshot(result_doc)
    age5_w107_contract_progress_snapshot = load_age5_w107_contract_progress_snapshot(result_doc)
    age5_age1_immediate_proof_operation_contract_progress_snapshot = (
        load_age5_age1_immediate_proof_operation_contract_progress_snapshot(result_doc)
    )
    age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot = (
        load_age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot(result_doc)
    )
    age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot = (
        load_age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot(result_doc)
    )
    age5_proof_certificate_v1_family_contract_progress_snapshot = (
        load_age5_proof_certificate_v1_family_contract_progress_snapshot(result_doc)
    )
    age5_proof_certificate_family_contract_progress_snapshot = (
        load_age5_proof_certificate_family_contract_progress_snapshot(result_doc)
    )
    age5_proof_family_contract_progress_snapshot = (
        load_age5_proof_family_contract_progress_snapshot(result_doc)
    )
    age5_proof_family_transport_contract_progress_snapshot = (
        load_age5_proof_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_lang_surface_family_contract_progress_snapshot = (
        load_age5_lang_surface_family_contract_progress_snapshot(result_doc)
    )
    age5_lang_runtime_family_contract_progress_snapshot = (
        load_age5_lang_runtime_family_contract_progress_snapshot(result_doc)
    )
    age5_gate0_family_contract_progress_snapshot = (
        load_age5_gate0_family_contract_progress_snapshot(result_doc)
    )
    age5_gate0_surface_family_contract_progress_snapshot = (
        load_age5_gate0_surface_family_contract_progress_snapshot(result_doc)
    )
    age5_gate0_surface_family_transport_contract_progress_snapshot = (
        load_age5_gate0_surface_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_gate0_family_transport_contract_progress_snapshot = (
        load_age5_gate0_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_gate0_transport_family_contract_progress_snapshot = (
        load_age5_gate0_transport_family_contract_progress_snapshot(result_doc)
    )
    age5_gate0_transport_family_transport_contract_progress_snapshot = (
        load_age5_gate0_transport_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_gate0_transport_family_transport_contract_progress_snapshot = (
        load_age5_gate0_transport_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_gate0_transport_family_transport_contract_progress_snapshot = (
        load_age5_gate0_transport_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_lang_surface_family_transport_contract_progress_snapshot = (
        load_age5_lang_surface_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_lang_runtime_family_transport_contract_progress_snapshot = (
        load_age5_lang_runtime_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_gate0_runtime_family_transport_contract_progress_snapshot = (
        load_age5_gate0_runtime_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_proof_certificate_family_transport_contract_progress_snapshot = (
        load_age5_proof_certificate_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_bogae_alias_family_contract_progress_snapshot = (
        load_age5_bogae_alias_family_contract_progress_snapshot(result_doc)
    )
    age5_bogae_alias_family_transport_contract_progress_snapshot = (
        load_age5_bogae_alias_family_transport_contract_progress_snapshot(result_doc)
    )
    tokens = [
        f"status={status}",
        f"reason={quote_token(clip(reason, 180))}",
        f"failed_steps_count={failed_steps_count}",
        f"failed_steps={quote_token(clip(failed_steps_joined, 180))}",
        f"top_step={top_step}",
        f"top_step_rc={top_step_rc}",
        f"top_step_cmd={quote_token(clip(top_step_cmd, 180))}",
        f"top_message={quote_token(clip(top_message, 180))}",
        f"final_line={quote_token(compact)}",
        f"{AGE5_DIGEST_SELFTEST_SUMMARY_KEY}={load_age5_digest_selftest_snapshot(index_doc)}",
        AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT,
        AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_FRAGMENT,
    ]
    tokens.extend(age4_proof_tokens(age4_proof_snapshot))
    tokens.append(
        f"{AGE4_PROOF_FAILED_PREVIEW_KEY}="
        f"{quote_token(str(age4_proof_snapshot.get(AGE4_PROOF_FAILED_PREVIEW_KEY, '-')).strip() or '-')}"
    )
    tokens.extend(
        [
            f"age5_w107_active={age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[0]]}",
            f"age5_w107_inactive={age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[1]]}",
            f"age5_w107_index_codes={age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[2]]}",
            f"age5_w107_current_probe={age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[3]]}",
            f"age5_w107_last_completed_probe={age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[4]]}",
            f"age5_w107_progress={age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[5]]}",
            f"age5_w107_contract_completed={age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[0]]}",
            f"age5_w107_contract_total={age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[1]]}",
            f"age5_w107_contract_checks_text={age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[2]]}",
            f"age5_w107_contract_current_probe={age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[3]]}",
            f"age5_w107_contract_last_completed_probe={age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[4]]}",
            f"age5_w107_contract_progress={age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_age1_immediate_proof_operation_contract_completed="
            f"{age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_age1_immediate_proof_operation_contract_total="
            f"{age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_age1_immediate_proof_operation_contract_checks_text="
            f"{age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_age1_immediate_proof_operation_contract_current_probe="
            f"{age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_age1_immediate_proof_operation_contract_last_completed_probe="
            f"{age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_age1_immediate_proof_operation_contract_progress="
            f"{age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_proof_certificate_v1_consumer_contract_completed="
            f"{age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_proof_certificate_v1_consumer_contract_total="
            f"{age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_proof_certificate_v1_consumer_contract_checks_text="
            f"{age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_proof_certificate_v1_consumer_contract_current_probe="
            f"{age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_proof_certificate_v1_consumer_contract_last_completed_probe="
            f"{age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_proof_certificate_v1_consumer_contract_progress="
            f"{age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_proof_certificate_v1_verify_report_digest_contract_completed="
            f"{age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_proof_certificate_v1_verify_report_digest_contract_total="
            f"{age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_proof_certificate_v1_verify_report_digest_contract_checks_text="
            f"{age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_proof_certificate_v1_verify_report_digest_contract_current_probe="
            f"{age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_proof_certificate_v1_verify_report_digest_contract_last_completed_probe="
            f"{age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_proof_certificate_v1_verify_report_digest_contract_progress="
            f"{age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_proof_certificate_v1_family_contract_completed="
            f"{age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_proof_certificate_v1_family_contract_total="
            f"{age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_proof_certificate_v1_family_contract_checks_text="
            f"{age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_proof_certificate_v1_family_contract_current_probe="
            f"{age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_proof_certificate_v1_family_contract_last_completed_probe="
            f"{age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_proof_certificate_v1_family_contract_progress="
            f"{age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_proof_certificate_family_contract_completed="
            f"{age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_proof_certificate_family_contract_total="
            f"{age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_proof_certificate_family_contract_checks_text="
            f"{age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_proof_certificate_family_contract_current_probe="
            f"{age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_proof_certificate_family_contract_last_completed_probe="
            f"{age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_proof_certificate_family_contract_progress="
            f"{age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_proof_family_contract_completed="
            f"{age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_proof_family_contract_total="
            f"{age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_proof_family_contract_checks_text="
            f"{age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_proof_family_contract_current_probe="
            f"{age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_proof_family_contract_last_completed_probe="
            f"{age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_proof_family_contract_progress="
            f"{age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_proof_family_transport_contract_completed="
            f"{age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_proof_family_transport_contract_total="
            f"{age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_proof_family_transport_contract_checks_text="
            f"{age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_proof_family_transport_contract_current_probe="
            f"{age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_proof_family_transport_contract_last_completed_probe="
            f"{age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_proof_family_transport_contract_progress="
            f"{age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_lang_surface_family_contract_completed="
            f"{age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_lang_surface_family_contract_total="
            f"{age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_lang_surface_family_contract_checks_text="
            f"{age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_lang_surface_family_contract_current_probe="
            f"{age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_lang_surface_family_contract_last_completed_probe="
            f"{age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_lang_surface_family_contract_progress="
            f"{age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_lang_runtime_family_contract_completed="
            f"{age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_lang_runtime_family_contract_total="
            f"{age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_lang_runtime_family_contract_checks_text="
            f"{age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_lang_runtime_family_contract_current_probe="
            f"{age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_lang_runtime_family_contract_last_completed_probe="
            f"{age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_lang_runtime_family_contract_progress="
            f"{age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_gate0_family_contract_completed="
            f"{age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_gate0_family_contract_total="
            f"{age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_gate0_family_contract_checks_text="
            f"{age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_gate0_family_contract_current_probe="
            f"{age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_gate0_family_contract_last_completed_probe="
            f"{age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_gate0_family_contract_progress="
            f"{age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_gate0_surface_family_contract_completed="
            f"{age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_gate0_surface_family_contract_total="
            f"{age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_gate0_surface_family_contract_checks_text="
            f"{age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_gate0_surface_family_contract_current_probe="
            f"{age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_gate0_surface_family_contract_last_completed_probe="
            f"{age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_gate0_surface_family_contract_progress="
            f"{age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_gate0_surface_family_transport_contract_completed="
            f"{age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_gate0_surface_family_transport_contract_total="
            f"{age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_gate0_surface_family_transport_contract_checks_text="
            f"{age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_gate0_surface_family_transport_contract_current_probe="
            f"{age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_gate0_surface_family_transport_contract_last_completed_probe="
            f"{age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_gate0_surface_family_transport_contract_progress="
            f"{age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_gate0_family_transport_contract_completed="
            f"{age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_gate0_family_transport_contract_total="
            f"{age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_gate0_family_transport_contract_checks_text="
            f"{age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_gate0_family_transport_contract_current_probe="
            f"{age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_gate0_family_transport_contract_last_completed_probe="
            f"{age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_gate0_family_transport_contract_progress="
            f"{age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_gate0_transport_family_contract_completed="
            f"{age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_gate0_transport_family_contract_total="
            f"{age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_gate0_transport_family_contract_checks_text="
            f"{age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_gate0_transport_family_contract_current_probe="
            f"{age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_gate0_transport_family_contract_last_completed_probe="
            f"{age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_gate0_transport_family_contract_progress="
            f"{age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_gate0_transport_family_transport_contract_completed="
            f"{age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_gate0_transport_family_transport_contract_total="
            f"{age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_gate0_transport_family_transport_contract_checks_text="
            f"{age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_gate0_transport_family_transport_contract_current_probe="
            f"{age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_gate0_transport_family_transport_contract_last_completed_probe="
            f"{age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_gate0_transport_family_transport_contract_progress="
            f"{age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_lang_runtime_family_transport_contract_completed="
            f"{age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_lang_runtime_family_transport_contract_total="
            f"{age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_lang_runtime_family_transport_contract_checks_text="
            f"{age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_lang_runtime_family_transport_contract_current_probe="
            f"{age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_lang_runtime_family_transport_contract_last_completed_probe="
            f"{age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_lang_runtime_family_transport_contract_progress="
            f"{age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_gate0_runtime_family_transport_contract_completed="
            f"{age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_gate0_runtime_family_transport_contract_total="
            f"{age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_gate0_runtime_family_transport_contract_checks_text="
            f"{age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_gate0_runtime_family_transport_contract_current_probe="
            f"{age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_gate0_runtime_family_transport_contract_last_completed_probe="
            f"{age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_gate0_runtime_family_transport_contract_progress="
            f"{age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_lang_surface_family_transport_contract_completed="
            f"{age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_lang_surface_family_transport_contract_total="
            f"{age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_lang_surface_family_transport_contract_checks_text="
            f"{age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_lang_surface_family_transport_contract_current_probe="
            f"{age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_lang_surface_family_transport_contract_last_completed_probe="
            f"{age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_lang_surface_family_transport_contract_progress="
            f"{age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_proof_certificate_family_transport_contract_completed="
            f"{age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_proof_certificate_family_transport_contract_total="
            f"{age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_proof_certificate_family_transport_contract_checks_text="
            f"{age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_proof_certificate_family_transport_contract_current_probe="
            f"{age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_proof_certificate_family_transport_contract_last_completed_probe="
            f"{age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_proof_certificate_family_transport_contract_progress="
            f"{age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_bogae_alias_family_contract_completed="
            f"{age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_bogae_alias_family_contract_total="
            f"{age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_bogae_alias_family_contract_checks_text="
            f"{age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_bogae_alias_family_contract_current_probe="
            f"{age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_bogae_alias_family_contract_last_completed_probe="
            f"{age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_bogae_alias_family_contract_progress="
            f"{age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_bogae_alias_family_transport_contract_completed="
            f"{age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_bogae_alias_family_transport_contract_total="
            f"{age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_bogae_alias_family_transport_contract_checks_text="
            f"{age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_bogae_alias_family_transport_contract_current_probe="
            f"{age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_bogae_alias_family_transport_contract_last_completed_probe="
            f"{age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_bogae_alias_family_transport_contract_progress="
            f"{age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
        ]
    )
    tokens.extend(age5_child_summary_tokens(load_age5_child_summary_snapshot(index_doc)))
    tokens.extend(age5_policy_tokens(age5_policy_snapshot))
    tokens.extend(profile_matrix_brief_tokens(profile_matrix_snapshot))
    return " ".join(tokens)


def resolve_failure_brief_out(raw: str, prefix: str) -> Path:
    token = "__PREFIX__"
    p = str(raw).strip()
    if token in p:
        resolved_prefix = prefix.strip() or "noprefix"
        p = p.replace(token, resolved_prefix)
    return Path(p)


def write_failure_brief(path: Path, line: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(line.rstrip() + "\n", encoding="utf-8")
    print(f"[ci-final-meta] failure_brief_out={path}")


def failed_steps_payload(index_doc: dict | None, limit: int = 8) -> list[dict[str, object]]:
    if not isinstance(index_doc, dict):
        return []
    steps = index_doc.get("steps")
    if not isinstance(steps, list):
        return []
    summary_detail_rows, summary_log_rows, _, _ = load_summary_failed_step_rows(index_doc)
    out: list[dict[str, object]] = []
    for row in sorted_failed_rows(steps)[: max(1, limit)]:
        if not isinstance(row, dict):
            continue
        name, stdout_log, stderr_log, brief = failed_row_details(row)
        detail_row = summary_detail_rows.get(name, {})
        summary_cmd = str(detail_row.get("cmd", "")).strip()
        try:
            summary_rc = int(detail_row.get("rc", int(row.get("returncode", -1))))
        except Exception:
            summary_rc = int(row.get("returncode", -1))
        cmd_value = row.get("cmd")
        if isinstance(cmd_value, list):
            index_cmd = " ".join(str(part) for part in cmd_value).strip()
        else:
            index_cmd = str(cmd_value).strip()
        cmd_text = clip(summary_cmd or index_cmd or "-", 220)
        log_row = summary_log_rows.get(name, {})
        stdout_path = stdout_log or str(log_row.get("stdout", "")).strip()
        stderr_path = stderr_log or str(log_row.get("stderr", "")).strip()
        ff_detail = f"name={name} rc={summary_rc} cmd={cmd_text}"
        ff_logs = f"name={name} stdout={stdout_path or '-'} stderr={stderr_path or '-'}"
        out.append(
            {
                "step_id": name,
                "name": name,
                "returncode": summary_rc,
                "cmd": cmd_text,
                "cmd_source": "summary" if summary_cmd else ("index" if index_cmd else "-"),
                "fast_fail_step_detail": ff_detail,
                "fast_fail_step_logs": ff_logs,
                "stdout_log_path": stdout_path,
                "stdout_log_path_norm": normalize_path_text(stdout_path),
                "stderr_log_path": stderr_path,
                "stderr_log_path_norm": normalize_path_text(stderr_path),
                "brief": clip(brief, 220) if brief else "",
            }
        )
    return out


def aggregate_digest_payload(index_doc: dict | None, limit: int = 8) -> list[str]:
    if not isinstance(index_doc, dict):
        return []
    aggregate_path = artifact_path(index_doc, "aggregate")
    if aggregate_path is None or not aggregate_path.exists():
        return []
    aggregate_doc = load_json(aggregate_path)
    if not isinstance(aggregate_doc, dict):
        return []
    failure_digest = aggregate_doc.get("failure_digest")
    if isinstance(failure_digest, list) and failure_digest:
        return [clip(str(item), 260) for item in failure_digest[: max(1, limit)]]
    for bucket_key in ("seamgrim", "age3", "oi405_406"):
        bucket = aggregate_doc.get(bucket_key)
        if not isinstance(bucket, dict):
            continue
        digest = bucket.get("failure_digest")
        if not isinstance(digest, list) or not digest:
            continue
        return [clip(f"{bucket_key}:{item}", 260) for item in digest[: max(1, limit)]]
    return []


def artifacts_payload(index_doc: dict | None) -> dict[str, dict[str, object]]:
    out: dict[str, dict[str, object]] = {}
    if not isinstance(index_doc, dict):
        return out
    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return out
    for key in sorted(reports.keys()):
        raw = str(reports.get(key, "")).strip()
        if not raw:
            continue
        path = normalize_path(raw)
        out[str(key)] = {
            "path": raw,
            "path_norm": normalize_path_text(raw),
            "exists": bool(path.exists()),
        }
    return out


def write_triage_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"[ci-final-meta] triage_json_out={path}")


def patch_triage_artifact_row(payload: dict, key: str, path: Path) -> None:
    artifacts = payload.get("artifacts")
    if not isinstance(artifacts, dict):
        artifacts = {}
        payload["artifacts"] = artifacts
    raw = str(path)
    artifacts[key] = {
        "path": raw,
        "path_norm": normalize_path_text(raw),
        "exists": True,
    }


def patch_triage_output_refs(payload: dict, brief_path: Path | None, triage_path: Path | None) -> None:
    if brief_path is not None:
        patch_triage_artifact_row(payload, "ci_fail_brief_txt", brief_path)
    if triage_path is not None:
        patch_triage_artifact_row(payload, "ci_fail_triage_json", triage_path)


def select_latest_index(report_dir: Path, pattern: str, prefix: str) -> tuple[Path | None, dict | None]:
    candidates = sorted(
        report_dir.glob(pattern),
        key=lambda p: (p.stat().st_mtime_ns, str(p)),
        reverse=True,
    )
    selected_path: Path | None = None
    selected_doc: dict | None = None
    for path in candidates:
        doc = load_json(path)
        if not isinstance(doc, dict):
            continue
        if str(doc.get("schema", "")).strip() != INDEX_SCHEMA:
            continue
        if prefix and str(doc.get("report_prefix", "")).strip() != prefix:
            continue
        selected_path = path
        selected_doc = doc
        break
    if selected_path is not None:
        return selected_path, selected_doc
    for path in candidates:
        doc = load_json(path)
        if not isinstance(doc, dict):
            continue
        if str(doc.get("schema", "")).strip() != INDEX_SCHEMA:
            continue
        return path, doc
    return None, None


def first_existing_line(report_dir: Path) -> str:
    for pattern in SUMMARY_PATTERNS:
        files = sorted(
            report_dir.glob(pattern),
            key=lambda p: (p.stat().st_mtime_ns, str(p)),
            reverse=True,
        )
        for path in files:
            line = load_line(path)
            if line:
                print(f"[ci-final-meta] fallback_line_source={path}")
                return line
    return ""


def artifact_path(index_doc: dict, key: str) -> Path | None:
    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return None
    raw_path = str(reports.get(key, "")).strip()
    if not raw_path:
        return None
    return normalize_path(raw_path)


def load_age5_child_summary_snapshot(index_doc: dict | None) -> dict[str, str]:
    snapshot = {key: "skipped" for key in AGE5_CHILD_SUMMARY_KEYS}
    snapshot.update(AGE5_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS)
    if not isinstance(index_doc, dict):
        return snapshot
    aggregate_path = artifact_path(index_doc, "aggregate")
    if aggregate_path is None or not aggregate_path.exists():
        return snapshot
    aggregate_doc = load_json(aggregate_path)
    if not isinstance(aggregate_doc, dict):
        return snapshot
    age5_doc = aggregate_doc.get("age5")
    if not isinstance(age5_doc, dict):
        return snapshot
    for key in AGE5_CHILD_SUMMARY_KEYS:
        value = str(age5_doc.get(key, "")).strip()
        if value in AGE5_CHILD_SUMMARY_VALUES:
            snapshot[key] = value
    for key, expected in AGE5_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS.items():
        value = str(age5_doc.get(key, "")).strip()
        snapshot[key] = value or expected
    return snapshot


def load_age5_digest_selftest_snapshot(index_doc: dict | None) -> str:
    if isinstance(index_doc, dict):
        reports = index_doc.get("reports")
        if isinstance(reports, dict):
            summary_path_raw = str(reports.get("summary", "")).strip()
            if summary_path_raw:
                summary_path = normalize_path(summary_path_raw)
                if summary_path.exists():
                    _, summary_kv, _ = parse_summary_report(summary_path)
                    summary_value = str(summary_kv.get(AGE5_DIGEST_SELFTEST_SUMMARY_KEY, "")).strip()
                    if summary_value in {"0", "1"}:
                        return summary_value
        steps = index_doc.get("steps")
        if isinstance(steps, list):
            for row in steps:
                if not isinstance(row, dict):
                    continue
                if str(row.get("name", "")).strip() != "age5_close_digest_selftest":
                    continue
                ok_value = row.get("ok")
                if isinstance(ok_value, bool):
                    return "1" if ok_value else "0"
                try:
                    return "1" if int(row.get("returncode", 1)) == 0 else "0"
                except Exception:
                    return "0"
    return "0"


def load_age5_w107_progress_snapshot(result_doc: dict | None) -> dict[str, str]:
    snapshot = {
        AGE5_W107_PROGRESS_KEYS[0]: "-",
        AGE5_W107_PROGRESS_KEYS[1]: "-",
        AGE5_W107_PROGRESS_KEYS[2]: "-",
        AGE5_W107_PROGRESS_KEYS[3]: "-",
        AGE5_W107_PROGRESS_KEYS[4]: "-",
        AGE5_W107_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_w107_contract_progress_snapshot(result_doc: dict | None) -> dict[str, str]:
    snapshot = {
        AGE5_W107_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_age1_immediate_proof_operation_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_proof_certificate_v1_family_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_proof_certificate_family_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_proof_family_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_proof_family_transport_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_lang_surface_family_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_lang_runtime_family_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_lang_surface_family_transport_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_lang_runtime_family_transport_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_gate0_family_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_gate0_surface_family_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_gate0_family_transport_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_gate0_transport_family_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_gate0_surface_family_transport_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_gate0_transport_family_transport_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_gate0_runtime_family_transport_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_proof_certificate_family_transport_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_bogae_alias_family_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_bogae_alias_family_transport_contract_progress_snapshot(
    result_doc: dict | None,
) -> dict[str, str]:
    snapshot = {
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
    }
    if not isinstance(result_doc, dict):
        return snapshot
    for key, fallback in snapshot.items():
        snapshot[key] = str(result_doc.get(key, fallback)).strip() or fallback
    return snapshot


def load_age5_policy_snapshot(index_doc: dict | None) -> dict[str, object]:
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
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY: "ok",
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY: 1,
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY: "-",
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY: "-",
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY: (
            build_age5_combined_heavy_policy_origin_trace_contract_compact_reason()
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY: (
            build_age5_combined_heavy_policy_origin_trace_contract_compact_failure_reason()
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY: build_age5_combined_heavy_policy_origin_trace_text(),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY: build_age5_combined_heavy_policy_origin_trace(),
    }
    if not isinstance(index_doc, dict):
        return snapshot
    aggregate_path = artifact_path(index_doc, "aggregate")
    if aggregate_path is None or not aggregate_path.exists():
        return snapshot
    aggregate_doc = load_json(aggregate_path)
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
    contract_status = str(
        age5_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY, "")
    ).strip()
    if contract_status:
        snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY] = contract_status
    snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY] = int(
        bool(age5_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY, False))
    )
    contract_issue = str(
        age5_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY, "")
    ).strip()
    if contract_issue:
        snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY] = contract_issue
    source_contract_issue = str(
        age5_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY, "")
    ).strip()
    if source_contract_issue:
        snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY] = source_contract_issue
    compact_reason = str(
        age5_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY, "")
    ).strip()
    if compact_reason:
        snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY] = compact_reason
    else:
        snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY] = (
            build_age5_combined_heavy_policy_origin_trace_contract_compact_reason(
                snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY],
                snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY],
            )
        )
    compact_failure_reason = str(
        age5_doc.get(
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY,
            "",
        )
    ).strip()
    if compact_failure_reason:
        snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY] = (
            compact_failure_reason
        )
    else:
        snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY] = (
            build_age5_combined_heavy_policy_origin_trace_contract_compact_failure_reason(
                snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY],
                snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY],
            )
        )
    origin_trace = age5_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY)
    if isinstance(origin_trace, dict) and origin_trace:
        snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY] = {
            str(key): str(value) for key, value in origin_trace.items()
        }
    else:
        snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY] = build_age5_combined_heavy_policy_origin_trace(
            report_path=str(snapshot[AGE5_POLICY_REPORT_PATH_KEY]),
            report_exists=snapshot[AGE5_POLICY_REPORT_EXISTS_KEY],
            text_path=str(snapshot[AGE5_POLICY_TEXT_PATH_KEY]),
            text_exists=snapshot[AGE5_POLICY_TEXT_EXISTS_KEY],
            summary_path=str(snapshot[AGE5_POLICY_SUMMARY_PATH_KEY]),
            summary_exists=snapshot[AGE5_POLICY_SUMMARY_EXISTS_KEY],
        )
    origin_trace_text = str(age5_doc.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY, "")).strip()
    if origin_trace_text:
        snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY] = origin_trace_text
    else:
        snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY] = build_age5_combined_heavy_policy_origin_trace_text(
            snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY]
        )
    return snapshot


def load_age4_proof_snapshot(index_doc: dict | None) -> dict[str, object]:
    snapshot: dict[str, object] = {
        AGE4_PROOF_OK_KEY: 0,
        AGE4_PROOF_FAILED_CRITERIA_KEY: -1,
        AGE4_PROOF_FAILED_PREVIEW_KEY: "-",
        AGE4_PROOF_SUMMARY_HASH_KEY: "-",
    }
    if not isinstance(index_doc, dict):
        return snapshot
    aggregate_path = artifact_path(index_doc, "aggregate")
    if aggregate_path is None or not aggregate_path.exists():
        return snapshot
    aggregate_doc = load_json(aggregate_path)
    if not isinstance(aggregate_doc, dict):
        return snapshot
    age4_doc = aggregate_doc.get("age4")
    if not isinstance(age4_doc, dict):
        return snapshot
    snapshot[AGE4_PROOF_OK_KEY] = int(bool(age4_doc.get("proof_artifact_ok", False)))
    failed = age4_doc.get("proof_artifact_failed_criteria")
    if isinstance(failed, list):
        snapshot[AGE4_PROOF_FAILED_CRITERIA_KEY] = len(failed)
    preview = str(age4_doc.get("proof_artifact_failed_preview", "")).strip()
    if preview:
        snapshot[AGE4_PROOF_FAILED_PREVIEW_KEY] = preview
    summary_hash = str(age4_doc.get("proof_artifact_summary_hash", "")).strip()
    if summary_hash:
        snapshot[AGE4_PROOF_SUMMARY_HASH_KEY] = summary_hash
    return snapshot


def age5_child_summary_tokens(snapshot: dict[str, str]) -> list[str]:
    tokens = [
        f"{key}={str(snapshot.get(key, 'skipped')).strip() or 'skipped'}"
        for key in AGE5_CHILD_SUMMARY_KEYS
    ]
    tokens.extend(
        f"{key}={str(snapshot.get(key, expected)).strip() or expected}"
        for key, expected in AGE5_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS.items()
    )
    return tokens


def age4_proof_tokens(snapshot: dict[str, object]) -> list[str]:
    ok = int(bool(snapshot.get(AGE4_PROOF_OK_KEY, 0)))
    try:
        failed = int(snapshot.get(AGE4_PROOF_FAILED_CRITERIA_KEY, -1))
    except Exception:
        failed = -1
    summary_hash = str(snapshot.get(AGE4_PROOF_SUMMARY_HASH_KEY, "-")).strip() or "-"
    return [
        f"{AGE4_PROOF_OK_KEY}={ok}",
        f"{AGE4_PROOF_FAILED_CRITERIA_KEY}={failed}",
        f"{AGE4_PROOF_SUMMARY_HASH_KEY}={summary_hash}",
    ]


def age5_policy_tokens(snapshot: dict[str, object]) -> list[str]:
    field_value = snapshot.get(AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY)
    if not isinstance(field_value, dict):
        field_value = dict(AGE5_DIGEST_SELFTEST_DEFAULT_FIELD)
    age4_snapshot_fields_text = (
        str(snapshot.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY, AGE4_PROOF_SNAPSHOT_FIELDS_TEXT)).strip()
        or AGE4_PROOF_SNAPSHOT_FIELDS_TEXT
    )
    age4_snapshot_text = (
        str(snapshot.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY, "")).strip()
        or build_age4_proof_snapshot_text(build_age4_proof_snapshot())
    )
    age4_source_snapshot_fields_text = (
        str(snapshot.get(AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY, AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT)).strip()
        or AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT
    )
    age4_gate_result_present = (
        str(snapshot.get(AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY, "0")).strip() or "0"
    )
    age4_gate_result_parity = (
        str(snapshot.get(AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY, "0")).strip() or "0"
    )
    age4_final_status_parse_present = (
        str(snapshot.get(AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY, "0")).strip() or "0"
    )
    age4_final_status_parse_parity = (
        str(snapshot.get(AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY, "0")).strip() or "0"
    )
    origin_trace = snapshot.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY)
    if not isinstance(origin_trace, dict):
        origin_trace = build_age5_combined_heavy_policy_origin_trace(
            report_path=str(snapshot.get(AGE5_POLICY_REPORT_PATH_KEY, "-")).strip() or "-",
            report_exists=snapshot.get(AGE5_POLICY_REPORT_EXISTS_KEY, 0),
            text_path=str(snapshot.get(AGE5_POLICY_TEXT_PATH_KEY, "-")).strip() or "-",
            text_exists=snapshot.get(AGE5_POLICY_TEXT_EXISTS_KEY, 0),
            summary_path=str(snapshot.get(AGE5_POLICY_SUMMARY_PATH_KEY, "-")).strip() or "-",
            summary_exists=snapshot.get(AGE5_POLICY_SUMMARY_EXISTS_KEY, 0),
        )
    origin_trace_text = (
        str(
            snapshot.get(
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY,
                build_age5_combined_heavy_policy_origin_trace_text(origin_trace),
            )
        ).strip()
        or build_age5_combined_heavy_policy_origin_trace_text(origin_trace)
    )
    return [
        f"{AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}="
        f"{str(snapshot.get(AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT)).strip() or AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT}",
        f"{AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY}="
        f"{json.dumps(field_value, ensure_ascii=False, sort_keys=True, separators=(',', ':'))}",
        f"{AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY}={age4_snapshot_fields_text}",
        f"{AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY}={age4_snapshot_text}",
        f"{AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY}={age4_source_snapshot_fields_text}",
        f"{AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY}={age4_gate_result_present}",
        f"{AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY}={age4_gate_result_parity}",
        f"{AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY}={age4_final_status_parse_present}",
        f"{AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY}={age4_final_status_parse_parity}",
        f"{AGE5_POLICY_REPORT_PATH_KEY}={str(snapshot.get(AGE5_POLICY_REPORT_PATH_KEY, '-')).strip() or '-'}",
        f"{AGE5_POLICY_REPORT_EXISTS_KEY}={int(bool(snapshot.get(AGE5_POLICY_REPORT_EXISTS_KEY, 0)))}",
        f"{AGE5_POLICY_TEXT_PATH_KEY}={str(snapshot.get(AGE5_POLICY_TEXT_PATH_KEY, '-')).strip() or '-'}",
        f"{AGE5_POLICY_TEXT_EXISTS_KEY}={int(bool(snapshot.get(AGE5_POLICY_TEXT_EXISTS_KEY, 0)))}",
        f"{AGE5_POLICY_SUMMARY_PATH_KEY}={str(snapshot.get(AGE5_POLICY_SUMMARY_PATH_KEY, '-')).strip() or '-'}",
        f"{AGE5_POLICY_SUMMARY_EXISTS_KEY}={int(bool(snapshot.get(AGE5_POLICY_SUMMARY_EXISTS_KEY, 0)))}",
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}="
        f"{str(snapshot.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY, 'ok')).strip() or 'ok'}",
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}="
        f"{int(bool(snapshot.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY, 0)))}",
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}="
        f"{str(snapshot.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY, '-')).strip() or '-'}",
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}="
        f"{str(snapshot.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY, '-')).strip() or '-'}",
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}="
        f"{str(snapshot.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY, '-')).strip() or '-'}",
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY}="
        f"{str(snapshot.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY, '-')).strip() or '-'}",
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}={origin_trace_text}",
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY}={json.dumps(origin_trace, ensure_ascii=False, sort_keys=True, separators=(',', ':'))}",
    ]


def load_profile_matrix_selftest_snapshot(index_doc: dict | None) -> dict[str, object]:
    default_snapshot: dict[str, object] = {
        output_key: default_value
        for output_key, _, __, default_value in PROFILE_MATRIX_SNAPSHOT_FIELD_SPECS
    }
    if not isinstance(index_doc, dict):
        return default_snapshot
    report_path = artifact_path(index_doc, "ci_profile_matrix_gate_selftest")
    if report_path is None or not report_path.exists():
        return default_snapshot
    doc = load_json(report_path)
    if not isinstance(doc, dict):
        return default_snapshot
    if str(doc.get("schema", "")).strip() != PROFILE_MATRIX_SELFTEST_SCHEMA:
        return default_snapshot
    real_profiles = doc.get("real_profiles")

    def read_names(key: str) -> list[str]:
        raw = doc.get(key)
        if not isinstance(raw, list):
            return []
        return [str(item).strip() for item in raw if str(item).strip()]

    def read_elapsed(profile_name: str) -> int | None:
        if not isinstance(real_profiles, dict):
            return None
        row = real_profiles.get(profile_name)
        if not isinstance(row, dict):
            return None
        raw = row.get("total_elapsed_ms")
        if raw is None:
            return None
        try:
            return max(0, int(raw))
        except Exception:
            return None

    def read_names_list(key: str) -> list[str]:
        raw = doc.get(key)
        if not isinstance(raw, list):
            return []
        return [str(item).strip() for item in raw if str(item).strip()]

    def read_aggregate_row(profile_name: str) -> dict | None:
        block = doc.get("aggregate_summary_sanity_by_profile")
        if not isinstance(block, dict):
            return None
        row = block.get(profile_name)
        return row if isinstance(row, dict) else None

    def read_aggregate_values(profile_name: str) -> str:
        row = read_aggregate_row(profile_name)
        if not isinstance(row, dict):
            return "-"
        values = row.get("values")
        if not isinstance(values, dict):
            return "-"
        parts = [str(values.get(key, "")).strip() or "-" for key in PROFILE_MATRIX_AGGREGATE_SUMMARY_VALUE_KEYS]
        return "/".join(parts)

    def read_timeout_defaults_sec() -> dict[str, float]:
        raw = doc.get("step_timeout_defaults_sec")
        source = raw if isinstance(raw, dict) else {}
        result: dict[str, float] = {}
        for profile_name in ("core_lang", "full", "seamgrim"):
            fallback = float(PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC[profile_name])
            value = source.get(profile_name, fallback) if isinstance(source, dict) else fallback
            try:
                result[profile_name] = float(value)
            except Exception:
                result[profile_name] = fallback
        return result

    def read_timeout_env_keys() -> dict[str, str]:
        raw = doc.get("step_timeout_env_keys")
        source = raw if isinstance(raw, dict) else {}
        result: dict[str, str] = {}
        for profile_name in ("core_lang", "full", "seamgrim"):
            fallback = str(PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS[profile_name]).strip()
            value = source.get(profile_name, fallback) if isinstance(source, dict) else fallback
            text = str(value).strip()
            result[profile_name] = text or fallback
        return result

    snapshot: dict[str, object] = {}
    for output_key, field_kind, source_key, default_value in PROFILE_MATRIX_SNAPSHOT_FIELD_SPECS:
        if field_kind == "report_path":
            snapshot[output_key] = str(report_path)
            continue
        if field_kind == "text":
            snapshot[output_key] = str(doc.get(source_key, default_value)).strip() or default_value
            continue
        if field_kind == "bool":
            snapshot[output_key] = bool(doc.get(source_key, default_value))
            continue
        if field_kind == "names":
            snapshot[output_key] = read_names(source_key)
            continue
        if field_kind == "elapsed":
            snapshot[output_key] = read_elapsed(source_key)
            continue
        if field_kind == "dict_float":
            snapshot[output_key] = read_timeout_defaults_sec()
            continue
        if field_kind == "dict_text":
            snapshot[output_key] = read_timeout_env_keys()
            continue
        if field_kind == "int":
            raw = doc.get(source_key)
            try:
                snapshot[output_key] = max(0, int(raw))
            except Exception:
                snapshot[output_key] = default_value
            continue
        snapshot[output_key] = default_value
    snapshot["aggregate_summary_sanity_ok"] = bool(doc.get("aggregate_summary_sanity_ok", False))
    snapshot["aggregate_summary_sanity_checked_profiles"] = read_names_list("aggregate_summary_sanity_checked_profiles")
    snapshot["aggregate_summary_sanity_failed_profiles"] = read_names_list("aggregate_summary_sanity_failed_profiles")
    snapshot["aggregate_summary_sanity_skipped_profiles"] = read_names_list("aggregate_summary_sanity_skipped_profiles")
    for profile_name in ("core_lang", "full", "seamgrim"):
        row = read_aggregate_row(profile_name)
        snapshot[f"{profile_name}_aggregate_summary_status"] = (
            str(row.get("status", "")).strip() if isinstance(row, dict) else "-"
        ) or "-"
        snapshot[f"{profile_name}_aggregate_summary_ok"] = bool(row.get("ok", False)) if isinstance(row, dict) else False
        snapshot[f"{profile_name}_aggregate_summary_values"] = read_aggregate_values(profile_name)
    return snapshot


def brief_profile_matrix_token(snapshot: dict[str, object], key: str) -> str:
    value = snapshot.get(key)
    if isinstance(value, list):
        names = [str(item).strip() for item in value if str(item).strip()]
        return ",".join(names) if names else "-"
    if value is None:
        return "-"
    if isinstance(value, bool):
        return "1" if value else "0"
    return str(value).strip() or "-"


def build_profile_matrix_tokens(
    snapshot: dict[str, object],
    specs: tuple[tuple[str, str, bool, int], ...],
) -> list[str]:
    tokens: list[str] = []
    for output_key, snapshot_key, quoted, clip_limit in specs:
        value = brief_profile_matrix_token(snapshot, snapshot_key)
        if clip_limit > 0:
            value = clip(value, clip_limit)
        if quoted:
            value = quote_token(value)
        tokens.append(f"{output_key}={value}")
    return tokens


def profile_matrix_stdout_tokens(snapshot: dict[str, object]) -> list[str]:
    total_elapsed = brief_profile_matrix_token(snapshot, "total_elapsed_ms")
    if total_elapsed == "-":
        return []
    return build_profile_matrix_tokens(snapshot, PROFILE_MATRIX_STDOUT_TOKEN_SPECS)


def profile_matrix_brief_tokens(snapshot: dict[str, object]) -> list[str]:
    return build_profile_matrix_tokens(snapshot, PROFILE_MATRIX_BRIEF_TOKEN_SPECS)


def render_ci_final_stdout_line(final_line: str, index_doc: dict | None) -> str:
    compact = clip(final_line, 360) if final_line else "-"
    age4_proof_snapshot = load_age4_proof_snapshot(index_doc)
    profile_matrix_snapshot = load_profile_matrix_selftest_snapshot(index_doc)
    age5_child_snapshot = load_age5_child_summary_snapshot(index_doc)
    age5_digest_selftest_snapshot = load_age5_digest_selftest_snapshot(index_doc)
    age5_policy_snapshot = load_age5_policy_snapshot(index_doc)
    result_doc = load_result_doc(index_doc) if isinstance(index_doc, dict) else None
    age5_w107_progress_snapshot = load_age5_w107_progress_snapshot(result_doc)
    age5_w107_contract_progress_snapshot = load_age5_w107_contract_progress_snapshot(result_doc)
    age5_age1_immediate_proof_operation_contract_progress_snapshot = (
        load_age5_age1_immediate_proof_operation_contract_progress_snapshot(result_doc)
    )
    age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot = (
        load_age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot(result_doc)
    )
    age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot = (
        load_age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot(result_doc)
    )
    age5_proof_certificate_v1_family_contract_progress_snapshot = (
        load_age5_proof_certificate_v1_family_contract_progress_snapshot(result_doc)
    )
    age5_proof_certificate_family_contract_progress_snapshot = (
        load_age5_proof_certificate_family_contract_progress_snapshot(result_doc)
    )
    age5_proof_family_contract_progress_snapshot = (
        load_age5_proof_family_contract_progress_snapshot(result_doc)
    )
    age5_proof_family_transport_contract_progress_snapshot = (
        load_age5_proof_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_lang_surface_family_contract_progress_snapshot = (
        load_age5_lang_surface_family_contract_progress_snapshot(result_doc)
    )
    age5_lang_runtime_family_contract_progress_snapshot = (
        load_age5_lang_runtime_family_contract_progress_snapshot(result_doc)
    )
    age5_gate0_family_contract_progress_snapshot = (
        load_age5_gate0_family_contract_progress_snapshot(result_doc)
    )
    age5_gate0_surface_family_contract_progress_snapshot = (
        load_age5_gate0_surface_family_contract_progress_snapshot(result_doc)
    )
    age5_gate0_surface_family_transport_contract_progress_snapshot = (
        load_age5_gate0_surface_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_gate0_family_transport_contract_progress_snapshot = (
        load_age5_gate0_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_gate0_transport_family_contract_progress_snapshot = (
        load_age5_gate0_transport_family_contract_progress_snapshot(result_doc)
    )
    age5_gate0_transport_family_transport_contract_progress_snapshot = (
        load_age5_gate0_transport_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_lang_surface_family_transport_contract_progress_snapshot = (
        load_age5_lang_surface_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_lang_runtime_family_transport_contract_progress_snapshot = (
        load_age5_lang_runtime_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_gate0_runtime_family_transport_contract_progress_snapshot = (
        load_age5_gate0_runtime_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_proof_certificate_family_transport_contract_progress_snapshot = (
        load_age5_proof_certificate_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_bogae_alias_family_contract_progress_snapshot = (
        load_age5_bogae_alias_family_contract_progress_snapshot(result_doc)
    )
    age5_bogae_alias_family_transport_contract_progress_snapshot = (
        load_age5_bogae_alias_family_transport_contract_progress_snapshot(result_doc)
    )
    tokens = profile_matrix_stdout_tokens(profile_matrix_snapshot)
    tokens.append(f"{AGE5_DIGEST_SELFTEST_SUMMARY_KEY}={age5_digest_selftest_snapshot}")
    tokens.append(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT)
    tokens.append(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_FRAGMENT)
    tokens.extend(age4_proof_tokens(age4_proof_snapshot))
    tokens.extend(
        [
            f"age5_w107_active={age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[0]]}",
            f"age5_w107_inactive={age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[1]]}",
            f"age5_w107_index_codes={age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[2]]}",
            f"age5_w107_current_probe={age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[3]]}",
            f"age5_w107_last_completed_probe={age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[4]]}",
            f"age5_w107_progress={age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[5]]}",
            f"age5_w107_contract_completed={age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[0]]}",
            f"age5_w107_contract_total={age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[1]]}",
            f"age5_w107_contract_checks_text={age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[2]]}",
            f"age5_w107_contract_current_probe={age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[3]]}",
            f"age5_w107_contract_last_completed_probe={age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[4]]}",
            f"age5_w107_contract_progress={age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_age1_immediate_proof_operation_contract_completed="
            f"{age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_age1_immediate_proof_operation_contract_total="
            f"{age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_age1_immediate_proof_operation_contract_checks_text="
            f"{age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_age1_immediate_proof_operation_contract_current_probe="
            f"{age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_age1_immediate_proof_operation_contract_last_completed_probe="
            f"{age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_age1_immediate_proof_operation_contract_progress="
            f"{age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_proof_certificate_v1_consumer_contract_completed="
            f"{age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_proof_certificate_v1_consumer_contract_total="
            f"{age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_proof_certificate_v1_consumer_contract_checks_text="
            f"{age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_proof_certificate_v1_consumer_contract_current_probe="
            f"{age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_proof_certificate_v1_consumer_contract_last_completed_probe="
            f"{age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_proof_certificate_v1_consumer_contract_progress="
            f"{age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_proof_certificate_v1_verify_report_digest_contract_completed="
            f"{age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_proof_certificate_v1_verify_report_digest_contract_total="
            f"{age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_proof_certificate_v1_verify_report_digest_contract_checks_text="
            f"{age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_proof_certificate_v1_verify_report_digest_contract_current_probe="
            f"{age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_proof_certificate_v1_verify_report_digest_contract_last_completed_probe="
            f"{age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_proof_certificate_v1_verify_report_digest_contract_progress="
            f"{age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_proof_certificate_v1_family_contract_completed="
            f"{age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_proof_certificate_v1_family_contract_total="
            f"{age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_proof_certificate_v1_family_contract_checks_text="
            f"{age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_proof_certificate_v1_family_contract_current_probe="
            f"{age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_proof_certificate_v1_family_contract_last_completed_probe="
            f"{age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_proof_certificate_v1_family_contract_progress="
            f"{age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_proof_certificate_family_contract_completed="
            f"{age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_proof_certificate_family_contract_total="
            f"{age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_proof_certificate_family_contract_checks_text="
            f"{age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_proof_certificate_family_contract_current_probe="
            f"{age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_proof_certificate_family_contract_last_completed_probe="
            f"{age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_proof_certificate_family_contract_progress="
            f"{age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_proof_family_contract_completed="
            f"{age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_proof_family_contract_total="
            f"{age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_proof_family_contract_checks_text="
            f"{age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_proof_family_contract_current_probe="
            f"{age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_proof_family_contract_last_completed_probe="
            f"{age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_proof_family_contract_progress="
            f"{age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_proof_family_transport_contract_completed="
            f"{age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_proof_family_transport_contract_total="
            f"{age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_proof_family_transport_contract_checks_text="
            f"{age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_proof_family_transport_contract_current_probe="
            f"{age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_proof_family_transport_contract_last_completed_probe="
            f"{age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_proof_family_transport_contract_progress="
            f"{age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_lang_surface_family_contract_completed="
            f"{age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_lang_surface_family_contract_total="
            f"{age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_lang_surface_family_contract_checks_text="
            f"{age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_lang_surface_family_contract_current_probe="
            f"{age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_lang_surface_family_contract_last_completed_probe="
            f"{age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_lang_surface_family_contract_progress="
            f"{age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_lang_runtime_family_contract_completed="
            f"{age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_lang_runtime_family_contract_total="
            f"{age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_lang_runtime_family_contract_checks_text="
            f"{age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_lang_runtime_family_contract_current_probe="
            f"{age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_lang_runtime_family_contract_last_completed_probe="
            f"{age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_lang_runtime_family_contract_progress="
            f"{age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_gate0_family_contract_completed="
            f"{age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_gate0_family_contract_total="
            f"{age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_gate0_family_contract_checks_text="
            f"{age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_gate0_family_contract_current_probe="
            f"{age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_gate0_family_contract_last_completed_probe="
            f"{age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_gate0_family_contract_progress="
            f"{age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_gate0_surface_family_contract_completed="
            f"{age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_gate0_surface_family_contract_total="
            f"{age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_gate0_surface_family_contract_checks_text="
            f"{age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_gate0_surface_family_contract_current_probe="
            f"{age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_gate0_surface_family_contract_last_completed_probe="
            f"{age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_gate0_surface_family_contract_progress="
            f"{age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_gate0_surface_family_transport_contract_completed="
            f"{age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_gate0_surface_family_transport_contract_total="
            f"{age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_gate0_surface_family_transport_contract_checks_text="
            f"{age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_gate0_surface_family_transport_contract_current_probe="
            f"{age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_gate0_surface_family_transport_contract_last_completed_probe="
            f"{age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_gate0_surface_family_transport_contract_progress="
            f"{age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_gate0_family_transport_contract_completed="
            f"{age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_gate0_family_transport_contract_total="
            f"{age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_gate0_family_transport_contract_checks_text="
            f"{age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_gate0_family_transport_contract_current_probe="
            f"{age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_gate0_family_transport_contract_last_completed_probe="
            f"{age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_gate0_family_transport_contract_progress="
            f"{age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_gate0_transport_family_contract_completed="
            f"{age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_gate0_transport_family_contract_total="
            f"{age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_gate0_transport_family_contract_checks_text="
            f"{age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_gate0_transport_family_contract_current_probe="
            f"{age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_gate0_transport_family_contract_last_completed_probe="
            f"{age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_gate0_transport_family_contract_progress="
            f"{age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_gate0_transport_family_transport_contract_completed="
            f"{age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_gate0_transport_family_transport_contract_total="
            f"{age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_gate0_transport_family_transport_contract_checks_text="
            f"{age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_gate0_transport_family_transport_contract_current_probe="
            f"{age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_gate0_transport_family_transport_contract_last_completed_probe="
            f"{age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_gate0_transport_family_transport_contract_progress="
            f"{age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_lang_runtime_family_transport_contract_completed="
            f"{age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_lang_runtime_family_transport_contract_total="
            f"{age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_lang_runtime_family_transport_contract_checks_text="
            f"{age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_lang_runtime_family_transport_contract_current_probe="
            f"{age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_lang_runtime_family_transport_contract_last_completed_probe="
            f"{age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_lang_runtime_family_transport_contract_progress="
            f"{age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_gate0_runtime_family_transport_contract_completed="
            f"{age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_gate0_runtime_family_transport_contract_total="
            f"{age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_gate0_runtime_family_transport_contract_checks_text="
            f"{age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_gate0_runtime_family_transport_contract_current_probe="
            f"{age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_gate0_runtime_family_transport_contract_last_completed_probe="
            f"{age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_gate0_runtime_family_transport_contract_progress="
            f"{age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_lang_surface_family_transport_contract_completed="
            f"{age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_lang_surface_family_transport_contract_total="
            f"{age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_lang_surface_family_transport_contract_checks_text="
            f"{age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_lang_surface_family_transport_contract_current_probe="
            f"{age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_lang_surface_family_transport_contract_last_completed_probe="
            f"{age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_lang_surface_family_transport_contract_progress="
            f"{age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_proof_certificate_family_transport_contract_completed="
            f"{age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_proof_certificate_family_transport_contract_total="
            f"{age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_proof_certificate_family_transport_contract_checks_text="
            f"{age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_proof_certificate_family_transport_contract_current_probe="
            f"{age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_proof_certificate_family_transport_contract_last_completed_probe="
            f"{age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_proof_certificate_family_transport_contract_progress="
            f"{age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_bogae_alias_family_contract_completed="
            f"{age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_bogae_alias_family_contract_total="
            f"{age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_bogae_alias_family_contract_checks_text="
            f"{age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_bogae_alias_family_contract_current_probe="
            f"{age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_bogae_alias_family_contract_last_completed_probe="
            f"{age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_bogae_alias_family_contract_progress="
            f"{age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
            "age5_bogae_alias_family_transport_contract_completed="
            f"{age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
            "age5_bogae_alias_family_transport_contract_total="
            f"{age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
            "age5_bogae_alias_family_transport_contract_checks_text="
            f"{age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
            "age5_bogae_alias_family_transport_contract_current_probe="
            f"{age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
            "age5_bogae_alias_family_transport_contract_last_completed_probe="
            f"{age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
            "age5_bogae_alias_family_transport_contract_progress="
            f"{age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
        ]
    )
    tokens.extend(age5_child_summary_tokens(age5_child_snapshot))
    tokens.extend(age5_policy_tokens(age5_policy_snapshot))
    if not tokens:
        return compact
    return f"{compact} {' '.join(tokens)}"


def print_artifact_lines(index_doc: dict) -> None:
    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return
    for key in ARTIFACT_KEYS:
        raw_path = str(reports.get(key, "")).strip()
        if not raw_path:
            continue
        path = normalize_path(raw_path)
        print(f"[ci-artifact] key={key} exists={int(path.exists())} path={path}")


def print_result_meta(index_doc: dict) -> None:
    result_path = artifact_path(index_doc, "ci_gate_result_json")
    if result_path is None or not result_path.exists():
        return
    result_doc = load_json(result_path)
    if not isinstance(result_doc, dict):
        return
    status = str(result_doc.get("status", "-")).strip() or "-"
    ok = int(bool(result_doc.get("ok", False)))
    failed_steps = result_doc.get("failed_steps", "-")
    aggregate_status = str(result_doc.get("aggregate_status", "-")).strip() or "-"
    print(
        f"[ci-final-meta] result_status={status} ok={ok} "
        f"failed_steps={failed_steps} aggregate_status={aggregate_status}"
    )
    badge_path = artifact_path(index_doc, "ci_gate_badge_json")
    if badge_path is None or not badge_path.exists():
        return
    badge_doc = load_json(badge_path)
    if not isinstance(badge_doc, dict):
        return
    badge_status = str(badge_doc.get("status", "-")).strip() or "-"
    color = str(badge_doc.get("color", "-")).strip() or "-"
    print(f"[ci-final-meta] badge_status={badge_status} badge_color={color}")


def default_report_dir() -> str:
    preferred = Path("I:/home/urihanl/ddn/codex/build/reports")
    if os.name == "nt":
        try:
            preferred.mkdir(parents=True, exist_ok=True)
        except OSError:
            pass
        return str(preferred)
    return "build/reports"


def parse_summary_report(path: Path) -> tuple[str | None, dict[str, str], list[tuple[str, str]]]:
    try:
        text = path.read_text(encoding="utf-8-sig")
    except Exception:
        return None, {}, []
    status: str | None = None
    kv: dict[str, str] = {}
    rows: list[tuple[str, str]] = []
    for raw in text.splitlines():
        line = str(raw).strip()
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
        if not key:
            continue
        kv[key] = value
        rows.append((key, value))
    return status, kv, rows


def summary_failed_step_names(value: str) -> list[str]:
    raw = str(value).strip()
    if not raw or raw == "(none)":
        return []
    return [item.strip() for item in raw.split(",") if item.strip()]


def verify_summary_report(index_doc: dict, result_doc: dict | None) -> tuple[bool, list[str], int, int]:
    def add_issue(out: list[str], code: str) -> None:
        if code not in out:
            out.append(code)

    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return False, [VERIFY_CODES["REPORTS_MISSING"]], 0, 0
    summary_raw = str(reports.get("summary", "")).strip()
    if not summary_raw:
        return False, [VERIFY_CODES["SUMMARY_PATH_MISSING"]], 0, 0
    summary_path = normalize_path(summary_raw)
    if not summary_path.exists():
        return False, [VERIFY_CODES["SUMMARY_FILE_MISSING"]], 0, 0
    status, kv, rows = parse_summary_report(summary_path)
    if status not in {"pass", "fail"}:
        return False, [VERIFY_CODES["STATUS_MISSING"]], 0, 0

    expected_status = str(result_doc.get("status", "")).strip() if isinstance(result_doc, dict) else ""
    issues: list[str] = []
    if expected_status in {"pass", "fail"} and expected_status != status:
        add_issue(issues, VERIFY_CODES["STATUS_MISMATCH"])

    detail_rows = [value for key, value in rows if key == "failed_step_detail"]
    log_rows = [value for key, value in rows if key == "failed_step_logs"]

    if status == "pass":
        if str(kv.get("failed_steps", "")).strip() != "(none)":
            add_issue(issues, VERIFY_CODES["PASS_FAILED_STEPS_NOT_NONE"])
        if detail_rows:
            add_issue(issues, VERIFY_CODES["PASS_HAS_DETAIL"])
        if log_rows:
            add_issue(issues, VERIFY_CODES["PASS_HAS_LOGS"])
        return len(issues) == 0, issues, len(detail_rows), len(log_rows)

    failed_steps = summary_failed_step_names(str(kv.get("failed_steps", "")))
    if not failed_steps:
        add_issue(issues, VERIFY_CODES["FAIL_FAILED_STEPS_EMPTY"])

    parsed_detail_steps: list[str] = []
    for row in detail_rows:
        match = SUMMARY_DETAIL_RE.match(f"failed_step_detail={row}")
        if not match:
            add_issue(issues, VERIFY_CODES["DETAIL_FORMAT_INVALID"])
            continue
        name = str(match.group(1)).strip()
        rc = int(match.group(2))
        cmd = str(match.group(3)).strip()
        parsed_detail_steps.append(name)
        if rc == 0:
            add_issue(issues, VERIFY_CODES["DETAIL_RC_ZERO"])
        if not cmd:
            add_issue(issues, VERIFY_CODES["DETAIL_CMD_EMPTY"])
        if failed_steps and name not in failed_steps:
            add_issue(issues, VERIFY_CODES["DETAIL_NOT_IN_FAILED_STEPS"])
    if not detail_rows:
        add_issue(issues, VERIFY_CODES["FAIL_DETAIL_MISSING"])

    parsed_log_steps: list[str] = []
    for row in log_rows:
        match = SUMMARY_LOGS_RE.match(f"failed_step_logs={row}")
        if not match:
            add_issue(issues, VERIFY_CODES["LOGS_FORMAT_INVALID"])
            continue
        name = str(match.group(1)).strip()
        stdout_path = str(match.group(2)).strip()
        stderr_path = str(match.group(3)).strip()
        parsed_log_steps.append(name)
        if failed_steps and name not in failed_steps:
            add_issue(issues, VERIFY_CODES["LOGS_NOT_IN_FAILED_STEPS"])
        for raw_path in (stdout_path, stderr_path):
            if raw_path == "-":
                continue
            path = normalize_path(raw_path)
            if not path.exists():
                add_issue(issues, VERIFY_CODES["LOG_PATH_MISSING"])

    steps = index_doc.get("steps")
    if isinstance(steps, list):
        index_failed = [str(row.get("name", "")).strip() for row in steps if isinstance(row, dict) and not bool(row.get("ok", False))]
        index_failed_set = {name for name in index_failed if name}
        if index_failed_set and failed_steps:
            for name in failed_steps:
                if name not in index_failed_set:
                    add_issue(issues, VERIFY_CODES["SUMMARY_FAILED_STEP_NOT_IN_INDEX"])
        for name in parsed_detail_steps:
            if index_failed_set and name not in index_failed_set:
                add_issue(issues, VERIFY_CODES["DETAIL_NOT_IN_INDEX"])
        for name in parsed_log_steps:
            if index_failed_set and name not in index_failed_set:
                add_issue(issues, VERIFY_CODES["LOGS_NOT_IN_INDEX"])
    return len(issues) == 0, issues, len(detail_rows), len(log_rows)


def print_summary_verify(index_doc: dict, result_doc: dict | None) -> bool:
    summary_ok, summary_issues, detail_count, logs_count = verify_summary_report(index_doc, result_doc)
    issue_count = len(summary_issues)
    top_issue_code = summary_issues[0] if summary_issues else "-"
    top_issues = ",".join(summary_issues[:3]) if summary_issues else "-"
    if summary_ok:
        print(
            f"[ci-fail-verify] summary=ok detail_rows={detail_count} "
            f"log_rows={logs_count} issue_count={issue_count} "
            f"top_issue_code={top_issue_code} top_issues={top_issues}"
        )
        return True
    print(
        f"[ci-fail-verify] summary=fail detail_rows={detail_count} "
        f"log_rows={logs_count} issue_count={issue_count} "
        f"top_issue_code={top_issue_code} top_issues={clip(top_issues, 260)}"
    )
    return False


def build_triage_payload(
    index_doc: dict | None,
    result_doc: dict | None,
    final_line: str,
    summary_verify_ok: bool | None = None,
    summary_verify_issues: list[str] | None = None,
    max_steps: int = 8,
    max_digest: int = 8,
) -> dict:
    status = str(result_doc.get("status", "unknown")).strip() if isinstance(result_doc, dict) else "unknown"
    reason = str(result_doc.get("reason", "-")).strip() if isinstance(result_doc, dict) else "-"
    if not status:
        status = "unknown"
    if not reason:
        reason = "-"
    prefix = str(index_doc.get("report_prefix", "")).strip() if isinstance(index_doc, dict) else ""
    summary_path_hint = "-"
    if isinstance(index_doc, dict):
        reports = index_doc.get("reports")
        if isinstance(reports, dict):
            summary_path_hint = str(reports.get("summary", "")).strip() or "-"
    failed_steps = failed_steps_payload(index_doc, limit=max_steps)
    summary_detail_rows, summary_log_rows, summary_detail_order, summary_log_order = load_summary_failed_step_rows(index_doc)
    digest = aggregate_digest_payload(index_doc, limit=max_digest)
    if summary_verify_ok is None:
        if isinstance(index_doc, dict):
            summary_verify_ok, verify_issues, _, _ = verify_summary_report(index_doc, result_doc)
            if summary_verify_issues is None:
                summary_verify_issues = verify_issues
        else:
            summary_verify_ok = False
    if summary_verify_issues is None:
        summary_verify_issues = []
    summary_verify_issue_codes = [str(item) for item in summary_verify_issues[:16]]
    summary_verify_top_issue = summary_verify_issue_codes[0] if summary_verify_issue_codes else "-"
    age4_proof_snapshot = load_age4_proof_snapshot(index_doc)
    profile_matrix_snapshot = load_profile_matrix_selftest_snapshot(index_doc)
    age5_child_snapshot = load_age5_child_summary_snapshot(index_doc)
    age5_digest_selftest_snapshot = load_age5_digest_selftest_snapshot(index_doc)
    age5_policy_snapshot = load_age5_policy_snapshot(index_doc)
    age5_w107_progress_snapshot = load_age5_w107_progress_snapshot(result_doc)
    age5_w107_contract_progress_snapshot = load_age5_w107_contract_progress_snapshot(result_doc)
    age5_age1_immediate_proof_operation_contract_progress_snapshot = (
        load_age5_age1_immediate_proof_operation_contract_progress_snapshot(result_doc)
    )
    age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot = (
        load_age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot(result_doc)
    )
    age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot = (
        load_age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot(result_doc)
    )
    age5_proof_certificate_v1_family_contract_progress_snapshot = (
        load_age5_proof_certificate_v1_family_contract_progress_snapshot(result_doc)
    )
    age5_proof_certificate_family_contract_progress_snapshot = (
        load_age5_proof_certificate_family_contract_progress_snapshot(result_doc)
    )
    age5_proof_family_contract_progress_snapshot = (
        load_age5_proof_family_contract_progress_snapshot(result_doc)
    )
    age5_proof_family_transport_contract_progress_snapshot = (
        load_age5_proof_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_lang_surface_family_contract_progress_snapshot = (
        load_age5_lang_surface_family_contract_progress_snapshot(result_doc)
    )
    age5_lang_runtime_family_contract_progress_snapshot = (
        load_age5_lang_runtime_family_contract_progress_snapshot(result_doc)
    )
    age5_gate0_family_contract_progress_snapshot = (
        load_age5_gate0_family_contract_progress_snapshot(result_doc)
    )
    age5_gate0_surface_family_contract_progress_snapshot = (
        load_age5_gate0_surface_family_contract_progress_snapshot(result_doc)
    )
    age5_gate0_surface_family_transport_contract_progress_snapshot = (
        load_age5_gate0_surface_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_gate0_family_transport_contract_progress_snapshot = (
        load_age5_gate0_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_gate0_transport_family_contract_progress_snapshot = (
        load_age5_gate0_transport_family_contract_progress_snapshot(result_doc)
    )
    age5_gate0_transport_family_transport_contract_progress_snapshot = (
        load_age5_gate0_transport_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_lang_surface_family_transport_contract_progress_snapshot = (
        load_age5_lang_surface_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_lang_runtime_family_transport_contract_progress_snapshot = (
        load_age5_lang_runtime_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_gate0_runtime_family_transport_contract_progress_snapshot = (
        load_age5_gate0_runtime_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_proof_certificate_family_transport_contract_progress_snapshot = (
        load_age5_proof_certificate_family_transport_contract_progress_snapshot(result_doc)
    )
    age5_bogae_alias_family_contract_progress_snapshot = (
        load_age5_bogae_alias_family_contract_progress_snapshot(result_doc)
    )
    age5_bogae_alias_family_transport_contract_progress_snapshot = (
        load_age5_bogae_alias_family_transport_contract_progress_snapshot(result_doc)
    )
    payload = {
        "schema": "ddn.ci.fail_triage.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": status,
        "reason": reason,
        "report_prefix": prefix,
        "final_line": clip(final_line, 360) if final_line else "-",
        "summary_verify_ok": bool(summary_verify_ok),
        "summary_verify_issues": summary_verify_issue_codes,
        "summary_verify_issues_count": len(summary_verify_issue_codes),
        "summary_verify_top_issue": summary_verify_top_issue,
        "failed_steps": failed_steps,
        "failed_steps_count": len(failed_steps),
        "failed_step_detail_rows_count": len(summary_detail_rows),
        "failed_step_logs_rows_count": len(summary_log_rows),
        "failed_step_detail_order": list(summary_detail_order),
        "failed_step_logs_order": list(summary_log_order),
        "aggregate_digest": digest,
        "aggregate_digest_count": len(digest),
        "summary_report_path_hint": summary_path_hint,
        "summary_report_path_hint_norm": normalize_path_text(summary_path_hint) if summary_path_hint != "-" else "-",
        "profile_matrix_selftest": profile_matrix_snapshot,
        AGE4_PROOF_OK_KEY: int(bool(age4_proof_snapshot.get(AGE4_PROOF_OK_KEY, 0))),
        AGE4_PROOF_FAILED_CRITERIA_KEY: int(age4_proof_snapshot.get(AGE4_PROOF_FAILED_CRITERIA_KEY, -1)),
        AGE4_PROOF_FAILED_PREVIEW_KEY: str(
            age4_proof_snapshot.get(AGE4_PROOF_FAILED_PREVIEW_KEY, "-")
        ).strip()
        or "-",
        AGE4_PROOF_SUMMARY_HASH_KEY: str(age4_proof_snapshot.get(AGE4_PROOF_SUMMARY_HASH_KEY, "-")).strip() or "-",
        AGE5_W107_PROGRESS_KEYS[0]: age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[0]],
        AGE5_W107_PROGRESS_KEYS[1]: age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[1]],
        AGE5_W107_PROGRESS_KEYS[2]: age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[2]],
        AGE5_W107_PROGRESS_KEYS[3]: age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[3]],
        AGE5_W107_PROGRESS_KEYS[4]: age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[4]],
        AGE5_W107_PROGRESS_KEYS[5]: age5_w107_progress_snapshot[AGE5_W107_PROGRESS_KEYS[5]],
        AGE5_W107_CONTRACT_PROGRESS_KEYS[0]: age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_W107_CONTRACT_PROGRESS_KEYS[1]: age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_W107_CONTRACT_PROGRESS_KEYS[2]: age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_W107_CONTRACT_PROGRESS_KEYS[3]: age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_W107_CONTRACT_PROGRESS_KEYS[4]: age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_W107_CONTRACT_PROGRESS_KEYS[5]: age5_w107_contract_progress_snapshot[AGE5_W107_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0]: age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1]: age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2]: age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[3]: age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4]: age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5]: age5_age1_immediate_proof_operation_contract_progress_snapshot[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_proof_certificate_v1_consumer_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0]: age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1]: age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2]: age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[3]: age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4]: age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5]: age5_proof_certificate_v1_verify_report_digest_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_proof_certificate_v1_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_proof_certificate_family_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_proof_family_contract_progress_snapshot[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_proof_family_transport_contract_progress_snapshot[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_lang_surface_family_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_lang_runtime_family_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_gate0_family_contract_progress_snapshot[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_gate0_surface_family_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_gate0_surface_family_transport_contract_progress_snapshot[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_gate0_family_transport_contract_progress_snapshot[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_gate0_transport_family_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_gate0_transport_family_transport_contract_progress_snapshot[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_lang_runtime_family_transport_contract_progress_snapshot[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_gate0_runtime_family_transport_contract_progress_snapshot[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_lang_surface_family_transport_contract_progress_snapshot[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_proof_certificate_family_transport_contract_progress_snapshot[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_bogae_alias_family_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]],
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]],
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]],
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]],
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]],
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_bogae_alias_family_transport_contract_progress_snapshot[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]],
        AGE5_DIGEST_SELFTEST_SUMMARY_KEY: age5_digest_selftest_snapshot,
        AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
        "combined_digest_selftest_default_field": AGE5_DIGEST_SELFTEST_DEFAULT_FIELD,
        AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: str(
            age5_policy_snapshot.get(
                AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY,
                AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
            )
        ).strip()
        or AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
        AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY: dict(
            age5_policy_snapshot.get(
                AGE5_POLICY_DIGEST_SELFTEST_DEFAULT_FIELD_KEY,
                AGE5_DIGEST_SELFTEST_DEFAULT_FIELD,
            )
        ),
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY: str(
            age5_policy_snapshot.get(
                AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY,
                AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
            )
        ).strip()
        or AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY: str(
            age5_policy_snapshot.get(
                AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY,
                build_age4_proof_snapshot_text(build_age4_proof_snapshot()),
            )
        ).strip()
        or build_age4_proof_snapshot_text(build_age4_proof_snapshot()),
        AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY: str(
            age5_policy_snapshot.get(
                AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY,
                AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
            )
        ).strip()
        or AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY: str(
            age5_policy_snapshot.get(AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY, "0")
        ).strip()
        or "0",
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY: str(
            age5_policy_snapshot.get(AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY, "0")
        ).strip()
        or "0",
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY: str(
            age5_policy_snapshot.get(AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY, "0")
        ).strip()
        or "0",
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY: str(
            age5_policy_snapshot.get(AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY, "0")
        ).strip()
        or "0",
        AGE5_POLICY_REPORT_PATH_KEY: str(
            age5_policy_snapshot.get(AGE5_POLICY_REPORT_PATH_KEY, "-")
        ).strip()
        or "-",
        AGE5_POLICY_REPORT_EXISTS_KEY: int(
            bool(age5_policy_snapshot.get(AGE5_POLICY_REPORT_EXISTS_KEY, 0))
        ),
        AGE5_POLICY_TEXT_PATH_KEY: str(
            age5_policy_snapshot.get(AGE5_POLICY_TEXT_PATH_KEY, "-")
        ).strip()
        or "-",
        AGE5_POLICY_TEXT_EXISTS_KEY: int(
            bool(age5_policy_snapshot.get(AGE5_POLICY_TEXT_EXISTS_KEY, 0))
        ),
        AGE5_POLICY_SUMMARY_PATH_KEY: str(
            age5_policy_snapshot.get(AGE5_POLICY_SUMMARY_PATH_KEY, "-")
        ).strip()
        or "-",
        AGE5_POLICY_SUMMARY_EXISTS_KEY: int(
            bool(age5_policy_snapshot.get(AGE5_POLICY_SUMMARY_EXISTS_KEY, 0))
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY: str(
            age5_policy_snapshot.get(
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY,
                "ok",
            )
        ).strip()
        or "ok",
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY: int(
            bool(
                age5_policy_snapshot.get(
                    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY,
                    0,
                )
            )
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY: str(
            age5_policy_snapshot.get(
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY,
                "-",
            )
        ).strip()
        or "-",
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY: str(
            age5_policy_snapshot.get(
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY,
                "-",
            )
        ).strip()
        or "-",
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY: str(
            age5_policy_snapshot.get(
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY,
                build_age5_combined_heavy_policy_origin_trace_contract_compact_reason(),
            )
        ).strip()
        or build_age5_combined_heavy_policy_origin_trace_contract_compact_reason(),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY: str(
            age5_policy_snapshot.get(
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY,
                build_age5_combined_heavy_policy_origin_trace_contract_compact_failure_reason(),
            )
        ).strip()
        or build_age5_combined_heavy_policy_origin_trace_contract_compact_failure_reason(),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY: str(
            age5_policy_snapshot.get(
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY,
                build_age5_combined_heavy_policy_origin_trace_text(),
            )
        ).strip()
        or build_age5_combined_heavy_policy_origin_trace_text(),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY: dict(
            age5_policy_snapshot.get(
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY,
                build_age5_combined_heavy_policy_origin_trace(),
            )
        ),
        **age5_child_snapshot,
        "artifacts": artifacts_payload(index_doc),
    }
    return payload


def load_result_doc(index_doc: dict) -> dict | None:
    result_path = artifact_path(index_doc, "ci_gate_result_json")
    if result_path is None or not result_path.exists():
        return None
    result_doc = load_json(result_path)
    return result_doc if isinstance(result_doc, dict) else None


def print_failure_digest(
    index_doc: dict,
    result_doc: dict | None,
    limit: int,
    tail_lines: int,
) -> bool:
    if limit <= 0:
        return print_summary_verify(index_doc, result_doc)

    reason = "-"
    status = "-"
    if isinstance(result_doc, dict):
        reason = str(result_doc.get("reason", "-")).strip() or "-"
        status = str(result_doc.get("status", "-")).strip() or "-"
    print(f"[ci-fail] status={status} reason={clip(reason, 220)}")

    steps = index_doc.get("steps")
    if isinstance(steps, list):
        failed_rows = sorted_failed_rows(steps)
        failed_steps = [str(row.get("name", "-")) for row in failed_rows]
        if failed_steps:
            joined = ",".join(failed_steps[:limit])
            print(f"[ci-fail] failed_steps={joined}")
        for row in failed_rows[:limit]:
            name, stdout_log, stderr_log, brief = failed_row_details(row)
            if brief:
                print(f"[ci-fail-brief] step={name} message={clip(brief, 220)}")
            if stdout_log or stderr_log:
                print(
                    f"[ci-fail] step_logs={name} "
                    f"stdout={stdout_log or '-'} stderr={stderr_log or '-'}"
                )
            if tail_lines <= 0:
                continue
            selected_stream = ""
            selected_path: Path | None = None
            for stream_name, raw_path in (("stderr", stderr_log), ("stdout", stdout_log)):
                if not raw_path:
                    continue
                candidate = normalize_path(raw_path)
                if not candidate.exists():
                    continue
                selected_stream = stream_name
                selected_path = candidate
                break
            if selected_path is None:
                continue
            tail = read_tail_lines(selected_path, tail_lines)
            if not tail:
                continue
            print(
                f"[ci-fail-tail] step={name} stream={selected_stream} "
                f"path={selected_path} lines={len(tail)}"
            )
            for line in tail:
                print(f"[ci-fail-tail] {clip(line, 240)}")

    aggregate_path = artifact_path(index_doc, "aggregate")
    aggregate_doc = load_json(aggregate_path) if aggregate_path is not None and aggregate_path.exists() else None
    digest_printed = False
    if isinstance(aggregate_doc, dict):
        failure_digest = aggregate_doc.get("failure_digest")
        if isinstance(failure_digest, list) and failure_digest:
            for item in failure_digest[:limit]:
                print(f"[ci-fail] digest={clip(str(item), 260)}")
            digest_printed = True
        if not digest_printed:
            for bucket_key in ("seamgrim", "age3", "oi405_406"):
                bucket = aggregate_doc.get(bucket_key)
                if not isinstance(bucket, dict):
                    continue
                digest = bucket.get("failure_digest")
                if not isinstance(digest, list) or not digest:
                    continue
                for item in digest[:limit]:
                    print(f"[ci-fail] {bucket_key}={clip(str(item), 240)}")
                digest_printed = True
                break
    if not digest_printed:
        print("[ci-fail] digest=-")

    return print_summary_verify(index_doc, result_doc)


def line_from_index(index_doc: dict) -> str:
    for key in ("summary_line", "ci_gate_result_line", "final_status_line", "aggregate_status_line"):
        path = artifact_path(index_doc, key)
        if path is None:
            continue
        line = load_line(path)
        if line:
            print(f"[ci-final-meta] primary_line_source={path}")
            return line
    return ""


def main() -> int:
    parser = argparse.ArgumentParser(description="Emit single final CI status line from aggregate gate reports")
    parser.add_argument("--report-dir", default=default_report_dir(), help="report directory")
    parser.add_argument("--index-pattern", default="*ci_gate_report_index.detjson", help="index file glob")
    parser.add_argument("--prefix", default="", help="optional expected report prefix")
    parser.add_argument("--print-artifacts", action="store_true", help="print key artifact paths and existence")
    parser.add_argument(
        "--print-failure-digest",
        type=int,
        default=0,
        help="on failed status, print up to N failure-digest lines",
    )
    parser.add_argument(
        "--print-failure-tail-lines",
        type=int,
        default=0,
        help="on failed status, print up to N tail lines from failed step logs (stderr first)",
    )
    parser.add_argument(
        "--failure-brief-out",
        default="",
        help="optional one-line failure-brief txt output path (supports __PREFIX__ token)",
    )
    parser.add_argument(
        "--triage-json-out",
        default="",
        help="optional ci failure triage json output path (supports __PREFIX__ token)",
    )
    parser.add_argument(
        "--fail-on-summary-verify-error",
        action="store_true",
        help="return non-zero when fail-case summary(detail/log rows) verification fails",
    )
    parser.add_argument("--require-final-line", action="store_true", help="return non-zero when final line is missing")
    args = parser.parse_args()

    report_dir = Path(args.report_dir)
    if not report_dir.exists():
        print(f"[ci-final-meta] report_dir_missing={report_dir}")
        if args.require_final_line:
            print("[ci-final] status=unknown reason=report_dir_missing")
            return 1
        print("[ci-final] status=unknown reason=report_dir_missing")
        return 0

    index_path, index_doc = select_latest_index(report_dir, args.index_pattern, args.prefix.strip())
    result_doc: dict | None = None
    if index_path is not None and isinstance(index_doc, dict):
        prefix = str(index_doc.get("report_prefix", "")).strip() or "-"
        print(f"[ci-final-meta] report_index={index_path} prefix={prefix}")
        step_log_dir = str(index_doc.get("step_log_dir", "")).strip()
        if step_log_dir:
            print(f"[ci-final-meta] step_log_dir={step_log_dir}")
        if args.print_artifacts:
            print_artifact_lines(index_doc)
        print_result_meta(index_doc)
        result_doc = load_result_doc(index_doc)
        final_line = line_from_index(index_doc)
    else:
        print("[ci-final-meta] report_index=missing")
        final_line = ""

    if not final_line:
        final_line = first_existing_line(report_dir)

    if final_line:
        print(f"[ci-final] {render_ci_final_stdout_line(final_line, index_doc if isinstance(index_doc, dict) else None)}")
        status = str(result_doc.get("status", "")).strip() if isinstance(result_doc, dict) else ""
        prefix_value = str(index_doc.get("report_prefix", "")).strip() if isinstance(index_doc, dict) else ""
        brief_path_resolved = (
            resolve_failure_brief_out(args.failure_brief_out, prefix_value) if args.failure_brief_out.strip() else None
        )
        triage_path_resolved = (
            resolve_failure_brief_out(args.triage_json_out, prefix_value) if args.triage_json_out.strip() else None
        )
        summary_verify_ok: bool | None = None
        summary_verify_issues: list[str] | None = None
        if args.print_failure_digest > 0 and status and status != "pass" and isinstance(index_doc, dict):
            summary_verify_ok = print_failure_digest(
                index_doc,
                result_doc,
                args.print_failure_digest,
                max(0, int(args.print_failure_tail_lines)),
            )
            if summary_verify_ok is False:
                ok, issues, _, _ = verify_summary_report(index_doc, result_doc)
                if not ok:
                    summary_verify_issues = issues
        if (
            args.fail_on_summary_verify_error
            and status
            and status != "pass"
            and isinstance(index_doc, dict)
        ):
            if summary_verify_ok is None:
                summary_verify_ok = print_summary_verify(index_doc, result_doc)
            if not summary_verify_ok:
                if brief_path_resolved is not None:
                    brief_line = build_failure_brief_line(
                        index_doc,
                        result_doc,
                        final_line,
                        max(1, int(args.print_failure_digest) or 6),
                    )
                    write_failure_brief(brief_path_resolved, brief_line)
                if triage_path_resolved is not None:
                    triage_payload = build_triage_payload(
                        index_doc,
                        result_doc,
                        final_line,
                        summary_verify_ok=False,
                        summary_verify_issues=summary_verify_issues,
                    )
                    patch_triage_output_refs(triage_payload, brief_path_resolved, triage_path_resolved)
                    write_triage_json(triage_path_resolved, triage_payload)
                return 2
        if brief_path_resolved is not None:
            brief_line = build_failure_brief_line(index_doc, result_doc, final_line, max(1, int(args.print_failure_digest) or 6))
            write_failure_brief(brief_path_resolved, brief_line)
        if triage_path_resolved is not None:
            triage_payload = build_triage_payload(
                index_doc,
                result_doc,
                final_line,
                summary_verify_ok=summary_verify_ok,
                summary_verify_issues=summary_verify_issues,
            )
            patch_triage_output_refs(triage_payload, brief_path_resolved, triage_path_resolved)
            write_triage_json(triage_path_resolved, triage_payload)
        return 0

    print("[ci-final] status=unknown reason=final_line_missing")
    if args.print_failure_digest > 0 and isinstance(index_doc, dict):
        summary_verify_ok = print_failure_digest(
            index_doc,
            result_doc,
            args.print_failure_digest,
            max(0, int(args.print_failure_tail_lines)),
        )
    else:
        summary_verify_ok = None
    prefix_value = str(index_doc.get("report_prefix", "")).strip() if isinstance(index_doc, dict) else ""
    brief_path_resolved = (
        resolve_failure_brief_out(args.failure_brief_out, prefix_value) if args.failure_brief_out.strip() else None
    )
    triage_path_resolved = (
        resolve_failure_brief_out(args.triage_json_out, prefix_value) if args.triage_json_out.strip() else None
    )
    if brief_path_resolved is not None:
        brief_line = build_failure_brief_line(index_doc, result_doc, final_line="", limit=max(1, int(args.print_failure_digest) or 6))
        write_failure_brief(brief_path_resolved, brief_line)
    if triage_path_resolved is not None:
        triage_payload = build_triage_payload(
            index_doc,
            result_doc,
            final_line="",
            summary_verify_ok=summary_verify_ok,
            summary_verify_issues=None,
        )
        patch_triage_output_refs(triage_payload, brief_path_resolved, triage_path_resolved)
        write_triage_json(triage_path_resolved, triage_payload)
    return 1 if args.require_final_line else 0


if __name__ == "__main__":
    raise SystemExit(main())
