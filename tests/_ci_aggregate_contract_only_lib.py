from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path

from _ci_age3_completion_gate_contract import (
    AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS,
    AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS,
)
from _ci_age5_combined_heavy_contract import (
    AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
    AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY,
    AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY,
    build_age5_combined_heavy_sanity_contract_fields,
    build_age5_combined_heavy_sync_contract_fields,
    build_age5_combined_heavy_child_summary_default_fields,
    build_age5_combined_heavy_child_summary_default_text_transport_fields,
    build_age5_full_real_w107_golden_index_selftest_progress,
    build_age5_full_real_w107_progress_contract_selftest_progress,
    build_age5_combined_heavy_policy_origin_trace,
    build_age5_combined_heavy_policy_origin_trace_text,
    build_age5_close_digest_selftest_default_field,
    build_age4_proof_snapshot,
    build_age4_proof_snapshot_text,
)
from _ci_aggregate_diag_specs_seamgrim import SEAMGRIM_FOCUS_STEP_SPECS
from _ci_seamgrim_step_contract import (
    SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS,
    merge_step_names,
)
from _ci_profile_matrix_selftest_lib import (
    build_profile_matrix_snapshot_from_doc,
    build_profile_matrix_triage_payload_from_snapshot,
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC,
    PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
    PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS,
    expected_profile_matrix_aggregate_summary_contract,
    format_profile_matrix_summary_values,
)
from run_ci_gate_report_index_check import (
    AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_BRIEF_KEY_MAP,
    AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_BRIEF_KEY_MAP,
    AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
    AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
    AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
    AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
    AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_BRIEF_KEY_MAP,
    AGE5_LANG_RUNTIME_FAMILY_CONTRACT_BRIEF_KEY_MAP,
    AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
    AGE5_LANG_SURFACE_FAMILY_CONTRACT_BRIEF_KEY_MAP,
    AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
    AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_BRIEF_KEY_MAP,
    AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
    AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
    AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_BRIEF_KEY_MAP,
    AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_BRIEF_KEY_MAP,
    AGE5_PROOF_FAMILY_CONTRACT_BRIEF_KEY_MAP,
    AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
    AGE5_W107_BRIEF_KEY_MAP,
    AGE5_W107_CONTRACT_BRIEF_KEY_MAP,
)
from run_ci_gate_result_check import (
    AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS,
    AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS,
    AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS,
    AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS,
    AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS,
    AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS,
    AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS,
    AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS,
    AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS,
    AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS,
    AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_W107_CONTRACT_PROGRESS_KEYS,
    AGE5_W107_PROGRESS_KEYS,
)


CONTRACT_ONLY_SEAMGRIM_STEP_NAMES = ("control_exposure_policy",) + tuple(
    step_name for _, step_name in SEAMGRIM_FOCUS_STEP_SPECS
)
CONTRACT_ONLY_AGE4_PROOF_OK = "1"
CONTRACT_ONLY_AGE4_PROOF_FAILED_CRITERIA = "0"
CONTRACT_ONLY_AGE4_PROOF_FAILED_PREVIEW = "-"
CONTRACT_ONLY_AGE4_PROOF_SUMMARY_HASH = "sha256:contract-only-age4-proof-summary"
AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY = "age5_policy_age4_proof_snapshot_fields_text"
AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY = "age5_policy_age4_proof_snapshot_text"
AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY = "age5_policy_age4_proof_source_snapshot_fields_text"
AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY = "age5_policy_age4_proof_gate_result_present"
AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY = "age5_policy_age4_proof_gate_result_parity"
AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY = "age5_policy_age4_proof_final_status_parse_present"
AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY = "age5_policy_age4_proof_final_status_parse_parity"
AGE5_PROGRESS_BRIEF_KEY_MAPS = (
    AGE5_W107_BRIEF_KEY_MAP,
    AGE5_W107_CONTRACT_BRIEF_KEY_MAP,
    AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_BRIEF_KEY_MAP,
    AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
    AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_BRIEF_KEY_MAP,
    AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_BRIEF_KEY_MAP,
    AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_BRIEF_KEY_MAP,
    AGE5_PROOF_FAMILY_CONTRACT_BRIEF_KEY_MAP,
    AGE5_LANG_SURFACE_FAMILY_CONTRACT_BRIEF_KEY_MAP,
    AGE5_LANG_RUNTIME_FAMILY_CONTRACT_BRIEF_KEY_MAP,
    AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
    AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
    AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
    AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
    AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_BRIEF_KEY_MAP,
    AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
    AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
    AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
    AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_BRIEF_KEY_MAP,
    AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_BRIEF_KEY_MAP,
)
AGE5_PROGRESS_KEY_GROUPS = (
    AGE5_W107_PROGRESS_KEYS,
    AGE5_W107_CONTRACT_PROGRESS_KEYS,
    AGE5_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_PROGRESS_KEYS,
    AGE5_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_PROGRESS_KEYS,
    AGE5_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_PROGRESS_KEYS,
    AGE5_PROOF_CERTIFICATE_FAMILY_CONTRACT_PROGRESS_KEYS,
    AGE5_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_PROOF_FAMILY_CONTRACT_PROGRESS_KEYS,
    AGE5_PROOF_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_LANG_SURFACE_FAMILY_CONTRACT_PROGRESS_KEYS,
    AGE5_LANG_RUNTIME_FAMILY_CONTRACT_PROGRESS_KEYS,
    AGE5_GATE0_FAMILY_CONTRACT_PROGRESS_KEYS,
    AGE5_GATE0_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_GATE0_TRANSPORT_FAMILY_CONTRACT_PROGRESS_KEYS,
    AGE5_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
    AGE5_BOGAE_ALIAS_FAMILY_CONTRACT_PROGRESS_KEYS,
    AGE5_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_PROGRESS_KEYS,
)
AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA = "ddn.bogae_geoul_visibility_smoke.v1"
SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA = "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1"
SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES = 20
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA = "ddn.pack_evidence_tier_runner_check.v1"
SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA = "ddn.numeric_factor_route_diag_contract.v1"
SEAMGRIM_NUMERIC_FACTOR_POLICY_KEYS = (
    "bit_limit",
    "pollard_iters",
    "pollard_c_seeds",
    "pollard_x0_seeds",
    "fallback_limit",
    "small_prime_max",
)
SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS: dict[str, int] = {
    "bit_limit": 512,
    "pollard_iters": 200000,
    "pollard_c_seeds": 64,
    "pollard_x0_seeds": 6,
    "fallback_limit": 1000000,
    "small_prime_max": 101,
}
FIXED64_DARWIN_REAL_REPORT_LIVE_CHECK_SCHEMA = "ddn.fixed64.darwin_real_report_live_check.v1"


def _default_age5_progress_value(key: str) -> str:
    return "0" if str(key).endswith("_progress_present") else "-"


def build_contract_only_age5_progress_fields() -> dict[str, str]:
    fields: dict[str, str] = {}
    for key_group in AGE5_PROGRESS_KEY_GROUPS:
        for key in key_group:
            fields.setdefault(key, _default_age5_progress_value(key))
    return fields


def build_contract_only_age5_progress_brief_alias_fields(progress_fields: dict[str, str]) -> dict[str, str]:
    aliases: dict[str, str] = {}
    for key_map in AGE5_PROGRESS_BRIEF_KEY_MAPS:
        for triage_key, brief_key in key_map:
            aliases[brief_key] = str(progress_fields.get(triage_key, _default_age5_progress_value(triage_key)))
    return aliases


def build_contract_only_age5_policy_snapshot() -> dict[str, object]:
    policy_origin_trace = build_age5_combined_heavy_policy_origin_trace()
    age4_proof_snapshot = build_age4_proof_snapshot()
    return {
        "age5_policy_combined_digest_selftest_default_field_text": f"{AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY}=0",
        "age5_policy_combined_digest_selftest_default_field": build_age5_close_digest_selftest_default_field(),
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY: AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY: build_age4_proof_snapshot_text(age4_proof_snapshot),
        AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY: AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY: "0",
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY: "0",
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY: "0",
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY: "0",
        "age5_combined_heavy_policy_report_path": "-",
        "age5_combined_heavy_policy_report_exists": "0",
        "age5_combined_heavy_policy_text_path": "-",
        "age5_combined_heavy_policy_text_exists": "0",
        "age5_combined_heavy_policy_summary_path": "-",
        "age5_combined_heavy_policy_summary_exists": "0",
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY: policy_origin_trace,
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY: build_age5_combined_heavy_policy_origin_trace_text(
            policy_origin_trace
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY: (
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY: (
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY: (
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY: (
            AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_DEFAULT
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY: "ok",
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY: "0",
    }


def build_contract_only_age5_brief_tokens() -> dict[str, str]:
    digest_default_field = build_age5_close_digest_selftest_default_field()
    child_summary = build_age5_combined_heavy_child_summary_default_fields()
    child_summary_transport = build_age5_combined_heavy_child_summary_default_text_transport_fields()
    policy_snapshot = build_contract_only_age5_policy_snapshot()
    progress_fields = build_contract_only_age5_progress_fields()
    progress_alias_fields = build_contract_only_age5_progress_brief_alias_fields(progress_fields)
    return {
        AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY: "1",
        AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: f"{AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY}=0",
        AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_KEY: json.dumps(
            digest_default_field, ensure_ascii=False, sort_keys=True, separators=(",", ":")
        ),
        "combined_heavy_child_timeout_sec": "0",
        "age5_combined_heavy_timeout_present": "0",
        "age5_combined_heavy_timeout_targets": "-",
        AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY: AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED,
        **child_summary,
        **child_summary_transport,
        "age5_policy_combined_digest_selftest_default_field_text": str(
            policy_snapshot["age5_policy_combined_digest_selftest_default_field_text"]
        ),
        "age5_policy_combined_digest_selftest_default_field": json.dumps(
            dict(policy_snapshot["age5_policy_combined_digest_selftest_default_field"]),
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        ),
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY: str(
            policy_snapshot[AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY: str(
            policy_snapshot[AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY: str(
            policy_snapshot[AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY: str(
            policy_snapshot[AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY: str(
            policy_snapshot[AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY: str(
            policy_snapshot[AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY: str(
            policy_snapshot[AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY]
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY: str(
            policy_snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY]
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY: json.dumps(
            dict(policy_snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY]),
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        ),
        "age5_combined_heavy_policy_report_path": str(policy_snapshot["age5_combined_heavy_policy_report_path"]),
        "age5_combined_heavy_policy_report_exists": str(policy_snapshot["age5_combined_heavy_policy_report_exists"]),
        "age5_combined_heavy_policy_text_path": str(policy_snapshot["age5_combined_heavy_policy_text_path"]),
        "age5_combined_heavy_policy_text_exists": str(policy_snapshot["age5_combined_heavy_policy_text_exists"]),
        "age5_combined_heavy_policy_summary_path": str(policy_snapshot["age5_combined_heavy_policy_summary_path"]),
        "age5_combined_heavy_policy_summary_exists": str(
            policy_snapshot["age5_combined_heavy_policy_summary_exists"]
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY: str(
            policy_snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY]
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY: str(
            policy_snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY]
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY: str(
            policy_snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY]
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY: str(
            policy_snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY]
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY: str(
            policy_snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY]
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY: str(
            policy_snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY]
        ),
        **progress_alias_fields,
    }


def build_contract_only_age5_triage_fields() -> dict[str, object]:
    child_summary = build_age5_combined_heavy_child_summary_default_fields()
    child_summary_transport = build_age5_combined_heavy_child_summary_default_text_transport_fields()
    policy_snapshot = build_contract_only_age5_policy_snapshot()
    progress_fields = build_contract_only_age5_progress_fields()
    return {
        AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY: "1",
        AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: f"{AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY}=0",
        AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_KEY: build_age5_close_digest_selftest_default_field(),
        "combined_heavy_child_timeout_sec": "0",
        "age5_combined_heavy_timeout_present": "0",
        "age5_combined_heavy_timeout_targets": "-",
        AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY: AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED,
        **child_summary,
        **child_summary_transport,
        "age5_policy_combined_digest_selftest_default_field_text": str(
            policy_snapshot["age5_policy_combined_digest_selftest_default_field_text"]
        ),
        "age5_policy_combined_digest_selftest_default_field": dict(
            policy_snapshot["age5_policy_combined_digest_selftest_default_field"]
        ),
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY: str(
            policy_snapshot[AGE5_POLICY_AGE4_PROOF_SNAPSHOT_FIELDS_TEXT_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY: str(
            policy_snapshot[AGE5_POLICY_AGE4_PROOF_SNAPSHOT_TEXT_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY: str(
            policy_snapshot[AGE5_POLICY_AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY: str(
            policy_snapshot[AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PRESENT_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY: str(
            policy_snapshot[AGE5_POLICY_AGE4_PROOF_GATE_RESULT_PARITY_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY: str(
            policy_snapshot[AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PRESENT_KEY]
        ),
        AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY: str(
            policy_snapshot[AGE5_POLICY_AGE4_PROOF_FINAL_STATUS_PARSE_PARITY_KEY]
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY: str(
            policy_snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_TEXT_KEY]
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY: dict(
            policy_snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY]
        ),
        "age5_combined_heavy_policy_report_path": str(policy_snapshot["age5_combined_heavy_policy_report_path"]),
        "age5_combined_heavy_policy_report_exists": str(policy_snapshot["age5_combined_heavy_policy_report_exists"]),
        "age5_combined_heavy_policy_text_path": str(policy_snapshot["age5_combined_heavy_policy_text_path"]),
        "age5_combined_heavy_policy_text_exists": str(policy_snapshot["age5_combined_heavy_policy_text_exists"]),
        "age5_combined_heavy_policy_summary_path": str(policy_snapshot["age5_combined_heavy_policy_summary_path"]),
        "age5_combined_heavy_policy_summary_exists": str(
            policy_snapshot["age5_combined_heavy_policy_summary_exists"]
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY: str(
            policy_snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_ISSUE_KEY]
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY: str(
            policy_snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_SOURCE_ISSUE_KEY]
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY: str(
            policy_snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_REASON_KEY]
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY: str(
            policy_snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_COMPACT_FAILURE_REASON_KEY]
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY: str(
            policy_snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_STATUS_KEY]
        ),
        AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY: str(
            policy_snapshot[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY]
        ),
        **progress_fields,
    }


def build_contract_only_age5_aggregate_fields() -> dict[str, object]:
    triage_fields = build_contract_only_age5_triage_fields()
    aggregate_fields = dict(triage_fields)
    aggregate_fields["age5_combined_heavy_policy_report_exists"] = False
    aggregate_fields["age5_combined_heavy_policy_text_exists"] = False
    aggregate_fields["age5_combined_heavy_policy_summary_exists"] = False
    aggregate_fields[AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_CONTRACT_OK_KEY] = False
    return aggregate_fields


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def write_contract_only_age3_bogae_geoul_visibility_smoke_report(report_dir: Path) -> Path:
    report_path = report_dir / "age3_bogae_geoul_visibility_smoke.contract_only.detjson"
    write_json(
        report_path,
        {
            "schema": AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA,
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "status": "pass",
            "overall_ok": True,
            "checks": [
                {
                    "name": "contract_only_stub",
                    "ok": True,
                }
            ],
            "simulation_hash_delta": {
                "state_hash_changes": True,
                "bogae_hash_changes": True,
            },
        },
    )
    return report_path


def write_contract_only_seamgrim_wasm_web_step_check_report(report_dir: Path) -> Path:
    report_path = report_dir / "seamgrim_wasm_web_step_check.contract_only.detjson"
    write_json(
        report_path,
        {
            "schema": SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA,
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "status": "pass",
            "ok": True,
            "code": "OK",
            "checked_files": SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES,
            "missing_count": 0,
            "missing": [],
        },
    )
    return report_path


def write_contract_only_seamgrim_pack_evidence_tier_runner_report(report_dir: Path) -> Path:
    report_path = report_dir / "seamgrim_pack_evidence_tier_runner_check.contract_only.detjson"
    write_json(
        report_path,
        {
            "schema": SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA,
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "status": "pass",
            "ok": True,
            "code": "OK",
            "msg": "-",
            "max_docs_issues": 10,
            "expected_repo_issues": 0,
            "docs_profile": {
                "name": "docs_ssot_rep10",
                "issue_count": 10,
                "ok_count": 0,
                "total": 10,
                "report_path": "-",
                "fix_plan_path": "-",
            },
            "repo_profile": {
                "name": "repo_rep10",
                "issue_count": 0,
                "ok_count": 10,
                "total": 10,
                "report_path": "-",
            },
        },
    )
    return report_path


def build_contract_only_numeric_factor_policy_text() -> str:
    return ";".join(
        f"{key}={SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS[key]}"
        for key in SEAMGRIM_NUMERIC_FACTOR_POLICY_KEYS
    )


def write_contract_only_seamgrim_numeric_factor_policy_report(report_dir: Path) -> Path:
    report_path = report_dir / "seamgrim_numeric_factor_policy.contract_only.detjson"
    policy_payload = {
        key: int(SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS[key])
        for key in SEAMGRIM_NUMERIC_FACTOR_POLICY_KEYS
    }
    write_json(
        report_path,
        {
            "schema": SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA,
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "status": "pass",
            "ok": True,
            "code": "OK",
            "numeric_factor_policy_text": build_contract_only_numeric_factor_policy_text(),
            "numeric_factor_policy": policy_payload,
        },
    )
    return report_path


def write_contract_only_fixed64_darwin_real_report_live_check_report(report_dir: Path) -> Path:
    report_path = report_dir / "fixed64_darwin_real_report_live_check.contract_only.detjson"
    write_json(
        report_path,
        {
            "schema": FIXED64_DARWIN_REAL_REPORT_LIVE_CHECK_SCHEMA,
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "ok": True,
            "status": "skip_disabled",
            "reason": "contract_only_stub",
            "resolved_status": "-",
            "resolved_source": "",
            "resolve_invalid_hits": [],
        },
    )
    return report_path


def write_contract_only_stub_reports(
    seamgrim_report: Path,
    seamgrim_ui_age3_report: Path,
    seamgrim_phase3_cleanup_report: Path,
    seamgrim_browse_selection_report: Path,
    age2_close_report: Path,
    age3_close_report: Path,
    age4_close_report: Path,
    age5_close_report: Path,
    age4_pack_report: Path,
    oi_report: Path,
    oi_pack_report: Path,
) -> None:
    generated_at_utc = datetime.now(timezone.utc).isoformat()
    write_json(
        seamgrim_report,
        {
            "schema": "ddn.seamgrim.ci_gate_report.v1",
            "generated_at_utc": generated_at_utc,
            "ok": True,
            "elapsed_total_ms": 0,
            "failure_digest": [],
            "steps": [
                {
                    "name": step_name,
                    "ok": True,
                    "returncode": 0,
                    "diagnostics": [],
                }
                for step_name in CONTRACT_ONLY_SEAMGRIM_STEP_NAMES
            ],
        },
    )
    for stub_path, schema in (
        (seamgrim_ui_age3_report, "ddn.seamgrim.ui_age3_gate_report.v1"),
        (seamgrim_phase3_cleanup_report, "ddn.seamgrim.phase3_cleanup_gate_report.v1"),
        (seamgrim_browse_selection_report, "ddn.seamgrim.browse_selection_flow_report.v1"),
        (age4_pack_report, "ddn.age4_close_pack_report.v1"),
        (oi_pack_report, "ddn.oi405_406_pack_report.v1"),
    ):
        write_json(
            stub_path,
            {
                "schema": schema,
                "generated_at_utc": generated_at_utc,
                "ok": True,
                "status": "pass",
            },
        )
    write_json(
        age2_close_report,
        {
            "schema": "ddn.age2_close_report.v1",
            "generated_at_utc": generated_at_utc,
            "overall_ok": True,
            "criteria": [],
            "failure_digest": [],
            "failure_codes": [],
        },
    )
    write_json(
        age3_close_report,
        {
            "schema": "ddn.seamgrim.age3_close_report.v1",
            "generated_at_utc": generated_at_utc,
            "overall_ok": True,
            "criteria": [],
            "failure_digest": [],
            "seamgrim_report_path": str(seamgrim_report),
            "ui_age3_report_path": str(seamgrim_ui_age3_report),
        },
    )
    for stub_path, schema in (
        (age4_close_report, "ddn.age4_close_report.v1"),
        (age5_close_report, "ddn.age5_close_report.v1"),
    ):
        payload = {
            "schema": schema,
            "generated_at_utc": generated_at_utc,
            "overall_ok": True,
            "criteria": [],
            "failure_digest": [],
        }
        if stub_path == age5_close_report:
            payload.update(build_age5_combined_heavy_child_summary_default_fields())
            payload.update(build_age5_full_real_w107_golden_index_selftest_progress())
            payload.update(build_age5_full_real_w107_progress_contract_selftest_progress())
        write_json(stub_path, payload)
    write_json(
        oi_report,
        {
            "schema": "ddn.oi405_406_close_report.v1",
            "generated_at_utc": generated_at_utc,
            "overall_ok": True,
            "packs": [],
            "failure_digest": [],
        },
    )


def write_contract_only_profile_matrix_selftest_report(report_path: Path, selected_profiles: list[str]) -> None:
    valid_profiles = ("core_lang", "full", "seamgrim")
    selected = [name for name in valid_profiles if name in selected_profiles]
    skipped = [name for name in valid_profiles if name not in selected]
    generated_at_utc = datetime.now(timezone.utc).isoformat()
    aggregate_summary_by_profile: dict[str, object] = {}
    real_profiles: dict[str, object] = {}
    for name in valid_profiles:
        expected_contract = expected_profile_matrix_aggregate_summary_contract(name)
        is_selected = name in selected
        if is_selected:
            real_profiles[name] = {
                "total_elapsed_ms": 0,
            }
            values = dict(expected_contract["values"])
            aggregate_summary_by_profile[name] = {
                "status": "pass",
                "ok": True,
                "expected_present": bool(expected_contract["expected_present"]),
                "present": True,
                "expected_profile": str(expected_contract["expected_profile"]),
                "expected_sync_profile": str(expected_contract["expected_sync_profile"]),
                "profile": name,
                "sync_profile": name,
                "expected_values": dict(expected_contract["values"]),
                "profile_ok": True,
                "sync_profile_ok": True,
                "values_ok": True,
                "missing_keys": [],
                "mismatched_keys": [],
                "gate_marker_expected": bool(expected_contract["gate_marker_expected"]),
                "gate_marker_present": bool(expected_contract["gate_marker_expected"]),
                "gate_marker_ok": True,
                "values": values,
            }
        else:
            real_profiles[name] = {}
            aggregate_summary_by_profile[name] = {
                "status": "skipped",
                "ok": True,
                "expected_present": bool(expected_contract["expected_present"]),
                "present": False,
                "expected_profile": str(expected_contract["expected_profile"]),
                "expected_sync_profile": str(expected_contract["expected_sync_profile"]),
                "expected_values": dict(expected_contract["values"]),
                "profile": name,
                "sync_profile": name,
                "profile_ok": True,
                "sync_profile_ok": True,
                "values_ok": True,
                "missing_keys": [],
                "mismatched_keys": [],
                "gate_marker_expected": bool(expected_contract["gate_marker_expected"]),
                "gate_marker_present": False,
                "gate_marker_ok": True,
                "values": {key: "" for key in dict(expected_contract["values"]).keys()},
            }
    write_json(
        report_path,
        {
            "schema": "ddn.ci.profile_matrix_gate_selftest.v1",
            "generated_at_utc": generated_at_utc,
            "status": "pass",
            "ok": True,
            "total_elapsed_ms": 0,
            "selected_real_profiles": selected,
            "skipped_real_profiles": skipped,
            "step_timeout_defaults_text": PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
            "step_timeout_defaults_sec": dict(PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC),
            "step_timeout_env_keys": dict(PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS),
            "real_profiles": real_profiles,
            "aggregate_summary_sanity_by_profile": aggregate_summary_by_profile,
            "aggregate_summary_sanity_checked_profiles": selected,
            "aggregate_summary_sanity_failed_profiles": [],
            "aggregate_summary_sanity_skipped_profiles": skipped,
            "aggregate_summary_sanity_ok": True,
        },
    )


def resolve_contract_only_selected_profiles(raw: str, fallback_profile: str) -> list[str]:
    valid_profiles = ("core_lang", "full", "seamgrim")
    requested = [part.strip() for part in str(raw).split(",") if part.strip()]
    selected = [name for name in valid_profiles if name in requested]
    if selected:
        return selected
    return [fallback_profile]


def build_contract_only_profile_matrix_triage_snapshot(
    report_path: Path, selected_profiles: list[str]
) -> dict[str, object]:
    valid_profiles = ("core_lang", "full", "seamgrim")
    selected = [name for name in valid_profiles if name in selected_profiles]
    skipped = [name for name in valid_profiles if name not in selected]

    try:
        report_doc = json.loads(report_path.read_text(encoding="utf-8"))
    except Exception:
        report_doc = None
    profile_matrix_snapshot = build_profile_matrix_snapshot_from_doc(
        report_doc,
        report_path=str(report_path),
    )
    if isinstance(profile_matrix_snapshot, dict):
        payload = build_profile_matrix_triage_payload_from_snapshot(profile_matrix_snapshot)
        skipped_values = format_profile_matrix_summary_values({})
        payload["selected_real_profiles"] = list(selected)
        payload["skipped_real_profiles"] = list(skipped)
        payload["aggregate_summary_sanity_checked_profiles"] = list(selected)
        payload["aggregate_summary_sanity_failed_profiles"] = []
        payload["aggregate_summary_sanity_skipped_profiles"] = list(skipped)
        for profile_name in valid_profiles:
            if profile_name in selected:
                continue
            payload[f"{profile_name}_elapsed_ms"] = None
            payload[f"{profile_name}_aggregate_summary_status"] = "skipped"
            payload[f"{profile_name}_aggregate_summary_ok"] = True
            payload[f"{profile_name}_aggregate_summary_values"] = skipped_values
        return payload

    # Fallback: keep contract-only reporter resilient even if fixture loading failed.
    return {
        "report_path": str(report_path),
        "status": "pass",
        "ok": True,
        "total_elapsed_ms": 0,
        "selected_real_profiles": selected,
        "skipped_real_profiles": skipped,
        "step_timeout_defaults_text": PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_TEXT,
        "step_timeout_defaults_sec": dict(PROFILE_MATRIX_STEP_TIMEOUT_DEFAULTS_SEC),
        "step_timeout_env_keys": dict(PROFILE_MATRIX_STEP_TIMEOUT_ENV_KEYS),
        "core_lang_elapsed_ms": 0 if "core_lang" in selected else None,
        "full_elapsed_ms": 0 if "full" in selected else None,
        "seamgrim_elapsed_ms": 0 if "seamgrim" in selected else None,
        "aggregate_summary_sanity_ok": True,
        "aggregate_summary_sanity_checked_profiles": selected,
        "aggregate_summary_sanity_failed_profiles": [],
        "aggregate_summary_sanity_skipped_profiles": skipped,
        "core_lang_aggregate_summary_status": "pass" if "core_lang" in selected else "skipped",
        "core_lang_aggregate_summary_ok": True,
        "core_lang_aggregate_summary_values": "-",
        "full_aggregate_summary_status": "pass" if "full" in selected else "skipped",
        "full_aggregate_summary_ok": True,
        "full_aggregate_summary_values": "-",
        "seamgrim_aggregate_summary_status": "pass" if "seamgrim" in selected else "skipped",
        "seamgrim_aggregate_summary_ok": True,
        "seamgrim_aggregate_summary_values": "-",
    }


def resolve_contract_only_sanity_steps(profile: str) -> tuple[str, ...]:
    base = (
        "backup_hygiene_selftest",
        "pipeline_emit_flags_check",
        "pipeline_emit_flags_selftest",
        "ci_emit_artifacts_sanity_contract_selftest",
        "age5_combined_heavy_policy_selftest",
        "profile_matrix_full_real_smoke_policy_selftest",
        "profile_matrix_full_real_smoke_check_selftest",
        "ci_sanity_dynamic_source_profile_split_selftest",
        "age2_completion_gate",
        "age2_completion_gate_selftest",
        "age2_close",
        "age2_close_selftest",
        "age3_completion_gate",
        "age3_completion_gate_selftest",
        "age3_close_selftest",
        "fixed64_darwin_real_report_contract_check",
        "fixed64_darwin_real_report_live_check",
        "fixed64_darwin_real_report_readiness_check_selftest",
        "ci_profile_split_contract_check",
        "ci_profile_matrix_lightweight_contract_selftest",
        "ci_profile_matrix_snapshot_helper_selftest",
        "contract_tier_unsupported_check",
        "contract_tier_age3_min_enforcement_check",
        "map_access_contract_check",
        "gaji_registry_strict_audit_check",
        "gaji_registry_defaults_check",
        "stdlib_catalog_check",
        "stdlib_catalog_check_selftest",
        "tensor_v0_pack_check",
        "tensor_v0_cli_check",
        "age5_close_pack_contract_selftest",
        "ci_pack_golden_age5_surface_selftest",
        "ci_pack_golden_guideblock_selftest",
        "ci_pack_golden_exec_policy_selftest",
        "ci_pack_golden_jjaim_flatten_selftest",
        "ci_pack_golden_event_model_selftest",
        "ci_pack_golden_lang_consistency_selftest",
        "ci_pack_golden_metadata_selftest",
        "ci_pack_golden_graph_export_selftest",
        "ci_canon_ast_dpack_selftest",
        "w92_aot_pack_check",
        "w93_universe_pack_check",
        "w94_social_pack_check",
        "w95_cert_pack_check",
        "w96_somssi_pack_check",
        "w97_self_heal_pack_check",
    )
    seamgrim_extra = merge_step_names(
        (
        "seamgrim_ci_gate_sam_seulgi_family_step_check",
        "seamgrim_ci_gate_seed_meta_step_check",
        "seamgrim_ci_gate_pack_evidence_tier_runner_check",
        "seamgrim_ci_gate_pack_evidence_tier_step_check",
        "seamgrim_ci_gate_pack_evidence_tier_step_check_selftest",
        "seamgrim_ci_gate_pack_evidence_tier_report_check",
        "seamgrim_ci_gate_pack_evidence_tier_report_check_selftest",
        "seamgrim_ci_gate_runtime5_passthrough_check",
        "seamgrim_ci_gate_lesson_warning_step_check",
        "seamgrim_ci_gate_stateful_preview_step_check",
        "seamgrim_ci_gate_wasm_web_smoke_step_check",
        "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
        "age3_close",
        "age3_close_selftest",
        "seamgrim_interface_boundary_contract_check",
        "seamgrim_overlay_session_wired_consistency_check",
        "seamgrim_overlay_session_diag_parity_check",
        "seamgrim_overlay_compare_diag_parity_check",
        "seamgrim_wasm_cli_diag_parity_check",
        ),
        SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS,
    )
    if profile == "seamgrim":
        return merge_step_names(
            (
            "fixed64_darwin_real_report_contract_check",
            "fixed64_darwin_real_report_live_check",
            "fixed64_darwin_real_report_readiness_check_selftest",
            "ci_emit_artifacts_sanity_contract_selftest",
            "age5_combined_heavy_policy_selftest",
            "profile_matrix_full_real_smoke_policy_selftest",
            "profile_matrix_full_real_smoke_check_selftest",
            "ci_sanity_dynamic_source_profile_split_selftest",
            "age2_completion_gate",
            "age2_completion_gate_selftest",
            "age2_close",
            "age2_close_selftest",
            "age3_completion_gate",
            "age3_completion_gate_selftest",
            "age3_close",
            "age3_close_selftest",
            "ci_profile_split_contract_check",
            "ci_profile_matrix_lightweight_contract_selftest",
            "ci_profile_matrix_snapshot_helper_selftest",
            "seamgrim_ci_gate_sam_seulgi_family_step_check",
            "seamgrim_ci_gate_seed_meta_step_check",
            "seamgrim_ci_gate_pack_evidence_tier_runner_check",
            "seamgrim_ci_gate_pack_evidence_tier_step_check",
            "seamgrim_ci_gate_pack_evidence_tier_step_check_selftest",
            "seamgrim_ci_gate_pack_evidence_tier_report_check",
            "seamgrim_ci_gate_pack_evidence_tier_report_check_selftest",
            "seamgrim_ci_gate_runtime5_passthrough_check",
            "seamgrim_ci_gate_lesson_warning_step_check",
            "seamgrim_ci_gate_stateful_preview_step_check",
            "seamgrim_ci_gate_wasm_web_smoke_step_check",
            "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
            "seamgrim_interface_boundary_contract_check",
            "seamgrim_overlay_session_wired_consistency_check",
            "seamgrim_overlay_session_diag_parity_check",
            "seamgrim_overlay_compare_diag_parity_check",
            "seamgrim_wasm_cli_diag_parity_check",
            ),
            SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS,
        )
    if profile == "core_lang":
        return base
    return base + seamgrim_extra


def resolve_contract_only_required_steps(profile: str) -> tuple[str, ...]:
    common = (
        "age5_close_digest_selftest",
        "ci_profile_split_contract_check",
        "ci_profile_matrix_gate_selftest",
        "ci_sanity_gate",
        "ci_sync_readiness_report_generate",
        "ci_sync_readiness_report_check",
        "ci_emit_artifacts_required_post_summary_check",
        "ci_fail_and_exit_contract_selftest",
        "ci_gate_report_index_selftest",
        "ci_gate_report_index_diagnostics_check",
        "ci_gate_report_index_latest_smoke_check",
    )
    seamgrim = merge_step_names(
        (
        "seamgrim_ci_gate_sam_seulgi_family_step_check",
        "seamgrim_ci_gate_seed_meta_step_check",
        "seamgrim_ci_gate_pack_evidence_tier_runner_check",
        "seamgrim_ci_gate_pack_evidence_tier_step_check",
        "seamgrim_ci_gate_pack_evidence_tier_step_check_selftest",
        "seamgrim_ci_gate_pack_evidence_tier_report_check",
        "seamgrim_ci_gate_pack_evidence_tier_report_check_selftest",
        "seamgrim_ci_gate_runtime5_passthrough_check",
        "seamgrim_ci_gate_guideblock_step_check",
        "seamgrim_ci_gate_lesson_warning_step_check",
        "seamgrim_ci_gate_stateful_preview_step_check",
        "seamgrim_ci_gate_wasm_web_smoke_step_check",
        "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
        "seamgrim_wasm_cli_diag_parity_check",
        ),
        SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS,
    )
    if profile == "core_lang":
        return common
    return merge_step_names(common, seamgrim)


def write_contract_only_ci_sanity_report(report_path: Path, profile: str) -> None:
    step_names = resolve_contract_only_sanity_steps(profile)
    steps = [{"step": name, "ok": True, "returncode": 0} for name in step_names]
    include_core_lang_keys = profile in {"full", "core_lang"}
    pack_golden_graph_export_ok = "1" if include_core_lang_keys else "0"
    age2_close_enabled = profile in {"full", "core_lang", "seamgrim"}
    age3_close_enabled = profile in {"full", "seamgrim"}
    smoke_report_path = write_contract_only_age3_bogae_geoul_visibility_smoke_report(report_path.parent)
    seamgrim_wasm_web_step_check_enabled = profile == "seamgrim"
    seamgrim_wasm_web_step_check_report_path = (
        write_contract_only_seamgrim_wasm_web_step_check_report(report_path.parent)
        if seamgrim_wasm_web_step_check_enabled
        else None
    )
    seamgrim_pack_evidence_tier_runner_enabled = profile == "seamgrim"
    seamgrim_pack_evidence_tier_runner_report_path = (
        write_contract_only_seamgrim_pack_evidence_tier_runner_report(report_path.parent)
        if seamgrim_pack_evidence_tier_runner_enabled
        else None
    )
    seamgrim_pack_evidence_tier_runner_ok = "1" if seamgrim_pack_evidence_tier_runner_enabled else "na"
    seamgrim_pack_evidence_tier_runner_report_path_text = (
        str(seamgrim_pack_evidence_tier_runner_report_path)
        if seamgrim_pack_evidence_tier_runner_enabled
        and seamgrim_pack_evidence_tier_runner_report_path is not None
        else "-"
    )
    seamgrim_pack_evidence_tier_runner_report_exists = "1" if seamgrim_pack_evidence_tier_runner_enabled else "na"
    seamgrim_pack_evidence_tier_runner_schema = (
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA if seamgrim_pack_evidence_tier_runner_enabled else "-"
    )
    seamgrim_pack_evidence_tier_runner_docs_issue_count = "10" if seamgrim_pack_evidence_tier_runner_enabled else "-"
    seamgrim_pack_evidence_tier_runner_repo_issue_count = "0" if seamgrim_pack_evidence_tier_runner_enabled else "-"
    seamgrim_numeric_factor_policy_enabled = profile in {"full", "seamgrim"}
    seamgrim_numeric_factor_policy_report_path = (
        write_contract_only_seamgrim_numeric_factor_policy_report(report_path.parent)
        if seamgrim_numeric_factor_policy_enabled
        else None
    )
    seamgrim_numeric_factor_policy_ok = "1" if seamgrim_numeric_factor_policy_enabled else "na"
    seamgrim_numeric_factor_policy_report_path_text = (
        str(seamgrim_numeric_factor_policy_report_path)
        if seamgrim_numeric_factor_policy_enabled and seamgrim_numeric_factor_policy_report_path is not None
        else "-"
    )
    seamgrim_numeric_factor_policy_report_exists = "1" if seamgrim_numeric_factor_policy_enabled else "na"
    seamgrim_numeric_factor_policy_schema = (
        SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA if seamgrim_numeric_factor_policy_enabled else "-"
    )
    seamgrim_numeric_factor_policy_text = (
        build_contract_only_numeric_factor_policy_text() if seamgrim_numeric_factor_policy_enabled else "-"
    )
    seamgrim_numeric_factor_policy_values = {
        key: (
            str(SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS[key])
            if seamgrim_numeric_factor_policy_enabled
            else "-"
        )
        for key in SEAMGRIM_NUMERIC_FACTOR_POLICY_KEYS
    }
    fixed64_live_report_path = write_contract_only_fixed64_darwin_real_report_live_check_report(report_path.parent)
    write_json(
        report_path,
        {
            "schema": "ddn.ci.sanity_gate.v1",
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "status": "pass",
            "ok": True,
            "code": "OK",
            "step": "all",
            "profile": profile,
            "ci_sanity_pipeline_emit_flags_ok": "1" if include_core_lang_keys else "na",
            "ci_sanity_pipeline_emit_flags_selftest_ok": "1" if include_core_lang_keys else "na",
            "ci_sanity_emit_artifacts_sanity_contract_selftest_ok": "1",
            "ci_sanity_age2_completion_gate_ok": "1",
            "ci_sanity_age2_completion_gate_selftest_ok": "1",
            "ci_sanity_age2_close_ok": "1" if age2_close_enabled else "na",
            "ci_sanity_age2_close_selftest_ok": "1" if age2_close_enabled else "na",
            "ci_sanity_age2_completion_gate_failure_codes": "-",
            "ci_sanity_age2_completion_gate_failure_code_count": "0",
            "ci_sanity_age3_completion_gate_ok": "1",
            "ci_sanity_age3_completion_gate_selftest_ok": "1",
            "ci_sanity_age3_close_ok": "1" if age3_close_enabled else "na",
            "ci_sanity_age3_close_selftest_ok": "1" if age3_close_enabled else "na",
            "ci_sanity_age3_completion_gate_failure_codes": "-",
            "ci_sanity_age3_completion_gate_failure_code_count": "0",
            **{key: "1" for key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS},
            "ci_sanity_age3_bogae_geoul_visibility_smoke_ok": "1",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path": str(smoke_report_path),
            "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists": "1",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_schema": AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA,
            "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok": "1",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok": "1",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes": "1",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes": "1",
            "ci_sanity_seamgrim_wasm_web_step_check_ok": "1" if seamgrim_wasm_web_step_check_enabled else "na",
            "ci_sanity_seamgrim_wasm_web_step_check_report_path": (
                str(seamgrim_wasm_web_step_check_report_path)
                if seamgrim_wasm_web_step_check_enabled and seamgrim_wasm_web_step_check_report_path is not None
                else "-"
            ),
            "ci_sanity_seamgrim_wasm_web_step_check_report_exists": (
                "1" if seamgrim_wasm_web_step_check_enabled else "na"
            ),
            "ci_sanity_seamgrim_wasm_web_step_check_schema": (
                SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA if seamgrim_wasm_web_step_check_enabled else "-"
            ),
            "ci_sanity_seamgrim_wasm_web_step_check_checked_files": (
                str(SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES) if seamgrim_wasm_web_step_check_enabled else "-"
            ),
            "ci_sanity_seamgrim_wasm_web_step_check_missing_count": (
                "0" if seamgrim_wasm_web_step_check_enabled else "-"
            ),
            "ci_sanity_seamgrim_pack_evidence_tier_runner_ok": seamgrim_pack_evidence_tier_runner_ok,
            "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path": (
                seamgrim_pack_evidence_tier_runner_report_path_text
            ),
            "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists": (
                seamgrim_pack_evidence_tier_runner_report_exists
            ),
            "ci_sanity_seamgrim_pack_evidence_tier_runner_schema": seamgrim_pack_evidence_tier_runner_schema,
            "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count": (
                seamgrim_pack_evidence_tier_runner_docs_issue_count
            ),
            "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count": (
                seamgrim_pack_evidence_tier_runner_repo_issue_count
            ),
            "ci_sanity_seamgrim_numeric_factor_policy_ok": seamgrim_numeric_factor_policy_ok,
            "ci_sanity_seamgrim_numeric_factor_policy_report_path": (
                seamgrim_numeric_factor_policy_report_path_text
            ),
            "ci_sanity_seamgrim_numeric_factor_policy_report_exists": (
                seamgrim_numeric_factor_policy_report_exists
            ),
            "ci_sanity_seamgrim_numeric_factor_policy_schema": seamgrim_numeric_factor_policy_schema,
            "ci_sanity_seamgrim_numeric_factor_policy_text": seamgrim_numeric_factor_policy_text,
            "ci_sanity_seamgrim_numeric_factor_policy_bit_limit": seamgrim_numeric_factor_policy_values[
                "bit_limit"
            ],
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_iters": seamgrim_numeric_factor_policy_values[
                "pollard_iters"
            ],
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds": seamgrim_numeric_factor_policy_values[
                "pollard_c_seeds"
            ],
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds": seamgrim_numeric_factor_policy_values[
                "pollard_x0_seeds"
            ],
            "ci_sanity_seamgrim_numeric_factor_policy_fallback_limit": seamgrim_numeric_factor_policy_values[
                "fallback_limit"
            ],
            "ci_sanity_seamgrim_numeric_factor_policy_small_prime_max": seamgrim_numeric_factor_policy_values[
                "small_prime_max"
            ],
            "ci_sanity_pack_golden_graph_export_ok": pack_golden_graph_export_ok,
            "ci_sanity_fixed64_darwin_real_report_live_report_path": str(fixed64_live_report_path),
            "ci_sanity_fixed64_darwin_real_report_live_report_exists": "1",
            "ci_sanity_fixed64_darwin_real_report_live_status": "skip_disabled",
            "ci_sanity_fixed64_darwin_real_report_live_resolved_status": "-",
            "ci_sanity_fixed64_darwin_real_report_live_resolved_source": "-",
            "ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count": "0",
            "ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip": "0",
            "ci_sanity_fixed64_darwin_real_report_live_check_selftest_ok": "1",
            "ci_sanity_fixed64_threeway_inputs_selftest_ok": "1",
            "ci_sanity_age5_combined_heavy_policy_selftest_ok": "1",
            "ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok": "1",
            "ci_sanity_dynamic_source_profile_split_selftest_ok": "1",
            **build_age5_combined_heavy_sanity_contract_fields(),
            "msg": "contract-only aggregate stub",
            "steps": steps,
        },
    )


def write_contract_only_ci_sync_readiness_report(report_path: Path, profile: str) -> None:
    include_core_lang_keys = profile in {"full", "core_lang"}
    pack_golden_graph_export_ok = "1" if include_core_lang_keys else "0"
    age2_close_enabled = profile in {"full", "core_lang", "seamgrim"}
    age3_close_enabled = profile in {"full", "seamgrim"}
    smoke_report_path = write_contract_only_age3_bogae_geoul_visibility_smoke_report(report_path.parent)
    smoke_report_path_text = str(smoke_report_path)
    seamgrim_wasm_web_step_check_enabled = profile == "seamgrim"
    seamgrim_wasm_web_step_check_report_path = (
        write_contract_only_seamgrim_wasm_web_step_check_report(report_path.parent)
        if seamgrim_wasm_web_step_check_enabled
        else None
    )
    seamgrim_wasm_web_step_check_report_path_text = (
        str(seamgrim_wasm_web_step_check_report_path)
        if seamgrim_wasm_web_step_check_enabled and seamgrim_wasm_web_step_check_report_path is not None
        else "-"
    )
    seamgrim_wasm_web_step_check_ok = "1" if seamgrim_wasm_web_step_check_enabled else "na"
    seamgrim_wasm_web_step_check_report_exists = "1" if seamgrim_wasm_web_step_check_enabled else "na"
    seamgrim_wasm_web_step_check_schema = (
        SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA if seamgrim_wasm_web_step_check_enabled else "-"
    )
    seamgrim_wasm_web_step_check_checked_files = (
        str(SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES) if seamgrim_wasm_web_step_check_enabled else "-"
    )
    seamgrim_wasm_web_step_check_missing_count = "0" if seamgrim_wasm_web_step_check_enabled else "-"
    seamgrim_pack_evidence_tier_runner_enabled = profile == "seamgrim"
    seamgrim_pack_evidence_tier_runner_report_path = (
        write_contract_only_seamgrim_pack_evidence_tier_runner_report(report_path.parent)
        if seamgrim_pack_evidence_tier_runner_enabled
        else None
    )
    seamgrim_pack_evidence_tier_runner_ok = "1" if seamgrim_pack_evidence_tier_runner_enabled else "na"
    seamgrim_pack_evidence_tier_runner_report_path_text = (
        str(seamgrim_pack_evidence_tier_runner_report_path)
        if seamgrim_pack_evidence_tier_runner_enabled
        and seamgrim_pack_evidence_tier_runner_report_path is not None
        else "-"
    )
    seamgrim_pack_evidence_tier_runner_report_exists = "1" if seamgrim_pack_evidence_tier_runner_enabled else "na"
    seamgrim_pack_evidence_tier_runner_schema = (
        SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA if seamgrim_pack_evidence_tier_runner_enabled else "-"
    )
    seamgrim_pack_evidence_tier_runner_docs_issue_count = "10" if seamgrim_pack_evidence_tier_runner_enabled else "-"
    seamgrim_pack_evidence_tier_runner_repo_issue_count = "0" if seamgrim_pack_evidence_tier_runner_enabled else "-"
    seamgrim_numeric_factor_policy_enabled = profile in {"full", "seamgrim"}
    seamgrim_numeric_factor_policy_report_path = (
        write_contract_only_seamgrim_numeric_factor_policy_report(report_path.parent)
        if seamgrim_numeric_factor_policy_enabled
        else None
    )
    seamgrim_numeric_factor_policy_ok = "1" if seamgrim_numeric_factor_policy_enabled else "na"
    seamgrim_numeric_factor_policy_report_path_text = (
        str(seamgrim_numeric_factor_policy_report_path)
        if seamgrim_numeric_factor_policy_enabled and seamgrim_numeric_factor_policy_report_path is not None
        else "-"
    )
    seamgrim_numeric_factor_policy_report_exists = "1" if seamgrim_numeric_factor_policy_enabled else "na"
    seamgrim_numeric_factor_policy_schema = (
        SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA if seamgrim_numeric_factor_policy_enabled else "-"
    )
    seamgrim_numeric_factor_policy_text = (
        build_contract_only_numeric_factor_policy_text() if seamgrim_numeric_factor_policy_enabled else "-"
    )
    seamgrim_numeric_factor_policy_values = {
        key: (
            str(SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS[key])
            if seamgrim_numeric_factor_policy_enabled
            else "-"
        )
        for key in SEAMGRIM_NUMERIC_FACTOR_POLICY_KEYS
    }
    fixed64_live_report_path = write_contract_only_fixed64_darwin_real_report_live_check_report(report_path.parent)
    fixed64_live_report_path_text = str(fixed64_live_report_path)
    fixed64_live_status = "skip_disabled"
    fixed64_live_resolved_status = "-"
    fixed64_live_resolved_source = "-"
    fixed64_live_invalid_count = "0"
    fixed64_live_source_zip = "0"
    write_json(
        report_path,
        {
            "schema": "ddn.ci.sync_readiness.v1",
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "status": "pass",
            "ok": True,
            "code": "OK",
            "step": "all",
            "sanity_profile": profile,
            "ci_sanity_pipeline_emit_flags_ok": "1" if include_core_lang_keys else "na",
            "ci_sanity_pipeline_emit_flags_selftest_ok": "1" if include_core_lang_keys else "na",
            "ci_sanity_emit_artifacts_sanity_contract_selftest_ok": "1",
            "ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok": "1",
            "ci_sanity_age2_completion_gate_ok": "1",
            "ci_sanity_age2_completion_gate_selftest_ok": "1",
            "ci_sanity_age2_close_ok": "1" if age2_close_enabled else "na",
            "ci_sanity_age2_close_selftest_ok": "1" if age2_close_enabled else "na",
            "ci_sanity_age2_completion_gate_failure_codes": "-",
            "ci_sanity_age2_completion_gate_failure_code_count": "0",
            "ci_sanity_age3_completion_gate_ok": "1",
            "ci_sanity_age3_completion_gate_selftest_ok": "1",
            "ci_sanity_age3_close_ok": "1" if age3_close_enabled else "na",
            "ci_sanity_age3_close_selftest_ok": "1" if age3_close_enabled else "na",
            "ci_sanity_age3_completion_gate_failure_codes": "-",
            "ci_sanity_age3_completion_gate_failure_code_count": "0",
            **{key: "1" for key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS},
            "ci_sync_readiness_ci_sanity_age2_completion_gate_failure_codes": "-",
            "ci_sync_readiness_ci_sanity_age2_completion_gate_failure_code_count": "0",
            "ci_sync_readiness_ci_sanity_age3_completion_gate_failure_codes": "-",
            "ci_sync_readiness_ci_sanity_age3_completion_gate_failure_code_count": "0",
            **{
                sync_key: "1"
                for _sanity_key, sync_key in AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS
            },
            "ci_sanity_age3_bogae_geoul_visibility_smoke_ok": "1",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path": smoke_report_path_text,
            "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists": "1",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_schema": AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA,
            "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok": "1",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok": "1",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes": "1",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes": "1",
            "ci_sanity_seamgrim_wasm_web_step_check_ok": seamgrim_wasm_web_step_check_ok,
            "ci_sanity_seamgrim_wasm_web_step_check_report_path": seamgrim_wasm_web_step_check_report_path_text,
            "ci_sanity_seamgrim_wasm_web_step_check_report_exists": seamgrim_wasm_web_step_check_report_exists,
            "ci_sanity_seamgrim_wasm_web_step_check_schema": seamgrim_wasm_web_step_check_schema,
            "ci_sanity_seamgrim_wasm_web_step_check_checked_files": seamgrim_wasm_web_step_check_checked_files,
            "ci_sanity_seamgrim_wasm_web_step_check_missing_count": seamgrim_wasm_web_step_check_missing_count,
            "ci_sanity_seamgrim_pack_evidence_tier_runner_ok": seamgrim_pack_evidence_tier_runner_ok,
            "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path": seamgrim_pack_evidence_tier_runner_report_path_text,
            "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists": (
                seamgrim_pack_evidence_tier_runner_report_exists
            ),
            "ci_sanity_seamgrim_pack_evidence_tier_runner_schema": seamgrim_pack_evidence_tier_runner_schema,
            "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count": (
                seamgrim_pack_evidence_tier_runner_docs_issue_count
            ),
            "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count": (
                seamgrim_pack_evidence_tier_runner_repo_issue_count
            ),
            "ci_sanity_seamgrim_numeric_factor_policy_ok": seamgrim_numeric_factor_policy_ok,
            "ci_sanity_seamgrim_numeric_factor_policy_report_path": (
                seamgrim_numeric_factor_policy_report_path_text
            ),
            "ci_sanity_seamgrim_numeric_factor_policy_report_exists": (
                seamgrim_numeric_factor_policy_report_exists
            ),
            "ci_sanity_seamgrim_numeric_factor_policy_schema": seamgrim_numeric_factor_policy_schema,
            "ci_sanity_seamgrim_numeric_factor_policy_text": seamgrim_numeric_factor_policy_text,
            "ci_sanity_seamgrim_numeric_factor_policy_bit_limit": seamgrim_numeric_factor_policy_values[
                "bit_limit"
            ],
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_iters": seamgrim_numeric_factor_policy_values[
                "pollard_iters"
            ],
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds": seamgrim_numeric_factor_policy_values[
                "pollard_c_seeds"
            ],
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds": seamgrim_numeric_factor_policy_values[
                "pollard_x0_seeds"
            ],
            "ci_sanity_seamgrim_numeric_factor_policy_fallback_limit": seamgrim_numeric_factor_policy_values[
                "fallback_limit"
            ],
            "ci_sanity_seamgrim_numeric_factor_policy_small_prime_max": seamgrim_numeric_factor_policy_values[
                "small_prime_max"
            ],
            "ci_sanity_pack_golden_graph_export_ok": pack_golden_graph_export_ok,
            "ci_sanity_fixed64_darwin_real_report_live_report_path": fixed64_live_report_path_text,
            "ci_sanity_fixed64_darwin_real_report_live_report_exists": "1",
            "ci_sanity_fixed64_darwin_real_report_live_status": fixed64_live_status,
            "ci_sanity_fixed64_darwin_real_report_live_resolved_status": fixed64_live_resolved_status,
            "ci_sanity_fixed64_darwin_real_report_live_resolved_source": fixed64_live_resolved_source,
            "ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count": fixed64_live_invalid_count,
            "ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip": fixed64_live_source_zip,
            "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_ok": "1",
            "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_path": smoke_report_path_text,
            "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists": "1",
            "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_schema": (
                AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA
            ),
            "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok": "1",
            "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok": "1",
            "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes": "1",
            "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes": "1",
            "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_ok": seamgrim_wasm_web_step_check_ok,
            "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_path": (
                seamgrim_wasm_web_step_check_report_path_text
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_exists": (
                seamgrim_wasm_web_step_check_report_exists
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_schema": (
                seamgrim_wasm_web_step_check_schema
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_checked_files": (
                seamgrim_wasm_web_step_check_checked_files
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_missing_count": (
                seamgrim_wasm_web_step_check_missing_count
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok": (
                seamgrim_pack_evidence_tier_runner_ok
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_path": (
                seamgrim_pack_evidence_tier_runner_report_path_text
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists": (
                seamgrim_pack_evidence_tier_runner_report_exists
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_schema": (
                seamgrim_pack_evidence_tier_runner_schema
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count": (
                seamgrim_pack_evidence_tier_runner_docs_issue_count
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count": (
                seamgrim_pack_evidence_tier_runner_repo_issue_count
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok": (
                seamgrim_numeric_factor_policy_ok
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_report_path": (
                seamgrim_numeric_factor_policy_report_path_text
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_report_exists": (
                seamgrim_numeric_factor_policy_report_exists
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_schema": (
                seamgrim_numeric_factor_policy_schema
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_text": (
                seamgrim_numeric_factor_policy_text
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_bit_limit": (
                seamgrim_numeric_factor_policy_values["bit_limit"]
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_iters": (
                seamgrim_numeric_factor_policy_values["pollard_iters"]
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds": (
                seamgrim_numeric_factor_policy_values["pollard_c_seeds"]
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds": (
                seamgrim_numeric_factor_policy_values["pollard_x0_seeds"]
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_fallback_limit": (
                seamgrim_numeric_factor_policy_values["fallback_limit"]
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_small_prime_max": (
                seamgrim_numeric_factor_policy_values["small_prime_max"]
            ),
            "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_report_path": fixed64_live_report_path_text,
            "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_report_exists": "1",
            "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_status": fixed64_live_status,
            "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_status": (
                fixed64_live_resolved_status
            ),
            "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_source": (
                fixed64_live_resolved_source
            ),
            "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count": (
                fixed64_live_invalid_count
            ),
            "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip": (
                fixed64_live_source_zip
            ),
            "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok": pack_golden_graph_export_ok,
            "ci_sanity_fixed64_darwin_real_report_live_check_selftest_ok": "1",
            "ci_sanity_fixed64_threeway_inputs_selftest_ok": "1",
            "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_check_selftest_ok": "1",
            "ci_sync_readiness_ci_sanity_fixed64_threeway_inputs_selftest_ok": "1",
            "ci_sanity_age5_combined_heavy_policy_selftest_ok": "1",
            "ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok": "1",
            "ci_sanity_dynamic_source_profile_split_selftest_ok": "1",
            **build_age5_combined_heavy_sanity_contract_fields(),
            **build_age5_combined_heavy_sync_contract_fields(),
            "msg": "contract-only aggregate stub",
            "steps": [
                {
                    "step": "validate_only_sanity_json",
                    "ok": True,
                    "returncode": 0,
                }
            ],
        },
    )


def write_contract_only_fixed64_reports(inputs_report: Path, gate_report: Path) -> None:
    generated_at_utc = datetime.now(timezone.utc).isoformat()
    write_json(
        inputs_report,
        {
            "schema": "ddn.fixed64.threeway_inputs.v1",
            "generated_at_utc": generated_at_utc,
            "status": "contract_only",
            "ok": True,
            "reason": "contract_only_stub",
        },
    )
    write_json(
        gate_report,
        {
            "schema": "ddn.fixed64.cross_platform_threeway_gate.v1",
            "generated_at_utc": generated_at_utc,
            "status": "pending_darwin",
            "ok": True,
            "reason": "contract_only_stub",
        },
    )


def write_contract_only_ci_gate_outputs(
    profile: str,
    selected_profiles: list[str],
    profile_matrix_report: Path,
    age3_close_status_json: Path,
    age3_close_status_line: Path,
    age3_close_badge_json: Path,
    age3_close_summary_md: Path,
    seamgrim_wasm_cli_diag_parity_report: Path,
    final_status_line: Path,
    final_status_parse_json: Path,
    ci_gate_result_json: Path,
    ci_gate_badge_json: Path,
    ci_fail_brief_txt: Path,
    ci_fail_triage_json: Path,
    summary_path: Path,
    summary_line_path: Path,
    index_report_path: Path,
) -> None:
    generated_at_utc = datetime.now(timezone.utc).isoformat()
    age5_brief_tokens = build_contract_only_age5_brief_tokens()
    age5_triage_fields = build_contract_only_age5_triage_fields()
    age5_progress_fields = build_contract_only_age5_progress_fields()
    compact_line = (
        "status=pass reason=ok failed_steps=0 aggregate_status=pass overall_ok=1 "
        f"age4_proof_ok={CONTRACT_ONLY_AGE4_PROOF_OK} "
        f"age4_proof_failed_criteria={CONTRACT_ONLY_AGE4_PROOF_FAILED_CRITERIA} "
        f"age4_proof_failed_preview={CONTRACT_ONLY_AGE4_PROOF_FAILED_PREVIEW} "
        f"age4_proof_summary_hash={CONTRACT_ONLY_AGE4_PROOF_SUMMARY_HASH}"
    )
    profile_matrix_snapshot = build_contract_only_profile_matrix_triage_snapshot(profile_matrix_report, selected_profiles)
    selected_profiles_text = ",".join(selected_profiles)
    final_status_line.parent.mkdir(parents=True, exist_ok=True)
    final_status_line.write_text(compact_line + "\n", encoding="utf-8")
    summary_line_path.parent.mkdir(parents=True, exist_ok=True)
    summary_line_path.write_text(compact_line + "\n", encoding="utf-8")
    write_json(
        final_status_parse_json,
        {
            "schema": "ddn.ci.gate_final_status_line_parse.v1",
            "generated_at_utc": generated_at_utc,
            "status_line_path": str(final_status_line),
            "parsed": {
                "status": "pass",
                "reason": "ok",
                "failed_steps": "0",
                "aggregate_status": "pass",
                "overall_ok": "1",
                "age4_proof_ok": CONTRACT_ONLY_AGE4_PROOF_OK,
                "age4_proof_failed_criteria": CONTRACT_ONLY_AGE4_PROOF_FAILED_CRITERIA,
                "age4_proof_failed_preview": CONTRACT_ONLY_AGE4_PROOF_FAILED_PREVIEW,
                "age4_proof_summary_hash": CONTRACT_ONLY_AGE4_PROOF_SUMMARY_HASH,
                **age5_progress_fields,
            },
        },
    )
    write_json(
        ci_gate_result_json,
        {
            "schema": "ddn.ci.gate_result.v1",
            "generated_at_utc": generated_at_utc,
            "ok": True,
            "status": "pass",
            "reason": "ok",
            "overall_ok": True,
            "aggregate_status": "pass",
            "failed_steps": 0,
            "age4_proof_ok": True,
            "age4_proof_failed_criteria": 0,
            "age4_proof_failed_preview": CONTRACT_ONLY_AGE4_PROOF_FAILED_PREVIEW,
            "age4_proof_summary_hash": CONTRACT_ONLY_AGE4_PROOF_SUMMARY_HASH,
            **age5_progress_fields,
            "summary_line_path": str(summary_line_path),
            "summary_line": compact_line,
            "final_status_parse_path": str(final_status_parse_json),
            "gate_index_path": str(index_report_path),
        },
    )
    ci_gate_result_parse_path_text = str(ci_gate_result_json).replace(
        ".ci_gate_result.detjson",
        ".ci_gate_result_parse.detjson",
    )
    ci_gate_result_parse_json = Path(ci_gate_result_parse_path_text)
    if ci_gate_result_parse_json == ci_gate_result_json:
        ci_gate_result_parse_json = ci_gate_result_json.with_name(f"{ci_gate_result_json.stem}_parse.detjson")
    write_json(
        ci_gate_result_parse_json,
        {
            "schema": "ddn.ci.gate_result_parse.v1",
            "generated_at_utc": generated_at_utc,
            "result_path": str(ci_gate_result_json),
            "compact_line": compact_line,
            "parsed": {
                "status": "pass",
                "ok": True,
                "reason": "ok",
                "aggregate_status": "pass",
                "overall_ok": "1",
                "age4_proof_ok": CONTRACT_ONLY_AGE4_PROOF_OK,
                "age4_proof_failed_criteria": CONTRACT_ONLY_AGE4_PROOF_FAILED_CRITERIA,
                "age4_proof_failed_preview": CONTRACT_ONLY_AGE4_PROOF_FAILED_PREVIEW,
                **age5_progress_fields,
            },
        },
    )
    write_json(
        ci_gate_badge_json,
        {
            "schema": "ddn.ci.gate_badge.v1",
            "generated_at_utc": generated_at_utc,
            "status": "pass",
            "ok": True,
            "label": "ci:pass",
            "result_path": str(ci_gate_result_json),
        },
    )
    age3_close_status_json.write_text(
        json.dumps({"schema": "ddn.seamgrim.age3_close_status.v1", "status": "pass", "ok": True}, ensure_ascii=False, indent=2)
        + "\n",
        encoding="utf-8",
    )
    age3_close_status_line.write_text("status=pass overall_ok=1\n", encoding="utf-8")
    age3_close_badge_json.write_text(
        json.dumps({"schema": "ddn.seamgrim.age3_close_badge.v1", "status": "pass", "ok": True}, ensure_ascii=False, indent=2)
        + "\n",
        encoding="utf-8",
    )
    age3_close_summary_md.write_text("# AGE3 Close\n\nstatus=pass\n", encoding="utf-8")
    write_json(
        seamgrim_wasm_cli_diag_parity_report,
        {
            "schema": "ddn.seamgrim.wasm_cli_diag_parity.v1",
            "generated_at_utc": generated_at_utc,
            "status": "pass",
            "ok": True,
            "code": "OK",
            "step": "all",
            "steps": [],
        },
    )
    brief_line = (
        "status=pass reason=ok failed_steps_count=0 failed_steps=- top_step=- top_message=- "
        f"final_line={json.dumps(compact_line, ensure_ascii=False)} "
        f"age4_proof_ok={CONTRACT_ONLY_AGE4_PROOF_OK} "
        f"age4_proof_failed_criteria={CONTRACT_ONLY_AGE4_PROOF_FAILED_CRITERIA} "
        f"age4_proof_failed_preview={CONTRACT_ONLY_AGE4_PROOF_FAILED_PREVIEW} "
        f"age4_proof_summary_hash={CONTRACT_ONLY_AGE4_PROOF_SUMMARY_HASH} "
        + " ".join(
            f"{key}={json.dumps(value, ensure_ascii=False)}"
            if key in {
                AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_KEY,
                "age5_policy_combined_digest_selftest_default_field",
                AGE5_COMBINED_HEAVY_POLICY_ORIGIN_TRACE_KEY,
            }
            else f"{key}={value}"
            for key, value in age5_brief_tokens.items()
        )
        + " "
        f"profile_matrix_total_elapsed_ms=0 "
        f"profile_matrix_selected_real_profiles={json.dumps(selected_profiles_text, ensure_ascii=False)} "
        f"profile_matrix_core_lang_elapsed_ms={'0' if 'core_lang' in selected_profiles else '-'} "
        f"profile_matrix_full_elapsed_ms={'0' if 'full' in selected_profiles else '-'} "
        f"profile_matrix_seamgrim_elapsed_ms={'0' if 'seamgrim' in selected_profiles else '-'}"
    )
    ci_fail_brief_txt.write_text(brief_line + "\n", encoding="utf-8")

    def triage_artifact_row(path: Path) -> dict[str, object]:
        path_text = str(path)
        return {
            "path": path_text,
            "path_norm": path_text.replace("\\", "/"),
            "exists": True,
        }

    write_json(
        ci_fail_triage_json,
        {
            "schema": "ddn.ci.fail_triage.v1",
            "generated_at_utc": generated_at_utc,
            "status": "pass",
            "reason": "ok",
            "report_prefix": index_report_path.stem.replace(".ci_gate_report_index", ""),
            "final_line": compact_line,
            "age4_proof_ok": CONTRACT_ONLY_AGE4_PROOF_OK,
            "age4_proof_failed_criteria": CONTRACT_ONLY_AGE4_PROOF_FAILED_CRITERIA,
            "age4_proof_failed_preview": CONTRACT_ONLY_AGE4_PROOF_FAILED_PREVIEW,
            "age4_proof_summary_hash": CONTRACT_ONLY_AGE4_PROOF_SUMMARY_HASH,
            **age5_triage_fields,
            "summary_verify_ok": True,
            "summary_verify_issues": [],
            "summary_verify_issues_count": 0,
            "summary_verify_top_issue": "-",
            "failed_steps": [],
            "failed_steps_count": 0,
            "failed_step_detail_rows_count": 0,
            "failed_step_logs_rows_count": 0,
            "failed_step_detail_order": [],
            "failed_step_logs_order": [],
            "aggregate_digest": [],
            "aggregate_digest_count": 0,
            "summary_report_path_hint": str(summary_path),
            "summary_report_path_hint_norm": str(summary_path).replace("\\", "/"),
            "profile_matrix_selftest": profile_matrix_snapshot,
            "artifacts": {
                "summary": triage_artifact_row(summary_path),
                "summary_line": triage_artifact_row(summary_line_path),
                "ci_gate_result_json": triage_artifact_row(ci_gate_result_json),
                "ci_gate_badge_json": triage_artifact_row(ci_gate_badge_json),
                "ci_fail_brief_txt": triage_artifact_row(ci_fail_brief_txt),
                "ci_fail_triage_json": triage_artifact_row(ci_fail_triage_json),
            },
        },
    )
