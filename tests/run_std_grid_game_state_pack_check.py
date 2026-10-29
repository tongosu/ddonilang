#!/usr/bin/env python3
"""Validate STD_GRID_GAME_STATE_MINIMUM_V1 evidence."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

LIFECYCLE_PACK = "std_grid_game_state_lifecycle_v1"
PAUSE_RESUME_PACK = "std_grid_game_state_pause_resume_v1"
MINIMUM_PACK = "std_grid_game_state_minimum_v1"
REQUIRED_PACKS = [LIFECYCLE_PACK, PAUSE_RESUME_PACK, MINIMUM_PACK]

LIFECYCLE_STDOUT = ["준비", "0", "진행", "1", "참", "끝", "2"]
PAUSE_RESUME_STDOUT = ["멈춤", "2", "진행", "3"]
MINIMUM_STDOUT = ["진행", "3"]


def fail(message: str) -> None:
    print(f"[std-grid-game-state] FAIL: {message}", file=sys.stderr)
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
    proc = subprocess.run(args, cwd=ROOT, text=True, encoding="utf-8", errors="replace")
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

    lifecycle_source = (ROOT / "pack" / LIFECYCLE_PACK / "input.ddn").read_text(encoding="utf-8")
    for token in ["격자게임상태.초기화", "격자게임상태.바꾸기", "격자게임상태.상태인가"]:
        if token not in lifecycle_source:
            fail(f"lifecycle input missing token: {token}")

    pause_source = (ROOT / "pack" / PAUSE_RESUME_PACK / "input.ddn").read_text(encoding="utf-8")
    for token in ["격자게임상태.멈춤", "격자게임상태.재개"]:
        if token not in pause_source:
            fail(f"pause/resume input missing token: {token}")

    require_stdout(LIFECYCLE_PACK, LIFECYCLE_STDOUT)
    require_stdout(PAUSE_RESUME_PACK, PAUSE_RESUME_STDOUT)
    require_stdout(MINIMUM_PACK, MINIMUM_STDOUT)

    contract_path = ROOT / "pack" / MINIMUM_PACK / "contract.detjson"
    require_file(contract_path)
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.std_grid_game_state_minimum.pack.contract.v1":
        fail("minimum contract schema mismatch")
    if contract.get("bundled_packs") != [LIFECYCLE_PACK, PAUSE_RESUME_PACK]:
        fail("minimum contract bundled_packs mismatch")
    allowed_states = contract.get("allowed_states")
    if allowed_states != ["준비", "진행", "잠금지연", "정리", "끝", "멈춤"]:
        fail("minimum contract allowed_states mismatch")
    representation = contract.get("representation", {})
    if representation.get("__종류") != "std_grid_game_state":
        fail("minimum contract representation kind mismatch")
    for field in ["상태", "이전상태", "틱"]:
        if field not in representation:
            fail(f"minimum contract representation missing field: {field}")

    run([sys.executable, "tests/run_pack_golden.py", *REQUIRED_PACKS])
    print("[std-grid-game-state] OK")


if __name__ == "__main__":
    main()

