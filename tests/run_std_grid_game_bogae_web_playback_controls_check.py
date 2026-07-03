#!/usr/bin/env python3
"""Validate STD_GRID_GAME_BOGAE_WEB_PLAYBACK_CONTROLS_6DAY_V1 evidence."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]

ASSETS = "std_grid_game_bogae_web_playback_assets_v1"
CONTROLS = "std_grid_game_bogae_web_playback_controls_v1"
CLOSURE = "std_grid_game_bogae_web_playback_closure_v1"
SHOWCASE_CLOSURE = "std_grid_game_bogae_web_showcase_closure_v1"
ALL_PACKS = [ASSETS, CONTROLS, CLOSURE]

EXPECTED_WEB_STDOUT = [
    "bogae_hash=blake3:dc4856b1baf4c5217660788764bfe0c81be70644c3260311f766b55e340387e2 cmd_count=65 codec=BDL1"
]
EXPECTED_CLOSURE_STDOUT = [
    "32",
    "32",
    "4",
    "BDL1",
    "bogae_web_playback_v1",
    "btn-play",
    "btn-step-back",
    "btn-step-forward",
    "seek",
    "ov-grid",
    "status",
]
REQUIRED_ASSETS = [
    "manifest.detjson",
    "frames/000000.bdl1.detbin",
    "frames/000001.bdl1.detbin",
    "frames/000002.bdl1.detbin",
    "frames/000003.bdl1.detbin",
    "viewer/index.html",
    "viewer/live.html",
    "viewer/viewer.js",
    "viewer/overlay.detjson",
]
CONTROL_IDS = [
    "btn-play",
    "btn-step-back",
    "btn-step-forward",
    "seek",
    "ov-grid",
    "ov-bounds",
    "ov-delta",
    "status",
]


def fail(message: str) -> None:
    print(f"[std-grid-game-bogae-web-playback] FAIL: {message}", file=sys.stderr)
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
        fail(f"{path.relative_to(ROOT)}: invalid JSON: {exc}")
    if not isinstance(data, dict):
        fail(f"{path.relative_to(ROOT)}: JSON root must be object")
    return data


def load_jsonl(path: Path) -> list[dict]:
    rows = []
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


def require_pack(pack: str) -> None:
    pack_dir = ROOT / "pack" / pack
    require_dir(pack_dir)
    require_file(pack_dir / "README.md")
    require_file(pack_dir / "input.ddn")
    require_file(pack_dir / "golden.jsonl")
    rows = load_jsonl(pack_dir / "golden.jsonl")
    if len(rows) != 1:
        fail(f"pack/{pack}/golden.jsonl must have exactly one row")
    expected_stdout = EXPECTED_CLOSURE_STDOUT if pack == CLOSURE else EXPECTED_WEB_STDOUT
    if rows[0].get("stdout") != expected_stdout:
        fail(f"pack/{pack}/golden stdout mismatch")


def require_contract() -> None:
    contract_path = ROOT / "pack" / CLOSURE / "contract.detjson"
    require_file(contract_path)
    contract = load_json(contract_path)
    if contract.get("schema") != "ddn.std_grid_game_bogae_web_playback_closure.pack.contract.v1":
        fail("closure contract schema mismatch")
    if contract.get("public_surface_delta") != "none":
        fail("closure public_surface_delta mismatch")
    if contract.get("bundled_packs") != [ASSETS, CONTROLS, SHOWCASE_CLOSURE]:
        fail("closure bundled_packs mismatch")
    if contract.get("manifest") != {"kind": "bogae_web_playback_v1", "codec": "BDL1"}:
        fail("closure manifest contract mismatch")
    expected_sample = {
        "grid_width": 8,
        "grid_height": 8,
        "cell_px": 4,
        "canvas_w": 32,
        "canvas_h": 32,
        "frame_count": 4,
    }
    if contract.get("web_sample") != expected_sample:
        fail("closure web_sample mismatch")
    if contract.get("required_assets") != REQUIRED_ASSETS:
        fail("closure required_assets mismatch")
    if contract.get("controls") != CONTROL_IDS:
        fail("closure controls mismatch")
    if "gaji/std_grid_game_bogae_web_playback" not in contract.get("gaji_skeletons", []):
        fail("closure missing gaji skeleton pointer")


def require_generated_assets(pack: str) -> None:
    out_dir = ROOT / "build" / pack
    require_dir(out_dir)
    for rel in REQUIRED_ASSETS:
        require_file(out_dir / rel)

    manifest = load_json(out_dir / "manifest.detjson")
    if manifest.get("kind") != "bogae_web_playback_v1":
        fail(f"{pack}: manifest kind mismatch")
    if manifest.get("codec") != "BDL1":
        fail(f"{pack}: manifest codec mismatch")
    if manifest.get("start_madi") != 0 or manifest.get("end_madi") != 4:
        fail(f"{pack}: manifest madi range mismatch")
    frames = manifest.get("frames")
    if not isinstance(frames, list) or len(frames) != 4:
        fail(f"{pack}: expected 4 frames")
    for idx, frame in enumerate(frames):
        if not isinstance(frame, dict):
            fail(f"{pack}: frame {idx} must be object")
        expected_file = f"frames/{idx:06d}.bdl1.detbin"
        if frame.get("madi") != idx:
            fail(f"{pack}: frame {idx} madi mismatch")
        if frame.get("file") != expected_file:
            fail(f"{pack}: frame {idx} file mismatch")
        if frame.get("cmd_count") != 65:
            fail(f"{pack}: frame {idx} cmd_count mismatch")
        if not str(frame.get("state_hash", "")).startswith("blake3:"):
            fail(f"{pack}: frame {idx} state_hash missing")
        if not str(frame.get("bogae_hash", "")).startswith("blake3:"):
            fail(f"{pack}: frame {idx} bogae_hash missing")

    index_html = (out_dir / "viewer" / "index.html").read_text(encoding="utf-8")
    live_html = (out_dir / "viewer" / "live.html").read_text(encoding="utf-8")
    viewer_js = (out_dir / "viewer" / "viewer.js").read_text(encoding="utf-8")
    overlay = load_json(out_dir / "viewer" / "overlay.detjson")
    if not isinstance(overlay, dict):
        fail(f"{pack}: overlay root must be object")

    if "Bogae Playback Viewer" not in index_html:
        fail(f"{pack}: index title mismatch")
    if "data-live=\"1\"" in index_html:
        fail(f"{pack}: index.html must remain static playback")
    if "Bogae Live Viewer" not in live_html or "data-live=\"1\"" not in live_html:
        fail(f"{pack}: live.html live marker mismatch")
    for required in CONTROL_IDS:
        if required not in index_html:
            fail(f"{pack}: index.html missing control {required}")
        if required not in live_html:
            fail(f"{pack}: live.html missing control {required}")
        if required not in viewer_js:
            fail(f"{pack}: viewer.js missing control {required}")
    for required in [
        "../manifest.detjson",
        "frameMeta.file",
        "loadFrame",
        "step(-1)",
        "step(1)",
        "updateOverlayFromToggles",
        "setupInputCapture",
    ]:
        if required not in viewer_js:
            fail(f"{pack}: viewer.js missing snippet {required}")


def run(args: list[str]) -> None:
    proc = subprocess.run(args, cwd=ROOT, text=True, encoding="utf-8", errors="replace")
    if proc.returncode != 0:
        fail(f"command failed: {' '.join(args)}")


def main() -> None:
    require_file(ROOT / "STD_GRID_GAME_BOGAE_WEB_SHOWCASE_7DAY_V1.md")
    require_file(ROOT / "STD_GRID_GAME_BOGAE_WEB_PLAYBACK_CONTROLS_6DAY_V1.md")
    require_file(ROOT / "tests" / "run_std_grid_game_bogae_web_showcase_check.py")
    require_file(ROOT / "tests" / "run_std_grid_game_bogae_web_output_determinism_check.py")
    require_file(ROOT / "pack" / SHOWCASE_CLOSURE / "contract.detjson")
    require_file(ROOT / "gaji" / "std_grid_game_bogae_web_playback" / "README.md")
    require_file(ROOT / "gaji" / "std_grid_game_bogae_web_playback" / "example.ddn")

    for pack in ALL_PACKS:
        require_pack(pack)
    require_contract()

    run([sys.executable, "tests/run_pack_golden.py", *ALL_PACKS])
    require_generated_assets(ASSETS)
    require_generated_assets(CONTROLS)
    print("[std-grid-game-bogae-web-playback] OK")


if __name__ == "__main__":
    main()
