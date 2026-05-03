#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
PACKS = [
    "numeric_exact_universe_v1",
    "numeric_factor_kernel_unbounded_v1",
    "numeric_factor_job_resume_v1",
]


def run(cmd: list[str]) -> int:
    proc = subprocess.run(cmd, cwd=ROOT, text=True)
    if proc.returncode != 0:
        return proc.returncode
    return 0


def main() -> int:
    commands = [
        ["cargo", "test", "-p", "ddonirang-numeric"],
        ["python", "tests/run_numeric_p0n_pack_check.py"],
        *[["python", "tests/run_pack_golden.py", pack] for pack in PACKS],
        ["python", "tests/run_numeric_factor_job_resume_check.py"],
        ["python", "tests/run_seamgrim_numeric_kernel_ui_check.py"],
    ]
    for cmd in commands:
        rc = run(cmd)
        if rc != 0:
            return rc
    print("numeric full kernel check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
