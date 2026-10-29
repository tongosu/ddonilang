#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SERVER = ROOT / "solutions" / "seamgrim_ui_mvp" / "tools" / "ddn_exec_server.py"
RUNNER = ROOT / "tests" / "seamgrim_public_default_route_runner.mjs"


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
    text = SERVER.read_text(encoding="utf-8")
    required = [
        "WORKBENCH_DEFAULT_ROUTE",
        "DEFAULT_SHELL_PATHS",
        "LEGACY_SHELL_PATHS",
        "LEGACY_DEEP_LINK_KEYS",
        "CAPABILITY_QUERY_KEYS",
        "send_redirect",
        "_wants_legacy_shell",
        "_safe_default_route_location",
    ]
    for token in required:
        if token not in text:
            return fail("E_PUBLIC_DEFAULT_ROUTE_SERVER_TOKEN", token)
    if 'send_file(self, UI_DIR / "index.html")' not in text:
        return fail("E_PUBLIC_DEFAULT_ROUTE_LEGACY_FILE", "legacy shell file serving boundary missing")
    return 0


def run_browser() -> int:
    proc = run(["node", str(RUNNER.relative_to(ROOT))], timeout=240)
    if proc.returncode != 0:
        return fail("E_PUBLIC_DEFAULT_ROUTE_BROWSER", proc.stdout.strip())
    if "seamgrim_public_default_route: ok" not in proc.stdout:
        return fail("E_PUBLIC_DEFAULT_ROUTE_BROWSER_OK", proc.stdout.strip())
    return 0


def require_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_PUBLIC_DEFAULT_ROUTE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_PUBLIC_DEFAULT_ROUTE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    if not SERVER.exists() or not RUNNER.exists():
        return fail("E_PUBLIC_DEFAULT_ROUTE_FILES", "server or browser runner missing")
    for check in (require_product_tokens, run_browser, require_docs_ssot_clean):
        rc = check()
        if rc:
            return rc
    print("[seamgrim-public-default-route] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
