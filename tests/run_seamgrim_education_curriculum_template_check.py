#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    proc = subprocess.run(
        ["node", "tests/seamgrim_education_curriculum_template_runner.mjs"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return proc.returncode


if __name__ == "__main__":
    raise SystemExit(main())
