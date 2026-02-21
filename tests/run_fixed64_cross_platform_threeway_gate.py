#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SCHEMA = "ddn.fixed64.cross_platform_threeway_gate.v1"


def write_report(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="fixed64 cross-platform 3way(windows/linux/darwin) gate check"
    )
    parser.add_argument("--python", default=sys.executable, help="python executable path")
    parser.add_argument(
        "--report-out",
        default="",
        help="detjson 출력 경로(기본: build/reports/fixed64_cross_platform_threeway_gate.detjson)",
    )
    parser.add_argument(
        "--windows-report",
        default="build/reports/fixed64_cross_platform_probe_windows.detjson",
        help="windows probe report path",
    )
    parser.add_argument(
        "--linux-report",
        default="build/reports/fixed64_cross_platform_probe_linux.detjson",
        help="linux probe report path",
    )
    parser.add_argument(
        "--darwin-report",
        default="build/reports/fixed64_cross_platform_probe_darwin.detjson",
        help="darwin probe report path",
    )
    parser.add_argument(
        "--require-darwin",
        action="store_true",
        help="darwin report 누락을 실패로 처리",
    )
    args = parser.parse_args()

    report_dir = ROOT / "build" / "reports"
    report_out = (
        Path(args.report_out).resolve()
        if args.report_out.strip()
        else report_dir / "fixed64_cross_platform_threeway_gate.detjson"
    )
    windows_report = Path(args.windows_report).resolve()
    linux_report = Path(args.linux_report).resolve()
    darwin_report = Path(args.darwin_report).resolve()

    payload: dict[str, object] = {
        "schema": SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": False,
        "status": "fail",
        "reason": "-",
        "reports": {
            "windows": str(windows_report),
            "linux": str(linux_report),
            "darwin": str(darwin_report),
        },
    }

    missing_required: list[str] = []
    if not windows_report.exists():
        missing_required.append("windows")
    if not linux_report.exists():
        missing_required.append("linux")
    if missing_required:
        payload["reason"] = f"missing required reports: {','.join(missing_required)}"
        write_report(report_out, payload)
        print(f"[fixed64-3way-gate] failed report={report_out}", file=sys.stderr)
        print(f" - missing required reports: {','.join(missing_required)}", file=sys.stderr)
        return 1

    if not darwin_report.exists():
        if args.require_darwin:
            payload["reason"] = "darwin report missing"
            write_report(report_out, payload)
            print(f"[fixed64-3way-gate] failed report={report_out}", file=sys.stderr)
            print(" - darwin report missing", file=sys.stderr)
            return 1
        payload["ok"] = True
        payload["status"] = "pending_darwin"
        payload["reason"] = "darwin report missing"
        write_report(report_out, payload)
        print(f"[fixed64-3way-gate] pending darwin report={darwin_report}")
        return 0

    cmd = [
        args.python,
        "tests/run_fixed64_cross_platform_matrix_check.py",
        "--report",
        str(windows_report),
        "--report",
        str(linux_report),
        "--report",
        str(darwin_report),
        "--require-systems",
        "windows,linux,darwin",
    ]
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    payload["cmd"] = cmd
    payload["returncode"] = int(proc.returncode)
    payload["stdout"] = (proc.stdout or "").strip().splitlines()
    payload["stderr"] = (proc.stderr or "").strip().splitlines()
    if proc.returncode != 0:
        payload["reason"] = "matrix check failed"
        write_report(report_out, payload)
        print(f"[fixed64-3way-gate] failed report={report_out}", file=sys.stderr)
        if proc.stdout:
            print(proc.stdout, end="" if proc.stdout.endswith("\n") else "\n", file=sys.stderr)
        if proc.stderr:
            print(proc.stderr, end="" if proc.stderr.endswith("\n") else "\n", file=sys.stderr)
        return int(proc.returncode)

    payload["ok"] = True
    payload["status"] = "pass_3way"
    payload["reason"] = "-"
    write_report(report_out, payload)
    print(f"[fixed64-3way-gate] ok report={report_out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
