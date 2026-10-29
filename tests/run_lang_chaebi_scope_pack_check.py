#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


PACK_SCHEMA = "ddn.lang_chaebi_scope.pack.contract.v1"
REQUIRED_CASES = {
    "c01_top_level_chaebi_ok": ("ok", "", ""),
    "c02_chaebi_in_repeat_forbidden": ("diag", "E_CHAEBI_IN_LOOP", ""),
    "c03_chaebi_in_hook_forbidden": ("diag", "E_CHAEBI_IN_LOOP", ""),
    "c04_chaebi_in_iter_forbidden": ("diag", "E_CHAEBI_IN_LOOP", ""),
    "c05_top_level_reassign_warning": ("warning", "", "W_CHAEBI_REDUNDANT_TOP_REASSIGN"),
}


def fail(code: str, msg: str) -> int:
    print(f"[lang-chaebi-scope-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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
    parser = argparse.ArgumentParser(description="Lang chaebi scope pack checker")
    parser.add_argument("--pack", default="pack/lang_chaebi_scope_v1")
    args = parser.parse_args()

    pack = Path(args.pack)
    required_files = [
        pack / "README.md",
        pack / "intent.md",
        pack / "contract.detjson",
        pack / "golden.jsonl",
        pack / "c01_top_level_chaebi_ok" / "input.ddn",
        pack / "c02_chaebi_in_repeat_forbidden" / "input.ddn",
        pack / "c03_chaebi_in_hook_forbidden" / "input.ddn",
        pack / "c04_chaebi_in_iter_forbidden" / "input.ddn",
        pack / "c05_top_level_reassign_warning" / "input.ddn",
        pack / "c05_top_level_reassign_warning" / "expected_canon.ddn",
    ]
    missing = [str(path).replace("\\", "/") for path in required_files if not path.exists()]
    if missing:
        return fail("E_LANG_CHAEBI_SCOPE_PACK_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
        golden = load_jsonl(pack / "golden.jsonl")
    except ValueError as exc:
        return fail("E_LANG_CHAEBI_SCOPE_INVALID", str(exc))

    if not isinstance(contract, dict):
        return fail("E_LANG_CHAEBI_SCOPE_CONTRACT_TYPE", "contract root must be object")
    if str(contract.get("schema", "")).strip() != PACK_SCHEMA:
        return fail("E_LANG_CHAEBI_SCOPE_SCHEMA", f"schema={contract.get('schema')}")
    if str(contract.get("evidence_tier", "")).strip() != "runner_fill":
        return fail("E_LANG_CHAEBI_SCOPE_EVIDENCE_TIER", f"tier={contract.get('evidence_tier')}")
    if str(contract.get("closure_claim", "")).strip() != "no":
        return fail("E_LANG_CHAEBI_SCOPE_CLOSURE_CLAIM", f"closure={contract.get('closure_claim')}")

    cases = contract.get("cases")
    if not isinstance(cases, list):
        return fail("E_LANG_CHAEBI_SCOPE_CASES_TYPE", "cases must be list")
    case_index = {
        str(row.get("id", "")).strip(): row for row in cases if isinstance(row, dict)
    }
    if set(case_index) != set(REQUIRED_CASES):
        return fail("E_LANG_CHAEBI_SCOPE_CASE_SET", f"cases={sorted(case_index)}")

    golden_index = {
        str(row.get("id", "")).strip(): row for row in golden if isinstance(row, dict)
    }
    if set(golden_index) != set(REQUIRED_CASES):
        return fail("E_LANG_CHAEBI_SCOPE_GOLDEN_SET", f"golden={sorted(golden_index)}")

    for case_id, (case_kind, expected_error, expected_warning) in REQUIRED_CASES.items():
        row = case_index[case_id]
        if str(row.get("case_kind", "")).strip() != case_kind:
            return fail("E_LANG_CHAEBI_SCOPE_CASE_KIND", f"id={case_id} kind={row.get('case_kind')}")
        if expected_error and str(row.get("expected_error", "")).strip() != expected_error:
            return fail("E_LANG_CHAEBI_SCOPE_CASE_ERROR", f"id={case_id}")
        if expected_warning and str(row.get("expected_warning", "")).strip() != expected_warning:
            return fail("E_LANG_CHAEBI_SCOPE_CASE_WARNING", f"id={case_id}")

        golden_row = golden_index[case_id]
        if expected_error and str(golden_row.get("expected_error_code", "")).strip() != expected_error:
            return fail("E_LANG_CHAEBI_SCOPE_GOLDEN_ERROR", f"id={case_id}")
        if expected_warning and str(golden_row.get("expected_warning_code", "")).strip() != expected_warning:
            return fail("E_LANG_CHAEBI_SCOPE_GOLDEN_WARNING", f"id={case_id}")

    print("[lang-chaebi-scope-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(golden_index)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
