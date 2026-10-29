#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path


EXPECTED_SCHEMA = "ddn.ci.gate_final_status_line.v1"
EXPECTED_KEYS = [
    "schema",
    "status",
    "overall_ok",
    "failed_steps",
    "aggregate_status",
    "age4_proof_ok",
    "age4_proof_failed_criteria",
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
    "age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe",
    "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present",
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
    "report_index",
    "aggregate_status_line",
    "aggregate_status_parse",
    "generated_at_utc",
    "reason",
]
TOKEN_RE = re.compile(r'([A-Za-z0-9_]+)=("([^"\\]|\\.)*"|[^ \t]+)')


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
    parser = argparse.ArgumentParser(description="Validate ci_gate_final_status_line.txt format")
    parser.add_argument("--status-line", required=True, help="path to ci_gate_final_status_line.txt")
    parser.add_argument("--aggregate-status-parse", required=True, help="path to ci_aggregate_status_line_parse.detjson")
    parser.add_argument("--gate-index", required=True, help="path to ci_gate_report_index.detjson")
    parser.add_argument("--require-pass", action="store_true", help="also require status=pass")
    args = parser.parse_args()

    status_line_path = Path(args.status_line)
    parse_path = Path(args.aggregate_status_parse)
    index_path = Path(args.gate_index)
    if not status_line_path.exists():
        print(f"missing status-line: {status_line_path}", file=sys.stderr)
        return 1
    parsed_line = parse_tokens(status_line_path.read_text(encoding="utf-8"))
    if parsed_line is None:
        print(f"invalid status-line format: {status_line_path}", file=sys.stderr)
        return 1
    if list(parsed_line.keys()) != EXPECTED_KEYS:
        print("status-line key order mismatch", file=sys.stderr)
        return 1
    if parsed_line.get("schema") != EXPECTED_SCHEMA:
        print("status-line schema mismatch", file=sys.stderr)
        return 1
    if parsed_line.get("status") not in {"pass", "fail"}:
        print("status-line status invalid", file=sys.stderr)
        return 1
    if parsed_line.get("overall_ok") not in {"0", "1"}:
        print("status-line overall_ok invalid", file=sys.stderr)
        return 1
    if parsed_line.get("aggregate_status") not in {"pass", "fail"}:
        print("status-line aggregate_status invalid", file=sys.stderr)
        return 1
    if parsed_line.get("age4_proof_ok") not in {"0", "1"}:
        print("status-line age4_proof_ok invalid", file=sys.stderr)
        return 1
    try:
        failed_steps = int(parsed_line.get("failed_steps", "-1"))
    except ValueError:
        print("status-line failed_steps invalid", file=sys.stderr)
        return 1
    try:
        age4_proof_failed = int(parsed_line.get("age4_proof_failed_criteria", "-1"))
    except ValueError:
        print("status-line age4_proof_failed_criteria invalid", file=sys.stderr)
        return 1
    if parsed_line.get("age5_full_real_w107_golden_index_selftest_progress_present") not in {"0", "1"}:
        print("status-line age5_full_real_w107_golden_index_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if parsed_line.get("age5_full_real_w107_progress_contract_selftest_progress_present") not in {"0", "1"}:
        print("status-line age5_full_real_w107_progress_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if parsed_line.get("age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed_line.get("age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed_line.get("age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed_line.get("age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed_line.get("age5_full_real_proof_certificate_family_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_proof_certificate_family_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed_line.get("age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed_line.get("age5_full_real_proof_family_contract_selftest_progress_present") not in {"0", "1"}:
        print("status-line age5_full_real_proof_family_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if parsed_line.get("age5_full_real_lang_surface_family_contract_selftest_progress_present") not in {"0", "1"}:
        print("status-line age5_full_real_lang_surface_family_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if parsed_line.get("age5_full_real_lang_runtime_family_contract_selftest_progress_present") not in {"0", "1"}:
        print("status-line age5_full_real_lang_runtime_family_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if parsed_line.get("age5_full_real_gate0_family_contract_selftest_progress_present") not in {"0", "1"}:
        print("status-line age5_full_real_gate0_family_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if parsed_line.get("age5_full_real_gate0_surface_family_contract_selftest_progress_present") not in {"0", "1"}:
        print("status-line age5_full_real_gate0_surface_family_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if parsed_line.get("age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed_line.get("age5_full_real_gate0_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_gate0_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed_line.get("age5_full_real_gate0_transport_family_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_gate0_transport_family_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed_line.get("age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed_line.get("age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed_line.get("age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed_line.get("age5_full_real_lang_surface_family_transport_contract_selftest_progress_present") not in {"0", "1"}:
        print("status-line age5_full_real_lang_surface_family_transport_contract_selftest_progress_present invalid", file=sys.stderr)
        return 1
    if parsed_line.get("age5_full_real_proof_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_proof_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed_line.get("age5_full_real_bogae_alias_family_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_bogae_alias_family_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    if parsed_line.get("age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        print(
            "status-line age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present invalid",
            file=sys.stderr,
        )
        return 1
    for key in (
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
        "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks",
        "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks",
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks",
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks",
        "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks",
        "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks",
        "age5_full_real_proof_family_transport_contract_selftest_completed_checks",
        "age5_full_real_proof_family_transport_contract_selftest_total_checks",
        "age5_full_real_bogae_alias_family_contract_selftest_completed_checks",
        "age5_full_real_bogae_alias_family_contract_selftest_total_checks",
        "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks",
        "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks",
    ):
        value = str(parsed_line.get(key, "")).strip()
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            print(f"status-line {key} invalid int", file=sys.stderr)
            return 1

    parse_doc = load_json(parse_path)
    if parse_doc is None:
        print(f"invalid aggregate status parse json: {parse_path}", file=sys.stderr)
        return 1
    parse_parsed = parse_doc.get("parsed")
    if not isinstance(parse_parsed, dict):
        print("aggregate status parse missing parsed object", file=sys.stderr)
        return 1
    expected_agg_status = str(parse_parsed.get("status", "fail")).strip() or "fail"
    expected_age4_proof_ok = "1" if str(parse_parsed.get("age4_proof_ok", "0")).strip() == "1" else "0"
    try:
        expected_age4_proof_failed = int(str(parse_parsed.get("age4_proof_failed_criteria", "-1")).strip())
    except ValueError:
        print("aggregate status parse age4_proof_failed_criteria invalid", file=sys.stderr)
        return 1
    if parsed_line.get("aggregate_status") != expected_agg_status:
        print("aggregate_status mismatch with parse json", file=sys.stderr)
        return 1
    if parsed_line.get("age4_proof_ok") != expected_age4_proof_ok:
        print("age4_proof_ok mismatch with parse json", file=sys.stderr)
        return 1
    if age4_proof_failed != expected_age4_proof_failed:
        print("age4_proof_failed_criteria mismatch with parse json", file=sys.stderr)
        return 1
    expected_w107_progress = {
        "age5_full_real_w107_golden_index_selftest_active_cases": (
            str(parse_parsed.get("age5_full_real_w107_golden_index_selftest_active_cases", "-")).strip() or "-"
        ),
        "age5_full_real_w107_golden_index_selftest_inactive_cases": (
            str(parse_parsed.get("age5_full_real_w107_golden_index_selftest_inactive_cases", "-")).strip() or "-"
        ),
        "age5_full_real_w107_golden_index_selftest_index_codes": (
            str(parse_parsed.get("age5_full_real_w107_golden_index_selftest_index_codes", "-")).strip() or "-"
        ),
        "age5_full_real_w107_golden_index_selftest_current_probe": (
            str(parse_parsed.get("age5_full_real_w107_golden_index_selftest_current_probe", "-")).strip() or "-"
        ),
        "age5_full_real_w107_golden_index_selftest_last_completed_probe": (
            str(parse_parsed.get("age5_full_real_w107_golden_index_selftest_last_completed_probe", "-")).strip() or "-"
        ),
        "age5_full_real_w107_golden_index_selftest_progress_present": (
            str(parse_parsed.get("age5_full_real_w107_golden_index_selftest_progress_present", "0")).strip() or "0"
        ),
        "age5_full_real_w107_progress_contract_selftest_completed_checks": (
            str(parse_parsed.get("age5_full_real_w107_progress_contract_selftest_completed_checks", "-")).strip() or "-"
        ),
        "age5_full_real_w107_progress_contract_selftest_total_checks": (
            str(parse_parsed.get("age5_full_real_w107_progress_contract_selftest_total_checks", "-")).strip() or "-"
        ),
        "age5_full_real_w107_progress_contract_selftest_checks_text": (
            str(parse_parsed.get("age5_full_real_w107_progress_contract_selftest_checks_text", "-")).strip() or "-"
        ),
        "age5_full_real_w107_progress_contract_selftest_current_probe": (
            str(parse_parsed.get("age5_full_real_w107_progress_contract_selftest_current_probe", "-")).strip() or "-"
        ),
        "age5_full_real_w107_progress_contract_selftest_last_completed_probe": (
            str(parse_parsed.get("age5_full_real_w107_progress_contract_selftest_last_completed_probe", "-")).strip() or "-"
        ),
        "age5_full_real_w107_progress_contract_selftest_progress_present": (
            str(parse_parsed.get("age5_full_real_w107_progress_contract_selftest_progress_present", "0")).strip() or "0"
        ),
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text": (
            str(
                parse_parsed.get(
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present": (
            str(
                parse_parsed.get(
                    "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present": (
            str(
                parse_parsed.get(
                    "age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text": (
            str(
                parse_parsed.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present": (
            str(
                parse_parsed.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_gate0_family_contract_selftest_completed_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_family_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_contract_selftest_total_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_family_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_contract_selftest_checks_text": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_family_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_contract_selftest_current_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_family_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_contract_selftest_last_completed_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_family_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_contract_selftest_progress_present": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_family_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_gate0_surface_family_contract_selftest_completed_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_surface_family_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_contract_selftest_total_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_surface_family_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_contract_selftest_checks_text": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_surface_family_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_contract_selftest_current_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_surface_family_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_contract_selftest_progress_present": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_surface_family_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_gate0_family_transport_contract_selftest_completed_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_transport_contract_selftest_total_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_transport_contract_selftest_checks_text": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_transport_contract_selftest_current_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_family_transport_contract_selftest_progress_present": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_gate0_transport_family_contract_selftest_completed_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_contract_selftest_total_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_contract_selftest_checks_text": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_contract_selftest_current_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_contract_selftest_progress_present": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present": (
            str(
                parse_parsed.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_surface_family_transport_contract_selftest_checks_text": (
            str(
                parse_parsed.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_surface_family_transport_contract_selftest_current_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present": (
            str(
                parse_parsed.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
        "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks": (
            str(
                parse_parsed.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text": (
            str(
                parse_parsed.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe": (
            str(
                parse_parsed.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ).strip()
            or "-"
        ),
        "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present": (
            str(
                parse_parsed.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ).strip()
            or "0"
        ),
    }
    for key, expected in expected_w107_progress.items():
        if str(parsed_line.get(key, "")).strip() != expected:
            print(f"{key} mismatch with parse json", file=sys.stderr)
            return 1
    if parsed_line.get("aggregate_status_parse") != str(parse_path):
        print("aggregate_status_parse path mismatch", file=sys.stderr)
        return 1
    if parsed_line.get("aggregate_status_line") != str(parse_doc.get("status_line_path", "")):
        print("aggregate_status_line path mismatch", file=sys.stderr)
        return 1

    index_doc = load_json(index_path)
    if index_doc is None:
        print(f"invalid gate index json: {index_path}", file=sys.stderr)
        return 1
    steps = index_doc.get("steps")
    if not isinstance(steps, list):
        print("gate index missing steps list", file=sys.stderr)
        return 1
    expected_failed = sum(1 for row in steps if isinstance(row, dict) and not bool(row.get("ok", False)))
    if failed_steps != expected_failed:
        print(f"failed_steps mismatch: line={failed_steps} index={expected_failed}", file=sys.stderr)
        return 1
    expected_ok = "1" if expected_failed == 0 else "0"
    if parsed_line.get("overall_ok") != expected_ok:
        print("overall_ok mismatch with gate index", file=sys.stderr)
        return 1
    if parsed_line.get("report_index") != str(index_path):
        print("report_index path mismatch", file=sys.stderr)
        return 1

    if args.require_pass and parsed_line.get("status") != "pass":
        print("final status is not pass", file=sys.stderr)
        return 1

    print(
        "[ci-gate-final-status-line-check] ok "
        f"status={parsed_line.get('status')} overall_ok={parsed_line.get('overall_ok')} "
        f"failed_steps={failed_steps} aggregate_status={parsed_line.get('aggregate_status')} "
        f"age4_proof_ok={parsed_line.get('age4_proof_ok')} age4_proof_failed={age4_proof_failed} "
        f"age5_w107_active={parsed_line.get('age5_full_real_w107_golden_index_selftest_active_cases')} "
        f"age5_w107_inactive={parsed_line.get('age5_full_real_w107_golden_index_selftest_inactive_cases')} "
        f"age5_w107_index_codes={parsed_line.get('age5_full_real_w107_golden_index_selftest_index_codes')} "
        f"age5_w107_last_completed_probe={parsed_line.get('age5_full_real_w107_golden_index_selftest_last_completed_probe')} "
        f"age5_w107_progress={parsed_line.get('age5_full_real_w107_golden_index_selftest_progress_present')} "
        f"age5_w107_contract_completed={parsed_line.get('age5_full_real_w107_progress_contract_selftest_completed_checks')} "
        f"age5_w107_contract_total={parsed_line.get('age5_full_real_w107_progress_contract_selftest_total_checks')} "
        f"age5_w107_contract_checks_text={parsed_line.get('age5_full_real_w107_progress_contract_selftest_checks_text')} "
        f"age5_w107_contract_last_completed_probe={parsed_line.get('age5_full_real_w107_progress_contract_selftest_last_completed_probe')} "
        f"age5_w107_contract_progress={parsed_line.get('age5_full_real_w107_progress_contract_selftest_progress_present')}"
        f" age5_age1_immediate_proof_operation_contract_completed={parsed_line.get('age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks')}"
        f" age5_age1_immediate_proof_operation_contract_total={parsed_line.get('age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks')}"
        f" age5_age1_immediate_proof_operation_contract_checks_text={parsed_line.get('age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text')}"
        f" age5_age1_immediate_proof_operation_contract_last_completed_probe={parsed_line.get('age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe')}"
        f" age5_age1_immediate_proof_operation_contract_progress={parsed_line.get('age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present')}"
        f" age5_proof_certificate_v1_consumer_contract_completed={parsed_line.get('age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks')}"
        f" age5_proof_certificate_v1_consumer_contract_total={parsed_line.get('age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks')}"
        f" age5_proof_certificate_v1_consumer_contract_checks_text={parsed_line.get('age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text')}"
        f" age5_proof_certificate_v1_consumer_contract_last_completed_probe={parsed_line.get('age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe')}"
        f" age5_proof_certificate_v1_consumer_contract_progress={parsed_line.get('age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present')}"
        f" age5_proof_certificate_v1_verify_report_digest_contract_completed={parsed_line.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks')}"
        f" age5_proof_certificate_v1_verify_report_digest_contract_total={parsed_line.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks')}"
        f" age5_proof_certificate_v1_verify_report_digest_contract_checks_text={parsed_line.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text')}"
        f" age5_proof_certificate_v1_verify_report_digest_contract_current_probe={parsed_line.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe')}"
        f" age5_proof_certificate_v1_verify_report_digest_contract_last_completed_probe={parsed_line.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe')}"
        f" age5_proof_certificate_v1_verify_report_digest_contract_progress={parsed_line.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present')}"
        f" age5_proof_certificate_v1_family_contract_completed={parsed_line.get('age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks')}"
        f" age5_proof_certificate_v1_family_contract_total={parsed_line.get('age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks')}"
        f" age5_proof_certificate_v1_family_contract_checks_text={parsed_line.get('age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text')}"
        f" age5_proof_certificate_v1_family_contract_current_probe={parsed_line.get('age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe')}"
        f" age5_proof_certificate_v1_family_contract_last_completed_probe={parsed_line.get('age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe')}"
        f" age5_proof_certificate_v1_family_contract_progress={parsed_line.get('age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present')}"
        f" age5_proof_certificate_family_contract_completed={parsed_line.get('age5_full_real_proof_certificate_family_contract_selftest_completed_checks')}"
        f" age5_proof_certificate_family_contract_total={parsed_line.get('age5_full_real_proof_certificate_family_contract_selftest_total_checks')}"
        f" age5_proof_certificate_family_contract_checks_text={parsed_line.get('age5_full_real_proof_certificate_family_contract_selftest_checks_text')}"
        f" age5_proof_certificate_family_contract_current_probe={parsed_line.get('age5_full_real_proof_certificate_family_contract_selftest_current_probe')}"
        f" age5_proof_certificate_family_contract_last_completed_probe={parsed_line.get('age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe')}"
        f" age5_proof_certificate_family_contract_progress={parsed_line.get('age5_full_real_proof_certificate_family_contract_selftest_progress_present')}"
        f" age5_proof_certificate_family_transport_contract_completed={parsed_line.get('age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks')}"
        f" age5_proof_certificate_family_transport_contract_total={parsed_line.get('age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks')}"
        f" age5_proof_certificate_family_transport_contract_checks_text={parsed_line.get('age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text')}"
        f" age5_proof_certificate_family_transport_contract_current_probe={parsed_line.get('age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe')}"
        f" age5_proof_certificate_family_transport_contract_last_completed_probe={parsed_line.get('age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe')}"
        f" age5_proof_certificate_family_transport_contract_progress={parsed_line.get('age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present')}"
        f" age5_proof_family_contract_completed={parsed_line.get('age5_full_real_proof_family_contract_selftest_completed_checks')}"
        f" age5_proof_family_contract_total={parsed_line.get('age5_full_real_proof_family_contract_selftest_total_checks')}"
        f" age5_proof_family_contract_checks_text={parsed_line.get('age5_full_real_proof_family_contract_selftest_checks_text')}"
        f" age5_proof_family_contract_current_probe={parsed_line.get('age5_full_real_proof_family_contract_selftest_current_probe')}"
        f" age5_proof_family_contract_last_completed_probe={parsed_line.get('age5_full_real_proof_family_contract_selftest_last_completed_probe')}"
        f" age5_proof_family_contract_progress={parsed_line.get('age5_full_real_proof_family_contract_selftest_progress_present')}"
        f" age5_lang_surface_family_contract_completed={parsed_line.get('age5_full_real_lang_surface_family_contract_selftest_completed_checks')}"
        f" age5_lang_surface_family_contract_total={parsed_line.get('age5_full_real_lang_surface_family_contract_selftest_total_checks')}"
        f" age5_lang_surface_family_contract_checks_text={parsed_line.get('age5_full_real_lang_surface_family_contract_selftest_checks_text')}"
        f" age5_lang_surface_family_contract_current_probe={parsed_line.get('age5_full_real_lang_surface_family_contract_selftest_current_probe')}"
        f" age5_lang_surface_family_contract_last_completed_probe={parsed_line.get('age5_full_real_lang_surface_family_contract_selftest_last_completed_probe')}"
        f" age5_lang_surface_family_contract_progress={parsed_line.get('age5_full_real_lang_surface_family_contract_selftest_progress_present')}"
        f" age5_lang_runtime_family_contract_completed={parsed_line.get('age5_full_real_lang_runtime_family_contract_selftest_completed_checks')}"
        f" age5_lang_runtime_family_contract_total={parsed_line.get('age5_full_real_lang_runtime_family_contract_selftest_total_checks')}"
        f" age5_lang_runtime_family_contract_checks_text={parsed_line.get('age5_full_real_lang_runtime_family_contract_selftest_checks_text')}"
        f" age5_lang_runtime_family_contract_current_probe={parsed_line.get('age5_full_real_lang_runtime_family_contract_selftest_current_probe')}"
        f" age5_lang_runtime_family_contract_last_completed_probe={parsed_line.get('age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe')}"
        f" age5_lang_runtime_family_contract_progress={parsed_line.get('age5_full_real_lang_runtime_family_contract_selftest_progress_present')}"
        f" age5_gate0_family_contract_completed={parsed_line.get('age5_full_real_gate0_family_contract_selftest_completed_checks')}"
        f" age5_gate0_family_contract_total={parsed_line.get('age5_full_real_gate0_family_contract_selftest_total_checks')}"
        f" age5_gate0_family_contract_checks_text={parsed_line.get('age5_full_real_gate0_family_contract_selftest_checks_text')}"
        f" age5_gate0_family_contract_current_probe={parsed_line.get('age5_full_real_gate0_family_contract_selftest_current_probe')}"
        f" age5_gate0_family_contract_last_completed_probe={parsed_line.get('age5_full_real_gate0_family_contract_selftest_last_completed_probe')}"
        f" age5_gate0_family_contract_progress={parsed_line.get('age5_full_real_gate0_family_contract_selftest_progress_present')}"
        f" age5_gate0_surface_family_contract_completed={parsed_line.get('age5_full_real_gate0_surface_family_contract_selftest_completed_checks')}"
        f" age5_gate0_surface_family_contract_total={parsed_line.get('age5_full_real_gate0_surface_family_contract_selftest_total_checks')}"
        f" age5_gate0_surface_family_contract_checks_text={parsed_line.get('age5_full_real_gate0_surface_family_contract_selftest_checks_text')}"
        f" age5_gate0_surface_family_contract_current_probe={parsed_line.get('age5_full_real_gate0_surface_family_contract_selftest_current_probe')}"
        f" age5_gate0_surface_family_contract_last_completed_probe={parsed_line.get('age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe')}"
        f" age5_gate0_surface_family_contract_progress={parsed_line.get('age5_full_real_gate0_surface_family_contract_selftest_progress_present')}"
        f" age5_gate0_surface_family_transport_contract_completed={parsed_line.get('age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks')}"
        f" age5_gate0_surface_family_transport_contract_total={parsed_line.get('age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks')}"
        f" age5_gate0_surface_family_transport_contract_checks_text={parsed_line.get('age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text')}"
        f" age5_gate0_surface_family_transport_contract_current_probe={parsed_line.get('age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe')}"
        f" age5_gate0_surface_family_transport_contract_last_completed_probe={parsed_line.get('age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe')}"
        f" age5_gate0_surface_family_transport_contract_progress={parsed_line.get('age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present')}"
        f" age5_gate0_family_transport_contract_completed={parsed_line.get('age5_full_real_gate0_family_transport_contract_selftest_completed_checks')}"
        f" age5_gate0_family_transport_contract_total={parsed_line.get('age5_full_real_gate0_family_transport_contract_selftest_total_checks')}"
        f" age5_gate0_family_transport_contract_checks_text={parsed_line.get('age5_full_real_gate0_family_transport_contract_selftest_checks_text')}"
        f" age5_gate0_family_transport_contract_current_probe={parsed_line.get('age5_full_real_gate0_family_transport_contract_selftest_current_probe')}"
        f" age5_gate0_family_transport_contract_last_completed_probe={parsed_line.get('age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe')}"
        f" age5_gate0_family_transport_contract_progress={parsed_line.get('age5_full_real_gate0_family_transport_contract_selftest_progress_present')}"
        f" age5_gate0_transport_family_contract_completed={parsed_line.get('age5_full_real_gate0_transport_family_contract_selftest_completed_checks')}"
        f" age5_gate0_transport_family_contract_total={parsed_line.get('age5_full_real_gate0_transport_family_contract_selftest_total_checks')}"
        f" age5_gate0_transport_family_contract_checks_text={parsed_line.get('age5_full_real_gate0_transport_family_contract_selftest_checks_text')}"
        f" age5_gate0_transport_family_contract_current_probe={parsed_line.get('age5_full_real_gate0_transport_family_contract_selftest_current_probe')}"
        f" age5_gate0_transport_family_contract_last_completed_probe={parsed_line.get('age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe')}"
        f" age5_gate0_transport_family_contract_progress={parsed_line.get('age5_full_real_gate0_transport_family_contract_selftest_progress_present')}"
        f" age5_gate0_transport_family_transport_contract_completed={parsed_line.get('age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks')}"
        f" age5_gate0_transport_family_transport_contract_total={parsed_line.get('age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks')}"
        f" age5_gate0_transport_family_transport_contract_checks_text={parsed_line.get('age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text')}"
        f" age5_gate0_transport_family_transport_contract_current_probe={parsed_line.get('age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe')}"
        f" age5_gate0_transport_family_transport_contract_last_completed_probe={parsed_line.get('age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe')}"
        f" age5_gate0_transport_family_transport_contract_progress={parsed_line.get('age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present')}"
        f" age5_lang_runtime_family_transport_contract_completed={parsed_line.get('age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks')}"
        f" age5_lang_runtime_family_transport_contract_total={parsed_line.get('age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks')}"
        f" age5_lang_runtime_family_transport_contract_checks_text={parsed_line.get('age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text')}"
        f" age5_lang_runtime_family_transport_contract_current_probe={parsed_line.get('age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe')}"
        f" age5_lang_runtime_family_transport_contract_last_completed_probe={parsed_line.get('age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe')}"
        f" age5_lang_runtime_family_transport_contract_progress={parsed_line.get('age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present')}"
        f" age5_gate0_runtime_family_transport_contract_completed={parsed_line.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks')}"
        f" age5_gate0_runtime_family_transport_contract_total={parsed_line.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks')}"
        f" age5_gate0_runtime_family_transport_contract_checks_text={parsed_line.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text')}"
        f" age5_gate0_runtime_family_transport_contract_current_probe={parsed_line.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe')}"
        f" age5_gate0_runtime_family_transport_contract_last_completed_probe={parsed_line.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe')}"
        f" age5_gate0_runtime_family_transport_contract_progress={parsed_line.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present')}"
        f" age5_lang_surface_family_transport_contract_completed={parsed_line.get('age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks')}"
        f" age5_lang_surface_family_transport_contract_total={parsed_line.get('age5_full_real_lang_surface_family_transport_contract_selftest_total_checks')}"
        f" age5_lang_surface_family_transport_contract_checks_text={parsed_line.get('age5_full_real_lang_surface_family_transport_contract_selftest_checks_text')}"
        f" age5_lang_surface_family_transport_contract_current_probe={parsed_line.get('age5_full_real_lang_surface_family_transport_contract_selftest_current_probe')}"
        f" age5_lang_surface_family_transport_contract_last_completed_probe={parsed_line.get('age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe')}"
        f" age5_lang_surface_family_transport_contract_progress={parsed_line.get('age5_full_real_lang_surface_family_transport_contract_selftest_progress_present')}"
        f" age5_proof_family_transport_contract_completed={parsed_line.get('age5_full_real_proof_family_transport_contract_selftest_completed_checks')}"
        f" age5_proof_family_transport_contract_total={parsed_line.get('age5_full_real_proof_family_transport_contract_selftest_total_checks')}"
        f" age5_proof_family_transport_contract_checks_text={parsed_line.get('age5_full_real_proof_family_transport_contract_selftest_checks_text')}"
        f" age5_proof_family_transport_contract_current_probe={parsed_line.get('age5_full_real_proof_family_transport_contract_selftest_current_probe')}"
        f" age5_proof_family_transport_contract_last_completed_probe={parsed_line.get('age5_full_real_proof_family_transport_contract_selftest_last_completed_probe')}"
        f" age5_proof_family_transport_contract_progress={parsed_line.get('age5_full_real_proof_family_transport_contract_selftest_progress_present')}"
        f" age5_bogae_alias_family_contract_completed={parsed_line.get('age5_full_real_bogae_alias_family_contract_selftest_completed_checks')}"
        f" age5_bogae_alias_family_contract_total={parsed_line.get('age5_full_real_bogae_alias_family_contract_selftest_total_checks')}"
        f" age5_bogae_alias_family_contract_checks_text={parsed_line.get('age5_full_real_bogae_alias_family_contract_selftest_checks_text')}"
        f" age5_bogae_alias_family_contract_current_probe={parsed_line.get('age5_full_real_bogae_alias_family_contract_selftest_current_probe')}"
        f" age5_bogae_alias_family_contract_last_completed_probe={parsed_line.get('age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe')}"
        f" age5_bogae_alias_family_contract_progress={parsed_line.get('age5_full_real_bogae_alias_family_contract_selftest_progress_present')}"
        f" age5_bogae_alias_family_transport_contract_completed={parsed_line.get('age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks')}"
        f" age5_bogae_alias_family_transport_contract_total={parsed_line.get('age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks')}"
        f" age5_bogae_alias_family_transport_contract_checks_text={parsed_line.get('age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text')}"
        f" age5_bogae_alias_family_transport_contract_current_probe={parsed_line.get('age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe')}"
        f" age5_bogae_alias_family_transport_contract_last_completed_probe={parsed_line.get('age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe')}"
        f" age5_bogae_alias_family_transport_contract_progress={parsed_line.get('age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present')}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
