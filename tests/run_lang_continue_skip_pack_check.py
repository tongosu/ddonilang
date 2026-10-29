#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


PACK_SCHEMA = "ddn.lang_continue_skip.pack.contract.v1"
REQUIRED_CASES = {
    "c01_foreach_skip_ok": ("ok", "", ["4"]),
    "c02_top_level_forbidden": ("diag", "E_RUNTIME_CONTINUE_OUTSIDE_FOREACH", []),
    "c03_hook_body_forbidden": ("diag", "E_RUNTIME_CONTINUE_OUTSIDE_FOREACH", []),
    "c04_repeat_body_forbidden": ("diag", "E_RUNTIME_CONTINUE_OUTSIDE_FOREACH", []),
}


def fail(code: str, msg: str) -> int:
    print(f"[lang-continue-skip-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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
    parser = argparse.ArgumentParser(description="Lang continue-skip pack checker")
    parser.add_argument("--pack", default="pack/lang_continue_skip_v1")
    args = parser.parse_args()

    pack = Path(args.pack)
    required = [pack / "README.md", pack / "intent.md", pack / "contract.detjson", pack / "golden.jsonl"]
    required.extend((pack / case_id / "input.ddn") for case_id in REQUIRED_CASES)
    missing = [str(path).replace("\\", "/") for path in required if not path.exists()]
    if missing:
        return fail("E_LANG_CONTINUE_SKIP_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
        golden = load_jsonl(pack / "golden.jsonl")
    except ValueError as exc:
        return fail("E_LANG_CONTINUE_SKIP_INVALID", str(exc))

    if not isinstance(contract, dict):
        return fail("E_LANG_CONTINUE_SKIP_CONTRACT_TYPE", "contract root must be object")
    if str(contract.get("schema", "")).strip() != PACK_SCHEMA:
        return fail("E_LANG_CONTINUE_SKIP_SCHEMA", f"schema={contract.get('schema')}")
    if str(contract.get("evidence_tier", "")).strip() != "runner_fill":
        return fail("E_LANG_CONTINUE_SKIP_TIER", f"tier={contract.get('evidence_tier')}")
    if str(contract.get("closure_claim", "")).strip() != "no":
        return fail("E_LANG_CONTINUE_SKIP_CLOSURE", f"closure={contract.get('closure_claim')}")

    cases = contract.get("cases")
    if not isinstance(cases, list):
        return fail("E_LANG_CONTINUE_SKIP_CASES_TYPE", "cases must be list")
    case_index = {str(row.get("id", "")).strip(): row for row in cases if isinstance(row, dict)}
    if set(case_index) != set(REQUIRED_CASES):
        return fail("E_LANG_CONTINUE_SKIP_CASE_SET", f"cases={sorted(case_index)}")

    golden_index = {str(row.get("id", "")).strip(): row for row in golden}
    if set(golden_index) != set(REQUIRED_CASES):
        return fail("E_LANG_CONTINUE_SKIP_GOLDEN_SET", f"golden={sorted(golden_index)}")

    for case_id, (case_kind, expected_error, expected_stdout) in REQUIRED_CASES.items():
        if str(case_index[case_id].get("case_kind", "")).strip() != case_kind:
            return fail("E_LANG_CONTINUE_SKIP_CASE_KIND", f"id={case_id}")
        row = golden_index[case_id]
        cmd = row.get("cmd")
        if not isinstance(cmd, list) or len(cmd) < 2 or cmd[0] != "run":
            return fail("E_LANG_CONTINUE_SKIP_CMD", f"id={case_id}")
        if cmd[1] != f"pack/lang_continue_skip_v1/{case_id}/input.ddn":
            return fail("E_LANG_CONTINUE_SKIP_CMD_TARGET", f"id={case_id} cmd={cmd}")
        if expected_error:
            if str(row.get("expected_error_code", "")).strip() != expected_error:
                return fail("E_LANG_CONTINUE_SKIP_ERROR", f"id={case_id}")
            if int(row.get("exit_code", 0)) != 1:
                return fail("E_LANG_CONTINUE_SKIP_EXIT", f"id={case_id} exit={row.get('exit_code')}")
            if "stdout" in row:
                return fail("E_LANG_CONTINUE_SKIP_ERROR_STDOUT", f"id={case_id}")
        elif row.get("stdout") != expected_stdout:
            return fail("E_LANG_CONTINUE_SKIP_STDOUT", f"id={case_id} stdout={row.get('stdout')}")
        elif int(row.get("exit_code", -1)) != 0:
            return fail("E_LANG_CONTINUE_SKIP_EXIT", f"id={case_id} exit={row.get('exit_code')}")

    print("[lang-continue-skip-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(golden_index)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
