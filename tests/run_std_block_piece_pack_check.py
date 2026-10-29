#!/usr/bin/env python3
"""Validate STD_BLOCK_PIECE_MINIMUM_V1 evidence."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

GEOMETRY_PACK = "std_block_piece_geometry_v1"
GRID_BRIDGE_PACK = "std_block_piece_grid_bridge_v1"
MINIMUM_PACK = "std_block_piece_minimum_v1"
REQUIRED_PACKS = [GEOMETRY_PACK, GRID_BRIDGE_PACK, MINIMUM_PACK]

GEOMETRY_STDOUT = [
    "차림[차림[0, 0], 차림[1, 0], 차림[0, 1]]",
    "차림[차림[2, 1], 차림[3, 1], 차림[2, 2]]",
    "차림[차림[-1, 0], 차림[0, 0], 차림[0, 1]]",
    "차림[차림[0, -1], 차림[0, 0], 차림[1, 0]]",
    "차림[차림[0, -1], 차림[-1, 0], 차림[0, 0]]",
]
GRID_BRIDGE_STDOUT = ["거짓", "참", "참", "X", "#"]
MINIMUM_STDOUT = ["차림[차림[1, 1], 차림[2, 1], 차림[1, 2]]", "P"]


def fail(message: str) -> None:
    print(f"[std-block-piece] FAIL: {message}", file=sys.stderr)
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


def require_stdout(pack: str, expected: list[str]) -> None:
    rows = read_jsonl(ROOT / "pack" / pack / "golden.jsonl")
    if len(rows) != 1:
        fail(f"pack/{pack}/golden.jsonl must have exactly one row")
    if rows[0].get("stdout") != expected:
        fail(f"pack/{pack}/golden stdout mismatch")


def run(args: list[str]) -> None:
    proc = subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        fail(f"command failed: {' '.join(args)}")


def main() -> None:
    require_file(ROOT / "docs" / "context" / "proposals" / "STANDARD_GRID_GAME_SET_SPEC_V1_20260406.md")

    for pack in REQUIRED_PACKS:
        pack_dir = ROOT / "pack" / pack
        if not pack_dir.is_dir():
            fail(f"missing pack: pack/{pack}")
        require_file(pack_dir / "README.md")
        require_file(pack_dir / "input.ddn")
        require_file(pack_dir / "golden.jsonl")

    geometry_source = (ROOT / "pack" / GEOMETRY_PACK / "input.ddn").read_text(encoding="utf-8")
    for token in ["블록조각.만들기", "블록조각.칸목록", "블록조각.이동", "블록조각.회전"]:
        if token not in geometry_source:
            fail(f"geometry input missing token: {token}")

    bridge_source = (ROOT / "pack" / GRID_BRIDGE_PACK / "input.ddn").read_text(encoding="utf-8")
    for token in ["격자.만들기", "격자.바꾼값", "블록조각.충돌?", "블록조각.고정"]:
        if token not in bridge_source:
            fail(f"grid bridge input missing token: {token}")

    require_stdout(GEOMETRY_PACK, GEOMETRY_STDOUT)
    require_stdout(GRID_BRIDGE_PACK, GRID_BRIDGE_STDOUT)
    require_stdout(MINIMUM_PACK, MINIMUM_STDOUT)

    contract_path = ROOT / "pack" / MINIMUM_PACK / "contract.detjson"
    require_file(contract_path)
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.std_block_piece_minimum.pack.contract.v1":
        fail("minimum contract schema mismatch")
    if contract.get("bundled_packs") != [GEOMETRY_PACK, GRID_BRIDGE_PACK]:
        fail("minimum contract bundled_packs mismatch")
    representation = contract.get("representation", {})
    if representation.get("__종류") != "std_block_piece":
        fail("minimum contract representation kind mismatch")
    if "칸들" not in representation:
        fail("minimum contract representation must record cells field")

    run([sys.executable, "tests/run_pack_golden.py", *REQUIRED_PACKS])
    print("[std-block-piece] OK")


if __name__ == "__main__":
    main()
