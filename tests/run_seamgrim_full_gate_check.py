#!/usr/bin/env python
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def fail(detail: str) -> int:
    print(f"check=full_gate detail={detail}")
    return 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Run seamgrim full gate wrapper")
    parser.add_argument(
        "--strict-graph",
        dest="strict_graph",
        action="store_true",
        help="forward strict graph mode (default: on)",
    )
    parser.add_argument(
        "--allow-graph-warnings",
        dest="strict_graph",
        action="store_false",
        help="allow graph export warnings (legacy compatibility)",
    )
    parser.set_defaults(strict_graph=True)
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    checker = root / "tests" / "run_seamgrim_full_check.py"
    cmd = [sys.executable, str(checker), "--skip-schema-gate"]
    if args.strict_graph:
        cmd.append("--strict-graph")
    proc = subprocess.run(
        cmd,
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
    print("seamgrim full gate check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
