#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


FAIL_CODE_RE = re.compile(r"fail code=([A-Z0-9_]+)")


def clip(text: str, limit: int = 180) -> str:
    normalized = " ".join(str(text).split())
    if not normalized:
        return "-"
    if len(normalized) <= limit:
        return normalized
    return normalized[: limit - 3] + "..."


def parse_fail_code(stdout: str, stderr: str, default_code: str) -> str:
    payload = f"{stdout}\n{stderr}"
    match = FAIL_CODE_RE.search(payload)
    if match:
        return match.group(1)
    return default_code


def first_message(stdout: str, stderr: str) -> str:
    for raw in (stderr.splitlines() + stdout.splitlines()):
        line = raw.strip()
        if line:
            return clip(line)
    return "-"


def run_step(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Run CI sanity checks and emit one-line status summary")
    parser.add_argument("--json-out", default="", help="optional path to write sanity result json")
    args = parser.parse_args()

    py = sys.executable
    steps = [
        (
            "backup_hygiene_selftest",
            [py, "tests/run_ci_backup_hygiene_selftest.py"],
            "E_CI_SANITY_BACKUP_SELFTEST_FAIL",
        ),
        (
            "pipeline_emit_flags_check",
            [py, "tests/run_ci_pipeline_emit_flags_check.py"],
            "E_CI_SANITY_PIPELINE_FLAGS_FAIL",
        ),
        (
            "pipeline_emit_flags_selftest",
            [py, "tests/run_ci_pipeline_emit_flags_check_selftest.py"],
            "E_CI_SANITY_PIPELINE_FLAGS_SELFTEST_FAIL",
        ),
    ]

    rows: list[dict[str, object]] = []
    for step_name, cmd, default_code in steps:
        proc = run_step(cmd)
        stdout = proc.stdout or ""
        stderr = proc.stderr or ""
        if stdout.strip():
            print(stdout, end="")
        if stderr.strip():
            print(stderr, end="", file=sys.stderr)
        ok = proc.returncode == 0
        row = {
            "step": step_name,
            "ok": ok,
            "returncode": int(proc.returncode),
            "cmd": cmd,
        }
        if not ok:
            code = parse_fail_code(stdout, stderr, default_code)
            msg = first_message(stdout, stderr)
            row["code"] = code
            row["msg"] = msg
            rows.append(row)
            payload = {
                "schema": "ddn.ci.sanity_gate.v1",
                "generated_at_utc": datetime.now(timezone.utc).isoformat(),
                "status": "fail",
                "code": code,
                "step": step_name,
                "msg": msg,
                "steps": rows,
            }
            if args.json_out.strip():
                out = Path(args.json_out)
                out.parent.mkdir(parents=True, exist_ok=True)
                out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            print(f'ci_sanity_status=fail code={code} step={step_name} msg="{msg}"')
            return 1
        rows.append(row)

    payload = {
        "schema": "ddn.ci.sanity_gate.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": "pass",
        "code": "OK",
        "step": "all",
        "msg": "-",
        "steps": rows,
    }
    if args.json_out.strip():
        out = Path(args.json_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print('ci_sanity_status=pass code=OK step=all msg="-"')
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
