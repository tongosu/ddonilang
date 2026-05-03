#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "relation_solve_ddn_bridge_v1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def main() -> int:
    required = [PACK / "README.md", PACK / "contract.detjson", PACK / "input.ddn", PACK / "golden.jsonl"]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_RELATION_BRIDGE_MISSING", str(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.relation_solve_ddn_bridge.pack.contract.v1":
        return fail("E_RELATION_BRIDGE_SCHEMA", str(contract.get("schema")))
    rows = [
        json.loads(line)
        for line in (PACK / "golden.jsonl").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    if len(rows) != 1 or rows[0].get("id") != "c01_bridge_surface":
        return fail("E_RELATION_BRIDGE_GOLDEN", str([row.get("id") for row in rows]))
    if (rows[0].get("cmd") or [None])[0] != "run":
        return fail("E_RELATION_BRIDGE_CMD", str(rows[0].get("cmd")))
    print("relation solve ddn bridge pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

