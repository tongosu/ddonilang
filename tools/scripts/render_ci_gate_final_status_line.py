#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path


SCHEMA = "ddn.ci.gate_final_status_line.v1"
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


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def q(value: object) -> str:
    return json.dumps(str(value), ensure_ascii=False)


def count_failed_steps(index_doc: dict | None) -> int:
    if not isinstance(index_doc, dict):
        return -1
    steps = index_doc.get("steps")
    if not isinstance(steps, list):
        return -1
    return sum(1 for row in steps if isinstance(row, dict) and not bool(row.get("ok", False)))


def build_line(
    parse_path: Path,
    parse_doc: dict | None,
    index_path: Path,
    index_doc: dict | None,
) -> tuple[str, bool]:
    if not isinstance(parse_doc, dict):
        parts = [
            f"schema={q(SCHEMA)}",
            "status=fail",
            "overall_ok=0",
            "failed_steps=-1",
            "aggregate_status=fail",
            "age4_proof_ok=0",
            "age4_proof_failed_criteria=-1",
            "age5_full_real_w107_golden_index_selftest_active_cases=-",
            "age5_full_real_w107_golden_index_selftest_inactive_cases=-",
            "age5_full_real_w107_golden_index_selftest_index_codes=-",
            "age5_full_real_w107_golden_index_selftest_current_probe=-",
            "age5_full_real_w107_golden_index_selftest_last_completed_probe=-",
            "age5_full_real_w107_golden_index_selftest_progress_present=0",
            "age5_full_real_w107_progress_contract_selftest_completed_checks=-",
            "age5_full_real_w107_progress_contract_selftest_total_checks=-",
            "age5_full_real_w107_progress_contract_selftest_checks_text=-",
            "age5_full_real_w107_progress_contract_selftest_current_probe=-",
            "age5_full_real_w107_progress_contract_selftest_last_completed_probe=-",
            "age5_full_real_w107_progress_contract_selftest_progress_present=0",
            "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks=-",
            "age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks=-",
            "age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text=-",
            "age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe=-",
            "age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe=-",
            "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present=0",
            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks=-",
            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks=-",
            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text=-",
            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe=-",
            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe=-",
            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present=0",
            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks=-",
            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks=-",
            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text=-",
            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe=-",
            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe=-",
            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present=0",
            "age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks=-",
            "age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks=-",
            "age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text=-",
            "age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe=-",
            "age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe=-",
            "age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present=0",
            "age5_full_real_proof_certificate_family_contract_selftest_completed_checks=-",
            "age5_full_real_proof_certificate_family_contract_selftest_total_checks=-",
            "age5_full_real_proof_certificate_family_contract_selftest_checks_text=-",
            "age5_full_real_proof_certificate_family_contract_selftest_current_probe=-",
            "age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe=-",
            "age5_full_real_proof_certificate_family_contract_selftest_progress_present=0",
            "age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks=-",
            "age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks=-",
            "age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text=-",
            "age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe=-",
            "age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe=-",
            "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present=0",
            "age5_full_real_proof_family_contract_selftest_completed_checks=-",
            "age5_full_real_proof_family_contract_selftest_total_checks=-",
            "age5_full_real_proof_family_contract_selftest_checks_text=-",
            "age5_full_real_proof_family_contract_selftest_current_probe=-",
            "age5_full_real_proof_family_contract_selftest_last_completed_probe=-",
            "age5_full_real_proof_family_contract_selftest_progress_present=0",
            "age5_full_real_lang_surface_family_contract_selftest_completed_checks=-",
            "age5_full_real_lang_surface_family_contract_selftest_total_checks=-",
            "age5_full_real_lang_surface_family_contract_selftest_checks_text=-",
            "age5_full_real_lang_surface_family_contract_selftest_current_probe=-",
            "age5_full_real_lang_surface_family_contract_selftest_last_completed_probe=-",
            "age5_full_real_lang_surface_family_contract_selftest_progress_present=0",
            "age5_full_real_lang_runtime_family_contract_selftest_completed_checks=-",
            "age5_full_real_lang_runtime_family_contract_selftest_total_checks=-",
            "age5_full_real_lang_runtime_family_contract_selftest_checks_text=-",
            "age5_full_real_lang_runtime_family_contract_selftest_current_probe=-",
            "age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe=-",
            "age5_full_real_lang_runtime_family_contract_selftest_progress_present=0",
            "age5_full_real_gate0_family_contract_selftest_completed_checks=-",
            "age5_full_real_gate0_family_contract_selftest_total_checks=-",
            "age5_full_real_gate0_family_contract_selftest_checks_text=-",
            "age5_full_real_gate0_family_contract_selftest_current_probe=-",
            "age5_full_real_gate0_family_contract_selftest_last_completed_probe=-",
            "age5_full_real_gate0_family_contract_selftest_progress_present=0",
            "age5_full_real_gate0_family_transport_contract_selftest_completed_checks=-",
            "age5_full_real_gate0_family_transport_contract_selftest_total_checks=-",
            "age5_full_real_gate0_family_transport_contract_selftest_checks_text=-",
            "age5_full_real_gate0_family_transport_contract_selftest_current_probe=-",
            "age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe=-",
            "age5_full_real_gate0_family_transport_contract_selftest_progress_present=0",
            "age5_full_real_gate0_transport_family_contract_selftest_completed_checks=-",
            "age5_full_real_gate0_transport_family_contract_selftest_total_checks=-",
            "age5_full_real_gate0_transport_family_contract_selftest_checks_text=-",
            "age5_full_real_gate0_transport_family_contract_selftest_current_probe=-",
            "age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe=-",
            "age5_full_real_gate0_transport_family_contract_selftest_progress_present=0",
            "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks=-",
            "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks=-",
            "age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text=-",
            "age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe=-",
            "age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe=-",
            "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present=0",
            "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks=-",
            "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks=-",
            "age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text=-",
            "age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe=-",
            "age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe=-",
            "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present=0",
            "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks=-",
            "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks=-",
            "age5_full_real_lang_surface_family_transport_contract_selftest_checks_text=-",
            "age5_full_real_lang_surface_family_transport_contract_selftest_current_probe=-",
            "age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe=-",
            "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present=0",
            "age5_full_real_bogae_alias_family_contract_selftest_completed_checks=-",
            "age5_full_real_bogae_alias_family_contract_selftest_total_checks=-",
            "age5_full_real_bogae_alias_family_contract_selftest_checks_text=-",
            "age5_full_real_bogae_alias_family_contract_selftest_current_probe=-",
            "age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe=-",
            "age5_full_real_bogae_alias_family_contract_selftest_progress_present=0",
            f"report_index={q(index_path)}",
            f"aggregate_status_line={q('-')}",
            f"aggregate_status_parse={q(parse_path)}",
            f"generated_at_utc={q('-')}",
            f"reason={q('invalid_or_missing_aggregate_status_parse')}",
        ]
        return " ".join(parts) + "\n", False

    parsed = parse_doc.get("parsed")
    if not isinstance(parsed, dict):
        parts = [
            f"schema={q(SCHEMA)}",
            "status=fail",
            "overall_ok=0",
            "failed_steps=-1",
            "aggregate_status=fail",
            "age4_proof_ok=0",
            "age4_proof_failed_criteria=-1",
            "age5_full_real_w107_golden_index_selftest_active_cases=-",
            "age5_full_real_w107_golden_index_selftest_inactive_cases=-",
            "age5_full_real_w107_golden_index_selftest_index_codes=-",
            "age5_full_real_w107_golden_index_selftest_current_probe=-",
            "age5_full_real_w107_golden_index_selftest_last_completed_probe=-",
            "age5_full_real_w107_golden_index_selftest_progress_present=0",
            "age5_full_real_w107_progress_contract_selftest_completed_checks=-",
            "age5_full_real_w107_progress_contract_selftest_total_checks=-",
            "age5_full_real_w107_progress_contract_selftest_checks_text=-",
            "age5_full_real_w107_progress_contract_selftest_current_probe=-",
            "age5_full_real_w107_progress_contract_selftest_last_completed_probe=-",
            "age5_full_real_w107_progress_contract_selftest_progress_present=0",
            "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks=-",
            "age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks=-",
            "age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text=-",
            "age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe=-",
            "age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe=-",
            "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present=0",
            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks=-",
            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks=-",
            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text=-",
            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe=-",
            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe=-",
            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present=0",
            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks=-",
            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks=-",
            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text=-",
            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe=-",
            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe=-",
            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present=0",
            "age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks=-",
            "age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks=-",
            "age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text=-",
            "age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe=-",
            "age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe=-",
            "age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present=0",
            "age5_full_real_proof_certificate_family_contract_selftest_completed_checks=-",
            "age5_full_real_proof_certificate_family_contract_selftest_total_checks=-",
            "age5_full_real_proof_certificate_family_contract_selftest_checks_text=-",
            "age5_full_real_proof_certificate_family_contract_selftest_current_probe=-",
            "age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe=-",
            "age5_full_real_proof_certificate_family_contract_selftest_progress_present=0",
            "age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks=-",
            "age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks=-",
            "age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text=-",
            "age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe=-",
            "age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe=-",
            "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present=0",
            "age5_full_real_proof_family_contract_selftest_completed_checks=-",
            "age5_full_real_proof_family_contract_selftest_total_checks=-",
            "age5_full_real_proof_family_contract_selftest_checks_text=-",
            "age5_full_real_proof_family_contract_selftest_current_probe=-",
            "age5_full_real_proof_family_contract_selftest_last_completed_probe=-",
            "age5_full_real_proof_family_contract_selftest_progress_present=0",
            "age5_full_real_bogae_alias_family_contract_selftest_completed_checks=-",
            "age5_full_real_bogae_alias_family_contract_selftest_total_checks=-",
            "age5_full_real_bogae_alias_family_contract_selftest_checks_text=-",
            "age5_full_real_bogae_alias_family_contract_selftest_current_probe=-",
            "age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe=-",
            "age5_full_real_bogae_alias_family_contract_selftest_progress_present=0",
            f"report_index={q(index_path)}",
            f"aggregate_status_line={q(str(parse_doc.get('status_line_path', '-')))}",
            f"aggregate_status_parse={q(parse_path)}",
            f"generated_at_utc={q('-')}",
            f"reason={q('invalid_parse_payload')}",
        ]
        return " ".join(parts) + "\n", False

    aggregate_status = str(parsed.get("status", "fail")).strip() or "fail"
    parsed_overall_ok = str(parsed.get("overall_ok", "0")).strip() == "1"
    failed_steps = count_failed_steps(index_doc)
    overall_ok = failed_steps == 0 if failed_steps >= 0 else parsed_overall_ok
    age4_proof_ok = "1" if str(parsed.get("age4_proof_ok", "0")).strip() == "1" else "0"
    try:
        age4_proof_failed_criteria = int(str(parsed.get("age4_proof_failed_criteria", "-1")).strip())
    except ValueError:
        age4_proof_failed_criteria = -1
    reason = str(parsed.get("reason", "-")).strip() or "-"
    if reason == "-" and failed_steps > 0:
        reason = f"failed_steps={failed_steps}"
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
    final_status = "pass" if overall_ok and aggregate_status == "pass" and failed_steps == 0 else "fail"
    parts = [
        f"schema={q(SCHEMA)}",
        f"status={final_status}",
        f"overall_ok={int(overall_ok)}",
        f"failed_steps={failed_steps}",
        f"aggregate_status={aggregate_status}",
        f"age4_proof_ok={age4_proof_ok}",
        f"age4_proof_failed_criteria={age4_proof_failed_criteria}",
        f"{AGE5_W107_PROGRESS_KEYS[0]}={age5_w107_progress[AGE5_W107_PROGRESS_KEYS[0]]}",
        f"{AGE5_W107_PROGRESS_KEYS[1]}={age5_w107_progress[AGE5_W107_PROGRESS_KEYS[1]]}",
        f"{AGE5_W107_PROGRESS_KEYS[2]}={age5_w107_progress[AGE5_W107_PROGRESS_KEYS[2]]}",
        f"{AGE5_W107_PROGRESS_KEYS[3]}={age5_w107_progress[AGE5_W107_PROGRESS_KEYS[3]]}",
        f"{AGE5_W107_PROGRESS_KEYS[4]}={age5_w107_progress[AGE5_W107_PROGRESS_KEYS[4]]}",
        f"{AGE5_W107_PROGRESS_KEYS[5]}={age5_w107_progress[AGE5_W107_PROGRESS_KEYS[5]]}",
        f"{AGE5_W107_CONTRACT_PROGRESS_KEYS[0]}={age5_w107_contract_progress[AGE5_W107_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_W107_CONTRACT_PROGRESS_KEYS[1]}={age5_w107_contract_progress[AGE5_W107_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_W107_CONTRACT_PROGRESS_KEYS[2]}={age5_w107_contract_progress[AGE5_W107_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_W107_CONTRACT_PROGRESS_KEYS[3]}={age5_w107_contract_progress[AGE5_W107_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_W107_CONTRACT_PROGRESS_KEYS[4]}={age5_w107_contract_progress[AGE5_W107_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_W107_CONTRACT_PROGRESS_KEYS[5]}={age5_w107_contract_progress[AGE5_W107_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0]}={age5_age1_immediate_proof_operation_contract_progress[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1]}={age5_age1_immediate_proof_operation_contract_progress[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2]}={age5_age1_immediate_proof_operation_contract_progress[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[3]}={age5_age1_immediate_proof_operation_contract_progress[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4]}={age5_age1_immediate_proof_operation_contract_progress[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5]}={age5_age1_immediate_proof_operation_contract_progress[AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]}={age5_proof_certificate_v1_consumer_transport_contract_progress[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]}={age5_proof_certificate_v1_consumer_transport_contract_progress[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]}={age5_proof_certificate_v1_consumer_transport_contract_progress[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]}={age5_proof_certificate_v1_consumer_transport_contract_progress[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]}={age5_proof_certificate_v1_consumer_transport_contract_progress[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]}={age5_proof_certificate_v1_consumer_transport_contract_progress[AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0]}={age5_proof_certificate_v1_verify_report_digest_contract_progress[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1]}={age5_proof_certificate_v1_verify_report_digest_contract_progress[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2]}={age5_proof_certificate_v1_verify_report_digest_contract_progress[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[3]}={age5_proof_certificate_v1_verify_report_digest_contract_progress[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4]}={age5_proof_certificate_v1_verify_report_digest_contract_progress[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5]}={age5_proof_certificate_v1_verify_report_digest_contract_progress[AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0]}={age5_proof_certificate_v1_family_contract_progress[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1]}={age5_proof_certificate_v1_family_contract_progress[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2]}={age5_proof_certificate_v1_family_contract_progress[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[3]}={age5_proof_certificate_v1_family_contract_progress[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4]}={age5_proof_certificate_v1_family_contract_progress[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5]}={age5_proof_certificate_v1_family_contract_progress[AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0]}={age5_proof_certificate_family_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1]}={age5_proof_certificate_family_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2]}={age5_proof_certificate_family_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[3]}={age5_proof_certificate_family_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4]}={age5_proof_certificate_family_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5]}={age5_proof_certificate_family_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]}={age5_proof_certificate_family_transport_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]}={age5_proof_certificate_family_transport_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]}={age5_proof_certificate_family_transport_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]}={age5_proof_certificate_family_transport_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]}={age5_proof_certificate_family_transport_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]}={age5_proof_certificate_family_transport_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0]}={age5_proof_family_contract_progress[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1]}={age5_proof_family_contract_progress[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2]}={age5_proof_family_contract_progress[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[3]}={age5_proof_family_contract_progress[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4]}={age5_proof_family_contract_progress[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5]}={age5_proof_family_contract_progress[AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]}={age5_proof_family_transport_contract_progress[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]}={age5_proof_family_transport_contract_progress[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]}={age5_proof_family_transport_contract_progress[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]}={age5_proof_family_transport_contract_progress[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]}={age5_proof_family_transport_contract_progress[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]}={age5_proof_family_transport_contract_progress[AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]}={age5_lang_surface_family_contract_progress[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]}={age5_lang_surface_family_contract_progress[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]}={age5_lang_surface_family_contract_progress[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]}={age5_lang_surface_family_contract_progress[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]}={age5_lang_surface_family_contract_progress[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]}={age5_lang_surface_family_contract_progress[AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0]}={age5_lang_runtime_family_contract_progress[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1]}={age5_lang_runtime_family_contract_progress[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2]}={age5_lang_runtime_family_contract_progress[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[3]}={age5_lang_runtime_family_contract_progress[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4]}={age5_lang_runtime_family_contract_progress[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5]}={age5_lang_runtime_family_contract_progress[AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[0]}={age5_gate0_family_contract_progress[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[1]}={age5_gate0_family_contract_progress[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[2]}={age5_gate0_family_contract_progress[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[3]}={age5_gate0_family_contract_progress[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[4]}={age5_gate0_family_contract_progress[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[5]}={age5_gate0_family_contract_progress[AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]}={age5_gate0_surface_family_contract_progress[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]}={age5_gate0_surface_family_contract_progress[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]}={age5_gate0_surface_family_contract_progress[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]}={age5_gate0_surface_family_contract_progress[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]}={age5_gate0_surface_family_contract_progress[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]}={age5_gate0_surface_family_contract_progress[AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]}={age5_gate0_surface_family_transport_contract_progress[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]}={age5_gate0_surface_family_transport_contract_progress[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]}={age5_gate0_surface_family_transport_contract_progress[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]}={age5_gate0_surface_family_transport_contract_progress[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]}={age5_gate0_surface_family_transport_contract_progress[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]}={age5_gate0_surface_family_transport_contract_progress[AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]}={age5_lang_runtime_family_transport_contract_progress[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]}={age5_lang_runtime_family_transport_contract_progress[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]}={age5_lang_runtime_family_transport_contract_progress[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]}={age5_lang_runtime_family_transport_contract_progress[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]}={age5_lang_runtime_family_transport_contract_progress[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]}={age5_lang_runtime_family_transport_contract_progress[AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]}={age5_gate0_runtime_family_transport_contract_progress[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]}={age5_gate0_runtime_family_transport_contract_progress[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]}={age5_gate0_runtime_family_transport_contract_progress[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]}={age5_gate0_runtime_family_transport_contract_progress[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]}={age5_gate0_runtime_family_transport_contract_progress[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]}={age5_gate0_runtime_family_transport_contract_progress[AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]}={age5_gate0_family_transport_contract_progress[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]}={age5_gate0_family_transport_contract_progress[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]}={age5_gate0_family_transport_contract_progress[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]}={age5_gate0_family_transport_contract_progress[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]}={age5_gate0_family_transport_contract_progress[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]}={age5_gate0_family_transport_contract_progress[AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[0]}={age5_gate0_transport_family_contract_progress[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[1]}={age5_gate0_transport_family_contract_progress[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[2]}={age5_gate0_transport_family_contract_progress[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[3]}={age5_gate0_transport_family_contract_progress[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[4]}={age5_gate0_transport_family_contract_progress[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[5]}={age5_gate0_transport_family_contract_progress[AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]}={age5_gate0_transport_family_transport_contract_progress[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]}={age5_gate0_transport_family_transport_contract_progress[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]}={age5_gate0_transport_family_transport_contract_progress[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]}={age5_gate0_transport_family_transport_contract_progress[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]}={age5_gate0_transport_family_transport_contract_progress[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]}={age5_gate0_transport_family_transport_contract_progress[AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]}={age5_lang_surface_family_transport_contract_progress[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]}={age5_lang_surface_family_transport_contract_progress[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]}={age5_lang_surface_family_transport_contract_progress[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]}={age5_lang_surface_family_transport_contract_progress[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]}={age5_lang_surface_family_transport_contract_progress[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]}={age5_lang_surface_family_transport_contract_progress[AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0]}={age5_bogae_alias_family_contract_progress[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1]}={age5_bogae_alias_family_contract_progress[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2]}={age5_bogae_alias_family_contract_progress[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[3]}={age5_bogae_alias_family_contract_progress[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4]}={age5_bogae_alias_family_contract_progress[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5]}={age5_bogae_alias_family_contract_progress[AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5]]}",
        f"{AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]}={age5_bogae_alias_family_transport_contract_progress[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]}={age5_bogae_alias_family_transport_contract_progress[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]}={age5_bogae_alias_family_transport_contract_progress[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]}={age5_bogae_alias_family_transport_contract_progress[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]}={age5_bogae_alias_family_transport_contract_progress[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]}={age5_bogae_alias_family_transport_contract_progress[AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
        f"report_index={q(index_path)}",
        f"aggregate_status_line={q(str(parse_doc.get('status_line_path', '-')))}",
        f"aggregate_status_parse={q(parse_path)}",
        f"generated_at_utc={q(str(parsed.get('generated_at_utc', '-')))}",
        f"reason={q(reason)}",
    ]
    final_ok = final_status == "pass"
    return " ".join(parts) + "\n", final_ok


def main() -> int:
    parser = argparse.ArgumentParser(description="Render one-line final CI gate status")
    parser.add_argument("--aggregate-status-parse", required=True, help="path to aggregate status-line parse detjson")
    parser.add_argument(
        "--gate-index",
        default="build/reports/ci_gate_report_index.detjson",
        help="path to aggregate gate index report",
    )
    parser.add_argument("--out", required=True, help="output final status-line txt path")
    parser.add_argument("--fail-on-bad", action="store_true", help="return non-zero when status is fail")
    args = parser.parse_args()

    parse_path = Path(args.aggregate_status_parse)
    index_path = Path(args.gate_index)
    out_path = Path(args.out)
    parse_doc = load_json(parse_path)
    index_doc = load_json(index_path)
    line, ok = build_line(parse_path, parse_doc, index_path, index_doc)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(line, encoding="utf-8")
    print(f"[ci-gate-final-status-line] out={out_path} ok={int(ok)}")
    if args.fail_on_bad and not ok:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
