#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path


CHECK_SCRIPT = Path(__file__).resolve().parent / "run_pack_evidence_tier_report_check.py"


def fail(msg: str) -> int:
    print(f"[pack-evidence-tier-report-check-selftest] fail {msg}", file=sys.stderr)
    return 1


def run_check(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def write_json(path: Path, obj: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Selftest for pack evidence tier report checker")
    parser.add_argument(
        "--verify-report",
        default="",
        help="optional real report path to verify via checker",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    with tempfile.TemporaryDirectory(prefix="pack_evidence_tier_report_check_selftest_") as temp_dir:
        root = Path(temp_dir)
        report_ok = root / "ok.detjson"
        report_bad_docs = root / "bad_docs.detjson"
        report_bad_schema = root / "bad_schema.detjson"

        base_doc = {
            "schema": "ddn.pack_evidence_tier_runner_check.v1",
            "ok": True,
            "docs_profile": {"name": "docs_ssot_rep10", "issue_count": 10},
            "repo_profile": {"name": "repo_rep10", "issue_count": 0},
        }
        write_json(report_ok, base_doc)

        ok_cmd = [
            sys.executable,
            str(CHECK_SCRIPT),
            "--repo-root",
            str(root),
            "--report-path",
            str(report_ok),
            "--max-docs-issues",
            "10",
            "--expected-repo-issues",
            "0",
        ]
        ok_proc = run_check(ok_cmd)
        if ok_proc.returncode != 0:
            return fail(f"ok case failed rc={ok_proc.returncode} out={ok_proc.stdout} err={ok_proc.stderr}")
        if "check=pack_evidence_tier_report detail=ok:" not in ok_proc.stdout:
            return fail(f"ok marker missing out={ok_proc.stdout} err={ok_proc.stderr}")

        bad_docs_doc = dict(base_doc)
        bad_docs_doc["docs_profile"] = {"name": "docs_ssot_rep10", "issue_count": 11}
        write_json(report_bad_docs, bad_docs_doc)
        bad_docs_cmd = [
            sys.executable,
            str(CHECK_SCRIPT),
            "--repo-root",
            str(root),
            "--report-path",
            str(report_bad_docs),
            "--max-docs-issues",
            "10",
            "--expected-repo-issues",
            "0",
        ]
        bad_docs_proc = run_check(bad_docs_cmd)
        if bad_docs_proc.returncode == 0:
            return fail("docs budget fail case must return non-zero")
        bad_docs_log = f"{bad_docs_proc.stdout}\n{bad_docs_proc.stderr}"
        if "docs_issue_budget_exceeded:" not in bad_docs_log:
            return fail(f"docs budget marker missing out={bad_docs_proc.stdout} err={bad_docs_proc.stderr}")

        bad_schema_doc = dict(base_doc)
        bad_schema_doc["schema"] = "ddn.pack_evidence_tier_runner_check.v0"
        write_json(report_bad_schema, bad_schema_doc)
        bad_schema_cmd = [
            sys.executable,
            str(CHECK_SCRIPT),
            "--repo-root",
            str(root),
            "--report-path",
            str(report_bad_schema),
        ]
        bad_schema_proc = run_check(bad_schema_cmd)
        if bad_schema_proc.returncode == 0:
            return fail("schema mismatch case must return non-zero")
        bad_schema_log = f"{bad_schema_proc.stdout}\n{bad_schema_proc.stderr}"
        if "schema_mismatch:" not in bad_schema_log:
            return fail(f"schema marker missing out={bad_schema_proc.stdout} err={bad_schema_proc.stderr}")

    verify_report = str(args.verify_report or "").strip()
    if verify_report:
        verify_cmd = [
            sys.executable,
            str(CHECK_SCRIPT),
            "--report-path",
            verify_report,
            "--max-docs-issues",
            "10",
            "--expected-repo-issues",
            "0",
        ]
        verify_proc = run_check(verify_cmd)
        if verify_proc.returncode != 0:
            return fail(
                "verify-report failed "
                f"rc={verify_proc.returncode} out={verify_proc.stdout} err={verify_proc.stderr}"
            )

    print("[pack-evidence-tier-report-check-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
