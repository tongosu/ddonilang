#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "AGE5_CI_FRONTIER_RECHECK_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        DOC,
        QUEUE,
        ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_CASE_SUITE_CHECK_RUNNER_V1V.md",
        ROOT / "pack" / "connect_flow_v1v_closure_v1" / "contract.detjson",
        ROOT / "STD_EVENT_MINIMUM_CLOSURE_V1.md",
        ROOT / "pack" / "std_event_minimum_closure_v1" / "golden.jsonl",
        ROOT / "tests" / "_ci_profile_matrix_full_real_smoke_contract.py",
        ROOT / "tests" / "run_profile_matrix_full_real_smoke_policy_selftest.py",
        ROOT / "tools" / "scripts" / "resolve_profile_matrix_full_real_smoke_policy.py",
        ROOT / "tests" / "run_ci_sanity_gate.py",
        ROOT / "tests" / "run_ci_sync_readiness_check_selftest.py",
        ROOT / "tests" / "run_ci_sync_readiness_diagnostics_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_AGE5_CI_FRONTIER_RECHECK_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_doc() -> int:
    return require_tokens(
        DOC,
        [
            "documentation/checker-only",
            "no product code",
            "profile_matrix_full_real_smoke_policy_selftest",
            "core_lang:900",
            "full:1200",
            "seamgrim:1500",
            "E_SYNC_READINESS_STEP_FAIL",
            "validate-only readiness path",
            "not treated as a blocker",
            "ROADMAP_V2_FOLLOWON_REBASE_V1",
            "ROOT_LOW_RISK_RETIRE_DELETE_V1",
            "approval-gated",
            "docs/ssot/**",
        ],
        "E_AGE5_CI_FRONTIER_RECHECK_DOC",
    )


def check_queue() -> int:
    rc = require_tokens(
        QUEUE,
        [
            "AGE5_CI_FRONTIER_RECHECK_V1",
            "closed by `AGE5_CI_FRONTIER_RECHECK_V1.md`",
            "not a current blocker",
            "ROADMAP_V2_FOLLOWON_REBASE_V1",
            "ROADMAP_V2_DA1_MATH_REBASE_V1",
            "ROOT_LOW_RISK_RETIRE_DELETE_V1",
            "Approval-gated",
            "connect_flow_v1w_*",
            "docs/ssot/**",
        ],
        "E_AGE5_CI_FRONTIER_RECHECK_QUEUE",
    )
    if rc:
        return rc
    text = read(QUEUE)
    if "1. `AGE5_CI_FRONTIER_RECHECK_V1`" in text:
        return fail(
            "E_AGE5_CI_FRONTIER_RECHECK_QUEUE_OPEN",
            "AGE5 recheck is still listed as the next open queue item",
        )
    return 0


def check_profile_matrix_frontier() -> int:
    checks = [
        (
            ROOT / "tests" / "_ci_profile_matrix_full_real_smoke_contract.py",
            [
                "PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT",
                "PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_CORE_LANG = 900.0",
                "PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_FULL = 1200.0",
                "PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_SEAMGRIM = 1500.0",
                "DDN_CI_PROFILE_GATE_WITH_PROFILE_MATRIX_FULL_REAL_SMOKE",
            ],
        ),
        (
            ROOT / "tests" / "run_profile_matrix_full_real_smoke_policy_selftest.py",
            [
                "gitlab_manual_optin",
                "gitlab_schedule",
                "azure_schedule",
                "fast_path_json",
                "step_timeout_defaults",
            ],
        ),
        (
            ROOT / "tests" / "run_ci_sanity_gate.py",
            [
                "profile_matrix_full_real_smoke_policy_selftest",
                "ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok",
            ],
        ),
    ]
    for path, tokens in checks:
        rc = require_tokens(path, tokens, "E_AGE5_CI_FRONTIER_RECHECK_PROFILE_MATRIX")
        if rc:
            return rc
    return 0


def check_readiness_frontier() -> int:
    checks = [
        (
            ROOT / "tests" / "run_ci_sync_readiness_check_selftest.py",
            [
                "validate_only_ok_should_pass",
                "validate_only_bad_should_fail",
                "SEAMGRIM_BLOCKER_SYNC_READINESS_VALIDATE_ONLY_CASES",
                "profile_matrix_full_real_smoke_policy_selftest",
            ],
        ),
        (
            ROOT / "tests" / "run_ci_sync_readiness_diagnostics_check.py",
            [
                "SYNC_READINESS_STEP_FAIL",
                "SYNC_READINESS_VALIDATE_ONLY_PATH_MISSING",
                "profile_matrix_full_real_smoke_policy_selftest",
                "seamgrim_v2_task_batch_check",
            ],
        ),
        (
            ROOT / "docs" / "context" / "all" / "DEV_SUMMARY.md",
            [
                "E_SYNC_READINESS_STEP_FAIL",
                "validate-only",
                "profile_matrix_full_real_smoke_policy_selftest",
                "step_timeout_defaults=core_lang:900,full:1200,seamgrim:1500",
            ],
        ),
    ]
    for path, tokens in checks:
        rc = require_tokens(path, tokens, "E_AGE5_CI_FRONTIER_RECHECK_READINESS")
        if rc:
            return rc
    return 0


def check_docs_ssot_clean() -> int:
    result = subprocess.run(
        ["git", "status", "--short", "--", "docs/ssot"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode != 0:
        return fail("E_AGE5_CI_FRONTIER_RECHECK_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_AGE5_CI_FRONTIER_RECHECK_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_doc,
        check_queue,
        check_profile_matrix_frontier,
        check_readiness_frontier,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[age5-ci-frontier-recheck-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
