#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PACKS = {
    "symbolic_mathir_canon_v1": "ddn.symbolic_mathir_canon.pack.contract.v1",
    "symbolic_polynomial_simplify_v1": "ddn.symbolic_polynomial_simplify.pack.contract.v1",
    "symbolic_expand_factor_v1": "ddn.symbolic_expand_factor.pack.contract.v1",
    "symbolic_diff_integral_v1": "ddn.symbolic_diff_integral.pack.contract.v1",
    "symbolic_equivalence_v1": "ddn.symbolic_equivalence.pack.contract.v1",
    "symbolic_relation_canon_v1": "ddn.symbolic_relation_canon.pack.contract.v1",
}


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def load_jsonl(path: Path) -> list[dict]:
    return [
        json.loads(line)
        for line in path.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]


def main() -> int:
    for pack_name, schema in PACKS.items():
        pack = ROOT / "pack" / pack_name
        required = [pack / "README.md", pack / "contract.detjson", pack / "golden.jsonl"]
        missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
        if missing:
            return fail("E_SYMBOLIC_PACK_MISSING", f"{pack_name}: {missing}")
        contract = load_json(pack / "contract.detjson")
        if contract.get("schema") != schema:
            return fail("E_SYMBOLIC_PACK_SCHEMA", f"{pack_name}: {contract.get('schema')}")
        rows = load_jsonl(pack / "golden.jsonl")
        if not rows:
            return fail("E_SYMBOLIC_GOLDEN_EMPTY", pack_name)
        for row in rows:
            cmd = row.get("cmd") or []
            if len(cmd) < 2 or cmd[0] != "symbolic":
                return fail("E_SYMBOLIC_CMD", f"{pack_name}:{row.get('id')} cmd={cmd}")
            if row.get("exit_code") != 0:
                return fail("E_SYMBOLIC_EXIT_CODE", f"{pack_name}:{row.get('id')}")
    print("symbolic kernel pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
