#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BUILD = ROOT / "build" / "declarative_closure_g5_unit"


def fail(message: str) -> None:
    print(f"declarative_closure_g5_unit_check: FAIL: {message}", file=sys.stderr)
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
    return json.loads(path.read_text(encoding="utf-8-sig"))


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "lang_unit_check_v1"], timeout=300)
    if proc.returncode != 0:
        fail(f"lang_unit_check_v1 pack failed:\n{proc.stdout}")

    legacy = run([sys.executable, "tests/run_pack_golden.py", "lang_unit_temp_smoke_v1"], timeout=300)
    if legacy.returncode != 0:
        fail(f"lang_unit_temp_smoke_v1 regression failed:\n{legacy.stdout}")


def check_contract() -> None:
    contract = read_json(ROOT / "pack" / "lang_unit_check_v1" / "contract.detjson")
    if contract.get("product_entrypoint") != "teul-cli run":
        fail("contract must bind to teul-cli run product path")
    if contract.get("state_hash_formula_change") is not False:
        fail("unit check must not claim state_hash formula change")
    if contract.get("solveplan_state_hash_inputs_change") is not False:
        fail("unit check must not change solveplan state_hash_inputs")
    if contract.get("assignment_dimension_check_claim") is not False:
        fail("assignment mismatch must stay outside current claim")
    if contract.get("scalar_unit_mix_policy") != "B_add_sub_compare_assignment_no_implicit_mix":
        fail("scalar-unit mix policy must record approved default B")
    if len(contract.get("accepted_cases", [])) != 5:
        fail("expected five accepted cases")
    rejected = contract.get("rejected_cases", [])
    if len(rejected) != 5:
        fail("expected five rejected cases")
    codes = [case.get("expected_error_code") for case in rejected]
    if codes != [
        "E_UNIT_MISMATCH",
        "E_UNIT_MISMATCH",
        "E_UNIT_MISMATCH",
        "E_UNIT_MISMATCH",
        "E_UNIT_UNKNOWN",
    ]:
        fail(f"unexpected rejected error codes: {codes!r}")


def check_unit_does_not_emit_solveplan_state_hash_inputs() -> None:
    BUILD.mkdir(parents=True, exist_ok=True)
    out_path = BUILD / "unit_literal_solveplan.detjson"
    proc = run(
        [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            "tools/teul-cli/Cargo.toml",
            "--",
            "run",
            "pack/lang_unit_check_v1/c01_meter_literal.ddn",
            "--solveplan-out",
            str(out_path),
        ],
        timeout=300,
    )
    if proc.returncode != 0:
        fail(f"unit solveplan probe failed:\n{proc.stdout}")
    payload = read_json(out_path)
    if payload.get("schema") != "ddn.solveplan.bundle.v1":
        fail(f"unit-only solveplan probe should produce an empty bundle, got {payload.get('schema')!r}")
    if payload.get("plans") != []:
        fail(f"unit-only source must not inject SolvePlan/state_hash_inputs entries: {payload!r}")


def check_no_docs_ssot_change() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed:\n{proc.stdout}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_pack_golden()
    check_contract()
    check_unit_does_not_emit_solveplan_state_hash_inputs()
    check_no_docs_ssot_change()
    print("declarative_closure_g5_unit_check: ok")


if __name__ == "__main__":
    main()
