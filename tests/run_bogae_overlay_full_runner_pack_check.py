#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


PACK_SCHEMA = "ddn.bogae_overlay_full_runner.pack.contract.v1"
REQUIRED_CASES = {
    "c01_primary_single": ("primary_1", "overlay_compare_case", "cases/c01_primary_single/case.detjson"),
    "c02_overlay_stack_order": ("overlay_stack_z_order", "overlay_session_case", "cases/c02_overlay_stack_order/case.detjson"),
    "c03_visibility_gate": ("visibility", "overlay_session_case", "cases/c03_visibility_gate/case.detjson"),
    "c04_state_hash_excluded": ("view_only_state_hash_excluded", "smoke_golden", "smoke_with_view_boundary.v1.json"),
}


def fail(code: str, msg: str) -> int:
    print(f"[bogae-overlay-full-runner-pack-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_json(path: Path) -> object:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise ValueError(f"missing file: {path}")
    except Exception as exc:
        raise ValueError(f"invalid json: {path} ({exc})")


def load_jsonl(path: Path) -> list[dict]:
    rows: list[dict] = []
    for line_no, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.strip()
        if not line:
            continue
        try:
            row = json.loads(line)
        except Exception as exc:
            raise ValueError(f"invalid jsonl row: {path}:{line_no} ({exc})")
        if not isinstance(row, dict):
            raise ValueError(f"jsonl row must be object: {path}:{line_no}")
        rows.append(row)
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description="Bogae overlay full-runner supporting pack checker")
    parser.add_argument("--pack", default="pack/bogae_overlay_full_runner_v1")
    args = parser.parse_args()

    pack = Path(args.pack)
    required = [pack / "README.md", pack / "intent.md", pack / "contract.detjson", pack / "golden.jsonl", pack / "tests" / "README.md"]
    required.extend((pack / "cases" / case_id / "expected.json") for case_id in REQUIRED_CASES)
    missing = [str(path).replace("\\", "/") for path in required if not path.exists()]
    if missing:
        return fail("E_BOGAE_OVERLAY_FULL_RUNNER_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
        golden = load_jsonl(pack / "golden.jsonl")
    except ValueError as exc:
        return fail("E_BOGAE_OVERLAY_FULL_RUNNER_INVALID", str(exc))

    if not isinstance(contract, dict):
        return fail("E_BOGAE_OVERLAY_FULL_RUNNER_CONTRACT_TYPE", "contract root must be object")
    if str(contract.get("schema", "")).strip() != PACK_SCHEMA:
        return fail("E_BOGAE_OVERLAY_FULL_RUNNER_SCHEMA", f"schema={contract.get('schema')}")
    if str(contract.get("evidence_tier", "")).strip() != "runner_fill":
        return fail("E_BOGAE_OVERLAY_FULL_RUNNER_TIER", f"tier={contract.get('evidence_tier')}")
    if str(contract.get("closure_claim", "")).strip() != "no":
        return fail("E_BOGAE_OVERLAY_FULL_RUNNER_CLOSURE", f"closure={contract.get('closure_claim')}")

    for rel in contract.get("backing_runner_checks", []):
        if not Path(rel).exists():
            return fail("E_BOGAE_OVERLAY_FULL_RUNNER_BACKING_CHECK", rel)
    for rel in contract.get("evidence_targets", []):
        if not Path(rel).exists():
            return fail("E_BOGAE_OVERLAY_FULL_RUNNER_TARGET", rel)

    cases = contract.get("cases")
    if not isinstance(cases, list):
        return fail("E_BOGAE_OVERLAY_FULL_RUNNER_CASES_TYPE", "cases must be list")
    case_index = {str(row.get("id", "")).strip(): row for row in cases if isinstance(row, dict)}
    if set(case_index) != set(REQUIRED_CASES):
        return fail("E_BOGAE_OVERLAY_FULL_RUNNER_CASE_SET", f"cases={sorted(case_index)}")
    golden_index = {str(row.get("id", "")).strip(): row for row in golden}
    if set(golden_index) != set(REQUIRED_CASES):
        return fail("E_BOGAE_OVERLAY_FULL_RUNNER_GOLDEN_SET", f"golden={sorted(golden_index)}")
    for case_id, (invariant, runner_mode, runner_ref) in REQUIRED_CASES.items():
        row = case_index[case_id]
        if str(row.get("invariant", "")).strip() != invariant:
            return fail("E_BOGAE_OVERLAY_FULL_RUNNER_INVARIANT", f"id={case_id}")
        if str(row.get("runner_mode", "")).strip() != runner_mode:
            return fail("E_BOGAE_OVERLAY_FULL_RUNNER_MODE", f"id={case_id}")
        if str(row.get("runner_ref", "")).strip() != runner_ref:
            return fail("E_BOGAE_OVERLAY_FULL_RUNNER_REF", f"id={case_id}")
        expected = load_json(pack / "cases" / case_id / "expected.json")
        if not isinstance(expected, dict) or str(expected.get("case_id", "")).strip() != case_id:
            return fail("E_BOGAE_OVERLAY_FULL_RUNNER_EXPECTED", case_id)
        if str(expected.get("runner_mode", "")).strip() != runner_mode:
            return fail("E_BOGAE_OVERLAY_FULL_RUNNER_EXPECTED_MODE", case_id)
        if str(expected.get("runner_ref", "")).strip() != runner_ref:
            return fail("E_BOGAE_OVERLAY_FULL_RUNNER_EXPECTED_REF", case_id)
        golden_row = golden_index[case_id]
        if int(golden_row.get("exit_code", -1)) != 0:
            return fail("E_BOGAE_OVERLAY_FULL_RUNNER_EXIT", case_id)
        if str(golden_row.get(runner_mode, "")).strip() != runner_ref:
            return fail("E_BOGAE_OVERLAY_FULL_RUNNER_GOLDEN_REF", case_id)

    print("[bogae-overlay-full-runner-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(golden_index)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
