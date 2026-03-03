#!/usr/bin/env python
from __future__ import annotations

import argparse
from datetime import datetime
import json
import sys
from pathlib import Path

from ci_check_error_codes import GATE_REPORT_INDEX_CODES as CODES

INDEX_SCHEMA = "ddn.ci.aggregate_gate.index.v1"

REQUIRED_REPORT_PATH_KEYS = (
    "summary",
    "summary_line",
    "final_status_parse_json",
    "ci_gate_result_json",
    "ci_gate_badge_json",
    "ci_fail_brief_txt",
    "ci_fail_triage_json",
    "ci_sanity_gate",
    "ci_sync_readiness",
    "seamgrim_wasm_cli_diag_parity",
)

ARTIFACT_SCHEMA_MAP = {
    "final_status_parse_json": (
        "ddn.ci.gate_final_status_line_parse.v1",
        "ddn.ci.status_line.parse.v1",
    ),
    "ci_gate_result_json": "ddn.ci.gate_result.v1",
    "ci_gate_badge_json": "ddn.ci.gate_badge.v1",
    "ci_fail_triage_json": "ddn.ci.fail_triage.v1",
    "ci_sanity_gate": "ddn.ci.sanity_gate.v1",
    "ci_sync_readiness": "ddn.ci.sync_readiness.v1",
    "seamgrim_wasm_cli_diag_parity": "ddn.seamgrim.wasm_cli_diag_parity.v1",
}

VALID_SANITY_PROFILES = ("full", "core_lang", "seamgrim")
PROFILE_REQUIRED_STEPS_COMMON = (
    "ci_profile_split_contract_check",
    "ci_profile_matrix_gate_selftest",
    "ci_sanity_gate",
    "ci_sync_readiness_report_generate",
    "ci_sync_readiness_report_check",
    "ci_gate_report_index_selftest",
    "ci_gate_report_index_diagnostics_check",
)
PROFILE_REQUIRED_STEPS_CORE_LANG = ()
PROFILE_REQUIRED_STEPS_SEAMGRIM = (
    "seamgrim_ci_gate_seed_meta_step_check",
    "seamgrim_ci_gate_runtime5_passthrough_check",
    "seamgrim_ci_gate_guideblock_step_check",
    "seamgrim_wasm_cli_diag_parity_check",
)


def fail(msg: str, code: str) -> int:
    print(f"[ci-gate-report-index-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def resolve_report_path(index_doc: dict, key: str) -> Path | None:
    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return None
    raw = str(reports.get(key, "")).strip()
    if not raw:
        return None
    return Path(raw.replace("\\", "/"))


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8").strip()
    except Exception:
        return ""


def normalize_path_text(raw: str) -> str:
    value = str(raw).strip()
    if not value:
        return ""
    return str(Path(value.replace("\\", "/")))


def parse_status_line_tokens(line: str) -> dict[str, str]:
    out: dict[str, str] = {}
    for part in str(line).strip().split():
        if "=" not in part:
            continue
        key, value = part.split("=", 1)
        key = key.strip()
        value = value.strip()
        if not key:
            continue
        if key in {"ci_gate_status", "ci_gate_result_status"}:
            out["status"] = value
            continue
        out[key] = value
    return out


def is_compatible_summary_line(result_summary_line: str, expected_summary_line: str) -> bool:
    result_tokens = parse_status_line_tokens(result_summary_line)
    expected_tokens = parse_status_line_tokens(expected_summary_line)
    required_keys = ("status", "overall_ok", "failed_steps", "aggregate_status", "reason")
    for key in required_keys:
        if result_tokens.get(key, "") != expected_tokens.get(key, ""):
            return False
    return True


def resolve_profile_required_steps(profile: str) -> tuple[str, ...]:
    if profile == "core_lang":
        return PROFILE_REQUIRED_STEPS_COMMON + PROFILE_REQUIRED_STEPS_CORE_LANG
    if profile == "seamgrim":
        return PROFILE_REQUIRED_STEPS_COMMON + PROFILE_REQUIRED_STEPS_SEAMGRIM
    return PROFILE_REQUIRED_STEPS_COMMON + PROFILE_REQUIRED_STEPS_SEAMGRIM


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate aggregate gate report-index schema and report paths")
    parser.add_argument("--index", required=True, help="path to ci_gate_report_index.detjson")
    parser.add_argument(
        "--required-step",
        action="append",
        default=[],
        help="required step name in index.steps (can be repeated)",
    )
    parser.add_argument(
        "--sanity-profile",
        choices=VALID_SANITY_PROFILES,
        default="full",
        help="sanity profile for implicit required-step contract",
    )
    parser.add_argument(
        "--enforce-profile-step-contract",
        action="store_true",
        help="enforce implicit required steps by --sanity-profile",
    )
    args = parser.parse_args()

    index_path = Path(args.index)
    if not index_path.exists():
        return fail(f"missing index file: {index_path}", CODES["INDEX_MISSING"])
    index_doc = load_json(index_path)
    if not isinstance(index_doc, dict):
        return fail(f"invalid index json: {index_path}", CODES["INDEX_JSON_INVALID"])
    if str(index_doc.get("schema", "")).strip() != INDEX_SCHEMA:
        return fail(
            f"index schema mismatch: {index_doc.get('schema')}",
            CODES["INDEX_SCHEMA"],
        )
    generated_at_utc = str(index_doc.get("generated_at_utc", "")).strip()
    if not generated_at_utc:
        return fail("index.generated_at_utc is missing", CODES["GENERATED_AT_MISSING"])
    try:
        dt_text = generated_at_utc[:-1] + "+00:00" if generated_at_utc.endswith("Z") else generated_at_utc
        datetime.fromisoformat(dt_text)
    except Exception:
        return fail(
            f"index.generated_at_utc invalid isoformat: {generated_at_utc}",
            CODES["GENERATED_AT_INVALID"],
        )
    report_dir_raw = str(index_doc.get("report_dir", "")).strip()
    if not report_dir_raw:
        return fail("index.report_dir is missing", CODES["REPORT_DIR_MISSING"])
    report_dir_path = Path(report_dir_raw.replace("\\", "/"))
    if not report_dir_path.exists():
        return fail(f"index.report_dir not found: {report_dir_path}", CODES["REPORT_DIR_NOT_FOUND"])
    report_prefix = str(index_doc.get("report_prefix", "")).strip()
    report_prefix_source = str(index_doc.get("report_prefix_source", "")).strip()
    if report_prefix:
        if not report_prefix_source:
            return fail(
                "index.report_prefix_source missing while report_prefix is set",
                CODES["REPORT_PREFIX_SOURCE_MISMATCH"],
            )
        if report_prefix_source != "arg" and not report_prefix_source.startswith("env:"):
            return fail(
                f"index.report_prefix_source invalid: {report_prefix_source}",
                CODES["REPORT_PREFIX_SOURCE_INVALID"],
            )
        if report_prefix_source.startswith("env:") and not report_prefix_source[4:].strip():
            return fail(
                f"index.report_prefix_source invalid: {report_prefix_source}",
                CODES["REPORT_PREFIX_SOURCE_INVALID"],
            )
    elif report_prefix_source:
        return fail(
            "index.report_prefix_source must be empty when report_prefix is empty",
            CODES["REPORT_PREFIX_SOURCE_MISMATCH"],
        )
    step_log_dir_raw = index_doc.get("step_log_dir", "")
    if not isinstance(step_log_dir_raw, str):
        return fail("index.step_log_dir must be string", CODES["STEP_LOG_DIR_TYPE"])
    step_log_dir = step_log_dir_raw.strip()
    if step_log_dir:
        step_log_dir_path = Path(step_log_dir.replace("\\", "/"))
        if not step_log_dir_path.exists():
            return fail(
                f"index.step_log_dir not found: {step_log_dir_path}",
                CODES["STEP_LOG_DIR_NOT_FOUND"],
            )
    step_log_failed_only = index_doc.get("step_log_failed_only")
    if not isinstance(step_log_failed_only, bool):
        return fail(
            "index.step_log_failed_only must be bool",
            CODES["STEP_LOG_FAILED_ONLY_TYPE"],
        )
    if step_log_failed_only and not step_log_dir:
        return fail(
            "index.step_log_failed_only=1 requires non-empty step_log_dir",
            CODES["STEP_LOG_CONFIG_MISMATCH"],
        )
    index_profile = str(index_doc.get("ci_sanity_profile", "")).strip()
    if index_profile not in VALID_SANITY_PROFILES:
        return fail(f"invalid ci_sanity_profile: {index_profile}", CODES["PROFILE_INVALID"])
    expected_profile = str(args.sanity_profile).strip()
    if expected_profile in VALID_SANITY_PROFILES and index_profile != expected_profile:
        return fail(
            f"ci_sanity_profile mismatch expected={expected_profile} actual={index_profile}",
            CODES["PROFILE_MISMATCH"],
        )

    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return fail("index.reports is missing", CODES["INDEX_REPORTS_MISSING"])

    steps = index_doc.get("steps")
    if steps is None:
        return fail("index.steps is missing", CODES["STEPS_MISSING"])
    if not isinstance(steps, list):
        return fail("index.steps must be list", CODES["STEPS_TYPE"])
    index_overall_ok = index_doc.get("overall_ok")
    if not isinstance(index_overall_ok, bool):
        return fail("index.overall_ok must be bool", CODES["INDEX_OVERALL_OK_TYPE"])
    seen_step_names: set[str] = set()
    failed_step_count = 0
    for idx, row in enumerate(steps):
        if not isinstance(row, dict):
            return fail(f"index.steps[{idx}] must be object", CODES["STEP_ROW_TYPE"])
        step_name = str(row.get("name", "")).strip()
        if not step_name:
            return fail(f"index.steps[{idx}].name missing", CODES["STEP_NAME"])
        if step_name in seen_step_names:
            return fail(f"index.steps duplicate name: {step_name}", CODES["STEP_DUP"])
        seen_step_names.add(step_name)
        ok_value = row.get("ok")
        if not isinstance(ok_value, bool):
            return fail(f"index.steps[{idx}].ok must be bool", CODES["STEP_OK_TYPE"])
        if not bool(ok_value):
            failed_step_count += 1
        rc = None
        try:
            rc = int(row.get("returncode"))
        except Exception:
            return fail(f"index.steps[{idx}].returncode must be int", CODES["STEP_RC_TYPE"])
        if bool(ok_value) != (rc == 0):
            return fail(
                f"index.steps[{idx}] ok/returncode mismatch ok={ok_value} returncode={rc}",
                CODES["STEP_OK_RC_MISMATCH"],
            )
        cmd_value = row.get("cmd")
        if not isinstance(cmd_value, list):
            return fail(f"index.steps[{idx}].cmd must be list", CODES["STEP_CMD_TYPE"])
        if not cmd_value:
            return fail(f"index.steps[{idx}].cmd must not be empty", CODES["STEP_CMD_EMPTY"])
        for part in cmd_value:
            if not isinstance(part, str) or not part.strip():
                return fail(
                    f"index.steps[{idx}].cmd[*] must be non-empty string",
                    CODES["STEP_CMD_ITEM_TYPE"],
                )
    expected_index_overall_ok = failed_step_count == 0
    if index_overall_ok != expected_index_overall_ok:
        return fail(
            f"index.overall_ok mismatch expected={expected_index_overall_ok} from steps failed_step_count={failed_step_count}",
            CODES["INDEX_OVERALL_OK_STEPS_MISMATCH"],
        )

    resolved_report_paths: dict[str, Path] = {}
    for key in REQUIRED_REPORT_PATH_KEYS:
        path = resolve_report_path(index_doc, key)
        if path is None:
            return fail(f"missing index reports key/path: {key}", CODES["REPORT_KEY_MISSING"])
        if not path.exists():
            return fail(f"missing report path for {key}: {path}", CODES["REPORT_PATH_MISSING"])
        resolved_report_paths[key] = path

    artifact_docs: dict[str, dict] = {}
    for key, expected_schema in ARTIFACT_SCHEMA_MAP.items():
        artifact_path = resolve_report_path(index_doc, key)
        if artifact_path is None:
            return fail(f"missing artifact path key: {key}", CODES["REPORT_KEY_MISSING"])
        artifact_doc = load_json(artifact_path)
        if not isinstance(artifact_doc, dict):
            return fail(
                f"artifact json invalid key={key} path={artifact_path}",
                CODES["ARTIFACT_JSON_INVALID"],
            )
        actual_schema = str(artifact_doc.get("schema", "")).strip()
        if isinstance(expected_schema, tuple):
            expected_schemas = expected_schema
        else:
            expected_schemas = (expected_schema,)
        if actual_schema not in expected_schemas:
            return fail(
                "artifact schema mismatch "
                f"key={key} schema={actual_schema} expected={','.join(expected_schemas)}",
                CODES["ARTIFACT_SCHEMA_MISMATCH"],
            )
        artifact_docs[key] = artifact_doc

    final_parse_doc = artifact_docs["final_status_parse_json"]
    final_parse_parsed = final_parse_doc.get("parsed")
    if not isinstance(final_parse_parsed, dict):
        return fail("final_status_parse parsed missing", CODES["FINAL_PARSE_PARSED_MISSING"])
    final_parse_status_line_path_raw = str(final_parse_doc.get("status_line_path", "")).strip()
    if not final_parse_status_line_path_raw:
        return fail("final_status_parse status_line_path missing", CODES["FINAL_PARSE_STATUS_LINE_PATH_MISSING"])
    final_parse_status_line_path = Path(final_parse_status_line_path_raw.replace("\\", "/"))
    if not final_parse_status_line_path.exists():
        return fail(
            f"final_status_parse status_line_path not found: {final_parse_status_line_path}",
            CODES["FINAL_PARSE_STATUS_LINE_PATH_NOT_FOUND"],
        )
    expected_final_parse_status = "pass" if index_overall_ok else "fail"
    final_parse_status = str(final_parse_parsed.get("status", "")).strip()
    if final_parse_status != expected_final_parse_status:
        return fail(
            f"final_status_parse status mismatch expected={expected_final_parse_status} actual={final_parse_status}",
            CODES["FINAL_PARSE_STATUS_MISMATCH"],
        )
    final_parse_overall_ok_raw = str(final_parse_parsed.get("overall_ok", "")).strip()
    if final_parse_overall_ok_raw not in {"0", "1"}:
        return fail(
            f"final_status_parse overall_ok invalid: {final_parse_overall_ok_raw}",
            CODES["FINAL_PARSE_OVERALL_OK_INVALID"],
        )
    final_parse_overall_ok = final_parse_overall_ok_raw == "1"
    if final_parse_overall_ok != index_overall_ok:
        return fail(
            f"final_status_parse overall_ok mismatch expected={int(index_overall_ok)} actual={int(final_parse_overall_ok)}",
            CODES["FINAL_PARSE_OVERALL_OK_MISMATCH"],
        )
    final_parse_aggregate_status = str(final_parse_parsed.get("aggregate_status", "")).strip()
    if final_parse_aggregate_status not in {"pass", "fail"}:
        return fail(
            f"final_status_parse aggregate_status invalid: {final_parse_aggregate_status}",
            CODES["FINAL_PARSE_AGGREGATE_STATUS_INVALID"],
        )
    final_parse_failed_steps_raw = str(final_parse_parsed.get("failed_steps", "")).strip()
    try:
        final_parse_failed_steps = int(final_parse_failed_steps_raw)
    except Exception:
        return fail(
            "final_status_parse failed_steps must be int string",
            CODES["FINAL_PARSE_FAILED_STEPS_TYPE"],
        )
    if final_parse_failed_steps != failed_step_count:
        return fail(
            f"final_status_parse failed_steps mismatch expected={failed_step_count} actual={final_parse_failed_steps}",
            CODES["FINAL_PARSE_FAILED_STEPS_MISMATCH"],
        )

    sanity_profile = str(artifact_docs["ci_sanity_gate"].get("profile", "")).strip()
    if sanity_profile not in VALID_SANITY_PROFILES:
        return fail(f"invalid sanity profile in ci_sanity_gate: {sanity_profile}", CODES["SANITY_PROFILE_INVALID"])
    if sanity_profile != index_profile:
        return fail(
            f"ci_sanity_gate profile mismatch index={index_profile} actual={sanity_profile}",
            CODES["SANITY_PROFILE_MISMATCH"],
        )

    sync_profile = str(artifact_docs["ci_sync_readiness"].get("sanity_profile", "")).strip()
    if sync_profile not in VALID_SANITY_PROFILES:
        return fail(
            f"invalid sanity_profile in ci_sync_readiness: {sync_profile}",
            CODES["SYNC_PROFILE_INVALID"],
        )
    if sync_profile != index_profile:
        return fail(
            f"ci_sync_readiness sanity_profile mismatch index={index_profile} actual={sync_profile}",
            CODES["SYNC_PROFILE_MISMATCH"],
        )

    result_doc = artifact_docs["ci_gate_result_json"]
    result_ok = result_doc.get("ok")
    if not isinstance(result_ok, bool):
        return fail("ci_gate_result ok must be bool", CODES["RESULT_OK_TYPE"])
    result_overall_ok = result_doc.get("overall_ok")
    if not isinstance(result_overall_ok, bool):
        return fail("ci_gate_result overall_ok must be bool", CODES["RESULT_OVERALL_OK_TYPE"])
    if result_overall_ok != index_overall_ok:
        return fail(
            f"ci_gate_result overall_ok mismatch index={index_overall_ok} actual={result_overall_ok}",
            CODES["RESULT_OVERALL_OK_MISMATCH"],
        )
    result_failed_steps_raw = result_doc.get("failed_steps")
    if not isinstance(result_failed_steps_raw, int) or isinstance(result_failed_steps_raw, bool):
        return fail("ci_gate_result failed_steps must be int", CODES["RESULT_FAILED_STEPS_TYPE"])
    result_failed_steps = int(result_failed_steps_raw)
    if result_failed_steps != failed_step_count:
        return fail(
            f"ci_gate_result failed_steps mismatch expected={failed_step_count} actual={result_failed_steps}",
            CODES["RESULT_FAILED_STEPS_MISMATCH"],
        )
    result_status = str(result_doc.get("status", "")).strip()
    expected_result_status = "pass" if index_overall_ok else "fail"
    if result_status != expected_result_status:
        return fail(
            f"ci_gate_result status mismatch expected={expected_result_status} actual={result_status}",
            CODES["RESULT_STATUS_MISMATCH"],
        )
    result_aggregate_status = str(result_doc.get("aggregate_status", "")).strip()
    if result_aggregate_status not in {"pass", "fail"}:
        return fail(
            f"ci_gate_result aggregate_status invalid: {result_aggregate_status}",
            CODES["RESULT_AGGREGATE_STATUS_INVALID"],
        )
    if result_aggregate_status != final_parse_aggregate_status:
        return fail(
            f"ci_gate_result aggregate_status mismatch expected={final_parse_aggregate_status} actual={result_aggregate_status}",
            CODES["RESULT_AGGREGATE_STATUS_MISMATCH"],
        )
    expected_result_ok = (
        result_status == "pass"
        and result_overall_ok
        and result_aggregate_status == "pass"
        and result_failed_steps == 0
    )
    if result_ok != expected_result_ok:
        return fail(
            f"ci_gate_result ok mismatch expected={int(expected_result_ok)} actual={int(result_ok)}",
            CODES["RESULT_OK_MISMATCH"],
        )
    result_summary_line_path = normalize_path_text(str(result_doc.get("summary_line_path", "")).strip())
    expected_summary_line_path = str(resolved_report_paths["summary_line"])
    if result_summary_line_path != expected_summary_line_path:
        return fail(
            f"ci_gate_result summary_line_path mismatch expected={expected_summary_line_path} actual={result_summary_line_path}",
            CODES["RESULT_SUMMARY_LINE_PATH_MISMATCH"],
        )
    expected_summary_line = read_text(resolved_report_paths["summary_line"])
    result_summary_line = str(result_doc.get("summary_line", "")).strip()
    if result_summary_line != expected_summary_line and not is_compatible_summary_line(
        result_summary_line,
        expected_summary_line,
    ):
        return fail(
            "ci_gate_result summary_line mismatch",
            CODES["RESULT_SUMMARY_LINE_MISMATCH"],
        )
    result_gate_index_path = normalize_path_text(str(result_doc.get("gate_index_path", "")).strip())
    expected_gate_index_path = str(index_path)
    if result_gate_index_path != expected_gate_index_path:
        return fail(
            f"ci_gate_result gate_index_path mismatch expected={expected_gate_index_path} actual={result_gate_index_path}",
            CODES["RESULT_GATE_INDEX_PATH_MISMATCH"],
        )
    result_final_status_parse_path = normalize_path_text(str(result_doc.get("final_status_parse_path", "")).strip())
    expected_final_status_parse_path = str(resolved_report_paths["final_status_parse_json"])
    if result_final_status_parse_path != expected_final_status_parse_path:
        return fail(
            "ci_gate_result final_status_parse_path mismatch",
            CODES["RESULT_FINAL_STATUS_PARSE_PATH_MISMATCH"],
        )

    result_reason = str(result_doc.get("reason", "")).strip() or "-"

    badge_doc = artifact_docs["ci_gate_badge_json"]
    badge_status = str(badge_doc.get("status", "")).strip()
    if badge_status != result_status:
        return fail(
            f"ci_gate_badge status mismatch expected={result_status} actual={badge_status}",
            CODES["BADGE_STATUS_MISMATCH"],
        )
    badge_ok = badge_doc.get("ok")
    if not isinstance(badge_ok, bool):
        return fail("ci_gate_badge ok must be bool", CODES["BADGE_OK_TYPE"])
    if bool(badge_ok) != bool(expected_result_ok):
        return fail(
            f"ci_gate_badge ok mismatch expected={int(bool(expected_result_ok))} actual={int(bool(badge_ok))}",
            CODES["BADGE_OK_MISMATCH"],
        )
    badge_result_path = normalize_path_text(str(badge_doc.get("result_path", "")).strip())
    expected_badge_result_path = str(resolved_report_paths["ci_gate_result_json"])
    if badge_result_path != expected_badge_result_path:
        return fail(
            f"ci_gate_badge result_path mismatch expected={expected_badge_result_path} actual={badge_result_path}",
            CODES["BADGE_RESULT_PATH_MISMATCH"],
        )

    triage_doc = artifact_docs["ci_fail_triage_json"]
    triage_status = str(triage_doc.get("status", "")).strip()
    if triage_status != result_status:
        return fail(
            f"ci_fail_triage status mismatch expected={result_status} actual={triage_status}",
            CODES["TRIAGE_STATUS_MISMATCH"],
        )
    triage_reason = str(triage_doc.get("reason", "")).strip() or "-"
    if triage_reason != result_reason:
        return fail(
            f"ci_fail_triage reason mismatch expected={result_reason} actual={triage_reason}",
            CODES["TRIAGE_REASON_MISMATCH"],
        )
    triage_summary_hint_norm = normalize_path_text(str(triage_doc.get("summary_report_path_hint_norm", "")).strip())
    expected_summary_hint_norm = normalize_path_text(str(resolved_report_paths["summary"]))
    if triage_summary_hint_norm != expected_summary_hint_norm:
        return fail(
            "ci_fail_triage summary_report_path_hint_norm mismatch",
            CODES["TRIAGE_SUMMARY_HINT_NORM_MISMATCH"],
        )
    triage_artifacts = triage_doc.get("artifacts")
    if not isinstance(triage_artifacts, dict):
        return fail("ci_fail_triage artifacts missing", CODES["TRIAGE_ARTIFACTS_MISSING"])
    triage_artifact_required_keys = (
        "summary",
        "ci_gate_result_json",
        "ci_gate_badge_json",
        "ci_fail_brief_txt",
        "ci_fail_triage_json",
    )
    for artifact_key in triage_artifact_required_keys:
        artifact_row = triage_artifacts.get(artifact_key)
        if not isinstance(artifact_row, dict):
            return fail(
                f"ci_fail_triage artifacts missing row key={artifact_key}",
                CODES["TRIAGE_ARTIFACTS_MISSING"],
            )
        artifact_path = normalize_path_text(str(artifact_row.get("path", "")).strip())
        expected_artifact_path = str(resolved_report_paths[artifact_key])
        if artifact_path != expected_artifact_path:
            return fail(
                f"ci_fail_triage artifacts path mismatch key={artifact_key}",
                CODES["TRIAGE_ARTIFACT_PATH_MISMATCH"],
            )
        artifact_path_norm = normalize_path_text(str(artifact_row.get("path_norm", "")).strip())
        expected_artifact_path_norm = str(resolved_report_paths[artifact_key])
        if artifact_path_norm != expected_artifact_path_norm:
            return fail(
                f"ci_fail_triage artifacts path_norm mismatch key={artifact_key}",
                CODES["TRIAGE_ARTIFACT_PATH_NORM_MISMATCH"],
            )
        artifact_exists = artifact_row.get("exists")
        if not isinstance(artifact_exists, bool) or not artifact_exists:
            return fail(
                f"ci_fail_triage artifacts exists mismatch key={artifact_key}",
                CODES["TRIAGE_ARTIFACT_EXISTS_MISMATCH"],
            )

    required_steps: list[str] = []
    if bool(args.enforce_profile_step_contract):
        required_steps.extend(resolve_profile_required_steps(str(args.sanity_profile).strip()))
    required_steps.extend([str(item).strip() for item in args.required_step if str(item).strip()])
    deduped_required_steps: list[str] = []
    seen_required_steps: set[str] = set()
    for step_name in required_steps:
        if not step_name or step_name in seen_required_steps:
            continue
        seen_required_steps.add(step_name)
        deduped_required_steps.append(step_name)
    missing_required_steps = [name for name in deduped_required_steps if name not in seen_step_names]
    if missing_required_steps:
        return fail(
            f"missing required index step(s): {','.join(missing_required_steps)}",
            CODES["REQUIRED_STEP_MISSING"],
        )

    print(f"[ci-gate-report-index-check] ok index={index_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
