#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SCHEMA = "ddn.fixed64.darwin_probe_schedule_policy.v1"


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def run_case(report: Path, max_age: float, interval: float) -> int:
    proc = subprocess.run(
        [
            sys.executable,
            "tests/run_fixed64_darwin_probe_schedule_policy_check.py",
            "--max-age-minutes",
            str(max_age),
            "--schedule-interval-minutes",
            str(interval),
            "--json-out",
            str(report),
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    return int(proc.returncode)


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="fixed64_darwin_probe_schedule_policy_selftest_") as td:
        root = Path(td)
        report = root / "schedule_policy.detjson"

        rc_pass = run_case(report, max_age=360, interval=180)
        if rc_pass != 0:
            print("[fixed64-darwin-schedule-policy-selftest] pass case failed", file=sys.stderr)
            return 1
        pass_doc = load_json(report)
        if not isinstance(pass_doc, dict) or str(pass_doc.get("schema", "")) != SCHEMA:
            print("[fixed64-darwin-schedule-policy-selftest] pass report invalid", file=sys.stderr)
            return 1
        if not bool(pass_doc.get("ok", False)) or str(pass_doc.get("status", "")) != "pass":
            print("[fixed64-darwin-schedule-policy-selftest] pass report status mismatch", file=sys.stderr)
            return 1

        rc_fail_equal = run_case(report, max_age=180, interval=180)
        if rc_fail_equal == 0:
            print("[fixed64-darwin-schedule-policy-selftest] equal case should fail", file=sys.stderr)
            return 1
        fail_equal_doc = load_json(report)
        if not isinstance(fail_equal_doc, dict):
            print("[fixed64-darwin-schedule-policy-selftest] equal case report invalid", file=sys.stderr)
            return 1
        if str(fail_equal_doc.get("reason", "")) != "schedule interval must be shorter than max age":
            print("[fixed64-darwin-schedule-policy-selftest] equal case reason mismatch", file=sys.stderr)
            return 1

        rc_fail_invalid = run_case(report, max_age=-1, interval=10)
        if rc_fail_invalid == 0:
            print("[fixed64-darwin-schedule-policy-selftest] invalid max age case should fail", file=sys.stderr)
            return 1
        fail_invalid_doc = load_json(report)
        if not isinstance(fail_invalid_doc, dict):
            print("[fixed64-darwin-schedule-policy-selftest] invalid case report invalid", file=sys.stderr)
            return 1
        if str(fail_invalid_doc.get("reason", "")) != "max_age_minutes must be > 0":
            print("[fixed64-darwin-schedule-policy-selftest] invalid case reason mismatch", file=sys.stderr)
            return 1

    print("[fixed64-darwin-schedule-policy-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
