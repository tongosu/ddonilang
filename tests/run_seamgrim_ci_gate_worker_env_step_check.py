#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path


FILE_TOKEN_REQUIREMENTS: dict[str, tuple[str, ...]] = {
    "tests/_ci_seamgrim_step_contract.py": (
        '"seamgrim_ci_gate_worker_env_step_check"',
    ),
    "tests/run_ci_sanity_gate.py": (
        '"seamgrim_ci_gate_worker_env_step_check"',
        "tests/run_seamgrim_ci_gate_worker_env_step_check.py",
    ),
    "tests/run_ci_sync_readiness_check.py": (
        '"seamgrim_ci_gate_worker_env_step_check"',
    ),
    "tests/run_ci_emit_artifacts_check.py": (
        '"seamgrim_ci_gate_worker_env_step_check"',
    ),
    "tests/run_ci_gate_report_index_check_selftest.py": (
        '"seamgrim_ci_gate_worker_env_step_check"',
        "tests/run_seamgrim_ci_gate_worker_env_step_check.py",
    ),
    "tests/run_ci_aggregate_gate_sync_diagnostics_check.py": (
        "seamgrim_ci_gate_worker_env_step_check",
        "tests/run_seamgrim_ci_gate_worker_env_step_check.py",
    ),
    "tests/run_ci_aggregate_gate_sanity_diagnostics_check.py": (
        "seamgrim_ci_gate_worker_env_step_check",
        "tests/run_seamgrim_ci_gate_worker_env_step_check.py",
    ),
    "tests/run_seamgrim_ci_gate.py": (
        "DDN_SEAMGRIM_CI_GATE_MAX_WORKERS",
        "DDN_SEAMGRIM_CI_GATE_FAMILY_MAX_WORKERS",
        "default_workers = 14",
        '_read_positive_int_env("DDN_SEAMGRIM_CI_GATE_FAMILY_MAX_WORKERS", 10)',
    ),
    "tests/run_ci_aggregate_gate.py": (
        "DDN_SEAMGRIM_CI_GATE_MAX_WORKERS",
        "DDN_SEAMGRIM_CI_GATE_FAMILY_MAX_WORKERS",
        "check_seamgrim_ci_gate_worker_env_step",
        "tests/run_seamgrim_ci_gate_worker_env_step_check.py",
        'or "14"',
        'or "10"',
        "env_extra=seamgrim_worker_env",
    ),
    ".github/workflows/seamgrim-ci.yml": (
        'DDN_SEAMGRIM_CI_GATE_MAX_WORKERS: "14"',
        'DDN_SEAMGRIM_CI_GATE_FAMILY_MAX_WORKERS: "10"',
    ),
}


def collect_missing_tokens(root: Path) -> list[str]:
    issues: list[str] = []
    for rel_path, required_tokens in FILE_TOKEN_REQUIREMENTS.items():
        target = root / rel_path
        if not target.exists():
            issues.append(f"{rel_path}::missing_file")
            continue
        text = target.read_text(encoding="utf-8")
        for token in required_tokens:
            if token not in text:
                issues.append(f"{rel_path}::token_missing::{token}")
    return issues


def build_report(*, root: Path, missing: list[str]) -> dict[str, object]:
    return {
        "schema": "ddn.seamgrim_ci_gate_worker_env_step_check.v1",
        "status": "pass" if not missing else "fail",
        "ok": not missing,
        "code": "OK" if not missing else "E_SEAMGRIM_CI_GATE_WORKER_ENV_STEP_CONTRACT_MISSING",
        "checked_files": len(FILE_TOKEN_REQUIREMENTS),
        "missing_count": len(missing),
        "missing": missing,
        "repo_root": str(root),
    }


def write_report(path: Path, report: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Check seamgrim ci gate worker env pin wiring across CI chain"
    )
    parser.add_argument(
        "--repo-root",
        default=str(Path(__file__).resolve().parent.parent),
        help="repository root path",
    )
    parser.add_argument(
        "--report-out",
        help="optional JSON report output path",
    )
    args = parser.parse_args()

    root = Path(args.repo_root).resolve()
    missing = collect_missing_tokens(root)
    report = build_report(root=root, missing=missing)
    if args.report_out:
        write_report(Path(args.report_out), report)

    if missing:
        print("seamgrim ci gate worker env step check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1

    print("seamgrim ci gate worker env step check ok")
    print(f"checked_files={len(FILE_TOKEN_REQUIREMENTS)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
