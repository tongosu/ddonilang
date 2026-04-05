#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from _ci_fail_and_exit_contract import (
    FAST_FAIL_REENTRY_GUARD_TOKENS,
    validate_fail_and_exit_block_contract,
)


RUNNER_TOKENS = (
    "check_ci_aggregate_gate_runtime5_diagnostics",
    "ci_aggregate_gate_runtime5_diagnostics_check",
    "tests/run_ci_aggregate_gate_runtime5_diagnostics_check.py",
    "ci_aggregate_gate_runtime5_diagnostics_rc",
    "seamgrim_5min_checklist_base_name",
    "seamgrim_5min_checklist_report",
)

CHECKLIST_OPTION_TOKENS = (
    "--skip-5min-checklist",
    "include_5min_checklist = not bool(args.skip_5min_checklist)",
    "--runtime-5min-skip-ui-common",
    "if args.runtime_5min_skip_ui_common:",
    'seamgrim_cmd.append("--runtime-5min-skip-ui-common")',
    "--runtime-5min-skip-showcase-check",
    "if args.runtime_5min_skip_showcase_check:",
    'seamgrim_cmd.append("--runtime-5min-skip-showcase-check")',
    "--runtime-5min-showcase-smoke",
    "if args.runtime_5min_showcase_smoke:",
    '"--runtime-5min-showcase-smoke",',
    "--runtime-5min-showcase-smoke-madi-pendulum",
    "--runtime-5min-showcase-smoke-madi-tetris",
    "--checklist-json-out",
)

CHECKLIST_STEP_TOKENS = (
    "append_runtime_5min_checklist_summary_lines(",
    "check_seamgrim_5min_checklist_selftest",
    "seamgrim_5min_checklist_selftest",
    "tests/run_seamgrim_5min_checklist_selftest.py",
    "seamgrim_5min_checklist_report.detjson",
)

RUNTIME5_SUMMARY_TOKENS = (
    "[ci-gate-summary] seamgrim_5min_checklist=",
    "[ci-gate-summary] seamgrim_runtime_5min_rewrite_motion_projectile=",
    "[ci-gate-summary] seamgrim_runtime_5min_moyang_view_boundary=",
    "[ci-gate-summary] seamgrim_runtime_5min_pendulum_tetris_showcase=",
)

RUNTIME5_DIAG_LIB_TOKENS = (
    "def load_runtime_5min_checklist_snapshot(report_path: Path) -> dict[str, str]:",
    "resolved_snapshot = {",
    "**RUNTIME5_CHECKLIST_DEFAULT_SNAPSHOT,",
    '"ok": ok_text,',
    "RUNTIME5_CHECKLIST_ROW_SPECS",
)

REQUIRED_TOKENS = [
    *RUNNER_TOKENS,
    *CHECKLIST_OPTION_TOKENS,
    *CHECKLIST_STEP_TOKENS,
    *RUNTIME5_SUMMARY_TOKENS,
    *FAST_FAIL_REENTRY_GUARD_TOKENS,
]


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    gate_target = root / "tests" / "run_ci_aggregate_gate.py"
    lib_target = root / "tests" / "_ci_aggregate_diag_lib.py"
    if not gate_target.exists():
        print(f"missing target: {gate_target}")
        return 1
    if not lib_target.exists():
        print(f"missing target: {lib_target}")
        return 1
    gate_text = gate_target.read_text(encoding="utf-8")
    lib_text = lib_target.read_text(encoding="utf-8")

    missing = [token for token in REQUIRED_TOKENS if token not in gate_text]
    missing.extend(token for token in RUNTIME5_DIAG_LIB_TOKENS if token not in lib_text)
    if missing:
        print("aggregate gate runtime5 diagnostics check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1
    fail_and_exit_contract_issues = validate_fail_and_exit_block_contract(gate_text)
    if fail_and_exit_contract_issues:
        print("aggregate gate runtime5 diagnostics check failed (fail_and_exit contract):")
        for issue in fail_and_exit_contract_issues[:12]:
            print(f" - {issue}")
        return 1

    print("ci aggregate gate runtime5 diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
