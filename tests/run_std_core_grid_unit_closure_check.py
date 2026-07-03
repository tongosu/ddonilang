#!/usr/bin/env python3
"""Validate STD_CORE_GRID_UNIT_CLOSURE_V1 evidence."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

REQUIRED_PACKS = [
    "stdlib_1_v1",
    "std_grid_pathfind_reachable_v1",
    "std_grid_pathfind_blocked_v1",
    "std_grid_pathfind_bounds_diag_v1",
    "std_physics_1d_basics_v1",
    "lang_unit_temp_smoke_v1",
]

CLOSURE_PACK = "std_core_grid_unit_closure_v1"


def fail(message: str) -> None:
    print(f"[std-core-grid-unit-closure] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read_jsonl(path: Path) -> list[dict]:
    rows = []
    for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.strip():
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError as exc:
            fail(f"{path}:{lineno}: invalid JSONL: {exc}")
    return rows


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


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
    require_file(ROOT / "tests" / "run_stdlib_1_check.py")
    require_file(ROOT / "tests" / "run_seamgrim_stdlib_1_wasm_check.py")

    for pack in [*REQUIRED_PACKS, CLOSURE_PACK]:
        pack_dir = ROOT / "pack" / pack
        if not pack_dir.is_dir():
            fail(f"missing pack: pack/{pack}")
        require_file(pack_dir / "golden.jsonl")

    stdlib_readme = (ROOT / "pack" / "stdlib_1_v1" / "README.md").read_text(encoding="utf-8")
    for token in ["std_core", "std_grid", "std_input_map"]:
        if token not in stdlib_readme:
            fail(f"stdlib_1_v1 README no longer records {token}")

    contract_path = ROOT / "pack" / CLOSURE_PACK / "contract.detjson"
    require_file(contract_path)
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.std_core_grid_unit_closure.pack.contract.v1":
        fail("closure contract schema mismatch")
    if contract.get("stdlib_first_run_umbrella") != "stdlib_1_v1":
        fail("closure contract must keep stdlib_1_v1 as first-run umbrella")
    if contract.get("bundled_packs") != REQUIRED_PACKS:
        fail("closure contract bundled_packs mismatch")

    source = (ROOT / "pack" / CLOSURE_PACK / "input.ddn").read_text(encoding="utf-8")
    for token in ["범위", "격자.길찾기", "물리1d.위치갱신", "@C", "@F"]:
        if token not in source:
            fail(f"closure input missing token: {token}")

    rows = read_jsonl(ROOT / "pack" / CLOSURE_PACK / "golden.jsonl")
    if len(rows) != 1:
        fail("closure pack must have exactly one representative golden row")
    expected_stdout = [
        "차림[0, 1, 2]",
        "차림[차림[0, 0], 차림[0, 1], 차림[0, 2], 차림[1, 2], 차림[2, 2], 차림[3, 2]]",
        "6",
        "16",
        "참",
    ]
    if rows[0].get("stdout") != expected_stdout:
        fail("closure golden stdout mismatch")

    run([sys.executable, "tests/run_pack_golden.py", *REQUIRED_PACKS, CLOSURE_PACK])
    run([sys.executable, "tests/run_seamgrim_stdlib_1_wasm_check.py"])
    print("[std-core-grid-unit-closure] OK")


if __name__ == "__main__":
    main()
