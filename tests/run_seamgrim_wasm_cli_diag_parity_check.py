#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone
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
    return sum(1 for raw in golden_path.read_text(encoding="utf-8").splitlines() if raw.strip())


def write_report(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="Check seamgrim wasm/cli diag parity contracts")
    parser.add_argument("--json-out", default="", help="optional detjson report path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    steps: list[dict[str, object]] = []
    status = "pass"
    code = "OK"
    step = "all"
    msg = "-"

    def record_step(
        name: str,
        ok: bool,
        detail: str,
        *,
        returncode: int = 0,
        cmd: list[str] | None = None,
        stdout_head: str = "-",
        stderr_head: str = "-",
    ) -> None:
        steps.append(
            {
                "name": name,
                "ok": bool(ok),
                "returncode": int(returncode),
                "detail": detail,
                "cmd": cmd or [],
                "stdout_head": stdout_head,
                "stderr_head": stderr_head,
            }
        )

    def finalize(exit_code: int) -> int:
        if args.json_out.strip():
            payload = {
                "schema": "ddn.seamgrim.wasm_cli_diag_parity.v1",
                "generated_at_utc": datetime.now(timezone.utc).isoformat(),
                "status": status,
                "ok": status == "pass",
                "code": code,
                "step": step,
                "msg": msg,
                "steps": steps,
                "steps_count": len(steps),
            }
            write_report(Path(args.json_out), payload)
        return exit_code

    block_header_golden = root / "pack" / "block_header_no_colon" / "golden.jsonl"
    event_surface_golden = root / "pack" / "seamgrim_event_surface_canon_v1" / "golden.jsonl"
    for name, path in (
        ("block_header_golden_exists", block_header_golden),
        ("event_surface_golden_exists", event_surface_golden),
    ):
        if not path.exists():
            record_step(name, False, f"missing file: {path}")
            status = "fail"
            code = "E_SEAMGRIM_WASM_CLI_PARITY_PATH_MISSING"
            step = name
            msg = f"missing file: {path}"
            return finalize(fail(msg))
        record_step(name, True, f"found: {path}")

    try:
        warning_codes = load_expected_codes(block_header_golden, "expected_warning_code")
    except Exception as exc:
        detail = str(exc)
        record_step("block_header_warning_codes", False, detail)
        status = "fail"
        code = "E_SEAMGRIM_WASM_CLI_PARITY_GOLDEN_PARSE"
        step = "block_header_warning_codes"
        msg = detail
        return finalize(fail(msg))
    if WARNING_CODE not in warning_codes:
        detail = f"{WARNING_CODE} missing in {block_header_golden}"
        record_step("block_header_warning_codes", False, detail)
        status = "fail"
        code = "E_SEAMGRIM_WASM_CLI_PARITY_WARNING_CODE_MISSING"
        step = "block_header_warning_codes"
        msg = detail
        return finalize(fail(msg))
    block_case_count = count_cases(block_header_golden)
    if block_case_count < BLOCK_HEADER_MIN_CASES:
        detail = f"block header golden case count underflow: {block_case_count}<{BLOCK_HEADER_MIN_CASES}"
        record_step("block_header_case_count", False, detail)
        status = "fail"
        code = "E_SEAMGRIM_WASM_CLI_PARITY_CASE_UNDERFLOW"
        step = "block_header_case_count"
        msg = detail
        return finalize(fail(msg))
    record_step("block_header_contract", True, f"warning={WARNING_CODE} case_count={block_case_count}")

    try:
        event_codes = load_expected_codes(event_surface_golden, "expected_error_code")
    except Exception as exc:
        detail = str(exc)
        record_step("event_surface_error_codes", False, detail)
        status = "fail"
        code = "E_SEAMGRIM_WASM_CLI_PARITY_GOLDEN_PARSE"
        step = "event_surface_error_codes"
        msg = detail
        return finalize(fail(msg))
    if EVENT_ALIAS_FORBIDDEN_CODE not in event_codes:
        detail = f"{EVENT_ALIAS_FORBIDDEN_CODE} missing in {event_surface_golden}"
        record_step("event_surface_error_codes", False, detail)
        status = "fail"
        code = "E_SEAMGRIM_WASM_CLI_PARITY_EVENT_CODE_MISSING"
        step = "event_surface_error_codes"
        msg = detail
        return finalize(fail(msg))
    event_case_count = count_cases(event_surface_golden)
    if event_case_count < EVENT_SURFACE_MIN_CASES:
        detail = f"event surface golden case count underflow: {event_case_count}<{EVENT_SURFACE_MIN_CASES}"
        record_step("event_surface_case_count", False, detail)
        status = "fail"
        code = "E_SEAMGRIM_WASM_CLI_PARITY_CASE_UNDERFLOW"
        step = "event_surface_case_count"
        msg = detail
        return finalize(fail(msg))
    record_step("event_surface_contract", True, f"error={EVENT_ALIAS_FORBIDDEN_CODE} case_count={event_case_count}")

    wasm_runner_files = (
        root / "tests" / "seamgrim_wasm_wrapper_runner.mjs",
        root / "tests" / "seamgrim_wasm_vm_runtime_runner.mjs",
    )
    for wasm_runner in wasm_runner_files:
        name = f"wasm_runner_token:{wasm_runner.name}"
        if not wasm_runner.exists():
            detail = f"missing file: {wasm_runner}"
            record_step(name, False, detail)
            status = "fail"
            code = "E_SEAMGRIM_WASM_CLI_PARITY_PATH_MISSING"
            step = name
            msg = detail
            return finalize(fail(msg))
        text = wasm_runner.read_text(encoding="utf-8")
        if WARNING_CODE not in text:
            detail = f"{WARNING_CODE} missing in {wasm_runner}"
            record_step(name, False, detail)
            status = "fail"
            code = "E_SEAMGRIM_WASM_CLI_PARITY_TOKEN_MISSING"
            step = name
            msg = detail
            return finalize(fail(msg))
        record_step(name, True, WARNING_CODE)

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
        merged = f"{proc.stdout}\n{proc.stderr}"
        if proc.returncode != 0:
            detail = (
                f"{step_name} rc={proc.returncode} cmd={' '.join(cmd)} "
                f"stdout={proc.stdout.strip()} stderr={proc.stderr.strip()}"
            )
            record_step(
                step_name,
                False,
                detail,
                returncode=proc.returncode,
                cmd=cmd,
                stdout_head=(proc.stdout or "").strip() or "-",
                stderr_head=(proc.stderr or "").strip() or "-",
            )
            status = "fail"
            code = "E_SEAMGRIM_WASM_CLI_PARITY_STEP_FAIL"
            step = step_name
            msg = detail
            return finalize(fail(msg))
        if ok_marker not in merged:
            detail = f"{step_name} marker missing: {ok_marker}"
            record_step(
                step_name,
                False,
                detail,
                returncode=proc.returncode,
                cmd=cmd,
                stdout_head=(proc.stdout or "").strip() or "-",
                stderr_head=(proc.stderr or "").strip() or "-",
            )
            status = "fail"
            code = "E_SEAMGRIM_WASM_CLI_PARITY_MARKER_MISSING"
            step = step_name
            msg = detail
            return finalize(fail(msg))
        record_step(
            step_name,
            True,
            ok_marker,
            returncode=0,
            cmd=cmd,
            stdout_head=(proc.stdout or "").strip() or "-",
            stderr_head=(proc.stderr or "").strip() or "-",
        )

    print("[seamgrim-wasm-cli-diag-parity] ok")
    return finalize(0)


if __name__ == "__main__":
    raise SystemExit(main())
