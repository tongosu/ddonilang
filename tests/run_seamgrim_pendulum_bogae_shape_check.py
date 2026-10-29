#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path


def fail(detail: str) -> int:
    print(f"check=pendulum_bogae_shape detail={detail}")
    return 1


def run_node_runner(root: Path, runner: Path, *, detail_tag: str) -> tuple[int, str]:
    if not runner.exists():
        return 1, f"{detail_tag}:runner_missing:{runner.as_posix()}"
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
        return 1, f"{detail_tag}:node_runner_failed:{detail}"
    return 0, (proc.stdout or "").strip()


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    pendulum_runner = root / "tests" / "seamgrim_pendulum_bogae_runner.mjs"
    warning_map_runner = root / "tests" / "seamgrim_run_warning_message_map_runner.mjs"

    rc, detail = run_node_runner(root, pendulum_runner, detail_tag="pendulum")
    if rc != 0:
        return fail(detail)
    rc, warning_detail = run_node_runner(root, warning_map_runner, detail_tag="warning_map")
    if rc != 0:
        return fail(warning_detail)

    print(detail or warning_detail or "pendulum bogae shape check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
