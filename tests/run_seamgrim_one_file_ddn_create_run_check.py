#!/usr/bin/env python
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HTML = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "index.html"
JS = ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "workbench" / "workbench.js"
RUNNER = ROOT / "tests" / "seamgrim_one_file_ddn_create_run_runner.mjs"


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
    html = HTML.read_text(encoding="utf-8")
    js = JS.read_text(encoding="utf-8")
    for token in ["new-ddn-project", "project-create-dialog", 'value="single"', "한 파일 시작본"]:
        if token not in html:
            return fail("E_ONE_FILE_DDN_HTML_TOKEN", token)
    for token in [
        "defaultOneFileDdnSource",
        'template === "single"',
        "buildNativeProject",
        "createNativeProject",
        "W_NATIVE_PROJECT_CREATED",
        "W_NATIVE_PROJECT_REOPENED",
        "E_SOURCE_EMPTY",
        "local_recovery",
    ]:
        if token not in js:
            return fail("E_ONE_FILE_DDN_JS_TOKEN", token)
    runner = RUNNER.read_text(encoding="utf-8")
    for token in ["project-create-dialog", "value='${template}'", "native_creator", "creation_template", "E_SOURCE_EMPTY"]:
        if token not in runner:
            return fail("E_ONE_FILE_DDN_RUNNER_TOKEN", token)
    if "local_one_file_ddn_v1" in runner:
        return fail("E_ONE_FILE_DDN_FIXED_ID_CONTRACT", "runner must accept native dynamic ids")
    return 0


def run_browser() -> int:
    proc = run(["node", str(RUNNER.relative_to(ROOT))], timeout=240)
    if proc.returncode != 0:
        return fail("E_ONE_FILE_DDN_BROWSER", proc.stdout.strip())
    if "seamgrim_one_file_ddn_create_run: ok" not in proc.stdout:
        return fail("E_ONE_FILE_DDN_BROWSER_OK", proc.stdout.strip())
    return 0


def require_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_ONE_FILE_DDN_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_ONE_FILE_DDN_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for path in (HTML, JS, RUNNER):
        if not path.exists():
            return fail("E_ONE_FILE_DDN_FILE_MISSING", str(path.relative_to(ROOT)))
    for check in (require_product_tokens, run_browser, require_docs_ssot_clean):
        rc = check()
        if rc:
            return rc
    print("[seamgrim-one-file-ddn-create-run] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
