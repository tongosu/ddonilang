#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TIMEOUT_SEC = 300


COMMANDS: tuple[tuple[str, list[str]], ...] = (
    (
        "sample_console_grid_scalar_show_check",
        [
            "cargo",
            "run",
            "-q",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            "check",
            "solutions/seamgrim_ui_mvp/samples/06_console_grid_scalar_show.ddn",
        ],
    ),
    (
        "sample_moyang_pendulum_check",
        [
            "cargo",
            "run",
            "-q",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            "check",
            "solutions/seamgrim_ui_mvp/samples/09_moyang_pendulum_working.ddn",
        ],
    ),
    (
        "sample_console_grid_mini_tetris_check",
        [
            "cargo",
            "run",
            "-q",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            "check",
            "solutions/seamgrim_ui_mvp/samples/10_console_grid_mini_tetris.ddn",
        ],
    ),
    (
        "sample_console_grid_maze_probe_check",
        [
            "cargo",
            "run",
            "-q",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            "check",
            "solutions/seamgrim_ui_mvp/samples/15_console_grid_maze_probe.ddn",
        ],
    ),
    (
        "sample_space2d_bounce_probe_check",
        [
            "cargo",
            "run",
            "-q",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            "check",
            "solutions/seamgrim_ui_mvp/samples/16_space2d_bounce_probe.ddn",
        ],
    ),
    ("seamgrim_ui_common_runner", ["node", "tests/seamgrim_ui_common_runner.mjs"]),
    ("seamgrim_pendulum_bogae_runner", ["node", "tests/seamgrim_pendulum_bogae_runner.mjs"]),
    ("seamgrim_sample_grid_space_runner", ["node", "tests/seamgrim_sample_grid_space_runner.mjs"]),
    ("seamgrim_run_manager_compare_runner", ["node", "tests/seamgrim_run_manager_compare_runner.mjs"]),
    ("seamgrim_studio_layout_contract_runner", ["node", "tests/seamgrim_studio_layout_contract_runner.mjs"]),
    ("seamgrim_korean_display_label_runner", ["node", "tests/seamgrim_korean_display_label_runner.mjs"]),
    ("seamgrim_layout_modes_check", [sys.executable, "tests/run_seamgrim_layout_modes_check.py"]),
    ("seamgrim_observe_output_contract_check", [sys.executable, "tests/run_seamgrim_observe_output_contract_check.py"]),
    (
        "seamgrim_runtime_view_source_strict_check",
        [sys.executable, "tests/run_seamgrim_runtime_view_source_strict_check.py"],
    ),
    (
        "block_editor_choose_exhaustive_check",
        [sys.executable, "tests/run_block_editor_choose_exhaustive_check.py"],
    ),
    (
        "seamgrim_wasm_cli_runtime_parity_check",
        [sys.executable, "tests/run_seamgrim_wasm_cli_runtime_parity_check.py"],
    ),
    (
        "runtime_support_integrity_audit_check",
        [sys.executable, "tests/run_runtime_support_integrity_audit_check.py"],
    ),
    (
        "ddonirang_vol4_bundle_cli_wasm_parity_check",
        [sys.executable, "tests/run_ddonirang_vol4_bundle_cli_wasm_parity_check.py"],
    ),
    (
        "seamgrim_bogae_madi_graph_ui_check",
        [sys.executable, "tests/run_seamgrim_bogae_madi_graph_ui_check.py"],
    ),
    (
        "bogae_observe_basics_pack_golden",
        [sys.executable, "tests/run_pack_golden.py", "bogae_observe_basics"],
    ),
    (
        "bogae_graph_prefix_check",
        [sys.executable, "tests/run_bogae_graph_prefix_check.py"],
    ),
    (
        "repo_structure_hygiene_check",
        [sys.executable, "tests/run_repo_structure_hygiene_check.py"],
    ),
)


def cleanup_ignored_root_runtime_logs() -> None:
    for name in ["geoul.diag.jsonl"]:
        path = ROOT / name
        if path.exists():
            path.unlink()


def run_step(name: str, cmd: list[str]) -> tuple[bool, str]:
    if name == "repo_structure_hygiene_check":
        cleanup_ignored_root_runtime_logs()
    try:
        proc = subprocess.run(
            cmd,
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=TIMEOUT_SEC,
        )
    except subprocess.TimeoutExpired:
        return False, f"timeout_after={TIMEOUT_SEC}s"

    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "").strip() or f"returncode={proc.returncode}"
        return False, detail
    detail = (proc.stdout or "").strip().splitlines()
    tail = detail[-1] if detail else "ok"
    return True, tail


def main() -> int:
    failures: list[str] = []
    for name, cmd in COMMANDS:
        ok, detail = run_step(name, cmd)
        if ok:
            print(f"ok: {name}: {detail}")
            continue
        print(f"fail: {name}: {detail}")
        failures.append(name)

    if failures:
        print("seamgrim product stabilization smoke failed: " + ",".join(failures))
        return 1

    print("seamgrim product stabilization smoke ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
