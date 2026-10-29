#!/usr/bin/env python
"""누리메이커 격자 웹 smoke 검증."""
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
PACK_DIR = ROOT / "pack" / "nurimaker_grid_render_smoke_v1"
RUNNER = ROOT / "tests" / "nurimaker_grid_runner.mjs"


def main() -> int:
    parser = argparse.ArgumentParser(description="누리메이커 격자 웹 smoke 검증")
    parser.add_argument("--update", action="store_true", help="golden 파일 갱신")
    args = parser.parse_args()

    cmd = ["node", "--no-warnings", str(RUNNER), str(PACK_DIR)]
    if args.update:
        cmd.append("--update")

    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    stdout = (proc.stdout or "").strip()
    stderr = (proc.stderr or "").strip()
    if proc.returncode != 0:
        print("[FAIL] nurimaker grid smoke", file=sys.stderr)
        if stderr:
            print(stderr, file=sys.stderr)
        if stdout:
            print(stdout, file=sys.stderr)
        return 1
    print(stdout or "[ok] nurimaker grid smoke")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
