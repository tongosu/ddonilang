#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "symbolic_relation_canon_v1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def main() -> int:
    required = [PACK / "README.md", PACK / "contract.detjson", PACK / "input.ddn", PACK / "golden.jsonl"]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SYMBOLIC_RELATION_CANON_MISSING", str(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.symbolic_relation_canon.pack.contract.v1":
        return fail("E_SYMBOLIC_RELATION_CANON_SCHEMA", str(contract.get("schema")))
    rows = [
        json.loads(line)
        for line in (PACK / "golden.jsonl").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    if len(rows) != 1 or rows[0].get("id") != "c01_relation_canonicalization":
        return fail("E_SYMBOLIC_RELATION_CANON_GOLDEN", str([row.get("id") for row in rows]))
    print("symbolic relation canon pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
