#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TESTS_DIR = ROOT / "tests"
if str(TESTS_DIR) not in sys.path:
    sys.path.insert(0, str(TESTS_DIR))

from _ci_age5_combined_heavy_contract import (  # type: ignore
    build_age5_combined_heavy_child_summary_default_text_transport_fields,
    build_age5_combined_heavy_full_real_source_trace,
)


SCHEMA = "ddn.ci.aggregate_gate_status_line.v1"
AGE5_CHILD_STATUS_KEYS = (
    "age5_combined_heavy_full_real_status",
    "age5_combined_heavy_runtime_helper_negative_status",
    "age5_combined_heavy_group_id_summary_negative_status",
)
AGE5_CHILD_STATUS_VALUES = {"pass", "fail", "skipped"}
AGE5_CHILD_SUMMARY_DEFAULT_TEXT_FIELDS = build_age5_combined_heavy_child_summary_default_text_transport_fields()
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


def age5_w107_progress_value(doc: dict | None, key: str, *, fallback: str) -> str:
    if not isinstance(doc, dict):
        return fallback
    value = str(doc.get(key, "")).strip()
    return value or fallback


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


def failed_count(doc: dict | None, key: str) -> int:
    if not isinstance(doc, dict):
        return -1
    rows = doc.get(key)
    if not isinstance(rows, list):
        return 0
    return len(rows)


def age5_child_status(doc: dict | None, key: str, *, fallback: str) -> str:
    if not isinstance(doc, dict):
        return fallback
    value = str(doc.get(key, "")).strip()
    if value in AGE5_CHILD_STATUS_VALUES:
        return value
    return fallback


def age5_child_default_text(doc: dict | None, key: str) -> str:
    expected = str(AGE5_CHILD_SUMMARY_DEFAULT_TEXT_FIELDS.get(key, "")).strip()
    if not isinstance(doc, dict):
        return expected
    value = str(doc.get(key, "")).strip()
    return value or expected


def build_line(report_path: Path, payload: dict | None) -> tuple[str, bool]:
    if not isinstance(payload, dict):
        parts = [
            f"schema={q(SCHEMA)}",
            "status=fail",
            "overall_ok=0",
            "seamgrim_failed_steps=-1",
            "age3_failed_criteria=-1",
            "age4_failed_criteria=-1",
            "age4_proof_ok=0",
            "age4_proof_failed_criteria=-1",
            "age5_failed_criteria=-1",
            "age5_combined_heavy_full_real_status=fail",
            "age5_full_real_source_check=0",
            "age5_full_real_source_selftest=0",
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
            "age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks=-",
            "age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks=-",
            "age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text=-",
            "age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe=-",
            "age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe=-",
            "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present=0",
            "age5_full_real_bogae_alias_family_contract_selftest_completed_checks=-",
            "age5_full_real_bogae_alias_family_contract_selftest_total_checks=-",
            "age5_full_real_bogae_alias_family_contract_selftest_checks_text=-",
            "age5_full_real_bogae_alias_family_contract_selftest_current_probe=-",
            "age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe=-",
            "age5_full_real_bogae_alias_family_contract_selftest_progress_present=0",
            "age5_combined_heavy_runtime_helper_negative_status=fail",
            "age5_combined_heavy_group_id_summary_negative_status=fail",
            f"ci_sanity_age5_combined_heavy_child_summary_default_fields={q(AGE5_CHILD_SUMMARY_DEFAULT_TEXT_FIELDS['ci_sanity_age5_combined_heavy_child_summary_default_fields'])}",
            f"ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields={q(AGE5_CHILD_SUMMARY_DEFAULT_TEXT_FIELDS['ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields'])}",
            "oi_failed_packs=-1",
            f"report_path={q(report_path)}",
            f"generated_at_utc={q('-')}",
            f"reason={q('invalid_or_missing_report')}",
        ]
        return " ".join(parts) + "\n", False

    overall_ok = bool(payload.get("overall_ok", False))
    seamgrim = payload.get("seamgrim") if isinstance(payload.get("seamgrim"), dict) else None
    age3 = payload.get("age3") if isinstance(payload.get("age3"), dict) else None
    age4 = payload.get("age4") if isinstance(payload.get("age4"), dict) else None
    age5 = payload.get("age5") if isinstance(payload.get("age5"), dict) else None
    oi = payload.get("oi405_406") if isinstance(payload.get("oi405_406"), dict) else None
    seamgrim_failed = failed_count(seamgrim, "failed_steps")
    age3_failed = failed_count(age3, "failed_criteria")
    age4_failed = failed_count(age4, "failed_criteria")
    age4_proof_ok = bool(age4.get("proof_artifact_ok", False)) if isinstance(age4, dict) else False
    age4_proof_failed = failed_count(age4, "proof_artifact_failed_criteria")
    age5_failed = failed_count(age5, "failed_criteria")
    oi_failed = failed_count(oi, "failed_packs")
    age5_full_real_status = age5_child_status(age5, AGE5_CHILD_STATUS_KEYS[0], fallback="skipped")
    age5_full_real_source_trace = (
        age5.get("full_real_source_trace") if isinstance(age5.get("full_real_source_trace"), dict) else None
    )
    if not isinstance(age5_full_real_source_trace, dict):
        age5_full_real_source_trace = build_age5_combined_heavy_full_real_source_trace()
    age5_full_real_source_check = (
        str(age5_full_real_source_trace.get("smoke_check_script_exists", "0")).strip() or "0"
    )
    age5_full_real_source_selftest = (
        str(age5_full_real_source_trace.get("smoke_check_selftest_script_exists", "0")).strip() or "0"
    )
    age5_w107_progress = {
        AGE5_W107_PROGRESS_KEYS[0]: age5_w107_progress_value(age5, AGE5_W107_PROGRESS_KEYS[0], fallback="-"),
        AGE5_W107_PROGRESS_KEYS[1]: age5_w107_progress_value(age5, AGE5_W107_PROGRESS_KEYS[1], fallback="-"),
        AGE5_W107_PROGRESS_KEYS[2]: age5_w107_progress_value(age5, AGE5_W107_PROGRESS_KEYS[2], fallback="-"),
        AGE5_W107_PROGRESS_KEYS[3]: age5_w107_progress_value(age5, AGE5_W107_PROGRESS_KEYS[3], fallback="-"),
        AGE5_W107_PROGRESS_KEYS[4]: age5_w107_progress_value(age5, AGE5_W107_PROGRESS_KEYS[4], fallback="-"),
        AGE5_W107_PROGRESS_KEYS[5]: age5_w107_progress_value(age5, AGE5_W107_PROGRESS_KEYS[5], fallback="0"),
    }
    age5_w107_contract_progress = {
        AGE5_W107_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_W107_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_W107_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_W107_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_W107_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_W107_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_W107_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_W107_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_W107_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_W107_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_W107_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_W107_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_age1_immediate_proof_operation_contract_progress = {
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_proof_certificate_v1_consumer_transport_contract_progress = {
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_proof_certificate_v1_verify_report_digest_contract_progress = {
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_proof_certificate_v1_family_contract_progress = {
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_proof_certificate_family_contract_progress = {
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_proof_family_contract_progress = {
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_proof_family_transport_contract_progress = {
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_lang_surface_family_contract_progress = {
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_lang_runtime_family_contract_progress = {
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_gate0_family_contract_progress = {
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_gate0_surface_family_contract_progress = {
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_GATE0_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_gate0_surface_family_transport_contract_progress = {
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_lang_surface_family_transport_contract_progress = {
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_lang_runtime_family_transport_contract_progress = {
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_gate0_runtime_family_transport_contract_progress = {
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_gate0_family_transport_contract_progress = {
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_gate0_transport_family_contract_progress = {
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_gate0_transport_family_transport_contract_progress = {
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_proof_certificate_family_transport_contract_progress = {
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_bogae_alias_family_contract_progress = {
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_bogae_alias_family_transport_contract_progress = {
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]: age5_w107_progress_value(
            age5, AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], fallback="-"
        ),
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]: age5_w107_progress_value(
            age5, AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], fallback="-"
        ),
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]: age5_w107_progress_value(
            age5, AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], fallback="-"
        ),
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]: age5_w107_progress_value(
            age5, AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], fallback="-"
        ),
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]: age5_w107_progress_value(
            age5, AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], fallback="-"
        ),
        AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]: age5_w107_progress_value(
            age5, AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], fallback="0"
        ),
    }
    age5_runtime_helper_negative_status = age5_child_status(age5, AGE5_CHILD_STATUS_KEYS[1], fallback="skipped")
    age5_group_id_summary_negative_status = age5_child_status(age5, AGE5_CHILD_STATUS_KEYS[2], fallback="skipped")
    age5_child_summary_default_fields = age5_child_default_text(
        age5, "ci_sanity_age5_combined_heavy_child_summary_default_fields"
    )
    age5_sync_child_summary_default_fields = age5_child_default_text(
        age5, "ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"
    )
    reason = "-"
    if not overall_ok:
        digest = payload.get("failure_digest")
        if isinstance(digest, list) and digest:
            reason = str(digest[0])[:220]
    parts = [
        f"schema={q(SCHEMA)}",
        f"status={'pass' if overall_ok else 'fail'}",
        f"overall_ok={int(overall_ok)}",
        f"seamgrim_failed_steps={seamgrim_failed}",
        f"age3_failed_criteria={age3_failed}",
        f"age4_failed_criteria={age4_failed}",
        f"age4_proof_ok={int(age4_proof_ok)}",
        f"age4_proof_failed_criteria={age4_proof_failed}",
        f"age5_failed_criteria={age5_failed}",
        f"age5_combined_heavy_full_real_status={age5_full_real_status}",
        f"age5_full_real_source_check={age5_full_real_source_check}",
        f"age5_full_real_source_selftest={age5_full_real_source_selftest}",
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
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]}={age5_proof_certificate_family_transport_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0]]}",
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]}={age5_proof_certificate_family_transport_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1]]}",
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]}={age5_proof_certificate_family_transport_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2]]}",
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]}={age5_proof_certificate_family_transport_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3]]}",
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]}={age5_proof_certificate_family_transport_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4]]}",
        f"{AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]}={age5_proof_certificate_family_transport_contract_progress[AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5]]}",
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
        f"age5_combined_heavy_runtime_helper_negative_status={age5_runtime_helper_negative_status}",
        f"age5_combined_heavy_group_id_summary_negative_status={age5_group_id_summary_negative_status}",
        f"ci_sanity_age5_combined_heavy_child_summary_default_fields={q(age5_child_summary_default_fields)}",
        f"ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields={q(age5_sync_child_summary_default_fields)}",
        f"oi_failed_packs={oi_failed}",
        f"report_path={q(report_path)}",
        f"generated_at_utc={q(str(payload.get('generated_at_utc', '-')))}",
        f"reason={q(reason)}",
    ]
    return " ".join(parts) + "\n", overall_ok


def main() -> int:
    parser = argparse.ArgumentParser(description="Render one-line aggregate gate status")
    parser.add_argument("aggregate_report", help="path to ddn.ci.aggregate_report.v1")
    parser.add_argument("--out", required=True, help="output status-line text path")
    parser.add_argument("--fail-on-bad", action="store_true", help="return non-zero when status is fail")
    args = parser.parse_args()

    report_path = Path(args.aggregate_report)
    out_path = Path(args.out)
    payload = load_json(report_path)
    line, overall_ok = build_line(report_path, payload)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(line, encoding="utf-8")
    print(f"[ci-aggregate-status-line] out={out_path} overall_ok={int(overall_ok)}")
    if args.fail_on_bad and not overall_ok:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
