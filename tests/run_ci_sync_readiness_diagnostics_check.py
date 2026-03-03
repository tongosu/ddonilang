#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    sync_check = root / "tests" / "run_ci_sync_readiness_check.py"
    sync_selftest = root / "tests" / "run_ci_sync_readiness_check_selftest.py"
    sync_report_check = root / "tests" / "run_ci_sync_readiness_report_check.py"
    sync_report_selftest = root / "tests" / "run_ci_sync_readiness_report_check_selftest.py"
    for target in (sync_check, sync_selftest, sync_report_check, sync_report_selftest):
        if not target.exists():
            print(f"missing target: {target}")
            return 1

    sync_text = sync_check.read_text(encoding="utf-8")
    selftest_text = sync_selftest.read_text(encoding="utf-8")
    report_check_text = sync_report_check.read_text(encoding="utf-8")
    report_selftest_text = sync_report_selftest.read_text(encoding="utf-8")

    required_sync_tokens = [
        "SANITY_REQUIRED_PASS_STEPS",
        "SYNC_READINESS_OK",
        "SYNC_READINESS_STEP_FAIL",
        "SYNC_READINESS_SANITY_CONTRACT_FAIL",
        "SYNC_READINESS_VALIDATE_ONLY_PATH_MISSING",
        "backup_hygiene_selftest",
        "pipeline_emit_flags_check",
        "pipeline_emit_flags_selftest",
        "seamgrim_ci_gate_seed_meta_step_check",
        "seamgrim_ci_gate_runtime5_passthrough_check",
        "seamgrim_interface_boundary_contract_check",
        "seamgrim_overlay_session_wired_consistency_check",
        "seamgrim_overlay_session_diag_parity_check",
        "seamgrim_overlay_compare_diag_parity_check",
        "age5_close_pack_contract_selftest",
        "ci_pack_golden_age5_surface_selftest",
        "ci_pack_golden_guideblock_selftest",
        "ci_pack_golden_exec_policy_selftest",
        "ci_pack_golden_jjaim_flatten_selftest",
        "ci_pack_golden_event_model_selftest",
        "w92_aot_pack_check",
        "w93_universe_pack_check",
        "w94_social_pack_check",
        "w95_cert_pack_check",
        "w96_somssi_pack_check",
        "w97_self_heal_pack_check",
        "seamgrim_wasm_cli_diag_parity_check",
        "validate_sanity_contract",
        '"name": "sanity_gate_contract"',
        "--validate-only-sanity-json",
        "code=",
        "step=",
        "msg=",
        "ci_sync_readiness_status=",
    ]
    required_selftest_tokens = [
        "steps_count_quick_should_be_5",
        "custom_steps_count_should_be_5",
        '"sanity_gate_contract",',
        "sanity_gate_contract_should_be_ok",
        "validate_only_ok_should_pass",
        "validate_only_bad_should_fail",
        "validate_only_missing_should_fail",
        "E_SYNC_READINESS_SANITY_CONTRACT_FAIL",
        "E_SYNC_READINESS_VALIDATE_ONLY_PATH_MISSING",
    ]
    required_report_check_tokens = [
        "ddn.ci.sync_readiness.v1",
        "--require-pass",
        "from ci_check_error_codes import SYNC_READINESS_REPORT_CODES as CODES",
        'CODES["PASS_STATUS_FIELDS"]',
        'CODES["MISSING_CONTRACT_ROW"]',
        'CODES["VALIDATE_ONLY_SHAPE"]',
        "sanity_gate_contract",
    ]
    required_report_selftest_tokens = [
        "tests/run_ci_sync_readiness_report_check.py",
        "SYNC_READINESS_REPORT_CODES as CODES",
        "report_check_should_pass",
        "validate_only_report_check_should_pass",
        "bad_code_should_fail",
        "bad_contract_row_should_fail",
        "bad_validate_shape_should_fail",
    ]

    missing: list[str] = []
    missing.extend([f"sync_check:{token}" for token in required_sync_tokens if token not in sync_text])
    missing.extend([f"sync_selftest:{token}" for token in required_selftest_tokens if token not in selftest_text])
    missing.extend([f"report_check:{token}" for token in required_report_check_tokens if token not in report_check_text])
    missing.extend(
        [f"report_selftest:{token}" for token in required_report_selftest_tokens if token not in report_selftest_text]
    )

    if missing:
        print("ci sync readiness diagnostics check failed:")
        for token in missing[:16]:
            print(f" - missing token: {token}")
        return 1

    print("ci sync readiness diagnostics check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
