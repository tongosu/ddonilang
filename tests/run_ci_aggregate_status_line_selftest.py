#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str) -> int:
    print(f"[ci-aggregate-status-line-selftest] fail: {msg}")
    return 1


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def run_cmd(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="ci_aggregate_status_line_selftest_") as tmp:
        root = Path(tmp)
        aggregate = root / "ci_aggregate_report.detjson"
        status_line = root / "ci_aggregate_status_line.txt"
        parsed = root / "ci_aggregate_status_line_parse.detjson"

        write_json(
            aggregate,
            {
                "schema": "ddn.ci.aggregate_report.v1",
                "generated_at_utc": "2026-02-21T00:00:00+00:00",
                "overall_ok": True,
                "seamgrim": {"ok": True, "failed_steps": []},
                "age3": {"ok": True, "failed_criteria": []},
                "age4": {"ok": True, "failed_criteria": []},
                "oi405_406": {"ok": True, "failed_packs": []},
                "failure_digest": [],
            },
        )

        render = run_cmd(
            [
                sys.executable,
                "tools/scripts/render_ci_aggregate_status_line.py",
                str(aggregate),
                "--out",
                str(status_line),
                "--fail-on-bad",
            ]
        )
        if render.returncode != 0:
            return fail(f"render failed: out={render.stdout} err={render.stderr}")

        parse = run_cmd(
            [
                sys.executable,
                "tools/scripts/parse_ci_aggregate_status_line.py",
                "--status-line",
                str(status_line),
                "--aggregate-report",
                str(aggregate),
                "--json-out",
                str(parsed),
                "--fail-on-invalid",
            ]
        )
        if parse.returncode != 0:
            return fail(f"parse failed: out={parse.stdout} err={parse.stderr}")
        if "age4_failed=0" not in parse.stdout:
            return fail(f"parse compact line missing age4_failed: out={parse.stdout}")

        check = run_cmd(
            [
                sys.executable,
                "tests/run_ci_aggregate_status_line_check.py",
                "--status-line",
                str(status_line),
                "--aggregate-report",
                str(aggregate),
                "--require-pass",
            ]
        )
        if check.returncode != 0:
            return fail(f"check failed: out={check.stdout} err={check.stderr}")

        # negative: age4 key dropped from status line must fail validation
        line = status_line.read_text(encoding="utf-8").strip()
        broken_line = line.replace(" age4_failed_criteria=0", "")
        status_line.write_text(broken_line + "\n", encoding="utf-8")
        broken = run_cmd(
            [
                sys.executable,
                "tests/run_ci_aggregate_status_line_check.py",
                "--status-line",
                str(status_line),
                "--aggregate-report",
                str(aggregate),
            ]
        )
        if broken.returncode == 0:
            return fail("broken status-line case must fail")

    print("[ci-aggregate-status-line-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
