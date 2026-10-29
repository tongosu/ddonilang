#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKS = ["seum_assertion_block_v1b", "seumssi_relation_bridge_v1"]


def fail(message: str) -> None:
    print(f"[seum-v1b] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        fail(f"failed to read JSON {path}: {exc}")


def read_jsonl(path: Path) -> list[dict]:
    rows = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            rows.append(json.loads(line))
    return rows


def require_file(path: Path) -> None:
    if not path.exists():
        fail(f"missing required file: {path}")


def run(cmd: list[str]) -> None:
    proc = subprocess.run(cmd, cwd=ROOT, text=True)
    if proc.returncode != 0:
        fail(f"command failed: {' '.join(cmd)}")


def main() -> int:
    require_file(ROOT / "SEUM_ASSERTION_BLOCK_V1.md")
    require_file(ROOT / "STD_GRID_GAME_RULES_MINIMUM_V1.md")
    require_file(ROOT / "pack" / "std_grid_game_rules_minimum_v1" / "golden.jsonl")
    require_file(ROOT / "SEUM_ASSERTION_BLOCK_V1B.md")

    for pack in PACKS:
        base = ROOT / "pack" / pack
        for name in ["README.md", "input.ddn", "contract.detjson", "golden.jsonl"]:
            require_file(base / name)
        rows = read_jsonl(base / "golden.jsonl")
        if len(rows) != 1:
            fail(f"{pack} golden.jsonl must have exactly one row")
        if rows[0].get("exit_code") != 0:
            fail(f"{pack} must be a passing compat pack")

    block_contract = read_json(ROOT / "pack" / "seum_assertion_block_v1b" / "contract.detjson")
    if block_contract.get("canonical_surface") != "세움{}":
        fail("seum_assertion_block_v1b must canonicalize to 세움{}")
    if block_contract.get("surface_role") != "AGE4 compat/internal alias":
        fail("seum_assertion_block_v1b must be compat/internal only")
    if block_contract.get("new_user_canonical_surface") is not False:
        fail("세움씨{} must not be claimed as a new user canonical surface")
    if block_contract.get("new_value_type") is not False:
        fail("세움씨{} must not introduce a new value type")
    if block_contract.get("gaji_skeleton") is not False:
        fail("gaji skeleton must remain absent for this internal alias")
    preserved = set(block_contract.get("negative_tests_preserved") or [])
    for item in ["malformed body", "unsupported assertion body", "type mismatch", "proof/runtime failure"]:
        if item not in preserved:
            fail(f"missing preserved negative boundary: {item}")

    relation_contract = read_json(ROOT / "pack" / "seumssi_relation_bridge_v1" / "contract.detjson")
    if relation_contract.get("expected_canonical") != "세움{수식관계: x + y =:= 5}":
        fail("relation bridge canonical mismatch")
    if relation_contract.get("new_proof_type") is not False:
        fail("relation bridge must not introduce a new proof type")
    if relation_contract.get("new_relation_semantics") is not False:
        fail("relation bridge must not introduce new relation semantics")

    block_stdout = read_jsonl(ROOT / "pack" / "seum_assertion_block_v1b" / "golden.jsonl")[0].get("stdout")
    if not block_stdout or not str(block_stdout[0]).startswith("세움{"):
        fail("seum_assertion_block_v1b stdout must display canonical 세움{}")
    if str(block_stdout[0]).startswith("세움씨{"):
        fail("seum_assertion_block_v1b stdout must not display 세움씨{}")
    if block_stdout[-1] != "참":
        fail("seum_assertion_block_v1b 살피기 result must be 참")

    relation_stdout = read_jsonl(ROOT / "pack" / "seumssi_relation_bridge_v1" / "golden.jsonl")[0].get("stdout")
    if relation_stdout != ["세움{수식관계: x + y =:= 5}", "참"]:
        fail("seumssi_relation_bridge_v1 stdout mismatch")

    root_doc = (ROOT / "SEUM_ASSERTION_BLOCK_V1B.md").read_text(encoding="utf-8")
    for needle in ["AGE4 compat/internal alias", "gaji skeleton", "docs/ssot/**"]:
        if needle not in root_doc:
            fail(f"root doc missing marker: {needle}")
    if (ROOT / "gaji" / "seumssi_assertion_block").exists():
        fail("gaji/seumssi_assertion_block must not exist for this internal alias")

    run([sys.executable, "tests/run_pack_golden.py", *PACKS])
    print("[seum-v1b] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
