#!/usr/bin/env python3
"""Validate STD_GRID_GAME_BOGAE_DRAWLIST_BRIDGE_8DAY_V1 evidence."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

COMPONENT_PACKS = [
    "std_grid_game_bogae_drawlist_v1",
    "std_grid_game_bogae_size_v1",
    "std_grid_game_bogae_backend_parity_v1",
]
CLOSURE_PACK = "std_grid_game_bogae_bridge_closure_v1"
ALL_PACKS = COMPONENT_PACKS + [CLOSURE_PACK]

EXPECTED_STDOUT = {
    "std_grid_game_bogae_drawlist_v1": [
        "16",
        "격자게임셀_0_0",
        "#보개/2D.Rect",
        "0",
        "0",
        "8",
        "8",
        "#111111ff",
        "격자게임셀_0_1",
        "8",
        "#ffcc00ff",
        "격자게임셀_3_0",
        "24",
        "#4a90e2ff",
    ],
    "std_grid_game_bogae_size_v1": ["std_grid_game_bogae_size", "32", "32"],
    "std_grid_game_bogae_backend_parity_v1": [
        "bogae_hash=blake3:3c9287fb6d48faf3f2e7d40f431f3777ba25c5f03596dc4b8a70cdae285faeaf"
    ],
    "std_grid_game_bogae_bridge_closure_v1": [
        "16",
        "격자게임셀_1_1",
        "8",
        "#ffcc00ff",
        "32",
        "32",
        "1",
    ],
}


def fail(message: str) -> None:
    print(f"[std-grid-game-bogae] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


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


def run(args: list[str]) -> None:
    proc = subprocess.run(args, cwd=ROOT, text=True, encoding="utf-8", errors="replace")
    if proc.returncode != 0:
        fail(f"command failed: {' '.join(args)}")


def main() -> None:
    require_file(ROOT / "STD_GRID_GAME_VIEW_SAMPLE_10DAY_V1.md")
    require_file(ROOT / "tests" / "run_std_grid_game_view_pack_check.py")
    require_file(ROOT / "tests" / "run_std_grid_game_view_wasm_cli_parity_check.py")
    require_file(ROOT / "pack" / "bogae_drawlist_listkey_v1" / "golden.jsonl")
    require_file(ROOT / "pack" / "bogae_backend_parity_console_web_v1" / "expected" / "smoke.detjson")

    for pack in ALL_PACKS:
        pack_dir = ROOT / "pack" / pack
        if not pack_dir.is_dir():
            fail(f"missing pack: pack/{pack}")
        require_file(pack_dir / "README.md")
        require_file(pack_dir / "input.ddn")
        require_file(pack_dir / "golden.jsonl")
        require_stdout(pack)

    contract_path = ROOT / "pack" / CLOSURE_PACK / "contract.detjson"
    require_file(contract_path)
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.std_grid_game_bogae_bridge_closure.pack.contract.v1":
        fail("closure contract schema mismatch")
    if contract.get("bundled_packs") != COMPONENT_PACKS:
        fail("closure bundled_packs mismatch")
    drawlist = contract.get("drawlist", {})
    if drawlist.get("trait") != "#보개/2D.Rect":
        fail("drawlist trait mismatch")
    if drawlist.get("id_format") != "격자게임셀_{y}_{x}":
        fail("drawlist id_format mismatch")
    if drawlist.get("fields") != ["id", "결", "x", "y", "w", "h", "채움색"]:
        fail("drawlist fields mismatch")
    if drawlist.get("colors") != {
        "빈칸": "#111111ff",
        "고정": "#4a90e2ff",
        "낙하": "#ffcc00ff",
    }:
        fail("drawlist colors mismatch")
    if contract.get("size_kind") != "std_grid_game_bogae_size":
        fail("size kind mismatch")
    for skeleton in contract.get("gaji_skeletons", []):
        skeleton_dir = ROOT / skeleton
        if not skeleton_dir.is_dir():
            fail(f"missing gaji skeleton: {skeleton}")
        require_file(skeleton_dir / "README.md")

    run([sys.executable, "tests/run_pack_golden.py", *ALL_PACKS])
    print("[std-grid-game-bogae] OK")


if __name__ == "__main__":
    main()

