#!/usr/bin/env python
from __future__ import annotations

import importlib.util
import json
import subprocess
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
    AGE5_FULL_REAL_PROFILE_ELAPSED_MAP_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_STATUS_MAP_FIELDS_TEXT,
    AGE5_FULL_REAL_TIMEOUT_BREAKDOWN_FIELDS_TEXT,
    AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROOF_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_GATE0_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT,
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
    build_age5_full_real_profile_elapsed_map,
    build_age5_full_real_profile_status_map,
    build_age5_full_real_timeout_breakdown,
    build_age5_full_real_w107_golden_index_selftest_progress,
    build_age5_full_real_w107_progress_contract_selftest_progress,
    build_age5_full_real_age1_immediate_proof_operation_contract_selftest_progress,
    build_age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress,
    build_age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress,
    build_age5_full_real_proof_certificate_v1_family_contract_selftest_progress,
    build_age5_full_real_proof_certificate_family_contract_selftest_progress,
    build_age5_full_real_proof_certificate_family_transport_contract_selftest_progress,
    build_age5_full_real_proof_family_contract_selftest_progress,
    build_age5_full_real_proof_family_transport_contract_selftest_progress,
    build_age5_full_real_lang_surface_family_contract_selftest_progress,
    build_age5_full_real_lang_surface_family_transport_contract_selftest_progress,
    build_age5_full_real_lang_runtime_family_contract_selftest_progress,
    build_age5_full_real_lang_runtime_family_transport_contract_selftest_progress,
    build_age5_full_real_gate0_family_contract_selftest_progress,
    build_age5_full_real_gate0_surface_family_contract_selftest_progress,
    build_age5_full_real_gate0_surface_family_transport_contract_selftest_progress,
    build_age5_full_real_gate0_family_transport_contract_selftest_progress,
    build_age5_full_real_gate0_transport_family_contract_selftest_progress,
    build_age5_full_real_gate0_transport_family_transport_contract_selftest_progress,
    build_age5_full_real_gate0_runtime_family_transport_contract_selftest_progress,
    build_age5_full_real_bogae_alias_family_contract_selftest_progress,
    build_age5_full_real_bogae_alias_family_transport_contract_selftest_progress,
)


def load_age5_close_module(root: Path):
    path = root / "tests" / "run_age5_close.py"
    spec = importlib.util.spec_from_file_location("age5_close_mod", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"module load failed: {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def completed(cmd: list[str], rc: int, stdout: str) -> subprocess.CompletedProcess[str]:
    return subprocess.CompletedProcess(cmd, rc, stdout=stdout, stderr="")


_FAIL_LOG_MUTED = False


def set_fail_log_muted(muted: bool) -> None:
    global _FAIL_LOG_MUTED
    _FAIL_LOG_MUTED = bool(muted)


def fail(detail: str) -> int:
    if (not _FAIL_LOG_MUTED) or detail.endswith("should fail"):
        print(f"[age5-close-combined-report-contract-selftest] fail: {detail}")
    return 1


def write_json(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def check_w107_progress(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(report.get("age5_full_real_w107_golden_index_selftest_progress_fields_text", "")).strip() != (
        AGE5_FULL_REAL_W107_GOLDEN_INDEX_SELFTEST_PROGRESS_FIELDS_TEXT
    ):
        return fail(f"{label} w107 progress fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} w107 progress mismatch: {key}")
    return 0


def check_w107_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(report.get("age5_full_real_w107_progress_contract_selftest_progress_fields_text", "")).strip() != (
        AGE5_FULL_REAL_W107_PROGRESS_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT
    ):
        return fail(f"{label} w107 progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} w107 progress-contract mismatch: {key}")
    return 0


def check_age1_immediate_proof_operation_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_fields_text", ""
        )
    ).strip() != AGE5_FULL_REAL_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} age1 immediate proof operation progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} age1 immediate proof operation progress-contract mismatch: {key}")
    return 0


def check_proof_certificate_v1_consumer_transport_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} proof_certificate_v1 consumer transport progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} proof_certificate_v1 consumer transport progress-contract mismatch: {key}")
    return 0


def check_proof_certificate_v1_verify_report_digest_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} proof_certificate_v1 verify-report digest progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} proof_certificate_v1 verify-report digest progress-contract mismatch: {key}")
    return 0


def check_proof_certificate_v1_family_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_proof_certificate_v1_family_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} proof_certificate_v1 family progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} proof_certificate_v1 family progress-contract mismatch: {key}")
    return 0


def check_proof_certificate_family_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_proof_certificate_family_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} proof_certificate family progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} proof_certificate family progress-contract mismatch: {key}")
    return 0


def check_proof_certificate_family_transport_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} proof_certificate family transport progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} proof_certificate family transport progress-contract mismatch: {key}")
    return 0


def check_proof_family_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_proof_family_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_PROOF_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} proof family progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} proof family progress-contract mismatch: {key}")
    return 0


def check_proof_family_transport_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_proof_family_transport_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} proof family transport progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} proof family transport progress-contract mismatch: {key}")
    return 0


def check_lang_surface_family_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_lang_surface_family_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} lang surface family progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} lang surface family progress-contract mismatch: {key}")
    return 0


def check_lang_surface_family_transport_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_lang_surface_family_transport_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} lang surface family transport progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} lang surface family transport progress-contract mismatch: {key}")
    return 0


def check_lang_runtime_family_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_lang_runtime_family_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} lang runtime family progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} lang runtime family progress-contract mismatch: {key}")
    return 0


def check_lang_runtime_family_transport_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} lang runtime family transport progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} lang runtime family transport progress-contract mismatch: {key}")
    return 0


def check_gate0_runtime_family_transport_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} gate0 runtime family transport progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} gate0 runtime family transport progress-contract mismatch: {key}")
    return 0


def check_gate0_family_transport_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_gate0_family_transport_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} gate0 family transport progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} gate0 family transport progress-contract mismatch: {key}")
    return 0


def check_gate0_transport_family_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_gate0_transport_family_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} gate0 transport family progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} gate0 transport family progress-contract mismatch: {key}")
    return 0


def check_gate0_transport_family_transport_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_gate0_transport_family_transport_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(
            f"{label} gate0 transport family transport progress-contract fields text mismatch"
        )
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(
                f"{label} gate0 transport family transport progress-contract mismatch: {key}"
            )
    return 0


def check_gate0_family_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_gate0_family_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_GATE0_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} gate0 family progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} gate0 family progress-contract mismatch: {key}")
    return 0


def check_gate0_surface_family_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_gate0_surface_family_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} gate0 surface family progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} gate0 surface family progress-contract mismatch: {key}")
    return 0


def check_gate0_surface_family_transport_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} gate0 surface family transport progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} gate0 surface family transport progress-contract mismatch: {key}")
    return 0


def check_bogae_alias_family_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_bogae_alias_family_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} bogae alias family progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} bogae alias family progress-contract mismatch: {key}")
    return 0


def check_bogae_alias_family_transport_progress_contract(
    report: dict[str, object],
    expected: dict[str, str],
    *,
    label: str,
) -> int:
    if str(
        report.get(
            "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_fields_text",
            "",
        )
    ).strip() != AGE5_FULL_REAL_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_FIELDS_TEXT:
        return fail(f"{label} bogae alias family transport progress-contract fields text mismatch")
    for key, expected_value in expected.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"{label} bogae alias family transport progress-contract mismatch: {key}")
    return 0


def check_report_contract(
    report: dict[str, object],
    env_enabled: bool,
    overall_ok: bool,
    expected_age4_proof_snapshot: dict[str, str] | None = None,
    expected_w107_progress: dict[str, str] | None = None,
    expected_w107_progress_contract: dict[str, str] | None = None,
    expected_age1_immediate_proof_operation_progress_contract: dict[str, str] | None = None,
    expected_proof_certificate_v1_consumer_transport_progress_contract: dict[str, str] | None = None,
    expected_proof_certificate_v1_verify_report_digest_progress_contract: dict[str, str] | None = None,
    expected_proof_certificate_v1_family_progress_contract: dict[str, str] | None = None,
    expected_proof_certificate_family_progress_contract: dict[str, str] | None = None,
    expected_proof_certificate_family_transport_progress_contract: dict[str, str] | None = None,
    expected_proof_family_progress_contract: dict[str, str] | None = None,
    expected_proof_family_transport_progress_contract: dict[str, str] | None = None,
    expected_lang_surface_family_progress_contract: dict[str, str] | None = None,
    expected_lang_surface_family_transport_progress_contract: dict[str, str] | None = None,
    expected_lang_runtime_family_progress_contract: dict[str, str] | None = None,
    expected_lang_runtime_family_transport_progress_contract: dict[str, str] | None = None,
    expected_gate0_family_progress_contract: dict[str, str] | None = None,
    expected_gate0_surface_family_progress_contract: dict[str, str] | None = None,
    expected_gate0_surface_family_transport_progress_contract: dict[str, str] | None = None,
    expected_gate0_family_transport_progress_contract: dict[str, str] | None = None,
    expected_gate0_transport_family_progress_contract: dict[str, str] | None = None,
    expected_gate0_transport_family_transport_progress_contract: dict[str, str] | None = None,
    expected_gate0_runtime_family_transport_progress_contract: dict[str, str] | None = None,
    expected_bogae_alias_family_progress_contract: dict[str, str] | None = None,
    expected_bogae_alias_family_transport_progress_contract: dict[str, str] | None = None,
) -> int:
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
    expected_age4_proof_snapshot = (
        build_age4_proof_snapshot(**expected_age4_proof_snapshot)
        if isinstance(expected_age4_proof_snapshot, dict)
        else build_age4_proof_snapshot(
            age4_proof_ok=report.get("age4_proof_ok", "0"),
            age4_proof_failed_criteria=report.get("age4_proof_failed_criteria", "-1"),
            age4_proof_failed_preview=report.get("age4_proof_failed_preview", "-"),
        )
    )
    expected_digest_default_field = build_age5_close_digest_selftest_default_field()
    criteria = report.get("criteria")
    if not isinstance(criteria, list):
        return fail("criteria missing")
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
    if str(report.get("schema", "")).strip() != AGE5_COMBINED_HEAVY_REPORT_SCHEMA:
        return fail("schema mismatch")
    if bool(report.get("with_combined_heavy_runtime_helper_check", False)) is not True:
        return fail("combined optin flag mismatch")
    if bool(report.get("combined_heavy_env_enabled", False)) is not env_enabled:
        return fail("combined env enabled mismatch")
    if bool(report.get("overall_ok", False)) is not overall_ok:
        return fail("overall_ok mismatch")
    if str(report.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip() != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
        return fail("top-level combined_digest_selftest_default_field_text mismatch")
    if dict(report.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_KEY, {})) != expected_digest_default_field:
        return fail("top-level combined_digest_selftest_default_field mismatch")
    if str(report.get("age4_proof_snapshot_fields_text", "")).strip() != AGE4_PROOF_SNAPSHOT_FIELDS_TEXT:
        return fail("top-level age4_proof_snapshot_fields_text mismatch")
    if str(report.get("age4_proof_snapshot_text", "")).strip() != build_age4_proof_snapshot_text(expected_age4_proof_snapshot):
        return fail("top-level age4_proof_snapshot_text mismatch")
    top_age4_proof_text = build_age4_proof_snapshot_text(expected_age4_proof_snapshot)
    gate_age4_proof_text = str(
        report.get(AGE4_PROOF_GATE_RESULT_SNAPSHOT_TEXT_KEY, top_age4_proof_text)
    ).strip() or top_age4_proof_text
    gate_age4_proof_present = str(
        report.get(AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY, "0")
    ).strip() or "0"
    final_age4_proof_text = str(
        report.get(AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_TEXT_KEY, top_age4_proof_text)
    ).strip() or top_age4_proof_text
    final_age4_proof_present = str(
        report.get(AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY, "0")
    ).strip() or "0"
    expected_source_fields = {
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
    for key, expected_value in expected_age4_proof_snapshot.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"top-level age4 proof snapshot mismatch: {key}")
    for key, expected_value in expected_source_fields.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"top-level age4 proof source mismatch: {key}")
    if dict(report.get("full_real_source_trace", {})) != expected_full_real_source_trace:
        return fail("top-level full_real_source_trace mismatch")
    if str(report.get("full_real_source_trace_text", "")).strip() != (
        build_age5_combined_heavy_full_real_source_trace_text(expected_full_real_source_trace)
    ):
        return fail("top-level full_real_source_trace_text mismatch")
    if str(report.get("age5_combined_heavy_timeout_policy_ok", "")).strip() != "1":
        return fail("top-level age5_combined_heavy_timeout_policy_ok mismatch")
    if str(report.get(AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_KEY, "")).strip() != (
        AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT
    ):
        return fail("top-level combined timeout policy reason mismatch")
    if str(report.get("age5_full_real_elapsed_fields_text", "")).strip() != (
        AGE5_FULL_REAL_ELAPSED_FIELDS_TEXT
    ):
        return fail("top-level age5_full_real_elapsed_fields_text mismatch")
    expected_full_real_elapsed_summary = build_age5_full_real_elapsed_summary(
        age5_full_real_total_elapsed_ms=report.get("age5_full_real_total_elapsed_ms", "-"),
        age5_full_real_slowest_profile=report.get("age5_full_real_slowest_profile", "-"),
        age5_full_real_slowest_elapsed_ms=report.get("age5_full_real_slowest_elapsed_ms", "-"),
        age5_full_real_elapsed_present=str(report.get("age5_full_real_elapsed_present", "0")).strip() == "1",
    )
    for key, expected_value in expected_full_real_elapsed_summary.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"top-level full_real elapsed summary mismatch: {key}")
    if str(report.get("age5_full_real_elapsed_present", "0")).strip() == "1":
        if str(report.get("age5_full_real_total_elapsed_ms", "-")).strip() == "-":
            return fail("top-level full_real total elapsed missing")
        if str(report.get("age5_full_real_slowest_profile", "-")).strip() == "-":
            return fail("top-level full_real slowest profile missing")
        if str(report.get("age5_full_real_slowest_elapsed_ms", "-")).strip() == "-":
            return fail("top-level full_real slowest elapsed missing")
    if str(report.get("age5_full_real_core_lang_sanity_elapsed_fields_text", "")).strip() != (
        AGE5_FULL_REAL_CORE_LANG_SANITY_ELAPSED_FIELDS_TEXT
    ):
        return fail("top-level age5_full_real_core_lang_sanity_elapsed_fields_text mismatch")
    expected_full_real_core_lang_sanity_elapsed_summary = build_age5_full_real_core_lang_sanity_elapsed_summary(
        age5_full_real_core_lang_sanity_total_elapsed_ms=report.get(
            "age5_full_real_core_lang_sanity_total_elapsed_ms", "-"
        ),
        age5_full_real_core_lang_sanity_slowest_step=report.get(
            "age5_full_real_core_lang_sanity_slowest_step", "-"
        ),
        age5_full_real_core_lang_sanity_slowest_elapsed_ms=report.get(
            "age5_full_real_core_lang_sanity_slowest_elapsed_ms", "-"
        ),
        age5_full_real_core_lang_sanity_elapsed_present=(
            str(report.get("age5_full_real_core_lang_sanity_elapsed_present", "0")).strip() == "1"
        ),
    )
    for key, expected in expected_full_real_core_lang_sanity_elapsed_summary.items():
        if str(report.get(key, "")).strip() != str(expected):
            return fail(f"top-level full_real core_lang sanity elapsed mismatch: {key}")
    if str(report.get("age5_full_real_core_lang_sanity_elapsed_present", "0")).strip() == "1":
        if str(report.get("age5_full_real_core_lang_sanity_total_elapsed_ms", "-")).strip() == "-":
            return fail("top-level full_real core_lang sanity total elapsed missing")
        if str(report.get("age5_full_real_core_lang_sanity_slowest_step", "-")).strip() == "-":
            return fail("top-level full_real core_lang sanity slowest step missing")
        if str(report.get("age5_full_real_core_lang_sanity_slowest_elapsed_ms", "-")).strip() == "-":
            return fail("top-level full_real core_lang sanity slowest elapsed missing")
    if str(report.get("age5_full_real_profile_elapsed_map_fields_text", "")).strip() != (
        AGE5_FULL_REAL_PROFILE_ELAPSED_MAP_FIELDS_TEXT
    ):
        return fail("top-level age5_full_real_profile_elapsed_map_fields_text mismatch")
    expected_full_real_profile_elapsed_map = build_age5_full_real_profile_elapsed_map(
        age5_full_real_profile_elapsed_map=report.get("age5_full_real_profile_elapsed_map", "-"),
        age5_full_real_profile_elapsed_map_present=(
            str(report.get("age5_full_real_profile_elapsed_map_present", "0")).strip() == "1"
        ),
    )
    for key, expected_value in expected_full_real_profile_elapsed_map.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"top-level full_real profile elapsed map mismatch: {key}")
    if str(report.get("age5_full_real_profile_elapsed_map_present", "0")).strip() == "1":
        if str(report.get("age5_full_real_profile_elapsed_map", "-")).strip() == "-":
            return fail("top-level full_real profile elapsed map missing")
    if str(report.get("age5_full_real_profile_status_map_fields_text", "")).strip() != (
        AGE5_FULL_REAL_PROFILE_STATUS_MAP_FIELDS_TEXT
    ):
        return fail("top-level age5_full_real_profile_status_map_fields_text mismatch")
    expected_full_real_profile_status_map = build_age5_full_real_profile_status_map(
        age5_full_real_profile_status_map=report.get("age5_full_real_profile_status_map", "-"),
        age5_full_real_profile_status_map_present=(
            str(report.get("age5_full_real_profile_status_map_present", "0")).strip() == "1"
        ),
    )
    for key, expected in expected_full_real_profile_status_map.items():
        if str(report.get(key, "")).strip() != str(expected):
            return fail(f"top-level full_real profile status map mismatch: {key}")
    if str(report.get("age5_full_real_profile_status_map_present", "0")).strip() == "1":
        if str(report.get("age5_full_real_profile_status_map", "-")).strip() == "-":
            return fail("top-level full_real profile status map missing")
    if str(report.get("age5_full_real_timeout_breakdown_fields_text", "")).strip() != (
        AGE5_FULL_REAL_TIMEOUT_BREAKDOWN_FIELDS_TEXT
    ):
        return fail("top-level age5_full_real_timeout_breakdown_fields_text mismatch")
    expected_full_real_timeout_breakdown = build_age5_full_real_timeout_breakdown(
        age5_full_real_timeout_step=report.get("age5_full_real_timeout_step", "-"),
        age5_full_real_timeout_profiles=report.get("age5_full_real_timeout_profiles", "-"),
        age5_full_real_timeout_present=str(report.get("age5_full_real_timeout_present", "0")).strip() == "1",
    )
    for key, expected_value in expected_full_real_timeout_breakdown.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"top-level full_real timeout breakdown mismatch: {key}")
    if str(report.get("age5_full_real_timeout_present", "0")).strip() == "1":
        if str(report.get("age5_full_real_timeout_step", "-")).strip() == "-":
            return fail("top-level full_real timeout step missing")
        if str(report.get("age5_full_real_timeout_profiles", "-")).strip() == "-":
            return fail("top-level full_real timeout profiles missing")
    normalized_expected_w107_progress = (
        dict(expected_w107_progress)
        if isinstance(expected_w107_progress, dict)
        else build_age5_full_real_w107_golden_index_selftest_progress()
    )
    normalized_expected_w107_progress_contract = (
        dict(expected_w107_progress_contract)
        if isinstance(expected_w107_progress_contract, dict)
        else build_age5_full_real_w107_progress_contract_selftest_progress()
    )
    normalized_expected_age1_immediate_proof_operation_progress_contract = (
        dict(expected_age1_immediate_proof_operation_progress_contract)
        if isinstance(expected_age1_immediate_proof_operation_progress_contract, dict)
        else build_age5_full_real_age1_immediate_proof_operation_contract_selftest_progress()
    )
    normalized_expected_proof_certificate_v1_consumer_transport_progress_contract = (
        dict(expected_proof_certificate_v1_consumer_transport_progress_contract)
        if isinstance(expected_proof_certificate_v1_consumer_transport_progress_contract, dict)
        else build_age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress()
    )
    normalized_expected_proof_certificate_v1_verify_report_digest_progress_contract = (
        dict(expected_proof_certificate_v1_verify_report_digest_progress_contract)
        if isinstance(expected_proof_certificate_v1_verify_report_digest_progress_contract, dict)
        else build_age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress()
    )
    normalized_expected_proof_certificate_v1_family_progress_contract = (
        dict(expected_proof_certificate_v1_family_progress_contract)
        if isinstance(expected_proof_certificate_v1_family_progress_contract, dict)
        else build_age5_full_real_proof_certificate_v1_family_contract_selftest_progress()
    )
    normalized_expected_proof_certificate_family_progress_contract = (
        dict(expected_proof_certificate_family_progress_contract)
        if isinstance(expected_proof_certificate_family_progress_contract, dict)
        else build_age5_full_real_proof_certificate_family_contract_selftest_progress()
    )
    normalized_expected_proof_certificate_family_transport_progress_contract = (
        dict(expected_proof_certificate_family_transport_progress_contract)
        if isinstance(expected_proof_certificate_family_transport_progress_contract, dict)
        else build_age5_full_real_proof_certificate_family_transport_contract_selftest_progress(
            age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks=(
                report.get(
                    "age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ),
            age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks=(
                report.get(
                    "age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ),
            age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text=(
                report.get(
                    "age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ),
            age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe=(
                report.get(
                    "age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ),
            age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe=(
                report.get(
                    "age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ),
            age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present=(
                report.get(
                    "age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ),
        )
    )
    normalized_expected_proof_family_progress_contract = (
        dict(expected_proof_family_progress_contract)
        if isinstance(expected_proof_family_progress_contract, dict)
        else build_age5_full_real_proof_family_contract_selftest_progress(
            age5_full_real_proof_family_contract_selftest_completed_checks=(
                report.get("age5_full_real_proof_family_contract_selftest_completed_checks", "-")
            ),
            age5_full_real_proof_family_contract_selftest_total_checks=(
                report.get("age5_full_real_proof_family_contract_selftest_total_checks", "-")
            ),
            age5_full_real_proof_family_contract_selftest_checks_text=(
                report.get("age5_full_real_proof_family_contract_selftest_checks_text", "-")
            ),
            age5_full_real_proof_family_contract_selftest_current_probe=(
                report.get("age5_full_real_proof_family_contract_selftest_current_probe", "-")
            ),
            age5_full_real_proof_family_contract_selftest_last_completed_probe=(
                report.get(
                    "age5_full_real_proof_family_contract_selftest_last_completed_probe",
                    "-",
                )
            ),
            age5_full_real_proof_family_contract_selftest_progress_present=(
                report.get("age5_full_real_proof_family_contract_selftest_progress_present", "0")
            ),
        )
    )
    normalized_expected_proof_family_transport_progress_contract = (
        dict(expected_proof_family_transport_progress_contract)
        if isinstance(expected_proof_family_transport_progress_contract, dict)
        else build_age5_full_real_proof_family_transport_contract_selftest_progress(
            age5_full_real_proof_family_transport_contract_selftest_completed_checks=(
                report.get("age5_full_real_proof_family_transport_contract_selftest_completed_checks", "-")
            ),
            age5_full_real_proof_family_transport_contract_selftest_total_checks=(
                report.get("age5_full_real_proof_family_transport_contract_selftest_total_checks", "-")
            ),
            age5_full_real_proof_family_transport_contract_selftest_checks_text=(
                report.get("age5_full_real_proof_family_transport_contract_selftest_checks_text", "-")
            ),
            age5_full_real_proof_family_transport_contract_selftest_current_probe=(
                report.get("age5_full_real_proof_family_transport_contract_selftest_current_probe", "-")
            ),
            age5_full_real_proof_family_transport_contract_selftest_last_completed_probe=(
                report.get("age5_full_real_proof_family_transport_contract_selftest_last_completed_probe", "-")
            ),
            age5_full_real_proof_family_transport_contract_selftest_progress_present=(
                report.get("age5_full_real_proof_family_transport_contract_selftest_progress_present", "0")
            ),
        )
    )
    normalized_expected_lang_surface_family_progress_contract = (
        dict(expected_lang_surface_family_progress_contract)
        if isinstance(expected_lang_surface_family_progress_contract, dict)
        else build_age5_full_real_lang_surface_family_contract_selftest_progress(
            age5_full_real_lang_surface_family_contract_selftest_completed_checks=(
                report.get("age5_full_real_lang_surface_family_contract_selftest_completed_checks", "-")
            ),
            age5_full_real_lang_surface_family_contract_selftest_total_checks=(
                report.get("age5_full_real_lang_surface_family_contract_selftest_total_checks", "-")
            ),
            age5_full_real_lang_surface_family_contract_selftest_checks_text=(
                report.get("age5_full_real_lang_surface_family_contract_selftest_checks_text", "-")
            ),
            age5_full_real_lang_surface_family_contract_selftest_current_probe=(
                report.get("age5_full_real_lang_surface_family_contract_selftest_current_probe", "-")
            ),
            age5_full_real_lang_surface_family_contract_selftest_last_completed_probe=(
                report.get(
                    "age5_full_real_lang_surface_family_contract_selftest_last_completed_probe",
                    "-",
                )
            ),
            age5_full_real_lang_surface_family_contract_selftest_progress_present=(
                report.get("age5_full_real_lang_surface_family_contract_selftest_progress_present", "0")
            ),
        )
    )
    normalized_expected_lang_surface_family_transport_progress_contract = (
        dict(expected_lang_surface_family_transport_progress_contract)
        if isinstance(expected_lang_surface_family_transport_progress_contract, dict)
        else build_age5_full_real_lang_surface_family_transport_contract_selftest_progress(
            age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks=(
                report.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ),
            age5_full_real_lang_surface_family_transport_contract_selftest_total_checks=(
                report.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ),
            age5_full_real_lang_surface_family_transport_contract_selftest_checks_text=(
                report.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ),
            age5_full_real_lang_surface_family_transport_contract_selftest_current_probe=(
                report.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ),
            age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe=(
                report.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ),
            age5_full_real_lang_surface_family_transport_contract_selftest_progress_present=(
                report.get(
                    "age5_full_real_lang_surface_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ),
        )
    )
    normalized_expected_lang_runtime_family_progress_contract = (
        dict(expected_lang_runtime_family_progress_contract)
        if isinstance(expected_lang_runtime_family_progress_contract, dict)
        else build_age5_full_real_lang_runtime_family_contract_selftest_progress(
            age5_full_real_lang_runtime_family_contract_selftest_completed_checks=(
                report.get("age5_full_real_lang_runtime_family_contract_selftest_completed_checks", "-")
            ),
            age5_full_real_lang_runtime_family_contract_selftest_total_checks=(
                report.get("age5_full_real_lang_runtime_family_contract_selftest_total_checks", "-")
            ),
            age5_full_real_lang_runtime_family_contract_selftest_checks_text=(
                report.get("age5_full_real_lang_runtime_family_contract_selftest_checks_text", "-")
            ),
            age5_full_real_lang_runtime_family_contract_selftest_current_probe=(
                report.get("age5_full_real_lang_runtime_family_contract_selftest_current_probe", "-")
            ),
            age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe=(
                report.get("age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe", "-")
            ),
            age5_full_real_lang_runtime_family_contract_selftest_progress_present=(
                report.get("age5_full_real_lang_runtime_family_contract_selftest_progress_present", "0")
            ),
        )
    )
    normalized_expected_lang_runtime_family_transport_progress_contract = (
        dict(expected_lang_runtime_family_transport_progress_contract)
        if isinstance(expected_lang_runtime_family_transport_progress_contract, dict)
        else build_age5_full_real_lang_runtime_family_transport_contract_selftest_progress(
            age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks=(
                report.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ),
            age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks=(
                report.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ),
            age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text=(
                report.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ),
            age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe=(
                report.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ),
            age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe=(
                report.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ),
            age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present=(
                report.get(
                    "age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ),
        )
    )
    normalized_expected_gate0_family_progress_contract = (
        dict(expected_gate0_family_progress_contract)
        if isinstance(expected_gate0_family_progress_contract, dict)
        else build_age5_full_real_gate0_family_contract_selftest_progress(
            age5_full_real_gate0_family_contract_selftest_completed_checks=(
                report.get("age5_full_real_gate0_family_contract_selftest_completed_checks", "-")
            ),
            age5_full_real_gate0_family_contract_selftest_total_checks=(
                report.get("age5_full_real_gate0_family_contract_selftest_total_checks", "-")
            ),
            age5_full_real_gate0_family_contract_selftest_checks_text=(
                report.get("age5_full_real_gate0_family_contract_selftest_checks_text", "-")
            ),
            age5_full_real_gate0_family_contract_selftest_current_probe=(
                report.get("age5_full_real_gate0_family_contract_selftest_current_probe", "-")
            ),
            age5_full_real_gate0_family_contract_selftest_last_completed_probe=(
                report.get("age5_full_real_gate0_family_contract_selftest_last_completed_probe", "-")
            ),
            age5_full_real_gate0_family_contract_selftest_progress_present=(
                report.get("age5_full_real_gate0_family_contract_selftest_progress_present", "0")
            ),
        )
    )
    normalized_expected_gate0_surface_family_progress_contract = (
        dict(expected_gate0_surface_family_progress_contract)
        if isinstance(expected_gate0_surface_family_progress_contract, dict)
        else build_age5_full_real_gate0_surface_family_contract_selftest_progress(
            age5_full_real_gate0_surface_family_contract_selftest_completed_checks=(
                report.get("age5_full_real_gate0_surface_family_contract_selftest_completed_checks", "-")
            ),
            age5_full_real_gate0_surface_family_contract_selftest_total_checks=(
                report.get("age5_full_real_gate0_surface_family_contract_selftest_total_checks", "-")
            ),
            age5_full_real_gate0_surface_family_contract_selftest_checks_text=(
                report.get("age5_full_real_gate0_surface_family_contract_selftest_checks_text", "-")
            ),
            age5_full_real_gate0_surface_family_contract_selftest_current_probe=(
                report.get("age5_full_real_gate0_surface_family_contract_selftest_current_probe", "-")
            ),
            age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe=(
                report.get(
                    "age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe",
                    "-",
                )
            ),
            age5_full_real_gate0_surface_family_contract_selftest_progress_present=(
                report.get("age5_full_real_gate0_surface_family_contract_selftest_progress_present", "0")
            ),
        )
    )
    normalized_expected_gate0_surface_family_transport_progress_contract = (
        dict(expected_gate0_surface_family_transport_progress_contract)
        if isinstance(expected_gate0_surface_family_transport_progress_contract, dict)
        else build_age5_full_real_gate0_surface_family_transport_contract_selftest_progress(
            age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks=(
                report.get(
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ),
            age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks=(
                report.get(
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ),
            age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text=(
                report.get(
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ),
            age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe=(
                report.get(
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ),
            age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe=(
                report.get(
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ),
            age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present=(
                report.get(
                    "age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ),
        )
    )
    normalized_expected_gate0_family_transport_progress_contract = (
        dict(expected_gate0_family_transport_progress_contract)
        if isinstance(expected_gate0_family_transport_progress_contract, dict)
        else build_age5_full_real_gate0_family_transport_contract_selftest_progress(
            age5_full_real_gate0_family_transport_contract_selftest_completed_checks=(
                report.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ),
            age5_full_real_gate0_family_transport_contract_selftest_total_checks=(
                report.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ),
            age5_full_real_gate0_family_transport_contract_selftest_checks_text=(
                report.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ),
            age5_full_real_gate0_family_transport_contract_selftest_current_probe=(
                report.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ),
            age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe=(
                report.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ),
            age5_full_real_gate0_family_transport_contract_selftest_progress_present=(
                report.get(
                    "age5_full_real_gate0_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ),
        )
    )
    normalized_expected_gate0_transport_family_progress_contract = (
        dict(expected_gate0_transport_family_progress_contract)
        if isinstance(expected_gate0_transport_family_progress_contract, dict)
        else build_age5_full_real_gate0_transport_family_contract_selftest_progress(
            age5_full_real_gate0_transport_family_contract_selftest_completed_checks=(
                report.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_completed_checks",
                    "-",
                )
            ),
            age5_full_real_gate0_transport_family_contract_selftest_total_checks=(
                report.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_total_checks",
                    "-",
                )
            ),
            age5_full_real_gate0_transport_family_contract_selftest_checks_text=(
                report.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_checks_text",
                    "-",
                )
            ),
            age5_full_real_gate0_transport_family_contract_selftest_current_probe=(
                report.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_current_probe",
                    "-",
                )
            ),
            age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe=(
                report.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe",
                    "-",
                )
            ),
            age5_full_real_gate0_transport_family_contract_selftest_progress_present=(
                report.get(
                    "age5_full_real_gate0_transport_family_contract_selftest_progress_present",
                    "0",
                )
            ),
        )
    )
    normalized_expected_gate0_transport_family_transport_progress_contract = (
        dict(expected_gate0_transport_family_transport_progress_contract)
        if isinstance(expected_gate0_transport_family_transport_progress_contract, dict)
        else build_age5_full_real_gate0_transport_family_transport_contract_selftest_progress(
            age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks=(
                report.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ),
            age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks=(
                report.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ),
            age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text=(
                report.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ),
            age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe=(
                report.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ),
            age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe=(
                report.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ),
            age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present=(
                report.get(
                    "age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ),
        )
    )
    normalized_expected_gate0_runtime_family_transport_progress_contract = (
        dict(expected_gate0_runtime_family_transport_progress_contract)
        if isinstance(expected_gate0_runtime_family_transport_progress_contract, dict)
        else build_age5_full_real_gate0_runtime_family_transport_contract_selftest_progress(
            age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks=(
                report.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ),
            age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks=(
                report.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ),
            age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text=(
                report.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ),
            age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe=(
                report.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ),
            age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe=(
                report.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ),
            age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present=(
                report.get(
                    "age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ),
        )
    )
    normalized_expected_bogae_alias_family_progress_contract = (
        dict(expected_bogae_alias_family_progress_contract)
        if isinstance(expected_bogae_alias_family_progress_contract, dict)
        else build_age5_full_real_bogae_alias_family_contract_selftest_progress(
            age5_full_real_bogae_alias_family_contract_selftest_completed_checks=(
                report.get(
                    "age5_full_real_bogae_alias_family_contract_selftest_completed_checks",
                    "-",
                )
            ),
            age5_full_real_bogae_alias_family_contract_selftest_total_checks=(
                report.get(
                    "age5_full_real_bogae_alias_family_contract_selftest_total_checks",
                    "-",
                )
            ),
            age5_full_real_bogae_alias_family_contract_selftest_checks_text=(
                report.get(
                    "age5_full_real_bogae_alias_family_contract_selftest_checks_text",
                    "-",
                )
            ),
            age5_full_real_bogae_alias_family_contract_selftest_current_probe=(
                report.get(
                    "age5_full_real_bogae_alias_family_contract_selftest_current_probe",
                    "-",
                )
            ),
            age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe=(
                report.get(
                    "age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe",
                    "-",
                )
            ),
            age5_full_real_bogae_alias_family_contract_selftest_progress_present=(
                report.get(
                    "age5_full_real_bogae_alias_family_contract_selftest_progress_present",
                    "0",
                )
            ),
        )
    )
    normalized_expected_bogae_alias_family_transport_progress_contract = (
        dict(expected_bogae_alias_family_transport_progress_contract)
        if isinstance(expected_bogae_alias_family_transport_progress_contract, dict)
        else build_age5_full_real_bogae_alias_family_transport_contract_selftest_progress(
            age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks=(
                report.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks",
                    "-",
                )
            ),
            age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks=(
                report.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks",
                    "-",
                )
            ),
            age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text=(
                report.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text",
                    "-",
                )
            ),
            age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe=(
                report.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe",
                    "-",
                )
            ),
            age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe=(
                report.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe",
                    "-",
                )
            ),
            age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present=(
                report.get(
                    "age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present",
                    "0",
                )
            ),
        )
    )
    rc = check_w107_progress(report, normalized_expected_w107_progress, label="top-level")
    if rc != 0:
        return rc
    rc = check_w107_progress_contract(report, normalized_expected_w107_progress_contract, label="top-level")
    if rc != 0:
        return rc
    rc = check_age1_immediate_proof_operation_progress_contract(
        report,
        normalized_expected_age1_immediate_proof_operation_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_proof_certificate_v1_consumer_transport_progress_contract(
        report,
        normalized_expected_proof_certificate_v1_consumer_transport_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_proof_certificate_v1_verify_report_digest_progress_contract(
        report,
        normalized_expected_proof_certificate_v1_verify_report_digest_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_proof_certificate_v1_family_progress_contract(
        report,
        normalized_expected_proof_certificate_v1_family_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_proof_certificate_family_progress_contract(
        report,
        normalized_expected_proof_certificate_family_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_proof_certificate_family_transport_progress_contract(
        report,
        normalized_expected_proof_certificate_family_transport_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_proof_family_progress_contract(
        report,
        normalized_expected_proof_family_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_proof_family_transport_progress_contract(
        report,
        normalized_expected_proof_family_transport_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_lang_surface_family_progress_contract(
        report,
        normalized_expected_lang_surface_family_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_lang_surface_family_transport_progress_contract(
        report,
        normalized_expected_lang_surface_family_transport_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_lang_runtime_family_progress_contract(
        report,
        normalized_expected_lang_runtime_family_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_lang_runtime_family_transport_progress_contract(
        report,
        normalized_expected_lang_runtime_family_transport_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_gate0_family_progress_contract(
        report,
        normalized_expected_gate0_family_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_gate0_surface_family_progress_contract(
        report,
        normalized_expected_gate0_surface_family_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_gate0_surface_family_transport_progress_contract(
        report,
        normalized_expected_gate0_surface_family_transport_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_gate0_family_transport_progress_contract(
        report,
        normalized_expected_gate0_family_transport_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_gate0_transport_family_progress_contract(
        report,
        normalized_expected_gate0_transport_family_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_gate0_transport_family_transport_progress_contract(
        report,
        normalized_expected_gate0_transport_family_transport_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_gate0_runtime_family_transport_progress_contract(
        report,
        normalized_expected_gate0_runtime_family_transport_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_bogae_alias_family_progress_contract(
        report,
        normalized_expected_bogae_alias_family_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    rc = check_bogae_alias_family_transport_progress_contract(
        report,
        normalized_expected_bogae_alias_family_transport_progress_contract,
        label="top-level",
    )
    if rc != 0:
        return rc
    for key, expected_value in expected_contract.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"contract key mismatch: {key}")
    for key, expected_value in expected_full_summary_contract.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"full summary contract key mismatch: {key}")
    for key, expected_value in expected_full_summary_transport.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"full summary transport key mismatch: {key}")
    for key, expected_value in expected_child_summary_default_transport.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"child summary default transport key mismatch: {key}")
    for key, expected_value in expected_child_summary.items():
        if str(report.get(key, "")).strip() != str(expected_value):
            return fail(f"child summary key mismatch: {key}")

    policy_contract = report.get("policy_contract")
    if not isinstance(policy_contract, dict):
        return fail("policy_contract missing")
    if str(policy_contract.get("env_key", "")).strip() != AGE5_COMBINED_HEAVY_ENV_KEY:
        return fail("policy_contract env_key mismatch")
    if str(policy_contract.get("scope", "")).strip() != AGE5_COMBINED_HEAVY_MODE:
        return fail("policy_contract scope mismatch")
    if str(policy_contract.get("combined_report_schema", "")).strip() != AGE5_COMBINED_HEAVY_REPORT_SCHEMA:
        return fail("policy_contract report_schema mismatch")
    if str(policy_contract.get("full_real_source_trace_text", "")).strip() != AGE5_COMBINED_HEAVY_FULL_REAL_SOURCE_TRACE_TEXT:
        return fail("policy_contract full_real_source_trace_text mismatch")
    if str(policy_contract.get("age4_proof_snapshot_fields_text", "")).strip() != AGE4_PROOF_SNAPSHOT_FIELDS_TEXT:
        return fail("policy_contract age4_proof_snapshot_fields_text mismatch")
    expected_policy_age4_proof_snapshot = build_age4_proof_snapshot()
    expected_policy_age4_proof_source = build_age4_proof_source_snapshot_fields(
        top_snapshot=expected_policy_age4_proof_snapshot
    )
    if str(policy_contract.get("age4_proof_snapshot_text", "")).strip() != (
        build_age4_proof_snapshot_text(expected_policy_age4_proof_snapshot)
    ):
        return fail("policy_contract age4_proof_snapshot_text mismatch")
    if str(policy_contract.get("age4_proof_source_snapshot_fields_text", "")).strip() != (
        AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT
    ):
        return fail("policy_contract age4_proof_source_snapshot_fields_text mismatch")
    for key, expected_value in expected_policy_age4_proof_snapshot.items():
        if str(policy_contract.get(key, "")).strip() != str(expected_value):
            return fail(f"policy_contract age4 proof snapshot mismatch: {key}")
    for key, expected_value in expected_policy_age4_proof_source.items():
        if str(policy_contract.get(key, "")).strip() != str(expected_value):
            return fail(f"policy_contract age4 proof source mismatch: {key}")
    if str(policy_contract.get("full_real_smoke_check_script", "")).strip() != expected_full_real_source_trace["smoke_check_script"]:
        return fail("policy_contract full_real_smoke_check_script mismatch")
    if str(policy_contract.get("full_real_smoke_check_selftest_script", "")).strip() != (
        expected_full_real_source_trace["smoke_check_selftest_script"]
    ):
        return fail("policy_contract full_real_smoke_check_selftest_script mismatch")
    if list(policy_contract.get("combined_required_reports", [])) != list(AGE5_COMBINED_HEAVY_REQUIRED_REPORTS):
        return fail("policy_contract required_reports mismatch")
    if list(policy_contract.get("combined_required_criteria", [])) != list(AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA):
        return fail("policy_contract required_criteria mismatch")
    if list(policy_contract.get("combined_child_summary_keys", [])) != list(AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS):
        return fail("policy_contract combined_child_summary_keys mismatch")
    if dict(policy_contract.get("combined_child_summary_default_fields", {})) != dict(
        AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS
    ):
        return fail("policy_contract combined_child_summary_default_fields mismatch")
    if str(policy_contract.get("combined_child_summary_default_fields_text", "")).strip() != (
        AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT
    ):
        return fail("policy_contract combined_child_summary_default_fields_text mismatch")
    if dict(policy_contract.get("combined_timeout_policy_fields", {})) != dict(
        build_age5_combined_heavy_timeout_policy_fields()
    ):
        return fail("policy_contract combined_timeout_policy_fields mismatch")
    if str(policy_contract.get(AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_KEY, "")).strip() != (
        AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_DEFAULT
    ):
        return fail("policy_contract combined_timeout_requires_optin mismatch")
    if str(policy_contract.get(AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_KEY, "")).strip() != (
        AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT
    ):
        return fail("policy_contract combined_timeout_policy_reason mismatch")
    if str(policy_contract.get(AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY, "")).strip() != AGE5_CLOSE_DIGEST_SELFTEST_OK_DEFAULT:
        return fail("policy_contract age5_close_digest_selftest_ok mismatch")
    if str(policy_contract.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip() != (
        AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT
    ):
        return fail("policy_contract combined_digest_selftest_default_field_text mismatch")
    if dict(policy_contract.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_KEY, {})) != expected_digest_default_field:
        return fail("policy_contract combined_digest_selftest_default_field mismatch")
    if dict(policy_contract.get("combined_child_summary_default_text_transport_fields", {})) != (
        expected_child_summary_default_transport
    ):
        return fail("policy_contract combined_child_summary_default_text_transport_fields mismatch")
    if str(policy_contract.get("combined_child_summary_default_text_transport_fields_text", "")).strip() != (
        AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS_TEXT
    ):
        return fail("policy_contract combined_child_summary_default_text_transport_fields_text mismatch")
    if dict(policy_contract.get("combined_contract_summary_fields", {})) != expected_contract:
        return fail("policy_contract combined_contract_summary_fields mismatch")
    if str(policy_contract.get("combined_contract_summary_fields_text", "")).strip() != (
        AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT
    ):
        return fail("policy_contract combined_contract_summary_fields_text mismatch")
    if dict(policy_contract.get("combined_full_summary_contract_fields", {})) != expected_full_summary_contract:
        return fail("policy_contract combined_full_summary_contract_fields mismatch")
    if str(policy_contract.get("combined_full_summary_contract_fields_text", "")).strip() != (
        AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT
    ):
        return fail("policy_contract combined_full_summary_contract_fields_text mismatch")
    if dict(policy_contract.get("combined_full_summary_text_transport_fields", {})) != expected_full_summary_transport:
        return fail("policy_contract combined_full_summary_text_transport_fields mismatch")
    if str(policy_contract.get("combined_full_summary_text_transport_fields_text", "")).strip() != (
        AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FIELDS_TEXT
    ):
        return fail("policy_contract combined_full_summary_text_transport_fields_text mismatch")

    if [str(row.get("name", "")).strip() for row in criteria if isinstance(row, dict)] != list(
        AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA
    ):
        return fail("criteria names mismatch")

    reports = report.get("reports")
    if not isinstance(reports, dict):
        return fail("reports missing")
    if list(reports.keys()) != list(AGE5_COMBINED_HEAVY_REQUIRED_REPORTS):
        return fail("report keys mismatch")
    return 0


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    mod = load_age5_close_module(root)

    full_real_cmd = [mod.sys.executable, "tests/run_age5_close.py", "--with-profile-matrix-full-real-smoke-check"]
    runtime_helper_cmd = [
        mod.sys.executable,
        "tests/run_age5_close.py",
        "--with-runtime-helper-mismatch-negative-check",
    ]
    group_id_cmd = [
        mod.sys.executable,
        "tests/run_age5_close.py",
        "--with-group-id-summary-mismatch-negative-check",
    ]
    full_real_report = root / "build" / "tmp" / "age5_close_combined_report_contract_selftest.full_real.detjson"
    runtime_helper_report = Path("build/reports/age5.close.runtime_helper_negative.detjson")
    group_id_report = Path("build/reports/age5.close.group_id_summary_negative.detjson")
    expected_w107_progress = build_age5_full_real_w107_golden_index_selftest_progress(
        age5_full_real_w107_golden_index_selftest_active_cases="54",
        age5_full_real_w107_golden_index_selftest_inactive_cases="1",
        age5_full_real_w107_golden_index_selftest_index_codes="34",
        age5_full_real_w107_golden_index_selftest_current_probe="-",
        age5_full_real_w107_golden_index_selftest_last_completed_probe="validate_pack_pointers",
        age5_full_real_w107_golden_index_selftest_progress_present=True,
    )
    expected_w107_progress_contract = build_age5_full_real_w107_progress_contract_selftest_progress(
        age5_full_real_w107_progress_contract_selftest_completed_checks="8",
        age5_full_real_w107_progress_contract_selftest_total_checks="8",
        age5_full_real_w107_progress_contract_selftest_checks_text="golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index",
        age5_full_real_w107_progress_contract_selftest_current_probe="-",
        age5_full_real_w107_progress_contract_selftest_last_completed_probe="report_index",
        age5_full_real_w107_progress_contract_selftest_progress_present=True,
    )
    expected_age1_immediate_proof_operation_progress_contract = (
        build_age5_full_real_age1_immediate_proof_operation_contract_selftest_progress(
            age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks="5",
            age5_full_real_age1_immediate_proof_operation_contract_selftest_total_checks="5",
            age5_full_real_age1_immediate_proof_operation_contract_selftest_checks_text=(
                "operation_matrix,solver_search_matrix,solver_search_parity,"
                "solver_operation_family,proof_operation_family"
            ),
            age5_full_real_age1_immediate_proof_operation_contract_selftest_current_probe="-",
            age5_full_real_age1_immediate_proof_operation_contract_selftest_last_completed_probe=(
                "proof_operation_family"
            ),
            age5_full_real_age1_immediate_proof_operation_contract_selftest_progress_present=True,
        )
    )
    expected_proof_certificate_v1_consumer_transport_progress_contract = (
        build_age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress(
            age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks="5",
            age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_total_checks="5",
            age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_checks_text=(
                "signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract"
            ),
            age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_current_probe="-",
            age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe=(
                "signed_contract"
            ),
            age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_progress_present=True,
        )
    )
    expected_proof_certificate_v1_verify_report_digest_progress_contract = (
        build_age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress(
            age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks="1",
            age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_total_checks="1",
            age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_checks_text=(
                "verify_report_digest_contract"
            ),
            age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_current_probe="-",
            age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe=(
                "readme_and_field_contract"
            ),
            age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_progress_present=True,
        )
    )
    expected_proof_certificate_v1_family_progress_contract = (
        build_age5_full_real_proof_certificate_v1_family_contract_selftest_progress(
            age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks="4",
            age5_full_real_proof_certificate_v1_family_contract_selftest_total_checks="4",
            age5_full_real_proof_certificate_v1_family_contract_selftest_checks_text=(
                "signed_contract,consumer_contract,promotion,family"
            ),
            age5_full_real_proof_certificate_v1_family_contract_selftest_current_probe="-",
            age5_full_real_proof_certificate_v1_family_contract_selftest_last_completed_probe=(
                "family"
            ),
            age5_full_real_proof_certificate_v1_family_contract_selftest_progress_present=True,
        )
    )
    expected_proof_certificate_family_progress_contract = (
        build_age5_full_real_proof_certificate_family_contract_selftest_progress(
            age5_full_real_proof_certificate_family_contract_selftest_completed_checks="3",
            age5_full_real_proof_certificate_family_contract_selftest_total_checks="3",
            age5_full_real_proof_certificate_family_contract_selftest_checks_text=(
                "artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family"
            ),
            age5_full_real_proof_certificate_family_contract_selftest_current_probe="-",
            age5_full_real_proof_certificate_family_contract_selftest_last_completed_probe=(
                "proof_certificate_family"
            ),
            age5_full_real_proof_certificate_family_contract_selftest_progress_present=True,
        )
    )
    expected_proof_certificate_family_transport_progress_contract = (
        build_age5_full_real_proof_certificate_family_transport_contract_selftest_progress(
            age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks="9",
            age5_full_real_proof_certificate_family_transport_contract_selftest_total_checks="9",
            age5_full_real_proof_certificate_family_transport_contract_selftest_checks_text=(
                "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,"
                "gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,"
                "report_index"
            ),
            age5_full_real_proof_certificate_family_transport_contract_selftest_current_probe="-",
            age5_full_real_proof_certificate_family_transport_contract_selftest_last_completed_probe=(
                "report_index"
            ),
            age5_full_real_proof_certificate_family_transport_contract_selftest_progress_present=True,
        )
    )
    expected_proof_family_progress_contract = (
        build_age5_full_real_proof_family_contract_selftest_progress(
            age5_full_real_proof_family_contract_selftest_completed_checks="3",
            age5_full_real_proof_family_contract_selftest_total_checks="3",
            age5_full_real_proof_family_contract_selftest_checks_text=(
                "proof_operation_family,proof_certificate_family,proof_family"
            ),
            age5_full_real_proof_family_contract_selftest_current_probe="-",
            age5_full_real_proof_family_contract_selftest_last_completed_probe="proof_family",
            age5_full_real_proof_family_contract_selftest_progress_present=True,
        )
    )
    expected_proof_family_transport_progress_contract = (
        build_age5_full_real_proof_family_transport_contract_selftest_progress(
            age5_full_real_proof_family_transport_contract_selftest_completed_checks="9",
            age5_full_real_proof_family_transport_contract_selftest_total_checks="9",
            age5_full_real_proof_family_transport_contract_selftest_checks_text=(
                "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,"
                "gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,"
                "report_index"
            ),
            age5_full_real_proof_family_transport_contract_selftest_current_probe="-",
            age5_full_real_proof_family_transport_contract_selftest_last_completed_probe="report_index",
            age5_full_real_proof_family_transport_contract_selftest_progress_present=True,
        )
    )
    expected_lang_surface_family_transport_progress_contract = (
        build_age5_full_real_lang_surface_family_transport_contract_selftest_progress(
            age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks="9",
            age5_full_real_lang_surface_family_transport_contract_selftest_total_checks="9",
            age5_full_real_lang_surface_family_transport_contract_selftest_checks_text=(
                "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,"
                "gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,"
                "report_index"
            ),
            age5_full_real_lang_surface_family_transport_contract_selftest_current_probe="-",
            age5_full_real_lang_surface_family_transport_contract_selftest_last_completed_probe=(
                "report_index"
            ),
            age5_full_real_lang_surface_family_transport_contract_selftest_progress_present=True,
        )
    )
    expected_lang_runtime_family_progress_contract = (
        build_age5_full_real_lang_runtime_family_contract_selftest_progress(
            age5_full_real_lang_runtime_family_contract_selftest_completed_checks="5",
            age5_full_real_lang_runtime_family_contract_selftest_total_checks="5",
            age5_full_real_lang_runtime_family_contract_selftest_checks_text=(
                "lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family"
            ),
            age5_full_real_lang_runtime_family_contract_selftest_current_probe="-",
            age5_full_real_lang_runtime_family_contract_selftest_last_completed_probe=(
                "lang_runtime_family"
            ),
            age5_full_real_lang_runtime_family_contract_selftest_progress_present=True,
        )
    )
    expected_lang_runtime_family_transport_progress_contract = (
        build_age5_full_real_lang_runtime_family_transport_contract_selftest_progress(
            age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks="9",
            age5_full_real_lang_runtime_family_transport_contract_selftest_total_checks="9",
            age5_full_real_lang_runtime_family_transport_contract_selftest_checks_text=(
                "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,"
                "gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,"
                "report_index"
            ),
            age5_full_real_lang_runtime_family_transport_contract_selftest_current_probe="-",
            age5_full_real_lang_runtime_family_transport_contract_selftest_last_completed_probe=(
                "report_index"
            ),
            age5_full_real_lang_runtime_family_transport_contract_selftest_progress_present=True,
        )
    )
    expected_gate0_family_progress_contract = (
        build_age5_full_real_gate0_family_contract_selftest_progress(
            age5_full_real_gate0_family_contract_selftest_completed_checks="5",
            age5_full_real_gate0_family_contract_selftest_total_checks="5",
            age5_full_real_gate0_family_contract_selftest_checks_text=(
                "gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family"
            ),
            age5_full_real_gate0_family_contract_selftest_current_probe="-",
            age5_full_real_gate0_family_contract_selftest_last_completed_probe="gate0_family",
            age5_full_real_gate0_family_contract_selftest_progress_present=True,
        )
    )
    expected_gate0_surface_family_progress_contract = (
        build_age5_full_real_gate0_surface_family_contract_selftest_progress(
            age5_full_real_gate0_surface_family_contract_selftest_completed_checks="5",
            age5_full_real_gate0_surface_family_contract_selftest_total_checks="5",
            age5_full_real_gate0_surface_family_contract_selftest_checks_text=(
                "lang_surface_family,lang_runtime_family,gate0_runtime_family,"
                "gate0_family,gate0_transport_family"
            ),
            age5_full_real_gate0_surface_family_contract_selftest_current_probe="-",
            age5_full_real_gate0_surface_family_contract_selftest_last_completed_probe=(
                "gate0_transport_family"
            ),
            age5_full_real_gate0_surface_family_contract_selftest_progress_present=True,
        )
    )
    expected_gate0_surface_family_transport_progress_contract = (
        build_age5_full_real_gate0_surface_family_transport_contract_selftest_progress(
            age5_full_real_gate0_surface_family_transport_contract_selftest_completed_checks="9",
            age5_full_real_gate0_surface_family_transport_contract_selftest_total_checks="9",
            age5_full_real_gate0_surface_family_transport_contract_selftest_checks_text=(
                "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,"
                "gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,"
                "report_index"
            ),
            age5_full_real_gate0_surface_family_transport_contract_selftest_current_probe="-",
            age5_full_real_gate0_surface_family_transport_contract_selftest_last_completed_probe=(
                "report_index"
            ),
            age5_full_real_gate0_surface_family_transport_contract_selftest_progress_present=True,
        )
    )
    expected_gate0_family_transport_progress_contract = (
        build_age5_full_real_gate0_family_transport_contract_selftest_progress(
            age5_full_real_gate0_family_transport_contract_selftest_completed_checks="9",
            age5_full_real_gate0_family_transport_contract_selftest_total_checks="9",
            age5_full_real_gate0_family_transport_contract_selftest_checks_text=(
                "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,"
                "gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,"
                "report_index"
            ),
            age5_full_real_gate0_family_transport_contract_selftest_current_probe="-",
            age5_full_real_gate0_family_transport_contract_selftest_last_completed_probe=(
                "report_index"
            ),
            age5_full_real_gate0_family_transport_contract_selftest_progress_present=True,
        )
    )
    expected_gate0_transport_family_progress_contract = (
        build_age5_full_real_gate0_transport_family_contract_selftest_progress(
            age5_full_real_gate0_transport_family_contract_selftest_completed_checks="4",
            age5_full_real_gate0_transport_family_contract_selftest_total_checks="4",
            age5_full_real_gate0_transport_family_contract_selftest_checks_text=(
                "lang_runtime_family_transport,gate0_runtime_family_transport,"
                "gate0_family_transport,gate0_transport_family"
            ),
            age5_full_real_gate0_transport_family_contract_selftest_current_probe="-",
            age5_full_real_gate0_transport_family_contract_selftest_last_completed_probe=(
                "gate0_transport_family"
            ),
            age5_full_real_gate0_transport_family_contract_selftest_progress_present=True,
        )
    )
    expected_gate0_transport_family_transport_progress_contract = (
        build_age5_full_real_gate0_transport_family_transport_contract_selftest_progress(
            age5_full_real_gate0_transport_family_transport_contract_selftest_completed_checks="9",
            age5_full_real_gate0_transport_family_transport_contract_selftest_total_checks="9",
            age5_full_real_gate0_transport_family_transport_contract_selftest_checks_text=(
                "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,"
                "gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,"
                "report_index"
            ),
            age5_full_real_gate0_transport_family_transport_contract_selftest_current_probe="-",
            age5_full_real_gate0_transport_family_transport_contract_selftest_last_completed_probe=(
                "report_index"
            ),
            age5_full_real_gate0_transport_family_transport_contract_selftest_progress_present=True,
        )
    )
    expected_gate0_runtime_family_transport_progress_contract = (
        build_age5_full_real_gate0_runtime_family_transport_contract_selftest_progress(
            age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks="9",
            age5_full_real_gate0_runtime_family_transport_contract_selftest_total_checks="9",
            age5_full_real_gate0_runtime_family_transport_contract_selftest_checks_text=(
                "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,"
                "gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,"
                "report_index"
            ),
            age5_full_real_gate0_runtime_family_transport_contract_selftest_current_probe="-",
            age5_full_real_gate0_runtime_family_transport_contract_selftest_last_completed_probe=(
                "report_index"
            ),
            age5_full_real_gate0_runtime_family_transport_contract_selftest_progress_present=True,
        )
    )
    expected_bogae_alias_family_progress_contract = (
        build_age5_full_real_bogae_alias_family_contract_selftest_progress(
            age5_full_real_bogae_alias_family_contract_selftest_completed_checks="3",
            age5_full_real_bogae_alias_family_contract_selftest_total_checks="3",
            age5_full_real_bogae_alias_family_contract_selftest_checks_text=(
                "shape_alias_contract,alias_family,alias_viewer_family"
            ),
            age5_full_real_bogae_alias_family_contract_selftest_current_probe="-",
            age5_full_real_bogae_alias_family_contract_selftest_last_completed_probe=(
                "alias_viewer_family"
            ),
            age5_full_real_bogae_alias_family_contract_selftest_progress_present=True,
        )
    )
    expected_bogae_alias_family_transport_progress_contract = (
        build_age5_full_real_bogae_alias_family_transport_contract_selftest_progress(
            age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks="9",
            age5_full_real_bogae_alias_family_transport_contract_selftest_total_checks="9",
            age5_full_real_bogae_alias_family_transport_contract_selftest_checks_text=(
                "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,"
                "gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,"
                "report_index"
            ),
            age5_full_real_bogae_alias_family_transport_contract_selftest_current_probe="-",
            age5_full_real_bogae_alias_family_transport_contract_selftest_last_completed_probe=(
                "report_index"
            ),
            age5_full_real_bogae_alias_family_transport_contract_selftest_progress_present=True,
        )
    )
    w107_detail = (
        "ci_profile_matrix_full_real_smoke_status=pass "
        "w107_golden_index_selftest_active_cases=54 "
        "w107_golden_index_selftest_inactive_cases=1 "
        "w107_golden_index_selftest_index_codes=34 "
        "w107_golden_index_selftest_current_probe=- "
        "w107_golden_index_selftest_last_completed_probe=validate_pack_pointers "
        "w107_progress_contract_selftest_completed_checks=8 "
        "w107_progress_contract_selftest_total_checks=8 "
        "w107_progress_contract_selftest_checks_text=golden_index,age5_close_transport,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,final_line_emitter,report_index "
        "w107_progress_contract_selftest_current_probe=- "
        "w107_progress_contract_selftest_last_completed_probe=report_index "
        "age1_immediate_proof_operation_contract_selftest_completed_checks=5 "
        "age1_immediate_proof_operation_contract_selftest_total_checks=5 "
        "age1_immediate_proof_operation_contract_selftest_checks_text=operation_matrix,solver_search_matrix,solver_search_parity,solver_operation_family,proof_operation_family "
        "age1_immediate_proof_operation_contract_selftest_current_probe=- "
        "age1_immediate_proof_operation_contract_selftest_last_completed_probe=proof_operation_family "
        "proof_certificate_v1_consumer_transport_contract_selftest_completed_checks=5 "
        "proof_certificate_v1_consumer_transport_contract_selftest_total_checks=5 "
        "proof_certificate_v1_consumer_transport_contract_selftest_checks_text=signed_emit_profiles,verify_bundle,verify_report,verify_report_digest_contract,consumer_contract,signed_contract "
        "proof_certificate_v1_consumer_transport_contract_selftest_current_probe=- "
        "proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe=signed_contract "
        "proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks=1 "
        "proof_certificate_v1_verify_report_digest_contract_selftest_total_checks=1 "
        "proof_certificate_v1_verify_report_digest_contract_selftest_checks_text=verify_report_digest_contract "
        "proof_certificate_v1_verify_report_digest_contract_selftest_current_probe=- "
        "proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe=readme_and_field_contract "
        "proof_certificate_v1_family_contract_selftest_completed_checks=4 "
        "proof_certificate_v1_family_contract_selftest_total_checks=4 "
        "proof_certificate_v1_family_contract_selftest_checks_text=signed_contract,consumer_contract,promotion,family "
        "proof_certificate_v1_family_contract_selftest_current_probe=- "
        "proof_certificate_v1_family_contract_selftest_last_completed_probe=family "
        "proof_certificate_family_contract_selftest_completed_checks=3 "
        "proof_certificate_family_contract_selftest_total_checks=3 "
        "proof_certificate_family_contract_selftest_checks_text=artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family "
        "proof_certificate_family_contract_selftest_current_probe=- "
        "proof_certificate_family_contract_selftest_last_completed_probe=proof_certificate_family "
        "proof_certificate_family_transport_contract_selftest_completed_checks=9 "
        "proof_certificate_family_transport_contract_selftest_total_checks=9 "
        "proof_certificate_family_transport_contract_selftest_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
        "proof_certificate_family_transport_contract_selftest_current_probe=- "
        "proof_certificate_family_transport_contract_selftest_last_completed_probe=report_index "
        "proof_family_contract_selftest_completed_checks=3 "
        "proof_family_contract_selftest_total_checks=3 "
        "proof_family_contract_selftest_checks_text=proof_operation_family,proof_certificate_family,proof_family "
        "proof_family_contract_selftest_current_probe=- "
        "proof_family_contract_selftest_last_completed_probe=proof_family "
        "proof_family_transport_contract_selftest_completed_checks=9 "
        "proof_family_transport_contract_selftest_total_checks=9 "
        "proof_family_transport_contract_selftest_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
        "proof_family_transport_contract_selftest_current_probe=- "
        "proof_family_transport_contract_selftest_last_completed_probe=report_index "
        "lang_surface_family_contract_selftest_completed_checks=4 "
        "lang_surface_family_contract_selftest_total_checks=4 "
        "lang_surface_family_contract_selftest_checks_text=proof_family,bogae_alias_family,compound_update_reject_contract,lang_surface_family "
        "lang_surface_family_contract_selftest_current_probe=- "
        "lang_surface_family_contract_selftest_last_completed_probe=lang_surface_family "
        "lang_surface_family_transport_contract_selftest_completed_checks=9 "
        "lang_surface_family_transport_contract_selftest_total_checks=9 "
        "lang_surface_family_transport_contract_selftest_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
        "lang_surface_family_transport_contract_selftest_current_probe=- "
        "lang_surface_family_transport_contract_selftest_last_completed_probe=report_index "
        "lang_runtime_family_contract_selftest_completed_checks=5 "
        "lang_runtime_family_contract_selftest_total_checks=5 "
        "lang_runtime_family_contract_selftest_checks_text=lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family "
        "lang_runtime_family_contract_selftest_current_probe=- "
        "lang_runtime_family_contract_selftest_last_completed_probe=lang_runtime_family "
        "gate0_family_contract_selftest_completed_checks=5 "
        "gate0_family_contract_selftest_total_checks=5 "
        "gate0_family_contract_selftest_checks_text=gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family "
        "gate0_family_contract_selftest_current_probe=- "
        "gate0_family_contract_selftest_last_completed_probe=gate0_family "
        "gate0_surface_family_contract_selftest_completed_checks=5 "
        "gate0_surface_family_contract_selftest_total_checks=5 "
        "gate0_surface_family_contract_selftest_checks_text=lang_surface_family,lang_runtime_family,gate0_runtime_family,gate0_family,gate0_transport_family "
        "gate0_surface_family_contract_selftest_current_probe=- "
        "gate0_surface_family_contract_selftest_last_completed_probe=gate0_transport_family "
        "gate0_surface_family_transport_contract_selftest_completed_checks=9 "
        "gate0_surface_family_transport_contract_selftest_total_checks=9 "
        "gate0_surface_family_transport_contract_selftest_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
        "gate0_surface_family_transport_contract_selftest_current_probe=- "
        "gate0_surface_family_transport_contract_selftest_last_completed_probe=report_index "
        "lang_runtime_family_transport_contract_selftest_completed_checks=9 "
        "lang_runtime_family_transport_contract_selftest_total_checks=9 "
        "lang_runtime_family_transport_contract_selftest_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
        "lang_runtime_family_transport_contract_selftest_current_probe=- "
        "lang_runtime_family_transport_contract_selftest_last_completed_probe=report_index "
        "gate0_family_transport_contract_selftest_completed_checks=9 "
        "gate0_family_transport_contract_selftest_total_checks=9 "
        "gate0_family_transport_contract_selftest_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
        "gate0_family_transport_contract_selftest_current_probe=- "
        "gate0_family_transport_contract_selftest_last_completed_probe=report_index "
        "gate0_runtime_family_transport_contract_selftest_completed_checks=9 "
        "gate0_runtime_family_transport_contract_selftest_total_checks=9 "
        "gate0_runtime_family_transport_contract_selftest_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
        "gate0_runtime_family_transport_contract_selftest_current_probe=- "
        "gate0_runtime_family_transport_contract_selftest_last_completed_probe=report_index "
        "gate0_transport_family_contract_selftest_completed_checks=4 "
        "gate0_transport_family_contract_selftest_total_checks=4 "
        "gate0_transport_family_contract_selftest_checks_text=lang_runtime_family_transport,gate0_runtime_family_transport,gate0_family_transport,gate0_transport_family "
        "gate0_transport_family_contract_selftest_current_probe=- "
        "gate0_transport_family_contract_selftest_last_completed_probe=gate0_transport_family "
        "gate0_transport_family_transport_contract_selftest_completed_checks=9 "
        "gate0_transport_family_transport_contract_selftest_total_checks=9 "
        "gate0_transport_family_transport_contract_selftest_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
        "gate0_transport_family_transport_contract_selftest_current_probe=- "
        "gate0_transport_family_transport_contract_selftest_last_completed_probe=report_index "
        "bogae_alias_family_contract_selftest_completed_checks=3 "
        "bogae_alias_family_contract_selftest_total_checks=3 "
        "bogae_alias_family_contract_selftest_checks_text=shape_alias_contract,alias_family,alias_viewer_family "
        "bogae_alias_family_contract_selftest_current_probe=- "
        "bogae_alias_family_contract_selftest_last_completed_probe=alias_viewer_family "
        "bogae_alias_family_transport_contract_selftest_completed_checks=9 "
        "bogae_alias_family_transport_contract_selftest_total_checks=9 "
        "bogae_alias_family_transport_contract_selftest_checks_text=family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index "
        "bogae_alias_family_transport_contract_selftest_current_probe=- "
        "bogae_alias_family_transport_contract_selftest_last_completed_probe=report_index"
    )

    temp_child_report = root / "build" / "tmp" / "age5_close_combined_report_contract_selftest.child.detjson"
    write_json(
        temp_child_report,
        {
            "schema": "ddn.age5_close_report.v1",
            "overall_ok": True,
            "criteria": [
                {"name": "age5_ci_profile_matrix_full_real_smoke_optin_pass", "ok": True},
            ],
        },
    )
    if not mod.cached_age5_close_child_report_ok(
        temp_child_report,
        "age5_ci_profile_matrix_full_real_smoke_optin_pass",
    ):
        return fail("cached child report positive contract mismatch")
    if mod.cached_age5_close_child_report_ok(
        temp_child_report,
        "age5_ci_profile_core_lang_runtime_helper_negative_optin_pass",
    ):
        return fail("cached child report wrong criterion should fail")
    standalone_full_real_report = mod.build_age5_close_report(
        strict=False,
        with_profile_matrix_full_real_smoke_check=True,
        with_runtime_helper_mismatch_negative_check=False,
        with_group_id_summary_mismatch_negative_check=False,
        with_combined_heavy_runtime_helper_check=False,
        combined_heavy_env_enabled=False,
        criteria=[
            {
                "name": "age5_ci_profile_matrix_full_real_smoke_optin_pass",
                "ok": True,
                "detail": w107_detail,
            }
        ],
        failure_digest=[],
        pending_items=[],
        repair={},
    )
    rc = check_w107_progress(
        standalone_full_real_report,
        expected_w107_progress,
        label="standalone full-real",
    )
    if rc != 0:
        return rc
    write_json(full_real_report, standalone_full_real_report)

    pass_report = mod.build_age5_combined_heavy_optin_report(
        root=root,
        strict=True,
        combined_heavy_env_enabled=True,
        full_real_cmd=full_real_cmd,
        full_real_proc=completed(full_real_cmd, 0, "ci_profile_matrix_full_real_smoke_status=pass"),
        full_real_report=full_real_report,
        runtime_helper_negative_cmd=runtime_helper_cmd,
        runtime_helper_negative_proc=completed(
            runtime_helper_cmd, 0, "ci_profile_core_lang_status=fail reason=aggregate_summary_runtime_helper_contract_mismatch"
        ),
        runtime_helper_negative_report=runtime_helper_report,
        group_id_summary_negative_cmd=group_id_cmd,
        group_id_summary_negative_proc=completed(
            group_id_cmd, 0, "ci_profile_core_lang_status=fail reason=aggregate_summary_group_id_summary_mismatch"
        ),
        group_id_summary_negative_report=group_id_report,
        age4_proof_snapshot=build_age4_proof_snapshot(
            age4_proof_ok="1",
            age4_proof_failed_criteria="0",
            age4_proof_failed_preview="-",
        ),
        age4_proof_source_fields=build_age4_proof_source_snapshot_fields(
            top_snapshot=build_age4_proof_snapshot(
                age4_proof_ok="1",
                age4_proof_failed_criteria="0",
                age4_proof_failed_preview="-",
            ),
            gate_result_snapshot=build_age4_proof_snapshot(
                age4_proof_ok="1",
                age4_proof_failed_criteria="0",
                age4_proof_failed_preview="-",
            ),
            gate_result_present=True,
            final_status_parse_snapshot=build_age4_proof_snapshot(
                age4_proof_ok="1",
                age4_proof_failed_criteria="0",
                age4_proof_failed_preview="-",
            ),
            final_status_parse_present=True,
        ),
    )
    rc = check_report_contract(
        pass_report,
        env_enabled=True,
        overall_ok=True,
        expected_age4_proof_snapshot=build_age4_proof_snapshot(
            age4_proof_ok="1",
            age4_proof_failed_criteria="0",
            age4_proof_failed_preview="-",
        ),
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
        expected_lang_surface_family_transport_progress_contract=(
            expected_lang_surface_family_transport_progress_contract
        ),
        expected_lang_runtime_family_transport_progress_contract=(
            expected_lang_runtime_family_transport_progress_contract
        ),
    )
    if rc != 0:
        return rc

    fail_report = mod.build_age5_combined_heavy_optin_report(
        root=root,
        strict=False,
        combined_heavy_env_enabled=False,
        full_real_cmd=full_real_cmd,
        full_real_proc=completed(full_real_cmd, 0, "ci_profile_matrix_full_real_smoke_status=pass"),
        full_real_report=full_real_report,
        runtime_helper_negative_cmd=runtime_helper_cmd,
        runtime_helper_negative_proc=completed(
            runtime_helper_cmd, 1, "ci_profile_core_lang_status=fail reason=aggregate_summary_runtime_helper_contract_mismatch"
        ),
        runtime_helper_negative_report=runtime_helper_report,
        group_id_summary_negative_cmd=group_id_cmd,
        group_id_summary_negative_proc=completed(
            group_id_cmd, 0, "ci_profile_core_lang_status=fail reason=aggregate_summary_group_id_summary_mismatch"
        ),
        group_id_summary_negative_report=group_id_report,
        age4_proof_source_fields=build_age4_proof_source_snapshot_fields(
            top_snapshot=build_age4_proof_snapshot(),
            gate_result_snapshot=build_age4_proof_snapshot(),
            gate_result_present=True,
            final_status_parse_snapshot=build_age4_proof_snapshot(),
            final_status_parse_present=True,
        ),
    )
    rc = check_report_contract(
        fail_report,
        env_enabled=False,
        overall_ok=False,
        expected_age4_proof_snapshot=build_age4_proof_snapshot(),
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
        expected_lang_surface_family_transport_progress_contract=(
            expected_lang_surface_family_transport_progress_contract
        ),
        expected_lang_runtime_family_transport_progress_contract=(
            expected_lang_runtime_family_transport_progress_contract
        ),
    )
    if rc != 0:
        return rc

    # Negative contract cases are expected to fail inside check_report_contract().
    # Mute those internal fail logs to keep CI output signal clean.
    set_fail_log_muted(True)

    transport_fail_report = dict(pass_report)
    transport_fail_report["ci_sanity_age5_combined_heavy_full_summary_contract_fields"] = "BROKEN"
    rc = check_report_contract(
        transport_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
    )
    if rc == 0:
        return fail("transport mismatch case should fail")

    default_transport_fail_report = dict(pass_report)
    default_transport_fail_report["ci_sanity_age5_combined_heavy_child_summary_default_fields"] = "BROKEN"
    rc = check_report_contract(
        default_transport_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
    )
    if rc == 0:
        return fail("default transport mismatch case should fail")

    child_fail_report = dict(pass_report)
    child_fail_report["age5_combined_heavy_full_real_status"] = "fail"
    rc = check_report_contract(
        child_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
    )
    if rc == 0:
        return fail("child summary mismatch case should fail")

    age4_source_fail_report = dict(pass_report)
    age4_source_fail_report[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY] = "0"
    rc = check_report_contract(
        age4_source_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
    )
    if rc == 0:
        return fail("age4 proof source parity mismatch case should fail")

    w107_progress_fail_report = dict(pass_report)
    w107_progress_fail_report["age5_full_real_w107_golden_index_selftest_active_cases"] = "999"
    rc = check_report_contract(
        w107_progress_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
    )
    if rc == 0:
        return fail("w107 progress mismatch case should fail")

    w107_progress_contract_fail_report = dict(pass_report)
    w107_progress_contract_fail_report["age5_full_real_w107_progress_contract_selftest_completed_checks"] = "999"
    rc = check_report_contract(
        w107_progress_contract_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
    )
    if rc == 0:
        return fail("w107 progress-contract mismatch case should fail")

    age1_immediate_proof_operation_progress_contract_fail_report = dict(pass_report)
    age1_immediate_proof_operation_progress_contract_fail_report[
        "age5_full_real_age1_immediate_proof_operation_contract_selftest_completed_checks"
    ] = "999"
    rc = check_report_contract(
        age1_immediate_proof_operation_progress_contract_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
    )
    if rc == 0:
        return fail("age1 immediate proof operation progress-contract mismatch case should fail")

    proof_certificate_v1_consumer_transport_progress_contract_fail_report = dict(pass_report)
    proof_certificate_v1_consumer_transport_progress_contract_fail_report[
        "age5_full_real_proof_certificate_v1_consumer_transport_contract_selftest_completed_checks"
    ] = "999"
    rc = check_report_contract(
        proof_certificate_v1_consumer_transport_progress_contract_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
    )
    if rc == 0:
        return fail("proof_certificate_v1 consumer transport progress-contract mismatch case should fail")

    proof_certificate_v1_verify_report_digest_progress_contract_fail_report = dict(pass_report)
    proof_certificate_v1_verify_report_digest_progress_contract_fail_report[
        "age5_full_real_proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks"
    ] = "999"
    rc = check_report_contract(
        proof_certificate_v1_verify_report_digest_progress_contract_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
    )
    if rc == 0:
        return fail("proof_certificate_v1 verify-report digest progress-contract mismatch case should fail")

    proof_certificate_v1_family_progress_contract_fail_report = dict(pass_report)
    proof_certificate_v1_family_progress_contract_fail_report[
        "age5_full_real_proof_certificate_v1_family_contract_selftest_completed_checks"
    ] = "999"
    rc = check_report_contract(
        proof_certificate_v1_family_progress_contract_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
    )
    if rc == 0:
        return fail("proof_certificate_v1 family progress-contract mismatch case should fail")

    proof_certificate_family_progress_contract_fail_report = dict(pass_report)
    proof_certificate_family_progress_contract_fail_report[
        "age5_full_real_proof_certificate_family_contract_selftest_completed_checks"
    ] = "999"
    rc = check_report_contract(
        proof_certificate_family_progress_contract_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
    )
    if rc == 0:
        return fail("proof_certificate family progress-contract mismatch case should fail")

    proof_certificate_family_transport_progress_contract_fail_report = dict(pass_report)
    proof_certificate_family_transport_progress_contract_fail_report[
        "age5_full_real_proof_certificate_family_transport_contract_selftest_completed_checks"
    ] = "999"
    rc = check_report_contract(
        proof_certificate_family_transport_progress_contract_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
        expected_proof_certificate_family_transport_progress_contract=(
            expected_proof_certificate_family_transport_progress_contract
        ),
        expected_proof_family_progress_contract=expected_proof_family_progress_contract,
    )
    if rc == 0:
        return fail("proof_certificate family transport progress-contract mismatch case should fail")

    lang_surface_family_transport_progress_contract_fail_report = dict(pass_report)
    lang_surface_family_transport_progress_contract_fail_report[
        "age5_full_real_lang_surface_family_transport_contract_selftest_completed_checks"
    ] = "999"
    rc = check_report_contract(
        lang_surface_family_transport_progress_contract_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
        expected_proof_certificate_family_transport_progress_contract=(
            expected_proof_certificate_family_transport_progress_contract
        ),
        expected_proof_family_progress_contract=expected_proof_family_progress_contract,
        expected_proof_family_transport_progress_contract=(
            expected_proof_family_transport_progress_contract
        ),
        expected_lang_surface_family_transport_progress_contract=(
            expected_lang_surface_family_transport_progress_contract
        ),
        expected_lang_runtime_family_transport_progress_contract=(
            expected_lang_runtime_family_transport_progress_contract
        ),
    )
    if rc == 0:
        return fail("lang surface family transport progress-contract mismatch case should fail")

    lang_runtime_family_transport_progress_contract_fail_report = dict(pass_report)
    lang_runtime_family_transport_progress_contract_fail_report[
        "age5_full_real_lang_runtime_family_transport_contract_selftest_completed_checks"
    ] = "999"
    rc = check_report_contract(
        lang_runtime_family_transport_progress_contract_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
        expected_proof_certificate_family_transport_progress_contract=(
            expected_proof_certificate_family_transport_progress_contract
        ),
        expected_proof_family_progress_contract=expected_proof_family_progress_contract,
        expected_proof_family_transport_progress_contract=(
            expected_proof_family_transport_progress_contract
        ),
        expected_lang_surface_family_transport_progress_contract=(
            expected_lang_surface_family_transport_progress_contract
        ),
        expected_lang_runtime_family_transport_progress_contract=(
            expected_lang_runtime_family_transport_progress_contract
        ),
    )
    if rc == 0:
        return fail("lang runtime family transport progress-contract mismatch case should fail")

    gate0_family_progress_contract_fail_report = dict(pass_report)
    gate0_family_progress_contract_fail_report[
        "age5_full_real_gate0_family_contract_selftest_completed_checks"
    ] = "999"
    rc = check_report_contract(
        gate0_family_progress_contract_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
        expected_proof_certificate_family_transport_progress_contract=(
            expected_proof_certificate_family_transport_progress_contract
        ),
        expected_proof_family_progress_contract=expected_proof_family_progress_contract,
        expected_proof_family_transport_progress_contract=(
            expected_proof_family_transport_progress_contract
        ),
        expected_lang_surface_family_transport_progress_contract=(
            expected_lang_surface_family_transport_progress_contract
        ),
        expected_lang_runtime_family_transport_progress_contract=(
            expected_lang_runtime_family_transport_progress_contract
        ),
        expected_gate0_family_progress_contract=expected_gate0_family_progress_contract,
        expected_gate0_surface_family_progress_contract=(
            expected_gate0_surface_family_progress_contract
        ),
        expected_gate0_surface_family_transport_progress_contract=(
            expected_gate0_surface_family_transport_progress_contract
        ),
        expected_gate0_family_transport_progress_contract=(
            expected_gate0_family_transport_progress_contract
        ),
        expected_gate0_transport_family_progress_contract=(
            expected_gate0_transport_family_progress_contract
        ),
        expected_gate0_transport_family_transport_progress_contract=(
            expected_gate0_transport_family_transport_progress_contract
        ),
    )
    if rc == 0:
        return fail("gate0 family progress-contract mismatch case should fail")

    gate0_family_transport_progress_contract_fail_report = dict(pass_report)
    gate0_family_transport_progress_contract_fail_report[
        "age5_full_real_gate0_family_transport_contract_selftest_completed_checks"
    ] = "999"
    rc = check_report_contract(
        gate0_family_transport_progress_contract_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
        expected_proof_certificate_family_transport_progress_contract=(
            expected_proof_certificate_family_transport_progress_contract
        ),
        expected_proof_family_progress_contract=expected_proof_family_progress_contract,
        expected_proof_family_transport_progress_contract=(
            expected_proof_family_transport_progress_contract
        ),
        expected_lang_surface_family_transport_progress_contract=(
            expected_lang_surface_family_transport_progress_contract
        ),
        expected_lang_runtime_family_transport_progress_contract=(
            expected_lang_runtime_family_transport_progress_contract
        ),
        expected_gate0_family_progress_contract=expected_gate0_family_progress_contract,
        expected_gate0_surface_family_progress_contract=(
            expected_gate0_surface_family_progress_contract
        ),
        expected_gate0_surface_family_transport_progress_contract=(
            expected_gate0_surface_family_transport_progress_contract
        ),
        expected_gate0_family_transport_progress_contract=(
            expected_gate0_family_transport_progress_contract
        ),
        expected_gate0_transport_family_progress_contract=(
            expected_gate0_transport_family_progress_contract
        ),
        expected_gate0_transport_family_transport_progress_contract=(
            expected_gate0_transport_family_transport_progress_contract
        ),
    )
    if rc == 0:
        return fail("gate0 family transport progress-contract mismatch case should fail")

    gate0_transport_family_progress_contract_fail_report = dict(pass_report)
    gate0_transport_family_progress_contract_fail_report[
        "age5_full_real_gate0_transport_family_contract_selftest_completed_checks"
    ] = "999"
    rc = check_report_contract(
        gate0_transport_family_progress_contract_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
        expected_proof_certificate_family_transport_progress_contract=(
            expected_proof_certificate_family_transport_progress_contract
        ),
        expected_proof_family_progress_contract=expected_proof_family_progress_contract,
        expected_proof_family_transport_progress_contract=(
            expected_proof_family_transport_progress_contract
        ),
        expected_lang_surface_family_transport_progress_contract=(
            expected_lang_surface_family_transport_progress_contract
        ),
        expected_lang_runtime_family_transport_progress_contract=(
            expected_lang_runtime_family_transport_progress_contract
        ),
        expected_gate0_transport_family_progress_contract=(
            expected_gate0_transport_family_progress_contract
        ),
        expected_gate0_transport_family_transport_progress_contract=(
            expected_gate0_transport_family_transport_progress_contract
        ),
    )
    if rc == 0:
        return fail("gate0 transport family progress-contract mismatch case should fail")

    gate0_runtime_family_transport_progress_contract_fail_report = dict(pass_report)
    gate0_runtime_family_transport_progress_contract_fail_report[
        "age5_full_real_gate0_runtime_family_transport_contract_selftest_completed_checks"
    ] = "999"
    rc = check_report_contract(
        gate0_runtime_family_transport_progress_contract_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
        expected_proof_certificate_family_transport_progress_contract=(
            expected_proof_certificate_family_transport_progress_contract
        ),
        expected_proof_family_progress_contract=expected_proof_family_progress_contract,
        expected_proof_family_transport_progress_contract=(
            expected_proof_family_transport_progress_contract
        ),
        expected_lang_surface_family_transport_progress_contract=(
            expected_lang_surface_family_transport_progress_contract
        ),
        expected_lang_runtime_family_transport_progress_contract=(
            expected_lang_runtime_family_transport_progress_contract
        ),
        expected_gate0_transport_family_progress_contract=(
            expected_gate0_transport_family_progress_contract
        ),
        expected_gate0_transport_family_transport_progress_contract=(
            expected_gate0_transport_family_transport_progress_contract
        ),
        expected_gate0_runtime_family_transport_progress_contract=(
            expected_gate0_runtime_family_transport_progress_contract
        ),
    )
    if rc == 0:
        return fail("gate0 runtime family transport progress-contract mismatch case should fail")

    bogae_alias_family_progress_contract_fail_report = dict(pass_report)
    bogae_alias_family_progress_contract_fail_report[
        "age5_full_real_bogae_alias_family_contract_selftest_completed_checks"
    ] = "999"
    rc = check_report_contract(
        bogae_alias_family_progress_contract_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
        expected_proof_certificate_family_transport_progress_contract=(
            expected_proof_certificate_family_transport_progress_contract
        ),
        expected_proof_family_progress_contract=expected_proof_family_progress_contract,
        expected_bogae_alias_family_progress_contract=(
            expected_bogae_alias_family_progress_contract
        ),
    )
    if rc == 0:
        return fail("bogae alias family progress-contract mismatch case should fail")

    bogae_alias_family_transport_progress_contract_fail_report = dict(pass_report)
    bogae_alias_family_transport_progress_contract_fail_report[
        "age5_full_real_bogae_alias_family_transport_contract_selftest_completed_checks"
    ] = "999"
    rc = check_report_contract(
        bogae_alias_family_transport_progress_contract_fail_report,
        env_enabled=True,
        overall_ok=True,
        expected_w107_progress=expected_w107_progress,
        expected_w107_progress_contract=expected_w107_progress_contract,
        expected_age1_immediate_proof_operation_progress_contract=(
            expected_age1_immediate_proof_operation_progress_contract
        ),
        expected_proof_certificate_v1_consumer_transport_progress_contract=(
            expected_proof_certificate_v1_consumer_transport_progress_contract
        ),
        expected_proof_certificate_v1_verify_report_digest_progress_contract=(
            expected_proof_certificate_v1_verify_report_digest_progress_contract
        ),
        expected_proof_certificate_v1_family_progress_contract=(
            expected_proof_certificate_v1_family_progress_contract
        ),
        expected_proof_certificate_family_progress_contract=(
            expected_proof_certificate_family_progress_contract
        ),
        expected_proof_certificate_family_transport_progress_contract=(
            expected_proof_certificate_family_transport_progress_contract
        ),
        expected_proof_family_progress_contract=expected_proof_family_progress_contract,
        expected_proof_family_transport_progress_contract=(
            expected_proof_family_transport_progress_contract
        ),
        expected_bogae_alias_family_progress_contract=(
            expected_bogae_alias_family_progress_contract
        ),
        expected_bogae_alias_family_transport_progress_contract=(
            expected_bogae_alias_family_transport_progress_contract
        ),
    )
    if rc == 0:
        return fail("bogae alias family transport progress-contract mismatch case should fail")

    set_fail_log_muted(False)
    print("[age5-close-combined-report-contract-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
