#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


PACK_SCHEMA = "ddn.lang_pragmatism.pack.contract.v1"
REQUIRED_CASES = {
    "c01_range_iteration": "pack/stdlib_range_basics/input.ddn",
    "c02_charim_ops": "pack/stdlib_charim_basics/input.ddn",
    "c03_text_ops": "pack/stdlib_text_basics/input.ddn",
}


def fail(code: str, msg: str) -> int:
    print(f"[lang-pragmatism-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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
    parser = argparse.ArgumentParser(description="Lang pragmatism pack checker")
    parser.add_argument("--pack", default="pack/lang_pragmatism_pack_v1")
    args = parser.parse_args()

    pack = Path(args.pack)
    required_files = [
        pack / "README.md",
        pack / "intent.md",
        pack / "contract.detjson",
        pack / "golden.jsonl",
    ]
    missing = [str(path).replace("\\", "/") for path in required_files if not path.exists()]
    if missing:
        return fail("E_LANG_PRAGMATISM_PACK_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
        golden = load_jsonl(pack / "golden.jsonl")
    except ValueError as exc:
        return fail("E_LANG_PRAGMATISM_PACK_INVALID", str(exc))

    if not isinstance(contract, dict):
        return fail("E_LANG_PRAGMATISM_CONTRACT_TYPE", "contract root must be object")
    if str(contract.get("schema", "")).strip() != PACK_SCHEMA:
        return fail("E_LANG_PRAGMATISM_SCHEMA", f"schema={contract.get('schema')}")
    if str(contract.get("evidence_tier", "")).strip() != "golden_closed":
        return fail("E_LANG_PRAGMATISM_EVIDENCE_TIER", f"tier={contract.get('evidence_tier')}")
    if str(contract.get("closure_claim", "")).strip() != "yes":
        return fail("E_LANG_PRAGMATISM_CLOSURE_CLAIM", f"closure={contract.get('closure_claim')}")

    targets = contract.get("evidence_targets")
    if targets != list(REQUIRED_CASES.values()):
        return fail("E_LANG_PRAGMATISM_TARGETS", f"targets={targets}")
    for rel in REQUIRED_CASES.values():
        if not Path(rel).exists():
            return fail("E_LANG_PRAGMATISM_TARGET_MISSING", rel)

    cases = contract.get("cases")
    if not isinstance(cases, list):
        return fail("E_LANG_PRAGMATISM_CASES_TYPE", "cases must be list")
    case_ids = {str(row.get("id", "")).strip() for row in cases if isinstance(row, dict)}
    if case_ids != set(REQUIRED_CASES):
        return fail("E_LANG_PRAGMATISM_CASE_SET", f"cases={sorted(case_ids)}")

    golden_ids = {str(row.get("id", "")).strip() for row in golden}
    if golden_ids != set(REQUIRED_CASES):
        return fail("E_LANG_PRAGMATISM_GOLDEN_SET", f"golden={sorted(golden_ids)}")
    for row in golden:
        case_id = str(row.get("id", "")).strip()
        cmd = row.get("cmd")
        if not isinstance(cmd, list) or len(cmd) < 2 or cmd[0] != "run":
            return fail("E_LANG_PRAGMATISM_CMD", f"id={case_id}")
        if cmd[1] != REQUIRED_CASES.get(case_id):
            return fail("E_LANG_PRAGMATISM_CMD_TARGET", f"id={case_id} cmd={cmd}")
        if int(row.get("exit_code", -1)) != 0:
            return fail("E_LANG_PRAGMATISM_EXIT", f"id={case_id} exit={row.get('exit_code')}")
        stdout = row.get("stdout")
        if not isinstance(stdout, list) or not stdout:
            return fail("E_LANG_PRAGMATISM_STDOUT", f"id={case_id}")

    print("[lang-pragmatism-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(golden_ids)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
