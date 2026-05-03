#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "formula_relation_solve_quadratic_v1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def main() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input_single_root.ddn",
        PACK / "input_non_unique.ddn",
        PACK / "input_unsupported.ddn",
        PACK / "golden.jsonl",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_RELATION_SOLVE_QUADRATIC_PACK_MISSING", str(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.formula_relation_solve_quadratic.pack.contract.v1":
        return fail("E_RELATION_SOLVE_QUADRATIC_SCHEMA", str(contract.get("schema")))
    rows = [
        json.loads(line)
        for line in (PACK / "golden.jsonl").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    expected = {
        "c01_quadratic_single_root",
        "c02_quadratic_non_unique",
        "c03_quadratic_unsupported",
    }
    if {row.get("id") for row in rows} != expected:
        return fail("E_RELATION_SOLVE_QUADRATIC_CASE_SET", str([row.get("id") for row in rows]))
    print("formula relation solve quadratic pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

