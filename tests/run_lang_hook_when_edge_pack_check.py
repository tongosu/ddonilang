#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


PACK_SCHEMA = "ddn.lang_hook_when_edge.pack.contract.v1"
REQUIRED_CASES = {
    "c01_false_true_edge_once": ["1"],
    "c02_same_madi_no_refire": ["1"],
}


def fail(code: str, msg: str) -> int:
    print(f"[lang-hook-when-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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
    parser = argparse.ArgumentParser(description="Lang hook when-edge pack checker")
    parser.add_argument("--pack", default="pack/lang_hook_when_edge_v1")
    args = parser.parse_args()

    pack = Path(args.pack)
    required = [pack / "README.md", pack / "intent.md", pack / "contract.detjson", pack / "golden.jsonl"]
    required.extend((pack / case_id / "input.ddn") for case_id in REQUIRED_CASES)
    missing = [str(path).replace("\\", "/") for path in required if not path.exists()]
    if missing:
        return fail("E_LANG_HOOK_WHEN_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
        golden = load_jsonl(pack / "golden.jsonl")
    except ValueError as exc:
        return fail("E_LANG_HOOK_WHEN_INVALID", str(exc))

    if not isinstance(contract, dict):
        return fail("E_LANG_HOOK_WHEN_CONTRACT_TYPE", "contract root must be object")
    if str(contract.get("schema", "")).strip() != PACK_SCHEMA:
        return fail("E_LANG_HOOK_WHEN_SCHEMA", f"schema={contract.get('schema')}")
    if str(contract.get("evidence_tier", "")).strip() != "runner_fill":
        return fail("E_LANG_HOOK_WHEN_TIER", f"tier={contract.get('evidence_tier')}")
    if str(contract.get("closure_claim", "")).strip() != "no":
        return fail("E_LANG_HOOK_WHEN_CLOSURE", f"closure={contract.get('closure_claim')}")

    cases = contract.get("cases")
    if not isinstance(cases, list):
        return fail("E_LANG_HOOK_WHEN_CASES_TYPE", "cases must be list")
    case_ids = {str(row.get("id", "")).strip() for row in cases if isinstance(row, dict)}
    if case_ids != set(REQUIRED_CASES):
        return fail("E_LANG_HOOK_WHEN_CASE_SET", f"cases={sorted(case_ids)}")

    golden_index = {str(row.get("id", "")).strip(): row for row in golden}
    if set(golden_index) != set(REQUIRED_CASES):
        return fail("E_LANG_HOOK_WHEN_GOLDEN_SET", f"golden={sorted(golden_index)}")
    for case_id, expected_stdout in REQUIRED_CASES.items():
        row = golden_index[case_id]
        cmd = row.get("cmd")
        if not isinstance(cmd, list) or len(cmd) < 2 or cmd[0] != "run":
            return fail("E_LANG_HOOK_WHEN_CMD", f"id={case_id}")
        if cmd[1] != f"pack/lang_hook_when_edge_v1/{case_id}/input.ddn":
            return fail("E_LANG_HOOK_WHEN_CMD_TARGET", f"id={case_id} cmd={cmd}")
        if row.get("stdout") != expected_stdout:
            return fail("E_LANG_HOOK_WHEN_STDOUT", f"id={case_id} stdout={row.get('stdout')}")
        if int(row.get("exit_code", -1)) != 0:
            return fail("E_LANG_HOOK_WHEN_EXIT", f"id={case_id} exit={row.get('exit_code')}")

    print("[lang-hook-when-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(golden_index)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
