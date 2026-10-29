#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str) -> int:
    print(f"[seamgrim-runtime-fallback-surface-contract-selftest] fail: {msg}")
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    with tempfile.TemporaryDirectory(prefix="seamgrim_runtime_fallback_surface_contract_selftest_") as tmp:
        out_path = Path(tmp) / "runtime_fallback_surface.detjson"
        proc = subprocess.run(
            [
                sys.executable,
                "tests/run_seamgrim_runtime_fallback_surface_contract_check.py",
                "--out",
                str(out_path),
            ],
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if proc.returncode != 0:
            return fail(f"runner failed out={proc.stdout} err={proc.stderr}")
        if not out_path.exists():
            return fail("report output missing")
        report = json.loads(out_path.read_text(encoding="utf-8"))
        if report.get("schema") != "ddn.seamgrim_runtime_fallback_surface_contract.v1":
            return fail(f"schema mismatch: {report.get('schema')}")
        if report.get("ok") is not True:
            return fail("report not ok")
        if report.get("check_count") != 5:
            return fail(f"check_count mismatch: {report.get('check_count')}")
        if report.get("ok_count") != 5:
            return fail(f"ok_count mismatch: {report.get('ok_count')}")
        if report.get("metrics_report_schema") != "seamgrim.runtime_fallback_metrics.v1":
            return fail(f"metrics_report_schema mismatch: {report.get('metrics_report_schema')}")
        if int(report.get("metrics_total", 0)) <= 0:
            return fail(f"metrics_total invalid: {report.get('metrics_total')}")
        checks = report.get("checks")
        if not isinstance(checks, list) or len(checks) != 5:
            return fail("checks payload mismatch")
        for expected_name in (
            "lesson_path_fallback",
            "shape_fallback_mode",
            "motion_projectile_fallback",
            "runtime_fallback_metrics",
            "runtime_fallback_policy",
        ):
            row = next((item for item in checks if item.get("name") == expected_name), None)
            if row is None:
                return fail(f"missing check row: {expected_name}")
            if row.get("ok") is not True:
                return fail(f"check not ok: {row}")
    print("[seamgrim-runtime-fallback-surface-contract-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
