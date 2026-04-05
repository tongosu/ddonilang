#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def fail(detail: str) -> int:
    print(f"check=patent_b_state_view_hash_isolation detail={detail}")
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    cmd = [
        sys.executable,
        "tests/run_seamgrim_wasm_smoke.py",
        "patent_b_state_view_hash_isolation_v1",
        "--skip-ui-common",
        "--skip-ui-pendulum",
        "--skip-wrapper",
        "--skip-vm-runtime",
        "--skip-space2d-source-gate",
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
        print(proc.stdout.strip())
    if proc.stderr:
        print(proc.stderr.strip())
    if proc.returncode != 0:
        detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or f"returncode={proc.returncode}"
        return fail(f"runner_failed:{detail}")

    print("patent_b state/view hash isolation check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
