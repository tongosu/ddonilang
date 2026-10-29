#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / "tests" / "seamgrim_public_share_clone_reopen_runner.mjs"
PROJECT_ADAPTER = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "persistence" / "project_adapter.js"
CAPABILITY = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "persistence" / "capability_session.js"
WORKBENCH = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js"


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
    combined = "\n".join(path.read_text(encoding="utf-8") for path in (PROJECT_ADAPTER, CAPABILITY, WORKBENCH))
    for token in [
        "buildWorkbenchCapabilityLink",
        "E_PROJECT_VIEW_WRITE_REJECTED",
        "cloneServerProject",
        "rememberMyWorkProject",
        "openProjectFromServerSession",
        "server_run_id",
    ]:
        if token not in combined:
            return fail("E_PUBLIC_SHARE_PRODUCT_TOKEN", token)
    return 0


def run_browser() -> int:
    proc = run(["node", str(RUNNER.relative_to(ROOT))], timeout=300)
    if proc.returncode != 0:
        return fail("E_PUBLIC_SHARE_BROWSER", proc.stdout.strip())
    if "seamgrim_public_share_clone_reopen: ok" not in proc.stdout:
        return fail("E_PUBLIC_SHARE_BROWSER_OK", proc.stdout.strip())
    return 0


def require_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_PUBLIC_SHARE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_PUBLIC_SHARE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    if not RUNNER.exists():
        return fail("E_PUBLIC_SHARE_RUNNER_MISSING", str(RUNNER.relative_to(ROOT)))
    for check in (require_product_tokens, run_browser, require_docs_ssot_clean):
        rc = check()
        if rc:
            return rc
    print("[seamgrim-public-share-clone-reopen] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
