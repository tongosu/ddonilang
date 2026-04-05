#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path


def run_cmd(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )


def fail(detail: str) -> int:
    print(f"check=pack_evidence_tier_closure_claim detail={detail}")
    return 1


def write_readme(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body, encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Check closure-claim lint mode for pack evidence tier tool"
    )
    parser.add_argument("--repo-root", default=".", help="repository root path")
    args = parser.parse_args()

    root = Path(args.repo_root).resolve()
    tool = root / "tools" / "scripts" / "check_pack_evidence_tier.py"
    if not tool.exists():
        return fail(f"tool_missing:{tool}")

    with tempfile.TemporaryDirectory(prefix="pack_evidence_closure_claim_") as temp_dir:
        temp_root = Path(temp_dir)
        pack_root = temp_root / "packs"

        write_readme(
            pack_root / "ok_golden" / "README.md",
            "# ok_golden\n\nevidence_tier: golden_closed\ncurrent_status: active\nclosure_claim: allowed\n",
        )
        write_readme(
            pack_root / "bad_closure" / "README.md",
            "# bad_closure\n\nevidence_tier: runner_fill\ncurrent_status: active\nclosure_claim: allowed\n",
        )
        write_readme(
            pack_root / "missing_fields" / "README.md",
            "# missing_fields\n\nevidence_tier: docs_first\n",
        )

        fail_report = temp_root / "fail_report.detjson"
        fail_cmd = [
            sys.executable,
            str(tool),
            "--repo-root",
            str(root),
            "--pack-root",
            str(pack_root),
            "--pack",
            "ok_golden",
            "--pack",
            "bad_closure",
            "--pack",
            "missing_fields",
            "--require-status",
            "--require-closure-claim",
            "--enforce-closure-tier",
            "--strict",
            "--report",
            str(fail_report),
        ]
        fail_proc = run_cmd(fail_cmd, cwd=root)
        if fail_proc.returncode == 0:
            return fail("strict_fail_case_must_fail")
        fail_log = f"{fail_proc.stdout}\n{fail_proc.stderr}"
        if "strict_failed" not in fail_log:
            return fail(f"strict_marker_missing:{fail_log}")
        if not fail_report.exists():
            return fail("fail_report_missing")
        fail_doc = json.loads(fail_report.read_text(encoding="utf-8"))
        if int(fail_doc.get("issue_count", 0)) <= 0:
            return fail(f"issue_count_not_positive:{fail_doc}")
        if "closure_tier_violation" not in fail_doc:
            return fail("closure_tier_violation_key_missing")

        ok_report = temp_root / "ok_report.detjson"
        ok_cmd = [
            sys.executable,
            str(tool),
            "--repo-root",
            str(root),
            "--pack-root",
            str(pack_root),
            "--pack",
            "ok_golden",
            "--require-status",
            "--require-closure-claim",
            "--enforce-closure-tier",
            "--strict",
            "--report",
            str(ok_report),
        ]
        ok_proc = run_cmd(ok_cmd, cwd=root)
        if ok_proc.returncode != 0:
            return fail(f"strict_ok_case_failed:{ok_proc.stdout}::{ok_proc.stderr}")
        if not ok_report.exists():
            return fail("ok_report_missing")
        ok_doc = json.loads(ok_report.read_text(encoding="utf-8"))
        if ok_doc.get("schema") != "ddn.pack_evidence_tier_check.v1":
            return fail(f"schema_mismatch:{ok_doc.get('schema')}")
        if int(ok_doc.get("issue_count", -1)) != 0:
            return fail(f"strict_ok_issue_count_nonzero:{ok_doc}")

    print("check=pack_evidence_tier_closure_claim detail=ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
