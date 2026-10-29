#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_workbench_shell_v1"
RUNNER = ROOT / "tests" / "seamgrim_workbench_shell_browser_runner.mjs"


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
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "browse.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "editor.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "lesson_loader_contract.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_edit_run_contract.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "active_allowlist.detjson",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "seed_lessons_v1" / "seed_manifest.detjson",
        ROOT / "tests" / "run_seamgrim_product_stabilization_smoke_check.py",
        ROOT / "tests" / "seamgrim_ui_common_runner.mjs",
        ROOT / "tests" / "seamgrim_studio_layout_contract_runner.mjs",
        ROOT / "tests" / "run_seamgrim_live_repl_check.py",
        ROOT / "tests" / "run_seamgrim_wasm_smoke.py",
        ROOT / "tests" / "run_block_editor_roundtrip_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_WORKBENCH_SHELL_MISSING", str(missing))
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
        "pack": "seamgrim_workbench_shell_v1",
        "kind": "studio_browser_shell_smoke",
        "runtime_claim": False,
        "product_code_change": False,
        "closed_by": "SEAMGRIM_WORKBENCH_SHELL_V1",
        "browser_runner": "tests/seamgrim_workbench_shell_browser_runner.mjs",
        "next_item": "SEAMGRIM_LESSON_AUTHORING_FLOW_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_SEAMGRIM_WORKBENCH_SHELL_CONTRACT", f"{key}={payload.get(key)!r}")
    covers = payload.get("covers")
    if not isinstance(covers, list) or "workbench_run_shell_open" not in covers:
        return fail("E_SEAMGRIM_WORKBENCH_SHELL_COVERS", repr(covers))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "SEAMGRIM_WORKBENCH_SHELL_V1",
        "studio workbench shell browser smoke sealed",
        "next: SEAMGRIM_LESSON_AUTHORING_FLOW_V1",
    ]
    if payload.get("stdout") != expected:
        return fail("E_SEAMGRIM_WORKBENCH_SHELL_GOLDEN", repr(payload.get("stdout")))
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
            "E_SEAMGRIM_WORKBENCH_SHELL_PLAYWRIGHT",
            (proc.stdout or "").strip() or "run `npx playwright install chromium`",
        )
    return 0


def run_browser_smoke() -> int:
    proc = run(["node", "tests/seamgrim_workbench_shell_browser_runner.mjs"], timeout=120)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_WORKBENCH_SHELL_BROWSER", proc.stdout.strip())
    if "seamgrim_workbench_shell_browser: ok" not in proc.stdout:
        return fail("E_SEAMGRIM_WORKBENCH_SHELL_BROWSER_OK", proc.stdout.strip())
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_WORKBENCH_SHELL_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_WORKBENCH_SHELL_SSOT_DIRTY", proc.stdout.strip())
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
    print("[seamgrim-workbench-shell-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
