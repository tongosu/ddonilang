#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


PACK_SCHEMA = "ddn.lang_generic_iterable_strict.pack.contract.v1"
REQUIRED_CASES = {
    "c01_finite_snapshot_required": ("finite_snapshot_required", "tools/teul-cli/tests/golden/W97/W97_G03_foreach_bad_iterable/main.ddn", "E_RUNTIME_TYPE_MISMATCH"),
    "c02_canonical_order_required": ("canonical_order_required", "tools/teul-cli/tests/golden/W97/W97_G04_foreach_map/main.ddn", ["차림[a, 1]", "차림[b, 2]"]),
    "c03_deterministic_exhaustion_required": ("deterministic_exhaustion_required", "pack/stdlib_range_basics/input.ddn", ["차림[1, 2, 3]", "차림[0, 2, 4]", "차림[3, 2, 1]", "차림[]", "차림[0@m, 1@m, 2@m, 3@m]"]),
}


def fail(code: str, msg: str) -> int:
    print(f"[lang-generic-iterable-strict-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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
    parser = argparse.ArgumentParser(description="Lang generic iterable strict supporting pack checker")
    parser.add_argument("--pack", default="pack/lang_generic_iterable_strict_v1")
    args = parser.parse_args()

    pack = Path(args.pack)
    required = [pack / "README.md", pack / "intent.md", pack / "contract.detjson", pack / "golden.jsonl", pack / "tests" / "README.md"]
    required.extend((pack / "cases" / case_id / "expected.json") for case_id in REQUIRED_CASES)
    missing = [str(path).replace("\\", "/") for path in required if not path.exists()]
    if missing:
        return fail("E_LANG_GENERIC_ITERABLE_STRICT_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
        golden = load_jsonl(pack / "golden.jsonl")
    except ValueError as exc:
        return fail("E_LANG_GENERIC_ITERABLE_STRICT_INVALID", str(exc))

    if not isinstance(contract, dict):
        return fail("E_LANG_GENERIC_ITERABLE_STRICT_CONTRACT_TYPE", "contract root must be object")
    if str(contract.get("schema", "")).strip() != PACK_SCHEMA:
        return fail("E_LANG_GENERIC_ITERABLE_STRICT_SCHEMA", f"schema={contract.get('schema')}")
    if str(contract.get("evidence_tier", "")).strip() != "runner_fill":
        return fail("E_LANG_GENERIC_ITERABLE_STRICT_TIER", f"tier={contract.get('evidence_tier')}")
    if str(contract.get("closure_claim", "")).strip() != "no":
        return fail("E_LANG_GENERIC_ITERABLE_STRICT_CLOSURE", f"closure={contract.get('closure_claim')}")
    for rel in contract.get("evidence_targets", []):
        if not Path(rel).exists():
            return fail("E_LANG_GENERIC_ITERABLE_STRICT_TARGET", rel)

    cases = contract.get("cases")
    if not isinstance(cases, list):
        return fail("E_LANG_GENERIC_ITERABLE_STRICT_CASES_TYPE", "cases must be list")
    case_index = {str(row.get("id", "")).strip(): row for row in cases if isinstance(row, dict)}
    if set(case_index) != set(REQUIRED_CASES):
        return fail("E_LANG_GENERIC_ITERABLE_STRICT_CASE_SET", f"cases={sorted(case_index)}")
    golden_index = {str(row.get("id", "")).strip(): row for row in golden}
    if set(golden_index) != set(REQUIRED_CASES):
        return fail("E_LANG_GENERIC_ITERABLE_STRICT_GOLDEN_SET", f"golden={sorted(golden_index)}")
    for case_id, (rule, target, expectation) in REQUIRED_CASES.items():
        row = case_index[case_id]
        if str(row.get("rule", "")).strip() != rule:
            return fail("E_LANG_GENERIC_ITERABLE_STRICT_RULE", case_id)
        if str(row.get("runner_target", "")).strip() != target:
            return fail("E_LANG_GENERIC_ITERABLE_STRICT_TARGET_ROW", case_id)
        expected = load_json(pack / "cases" / case_id / "expected.json")
        if not isinstance(expected, dict) or str(expected.get("rule", "")).strip() != rule:
            return fail("E_LANG_GENERIC_ITERABLE_STRICT_EXPECTED", case_id)
        golden_row = golden_index[case_id]
        cmd = golden_row.get("cmd")
        if not isinstance(cmd, list) or len(cmd) < 2 or cmd[0] != "run" or cmd[1] != target:
            return fail("E_LANG_GENERIC_ITERABLE_STRICT_CMD", case_id)
        if isinstance(expectation, str):
            if str(golden_row.get("expected_error_code", "")).strip() != expectation:
                return fail("E_LANG_GENERIC_ITERABLE_STRICT_ERROR", case_id)
            if int(golden_row.get("exit_code", -1)) == 0:
                return fail("E_LANG_GENERIC_ITERABLE_STRICT_EXIT", case_id)
        else:
            if golden_row.get("stdout") != expectation:
                return fail("E_LANG_GENERIC_ITERABLE_STRICT_STDOUT", case_id)
            if int(golden_row.get("exit_code", -1)) != 0:
                return fail("E_LANG_GENERIC_ITERABLE_STRICT_EXIT", case_id)

    print("[lang-generic-iterable-strict-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(golden_index)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
