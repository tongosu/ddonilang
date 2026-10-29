#!/usr/bin/env python3
"""Validate Seamgrim Wave2E G8 full durable project workflow across four viewports."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def run(cmd: list[str], timeout: int = 120) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def fail(code: str, detail: str = "") -> int:
    print(f"[seamgrim-project-full-workflow-viewport-v0] FAIL {code}{': ' + detail if detail else ''}")
    return 1


def main() -> int:
    runner_path = ROOT / "tests/seamgrim_project_full_workflow_viewport_runner.mjs"
    runner_text = runner_path.read_text(encoding="utf-8")
    required_tokens = [
        "1440x900",
        "1280x720",
        "1024x768",
        "390x844",
        "runAndCheckCanvas",
        "nonblank",
        "keyboard tab",
        "view permissions",
        "내 버전 생성",
        "stopServer(serverRef.child)",
        "restart",
      ]
    missing = [token for token in required_tokens if token not in runner_text]
    if missing:
        return fail("E_PROJECT_FULL_WORKFLOW_STATIC", repr(missing))

    syntax = run(["node", "--check", "tests/seamgrim_project_full_workflow_viewport_runner.mjs"], timeout=60)
    if syntax.returncode != 0:
        return fail("E_PROJECT_FULL_WORKFLOW_SYNTAX", syntax.stdout.strip())

    runner = run(["node", "tests/seamgrim_project_full_workflow_viewport_runner.mjs"], timeout=420)
    if runner.returncode != 0:
        return fail("E_PROJECT_FULL_WORKFLOW_RUNNER", runner.stdout.strip())
    last_line = runner.stdout.strip().splitlines()[-1]
    try:
        result = json.loads(last_line)
    except json.JSONDecodeError as exc:
        return fail("E_PROJECT_FULL_WORKFLOW_JSON", f"{exc}: {last_line}")
    if result.get("viewport_count") != 4:
        return fail("E_PROJECT_FULL_WORKFLOW_VIEWPORT_COUNT", repr(result))
    if result.get("revision_advanced_all") is not True or result.get("clone_distinct_all") is not True:
        return fail("E_PROJECT_FULL_WORKFLOW_RESULT", repr(result))
    if set(result.get("viewports") or []) != {"1440x900", "1280x720", "1024x768", "390x844"}:
        return fail("E_PROJECT_FULL_WORKFLOW_VIEWPORTS", repr(result))

    golden = run(["python", "tests/run_pack_golden.py", "seamgrim_project_full_workflow_viewport_v0"], timeout=120)
    if golden.returncode != 0:
        return fail("E_PROJECT_FULL_WORKFLOW_GOLDEN", golden.stdout.strip())
    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode != 0 or ssot.stdout.strip():
        return fail("E_PROJECT_FULL_WORKFLOW_SSOT_DIRTY", ssot.stdout.strip())
    print("[seamgrim-project-full-workflow-viewport-v0] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
