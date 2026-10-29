#!/usr/bin/env python3
"""Fail-closed checker for D-PACK-V1-PRODUCT-CLOSURE-CENSUS-R1."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "work/codex/20260827_ddn_v1_product_closure_long_range_r1"
REGISTRY = OUT / "v1_capability_registry.detjson"
MATRIX = OUT / "v1_implementation_matrix.detjson"
HORIZON = OUT / "product_version_horizon.detjson"
DAG = OUT / "v1_dependency_dag.detjson"
CONTRACT = ROOT / "pack/d_pack_v1_product_closure_census_r1/contract.detjson"

ALLOWED_IMPLEMENTATION = {
    "IMPLEMENTED_VERIFIED", "IMPLEMENTED_UNVERIFIED", "PARTIAL", "PLANNED",
    "NOT_IMPLEMENTED", "NOT_APPLICABLE", "UNKNOWN",
}
ALLOWED_MEMBERSHIP = {"REQUIRED", "OPTIONAL", "DEFERRED", "OPEN"}
VERSION_ORDER = {
    "V0.9": 9, "V1.0": 10, "V1.2": 12, "V1.5": 15, "V1.8": 18,
    "V2.0": 20, "V2.2": 22, "V2.5": 25, "V2.8": 28,
    "V3.0": 30, "V3.5": 35, "V4_PLUS": 40, "CONTINUOUS": 99, "OPEN": 100,
}
REQUIRED_EVIDENCE = {
    "a633c43f93299c3ec60151857e6aeb54c1d852ae",  # ForEach/current item
    "ab16db245f7ff7234cc604923e527355f5b7d53a",  # field surface
    "905efb4a",  # canonical view program closure lineage
    "de9cb9ed",  # G30 native CLI/Workbench review lineage
    "0e3de501fd5b95a47dba7ba67f2bb7f57583bbb3",  # RD206 handoff/review lineage
    "b8530bd6eadcf79658a821f4e9b4538991b89b80",  # current PC01 fixed head
}


def fail(message: str) -> None:
    raise AssertionError(message)


def load(path: Path) -> dict:
    try:
        raw = path.read_bytes()
    except FileNotFoundError:
        fail(f"missing required artifact: {path}")
    if raw.startswith(b"\xef\xbb\xbf"):
        fail(f"UTF-8 BOM forbidden: {path}")
    try:
        return json.loads(raw.decode("utf-8"))
    except Exception as exc:
        fail(f"invalid UTF-8 JSON {path}: {exc}")


def topo_check(rows: list[dict]) -> None:
    graph = {row["capability_id"]: row["dependency_ids"] for row in rows}
    visiting: set[str] = set()
    done: set[str] = set()

    def visit(node: str) -> None:
        if node in visiting:
            fail(f"dependency cycle at {node}")
        if node in done:
            return
        visiting.add(node)
        for dep in graph[node]:
            if dep not in graph:
                fail(f"unknown dependency {dep} from {node}")
            visit(dep)
        visiting.remove(node)
        done.add(node)

    for node in graph:
        visit(node)


def main() -> None:
    registry = load(REGISTRY)
    matrix = load(MATRIX)
    horizon = load(HORIZON)
    dag = load(DAG)
    contract = load(CONTRACT)
    rows = registry["capabilities"]

    if len(contract["checks"]) != 17:
        fail("contract must contain exactly 17 checks")
    if registry["authority"]["internal_ssot"] != "v25.15.0":
        fail("stale internal SSOT")
    if registry["authority"]["public_spec"] != "v0.5.0":
        fail("public SPEC custody mismatch")
    if registry["authority"]["authoritative_v1_membership_rows"] != 0:
        fail("authoritative SSOT membership must remain 0")
    if registry["authority"]["verified_product_floor"] != "UNCLAIMABLE":
        fail("floor must remain UNCLAIMABLE")

    ids = [row["capability_id"] for row in rows]
    if len(ids) != len(set(ids)):
        fail("duplicate capability id")
    names = [row["korean_name"] for row in rows]
    if len(names) != len(set(names)):
        fail("duplicate semantic family without alias record")

    axes = [
        "lexer_status", "parser_status", "ast_status", "typed_ir_status",
        "typecheck_status", "canon_status", "formatter_status", "runtime_status",
        "tool_status", "teul_cli_status", "official_cli_status", "native_status",
        "wasm_status", "runtime_bridge_status", "workbench_status",
        "diagnostic_status", "migration_status", "pack_status", "conformance_status",
    ]
    evidence_text = "\n".join(ref for row in rows for ref in row["evidence_refs"])
    for row in rows:
        cid = row["capability_id"]
        if not row["authority_source"] or not row["authority_version"]:
            fail(f"missing authority: {cid}")
        if row["v1_membership"] not in ALLOWED_MEMBERSHIP:
            fail(f"invalid membership: {cid}")
        if row["authoritative_ssot_membership"] != "OPEN_NOT_YET_POPULATED":
            fail(f"roadmap membership promoted as SSOT: {cid}")
        if row["v1_membership"] == "REQUIRED" and row["required_closure_gate_status"] not in {"OPEN", "UNKNOWN"}:
            fail(f"unreviewed required gate closed: {cid}")
        if row["v1_membership"] != "REQUIRED" and row["required_closure_gate_status"] != "NOT_IN_V1_FLOOR":
            fail(f"optional/deferred leaked into floor: {cid}")
        if VERSION_ORDER[row["required_from"]] < VERSION_ORDER[row["first_target_version"]]:
            fail(f"required before first target: {cid}")
        for axis in axes:
            if row[axis] not in ALLOWED_IMPLEMENTATION:
                fail(f"invalid implementation status {axis}: {cid}")
        maturity = row["current_product_maturity"]
        if maturity == "IMPLEMENTED_VERIFIED":
            if not row["evidence_refs"]:
                fail(f"verified without evidence: {cid}")
            if all(ref.startswith("docs/") for ref in row["evidence_refs"]):
                fail(f"docs-only counted product-closed: {cid}")
        if "UNSUPPORTED" in maturity or maturity == "PASS":
            fail(f"unsupported/pass used as maturity: {cid}")

    by_id = {row["capability_id"]: row for row in rows}
    topo_check(rows)
    for row in rows:
        for dep in row["dependency_ids"]:
            dep_row = by_id[dep]
            if row["v1_membership"] == "REQUIRED" and dep_row["v1_membership"] == "REQUIRED":
                if VERSION_ORDER[dep_row["required_from"]] > VERSION_ORDER[row["required_from"]]:
                    fail(f"dependency required too late: {dep} -> {row['capability_id']}")

    for ref in REQUIRED_EVIDENCE:
        if ref not in evidence_text:
            fail(f"required historical/current evidence lost: {ref}")

    retirement = (OUT / "09_LEGACY_AUTHORITY_RETIREMENT_PLAN.md").read_text(encoding="utf-8")
    for token in ("tools/teul-cli/src/lang", "tools/teul-cli/src/runtime", "lang/src/currentline.rs", "source_readiness.js"):
        if token not in retirement:
            fail(f"hidden semantic authority candidate missing: {token}")

    if len(matrix["rows"]) != len(rows) or len(horizon["rows"]) != len(rows) or len(dag["nodes"]) != len(rows):
        fail("registry/matrix/horizon/dag row count mismatch")

    # Regenerate and require byte stability for all four machine artifacts.
    before = {p: hashlib.sha256(p.read_bytes()).hexdigest() for p in (REGISTRY, MATRIX, HORIZON, DAG)}
    result = subprocess.run([sys.executable, str(ROOT / "tools/generate_v1_product_closure_program_r1.py")], cwd=ROOT, capture_output=True, text=True)
    if result.returncode != 0:
        fail(f"generator failed: {result.stdout}\n{result.stderr}")
    after = {p: hashlib.sha256(p.read_bytes()).hexdigest() for p in before}
    if before != after:
        fail("machine artifacts are not deterministic under regeneration")

    counts = {m: sum(row["v1_membership"] == m for row in rows) for m in ALLOWED_MEMBERSHIP}
    print(json.dumps({
        "status": "PASS",
        "pack": "D-PACK-V1-PRODUCT-CLOSURE-CENSUS-R1",
        "checks": len(contract["checks"]),
        "capabilities": len(rows),
        "required": counts["REQUIRED"],
        "optional": counts["OPTIONAL"],
        "deferred": counts["DEFERRED"],
        "authoritative_membership_rows": 0,
        "verified_product_floor": "UNCLAIMABLE",
    }, ensure_ascii=False, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except AssertionError as exc:
        print(json.dumps({"status": "FAIL", "reason": str(exc)}, ensure_ascii=False, sort_keys=True))
        raise SystemExit(1)
