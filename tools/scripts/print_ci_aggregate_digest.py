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
    AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY,
    AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY,
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY,
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY,
    AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
    AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
    build_age4_proof_snapshot,
    build_age4_proof_source_snapshot_fields,
    build_age4_proof_snapshot_text,
    build_age5_combined_heavy_policy_origin_trace,
    build_age5_combined_heavy_policy_origin_trace_text,
    build_age5_close_digest_selftest_default_field,
    build_age5_combined_heavy_child_summary_default_text_transport_fields,
    build_age5_combined_heavy_full_real_source_trace,
)

# diagnostics token anchors:
# combined_digest_selftest_default_field_text=
# combined_digest_selftest_default_field=
# age5_policy_combined_digest_selftest_default_field_text=
# age5_policy_combined_digest_selftest_default_field=
# age5_policy_summary=


def load_payload(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def clip(text: str, limit: int = 160) -> str:
    normalized = " ".join(text.split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


def resolve_path(base_report: Path, raw: str) -> Path:
    candidate = Path(raw)
    if candidate.is_absolute():
        return candidate
    # 기본은 현재 작업 디렉터리 기준, 없으면 aggregate report 기준으로 재해석.
    if candidate.exists():
        return candidate
    if candidate.parent != Path("."):
        return candidate.resolve()
    return (base_report.parent / candidate).resolve()


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


def main() -> int:
    parser = argparse.ArgumentParser(description="Print top digest lines from ci_aggregate_report.detjson")
    parser.add_argument("report", help="path to ci_aggregate_report.detjson")
    parser.add_argument("--top", type=int, default=1, help="number of digest lines to print")
    parser.add_argument("--only-failed", action="store_true", help="print digest only when overall_ok=false")
    parser.add_argument(
        "--show-steps",
        action="store_true",
        help="print failed step names from gate index when available",
    )
    args = parser.parse_args()

    path = Path(args.report)
    payload = load_payload(path)
    if payload is None:
        print(f"[ci-aggregate] report missing_or_invalid: {path}")
        return 0

    overall_ok = bool(payload.get("overall_ok", False))
    digest_raw = payload.get("failure_digest")
    digest = [str(item) for item in digest_raw] if isinstance(digest_raw, list) else []
    top = max(1, int(args.top))

    seamgrim = payload.get("seamgrim") if isinstance(payload.get("seamgrim"), dict) else {}
    age3 = payload.get("age3") if isinstance(payload.get("age3"), dict) else {}
    age4 = payload.get("age4") if isinstance(payload.get("age4"), dict) else {}
    age5 = payload.get("age5") if isinstance(payload.get("age5"), dict) else {}
    oi = payload.get("oi405_406") if isinstance(payload.get("oi405_406"), dict) else {}
    seamgrim_failed = len(seamgrim.get("failed_steps", [])) if isinstance(seamgrim, dict) else 0
    age3_failed = len(age3.get("failed_criteria", [])) if isinstance(age3, dict) else 0
    age4_failed = len(age4.get("failed_criteria", [])) if isinstance(age4, dict) else 0
    age4_proof_ok = int(bool(age4.get("proof_artifact_ok", False))) if isinstance(age4, dict) else 0
    age4_proof_failed = len(age4.get("proof_artifact_failed_criteria", [])) if isinstance(age4, dict) and isinstance(age4.get("proof_artifact_failed_criteria"), list) else 0
    age4_proof_failed_preview = (
        str(age4.get("proof_artifact_failed_preview", "")).strip()
        if isinstance(age4, dict)
        else ""
    ) or format_age4_proof_failed_preview(
        age4.get("proof_artifact_failed_criteria", []) if isinstance(age4, dict) else []
    )
    age5_failed = len(age5.get("failed_criteria", [])) if isinstance(age5, dict) else 0
    oi_failed = len(oi.get("failed_packs", [])) if isinstance(oi, dict) else 0
    age5_full_real_status = str(age5.get("age5_combined_heavy_full_real_status", "skipped")).strip() or "skipped"
    age5_full_real_source_trace = age5.get("full_real_source_trace")
    if not isinstance(age5_full_real_source_trace, dict):
        age5_full_real_source_trace = build_age5_combined_heavy_full_real_source_trace()
    age5_full_real_source_check = (
        str(age5_full_real_source_trace.get("smoke_check_script_exists", "0")).strip() or "0"
    )
    age5_full_real_source_selftest = (
        str(age5_full_real_source_trace.get("smoke_check_selftest_script_exists", "0")).strip() or "0"
    )
    age5_runtime_helper_negative_status = (
        str(age5.get("age5_combined_heavy_runtime_helper_negative_status", "skipped")).strip() or "skipped"
    )
    age5_group_id_summary_negative_status = (
        str(age5.get("age5_combined_heavy_group_id_summary_negative_status", "skipped")).strip() or "skipped"
    )
    age5_combined_heavy_child_timeout_sec = (
        str(age5.get("combined_heavy_child_timeout_sec", "0")).strip() or "0"
    )
    age5_combined_heavy_timeout_mode = (
        str(age5.get(AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY, AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED)).strip()
        or AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED
    )
    age5_combined_heavy_timeout_present = (
        str(age5.get("age5_combined_heavy_timeout_present", "0")).strip() or "0"
    )
    age5_combined_heavy_timeout_targets = (
        str(age5.get("age5_combined_heavy_timeout_targets", "-")).strip() or "-"
    )
    age5_close_digest_selftest_ok = str(age5.get("age5_close_digest_selftest_ok", "0")).strip() or "0"
    age5_digest_selftest_default_text = (
        str(age5.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT)).strip()
        or AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT
    )
    age5_digest_selftest_default_field = age5.get("combined_digest_selftest_default_field")
    if not isinstance(age5_digest_selftest_default_field, dict):
        age5_digest_selftest_default_field = build_age5_close_digest_selftest_default_field()
    age5_policy_digest_selftest_default_text = (
        str(age5.get("age5_policy_combined_digest_selftest_default_field_text", AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT)).strip()
        or AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT
    )
    age5_policy_digest_selftest_default_field = age5.get("age5_policy_combined_digest_selftest_default_field")
    if not isinstance(age5_policy_digest_selftest_default_field, dict):
        age5_policy_digest_selftest_default_field = build_age5_close_digest_selftest_default_field()
    age5_policy_age4_proof_snapshot_fields_text = (
        str(age5.get("age5_policy_age4_proof_snapshot_fields_text", AGE4_PROOF_SNAPSHOT_FIELDS_TEXT)).strip()
        or AGE4_PROOF_SNAPSHOT_FIELDS_TEXT
    )
    age5_policy_age4_proof_source_snapshot_fields_text = (
        str(age5.get("age5_policy_age4_proof_source_snapshot_fields_text", AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT)).strip()
        or AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT
    )
    age5_policy_age4_proof_snapshot_text = (
        str(age5.get("age5_policy_age4_proof_snapshot_text", "")).strip()
        or build_age4_proof_snapshot_text(build_age4_proof_snapshot())
    )
    age5_policy_age4_proof_gate_result_present = (
        str(age5.get("age5_policy_age4_proof_gate_result_present", "0")).strip() or "0"
    )
    age5_policy_age4_proof_gate_result_parity = (
        str(age5.get("age5_policy_age4_proof_gate_result_parity", "0")).strip() or "0"
    )
    age5_policy_age4_proof_final_status_parse_present = (
        str(age5.get("age5_policy_age4_proof_final_status_parse_present", "0")).strip() or "0"
    )
    age5_policy_age4_proof_final_status_parse_parity = (
        str(age5.get("age5_policy_age4_proof_final_status_parse_parity", "0")).strip() or "0"
    )
    age5_policy_report = str(age5.get("age5_combined_heavy_policy_report_path", "-")).strip() or "-"
    age5_policy_report_exists = int(bool(age5.get("age5_combined_heavy_policy_report_exists", False)))
    age5_policy_text = str(age5.get("age5_combined_heavy_policy_text_path", "-")).strip() or "-"
    age5_policy_text_exists = int(bool(age5.get("age5_combined_heavy_policy_text_exists", False)))
    age5_policy_summary = str(age5.get("age5_combined_heavy_policy_summary_path", "-")).strip() or "-"
    age5_policy_summary_exists = int(bool(age5.get("age5_combined_heavy_policy_summary_exists", False)))
    age5_policy_origin_trace_contract_status = (
        str(age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY, "ok")).strip() or "ok"
    )
    age5_policy_origin_trace_contract_issue = (
        str(age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY, "-")).strip() or "-"
    )
    age5_policy_origin_trace_contract_source_issue = (
        str(age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY, "-")).strip() or "-"
    )
    age5_policy_origin_trace_contract_compact_reason = (
        str(age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY, "-")).strip() or "-"
    )
    age5_policy_origin_trace_contract_compact_failure_reason = (
        str(age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY, "-")).strip()
        or "-"
    )
    age5_policy_origin_trace_contract_ok = int(
        bool(age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY, False))
    )
    age5_policy_origin_trace = age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY)
    if not isinstance(age5_policy_origin_trace, dict):
        age5_policy_origin_trace = build_age5_combined_heavy_policy_origin_trace(
            report_path=age5_policy_report,
            report_exists=age5_policy_report_exists,
            text_path=age5_policy_text,
            text_exists=age5_policy_text_exists,
            summary_path=age5_policy_summary,
            summary_exists=age5_policy_summary_exists,
        )
    age5_policy_origin_trace_text = (
        str(
            age5.get(
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY,
                build_age5_combined_heavy_policy_origin_trace_text(age5_policy_origin_trace),
            )
        ).strip()
        or build_age5_combined_heavy_policy_origin_trace_text(age5_policy_origin_trace)
    )
    expected_default_transport = build_age5_combined_heavy_child_summary_default_text_transport_fields()
    age5_child_summary_defaults = (
        str(
            age5.get(
                "ci_sanity_age5_combined_heavy_child_summary_default_fields",
                expected_default_transport["ci_sanity_age5_combined_heavy_child_summary_default_fields"],
            )
        ).strip()
        or expected_default_transport["ci_sanity_age5_combined_heavy_child_summary_default_fields"]
    )
    age5_sync_child_summary_defaults = (
        str(
            age5.get(
                "ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields",
                expected_default_transport["ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"],
            )
        ).strip()
        or expected_default_transport["ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields"]
    )
    age5_age4_proof_snapshot_fields_text = (
        str(age5.get("age4_proof_snapshot_fields_text", AGE4_PROOF_SNAPSHOT_FIELDS_TEXT)).strip()
        or AGE4_PROOF_SNAPSHOT_FIELDS_TEXT
    )
    age5_age4_proof_snapshot = build_age4_proof_snapshot(
        age4_proof_ok=age5.get("age4_proof_ok", "0"),
        age4_proof_failed_criteria=age5.get("age4_proof_failed_criteria", "-1"),
        age4_proof_failed_preview=age5.get("age4_proof_failed_preview", "-"),
    )
    age5_age4_proof_snapshot_text = (
        str(age5.get("age4_proof_snapshot_text", "")).strip()
        or build_age4_proof_snapshot_text(age5_age4_proof_snapshot)
    )
    age5_age4_proof_source_fields = build_age4_proof_source_snapshot_fields(
        top_snapshot=age5_age4_proof_snapshot
    )
    age5_age4_proof_gate_result_present = (
        str(
            age5.get(
                AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY,
                age5_age4_proof_source_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY],
            )
        ).strip()
        or age5_age4_proof_source_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY]
    )
    age5_age4_proof_gate_result_parity = (
        str(
            age5.get(
                AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY,
                age5_age4_proof_source_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY],
            )
        ).strip()
        or age5_age4_proof_source_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY]
    )
    age5_age4_proof_final_status_parse_present = (
        str(
            age5.get(
                AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY,
                age5_age4_proof_source_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY],
            )
        ).strip()
        or age5_age4_proof_source_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY]
    )
    age5_age4_proof_final_status_parse_parity = (
        str(
            age5.get(
                AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY,
                age5_age4_proof_source_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY],
            )
        ).strip()
        or age5_age4_proof_source_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY]
    )
    print(
        f"[ci-aggregate] overall_ok={int(overall_ok)} seamgrim_failed={seamgrim_failed} "
        f"age3_failed={age3_failed} age4_failed={age4_failed} "
        f"age4_proof_ok={age4_proof_ok} age4_proof_failed={age4_proof_failed} "
        f"age4_proof_failed_preview={age4_proof_failed_preview} "
        f"age5_failed={age5_failed} "
        f"oi405_406_failed={oi_failed} "
        f"age5_close_digest_selftest_ok={age5_close_digest_selftest_ok} "
        f"{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}={age5_digest_selftest_default_text} "
        f"combined_digest_selftest_default_field={json.dumps(age5_digest_selftest_default_field, ensure_ascii=False, sort_keys=True, separators=(',', ':'))} "
        f"age5_policy_combined_digest_selftest_default_field_text={age5_policy_digest_selftest_default_text} "
        f"age5_policy_combined_digest_selftest_default_field={json.dumps(age5_policy_digest_selftest_default_field, ensure_ascii=False, sort_keys=True, separators=(',', ':'))} "
        f"age5_policy_age4_proof_snapshot_fields_text={age5_policy_age4_proof_snapshot_fields_text} "
        f"age5_policy_age4_proof_source_snapshot_fields_text={age5_policy_age4_proof_source_snapshot_fields_text} "
        f"age5_policy_age4_proof_snapshot_text={age5_policy_age4_proof_snapshot_text} "
        f"age5_policy_age4_proof_gate_result_present={age5_policy_age4_proof_gate_result_present} "
        f"age5_policy_age4_proof_gate_result_parity={age5_policy_age4_proof_gate_result_parity} "
        f"age5_policy_age4_proof_final_status_parse_present={age5_policy_age4_proof_final_status_parse_present} "
        f"age5_policy_age4_proof_final_status_parse_parity={age5_policy_age4_proof_final_status_parse_parity} "
        f"age5_policy_report={age5_policy_report} age5_policy_report_exists={age5_policy_report_exists} "
        f"age5_policy_text={age5_policy_text} age5_policy_text_exists={age5_policy_text_exists} "
        f"age5_policy_summary={age5_policy_summary} age5_policy_summary_exists={age5_policy_summary_exists} "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}={age5_policy_origin_trace_contract_issue} "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}={age5_policy_origin_trace_contract_source_issue} "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}={age5_policy_origin_trace_contract_compact_reason} "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY}={age5_policy_origin_trace_contract_compact_failure_reason} "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}={age5_policy_origin_trace_contract_status} "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}={age5_policy_origin_trace_contract_ok} "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}={age5_policy_origin_trace_text} "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY}={json.dumps(age5_policy_origin_trace, ensure_ascii=False, sort_keys=True, separators=(',', ':'))} "
        f"age5_full_real={age5_full_real_status} "
        f"age5_combined_heavy_child_timeout_sec={age5_combined_heavy_child_timeout_sec} "
        f"age5_combined_heavy_timeout_mode={age5_combined_heavy_timeout_mode} "
        f"age5_combined_heavy_timeout_present={age5_combined_heavy_timeout_present} "
        f"age5_combined_heavy_timeout_targets={age5_combined_heavy_timeout_targets} "
        f"age5_full_real_source_check={age5_full_real_source_check} "
        f"age5_full_real_source_selftest={age5_full_real_source_selftest} "
        f"age5_runtime_helper_negative={age5_runtime_helper_negative_status} "
        f"age5_group_id_summary_negative={age5_group_id_summary_negative_status} "
        f"age5_age4_proof_snapshot_fields_text={age5_age4_proof_snapshot_fields_text} "
        f"age5_age4_proof_snapshot_text={age5_age4_proof_snapshot_text} "
        f"age5_age4_proof_gate_result_present={age5_age4_proof_gate_result_present} "
        f"age5_age4_proof_gate_result_parity={age5_age4_proof_gate_result_parity} "
        f"age5_age4_proof_final_status_parse_present={age5_age4_proof_final_status_parse_present} "
        f"age5_age4_proof_final_status_parse_parity={age5_age4_proof_final_status_parse_parity} "
        f"age5_child_summary_defaults={age5_child_summary_defaults} "
        f"age5_sync_child_summary_defaults={age5_sync_child_summary_defaults} "
        f"report={path}"
    )
    gate_index_raw = payload.get("gate_index_report_path")
    age3_status_raw = payload.get("age3_status_report_path")
    age3_status_line_raw = payload.get("age3_status_line")
    age3_status_line_path_raw = payload.get("age3_status_line_path")
    age3_badge_path_raw = payload.get("age3_badge_path")
    should_print_steps = bool(args.show_steps and (not args.only_failed or not overall_ok))
    gate_steps: list[dict] = []
    if isinstance(gate_index_raw, str) and gate_index_raw.strip():
        gate_index_path = resolve_path(path, gate_index_raw.strip())
        index_doc = load_payload(gate_index_path)
        index_ok = bool(index_doc.get("overall_ok", False)) if isinstance(index_doc, dict) else False
        if isinstance(index_doc, dict) and isinstance(index_doc.get("steps"), list):
            gate_steps = [row for row in index_doc.get("steps", []) if isinstance(row, dict)]
        step_count = len(gate_steps)
        print(
            f"[ci-aggregate] gate_index_path={gate_index_path} "
            f"exists={int(gate_index_path.exists())} index_ok={int(index_ok)} step_count={step_count}"
        )
        if should_print_steps:
            failed_rows = [row for row in gate_steps if not bool(row.get("ok", False))]
            if failed_rows:
                names = ", ".join(str(row.get("name", "-")) for row in failed_rows[:8])
                if len(failed_rows) > 8:
                    names = f"{names}, ..."
                print(f"[ci-aggregate] failed_steps={names}")
            elif gate_steps:
                print("[ci-aggregate] failed_steps=(none)")
            else:
                print("[ci-aggregate] failed_steps=(index_has_no_steps)")
    elif should_print_steps:
        print("[ci-aggregate] failed_steps=(gate_index_missing)")

    if isinstance(age3_status_raw, str) and age3_status_raw.strip():
        age3_status_path = resolve_path(path, age3_status_raw.strip())
        age3_status_doc = load_payload(age3_status_path)
        status_value = (
            str(age3_status_doc.get("status", "-"))
            if isinstance(age3_status_doc, dict)
            else "-"
        )
        status_ok = bool(age3_status_doc.get("overall_ok", False)) if isinstance(age3_status_doc, dict) else False
        print(
            f"[ci-aggregate] age3_status_path={age3_status_path} "
            f"exists={int(age3_status_path.exists())} status={status_value} ok={int(status_ok)}"
        )
    if isinstance(age3_status_line_path_raw, str) and age3_status_line_path_raw.strip():
        age3_status_line_path = resolve_path(path, age3_status_line_path_raw.strip())
        print(
            f"[ci-aggregate] age3_status_line_path={age3_status_line_path} "
            f"exists={int(age3_status_line_path.exists())}"
        )
    if isinstance(age3_status_line_raw, str) and age3_status_line_raw.strip():
        print(f"[ci-aggregate] age3_status_line={clip(age3_status_line_raw, 200)}")
    if isinstance(age3_badge_path_raw, str) and age3_badge_path_raw.strip():
        age3_badge_path = resolve_path(path, age3_badge_path_raw.strip())
        age3_badge_doc = load_payload(age3_badge_path)
        badge_status = str(age3_badge_doc.get("status", "-")) if isinstance(age3_badge_doc, dict) else "-"
        badge_color = str(age3_badge_doc.get("color", "-")) if isinstance(age3_badge_doc, dict) else "-"
        print(
            f"[ci-aggregate] age3_badge_path={age3_badge_path} "
            f"exists={int(age3_badge_path.exists())} status={badge_status} color={badge_color}"
        )

    if args.only_failed and overall_ok:
        return 0

    for idx, line in enumerate(digest[:top], 1):
        print(f" - top{idx}: {clip(line)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
