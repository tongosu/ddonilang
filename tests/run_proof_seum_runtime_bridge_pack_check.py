#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "proof_seum_runtime_bridge_v1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def main() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input_pass.ddn",
        PACK / "input_fail.ddn",
        PACK / "golden.jsonl",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_PROOF_SEUM_RUNTIME_MISSING", str(missing))
    contract = json.loads((PACK / "contract.detjson").read_text(encoding="utf-8"))
    if contract.get("schema") != "ddn.proof_seum_runtime_bridge.pack.contract.v1":
        return fail("E_PROOF_SEUM_RUNTIME_SCHEMA", str(contract.get("schema")))
    rows = [
        json.loads(line)
        for line in (PACK / "golden.jsonl").read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    if {row.get("id") for row in rows} != {"c01_symbolic_equivalence_pass", "c02_symbolic_equivalence_fail"}:
        return fail("E_PROOF_SEUM_RUNTIME_CASES", str([row.get("id") for row in rows]))
    if "E_ECO_DIVERGENCE_DETECTED" not in {row.get("expected_error_code") for row in rows}:
        return fail("E_PROOF_SEUM_RUNTIME_FAIL_CASE", "missing divergence failure case")
    print("proof seum runtime bridge pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

