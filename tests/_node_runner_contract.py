#!/usr/bin/env python
from __future__ import annotations

import subprocess
import time
from pathlib import Path


def run_node_runner(*, root: Path, runner: Path, timeout_sec: int) -> tuple[bool, int, str]:
    started = time.perf_counter()
    try:
        proc = subprocess.run(
            ["node", "--no-warnings", str(runner)],
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=timeout_sec,
        )
    except subprocess.TimeoutExpired as exc:
        elapsed_ms = int((time.perf_counter() - started) * 1000)
        output = (exc.stderr or "").strip() or (exc.stdout or "").strip()
        detail = output or f"timeout={timeout_sec}s"
        return False, elapsed_ms, f"timeout:{detail}"

    elapsed_ms = int((time.perf_counter() - started) * 1000)
    if proc.returncode != 0:
        detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or f"returncode={proc.returncode}"
        return False, elapsed_ms, detail

    detail = (proc.stdout or "").strip() or "ok"
    return True, elapsed_ms, detail

