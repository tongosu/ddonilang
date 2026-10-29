#!/usr/bin/env python
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def fail(detail: str) -> int:
    print(f"check=guideblock_keys_pack detail={detail}")
    return 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Run guideblock keys basics pack checks")
    parser.add_argument(
        "--pack-root",
        default="pack/guideblock_keys_basics",
        help="guideblock keys pack root",
    )
    parser.add_argument(
        "--json-out",
        default="build/reports/seamgrim_guideblock_keys_pack_report.detjson",
        help="optional report output path",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    runner = root / "tests" / "seamgrim_guideblock_keys_pack_runner.mjs"
    if not runner.exists():
        return fail(f"runner_missing:{runner.as_posix()}")

    cmd = [
        "node",
        str(runner),
        "--pack-root",
        str(args.pack_root),
        "--json-out",
        str(args.json_out),
    ]
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.stdout:
        print(proc.stdout.strip())
    if proc.stderr:
        print(proc.stderr.strip())
    if proc.returncode != 0:
        detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or f"returncode={proc.returncode}"
        return fail(f"runner_failed:{detail}")
    print("guideblock keys pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
