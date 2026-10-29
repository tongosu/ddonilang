#!/usr/bin/env python
import subprocess
import sys
from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    proc = subprocess.run(
        ["node", "tests/seamgrim_stdlib_1_wasm_runner.mjs"],
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=180,
    )
    if proc.returncode != 0:
        print(proc.stderr.strip() or proc.stdout.strip(), file=sys.stderr)
        return proc.returncode
    print(proc.stdout.strip())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
