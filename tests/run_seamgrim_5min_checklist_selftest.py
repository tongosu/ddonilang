#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    script = root / "tests" / "run_seamgrim_5min_checklist.py"

    with tempfile.TemporaryDirectory(prefix="seamgrim-5min-checklist-selftest-") as td:
        temp_root = Path(td)
        pass_report = temp_root / "runtime_pass.detjson"
        fail_report = temp_root / "runtime_fail.detjson"
        summary_json = temp_root / "summary.detjson"
        summary_md = temp_root / "summary.md"

        pass_report.write_text(
            json.dumps(
                {
                    "schema": "seamgrim.runtime_5min_check.v1",
                    "ok": True,
                    "steps": [
                        {"name": "ddn_exec_server_check", "ok": True, "elapsed_ms": 10, "returncode": 0},
                        {"name": "browse_selection_report", "ok": True, "elapsed_ms": 5, "returncode": 0},
                    ],
                },
                ensure_ascii=False,
            )
            + "\n",
            encoding="utf-8",
        )
        fail_report.write_text(
            json.dumps(
                {
                    "schema": "seamgrim.runtime_5min_check.v1",
                    "ok": False,
                    "steps": [
                        {"name": "ddn_exec_server_check", "ok": False, "elapsed_ms": 12, "returncode": 1},
                    ],
                },
                ensure_ascii=False,
            )
            + "\n",
            encoding="utf-8",
        )

        pass_proc = run(
            [
                py,
                str(script),
                "--from-runtime-report",
                str(pass_report),
                "--json-out",
                str(summary_json),
                "--markdown-out",
                str(summary_md),
            ],
            cwd=root,
        )
        if pass_proc.returncode != 0:
            print("check=seamgrim_5min_checklist_selftest detail=pass_report_should_pass")
            if pass_proc.stdout.strip():
                print(pass_proc.stdout.strip())
            if pass_proc.stderr.strip():
                print(pass_proc.stderr.strip())
            return 1
        if not summary_json.exists() or not summary_md.exists():
            print("check=seamgrim_5min_checklist_selftest detail=summary_outputs_missing")
            return 1

        fail_proc = run([py, str(script), "--from-runtime-report", str(fail_report)], cwd=root)
        if fail_proc.returncode == 0:
            print("check=seamgrim_5min_checklist_selftest detail=fail_report_should_fail")
            return 1
        merged = "\n".join([fail_proc.stdout or "", fail_proc.stderr or ""])
        if "seamgrim 5min checklist failed" not in merged:
            print("check=seamgrim_5min_checklist_selftest detail=missing_failed_footer")
            if fail_proc.stdout.strip():
                print(fail_proc.stdout.strip())
            if fail_proc.stderr.strip():
                print(fail_proc.stderr.strip())
            return 1

    print("seamgrim 5min checklist selftest ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

