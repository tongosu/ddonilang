from __future__ import annotations

FAST_FAIL_REENTRY_GUARD_TOKENS = (
    "def find_step_record(name: str) -> dict[str, object] | None:",
    "def run_step_if_missing(name: str, runner) -> int:",
    'run_step_if_missing("ci_fail_and_exit_contract_selftest", check_ci_fail_and_exit_contract_selftest)',
    'run_step_if_missing("ci_gate_report_index_selftest", check_ci_gate_report_index_selftest)',
    'run_step_if_missing("ci_gate_report_index_diagnostics_check", check_ci_gate_report_index_diagnostics)',
    'run_step_if_missing("ci_emit_artifacts_baseline_check", check_ci_emit_artifacts_baseline)',
)

FAIL_AND_EXIT_BLOCK_REQUIRED_TOKENS = (
    'run_step_if_missing("ci_fail_and_exit_contract_selftest", check_ci_fail_and_exit_contract_selftest)',
    'run_step_if_missing("ci_gate_report_index_selftest", check_ci_gate_report_index_selftest)',
    'run_step_if_missing("ci_gate_report_index_diagnostics_check", check_ci_gate_report_index_diagnostics)',
    'run_step_if_missing("ci_emit_artifacts_baseline_check", check_ci_emit_artifacts_baseline)',
    "refresh_status_outputs_for_index(strict_summary_verify=False)",
)

FAIL_AND_EXIT_BLOCK_FORBIDDEN_TOKENS = (
    "check_ci_gate_report_index_selftest()",
    "check_ci_gate_report_index_diagnostics()",
    "check_ci_emit_artifacts_baseline()",
    "check_ci_gate_report_index(require_step_contract=False)",
)


def validate_fail_and_exit_block_contract(gate_text: str) -> list[str]:
    issues: list[str] = []
    block_start = gate_text.find("def fail_and_exit(")
    block_end = gate_text.find("if args.contract_only_aggregate:")
    if block_start < 0 or block_end <= block_start:
        issues.append("fail_and_exit block boundary not found")
        return issues
    fail_and_exit_block = gate_text[block_start:block_end]
    for token in FAIL_AND_EXIT_BLOCK_REQUIRED_TOKENS:
        if token not in fail_and_exit_block:
            issues.append(f"missing required token in fail_and_exit: {token}")
    for token in FAIL_AND_EXIT_BLOCK_FORBIDDEN_TOKENS:
        if token in fail_and_exit_block:
            issues.append(f"forbidden direct rerun token in fail_and_exit: {token}")
    return issues
