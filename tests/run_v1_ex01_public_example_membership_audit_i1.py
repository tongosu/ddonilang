#!/usr/bin/env python3
"""Fail-closed checker for EX01 candidate membership custody."""

from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "work" / "codex" / "20260827_ddn_v1_ex01_public_example_membership_audit_i1"


def load(name: str) -> dict:
    return json.loads((OUT / name).read_text(encoding="utf-8"))


def main() -> None:
    ledger = load("example_candidate_ledger.detjson")
    dag = load("example_migration_dag.detjson")
    checks: list[str] = []

    def check(value: bool, label: str) -> None:
        if not value:
            raise SystemExit(f"FAIL: {label}")
        checks.append(label)

    rows = ledger["rows"]
    ids = [row["id"] for row in rows]
    paths = [row["path"] for row in rows]
    check(len(rows) == 15, "fifteen bounded candidates")
    check(len(ids) == len(set(ids)), "candidate IDs unique")
    check(len(paths) == len(set(paths)), "candidate paths unique")
    check(all((ROOT / path).is_file() for path in paths), "all candidate files exist")
    check(ledger["counts"]["public_spec_docs_first"] == 7, "seven docs-first SPEC examples")
    check(ledger["counts"]["pre_v25_parse"] == 6, "six pre-V25 parser successes")
    check(ledger["counts"]["v25_parse"] == 1, "one V25 parser success")
    check(ledger["counts"]["neither_parser"] == 8, "eight dual-parser failures")
    check(ledger["counts"]["proposed_tier_0_or_1"] == 4, "four first migration candidates")
    check(ledger["counts"]["release_members"] == 0, "release membership remains zero")
    check(ledger["recommended_first_migration_ids"] == ["PC01", "HELLO", "GRID_CURRENT", "TENSOR"], "first migration ordering fixed")
    check(all(not row["release_member"] for row in rows), "no self-promoted release member")
    check(all(len(row["sha256"]) == 64 for row in rows), "all source identities sealed")
    check(all(row["rights"] != "RIGHTS_CLOSED" for row in rows), "rights closure is not invented")
    spec_rows = [row for row in rows if row["role"] == "PUBLIC_SPEC_DOCS_FIRST"]
    check(len(spec_rows) == 7 and all(not row["v25_parse"] for row in spec_rows), "SPEC examples are not V25 product examples")
    pc01 = next(row for row in rows if row["id"] == "PC01")
    check(pc01["v25_parse"] and pc01["rights"] == "OWNER_CONFIRMATION_REQUIRED", "PC01 technical and rights gates separated")
    projectile = next(row for row in rows if row["id"] == "PROJECTILE")
    check(projectile["rights"] == "OWNER_CONFIRMATION_REQUIRED" and projectile["migration_tier"] == "TIER_2_AFTER_CORE", "projectile remains deferred and rights-open")
    check(ledger["rights_clean_release_membership_authorized"] is False, "rights-clean promotion not authorized")
    check(ledger["product_implementation_authorized"] is False, "product implementation not authorized by audit")
    node_ids = [node["id"] for node in dag["nodes"]]
    check(len(node_ids) == len(set(node_ids)), "DAG IDs unique")
    check(all(set(node["depends_on"]) <= set(node_ids) for node in dag["nodes"]), "DAG dependencies exist")
    check(dag["nodes"][-1]["name"] == "owner_promote_exact_release_membership", "Owner promotion is final gate")
    check("SELF_PROMOTE_RELEASE_MEMBERSHIP" in dag["forbidden"], "self-promotion forbidden")
    print(json.dumps({"status": "PASS", "checks": len(checks), "candidates": len(rows)}, sort_keys=True))


if __name__ == "__main__":
    main()

