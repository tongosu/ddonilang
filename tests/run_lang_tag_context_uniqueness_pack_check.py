#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "lang_tag_context_uniqueness_v1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def main() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "golden.jsonl",
        PACK / "cases" / "c01_single_context_tag_ok.ddn",
        PACK / "cases" / "c02_same_context_duplicate_reject.ddn",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_TAG_CONTEXT_PACK_MISSING", str(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("decision") != "DR-alpha B: context uniqueness":
        return fail("E_TAG_CONTEXT_DECISION", str(contract.get("decision")))
    rows = [
        json.loads(line)
        for line in (PACK / "golden.jsonl").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    codes = {row.get("expected_error_code") for row in rows if row.get("expected_error_code")}
    if "E_TAG_DUPLICATE_IN_CONTEXT" not in codes:
        return fail("E_TAG_CONTEXT_DIAG", "missing E_TAG_DUPLICATE_IN_CONTEXT golden")
    print("lang tag context uniqueness pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

