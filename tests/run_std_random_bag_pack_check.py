#!/usr/bin/env python3
"""Validate STD_RANDOM_BAG_MINIMUM_V1 evidence."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

ORDER_PACK = "std_random_bag_order_v1"
REFILL_PREVIEW_PACK = "std_random_bag_refill_preview_v1"
MINIMUM_PACK = "std_random_bag_minimum_v1"
REQUIRED_PACKS = [ORDER_PACK, REFILL_PREVIEW_PACK, MINIMUM_PACK]

ORDER_STDOUT = ["O", "I", "T", "참", "O", "차림[I, T]"]
REFILL_PREVIEW_STDOUT = ["차림[O, I, T, O, T]", "차림[I, O, T]", "차림[]", "O"]
MINIMUM_STDOUT = ["차림[Z, L, S]", "Z", "차림[S, L]"]


def fail(message: str) -> None:
    print(f"[std-random-bag] FAIL: {message}", file=sys.stderr)
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

    order_source = (ROOT / "pack" / ORDER_PACK / "input.ddn").read_text(encoding="utf-8")
    for token in ["무작위가방.만들기", "무작위가방.꺼내기", "무작위가방.비었나", "무작위가방.남은것"]:
        if token not in order_source:
            fail(f"order input missing token: {token}")

    preview_source = (ROOT / "pack" / REFILL_PREVIEW_PACK / "input.ddn").read_text(encoding="utf-8")
    if "무작위가방.미리보기" not in preview_source:
        fail("preview input missing 미리보기")

    require_stdout(ORDER_PACK, ORDER_STDOUT)
    require_stdout(REFILL_PREVIEW_PACK, REFILL_PREVIEW_STDOUT)
    require_stdout(MINIMUM_PACK, MINIMUM_STDOUT)

    contract_path = ROOT / "pack" / MINIMUM_PACK / "contract.detjson"
    require_file(contract_path)
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.std_random_bag_minimum.pack.contract.v1":
        fail("minimum contract schema mismatch")
    if contract.get("bundled_packs") != [ORDER_PACK, REFILL_PREVIEW_PACK]:
        fail("minimum contract bundled_packs mismatch")
    algorithm = str(contract.get("draw_algorithm", ""))
    if "SplitMix64" not in algorithm or "sample % len" not in algorithm:
        fail("minimum contract draw algorithm mismatch")
    representation = contract.get("representation", {})
    if representation.get("__종류") != "std_random_bag":
        fail("minimum contract representation kind mismatch")
    for field in ["시앗", "상태", "원본", "남은것", "뽑은수"]:
        if field not in representation:
            fail(f"minimum contract representation missing field: {field}")

    run([sys.executable, "tests/run_pack_golden.py", *REQUIRED_PACKS])
    print("[std-random-bag] OK")


if __name__ == "__main__":
    main()

