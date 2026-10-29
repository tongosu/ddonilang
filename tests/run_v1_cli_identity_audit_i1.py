#!/usr/bin/env python3
"""Fail-closed checker for the CLI01 I1 read-only audit."""

from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "work" / "codex" / "20260827_ddn_v1_cli01_official_cli_identity_audit_i1"


def load(name: str) -> dict:
    return json.loads((OUT / name).read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def main() -> None:
    registry = load("cli_command_registry.detjson")
    ownership = load("cli_semantic_ownership.detjson")
    dag = load("cli_migration_dag.detjson")
    rows = registry["commands"]
    checks = []

    def check(condition: bool, message: str) -> None:
        require(condition, message)
        checks.append(message)

    check(registry["tool_command_count"] == 28, "tool command count is 28")
    check(registry["teul_command_count"] == 63, "teul command count is 63")
    check(len(rows) == 91, "registry has 91 binary-command rows")
    check(len({(r["binary"], r["command"]) for r in rows}) == 91, "binary-command identities are unique")
    check(all(r["owner_decision_required"] for r in rows), "all dispositions remain owner proposals")
    check(not any("DROP" in r["disposition_proposal"] for r in rows), "I1 drops no commands")
    check(any(r["binary"] == "teul-cli" and r["command"] == "run" for r in rows), "teul run is inventoried")
    check(any(r["binary"] == "ddonirang-tool" and r["command"] == "run" for r in rows), "tool run is inventoried")
    check(ownership["verdict"] == "SOURCE_OWNERSHIP_CYCLE_WITHOUT_CARGO_CYCLE", "ownership-cycle verdict is explicit")
    kinds = {edge["kind"] for edge in ownership["edges"]}
    check("CARGO_DEPENDENCY" in kinds, "Cargo dependency edge is recorded")
    check(sum(kind == "PATH_SOURCE_IMPORT" for kind in [e["kind"] for e in ownership["edges"]]) == 2, "two cross-package path imports are recorded")
    check(all(Path(ROOT / edge["evidence"].split(":", 1)[0]).exists() for edge in ownership["edges"]), "ownership evidence files exist")
    ids = [node["id"] for node in dag["nodes"]]
    check(len(ids) == len(set(ids)), "migration node IDs are unique")
    known = set(ids)
    check(all(set(node["depends_on"]) <= known for node in dag["nodes"]), "migration dependencies exist")
    check(dag["nodes"][-1]["id"] == "C8", "binary retirement is the last gate")
    check(dag["nodes"][-1]["depends_on"] == ["C7"], "retirement requires independent parity review")
    check(registry["authority_state"] == "READ_ONLY_PROPOSAL_NOT_OWNER_DECISION", "audit is non-authoritative research")

    print(json.dumps({"status": "PASS", "checks": len(checks), "commands": len(rows)}, sort_keys=True))


if __name__ == "__main__":
    main()
