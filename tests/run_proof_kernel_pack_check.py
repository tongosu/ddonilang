#!/usr/bin/env python
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PACKS = {
    "proof_kernel_term_replay_v1": "ddn.proof_kernel_term_replay.pack.contract.v1",
    "proof_numeric_certificate_verify_v1": "ddn.proof_numeric_certificate_verify.pack.contract.v1",
    "proof_symbolic_rewrite_verify_v1": "ddn.proof_symbolic_rewrite_verify.pack.contract.v1",
    "proof_seum_bridge_v1": "ddn.proof_seum_bridge.pack.contract.v1",
    "proof_relation_equivalence_v1": "ddn.proof_relation_equivalence.pack.contract.v1",
    "proof_relation_solve_consistency_v1": "ddn.proof_relation_solve_consistency.pack.contract.v1",
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
            return fail("E_PROOF_PACK_MISSING", f"{pack_name}: {missing}")
        contract = load_json(pack / "contract.detjson")
        if contract.get("schema") != schema:
            return fail("E_PROOF_PACK_SCHEMA", f"{pack_name}: {contract.get('schema')}")
        rows = load_jsonl(pack / "golden.jsonl")
        if not rows:
            return fail("E_PROOF_GOLDEN_EMPTY", pack_name)
        for row in rows:
            cmd = row.get("cmd") or []
            if len(cmd) < 3 or cmd[0] != "proof":
                return fail("E_PROOF_CMD", f"{pack_name}:{row.get('id')} cmd={cmd}")
            fixture = ROOT / cmd[2]
            if not fixture.exists():
                return fail("E_PROOF_FIXTURE_MISSING", str(fixture.relative_to(ROOT)))
    print("proof kernel pack check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
