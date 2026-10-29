#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "tests" / "seamgrim_bogae_madi_graph_ui_runner.mjs"


def fail(message: str) -> int:
    print(f"[seamgrim-bogae-madi-graph-ui] fail: {message}")
    return 1


def main() -> int:
    if not RUNNER.exists():
        return fail(f"missing runner: {RUNNER}")
    proc = subprocess.run(
        ["node", "--no-warnings", str(RUNNER)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=120,
    )
    if proc.returncode != 0:
        return fail(proc.stderr.strip() or proc.stdout.strip() or f"returncode={proc.returncode}")
    if "seamgrim bogae madi graph ui ok" not in proc.stdout:
        return fail(f"unexpected output: {proc.stdout.strip()}")
    print("seamgrim bogae madi graph ui ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
