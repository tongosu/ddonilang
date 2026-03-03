#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

WARNING_CODE = "W_BLOCK_HEADER_COLON_DEPRECATED"
EVENT_ALIAS_FORBIDDEN_CODE = "E_EVENT_SURFACE_ALIAS_FORBIDDEN"
BLOCK_HEADER_MIN_CASES = 5
EVENT_SURFACE_MIN_CASES = 7

OVERLAY_COMPARE_PARITY_SCRIPT = "tests/run_seamgrim_overlay_compare_diag_parity_check.py"
OVERLAY_SESSION_PARITY_SCRIPT = "tests/run_seamgrim_overlay_session_diag_parity_check.py"
OVERLAY_SESSION_WIRED_SCRIPT = "tests/run_seamgrim_overlay_session_wired_consistency_check.py"


def fail(msg: str) -> int:
    print(f"[seamgrim-wasm-cli-diag-parity] fail: {msg}")
    return 1


def run_cmd(root: Path, cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def load_expected_codes(golden_path: Path, key: str) -> set[str]:
    codes: set[str] = set()
    for idx, raw in enumerate(golden_path.read_text(encoding="utf-8").splitlines(), start=1):
        line = raw.strip()
        if not line:
            continue
        try:
            row = json.loads(line)
        except json.JSONDecodeError as exc:
            raise ValueError(f"{golden_path} line {idx}: invalid json: {exc}") from exc
        value = str(row.get(key, "")).strip()
        if value:
            codes.add(value)
    return codes


def count_cases(golden_path: Path) -> int:
    count = 0
    for raw in golden_path.read_text(encoding="utf-8").splitlines():
        if raw.strip():
            count += 1
    return count


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    py = sys.executable

    block_header_golden = root / "pack" / "block_header_no_colon" / "golden.jsonl"
    event_surface_golden = root / "pack" / "seamgrim_event_surface_canon_v1" / "golden.jsonl"
    if not block_header_golden.exists():
        return fail(f"missing file: {block_header_golden}")
    if not event_surface_golden.exists():
        return fail(f"missing file: {event_surface_golden}")

    warning_codes = load_expected_codes(block_header_golden, "expected_warning_code")
    if WARNING_CODE not in warning_codes:
        return fail(f"{WARNING_CODE} missing in {block_header_golden}")
    block_case_count = count_cases(block_header_golden)
    if block_case_count < BLOCK_HEADER_MIN_CASES:
        return fail(f"block header golden case count underflow: {block_case_count}<{BLOCK_HEADER_MIN_CASES}")

    event_codes = load_expected_codes(event_surface_golden, "expected_error_code")
    if EVENT_ALIAS_FORBIDDEN_CODE not in event_codes:
        return fail(f"{EVENT_ALIAS_FORBIDDEN_CODE} missing in {event_surface_golden}")
    event_case_count = count_cases(event_surface_golden)
    if event_case_count < EVENT_SURFACE_MIN_CASES:
        return fail(f"event surface golden case count underflow: {event_case_count}<{EVENT_SURFACE_MIN_CASES}")

    wasm_runner_files = (
        root / "tests" / "seamgrim_wasm_wrapper_runner.mjs",
        root / "tests" / "seamgrim_wasm_vm_runtime_runner.mjs",
    )
    for wasm_runner in wasm_runner_files:
        if not wasm_runner.exists():
            return fail(f"missing file: {wasm_runner}")
        text = wasm_runner.read_text(encoding="utf-8")
        if WARNING_CODE not in text:
            return fail(f"{WARNING_CODE} missing in {wasm_runner}")

    commands: list[tuple[str, list[str], str]] = [
        (
            "wasm_wrapper_runner",
            ["node", "--no-warnings", "tests/seamgrim_wasm_wrapper_runner.mjs"],
            "seamgrim wasm wrapper ok",
        ),
        (
            "wasm_vm_runtime_runner",
            ["node", "--no-warnings", "tests/seamgrim_wasm_vm_runtime_runner.mjs"],
            "seamgrim wasm vm runtime ok",
        ),
        (
            "cli_pack_block_header",
            [py, "tests/run_pack_golden.py", "block_header_no_colon"],
            "pack golden ok",
        ),
        (
            "cli_pack_event_surface",
            [py, "tests/run_pack_golden.py", "seamgrim_event_surface_canon_v1"],
            "pack golden ok",
        ),
        (
            "overlay_compare_diag_parity",
            [py, OVERLAY_COMPARE_PARITY_SCRIPT],
            "overlay compare diag parity check ok",
        ),
        (
            "overlay_session_diag_parity",
            [py, OVERLAY_SESSION_PARITY_SCRIPT],
            "overlay session diag parity check ok",
        ),
        (
            "overlay_session_wired_consistency",
            [py, OVERLAY_SESSION_WIRED_SCRIPT],
            "overlay session wired consistency check ok",
        ),
    ]

    for step_name, cmd, ok_marker in commands:
        proc = run_cmd(root, cmd)
        if proc.returncode != 0:
            return fail(
                f"{step_name} rc={proc.returncode} cmd={' '.join(cmd)} "
                f"stdout={proc.stdout.strip()} stderr={proc.stderr.strip()}"
            )
        merged = f"{proc.stdout}\n{proc.stderr}"
        if ok_marker not in merged:
            return fail(f"{step_name} marker missing: {ok_marker}")

    print("[seamgrim-wasm-cli-diag-parity] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
