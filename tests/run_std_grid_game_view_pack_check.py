#!/usr/bin/env python3
"""Validate STD_GRID_GAME_VIEW_SAMPLE_10DAY_V1 evidence."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

COMPONENT_PACKS = [
    "std_grid_game_view_text_v1",
    "std_grid_game_view_overlay_v1",
    "std_grid_game_view_summary_v1",
]
CLOSURE_PACK = "std_grid_game_playable_view_closure_v1"
ALL_PACKS = COMPONENT_PACKS + [CLOSURE_PACK]

EXPECTED_STDOUT = {
    "std_grid_game_view_text_v1": [".OO.", ".OO.", "....", "X..."],
    "std_grid_game_view_overlay_v1": ["16", "빈칸", "낙하", "고정", "O"],
    "std_grid_game_view_summary_v1": [
        "묶음{__종류=std_grid_game_view_summary, 낙하조각위치=차림[1, 0], 레벨=1, 상태=진행, 점수=300, 줄수=2, 틱=1}",
        "std_grid_game_view_summary",
        "1",
        "300",
        "2",
        "1",
        "차림[1, 0]",
    ],
    "std_grid_game_playable_view_closure_v1": [
        "....",
        ".OO.",
        ".OO.",
        "....",
        "....",
        "....",
        ".OO.",
        ".OO.",
        "1",
        "차림[1, 2]",
    ],
}


def fail(message: str) -> None:
    print(f"[std-grid-game-view] FAIL: {message}", file=sys.stderr)
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
    require_file(ROOT / "STD_GRID_GAME_PLAYABLE_KIT_12DAY_V1.md")
    require_file(ROOT / "tests" / "run_std_grid_game_playable_pack_check.py")
    require_file(ROOT / "tests" / "run_std_grid_game_playable_wasm_cli_parity_check.py")

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
    if contract.get("schema") != "ddn.std_grid_game_playable_view_closure.pack.contract.v1":
        fail("closure contract schema mismatch")
    if contract.get("bundled_packs") != COMPONENT_PACKS:
        fail("closure contract bundled_packs mismatch")
    if contract.get("summary_fields") != [
        "__종류",
        "상태",
        "틱",
        "점수",
        "줄수",
        "레벨",
        "낙하조각위치",
    ]:
        fail("summary_fields mismatch")
    projection = contract.get("projection", {})
    if projection.get("sources") != ["빈칸", "고정", "낙하"]:
        fail("projection sources mismatch")
    for skeleton in contract.get("gaji_skeletons", []):
        skeleton_dir = ROOT / skeleton
        if not skeleton_dir.is_dir():
            fail(f"missing gaji skeleton: {skeleton}")
        require_file(skeleton_dir / "README.md")

    run([sys.executable, "tests/run_pack_golden.py", *ALL_PACKS])
    print("[std-grid-game-view] OK")


if __name__ == "__main__":
    main()
