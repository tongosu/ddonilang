#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    runner = root / "tests" / "run_seamgrim_lesson_migration_lint_check.py"
    if not runner.exists():
        print(f"check=lesson_migration_lint_preview detail=runner_missing:{runner}")
        return 1

    cmd = [
        sys.executable,
        str(runner),
        "--include-preview",
    ]
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "").strip() or "runner_failed"
        print(f"check=lesson_migration_lint_preview detail=runner_failed:{detail}")
        return 1
    print("check=lesson_migration_lint_preview detail=ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
