#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
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
AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_gate0_family_contract_selftest_completed_checks",
    "age5_full_real_gate0_family_contract_selftest_total_checks",
    "age5_full_real_gate0_family_contract_selftest_checks_text",
    "age5_full_real_gate0_family_contract_selftest_current_probe",
    "age5_full_real_gate0_family_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_family_contract_selftest_progress_present",
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
AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present",
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
AGE5_POLICY_KEYS = (
    "age5_policy_age4_proof_snapshot_fields_text",
    "age5_policy_age4_proof_snapshot_text",
    "age5_policy_age4_proof_source_snapshot_fields_text",
    "age5_policy_age4_proof_gate_result_present",
    "age5_policy_age4_proof_gate_result_parity",
    "age5_policy_age4_proof_final_status_parse_present",
    "age5_policy_age4_proof_final_status_parse_parity",
)


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate ci_gate_result.detjson")
    parser.add_argument("--result", required=True, help="path to ci_gate_result.detjson")
    parser.add_argument("--final-status-parse", required=True, help="path to ci_gate_final_status_line_parse.detjson")
    parser.add_argument("--summary-line", required=True, help="path to ci_gate_summary_line.txt")
    parser.add_argument("--require-pass", action="store_true", help="also require status=pass and ok=true")
    args = parser.parse_args()

    result_path = Path(args.result)
    final_parse_path = Path(args.final_status_parse)
    summary_line_path = Path(args.summary_line)
    result_doc = load_json(result_path)
    if result_doc is None:
        print(f"invalid result json: {result_path}", file=sys.stderr)
        return 1
    if result_doc.get("schema") != EXPECTED_SCHEMA:
        print(f"schema mismatch: {result_doc.get('schema')}", file=sys.stderr)
        return 1

    final_parse_doc = load_json(final_parse_path)
    if final_parse_doc is None:
        print(f"invalid final parse json: {final_parse_path}", file=sys.stderr)
        return 1
    parsed = final_parse_doc.get("parsed")
    if not isinstance(parsed, dict):
        print("final parse json missing parsed object", file=sys.stderr)
        return 1

    summary_line = summary_line_path.read_text(encoding="utf-8").strip() if summary_line_path.exists() else ""
    if not summary_line:
        print(f"summary line missing/empty: {summary_line_path}", file=sys.stderr)
        return 1
    if str(result_doc.get("summary_line", "")).strip() != summary_line:
        print("summary_line mismatch", file=sys.stderr)
        return 1
    if str(result_doc.get("summary_line_path", "")).strip() != str(summary_line_path):
        print("summary_line_path mismatch", file=sys.stderr)
        return 1
    if str(result_doc.get("final_status_parse_path", "")).strip() != str(final_parse_path):
        print("final_status_parse_path mismatch", file=sys.stderr)
        return 1

    expected_status = str(parsed.get("status", "fail")).strip() or "fail"
    expected_overall_ok = str(parsed.get("overall_ok", "0")).strip() == "1"
    expected_aggregate_status = str(parsed.get("aggregate_status", "fail")).strip() or "fail"
    expected_age4_proof_ok = str(parsed.get(AGE4_PROOF_OK_KEY, "0")).strip() == "1"
    try:
        expected_failed_steps = int(parsed.get("failed_steps", "-1"))
    except ValueError:
        expected_failed_steps = -1
    try:
        expected_age4_proof_failed = int(parsed.get(AGE4_PROOF_FAILED_CRITERIA_KEY, "-1"))
    except ValueError:
        expected_age4_proof_failed = -1
    expected_age4_proof_failed_preview = str(parsed.get(AGE4_PROOF_FAILED_PREVIEW_KEY, "-")).strip() or "-"
    expected_age5_w107_progress = {
        AGE5_W107_PROGRESS_KEYS[0]: str(parsed.get(AGE5_W107_PROGRESS_KEYS[0], "-")).strip() or "-",
        AGE5_W107_PROGRESS_KEYS[1]: str(parsed.get(AGE5_W107_PROGRESS_KEYS[1], "-")).strip() or "-",
        AGE5_W107_PROGRESS_KEYS[2]: str(parsed.get(AGE5_W107_PROGRESS_KEYS[2], "-")).strip() or "-",
        AGE5_W107_PROGRESS_KEYS[3]: str(parsed.get(AGE5_W107_PROGRESS_KEYS[3], "-")).strip() or "-",
        AGE5_W107_PROGRESS_KEYS[4]: str(parsed.get(AGE5_W107_PROGRESS_KEYS[4], "-")).strip() or "-",
        AGE5_W107_PROGRESS_KEYS[5]: str(parsed.get(AGE5_W107_PROGRESS_KEYS[5], "0")).strip() or "0",
    }
    expected_age5_w107_contract_progress = {
        AGE5_W107_CONTRACT_PROGRESS_KEYS[0]: str(parsed.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[0], "-")).strip() or "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[1]: str(parsed.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[1], "-")).strip() or "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[2]: str(parsed.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[2], "-")).strip() or "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[3]: str(parsed.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[3], "-")).strip() or "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[4]: str(parsed.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[4], "-")).strip() or "-",
        AGE5_W107_CONTRACT_PROGRESS_KEYS[5]: str(parsed.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[5], "0")).strip() or "0",
    }
    expected_age5_age1_immediate_proof_operation_contract_progress = {
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
    expected_age5_proof_certificate_v1_consumer_transport_contract_progress = {
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
    expected_age5_proof_certificate_v1_verify_report_digest_contract_progress = {
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
    expected_age5_proof_certificate_v1_family_contract_progress = {
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
    expected_age5_proof_certificate_family_contract_progress = {
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
    expected_age5_proof_certificate_family_transport_contract_progress = {
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
    expected_age5_proof_family_contract_progress = {
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
    expected_age5_proof_family_transport_contract_progress = {
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
    expected_age5_lang_surface_family_contract_progress = {
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
    expected_age5_lang_runtime_family_contract_progress = {
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
    expected_age5_gate0_family_contract_progress = {
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
    expected_age5_gate0_family_transport_contract_progress = {
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
    expected_age5_gate0_transport_family_contract_progress = {
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
    expected_age5_gate0_transport_family_transport_contract_progress = {
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
    expected_age5_gate0_surface_family_transport_contract_progress = {
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
    expected_age5_lang_runtime_family_transport_contract_progress = {
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
    expected_age5_gate0_runtime_family_transport_contract_progress = {
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
    expected_age5_lang_surface_family_transport_contract_progress = {
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
    expected_age5_bogae_alias_family_contract_progress = {
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
    expected_age5_bogae_alias_family_transport_contract_progress = {
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
    expected_age5_policy = {key: str(parsed.get(key, "")).strip() for key in AGE5_POLICY_KEYS}

    if str(result_doc.get("status", "")).strip() != expected_status:
        print("status mismatch", file=sys.stderr)
        return 1
    if bool(result_doc.get("overall_ok", False)) != expected_overall_ok:
        print("overall_ok mismatch", file=sys.stderr)
        return 1
    if str(result_doc.get("aggregate_status", "")).strip() != expected_aggregate_status:
        print("aggregate_status mismatch", file=sys.stderr)
        return 1
    if bool(result_doc.get(AGE4_PROOF_OK_KEY, False)) != expected_age4_proof_ok:
        print("age4_proof_ok mismatch", file=sys.stderr)
        return 1
    if int(result_doc.get(AGE4_PROOF_FAILED_CRITERIA_KEY, -1)) != expected_age4_proof_failed:
        print("age4_proof_failed_criteria mismatch", file=sys.stderr)
        return 1
    if str(result_doc.get(AGE4_PROOF_FAILED_PREVIEW_KEY, "-")).strip() != expected_age4_proof_failed_preview:
        print("age4_proof_failed_preview mismatch", file=sys.stderr)
        return 1
    if str(result_doc.get(AGE5_W107_PROGRESS_KEYS[5], "0")).strip() not in {"0", "1"}:
        print("age5_full_real_w107_golden_index_selftest_progress_present invalid", file=sys.stderr)
        return 1
    for key in AGE5_W107_PROGRESS_KEYS[:3]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_W107_PROGRESS_KEYS:
        if str(result_doc.get(key, "")).strip() != expected_age5_w107_progress[key]:
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    if str(result_doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {"0", "1"}:
        print("age5_full_real_w107_progress_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    for key in AGE5_W107_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_W107_CONTRACT_PROGRESS_KEYS:
        if str(result_doc.get(key, "")).strip() != expected_age5_w107_contract_progress[key]:
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    if str(result_doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {
        "0",
        "1",
    }:
        print(
            "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    for key in AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS:
        if str(result_doc.get(key, "")).strip() != expected_age5_age1_immediate_proof_operation_contract_progress[key]:
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    if str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {
        "0",
        "1",
    }:
        print(
            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    for key in AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if str(result_doc.get(key, "")).strip() != expected_age5_proof_certificate_v1_consumer_transport_contract_progress[key]:
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    if str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {
        "0",
        "1",
    }:
        print(
            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    for key in AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS:
        if str(result_doc.get(key, "")).strip() != expected_age5_proof_certificate_v1_verify_report_digest_contract_progress[key]:
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    if str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {
        "0",
        "1",
    }:
        print(
            "age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    for key in AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS:
        if str(result_doc.get(key, "")).strip() != expected_age5_proof_certificate_v1_family_contract_progress[key]:
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    if str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {
        "0",
        "1",
    }:
        print(
            "age5_full_real_proof_certificate_family_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    for key in AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS:
        if str(result_doc.get(key, "")).strip() != expected_age5_proof_certificate_family_contract_progress[key]:
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    if str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {
        "0",
        "1",
    }:
        print(
            "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    for key in AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if str(result_doc.get(key, "")).strip() != expected_age5_proof_certificate_family_transport_contract_progress[key]:
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    if str(result_doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {"0", "1"}:
        print("age5_full_real_proof_family_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if str(result_doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {
        "0",
        "1",
    }:
        print("age5_full_real_proof_family_transport_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {"0", "1"}:
        print("age5_full_real_lang_surface_family_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if str(result_doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {"0", "1"}:
        print("age5_full_real_lang_runtime_family_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if str(result_doc.get(AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {"0", "1"}:
        print("age5_full_real_gate0_family_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if str(result_doc.get(AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {
        "0",
        "1",
    }:
        print("age5_full_real_gate0_family_transport_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {
        "0",
        "1",
    }:
        print(
            "age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if str(result_doc.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {
        "0",
        "1",
    }:
        print(
            "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if str(result_doc.get(AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {
        "0",
        "1",
    }:
        print(
            "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if str(result_doc.get(AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {
        "0",
        "1",
    }:
        print(
            "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {
        "0",
        "1",
    }:
        print(
            "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    for key in AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS:
        if str(result_doc.get(key, "")).strip() != expected_age5_proof_family_contract_progress[key]:
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    for key in AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if str(result_doc.get(key, "")).strip() != expected_age5_proof_family_transport_contract_progress[key]:
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    for key in AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS:
        if str(result_doc.get(key, "")).strip() != expected_age5_lang_surface_family_contract_progress[key]:
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    for key in AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS:
        if str(result_doc.get(key, "")).strip() != expected_age5_lang_runtime_family_contract_progress[key]:
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    for key in AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS:
        if str(result_doc.get(key, "")).strip() != expected_age5_gate0_family_contract_progress[key]:
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    for key in AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if (
            str(result_doc.get(key, "")).strip()
            != expected_age5_gate0_family_transport_contract_progress[key]
        ):
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    for key in AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS:
        if (
            str(result_doc.get(key, "")).strip()
            != expected_age5_gate0_transport_family_contract_progress[key]
        ):
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    for key in AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if (
            str(result_doc.get(key, "")).strip()
            != expected_age5_gate0_transport_family_transport_contract_progress[key]
        ):
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    for key in AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if (
            str(result_doc.get(key, "")).strip()
            != expected_age5_gate0_surface_family_transport_contract_progress[key]
        ):
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    for key in AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if (
            str(result_doc.get(key, "")).strip()
            != expected_age5_lang_runtime_family_transport_contract_progress[key]
        ):
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    for key in AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if (
            str(result_doc.get(key, "")).strip()
            != expected_age5_gate0_runtime_family_transport_contract_progress[key]
        ):
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    for key in AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if (
            str(result_doc.get(key, "")).strip()
            != expected_age5_lang_surface_family_transport_contract_progress[key]
        ):
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    if str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {"0", "1"}:
        print("age5_full_real_bogae_alias_family_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    for key in AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS:
        if str(result_doc.get(key, "")).strip() != expected_age5_bogae_alias_family_contract_progress[key]:
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    if str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], "0")).strip() not in {
        "0",
        "1",
    }:
        print("age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    for key in AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[:2]:
        value = str(result_doc.get(key, "-")).strip() or "-"
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"{key} invalid int", file=sys.stderr)
            return 1
    for key in AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        if str(result_doc.get(key, "")).strip() != expected_age5_bogae_alias_family_transport_contract_progress[key]:
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    for key in AGE5_POLICY_KEYS:
        if str(result_doc.get(key, "")).strip() != expected_age5_policy[key]:
            print(f"{key} mismatch", file=sys.stderr)
            return 1
    if int(result_doc.get("failed_steps", -1)) != expected_failed_steps:
        print("failed_steps mismatch", file=sys.stderr)
        return 1

    expected_ok = (
        expected_status == "pass"
        and expected_overall_ok
        and expected_aggregate_status == "pass"
        and expected_failed_steps == 0
    )
    if bool(result_doc.get("ok", False)) != expected_ok:
        print("ok mismatch", file=sys.stderr)
        return 1
    if args.require_pass and not expected_ok:
        print("result is not pass", file=sys.stderr)
        return 1

    print(
        "[ci-gate-result-check] ok "
        f"status={result_doc.get('status')} ok={int(bool(result_doc.get('ok', False)))} "
        f"failed_steps={result_doc.get('failed_steps')} "
        f"age4_proof_ok={int(bool(result_doc.get(AGE4_PROOF_OK_KEY, False)))} "
        f"age4_proof_failed={result_doc.get(AGE4_PROOF_FAILED_CRITERIA_KEY)} "
        f"age4_proof_failed_preview={str(result_doc.get(AGE4_PROOF_FAILED_PREVIEW_KEY, '-')).strip() or '-'} "
        f"age5_w107_active={str(result_doc.get(AGE5_W107_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_w107_inactive={str(result_doc.get(AGE5_W107_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_w107_index_codes={str(result_doc.get(AGE5_W107_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_w107_last_completed_probe={str(result_doc.get(AGE5_W107_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_w107_progress={str(result_doc.get(AGE5_W107_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_w107_contract_completed={str(result_doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_w107_contract_total={str(result_doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_w107_contract_checks_text={str(result_doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_w107_contract_last_completed_probe={str(result_doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_w107_contract_progress={str(result_doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_age1_immediate_proof_operation_contract_completed={str(result_doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_age1_immediate_proof_operation_contract_total={str(result_doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_age1_immediate_proof_operation_contract_checks_text={str(result_doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_age1_immediate_proof_operation_contract_last_completed_probe={str(result_doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_age1_immediate_proof_operation_contract_progress={str(result_doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_proof_certificate_v1_consumer_contract_completed={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_proof_certificate_v1_consumer_contract_total={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_proof_certificate_v1_consumer_contract_checks_text={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_proof_certificate_v1_consumer_contract_last_completed_probe={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_proof_certificate_v1_consumer_contract_progress={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_proof_certificate_v1_verify_report_digest_contract_completed={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_proof_certificate_v1_verify_report_digest_contract_total={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_proof_certificate_v1_verify_report_digest_contract_checks_text={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_proof_certificate_v1_verify_report_digest_contract_last_completed_probe={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_proof_certificate_v1_verify_report_digest_contract_progress={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_proof_certificate_v1_family_contract_completed={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_proof_certificate_v1_family_contract_total={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_proof_certificate_v1_family_contract_checks_text={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_proof_certificate_v1_family_contract_last_completed_probe={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_proof_certificate_v1_family_contract_progress={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_proof_certificate_family_contract_completed={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_proof_certificate_family_contract_total={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_proof_certificate_family_contract_checks_text={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_proof_certificate_family_contract_last_completed_probe={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_proof_certificate_family_contract_progress={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_proof_certificate_family_transport_contract_completed={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_proof_certificate_family_transport_contract_total={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_proof_certificate_family_transport_contract_checks_text={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_proof_certificate_family_transport_contract_last_completed_probe={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_proof_certificate_family_transport_contract_progress={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_proof_family_contract_completed={str(result_doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_proof_family_contract_total={str(result_doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_proof_family_contract_checks_text={str(result_doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_proof_family_contract_last_completed_probe={str(result_doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_proof_family_contract_progress={str(result_doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_proof_family_transport_contract_completed={str(result_doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_proof_family_transport_contract_total={str(result_doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_proof_family_transport_contract_checks_text={str(result_doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_proof_family_transport_contract_last_completed_probe={str(result_doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_proof_family_transport_contract_progress={str(result_doc.get(AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_lang_surface_family_contract_completed={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_lang_surface_family_contract_total={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_lang_surface_family_contract_checks_text={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_lang_surface_family_contract_last_completed_probe={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_lang_surface_family_contract_progress={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_lang_runtime_family_contract_completed={str(result_doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_lang_runtime_family_contract_total={str(result_doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_lang_runtime_family_contract_checks_text={str(result_doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_lang_runtime_family_contract_last_completed_probe={str(result_doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_lang_runtime_family_contract_progress={str(result_doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_gate0_family_contract_completed={str(result_doc.get(AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_gate0_family_contract_total={str(result_doc.get(AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_gate0_family_contract_checks_text={str(result_doc.get(AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_gate0_family_contract_last_completed_probe={str(result_doc.get(AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_gate0_family_contract_progress={str(result_doc.get(AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_gate0_family_transport_contract_completed={str(result_doc.get(AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_gate0_family_transport_contract_total={str(result_doc.get(AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_gate0_family_transport_contract_checks_text={str(result_doc.get(AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_gate0_family_transport_contract_last_completed_probe={str(result_doc.get(AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_gate0_family_transport_contract_progress={str(result_doc.get(AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_gate0_transport_family_contract_completed={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_gate0_transport_family_contract_total={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_gate0_transport_family_contract_checks_text={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_gate0_transport_family_contract_last_completed_probe={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_gate0_transport_family_contract_progress={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_gate0_transport_family_transport_contract_completed={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_gate0_transport_family_transport_contract_total={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_gate0_transport_family_transport_contract_checks_text={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_gate0_transport_family_transport_contract_current_probe={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'}"
        f" age5_gate0_transport_family_transport_contract_last_completed_probe={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_gate0_transport_family_transport_contract_progress={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_gate0_surface_family_transport_contract_completed={str(result_doc.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_gate0_surface_family_transport_contract_total={str(result_doc.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_gate0_surface_family_transport_contract_checks_text={str(result_doc.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_gate0_surface_family_transport_contract_current_probe={str(result_doc.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'}"
        f" age5_gate0_surface_family_transport_contract_last_completed_probe={str(result_doc.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_gate0_surface_family_transport_contract_progress={str(result_doc.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_lang_runtime_family_transport_contract_completed={str(result_doc.get(AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_lang_runtime_family_transport_contract_total={str(result_doc.get(AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_lang_runtime_family_transport_contract_checks_text={str(result_doc.get(AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_lang_runtime_family_transport_contract_last_completed_probe={str(result_doc.get(AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_lang_runtime_family_transport_contract_progress={str(result_doc.get(AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_gate0_runtime_family_transport_contract_completed={str(result_doc.get(AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_gate0_runtime_family_transport_contract_total={str(result_doc.get(AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_gate0_runtime_family_transport_contract_checks_text={str(result_doc.get(AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_gate0_runtime_family_transport_contract_last_completed_probe={str(result_doc.get(AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_gate0_runtime_family_transport_contract_progress={str(result_doc.get(AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_lang_surface_family_transport_contract_completed={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_lang_surface_family_transport_contract_total={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_lang_surface_family_transport_contract_checks_text={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_lang_surface_family_transport_contract_last_completed_probe={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_lang_surface_family_transport_contract_progress={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_bogae_alias_family_contract_completed={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_bogae_alias_family_contract_total={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_bogae_alias_family_contract_checks_text={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_bogae_alias_family_contract_last_completed_probe={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_bogae_alias_family_contract_progress={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
        f" age5_bogae_alias_family_transport_contract_completed={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'}"
        f" age5_bogae_alias_family_transport_contract_total={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'}"
        f" age5_bogae_alias_family_transport_contract_checks_text={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'}"
        f" age5_bogae_alias_family_transport_contract_last_completed_probe={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'}"
        f" age5_bogae_alias_family_transport_contract_progress={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
