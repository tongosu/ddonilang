#!/usr/bin/env python3
"""Validate STD_GRID_GAME_BOGAE_BROWSER_DOM_SMOKE_V1 evidence."""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = "std_grid_game_bogae_browser_dom_smoke_v1"
LIVE_ASSETS = "std_grid_game_bogae_live_web_assets_v1"
NODE_OK = "std_grid_game_bogae_browser_dom_smoke: ok"


def fail(message: str) -> None:
    print(f"[std-grid-game-bogae-browser-dom] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def require_dir(path: Path) -> None:
    if not path.is_dir():
        fail(f"missing dir: {path.relative_to(ROOT)}")


def run(args: list[str]) -> subprocess.CompletedProcess[str]:
    resolved = shutil.which(args[0]) or args[0]
    proc = subprocess.run(
        [resolved, *args[1:]],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if proc.returncode != 0:
        print(proc.stdout, end="")
        print(proc.stderr, end="", file=sys.stderr)
        fail(f"command failed: {' '.join(args)}")
    return proc


def load_json(path: Path) -> dict:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")
    if not isinstance(data, dict):
        fail(f"{path.relative_to(ROOT)} root must be object")
    return data


def load_jsonl(path: Path) -> list[dict]:
    rows: list[dict] = []
    for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.strip():
            continue
        try:
            row = json.loads(line)
        except json.JSONDecodeError as exc:
            fail(f"{path.relative_to(ROOT)}:{lineno}: invalid JSONL: {exc}")
        if not isinstance(row, dict):
            fail(f"{path.relative_to(ROOT)}:{lineno}: row must be object")
        rows.append(row)
    return rows


def require_node_and_playwright() -> None:
    node = run(["node", "--version"]).stdout.strip()
    if not node.startswith("v"):
        fail(f"unexpected node version: {node}")
    major = int(node[1:].split(".", 1)[0])
    if major < 22:
        fail(f"node v22+ required, got {node}")
    npm = run(["npm", "--version"]).stdout.strip()
    if not npm:
        fail("npm version is empty")
    pkg = load_json(ROOT / "package.json")
    dev_deps = pkg.get("devDependencies")
    if not isinstance(dev_deps, dict) or dev_deps.get("playwright") != "1.44.0":
        fail("package.json must pin playwright to 1.44.0")
    require_file(ROOT / "package-lock.json")
    resolved = run(["node", "-e", "console.log(require.resolve('playwright'))"]).stdout.strip()
    if "node_modules" not in resolved.replace("\\", "/"):
        fail(f"unexpected playwright resolve path: {resolved}")


def require_generated_assets() -> Path:
    out_dir = ROOT / "build" / LIVE_ASSETS
    require_dir(out_dir)
    for rel in [
        "manifest.detjson",
        "frames/000000.bdl1.detbin",
        "frames/000001.bdl1.detbin",
        "frames/000002.bdl1.detbin",
        "frames/000003.bdl1.detbin",
        "viewer/index.html",
        "viewer/live.html",
        "viewer/viewer.js",
        "viewer/overlay.detjson",
        "viewer/skin.detjson",
    ]:
        require_file(out_dir / rel)
    manifest = load_json(out_dir / "manifest.detjson")
    if manifest.get("kind") != "bogae_web_playback_v1":
        fail("generated manifest kind mismatch")
    if manifest.get("codec") != "BDL1":
        fail("generated manifest codec mismatch")
    if len(manifest.get("frames", [])) != 4:
        fail("generated manifest frame count mismatch")
    return out_dir


def require_pack() -> None:
    pack_dir = ROOT / "pack" / PACK
    require_dir(pack_dir)
    require_file(pack_dir / "README.md")
    require_file(pack_dir / "input.ddn")
    require_file(pack_dir / "golden.jsonl")
    require_file(pack_dir / "contract.detjson")
    rows = load_jsonl(pack_dir / "golden.jsonl")
    expected = [
        "std_grid_game_bogae_browser_dom_smoke_v1",
        "playwright_chromium",
        "index_live_canvas",
        "playback_controls_overlay",
        "no_keyboard_delivery_claim",
    ]
    if len(rows) != 1 or rows[0].get("stdout") != expected:
        fail("browser dom smoke golden stdout mismatch")
    contract = load_json(pack_dir / "contract.detjson")
    if contract.get("schema") != "ddn.std_grid_game_bogae_browser_dom_smoke.pack.contract.v1":
        fail("browser dom smoke contract schema mismatch")
    if contract.get("playwright_version") != "1.44.0":
        fail("browser dom smoke contract playwright_version mismatch")
    if contract.get("runner") != "tests/std_grid_game_bogae_browser_dom_smoke_runner.mjs":
        fail("browser dom smoke contract runner mismatch")
    if contract.get("generated_asset_dependency") != LIVE_ASSETS:
        fail("browser dom smoke generated_asset_dependency mismatch")


def main() -> None:
    require_file(ROOT / "NEXT_DEV_ROADMAP_AFTER_VIEWER_JS_DOM_V2.md")
    require_file(ROOT / "STD_GRID_GAME_BOGAE_BROWSER_DOM_SMOKE_V1.md")
    require_file(ROOT / "tests" / "run_std_grid_game_bogae_viewer_js_dom_check.py")
    require_file(ROOT / "tests" / "std_grid_game_bogae_browser_dom_smoke_runner.mjs")
    require_file(ROOT / "pack" / "std_grid_game_bogae_viewer_js_dom_closure_v1" / "contract.detjson")
    require_file(ROOT / "pack" / LIVE_ASSETS / "golden.jsonl")
    require_file(ROOT / "gaji" / "std_grid_game_bogae_browser_dom_smoke" / "README.md")
    require_file(ROOT / "gaji" / "std_grid_game_bogae_browser_dom_smoke" / "example.ddn")
    require_node_and_playwright()
    require_pack()

    run([sys.executable, "tests/run_pack_golden.py", LIVE_ASSETS])
    out_dir = require_generated_assets()
    node_proc = run(["node", "tests/std_grid_game_bogae_browser_dom_smoke_runner.mjs", str(out_dir)])
    if node_proc.stdout.strip() != NODE_OK:
        fail(f"node runner stdout mismatch: {node_proc.stdout.strip()}")
    run([sys.executable, "tests/run_pack_golden.py", PACK])
    print("[std-grid-game-bogae-browser-dom] OK")


if __name__ == "__main__":
    main()
