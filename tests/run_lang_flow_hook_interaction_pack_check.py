#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


PACK_SCHEMA = "ddn.lang_flow_hook_interaction.pack.contract.v1"
REQUIRED_CASES = {
    "c01_flow_then_hook_snapshot": ("ok", ""),
    "c02_no_same_tick_refire": ("ok", ""),
    "c03_multiple_flow_source_conflict": ("diag", "E_FLOW_MULTIPLE_SOURCE_CONFLICT"),
    "c04_flow_cycle_fatal": ("diag", "E_FLOW_CIRCULAR_REFERENCE"),
}


def fail(code: str, msg: str) -> int:
    print(f"[lang-flow-hook-pack-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_json(path: Path) -> object:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise ValueError(f"missing file: {path}")
    except Exception as exc:
        raise ValueError(f"invalid json: {path} ({exc})")


def main() -> int:
    parser = argparse.ArgumentParser(description="Lang flow/hook interaction pack checker")
    parser.add_argument("--pack", default="pack/lang_flow_hook_interaction_v1")
    args = parser.parse_args()

    pack = Path(args.pack)
    required_files = [
        pack / "README.md",
        pack / "intent.md",
        pack / "contract.detjson",
        pack / "tests" / "README.md",
    ]
    missing = [str(path).replace("\\", "/") for path in required_files if not path.exists()]
    if missing:
        return fail("E_LANG_FLOW_HOOK_PACK_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
    except ValueError as exc:
        return fail("E_LANG_FLOW_HOOK_CONTRACT_INVALID", str(exc))

    if not isinstance(contract, dict):
        return fail("E_LANG_FLOW_HOOK_CONTRACT_TYPE", "contract root must be object")
    if str(contract.get("schema", "")).strip() != PACK_SCHEMA:
        return fail("E_LANG_FLOW_HOOK_SCHEMA", f"schema={contract.get('schema')}")
    if str(contract.get("evidence_tier", "")).strip() != "docs_first":
        return fail("E_LANG_FLOW_HOOK_EVIDENCE_TIER", f"tier={contract.get('evidence_tier')}")
    if str(contract.get("closure_claim", "")).strip() != "no":
        return fail("E_LANG_FLOW_HOOK_CLOSURE_CLAIM", f"closure={contract.get('closure_claim')}")
    if contract.get("ordering_contract") != [
        "ordinary_assignment",
        "flow_fixed_point",
        "tail_phase_hook",
    ]:
        return fail("E_LANG_FLOW_HOOK_ORDERING", f"ordering={contract.get('ordering_contract')}")

    cases = contract.get("cases")
    if not isinstance(cases, list):
        return fail("E_LANG_FLOW_HOOK_CASES_TYPE", "cases must be list")
    case_index = {
        str(row.get("id", "")).strip(): row for row in cases if isinstance(row, dict)
    }
    if set(case_index) != set(REQUIRED_CASES):
        return fail("E_LANG_FLOW_HOOK_CASE_SET", f"cases={sorted(case_index)}")

    for case_id, (case_kind, expected_error) in REQUIRED_CASES.items():
        row = case_index[case_id]
        if str(row.get("case_kind", "")).strip() != case_kind:
            return fail("E_LANG_FLOW_HOOK_CASE_KIND", f"id={case_id} kind={row.get('case_kind')}")
        if expected_error and str(row.get("expected_error", "")).strip() != expected_error:
            return fail("E_LANG_FLOW_HOOK_CASE_ERROR", f"id={case_id} error={row.get('expected_error')}")

        case_dir = pack / "cases" / case_id
        input_path = case_dir / "input.ddn"
        expected_path = case_dir / "expected.json"
        if not input_path.exists() or not expected_path.exists():
            return fail("E_LANG_FLOW_HOOK_CASE_FILE_MISSING", case_id)
        try:
            expected = load_json(expected_path)
        except ValueError as exc:
            return fail("E_LANG_FLOW_HOOK_EXPECTED_INVALID", str(exc))
        if not isinstance(expected, dict):
            return fail("E_LANG_FLOW_HOOK_EXPECTED_TYPE", case_id)
        if str(expected.get("case_id", "")).strip() != case_id:
            return fail("E_LANG_FLOW_HOOK_EXPECTED_ID", f"id={case_id}")
        if str(expected.get("case_kind", "")).strip() != case_kind:
            return fail("E_LANG_FLOW_HOOK_EXPECTED_KIND", f"id={case_id}")
        if expected_error and str(expected.get("expected_error", "")).strip() != expected_error:
            return fail("E_LANG_FLOW_HOOK_EXPECTED_ERROR", f"id={case_id}")

    print("[lang-flow-hook-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(case_index)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
