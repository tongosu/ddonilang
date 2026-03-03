#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


REQUIRED_TOKENS = [
    "tests/run_ci_sync_readiness_check_selftest.py",
    "check_ci_sync_readiness_selftest",
    "ci_sync_readiness_selftest",
    "check_ci_sync_readiness_diagnostics",
    "ci_sync_readiness_diagnostics_check",
    "tests/run_ci_sync_readiness_diagnostics_check.py",
    "check_ci_sync_readiness_report_selftest",
    "ci_sync_readiness_report_selftest",
    "tests/run_ci_sync_readiness_report_check_selftest.py",
    "run_ci_sync_readiness_report_generate",
    "ci_sync_readiness_report_generate",
    "check_ci_sync_readiness_report_check",
    "ci_sync_readiness_report_check",
    "tests/run_ci_sync_readiness_report_check.py",
    "check_ci_gate_report_index_selftest",
    "ci_gate_report_index_selftest",
    "tests/run_ci_gate_report_index_check_selftest.py",
    "check_ci_gate_report_index",
    "ci_gate_report_index_check",
    "tests/run_ci_gate_report_index_check.py",
    "check_ci_gate_report_index_diagnostics",
    "ci_gate_report_index_diagnostics_check",
    "tests/run_ci_gate_report_index_diagnostics_check.py",
    "report_index_required_steps",
    "require_step_contract",
    "--required-step",
    "check_ci_gate_report_index(require_step_contract=True)",
    "--index",
    "append_ci_sync_readiness_summary_lines(",
    "[ci-gate-summary] ci_sync_readiness_report=",
    "[ci-gate-summary] ci_sync_readiness_status=",
    "[ci-gate-summary] ci_sync_readiness_code=",
    "check_seamgrim_ci_gate_runtime5_passthrough",
    "seamgrim_ci_gate_runtime5_passthrough_check",
    "tests/run_seamgrim_ci_gate_runtime5_passthrough_check.py",
    "check_seamgrim_ci_gate_seed_meta_step",
    "seamgrim_ci_gate_seed_meta_step_check",
    "tests/run_seamgrim_ci_gate_seed_meta_step_check.py",
    "check_seamgrim_ci_gate_guideblock_step",
    "seamgrim_ci_gate_guideblock_step_check",
    "tests/run_seamgrim_ci_gate_guideblock_step_check.py",
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
    "check_ci_aggregate_gate_sync_diagnostics",
    "ci_aggregate_gate_sync_diagnostics_check",
    "tests/run_ci_aggregate_gate_sync_diagnostics_check.py",
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
        print("aggregate gate sync diagnostics check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1

    print("ci aggregate gate sync diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
