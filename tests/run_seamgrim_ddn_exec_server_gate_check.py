#!/usr/bin/env python
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def safe_print(line: str) -> None:
    try:
        print(line)
    except UnicodeEncodeError:
        encoding = sys.stdout.encoding or "utf-8"
        sys.stdout.buffer.write((line + "\n").encode(encoding, errors="replace"))


def fail(detail: str) -> int:
    safe_print(f"check=ddn_exec_server_gate detail={detail}")
    return 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Run seamgrim ddn_exec_server gate check")
    parser.add_argument(
        "--profile",
        choices=["release", "legacy"],
        default="release",
        help="ddn_exec_server_check profile (default: release)",
    )
    args = parser.parse_args()
    base_url = "http://127.0.0.1:18788" if str(args.profile) == "legacy" else "http://127.0.0.1:18787"

    root = Path(__file__).resolve().parent.parent
    checker = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "ddn_exec_server_check.py"
    proc = subprocess.run(
        [
            sys.executable,
            str(checker),
            "--base-url",
            base_url,
            "--timeout-sec",
            "15",
            "--profile",
            str(args.profile),
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
        safe_print(stdout)
    safe_print("seamgrim ddn exec server gate check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
