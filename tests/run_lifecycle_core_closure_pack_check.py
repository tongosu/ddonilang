#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


PACK_SCHEMA = "ddn.lifecycle_core_closure.pack.contract.v1"
RUN_CASES = {
    "c01_reset_restores_start_snapshot": ["run", "tools/teul-cli/tests/golden/W111/W111_G03_reset_nuri_restores_start_snapshot/main.ddn", "--madi", "3"],
    "c02_reset_all_restores_live_input_snapshot": ["run", "tools/teul-cli/tests/golden/W111/W111_G35_reset_all_restores_live_input_snapshot/main.ddn", "--madi", "4", "--state", "샘.키보드.누르고있음.ArrowRight=1", "--sam", "pack/gogae5_w49_latency_madi/sam/right_hold_2ticks.input.bin"],
    "c03_start_next_executes_madang": ["run", "tools/teul-cli/tests/golden/W111/W111_G09_lifecycle_transition_start_next_executes_madang/main.ddn"],
    "c04_next_wraps_last_to_first_madang": ["run", "tools/teul-cli/tests/golden/W111/W111_G14_lifecycle_next_wraps_last_to_first_madang/main.ddn"],
    "c05_active_pan_replay": ["run", "tools/teul-cli/tests/golden/W111/W111_G10_lifecycle_transition_call_replays_active_pan/main.ddn"]
}
SMOKE_CASE = ("c06_state_view_boundary", "smoke_with_view_boundary.v1.json")


def fail(code: str, msg: str) -> int:
    print(f"[lifecycle-core-closure-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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
    parser = argparse.ArgumentParser(description="Lifecycle core closure pack checker")
    parser.add_argument("--pack", default="pack/lifecycle_core_closure_v1")
    args = parser.parse_args()

    pack = Path(args.pack)
    required = [pack / "README.md", pack / "intent.md", pack / "contract.detjson", pack / "golden.jsonl", pack / "tests" / "README.md", pack / SMOKE_CASE[1]]
    required.extend((pack / "cases" / case_id / "expected.json") for case_id in list(RUN_CASES) + [SMOKE_CASE[0]])
    missing = [str(path).replace("\\", "/") for path in required if not path.exists()]
    if missing:
        return fail("E_LIFECYCLE_CORE_CLOSURE_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
        golden = load_jsonl(pack / "golden.jsonl")
    except ValueError as exc:
        return fail("E_LIFECYCLE_CORE_CLOSURE_INVALID", str(exc))

    if not isinstance(contract, dict):
        return fail("E_LIFECYCLE_CORE_CLOSURE_CONTRACT_TYPE", "contract root must be object")
    if str(contract.get("schema", "")).strip() != PACK_SCHEMA:
        return fail("E_LIFECYCLE_CORE_CLOSURE_SCHEMA", f"schema={contract.get('schema')}")
    if str(contract.get("evidence_tier", "")).strip() != "golden_closed":
        return fail("E_LIFECYCLE_CORE_CLOSURE_TIER", f"tier={contract.get('evidence_tier')}")
    if str(contract.get("closure_claim", "")).strip() != "yes":
        return fail("E_LIFECYCLE_CORE_CLOSURE_CLAIM", f"closure={contract.get('closure_claim')}")
    for rel in contract.get("evidence_targets", []):
        if not Path(rel).exists():
            return fail("E_LIFECYCLE_CORE_CLOSURE_TARGET", rel)

    golden_index = {str(row.get("id", "")).strip(): row for row in golden}
    if set(golden_index) != set(RUN_CASES) | {SMOKE_CASE[0]}:
        return fail("E_LIFECYCLE_CORE_CLOSURE_GOLDEN_SET", f"golden={sorted(golden_index)}")

    for case_id, expected_cmd in RUN_CASES.items():
        expected = load_json(pack / "cases" / case_id / "expected.json")
        if not isinstance(expected, dict):
            return fail("E_LIFECYCLE_CORE_CLOSURE_EXPECTED", case_id)
        golden_row = golden_index[case_id]
        if golden_row.get("cmd") != expected_cmd:
            return fail("E_LIFECYCLE_CORE_CLOSURE_CMD", case_id)
        if golden_row.get("stdout") != expected.get("stdout"):
            return fail("E_LIFECYCLE_CORE_CLOSURE_STDOUT", case_id)
        if int(golden_row.get("exit_code", -1)) != 0:
            return fail("E_LIFECYCLE_CORE_CLOSURE_EXIT", case_id)

    smoke_expected = load_json(pack / "cases" / SMOKE_CASE[0] / "expected.json")
    smoke_row = golden_index[SMOKE_CASE[0]]
    if str(smoke_row.get("smoke_golden", "")).strip() != SMOKE_CASE[1]:
        return fail("E_LIFECYCLE_CORE_CLOSURE_SMOKE", SMOKE_CASE[0])
    if str(smoke_expected.get("target", "")).strip() != SMOKE_CASE[1]:
        return fail("E_LIFECYCLE_CORE_CLOSURE_SMOKE_EXPECTED", SMOKE_CASE[0])

    print("[lifecycle-core-closure-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(golden_index)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
