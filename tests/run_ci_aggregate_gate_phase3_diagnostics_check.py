#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from _ci_fail_and_exit_contract import (
    FAST_FAIL_REENTRY_GUARD_TOKENS,
    validate_fail_and_exit_block_contract,
)


RUNNER_TOKENS = (
    "check_ci_aggregate_gate_phase3_diagnostics",
    "ci_aggregate_gate_phase3_diagnostics_check",
    "tests/run_ci_aggregate_gate_phase3_diagnostics_check.py",
    "ci_aggregate_gate_phase3_diagnostics_rc",
    "tests/run_seamgrim_ci_gate.py",
)

PHASE3_REPORT_TOKENS = (
    "seamgrim_phase3_cleanup_base_name",
    "seamgrim_phase3_cleanup_report",
    "--phase3-cleanup-json-out",
    "str(seamgrim_phase3_cleanup_report)",
    "seamgrim_phase3_cleanup",
    "[ci-gate-summary] seamgrim_phase3_cleanup=",
    "seamgrim_phase3_cleanup_gate_report.detjson",
)

REQUIRED_TOKENS = [
    *RUNNER_TOKENS,
    *PHASE3_REPORT_TOKENS,
    *FAST_FAIL_REENTRY_GUARD_TOKENS,
]


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    target = root / "tests" / "run_ci_aggregate_gate.py"
    if not target.exists():
        print(f"missing target: {target}")
        return 1
    text = target.read_text(encoding="utf-8")

    missing = [token for token in REQUIRED_TOKENS if token not in text]
    if missing:
        print("aggregate gate phase3 diagnostics check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1
    fail_and_exit_contract_issues = validate_fail_and_exit_block_contract(text)
    if fail_and_exit_contract_issues:
        print("aggregate gate phase3 diagnostics check failed (fail_and_exit contract):")
        for issue in fail_and_exit_contract_issues[:12]:
            print(f" - {issue}")
        return 1

    print("ci aggregate gate phase3 diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
