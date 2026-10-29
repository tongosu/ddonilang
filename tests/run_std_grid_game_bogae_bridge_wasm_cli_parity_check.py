#!/usr/bin/env python3
"""Check focused CLI/direct WASM-feature coverage for the grid-game Bogae bridge."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def fail(message: str) -> None:
    print(f"[std-grid-game-bogae-parity] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def run_and_require_tests(args: list[str]) -> None:
    proc = subprocess.run(
        args,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=300,
    )
    output = f"{proc.stdout}\n{proc.stderr}"
    if proc.returncode != 0:
        fail(output.strip() or f"command failed: {' '.join(args)}")
    if "running 0 tests" in output or "0 tests ran" in output:
        fail(f"0 tests ran: {' '.join(args)}")


def main() -> None:
    for rel in [
        "pack/std_grid_game_bogae_bridge_closure_v1/golden.jsonl",
        "tests/run_std_grid_game_bogae_bridge_pack_check.py",
    ]:
        if not (ROOT / rel).is_file():
            fail(f"missing file: {rel}")

    run_and_require_tests([
        "cargo",
        "test",
        "--manifest-path",
        "tools/teul-cli/Cargo.toml",
        "std_grid_game_bogae",
        "--",
        "--nocapture",
    ])
    run_and_require_tests([
        "cargo",
        "test",
        "--manifest-path",
        "tool/Cargo.toml",
        "--features",
        "wasm",
        "std_grid_game_bogae",
        "--",
        "--nocapture",
    ])
    subprocess.run(
        [sys.executable, "tests/run_pack_golden.py", "std_grid_game_bogae_bridge_closure_v1"],
        cwd=ROOT,
        check=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    print("[std-grid-game-bogae-parity] OK")


if __name__ == "__main__":
    main()

