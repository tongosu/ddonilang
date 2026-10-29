#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_primary_view_contract_v1"
RULES = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "primary_view_rules.v1.detjson"
WORLDS = {
    "rep_math_function_line_v1": "graph",
    "rep_phys_projectile_xy_v1": "space2d",
    "rep_econ_supply_demand_tax_v1": "graph",
}


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def run(cmd: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def check_files_and_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        RULES,
        ROOT / "tests" / "seamgrim_primary_view_contract_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_PRIMARY_VIEW_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("pack") != "seamgrim_primary_view_contract_v1":
        return fail("E_SEAMGRIM_PRIMARY_VIEW_PACK", repr(contract))
    if contract.get("heuristic_selection") is not False:
        return fail("E_SEAMGRIM_PRIMARY_VIEW_HEURISTIC", repr(contract.get("heuristic_selection")))
    for key in ("parser_change", "runtime_semantics_change", "state_hash_formula_change", "docs_ssot_change"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_PRIMARY_VIEW_BOUNDARY", f"{key}={contract.get(key)!r}")
    return 0


def check_rules_and_meta() -> int:
    rules = json.loads(read(RULES))
    if rules.get("schema_id") != "seamgrim.primary_view_rules" or rules.get("schema_version") != "v1":
        return fail("E_SEAMGRIM_PRIMARY_VIEW_RULE_SCHEMA", repr(rules))
    if not str(rules.get("ruleset_hash", "")).startswith("candidate:fnv1a32:primary_view_rules:"):
        return fail("E_SEAMGRIM_PRIMARY_VIEW_RULE_HASH", repr(rules.get("ruleset_hash")))
    rows = rules.get("rules")
    if not isinstance(rows, list) or len(rows) != 3:
        return fail("E_SEAMGRIM_PRIMARY_VIEW_RULE_COUNT", repr(rows))
    actual = {str(row.get("lesson_id")): str(row.get("primary_view")) for row in rows if isinstance(row, dict)}
    if actual != WORLDS:
        return fail("E_SEAMGRIM_PRIMARY_VIEW_RULES", repr(actual))
    if rules.get("heuristic_selection") is not False:
        return fail("E_SEAMGRIM_PRIMARY_VIEW_RULE_HEURISTIC", repr(rules.get("heuristic_selection")))

    for lesson_id, primary_view in WORLDS.items():
        meta = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / lesson_id / "meta.toml")
        for token in [
            f'primary_view = "{primary_view}"',
            'primary_view_rules_path = "solutions/seamgrim_ui_mvp/lessons/primary_view_rules.v1.detjson"',
        ]:
            if token not in meta:
                return fail("E_SEAMGRIM_PRIMARY_VIEW_META", f"{lesson_id}: {token}")
    return 0


def check_product_sources() -> int:
    app = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js")
    run_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js")
    for token in ["buildPrimaryViewContract", "lesson_declaration_hash", "primary_view_rules_path"]:
        if token not in app:
            return fail("E_SEAMGRIM_PRIMARY_VIEW_APP", token)
    for token in [
        "synthesizeGraphViewFromObservation",
        "lessonPrimaryViewContract",
        "primaryViewHeuristicSelection",
        "mainGraphPointCount",
        'declaredPrimaryView === "space2d"',
    ]:
        if token not in run_js:
            return fail("E_SEAMGRIM_PRIMARY_VIEW_RUN_JS", token)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_PRIMARY_VIEW_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_PRIMARY_VIEW_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_files_and_contract, check_rules_and_meta, check_product_sources):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "tests/seamgrim_primary_view_contract_runner.mjs"], "E_SEAMGRIM_PRIMARY_VIEW_NODE_CHECK", 60),
        (["node", "tests/seamgrim_primary_view_contract_runner.mjs"], "E_SEAMGRIM_PRIMARY_VIEW_BROWSER", 240),
        (["python", "tests/run_pack_golden.py", "seamgrim_primary_view_contract_v1"], "E_SEAMGRIM_PRIMARY_VIEW_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-primary-view-contract-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
