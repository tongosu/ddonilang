#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from _ci_fail_and_exit_contract import validate_fail_and_exit_block_contract
from _ci_latest_smoke_contract import (
    LATEST_SMOKE_SKIP_REASON_CI_GATE_RESULT_STATUS_NOT_PASS,
    LATEST_SMOKE_SKIP_REASON_EXPECTED,
    LATEST_SMOKE_SKIP_REASON_FAST_FAIL_PATH,
    LATEST_SMOKE_SKIP_REASON_FLAG_DISABLED,
    LATEST_SMOKE_SKIP_REASON_PENDING_FAILURE_SUMMARY_REGENERATION,
)


SYNC_READINESS_SELFTEST_TOKENS = [
    "tests/run_ci_sync_readiness_check_selftest.py",
    "check_ci_sync_readiness_selftest",
    "ci_sync_readiness_selftest",
    "check_ci_sync_readiness_diagnostics",
    "ci_sync_readiness_diagnostics_check",
    "tests/run_ci_sync_readiness_diagnostics_check.py",
]

SYNC_READINESS_REPORT_TOKENS = [
    "check_ci_sync_readiness_report_selftest",
    "ci_sync_readiness_report_selftest",
    "tests/run_ci_sync_readiness_report_check_selftest.py",
    "run_ci_sync_readiness_report_generate",
    "ci_sync_readiness_report_generate",
    "check_ci_sync_readiness_report_check",
    "ci_sync_readiness_report_check",
    "tests/run_ci_sync_readiness_report_check.py",
    "append_ci_sync_readiness_summary_lines(",
    "[ci-gate-summary] ci_sync_readiness_report=",
    "[ci-gate-summary] ci_sync_readiness_status=",
    "[ci-gate-summary] ci_sync_readiness_code=",
]

REPORT_INDEX_AND_PROFILE_TOKENS = [
    "check_ci_gate_report_index_selftest",
    "ci_gate_report_index_selftest",
    "tests/run_ci_gate_report_index_check_selftest.py",
    "check_ci_gate_report_index",
    "ci_gate_report_index_check",
    "tests/run_ci_gate_report_index_check.py",
    "check_ci_gate_report_index_diagnostics",
    "ci_gate_report_index_diagnostics_check",
    "tests/run_ci_gate_report_index_diagnostics_check.py",
    "check_ci_fail_and_exit_contract_selftest",
    "check_ci_fail_and_exit_contract_selftest_skipped",
    "ci_fail_and_exit_contract_selftest",
    "tests/run_ci_fail_and_exit_contract_selftest.py",
    "--skip-fail-and-exit-contract-selftest",
    "check_ci_gate_report_index_latest_smoke",
    "ci_gate_report_index_latest_smoke_check",
    "tests/run_ci_gate_report_index_latest_smoke_check.py",
    "check_ci_emit_artifacts_required_post_summary",
    "ci_emit_artifacts_required_post_summary_check",
    "allow-triage-exists-upgrade",
    "report_index_required_steps_common",
    "report_index_required_steps_seamgrim",
    "resolve_report_index_required_steps",
    "report_index_required_steps",
    "require_step_contract",
    "--enforce-profile-step-contract",
    "--required-step",
    "check_ci_gate_report_index(require_step_contract=True)",
    "def find_step_record(name: str) -> dict[str, object] | None:",
    "def run_step_if_missing(name: str, runner) -> int:",
    'run_step_if_missing("ci_fail_and_exit_contract_selftest", check_ci_fail_and_exit_contract_selftest)',
    'run_step_if_missing("ci_gate_report_index_selftest", check_ci_gate_report_index_selftest)',
    'run_step_if_missing("ci_gate_report_index_diagnostics_check", check_ci_gate_report_index_diagnostics)',
    'run_step_if_missing("ci_emit_artifacts_baseline_check", check_ci_emit_artifacts_baseline)',
    "check_ci_profile_matrix_gate_selftest",
    "ci_profile_matrix_gate_selftest",
    "tests/run_ci_profile_matrix_gate_selftest.py",
    "--index",
]

LATEST_SMOKE_SKIP_REASON_EXPECTED_TOKENS = [
    "LATEST_SMOKE_SKIP_REASON_EXPECTED",
    "LATEST_SMOKE_SKIP_REASON_FAST_FAIL_PATH",
    "LATEST_SMOKE_SKIP_REASON_FLAG_DISABLED",
    "LATEST_SMOKE_SKIP_REASON_PENDING_FAILURE_SUMMARY_REGENERATION",
    "LATEST_SMOKE_SKIP_REASON_CI_GATE_RESULT_STATUS_NOT_PASS",
]

LATEST_SMOKE_REASON_BRANCH_TOKENS = [
    "from _ci_latest_smoke_contract import (",
    "def check_ci_gate_report_index_latest_smoke_skipped(reason: str) -> int:",
    "def should_run_ci_gate_report_index_latest_smoke() -> bool:",
    "return LATEST_SMOKE_SKIP_REASON_FLAG_DISABLED",
    "return LATEST_SMOKE_SKIP_REASON_PENDING_FAILURE_SUMMARY_REGENERATION",
    "return LATEST_SMOKE_SKIP_REASON_CI_GATE_RESULT_STATUS_NOT_PASS",
    "def resolve_ci_gate_report_index_latest_smoke_skip_reason(",
    "def run_ci_gate_report_index_latest_smoke_step(",
    "check_ci_gate_report_index_latest_smoke_skipped(LATEST_SMOKE_SKIP_REASON_FAST_FAIL_PATH)",
    "ci_gate_report_index_latest_smoke_rc = run_ci_gate_report_index_latest_smoke_step(",
    "if not bool(args.run_report_index_latest_smoke):",
    "elif has_failed_steps or ci_gate_report_index_rc != 0:",
    "elif should_run_ci_gate_report_index_latest_smoke():",
    *LATEST_SMOKE_SKIP_REASON_EXPECTED_TOKENS,
]

SEAMGRIM_SYNC_STEP_TOKENS = [
    "--require-preview-synced",
    "check_seamgrim_ci_gate_preview_sync_passthrough",
    "seamgrim_ci_gate_preview_sync_passthrough_check",
    "tests/run_seamgrim_ci_gate_preview_sync_passthrough_check.py",
    "check_seamgrim_ci_gate_runtime5_passthrough",
    "seamgrim_ci_gate_runtime5_passthrough_check",
    "tests/run_seamgrim_ci_gate_runtime5_passthrough_check.py",
    "check_seamgrim_ci_gate_seed_meta_step",
    "seamgrim_ci_gate_seed_meta_step_check",
    "tests/run_seamgrim_ci_gate_seed_meta_step_check.py",
    "check_seamgrim_ci_gate_sam_seulgi_family_step",
    "seamgrim_ci_gate_sam_seulgi_family_step_check",
    "tests/run_seamgrim_ci_gate_sam_seulgi_family_step_check.py",
    "check_seamgrim_ci_gate_guideblock_step",
    "seamgrim_ci_gate_guideblock_step_check",
    "tests/run_seamgrim_ci_gate_guideblock_step_check.py",
    "check_seamgrim_ci_gate_lesson_warning_step",
    "seamgrim_ci_gate_lesson_warning_step_check",
    "tests/run_seamgrim_ci_gate_lesson_warning_step_check.py",
    "check_seamgrim_ci_gate_stateful_preview_step",
    "seamgrim_ci_gate_stateful_preview_step_check",
    "tests/run_seamgrim_ci_gate_stateful_preview_step_check.py",
    "check_seamgrim_ci_gate_wasm_web_smoke_step",
    "seamgrim_ci_gate_wasm_web_smoke_step_check",
    "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check.py",
    "check_seamgrim_ci_gate_wasm_web_smoke_step_selftest",
    "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
    "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check_selftest.py",
    "--lesson-warning-report-json-out",
    "--lesson-warning-require-zero",
]

GUIDEBLOCK_AND_SANITY_TOKENS = [
    "check_ci_pack_golden_guideblock_selftest",
    "ci_pack_golden_guideblock_selftest",
    "tests/run_pack_golden_guideblock_selftest.py",
    "[ci-gate-summary] ci_pack_golden_guideblock_selftest_ok=",
    "check_ci_sanity_gate",
    "ci_sanity_gate",
    "tests/run_ci_sanity_gate.py",
    "--ci-sanity-profile",
    "--profile",
    "--sanity-profile",
    "ci_sanity_gate_profile",
    "ci_sync_readiness_sanity_profile",
]

RUNNER_TOKENS = [
    "check_ci_aggregate_gate_sync_diagnostics",
    "ci_aggregate_gate_sync_diagnostics_check",
    "tests/run_ci_aggregate_gate_sync_diagnostics_check.py",
    "ci_aggregate_gate_sync_diagnostics_rc",
]

AGE3_CRITERIA_SYNC_TOKENS = [
    "from _ci_age3_completion_gate_contract import (",
    "AGE3_COMPLETION_GATE_CRITERIA_NAMES,",
    "age3_completion_gate_criteria_summary_key,",
    "age3_completion_gate_criteria_sync_summary_key,",
    "SYNC_SUMMARY_TOKEN_CONTRACT.extend(",
]

REQUIRED_TOKENS = [
    *SYNC_READINESS_SELFTEST_TOKENS,
    *SYNC_READINESS_REPORT_TOKENS,
    *REPORT_INDEX_AND_PROFILE_TOKENS,
    *LATEST_SMOKE_REASON_BRANCH_TOKENS,
    *SEAMGRIM_SYNC_STEP_TOKENS,
    *GUIDEBLOCK_AND_SANITY_TOKENS,
    *RUNNER_TOKENS,
    *AGE3_CRITERIA_SYNC_TOKENS,
]


def main() -> int:
    expected_reason_set = {
        LATEST_SMOKE_SKIP_REASON_FAST_FAIL_PATH,
        LATEST_SMOKE_SKIP_REASON_FLAG_DISABLED,
        LATEST_SMOKE_SKIP_REASON_PENDING_FAILURE_SUMMARY_REGENERATION,
        LATEST_SMOKE_SKIP_REASON_CI_GATE_RESULT_STATUS_NOT_PASS,
    }
    if set(LATEST_SMOKE_SKIP_REASON_EXPECTED) != expected_reason_set:
        print("aggregate gate sync diagnostics check failed: latest-smoke reason contract mismatch")
        return 1

    root = Path(__file__).resolve().parent.parent
    target = root / "tests" / "run_ci_aggregate_gate.py"
    if not target.exists():
        print(f"missing target: {target}")
        return 1
    text = target.read_text(encoding="utf-8")

    missing = [token for token in REQUIRED_TOKENS if token not in text]
    if missing:
        print("aggregate gate sync diagnostics check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1
    fail_and_exit_contract_issues = validate_fail_and_exit_block_contract(text)
    if fail_and_exit_contract_issues:
        print("aggregate gate sync diagnostics check failed (fail_and_exit contract):")
        for issue in fail_and_exit_contract_issues[:12]:
            print(f" - {issue}")
        return 1

    print("ci aggregate gate sync diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
