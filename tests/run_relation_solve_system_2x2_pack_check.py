#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "relation_solve_system_2x2_v1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def main() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input_success.ddn",
        PACK / "input_no_solution.ddn",
        PACK / "input_non_unique.ddn",
        PACK / "input_unsupported.ddn",
        PACK / "golden.jsonl",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_RELATION_SOLVE_SYSTEM_PACK_MISSING", str(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.relation_solve_system_2x2.pack.contract.v1":
        return fail("E_RELATION_SOLVE_SYSTEM_SCHEMA", str(contract.get("schema")))
    rows = [
        json.loads(line)
        for line in (PACK / "golden.jsonl").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    expected = {
        "c01_system_success",
        "c02_system_no_solution",
        "c03_system_non_unique",
        "c04_system_unsupported",
    }
    if {row.get("id") for row in rows} != expected:
        return fail("E_RELATION_SOLVE_SYSTEM_CASE_SET", str([row.get("id") for row in rows]))
    print("relation solve system 2x2 pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

