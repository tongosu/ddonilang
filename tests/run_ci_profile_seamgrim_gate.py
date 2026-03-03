#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    cmd = [py, "tests/run_ci_sanity_gate.py", "--profile", "seamgrim"]
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    stdout = (proc.stdout or "").strip()
    stderr = (proc.stderr or "").strip()
    if stdout:
        print(stdout)
    if stderr:
        print(stderr)

    if proc.returncode != 0:
        print("ci_profile_seamgrim_status=fail reason=sanity_gate_failed")
        return proc.returncode
    if "ci_sanity_status=pass" not in stdout or "profile=seamgrim" not in stdout:
        print("ci_profile_seamgrim_status=fail reason=pass_marker_missing")
        return 1
    print("ci_profile_seamgrim_status=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
