#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "lang_inline_lambda_function_pin_v1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def main() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "golden.jsonl",
        PACK / "cases" / "c01_function_pin_extension.ddn",
        PACK / "cases" / "c01_function_pin_extension.expected.json",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_INLINE_LAMBDA_PACK_MISSING", str(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("evidence_tier") != "runner_fill":
        return fail("E_INLINE_LAMBDA_TIER", str(contract.get("evidence_tier")))
    if contract.get("decision") != "DR-beta B: function pin whole surface; stored/returned lambda excluded":
        return fail("E_INLINE_LAMBDA_DECISION", str(contract.get("decision")))
    rows = [
        json.loads(line)
        for line in (PACK / "golden.jsonl").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    if len(rows) < 3 or any(row.get("cmd", [None])[0] != "run" for row in rows):
        return fail("E_INLINE_LAMBDA_GOLDEN", "runner golden missing")
    print("lang inline lambda function pin pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
