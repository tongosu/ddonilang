#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

from _ci_age5_combined_heavy_contract import (
    AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY,
    AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY,
    AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_TEXT_KEY,
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY,
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY,
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_TEXT_KEY,
    AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
    AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_DEFAULT,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS,
    AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_ENV_KEY,
    AGE5_COMBINED_HEAVY_FULL_REAL_SOURCE_TRACE_TEXT,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_MODE,
    AGE5_COMBINED_HEAVY_REPORT_SCHEMA,
    AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA,
    AGE5_COMBINED_HEAVY_REQUIRED_REPORTS,
    AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT,
    AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_DEFAULT,
    AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_KEY,
    AGE5_FULL_REAL_CORE_LANG_SANITY_ELAPSED_FIELDS_TEXT,
    AGE5_FULL_REAL_ELAPSED_FIELDS_TEXT,
    AGE5_FULL_REAL_FIXED64_DARWIN_REAL_REPORT_READINESS_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_ELAPSED_MAP_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_STATUS_MAP_FIELDS_TEXT,
    AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROBE_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_TIMEOUT_BREAKDOWN_FIELDS_TEXT,
    build_age5_combined_heavy_child_summary_fields,
    build_age5_combined_heavy_combined_report_contract_fields,
    build_age5_combined_heavy_child_summary_default_text_transport_fields,
    build_age5_combined_heavy_full_real_source_trace,
    build_age5_combined_heavy_full_real_source_trace_text,
    build_age5_combined_heavy_full_summary_contract_fields,
    build_age5_combined_heavy_full_summary_text_transport_fields,
    build_age5_combined_heavy_timeout_policy_fields,
    build_age4_proof_snapshot,
    build_age4_proof_source_snapshot_fields,
    build_age4_proof_snapshot_text,
    build_age5_close_digest_selftest_default_field,
    build_age5_full_real_core_lang_sanity_elapsed_summary,
    build_age5_full_real_elapsed_summary,
    build_age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress,
    build_age5_full_real_pipeline_emit_flags_selftest_progress,
    build_age5_full_real_pipeline_emit_flags_selftest_probe,
    build_age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress,
    build_age5_full_real_profile_elapsed_map,
    build_age5_full_real_profile_status_map,
    build_age5_full_real_timeout_breakdown,
)


ROOT = Path(__file__).resolve().parent.parent
CONTRACT_SCHEMA = "ddn.age5.combined_heavy_real_report_contract.v1"


def read_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def write_json(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def fail(summary_path: Path, payload: dict[str, object], reason: str) -> int:
    payload["ok"] = False
    payload["status"] = "fail"
    payload["reason"] = reason
    write_json(summary_path, payload)
    print(f"[age5-combined-heavy-real-report] fail: {reason}", file=sys.stderr)
    return 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate AGE5 combined heavy real report contract.")
    parser.add_argument(
        "--report",
        default="",
        help="AGE5 combined heavy detjson path (default: build/reports/age5_close_report.detjson)",
    )
    parser.add_argument(
        "--json-out",
        default="",
        help="contract detjson output path (default: build/reports/age5_combined_heavy_real_report_contract.detjson)",
    )
    parser.add_argument(
        "--allow-overall-fail",
        action="store_true",
        help="allow overall_ok=false while still validating report schema/contract",
    )
    args = parser.parse_args()

    report_path = (
        Path(args.report).resolve()
        if args.report.strip()
        else (ROOT / "build" / "reports" / "age5_close_report.detjson").resolve()
    )
    summary_path = (
        Path(args.json_out).resolve()
        if args.json_out.strip()
        else (ROOT / "build" / "reports" / "age5_combined_heavy_real_report_contract.detjson").resolve()
    )

    payload: dict[str, object] = {
        "schema": CONTRACT_SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": False,
        "status": "fail",
        "reason": "-",
        "report_path": str(report_path),
        "allow_overall_fail": bool(args.allow_overall_fail),
    }

    doc = read_json(report_path)
    if not isinstance(doc, dict):
        return fail(summary_path, payload, "report missing or invalid json")
    if str(doc.get("schema", "")).strip() != AGE5_COMBINED_HEAVY_REPORT_SCHEMA:
        return fail(summary_path, payload, "report schema mismatch")
    if bool(doc.get("with_combined_heavy_runtime_helper_check", False)) is not True:
        return fail(summary_path, payload, "with_combined_heavy_runtime_helper_check mismatch")
    if not args.allow_overall_fail and bool(doc.get("overall_ok", False)) is not True:
        return fail(summary_path, payload, "overall_ok mismatch")

    expected_contract = build_age5_combined_heavy_combined_report_contract_fields()
    expected_full_summary_contract = build_age5_combined_heavy_full_summary_contract_fields()
    expected_full_summary_transport = build_age5_combined_heavy_full_summary_text_transport_fields()
    expected_child_summary_default_transport = (
        build_age5_combined_heavy_child_summary_default_text_transport_fields()
    )
    expected_full_real_source_trace = build_age5_combined_heavy_full_real_source_trace(
        smoke_check_script_exists=True,
        smoke_check_selftest_script_exists=True,
    )
    expected_digest_default_field = build_age5_close_digest_selftest_default_field()
    criteria = doc.get("criteria")
    if not isinstance(criteria, list):
        return fail(summary_path, payload, "criteria missing or invalid")
    criteria_ok = {
        str(row.get("name", "")).strip(): bool(row.get("ok", False))
        for row in criteria
        if isinstance(row, dict)
    }
    expected_child_summary = build_age5_combined_heavy_child_summary_fields(
        full_real_ok=criteria_ok.get("age5_ci_profile_matrix_full_real_smoke_optin_pass", False),
        runtime_helper_negative_ok=criteria_ok.get(
            "age5_ci_profile_core_lang_runtime_helper_negative_optin_pass", False
        ),
        group_id_summary_negative_ok=criteria_ok.get(
            "age5_ci_profile_core_lang_group_id_summary_negative_optin_pass", False
        ),
    )
    expected_full_real_elapsed_summary = build_age5_full_real_elapsed_summary(
        age5_full_real_total_elapsed_ms=doc.get("age5_full_real_total_elapsed_ms", "-"),
        age5_full_real_slowest_profile=doc.get("age5_full_real_slowest_profile", "-"),
        age5_full_real_slowest_elapsed_ms=doc.get("age5_full_real_slowest_elapsed_ms", "-"),
        age5_full_real_elapsed_present=str(doc.get("age5_full_real_elapsed_present", "0")).strip() == "1",
    )
    expected_full_real_core_lang_sanity_elapsed_summary = build_age5_full_real_core_lang_sanity_elapsed_summary(
        age5_full_real_core_lang_sanity_total_elapsed_ms=doc.get(
            "age5_full_real_core_lang_sanity_total_elapsed_ms", "-"
        ),
        age5_full_real_core_lang_sanity_slowest_step=doc.get(
            "age5_full_real_core_lang_sanity_slowest_step", "-"
        ),
        age5_full_real_core_lang_sanity_slowest_elapsed_ms=doc.get(
            "age5_full_real_core_lang_sanity_slowest_elapsed_ms", "-"
        ),
        age5_full_real_core_lang_sanity_elapsed_present=(
            str(doc.get("age5_full_real_core_lang_sanity_elapsed_present", "0")).strip() == "1"
        ),
    )
    expected_full_real_profile_elapsed_map = build_age5_full_real_profile_elapsed_map(
        age5_full_real_profile_elapsed_map=doc.get("age5_full_real_profile_elapsed_map", "-"),
        age5_full_real_profile_elapsed_map_present=(
            str(doc.get("age5_full_real_profile_elapsed_map_present", "0")).strip() == "1"
        ),
    )
    expected_full_real_profile_status_map = build_age5_full_real_profile_status_map(
        age5_full_real_profile_status_map=doc.get("age5_full_real_profile_status_map", "-"),
        age5_full_real_profile_status_map_present=(
            str(doc.get("age5_full_real_profile_status_map_present", "0")).strip() == "1"
        ),
    )
    expected_full_real_pipeline_emit_flags_selftest_progress = (
        build_age5_full_real_pipeline_emit_flags_selftest_progress(
            age5_full_real_pipeline_emit_flags_selftest_current_case=doc.get(
                "age5_full_real_pipeline_emit_flags_selftest_current_case", "-"
            ),
            age5_full_real_pipeline_emit_flags_selftest_last_completed_case=doc.get(
                "age5_full_real_pipeline_emit_flags_selftest_last_completed_case", "-"
            ),
            age5_full_real_pipeline_emit_flags_selftest_total_elapsed_ms=doc.get(
                "age5_full_real_pipeline_emit_flags_selftest_total_elapsed_ms", "-"
            ),
            age5_full_real_pipeline_emit_flags_selftest_progress_present=(
                str(doc.get("age5_full_real_pipeline_emit_flags_selftest_progress_present", "0")).strip()
                == "1"
            ),
        )
    )
    expected_full_real_pipeline_emit_flags_selftest_probe = (
        build_age5_full_real_pipeline_emit_flags_selftest_probe(
            age5_full_real_pipeline_emit_flags_selftest_current_probe=doc.get(
                "age5_full_real_pipeline_emit_flags_selftest_current_probe", "-"
            ),
            age5_full_real_pipeline_emit_flags_selftest_last_completed_probe=doc.get(
                "age5_full_real_pipeline_emit_flags_selftest_last_completed_probe", "-"
            ),
            age5_full_real_pipeline_emit_flags_selftest_probe_present=(
                str(doc.get("age5_full_real_pipeline_emit_flags_selftest_probe_present", "0")).strip()
                == "1"
            ),
        )
    )
    expected_full_real_profile_matrix_full_real_smoke_check_selftest_progress = (
        build_age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress(
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_case=doc.get(
                "age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_case", "-"
            ),
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_case=doc.get(
                "age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_case", "-"
            ),
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_total_elapsed_ms=doc.get(
                "age5_full_real_profile_matrix_full_real_smoke_check_selftest_total_elapsed_ms", "-"
            ),
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_probe=doc.get(
                "age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_probe", "-"
            ),
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_probe=doc.get(
                "age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_probe", "-"
            ),
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_present=(
                str(doc.get("age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_present", "0")).strip()
                == "1"
            ),
        )
    )
    expected_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress = (
        build_age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress(
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_case=doc.get(
                "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_case", "-"
            ),
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_case=doc.get(
                "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_case", "-"
            ),
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_total_elapsed_ms=doc.get(
                "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_total_elapsed_ms", "-"
            ),
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_probe=doc.get(
                "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_probe", "-"
            ),
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_probe=doc.get(
                "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_probe", "-"
            ),
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_present=(
                str(
                    doc.get(
                        "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_present",
                        "0",
                    )
                ).strip()
                == "1"
            ),
        )
    )
    timeout_targets_text = str(doc.get("age5_combined_heavy_timeout_targets", "-")).strip() or "-"
    full_real_timeout_present = "1" if "full_real" in timeout_targets_text.split(",") else "0"
    full_real_timeout_step = str(doc.get("age5_full_real_timeout_step", "-")).strip() or "-"
    full_real_timeout_profiles = str(doc.get("age5_full_real_timeout_profiles", "-")).strip() or "-"
    expected_full_real_timeout_breakdown = build_age5_full_real_timeout_breakdown(
        age5_full_real_timeout_step=full_real_timeout_step,
        age5_full_real_timeout_profiles=full_real_timeout_profiles,
        age5_full_real_timeout_present=(full_real_timeout_present == "1"),
    )
    if str(doc.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip() != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
        return fail(summary_path, payload, "top-level combined_digest_selftest_default_field_text mismatch")
    if dict(doc.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_KEY, {})) != expected_digest_default_field:
        return fail(summary_path, payload, "top-level combined_digest_selftest_default_field mismatch")
    if str(doc.get("age4_proof_snapshot_fields_text", "")).strip() != AGE4_PROOF_SNAPSHOT_FIELDS_TEXT:
        return fail(summary_path, payload, "top-level age4_proof_snapshot_fields_text mismatch")
    actual_age4_proof_snapshot = build_age4_proof_snapshot(
        age4_proof_ok=doc.get("age4_proof_ok", "0"),
        age4_proof_failed_criteria=doc.get("age4_proof_failed_criteria", "-1"),
        age4_proof_failed_preview=doc.get("age4_proof_failed_preview", "-"),
    )
    if str(doc.get("age4_proof_snapshot_text", "")).strip() != build_age4_proof_snapshot_text(actual_age4_proof_snapshot):
        return fail(summary_path, payload, "top-level age4_proof_snapshot_text mismatch")
    for key, expected in actual_age4_proof_snapshot.items():
        if str(doc.get(key, "")).strip() != str(expected):
            return fail(summary_path, payload, f"age4 proof snapshot mismatch: {key}")
    top_age4_proof_text = build_age4_proof_snapshot_text(actual_age4_proof_snapshot)
    gate_age4_proof_text = str(
        doc.get(AGE4_PROOF_GATE_RESULT_SNAPSHOT_TEXT_KEY, top_age4_proof_text)
    ).strip() or top_age4_proof_text
    gate_age4_proof_present = str(
        doc.get(AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY, "0")
    ).strip() or "0"
    final_age4_proof_text = str(
        doc.get(AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_TEXT_KEY, top_age4_proof_text)
    ).strip() or top_age4_proof_text
    final_age4_proof_present = str(
        doc.get(AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY, "0")
    ).strip() or "0"
    expected_age4_proof_source_fields = {
        AGE4_PROOF_GATE_RESULT_SNAPSHOT_TEXT_KEY: gate_age4_proof_text,
        AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY: gate_age4_proof_present,
        AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY: (
            "1" if gate_age4_proof_present == "1" and gate_age4_proof_text == top_age4_proof_text else "0"
        ),
        AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_TEXT_KEY: final_age4_proof_text,
        AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY: final_age4_proof_present,
        AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY: (
            "1" if final_age4_proof_present == "1" and final_age4_proof_text == top_age4_proof_text else "0"
        ),
    }
    for key, expected in expected_age4_proof_source_fields.items():
        if str(doc.get(key, "")).strip() != str(expected):
            return fail(summary_path, payload, f"age4 proof source mismatch: {key}")
    if dict(doc.get("full_real_source_trace", {})) != expected_full_real_source_trace:
        return fail(summary_path, payload, "top-level full_real_source_trace mismatch")
    if str(doc.get("full_real_source_trace_text", "")).strip() != (
        build_age5_combined_heavy_full_real_source_trace_text(expected_full_real_source_trace)
    ):
        return fail(summary_path, payload, "top-level full_real_source_trace_text mismatch")
    if str(doc.get("age5_combined_heavy_timeout_policy_ok", "")).strip() != "1":
        return fail(summary_path, payload, "top-level age5_combined_heavy_timeout_policy_ok mismatch")
    if str(doc.get(AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_KEY, "")).strip() != (
        AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT
    ):
        return fail(summary_path, payload, "top-level combined timeout policy reason mismatch")
    if str(doc.get("age5_full_real_elapsed_fields_text", "")).strip() != (
        AGE5_FULL_REAL_ELAPSED_FIELDS_TEXT
    ):
        return fail(summary_path, payload, "top-level age5_full_real_elapsed_fields_text mismatch")
    for key, expected in expected_full_real_elapsed_summary.items():
        if str(doc.get(key, "")).strip() != str(expected):
            return fail(summary_path, payload, f"top-level full_real elapsed summary mismatch: {key}")
    if str(doc.get("age5_full_real_elapsed_present", "0")).strip() == "1":
        if str(doc.get("age5_full_real_total_elapsed_ms", "-")).strip() == "-":
            return fail(summary_path, payload, "top-level full_real total elapsed missing")
        if str(doc.get("age5_full_real_slowest_profile", "-")).strip() == "-":
            return fail(summary_path, payload, "top-level full_real slowest profile missing")
        if str(doc.get("age5_full_real_slowest_elapsed_ms", "-")).strip() == "-":
            return fail(summary_path, payload, "top-level full_real slowest elapsed missing")
    if str(doc.get("age5_full_real_core_lang_sanity_elapsed_fields_text", "")).strip() != (
        AGE5_FULL_REAL_CORE_LANG_SANITY_ELAPSED_FIELDS_TEXT
    ):
        return fail(summary_path, payload, "top-level age5_full_real_core_lang_sanity_elapsed_fields_text mismatch")
    for key, expected in expected_full_real_core_lang_sanity_elapsed_summary.items():
        if str(doc.get(key, "")).strip() != str(expected):
            return fail(summary_path, payload, f"top-level full_real core_lang sanity elapsed mismatch: {key}")
    if str(doc.get("age5_full_real_core_lang_sanity_elapsed_present", "0")).strip() == "1":
        if str(doc.get("age5_full_real_core_lang_sanity_total_elapsed_ms", "-")).strip() == "-":
            return fail(summary_path, payload, "top-level full_real core_lang sanity total elapsed missing")
        if str(doc.get("age5_full_real_core_lang_sanity_slowest_step", "-")).strip() == "-":
            return fail(summary_path, payload, "top-level full_real core_lang sanity slowest step missing")
        if str(doc.get("age5_full_real_core_lang_sanity_slowest_elapsed_ms", "-")).strip() == "-":
            return fail(summary_path, payload, "top-level full_real core_lang sanity slowest elapsed missing")
    if str(doc.get("age5_full_real_profile_elapsed_map_fields_text", "")).strip() != (
        AGE5_FULL_REAL_PROFILE_ELAPSED_MAP_FIELDS_TEXT
    ):
        return fail(summary_path, payload, "top-level age5_full_real_profile_elapsed_map_fields_text mismatch")
    for key, expected in expected_full_real_profile_elapsed_map.items():
        if str(doc.get(key, "")).strip() != str(expected):
            return fail(summary_path, payload, f"top-level full_real profile elapsed map mismatch: {key}")
    if str(doc.get("age5_full_real_profile_elapsed_map_present", "0")).strip() == "1":
        if str(doc.get("age5_full_real_profile_elapsed_map", "-")).strip() == "-":
            return fail(summary_path, payload, "top-level full_real profile elapsed map missing")
    if str(doc.get("age5_full_real_profile_status_map_fields_text", "")).strip() != (
        AGE5_FULL_REAL_PROFILE_STATUS_MAP_FIELDS_TEXT
    ):
        return fail(summary_path, payload, "top-level age5_full_real_profile_status_map_fields_text mismatch")
    for key, expected in expected_full_real_profile_status_map.items():
        if str(doc.get(key, "")).strip() != str(expected):
            return fail(summary_path, payload, f"top-level full_real profile status map mismatch: {key}")
    if str(doc.get("age5_full_real_profile_status_map_present", "0")).strip() == "1":
        if str(doc.get("age5_full_real_profile_status_map", "-")).strip() == "-":
            return fail(summary_path, payload, "top-level full_real profile status map missing")
    if str(doc.get("age5_full_real_pipeline_emit_flags_selftest_progress_fields_text", "")).strip() != (
        AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROGRESS_FIELDS_TEXT
    ):
        return fail(summary_path, payload, "top-level age5_full_real_pipeline_emit_flags_selftest_progress_fields_text mismatch")
    for key, expected in expected_full_real_pipeline_emit_flags_selftest_progress.items():
        if str(doc.get(key, "")).strip() != str(expected):
            return fail(summary_path, payload, f"top-level full_real pipeline emit flags selftest progress mismatch: {key}")
    if str(doc.get("age5_full_real_pipeline_emit_flags_selftest_progress_present", "0")).strip() == "1":
        if str(doc.get("age5_full_real_pipeline_emit_flags_selftest_total_elapsed_ms", "-")).strip() == "-":
            return fail(summary_path, payload, "top-level full_real pipeline emit flags selftest elapsed missing")
    if str(doc.get("age5_full_real_pipeline_emit_flags_selftest_probe_fields_text", "")).strip() != (
        AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROBE_FIELDS_TEXT
    ):
        return fail(summary_path, payload, "top-level age5_full_real_pipeline_emit_flags_selftest_probe_fields_text mismatch")
    for key, expected in expected_full_real_pipeline_emit_flags_selftest_probe.items():
        if str(doc.get(key, "")).strip() != str(expected):
            return fail(summary_path, payload, f"top-level full_real pipeline emit flags selftest probe mismatch: {key}")
    if str(doc.get("age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_fields_text", "")).strip() != (
        AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT
    ):
        return fail(
            summary_path,
            payload,
            "top-level age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_fields_text mismatch",
        )
    for key, expected in expected_full_real_profile_matrix_full_real_smoke_check_selftest_progress.items():
        if str(doc.get(key, "")).strip() != str(expected):
            return fail(
                summary_path,
                payload,
                f"top-level full_real profile_matrix_full_real_smoke_check_selftest mismatch: {key}",
            )
    if str(doc.get("age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_present", "0")).strip() == "1":
        if str(doc.get("age5_full_real_profile_matrix_full_real_smoke_check_selftest_total_elapsed_ms", "-")).strip() == "-":
            return fail(
                summary_path,
                payload,
                "top-level full_real profile_matrix_full_real_smoke_check_selftest elapsed missing",
            )
    if str(
        doc.get("age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_fields_text", "")
    ).strip() != AGE5_FULL_REAL_FIXED64_DARWIN_REAL_REPORT_READINESS_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(
            summary_path,
            payload,
            "top-level age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_fields_text mismatch",
        )
    for key, expected in expected_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress.items():
        if str(doc.get(key, "")).strip() != str(expected):
            return fail(
                summary_path,
                payload,
                f"top-level full_real fixed64_darwin_real_report_readiness_check_selftest mismatch: {key}",
            )
    if str(
        doc.get("age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_present", "0")
    ).strip() == "1":
        if str(
            doc.get("age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_total_elapsed_ms", "-")
        ).strip() == "-":
            return fail(
                summary_path,
                payload,
                "top-level full_real fixed64_darwin_real_report_readiness_check_selftest elapsed missing",
            )
    if str(doc.get("age5_full_real_timeout_breakdown_fields_text", "")).strip() != (
        AGE5_FULL_REAL_TIMEOUT_BREAKDOWN_FIELDS_TEXT
    ):
        return fail(summary_path, payload, "top-level age5_full_real_timeout_breakdown_fields_text mismatch")
    for key, expected in expected_full_real_timeout_breakdown.items():
        if str(doc.get(key, "")).strip() != str(expected):
            return fail(summary_path, payload, f"top-level full_real timeout breakdown mismatch: {key}")
    if full_real_timeout_present == "1":
        if full_real_timeout_step == "-":
            return fail(summary_path, payload, "top-level full_real timeout step missing")
        if full_real_timeout_profiles == "-":
            return fail(summary_path, payload, "top-level full_real timeout profiles missing")
    for key, expected in expected_contract.items():
        if str(doc.get(key, "")).strip() != str(expected):
            return fail(summary_path, payload, f"contract field mismatch: {key}")
    for key, expected in expected_full_summary_contract.items():
        if str(doc.get(key, "")).strip() != str(expected):
            return fail(summary_path, payload, f"full summary contract field mismatch: {key}")
    for key, expected in expected_full_summary_transport.items():
        if str(doc.get(key, "")).strip() != str(expected):
            return fail(summary_path, payload, f"full summary transport field mismatch: {key}")
    for key, expected in expected_child_summary_default_transport.items():
        if str(doc.get(key, "")).strip() != str(expected):
            return fail(summary_path, payload, f"child summary default transport field mismatch: {key}")
    for key, expected in expected_child_summary.items():
        if str(doc.get(key, "")).strip() != str(expected):
            return fail(summary_path, payload, f"child summary field mismatch: {key}")

    policy_contract = doc.get("policy_contract")
    if not isinstance(policy_contract, dict):
        return fail(summary_path, payload, "policy_contract missing")
    if str(policy_contract.get("env_key", "")).strip() != AGE5_COMBINED_HEAVY_ENV_KEY:
        return fail(summary_path, payload, "policy_contract.env_key mismatch")
    if str(policy_contract.get("scope", "")).strip() != AGE5_COMBINED_HEAVY_MODE:
        return fail(summary_path, payload, "policy_contract.scope mismatch")
    if str(policy_contract.get("combined_report_schema", "")).strip() != AGE5_COMBINED_HEAVY_REPORT_SCHEMA:
        return fail(summary_path, payload, "policy_contract.combined_report_schema mismatch")
    if str(policy_contract.get("full_real_source_trace_text", "")).strip() != AGE5_COMBINED_HEAVY_FULL_REAL_SOURCE_TRACE_TEXT:
        return fail(summary_path, payload, "policy_contract.full_real_source_trace_text mismatch")
    if str(policy_contract.get("age4_proof_snapshot_fields_text", "")).strip() != AGE4_PROOF_SNAPSHOT_FIELDS_TEXT:
        return fail(summary_path, payload, "policy_contract.age4_proof_snapshot_fields_text mismatch")
    expected_policy_age4_proof_snapshot = build_age4_proof_snapshot()
    expected_policy_age4_proof_source_fields = build_age4_proof_source_snapshot_fields(
        top_snapshot=expected_policy_age4_proof_snapshot
    )
    if str(policy_contract.get("age4_proof_snapshot_text", "")).strip() != (
        build_age4_proof_snapshot_text(expected_policy_age4_proof_snapshot)
    ):
        return fail(summary_path, payload, "policy_contract.age4_proof_snapshot_text mismatch")
    if str(policy_contract.get("age4_proof_source_snapshot_fields_text", "")).strip() != (
        AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT
    ):
        return fail(summary_path, payload, "policy_contract.age4_proof_source_snapshot_fields_text mismatch")
    for key, expected in expected_policy_age4_proof_snapshot.items():
        if str(policy_contract.get(key, "")).strip() != str(expected):
            return fail(summary_path, payload, f"policy_contract.age4_proof_snapshot mismatch: {key}")
    for key, expected in expected_policy_age4_proof_source_fields.items():
        if str(policy_contract.get(key, "")).strip() != str(expected):
            return fail(summary_path, payload, f"policy_contract.age4_proof_source mismatch: {key}")
    if str(policy_contract.get("full_real_smoke_check_script", "")).strip() != expected_full_real_source_trace["smoke_check_script"]:
        return fail(summary_path, payload, "policy_contract.full_real_smoke_check_script mismatch")
    if str(policy_contract.get("full_real_smoke_check_selftest_script", "")).strip() != (
        expected_full_real_source_trace["smoke_check_selftest_script"]
    ):
        return fail(summary_path, payload, "policy_contract.full_real_smoke_check_selftest_script mismatch")
    if list(policy_contract.get("combined_required_reports", [])) != list(AGE5_COMBINED_HEAVY_REQUIRED_REPORTS):
        return fail(summary_path, payload, "policy_contract.combined_required_reports mismatch")
    if list(policy_contract.get("combined_required_criteria", [])) != list(AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA):
        return fail(summary_path, payload, "policy_contract.combined_required_criteria mismatch")
    if list(policy_contract.get("combined_child_summary_keys", [])) != list(AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS):
        return fail(summary_path, payload, "policy_contract.combined_child_summary_keys mismatch")
    if dict(policy_contract.get("combined_child_summary_default_fields", {})) != dict(
        AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS
    ):
        return fail(summary_path, payload, "policy_contract.combined_child_summary_default_fields mismatch")
    if str(policy_contract.get("combined_child_summary_default_fields_text", "")).strip() != (
        AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT
    ):
        return fail(summary_path, payload, "policy_contract.combined_child_summary_default_fields_text mismatch")
    if dict(policy_contract.get("combined_timeout_policy_fields", {})) != dict(
        build_age5_combined_heavy_timeout_policy_fields()
    ):
        return fail(summary_path, payload, "policy_contract.combined_timeout_policy_fields mismatch")
    if str(policy_contract.get(AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_KEY, "")).strip() != (
        AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_DEFAULT
    ):
        return fail(summary_path, payload, "policy_contract.combined_timeout_requires_optin mismatch")
    if str(policy_contract.get(AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_KEY, "")).strip() != (
        AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT
    ):
        return fail(summary_path, payload, "policy_contract.combined_timeout_policy_reason mismatch")
    if str(policy_contract.get(AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY, "")).strip() != AGE5_CLOSE_DIGEST_SELFTEST_OK_DEFAULT:
        return fail(summary_path, payload, "policy_contract.age5_close_digest_selftest_ok mismatch")
    if str(policy_contract.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip() != (
        AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT
    ):
        return fail(summary_path, payload, "policy_contract.combined_digest_selftest_default_field_text mismatch")
    if dict(policy_contract.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_KEY, {})) != expected_digest_default_field:
        return fail(summary_path, payload, "policy_contract.combined_digest_selftest_default_field mismatch")
    if dict(policy_contract.get("combined_child_summary_default_text_transport_fields", {})) != (
        expected_child_summary_default_transport
    ):
        return fail(summary_path, payload, "policy_contract.combined_child_summary_default_text_transport_fields mismatch")
    if str(policy_contract.get("combined_child_summary_default_text_transport_fields_text", "")).strip() != (
        AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS_TEXT
    ):
        return fail(summary_path, payload, "policy_contract.combined_child_summary_default_text_transport_fields_text mismatch")
    if dict(policy_contract.get("combined_contract_summary_fields", {})) != expected_contract:
        return fail(summary_path, payload, "policy_contract.combined_contract_summary_fields mismatch")
    if str(policy_contract.get("combined_contract_summary_fields_text", "")).strip() != (
        AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT
    ):
        return fail(summary_path, payload, "policy_contract.combined_contract_summary_fields_text mismatch")
    if dict(policy_contract.get("combined_full_summary_contract_fields", {})) != expected_full_summary_contract:
        return fail(summary_path, payload, "policy_contract.combined_full_summary_contract_fields mismatch")
    if str(policy_contract.get("combined_full_summary_contract_fields_text", "")).strip() != (
        AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT
    ):
        return fail(summary_path, payload, "policy_contract.combined_full_summary_contract_fields_text mismatch")
    if dict(policy_contract.get("combined_full_summary_text_transport_fields", {})) != expected_full_summary_transport:
        return fail(summary_path, payload, "policy_contract.combined_full_summary_text_transport_fields mismatch")
    if str(policy_contract.get("combined_full_summary_text_transport_fields_text", "")).strip() != (
        AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FIELDS_TEXT
    ):
        return fail(summary_path, payload, "policy_contract.combined_full_summary_text_transport_fields_text mismatch")

    criteria_names = [str(row.get("name", "")).strip() for row in criteria if isinstance(row, dict)]
    if criteria_names != list(AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA):
        return fail(summary_path, payload, "criteria names mismatch")

    reports = doc.get("reports")
    if not isinstance(reports, dict):
        return fail(summary_path, payload, "reports missing or invalid")
    if list(reports.keys()) != list(AGE5_COMBINED_HEAVY_REQUIRED_REPORTS):
        return fail(summary_path, payload, "report keys mismatch")
    for key in AGE5_COMBINED_HEAVY_REQUIRED_REPORTS:
        value = str(reports.get(key, "")).strip()
        if not value:
            return fail(summary_path, payload, f"report path missing: {key}")

    payload["ok"] = True
    payload["status"] = "pass"
    payload["reason"] = "-"
    payload["overall_ok"] = bool(doc.get("overall_ok", False))
    payload["combined_heavy_env_enabled"] = bool(doc.get("combined_heavy_env_enabled", False))
    payload["criteria_names"] = criteria_names
    payload["report_keys"] = list(reports.keys())
    write_json(summary_path, payload)
    print(f"[age5-combined-heavy-real-report] ok report={report_path} summary={summary_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
