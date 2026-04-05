#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str) -> int:
    print(f"[pack-evidence-tier-check-selftest] fail {msg}", file=sys.stderr)
    return 1


def run_case(cmd: list[str], *, cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def load_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    parser = argparse.ArgumentParser(description="Selftest for run_pack_evidence_tier_check.py")
    parser.add_argument(
        "--repo-root",
        default=str(Path(__file__).resolve().parent.parent),
        help="repository root path",
    )
    args = parser.parse_args()

    root = Path(args.repo_root).resolve()
    check_script = root / "tests" / "run_pack_evidence_tier_check.py"
    if not check_script.exists():
        return fail(f"missing script: {check_script}")

    with tempfile.TemporaryDirectory(prefix="pack_evidence_tier_check_selftest_") as temp_dir:
        report_out = Path(temp_dir) / "pack_evidence_tier_runner_check.detjson"
        ok_cmd = [
            sys.executable,
            str(check_script),
            "--repo-root",
            str(root),
            "--max-docs-issues",
            "999999",
            "--expected-repo-issues",
            "0",
            "--report-out",
            str(report_out),
        ]
        ok_proc = run_case(ok_cmd, cwd=root)
        if ok_proc.returncode != 0:
            return fail(f"ok case failed rc={ok_proc.returncode} out={ok_proc.stdout} err={ok_proc.stderr}")
        if "check=pack_evidence_tier_check detail=ok:" not in ok_proc.stdout:
            return fail(f"ok marker missing out={ok_proc.stdout} err={ok_proc.stderr}")
        if not report_out.exists():
            return fail(f"ok case report missing path={report_out}")

        report_doc = load_json(report_out)
        if report_doc.get("schema") != "ddn.pack_evidence_tier_runner_check.v1":
            return fail(f"report schema mismatch doc={report_doc}")
        if bool(report_doc.get("ok", False)) is not True:
            return fail(f"report ok field must be true doc={report_doc}")
        if int(report_doc.get("max_docs_issues", -1)) != 999999:
            return fail(f"report max_docs_issues mismatch doc={report_doc}")
        if int(report_doc.get("expected_repo_issues", -1)) != 0:
            return fail(f"report expected_repo_issues mismatch doc={report_doc}")

        docs_profile = report_doc.get("docs_profile")
        repo_profile = report_doc.get("repo_profile")
        if not isinstance(docs_profile, dict) or not isinstance(repo_profile, dict):
            return fail(f"report profile fields must be dict doc={report_doc}")
        if docs_profile.get("name") != "docs_ssot_rep10":
            return fail(f"docs_profile name mismatch doc={report_doc}")
        if repo_profile.get("name") != "repo_rep10":
            return fail(f"repo_profile name mismatch doc={report_doc}")
        if int(repo_profile.get("issue_count", -1)) != 0:
            return fail(f"repo_profile issue_count must be 0 doc={report_doc}")

        fail_docs_budget_cmd = [
            sys.executable,
            str(check_script),
            "--repo-root",
            str(root),
            "--max-docs-issues",
            "-1",
            "--expected-repo-issues",
            "0",
        ]
        fail_docs_budget_proc = run_case(fail_docs_budget_cmd, cwd=root)
        if fail_docs_budget_proc.returncode == 0:
            return fail("docs budget fail case must return non-zero")
        docs_budget_log = f"{fail_docs_budget_proc.stdout}\n{fail_docs_budget_proc.stderr}"
        if "docs_issue_budget_exceeded:" not in docs_budget_log:
            return fail(
                f"docs budget fail marker missing out={fail_docs_budget_proc.stdout} err={fail_docs_budget_proc.stderr}"
            )

        fail_repo_expect_cmd = [
            sys.executable,
            str(check_script),
            "--repo-root",
            str(root),
            "--max-docs-issues",
            "999999",
            "--expected-repo-issues",
            "1",
        ]
        fail_repo_expect_proc = run_case(fail_repo_expect_cmd, cwd=root)
        if fail_repo_expect_proc.returncode == 0:
            return fail("repo expected-issues mismatch case must return non-zero")
        repo_expect_log = f"{fail_repo_expect_proc.stdout}\n{fail_repo_expect_proc.stderr}"
        if "repo_issue_count_unexpected:" not in repo_expect_log:
            return fail(
                "repo expected-issues marker missing "
                f"out={fail_repo_expect_proc.stdout} err={fail_repo_expect_proc.stderr}"
            )

    print("[pack-evidence-tier-check-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
