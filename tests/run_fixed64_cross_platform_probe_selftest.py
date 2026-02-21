#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def run_step(cmd: list[str]) -> int:
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
    return int(proc.returncode)


def main() -> int:
    py = sys.executable
    report_dir = ROOT / "build" / "reports"
    report_dir.mkdir(parents=True, exist_ok=True)
    report_a = report_dir / "fixed64_cross_platform_probe_selftest_a.detjson"
    report_b = report_dir / "fixed64_cross_platform_probe_selftest_b.detjson"

    rc_a = run_step(
        [
            py,
            "tests/run_fixed64_cross_platform_probe.py",
            "--report-out",
            str(report_a),
        ]
    )
    if rc_a != 0:
        print("[fixed64-probe-selftest] probe A failed", file=sys.stderr)
        return rc_a

    rc_b = run_step(
        [
            py,
            "tests/run_fixed64_cross_platform_probe.py",
            "--report-out",
            str(report_b),
        ]
    )
    if rc_b != 0:
        print("[fixed64-probe-selftest] probe B failed", file=sys.stderr)
        return rc_b

    rc_matrix = run_step(
        [
            py,
            "tests/run_fixed64_cross_platform_matrix_check.py",
            "--report",
            str(report_a),
            "--report",
            str(report_b),
        ]
    )
    if rc_matrix != 0:
        print("[fixed64-probe-selftest] matrix check failed", file=sys.stderr)
        return rc_matrix

    print("[fixed64-probe-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
