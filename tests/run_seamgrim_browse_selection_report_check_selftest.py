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
    check_script = root / "tests" / "run_seamgrim_browse_selection_report_check.py"

    with tempfile.TemporaryDirectory(prefix="browse-report-check-selftest-") as td:
        temp_root = Path(td)
        clean_report = temp_root / "clean.detjson"
        noisy_report = temp_root / "noisy.detjson"
        warn_report = temp_root / "warn.detjson"

        clean_report.write_text(
            json.dumps(
                {
                    "schema": "seamgrim.browse_selection_flow_check.v1",
                    "ok": True,
                    "stdout": "seamgrim browse selection runner ok",
                    "stderr": "",
                },
                ensure_ascii=False,
            )
            + "\n",
            encoding="utf-8",
        )

        noisy_report.write_text(
            json.dumps(
                {
                    "schema": "seamgrim.browse_selection_flow_check.v1",
                    "ok": True,
                    "stdout": "GET /build/reports/seamgrim_lesson_inventory.json 404 (Not Found)",
                    "stderr": "",
                },
                ensure_ascii=False,
            )
            + "\n",
            encoding="utf-8",
        )
        warn_report.write_text(
            json.dumps(
                {
                    "schema": "seamgrim.browse_selection_flow_check.v1",
                    "ok": True,
                    "stdout": "",
                    "stderr": "(node:1) [MODULE_TYPELESS_PACKAGE_JSON] Warning: reparsing as ES module",
                },
                ensure_ascii=False,
            )
            + "\n",
            encoding="utf-8",
        )

        pass_proc = run([py, str(check_script), "--report", str(clean_report)], cwd=root)
        if pass_proc.returncode != 0:
            print("check=browse_selection_report_selftest detail=clean_report_should_pass")
            if pass_proc.stdout.strip():
                print(pass_proc.stdout.strip())
            if pass_proc.stderr.strip():
                print(pass_proc.stderr.strip())
            return 1

        fail_proc = run([py, str(check_script), "--report", str(noisy_report)], cwd=root)
        if fail_proc.returncode == 0:
            print("check=browse_selection_report_selftest detail=noisy_report_should_fail")
            return 1
        merged = "\n".join([fail_proc.stdout or "", fail_proc.stderr or ""])
        if "check=browse_selection_report_forbidden_pattern" not in merged:
            print("check=browse_selection_report_selftest detail=forbidden_pattern_error_missing")
            if fail_proc.stdout.strip():
                print(fail_proc.stdout.strip())
            if fail_proc.stderr.strip():
                print(fail_proc.stderr.strip())
            return 1

        warn_proc = run([py, str(check_script), "--report", str(warn_report)], cwd=root)
        if warn_proc.returncode == 0:
            print("check=browse_selection_report_selftest detail=warn_report_should_fail")
            return 1
        warn_merged = "\n".join([warn_proc.stdout or "", warn_proc.stderr or ""])
        if "check=browse_selection_report_forbidden_pattern" not in warn_merged:
            print("check=browse_selection_report_selftest detail=warn_forbidden_pattern_error_missing")
            if warn_proc.stdout.strip():
                print(warn_proc.stdout.strip())
            if warn_proc.stderr.strip():
                print(warn_proc.stderr.strip())
            return 1

        allow_proc = run(
            [py, str(check_script), "--report", str(noisy_report), "--allow-forbidden-pattern"],
            cwd=root,
        )
        if allow_proc.returncode != 0:
            print("check=browse_selection_report_selftest detail=allow_forbidden_should_pass")
            if allow_proc.stdout.strip():
                print(allow_proc.stdout.strip())
            if allow_proc.stderr.strip():
                print(allow_proc.stderr.strip())
            return 1

    print("seamgrim browse selection report check selftest ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
