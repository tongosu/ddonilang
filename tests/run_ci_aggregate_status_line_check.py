#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path


EXPECTED_SCHEMA = "ddn.ci.aggregate_gate_status_line.v1"
EXPECTED_KEYS = [
    "schema",
    "status",
    "overall_ok",
    "seamgrim_failed_steps",
    "age3_failed_criteria",
    "age4_failed_criteria",
    "age4_proof_ok",
    "age4_proof_failed_criteria",
    "age5_failed_criteria",
    "age5_combined_heavy_full_real_status",
    "age5_full_real_source_check",
    "age5_full_real_source_selftest",
    "age5_full_real_w107_golden_index_selftest_active_cases",
    "age5_full_real_w107_golden_index_selftest_inactive_cases",
    "age5_full_real_w107_golden_index_selftest_index_codes",
    "age5_full_real_w107_golden_index_selftest_current_probe",
    "age5_full_real_w107_golden_index_selftest_last_completed_probe",
    "age5_full_real_w107_golden_index_selftest_progress_present",
    "age5_full_real_w107_progress_contract_selftest_completed_checks",
    "age5_full_real_w107_progress_contract_selftest_total_checks",
    "age5_full_real_w107_progress_contract_selftest_checks_text",
    "age5_full_real_w107_progress_contract_selftest_current_probe",
    "age5_full_real_w107_progress_contract_selftest_last_completed_probe",
    "age5_full_real_w107_progress_contract_selftest_progress_present",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe",
    "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe",
    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe",
    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe",
    "age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present",
    "age5_full_real_proof_certificate_family_contract_selftest_completed_checks",
    "age5_full_real_proof_certificate_family_contract_selftest_total_checks",
    "age5_full_real_proof_certificate_family_contract_selftest_checks_text",
    "age5_full_real_proof_certificate_family_contract_selftest_current_probe",
    "age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe",
    "age5_full_real_proof_certificate_family_contract_selftest_progress_present",
    "age5_full_real_proof_family_contract_selftest_completed_checks",
    "age5_full_real_proof_family_contract_selftest_total_checks",
    "age5_full_real_proof_family_contract_selftest_checks_text",
    "age5_full_real_proof_family_contract_selftest_current_probe",
    "age5_full_real_proof_family_contract_selftest_last_completed_probe",
    "age5_full_real_proof_family_contract_selftest_progress_present",
    "age5_full_real_proof_family_transport_contract_selftest_completed_checks",
    "age5_full_real_proof_family_transport_contract_selftest_total_checks",
    "age5_full_real_proof_family_transport_contract_selftest_checks_text",
    "age5_full_real_proof_family_transport_contract_selftest_current_probe",
    "age5_full_real_proof_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_proof_family_transport_contract_selftest_progress_present",
    "age5_full_real_lang_surface_family_contract_selftest_completed_checks",
    "age5_full_real_lang_surface_family_contract_selftest_total_checks",
    "age5_full_real_lang_surface_family_contract_selftest_checks_text",
    "age5_full_real_lang_surface_family_contract_selftest_current_probe",
    "age5_full_real_lang_surface_family_contract_selftest_last_completed_probe",
    "age5_full_real_lang_surface_family_contract_selftest_progress_present",
    "age5_full_real_lang_runtime_family_contract_selftest_completed_checks",
    "age5_full_real_lang_runtime_family_contract_selftest_total_checks",
    "age5_full_real_lang_runtime_family_contract_selftest_checks_text",
    "age5_full_real_lang_runtime_family_contract_selftest_current_probe",
    "age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe",
    "age5_full_real_lang_runtime_family_contract_selftest_progress_present",
    "age5_full_real_gate0_family_contract_selftest_completed_checks",
    "age5_full_real_gate0_family_contract_selftest_total_checks",
    "age5_full_real_gate0_family_contract_selftest_checks_text",
    "age5_full_real_gate0_family_contract_selftest_current_probe",
    "age5_full_real_gate0_family_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_family_contract_selftest_progress_present",
    "age5_full_real_gate0_surface_family_contract_selftest_completed_checks",
    "age5_full_real_gate0_surface_family_contract_selftest_total_checks",
    "age5_full_real_gate0_surface_family_contract_selftest_checks_text",
    "age5_full_real_gate0_surface_family_contract_selftest_current_probe",
    "age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_surface_family_contract_selftest_progress_present",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present",
    "age5_full_real_gate0_family_transport_contract_selftest_completed_checks",
    "age5_full_real_gate0_family_transport_contract_selftest_total_checks",
    "age5_full_real_gate0_family_transport_contract_selftest_checks_text",
    "age5_full_real_gate0_family_transport_contract_selftest_current_probe",
    "age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_family_transport_contract_selftest_progress_present",
    "age5_full_real_gate0_transport_family_contract_selftest_completed_checks",
    "age5_full_real_gate0_transport_family_contract_selftest_total_checks",
    "age5_full_real_gate0_transport_family_contract_selftest_checks_text",
    "age5_full_real_gate0_transport_family_contract_selftest_current_probe",
    "age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_transport_family_contract_selftest_progress_present",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present",
    "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks",
    "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks",
    "age5_full_real_lang_surface_family_transport_contract_selftest_checks_text",
    "age5_full_real_lang_surface_family_transport_contract_selftest_current_probe",
    "age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present",
    "age5_full_real_bogae_alias_family_contract_selftest_completed_checks",
    "age5_full_real_bogae_alias_family_contract_selftest_total_checks",
    "age5_full_real_bogae_alias_family_contract_selftest_checks_text",
    "age5_full_real_bogae_alias_family_contract_selftest_current_probe",
    "age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe",
    "age5_full_real_bogae_alias_family_contract_selftest_progress_present",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present",
    "age5_combined_heavy_runtime_helper_negative_status",
    "age5_combined_heavy_group_id_summary_negative_status",
    "ci_sanity_age5_combined_heavy_child_summary_default_fields",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields",
    "oi_failed_packs",
    "report_path",
    "generated_at_utc",
    "reason",
]
TOKEN_RE = re.compile(r'([A-Za-z0-9_]+)=("([^"\\]|\\.)*"|[^ \t]+)')
SUMMARY_STATUS_VALUES = {"pass", "fail", "skipped"}


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def parse_tokens(line: str) -> dict[str, str] | None:
    text = line.strip()
    if not text:
        return None
    out: dict[str, str] = {}
    pos = 0
    for match in TOKEN_RE.finditer(text):
        if text[pos : match.start()].strip():
            return None
        key = match.group(1)
        raw = match.group(2)
        if raw.startswith('"'):
            try:
                value = json.loads(raw)
            except Exception:
                return None
        else:
            value = raw
        out[key] = str(value)
        pos = match.end()
    if text[pos:].strip():
        return None
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate ci_aggregate_status_line.txt format")
    parser.add_argument("--status-line", required=True, help="path to ci_aggregate_status_line.txt")
    parser.add_argument("--aggregate-report", required=True, help="path to ci_aggregate_report.detjson")
    parser.add_argument("--require-pass", action="store_true", help="also require overall_ok=true")
    args = parser.parse_args()

    status_line_path = Path(args.status_line)
    aggregate_report_path = Path(args.aggregate_report)
    if not status_line_path.exists():
        print(f"missing status-line: {status_line_path}", file=sys.stderr)
        return 1
    parsed = parse_tokens(status_line_path.read_text(encoding="utf-8"))
    if parsed is None:
        print(f"invalid status-line format: {status_line_path}", file=sys.stderr)
        return 1
    if list(parsed.keys()) != EXPECTED_KEYS:
        print("status-line key order mismatch", file=sys.stderr)
        return 1
    if parsed.get("schema") != EXPECTED_SCHEMA:
        print("status-line schema mismatch", file=sys.stderr)
        return 1
    if parsed.get("status") not in {"pass", "fail"}:
        print("status-line status invalid", file=sys.stderr)
        return 1
    if parsed.get("overall_ok") not in {"0", "1"}:
        print("status-line overall_ok invalid", file=sys.stderr)
        return 1

    if parsed.get("age4_proof_ok") not in {"0", "1"}:
        print("status-line age4_proof_ok invalid", file=sys.stderr)
        return 1
    for key in ("age5_full_real_source_check", "age5_full_real_source_selftest"):
        if parsed.get(key) not in {"0", "1"}:
            print(f"status-line {key} invalid", file=sys.stderr)
            return 1
    if parsed.get("age5_full_real_w107_golden_index_selftest_progress_present") not in {"0", "1"}:
        print("status-line age5_full_real_w107_golden_index_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if parsed.get("age5_full_real_w107_progress_contract_selftest_progress_present") not in {"0", "1"}:
        print("status-line age5_full_real_w107_progress_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if parsed.get("age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed.get("age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed.get("age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed.get("age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed.get("age5_full_real_proof_certificate_family_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_proof_certificate_family_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed.get("age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed.get("age5_full_real_proof_family_contract_selftest_progress_present") not in {"0", "1"}:
        print("status-line age5_full_real_proof_family_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if parsed.get("age5_full_real_lang_surface_family_contract_selftest_progress_present") not in {"0", "1"}:
        print("status-line age5_full_real_lang_surface_family_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if parsed.get("age5_full_real_lang_runtime_family_contract_selftest_progress_present") not in {"0", "1"}:
        print("status-line age5_full_real_lang_runtime_family_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if parsed.get("age5_full_real_gate0_family_contract_selftest_progress_present") not in {"0", "1"}:
        print("status-line age5_full_real_gate0_family_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if parsed.get("age5_full_real_gate0_surface_family_contract_selftest_progress_present") not in {"0", "1"}:
        print("status-line age5_full_real_gate0_surface_family_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if parsed.get("age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed.get("age5_full_real_gate0_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_gate0_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed.get("age5_full_real_gate0_transport_family_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_gate0_transport_family_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed.get("age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed.get("age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed.get("age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed.get("age5_full_real_lang_surface_family_transport_contract_selftest_progress_present") not in {"0", "1"}:
        print("status-line age5_full_real_lang_surface_family_transport_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if parsed.get("age5_full_real_proof_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_proof_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed.get("age5_full_real_bogae_alias_family_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_bogae_alias_family_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed.get("age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    for key in (
        "seamgrim_failed_steps",
        "age3_failed_criteria",
        "age4_failed_criteria",
        "age4_proof_failed_criteria",
        "age5_failed_criteria",
        "oi_failed_packs",
        "age5_full_real_w107_golden_index_selftest_active_cases",
        "age5_full_real_w107_golden_index_selftest_inactive_cases",
        "age5_full_real_w107_golden_index_selftest_index_codes",
        "age5_full_real_w107_progress_contract_selftest_completed_checks",
        "age5_full_real_w107_progress_contract_selftest_total_checks",
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks",
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks",
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks",
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks",
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks",
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks",
        "age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks",
        "age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks",
        "age5_full_real_proof_certificate_family_contract_selftest_completed_checks",
        "age5_full_real_proof_certificate_family_contract_selftest_total_checks",
        "age5_full_real_proof_family_transport_contract_selftest_completed_checks",
        "age5_full_real_proof_family_transport_contract_selftest_total_checks",
        "age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks",
        "age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks",
        "age5_full_real_proof_family_contract_selftest_completed_checks",
        "age5_full_real_proof_family_contract_selftest_total_checks",
        "age5_full_real_lang_surface_family_contract_selftest_completed_checks",
        "age5_full_real_lang_surface_family_contract_selftest_total_checks",
        "age5_full_real_lang_runtime_family_contract_selftest_completed_checks",
        "age5_full_real_lang_runtime_family_contract_selftest_total_checks",
        "age5_full_real_gate0_family_contract_selftest_completed_checks",
        "age5_full_real_gate0_family_contract_selftest_total_checks",
        "age5_full_real_gate0_surface_family_contract_selftest_completed_checks",
        "age5_full_real_gate0_surface_family_contract_selftest_total_checks",
        "age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks",
        "age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks",
        "age5_full_real_gate0_family_transport_contract_selftest_completed_checks",
        "age5_full_real_gate0_family_transport_contract_selftest_total_checks",
        "age5_full_real_gate0_transport_family_contract_selftest_completed_checks",
        "age5_full_real_gate0_transport_family_contract_selftest_total_checks",
        "age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks",
        "age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks",
        "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks",
        "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks",
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks",
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks",
        "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks",
        "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks",
        "age5_full_real_bogae_alias_family_contract_selftest_completed_checks",
        "age5_full_real_bogae_alias_family_contract_selftest_total_checks",
        "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks",
        "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks",
    ):
        text = str(parsed.get(key, "0")).strip()
        if text == "-":
            continue
        try:
            int(text)
        except ValueError:
            print(f"status-line {key} invalid int", file=sys.stderr)
            return 1
    for key in (
        "age5_combined_heavy_full_real_status",
        "age5_combined_heavy_runtime_helper_negative_status",
        "age5_combined_heavy_group_id_summary_negative_status",
    ):
        if parsed.get(key) not in SUMMARY_STATUS_VALUES:
            print(f"status-line {key} invalid status", file=sys.stderr)
            return 1

    report = load_json(aggregate_report_path)
    if report is None:
        print(f"invalid aggregate report: {aggregate_report_path}", file=sys.stderr)
        return 1

    overall_ok = bool(report.get("overall_ok", False))
    expected_status = "pass" if overall_ok else "fail"
    if parsed.get("status") != expected_status:
        print("status mismatch with aggregate report", file=sys.stderr)
        return 1
    if parsed.get("overall_ok") != ("1" if overall_ok else "0"):
        print("overall_ok mismatch with aggregate report", file=sys.stderr)
        return 1

    seamgrim = report.get("seamgrim") if isinstance(report.get("seamgrim"), dict) else {}
    age3 = report.get("age3") if isinstance(report.get("age3"), dict) else {}
    age4 = report.get("age4") if isinstance(report.get("age4"), dict) else {}
    age5 = report.get("age5") if isinstance(report.get("age5"), dict) else {}
    oi = report.get("oi405_406") if isinstance(report.get("oi405_406"), dict) else {}
    expected_counts = {
        "seamgrim_failed_steps": len(seamgrim.get("failed_steps", [])) if isinstance(seamgrim.get("failed_steps"), list) else 0,
        "age3_failed_criteria": len(age3.get("failed_criteria", [])) if isinstance(age3.get("failed_criteria"), list) else 0,
        "age4_failed_criteria": len(age4.get("failed_criteria", [])) if isinstance(age4.get("failed_criteria"), list) else 0,
        "age4_proof_failed_criteria": len(age4.get("proof_artifact_failed_criteria", []))
        if isinstance(age4.get("proof_artifact_failed_criteria"), list)
        else 0,
        "age5_failed_criteria": len(age5.get("failed_criteria", [])) if isinstance(age5.get("failed_criteria"), list) else 0,
        "oi_failed_packs": len(oi.get("failed_packs", [])) if isinstance(oi.get("failed_packs"), list) else 0,
    }
    for key, expected in expected_counts.items():
        if int(parsed.get(key, "-1")) != expected:
            print(f"{key} mismatch: line={parsed.get(key)} report={expected}", file=sys.stderr)
            return 1
    expected_age4_proof_ok = "1" if bool(age4.get("proof_artifact_ok", False)) else "0"
    if str(parsed.get("age4_proof_ok", "")).strip() != expected_age4_proof_ok:
        print(f"age4_proof_ok mismatch: line={parsed.get('age4_proof_ok')} report={expected_age4_proof_ok}", file=sys.stderr)
        return 1
    expected_age5_child_statuses = {
        "age5_combined_heavy_full_real_status": str(age5.get("age5_combined_heavy_full_real_status", "skipped")).strip() or "skipped",
        "age5_combined_heavy_runtime_helper_negative_status": str(
            age5.get("age5_combined_heavy_runtime_helper_negative_status", "skipped")
        ).strip()
        or "skipped",
        "age5_combined_heavy_group_id_summary_negative_status": str(
            age5.get("age5_combined_heavy_group_id_summary_negative_status", "skipped")
        ).strip()
        or "skipped",
    }
    for key, expected in expected_age5_child_statuses.items():
        if expected not in SUMMARY_STATUS_VALUES:
            expected = "skipped"
        if str(parsed.get(key, "")).strip() != expected:
            print(f"{key} mismatch: line={parsed.get(key)} report={expected}", file=sys.stderr)
            return 1
    age5_full_real_source_trace = (
        age5.get("full_real_source_trace") if isinstance(age5.get("full_real_source_trace"), dict) else {}
    )
    expected_age5_source_flags = {
        "age5_full_real_source_check": (
            str(age5_full_real_source_trace.get("smoke_check_script_exists", "0")).strip() or "0"
        ),
        "age5_full_real_source_selftest": (
            str(age5_full_real_source_trace.get("smoke_check_selftest_script_exists", "0")).strip() or "0"
        ),
    }
    for key, expected in expected_age5_source_flags.items():
        if str(parsed.get(key, "")).strip() != expected:
            print(f"{key} mismatch: line={parsed.get(key)} report={expected}", file=sys.stderr)
            return 1
    expected_age5_w107_progress = {
        "age5_full_real_w107_golden_index_selftest_active_cases": (
            str(age5.get("age5_full_real_w107_golden_index_selftest_active_cases", "-")).strip() or "-"
        ),
        "age5_full_real_w107_golden_index_selftest_inactive_cases": (
            str(age5.get("age5_full_real_w107_golden_index_selftest_inactive_cases", "-")).strip() or "-"
        ),
        "age5_full_real_w107_golden_index_selftest_index_codes": (
            str(age5.get("age5_full_real_w107_golden_index_selftest_index_codes", "-")).strip() or "-"
        ),
        "age5_full_real_w107_golden_index_selftest_current_probe": (
            str(age5.get("age5_full_real_w107_golden_index_selftest_current_probe", "-")).strip() or "-"
        ),
        "age5_full_real_w107_golden_index_selftest_last_completed_probe": (
            str(age5.get("age5_full_real_w107_golden_index_selftest_last_completed_probe", "-")).strip() or "-"
        ),
        "age5_full_real_w107_golden_index_selftest_progress_present": (
            str(age5.get("age5_full_real_w107_golden_index_selftest_progress_present", "0")).strip() or "0"
        ),
        "age5_full_real_w107_progress_contract_selftest_completed_checks": (
            str(age5.get("age5_full_real_w107_progress_contract_selftest_completed_checks", "-")).strip() or "-"
        ),
        "age5_full_real_w107_progress_contract_selftest_total_checks": (
            str(age5.get("age5_full_real_w107_progress_contract_selftest_total_checks", "-")).strip() or "-"
        ),
        "age5_full_real_w107_progress_contract_selftest_checks_text": (
            str(age5.get("age5_full_real_w107_progress_contract_selftest_checks_text", "-")).strip() or "-"
        ),
        "age5_full_real_w107_progress_contract_selftest_current_probe": (
            str(age5.get("age5_full_real_w107_progress_contract_selftest_current_probe", "-")).strip() or "-"
        ),
        "age5_full_real_w107_progress_contract_selftest_last_completed_probe": (
            str(age5.get("age5_full_real_w107_progress_contract_selftest_last_completed_probe", "-")).strip() or "-"
        ),
        "age5_full_real_w107_progress_contract_selftest_progress_present": (
            str(age5.get("age5_full_real_w107_progress_contract_selftest_progress_present", "0")).strip() or "0"
        ),
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks": (
            str(
                age5.get(
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks": (
            str(
                age5.get(
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text": (
            str(
                age5.get(
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe": (
            str(
                age5.get(
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe": (
            str(
                age5.get(
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present": (
            str(
                age5.get(
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks": (
            str(
                age5.get(
                    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks": (
            str(
                age5.get(
                    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text": (
            str(
                age5.get(
                    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe": (
            str(
                age5.get(
                    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe": (
            str(
                age5.get(
                    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present": (
            str(
                age5.get(
                    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks": (
            str(
                age5.get(
                    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks": (
            str(
                age5.get(
                    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text": (
            str(
                age5.get(
                    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe": (
            str(
                age5.get(
                    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe": (
            str(
                age5.get(
                    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present": (
            str(
                age5.get(
                    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_lang_surface_family_contract_selftest_completed_checks": (
            str(
                age5.get(
                    "age5_full_real_lang_surface_family_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_surface_family_contract_selftest_total_checks": (
            str(
                age5.get(
                    "age5_full_real_lang_surface_family_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_surface_family_contract_selftest_checks_text": (
            str(
                age5.get(
                    "age5_full_real_lang_surface_family_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_surface_family_contract_selftest_current_probe": (
            str(
                age5.get(
                    "age5_full_real_lang_surface_family_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_surface_family_contract_selftest_last_completed_probe": (
            str(
                age5.get(
                    "age5_full_real_lang_surface_family_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_surface_family_contract_selftest_progress_present": (
            str(
                age5.get(
                    "age5_full_real_lang_surface_family_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_lang_runtime_family_contract_selftest_completed_checks": (
            str(age5.get("age5_full_real_lang_runtime_family_contract_selftest_completed_checks", "-")).strip()
            or "-"
        ),
        "age5_full_real_lang_runtime_family_contract_selftest_total_checks": (
            str(age5.get("age5_full_real_lang_runtime_family_contract_selftest_total_checks", "-")).strip()
            or "-"
        ),
        "age5_full_real_lang_runtime_family_contract_selftest_checks_text": (
            str(age5.get("age5_full_real_lang_runtime_family_contract_selftest_checks_text", "-")).strip()
            or "-"
        ),
        "age5_full_real_lang_runtime_family_contract_selftest_current_probe": (
            str(age5.get("age5_full_real_lang_runtime_family_contract_selftest_current_probe", "-")).strip()
            or "-"
        ),
        "age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe": (
            str(age5.get("age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe", "-")).strip()
            or "-"
        ),
        "age5_full_real_lang_runtime_family_contract_selftest_progress_present": (
            str(age5.get("age5_full_real_lang_runtime_family_contract_selftest_progress_present", "0")).strip()
            or "0"
        ),
        "age5_full_real_gate0_family_contract_selftest_completed_checks": (
            str(age5.get("age5_full_real_gate0_family_contract_selftest_completed_checks", "-")).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_contract_selftest_total_checks": (
            str(age5.get("age5_full_real_gate0_family_contract_selftest_total_checks", "-")).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_contract_selftest_checks_text": (
            str(age5.get("age5_full_real_gate0_family_contract_selftest_checks_text", "-")).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_contract_selftest_current_probe": (
            str(age5.get("age5_full_real_gate0_family_contract_selftest_current_probe", "-")).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_contract_selftest_last_completed_probe": (
            str(age5.get("age5_full_real_gate0_family_contract_selftest_last_completed_probe", "-")).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_contract_selftest_progress_present": (
            str(age5.get("age5_full_real_gate0_family_contract_selftest_progress_present", "0")).strip()
            or "0"
        ),
        "age5_full_real_gate0_surface_family_contract_selftest_completed_checks": (
            str(age5.get("age5_full_real_gate0_surface_family_contract_selftest_completed_checks", "-")).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_contract_selftest_total_checks": (
            str(age5.get("age5_full_real_gate0_surface_family_contract_selftest_total_checks", "-")).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_contract_selftest_checks_text": (
            str(age5.get("age5_full_real_gate0_surface_family_contract_selftest_checks_text", "-")).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_contract_selftest_current_probe": (
            str(age5.get("age5_full_real_gate0_surface_family_contract_selftest_current_probe", "-")).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe": (
            str(age5.get("age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe", "-")).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_contract_selftest_progress_present": (
            str(age5.get("age5_full_real_gate0_surface_family_contract_selftest_progress_present", "0")).strip()
            or "0"
        ),
        "age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks": (
            str(age5.get("age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks", "-")).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks": (
            str(age5.get("age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks", "-")).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text": (
            str(age5.get("age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text", "-")).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe": (
            str(age5.get("age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe", "-")).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe": (
            str(age5.get("age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe", "-")).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present": (
            str(age5.get("age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present", "0")).strip()
            or "0"
        ),
        "age5_full_real_gate0_family_transport_contract_selftest_completed_checks": (
            str(
                age5.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_transport_contract_selftest_total_checks": (
            str(
                age5.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_transport_contract_selftest_checks_text": (
            str(
                age5.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_transport_contract_selftest_current_probe": (
            str(
                age5.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe": (
            str(
                age5.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_transport_contract_selftest_progress_present": (
            str(
                age5.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_gate0_transport_family_contract_selftest_completed_checks": (
            str(
                age5.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_contract_selftest_total_checks": (
            str(
                age5.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_contract_selftest_checks_text": (
            str(
                age5.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_contract_selftest_current_probe": (
            str(
                age5.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe": (
            str(
                age5.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_contract_selftest_progress_present": (
            str(
                age5.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks": (
            str(
                age5.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks": (
            str(
                age5.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text": (
            str(
                age5.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe": (
            str(
                age5.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe": (
            str(
                age5.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present": (
            str(
                age5.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks": (
            str(
                age5.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks": (
            str(
                age5.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text": (
            str(
                age5.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe": (
            str(
                age5.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe": (
            str(
                age5.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present": (
            str(
                age5.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks": (
            str(
                age5.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks": (
            str(
                age5.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text": (
            str(
                age5.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe": (
            str(
                age5.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe": (
            str(
                age5.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present": (
            str(
                age5.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks": (
            str(
                age5.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks": (
            str(
                age5.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_surface_family_transport_contract_selftest_checks_text": (
            str(
                age5.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_surface_family_transport_contract_selftest_current_probe": (
            str(
                age5.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe": (
            str(
                age5.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present": (
            str(
                age5.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks": (
            str(
                age5.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks": (
            str(
                age5.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text": (
            str(
                age5.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe": (
            str(
                age5.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe": (
            str(
                age5.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present": (
            str(
                age5.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
    }
    for key, expected in expected_age5_w107_progress.items():
        if str(parsed.get(key, "")).strip() != expected:
            print(f"{key} mismatch: line={parsed.get(key)} report={expected}", file=sys.stderr)
            return 1
    expected_age5_child_default_texts = {
        "ci_sanity_age5_combined_heavy_child_summary_default_fields": str(
            age5.get("ci_sanity_age5_combined_heavy_child_summary_default_fields", "")
        ).strip(),
        "ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields": str(
            age5.get("ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields", "")
        ).strip(),
    }
    for key, expected in expected_age5_child_default_texts.items():
        if str(parsed.get(key, "")).strip() != expected:
            print(f"{key} mismatch: line={parsed.get(key)} report={expected}", file=sys.stderr)
            return 1

    if str(parsed.get("report_path", "")).strip() != str(aggregate_report_path):
        print("report_path mismatch", file=sys.stderr)
        return 1
    if args.require_pass and not overall_ok:
        print("aggregate overall_ok=false", file=sys.stderr)
        return 1

    print(
        "[ci-aggregate-status-line-check] ok "
        f"status={parsed.get('status')} overall_ok={parsed.get('overall_ok')} "
        f"seamgrim_failed={parsed.get('seamgrim_failed_steps')} "
        f"age3_failed={parsed.get('age3_failed_criteria')} "
        f"age4_failed={parsed.get('age4_failed_criteria')} "
        f"age4_proof_ok={parsed.get('age4_proof_ok')} "
        f"age4_proof_failed={parsed.get('age4_proof_failed_criteria')} "
        f"age5_failed={parsed.get('age5_failed_criteria')} "
        f"age5_full_real={parsed.get('age5_combined_heavy_full_real_status')} "
        f"age5_full_real_source_check={parsed.get('age5_full_real_source_check')} "
        f"age5_full_real_source_selftest={parsed.get('age5_full_real_source_selftest')} "
        f"age5_w107_active={parsed.get('age5_full_real_w107_golden_index_selftest_active_cases')} "
        f"age5_w107_inactive={parsed.get('age5_full_real_w107_golden_index_selftest_inactive_cases')} "
        f"age5_w107_index_codes={parsed.get('age5_full_real_w107_golden_index_selftest_index_codes')} "
        f"age5_w107_current_probe={parsed.get('age5_full_real_w107_golden_index_selftest_current_probe')} "
        f"age5_w107_last_completed_probe={parsed.get('age5_full_real_w107_golden_index_selftest_last_completed_probe')} "
        f"age5_w107_progress={parsed.get('age5_full_real_w107_golden_index_selftest_progress_present')} "
        f"age5_w107_contract_completed={parsed.get('age5_full_real_w107_progress_contract_selftest_completed_checks')} "
        f"age5_w107_contract_total={parsed.get('age5_full_real_w107_progress_contract_selftest_total_checks')} "
        f"age5_w107_contract_checks_text={parsed.get('age5_full_real_w107_progress_contract_selftest_checks_text')} "
        f"age5_w107_contract_current_probe={parsed.get('age5_full_real_w107_progress_contract_selftest_current_probe')} "
        f"age5_w107_contract_last_completed_probe={parsed.get('age5_full_real_w107_progress_contract_selftest_last_completed_probe')} "
        f"age5_w107_contract_progress={parsed.get('age5_full_real_w107_progress_contract_selftest_progress_present')} "
        f"age5_age1_immediate_proof_operation_contract_completed={parsed.get('age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks')} "
        f"age5_age1_immediate_proof_operation_contract_total={parsed.get('age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks')} "
        f"age5_age1_immediate_proof_operation_contract_checks_text={parsed.get('age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text')} "
        f"age5_age1_immediate_proof_operation_contract_current_probe={parsed.get('age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe')} "
        f"age5_age1_immediate_proof_operation_contract_last_completed_probe={parsed.get('age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe')} "
        f"age5_age1_immediate_proof_operation_contract_progress={parsed.get('age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present')} "
        f"age5_proof_certificate_v1_consumer_contract_completed={parsed.get('age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks')} "
        f"age5_proof_certificate_v1_consumer_contract_total={parsed.get('age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks')} "
        f"age5_proof_certificate_v1_consumer_contract_checks_text={parsed.get('age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text')} "
        f"age5_proof_certificate_v1_consumer_contract_current_probe={parsed.get('age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe')} "
        f"age5_proof_certificate_v1_consumer_contract_last_completed_probe={parsed.get('age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe')} "
        f"age5_proof_certificate_v1_consumer_contract_progress={parsed.get('age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present')} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_completed={parsed.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks')} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_total={parsed.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks')} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_checks_text={parsed.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text')} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_current_probe={parsed.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe')} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_last_completed_probe={parsed.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe')} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_progress={parsed.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present')} "
        f"age5_proof_certificate_family_contract_completed={parsed.get('age5_full_real_proof_certificate_family_contract_selftest_completed_checks')} "
        f"age5_proof_certificate_family_contract_total={parsed.get('age5_full_real_proof_certificate_family_contract_selftest_total_checks')} "
        f"age5_proof_certificate_family_contract_checks_text={parsed.get('age5_full_real_proof_certificate_family_contract_selftest_checks_text')} "
        f"age5_proof_certificate_family_contract_current_probe={parsed.get('age5_full_real_proof_certificate_family_contract_selftest_current_probe')} "
        f"age5_proof_certificate_family_contract_last_completed_probe={parsed.get('age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe')} "
        f"age5_proof_certificate_family_contract_progress={parsed.get('age5_full_real_proof_certificate_family_contract_selftest_progress_present')} "
        f"age5_proof_certificate_family_transport_contract_completed={parsed.get('age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks')} "
        f"age5_proof_certificate_family_transport_contract_total={parsed.get('age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks')} "
        f"age5_proof_certificate_family_transport_contract_checks_text={parsed.get('age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text')} "
        f"age5_proof_certificate_family_transport_contract_current_probe={parsed.get('age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe')} "
        f"age5_proof_certificate_family_transport_contract_last_completed_probe={parsed.get('age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe')} "
        f"age5_proof_certificate_family_transport_contract_progress={parsed.get('age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present')} "
        f"age5_lang_surface_family_contract_completed={parsed.get('age5_full_real_lang_surface_family_contract_selftest_completed_checks')} "
        f"age5_lang_surface_family_contract_total={parsed.get('age5_full_real_lang_surface_family_contract_selftest_total_checks')} "
        f"age5_lang_surface_family_contract_checks_text={parsed.get('age5_full_real_lang_surface_family_contract_selftest_checks_text')} "
        f"age5_lang_surface_family_contract_current_probe={parsed.get('age5_full_real_lang_surface_family_contract_selftest_current_probe')} "
        f"age5_lang_surface_family_contract_last_completed_probe={parsed.get('age5_full_real_lang_surface_family_contract_selftest_last_completed_probe')} "
        f"age5_lang_surface_family_contract_progress={parsed.get('age5_full_real_lang_surface_family_contract_selftest_progress_present')} "
        f"age5_lang_runtime_family_contract_completed={parsed.get('age5_full_real_lang_runtime_family_contract_selftest_completed_checks')} "
        f"age5_lang_runtime_family_contract_total={parsed.get('age5_full_real_lang_runtime_family_contract_selftest_total_checks')} "
        f"age5_lang_runtime_family_contract_checks_text={parsed.get('age5_full_real_lang_runtime_family_contract_selftest_checks_text')} "
        f"age5_lang_runtime_family_contract_current_probe={parsed.get('age5_full_real_lang_runtime_family_contract_selftest_current_probe')} "
        f"age5_lang_runtime_family_contract_last_completed_probe={parsed.get('age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe')} "
        f"age5_lang_runtime_family_contract_progress={parsed.get('age5_full_real_lang_runtime_family_contract_selftest_progress_present')} "
        f"age5_gate0_family_contract_completed={parsed.get('age5_full_real_gate0_family_contract_selftest_completed_checks')} "
        f"age5_gate0_family_contract_total={parsed.get('age5_full_real_gate0_family_contract_selftest_total_checks')} "
        f"age5_gate0_family_contract_checks_text={parsed.get('age5_full_real_gate0_family_contract_selftest_checks_text')} "
        f"age5_gate0_family_contract_current_probe={parsed.get('age5_full_real_gate0_family_contract_selftest_current_probe')} "
        f"age5_gate0_family_contract_last_completed_probe={parsed.get('age5_full_real_gate0_family_contract_selftest_last_completed_probe')} "
        f"age5_gate0_family_contract_progress={parsed.get('age5_full_real_gate0_family_contract_selftest_progress_present')} "
        f"age5_gate0_surface_family_contract_completed={parsed.get('age5_full_real_gate0_surface_family_contract_selftest_completed_checks')} "
        f"age5_gate0_surface_family_contract_total={parsed.get('age5_full_real_gate0_surface_family_contract_selftest_total_checks')} "
        f"age5_gate0_surface_family_contract_checks_text={parsed.get('age5_full_real_gate0_surface_family_contract_selftest_checks_text')} "
        f"age5_gate0_surface_family_contract_current_probe={parsed.get('age5_full_real_gate0_surface_family_contract_selftest_current_probe')} "
        f"age5_gate0_surface_family_contract_last_completed_probe={parsed.get('age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe')} "
        f"age5_gate0_surface_family_contract_progress={parsed.get('age5_full_real_gate0_surface_family_contract_selftest_progress_present')} "
        f"age5_gate0_surface_family_transport_contract_completed={parsed.get('age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks')} "
        f"age5_gate0_surface_family_transport_contract_total={parsed.get('age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks')} "
        f"age5_gate0_surface_family_transport_contract_checks_text={parsed.get('age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text')} "
        f"age5_gate0_surface_family_transport_contract_current_probe={parsed.get('age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe')} "
        f"age5_gate0_surface_family_transport_contract_last_completed_probe={parsed.get('age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe')} "
        f"age5_gate0_surface_family_transport_contract_progress={parsed.get('age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present')} "
        f"age5_gate0_family_transport_contract_completed={parsed.get('age5_full_real_gate0_family_transport_contract_selftest_completed_checks')} "
        f"age5_gate0_family_transport_contract_total={parsed.get('age5_full_real_gate0_family_transport_contract_selftest_total_checks')} "
        f"age5_gate0_family_transport_contract_checks_text={parsed.get('age5_full_real_gate0_family_transport_contract_selftest_checks_text')} "
        f"age5_gate0_family_transport_contract_current_probe={parsed.get('age5_full_real_gate0_family_transport_contract_selftest_current_probe')} "
        f"age5_gate0_family_transport_contract_last_completed_probe={parsed.get('age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe')} "
        f"age5_gate0_family_transport_contract_progress={parsed.get('age5_full_real_gate0_family_transport_contract_selftest_progress_present')} "
        f"age5_gate0_transport_family_contract_completed={parsed.get('age5_full_real_gate0_transport_family_contract_selftest_completed_checks')} "
        f"age5_gate0_transport_family_contract_total={parsed.get('age5_full_real_gate0_transport_family_contract_selftest_total_checks')} "
        f"age5_gate0_transport_family_contract_checks_text={parsed.get('age5_full_real_gate0_transport_family_contract_selftest_checks_text')} "
        f"age5_gate0_transport_family_contract_current_probe={parsed.get('age5_full_real_gate0_transport_family_contract_selftest_current_probe')} "
        f"age5_gate0_transport_family_contract_last_completed_probe={parsed.get('age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe')} "
        f"age5_gate0_transport_family_contract_progress={parsed.get('age5_full_real_gate0_transport_family_contract_selftest_progress_present')} "
        f"age5_gate0_transport_family_transport_contract_completed={parsed.get('age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks')} "
        f"age5_gate0_transport_family_transport_contract_total={parsed.get('age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks')} "
        f"age5_gate0_transport_family_transport_contract_checks_text={parsed.get('age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text')} "
        f"age5_gate0_transport_family_transport_contract_current_probe={parsed.get('age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe')} "
        f"age5_gate0_transport_family_transport_contract_last_completed_probe={parsed.get('age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe')} "
        f"age5_gate0_transport_family_transport_contract_progress={parsed.get('age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present')} "
        f"age5_lang_runtime_family_transport_contract_completed={parsed.get('age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks')} "
        f"age5_lang_runtime_family_transport_contract_total={parsed.get('age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks')} "
        f"age5_lang_runtime_family_transport_contract_checks_text={parsed.get('age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text')} "
        f"age5_lang_runtime_family_transport_contract_current_probe={parsed.get('age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe')} "
        f"age5_lang_runtime_family_transport_contract_last_completed_probe={parsed.get('age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe')} "
        f"age5_lang_runtime_family_transport_contract_progress={parsed.get('age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present')} "
        f"age5_gate0_runtime_family_transport_contract_completed={parsed.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks')} "
        f"age5_gate0_runtime_family_transport_contract_total={parsed.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks')} "
        f"age5_gate0_runtime_family_transport_contract_checks_text={parsed.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text')} "
        f"age5_gate0_runtime_family_transport_contract_current_probe={parsed.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe')} "
        f"age5_gate0_runtime_family_transport_contract_last_completed_probe={parsed.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe')} "
        f"age5_gate0_runtime_family_transport_contract_progress={parsed.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present')} "
        f"age5_lang_surface_family_transport_contract_completed={parsed.get('age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks')} "
        f"age5_lang_surface_family_transport_contract_total={parsed.get('age5_full_real_lang_surface_family_transport_contract_selftest_total_checks')} "
        f"age5_lang_surface_family_transport_contract_checks_text={parsed.get('age5_full_real_lang_surface_family_transport_contract_selftest_checks_text')} "
        f"age5_lang_surface_family_transport_contract_current_probe={parsed.get('age5_full_real_lang_surface_family_transport_contract_selftest_current_probe')} "
        f"age5_lang_surface_family_transport_contract_last_completed_probe={parsed.get('age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe')} "
        f"age5_lang_surface_family_transport_contract_progress={parsed.get('age5_full_real_lang_surface_family_transport_contract_selftest_progress_present')} "
        f"age5_bogae_alias_family_contract_completed={parsed.get('age5_full_real_bogae_alias_family_contract_selftest_completed_checks')} "
        f"age5_bogae_alias_family_contract_total={parsed.get('age5_full_real_bogae_alias_family_contract_selftest_total_checks')} "
        f"age5_bogae_alias_family_contract_checks_text={parsed.get('age5_full_real_bogae_alias_family_contract_selftest_checks_text')} "
        f"age5_bogae_alias_family_contract_current_probe={parsed.get('age5_full_real_bogae_alias_family_contract_selftest_current_probe')} "
        f"age5_bogae_alias_family_contract_last_completed_probe={parsed.get('age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe')} "
        f"age5_bogae_alias_family_contract_progress={parsed.get('age5_full_real_bogae_alias_family_contract_selftest_progress_present')} "
        f"age5_bogae_alias_family_transport_contract_completed={parsed.get('age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks')} "
        f"age5_bogae_alias_family_transport_contract_total={parsed.get('age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks')} "
        f"age5_bogae_alias_family_transport_contract_checks_text={parsed.get('age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text')} "
        f"age5_bogae_alias_family_transport_contract_current_probe={parsed.get('age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe')} "
        f"age5_bogae_alias_family_transport_contract_last_completed_probe={parsed.get('age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe')} "
        f"age5_bogae_alias_family_transport_contract_progress={parsed.get('age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present')} "
        f"age5_runtime_helper_negative={parsed.get('age5_combined_heavy_runtime_helper_negative_status')} "
        f"age5_group_id_summary_negative={parsed.get('age5_combined_heavy_group_id_summary_negative_status')} "
        f"age5_child_summary_defaults={parsed.get('ci_sanity_age5_combined_heavy_child_summary_default_fields')} "
        f"age5_sync_child_summary_defaults={parsed.get('ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields')} "
        f"oi_failed={parsed.get('oi_failed_packs')}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
