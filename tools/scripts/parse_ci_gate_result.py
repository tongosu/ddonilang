#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path


EXPECTED_SCHEMA = "ddn.ci.gate_result.v1"
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


def compact_line(doc: dict) -> str:
    status = str(doc.get("status", "fail")).strip() or "fail"
    ok = 1 if bool(doc.get("ok", False)) else 0
    overall_ok = 1 if bool(doc.get("overall_ok", False)) else 0
    failed_steps = int(doc.get("failed_steps", -1))
    aggregate_status = str(doc.get("aggregate_status", "fail")).strip() or "fail"
    age4_proof_ok = 1 if bool(doc.get(AGE4_PROOF_OK_KEY, False)) else 0
    age4_proof_failed_criteria = int(doc.get(AGE4_PROOF_FAILED_CRITERIA_KEY, -1))
    reason = str(doc.get("reason", "-")).strip() or "-"
    return (
        f"ci_gate_result_status={status} ok={ok} overall_ok={overall_ok} "
        f"failed_steps={failed_steps} aggregate_status={aggregate_status} "
        f"{AGE4_PROOF_OK_KEY}={age4_proof_ok} {AGE4_PROOF_FAILED_CRITERIA_KEY}={age4_proof_failed_criteria} "
        f"age5_w107_active={str(doc.get(AGE5_W107_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_w107_inactive={str(doc.get(AGE5_W107_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_w107_index_codes={str(doc.get(AGE5_W107_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_w107_current_probe={str(doc.get(AGE5_W107_PROGRESS_KEYS[3], '-')).strip() or '-'} "
        f"age5_w107_last_completed_probe={str(doc.get(AGE5_W107_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_w107_progress={str(doc.get(AGE5_W107_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_w107_contract_completed={str(doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_w107_contract_total={str(doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_w107_contract_checks_text={str(doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_w107_contract_current_probe={str(doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'} "
        f"age5_w107_contract_last_completed_probe={str(doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_w107_contract_progress={str(doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_age1_immediate_proof_operation_contract_completed={str(doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_age1_immediate_proof_operation_contract_total={str(doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_age1_immediate_proof_operation_contract_checks_text={str(doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_age1_immediate_proof_operation_contract_current_probe={str(doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'} "
        f"age5_age1_immediate_proof_operation_contract_last_completed_probe={str(doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_age1_immediate_proof_operation_contract_progress={str(doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_proof_certificate_v1_consumer_contract_completed={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_consumer_contract_total={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_consumer_contract_checks_text={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_consumer_contract_current_probe={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_consumer_contract_last_completed_probe={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_consumer_contract_progress={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_completed={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_total={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_checks_text={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_current_probe={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_last_completed_probe={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_progress={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_proof_certificate_v1_family_contract_completed={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_family_contract_total={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_family_contract_checks_text={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_family_contract_current_probe={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_family_contract_last_completed_probe={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_family_contract_progress={str(doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_proof_certificate_family_contract_completed={str(doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_contract_total={str(doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_contract_checks_text={str(doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_contract_current_probe={str(doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_contract_last_completed_probe={str(doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_contract_progress={str(doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_proof_certificate_family_transport_contract_completed={str(doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_transport_contract_total={str(doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_transport_contract_checks_text={str(doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_transport_contract_current_probe={str(doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_transport_contract_last_completed_probe={str(doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_transport_contract_progress={str(doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_proof_family_contract_completed={str(doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_proof_family_contract_total={str(doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_proof_family_contract_checks_text={str(doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_proof_family_contract_current_probe={str(doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'} "
        f"age5_proof_family_contract_last_completed_probe={str(doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_proof_family_contract_progress={str(doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_proof_family_transport_contract_completed={str(doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_proof_family_transport_contract_total={str(doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_proof_family_transport_contract_checks_text={str(doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_proof_family_transport_contract_current_probe={str(doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'} "
        f"age5_proof_family_transport_contract_last_completed_probe={str(doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_proof_family_transport_contract_progress={str(doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_lang_surface_family_contract_completed={str(doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_lang_surface_family_contract_total={str(doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_lang_surface_family_contract_checks_text={str(doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_lang_surface_family_contract_current_probe={str(doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'} "
        f"age5_lang_surface_family_contract_last_completed_probe={str(doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_lang_surface_family_contract_progress={str(doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_lang_runtime_family_contract_completed={str(doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_lang_runtime_family_contract_total={str(doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_lang_runtime_family_contract_checks_text={str(doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_lang_runtime_family_contract_current_probe={str(doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'} "
        f"age5_lang_runtime_family_contract_last_completed_probe={str(doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_lang_runtime_family_contract_progress={str(doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_lang_surface_family_transport_contract_completed={str(doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_lang_surface_family_transport_contract_total={str(doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_lang_surface_family_transport_contract_checks_text={str(doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_lang_surface_family_transport_contract_current_probe={str(doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'} "
        f"age5_lang_surface_family_transport_contract_last_completed_probe={str(doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_lang_surface_family_transport_contract_progress={str(doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_bogae_alias_family_contract_completed={str(doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_contract_total={str(doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_contract_checks_text={str(doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_contract_current_probe={str(doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_contract_last_completed_probe={str(doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_contract_progress={str(doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_bogae_alias_family_transport_contract_completed={str(doc.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_transport_contract_total={str(doc.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_transport_contract_checks_text={str(doc.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_transport_contract_current_probe={str(doc.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_transport_contract_last_completed_probe={str(doc.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_transport_contract_progress={str(doc.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"reason={reason}"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Parse ci_gate_result.detjson and print compact status line")
    parser.add_argument("--result", required=True, help="path to ci_gate_result.detjson")
    parser.add_argument("--json-out", help="optional output parse detjson path")
    parser.add_argument("--compact-out", help="optional output compact line txt path")
    parser.add_argument("--fail-on-invalid", action="store_true", help="return non-zero when parse/validation fails")
    parser.add_argument("--fail-on-fail", action="store_true", help="return non-zero when status is fail")
    args = parser.parse_args()

    result_path = Path(args.result)
    doc = load_json(result_path)
    if not isinstance(doc, dict):
        print(f"[ci-gate-result-parse] invalid reason=missing_or_invalid_result path={result_path}")
        if args.fail_on_invalid:
            return 1
        return 0
    if doc.get("schema") != EXPECTED_SCHEMA:
        print(
            "[ci-gate-result-parse] invalid "
            f"reason=schema_mismatch schema={doc.get('schema')} expected={EXPECTED_SCHEMA}"
        )
        if args.fail_on_invalid:
            return 1
        return 0
    status = str(doc.get("status", "fail")).strip() or "fail"
    aggregate_status = str(doc.get("aggregate_status", "fail")).strip() or "fail"
    if status not in {"pass", "fail"} or aggregate_status not in {"pass", "fail"}:
        print(
            "[ci-gate-result-parse] invalid "
            f"reason=status_field_invalid status={status} aggregate_status={aggregate_status}"
        )
        if args.fail_on_invalid:
            return 1
        return 0
    summary_line = str(doc.get("summary_line", "")).strip()
    if not summary_line:
        print("[ci-gate-result-parse] invalid reason=summary_line_missing")
        if args.fail_on_invalid:
            return 1
        return 0

    compact = summary_line or compact_line(doc)
    print(f"[ci-gate-result-parse] {compact}")

    if args.json_out:
        out = Path(args.json_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        parsed_payload = {
            "schema": "ddn.ci.gate_result_parse.v1",
            "result_path": str(result_path),
            "parsed": {
                "status": status,
                "ok": bool(doc.get("ok", False)),
                "overall_ok": bool(doc.get("overall_ok", False)),
                "failed_steps": int(doc.get("failed_steps", -1)),
                "aggregate_status": aggregate_status,
                AGE4_PROOF_OK_KEY: bool(doc.get(AGE4_PROOF_OK_KEY, False)),
                AGE4_PROOF_FAILED_CRITERIA_KEY: int(doc.get(AGE4_PROOF_FAILED_CRITERIA_KEY, -1)),
                AGE4_PROOF_FAILED_PREVIEW_KEY: str(doc.get(AGE4_PROOF_FAILED_PREVIEW_KEY, "-")).strip() or "-",
                AGE5_W107_PROGRESS_KEYS[0]: str(doc.get(AGE5_W107_PROGRESS_KEYS[0], "-")).strip() or "-",
                AGE5_W107_PROGRESS_KEYS[1]: str(doc.get(AGE5_W107_PROGRESS_KEYS[1], "-")).strip() or "-",
                AGE5_W107_PROGRESS_KEYS[2]: str(doc.get(AGE5_W107_PROGRESS_KEYS[2], "-")).strip() or "-",
                AGE5_W107_PROGRESS_KEYS[3]: str(doc.get(AGE5_W107_PROGRESS_KEYS[3], "-")).strip() or "-",
                AGE5_W107_PROGRESS_KEYS[4]: str(doc.get(AGE5_W107_PROGRESS_KEYS[4], "-")).strip() or "-",
                AGE5_W107_PROGRESS_KEYS[5]: str(doc.get(AGE5_W107_PROGRESS_KEYS[5], "0")).strip() or "0",
                AGE5_W107_CONTRACT_PROGRESS_KEYS[0]: str(doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[0], "-")).strip() or "-",
                AGE5_W107_CONTRACT_PROGRESS_KEYS[1]: str(doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[1], "-")).strip() or "-",
                AGE5_W107_CONTRACT_PROGRESS_KEYS[2]: str(doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[2], "-")).strip() or "-",
                AGE5_W107_CONTRACT_PROGRESS_KEYS[3]: str(doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[3], "-")).strip() or "-",
                AGE5_W107_CONTRACT_PROGRESS_KEYS[4]: str(doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[4], "-")).strip() or "-",
                AGE5_W107_CONTRACT_PROGRESS_KEYS[5]: str(doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[5], "0")).strip() or "0",
                AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0]: str(
                    doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0], "-")
                ).strip()
                or "-",
                AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1]: str(
                    doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1], "-")
                ).strip()
                or "-",
                AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2]: str(
                    doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2], "-")
                ).strip()
                or "-",
                AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[3]: str(
                    doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[3], "-")
                ).strip()
                or "-",
                AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4]: str(
                    doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4], "-")
                ).strip()
                or "-",
                AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5]: str(
                    doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5], "0")
                ).strip()
                or "0",
                AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")
                ).strip()
                or "0",
                AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[3]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[3], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5], "0")
                ).strip()
                or "0",
                AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[3]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[3], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")
                ).strip()
                or "0",
                AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[3], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")
                ).strip()
                or "0",
                AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], "-")
                ).strip()
                or "-",
                AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: str(
                    doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")
                ).strip()
                or "0",
                AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0]: str(
                    doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0], "-")
                ).strip()
                or "-",
                AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1]: str(
                    doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1], "-")
                ).strip()
                or "-",
                AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2]: str(
                    doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2], "-")
                ).strip()
                or "-",
                AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[3]: str(
                    doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[3], "-")
                ).strip()
                or "-",
                AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4]: str(
                    doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4], "-")
                ).strip()
                or "-",
                AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5]: str(
                    doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")
                ).strip()
                or "0",
                AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: str(
                    doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], "-")
                ).strip()
                or "-",
                AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: str(
                    doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], "-")
                ).strip()
                or "-",
                AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: str(
                    doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], "-")
                ).strip()
                or "-",
                AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: str(
                    doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], "-")
                ).strip()
                or "-",
                AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: str(
                    doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], "-")
                ).strip()
                or "-",
                AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: str(
                    doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")
                ).strip()
                or "0",
                AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: str(
                    doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0], "-")
                ).strip()
                or "-",
                AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: str(
                    doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1], "-")
                ).strip()
                or "-",
                AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: str(
                    doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2], "-")
                ).strip()
                or "-",
                AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: str(
                    doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3], "-")
                ).strip()
                or "-",
                AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: str(
                    doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4], "-")
                ).strip()
                or "-",
                AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: str(
                    doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")
                ).strip()
                or "0",
                AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0]: str(
                    doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0], "-")
                ).strip()
                or "-",
                AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1]: str(
                    doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1], "-")
                ).strip()
                or "-",
                AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2]: str(
                    doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2], "-")
                ).strip()
                or "-",
                AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[3]: str(
                    doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[3], "-")
                ).strip()
                or "-",
                AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4]: str(
                    doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4], "-")
                ).strip()
                or "-",
                AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5]: str(
                    doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")
                ).strip()
                or "0",
                AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: str(
                    doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], "-")
                ).strip()
                or "-",
                AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: str(
                    doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], "-")
                ).strip()
                or "-",
                AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: str(
                    doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], "-")
                ).strip()
                or "-",
                AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: str(
                    doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], "-")
                ).strip()
                or "-",
                AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: str(
                    doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], "-")
                ).strip()
                or "-",
                AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: str(
                    doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")
                ).strip()
                or "0",
                AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0]: str(
                    doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0], "-")
                ).strip()
                or "-",
                AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1]: str(
                    doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1], "-")
                ).strip()
                or "-",
                AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2]: str(
                    doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2], "-")
                ).strip()
                or "-",
                AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[3]: str(
                    doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[3], "-")
                ).strip()
                or "-",
                AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4]: str(
                    doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4], "-")
                ).strip()
                or "-",
                AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5]: str(
                    doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")
                ).strip()
                or "0",
                AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY: str(doc.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY, "")).strip(),
                AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY: str(doc.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY, "")).strip(),
                AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY: str(doc.get(AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY, "")).strip(),
                AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY: str(doc.get(AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY, "0")).strip() or "0",
                AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY: str(doc.get(AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY, "0")).strip() or "0",
                AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY: str(doc.get(AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY, "0")).strip() or "0",
                AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY: str(doc.get(AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY, "0")).strip() or "0",
                "reason": str(doc.get("reason", "-")),
            },
        }
        # Keep parser output forward-compatible: include any newer result fields verbatim
        # so consistency checks comparing result vs parse do not drift on key additions.
        parsed_doc = parsed_payload.get("parsed")
        if isinstance(parsed_doc, dict):
            for key, value in doc.items():
                normalized_key = str(key).strip()
                if not normalized_key:
                    continue
                if normalized_key not in parsed_doc:
                    parsed_doc[normalized_key] = value
        payload = {
            **parsed_payload,
            "compact_line": compact,
        }
        out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    if args.compact_out:
        out = Path(args.compact_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(compact + "\n", encoding="utf-8")

    if args.fail_on_fail and status != "pass":
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
