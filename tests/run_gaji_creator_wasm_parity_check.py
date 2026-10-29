#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

proc = subprocess.run(
    ["node", "tests/gaji_creator_wasm_parity_runner.mjs"],
    cwd=ROOT,
    text=True,
    encoding="utf-8",
    errors="replace",
)
raise SystemExit(proc.returncode)
