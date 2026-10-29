#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "malblock_authoring_ui_v1"
RUNNER = ROOT / "tests" / "malblock_authoring_ui_browser_runner.mjs"


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
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "block_editor.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "block_editor" / "ddn_block_engine.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "block_editor" / "ddn_block_codec.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "block_editor" / "seamgrim_palette.js",
        ROOT / "tests" / "run_block_editor_roundtrip_check.py",
        ROOT / "tests" / "run_seamgrim_block_editor_smoke_check.py",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_MALBLOCK_AUTHORING_UI_MISSING", str(missing))
    return 0


def require_tokens(path: Path, tokens: list[str], code: str) -> int:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        return fail(code, f"{path.relative_to(ROOT)} missing {missing}")
    return 0


def check_product_markers() -> int:
    engine = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "block_editor" / "ddn_block_engine.js")
    for token in ["insertPaletteBlock(", "moveBlock(", "removeBlock(", "onChange?.(this.getBlocks())"]:
      if token not in engine:
          return fail("E_MALBLOCK_AUTHORING_UI_ENGINE", token)
    return 0


def run_pack_golden() -> int:
    proc = run([sys.executable, "tests/run_pack_golden.py", "malblock_authoring_ui_v1"], timeout=120)
    if proc.returncode != 0:
        return fail("E_MALBLOCK_AUTHORING_UI_PACK_GOLDEN", proc.stdout.strip())
    if "pack golden ok" not in proc.stdout:
        return fail("E_MALBLOCK_AUTHORING_UI_PACK_GOLDEN_OK", proc.stdout.strip())
    return 0


def check_pack_contract() -> int:
    payload = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "malblock_authoring_ui_v1",
        "kind": "studio_malblock_authoring_browser_smoke",
        "runtime_claim": False,
        "product_code_change": True,
        "closed_by": "MALBLOCK_AUTHORING_UI_V1",
        "browser_runner": "tests/malblock_authoring_ui_browser_runner.mjs",
        "next_item": "STUDIO_DIAGNOSTIC_FIXIT_PREVIEW_V1",
        "requires_docs_ssot_clean": True,
    }
    for key, value in expected.items():
        if payload.get(key) != value:
            return fail("E_MALBLOCK_AUTHORING_UI_CONTRACT", f"{key}={payload.get(key)!r}")
    covers = payload.get("covers")
    required = {
        "palette_grouping",
        "palette_insert",
        "top_level_delete",
        "top_level_reorder",
        "ddn_generation",
        "text_and_run_callbacks",
        "decode_error_display",
    }
    if not isinstance(covers, list) or not required.issubset(set(covers)):
        return fail("E_MALBLOCK_AUTHORING_UI_COVERS", repr(covers))
    return 0


def check_golden() -> int:
    payload = json.loads((PACK / "golden.jsonl").read_text(encoding="utf-8").strip())
    expected = [
        "MALBLOCK_AUTHORING_UI_V1",
        "studio malblock authoring browser smoke sealed",
        "next: STUDIO_DIAGNOSTIC_FIXIT_PREVIEW_V1",
    ]
    if payload.get("stdout") != expected:
        return fail("E_MALBLOCK_AUTHORING_UI_GOLDEN", repr(payload.get("stdout")))
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
            "E_MALBLOCK_AUTHORING_UI_PLAYWRIGHT",
            (proc.stdout or "").strip() or "run `npx playwright install chromium`",
        )
    return 0


def run_browser_smoke() -> int:
    proc = run(["node", "tests/malblock_authoring_ui_browser_runner.mjs"], timeout=120)
    if proc.returncode != 0:
        return fail("E_MALBLOCK_AUTHORING_UI_BROWSER", proc.stdout.strip())
    if "malblock_authoring_ui_browser: ok" not in proc.stdout:
        return fail("E_MALBLOCK_AUTHORING_UI_BROWSER_OK", proc.stdout.strip())
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_MALBLOCK_AUTHORING_UI_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_MALBLOCK_AUTHORING_UI_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    checks = (
        require_files,
        check_product_markers,
        check_pack_contract,
        check_golden,
        run_pack_golden,
        check_playwright_available,
        run_browser_smoke,
        check_docs_ssot_clean,
    )
    for check in checks:
        rc = check()
        if rc:
            return rc
    print("[malblock-authoring-ui-v1] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
