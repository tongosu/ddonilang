#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_workbench_polish_v2"
RUNNER = ROOT / "tests" / "seamgrim_workbench_polish_runner.mjs"
NEXT = "SEAMGRIM_LESSON_LIBRARY_CURATION_V1"


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
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_WORKBENCH_POLISH_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_product_tokens() -> int:
    checks = [
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html",
            [
                "data-shell-status-rail",
                "data-shell-current-screen",
                "data-shell-source-label",
                "data-shell-save-status",
                "data-shell-session-status",
                "id=\"screen-browse\"",
                "id=\"screen-run\"",
            ],
            "E_SEAMGRIM_WORKBENCH_POLISH_HTML",
        ),
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js",
            [
                "updateShellStatusRail",
                "buildShellStatusModel",
                "resolveShellSessionStatus",
                "localSaveStatus",
                "로컬 저장됨",
            ],
            "E_SEAMGRIM_WORKBENCH_POLISH_APP",
        ),
        (
            ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
            [
                ".shell-status-rail",
                ".shell-status-chip",
                "data-status=\"restored\"",
                "data-status=\"saved\"",
            ],
            "E_SEAMGRIM_WORKBENCH_POLISH_CSS",
        ),
    ]
    for path, tokens, code in checks:
        rc = require_tokens(path, tokens, code)
        if rc:
            return rc
    return 0


def check_pack_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_workbench_polish_v2",
        "kind": "studio_workbench_polish",
        "runtime_claim": False,
        "product_code_change": True,
        "closed_by": "SEAMGRIM_WORKBENCH_POLISH_V2",
        "browser_runner": "tests/seamgrim_workbench_polish_runner.mjs",
        "based_on": "STUDIO_PRIVATE_PRODUCTIZATION_REBASE_V1",
        "next_item": NEXT,
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_SEAMGRIM_WORKBENCH_POLISH_CONTRACT", f"{key}={payload.get(key)!r}")
    covers = payload.get("covers")
    if not isinstance(covers, list) or "local_save_status_sync" not in covers:
        return fail("E_SEAMGRIM_WORKBENCH_POLISH_COVERS", repr(covers))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "SEAMGRIM_WORKBENCH_POLISH_V2",
        "studio workbench polish status rail sealed",
        f"next: {NEXT}",
    ]
    if payload.get("stdout") != expected:
        return fail("E_SEAMGRIM_WORKBENCH_POLISH_GOLDEN", repr(payload.get("stdout")))
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
            "E_SEAMGRIM_WORKBENCH_POLISH_PLAYWRIGHT",
            (proc.stdout or "").strip() or "run `npx playwright install chromium`",
        )
    return 0


def run_browser_smoke() -> int:
    proc = run(["node", "tests/seamgrim_workbench_polish_runner.mjs"], timeout=120)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_WORKBENCH_POLISH_BROWSER", proc.stdout.strip())
    if "seamgrim_workbench_polish: ok" not in proc.stdout:
        return fail("E_SEAMGRIM_WORKBENCH_POLISH_BROWSER_OK", proc.stdout.strip())
    return 0


def run_required_gates() -> int:
    commands = [
        ["python", "tests/run_pack_golden.py", "seamgrim_workbench_polish_v2"],
    ]
    for cmd in commands:
        proc = run(cmd, timeout=180)
        if proc.returncode != 0:
            return fail("E_SEAMGRIM_WORKBENCH_POLISH_GATE_FAILED", f"{' '.join(cmd)}: {proc.stdout.strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_WORKBENCH_POLISH_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_WORKBENCH_POLISH_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_product_tokens,
        check_pack_contract,
        check_golden,
        check_playwright_available,
        run_browser_smoke,
        run_required_gates,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[seamgrim-workbench-polish-v2] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
