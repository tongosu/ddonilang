#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path


SCHEMA = "ddn.ci.gate_result.v1"
AGE4_PROOF_OK_KEY = "age4_proof_ok"
AGE4_PROOF_FAILED_CRITERIA_KEY = "age4_proof_failed_criteria"
AGE4_PROOF_FAILED_PREVIEW_KEY = "age4_proof_failed_preview"
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
AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present",
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
AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY = "age5_policy_age4_proof_snapshot_fields_text"
AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY = "age5_policy_age4_proof_snapshot_text"
AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY = "age5_policy_age4_proof_source_snapshot_fields_text"
AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY = "age5_policy_age4_proof_gate_result_present"
AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY = "age5_policy_age4_proof_gate_result_parity"
AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY = "age5_policy_age4_proof_final_status_parse_present"
AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY = "age5_policy_age4_proof_final_status_parse_parity"


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def load_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8").strip()
    except Exception:
        return ""


def build_result(
    final_parse_path: Path,
    summary_line_path: Path,
    gate_index_path: Path | None,
    final_parse_doc: dict | None,
) -> tuple[dict, bool]:
    if not isinstance(final_parse_doc, dict):
        payload = {
            "schema": SCHEMA,
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "ok": False,
            "status": "fail",
            "overall_ok": False,
            "failed_steps": -1,
            "aggregate_status": "fail",
            AGE4_PROOF_OK_KEY: False,
            AGE4_PROOF_FAILED_CRITERIA_KEY: -1,
            AGE4_PROOF_FAILED_PREVIEW_KEY: "-",
            AGE5_W107_PROGRESS_KEYS[0]: "-",
            AGE5_W107_PROGRESS_KEYS[1]: "-",
            AGE5_W107_PROGRESS_KEYS[2]: "-",
            AGE5_W107_PROGRESS_KEYS[3]: "-",
            AGE5_W107_PROGRESS_KEYS[4]: "-",
            AGE5_W107_PROGRESS_KEYS[5]: "0",
            AGE5_W107_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_W107_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_W107_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_W107_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_W107_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_W107_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
            "reason": "invalid_or_missing_final_parse",
            "summary_line_path": str(summary_line_path),
            "summary_line": load_text(summary_line_path),
            "final_status_parse_path": str(final_parse_path),
            "gate_index_path": str(gate_index_path) if gate_index_path is not None else "",
        }
        return payload, False

    parsed = final_parse_doc.get("parsed")
    if not isinstance(parsed, dict):
        payload = {
            "schema": SCHEMA,
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "ok": False,
            "status": "fail",
            "overall_ok": False,
            "failed_steps": -1,
            "aggregate_status": "fail",
            AGE4_PROOF_OK_KEY: False,
            AGE4_PROOF_FAILED_CRITERIA_KEY: -1,
            AGE4_PROOF_FAILED_PREVIEW_KEY: "-",
            AGE5_W107_PROGRESS_KEYS[0]: "-",
            AGE5_W107_PROGRESS_KEYS[1]: "-",
            AGE5_W107_PROGRESS_KEYS[2]: "-",
            AGE5_W107_PROGRESS_KEYS[3]: "-",
            AGE5_W107_PROGRESS_KEYS[4]: "-",
            AGE5_W107_PROGRESS_KEYS[5]: "0",
            AGE5_W107_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_W107_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_W107_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_W107_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_W107_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_W107_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5]: "0",
            AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: "-",
            AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: "0",
            "reason": "invalid_final_parse_payload",
            "summary_line_path": str(summary_line_path),
            "summary_line": load_text(summary_line_path),
            "final_status_parse_path": str(final_parse_path),
            "gate_index_path": str(gate_index_path) if gate_index_path is not None else "",
        }
        return payload, False

    status = str(parsed.get("status", "fail")).strip() or "fail"
    overall_ok = str(parsed.get("overall_ok", "0")).strip() == "1"
    try:
        failed_steps = int(parsed.get("failed_steps", "-1"))
    except ValueError:
        failed_steps = -1
    aggregate_status = str(parsed.get("aggregate_status", "fail")).strip() or "fail"
    age4_proof_ok = str(parsed.get(AGE4_PROOF_OK_KEY, "0")).strip() == "1"
    try:
        age4_proof_failed_criteria = int(parsed.get(AGE4_PROOF_FAILED_CRITERIA_KEY, "-1"))
    except ValueError:
        age4_proof_failed_criteria = -1
    age4_proof_failed_preview = str(parsed.get(AGE4_PROOF_FAILED_PREVIEW_KEY, "-")).strip() or "-"
    age5_w107_progress = {
        AGE5_W107_PROGRESS_KEYS[0]: str(parsed.get(AGE5_W107_PROGRESS_KEYS[0], "-")).strip() or "-",
        AGE5_W107_PROGRESS_KEYS[1]: str(parsed.get(AGE5_W107_PROGRESS_KEYS[1], "-")).strip() or "-",
        AGE5_W107_PROGRESS_KEYS[2]: str(parsed.get(AGE5_W107_PROGRESS_KEYS[2], "-")).strip() or "-",
        AGE5_W107_PROGRESS_KEYS[3]: str(parsed.get(AGE5_W107_PROGRESS_KEYS[3], "-")).strip() or "-",
        AGE5_W107_PROGRESS_KEYS[4]: str(parsed.get(AGE5_W107_PROGRESS_KEYS[4], "-")).strip() or "-",
        AGE5_W107_PROGRESS_KEYS[5]: str(parsed.get(AGE5_W107_PROGRESS_KEYS[5], "0")).strip() or "0",
    }
    age5_w107_contract_progress = {
        AGE5_W107_CONTRACT_PROGRESS_KEYS[0]: str(parsed.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[0], "-")).strip() or "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[1]: str(parsed.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[1], "-")).strip() or "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[2]: str(parsed.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[2], "-")).strip() or "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[3]: str(parsed.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[3], "-")).strip() or "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[4]: str(parsed.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[4], "-")).strip() or "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[5]: str(parsed.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[5], "0")).strip() or "0",
    }
    age5_age1_immediate_proof_operation_contract_progress = {
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_proof_certificate_v1_consumer_transport_contract_progress = {
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_proof_certificate_v1_verify_report_digest_contract_progress = {
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_proof_certificate_v1_family_contract_progress = {
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_proof_certificate_family_contract_progress = {
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_proof_certificate_family_transport_contract_progress = {
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_proof_family_contract_progress = {
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_proof_family_transport_contract_progress = {
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_lang_surface_family_contract_progress = {
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_lang_runtime_family_contract_progress = {
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_lang_surface_family_transport_contract_progress = {
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_gate0_family_contract_progress = {
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_gate0_surface_family_contract_progress = {
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_gate0_surface_family_transport_contract_progress = {
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_gate0_family_transport_contract_progress = {
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_gate0_transport_family_contract_progress = {
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_gate0_transport_family_transport_contract_progress = {
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_lang_runtime_family_transport_contract_progress = {
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_gate0_runtime_family_transport_contract_progress = {
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_bogae_alias_family_contract_progress = {
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_bogae_alias_family_transport_contract_progress = {
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: str(
            parsed.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], "-")
        ).strip()
        or "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: str(
            parsed.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], "-")
        ).strip()
        or "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: str(
            parsed.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], "-")
        ).strip()
        or "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: str(
            parsed.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], "-")
        ).strip()
        or "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: str(
            parsed.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], "-")
        ).strip()
        or "-",
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: str(
            parsed.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")
        ).strip()
        or "0",
    }
    age5_policy_fields = {
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY: str(
            parsed.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY, "")
        ).strip()
        or "age4_proof_ok=0|age4_proof_failed_criteria=-1|age4_proof_failed_preview=-",
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY: str(
            parsed.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY, "")
        ).strip()
        or "age4_proof_ok=0|age4_proof_failed_criteria=-1|age4_proof_failed_preview=-",
        AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY: str(
            parsed.get(AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY, "")
        ).strip()
        or "age4_proof_gate_result_snapshot_text=age4_proof_ok=0|age4_proof_failed_criteria=-1|age4_proof_failed_preview=-|age4_proof_gate_result_snapshot_present=0|age4_proof_gate_result_snapshot_parity=0|age4_proof_final_status_parse_snapshot_text=age4_proof_ok=0|age4_proof_failed_criteria=-1|age4_proof_failed_preview=-|age4_proof_final_status_parse_snapshot_present=0|age4_proof_final_status_parse_snapshot_parity=0",
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY: str(
            parsed.get(AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY, "0")
        ).strip()
        or "0",
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY: str(
            parsed.get(AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY, "0")
        ).strip()
        or "0",
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY: str(
            parsed.get(AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY, "0")
        ).strip()
        or "0",
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY: str(
            parsed.get(AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY, "0")
        ).strip()
        or "0",
    }
    reason = str(parsed.get("reason", "-")).strip() or "-"
    summary_line = load_text(summary_line_path)
    ok = status == "pass" and overall_ok and aggregate_status == "pass" and failed_steps == 0
    payload = {
        "schema": SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": ok,
        "status": status,
        "overall_ok": overall_ok,
        "failed_steps": failed_steps,
        "aggregate_status": aggregate_status,
        AGE4_PROOF_OK_KEY: age4_proof_ok,
        AGE4_PROOF_FAILED_CRITERIA_KEY: age4_proof_failed_criteria,
        AGE4_PROOF_FAILED_PREVIEW_KEY: age4_proof_failed_preview,
        **age5_w107_progress,
        **age5_w107_contract_progress,
        **age5_age1_immediate_proof_operation_contract_progress,
        **age5_proof_certificate_v1_consumer_transport_contract_progress,
        **age5_proof_certificate_v1_verify_report_digest_contract_progress,
        **age5_proof_certificate_v1_family_contract_progress,
        **age5_proof_certificate_family_contract_progress,
        **age5_proof_certificate_family_transport_contract_progress,
        **age5_proof_family_contract_progress,
        **age5_proof_family_transport_contract_progress,
        **age5_lang_surface_family_contract_progress,
        **age5_lang_runtime_family_contract_progress,
        **age5_gate0_family_contract_progress,
        **age5_gate0_surface_family_contract_progress,
        **age5_gate0_surface_family_transport_contract_progress,
        **age5_lang_surface_family_transport_contract_progress,
        **age5_lang_runtime_family_transport_contract_progress,
        **age5_gate0_family_transport_contract_progress,
        **age5_gate0_transport_family_contract_progress,
        **age5_gate0_transport_family_transport_contract_progress,
        **age5_gate0_runtime_family_transport_contract_progress,
        **age5_bogae_alias_family_contract_progress,
        **age5_bogae_alias_family_transport_contract_progress,
        **age5_policy_fields,
        "reason": reason,
        "summary_line_path": str(summary_line_path),
        "summary_line": summary_line,
        "final_status_parse_path": str(final_parse_path),
        "gate_index_path": str(gate_index_path) if gate_index_path is not None else "",
    }
    return payload, ok


def main() -> int:
    parser = argparse.ArgumentParser(description="Render compact CI gate result JSON")
    parser.add_argument("--final-status-parse", required=True, help="path to ci_gate_final_status_line_parse.detjson")
    parser.add_argument("--summary-line", required=True, help="path to ci_gate_summary_line.txt")
    parser.add_argument("--gate-index", default="", help="optional path to ci_gate_report_index.detjson")
    parser.add_argument("--out", required=True, help="output ci_gate_result.detjson path")
    parser.add_argument("--fail-on-bad", action="store_true", help="return non-zero when result status is fail")
    args = parser.parse_args()

    final_parse_path = Path(args.final_status_parse)
    summary_line_path = Path(args.summary_line)
    gate_index_path = Path(args.gate_index) if args.gate_index.strip() else None
    out_path = Path(args.out)
    final_parse_doc = load_json(final_parse_path)
    payload, ok = build_result(final_parse_path, summary_line_path, gate_index_path, final_parse_doc)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"[ci-gate-result] out={out_path} ok={int(ok)} status={payload.get('status')}")
    if args.fail_on_bad and not ok:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
