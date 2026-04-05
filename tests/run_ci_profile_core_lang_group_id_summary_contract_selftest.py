#!/usr/bin/env python
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


FORCE_ENV = "DDN_CI_PROFILE_AGGREGATE_SMOKE_FORCE_GROUP_ID_SUMMARY_MISMATCH"
FAIL_TOKEN = "ci_profile_core_lang_aggregate_smoke_status=fail reason=aggregate_summary_group_id_summary_mismatch"
MUTATION_TOKEN = (
    "[ci-profile-core-lang-aggregate-smoke] group_id summary selftest mismatch applied key=seamgrim_group_id_summary_status"
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
    print(f"check=ci_profile_core_lang_group_id_summary_contract_selftest detail={detail}")
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
    proc = run([py, "tests/run_ci_profile_core_lang_aggregate_smoke_check.py"], root, env)
    if expect(proc.returncode != 0, "core_lang_aggregate_smoke_case_should_fail", proc) != 0:
        return 1
    stdout = proc.stdout or ""
    if expect(FAIL_TOKEN in stdout, "core_lang_aggregate_smoke_fail_token_missing", proc) != 0:
        return 1
    if expect(MUTATION_TOKEN in stdout, "core_lang_aggregate_smoke_mutation_marker_missing", proc) != 0:
        return 1
    print("[ci-profile-core-lang-group-id-summary-contract-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
