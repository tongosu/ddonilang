#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "tests" / "ddn_current_public_rejection_parity_runner.mjs"


def main() -> int:
    proc = subprocess.run(
        ["node", str(RUNNER)],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=600,
        check=False,
    )
    if proc.returncode != 0:
        print(proc.stdout, end="" if proc.stdout.endswith("\n") else "\n")
        return proc.returncode
    print("[ddn-cli-wasm-true-parity-successor] PASS current public expected-rejection parity 3/3")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
