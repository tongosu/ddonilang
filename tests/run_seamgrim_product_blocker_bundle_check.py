#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from _ci_seamgrim_step_contract import (
    SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_FAILED_STEP_CASES,
    SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_MISSING_STEP_CASES,
    SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_BY_STEP,
    collect_blocker_contract_issues,
    SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_FIELDS,
    SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS,
    SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES,
    SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES,
    SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME,
    SEAMGRIM_BLOCKER_STEP_CONTRACT_STEPS,
    SEAMGRIM_BLOCKER_STEP_SCRIPT_PATHS,
)


REQUIRED_TOKENS: dict[str, tuple[str, ...]] = {
    "tests/_ci_seamgrim_step_contract.py": (
        "SEAMGRIM_BLOCKER_STEP_CONTRACT_STEPS",
        "SEAMGRIM_BLOCKER_STEP_SCRIPT_PATHS",
        "SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS",
        "SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_FIELDS",
        "SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME",
        "SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_BY_STEP",
        "SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES",
        "SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES",
        "SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_MISSING_STEP_CASES",
        "SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_FAILED_STEP_CASES",
        *SEAMGRIM_BLOCKER_STEP_CONTRACT_STEPS,
        *(script_path for _step_name, script_path in SEAMGRIM_BLOCKER_STEP_SCRIPT_PATHS),
        *(summary_key for summary_key, _step_name in SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS),
        *(fail_code for _step_name, fail_code in SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_FIELDS),
        *(step_name for step_name in SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME),
        *(step_name for step_name in SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_BY_STEP),
        *(
            case_slug
            for _step_name, case_slug, _fail_label, _msg_label in SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES
        ),
        *(
            case_slug
            for _step_name, case_slug, _fail_label, _msg_label in SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES
        ),
        *(
            fail_label
            for _step_name, _case_slug, fail_label, _msg_label in SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES
        ),
        *(
            fail_label
            for _step_name, _case_slug, fail_label, _msg_label in SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES
        ),
        *(
            msg_label
            for _step_name, _case_slug, _fail_label, msg_label in SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES
        ),
        *(
            msg_label
            for _step_name, _case_slug, _fail_label, msg_label in SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES
        ),
        *(
            fail_message
            for _step_name, fail_message, _code_message in SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_MISSING_STEP_CASES
        ),
        *(
            code_message
            for _step_name, _fail_message, code_message in SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_MISSING_STEP_CASES
        ),
        *(
            fail_message
            for _step_name, fail_message, _code_message in SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_FAILED_STEP_CASES
        ),
        *(
            code_message
            for _step_name, _fail_message, code_message in SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_FAILED_STEP_CASES
        ),
    ),
    "tests/run_seamgrim_ci_gate.py": (
        "from _ci_seamgrim_step_contract import SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME",
        "SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME[\"seamgrim_runtime_view_source_strict_check\"]",
        "SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME[\"seamgrim_view_only_state_hash_invariant_check\"]",
        "SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME[\"seamgrim_run_legacy_autofix_check\"]",
        "SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME[\"seamgrim_observe_output_contract_check\"]",
        "tests/run_seamgrim_ddn_exec_server_gate_check.py",
        "tests/run_seamgrim_lesson_migration_lint_check.py",
        "tests/run_seamgrim_lesson_migration_autofix_check.py",
        "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check.py",
    ),
    "tests/run_ci_sanity_gate.py": (
        "SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS",
        "SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_BY_STEP",
        "SEAMGRIM_BLOCKER_STEP_SCRIPT_PATHS",
        "SEAMGRIM_BLOCKER_FAIL_CODES",
        "SEAMGRIM_BLOCKER_FAIL_CODES: dict[str, str] = SEAMGRIM_BLOCKER_SANITY_FAIL_CODE_BY_STEP",
        "seamgrim_ci_gate_lesson_warning_step_check",
        "seamgrim_ci_gate_stateful_preview_step_check",
        "seamgrim_ci_gate_wasm_web_smoke_step_check",
        "seamgrim_interface_boundary_contract_check",
    ),
    "tests/run_ci_sync_readiness_check.py": (
        "SEAMGRIM_BLOCKER_STEP_CONTRACT_STEPS",
        "*SEAMGRIM_BLOCKER_STEP_CONTRACT_STEPS,",
        "SEAMGRIM_BLOCKER_SANITY_SUMMARY_STEP_FIELDS",
        "SANITY_REQUIRED_PASS_STEPS = merge_step_names(",
        "SANITY_REQUIRED_PASS_STEPS_SEAMGRIM = merge_step_names(",
    ),
    "tests/run_ci_sync_readiness_check_selftest.py": (
        "SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES",
        "SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES",
        ") in SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES:",
        ") in SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES:",
    ),
    "tests/run_ci_sync_readiness_diagnostics_check.py": (
        "SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES",
        "SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES",
        ") in SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_FAILED_CASES:",
    ),
    "tests/run_ci_gate_report_index_check_selftest.py": (
        "SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_MISSING_STEP_CASES",
        "for missing_step_name, fail_message, code_message in SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_MISSING_STEP_CASES:",
        "SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_FAILED_STEP_CASES",
        "for failed_step_name, fail_message, code_message in SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_FAILED_STEP_CASES:",
    ),
    "tests/run_ci_gate_report_index_diagnostics_check.py": (
        "SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_MISSING_STEP_CASES",
        "SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_FAILED_STEP_CASES",
        "for failed_step_name, fail_message, code_message in SEAMGRIM_BLOCKER_GATE_REPORT_INDEX_FAILED_STEP_CASES:",
    ),
    "tests/run_ci_aggregate_gate.py": (
        "SEAMGRIM_BLOCKER_STEP_SCRIPT_PATHS",
        "SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME",
        "def run_seamgrim_blocker_step(step_name: str) -> int:",
        "script_path = SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME[step_name]",
        "return run_seamgrim_blocker_step(",
        *(step_name for step_name, _script_path in SEAMGRIM_BLOCKER_STEP_SCRIPT_PATHS),
    ),
    "tests/run_ci_aggregate_gate_sync_diagnostics_check.py": (
        "SEAMGRIM_BLOCKER_STEP_SCRIPT_PATHS",
        "SEAMGRIM_BLOCKER_STEP_SCRIPT_PATH_BY_NAME",
        "def run_seamgrim_blocker_step(step_name: str) -> int:",
        "return run_seamgrim_blocker_step(",
    ),
}


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    failures: list[str] = []

    contract_issues = collect_blocker_contract_issues()
    if contract_issues:
        failures.extend([f"contract issue: {row}" for row in contract_issues])

    for rel_path, tokens in REQUIRED_TOKENS.items():
        target = root / rel_path
        if not target.exists():
            failures.append(f"missing target: {rel_path}")
            continue
        text = target.read_text(encoding="utf-8")
        for token in tokens:
            if token not in text:
                failures.append(f"missing token: {rel_path}::{token}")

    if failures:
        print("seamgrim product blocker bundle check failed:")
        for row in failures[:24]:
            print(f" - {row}")
        return 1

    print("seamgrim product blocker bundle check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
