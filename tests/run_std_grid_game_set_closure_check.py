#!/usr/bin/env python3
"""Validate STD_GRID_GAME_SET_CLOSURE_V1 evidence."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

CLOSURE_PACK = "std_grid_game_set_closure_v1"
REQUIRED_COMPONENT_PACKS = [
    "std_core_grid_unit_closure_v1",
    "std_input_map_closure_v1",
    "std_block_piece_minimum_v1",
    "std_random_bag_minimum_v1",
    "std_grid_game_state_minimum_v1",
]
CLOSURE_STDOUT = ["차림[0, 0]", "P", "O", "진행", "3"]


def fail(message: str) -> None:
    print(f"[std-grid-game-set] FAIL: {message}", file=sys.stderr)
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


def run(args: list[str]) -> None:
    proc = subprocess.run(args, cwd=ROOT, text=True, encoding="utf-8", errors="replace")
    if proc.returncode != 0:
        fail(f"command failed: {' '.join(args)}")


def main() -> None:
    require_file(ROOT / "docs" / "context" / "proposals" / "STANDARD_GRID_GAME_SET_SPEC_V1_20260406.md")

    for pack in REQUIRED_COMPONENT_PACKS + [CLOSURE_PACK]:
        if not (ROOT / "pack" / pack).is_dir():
            fail(f"missing pack: pack/{pack}")

    closure_dir = ROOT / "pack" / CLOSURE_PACK
    require_file(closure_dir / "README.md")
    require_file(closure_dir / "input.ddn")
    require_file(closure_dir / "golden.jsonl")
    require_file(closure_dir / "contract.detjson")

    source = (closure_dir / "input.ddn").read_text(encoding="utf-8")
    for token in ["격자.만들기", "입력사상.방향", "블록조각.고정", "무작위가방.꺼내기", "격자게임상태.재개"]:
        if token not in source:
            fail(f"closure input missing token: {token}")

    rows = read_jsonl(closure_dir / "golden.jsonl")
    if len(rows) != 1:
        fail("closure golden must have exactly one row")
    if rows[0].get("stdout") != CLOSURE_STDOUT:
        fail("closure golden stdout mismatch")

    contract = json.loads((closure_dir / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.std_grid_game_set_closure.pack.contract.v1":
        fail("closure contract schema mismatch")
    if contract.get("bundled_packs") != REQUIRED_COMPONENT_PACKS:
        fail("closure contract bundled_packs mismatch")
    for skeleton in contract.get("gaji_skeletons", []):
        skeleton_dir = ROOT / skeleton
        if not skeleton_dir.is_dir():
            fail(f"missing gaji skeleton: {skeleton}")
        require_file(skeleton_dir / "README.md")

    run([sys.executable, "tests/run_std_random_bag_pack_check.py"])
    run([sys.executable, "tests/run_std_grid_game_state_pack_check.py"])
    run([sys.executable, "tests/run_pack_golden.py", CLOSURE_PACK])
    print("[std-grid-game-set] OK")


if __name__ == "__main__":
    main()

