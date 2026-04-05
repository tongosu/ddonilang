#!/usr/bin/env python
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


FORCE_ENV = "DDN_CI_PROFILE_GATE_FORCE_RUNTIME_HELPER_SUMMARY_MISMATCH"
FORCE_KEY_ENV = "DDN_CI_PROFILE_GATE_FORCE_RUNTIME_HELPER_SUMMARY_MISMATCH_KEY"
CASES = (
    (
        "full",
        "tests/run_ci_profile_full_gate.py",
        [],
        "ci_sanity_age5_combined_heavy_policy_selftest_ok",
        "ci_profile_full_status=fail reason=aggregate_summary_runtime_helper_contract_mismatch",
        "[ci-profile-full] runtime helper summary selftest mismatch applied key=ci_sanity_age5_combined_heavy_policy_selftest_ok",
    ),
    (
        "seamgrim",
        "tests/run_ci_profile_seamgrim_gate.py",
        [],
        "ci_sync_readiness_ci_sanity_age5_combined_heavy_policy_selftest_ok",
        "ci_profile_seamgrim_status=fail reason=aggregate_summary_runtime_helper_contract_mismatch",
        "[ci-profile-seamgrim] runtime helper summary selftest mismatch applied key=ci_sync_readiness_ci_sanity_age5_combined_heavy_policy_selftest_ok",
    ),
)


def run(cmd: list[str], cwd: Path, env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def expect(cond: bool, detail: str, proc: subprocess.CompletedProcess[str] | None = None) -> int:
    if cond:
        return 0
    print(f"check=ci_profile_gate_runtime_helper_contract_selftest detail={detail}")
    if proc is not None:
        if (proc.stdout or "").strip():
            print(proc.stdout.strip())
        if (proc.stderr or "").strip():
            print(proc.stderr.strip())
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    py = sys.executable

    for profile_name, script_path, extra_args, target_key, fail_token, mutation_token in CASES:
        env = os.environ.copy()
        env[FORCE_ENV] = "1"
        env[FORCE_KEY_ENV] = target_key
        proc = run([py, script_path, *extra_args], root, env)
        if expect(proc.returncode != 0, f"{profile_name}_case_should_fail", proc) != 0:
            return 1
        stdout = proc.stdout or ""
        if expect(fail_token in stdout, f"{profile_name}_fail_token_missing", proc) != 0:
            return 1
        if expect(mutation_token in stdout, f"{profile_name}_mutation_marker_missing", proc) != 0:
            return 1

    print("[ci-profile-gate-runtime-helper-contract-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
