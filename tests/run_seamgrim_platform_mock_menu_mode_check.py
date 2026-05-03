#!/usr/bin/env python
from __future__ import annotations

from pathlib import Path

from _node_runner_contract import run_node_runner


NODE_RUNNER_TIMEOUT_SEC = 120


def fail(detail: str) -> int:
    print(f"check=seamgrim_platform_mock_menu_mode detail={detail}")
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    runner = root / "tests" / "seamgrim_platform_mock_menu_mode_runner.mjs"
    if not runner.exists():
        return fail(f"runner_missing:{runner.as_posix()}")

    ok, elapsed_ms, detail = run_node_runner(root=root, runner=runner, timeout_sec=NODE_RUNNER_TIMEOUT_SEC)
    if not ok:
        return fail(f"node_runner_failed:{runner.name}:elapsed_ms={elapsed_ms}:{detail}")

    print(f"[menu-mode-runner] {runner.name} elapsed_ms={elapsed_ms}")
    print(detail or "seamgrim platform mock menu mode check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
