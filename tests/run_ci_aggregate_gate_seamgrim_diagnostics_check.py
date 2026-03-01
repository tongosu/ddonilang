#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


REQUIRED_TOKENS = [
    "seamgrim_control_exposure_failures_base_name",
    "seamgrim_control_exposure_failures_report",
    "write_control_exposure_failure_report(",
    "append_seamgrim_focus_summary_lines(",
    "[ci-gate-summary] seamgrim_control_exposure_policy_report=",
    "[ci-gate-summary] seamgrim_control_exposure_policy_status=",
    "[ci-gate-summary] seamgrim_seed_meta_files_status=",
    "[ci-gate-summary] seamgrim_seed_overlay_quality_status=",
    "[ci-gate-summary] seamgrim_rewrite_overlay_quality_status=",
    "[ci-gate-summary] seamgrim_rewrite_overlay_quality_report=",
    "[ci-gate-summary] seamgrim_rewrite_overlay_quality_top=",
    "[ci-gate-summary] seamgrim_pendulum_surface_contract_status=",
    "[ci-gate-summary] seamgrim_seed_pendulum_export_status=",
    "[ci-gate-summary] seamgrim_pendulum_runtime_visual_status=",
    "[ci-gate-summary] seamgrim_seed_runtime_visual_pack_status=",
    "[ci-gate-summary] seamgrim_runtime_fallback_metrics_status=",
    "[ci-gate-summary] seamgrim_runtime_fallback_policy_status=",
    "[ci-gate-summary] seamgrim_pendulum_bogae_shape_status=",
    "check_ci_aggregate_gate_seamgrim_diagnostics",
    "ci_aggregate_gate_seamgrim_diagnostics_check",
    "tests/run_ci_aggregate_gate_seamgrim_diagnostics_check.py",
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

    print("ci aggregate gate seamgrim diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
