#!/usr/bin/env python3
"""Validate STD_GRID_GAME_PLAYABLE_KIT_12DAY_V1 evidence."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

COMPONENT_PACKS = [
    "std_tetromino_catalog_v1",
    "std_grid_line_clear_v1",
    "std_falling_piece_state_v1",
    "std_grid_game_step_v1",
    "std_grid_game_lock_spawn_v1",
    "std_grid_game_score_v1",
    "std_grid_game_session_v1",
    "std_grid_game_one_tick_v1",
]
CLOSURE_PACK = "std_grid_game_playable_closure_v1"
ALL_PACKS = COMPONENT_PACKS + [CLOSURE_PACK]

EXPECTED_STDOUT = {
    "std_tetromino_catalog_v1": [
        "차림[I, O, T, S, Z, J, L]",
        "차림[차림[-1, 0], 차림[0, 0], 차림[1, 0], 차림[2, 0]]",
        "차림[차림[0, 0], 차림[1, 0], 차림[0, 1], 차림[1, 1]]",
        "7",
    ],
    "std_grid_line_clear_v1": ["차림[2]", "1", "P", "."],
    "std_falling_piece_state_v1": [
        "차림[2, 3]",
        "차림[차림[2, 3], 차림[3, 3], 차림[2, 4], 차림[3, 4]]",
        "차림[1, 5]",
    ],
    "std_grid_game_step_v1": ["참", "차림[1, 1]"],
    "std_grid_game_lock_spawn_v1": ["참", "0", "X", "차림[1, 0]"],
    "std_grid_game_score_v1": ["0", "1", "2100", "11", "2"],
    "std_grid_game_session_v1": ["진행", "차림[1, 0]", "4", "0"],
    "std_grid_game_one_tick_v1": ["거짓", "0", "차림[1, 1]"],
    "std_grid_game_playable_closure_v1": [
        "차림[1, 1]",
        "차림[1, 2]",
        "참",
        "X",
        "0",
        "진행",
    ],
}


def fail(message: str) -> None:
    print(f"[std-grid-game-playable] FAIL: {message}", file=sys.stderr)
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
    require_file(ROOT / "docs" / "context" / "proposals" / "STANDARD_GRID_GAME_SET_SPEC_V1_20260406.md")

    for pack in ALL_PACKS:
        pack_dir = ROOT / "pack" / pack
        if not pack_dir.is_dir():
            fail(f"missing pack: pack/{pack}")
        require_file(pack_dir / "README.md")
        require_file(pack_dir / "input.ddn")
        require_file(pack_dir / "golden.jsonl")
        require_stdout(pack)

    closure_dir = ROOT / "pack" / CLOSURE_PACK
    require_file(closure_dir / "contract.detjson")
    contract = json.loads((closure_dir / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.std_grid_game_playable_closure.pack.contract.v1":
        fail("closure contract schema mismatch")
    if contract.get("bundled_packs") != COMPONENT_PACKS:
        fail("closure contract bundled_packs mismatch")
    if contract.get("score_table", {}).get("level_formula") != "1 + floor(total_lines / 10)":
        fail("score level formula missing")
    coords = contract.get("tetromino_coordinates", {})
    if sorted(coords.keys()) != ["I", "J", "L", "O", "S", "T", "Z"]:
        fail("tetromino coordinate set mismatch")
    source = (closure_dir / "input.ddn").read_text(encoding="utf-8")
    for token in [
        "테트로미노.만들기",
        "입력사상.만들기",
        "격자게임세션.만들기",
        "격자게임.한틱",
        "격자게임점수.점수",
    ]:
        if token not in source:
            fail(f"closure input missing token: {token}")
    for skeleton in contract.get("gaji_skeletons", []):
        skeleton_dir = ROOT / skeleton
        if not skeleton_dir.is_dir():
            fail(f"missing gaji skeleton: {skeleton}")
        require_file(skeleton_dir / "README.md")

    run([sys.executable, "tests/run_pack_golden.py", *ALL_PACKS])
    print("[std-grid-game-playable] OK")


if __name__ == "__main__":
    main()

