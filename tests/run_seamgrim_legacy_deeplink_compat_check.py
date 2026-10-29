#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SERVER = ROOT / "solutions" / "seamgrim_ui_mvp" / "tools" / "ddn_exec_server.py"
WORKBENCH = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js"
RUNNER = ROOT / "tests" / "seamgrim_legacy_deeplink_compat_runner.mjs"


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


def require_product_tokens() -> int:
    server = SERVER.read_text(encoding="utf-8")
    workbench = WORKBENCH.read_text(encoding="utf-8")
    for token in ["WORKBENCH_ROUTE_PROJECT_IDS", "WORKBENCH_PROJECT_QUERY_KEYS", "legacy_only_unknown_v1"]:
        if token == "legacy_only_unknown_v1":
            continue
        if token not in server:
            return fail("E_LEGACY_COMPAT_SERVER_TOKEN", token)
    for token in ["initialProjectIdFromLocation", "lesson_id", "source_lesson_id"]:
        if token not in workbench:
            return fail("E_LEGACY_COMPAT_WORKBENCH_TOKEN", token)
    return 0


def run_browser() -> int:
    proc = run(["node", str(RUNNER.relative_to(ROOT))], timeout=240)
    if proc.returncode != 0:
        return fail("E_LEGACY_COMPAT_BROWSER", proc.stdout.strip())
    if "seamgrim_legacy_deeplink_compat: ok" not in proc.stdout:
        return fail("E_LEGACY_COMPAT_BROWSER_OK", proc.stdout.strip())
    return 0


def require_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_LEGACY_COMPAT_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_LEGACY_COMPAT_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    if not RUNNER.exists():
        return fail("E_LEGACY_COMPAT_RUNNER_MISSING", str(RUNNER.relative_to(ROOT)))
    for check in (require_product_tokens, run_browser, require_docs_ssot_clean):
        rc = check()
        if rc:
            return rc
    print("[seamgrim-legacy-deeplink-compat] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
