#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKS = {
    "proof_tactic_symbolic_eq_v1": "ddn.proof_tactic_symbolic_eq.pack.contract.v1",
    "proof_tactic_rewrite_chain_v1": "ddn.proof_tactic_rewrite_chain.pack.contract.v1",
    "proof_ddn_jeunggeo_bridge_v1": "ddn.proof_ddn_jeunggeo_bridge.pack.contract.v1",
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
            return fail("E_PROOF_TACTIC_MISSING", f"{name}: {missing}")
        contract = json.loads((pack / "contract.detjson").read_text(encoding="utf-8"))
        if contract.get("schema") != schema:
            return fail("E_PROOF_TACTIC_SCHEMA", name)
        rows = [json.loads(line) for line in (pack / "golden.jsonl").read_text(encoding="utf-8").splitlines() if line.strip()]
        if not rows:
            return fail("E_PROOF_TACTIC_EMPTY", name)
    print("proof tactic pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

