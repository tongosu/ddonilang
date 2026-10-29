#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACKS = [
    "econ_model_supply_demand_equilibrium_v1",
    "econ_model_money_supply_v1",
    "econ_model_tax_welfare_v1",
]
AGENT_FIRST_SLOTS = [
    "agent_first_household_v1",
    "agent_first_firm_v1",
    "agent_first_market_v1",
    "agent_first_government_v1",
    "agent_first_central_bank_v1",
]


def fail(message: str) -> int:
    print(f"econ_model_first_g1_check: FAIL: {message}", file=sys.stderr)
    return 1


def main() -> int:
    for pack_id in PACKS:
        pack = ROOT / "pack" / pack_id
        required = [pack / "README.md", pack / "contract.detjson", pack / "input.ddn", pack / "golden.jsonl"]
        missing = [path.relative_to(ROOT).as_posix() for path in required if not path.exists()]
        if missing:
            return fail(f"{pack_id}: missing files: {missing}")

        readme = (pack / "README.md").read_text(encoding="utf-8")
        contract = json.loads((pack / "contract.detjson").read_text(encoding="utf-8"))
        if contract.get("schema") != "ddn.econ_model_first_g1.contract.v1":
            return fail(f"{pack_id}: contract schema mismatch")
        if contract.get("evidence_tier") != "skeleton/docs_first":
            return fail(f"{pack_id}: evidence_tier mismatch")
        if contract.get("model_first") is not True or contract.get("product_behavior_claim") is not False:
            return fail(f"{pack_id}: model/product claim mismatch")
        if contract.get("agent_first_symmetry_slots") != AGENT_FIRST_SLOTS:
            return fail(f"{pack_id}: agent-first symmetry slot mismatch")
        for token in ["evidence_tier: `skeleton/docs_first`", "agent-first", "real-world prediction"]:
            if token not in readme:
                return fail(f"{pack_id}: README missing token {token!r}")

    proc = subprocess.run(
        [sys.executable, "tests/run_pack_golden.py", *PACKS],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=120,
    )
    if proc.returncode != 0:
        return fail(proc.stderr.strip() or proc.stdout.strip() or "run_pack_golden failed")

    print("econ_model_first_g1_check: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
