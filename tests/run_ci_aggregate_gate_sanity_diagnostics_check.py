#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


REQUIRED_TOKEN_MAP = {
    "tests/run_ci_aggregate_gate.py": [
        "ci_sanity_gate_base_name",
        "ci_sanity_gate_report",
        "check_ci_sanity_gate",
        "tests/run_ci_sanity_gate.py",
        "check_ci_sanity_gate_diagnostics",
        "tests/run_ci_sanity_gate_diagnostics_check.py",
        "check_ci_emit_artifacts_sanity_contract",
        "tests/run_ci_emit_artifacts_sanity_contract_check.py",
        "ci_emit_artifacts_sanity_contract_check",
        "append_ci_sanity_summary_lines(",
        "check_ci_aggregate_gate_sanity_diagnostics",
        "ci_aggregate_gate_sanity_diagnostics_check",
        "tests/run_ci_aggregate_gate_sanity_diagnostics_check.py",
        "check_ci_pack_golden_event_model_selftest",
        "ci_pack_golden_event_model_selftest",
        "tests/run_pack_golden_event_model_selftest.py",
        "ci_pack_golden_event_model_selftest_ok=",
    ],
    "tests/_ci_aggregate_diag_lib.py": [
        "[ci-gate-summary] ci_sanity_gate_report=",
        "[ci-gate-summary] ci_sanity_gate_status=",
        "[ci-gate-summary] ci_sanity_gate_code=",
        "ci_sanity_seamgrim_interface_boundary_ok",
        "ci_sanity_overlay_session_wired_consistency_ok",
        "ci_sanity_overlay_session_diag_parity_ok",
        "ci_sanity_overlay_compare_diag_parity_ok",
    ],
    "tests/run_ci_gate_summary_report_check.py": [
        "ci_sanity_seamgrim_interface_boundary_ok",
        "ci_sanity_overlay_session_wired_consistency_ok",
        "ci_sanity_overlay_session_diag_parity_ok",
        "ci_sanity_overlay_compare_diag_parity_ok",
        "PASS summary requires ci_sanity_seamgrim_interface_boundary_ok=1",
        "PASS summary requires ci_sanity_overlay_session_wired_consistency_ok=1",
        "PASS summary requires ci_sanity_overlay_session_diag_parity_ok=1",
        "PASS summary requires ci_sanity_overlay_compare_diag_parity_ok=1",
    ],
    "tests/run_ci_sanity_gate.py": [
        "ci_pack_golden_age5_surface_selftest",
        "ci_pack_golden_guideblock_selftest",
        "tests/run_pack_golden_age5_surface_selftest.py",
        "tests/run_pack_golden_guideblock_selftest.py",
        "ci_pack_golden_exec_policy_selftest",
        "tests/run_pack_golden_exec_policy_selftest.py",
        "ci_pack_golden_jjaim_flatten_selftest",
        "tests/run_pack_golden_jjaim_flatten_selftest.py",
        "ci_pack_golden_event_model_selftest",
        "tests/run_pack_golden_event_model_selftest.py",
        "ci_profile_split_contract_check",
        "tests/run_ci_profile_split_contract_check.py",
        "w92_aot_pack_check",
        "tests/run_w92_aot_pack_check.py",
        "w93_universe_pack_check",
        "tests/run_w93_universe_pack_check.py",
        "w94_social_pack_check",
        "tests/run_w94_social_pack_check.py",
        "w95_cert_pack_check",
        "tests/run_w95_cert_pack_check.py",
        "w96_somssi_pack_check",
        "tests/run_w96_somssi_pack_check.py",
        "w97_self_heal_pack_check",
        "tests/run_w97_self_heal_pack_check.py",
        "seamgrim_interface_boundary_contract_check",
        "tests/run_seamgrim_interface_boundary_contract_check.py",
        "seamgrim_wasm_cli_diag_parity_check",
        "tests/run_seamgrim_wasm_cli_diag_parity_check.py",
    ],
    "tests/run_ci_sync_readiness_check.py": [
        "SANITY_REQUIRED_PASS_STEPS",
        "ci_pack_golden_age5_surface_selftest",
        "ci_pack_golden_guideblock_selftest",
        "ci_pack_golden_exec_policy_selftest",
        "ci_pack_golden_jjaim_flatten_selftest",
        "ci_pack_golden_event_model_selftest",
        "ci_profile_split_contract_check",
        "w92_aot_pack_check",
        "w93_universe_pack_check",
        "w94_social_pack_check",
        "w95_cert_pack_check",
        "w96_somssi_pack_check",
        "w97_self_heal_pack_check",
        "seamgrim_interface_boundary_contract_check",
        "seamgrim_wasm_cli_diag_parity_check",
    ],
    "tests/run_ci_emit_artifacts_check.py": [
        "SANITY_REQUIRED_PASS_STEPS",
        "ci_pack_golden_age5_surface_selftest",
        "ci_pack_golden_guideblock_selftest",
        "ci_pack_golden_exec_policy_selftest",
        "ci_pack_golden_jjaim_flatten_selftest",
        "ci_pack_golden_event_model_selftest",
        "ci_profile_split_contract_check",
        "w92_aot_pack_check",
        "w93_universe_pack_check",
        "w94_social_pack_check",
        "w95_cert_pack_check",
        "w96_somssi_pack_check",
        "w97_self_heal_pack_check",
        "seamgrim_interface_boundary_contract_check",
        "seamgrim_wasm_cli_diag_parity_check",
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
        print("aggregate gate sanity diagnostics check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1

    print("ci aggregate gate sanity diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
