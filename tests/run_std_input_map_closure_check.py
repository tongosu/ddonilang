#!/usr/bin/env python3
"""Validate STD_INPUT_MAP_CLOSURE_V1 evidence."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

REQUIRED_PACKS = [
    "std_input_map_keyboard_v1",
    "std_input_map_web_snapshot_v1",
]

CLOSURE_PACK = "std_input_map_closure_v1"
SAM_FIXTURE = ROOT / "pack" / "input_key_alias_ko_v1" / "sam" / "key_alias.input.bin"
EXPECTED_STDOUT = ["차림[1, 0]", "참", "거짓"]


def fail(message: str) -> None:
    print(f"[std-input-map-closure] FAIL: {message}", file=sys.stderr)
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
    require_file(SAM_FIXTURE)

    for pack in [*REQUIRED_PACKS, CLOSURE_PACK]:
        pack_dir = ROOT / "pack" / pack
        if not pack_dir.is_dir():
            fail(f"missing pack: pack/{pack}")
        require_file(pack_dir / "golden.jsonl")

    stdlib_readme = (ROOT / "pack" / "stdlib_1_v1" / "README.md").read_text(encoding="utf-8")
    if "std_input_map" not in stdlib_readme:
        fail("stdlib_1_v1 README no longer records std_input_map")

    for pack in REQUIRED_PACKS:
        rows = read_jsonl(ROOT / "pack" / pack / "golden.jsonl")
        if not rows:
            fail(f"pack/{pack}/golden.jsonl has no cases")
        for row in rows:
            cli = row.get("cli", [])
            if "pack/input_key_alias_ko_v1/sam/key_alias.input.bin" not in cli:
                fail(f"pack/{pack} must use the shared SAM input fixture")
            if "stdout" not in row:
                fail(f"pack/{pack} must be runner-backed with stdout expectations")

    contract_path = ROOT / "pack" / CLOSURE_PACK / "contract.detjson"
    require_file(contract_path)
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.std_input_map_closure.pack.contract.v1":
        fail("closure contract schema mismatch")
    if contract.get("bundled_packs") != REQUIRED_PACKS:
        fail("closure contract bundled_packs mismatch")
    if contract.get("sam_fixture") != "pack/input_key_alias_ko_v1/sam/key_alias.input.bin":
        fail("closure contract sam_fixture mismatch")
    if contract.get("stdlib_first_run_umbrella") != "stdlib_1_v1":
        fail("closure contract must keep stdlib_1_v1 as first-run umbrella")

    source = (ROOT / "pack" / CLOSURE_PACK / "input.ddn").read_text(encoding="utf-8")
    for token in ["입력사상.만들기", "입력사상.방향", "입력사상.동작", "ArrowRight", "Space"]:
        if token not in source:
            fail(f"closure input missing token: {token}")

    rows = read_jsonl(ROOT / "pack" / CLOSURE_PACK / "golden.jsonl")
    if len(rows) != 1:
        fail("closure pack must have exactly one representative golden row")
    if rows[0].get("stdout") != EXPECTED_STDOUT:
        fail("closure golden stdout mismatch")
    if rows[0].get("cli") != [
        "--sam",
        "pack/input_key_alias_ko_v1/sam/key_alias.input.bin",
        "--madi",
        "1",
    ]:
        fail("closure golden cli mismatch")

    run([sys.executable, "tests/run_pack_golden.py", *REQUIRED_PACKS, CLOSURE_PACK])
    print("[std-input-map-closure] OK")


if __name__ == "__main__":
    main()
