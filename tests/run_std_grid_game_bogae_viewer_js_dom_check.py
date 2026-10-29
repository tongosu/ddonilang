#!/usr/bin/env python3
"""Validate STD_GRID_GAME_BOGAE_VIEWER_JS_DOM_HARNESS_6DAY_V1 evidence."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
LIVE_INPUT = "std_grid_game_bogae_viewer_js_live_input_v1"
PLAYBACK = "std_grid_game_bogae_viewer_js_playback_controls_v1"
CLOSURE = "std_grid_game_bogae_viewer_js_dom_closure_v1"
LIVE_ASSETS = "std_grid_game_bogae_live_web_assets_v1"
LIVE_BRIDGE = "std_grid_game_bogae_live_bridge_closure_v1"
ALL_PACKS = [LIVE_INPUT, PLAYBACK, CLOSURE]
NODE_OK = "std_grid_game_bogae_viewer_js_dom: ok"


def fail(message: str) -> None:
    print(f"[std-grid-game-bogae-viewer-js-dom] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def require_dir(path: Path) -> None:
    if not path.is_dir():
        fail(f"missing dir: {path.relative_to(ROOT)}")


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


def run(args: list[str]) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        args,
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


def require_pack(pack: str, expected_stdout: list[str]) -> None:
    pack_dir = ROOT / "pack" / pack
    require_dir(pack_dir)
    require_file(pack_dir / "README.md")
    require_file(pack_dir / "input.ddn")
    require_file(pack_dir / "golden.jsonl")
    rows = load_jsonl(pack_dir / "golden.jsonl")
    if len(rows) != 1:
        fail(f"pack/{pack}/golden.jsonl must have exactly one row")
    if rows[0].get("stdout") != expected_stdout:
        fail(f"pack/{pack}/golden stdout mismatch")


def require_contract() -> None:
    contract = load_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if contract.get("schema") != "ddn.std_grid_game_bogae_viewer_js_dom_closure.pack.contract.v1":
        fail("closure contract schema mismatch")
    if contract.get("public_surface_delta") != "none":
        fail("closure public_surface_delta mismatch")
    if contract.get("bundled_packs") != [LIVE_INPUT, PLAYBACK, LIVE_BRIDGE]:
        fail("closure bundled_packs mismatch")
    harness = contract.get("harness")
    if not isinstance(harness, dict):
        fail("closure harness missing")
    if harness.get("runtime") != "node" or harness.get("minimum_major") != 22:
        fail("closure harness runtime mismatch")
    if harness.get("script") != "tests/std_grid_game_bogae_viewer_js_dom_runner.mjs":
        fail("closure harness script mismatch")
    if contract.get("generated_asset_dependency") != LIVE_ASSETS:
        fail("closure generated_asset_dependency mismatch")
    if "gaji/std_grid_game_bogae_viewer_js_dom" not in contract.get("gaji_skeletons", []):
        fail("closure missing gaji skeleton pointer")


def require_node_version() -> None:
    proc = run(["node", "--version"])
    text = proc.stdout.strip()
    if not text.startswith("v"):
        fail(f"unexpected node version: {text}")
    try:
        major = int(text[1:].split(".", 1)[0])
    except ValueError:
        fail(f"unparseable node version: {text}")
    if major < 22:
        fail(f"node v22+ required, got {text}")


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
    ]:
        require_file(out_dir / rel)
    manifest = load_json(out_dir / "manifest.detjson")
    if manifest.get("kind") != "bogae_web_playback_v1":
        fail("generated manifest kind mismatch")
    if manifest.get("codec") != "BDL1":
        fail("generated manifest codec mismatch")
    frames = manifest.get("frames")
    if not isinstance(frames, list) or len(frames) != 4:
        fail("generated manifest frame count mismatch")
    return out_dir


def main() -> None:
    require_file(ROOT / "STD_GRID_GAME_BOGAE_LIVE_INPUT_BRIDGE_7DAY_V1.md")
    require_file(ROOT / "STD_GRID_GAME_BOGAE_VIEWER_JS_DOM_HARNESS_6DAY_V1.md")
    require_file(ROOT / "tests" / "run_std_grid_game_bogae_live_bridge_check.py")
    require_file(ROOT / "tests" / "std_grid_game_bogae_viewer_js_dom_runner.mjs")
    require_file(ROOT / "pack" / LIVE_BRIDGE / "contract.detjson")
    require_file(ROOT / "gaji" / "std_grid_game_bogae_viewer_js_dom" / "README.md")
    require_file(ROOT / "gaji" / "std_grid_game_bogae_viewer_js_dom" / "example.ddn")

    require_pack(
        LIVE_INPUT,
        [
            "std_grid_game_bogae_viewer_js_live_input_v1",
            "node_fake_dom",
            "data-live=1",
            "input_url_port_placeholder",
            "keydown_arrowleft_down",
            "repeat_keydown_no_duplicate",
            "keyup_arrowleft_up",
            "blur_visibility_clear",
        ],
    )
    require_pack(
        PLAYBACK,
        [
            "std_grid_game_bogae_viewer_js_playback_controls_v1",
            "node_fake_dom",
            "index_without_data_live",
            "btn_step_forward_loadframe",
            "btn_step_back_loadframe",
            "seek_loadframe",
            "overlay_toggles_render",
        ],
    )
    require_pack(
        CLOSURE,
        [
            "std_grid_game_bogae_viewer_js_dom_closure_v1",
            "node_v22_fake_dom",
            "bogae_web_playback_v1",
            "BDL1",
            "keydown_keyup_blur_visibilitychange",
            "step_seek_overlay",
            "no real browser claim",
        ],
    )
    require_contract()
    require_node_version()

    run([sys.executable, "tests/run_pack_golden.py", LIVE_ASSETS])
    out_dir = require_generated_assets()
    node_proc = run(["node", "tests/std_grid_game_bogae_viewer_js_dom_runner.mjs", str(out_dir)])
    if node_proc.stdout.strip() != NODE_OK:
        fail(f"node runner stdout mismatch: {node_proc.stdout.strip()}")
    run([sys.executable, "tests/run_pack_golden.py", *ALL_PACKS])
    print("[std-grid-game-bogae-viewer-js-dom] OK")


if __name__ == "__main__":
    main()
