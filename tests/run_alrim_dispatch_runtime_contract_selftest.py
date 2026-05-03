#!/usr/bin/env python
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

from _teul_cli_freshness import build_teul_cli_cmd as shared_build_teul_cli_cmd

DISPATCH_TEST_FILTER = "signal_send_dispatch"
REQUIRED_DISPATCH_TEST_NAMES = (
    "signal_send_dispatch_same_rank_preserves_declaration_order",
    "signal_send_dispatch_error_clears_remaining_pending_queue",
    "signal_send_dispatch_error_clears_pending_queue_tail",
)


def fail(msg: str) -> int:
    print(f"[alrim-dispatch-runtime-contract-selftest] fail: {msg}")
    return 1


def teul_cli_candidates(root: Path) -> list[Path]:
    suffix = ".exe" if os.name == "nt" else ""
    return [
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
        Path(f"C:/ddn/codex/target/debug/teul-cli{suffix}"),
        root / "target" / "debug" / f"teul-cli{suffix}",
    ]


def run_teul_cli(root: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
    cmd = shared_build_teul_cli_cmd(
        root,
        args,
        candidates=teul_cli_candidates(root),
        include_which=False,
        manifest_path=root / "tools" / "teul-cli" / "Cargo.toml",
    )
    return subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def run_cargo_test(root: Path, test_name: str) -> subprocess.CompletedProcess[str]:
    cmd = [
        "cargo",
        "test",
        "--manifest-path",
        str(root / "tools" / "teul-cli" / "Cargo.toml"),
        test_name,
        "--",
        "--nocapture",
    ]
    return subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def run_dispatch_test_binary(binary: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(binary), DISPATCH_TEST_FILTER, "--nocapture"],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def resolve_teul_cli_test_bin_dirs(root: Path) -> list[Path]:
    return [
        Path("I:/home/urihanl/ddn/codex/target/debug/deps"),
        Path("C:/ddn/codex/target/debug/deps"),
        root / "target" / "debug" / "deps",
    ]


def resolve_dispatch_test_binary(root: Path) -> Path | None:
    suffix = ".exe" if os.name == "nt" else ""
    candidates: list[Path] = []
    for base in resolve_teul_cli_test_bin_dirs(root):
        if not base.exists():
            continue
        rows = sorted(base.glob(f"teul_cli-*{suffix}"), key=lambda path: path.stat().st_mtime, reverse=True)
        candidates.extend(rows[:10])
    for candidate in candidates:
        listed = subprocess.run(
            [str(candidate), "--list"],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if listed.returncode != 0:
            continue
        stdout = listed.stdout
        if all(name in stdout for name in REQUIRED_DISPATCH_TEST_NAMES):
            return candidate
    return None


def ensure_dispatch_test_binary(root: Path) -> Path | None:
    existing = resolve_dispatch_test_binary(root)
    if existing is not None:
        return existing
    build = subprocess.run(
        [
            "cargo",
            "test",
            "-q",
            "--manifest-path",
            str(root / "tools" / "teul-cli" / "Cargo.toml"),
            DISPATCH_TEST_FILTER,
            "--no-run",
        ],
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if build.returncode != 0:
        return None
    return resolve_dispatch_test_binary(root)


def run_dispatch_suite(root: Path) -> subprocess.CompletedProcess[str]:
    binary = ensure_dispatch_test_binary(root)
    if binary is not None:
        return run_dispatch_test_binary(binary)
    return run_cargo_test(root, DISPATCH_TEST_FILTER)


def main() -> int:
    root = Path(__file__).resolve().parent.parent

    rank_proc = run_teul_cli(
        root,
        ["run", "pack/lang_consistency_v1/c23_receive_hooks_same_rank_decl_order_run/input.ddn"],
    )
    if rank_proc.returncode != 0:
        return fail(f"rank order pack failed out={rank_proc.stdout} err={rank_proc.stderr}")
    rank_lines = [line.strip() for line in rank_proc.stdout.splitlines() if line.strip()]
    if not rank_lines or rank_lines[0] != "12345":
        return fail(f"rank order stdout mismatch out={rank_proc.stdout}")

    dispatch_suite = run_dispatch_suite(root)
    if dispatch_suite.returncode != 0:
        return fail(
            "signal-send dispatch suite test failed "
            f"out={dispatch_suite.stdout} err={dispatch_suite.stderr}"
        )
    required_tokens = (
        "signal_send_dispatch_same_rank_preserves_declaration_order ... ok",
        "signal_send_dispatch_error_clears_remaining_pending_queue ... ok",
        "signal_send_dispatch_error_clears_pending_queue_tail ... ok",
    )
    for token in required_tokens:
        if token not in dispatch_suite.stdout:
            return fail(f"dispatch suite missing token={token} out={dispatch_suite.stdout}")

    print("[alrim-dispatch-runtime-contract-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
