#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path


FILE_TOKEN_REQUIREMENTS = {
    "tests/run_seamgrim_ci_gate.py": (
        '"wasm_web_smoke_contract"',
        "tests/run_seamgrim_wasm_web_smoke_contract_pack_check.py",
        '"seamgrim_ci_gate_wasm_web_smoke_step_check"',
        "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check.py",
        '"seamgrim_ci_gate_wasm_web_smoke_step_check_selftest"',
        "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check_selftest.py",
    ),
    "tests/run_age3_completion_gate.py": (
        '"seamgrim_wasm_web_smoke_contract"',
        "tests/run_seamgrim_wasm_web_smoke_contract_pack_check.py",
        '"seamgrim_wasm_web_smoke_contract_pass"',
        '"seamgrim_wasm_web_step_check"',
        "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check.py",
        '"seamgrim_wasm_web_step_check_pass"',
        "--report-out",
        '"seamgrim_wasm_web_step_check_report_path"',
    ),
    "tests/run_age3_completion_gate_selftest.py": (
        '"seamgrim_wasm_web_step_check_pass"',
        '"seamgrim_wasm_web_step_check_report_path"',
        "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1",
    ),
    "tests/run_ci_sanity_gate.py": (
        '"seamgrim_ci_gate_wasm_web_smoke_step_check"',
        "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check.py",
        '"seamgrim_ci_gate_wasm_web_smoke_step_check_selftest"',
        "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check_selftest.py",
        "--verify-report",
        'completion_gate_reports["seamgrim_wasm_web_step_check_report"]',
    ),
    "tests/run_ci_sync_readiness_check.py": (
        '"seamgrim_ci_gate_wasm_web_smoke_step_check"',
        '"seamgrim_ci_gate_wasm_web_smoke_step_check_selftest"',
    ),
    "tests/run_ci_emit_artifacts_check.py": (
        '"seamgrim_ci_gate_wasm_web_smoke_step_check"',
        '"seamgrim_ci_gate_wasm_web_smoke_step_check_selftest"',
    ),
    "tests/run_ci_aggregate_gate.py": (
        '"seamgrim_ci_gate_wasm_web_smoke_step_check"',
        '"seamgrim_ci_gate_wasm_web_smoke_step_check_selftest"',
        "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check.py",
        "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check_selftest.py",
        "check_seamgrim_ci_gate_wasm_web_smoke_step_selftest",
        "--verify-report",
        "seamgrim_wasm_web_step_check_report",
    ),
    "tests/run_ci_gate_report_index_check.py": (
        "from _ci_seamgrim_step_contract import SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS",
        "PROFILE_REQUIRED_STEPS_SEAMGRIM = SEAMGRIM_PROFILE_REQUIRED_STEP_CONTRACT_STEPS",
    ),
    "tests/run_ci_gate_report_index_check_selftest.py": (
        '"seamgrim_ci_gate_wasm_web_smoke_step_check"',
        '"seamgrim_ci_gate_wasm_web_smoke_step_check_selftest"',
        "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check.py",
        "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check_selftest.py",
    ),
    "tests/run_ci_emit_artifacts_check_selftest.py": (
        '"seamgrim_ci_gate_wasm_web_smoke_step_check"',
        '"seamgrim_ci_gate_wasm_web_smoke_step_check_selftest"',
    ),
    "tests/run_ci_sync_readiness_check_selftest.py": (
        '"seamgrim_ci_gate_wasm_web_smoke_step_check_selftest"',
        "filtered_steps_should_remove_seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
        "validate_missing_wasm_web_smoke_selftest_msg_should_mention_step",
    ),
    "tests/run_ci_sync_readiness_report_check_selftest.py": (
        '"ci_sanity_seamgrim_wasm_web_step_check_ok"',
        '"ci_sanity_seamgrim_wasm_web_step_check_report_path"',
        '"ci_sanity_seamgrim_wasm_web_step_check_schema"',
    ),
    "tests/run_ci_sanity_gate_diagnostics_check.py": (
        "seamgrim_ci_gate_wasm_web_smoke_step_check",
        "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check.py",
        "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
        "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check_selftest.py",
    ),
    "tests/run_ci_sync_readiness_diagnostics_check.py": (
        "seamgrim_ci_gate_wasm_web_smoke_step_check",
        "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
    ),
    "tests/run_ci_emit_artifacts_sanity_contract_check.py": (
        '"seamgrim_ci_gate_wasm_web_smoke_step_check"',
        '"seamgrim_ci_gate_wasm_web_smoke_step_check_selftest"',
    ),
    "tests/run_ci_gate_summary_report_check.py": (
        '"ci_sanity_seamgrim_wasm_web_step_check_ok"',
        '"ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_ok"',
        "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1",
    ),
    "tests/run_ci_gate_summary_report_check_selftest.py": (
        '"ci_sanity_seamgrim_wasm_web_step_check_ok"',
        '"ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_ok"',
        "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1",
    ),
    "tests/run_ci_sync_readiness_report_check.py": (
        'SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA = "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1"',
        '"ci_sanity_seamgrim_wasm_web_step_check_ok"',
    ),
    "tests/run_ci_gate_report_index_diagnostics_check.py": (
        "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
        "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check_selftest.py",
    ),
    "tests/run_ci_aggregate_gate_sanity_diagnostics_check.py": (
        "seamgrim_ci_gate_wasm_web_smoke_step_check",
        "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check.py",
        "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
        "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check_selftest.py",
    ),
    "tests/run_ci_aggregate_gate_sync_diagnostics_check.py": (
        "seamgrim_ci_gate_wasm_web_smoke_step_check",
        "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check.py",
        "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
        "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check_selftest.py",
    ),
    "tests/_ci_aggregate_contract_only_lib.py": (
        '"seamgrim_ci_gate_wasm_web_smoke_step_check"',
        '"seamgrim_ci_gate_wasm_web_smoke_step_check_selftest"',
    ),
}

CONTRACT_MARKERS = (
    "wasm_web_smoke_contract",
    "seamgrim_wasm_web_smoke_contract",
    "seamgrim_wasm_web_smoke_contract_pass",
    "run_seamgrim_wasm_web_smoke_contract_pack_check.py",
    "seamgrim_ci_gate_wasm_web_smoke_step_check",
    "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
    "run_seamgrim_ci_gate_wasm_web_smoke_step_check.py",
    "run_seamgrim_ci_gate_wasm_web_smoke_step_check_selftest.py",
)

AUTO_EXEMPT_FILES = {
    "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check.py",
    "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check_selftest.py",
}


def is_contract_scan_candidate(rel_path: str) -> bool:
    return rel_path.startswith("tests/run_ci_") or rel_path in {
        "tests/run_seamgrim_ci_gate.py",
        "tests/run_age3_completion_gate.py",
    }


def collect_untracked_contract_files(root: Path) -> list[str]:
    issues: list[str] = []
    tests_dir = root / "tests"
    if not tests_dir.exists():
        return issues
    for target in sorted(tests_dir.rglob("*.py")):
        rel_path = target.relative_to(root).as_posix()
        if not is_contract_scan_candidate(rel_path):
            continue
        if rel_path in FILE_TOKEN_REQUIREMENTS or rel_path in AUTO_EXEMPT_FILES:
            continue
        text = target.read_text(encoding="utf-8")
        if any(marker in text for marker in CONTRACT_MARKERS):
            issues.append(f"{rel_path}::untracked_contract_file")
    return issues


def collect_missing_tokens(root: Path) -> list[str]:
    issues: list[str] = []
    issues.extend(collect_untracked_contract_files(root))
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
        "schema": "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1",
        "status": "pass" if not missing else "fail",
        "ok": not missing,
        "code": "OK" if not missing else "E_SEAMGRIM_WASM_WEB_STEP_CONTRACT_MISSING",
        "checked_files": len(FILE_TOKEN_REQUIREMENTS),
        "missing_count": len(missing),
        "missing": missing,
        "repo_root": str(root),
    }


def write_report(path: Path, report: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="Check seamgrim wasm/web smoke wiring across CI chain")
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
        print("seamgrim ci gate wasm/web smoke step check failed:")
        for token in missing[:12]:
            print(f" - missing token: {token}")
        return 1

    print("seamgrim ci gate wasm/web smoke step check ok")
    print(f"checked_files={len(FILE_TOKEN_REQUIREMENTS)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
