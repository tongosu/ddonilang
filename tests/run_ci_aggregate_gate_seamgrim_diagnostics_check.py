#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from _ci_fail_and_exit_contract import (
    FAST_FAIL_REENTRY_GUARD_TOKENS,
    validate_fail_and_exit_block_contract,
)


CONTROL_EXPOSURE_TOKENS = (
    "seamgrim_control_exposure_failures_base_name",
    "seamgrim_control_exposure_failures_report",
    "write_control_exposure_failure_report(",
    "[ci-gate-summary] seamgrim_control_exposure_policy_report=",
    "[ci-gate-summary] seamgrim_control_exposure_policy_status=",
)

SEAMGRIM_SUMMARY_TOKENS = (
    "append_seamgrim_focus_summary_lines(",
    "[ci-gate-summary] seamgrim_seed_meta_files_status=",
    "[ci-gate-summary] seamgrim_seed_overlay_quality_status=",
    "[ci-gate-summary] seamgrim_rewrite_overlay_quality_status=",
    "[ci-gate-summary] seamgrim_guideblock_keys_pack_status=",
    "[ci-gate-summary] seamgrim_moyang_view_boundary_pack_status=",
    "[ci-gate-summary] seamgrim_rewrite_overlay_quality_report=",
    "[ci-gate-summary] seamgrim_rewrite_overlay_quality_top=",
    "[ci-gate-summary] seamgrim_pendulum_surface_contract_status=",
    "[ci-gate-summary] seamgrim_seed_pendulum_export_status=",
    "[ci-gate-summary] seamgrim_pendulum_runtime_visual_status=",
    "[ci-gate-summary] seamgrim_seed_runtime_visual_pack_status=",
    "[ci-gate-summary] seamgrim_group_id_summary_status=",
    "[ci-gate-summary] seamgrim_runtime_fallback_metrics_status=",
    "[ci-gate-summary] seamgrim_runtime_fallback_policy_status=",
    "[ci-gate-summary] seamgrim_pendulum_bogae_shape_status=",
)

WASM_PARITY_TOKENS = (
    "[ci-gate-summary] seamgrim_wasm_cli_diag_parity_report=",
    "[ci-gate-summary] seamgrim_wasm_cli_diag_parity_ok=",
    "seamgrim_wasm_cli_diag_parity_base_name",
    "seamgrim_wasm_cli_diag_parity_report",
    '"seamgrim_wasm_cli_diag_parity":',
    "check_seamgrim_wasm_cli_diag_parity",
    "seamgrim_wasm_cli_diag_parity_check",
    "tests/run_seamgrim_wasm_cli_diag_parity_check.py",
)

RUNNER_TOKENS = (
    "--json-out",
    "check_ci_aggregate_gate_seamgrim_diagnostics",
    "ci_aggregate_gate_seamgrim_diagnostics_check",
    "tests/run_ci_aggregate_gate_seamgrim_diagnostics_check.py",
    "ci_aggregate_gate_seamgrim_diagnostics_rc",
    "check_seamgrim_browse_selection_report_selftest",
    "seamgrim_browse_selection_report_selftest",
    "tests/run_seamgrim_browse_selection_report_check_selftest.py",
)

REQUIRED_TOKENS = [
    *CONTROL_EXPOSURE_TOKENS,
    *SEAMGRIM_SUMMARY_TOKENS,
    *WASM_PARITY_TOKENS,
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
        print("aggregate gate seamgrim diagnostics check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1
    fail_and_exit_contract_issues = validate_fail_and_exit_block_contract(text)
    if fail_and_exit_contract_issues:
        print("aggregate gate seamgrim diagnostics check failed (fail_and_exit contract):")
        for issue in fail_and_exit_contract_issues[:12]:
            print(f" - {issue}")
        return 1

    print("ci aggregate gate seamgrim diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
