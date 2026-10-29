#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def load_text(path: Path) -> str:
    if not path.exists():
        return ""
    try:
        return path.read_text(encoding="utf-8").strip()
    except Exception:
        return ""


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
AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS = (
    "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks",
    "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks",
    "age5_full_real_lang_surface_family_transport_contract_selftest_checks_text",
    "age5_full_real_lang_surface_family_transport_contract_selftest_current_probe",
    "age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present",
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
AGE5_POLICY_KEYS = (
    "age5_policy_age4_proof_snapshot_fields_text",
    "age5_policy_age4_proof_snapshot_text",
    "age5_policy_age4_proof_source_snapshot_fields_text",
    "age5_policy_age4_proof_gate_result_present",
    "age5_policy_age4_proof_gate_result_parity",
    "age5_policy_age4_proof_final_status_parse_present",
    "age5_policy_age4_proof_final_status_parse_parity",
)


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate consistency across final CI gate artifacts")
    parser.add_argument("--summary-line", required=True, help="path to ci_gate_summary_line.txt")
    parser.add_argument("--result", required=True, help="path to ci_gate_result.detjson")
    parser.add_argument("--result-parse", required=True, help="path to ci_gate_result_parse.detjson")
    parser.add_argument("--badge", required=True, help="path to ci_gate_badge.detjson")
    parser.add_argument("--final-status-parse", required=True, help="path to ci_gate_final_status_line_parse.detjson")
    parser.add_argument("--require-pass", action="store_true", help="also require all statuses to be pass")
    args = parser.parse_args()

    summary_line_path = Path(args.summary_line)
    result_path = Path(args.result)
    result_parse_path = Path(args.result_parse)
    badge_path = Path(args.badge)
    final_status_parse_path = Path(args.final_status_parse)

    summary_line = load_text(summary_line_path)
    result_doc = load_json(result_path)
    result_parse_doc = load_json(result_parse_path)
    badge_doc = load_json(badge_path)
    final_status_parse_doc = load_json(final_status_parse_path)

    errors: list[str] = []
    if not summary_line:
        errors.append(f"missing summary_line: {summary_line_path}")
    if result_doc is None:
        errors.append(f"invalid result json: {result_path}")
    if result_parse_doc is None:
        errors.append(f"invalid result parse json: {result_parse_path}")
    if badge_doc is None:
        errors.append(f"invalid badge json: {badge_path}")
    if final_status_parse_doc is None:
        errors.append(f"invalid final status parse json: {final_status_parse_path}")
    if errors:
        for line in errors:
            print(line, file=sys.stderr)
        return 1

    compact = str(result_parse_doc.get("compact_line", "")).strip()
    if summary_line != compact:
        errors.append("summary_line != result_parse.compact_line")

    parsed_result = result_parse_doc.get("parsed")
    if not isinstance(parsed_result, dict):
        errors.append("result_parse.parsed missing")
        parsed_result = {}

    result_status = str(result_doc.get("status", "fail")).strip() or "fail"
    parse_status = str(parsed_result.get("status", "fail")).strip() or "fail"
    if result_status != parse_status:
        errors.append(f"result.status mismatch: result={result_status} parse={parse_status}")

    result_ok = bool(result_doc.get("ok", False))
    parse_ok = bool(parsed_result.get("ok", False))
    if result_ok != parse_ok:
        errors.append(f"result.ok mismatch: result={int(result_ok)} parse={int(parse_ok)}")
    result_age4_proof_ok = bool(result_doc.get(AGE4_PROOF_OK_KEY, False))
    parse_age4_proof_ok = bool(parsed_result.get(AGE4_PROOF_OK_KEY, False))
    if result_age4_proof_ok != parse_age4_proof_ok:
        errors.append(
            f"result.{AGE4_PROOF_OK_KEY} mismatch: result={int(result_age4_proof_ok)} parse={int(parse_age4_proof_ok)}"
        )
    result_age4_proof_failed = int(result_doc.get(AGE4_PROOF_FAILED_CRITERIA_KEY, -1))
    parse_age4_proof_failed = int(parsed_result.get(AGE4_PROOF_FAILED_CRITERIA_KEY, -1))
    if result_age4_proof_failed != parse_age4_proof_failed:
        errors.append(
            f"result.{AGE4_PROOF_FAILED_CRITERIA_KEY} mismatch: "
            f"result={result_age4_proof_failed} parse={parse_age4_proof_failed}"
        )
    result_age4_proof_preview = str(result_doc.get(AGE4_PROOF_FAILED_PREVIEW_KEY, "-")).strip() or "-"
    parse_age4_proof_preview = str(parsed_result.get(AGE4_PROOF_FAILED_PREVIEW_KEY, "-")).strip() or "-"
    if result_age4_proof_preview != parse_age4_proof_preview:
        errors.append(
            f"result.{AGE4_PROOF_FAILED_PREVIEW_KEY} mismatch: "
            f"result={result_age4_proof_preview} parse={parse_age4_proof_preview}"
        )
    for key in AGE5_W107_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_W107_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")
    for key in AGE5_POLICY_KEYS:
        result_value = str(result_doc.get(key, "")).strip()
        parse_value = str(parsed_result.get(key, "")).strip()
        if result_value != parse_value:
            errors.append(f"result.{key} mismatch: result={result_value} parse={parse_value}")

    badge_status = str(badge_doc.get("status", "fail")).strip() or "fail"
    badge_ok = bool(badge_doc.get("ok", False))
    if badge_status != result_status:
        errors.append(f"badge.status mismatch: badge={badge_status} result={result_status}")
    if badge_ok != result_ok:
        errors.append(f"badge.ok mismatch: badge={int(badge_ok)} result={int(result_ok)}")

    final_parsed = final_status_parse_doc.get("parsed")
    final_status = ""
    if isinstance(final_parsed, dict):
        final_status = str(final_parsed.get("status", "fail")).strip() or "fail"
        if final_status != result_status:
            errors.append(f"final.status mismatch: final={final_status} result={result_status}")
        final_age4_proof_ok = str(final_parsed.get(AGE4_PROOF_OK_KEY, "0")).strip() == "1"
        if final_age4_proof_ok != result_age4_proof_ok:
            errors.append(
                f"final.{AGE4_PROOF_OK_KEY} mismatch: final={int(final_age4_proof_ok)} result={int(result_age4_proof_ok)}"
            )
        try:
            final_age4_proof_failed = int(final_parsed.get(AGE4_PROOF_FAILED_CRITERIA_KEY, -1))
        except Exception:
            final_age4_proof_failed = -1
        if final_age4_proof_failed != result_age4_proof_failed:
            errors.append(
                f"final.{AGE4_PROOF_FAILED_CRITERIA_KEY} mismatch: "
                f"final={final_age4_proof_failed} result={result_age4_proof_failed}"
            )
        final_age4_proof_preview = str(final_parsed.get(AGE4_PROOF_FAILED_PREVIEW_KEY, "-")).strip() or "-"
        if final_age4_proof_preview != result_age4_proof_preview:
            errors.append(
                f"final.{AGE4_PROOF_FAILED_PREVIEW_KEY} mismatch: "
                f"final={final_age4_proof_preview} result={result_age4_proof_preview}"
            )
        for key in AGE5_W107_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_W107_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
        for key in AGE5_POLICY_KEYS:
            final_value = str(final_parsed.get(key, "")).strip()
            result_value = str(result_doc.get(key, "")).strip()
            if final_value != result_value:
                errors.append(f"final.{key} mismatch: final={final_value} result={result_value}")
    else:
        errors.append("final_status_parse.parsed missing")

    if args.require_pass:
        if result_status != "pass" or not result_ok or badge_status != "pass" or not badge_ok or final_status != "pass":
            errors.append("require-pass violated")

    if errors:
        print("ci gate outputs consistency check failed", file=sys.stderr)
        for line in errors[:16]:
            print(f" - {line}", file=sys.stderr)
        return 1

    print(
        "[ci-gate-outputs-consistency-check] ok "
        f"status={result_status} ok={int(result_ok)} "
        f"age4_proof_ok={int(result_age4_proof_ok)} age4_proof_failed={result_age4_proof_failed} "
        f"age4_proof_failed_preview={result_age4_proof_preview} "
        f"age5_w107_active={str(result_doc.get(AGE5_W107_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_w107_inactive={str(result_doc.get(AGE5_W107_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_w107_index_codes={str(result_doc.get(AGE5_W107_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_w107_last_completed_probe={str(result_doc.get(AGE5_W107_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_w107_progress={str(result_doc.get(AGE5_W107_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_w107_contract_completed={str(result_doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_w107_contract_total={str(result_doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_w107_contract_checks_text={str(result_doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_w107_contract_last_completed_probe={str(result_doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_w107_contract_progress={str(result_doc.get(AGE5_W107_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_age1_immediate_proof_operation_contract_completed={str(result_doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_age1_immediate_proof_operation_contract_total={str(result_doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_age1_immediate_proof_operation_contract_checks_text={str(result_doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_age1_immediate_proof_operation_contract_last_completed_probe={str(result_doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_age1_immediate_proof_operation_contract_progress={str(result_doc.get(AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_proof_certificate_v1_consumer_contract_completed={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_consumer_contract_total={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_consumer_contract_checks_text={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_consumer_contract_last_completed_probe={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_consumer_contract_progress={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_completed={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_total={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_checks_text={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_last_completed_probe={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_progress={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_proof_certificate_v1_family_contract_completed={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_family_contract_total={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_family_contract_checks_text={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_family_contract_last_completed_probe={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_proof_certificate_v1_family_contract_progress={str(result_doc.get(AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_proof_certificate_family_contract_completed={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_contract_total={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_contract_checks_text={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_contract_last_completed_probe={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_contract_progress={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_proof_certificate_family_transport_contract_completed={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_transport_contract_total={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_transport_contract_checks_text={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_transport_contract_last_completed_probe={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_proof_certificate_family_transport_contract_progress={str(result_doc.get(AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_proof_family_contract_completed={str(result_doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_proof_family_contract_total={str(result_doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_proof_family_contract_checks_text={str(result_doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_proof_family_contract_last_completed_probe={str(result_doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_proof_family_contract_progress={str(result_doc.get(AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_lang_surface_family_contract_completed={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_lang_surface_family_contract_total={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_lang_surface_family_contract_checks_text={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_lang_surface_family_contract_last_completed_probe={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_lang_surface_family_contract_progress={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_lang_runtime_family_contract_completed={str(result_doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_lang_runtime_family_contract_total={str(result_doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_lang_runtime_family_contract_checks_text={str(result_doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_lang_runtime_family_contract_last_completed_probe={str(result_doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_lang_runtime_family_contract_progress={str(result_doc.get(AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_gate0_family_contract_completed={str(result_doc.get(AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_gate0_family_contract_total={str(result_doc.get(AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_gate0_family_contract_checks_text={str(result_doc.get(AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_gate0_family_contract_last_completed_probe={str(result_doc.get(AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_gate0_family_contract_progress={str(result_doc.get(AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_gate0_family_transport_contract_completed={str(result_doc.get(AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_gate0_family_transport_contract_total={str(result_doc.get(AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_gate0_family_transport_contract_checks_text={str(result_doc.get(AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_gate0_family_transport_contract_last_completed_probe={str(result_doc.get(AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_gate0_family_transport_contract_progress={str(result_doc.get(AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_gate0_transport_family_contract_completed={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_gate0_transport_family_contract_total={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_gate0_transport_family_contract_checks_text={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_gate0_transport_family_contract_last_completed_probe={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_gate0_transport_family_contract_progress={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_gate0_transport_family_transport_contract_completed={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_gate0_transport_family_transport_contract_total={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_gate0_transport_family_transport_contract_checks_text={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_gate0_transport_family_transport_contract_current_probe={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'} "
        f"age5_gate0_transport_family_transport_contract_last_completed_probe={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_gate0_transport_family_transport_contract_progress={str(result_doc.get(AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_gate0_surface_family_transport_contract_completed={str(result_doc.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_gate0_surface_family_transport_contract_total={str(result_doc.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_gate0_surface_family_transport_contract_checks_text={str(result_doc.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_gate0_surface_family_transport_contract_current_probe={str(result_doc.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[3], '-')).strip() or '-'} "
        f"age5_gate0_surface_family_transport_contract_last_completed_probe={str(result_doc.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_gate0_surface_family_transport_contract_progress={str(result_doc.get(AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_lang_surface_family_transport_contract_completed={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_lang_surface_family_transport_contract_total={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_lang_surface_family_transport_contract_checks_text={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_lang_surface_family_transport_contract_last_completed_probe={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_lang_surface_family_transport_contract_progress={str(result_doc.get(AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_gate0_runtime_family_transport_contract_completed={str(result_doc.get(AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_gate0_runtime_family_transport_contract_total={str(result_doc.get(AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_gate0_runtime_family_transport_contract_checks_text={str(result_doc.get(AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_gate0_runtime_family_transport_contract_last_completed_probe={str(result_doc.get(AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_gate0_runtime_family_transport_contract_progress={str(result_doc.get(AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_bogae_alias_family_contract_completed={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_contract_total={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_contract_checks_text={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_contract_last_completed_probe={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_contract_progress={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"age5_bogae_alias_family_transport_contract_completed={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[0], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_transport_contract_total={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[1], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_transport_contract_checks_text={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[2], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_transport_contract_last_completed_probe={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[4], '-')).strip() or '-'} "
        f"age5_bogae_alias_family_transport_contract_progress={str(result_doc.get(AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS[5], '0')).strip() or '0'} "
        f"summary_line={summary_line}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
