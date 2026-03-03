#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


REQUIRED_TOKEN_MAP = {
    "tests/ci_check_error_codes.py": [
        "GATE_REPORT_INDEX_CODES",
        '"INDEX_MISSING": "E_GATE_INDEX_MISSING"',
        '"INDEX_JSON_INVALID": "E_GATE_INDEX_JSON_INVALID"',
        '"INDEX_SCHEMA": "E_GATE_INDEX_SCHEMA"',
        '"INDEX_REPORTS_MISSING": "E_GATE_INDEX_REPORTS_MISSING"',
        '"PROFILE_INVALID": "E_GATE_INDEX_PROFILE_INVALID"',
        '"PROFILE_MISMATCH": "E_GATE_INDEX_PROFILE_MISMATCH"',
        '"REPORT_KEY_MISSING": "E_GATE_INDEX_REPORT_KEY_MISSING"',
        '"REPORT_PATH_MISSING": "E_GATE_INDEX_REPORT_PATH_MISSING"',
        '"ARTIFACT_JSON_INVALID": "E_GATE_INDEX_ARTIFACT_JSON_INVALID"',
        '"ARTIFACT_SCHEMA_MISMATCH": "E_GATE_INDEX_ARTIFACT_SCHEMA_MISMATCH"',
        '"SANITY_PROFILE_INVALID": "E_GATE_INDEX_SANITY_PROFILE_INVALID"',
        '"SANITY_PROFILE_MISMATCH": "E_GATE_INDEX_SANITY_PROFILE_MISMATCH"',
        '"SYNC_PROFILE_INVALID": "E_GATE_INDEX_SYNC_PROFILE_INVALID"',
        '"SYNC_PROFILE_MISMATCH": "E_GATE_INDEX_SYNC_PROFILE_MISMATCH"',
        '"STEPS_MISSING": "E_GATE_INDEX_STEPS_MISSING"',
        '"STEPS_TYPE": "E_GATE_INDEX_STEPS_TYPE"',
        '"STEP_ROW_TYPE": "E_GATE_INDEX_STEP_ROW_TYPE"',
        '"STEP_NAME": "E_GATE_INDEX_STEP_NAME"',
        '"STEP_DUP": "E_GATE_INDEX_STEP_DUP"',
        '"STEP_OK_TYPE": "E_GATE_INDEX_STEP_OK_TYPE"',
        '"STEP_OK_RC_MISMATCH": "E_GATE_INDEX_STEP_OK_RC_MISMATCH"',
        '"STEP_RC_TYPE": "E_GATE_INDEX_STEP_RC_TYPE"',
        '"STEP_CMD_TYPE": "E_GATE_INDEX_STEP_CMD_TYPE"',
        '"STEP_CMD_EMPTY": "E_GATE_INDEX_STEP_CMD_EMPTY"',
        '"STEP_CMD_ITEM_TYPE": "E_GATE_INDEX_STEP_CMD_ITEM_TYPE"',
        '"REQUIRED_STEP_MISSING": "E_GATE_INDEX_REQUIRED_STEP_MISSING"',
    ],
    "tests/run_ci_gate_report_index_check.py": [
        "from ci_check_error_codes import GATE_REPORT_INDEX_CODES as CODES",
        "INDEX_SCHEMA = \"ddn.ci.aggregate_gate.index.v1\"",
        "VALID_SANITY_PROFILES",
        "PROFILE_REQUIRED_STEPS_COMMON",
        "PROFILE_REQUIRED_STEPS_SEAMGRIM",
        "resolve_profile_required_steps",
        "--sanity-profile",
        "--enforce-profile-step-contract",
        "--required-step",
        "REQUIRED_REPORT_PATH_KEYS",
        "\"seamgrim_wasm_cli_diag_parity\"",
        "ARTIFACT_SCHEMA_MAP",
        "\"ddn.ci.gate_result.v1\"",
        "\"ddn.ci.sanity_gate.v1\"",
        "\"ddn.ci.sync_readiness.v1\"",
        "\"ddn.seamgrim.wasm_cli_diag_parity.v1\"",
        "index.steps is missing",
        "index.steps must be list",
        "index.steps[",
        "duplicate name",
        ".ok must be bool",
        "ok/returncode mismatch",
        ".returncode must be int",
        ".cmd must be list",
        ".cmd must not be empty",
        ".cmd[*] must be non-empty string",
        "invalid ci_sanity_profile",
        "ci_sanity_profile mismatch",
        "missing required index step(s)",
        "missing index reports key/path",
        "missing report path for",
        "artifact schema mismatch",
        "invalid sanity profile in ci_sanity_gate",
        "ci_sanity_gate profile mismatch",
        "invalid sanity_profile in ci_sync_readiness",
        "ci_sync_readiness sanity_profile mismatch",
    ],
    "tests/run_ci_gate_report_index_check_selftest.py": [
        "run_ci_gate_report_index_check.py",
        "missing key case must fail",
        "missing path case must fail",
        "bad schema case must fail",
        "missing required step case must fail",
        "bad step shape case must fail",
        "bad profile case must fail",
        "profile mismatch case must fail",
        "cmd empty case must fail",
        "cmd item type case must fail",
        "ok/rc mismatch case must fail",
        "sanity profile mismatch case must fail",
        "sync profile mismatch case must fail",
        "PROFILE_INVALID",
        "PROFILE_MISMATCH",
        "SANITY_PROFILE_MISMATCH",
        "SYNC_PROFILE_MISMATCH",
        "REPORT_KEY_MISSING",
        "REPORT_PATH_MISSING",
        "ARTIFACT_SCHEMA_MISMATCH",
        "REQUIRED_STEP_MISSING",
        "STEP_ROW_TYPE",
        "STEP_OK_RC_MISMATCH",
        "STEP_CMD_EMPTY",
        "STEP_CMD_ITEM_TYPE",
        "REQUIRED_STEPS",
        "sanity_profile",
        "enforce_profile_step_contract",
        "core_lang profile should allow missing seamgrim steps",
        "seamgrim profile missing parity step case must fail",
        "--required-step",
    ],
    "tests/run_ci_aggregate_gate.py": [
        "check_ci_gate_report_index",
        "check_ci_gate_report_index_selftest",
        "check_ci_gate_report_index_diagnostics",
        "report_index_required_steps_common",
        "report_index_required_steps_seamgrim",
        "resolve_report_index_required_steps",
        "report_index_required_steps",
        "require_step_contract",
        "--sanity-profile",
        "--enforce-profile-step-contract",
        "--required-step",
        "check_ci_gate_report_index(require_step_contract=True)",
        "ci_gate_report_index_check",
        "ci_gate_report_index_selftest",
        "ci_gate_report_index_diagnostics_check",
        "tests/run_ci_gate_report_index_check.py",
        "tests/run_ci_gate_report_index_check_selftest.py",
        "tests/run_ci_gate_report_index_diagnostics_check.py",
    ],
}


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    missing: list[str] = []
    for rel_path, tokens in REQUIRED_TOKEN_MAP.items():
        target = root / rel_path
        if not target.exists():
            print(f"missing target: {target}")
            return 1
        text = target.read_text(encoding="utf-8")
        for token in tokens:
            if token not in text:
                missing.append(f"{rel_path}::{token}")

    if missing:
        print("ci gate report-index diagnostics check failed:")
        for token in missing[:16]:
            print(f" - missing token: {token}")
        return 1

    print("ci gate report-index diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
