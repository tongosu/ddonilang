#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path


def fail(detail: str) -> int:
    print(f"check=pendulum_bogae_shape detail={detail}")
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    runner = root / "tests" / "seamgrim_pendulum_bogae_runner.mjs"
    if not runner.exists():
        return fail(f"runner_missing:{runner.as_posix()}")

    proc = subprocess.run(
        ["node", str(runner)],
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or f"returncode={proc.returncode}"
        return fail(f"node_runner_failed:{detail}")

    print((proc.stdout or "").strip() or "pendulum bogae shape check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
