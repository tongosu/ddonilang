#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
TARGET = "7429"


def run_cli(args: list[str]) -> dict:
    proc = subprocess.run(
        ["cargo", "run", "-q", "--manifest-path", "tools/teul-cli/Cargo.toml", "--", *args],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
    )
    if proc.returncode != 0:
        print(proc.stdout)
        print(proc.stderr)
        raise SystemExit(proc.returncode)
    return json.loads(proc.stdout)


def main() -> int:
    complete = run_cli(["numeric", "factor", TARGET, "--mode", "complete"])
    with tempfile.TemporaryDirectory() as tmp:
        job = Path(tmp) / "factor_job.detjson"
        current = run_cli(
            [
                "numeric",
                "factor",
                TARGET,
                "--mode",
                "step",
                "--budget-ops",
                "1",
                "--job-out",
                str(job),
            ]
        )
        for _ in range(10_000):
            if current.get("status") == "done":
                break
            current = run_cli(
                [
                    "numeric",
                    "factor",
                    "--mode",
                    "step",
                    "--resume",
                    str(job),
                    "--budget-ops",
                    "1",
                    "--job-out",
                    str(job),
                ]
            )
        if current.get("status") != "done":
            print("numeric factor resume check fail: status did not reach done")
            return 1
        if current.get("canonical") != complete.get("canonical"):
            print("numeric factor resume check fail: canonical mismatch")
            return 1
        if not current.get("certificate", {}).get("product_matches_input"):
            print("numeric factor resume check fail: certificate mismatch")
            return 1
    print("numeric factor job resume check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
