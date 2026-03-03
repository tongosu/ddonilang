#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


REQUIRED_TOKENS = [
    "pipeline_emit_flags_check",
    "tests/run_ci_pipeline_emit_flags_check.py",
    "pipeline_emit_flags_selftest",
    "tests/run_ci_pipeline_emit_flags_check_selftest.py",
    "E_CI_SANITY_PIPELINE_FLAGS_SELFTEST_FAIL",
    "seamgrim_ci_gate_seed_meta_step_check",
    "tests/run_seamgrim_ci_gate_seed_meta_step_check.py",
    "E_CI_SANITY_SEED_META_STEP_FAIL",
    "seamgrim_ci_gate_runtime5_passthrough_check",
    "tests/run_seamgrim_ci_gate_runtime5_passthrough_check.py",
    "E_CI_SANITY_RUNTIME5_PASSTHROUGH_FAIL",
    "seamgrim_overlay_session_wired_consistency_check",
    "tests/run_seamgrim_overlay_session_wired_consistency_check.py",
    "E_CI_SANITY_OVERLAY_SESSION_WIRED_CONSISTENCY_FAIL",
    "seamgrim_overlay_session_diag_parity_check",
    "tests/run_seamgrim_overlay_session_diag_parity_check.py",
    "E_CI_SANITY_OVERLAY_SESSION_DIAG_PARITY_FAIL",
    "seamgrim_overlay_compare_diag_parity_check",
    "tests/run_seamgrim_overlay_compare_diag_parity_check.py",
    "E_CI_SANITY_OVERLAY_COMPARE_DIAG_PARITY_FAIL",
    "age5_close_pack_contract_selftest",
    "tests/run_age5_close_pack_contract_selftest.py",
    "E_CI_SANITY_AGE5_CLOSE_PACK_CONTRACT_SELFTEST_FAIL",
    "ci_pack_golden_age5_surface_selftest",
    "tests/run_pack_golden_age5_surface_selftest.py",
    "E_CI_SANITY_PACK_GOLDEN_AGE5_SURFACE_SELFTEST_FAIL",
    "ci_pack_golden_guideblock_selftest",
    "tests/run_pack_golden_guideblock_selftest.py",
    "E_CI_SANITY_PACK_GOLDEN_GUIDEBLOCK_SELFTEST_FAIL",
    "ci_pack_golden_exec_policy_selftest",
    "tests/run_pack_golden_exec_policy_selftest.py",
    "E_CI_SANITY_PACK_GOLDEN_EXEC_POLICY_SELFTEST_FAIL",
    "ci_pack_golden_jjaim_flatten_selftest",
    "tests/run_pack_golden_jjaim_flatten_selftest.py",
    "E_CI_SANITY_PACK_GOLDEN_JJAIM_FLATTEN_SELFTEST_FAIL",
    "ci_pack_golden_event_model_selftest",
    "tests/run_pack_golden_event_model_selftest.py",
    "E_CI_SANITY_PACK_GOLDEN_EVENT_MODEL_SELFTEST_FAIL",
    "w92_aot_pack_check",
    "tests/run_w92_aot_pack_check.py",
    "E_CI_SANITY_W92_AOT_PACK_CHECK_FAIL",
    "w93_universe_pack_check",
    "tests/run_w93_universe_pack_check.py",
    "E_CI_SANITY_W93_UNIVERSE_PACK_CHECK_FAIL",
    "w94_social_pack_check",
    "tests/run_w94_social_pack_check.py",
    "E_CI_SANITY_W94_SOCIAL_PACK_CHECK_FAIL",
    "w95_cert_pack_check",
    "tests/run_w95_cert_pack_check.py",
    "E_CI_SANITY_W95_CERT_PACK_CHECK_FAIL",
    "w96_somssi_pack_check",
    "tests/run_w96_somssi_pack_check.py",
    "E_CI_SANITY_W96_SOMSSI_PACK_CHECK_FAIL",
    "w97_self_heal_pack_check",
    "tests/run_w97_self_heal_pack_check.py",
    "E_CI_SANITY_W97_SELF_HEAL_PACK_CHECK_FAIL",
    "seamgrim_wasm_cli_diag_parity_check",
    "tests/run_seamgrim_wasm_cli_diag_parity_check.py",
    "E_CI_SANITY_WASM_CLI_DIAG_PARITY_FAIL",
]


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    target = root / "tests" / "run_ci_sanity_gate.py"
    if not target.exists():
        print(f"missing target: {target}")
        return 1
    text = target.read_text(encoding="utf-8")

    missing = [token for token in REQUIRED_TOKENS if token not in text]
    if missing:
        print("ci sanity gate diagnostics check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1

    print("ci sanity gate diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
