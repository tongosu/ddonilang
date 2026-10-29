#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path


EXPECTED_SCHEMA = "ddn.pack_evidence_tier_runner_check.v1"


def fail(detail: str) -> int:
    print(f"check=pack_evidence_tier_report detail={detail}")
    return 1


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Validate pack evidence tier runner report contract")
    parser.add_argument(
        "--repo-root",
        default=str(Path(__file__).resolve().parent.parent),
        help="repository root path",
    )
    parser.add_argument(
        "--report-path",
        default="build/reports/seamgrim_pack_evidence_tier_runner_check.detjson",
        help="runner report path",
    )
    parser.add_argument(
        "--max-docs-issues",
        type=int,
        default=10,
        help="maximum allowed docs profile issue_count",
    )
    parser.add_argument(
        "--expected-repo-issues",
        type=int,
        default=0,
        help="expected repo profile issue_count",
    )
    return parser.parse_args()


def resolve_report_path(root: Path, value: str) -> Path:
    path = Path(value)
    if not path.is_absolute():
        path = root / path
    return path


def main() -> int:
    args = parse_args()
    root = Path(args.repo_root).resolve()
    report_path = resolve_report_path(root, str(args.report_path))

    if not report_path.exists():
        return fail(f"report_missing:path={report_path}")

    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except Exception as exc:
        return fail(f"report_parse_failed:{exc}")

    if str(report.get("schema", "")).strip() != EXPECTED_SCHEMA:
        return fail(f"schema_mismatch:expected={EXPECTED_SCHEMA}:actual={report.get('schema')}")
    if bool(report.get("ok", False)) is not True:
        return fail("report_not_ok")

    docs_profile = report.get("docs_profile")
    repo_profile = report.get("repo_profile")
    if not isinstance(docs_profile, dict) or not isinstance(repo_profile, dict):
        return fail("report_keys_missing:profiles")

    if str(docs_profile.get("name", "")).strip() != "docs_ssot_rep10":
        return fail("report_keys_missing:docs_profile_name")
    if str(repo_profile.get("name", "")).strip() != "repo_rep10":
        return fail("report_keys_missing:repo_profile_name")

    try:
        docs_issue_count = int(docs_profile.get("issue_count", 0))
        repo_issue_count = int(repo_profile.get("issue_count", 0))
    except Exception as exc:
        return fail(f"report_keys_missing:issue_count:{exc}")

    max_docs_issues = int(args.max_docs_issues)
    expected_repo_issues = int(args.expected_repo_issues)
    if docs_issue_count > max_docs_issues:
        return fail(f"docs_issue_budget_exceeded:issue_count={docs_issue_count}:max={max_docs_issues}")
    if repo_issue_count != expected_repo_issues:
        return fail(f"repo_issue_count_unexpected:actual={repo_issue_count}:expected={expected_repo_issues}")

    print(
        "check=pack_evidence_tier_report detail="
        f"ok:docs_profile_issues={docs_issue_count}:repo_profile_issues={repo_issue_count}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
