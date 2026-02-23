#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    cmd = ["node", "tests/seamgrim_overlay_session_contract_runner.mjs"]
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
        print("overlay session contract failed")
        return proc.returncode
    if "[overlay-session-contract] ok" not in stdout:
        print("overlay session contract failed: pass marker missing")
        return 1
    print("overlay session contract ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
