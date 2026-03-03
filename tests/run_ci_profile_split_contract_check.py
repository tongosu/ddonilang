#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    targets = {
        "tests/run_ci_sanity_gate.py": [
            '--profile',
            '"full", "core_lang", "seamgrim"',
            "CORE_LANG_PROFILE_STEPS",
            "SEAMGRIM_PROFILE_STEPS",
            "ci_profile_split_contract_check",
            "E_CI_SANITY_PROFILE_SPLIT_CONTRACT_FAIL",
            "profile={args.profile}",
        ],
        "tests/run_ci_profile_core_lang_gate.py": [
            "tests/run_ci_sanity_gate.py",
            "tests/run_ci_sync_readiness_check.py",
            "tests/run_ci_sync_readiness_report_check.py",
            "--profile",
            "core_lang",
            "--skip-aggregate",
            "--sanity-profile",
            "--require-pass",
            "ci_sync_readiness_status=pass",
            "sanity_profile=core_lang",
            "ci_profile_core_lang_status=pass",
        ],
        "tests/run_ci_profile_full_gate.py": [
            "tests/run_ci_sanity_gate.py",
            "tests/run_ci_sync_readiness_check.py",
            "tests/run_ci_sync_readiness_report_check.py",
            "--profile",
            "full",
            "--skip-aggregate",
            "--sanity-profile",
            "--require-pass",
            "ci_sync_readiness_status=pass",
            "sanity_profile=full",
            "ci_profile_full_status=pass",
        ],
        "tests/run_ci_profile_seamgrim_gate.py": [
            "tests/run_ci_sanity_gate.py",
            "tests/run_ci_sync_readiness_check.py",
            "tests/run_ci_sync_readiness_report_check.py",
            "--profile",
            "seamgrim",
            "--skip-aggregate",
            "--sanity-profile",
            "--require-pass",
            "ci_sync_readiness_status=pass",
            "sanity_profile=seamgrim",
            "ci_profile_seamgrim_status=pass",
        ],
        "tests/run_ci_profile_matrix_gate.py": [
            "MATRIX_SCHEMA",
            "MATRIX_OK",
            "MATRIX_PROFILE_INVALID",
            "MATRIX_STEP_FAIL",
            "VALID_PROFILES = (\"core_lang\", \"full\", \"seamgrim\")",
            "PROFILE_GATE_SCRIPTS",
            "\"core_lang\": \"tests/run_ci_profile_core_lang_gate.py\"",
            "\"full\": \"tests/run_ci_profile_full_gate.py\"",
            "\"seamgrim\": \"tests/run_ci_profile_seamgrim_gate.py\"",
            "PROFILE_PASS_MARKERS",
            "ci_profile_core_lang_status=pass",
            "ci_profile_full_status=pass",
            "ci_profile_seamgrim_status=pass",
            "--profiles",
            "--dry-run",
            "--stop-on-fail",
            "ci_profile_matrix_status=",
            "ddn.ci.profile_matrix_gate.v1",
        ],
        "tests/run_ci_profile_matrix_gate_selftest.py": [
            "run_ci_profile_matrix_gate.py",
            "CI_PROFILE_MATRIX_CODES as CODES",
            "MATRIX_SCHEMA = \"ddn.ci.profile_matrix_gate.v1\"",
            "dry_run_should_pass",
            "invalid_profile_should_fail",
            "dedupe_case_should_pass",
            "real_core_lang_should_pass",
            "ci_profile_core_lang_status=pass",
            "ci_profile_matrix_status=pass",
            "ci profile matrix gate selftest ok",
            "PROFILE_INVALID",
        ],
    }

    missing: list[str] = []
    for rel_path, required_tokens in targets.items():
        path = root / rel_path
        if not path.exists():
            missing.append(f"{rel_path}: file missing")
            continue
        text = path.read_text(encoding="utf-8")
        for token in required_tokens:
            if token not in text:
                missing.append(f"{rel_path}: missing token: {token}")

    if missing:
        print("ci profile split contract check failed:")
        for item in missing[:16]:
            print(f" - {item}")
        return 1

    print("ci profile split contract check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
