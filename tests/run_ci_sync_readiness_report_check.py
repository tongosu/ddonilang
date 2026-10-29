#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

from _ci_age3_completion_gate_contract import (
    AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS,
    AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS,
)
from _ci_age5_combined_heavy_contract import (
    AGE5_COMBINED_HEAVY_SANITY_CONTRACT_SUMMARY_FIELDS,
    AGE5_COMBINED_HEAVY_SYNC_CONTRACT_SUMMARY_FIELDS,
)
from _ci_seamgrim_step_contract import (
    SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS,
    SEAMGRIM_PLATFORM_SANITY_SUMMARY_STEP_FIELDS,
)
from ci_check_error_codes import SYNC_READINESS_REPORT_CODES as CODES


VALID_STATUS = {"pass", "fail"}
VALID_FAIL_CODES = {
    "E_SYNC_READINESS_STEP_FAIL",
    "E_SYNC_READINESS_SANITY_CONTRACT_FAIL",
    "E_SYNC_READINESS_VALIDATE_ONLY_PATH_MISSING",
}
BASE_STEP_PREFIX = [
    "pipeline_emit_flags_check",
    "pipeline_emit_flags_selftest",
    "sanity_gate_diagnostics_check",
    "sanity_gate",
]
VALID_SANITY_PROFILES = {"full", "core_lang", "seamgrim"}
SANITY_SUMMARY_FIELDS = (
    ("ci_sanity_pipeline_emit_flags_ok", {"full", "core_lang"}),
    ("ci_sanity_pipeline_emit_flags_selftest_ok", {"full", "core_lang"}),
    ("ci_sanity_emit_artifacts_sanity_contract_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age2_close_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age2_close_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_close_ok", {"full", "seamgrim"}),
    ("ci_sanity_age3_close_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age5_combined_heavy_policy_selftest_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok", {"full", "core_lang", "seamgrim"}),
    *[(summary_key, {"seamgrim"}) for summary_key, _step_name in SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS],
    *[(summary_key, {"seamgrim"}) for summary_key, _step_name in SEAMGRIM_PLATFORM_SANITY_SUMMARY_STEP_FIELDS],
)
PACK_GOLDEN_GRAPH_EXPORT_SUMMARY_KEY = "ci_sanity_pack_golden_graph_export_ok"
PACK_GOLDEN_GRAPH_EXPORT_SYNC_KEY = "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok"
PACK_GOLDEN_GRAPH_EXPORT_REQUIRED_PROFILES = {"full", "core_lang"}
CLOSE_SUMMARY_SYNC_FIELDS = (
    "ci_sync_readiness_ci_sanity_emit_artifacts_sanity_contract_selftest_ok",
    "ci_sync_readiness_ci_sanity_age2_close_ok",
    "ci_sync_readiness_ci_sanity_age2_close_selftest_ok",
    "ci_sync_readiness_ci_sanity_age3_close_ok",
    "ci_sync_readiness_ci_sanity_age3_close_selftest_ok",
)
AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA = "ddn.bogae_geoul_visibility_smoke.v1"
AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_ENABLED_PROFILES = {"full", "core_lang", "seamgrim"}
AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_BOOL_FIELDS = (
    ("ci_sanity_age3_bogae_geoul_visibility_smoke_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes", {"full", "core_lang", "seamgrim"}),
)
AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_TEXT_FIELDS = (
    "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_schema",
)
AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SYNC_FIELD_PAIRS = (
    (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_ok",
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_ok",
    ),
    (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path",
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_path",
    ),
    (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists",
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists",
    ),
    (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_schema",
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_schema",
    ),
    (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok",
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok",
    ),
    (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok",
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok",
    ),
    (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes",
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes",
    ),
    (
        "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes",
        "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes",
    ),
)
COMPLETION_FAILURE_CODE_ENABLED_PROFILES = {"full", "core_lang", "seamgrim"}
AGE3_COMPLETION_GATE_CRITERIA_ENABLED_PROFILES = {"full", "core_lang", "seamgrim"}
COMPLETION_FAILURE_CODE_FIELD_PAIRS = (
    (
        "ci_sanity_age2_completion_gate_failure_codes",
        "ci_sanity_age2_completion_gate_failure_code_count",
        COMPLETION_FAILURE_CODE_ENABLED_PROFILES,
    ),
    (
        "ci_sanity_age3_completion_gate_failure_codes",
        "ci_sanity_age3_completion_gate_failure_code_count",
        COMPLETION_FAILURE_CODE_ENABLED_PROFILES,
    ),
)
SYNC_COMPLETION_FAILURE_CODE_FIELD_PAIRS = (
    (
        "ci_sync_readiness_ci_sanity_age2_completion_gate_failure_codes",
        "ci_sync_readiness_ci_sanity_age2_completion_gate_failure_code_count",
        "ci_sanity_age2_completion_gate_failure_codes",
        "ci_sanity_age2_completion_gate_failure_code_count",
        COMPLETION_FAILURE_CODE_ENABLED_PROFILES,
    ),
    (
        "ci_sync_readiness_ci_sanity_age3_completion_gate_failure_codes",
        "ci_sync_readiness_ci_sanity_age3_completion_gate_failure_code_count",
        "ci_sanity_age3_completion_gate_failure_codes",
        "ci_sanity_age3_completion_gate_failure_code_count",
        COMPLETION_FAILURE_CODE_ENABLED_PROFILES,
    ),
)
FAILURE_CODE_PATTERN = re.compile(r"[EW]_[A-Z0-9_]+")
VALID_SANITY_SUMMARY_VALUES = {"1", "0", "na", "pending"}
SANITY_CONTRACT_SUMMARY_FIELDS = AGE5_COMBINED_HEAVY_SANITY_CONTRACT_SUMMARY_FIELDS
SYNC_CONTRACT_SUMMARY_FIELDS = AGE5_COMBINED_HEAVY_SYNC_CONTRACT_SUMMARY_FIELDS
AGE5_COMBINED_HEAVY_SANITY_CONTRACT_SUMMARY_KEYS = (
    "ci_sanity_age5_combined_heavy_report_schema",
    "ci_sanity_age5_combined_heavy_required_reports",
    "ci_sanity_age5_combined_heavy_required_criteria",
    "ci_sanity_age5_combined_heavy_child_summary_default_fields",
    "ci_sanity_age5_combined_heavy_combined_contract_summary_fields",
    "ci_sanity_age5_combined_heavy_full_summary_contract_fields",
)
AGE5_COMBINED_HEAVY_SYNC_CONTRACT_SUMMARY_KEYS = (
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_report_schema",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_required_reports",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_required_criteria",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_combined_contract_summary_fields",
    "ci_sync_readiness_ci_sanity_age5_combined_heavy_full_summary_contract_fields",
)
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA = "ddn.pack_evidence_tier_runner_check.v1"
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_MAX_DOCS_ISSUES = 10
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_EXPECTED_REPO_ISSUES = 0
SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA = "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1"
SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES = 20
SEAMGRIM_PACK_EVIDENCE_SYNC_FIELD_PAIRS = (
    (
        "ci_sanity_seamgrim_pack_evidence_tier_runner_ok",
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_ok",
    ),
    (
        "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path",
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_path",
    ),
    (
        "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists",
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists",
    ),
    (
        "ci_sanity_seamgrim_pack_evidence_tier_runner_schema",
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_schema",
    ),
    (
        "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count",
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count",
    ),
    (
        "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count",
        "ci_sync_readiness_ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count",
    ),
)
SEAMGRIM_WASM_WEB_STEP_CHECK_SYNC_FIELD_PAIRS = (
    (
        "ci_sanity_seamgrim_wasm_web_step_check_ok",
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_ok",
    ),
    (
        "ci_sanity_seamgrim_wasm_web_step_check_report_path",
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_path",
    ),
    (
        "ci_sanity_seamgrim_wasm_web_step_check_report_exists",
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_exists",
    ),
    (
        "ci_sanity_seamgrim_wasm_web_step_check_schema",
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_schema",
    ),
    (
        "ci_sanity_seamgrim_wasm_web_step_check_checked_files",
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_checked_files",
    ),
    (
        "ci_sanity_seamgrim_wasm_web_step_check_missing_count",
        "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_missing_count",
    ),
)


def fail(msg: str, code: str = "E_CHECK") -> int:
    print(f"[ci-sync-readiness-report-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8-sig"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def sync_mirror_key(sanity_key: str) -> str:
    return f"ci_sync_readiness_{sanity_key}"


def validate_failure_code_pair(
    doc: dict,
    code_key: str,
    count_key: str,
    enabled: bool,
    allow_pending: bool,
) -> str | None:
    code_value = str(doc.get(code_key, "")).strip()
    if not code_value:
        return f"failure-code key missing: {code_key}"
    count_value = str(doc.get(count_key, "")).strip()
    if not count_value:
        return f"failure-code key missing: {count_key}"
    if not enabled:
        if code_value != "na":
            return f"failure-code expected na: {code_key}={code_value}"
        if count_value != "na":
            return f"failure-code expected na: {count_key}={count_value}"
        return None
    if code_value == "pending" or count_value == "pending":
        if allow_pending and code_value == "pending" and count_value == "pending":
            return None
        return f"failure-code pending mismatch: {code_key}={code_value} {count_key}={count_value}"
    if code_value == "na" or count_value == "na":
        return f"failure-code na mismatch: {code_key}={code_value} {count_key}={count_value}"
    try:
        count_num = int(count_value)
    except Exception:
        return f"failure-code count invalid: {count_key}={count_value}"
    if count_num < 0:
        return f"failure-code count negative: {count_key}={count_num}"
    if code_value == "-":
        if count_num != 0:
            return f"failure-code/count mismatch: {code_key}={code_value} {count_key}={count_num}"
        return None
    code_items = [token.strip() for token in code_value.split(",") if token.strip()]
    if len(code_items) != count_num:
        return f"failure-code/count mismatch: {code_key}={code_value} {count_key}={count_num}"
    if len(set(code_items)) != len(code_items):
        return f"failure-code duplicated: {code_key}={code_value}"
    for token in code_items:
        if not FAILURE_CODE_PATTERN.fullmatch(token):
            return f"failure-code token invalid: {code_key}={token}"
    return None


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate ci_sync_readiness.detjson contract")
    parser.add_argument("--report", required=True, help="path to ci_sync_readiness.detjson")
    parser.add_argument("--require-pass", action="store_true", help="require status=pass")
    parser.add_argument(
        "--sanity-profile",
        choices=("full", "core_lang", "seamgrim"),
        default="",
        help="optional expected sanity profile",
    )
    args = parser.parse_args()

    report_path = Path(args.report)
    if not report_path.exists():
        return fail(f"missing report: {report_path}", code=CODES["REPORT_MISSING"])
    doc = load_json(report_path)
    if not isinstance(doc, dict):
        return fail(f"invalid json: {report_path}", code=CODES["JSON_INVALID"])

    if str(doc.get("schema", "")).strip() != "ddn.ci.sync_readiness.v1":
        return fail(f"schema mismatch: {doc.get('schema')}", code=CODES["SCHEMA"])

    status = str(doc.get("status", "")).strip()
    if status not in VALID_STATUS:
        return fail(f"invalid status: {status}", code=CODES["STATUS"])
    if args.require_pass and status != "pass":
        return fail(f"require-pass set but status={status}", code=CODES["STATUS"])

    ok_value = doc.get("ok")
    if not isinstance(ok_value, bool):
        return fail("ok must be bool", code=CODES["OK_TYPE"])
    if ok_value != (status == "pass"):
        return fail(f"status/ok mismatch status={status} ok={ok_value}", code=CODES["STATUS_OK_MISMATCH"])

    sanity_profile = str(doc.get("sanity_profile", "")).strip() or "full"
    if sanity_profile not in VALID_SANITY_PROFILES:
        return fail(f"invalid sanity_profile: {sanity_profile}", code=CODES["STATUS"])
    expected_sanity_profile = args.sanity_profile.strip()
    if expected_sanity_profile and sanity_profile != expected_sanity_profile:
        return fail(
            f"sanity_profile mismatch expected={expected_sanity_profile} actual={sanity_profile}",
            code=CODES["STATUS_OK_MISMATCH"],
        )

    code = str(doc.get("code", "")).strip()
    step = str(doc.get("step", "")).strip()
    msg = str(doc.get("msg", "")).strip()
    if not code:
        return fail("code missing", code=CODES["CODE"])
    if not step:
        return fail("step missing", code=CODES["STEP"])
    if not msg:
        return fail("msg missing", code=CODES["MSG"])

    steps = doc.get("steps")
    if not isinstance(steps, list):
        return fail("steps must be list", code=CODES["STEPS_TYPE"])
    try:
        steps_count = int(doc.get("steps_count", -1))
    except Exception:
        return fail("steps_count must be int", code=CODES["STEPS_COUNT"])
    if steps_count != len(steps):
        return fail(f"steps_count mismatch report={steps_count} actual={len(steps)}", code=CODES["STEPS_COUNT"])

    names: list[str] = []
    for idx, row in enumerate(steps):
        if not isinstance(row, dict):
            return fail(f"steps[{idx}] must be object", code=CODES["ROW_TYPE"])
        name = str(row.get("name", "")).strip()
        if not name:
            return fail(f"steps[{idx}] name missing", code=CODES["ROW_NAME"])
        if not isinstance(row.get("ok"), bool):
            return fail(f"steps[{idx}] ok must be bool", code=CODES["ROW_OK_TYPE"])
        try:
            int(row.get("returncode", -1))
        except Exception:
            return fail(f"steps[{idx}] returncode must be int", code=CODES["ROW_RC_TYPE"])
        names.append(name)

    for key, enabled_profiles in SANITY_SUMMARY_FIELDS:
        raw_value = str(doc.get(key, "")).strip()
        if not raw_value:
            return fail(f"sanity summary key missing: {key}", code=CODES["SANITY_SUMMARY_KEY_MISSING"])
        if raw_value not in VALID_SANITY_SUMMARY_VALUES:
            return fail(
                f"sanity summary value invalid: {key}={raw_value}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if sanity_profile not in enabled_profiles and raw_value != "na":
            return fail(
                f"sanity summary must be na for profile={sanity_profile}: {key}={raw_value}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        sync_key = sync_mirror_key(key)
        sync_value = str(doc.get(sync_key, "")).strip()
        if not sync_value:
            return fail(
                f"sync sanity summary key missing: {sync_key}",
                code=CODES["SANITY_SUMMARY_KEY_MISSING"],
            )
        if sync_value not in VALID_SANITY_SUMMARY_VALUES:
            return fail(
                f"sync sanity summary value invalid: {sync_key}={sync_value}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if sync_value != raw_value:
            return fail(
                f"sync sanity summary mismatch: {sync_key} expected={raw_value} actual={sync_value}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
    pack_golden_graph_export_value = str(doc.get(PACK_GOLDEN_GRAPH_EXPORT_SUMMARY_KEY, "")).strip()
    if pack_golden_graph_export_value not in {"0", "1"}:
        return fail(
            "sanity summary value invalid: "
            f"{PACK_GOLDEN_GRAPH_EXPORT_SUMMARY_KEY}={pack_golden_graph_export_value}",
            code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
        )
    expected_pack_golden_graph_export_value = (
        "1" if sanity_profile in PACK_GOLDEN_GRAPH_EXPORT_REQUIRED_PROFILES else "0"
    )
    if pack_golden_graph_export_value != expected_pack_golden_graph_export_value:
        return fail(
            "sanity summary mismatch: "
            f"{PACK_GOLDEN_GRAPH_EXPORT_SUMMARY_KEY} expected={expected_pack_golden_graph_export_value} "
            f"actual={pack_golden_graph_export_value}",
            code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
        )
    pack_golden_graph_export_sync_value = str(doc.get(PACK_GOLDEN_GRAPH_EXPORT_SYNC_KEY, "")).strip()
    if not pack_golden_graph_export_sync_value:
        return fail(
            f"sync sanity summary key missing: {PACK_GOLDEN_GRAPH_EXPORT_SYNC_KEY}",
            code=CODES["SANITY_SUMMARY_KEY_MISSING"],
        )
    if pack_golden_graph_export_sync_value not in {"0", "1"}:
        return fail(
            "sync sanity summary value invalid: "
            f"{PACK_GOLDEN_GRAPH_EXPORT_SYNC_KEY}={pack_golden_graph_export_sync_value}",
            code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
        )
    if pack_golden_graph_export_sync_value != pack_golden_graph_export_value:
        return fail(
            "sync sanity summary mismatch: "
            f"{PACK_GOLDEN_GRAPH_EXPORT_SYNC_KEY} expected={pack_golden_graph_export_value} "
            f"actual={pack_golden_graph_export_sync_value}",
            code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
        )
    for sanity_key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS:
        raw_value = str(doc.get(sanity_key, "")).strip()
        if not raw_value:
            return fail(
                f"sanity age3 criteria key missing: {sanity_key}",
                code=CODES["SANITY_SUMMARY_KEY_MISSING"],
            )
        if raw_value not in VALID_SANITY_SUMMARY_VALUES:
            return fail(
                f"sanity age3 criteria value invalid: {sanity_key}={raw_value}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if sanity_profile not in AGE3_COMPLETION_GATE_CRITERIA_ENABLED_PROFILES and raw_value != "na":
            return fail(
                f"sanity age3 criteria must be na for profile={sanity_profile}: {sanity_key}={raw_value}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if (
            sanity_profile in AGE3_COMPLETION_GATE_CRITERIA_ENABLED_PROFILES
            and status == "pass"
            and raw_value != "1"
        ):
            return fail(
                f"pass sanity age3 criteria invalid: {sanity_key}={raw_value}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
    for sanity_key, sync_key in AGE3_COMPLETION_GATE_CRITERIA_SYNC_FIELD_PAIRS:
        sanity_value = str(doc.get(sanity_key, "")).strip()
        sync_value = str(doc.get(sync_key, "")).strip()
        if not sync_value:
            return fail(
                f"sync age3 criteria key missing: {sync_key}",
                code=CODES["SANITY_SUMMARY_KEY_MISSING"],
            )
        if sync_value != sanity_value:
            return fail(
                f"sync age3 criteria mismatch: {sync_key} expected={sanity_value} actual={sync_value}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
    for key, enabled_profiles in AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_BOOL_FIELDS:
        raw_value = str(doc.get(key, "")).strip()
        if not raw_value:
            return fail(f"sanity smoke summary key missing: {key}", code=CODES["SANITY_SUMMARY_KEY_MISSING"])
        if raw_value not in VALID_SANITY_SUMMARY_VALUES:
            return fail(
                f"sanity smoke summary value invalid: {key}={raw_value}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if sanity_profile not in enabled_profiles and raw_value != "na":
            return fail(
                f"sanity smoke summary must be na for profile={sanity_profile}: {key}={raw_value}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
    for code_key, count_key, enabled_profiles in COMPLETION_FAILURE_CODE_FIELD_PAIRS:
        error = validate_failure_code_pair(
            doc,
            code_key=code_key,
            count_key=count_key,
            enabled=sanity_profile in enabled_profiles,
            allow_pending=status != "pass",
        )
        if error is not None:
            return fail(error, code=CODES["SANITY_SUMMARY_VALUE_INVALID"])
    for sync_code_key, sync_count_key, source_code_key, source_count_key, enabled_profiles in SYNC_COMPLETION_FAILURE_CODE_FIELD_PAIRS:
        error = validate_failure_code_pair(
            doc,
            code_key=sync_code_key,
            count_key=sync_count_key,
            enabled=sanity_profile in enabled_profiles,
            allow_pending=status != "pass",
        )
        if error is not None:
            return fail(error, code=CODES["SANITY_SUMMARY_VALUE_INVALID"])
        source_code_value = str(doc.get(source_code_key, "")).strip()
        sync_code_value = str(doc.get(sync_code_key, "")).strip()
        if sync_code_value != source_code_value:
            return fail(
                f"sync failure-code mismatch: {sync_code_key} expected={source_code_value} actual={sync_code_value}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        source_count_value = str(doc.get(source_count_key, "")).strip()
        sync_count_value = str(doc.get(sync_count_key, "")).strip()
        if sync_count_value != source_count_value:
            return fail(
                f"sync failure-code count mismatch: {sync_count_key} expected={source_count_value} actual={sync_count_value}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
    smoke_report_path = str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_report_path", "")).strip() or "-"
    smoke_schema = str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_schema", "")).strip() or "-"
    smoke_enabled = sanity_profile in AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_ENABLED_PROFILES
    if not smoke_enabled:
        if smoke_report_path != "-":
            return fail(
                f"sanity smoke report_path must be '-' for profile={sanity_profile}: {smoke_report_path}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if smoke_schema != "-":
            return fail(
                f"sanity smoke schema must be '-' for profile={sanity_profile}: {smoke_schema}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
    else:
        if smoke_report_path == "-":
            return fail(
                "sanity smoke report_path missing",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if smoke_schema != AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA:
            return fail(
                f"sanity smoke schema mismatch: {smoke_schema}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        smoke_report_doc = load_json(Path(smoke_report_path))
        if not isinstance(smoke_report_doc, dict):
            return fail(
                f"invalid sanity smoke report: {smoke_report_path}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if str(smoke_report_doc.get("schema", "")).strip() != AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA:
            return fail(
                f"sanity smoke report schema mismatch: {smoke_report_doc.get('schema')}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if not bool(smoke_report_doc.get("overall_ok", False)):
            return fail(
                "sanity smoke report overall_ok must be true",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        smoke_checks = smoke_report_doc.get("checks")
        if not isinstance(smoke_checks, list) or not smoke_checks:
            return fail(
                "sanity smoke report checks must be non-empty list",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        smoke_sim_hash_delta = smoke_report_doc.get("simulation_hash_delta")
        if not isinstance(smoke_sim_hash_delta, dict):
            return fail(
                "sanity smoke report simulation_hash_delta must be object",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        smoke_sim_state_hash_changes = "1" if bool(smoke_sim_hash_delta.get("state_hash_changes", False)) else "0"
        smoke_sim_bogae_hash_changes = "1" if bool(smoke_sim_hash_delta.get("bogae_hash_changes", False)) else "0"
        if smoke_sim_state_hash_changes != "1":
            return fail(
                "sanity smoke report requires simulation_hash_delta.state_hash_changes=true",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if smoke_sim_bogae_hash_changes != "1":
            return fail(
                "sanity smoke report requires simulation_hash_delta.bogae_hash_changes=true",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes", "")).strip() != (
            smoke_sim_state_hash_changes
        ):
            return fail(
                "sanity smoke summary/report mismatch: sim_state_hash_changes",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes", "")).strip() != (
            smoke_sim_bogae_hash_changes
        ):
            return fail(
                "sanity smoke summary/report mismatch: sim_bogae_hash_changes",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
    for sanity_key, sync_key in AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SYNC_FIELD_PAIRS:
        sanity_value = str(doc.get(sanity_key, "")).strip()
        sync_value = str(doc.get(sync_key, "")).strip()
        if not sync_value:
            return fail(f"sync smoke summary key missing: {sync_key}", code=CODES["SANITY_SUMMARY_KEY_MISSING"])
        if sync_value != sanity_value:
            return fail(
                f"sync smoke summary mismatch: {sync_key} expected={sanity_value} actual={sync_value}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
    for sanity_key, sync_key in SEAMGRIM_PACK_EVIDENCE_SYNC_FIELD_PAIRS:
        sanity_value = str(doc.get(sanity_key, "")).strip()
        if not sanity_value:
            return fail(
                f"seamgrim pack evidence key missing: {sanity_key}",
                code=CODES["SANITY_SUMMARY_KEY_MISSING"],
            )
        sync_value = str(doc.get(sync_key, "")).strip()
        if not sync_value:
            return fail(
                f"seamgrim pack evidence key missing: {sync_key}",
                code=CODES["SANITY_SUMMARY_KEY_MISSING"],
            )
        if sync_value != sanity_value:
            return fail(
                f"sync seamgrim pack evidence mismatch: {sync_key} expected={sanity_value} actual={sync_value}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
    for sanity_key, sync_key in SEAMGRIM_WASM_WEB_STEP_CHECK_SYNC_FIELD_PAIRS:
        sanity_value = str(doc.get(sanity_key, "")).strip()
        if not sanity_value:
            return fail(
                f"seamgrim wasm step key missing: {sanity_key}",
                code=CODES["SANITY_SUMMARY_KEY_MISSING"],
            )
        sync_value = str(doc.get(sync_key, "")).strip()
        if not sync_value:
            return fail(
                f"seamgrim wasm step key missing: {sync_key}",
                code=CODES["SANITY_SUMMARY_KEY_MISSING"],
            )
        if sync_value != sanity_value:
            return fail(
                f"sync seamgrim wasm step mismatch: {sync_key} expected={sanity_value} actual={sync_value}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )

    seamgrim_enabled = sanity_profile == "seamgrim"
    pack_evidence_ok = str(doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_ok", "")).strip()
    pack_evidence_report_path = str(doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_report_path", "")).strip()
    pack_evidence_report_exists = str(doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists", "")).strip()
    pack_evidence_schema = str(doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_schema", "")).strip()
    pack_evidence_docs_issue_count = str(doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count", "")).strip()
    pack_evidence_repo_issue_count = str(doc.get("ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count", "")).strip()
    seamgrim_wasm_ok = str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_ok", "")).strip()
    seamgrim_wasm_report_path = str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_report_path", "")).strip()
    seamgrim_wasm_report_exists = str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_report_exists", "")).strip()
    seamgrim_wasm_schema = str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_schema", "")).strip()
    seamgrim_wasm_checked_files = str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_checked_files", "")).strip()
    seamgrim_wasm_missing_count = str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_missing_count", "")).strip()
    if not seamgrim_enabled:
        disabled_expected = {
            "ci_sanity_seamgrim_pack_evidence_tier_runner_ok": "na",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path": "-",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists": "na",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_schema": "-",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count": "-",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count": "-",
            "ci_sanity_seamgrim_wasm_web_step_check_ok": "na",
            "ci_sanity_seamgrim_wasm_web_step_check_report_path": "-",
            "ci_sanity_seamgrim_wasm_web_step_check_report_exists": "na",
            "ci_sanity_seamgrim_wasm_web_step_check_schema": "-",
            "ci_sanity_seamgrim_wasm_web_step_check_checked_files": "-",
            "ci_sanity_seamgrim_wasm_web_step_check_missing_count": "-",
        }
        for key, expected_value in disabled_expected.items():
            raw_value = str(doc.get(key, "")).strip()
            if raw_value != expected_value:
                return fail(
                    f"seamgrim disabled value mismatch: {key} expected={expected_value} actual={raw_value}",
                    code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
                )
    else:
        if pack_evidence_ok != "1":
            return fail(
                f"seamgrim pack evidence ok must be 1, got={pack_evidence_ok}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if pack_evidence_report_exists != "1":
            return fail(
                f"seamgrim pack evidence report_exists must be 1, got={pack_evidence_report_exists}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if not pack_evidence_report_path or pack_evidence_report_path == "-":
            return fail(
                "seamgrim pack evidence report_path missing",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if pack_evidence_schema != SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA:
            return fail(
                f"seamgrim pack evidence schema mismatch: {pack_evidence_schema}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        try:
            pack_docs_issue_count_num = int(pack_evidence_docs_issue_count)
        except Exception:
            return fail(
                f"seamgrim pack evidence docs_issue_count invalid: {pack_evidence_docs_issue_count}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if (
            pack_docs_issue_count_num < 0
            or pack_docs_issue_count_num > SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_MAX_DOCS_ISSUES
        ):
            return fail(
                "seamgrim pack evidence docs_issue_count out of range: "
                f"{pack_docs_issue_count_num}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        try:
            pack_repo_issue_count_num = int(pack_evidence_repo_issue_count)
        except Exception:
            return fail(
                f"seamgrim pack evidence repo_issue_count invalid: {pack_evidence_repo_issue_count}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if pack_repo_issue_count_num != SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_EXPECTED_REPO_ISSUES:
            return fail(
                "seamgrim pack evidence repo_issue_count mismatch: "
                f"{pack_repo_issue_count_num}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        pack_report_doc = load_json(Path(pack_evidence_report_path))
        if not isinstance(pack_report_doc, dict):
            return fail(
                f"invalid seamgrim pack evidence report: {pack_evidence_report_path}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if str(pack_report_doc.get("schema", "")).strip() != SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA:
            return fail(
                f"seamgrim pack evidence report schema mismatch: {pack_report_doc.get('schema')}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if str(pack_report_doc.get("status", "")).strip() != "pass" or not bool(pack_report_doc.get("ok", False)):
            return fail(
                "seamgrim pack evidence report status/ok mismatch",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        docs_profile_doc = pack_report_doc.get("docs_profile")
        repo_profile_doc = pack_report_doc.get("repo_profile")
        if not isinstance(docs_profile_doc, dict) or not isinstance(repo_profile_doc, dict):
            return fail(
                "seamgrim pack evidence report profile payload missing",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if str(docs_profile_doc.get("name", "")).strip() != "docs_ssot_rep10":
            return fail(
                "seamgrim pack evidence docs_profile.name mismatch",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if str(repo_profile_doc.get("name", "")).strip() != "repo_rep10":
            return fail(
                "seamgrim pack evidence repo_profile.name mismatch",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        try:
            docs_issue_count_doc = int(docs_profile_doc.get("issue_count", -1))
            repo_issue_count_doc = int(repo_profile_doc.get("issue_count", -1))
        except Exception:
            return fail(
                "seamgrim pack evidence issue_count type mismatch",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if docs_issue_count_doc != pack_docs_issue_count_num:
            return fail(
                "seamgrim pack evidence docs_issue_count summary/report mismatch: "
                f"summary={pack_docs_issue_count_num} report={docs_issue_count_doc}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if repo_issue_count_doc != pack_repo_issue_count_num:
            return fail(
                "seamgrim pack evidence repo_issue_count summary/report mismatch: "
                f"summary={pack_repo_issue_count_num} report={repo_issue_count_doc}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )

        if seamgrim_wasm_ok != "1":
            return fail(
                f"seamgrim wasm step ok must be 1, got={seamgrim_wasm_ok}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if seamgrim_wasm_report_exists != "1":
            return fail(
                f"seamgrim wasm step report_exists must be 1, got={seamgrim_wasm_report_exists}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if not seamgrim_wasm_report_path or seamgrim_wasm_report_path == "-":
            return fail(
                "seamgrim wasm step report_path missing",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if seamgrim_wasm_schema != SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA:
            return fail(
                f"seamgrim wasm step schema mismatch: {seamgrim_wasm_schema}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        try:
            seamgrim_wasm_checked_files_num = int(seamgrim_wasm_checked_files)
            seamgrim_wasm_missing_count_num = int(seamgrim_wasm_missing_count)
        except Exception:
            return fail(
                "seamgrim wasm step numeric fields invalid",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if seamgrim_wasm_checked_files_num < SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES:
            return fail(
                "seamgrim wasm step checked_files too small: "
                f"{seamgrim_wasm_checked_files_num}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if seamgrim_wasm_missing_count_num != 0:
            return fail(
                f"seamgrim wasm step missing_count must be 0, got={seamgrim_wasm_missing_count_num}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        seamgrim_wasm_report_doc = load_json(Path(seamgrim_wasm_report_path))
        if not isinstance(seamgrim_wasm_report_doc, dict):
            return fail(
                f"invalid seamgrim wasm step report: {seamgrim_wasm_report_path}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if str(seamgrim_wasm_report_doc.get("schema", "")).strip() != SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA:
            return fail(
                f"seamgrim wasm step report schema mismatch: {seamgrim_wasm_report_doc.get('schema')}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if str(seamgrim_wasm_report_doc.get("status", "")).strip() != "pass" or not bool(
            seamgrim_wasm_report_doc.get("ok", False)
        ):
            return fail(
                "seamgrim wasm step report status/ok mismatch",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        try:
            seamgrim_wasm_report_checked_files_num = int(seamgrim_wasm_report_doc.get("checked_files", -1))
            seamgrim_wasm_report_missing_count_num = int(seamgrim_wasm_report_doc.get("missing_count", -1))
        except Exception:
            return fail(
                "seamgrim wasm step report numeric fields invalid",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if seamgrim_wasm_report_checked_files_num < SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES:
            return fail(
                "seamgrim wasm step report checked_files too small: "
                f"{seamgrim_wasm_report_checked_files_num}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
        if seamgrim_wasm_report_missing_count_num != seamgrim_wasm_missing_count_num:
            return fail(
                "seamgrim wasm step missing_count summary/report mismatch: "
                f"summary={seamgrim_wasm_missing_count_num} report={seamgrim_wasm_report_missing_count_num}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
    for key, expected_value in SANITY_CONTRACT_SUMMARY_FIELDS:
        raw_value = str(doc.get(key, "")).strip()
        if not raw_value:
            return fail(f"sanity contract key missing: {key}", code=CODES["SANITY_SUMMARY_KEY_MISSING"])
        if raw_value != expected_value:
            return fail(
                f"sanity contract value mismatch: {key} expected={expected_value} actual={raw_value}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )
    for key, expected_value in SYNC_CONTRACT_SUMMARY_FIELDS:
        raw_value = str(doc.get(key, "")).strip()
        if not raw_value:
            return fail(f"sync contract key missing: {key}", code=CODES["SANITY_SUMMARY_KEY_MISSING"])
        if raw_value != expected_value:
            return fail(
                f"sync contract value mismatch: {key} expected={expected_value} actual={raw_value}",
                code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
            )

    validate_only_path = str(doc.get("validate_only_sanity_json", "")).strip()
    has_contract_row = "sanity_gate_contract" in names
    if not has_contract_row:
        return fail("missing sanity_gate_contract row", code=CODES["MISSING_CONTRACT_ROW"])

    if status == "pass":
        if code != "OK" or step != "all" or msg != "-":
            return fail(
                f"pass fields invalid code={code} step={step} msg={msg}",
                code=CODES["PASS_STATUS_FIELDS"],
            )
        for key, enabled_profiles in SANITY_SUMMARY_FIELDS:
            raw_value = str(doc.get(key, "")).strip()
            expected_value = "na" if sanity_profile not in enabled_profiles else "1"
            if raw_value != expected_value:
                return fail(
                    f"pass sanity summary invalid: {key} expected={expected_value} actual={raw_value}",
                    code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
                )
        for key, enabled_profiles in AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_BOOL_FIELDS:
            raw_value = str(doc.get(key, "")).strip()
            expected_value = "na" if sanity_profile not in enabled_profiles else "1"
            if raw_value != expected_value:
                return fail(
                    f"pass sanity smoke summary invalid: {key} expected={expected_value} actual={raw_value}",
                    code=CODES["SANITY_SUMMARY_VALUE_INVALID"],
                )
        for idx, row in enumerate(steps):
            if not bool(row.get("ok", False)):
                return fail(f"pass row must be ok=1 idx={idx}", code=CODES["PASS_ROW_FAIL"])
            if int(row.get("returncode", 1)) != 0:
                return fail(
                    f"pass row returncode must be 0 idx={idx} rc={row.get('returncode')}",
                    code=CODES["PASS_ROW_FAIL"],
                )
    else:
        if code not in VALID_FAIL_CODES:
            return fail(f"invalid fail code: {code}", code=CODES["FAIL_STATUS_FIELDS"])
        if step == "all":
            return fail("fail step must not be all", code=CODES["FAIL_STATUS_FIELDS"])

    if validate_only_path:
        if len(steps) != 1 or names != ["sanity_gate_contract"]:
            return fail(
                f"validate-only shape invalid steps={names}",
                code=CODES["VALIDATE_ONLY_SHAPE"],
            )
    else:
        if status == "pass":
            if len(names) < 5:
                return fail(f"pass non-validate steps too small: {len(names)}", code=CODES["QUICK_BASE_STEPS"])
            if names[:4] != BASE_STEP_PREFIX:
                return fail(
                    f"base steps mismatch got={names[:4]}",
                    code=CODES["QUICK_BASE_STEPS"],
                )

    print(
        f"[ci-sync-readiness-report-check] ok report={report_path} "
        f"status={status} sanity_profile={sanity_profile} steps={len(steps)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
