#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "tests" / "seamgrim_public_four_viewport_quality_runner.mjs"
OUT = ROOT / "out" / "seamgrim_public_wedge_g8"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def run(cmd: list[str], *, timeout: int = 180) -> subprocess.CompletedProcess[str]:
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


def run_browser() -> int:
    proc = run(["node", str(RUNNER.relative_to(ROOT))], timeout=360)
    if proc.returncode != 0:
        return fail("E_FOUR_VIEWPORT_BROWSER", proc.stdout.strip())
    if "seamgrim_public_four_viewport_quality: ok" not in proc.stdout:
        return fail("E_FOUR_VIEWPORT_BROWSER_OK", proc.stdout.strip())
    summary = OUT / "summary.detjson"
    if not summary.is_file():
        return fail("E_FOUR_VIEWPORT_SUMMARY", "summary.detjson missing")
    payload = json.loads(summary.read_text(encoding="utf-8"))
    if payload.get("schema") != "seamgrim.public_four_viewport_quality.runner_result.v1":
        return fail("E_FOUR_VIEWPORT_SCHEMA", repr(payload.get("schema")))
    if len(payload.get("results", [])) != 4:
        return fail("E_FOUR_VIEWPORT_COUNT", repr(len(payload.get("results", []))))
    for row in payload["results"]:
        screenshot = Path(row.get("screenshot", ""))
        if not screenshot.is_file() or screenshot.stat().st_size <= 0:
            return fail("E_FOUR_VIEWPORT_SCREENSHOT", str(screenshot))
        metrics = row.get("metrics", {})
        if metrics.get("nonBackground", 0) <= 200:
            return fail("E_FOUR_VIEWPORT_PIXELS", repr(row))
    return 0


def require_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_FOUR_VIEWPORT_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_FOUR_VIEWPORT_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    if not RUNNER.exists():
        return fail("E_FOUR_VIEWPORT_RUNNER_MISSING", str(RUNNER.relative_to(ROOT)))
    syntax = run(["node", "--check", str(RUNNER.relative_to(ROOT))], timeout=60)
    if syntax.returncode != 0:
        return fail("E_FOUR_VIEWPORT_RUNNER_SYNTAX", syntax.stdout)
    for check in (run_browser, require_docs_ssot_clean):
        rc = check()
        if rc:
            return rc
    print("[seamgrim-public-four-viewport-quality] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
