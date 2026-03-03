#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    emit_check = root / "tests" / "run_ci_emit_artifacts_check.py"
    emit_selftest = root / "tests" / "run_ci_emit_artifacts_check_selftest.py"
    code_map = root / "tests" / "ci_check_error_codes.py"

    for target in (emit_check, emit_selftest, code_map):
        if not target.exists():
            print(f"missing target: {target}")
            return 1

    emit_check_text = emit_check.read_text(encoding="utf-8")
    emit_selftest_text = emit_selftest.read_text(encoding="utf-8")
    code_map_text = code_map.read_text(encoding="utf-8")

    required_emit_check_tokens = [
        "SANITY_REQUIRED_PASS_STEPS",
        '"seamgrim_overlay_session_diag_parity_check"',
        '"seamgrim_overlay_compare_diag_parity_check"',
        '"seamgrim_overlay_session_wired_consistency_check"',
        '"seamgrim_interface_boundary_contract_check"',
        '"age5_close_pack_contract_selftest"',
        '"ci_pack_golden_age5_surface_selftest"',
        '"ci_pack_golden_guideblock_selftest"',
        '"ci_pack_golden_exec_policy_selftest"',
        '"ci_pack_golden_jjaim_flatten_selftest"',
        '"ci_pack_golden_event_model_selftest"',
        '"w92_aot_pack_check"',
        '"w93_universe_pack_check"',
        '"w94_social_pack_check"',
        '"w95_cert_pack_check"',
        '"w96_somssi_pack_check"',
        '"w97_self_heal_pack_check"',
        '"seamgrim_wasm_cli_diag_parity_check"',
        '"ci_sync_readiness"',
        "ddn.ci.sync_readiness.v1",
        'code=CODES["SYNC_READINESS_PATH_MISSING"]',
        'code=CODES["SYNC_READINESS_SCHEMA_MISMATCH"]',
        'code=CODES["SYNC_READINESS_STATUS_UNSUPPORTED"]',
        'code=CODES["SYNC_READINESS_STATUS_MISMATCH"]',
        'code=CODES["SYNC_READINESS_PASS_STATUS_FIELDS"]',
        'code=CODES["SANITY_REQUIRED_STEP_MISSING"]',
        'code=CODES["SANITY_REQUIRED_STEP_FAILED"]',
    ]
    required_emit_selftest_tokens = [
        "broken_sanity_compare_step_missing",
        "broken_sanity_compare_step_failed",
        "badsanitycomparemissing",
        "badsanitycomparefailed",
        "broken_sanity_wired_step_missing",
        "broken_sanity_wired_step_failed",
        "badsanitywiredmissing",
        "badsanitywiredfailed",
        "broken_sanity_required_step_missing",
        "broken_sanity_required_step_failed",
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
        "seamgrim_interface_boundary_contract_check",
        "seamgrim_wasm_cli_diag_parity_check",
        "with_sync_readiness",
        "broken_sync_readiness_schema",
        "broken_sync_readiness_status_unsupported",
        "broken_sync_readiness_status_mismatch",
        "broken_sync_readiness_pass_fields",
        "misssync",
        "badsyncschema",
        "badsyncstatus",
        "badsyncmismatch",
        "badsyncpassfields",
    ]
    required_code_tokens = [
        '"SANITY_REQUIRED_STEP_MISSING": "E_SANITY_REQUIRED_STEP_MISSING"',
        '"SANITY_REQUIRED_STEP_FAILED": "E_SANITY_REQUIRED_STEP_FAILED"',
        '"SYNC_READINESS_PATH_MISSING": "E_SYNC_READINESS_PATH_MISSING"',
        '"SYNC_READINESS_SCHEMA_MISMATCH": "E_SYNC_READINESS_SCHEMA_MISMATCH"',
        '"SYNC_READINESS_STATUS_UNSUPPORTED": "E_SYNC_READINESS_STATUS_UNSUPPORTED"',
        '"SYNC_READINESS_STATUS_MISMATCH": "E_SYNC_READINESS_STATUS_MISMATCH"',
        '"SYNC_READINESS_PASS_STATUS_FIELDS": "E_SYNC_READINESS_PASS_STATUS_FIELDS"',
    ]

    missing: list[str] = []
    missing.extend([f"emit_check:{token}" for token in required_emit_check_tokens if token not in emit_check_text])
    missing.extend([f"emit_selftest:{token}" for token in required_emit_selftest_tokens if token not in emit_selftest_text])
    missing.extend([f"code_map:{token}" for token in required_code_tokens if token not in code_map_text])

    if missing:
        print("ci emit artifacts sanity contract check failed:")
        for token in missing[:16]:
            print(f" - missing token: {token}")
        return 1

    print("ci emit artifacts sanity contract check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
