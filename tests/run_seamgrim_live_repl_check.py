#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def main() -> int:
    proc = subprocess.run(
        ["node", "tests/seamgrim_live_repl_runner.mjs"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        return proc.returncode
    print("[seamgrim-live-repl-check] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
