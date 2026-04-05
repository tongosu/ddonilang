#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path


FILE_TOKEN_REQUIREMENTS: dict[str, tuple[str, ...]] = {
    "tests/run_seamgrim_ci_gate.py": (
        '"lesson_migration_autofix"',
        "tests/run_seamgrim_lesson_migration_autofix_check.py",
        '"seamgrim_ci_gate_lesson_migration_autofix_step_check"',
        "tests/run_seamgrim_ci_gate_lesson_migration_autofix_step_check.py",
        '"seamgrim_ci_gate_lesson_migration_autofix_step_check_selftest"',
        "tests/run_seamgrim_ci_gate_lesson_migration_autofix_step_check_selftest.py",
    ),
    "tests/_seamgrim_ci_diag_lib.py": (
        'elif name == "lesson_migration_autofix":',
        "lesson_migration_autofix_tool_failed",
        "lesson_migration_autofix_detail",
        'elif name == "seamgrim_ci_gate_lesson_migration_autofix_step_check":',
        "lesson_migration_autofix_step_check_failed",
        "lesson_migration_autofix_step_check_token_missing",
        'elif name == "seamgrim_ci_gate_lesson_migration_autofix_step_check_selftest":',
        "lesson_migration_autofix_step_selftest_failed",
    ),
    "tests/run_seamgrim_ci_gate_diagnostics_check.py": (
        '"lesson_migration_autofix",',
        "lesson_migration_autofix_tool_failed",
        '"seamgrim_ci_gate_lesson_migration_autofix_step_check",',
        "lesson_migration_autofix_step_check_failed",
        '"seamgrim_ci_gate_lesson_migration_autofix_step_check_selftest",',
        "lesson_migration_autofix_step_selftest_failed",
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
        "schema": "ddn.seamgrim_ci_gate_lesson_migration_autofix_step_check.v1",
        "status": "pass" if not missing else "fail",
        "ok": not missing,
        "code": "OK" if not missing else "E_SEAMGRIM_LESSON_MIGRATION_AUTOFIX_STEP_CONTRACT_MISSING",
        "checked_files": len(FILE_TOKEN_REQUIREMENTS),
        "missing_count": len(missing),
        "missing": missing,
        "repo_root": str(root),
    }


def write_report(path: Path, report: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="Check lesson_migration_autofix wiring across seamgrim ci gate flow")
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
        print("seamgrim ci gate lesson migration autofix step check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1

    print("seamgrim ci gate lesson migration autofix step check ok")
    print(f"checked_files={len(FILE_TOKEN_REQUIREMENTS)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
