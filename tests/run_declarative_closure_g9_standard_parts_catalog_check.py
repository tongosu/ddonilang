#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "sim_standard_parts_catalog_v1"

EXPECTED_PARTS = {
    "mechanical": {"mass", "spring", "damper"},
    "circuit": {"resistor", "capacitor", "source"},
    "econ": {"supply-demand", "tax"},
    "queue": {"queue", "server"},
}

EXPECTED_LINKED = {
    ("circuit", "source"),
    ("econ", "supply-demand"),
    ("econ", "tax"),
}


def fail(message: str) -> None:
    print(f"declarative_closure_g9_standard_parts_catalog_check: FAIL: {message}", file=sys.stderr)
    sys.exit(1)


def run(cmd: list[str], *, timeout: int = 240) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
        check=False,
    )


def read_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8-sig"))
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON {path.relative_to(ROOT)}: {exc}")


def check_pack_golden() -> None:
    proc = run([sys.executable, "tests/run_pack_golden.py", "sim_standard_parts_catalog_v1"])
    if proc.returncode != 0:
        fail(f"pack golden failed:\n{proc.stdout}")


def check_catalog() -> None:
    contract = read_json(PACK / "contract.detjson")
    catalog = read_json(PACK / "standard_parts_catalog.detjson")
    if contract.get("new_language_block") is not False or catalog.get("new_language_block") is not False:
        fail("catalog must not create a language block")
    for field in ("runtime_claim", "parser_landed_claim", "state_hash_formula_change"):
        if contract.get(field) is not False or catalog.get(field) is not False:
            fail(f"{field} must be false in contract and catalog")

    families = catalog.get("families")
    if not isinstance(families, list):
        fail("catalog families must be a list")
    got_families = {family.get("family") for family in families}
    if got_families != set(EXPECTED_PARTS):
        fail(f"family mismatch: {got_families!r}")
    if set(contract.get("required_families", [])) != set(EXPECTED_PARTS):
        fail("contract required_families mismatch")

    for family in families:
        family_id = family["family"]
        parts = family.get("parts", [])
        got_parts = {part.get("part_id") for part in parts}
        if got_parts != EXPECTED_PARTS[family_id]:
            fail(f"{family_id} parts mismatch: {got_parts!r}")
        for part in parts:
            key = (family_id, part["part_id"])
            evidence = part.get("evidence", [])
            if key in EXPECTED_LINKED:
                if part.get("status") != "evidence_linked":
                    fail(f"{key!r} must be evidence_linked")
                if not evidence:
                    fail(f"{key!r} must link at least one evidence pack")
                for item in evidence:
                    evidence_path = ROOT / item.get("path", "")
                    if not evidence_path.exists():
                        fail(f"missing evidence path for {key!r}: {item!r}")
                    pack_name = item.get("pack")
                    if not pack_name or not (ROOT / "pack" / pack_name).exists():
                        fail(f"missing evidence pack for {key!r}: {item!r}")
            else:
                if part.get("status") != "candidate_only":
                    fail(f"{key!r} must remain candidate_only")
                if evidence:
                    fail(f"{key!r} candidate_only item must not link evidence")

    blocked = set(catalog.get("blocked_claims", []))
    for required in ("runtime component model", "new language block", "solver behavior"):
        if required not in blocked:
            fail(f"blocked claim missing: {required}")


def check_no_docs_ssot_change() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed:\n{proc.stdout}")
    if proc.stdout.strip():
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    check_pack_golden()
    check_catalog()
    check_no_docs_ssot_change()
    print("declarative_closure_g9_standard_parts_catalog_check: ok")


if __name__ == "__main__":
    main()
