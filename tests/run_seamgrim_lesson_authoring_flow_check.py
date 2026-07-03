#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_lesson_authoring_flow_v1"
RUNNER = ROOT / "tests" / "seamgrim_lesson_authoring_flow_browser_runner.mjs"


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


def require_files() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RUNNER,
        ROOT / "package.json",
        ROOT / "package-lock.json",
        ROOT / "pack" / "seamgrim_workbench_shell_v1" / "contract.detjson",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "browse.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "editor.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "lesson_loader_contract.js",
        ROOT / "tests" / "run_seamgrim_workbench_shell_check.py",
        ROOT / "tests" / "seamgrim_lesson_loader_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_LESSON_AUTHORING_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_pack_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_lesson_authoring_flow_v1",
        "kind": "studio_lesson_authoring_browser_smoke",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "SEAMGRIM_LESSON_AUTHORING_FLOW_V1",
        "browser_runner": "tests/seamgrim_lesson_authoring_flow_browser_runner.mjs",
        "next_item": "MALBLOCK_AUTHORING_UI_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_SEAMGRIM_LESSON_AUTHORING_CONTRACT", f"{key}={payload.get(key)!r}")
    covers = payload.get("covers")
    required = {
        "direct_ddn_route_to_workbench",
        "run_editor_text_edit",
        "local_save_action_contract",
        "browse_create_authoring_entry",
        "lesson_loader_contract_reuse",
    }
    if not isinstance(covers, list) or not required.issubset(set(covers)):
        return fail("E_SEAMGRIM_LESSON_AUTHORING_COVERS", repr(covers))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "SEAMGRIM_LESSON_AUTHORING_FLOW_V1",
        "studio lesson authoring flow browser smoke sealed",
        "next: MALBLOCK_AUTHORING_UI_V1",
    ]
    if payload.get("stdout") != expected:
        return fail("E_SEAMGRIM_LESSON_AUTHORING_GOLDEN", repr(payload.get("stdout")))
    return 0


def check_playwright_available() -> int:
    proc = run(
        [
            "node",
            "-e",
            "const { chromium } = require('playwright'); chromium.launch({headless:true}).then(b=>b.close()).catch(e=>{console.error(e.message); process.exit(1)})",
        ],
        timeout=60,
    )
    if proc.returncode != 0:
        return fail(
            "E_SEAMGRIM_LESSON_AUTHORING_PLAYWRIGHT",
            (proc.stdout or "").strip() or "run `npx playwright install chromium`",
        )
    return 0


def run_browser_smoke() -> int:
    proc = run(["node", "tests/seamgrim_lesson_authoring_flow_browser_runner.mjs"], timeout=120)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_LESSON_AUTHORING_BROWSER", proc.stdout.strip())
    if "seamgrim_lesson_authoring_flow_browser: ok" not in proc.stdout:
        return fail("E_SEAMGRIM_LESSON_AUTHORING_BROWSER_OK", proc.stdout.strip())
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_LESSON_AUTHORING_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_LESSON_AUTHORING_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_pack_contract,
        check_golden,
        check_playwright_available,
        run_browser_smoke,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[seamgrim-lesson-authoring-flow-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
