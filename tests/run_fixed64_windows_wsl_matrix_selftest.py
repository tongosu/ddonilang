#!/usr/bin/env python
from __future__ import annotations

import json
import platform
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SCHEMA = "ddn.fixed64.windows_wsl_matrix_check.v1"


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def main() -> int:
    py = sys.executable
    report = ROOT / "build" / "reports" / "fixed64_windows_wsl_matrix_selftest.detjson"
    cmd = [
        py,
        "tests/run_fixed64_windows_wsl_matrix_check.py",
        "--report-out",
        str(report),
    ]
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        if proc.stdout:
            print(proc.stdout, end="")
        if proc.stderr:
            print(proc.stderr, end="", file=sys.stderr)
        print("[fixed64-win-wsl-selftest] runner failed", file=sys.stderr)
        return int(proc.returncode)

    payload = load_json(report)
    if payload is None:
        print(f"[fixed64-win-wsl-selftest] invalid report: {report}", file=sys.stderr)
        return 1
    if str(payload.get("schema", "")) != SCHEMA:
        print("[fixed64-win-wsl-selftest] schema mismatch", file=sys.stderr)
        return 1
    if not bool(payload.get("ok", False)):
        print("[fixed64-win-wsl-selftest] report ok=false", file=sys.stderr)
        return 1

    status = str(payload.get("status", ""))
    allowed = {"skip_non_windows", "pass_windows_only", "pass_pending_darwin", "pass_3way"}
    if status not in allowed:
        print(f"[fixed64-win-wsl-selftest] unexpected status: {status}", file=sys.stderr)
        return 1

    host = platform.system().lower()
    if host == "windows" and status == "skip_non_windows":
        print("[fixed64-win-wsl-selftest] unexpected skip on windows", file=sys.stderr)
        return 1

    if host == "windows":
        missing_report = ROOT / "build" / "reports" / "fixed64_cross_platform_probe_darwin_missing_for_selftest.detjson"
        missing_check_report = ROOT / "build" / "reports" / "fixed64_windows_wsl_matrix_selftest_missing_darwin.detjson"
        negative_cmd = [
            py,
            "tests/run_fixed64_windows_wsl_matrix_check.py",
            "--report-out",
            str(missing_check_report),
            "--require-darwin",
            "--darwin-report",
            str(missing_report),
        ]
        negative = subprocess.run(
            negative_cmd,
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if negative.returncode == 0:
            print("[fixed64-win-wsl-selftest] expected failure for missing darwin report", file=sys.stderr)
            return 1
        negative_payload = load_json(missing_check_report)
        if not isinstance(negative_payload, dict):
            print("[fixed64-win-wsl-selftest] missing negative report payload", file=sys.stderr)
            return 1
        if bool(negative_payload.get("ok", True)):
            print("[fixed64-win-wsl-selftest] negative report ok should be false", file=sys.stderr)
            return 1

    print(f"[fixed64-win-wsl-selftest] ok status={status} report={report}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
