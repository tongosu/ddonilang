#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from ci_check_error_codes import GATE_REPORT_INDEX_CODES as CODES

INDEX_SCHEMA = "ddn.ci.aggregate_gate.index.v1"

REQUIRED_REPORT_PATH_KEYS = (
    "summary",
    "summary_line",
    "ci_gate_result_json",
    "ci_gate_badge_json",
    "ci_fail_brief_txt",
    "ci_fail_triage_json",
    "ci_sanity_gate",
    "ci_sync_readiness",
    "seamgrim_wasm_cli_diag_parity",
)

ARTIFACT_SCHEMA_MAP = {
    "ci_gate_result_json": "ddn.ci.gate_result.v1",
    "ci_sanity_gate": "ddn.ci.sanity_gate.v1",
    "ci_sync_readiness": "ddn.ci.sync_readiness.v1",
    "seamgrim_wasm_cli_diag_parity": "ddn.seamgrim.wasm_cli_diag_parity.v1",
}

VALID_SANITY_PROFILES = ("full", "core_lang", "seamgrim")
PROFILE_REQUIRED_STEPS_COMMON = (
    "ci_profile_split_contract_check",
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

    for key in REQUIRED_REPORT_PATH_KEYS:
        path = resolve_report_path(index_doc, key)
        if path is None:
            return fail(f"missing index reports key/path: {key}", CODES["REPORT_KEY_MISSING"])
        if not path.exists():
            return fail(f"missing report path for {key}: {path}", CODES["REPORT_PATH_MISSING"])

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
        if actual_schema != expected_schema:
            return fail(
                f"artifact schema mismatch key={key} schema={actual_schema} expected={expected_schema}",
                CODES["ARTIFACT_SCHEMA_MISMATCH"],
            )
        artifact_docs[key] = artifact_doc

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
