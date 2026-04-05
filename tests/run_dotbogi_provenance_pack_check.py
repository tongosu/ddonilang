#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


PACKS = (
    "dotbogi_ddn_interface_v1_smoke",
    "dotbogi_ddn_interface_v1_event_roundtrip",
    "dotbogi_ddn_interface_v1_write_forbidden",
)


def fail(detail: str) -> int:
    print(f"check=dotbogi_provenance_pack detail={detail}")
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    cmd = [sys.executable, "tests/run_pack_golden.py", *PACKS]
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.stdout:
        print(proc.stdout.strip())
    if proc.stderr:
        print(proc.stderr.strip())
    if proc.returncode != 0:
        detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or f"returncode={proc.returncode}"
        return fail(f"runner_failed:{detail}")

    print("dotbogi provenance pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
