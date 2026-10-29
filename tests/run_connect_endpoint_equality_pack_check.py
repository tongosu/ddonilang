#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "connect_endpoint_equality_v1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def main() -> int:
    required = [PACK / "README.md", PACK / "contract.detjson", PACK / "input.ddn", PACK / "golden.jsonl"]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_CONNECT_ENDPOINT_EQUALITY_MISSING", str(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.connect_endpoint_equality.pack.contract.v1":
        return fail("E_CONNECT_ENDPOINT_EQUALITY_SCHEMA", str(contract.get("schema")))
    if contract.get("closure_scope") != "endpoint_equality_only":
        return fail("E_CONNECT_ENDPOINT_EQUALITY_SCOPE", str(contract.get("closure_scope")))
    rows = [
        json.loads(line)
        for line in (PACK / "golden.jsonl").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    if len(rows) != 1 or rows[0].get("id") != "c01_connect_endpoint_equality":
        return fail("E_CONNECT_ENDPOINT_EQUALITY_GOLDEN", str([row.get("id") for row in rows]))
    stdout = rows[0].get("stdout", [])
    if stdout != ["endpoint_equality"]:
        return fail("E_CONNECT_ENDPOINT_EQUALITY_STDOUT", str(stdout))
    source = (PACK / "input.ddn").read_text(encoding="utf-8")
    required_source_terms = ["전지.양극", "전구.왼핀", "전압은 같게", "잇기"]
    missing_terms = [term for term in required_source_terms if term not in source]
    if missing_terms:
        return fail("E_CONNECT_ENDPOINT_EQUALITY_SOURCE", str(missing_terms))
    print("connect endpoint equality pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
