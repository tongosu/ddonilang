#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    cmd = [
        sys.executable,
        "solutions/seamgrim_ui_mvp/tools/sync_featured_seed_catalog.py",
        "--check",
    ]
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
        print("seamgrim featured seed catalog autogen check failed")
        return proc.returncode
    print("seamgrim featured seed catalog autogen check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
