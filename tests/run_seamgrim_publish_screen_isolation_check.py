#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_publish_screen_isolation_v1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


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


def check_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "seamgrim_publish_screen_isolation_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_PUBLISH_ISOLATION_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_publish_screen_isolation_v1",
        "kind": "seamgrim_publish_screen_isolation",
        "product_behavior_claim": True,
        "parser_change": False,
        "runtime_semantics_change": False,
        "state_hash_formula_change": False,
        "docs_ssot_change": False,
        "public_upload_claim": False,
        "browser_runner": "tests/seamgrim_publish_screen_isolation_runner.mjs",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_SEAMGRIM_PUBLISH_ISOLATION_CONTRACT", f"{key}={contract.get(key)!r}")
    covers = set(contract.get("covers") or [])
    required_covers = {
        "independent_screen_publish",
        "open_paste_copy_download_actions",
        "desktop_tablet_mobile_smoke",
        "no_browse_space_publish_state",
    }
    if not required_covers.issubset(covers):
        return fail("E_SEAMGRIM_PUBLISH_ISOLATION_COVERS", repr(sorted(covers)))
    return 0


def check_sources() -> int:
    html = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html")
    app = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js")
    browse = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "browse.js")
    css = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css")
    for token in [
        'id="screen-publish"',
        'data-publish-screen',
        'id="publish-panel"',
        'data-publish-panel',
        'id="btn-open-local-package"',
        'id="btn-paste-local-package"',
        'id="btn-copy-course-package"',
        'id="btn-download-course-package"',
    ]:
        if token not in html:
            return fail("E_SEAMGRIM_PUBLISH_ISOLATION_HTML", token)
    if 'id="browse-publish-panel"' in html:
        return fail("E_SEAMGRIM_PUBLISH_ISOLATION_HTML_LEGACY", "browse-publish-panel")
    for token in ["MAIN_TAB_PUBLISH", 'setScreen("publish"', 'byId("screen-publish")', 'publishRoot: byId("screen-publish")']:
        if token not in app:
            return fail("E_SEAMGRIM_PUBLISH_ISOLATION_APP", token)
    if "browse-space-publish" in app or "browse-space-publish" in css:
        return fail("E_SEAMGRIM_PUBLISH_ISOLATION_LEAK", "browse-space-publish")
    if "this.publishRoot = publishRoot || root" not in browse:
        return fail("E_SEAMGRIM_PUBLISH_ISOLATION_BINDING", "publishRoot fallback missing")
    if "#screen-publish" not in css:
        return fail("E_SEAMGRIM_PUBLISH_ISOLATION_CSS", "#screen-publish")
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_PUBLISH_ISOLATION_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_PUBLISH_ISOLATION_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_contract, check_sources):
        rc = check()
        if rc:
            return rc
    rc = run_gate(["node", "tests/seamgrim_publish_screen_isolation_runner.mjs"], "E_SEAMGRIM_PUBLISH_ISOLATION_BROWSER", 180)
    if rc:
        return rc
    rc = run_gate(["python", "tests/run_pack_golden.py", "seamgrim_publish_screen_isolation_v1"], "E_SEAMGRIM_PUBLISH_ISOLATION_PACK", 120)
    if rc:
        return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-publish-screen-isolation-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
