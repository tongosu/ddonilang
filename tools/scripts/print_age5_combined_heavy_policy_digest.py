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
    AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
    AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY,
    AGE5_COMBINED_HEAVY_MODE,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_TEXT,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DEFAULT_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_DEFAULT,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_SCOPE_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT,
    AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_DEFAULT,
    AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_KEY,
    build_age4_proof_snapshot,
    build_age4_proof_snapshot_text,
    build_age4_proof_source_snapshot_fields,
    build_age5_combined_heavy_policy_origin_trace_contract_compact_reason,
    build_age5_combined_heavy_policy_origin_trace,
    build_age5_combined_heavy_policy_origin_trace_text,
    build_age5_close_digest_selftest_default_field,
)


def load_payload(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def main() -> int:
    parser = argparse.ArgumentParser(description="Print compact digest from age5 combined-heavy policy artifact")
    parser.add_argument("report", help="path to age5_combined_heavy_policy.detjson")
    parser.add_argument(
        "--policy-text",
        default="build/reports/age5_combined_heavy_policy.txt",
        help="path to age5 combined-heavy policy text artifact",
    )
    parser.add_argument(
        "--summary-out",
        default="build/reports/age5_combined_heavy_policy_summary.txt",
        help="path to compact age5 combined-heavy policy summary payload",
    )
    args = parser.parse_args()

    report_path = Path(args.report)
    payload = load_payload(report_path)
    if payload is None:
        print(f"[age5-combined-heavy-policy] report missing_or_invalid: {report_path}", file=sys.stderr)
        return 1

    policy_text_path = Path(args.policy_text)
    summary_out_path = Path(args.summary_out)
    default_field = payload.get("combined_digest_selftest_default_field")
    if not isinstance(default_field, dict):
        default_field = build_age5_close_digest_selftest_default_field()
    default_field_text = (
        str(payload.get("combined_digest_selftest_default_field_text", AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT)).strip()
        or AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT
    )
    age4_proof_snapshot_fields_text = (
        str(payload.get("age4_proof_snapshot_fields_text", AGE4_PROOF_SNAPSHOT_FIELDS_TEXT)).strip()
        or AGE4_PROOF_SNAPSHOT_FIELDS_TEXT
    )
    age4_proof_snapshot = build_age4_proof_snapshot(
        age4_proof_ok=payload.get("age4_proof_ok", "0"),
        age4_proof_failed_criteria=payload.get("age4_proof_failed_criteria", "-1"),
        age4_proof_failed_preview=payload.get("age4_proof_failed_preview", "-"),
    )
    age4_proof_snapshot_text = (
        str(payload.get("age4_proof_snapshot_text", "")).strip()
        or build_age4_proof_snapshot_text(age4_proof_snapshot)
    )
    age4_proof_source_snapshot_fields = build_age4_proof_source_snapshot_fields(
        top_snapshot=age4_proof_snapshot
    )
    age4_proof_source_snapshot_fields_text = (
        str(payload.get("age4_proof_source_snapshot_fields_text", AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT)).strip()
        or AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT
    )
    age4_proof_gate_result_present = (
        str(
            payload.get(
                AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY,
                age4_proof_source_snapshot_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY],
            )
        ).strip()
        or age4_proof_source_snapshot_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY]
    )
    age4_proof_gate_result_parity = (
        str(
            payload.get(
                AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY,
                age4_proof_source_snapshot_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY],
            )
        ).strip()
        or age4_proof_source_snapshot_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY]
    )
    age4_proof_final_status_parse_present = (
        str(
            payload.get(
                AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY,
                age4_proof_source_snapshot_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY],
            )
        ).strip()
        or age4_proof_source_snapshot_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY]
    )
    age4_proof_final_status_parse_parity = (
        str(
            payload.get(
                AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY,
                age4_proof_source_snapshot_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY],
            )
        ).strip()
        or age4_proof_source_snapshot_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY]
    )
    combined_timeout_mode_default = (
        str(payload.get(AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DEFAULT_KEY, AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED)).strip()
        or AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED
    )
    combined_timeout_mode_allowed_values = (
        str(
            payload.get(
                AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_KEY,
                AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_TEXT,
            )
        ).strip()
        or AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_TEXT
    )
    combined_timeout_mode_preview_only = (
        str(
            payload.get(
                AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_KEY,
                AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_DEFAULT,
            )
        ).strip()
        or AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_DEFAULT
    )
    combined_timeout_mode_scope = (
        str(payload.get(AGE5_COMBINED_HEAVY_TIMEOUT_MODE_SCOPE_KEY, AGE5_COMBINED_HEAVY_MODE)).strip()
        or AGE5_COMBINED_HEAVY_MODE
    )
    combined_timeout_requires_optin = (
        str(
            payload.get(
                AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_KEY,
                AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_DEFAULT,
            )
        ).strip()
        or AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_DEFAULT
    )
    combined_timeout_policy_reason = (
        str(
            payload.get(
                AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_KEY,
                AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT,
            )
        ).strip()
        or AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT
    )

    provider = str(payload.get("provider", "-")).strip() or "-"
    enabled = "1" if bool(payload.get("enabled", False)) else "0"
    reason = str(payload.get("reason", "-")).strip() or "-"
    scope = str(payload.get("scope", AGE5_COMBINED_HEAVY_MODE)).strip() or AGE5_COMBINED_HEAVY_MODE
    policy_origin_trace = build_age5_combined_heavy_policy_origin_trace(
        report_path=report_path,
        report_exists=report_path.exists(),
        text_path=policy_text_path,
        text_exists=policy_text_path.exists(),
        summary_path=summary_out_path,
        summary_exists=summary_out_path.exists(),
    )
    policy_origin_trace_text = build_age5_combined_heavy_policy_origin_trace_text(policy_origin_trace)
    compact_reason = build_age5_combined_heavy_policy_origin_trace_contract_compact_reason()

    print(
        f"[age5-combined-heavy-policy] provider={provider} enabled={enabled} reason={reason} scope={scope} "
        f"age5_policy_combined_timeout_mode_default={combined_timeout_mode_default} "
        f"age5_policy_combined_timeout_mode_allowed_values={combined_timeout_mode_allowed_values} "
        f"age5_policy_combined_timeout_mode_preview_only={combined_timeout_mode_preview_only} "
        f"age5_policy_combined_timeout_mode_scope={combined_timeout_mode_scope} "
        f"age5_policy_combined_timeout_requires_optin={combined_timeout_requires_optin} "
        f"age5_policy_combined_timeout_policy_reason={combined_timeout_policy_reason} "
        f"age5_policy_age4_proof_snapshot_fields_text={age4_proof_snapshot_fields_text} "
        f"age5_policy_age4_proof_source_snapshot_fields_text={age4_proof_source_snapshot_fields_text} "
        f"age5_policy_age4_proof_snapshot_text={age4_proof_snapshot_text} "
        f"age5_policy_age4_proof_gate_result_present={age4_proof_gate_result_present} "
        f"age5_policy_age4_proof_gate_result_parity={age4_proof_gate_result_parity} "
        f"age5_policy_age4_proof_final_status_parse_present={age4_proof_final_status_parse_present} "
        f"age5_policy_age4_proof_final_status_parse_parity={age4_proof_final_status_parse_parity} "
        f"age5_policy_combined_digest_selftest_default_field_text={default_field_text} "
        f"age5_policy_combined_digest_selftest_default_field={json.dumps(default_field, ensure_ascii=False, sort_keys=True, separators=(',', ':'))} "
        f"age5_combined_heavy_policy_report_path={report_path} "
        f"age5_combined_heavy_policy_report_exists={int(report_path.exists())} "
        f"age5_combined_heavy_policy_text_path={policy_text_path} "
        f"age5_combined_heavy_policy_text_exists={int(policy_text_path.exists())} "
        f"age5_combined_heavy_policy_summary_path={summary_out_path} "
        f"age5_combined_heavy_policy_summary_exists={int(summary_out_path.exists())} "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}="
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT} "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}="
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT} "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}="
        f"{compact_reason} "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}=ok "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}=1 "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}={policy_origin_trace_text} "
        f"{AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY}="
        f"{json.dumps(policy_origin_trace, ensure_ascii=False, sort_keys=True, separators=(',', ':'))}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
