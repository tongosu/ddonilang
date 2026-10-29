#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "tests" / "seamgrim_workbench_four_viewport_recovery_gate_runner.mjs"
OUT = ROOT / "out" / "seamgrim_workbench_g13"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def run(cmd: list[str], *, timeout: int = 240) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def main() -> int:
    if not RUNNER.exists():
        return fail("E_SEAMGRIM_G13_RUNNER_MISSING", str(RUNNER.relative_to(ROOT)))
    runner_text = RUNNER.read_text(encoding="utf-8")
    required_tokens = [
        "1440",
        "1024",
        "778",
        "390",
        "managementInputs",
        "visibleActionControls",
        "canvasEvidence",
        "pixel_hash",
        "view-role-side",
        "view-role-overlay",
        "view-role-magnifier",
        "overlapReport",
        "summary.detjson",
    ]
    for token in required_tokens:
        if token not in runner_text:
            return fail("E_SEAMGRIM_G13_RUNNER_TOKEN", token)

    syntax = run(["node", "--check", str(RUNNER.relative_to(ROOT))], timeout=60)
    if syntax.returncode:
        return fail("E_SEAMGRIM_G13_SYNTAX", syntax.stdout.strip())

    browser = run(["node", str(RUNNER.relative_to(ROOT))], timeout=420)
    if browser.returncode:
        return fail("E_SEAMGRIM_G13_BROWSER", browser.stdout.strip())
    if "seamgrim_workbench_four_viewport_recovery_gate: ok" not in browser.stdout:
        return fail("E_SEAMGRIM_G13_BROWSER_OK", browser.stdout.strip())

    summary_path = OUT / "summary.detjson"
    if not summary_path.is_file():
        return fail("E_SEAMGRIM_G13_SUMMARY", str(summary_path))
    summary = json.loads(summary_path.read_text(encoding="utf-8"))
    if summary.get("schema") != "seamgrim.workbench_four_viewport_recovery_gate.runner_result.v1":
        return fail("E_SEAMGRIM_G13_SCHEMA", repr(summary.get("schema")))
    expected = {"1440x900", "1024x768", "778x912", "390x844"}
    if set(summary.get("viewports", [])) != expected:
        return fail("E_SEAMGRIM_G13_VIEWPORTS", repr(summary.get("viewports")))
    for row in summary.get("results", []):
        screenshot = ROOT / row.get("screenshot", "")
        if not screenshot.is_file() or screenshot.stat().st_size <= 0:
            return fail("E_SEAMGRIM_G13_SCREENSHOT", str(screenshot))
        first = row.get("canvas", {}).get("first", {})
        second = row.get("canvas", {}).get("second", {})
        if first.get("non_white", 0) <= 500:
            return fail("E_SEAMGRIM_G13_NONBLANK", repr(row))
        if first.get("pixel_hash") == second.get("pixel_hash"):
            return fail("E_SEAMGRIM_G13_TIME_CHANGE", repr(row))
        if row.get("browser_issues"):
            return fail("E_SEAMGRIM_G13_BROWSER_ISSUES", repr(row.get("browser_issues")))

    ssot = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if ssot.returncode or ssot.stdout.strip():
        return fail("E_SEAMGRIM_G13_SSOT", ssot.stdout.strip())

    print("[seamgrim-workbench-four-viewport-recovery-gate-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
