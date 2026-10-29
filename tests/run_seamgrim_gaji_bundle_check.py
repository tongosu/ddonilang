#!/usr/bin/env python
from __future__ import annotations
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
raise SystemExit(subprocess.run(["node", "tests/seamgrim_gaji_bundle_runner.mjs"], cwd=ROOT).returncode)
