#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js"
CSS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.css"
RUNNER = ROOT / "tests" / "seamgrim_public_physics_change_storyboard_runner.mjs"


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
    js = JS.read_text(encoding="utf-8")
    css = CSS.read_text(encoding="utf-8")
    for token in ["sourceDirty", "source candidate differs from committed run", "activeSourceHash"]:
        if token not in js:
            return fail("E_PUBLIC_CHANGE_JS_TOKEN", token)
    if '[data-source-dirty="true"] .source-panel' not in css:
        return fail("E_PUBLIC_CHANGE_CSS_TOKEN", "source dirty visual boundary missing")
    return 0


def run_browser() -> int:
    proc = run(["node", str(RUNNER.relative_to(ROOT))], timeout=240)
    if proc.returncode != 0:
        return fail("E_PUBLIC_CHANGE_BROWSER", proc.stdout.strip())
    if "seamgrim_public_physics_change_storyboard: ok" not in proc.stdout:
        return fail("E_PUBLIC_CHANGE_BROWSER_OK", proc.stdout.strip())
    return 0


def require_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_PUBLIC_CHANGE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_PUBLIC_CHANGE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for path in (JS, CSS, RUNNER):
        if not path.exists():
            return fail("E_PUBLIC_CHANGE_FILE_MISSING", str(path.relative_to(ROOT)))
    for check in (require_product_tokens, run_browser, require_docs_ssot_clean):
        rc = check()
        if rc:
            return rc
    print("[seamgrim-public-physics-change-storyboard] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
