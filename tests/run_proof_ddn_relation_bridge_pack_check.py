#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "proof_ddn_relation_bridge_v1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def main() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_PROOF_DDN_RELATION_MISSING", str(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.proof_ddn_relation_bridge.pack.contract.v1":
        return fail("E_PROOF_DDN_RELATION_SCHEMA", str(contract.get("schema")))
    rows = [
        json.loads(line)
        for line in (PACK / "golden.jsonl").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    if {row.get("id") for row in rows} != {"c01_relation_lowering_bridge"}:
        return fail("E_PROOF_DDN_RELATION_CASES", str([row.get("id") for row in rows]))
    row = rows[0]
    if row.get("cmd") != ["run", "pack/proof_ddn_relation_bridge_v1/input.ddn"]:
        return fail("E_PROOF_DDN_RELATION_CMD", str(row.get("cmd")))
    print("proof ddn relation bridge pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
