#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description="Run pack evidence tier check tool contract")
    parser.add_argument("--repo-root", default=".", help="repository root path")
    parser.add_argument(
        "--report-out",
        default="",
        help="optional output path for runner contract report json",
    )
    parser.add_argument(
        "--max-docs-issues",
        type=int,
        default=10,
        help="maximum allowed docs_ssot_rep10 issue_count",
    )
    parser.add_argument(
        "--expected-repo-issues",
        type=int,
        default=0,
        help="expected repo_rep10 issue_count under strict mode",
    )
    args = parser.parse_args()

    root = Path(args.repo_root).resolve()
    tool = root / "tools" / "scripts" / "check_pack_evidence_tier.py"
    if not tool.exists():
        print(f"check=pack_evidence_tier_check detail=tool_missing:{tool}")
        return 1

    with tempfile.TemporaryDirectory(prefix="pack_evidence_tier_check_") as temp_dir:
        docs_report_path = Path(temp_dir) / "pack_evidence_tier_docs.detjson"
        docs_fix_plan_path = Path(temp_dir) / "pack_evidence_tier_docs_fix_plan.md"
        repo_report_path = Path(temp_dir) / "pack_evidence_tier_repo.detjson"
        support_report_path = Path(temp_dir) / "pack_evidence_tier_support.detjson"
        product_report_path = Path(temp_dir) / "pack_evidence_tier_product.detjson"

        def run_check(
            report_path: Path,
            profile: str,
            strict: bool,
            fix_plan_path: Path | None = None,
        ) -> tuple[int, str, str]:
            cmd = [
                sys.executable,
                str(tool),
                "--repo-root",
                str(root),
                "--profile",
                profile,
                "--report",
                str(report_path),
            ]
            if fix_plan_path is not None:
                cmd.extend(["--fix-plan", str(fix_plan_path)])
            if strict:
                cmd.append("--strict")
            completed = subprocess.run(
                cmd,
                cwd=root,
                capture_output=True,
                text=True,
                check=False,
            )
            return completed.returncode, completed.stdout, completed.stderr

        docs_rc, docs_out, docs_err = run_check(
            docs_report_path,
            "docs_ssot_rep10",
            strict=False,
            fix_plan_path=docs_fix_plan_path,
        )
        if docs_rc != 0:
            detail = docs_err.strip() or docs_out.strip() or "docs_profile_failed"
            print(f"check=pack_evidence_tier_check detail=docs_profile_failed:{detail}")
            return 1

        repo_rc, repo_out, repo_err = run_check(repo_report_path, "repo_rep10", strict=True)
        if repo_rc != 0:
            detail = repo_err.strip() or repo_out.strip() or "repo_profile_strict_failed"
            print(f"check=pack_evidence_tier_check detail=repo_profile_strict_failed:{detail}")
            return 1
        support_rc, support_out, support_err = run_check(support_report_path, "repo_current_line_support", strict=True)
        if support_rc != 0:
            detail = support_err.strip() or support_out.strip() or "support_profile_strict_failed"
            print(f"check=pack_evidence_tier_check detail=support_profile_strict_failed:{detail}")
            return 1
        product_rc, product_out, product_err = run_check(product_report_path, "repo_current_line_product", strict=True)
        if product_rc != 0:
            detail = product_err.strip() or product_out.strip() or "product_profile_strict_failed"
            print(f"check=pack_evidence_tier_check detail=product_profile_strict_failed:{detail}")
            return 1

        if not docs_report_path.exists() or not repo_report_path.exists() or not support_report_path.exists() or not product_report_path.exists():
            print("check=pack_evidence_tier_check detail=report_missing")
            return 1
        if not docs_fix_plan_path.exists():
            print("check=pack_evidence_tier_check detail=fix_plan_missing")
            return 1

        try:
            docs_report = json.loads(docs_report_path.read_text(encoding="utf-8"))
            repo_report = json.loads(repo_report_path.read_text(encoding="utf-8"))
            support_report = json.loads(support_report_path.read_text(encoding="utf-8"))
            product_report = json.loads(product_report_path.read_text(encoding="utf-8"))
        except Exception as exc:  # pragma: no cover - defensive parse guard
            print(f"check=pack_evidence_tier_check detail=report_parse_failed:{exc}")
            return 1

        for key_report in (docs_report, repo_report, support_report, product_report):
            if key_report.get("schema") != "ddn.pack_evidence_tier_check.v1":
                print("check=pack_evidence_tier_check detail=schema_mismatch")
                return 1
            if "total" not in key_report or "issue_count" not in key_report or "ok_count" not in key_report:
                print("check=pack_evidence_tier_check detail=report_keys_missing")
                return 1
            if "suggested_fixes" not in key_report:
                print("check=pack_evidence_tier_check detail=suggested_fixes_missing")
                return 1

        docs_issue_count = int(docs_report.get("issue_count", 0))
        docs_max_issues = int(args.max_docs_issues)
        if docs_issue_count > docs_max_issues:
            print(
                "check=pack_evidence_tier_check detail="
                f"docs_issue_budget_exceeded:issue_count={docs_issue_count}:max={docs_max_issues}"
            )
            return 1

        repo_issue_count = int(repo_report.get("issue_count", 0))
        expected_repo_issues = int(args.expected_repo_issues)
        if repo_issue_count != expected_repo_issues:
            print(
                "check=pack_evidence_tier_check detail="
                f"repo_issue_count_unexpected:actual={repo_issue_count}:expected={expected_repo_issues}"
            )
            return 1
        support_issue_count = int(support_report.get("issue_count", 0))
        if support_issue_count != 0:
            print(
                "check=pack_evidence_tier_check detail="
                f"support_issue_count_unexpected:actual={support_issue_count}:expected=0"
            )
            return 1
        product_issue_count = int(product_report.get("issue_count", 0))
        if product_issue_count != 0:
            print(
                "check=pack_evidence_tier_check detail="
                f"product_issue_count_unexpected:actual={product_issue_count}:expected=0"
            )
            return 1

        if args.report_out:
            report_out = Path(args.report_out)
            if not report_out.is_absolute():
                report_out = root / report_out
            report_out.parent.mkdir(parents=True, exist_ok=True)
            contract_report = {
                "schema": "ddn.pack_evidence_tier_runner_check.v1",
                "status": "pass",
                "repo_root": str(root),
                "ok": True,
                "code": "OK",
                "msg": "-",
                "max_docs_issues": docs_max_issues,
                "expected_repo_issues": expected_repo_issues,
                "docs_profile": {
                    "name": "docs_ssot_rep10",
                    "issue_count": docs_issue_count,
                    "ok_count": int(docs_report.get("ok_count", 0)),
                    "total": int(docs_report.get("total", 0)),
                    "report_path": str(docs_report_path),
                    "fix_plan_path": str(docs_fix_plan_path),
                },
                "repo_profile": {
                    "name": "repo_rep10",
                    "issue_count": repo_issue_count,
                    "ok_count": int(repo_report.get("ok_count", 0)),
                    "total": int(repo_report.get("total", 0)),
                    "report_path": str(repo_report_path),
                },
                "support_profile": {
                    "name": "repo_current_line_support",
                    "issue_count": support_issue_count,
                    "ok_count": int(support_report.get("ok_count", 0)),
                    "total": int(support_report.get("total", 0)),
                    "report_path": str(support_report_path),
                },
                "product_profile": {
                    "name": "repo_current_line_product",
                    "issue_count": product_issue_count,
                    "ok_count": int(product_report.get("ok_count", 0)),
                    "total": int(product_report.get("total", 0)),
                    "report_path": str(product_report_path),
                },
            }
            report_out.write_text(json.dumps(contract_report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

        print(
            "check=pack_evidence_tier_check detail="
            f"ok:docs_profile_issues={docs_issue_count}:repo_profile_issues={repo_issue_count}:"
            f"support_profile_issues={support_issue_count}:product_profile_issues={product_issue_count}"
        )
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
