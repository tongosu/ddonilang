#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from _ci_fail_and_exit_contract import (
    FAST_FAIL_REENTRY_GUARD_TOKENS,
    validate_fail_and_exit_block_contract,
)


AGE4_CLOSE_TOKENS = (
    "age4_close",
    "tests/run_age4_close.py",
    "tools/scripts/print_age4_close_digest.py",
    "--require-age4",
    "--age4-report",
    "age4_close_report.detjson",
    "age4_close_pack_report.detjson",
)

PHASE3_BRIDGE_TOKENS = (
    "--phase3-cleanup-json-out",
    "seamgrim_phase3_cleanup",
    "seamgrim_phase3_cleanup_gate_report.detjson",
)

RUNNER_TOKENS = (
    "check_ci_aggregate_gate_age4_diagnostics",
    "ci_aggregate_gate_age4_diagnostics_check",
    "tests/run_ci_aggregate_gate_age4_diagnostics_check.py",
    "ci_aggregate_gate_age4_diagnostics_rc",
    "[ci-gate-summary] age4_status=",
)

REQUIRED_TOKENS = [
    *AGE4_CLOSE_TOKENS,
    *PHASE3_BRIDGE_TOKENS,
    *RUNNER_TOKENS,
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
        print("aggregate gate age4 diagnostics check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1
    fail_and_exit_contract_issues = validate_fail_and_exit_block_contract(text)
    if fail_and_exit_contract_issues:
        print("aggregate gate age4 diagnostics check failed (fail_and_exit contract):")
        for issue in fail_and_exit_contract_issues[:12]:
            print(f" - {issue}")
        return 1

    print("ci aggregate gate age4 diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
