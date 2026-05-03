#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


PACK_SCHEMA = "ddn.geoul_record_query_timeline_minimum.pack.contract.v1"
REQUIRED_CASES = {
    "c01_record_check": ["geoul", "record", "check", "pack/geoul_min_schema_v0/record_ok.jsonl"],
    "c02_bundle_hash": ["geoul", "hash", "--geoul", "tools/teul-cli/tests/golden/W43/W43_G01_geoul_query/out/geoul"],
    "c03_state_seek": ["geoul", "seek", "--geoul", "tools/teul-cli/tests/golden/W43/W43_G01_geoul_query/out/geoul", "--madi", "3"],
    "c04_key_query": ["geoul", "query", "--geoul", "tools/teul-cli/tests/golden/W43/W43_G01_geoul_query/out/geoul", "--madi", "3", "--key", "점수"],
    "c05_key_backtrace": ["geoul", "backtrace", "--geoul", "tools/teul-cli/tests/golden/W43/W43_G02_geoul_backtrace/out/geoul", "--key", "점수", "--from", "0", "--to", "5"],
    "c06_story_make": ["story", "make", "--geoul", "tools/teul-cli/tests/golden/W43/W43_G01_geoul_query/out/geoul", "--out", "build/tmp/geoul_story.detjson"],
    "c07_timeline_make": ["timeline", "make", "--geoul", "tools/teul-cli/tests/golden/W43/W43_G01_geoul_query/out/geoul", "--story", "build/tmp/geoul_story.detjson", "--out", "build/tmp/geoul_timeline.detjson"],
}


def fail(code: str, msg: str) -> int:
    print(f"[geoul-record-query-timeline-minimum-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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
    parser = argparse.ArgumentParser(description="Geoul record/query/timeline minimum pack checker")
    parser.add_argument("--pack", default="pack/geoul_record_query_timeline_minimum_v1")
    args = parser.parse_args()

    pack = Path(args.pack)
    required = [pack / "README.md", pack / "intent.md", pack / "contract.detjson", pack / "golden.jsonl", pack / "tests" / "README.md"]
    required.extend((pack / "cases" / case_id / "expected.json") for case_id in REQUIRED_CASES)
    missing = [str(path).replace("\\", "/") for path in required if not path.exists()]
    if missing:
        return fail("E_GEOUL_MIN_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
        golden = load_jsonl(pack / "golden.jsonl")
    except ValueError as exc:
        return fail("E_GEOUL_MIN_INVALID", str(exc))

    if not isinstance(contract, dict):
        return fail("E_GEOUL_MIN_CONTRACT_TYPE", "contract root must be object")
    if str(contract.get("schema", "")).strip() != PACK_SCHEMA:
        return fail("E_GEOUL_MIN_SCHEMA", f"schema={contract.get('schema')}")
    if str(contract.get("evidence_tier", "")).strip() != "runner_fill":
        return fail("E_GEOUL_MIN_TIER", f"tier={contract.get('evidence_tier')}")
    if str(contract.get("closure_claim", "")).strip() != "no":
        return fail("E_GEOUL_MIN_CLOSURE", f"closure={contract.get('closure_claim')}")
    for rel in contract.get("evidence_targets", []):
        if not Path(rel).exists():
            return fail("E_GEOUL_MIN_TARGET", rel)

    cases = contract.get("cases")
    if not isinstance(cases, list):
        return fail("E_GEOUL_MIN_CASES_TYPE", "cases must be list")
    case_index = {str(row.get("id", "")).strip(): row for row in cases if isinstance(row, dict)}
    if set(case_index) != set(REQUIRED_CASES):
        return fail("E_GEOUL_MIN_CASE_SET", f"cases={sorted(case_index)}")

    golden_index = {str(row.get("id", "")).strip(): row for row in golden}
    if set(golden_index) != set(REQUIRED_CASES):
        return fail("E_GEOUL_MIN_GOLDEN_SET", f"golden={sorted(golden_index)}")

    for case_id, expected_cmd in REQUIRED_CASES.items():
        expected = load_json(pack / "cases" / case_id / "expected.json")
        if not isinstance(expected, dict) or str(expected.get("case_id", "")).strip() != case_id:
            return fail("E_GEOUL_MIN_EXPECTED", case_id)
        if str(case_index[case_id].get("command_kind", "")).strip() != str(expected.get("command_kind", "")).strip():
            return fail("E_GEOUL_MIN_KIND", case_id)
        golden_row = golden_index[case_id]
        if golden_row.get("cmd") != expected_cmd:
            return fail("E_GEOUL_MIN_CMD", case_id)
        if golden_row.get("stdout") != expected.get("stdout"):
            return fail("E_GEOUL_MIN_STDOUT", case_id)
        if int(golden_row.get("exit_code", -1)) != 0:
            return fail("E_GEOUL_MIN_EXIT", case_id)

    print("[geoul-record-query-timeline-minimum-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(golden_index)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
