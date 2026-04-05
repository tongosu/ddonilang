#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TESTS_DIR = ROOT / "tests"
if str(TESTS_DIR) not in sys.path:
    sys.path.insert(0, str(TESTS_DIR))

from _ci_age5_combined_heavy_contract import (
    AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
    build_age4_proof_snapshot,
    build_age4_proof_snapshot_text,
)

EXPECTED_SCHEMA = "ddn.ci.gate_final_status_line.v1"
AGE4_PROOF_FAILED_PREVIEW_KEY = "age4_proof_failed_preview"
AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY = "age5_policy_age4_proof_snapshot_fields_text"
AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY = "age5_policy_age4_proof_snapshot_text"
AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY = "age5_policy_age4_proof_source_snapshot_fields_text"
AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY = "age5_policy_age4_proof_gate_result_present"
AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY = "age5_policy_age4_proof_gate_result_parity"
AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY = "age5_policy_age4_proof_final_status_parse_present"
AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY = "age5_policy_age4_proof_final_status_parse_parity"
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


def format_age4_proof_failed_preview(failed: object) -> str:
    if isinstance(failed, list):
        items = [str(item).strip() for item in failed if str(item).strip()]
        if not items:
            return "-"
        preview = items[:2]
        if len(items) > 2:
            preview.append(f"+{len(items) - 2}more")
        return ",".join(preview)
    try:
        count = int(failed)
    except Exception:
        return "-"
    if count <= 0:
        return "-"
    return f"count:{count}"


def load_age4_proof_failed_preview(gate_index_path: Path | None) -> str:
    if gate_index_path is None:
        return "-"
    index_doc = load_json(gate_index_path)
    if not isinstance(index_doc, dict):
        return "-"
    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return "-"
    aggregate_path_text = str(reports.get("aggregate", "")).strip()
    if not aggregate_path_text:
        return "-"
    aggregate_doc = load_json(Path(aggregate_path_text))
    if not isinstance(aggregate_doc, dict):
        return "-"
    age4_doc = aggregate_doc.get("age4")
    if not isinstance(age4_doc, dict):
        return "-"
    preview = str(age4_doc.get("proof_artifact_failed_preview", "")).strip()
    if preview:
        return preview
    return format_age4_proof_failed_preview(age4_doc.get("proof_artifact_failed_criteria"))


def load_age5_policy_snapshot(gate_index_path: Path | None) -> dict[str, str]:
    default_snapshot = {
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY: "age4_proof_ok=0|age4_proof_failed_criteria=-1|age4_proof_failed_preview=-",
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY: build_age4_proof_snapshot_text(build_age4_proof_snapshot()),
        AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY: AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY: "0",
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY: "0",
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY: "0",
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY: "0",
    }
    if gate_index_path is None:
        return default_snapshot
    index_doc = load_json(gate_index_path)
    if not isinstance(index_doc, dict):
        return default_snapshot
    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return default_snapshot
    aggregate_path_text = str(reports.get("aggregate", "")).strip()
    if not aggregate_path_text:
        return default_snapshot
    aggregate_doc = load_json(Path(aggregate_path_text))
    if not isinstance(aggregate_doc, dict):
        return default_snapshot
    age5_doc = aggregate_doc.get("age5")
    if not isinstance(age5_doc, dict):
        return default_snapshot
    return {
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY: (
            str(age5_doc.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY, default_snapshot[AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY])).strip()
            or default_snapshot[AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY: (
            str(age5_doc.get(AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY, default_snapshot[AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY])).strip()
            or default_snapshot[AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY: (
            str(age5_doc.get(AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY, default_snapshot[AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY])).strip()
            or default_snapshot[AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY: (
            str(age5_doc.get(AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY, default_snapshot[AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY])).strip()
            or default_snapshot[AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY: (
            str(age5_doc.get(AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY, default_snapshot[AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY])).strip()
            or default_snapshot[AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY: (
            str(age5_doc.get(AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY, default_snapshot[AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY])).strip()
            or default_snapshot[AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY: (
            str(age5_doc.get(AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY, default_snapshot[AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY])).strip()
            or default_snapshot[AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY]
        ),
    }


def parse_tokens(text: str) -> dict[str, str] | None:
    line = text.strip()
    if not line:
        return None
    out: dict[str, str] = {}
    pos = 0
    for match in TOKEN_RE.finditer(line):
        if line[pos : match.start()].strip():
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
    if line[pos:].strip():
        return None
    return out


def parse_status_line(path: Path) -> tuple[dict[str, str] | None, str]:
    if not path.exists():
        return None, f"missing status line: {path}"
    parsed = parse_tokens(path.read_text(encoding="utf-8"))
    if parsed is None:
        return None, "invalid token format"
    if list(parsed.keys()) != EXPECTED_KEYS:
        return None, "key order mismatch"
    if parsed.get("schema") != EXPECTED_SCHEMA:
        return None, f"schema mismatch: {parsed.get('schema')}"
    if parsed.get("status") not in {"pass", "fail"}:
        return None, f"invalid status: {parsed.get('status')}"
    if parsed.get("overall_ok") not in {"0", "1"}:
        return None, f"invalid overall_ok: {parsed.get('overall_ok')}"
    if parsed.get("aggregate_status") not in {"pass", "fail"}:
        return None, f"invalid aggregate_status: {parsed.get('aggregate_status')}"
    if parsed.get("age4_proof_ok") not in {"0", "1"}:
        return None, f"invalid age4_proof_ok: {parsed.get('age4_proof_ok')}"
    try:
        int(parsed.get("failed_steps", "-1"))
    except ValueError:
        return None, "failed_steps must be int"
    try:
        int(parsed.get("age4_proof_failed_criteria", "-1"))
    except ValueError:
        return None, "age4_proof_failed_criteria must be int"
    if parsed.get("age5_full_real_w107_golden_index_selftest_progress_present") not in {"0", "1"}:
        return None, "age5_full_real_w107_golden_index_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_w107_progress_contract_selftest_progress_present") not in {"0", "1"}:
        return None, "age5_full_real_w107_progress_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        return None, "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        return None, "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        return None, "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_proof_certificate_family_contract_selftest_progress_present") not in {"0", "1"}:
        return None, "age5_full_real_proof_certificate_family_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        return None, "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_proof_family_contract_selftest_progress_present") not in {"0", "1"}:
        return None, "age5_full_real_proof_family_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_proof_family_transport_contract_selftest_progress_present") not in {"0", "1"}:
        return None, "age5_full_real_proof_family_transport_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_lang_surface_family_contract_selftest_progress_present") not in {"0", "1"}:
        return None, "age5_full_real_lang_surface_family_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_lang_runtime_family_contract_selftest_progress_present") not in {"0", "1"}:
        return None, "age5_full_real_lang_runtime_family_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_gate0_family_contract_selftest_progress_present") not in {"0", "1"}:
        return None, "age5_full_real_gate0_family_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_gate0_surface_family_contract_selftest_progress_present") not in {"0", "1"}:
        return None, "age5_full_real_gate0_surface_family_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        return None, "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        return None, "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        return None, "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_gate0_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        return None, "age5_full_real_gate0_family_transport_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_gate0_transport_family_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        return None, "age5_full_real_gate0_transport_family_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        return None, (
            "age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present must be 0/1"
        )
    if parsed.get("age5_full_real_lang_surface_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        return None, "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_bogae_alias_family_contract_selftest_progress_present") not in {"0", "1"}:
        return None, "age5_full_real_bogae_alias_family_contract_selftest_progress_present must be 0/1"
    if parsed.get("age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present") not in {
        "0",
        "1",
    }:
        return None, "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present must be 0/1"
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
        "age5_full_real_proof_family_transport_contract_selftest_completed_checks",
        "age5_full_real_proof_family_transport_contract_selftest_total_checks",
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
        "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks",
        "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks",
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks",
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks",
        "age5_full_real_gate0_family_transport_contract_selftest_completed_checks",
        "age5_full_real_gate0_family_transport_contract_selftest_total_checks",
        "age5_full_real_gate0_transport_family_contract_selftest_completed_checks",
        "age5_full_real_gate0_transport_family_contract_selftest_total_checks",
        "age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks",
        "age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks",
        "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks",
        "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks",
        "age5_full_real_bogae_alias_family_contract_selftest_completed_checks",
        "age5_full_real_bogae_alias_family_contract_selftest_total_checks",
        "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks",
        "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks",
    ):
        value = str(parsed.get(key, "")).strip()
        if value == "-":
            continue
        try:
            int(value)
        except ValueError:
            return None, f"{key} must be int-or-dash"
    return parsed, ""


def compact_line(parsed: dict[str, str]) -> str:
    return (
        f"ci_gate_status={parsed.get('status', 'fail')} "
        f"overall_ok={parsed.get('overall_ok', '0')} "
        f"failed_steps={parsed.get('failed_steps', '-1')} "
        f"aggregate_status={parsed.get('aggregate_status', 'fail')} "
        f"age4_proof_ok={parsed.get('age4_proof_ok', '0')} "
        f"age4_proof_failed={parsed.get('age4_proof_failed_criteria', '-1')} "
        f"age5_w107_active={parsed.get('age5_full_real_w107_golden_index_selftest_active_cases', '-')} "
        f"age5_w107_inactive={parsed.get('age5_full_real_w107_golden_index_selftest_inactive_cases', '-')} "
        f"age5_w107_index_codes={parsed.get('age5_full_real_w107_golden_index_selftest_index_codes', '-')} "
        f"age5_w107_current_probe={parsed.get('age5_full_real_w107_golden_index_selftest_current_probe', '-')} "
        f"age5_w107_last_completed_probe={parsed.get('age5_full_real_w107_golden_index_selftest_last_completed_probe', '-')} "
        f"age5_w107_progress={parsed.get('age5_full_real_w107_golden_index_selftest_progress_present', '0')} "
        f"age5_w107_contract_completed={parsed.get('age5_full_real_w107_progress_contract_selftest_completed_checks', '-')} "
        f"age5_w107_contract_total={parsed.get('age5_full_real_w107_progress_contract_selftest_total_checks', '-')} "
        f"age5_w107_contract_checks_text={parsed.get('age5_full_real_w107_progress_contract_selftest_checks_text', '-')} "
        f"age5_w107_contract_current_probe={parsed.get('age5_full_real_w107_progress_contract_selftest_current_probe', '-')} "
        f"age5_w107_contract_last_completed_probe={parsed.get('age5_full_real_w107_progress_contract_selftest_last_completed_probe', '-')} "
        f"age5_w107_contract_progress={parsed.get('age5_full_real_w107_progress_contract_selftest_progress_present', '0')} "
        f"age5_age1_immediate_proof_operation_contract_completed={parsed.get('age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks', '-')} "
        f"age5_age1_immediate_proof_operation_contract_total={parsed.get('age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks', '-')} "
        f"age5_age1_immediate_proof_operation_contract_checks_text={parsed.get('age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text', '-')} "
        f"age5_age1_immediate_proof_operation_contract_current_probe={parsed.get('age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe', '-')} "
        f"age5_age1_immediate_proof_operation_contract_last_completed_probe={parsed.get('age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe', '-')} "
        f"age5_age1_immediate_proof_operation_contract_progress={parsed.get('age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present', '0')} "
        f"age5_proof_certificate_v1_consumer_contract_completed={parsed.get('age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks', '-')} "
        f"age5_proof_certificate_v1_consumer_contract_total={parsed.get('age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks', '-')} "
        f"age5_proof_certificate_v1_consumer_contract_checks_text={parsed.get('age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text', '-')} "
        f"age5_proof_certificate_v1_consumer_contract_current_probe={parsed.get('age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe', '-')} "
        f"age5_proof_certificate_v1_consumer_contract_last_completed_probe={parsed.get('age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe', '-')} "
        f"age5_proof_certificate_v1_consumer_contract_progress={parsed.get('age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present', '0')} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_completed={parsed.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks', '-')} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_total={parsed.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks', '-')} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_checks_text={parsed.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text', '-')} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_current_probe={parsed.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe', '-')} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_last_completed_probe={parsed.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe', '-')} "
        f"age5_proof_certificate_v1_verify_report_digest_contract_progress={parsed.get('age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present', '0')} "
        f"age5_proof_certificate_v1_family_contract_completed={parsed.get('age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks', '-')} "
        f"age5_proof_certificate_v1_family_contract_total={parsed.get('age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks', '-')} "
        f"age5_proof_certificate_v1_family_contract_checks_text={parsed.get('age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text', '-')} "
        f"age5_proof_certificate_v1_family_contract_current_probe={parsed.get('age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe', '-')} "
        f"age5_proof_certificate_v1_family_contract_last_completed_probe={parsed.get('age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe', '-')} "
        f"age5_proof_certificate_v1_family_contract_progress={parsed.get('age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present', '0')} "
        f"age5_proof_certificate_family_contract_completed={parsed.get('age5_full_real_proof_certificate_family_contract_selftest_completed_checks', '-')} "
        f"age5_proof_certificate_family_contract_total={parsed.get('age5_full_real_proof_certificate_family_contract_selftest_total_checks', '-')} "
        f"age5_proof_certificate_family_contract_checks_text={parsed.get('age5_full_real_proof_certificate_family_contract_selftest_checks_text', '-')} "
        f"age5_proof_certificate_family_contract_current_probe={parsed.get('age5_full_real_proof_certificate_family_contract_selftest_current_probe', '-')} "
        f"age5_proof_certificate_family_contract_last_completed_probe={parsed.get('age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe', '-')} "
        f"age5_proof_certificate_family_contract_progress={parsed.get('age5_full_real_proof_certificate_family_contract_selftest_progress_present', '0')} "
        f"age5_proof_certificate_family_transport_contract_completed={parsed.get('age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks', '-')} "
        f"age5_proof_certificate_family_transport_contract_total={parsed.get('age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks', '-')} "
        f"age5_proof_certificate_family_transport_contract_checks_text={parsed.get('age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text', '-')} "
        f"age5_proof_certificate_family_transport_contract_current_probe={parsed.get('age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe', '-')} "
        f"age5_proof_certificate_family_transport_contract_last_completed_probe={parsed.get('age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe', '-')} "
        f"age5_proof_certificate_family_transport_contract_progress={parsed.get('age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present', '0')} "
        f"age5_proof_family_contract_completed={parsed.get('age5_full_real_proof_family_contract_selftest_completed_checks', '-')} "
        f"age5_proof_family_contract_total={parsed.get('age5_full_real_proof_family_contract_selftest_total_checks', '-')} "
        f"age5_proof_family_contract_checks_text={parsed.get('age5_full_real_proof_family_contract_selftest_checks_text', '-')} "
        f"age5_proof_family_contract_current_probe={parsed.get('age5_full_real_proof_family_contract_selftest_current_probe', '-')} "
        f"age5_proof_family_contract_last_completed_probe={parsed.get('age5_full_real_proof_family_contract_selftest_last_completed_probe', '-')} "
        f"age5_proof_family_contract_progress={parsed.get('age5_full_real_proof_family_contract_selftest_progress_present', '0')} "
        f"age5_proof_family_transport_contract_completed={parsed.get('age5_full_real_proof_family_transport_contract_selftest_completed_checks', '-')} "
        f"age5_proof_family_transport_contract_total={parsed.get('age5_full_real_proof_family_transport_contract_selftest_total_checks', '-')} "
        f"age5_proof_family_transport_contract_checks_text={parsed.get('age5_full_real_proof_family_transport_contract_selftest_checks_text', '-')} "
        f"age5_proof_family_transport_contract_current_probe={parsed.get('age5_full_real_proof_family_transport_contract_selftest_current_probe', '-')} "
        f"age5_proof_family_transport_contract_last_completed_probe={parsed.get('age5_full_real_proof_family_transport_contract_selftest_last_completed_probe', '-')} "
        f"age5_proof_family_transport_contract_progress={parsed.get('age5_full_real_proof_family_transport_contract_selftest_progress_present', '0')} "
        f"age5_lang_surface_family_contract_completed={parsed.get('age5_full_real_lang_surface_family_contract_selftest_completed_checks', '-')} "
        f"age5_lang_surface_family_contract_total={parsed.get('age5_full_real_lang_surface_family_contract_selftest_total_checks', '-')} "
        f"age5_lang_surface_family_contract_checks_text={parsed.get('age5_full_real_lang_surface_family_contract_selftest_checks_text', '-')} "
        f"age5_lang_surface_family_contract_current_probe={parsed.get('age5_full_real_lang_surface_family_contract_selftest_current_probe', '-')} "
        f"age5_lang_surface_family_contract_last_completed_probe={parsed.get('age5_full_real_lang_surface_family_contract_selftest_last_completed_probe', '-')} "
        f"age5_lang_surface_family_contract_progress={parsed.get('age5_full_real_lang_surface_family_contract_selftest_progress_present', '0')} "
        f"age5_lang_runtime_family_contract_completed={parsed.get('age5_full_real_lang_runtime_family_contract_selftest_completed_checks', '-')} "
        f"age5_lang_runtime_family_contract_total={parsed.get('age5_full_real_lang_runtime_family_contract_selftest_total_checks', '-')} "
        f"age5_lang_runtime_family_contract_checks_text={parsed.get('age5_full_real_lang_runtime_family_contract_selftest_checks_text', '-')} "
        f"age5_lang_runtime_family_contract_current_probe={parsed.get('age5_full_real_lang_runtime_family_contract_selftest_current_probe', '-')} "
        f"age5_lang_runtime_family_contract_last_completed_probe={parsed.get('age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe', '-')} "
        f"age5_lang_runtime_family_contract_progress={parsed.get('age5_full_real_lang_runtime_family_contract_selftest_progress_present', '0')} "
        f"age5_gate0_family_contract_completed={parsed.get('age5_full_real_gate0_family_contract_selftest_completed_checks', '-')} "
        f"age5_gate0_family_contract_total={parsed.get('age5_full_real_gate0_family_contract_selftest_total_checks', '-')} "
        f"age5_gate0_family_contract_checks_text={parsed.get('age5_full_real_gate0_family_contract_selftest_checks_text', '-')} "
        f"age5_gate0_family_contract_current_probe={parsed.get('age5_full_real_gate0_family_contract_selftest_current_probe', '-')} "
        f"age5_gate0_family_contract_last_completed_probe={parsed.get('age5_full_real_gate0_family_contract_selftest_last_completed_probe', '-')} "
        f"age5_gate0_family_contract_progress={parsed.get('age5_full_real_gate0_family_contract_selftest_progress_present', '0')} "
        f"age5_gate0_surface_family_contract_completed={parsed.get('age5_full_real_gate0_surface_family_contract_selftest_completed_checks', '-')} "
        f"age5_gate0_surface_family_contract_total={parsed.get('age5_full_real_gate0_surface_family_contract_selftest_total_checks', '-')} "
        f"age5_gate0_surface_family_contract_checks_text={parsed.get('age5_full_real_gate0_surface_family_contract_selftest_checks_text', '-')} "
        f"age5_gate0_surface_family_contract_current_probe={parsed.get('age5_full_real_gate0_surface_family_contract_selftest_current_probe', '-')} "
        f"age5_gate0_surface_family_contract_last_completed_probe={parsed.get('age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe', '-')} "
        f"age5_gate0_surface_family_contract_progress={parsed.get('age5_full_real_gate0_surface_family_contract_selftest_progress_present', '0')} "
        f"age5_gate0_surface_family_transport_contract_completed={parsed.get('age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks', '-')} "
        f"age5_gate0_surface_family_transport_contract_total={parsed.get('age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks', '-')} "
        f"age5_gate0_surface_family_transport_contract_checks_text={parsed.get('age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text', '-')} "
        f"age5_gate0_surface_family_transport_contract_current_probe={parsed.get('age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe', '-')} "
        f"age5_gate0_surface_family_transport_contract_last_completed_probe={parsed.get('age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe', '-')} "
        f"age5_gate0_surface_family_transport_contract_progress={parsed.get('age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present', '0')} "
        f"age5_lang_runtime_family_transport_contract_completed={parsed.get('age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks', '-')} "
        f"age5_lang_runtime_family_transport_contract_total={parsed.get('age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks', '-')} "
        f"age5_lang_runtime_family_transport_contract_checks_text={parsed.get('age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text', '-')} "
        f"age5_lang_runtime_family_transport_contract_current_probe={parsed.get('age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe', '-')} "
        f"age5_lang_runtime_family_transport_contract_last_completed_probe={parsed.get('age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe', '-')} "
        f"age5_lang_runtime_family_transport_contract_progress={parsed.get('age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present', '0')} "
        f"age5_gate0_runtime_family_transport_contract_completed={parsed.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks', '-')} "
        f"age5_gate0_runtime_family_transport_contract_total={parsed.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks', '-')} "
        f"age5_gate0_runtime_family_transport_contract_checks_text={parsed.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text', '-')} "
        f"age5_gate0_runtime_family_transport_contract_current_probe={parsed.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe', '-')} "
        f"age5_gate0_runtime_family_transport_contract_last_completed_probe={parsed.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe', '-')} "
        f"age5_gate0_runtime_family_transport_contract_progress={parsed.get('age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present', '0')} "
        f"age5_gate0_family_transport_contract_completed={parsed.get('age5_full_real_gate0_family_transport_contract_selftest_completed_checks', '-')} "
        f"age5_gate0_family_transport_contract_total={parsed.get('age5_full_real_gate0_family_transport_contract_selftest_total_checks', '-')} "
        f"age5_gate0_family_transport_contract_checks_text={parsed.get('age5_full_real_gate0_family_transport_contract_selftest_checks_text', '-')} "
        f"age5_gate0_family_transport_contract_current_probe={parsed.get('age5_full_real_gate0_family_transport_contract_selftest_current_probe', '-')} "
        f"age5_gate0_family_transport_contract_last_completed_probe={parsed.get('age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe', '-')} "
        f"age5_gate0_family_transport_contract_progress={parsed.get('age5_full_real_gate0_family_transport_contract_selftest_progress_present', '0')} "
        f"age5_gate0_transport_family_contract_completed={parsed.get('age5_full_real_gate0_transport_family_contract_selftest_completed_checks', '-')} "
        f"age5_gate0_transport_family_contract_total={parsed.get('age5_full_real_gate0_transport_family_contract_selftest_total_checks', '-')} "
        f"age5_gate0_transport_family_contract_checks_text={parsed.get('age5_full_real_gate0_transport_family_contract_selftest_checks_text', '-')} "
        f"age5_gate0_transport_family_contract_current_probe={parsed.get('age5_full_real_gate0_transport_family_contract_selftest_current_probe', '-')} "
        f"age5_gate0_transport_family_contract_last_completed_probe={parsed.get('age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe', '-')} "
        f"age5_gate0_transport_family_contract_progress={parsed.get('age5_full_real_gate0_transport_family_contract_selftest_progress_present', '0')} "
        f"age5_gate0_transport_family_transport_contract_completed={parsed.get('age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks', '-')} "
        f"age5_gate0_transport_family_transport_contract_total={parsed.get('age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks', '-')} "
        f"age5_gate0_transport_family_transport_contract_checks_text={parsed.get('age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text', '-')} "
        f"age5_gate0_transport_family_transport_contract_current_probe={parsed.get('age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe', '-')} "
        f"age5_gate0_transport_family_transport_contract_last_completed_probe={parsed.get('age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe', '-')} "
        f"age5_gate0_transport_family_transport_contract_progress={parsed.get('age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present', '0')} "
        f"age5_lang_surface_family_transport_contract_completed={parsed.get('age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks', '-')} "
        f"age5_lang_surface_family_transport_contract_total={parsed.get('age5_full_real_lang_surface_family_transport_contract_selftest_total_checks', '-')} "
        f"age5_lang_surface_family_transport_contract_checks_text={parsed.get('age5_full_real_lang_surface_family_transport_contract_selftest_checks_text', '-')} "
        f"age5_lang_surface_family_transport_contract_current_probe={parsed.get('age5_full_real_lang_surface_family_transport_contract_selftest_current_probe', '-')} "
        f"age5_lang_surface_family_transport_contract_last_completed_probe={parsed.get('age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe', '-')} "
        f"age5_lang_surface_family_transport_contract_progress={parsed.get('age5_full_real_lang_surface_family_transport_contract_selftest_progress_present', '0')} "
        f"age5_bogae_alias_family_contract_completed={parsed.get('age5_full_real_bogae_alias_family_contract_selftest_completed_checks', '-')} "
        f"age5_bogae_alias_family_contract_total={parsed.get('age5_full_real_bogae_alias_family_contract_selftest_total_checks', '-')} "
        f"age5_bogae_alias_family_contract_checks_text={parsed.get('age5_full_real_bogae_alias_family_contract_selftest_checks_text', '-')} "
        f"age5_bogae_alias_family_contract_current_probe={parsed.get('age5_full_real_bogae_alias_family_contract_selftest_current_probe', '-')} "
        f"age5_bogae_alias_family_contract_last_completed_probe={parsed.get('age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe', '-')} "
        f"age5_bogae_alias_family_contract_progress={parsed.get('age5_full_real_bogae_alias_family_contract_selftest_progress_present', '0')} "
        f"age5_bogae_alias_family_transport_contract_completed={parsed.get('age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks', '-')} "
        f"age5_bogae_alias_family_transport_contract_total={parsed.get('age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks', '-')} "
        f"age5_bogae_alias_family_transport_contract_checks_text={parsed.get('age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text', '-')} "
        f"age5_bogae_alias_family_transport_contract_current_probe={parsed.get('age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe', '-')} "
        f"age5_bogae_alias_family_transport_contract_last_completed_probe={parsed.get('age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe', '-')} "
        f"age5_bogae_alias_family_transport_contract_progress={parsed.get('age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present', '0')} "
        f"reason={parsed.get('reason', '-')}"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Parse ci_gate_final_status_line.txt and print compact status")
    parser.add_argument("--status-line", required=True, help="path to ci_gate_final_status_line.txt")
    parser.add_argument("--gate-index", help="optional path to ci_gate_report_index.detjson for preview enrichment")
    parser.add_argument("--json-out", help="optional parse result detjson path")
    parser.add_argument("--compact-out", help="optional compact one-line txt path")
    parser.add_argument("--fail-on-invalid", action="store_true", help="return non-zero when parse/validation fails")
    parser.add_argument("--fail-on-fail", action="store_true", help="return non-zero when parsed status=fail")
    args = parser.parse_args()

    status_line_path = Path(args.status_line)
    gate_index_path = Path(args.gate_index) if args.gate_index and args.gate_index.strip() else None
    parsed, error = parse_status_line(status_line_path)
    if parsed is None:
        print(f"[ci-gate-final-status-line-parse] invalid reason={error}")
        if args.fail_on_invalid:
            return 1
        return 0

    compact = compact_line(parsed)
    print(f"[ci-gate-final-status-line-parse] {compact}")

    if args.json_out:
        out = Path(args.json_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        parsed_payload = dict(parsed)
        parsed_payload[AGE4_PROOF_FAILED_PREVIEW_KEY] = load_age4_proof_failed_preview(gate_index_path)
        parsed_payload.update(load_age5_policy_snapshot(gate_index_path))
        payload = {
            "schema": "ddn.ci.gate_final_status_line_parse.v1",
            "status_line_path": str(status_line_path),
            "parsed": parsed_payload,
            "compact_line": compact,
        }
        out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    if args.compact_out:
        out = Path(args.compact_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(compact + "\n", encoding="utf-8")

    if args.fail_on_fail and parsed.get("status") != "pass":
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
