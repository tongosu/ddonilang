#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKS = {
    "symbolic_rational_expr_v1": "ddn.symbolic_rational_expr.pack.contract.v1",
    "symbolic_multivar_polynomial_v1": "ddn.symbolic_multivar_polynomial.pack.contract.v1",
    "symbolic_ddn_cli_parity_v1": "ddn.symbolic_ddn_cli_parity.pack.contract.v1",
}


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def main() -> int:
    for name, schema in PACKS.items():
        pack = ROOT / "pack" / name
        required = [pack / "README.md", pack / "contract.detjson", pack / "golden.jsonl"]
        missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
        if missing:
            return fail("E_SYMBOLIC_V2_MISSING", f"{name}: {missing}")
        contract = json.loads((pack / "contract.detjson").read_text(encoding="utf-8"))
        if contract.get("schema") != schema:
            return fail("E_SYMBOLIC_V2_SCHEMA", name)
        rows = [json.loads(line) for line in (pack / "golden.jsonl").read_text(encoding="utf-8").splitlines() if line.strip()]
        if not rows or not any((row.get("cmd") or [None])[0] == "symbolic" for row in rows):
            return fail("E_SYMBOLIC_V2_CLI_CASE", name)
        if not any((row.get("cmd") or [None])[0] == "run" for row in rows):
            return fail("E_SYMBOLIC_V2_DDN_CASE", name)
    print("symbolic v2 pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

