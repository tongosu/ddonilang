#!/usr/bin/env python3
"""Validate STD_GRID_GAME_RULES_MINIMUM_V1 evidence."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

HOLD = "std_grid_game_hold_queue_v1"
GHOST = "std_grid_game_ghost_piece_v1"
WALL_KICK = "std_grid_game_simple_wall_kick_v1"
MINIMUM = "std_grid_game_rules_minimum_v1"
COMPONENT_PACKS = [HOLD, GHOST, WALL_KICK]
ALL_PACKS = COMPONENT_PACKS + [MINIMUM]

EXPECTED_STDOUT = {
    HOLD: [
        "없음",
        "거짓",
        "차림[차림[-1, 0], 차림[0, 0], 차림[1, 0], 차림[0, 1]]",
        "참",
        "차림[2, 0]",
        "거짓",
    ],
    GHOST: ["차림[1, 4]", "#ffcc00ff", "#88ffffff"],
    WALL_KICK: ["참", "차림[1, 0]", "차림[1, 1]"],
    MINIMUM: [
        "std_grid_game_rules_minimum_v1",
        HOLD,
        GHOST,
        WALL_KICK,
        "hold_external_v1",
        "deterministic_fallback_offsets_only",
    ],
}


def fail(message: str) -> None:
    print(f"[std-grid-game-rules] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def read_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")


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
    require_file(ROOT / "STD_GRID_GAME_BOGAE_FINITE_LIVE_LOOP_V1.md")
    require_file(ROOT / "tests" / "run_std_grid_game_bogae_finite_live_loop_check.py")
    require_file(ROOT / "pack" / "std_grid_game_bogae_finite_live_loop_closure_v1" / "contract.detjson")

    for pack in ALL_PACKS:
        pack_dir = ROOT / "pack" / pack
        if not pack_dir.is_dir():
            fail(f"missing pack: pack/{pack}")
        for filename in ["README.md", "input.ddn", "golden.jsonl", "contract.detjson"]:
            require_file(pack_dir / filename)
        require_stdout(pack)

    hold_contract = read_json(ROOT / "pack" / HOLD / "contract.detjson")
    if hold_contract.get("schema") != "ddn.std_grid_game_hold_queue.pack.contract.v1":
        fail("hold contract schema mismatch")
    if hold_contract.get("representation", {}).get("__종류") != "std_grid_game_hold_queue":
        fail("hold representation kind mismatch")
    if hold_contract.get("session_integration") is not False:
        fail("hold must remain external to session in V1")

    ghost_contract = read_json(ROOT / "pack" / GHOST / "contract.detjson")
    if ghost_contract.get("schema") != "ddn.std_grid_game_ghost_piece.pack.contract.v1":
        fail("ghost contract schema mismatch")
    colors = ghost_contract.get("bogae_rect", {}).get("colors", {})
    if colors.get("유령") != "#88ffffff" or colors.get("낙하") != "#ffcc00ff":
        fail("ghost/falling colors mismatch")
    if ghost_contract.get("bogae_rect", {}).get("결") != "#보개/2D.Rect":
        fail("ghost drawlist trait mismatch")
    if ghost_contract.get("bogae_rect", {}).get("id") != "격자게임셀_{y}_{x}":
        fail("ghost drawlist id format mismatch")

    wall_contract = read_json(ROOT / "pack" / WALL_KICK / "contract.detjson")
    if wall_contract.get("schema") != "ddn.std_grid_game_simple_wall_kick.pack.contract.v1":
        fail("wall-kick contract schema mismatch")
    if wall_contract.get("fallback_offsets") != [[0, 0], [-1, 0], [1, 0], [-2, 0], [2, 0], [0, -1]]:
        fail("wall-kick fallback offsets mismatch")
    if wall_contract.get("srs_full_table") is not False:
        fail("wall-kick must not claim full SRS")

    minimum_contract = read_json(ROOT / "pack" / MINIMUM / "contract.detjson")
    if minimum_contract.get("schema") != "ddn.std_grid_game_rules_minimum.pack.contract.v1":
        fail("minimum contract schema mismatch")
    if minimum_contract.get("bundled_packs") != COMPONENT_PACKS:
        fail("minimum bundled_packs mismatch")
    if "격자게임.한틱" not in minimum_contract.get("unchanged", []):
        fail("minimum contract must record one-tick unchanged")
    if "gaji/std_grid_game_rules_minimum" not in minimum_contract.get("gaji_skeletons", []):
        fail("minimum contract missing gaji skeleton")

    token_checks = {
        HOLD: ["격자게임홀드.초기화", "격자게임홀드.교체", "격자게임홀드.초기화턴"],
        GHOST: ["격자게임보기.유령조각", "격자게임보기.유령보개목록"],
        WALL_KICK: ["격자게임.회전시도"],
        MINIMUM: ["hold_external_v1", "deterministic_fallback_offsets_only"],
    }
    for pack, tokens in token_checks.items():
        source = (ROOT / "pack" / pack / "input.ddn").read_text(encoding="utf-8")
        for token in tokens:
            if token not in source:
                fail(f"pack/{pack}/input.ddn missing token: {token}")

    require_file(ROOT / "STD_GRID_GAME_RULES_MINIMUM_V1.md")
    require_file(ROOT / "gaji" / "std_grid_game_rules_minimum" / "README.md")
    require_file(ROOT / "gaji" / "std_grid_game_rules_minimum" / "example.ddn")

    run([sys.executable, "tests/run_pack_golden.py", *ALL_PACKS])
    print("[std-grid-game-rules] OK")


if __name__ == "__main__":
    main()

