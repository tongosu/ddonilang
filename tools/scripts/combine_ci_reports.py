#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timezone
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
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
    build_age4_proof_snapshot,
    build_age4_proof_snapshot_text,
    build_age5_combined_heavy_full_real_source_trace,
    build_age5_combined_heavy_full_real_source_trace_text,
    build_age5_combined_heavy_policy_origin_trace_contract_compact_failure_reason,
    build_age5_combined_heavy_policy_origin_trace_contract_compact_reason,
    build_age5_combined_heavy_policy_origin_trace,
    build_age5_combined_heavy_policy_origin_trace_text,
    build_age5_close_digest_selftest_default_field,
    build_age5_combined_heavy_child_summary_default_text_transport_fields,
)


AGE5_CHILD_STATUS_KEYS = (
    "age5_combined_heavy_full_real_status",
    "age5_combined_heavy_runtime_helper_negative_status",
    "age5_combined_heavy_group_id_summary_negative_status",
)
AGE5_CHILD_STATUS_VALUES = {"pass", "fail", "skipped"}
AGE5_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS = (
    build_age5_combined_heavy_child_summary_default_text_transport_fields()
)
AGE5_DIGEST_SELFTEST_DEFAULT_FIELD = build_age5_close_digest_selftest_default_field()
AGE5_POLICY_DIGEST_DEFAULT_FIELD_TEXT_KEY = "age5_policy_combined_digest_selftest_default_field_text"
AGE5_POLICY_DIGEST_DEFAULT_FIELD_KEY = "age5_policy_combined_digest_selftest_default_field"
AGE5_POLICY_SUMMARY_PATH_KEY = "age5_combined_heavy_policy_summary_path"
AGE5_POLICY_SUMMARY_EXISTS_KEY = "age5_combined_heavy_policy_summary_exists"
AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY = "age5_policy_age4_proof_snapshot_fields_text"
AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY = "age5_policy_age4_proof_snapshot_text"
AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY = "age5_policy_age4_proof_source_snapshot_fields_text"
AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY = "age5_policy_age4_proof_gate_result_present"
AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY = "age5_policy_age4_proof_gate_result_parity"
AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY = "age5_policy_age4_proof_final_status_parse_present"
AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY = "age5_policy_age4_proof_final_status_parse_parity"
AGE5_COMBINED_HEAVY_CHILD_TIMEOUT_SEC_KEY = "combined_heavy_child_timeout_sec"
AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY = "age5_combined_heavy_timeout_mode"
AGE5_COMBINED_HEAVY_TIMEOUT_PRESENT_KEY = "age5_combined_heavy_timeout_present"
AGE5_COMBINED_HEAVY_TIMEOUT_TARGETS_KEY = "age5_combined_heavy_timeout_targets"


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def load_text(path: Path) -> str | None:
    if not path.exists():
        return None
    try:
        return path.read_text(encoding="utf-8").strip()
    except Exception:
        return None


def parse_compact_kv_line(text: str | None) -> dict[str, str]:
    if not text:
        return {}
    first_line = ""
    for raw in str(text).splitlines():
        line = str(raw).strip()
        if line:
            first_line = line
            break
    if not first_line:
        return {}
    if first_line.startswith("[") and "]" in first_line:
        first_line = first_line.split("]", 1)[1].strip()
    out: dict[str, str] = {}
    for token in first_line.split():
        if "=" not in token:
            continue
        key, value = token.split("=", 1)
        key = key.strip()
        value = value.strip()
        if key:
            out[key] = value
    return out


def build_age5_policy_digest_selftest_default_text(policy_doc: dict | None) -> str:
    source = policy_doc if isinstance(policy_doc, dict) else {}
    return (
        str(source.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT)).strip()
        or AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT
    )


def build_age5_policy_digest_selftest_default_field(policy_doc: dict | None) -> dict[str, str]:
    source = policy_doc if isinstance(policy_doc, dict) else {}
    row = source.get("combined_digest_selftest_default_field")
    if isinstance(row, dict):
        parsed = {str(key): str(value) for key, value in row.items()}
        if parsed:
            return parsed
    return dict(AGE5_DIGEST_SELFTEST_DEFAULT_FIELD)


def normalize_age5_child_status(value: object, *, default: str) -> str:
    text = str(value).strip()
    if text in AGE5_CHILD_STATUS_VALUES:
        return text
    return default


def build_age5_child_summary_fields(doc: dict | None, *, default: str) -> dict[str, str]:
    source = doc if isinstance(doc, dict) else {}
    return {
        key: normalize_age5_child_status(source.get(key), default=default)
        for key in AGE5_CHILD_STATUS_KEYS
    }


def build_age5_child_summary_default_transport_fields(doc: dict | None) -> dict[str, str]:
    source = doc if isinstance(doc, dict) else {}
    return {
        key: str(source.get(key, expected)).strip() or expected
        for key, expected in AGE5_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS.items()
    }


def build_age5_digest_selftest_default_text(doc: dict | None) -> str:
    source = doc if isinstance(doc, dict) else {}
    return str(source.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT)).strip() or AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT


def build_age5_digest_selftest_default_field(doc: dict | None) -> dict[str, str]:
    source = doc if isinstance(doc, dict) else {}
    row = source.get("combined_digest_selftest_default_field")
    if isinstance(row, dict):
        parsed = {str(key): str(value) for key, value in row.items()}
        if parsed:
            return parsed
    return dict(AGE5_DIGEST_SELFTEST_DEFAULT_FIELD)


def clip(text: str, limit: int = 140) -> str:
    normalized = " ".join(text.split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


def default_report_path(file_name: str) -> str:
    preferred = Path("I:/home/urihanl/ddn/codex/build/reports")
    if os.name == "nt":
        try:
            preferred.mkdir(parents=True, exist_ok=True)
        except OSError:
            pass
        return str(preferred / file_name)
    return f"build/reports/{file_name}"


def seamgrim_summary(doc: dict | None, path: Path) -> dict[str, object]:
    if not isinstance(doc, dict):
        return {
            "ok": False,
            "report_path": str(path),
            "error": "missing_or_invalid_report",
            "failed_steps": [],
            "failure_digest": [f"seamgrim report missing_or_invalid: {path}"],
        }
    steps = doc.get("steps")
    failed_steps: list[str] = []
    if isinstance(steps, list):
        for row in steps:
            if isinstance(row, dict) and not bool(row.get("ok", False)):
                failed_steps.append(str(row.get("name", "-")))
    digest = doc.get("failure_digest")
    failure_digest = [str(item) for item in digest] if isinstance(digest, list) else []
    if not failure_digest and failed_steps:
        failure_digest = [f"step={name}" for name in failed_steps]
    return {
        "ok": bool(doc.get("ok", False)),
        "report_path": str(path),
        "schema": doc.get("schema"),
        "failed_steps": failed_steps,
        "failure_digest": failure_digest,
        "elapsed_total_ms": int(doc.get("elapsed_total_ms", 0)),
    }


def oi_summary(doc: dict | None, path: Path) -> dict[str, object]:
    if not isinstance(doc, dict):
        return {
            "ok": False,
            "report_path": str(path),
            "error": "missing_or_invalid_report",
            "failed_packs": [],
            "failure_digest": [f"oi report missing_or_invalid: {path}"],
        }
    packs = doc.get("packs")
    failed_packs: list[str] = []
    if isinstance(packs, list):
        for row in packs:
            if isinstance(row, dict) and not bool(row.get("ok", False)):
                failed_packs.append(str(row.get("pack", "-")))
    digest = doc.get("failure_digest")
    failure_digest = [str(item) for item in digest] if isinstance(digest, list) else []
    if not failure_digest and failed_packs:
        failure_digest = [f"pack={name}" for name in failed_packs]
    return {
        "ok": bool(doc.get("overall_ok", False)),
        "report_path": str(path),
        "schema": doc.get("schema"),
        "failed_packs": failed_packs,
        "failure_digest": failure_digest,
    }


def age3_summary(doc: dict | None, path: Path, require_age3: bool) -> dict[str, object]:
    if doc is None and not require_age3:
        return {
            "ok": True,
            "skipped": True,
            "report_path": str(path),
            "failed_criteria": [],
            "failure_digest": [],
        }
    if not isinstance(doc, dict):
        return {
            "ok": False,
            "report_path": str(path),
            "error": "missing_or_invalid_report",
            "failed_criteria": [],
            "failure_digest": [f"age3 report missing_or_invalid: {path}"],
        }
    criteria = doc.get("criteria")
    failed_criteria: list[str] = []
    if isinstance(criteria, list):
        for row in criteria:
            if isinstance(row, dict) and not bool(row.get("ok", False)):
                failed_criteria.append(str(row.get("name", "-")))
    digest = doc.get("failure_digest")
    failure_digest = [str(item) for item in digest] if isinstance(digest, list) else []
    if not failure_digest and failed_criteria:
        failure_digest = [f"criteria={name}" for name in failed_criteria]
    return {
        "ok": bool(doc.get("overall_ok", False)),
        "report_path": str(path),
        "schema": doc.get("schema"),
        "failed_criteria": failed_criteria,
        "failure_digest": failure_digest,
    }


def age4_summary(
    doc: dict | None,
    path: Path,
    require_age4: bool,
    *,
    proof_doc: dict | None,
    proof_report_path: Path,
) -> dict[str, object]:
    if doc is None and not require_age4:
        return {
            "ok": True,
            "skipped": True,
            "report_path": str(path),
            "failed_criteria": [],
            "failure_digest": [],
            "proof_artifact_report_path": str(proof_report_path),
            "proof_artifact_report_exists": isinstance(proof_doc, dict),
            "proof_artifact_ok": True,
            "proof_artifact_failed_criteria": [],
            "proof_artifact_failed_preview": "-",
            "proof_artifact_failure_digest": [],
        }
    if not isinstance(doc, dict):
        return {
            "ok": False,
            "report_path": str(path),
            "error": "missing_or_invalid_report",
            "failed_criteria": [],
            "failure_digest": [f"age4 report missing_or_invalid: {path}"],
            "proof_artifact_report_path": str(proof_report_path),
            "proof_artifact_report_exists": isinstance(proof_doc, dict),
            "proof_artifact_ok": True,
            "proof_artifact_failed_criteria": [],
            "proof_artifact_failed_preview": "-",
            "proof_artifact_failure_digest": [],
        }
    criteria = doc.get("criteria")
    failed_criteria: list[str] = []
    if isinstance(criteria, list):
        for row in criteria:
            if isinstance(row, dict) and not bool(row.get("ok", False)):
                failed_criteria.append(str(row.get("name", "-")))
    digest = doc.get("failure_digest")
    failure_digest = [str(item) for item in digest] if isinstance(digest, list) else []
    if not failure_digest and failed_criteria:
        failure_digest = [f"criteria={name}" for name in failed_criteria]
    proof_failed_criteria: list[str] = []
    proof_failure_digest: list[str] = []
    proof_failed_preview = "-"
    proof_ok = True
    proof_summary_path = "-"
    proof_summary_hash = "-"
    if isinstance(proof_doc, dict):
        proof_ok = bool(proof_doc.get("overall_ok", False))
        proof_criteria = proof_doc.get("criteria")
        if isinstance(proof_criteria, list):
            for row in proof_criteria:
                if isinstance(row, dict) and not bool(row.get("ok", False)):
                    proof_failed_criteria.append(str(row.get("name", "-")))
        proof_digest = proof_doc.get("failure_digest")
        proof_failure_digest = [str(item) for item in proof_digest] if isinstance(proof_digest, list) else []
        if not proof_failure_digest and proof_failed_criteria:
            proof_failure_digest = [f"criteria={name}" for name in proof_failed_criteria]
        proof_failed_preview = str(proof_doc.get("failed_criteria_preview", "")).strip() or "-"
        proof_summary_path = str(proof_doc.get("proof_summary_path", "-")).strip() or "-"
        proof_summary_hash = str(proof_doc.get("proof_summary_hash", "-")).strip() or "-"
        if not proof_ok:
            failed_criteria.extend([f"proof_artifact::{name}" for name in proof_failed_criteria])
            failure_digest.extend([f"proof_artifact: {line}" for line in proof_failure_digest])
    return {
        "ok": bool(doc.get("overall_ok", False)) and proof_ok,
        "report_path": str(path),
        "schema": doc.get("schema"),
        "failed_criteria": failed_criteria,
        "failure_digest": failure_digest,
        "proof_artifact_report_path": str(proof_report_path),
        "proof_artifact_report_exists": isinstance(proof_doc, dict),
        "proof_artifact_ok": proof_ok,
        "proof_artifact_schema": proof_doc.get("schema") if isinstance(proof_doc, dict) else None,
        "proof_artifact_failed_criteria": proof_failed_criteria,
        "proof_artifact_failed_preview": proof_failed_preview,
        "proof_artifact_failure_digest": proof_failure_digest,
        "proof_artifact_summary_path": proof_summary_path,
        "proof_artifact_summary_hash": proof_summary_hash,
    }


def age5_summary(
    doc: dict | None,
    path: Path,
    require_age5: bool,
    *,
    policy_doc: dict | None,
    policy_report_path: Path,
    policy_text: str | None,
    policy_text_path: Path,
    policy_summary_text: str | None,
    policy_summary_path: Path,
) -> dict[str, object]:
    expected_policy_origin_trace = build_age5_combined_heavy_policy_origin_trace(
        report_path=str(policy_report_path),
        report_exists=isinstance(policy_doc, dict),
        text_path=str(policy_text_path),
        text_exists=policy_text is not None,
        summary_path=str(policy_summary_path),
        summary_exists=policy_summary_path.exists(),
    )
    expected_full_real_source_trace = build_age5_combined_heavy_full_real_source_trace()
    full_real_source_trace = expected_full_real_source_trace
    full_real_source_trace_text = build_age5_combined_heavy_full_real_source_trace_text(
        expected_full_real_source_trace
    )
    default_policy_age4_proof_snapshot = build_age4_proof_snapshot()
    default_policy_age4_proof_snapshot_text = build_age4_proof_snapshot_text(
        default_policy_age4_proof_snapshot
    )
    expected_policy_origin_trace_text = build_age5_combined_heavy_policy_origin_trace_text(
        expected_policy_origin_trace
    )
    summary_tokens = parse_compact_kv_line(policy_summary_text)
    policy_origin_trace = dict(expected_policy_origin_trace)
    policy_origin_trace_text = expected_policy_origin_trace_text
    policy_origin_trace_contract_error = ""
    policy_origin_trace_contract_summary_error = ""
    summary_origin_trace_text = str(
        summary_tokens.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY, "")
    ).strip()
    summary_origin_trace_raw = str(
        summary_tokens.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY, "")
    ).strip()
    if summary_origin_trace_text or summary_origin_trace_raw:
        summary_origin_trace_doc: dict | None = None
        if summary_origin_trace_raw:
            try:
                parsed = json.loads(summary_origin_trace_raw)
            except Exception:
                parsed = None
            if isinstance(parsed, dict):
                summary_origin_trace_doc = build_age5_combined_heavy_policy_origin_trace(
                    report_path=parsed.get("report_path", "-"),
                    report_exists=str(parsed.get("report_exists", "0")).strip() == "1",
                    text_path=parsed.get("text_path", "-"),
                    text_exists=str(parsed.get("text_exists", "0")).strip() == "1",
                    summary_path=parsed.get("summary_path", "-"),
                    summary_exists=str(parsed.get("summary_exists", "0")).strip() == "1",
                )
        if (
            not isinstance(summary_origin_trace_doc, dict)
            or not summary_origin_trace_text
            or summary_origin_trace_doc != expected_policy_origin_trace
            or summary_origin_trace_text != expected_policy_origin_trace_text
        ):
            policy_origin_trace_contract_error = "policy_summary_origin_trace_mismatch"
        else:
            policy_origin_trace = dict(summary_origin_trace_doc)
            policy_origin_trace_text = summary_origin_trace_text

    expected_policy_origin_trace_contract_status = (
        "mismatch" if policy_origin_trace_contract_error else "ok"
    )
    expected_policy_origin_trace_contract_ok = (
        not bool(policy_origin_trace_contract_error)
    )
    expected_policy_origin_trace_contract_issue = (
        "policy_summary_origin_trace_contract_mismatch"
        if policy_origin_trace_contract_error
        else AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT
    )
    summary_contract_issue = ""
    summary_contract_compact_reason = ""
    if summary_tokens:
        summary_contract_status = str(
            summary_tokens.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY, "")
        ).strip()
        summary_contract_ok = str(
            summary_tokens.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY, "")
        ).strip()
        summary_contract_issue = str(
            summary_tokens.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY, "")
        ).strip()
        summary_contract_compact_reason = str(
            summary_tokens.get(
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY, ""
            )
        ).strip()
        expected_policy_origin_trace_contract_compact_reason = (
            build_age5_combined_heavy_policy_origin_trace_contract_compact_reason(
                expected_policy_origin_trace_contract_issue,
                summary_contract_issue
                or AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT,
            )
        )
        if (
            summary_contract_status != expected_policy_origin_trace_contract_status
            or summary_contract_ok != ("1" if expected_policy_origin_trace_contract_ok else "0")
        ):
            policy_origin_trace_contract_summary_error = (
                "policy_summary_origin_trace_contract_mismatch"
            )
        elif summary_contract_issue != expected_policy_origin_trace_contract_issue:
            policy_origin_trace_contract_summary_error = (
                "policy_summary_origin_trace_contract_issue_mismatch"
            )
        elif (
            summary_contract_compact_reason
            != expected_policy_origin_trace_contract_compact_reason
        ):
            policy_origin_trace_contract_summary_error = (
                "policy_summary_origin_trace_contract_compact_reason_mismatch"
            )

    def finalize_age5_row(row: dict[str, object]) -> dict[str, object]:
        source_trace_raw = row.get("full_real_source_trace")
        if isinstance(source_trace_raw, dict):
            full_real_row = {
                str(key): str(source_trace_raw.get(key, default)).strip() or default
                for key, default in expected_full_real_source_trace.items()
            }
        else:
            full_real_row = dict(expected_full_real_source_trace)
        row["full_real_source_trace"] = full_real_row
        row["full_real_source_trace_text"] = (
            str(
                row.get(
                    "full_real_source_trace_text",
                    build_age5_combined_heavy_full_real_source_trace_text(full_real_row),
                )
            ).strip()
            or build_age5_combined_heavy_full_real_source_trace_text(full_real_row)
        )
        resolved_contract_issue = (
            policy_origin_trace_contract_summary_error
            or AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT
        )
        source_contract_issue = (
            summary_contract_issue
            or AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT
        )
        row[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY] = (
            expected_policy_origin_trace_contract_status
        )
        row[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY] = (
            expected_policy_origin_trace_contract_ok
        )
        row[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY] = (
            source_contract_issue
        )
        row[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY] = (
            resolved_contract_issue
        )
        row[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY] = (
            build_age5_combined_heavy_policy_origin_trace_contract_compact_reason(
                resolved_contract_issue,
                source_contract_issue,
            )
        )
        row[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY] = (
            build_age5_combined_heavy_policy_origin_trace_contract_compact_failure_reason(
                resolved_contract_issue,
                source_contract_issue,
            )
        )
        if not policy_origin_trace_contract_error and not policy_origin_trace_contract_summary_error:
            return row
        row["ok"] = False
        failure_digest = [str(item) for item in row.get("failure_digest", [])]
        if policy_origin_trace_contract_summary_error:
            failure_digest.insert(
                0,
                (
                    f"{policy_origin_trace_contract_summary_error}: expected="
                    f"status={expected_policy_origin_trace_contract_status}|"
                    f"ok={int(expected_policy_origin_trace_contract_ok)}|"
                    f"issue={expected_policy_origin_trace_contract_issue}|"
                    f"compact_reason={row[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY]}"
                ),
            )
        if policy_origin_trace_contract_error:
            failure_digest.insert(
                0,
                f"{policy_origin_trace_contract_error}: expected={expected_policy_origin_trace_text}",
            )
        row["failure_digest"] = failure_digest
        return row

    if doc is None and not require_age5:
        return finalize_age5_row({
            "ok": True,
            "skipped": True,
            "report_path": str(path),
            "age5_combined_heavy_policy_report_path": str(policy_report_path),
            "age5_combined_heavy_policy_text_path": str(policy_text_path),
            AGE5_POLICY_SUMMARY_PATH_KEY: str(policy_summary_path),
            "age5_combined_heavy_policy_report_exists": isinstance(policy_doc, dict),
            "age5_combined_heavy_policy_text_exists": policy_text is not None,
            AGE5_POLICY_SUMMARY_EXISTS_KEY: policy_summary_path.exists(),
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY: dict(policy_origin_trace),
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY: policy_origin_trace_text,
            "failed_criteria": [],
            "failure_digest": [],
            "age5_close_digest_selftest_ok": "0",
            AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
            "combined_digest_selftest_default_field": dict(AGE5_DIGEST_SELFTEST_DEFAULT_FIELD),
            "full_real_source_trace": dict(expected_full_real_source_trace),
            "full_real_source_trace_text": full_real_source_trace_text,
            AGE5_POLICY_DIGEST_DEFAULT_FIELD_TEXT_KEY: build_age5_policy_digest_selftest_default_text(policy_doc),
            AGE5_POLICY_DIGEST_DEFAULT_FIELD_KEY: build_age5_policy_digest_selftest_default_field(policy_doc),
            AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY: AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
            AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY: default_policy_age4_proof_snapshot_text,
            AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY: AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
            AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY: "0",
            AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY: "0",
            AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY: "0",
            AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY: "0",
            **build_age5_child_summary_fields(None, default="skipped"),
            **build_age5_child_summary_default_transport_fields(None),
        })
    if not isinstance(doc, dict):
        return finalize_age5_row({
            "ok": False,
            "report_path": str(path),
            "age5_combined_heavy_policy_report_path": str(policy_report_path),
            "age5_combined_heavy_policy_text_path": str(policy_text_path),
            AGE5_POLICY_SUMMARY_PATH_KEY: str(policy_summary_path),
            "age5_combined_heavy_policy_report_exists": isinstance(policy_doc, dict),
            "age5_combined_heavy_policy_text_exists": policy_text is not None,
            AGE5_POLICY_SUMMARY_EXISTS_KEY: policy_summary_path.exists(),
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY: dict(policy_origin_trace),
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY: policy_origin_trace_text,
            "error": "missing_or_invalid_report",
            "failed_criteria": [],
            "failure_digest": [f"age5 report missing_or_invalid: {path}"],
            "age5_close_digest_selftest_ok": "0",
            AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
            "combined_digest_selftest_default_field": dict(AGE5_DIGEST_SELFTEST_DEFAULT_FIELD),
            "full_real_source_trace": dict(expected_full_real_source_trace),
            "full_real_source_trace_text": full_real_source_trace_text,
            AGE5_POLICY_DIGEST_DEFAULT_FIELD_TEXT_KEY: build_age5_policy_digest_selftest_default_text(policy_doc),
            AGE5_POLICY_DIGEST_DEFAULT_FIELD_KEY: build_age5_policy_digest_selftest_default_field(policy_doc),
            AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY: AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
            AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY: default_policy_age4_proof_snapshot_text,
            AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY: AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
            AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY: "0",
            AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY: "0",
            AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY: "0",
            AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY: "0",
            **build_age5_child_summary_fields(None, default="fail"),
            **build_age5_child_summary_default_transport_fields(None),
        })
    criteria = doc.get("criteria")
    failed_criteria: list[str] = []
    if isinstance(criteria, list):
        for row in criteria:
            if isinstance(row, dict) and not bool(row.get("ok", False)):
                failed_criteria.append(str(row.get("name", "-")))
    digest = doc.get("failure_digest")
    failure_digest = [str(item) for item in digest] if isinstance(digest, list) else []
    if not failure_digest and failed_criteria:
        failure_digest = [f"criteria={name}" for name in failed_criteria]
    return finalize_age5_row({
        "ok": bool(doc.get("overall_ok", False)),
        "report_path": str(path),
        "age5_combined_heavy_policy_report_path": str(policy_report_path),
        "age5_combined_heavy_policy_text_path": str(policy_text_path),
        AGE5_POLICY_SUMMARY_PATH_KEY: str(policy_summary_path),
        "age5_combined_heavy_policy_report_exists": isinstance(policy_doc, dict),
        "age5_combined_heavy_policy_text_exists": policy_text is not None,
        AGE5_POLICY_SUMMARY_EXISTS_KEY: policy_summary_path.exists(),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY: dict(policy_origin_trace),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY: policy_origin_trace_text,
        "schema": doc.get("schema"),
        "failed_criteria": failed_criteria,
        "failure_digest": failure_digest,
        "age5_close_digest_selftest_ok": (
            str(doc.get("age5_close_digest_selftest_ok", "0")).strip() or "0"
        ),
        AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: build_age5_digest_selftest_default_text(doc),
        "combined_digest_selftest_default_field": build_age5_digest_selftest_default_field(doc),
        "full_real_source_trace": (
            doc.get("full_real_source_trace")
            if isinstance(doc.get("full_real_source_trace"), dict)
            else dict(expected_full_real_source_trace)
        ),
        "full_real_source_trace_text": (
            str(doc.get("full_real_source_trace_text", "")).strip()
            or build_age5_combined_heavy_full_real_source_trace_text(
                doc.get("full_real_source_trace")
                if isinstance(doc.get("full_real_source_trace"), dict)
                else expected_full_real_source_trace
            )
        ),
        AGE5_COMBINED_HEAVY_CHILD_TIMEOUT_SEC_KEY: (
            str(doc.get(AGE5_COMBINED_HEAVY_CHILD_TIMEOUT_SEC_KEY, "0")).strip() or "0"
        ),
        AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY: (
            str(doc.get(AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY, "disabled")).strip() or "disabled"
        ),
        AGE5_COMBINED_HEAVY_TIMEOUT_PRESENT_KEY: (
            str(doc.get(AGE5_COMBINED_HEAVY_TIMEOUT_PRESENT_KEY, "0")).strip() or "0"
        ),
        AGE5_COMBINED_HEAVY_TIMEOUT_TARGETS_KEY: (
            str(doc.get(AGE5_COMBINED_HEAVY_TIMEOUT_TARGETS_KEY, "-")).strip() or "-"
        ),
        AGE5_POLICY_DIGEST_DEFAULT_FIELD_TEXT_KEY: build_age5_policy_digest_selftest_default_text(policy_doc),
        AGE5_POLICY_DIGEST_DEFAULT_FIELD_KEY: build_age5_policy_digest_selftest_default_field(policy_doc),
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY: (
            str((policy_doc or {}).get("age4_proof_snapshot_fields_text", AGE4_PROOF_SNAPSHOT_FIELDS_TEXT)).strip()
            or AGE4_PROOF_SNAPSHOT_FIELDS_TEXT
        ),
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY: (
            str((policy_doc or {}).get("age4_proof_snapshot_text", "")).strip()
            or default_policy_age4_proof_snapshot_text
        ),
        AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY: (
            str((policy_doc or {}).get("age4_proof_source_snapshot_fields_text", AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT)).strip()
            or AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT
        ),
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY: (
            str((policy_doc or {}).get(AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY, "0")).strip() or "0"
        ),
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY: (
            str((policy_doc or {}).get(AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY, "0")).strip() or "0"
        ),
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY: (
            str((policy_doc or {}).get(AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY, "0")).strip() or "0"
        ),
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY: (
            str((policy_doc or {}).get(AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY, "0")).strip() or "0"
        ),
        **build_age5_child_summary_fields(doc, default="skipped"),
        **build_age5_child_summary_default_transport_fields(doc),
    })


def main() -> int:
    parser = argparse.ArgumentParser(description="Combine seamgrim/oi close reports into one detjson report")
    parser.add_argument(
        "--seamgrim-report",
        default=default_report_path("seamgrim_ci_gate_report.json"),
        help="path to seamgrim ci gate report",
    )
    parser.add_argument(
        "--oi-report",
        default=default_report_path("oi405_406_close_report.detjson"),
        help="path to oi405/406 close report",
    )
    parser.add_argument(
        "--age3-report",
        default=default_report_path("age3_close_report.detjson"),
        help="path to age3 close report",
    )
    parser.add_argument(
        "--require-age3",
        action="store_true",
        help="require age3 close report to exist and pass",
    )
    parser.add_argument(
        "--age4-report",
        default=default_report_path("age4_close_report.detjson"),
        help="path to age4 close report",
    )
    parser.add_argument(
        "--age4-proof-report",
        default=default_report_path("age4_proof_artifact_report.detjson"),
        help="optional path to age4 proof artifact aggregate report",
    )
    parser.add_argument(
        "--require-age4",
        action="store_true",
        help="require age4 close report to exist and pass",
    )
    parser.add_argument(
        "--age5-report",
        default=default_report_path("age5_close_report.detjson"),
        help="path to age5 close report",
    )
    parser.add_argument(
        "--age5-combined-heavy-policy-report",
        default=default_report_path("age5_combined_heavy_policy.detjson"),
        help="optional path to AGE5 combined-heavy policy helper detjson",
    )
    parser.add_argument(
        "--age5-combined-heavy-policy-text",
        default=default_report_path("age5_combined_heavy_policy.txt"),
        help="optional path to AGE5 combined-heavy policy helper text payload",
    )
    parser.add_argument(
        "--age5-combined-heavy-policy-summary",
        default=default_report_path("age5_combined_heavy_policy_summary.txt"),
        help="optional path to AGE5 combined-heavy policy helper compact summary payload",
    )
    parser.add_argument(
        "--require-age5",
        action="store_true",
        help="require age5 close report to exist and pass",
    )
    parser.add_argument(
        "--age3-status",
        default=default_report_path("age3_close_status.detjson"),
        help="optional path to age3 close status json (link metadata)",
    )
    parser.add_argument(
        "--age3-status-line",
        default=default_report_path("age3_close_status_line.txt"),
        help="optional path to one-line age3 status text (link metadata)",
    )
    parser.add_argument(
        "--age3-badge",
        default=default_report_path("age3_close_badge.detjson"),
        help="optional path to age3 close badge json (link metadata)",
    )
    parser.add_argument(
        "--out",
        default=default_report_path("ci_aggregate_report.detjson"),
        help="output aggregate report path",
    )
    parser.add_argument(
        "--index-report-path",
        default="",
        help="optional aggregate gate index report path to embed as link metadata",
    )
    parser.add_argument("--print-summary", action="store_true", help="print aggregate summary")
    parser.add_argument("--fail-on-bad", action="store_true", help="return non-zero when any check failed")
    args = parser.parse_args()

    seamgrim_path = Path(args.seamgrim_report)
    age3_path = Path(args.age3_report)
    age4_path = Path(args.age4_report)
    age4_proof_path = Path(args.age4_proof_report)
    age5_path = Path(args.age5_report)
    age5_policy_report_path = Path(args.age5_combined_heavy_policy_report)
    age5_policy_text_path = Path(args.age5_combined_heavy_policy_text)
    age5_policy_summary_path = Path(args.age5_combined_heavy_policy_summary)
    age3_status_path = Path(args.age3_status) if args.age3_status.strip() else None
    age3_status_line_path = Path(args.age3_status_line) if args.age3_status_line.strip() else None
    age3_badge_path = Path(args.age3_badge) if args.age3_badge.strip() else None
    oi_path = Path(args.oi_report)
    out_path = Path(args.out)
    index_report_path = Path(args.index_report_path) if args.index_report_path.strip() else None

    seamgrim = seamgrim_summary(load_json(seamgrim_path), seamgrim_path)
    age3 = age3_summary(load_json(age3_path), age3_path, bool(args.require_age3))
    age4 = age4_summary(
        load_json(age4_path),
        age4_path,
        bool(args.require_age4),
        proof_doc=load_json(age4_proof_path),
        proof_report_path=age4_proof_path,
    )
    age5 = age5_summary(
        load_json(age5_path),
        age5_path,
        bool(args.require_age5),
        policy_doc=load_json(age5_policy_report_path),
        policy_report_path=age5_policy_report_path,
        policy_text=load_text(age5_policy_text_path),
        policy_text_path=age5_policy_text_path,
        policy_summary_text=load_text(age5_policy_summary_path),
        policy_summary_path=age5_policy_summary_path,
    )
    oi = oi_summary(load_json(oi_path), oi_path)
    overall_ok = (
        bool(seamgrim.get("ok", False))
        and bool(age3.get("ok", False))
        and bool(age4.get("ok", False))
        and bool(age5.get("ok", False))
        and bool(oi.get("ok", False))
    )

    failure_digest: list[str] = []
    for item in seamgrim.get("failure_digest", []):
        failure_digest.append(f"seamgrim: {clip(str(item))}")
    for item in age3.get("failure_digest", []):
        failure_digest.append(f"age3: {clip(str(item))}")
    for item in age4.get("failure_digest", []):
        failure_digest.append(f"age4: {clip(str(item))}")
    for item in age5.get("failure_digest", []):
        failure_digest.append(f"age5: {clip(str(item))}")
    for item in oi.get("failure_digest", []):
        failure_digest.append(f"oi405_406: {clip(str(item))}")

    payload = {
        "schema": "ddn.ci.aggregate_report.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "overall_ok": overall_ok,
        "seamgrim": seamgrim,
        "age3": age3,
        "age4": age4,
        "age5": age5,
        "oi405_406": oi,
        "failure_digest": failure_digest[:16],
    }
    if age3_status_path is not None:
        payload["age3_status_report_path"] = str(age3_status_path)
        age3_status_doc = load_json(age3_status_path)
        payload["age3_status_ok"] = bool(age3_status_doc.get("overall_ok", False)) if isinstance(age3_status_doc, dict) else False
    if age3_status_line_path is not None:
        payload["age3_status_line_path"] = str(age3_status_line_path)
        age3_status_line = load_text(age3_status_line_path)
        payload["age3_status_line_exists"] = age3_status_line is not None
        payload["age3_status_line"] = age3_status_line or ""
    if age3_badge_path is not None:
        payload["age3_badge_path"] = str(age3_badge_path)
        age3_badge_doc = load_json(age3_badge_path)
        payload["age3_badge_exists"] = age3_badge_doc is not None
        payload["age3_badge_status"] = str(age3_badge_doc.get("status", "-")) if isinstance(age3_badge_doc, dict) else "-"
        payload["age3_badge_color"] = str(age3_badge_doc.get("color", "-")) if isinstance(age3_badge_doc, dict) else "-"
    if index_report_path is not None:
        payload["gate_index_report_path"] = str(index_report_path)

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    if args.print_summary:
        print(f"[ci-aggregate] overall_ok={int(overall_ok)} out={out_path}")
        print(
            f" - seamgrim: ok={int(bool(seamgrim.get('ok', False)))} "
            f"failed_steps={len(seamgrim.get('failed_steps', []))}"
        )
        print(
            f" - age3: ok={int(bool(age3.get('ok', False)))} "
            f"failed_criteria={len(age3.get('failed_criteria', []))}"
        )
        print(
            f" - age4: ok={int(bool(age4.get('ok', False)))} "
            f"failed_criteria={len(age4.get('failed_criteria', []))}"
        )
        print(
            f" - age5: ok={int(bool(age5.get('ok', False)))} "
            f"failed_criteria={len(age5.get('failed_criteria', []))} "
            f"age5_close_digest_selftest_ok={age5.get('age5_close_digest_selftest_ok', '0')}"
            f" "
            f"{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}={age5.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT)}"
            f" "
            f"combined_digest_selftest_default_field={json.dumps(age5.get('combined_digest_selftest_default_field', AGE5_DIGEST_SELFTEST_DEFAULT_FIELD), ensure_ascii=False, sort_keys=True, separators=(',', ':'))}"
            f" {AGE5_POLICY_DIGEST_DEFAULT_FIELD_TEXT_KEY}={age5.get(AGE5_POLICY_DIGEST_DEFAULT_FIELD_TEXT_KEY, AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT)}"
            f" {AGE5_POLICY_DIGEST_DEFAULT_FIELD_KEY}={json.dumps(age5.get(AGE5_POLICY_DIGEST_DEFAULT_FIELD_KEY, AGE5_DIGEST_SELFTEST_DEFAULT_FIELD), ensure_ascii=False, sort_keys=True, separators=(',', ':'))}"
            f" policy_report={age5.get('age5_combined_heavy_policy_report_path', '-')}"
            f" policy_report_exists={int(bool(age5.get('age5_combined_heavy_policy_report_exists', False)))}"
            f" policy_text={age5.get('age5_combined_heavy_policy_text_path', '-')}"
            f" policy_text_exists={int(bool(age5.get('age5_combined_heavy_policy_text_exists', False)))}"
            f" policy_summary={age5.get(AGE5_POLICY_SUMMARY_PATH_KEY, '-')}"
            f" policy_summary_exists={int(bool(age5.get(AGE5_POLICY_SUMMARY_EXISTS_KEY, False)))}"
            f" {AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY}="
            f"{age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY, '-')}"
            f" {AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY}="
            f"{age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY, 'ok')}"
            f" {AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY}="
            f"{int(bool(age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY, False)))}"
            f" {AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY}="
            f"{age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY, '-')}"
            f" {AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY}="
            f"{age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY, '-')}"
            f" {AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY}="
            f"{age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY, '-')}"
            f" {AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY}={age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY, '-')}"
            f" {AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY}={json.dumps(age5.get(AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY, {}), ensure_ascii=False, sort_keys=True, separators=(',', ':'))}"
            f" "
            f"full_real={age5.get('age5_combined_heavy_full_real_status', '-')}"
            f" combined_heavy_child_timeout_sec={age5.get(AGE5_COMBINED_HEAVY_CHILD_TIMEOUT_SEC_KEY, '0')}"
            f" {AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY}={age5.get(AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY, 'disabled')}"
            f" {AGE5_COMBINED_HEAVY_TIMEOUT_PRESENT_KEY}={age5.get(AGE5_COMBINED_HEAVY_TIMEOUT_PRESENT_KEY, '0')}"
            f" {AGE5_COMBINED_HEAVY_TIMEOUT_TARGETS_KEY}={age5.get(AGE5_COMBINED_HEAVY_TIMEOUT_TARGETS_KEY, '-')}"
            f" runtime_helper_negative={age5.get('age5_combined_heavy_runtime_helper_negative_status', '-')}"
            f" group_id_summary_negative={age5.get('age5_combined_heavy_group_id_summary_negative_status', '-')}"
            f" child_summary_defaults={age5.get('ci_sanity_age5_combined_heavy_child_summary_default_fields', '-')}"
            f" sync_child_summary_defaults={age5.get('ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields', '-')}"
        )
        if age3_status_path is not None:
            print(
                f" - age3_status_path: {age3_status_path} "
                f"ok={int(bool(payload.get('age3_status_ok', False)))}"
            )
        if age3_status_line_path is not None:
            print(
                f" - age3_status_line_path: {age3_status_line_path} "
                f"exists={int(bool(payload.get('age3_status_line_exists', False)))}"
            )
            age3_status_line_print = str(payload.get("age3_status_line", "")).strip()
            if age3_status_line_print:
                print(f"   {clip(age3_status_line_print, 180)}")
        if age3_badge_path is not None:
            print(
                f" - age3_badge_path: {age3_badge_path} "
                f"exists={int(bool(payload.get('age3_badge_exists', False)))} "
                f"status={payload.get('age3_badge_status', '-')}"
            )
        print(
            f" - oi405_406: ok={int(bool(oi.get('ok', False)))} "
            f"failed_packs={len(oi.get('failed_packs', []))}"
        )
        if index_report_path is not None:
            print(f" - gate_index_path: {index_report_path}")
        for line in payload["failure_digest"][:6]:
            print(f"   {line}")

    if args.fail_on_bad and not overall_ok:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
