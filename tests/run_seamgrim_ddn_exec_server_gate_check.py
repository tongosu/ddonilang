#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def fail(detail: str) -> int:
    print(f"check=ddn_exec_server_gate detail={detail}")
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    checker = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "ddn_exec_server_check.py"
    proc = subprocess.run(
        [
            sys.executable,
            str(checker),
            "--base-url",
            "http://127.0.0.1:18787",
            "--timeout-sec",
            "15",
        ],
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or f"returncode={proc.returncode}"
        return fail(detail)
    stdout = (proc.stdout or "").strip()
    if stdout:
        print(stdout)
    print("seamgrim ddn exec server gate check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

