#!/usr/bin/env python
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


FORCE_ENV = "DDN_CI_PROFILE_AGGREGATE_SMOKE_FORCE_RUNTIME_HELPER_SUMMARY_MISMATCH"
FORCE_KEY_ENV = "DDN_CI_PROFILE_AGGREGATE_SMOKE_FORCE_RUNTIME_HELPER_SUMMARY_MISMATCH_KEY"
FORCE_KEY = "ci_sanity_age5_combined_heavy_policy_selftest_ok"
FAIL_TOKEN = "ci_profile_core_lang_aggregate_smoke_status=fail reason=aggregate_summary_runtime_helper_contract_mismatch"
MUTATION_TOKEN = "[ci-profile-core-lang-aggregate-smoke] runtime helper summary selftest mismatch applied key=ci_sanity_age5_combined_heavy_policy_selftest_ok"


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
    print(f"check=ci_profile_core_lang_runtime_helper_contract_selftest detail={detail}")
    if proc is not None:
        if (proc.stdout or "").strip():
            print(proc.stdout.strip())
        if (proc.stderr or "").strip():
            print(proc.stderr.strip())
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    env = os.environ.copy()
    env[FORCE_ENV] = "1"
    env[FORCE_KEY_ENV] = FORCE_KEY
    proc = run([py, "tests/run_ci_profile_core_lang_aggregate_smoke_check.py"], root, env)
    if expect(proc.returncode != 0, "core_lang_aggregate_smoke_case_should_fail", proc) != 0:
        return 1
    stdout = proc.stdout or ""
    if expect(FAIL_TOKEN in stdout, "core_lang_aggregate_smoke_fail_token_missing", proc) != 0:
        return 1
    if expect(MUTATION_TOKEN in stdout, "core_lang_aggregate_smoke_mutation_marker_missing", proc) != 0:
        return 1
    print("[ci-profile-core-lang-runtime-helper-contract-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
