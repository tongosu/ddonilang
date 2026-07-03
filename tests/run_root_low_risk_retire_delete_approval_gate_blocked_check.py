#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOC = ROOT / "ROOT_LOW_RISK_RETIRE_DELETE_APPROVAL_GATE_BLOCKED_V1.md"
APPROVAL = ROOT / "ROOT_LOW_RISK_RETIRE_APPROVAL_v1.md"
PREFLIGHT = ROOT / "ROOT_LOW_RISK_RETIRE_DELETE_PREFLIGHT_V1.md"
DRY_RUN = ROOT / "ROOT_LOW_RISK_RETIRE_DELETE_DRY_RUN_PLAN_V1.md"
QUEUE = ROOT / "NEXT_WORK_QUEUE_AFTER_CONNECT_V1.md"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def require_files() -> int:
    required = [
        DOC,
        APPROVAL,
        PREFLIGHT,
        DRY_RUN,
        QUEUE,
        ROOT / "tests" / "run_root_low_risk_retire_delete_preflight_check.py",
        ROOT / "tests" / "run_root_low_risk_retire_delete_dry_run_plan_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_ROOT_RETIRE_APPROVAL_GATE_MISSING", str(missing))
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
            "blocked_waiting_for_explicit_delete_approval",
            "This document is not approval",
            "Repeated requests to \"continue next development\" do not override the approval gate",
            "Not allowed without approval: delete, move, retire, archive, or path-rewrite candidate files",
            "docs/ssot/**",
        ],
        "E_ROOT_RETIRE_APPROVAL_GATE_DOC",
    )


def check_approval_package() -> int:
    return require_tokens(
        APPROVAL,
        [
            "approval_requested",
            "no_delete",
            "no_move",
            "This approval package is not deletion approval by itself",
            "must receive explicit user approval",
        ],
        "E_ROOT_RETIRE_APPROVAL_GATE_APPROVAL_DOC",
    )


def check_queue() -> int:
    text = read(QUEUE)
    required = [
        "ROOT_LOW_RISK_RETIRE_DELETE_APPROVAL_GATE_BLOCKED_V1",
        "closed by `ROOT_LOW_RISK_RETIRE_DELETE_APPROVAL_GATE_BLOCKED_V1.md`",
        "blocked_waiting_for_explicit_delete_approval",
        "ROOT_LOW_RISK_RETIRE_DELETE_V1",
        "Approval-gated",
    ]
    missing = [token for token in required if token not in text]
    if missing:
        return fail("E_ROOT_RETIRE_APPROVAL_GATE_QUEUE", str(missing))
    if "1. `ROOT_LOW_RISK_RETIRE_DELETE_V1`" not in text:
        return fail("E_ROOT_RETIRE_APPROVAL_GATE_NEXT", "root delete gate is not next")
    return 0


def run_check(script: str, code: str) -> int:
    result = subprocess.run(
        ["python", script],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode != 0:
        return fail(code, result.stdout.strip())
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
        return fail("E_ROOT_RETIRE_APPROVAL_GATE_SSOT_STATUS", result.stdout.strip())
    if result.stdout.strip():
        return fail("E_ROOT_RETIRE_APPROVAL_GATE_SSOT_DIRTY", result.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_doc,
        check_approval_package,
        check_queue,
        lambda: run_check(
            "tests/run_root_low_risk_retire_delete_preflight_check.py",
            "E_ROOT_RETIRE_APPROVAL_GATE_PREFLIGHT",
        ),
        lambda: run_check(
            "tests/run_root_low_risk_retire_delete_dry_run_plan_check.py",
            "E_ROOT_RETIRE_APPROVAL_GATE_DRY_RUN",
        ),
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[root-low-risk-retire-delete-approval-gate-blocked-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
