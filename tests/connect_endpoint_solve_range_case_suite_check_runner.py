#!/usr/bin/env python
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

from _teul_cli_freshness import build_teul_cli_cmd

EXPECTED_KIND = "endpoint_solve_range_case_suite_check"
EXPECTED_STDOUT_MARKER = "connect_case_suite_check_runner_expected_check_stdout"
TOOL_FAILED_MARKER = "connect_case_suite_check_runner_tool_failed"
INVALID_JUDGEMENT_MARKER = "connect_case_suite_check_runner_invalid_judgement"


def teul_candidates(root: Path) -> list[Path]:
    suffix = ".exe" if os.name == "nt" else ""
    return [
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        root / "target" / "debug" / f"teul-cli{suffix}",
    ]


def run_teul(root: Path, input_path: Path) -> subprocess.CompletedProcess[str]:
    cmd = build_teul_cli_cmd(
        root,
        ["run", str(input_path)],
        candidates=teul_candidates(root),
        manifest_path=root / "tools" / "teul-cli" / "Cargo.toml",
    )
    env = os.environ.copy()
    env.setdefault("RUST_MIN_STACK", str(64 * 1024 * 1024))
    env.setdefault("CARGO_TARGET_DIR", str((root / "build" / "cargo-target-check-runner").resolve()))
    return subprocess.run(
        cmd,
        cwd=root,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=env,
    )


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print(f"{EXPECTED_STDOUT_MARKER}: usage: <input.ddn>", file=sys.stderr)
        return 2

    root = Path(__file__).resolve().parents[1]
    input_path = Path(argv[1])
    if not input_path.is_absolute():
        input_path = root / input_path

    result = run_teul(root, input_path)
    if result.returncode != 0:
        payload = (result.stderr or result.stdout or "").strip()
        if payload:
            print(payload, file=sys.stderr)
        print(TOOL_FAILED_MARKER, file=sys.stderr)
        return 2

    lines = result.stdout.splitlines()
    if len(lines) < 2 or lines[0] != EXPECTED_KIND:
        print(EXPECTED_STDOUT_MARKER, file=sys.stderr)
        return 2

    judgement = lines[1]
    if judgement == "통과":
        return 0
    if judgement == "실패":
        return 1

    print(INVALID_JUDGEMENT_MARKER, file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
