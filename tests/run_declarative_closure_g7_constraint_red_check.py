#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACKS = [
    "sim_constraint_reaction_v1",
    "sim_mageum_floor_v1",
    "sim_mageum_box_v1",
    "sim_mageum_chain_v1",
    "lang_mageum_diag_v1",
]
VOCAB = ["멈춤", "튕김(e)", "조임", "감쌈"]


def fail(message: str) -> None:
    print(f"declarative_closure_g7_constraint_red_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def run(cmd: list[str], *, timeout: int = 240) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
        check=False,
    )


def read_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8-sig"))
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON {path.relative_to(ROOT)}: {exc}")


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", *PACKS], timeout=300)
    if proc.returncode != 0:
        fail(f"G7 pack golden failed:\n{proc.stdout}")


def require_common_contract(pack: str) -> dict:
    contract = read_json(ROOT / "pack" / pack / "contract.detjson")
    expected_false = [
        "runtime_claim",
        "parser_landed_claim",
        "product_code_change",
        "ssot_edit_claim",
        "new_language_block",
    ]
    for key in expected_false:
        if contract.get(key) is not False:
            fail(f"{pack} {key} must be false, got {contract.get(key)!r}")
    if contract.get("goal") != "G7":
        fail(f"{pack} goal mismatch: {contract.get('goal')!r}")
    if contract.get("canonical_family") != "막음":
        fail(f"{pack} canonical family mismatch")
    if contract.get("canonical_verb") != "막기":
        fail(f"{pack} canonical verb mismatch")
    return contract


def check_umbrella() -> None:
    contract = require_common_contract("sim_constraint_reaction_v1")
    if contract.get("reaction_vocabulary") != VOCAB:
        fail(f"umbrella vocabulary mismatch: {contract.get('reaction_vocabulary')!r}")
    if contract.get("subpacks") != PACKS[1:]:
        fail(f"umbrella subpacks mismatch: {contract.get('subpacks')!r}")
    catalog = read_json(ROOT / "pack" / "sim_constraint_reaction_v1" / "constraint_reaction_catalog.detjson")
    if catalog.get("schema") != "ddn.declarative_closure.constraint_reaction.red.v1":
        fail("catalog schema mismatch")
    if catalog.get("reaction_vocabulary") != VOCAB:
        fail("catalog vocabulary mismatch")
    if catalog.get("surface_policy", {}).get("accepted_product_surface_count") != 0:
        fail("catalog must not claim accepted product surfaces")
    if catalog.get("runtime_claim") is not False or catalog.get("parser_landed_claim") is not False:
        fail("catalog runtime/parser claims must be false")


def check_fixture_pack(pack: str, fixture_name: str, expected_reaction: str | None = None) -> None:
    contract = require_common_contract(pack)
    if expected_reaction is not None and contract.get("reaction") != expected_reaction:
        fail(f"{pack} reaction mismatch: {contract.get('reaction')!r}")
    fixture = read_json(ROOT / "pack" / pack / fixture_name)
    for key in ["accepted_product_surface", "runtime_landed", "parser_landed"]:
        if fixture.get(key) is not False:
            fail(f"{pack}/{fixture_name} {key} must be false")
    if fixture.get("canonical_family") != "막음" or fixture.get("canonical_verb") != "막기":
        fail(f"{pack}/{fixture_name} canonical names mismatch")
    if fixture.get("expected_status") != "red_contract_only":
        fail(f"{pack}/{fixture_name} expected_status mismatch")


def check_chain() -> None:
    contract = require_common_contract("sim_mageum_chain_v1")
    if contract.get("reactions") != ["조임", "감쌈"]:
        fail(f"chain contract reactions mismatch: {contract.get('reactions')!r}")
    fixture = read_json(ROOT / "pack" / "sim_mageum_chain_v1" / "chain_fixture.detjson")
    if fixture.get("reactions") != ["조임", "감쌈"]:
        fail(f"chain fixture reactions mismatch: {fixture.get('reactions')!r}")
    for key in ["accepted_product_surface", "runtime_landed", "parser_landed"]:
        if fixture.get(key) is not False:
            fail(f"chain fixture {key} must be false")


def check_diag() -> None:
    require_common_contract("lang_mageum_diag_v1")
    payload = read_json(ROOT / "pack" / "lang_mageum_diag_v1" / "diagnostics.detjson")
    codes = [row.get("code") for row in payload.get("diagnostics", [])]
    if codes != [
        "E_MAGEUM_RUNTIME_UNLANDED",
        "E_MAGEUM_UNSUPPORTED_REACTION",
        "E_MAGEUM_BLOCK_UNLANDED",
    ]:
        fail(f"diagnostic codes mismatch: {codes!r}")
    for key in ["accepted_product_surface", "runtime_landed", "parser_landed"]:
        if payload.get(key) is not False:
            fail(f"diagnostics {key} must be false")


def check_no_docs_ssot_change() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed:\n{proc.stdout}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_pack_golden()
    check_umbrella()
    check_fixture_pack("sim_mageum_floor_v1", "floor_fixture.detjson", "멈춤")
    check_fixture_pack("sim_mageum_box_v1", "box_fixture.detjson", "튕김(e)")
    check_chain()
    check_diag()
    check_no_docs_ssot_change()
    print("declarative_closure_g7_constraint_red_check: ok")


if __name__ == "__main__":
    main()
