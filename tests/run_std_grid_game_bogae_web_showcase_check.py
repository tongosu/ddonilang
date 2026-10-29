#!/usr/bin/env python3
"""Validate STD_GRID_GAME_BOGAE_WEB_SHOWCASE_7DAY_V1 pack evidence."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]

WEB_VIEWER = "std_grid_game_bogae_web_viewer_v1"
WEB_OUT = "std_grid_game_bogae_web_out_determinism_v1"
CLOSURE = "std_grid_game_bogae_web_showcase_closure_v1"
ALL_PACKS = [WEB_VIEWER, WEB_OUT, CLOSURE]

EXPECTED_STDOUT = {
    WEB_VIEWER: [
        "bogae_hash=blake3:c729f3eef48652d6f9819a48bc275e61b7ec559633c14bcd1b9fbef7043fe980 cmd_count=65 codec=BDL1"
    ],
    WEB_OUT: [
        "bogae_hash=blake3:dc4856b1baf4c5217660788764bfe0c81be70644c3260311f766b55e340387e2 cmd_count=65 codec=BDL1"
    ],
    CLOSURE: ["32", "32", "64", "격자게임셀_0_3", "#보개/2D.Rect", "#ffcc00ff"],
}


def fail(message: str) -> None:
    print(f"[std-grid-game-bogae-web-showcase] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def require_dir(path: Path) -> None:
    if not path.is_dir():
        fail(f"missing dir: {path.relative_to(ROOT)}")


def read_jsonl(path: Path) -> list[dict]:
    rows = []
    for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.strip():
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError as exc:
            fail(f"{path.relative_to(ROOT)}:{lineno}: invalid JSONL: {exc}")
    return rows


def require_stdout(pack: str) -> None:
    rows = read_jsonl(ROOT / "pack" / pack / "golden.jsonl")
    if len(rows) != 1:
        fail(f"pack/{pack}/golden.jsonl must have exactly one row")
    if rows[0].get("stdout") != EXPECTED_STDOUT[pack]:
        fail(f"pack/{pack}/golden stdout mismatch")


def require_pack(pack: str) -> None:
    pack_dir = ROOT / "pack" / pack
    require_dir(pack_dir)
    require_file(pack_dir / "README.md")
    require_file(pack_dir / "input.ddn")
    require_file(pack_dir / "golden.jsonl")
    require_stdout(pack)


def require_contract() -> None:
    contract_path = ROOT / "pack" / CLOSURE / "contract.detjson"
    require_file(contract_path)
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.std_grid_game_bogae_web_showcase_closure.pack.contract.v1":
        fail("closure contract schema mismatch")
    if contract.get("bundled_packs") != [
        WEB_VIEWER,
        WEB_OUT,
        "std_grid_game_bogae_bridge_closure_v1",
    ]:
        fail("closure bundled_packs mismatch")
    drawlist = contract.get("drawlist", {})
    if drawlist.get("trait") != "#보개/2D.Rect":
        fail("closure drawlist trait mismatch")
    if drawlist.get("id_format") != "격자게임셀_{y}_{x}":
        fail("closure drawlist id_format mismatch")
    sample = contract.get("web_sample", {})
    expected_sample = {
        "grid_width": 8,
        "grid_height": 8,
        "cell_px": 4,
        "canvas_w": 32,
        "canvas_h": 32,
        "frame_count": 4,
    }
    if sample != expected_sample:
        fail("closure web_sample mismatch")
    if contract.get("public_surface_delta") != "none":
        fail("closure public_surface_delta mismatch")
    if "gaji/std_grid_game_bogae_web_showcase" not in contract.get("gaji_skeletons", []):
        fail("closure missing gaji skeleton pointer")


def require_determinism_golden() -> None:
    golden_path = ROOT / "pack" / WEB_OUT / "golden" / "webout_001_manifest_hash.test.json"
    require_file(golden_path)
    payload = json.loads(golden_path.read_text(encoding="utf-8"))
    if payload.get("name") != "std_grid_game_bogae_webout_001_manifest_hash":
        fail("web output golden name mismatch")
    observations = payload.get("det_expected", {}).get("expected_observations", [])
    if not any(isinstance(item, str) and item.startswith("manifest_sha256=sha256:") for item in observations):
        fail("web output golden missing manifest sha256 observation")
    frame_observations = [
        item for item in observations if isinstance(item, str) and item.startswith("frame_sha256 ")
    ]
    if len(frame_observations) != 4:
        fail("web output golden must contain 4 frame sha256 observations")


def run(args: list[str]) -> None:
    proc = subprocess.run(args, cwd=ROOT, text=True, encoding="utf-8", errors="replace")
    if proc.returncode != 0:
        fail(f"command failed: {' '.join(args)}")


def main() -> None:
    require_file(ROOT / "STD_GRID_GAME_BOGAE_DRAWLIST_BRIDGE_8DAY_V1.md")
    require_file(ROOT / "STD_GRID_GAME_BOGAE_WEB_SHOWCASE_7DAY_V1.md")
    require_file(ROOT / "tests" / "run_std_grid_game_bogae_bridge_pack_check.py")
    require_file(ROOT / "tests" / "run_std_grid_game_bogae_bridge_wasm_cli_parity_check.py")
    require_file(ROOT / "tests" / "run_std_grid_game_bogae_web_output_determinism_check.py")
    require_file(ROOT / "pack" / "std_grid_game_bogae_bridge_closure_v1" / "contract.detjson")
    require_file(ROOT / "pack" / "bogae_web_viewer_v1" / "golden.jsonl")
    require_file(
        ROOT
        / "pack"
        / "bogae_web_out_determinism"
        / "golden"
        / "webout_001_manifest_hash.test.json"
    )
    require_file(ROOT / "tests" / "run_bogae_alias_viewer_family_selftest.py")
    require_file(ROOT / "gaji" / "std_grid_game_bogae_web_showcase" / "README.md")
    require_file(ROOT / "gaji" / "std_grid_game_bogae_web_showcase" / "example.ddn")

    for pack in ALL_PACKS:
        require_pack(pack)
    require_contract()
    require_determinism_golden()

    run([sys.executable, "tests/run_pack_golden.py", *ALL_PACKS])
    run([sys.executable, "tests/run_std_grid_game_bogae_web_output_determinism_check.py"])
    print("[std-grid-game-bogae-web-showcase] OK")


if __name__ == "__main__":
    main()
