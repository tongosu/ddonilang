#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TIMEOUT_SEC = 180


COMMANDS: tuple[tuple[str, list[str]], ...] = (
    (
        "age5_close_child_summary_selftest",
        [sys.executable, "tests/run_age5_close_child_summary_selftest.py"],
    ),
    (
        "age5_close_combined_heavy_timeout_selftest",
        [sys.executable, "tests/run_age5_close_combined_heavy_timeout_selftest.py"],
    ),
    (
        "age5_close_digest_selftest",
        [sys.executable, "tests/run_age5_close_digest_selftest.py"],
    ),
    (
        "age5_close_combined_report_contract_selftest",
        [sys.executable, "tests/run_age5_close_combined_report_contract_selftest.py"],
    ),
)


def run_step(name: str, cmd: list[str]) -> tuple[bool, str]:
    try:
        proc = subprocess.run(
            cmd,
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=TIMEOUT_SEC,
        )
    except subprocess.TimeoutExpired:
        return False, f"timeout_after={TIMEOUT_SEC}s"

    detail = (proc.stdout or proc.stderr or "").strip().splitlines()
    tail = detail[-1] if detail else f"rc={proc.returncode}"
    if proc.returncode != 0:
        return False, tail
    return True, tail


def main() -> int:
    failures: list[str] = []
    for name, cmd in COMMANDS:
        ok, detail = run_step(name, cmd)
        if ok:
            print(f"ok: {name}: {detail}")
            continue
        print(f"fail: {name}: {detail}")
        failures.append(name)

    if failures:
        print("[age5-close-review-suite] FAIL " + ",".join(failures))
        return 1
    print("[age5-close-review-suite] PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
