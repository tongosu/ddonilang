#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "formula_relation_solve_v1"


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
        return fail("E_RELATION_SOLVE_PACK_MISSING", str(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.formula_relation_solve.pack.contract.v1":
        return fail("E_RELATION_SOLVE_SCHEMA", str(contract.get("schema")))
    rows = [
        json.loads(line)
        for line in (PACK / "golden.jsonl").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    expected = {
        "c01_linear_success",
        "c02_no_solution",
        "c03_non_unique",
        "c04_unsupported",
    }
    if {row.get("id") for row in rows} != expected:
        return fail("E_RELATION_SOLVE_CASE_SET", str([row.get("id") for row in rows]))
    if any((row.get("cmd") or [None])[0] != "run" for row in rows):
        return fail("E_RELATION_SOLVE_CMD", "all golden rows must use teul-cli run")
    print("formula relation solve pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

