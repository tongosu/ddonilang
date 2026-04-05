#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def fail(detail: str) -> int:
    print(f"check=state_machine_transition_guard_unresolved_report_pack detail={detail}")
    return 1


def safe_print(text: str) -> None:
    try:
        print(text)
    except UnicodeEncodeError:
        sys.stdout.buffer.write(text.encode("utf-8", errors="replace") + b"\n")


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    cmd = [
        sys.executable,
        "tests/run_pack_golden.py",
        "--manifest-path",
        "tool/Cargo.toml",
        "state_machine_transition_guard_unresolved_report_v1",
    ]
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.stdout:
        safe_print(proc.stdout.strip())
    if proc.stderr:
        safe_print(proc.stderr.strip())
    if proc.returncode != 0:
        detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or f"returncode={proc.returncode}"
        return fail(f"runner_failed:{detail}")

    print("state machine transition guard unresolved report pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
