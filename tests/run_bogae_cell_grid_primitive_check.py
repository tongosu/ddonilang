#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "tests" / "bogae_cell_grid_primitive_runner.mjs"


def main() -> int:
    proc = subprocess.run(
        ["node", "--no-warnings", str(RUNNER)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        print("[FAIL] bogae cell-grid primitive", file=sys.stderr)
        if proc.stderr:
            print(proc.stderr, file=sys.stderr)
        if proc.stdout:
            print(proc.stdout, file=sys.stderr)
        return proc.returncode
    print((proc.stdout or "[bogae-cell-grid-primitive] ok").strip())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
