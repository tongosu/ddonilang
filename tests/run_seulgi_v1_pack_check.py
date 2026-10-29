#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


REQUIRED_CASES = {
    "s01_replay_source_recall_forbidden",
    "s02_gatekeeper_reject_no_commit",
    "s03_target_madi_fixed_apply_later",
    "s04_trace_non_hash",
    "s05_world_mutation_only_via_injection",
}


def fail(code: str, msg: str) -> int:
    print(f"[seulgi-v1-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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
        text = raw.strip()
        if not text:
            continue
        try:
            row = json.loads(text)
        except Exception as exc:
            raise ValueError(f"invalid jsonl row: {path}:{line_no} ({exc})")
        if not isinstance(row, dict):
            raise ValueError(f"jsonl row must be object: {path}:{line_no}")
        rows.append(row)
    return rows


def main() -> int:
    parser = argparse.ArgumentParser(description="Seulgi v1 pack contract checker")
    parser.add_argument(
        "--pack",
        default="pack/seulgi_v1",
        help="pack directory path",
    )
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
        return fail("E_SEULGI_V1_FILE_MISSING", ",".join(missing))

    try:
        contract_doc = load_json(pack / "contract.detjson")
    except ValueError as exc:
        return fail("E_SEULGI_V1_CONTRACT_INVALID", str(exc))
    if not isinstance(contract_doc, dict):
        return fail("E_SEULGI_V1_CONTRACT_TYPE", "contract root must be object")
    if str(contract_doc.get("schema", "")).strip() != "ddn.seulgi_v1.pack.contract.v1":
        return fail("E_SEULGI_V1_SCHEMA", f"schema={contract_doc.get('schema')}")
    if str(contract_doc.get("role", "")).strip() != "async_advisor":
        return fail("E_SEULGI_V1_ROLE", f"role={contract_doc.get('role')}")
    if not bool(contract_doc.get("direct_world_mutation_forbidden", False)):
        return fail("E_SEULGI_V1_WORLD_MUTATION", "direct_world_mutation_forbidden must be true")
    if str(contract_doc.get("injection_boundary", "")).strip() != "sam_input_snapshot":
        return fail("E_SEULGI_V1_BOUNDARY", f"injection_boundary={contract_doc.get('injection_boundary')}")

    replay_doc = contract_doc.get("replay")
    if not isinstance(replay_doc, dict):
        return fail("E_SEULGI_V1_REPLAY_TYPE", "replay must be object")
    if not bool(replay_doc.get("source_recall_forbidden", False)):
        return fail("E_SEULGI_V1_REPLAY_SOURCE_RECALL", "source_recall_forbidden must be true")
    if not bool(replay_doc.get("reinject_recorded_injections_only", False)):
        return fail("E_SEULGI_V1_REPLAY_REINJECT", "reinject_recorded_injections_only must be true")

    gatekeeper_doc = contract_doc.get("gatekeeper")
    if not isinstance(gatekeeper_doc, dict):
        return fail("E_SEULGI_V1_GATEKEEPER_TYPE", "gatekeeper must be object")
    if not bool(gatekeeper_doc.get("required", False)):
        return fail("E_SEULGI_V1_GATEKEEPER_REQUIRED", "gatekeeper.required must be true")
    if not bool(gatekeeper_doc.get("reject_keeps_state_uncommitted", False)):
        return fail(
            "E_SEULGI_V1_GATEKEEPER_REJECT",
            "gatekeeper.reject_keeps_state_uncommitted must be true",
        )

    cases = contract_doc.get("cases")
    if not isinstance(cases, list):
        return fail("E_SEULGI_V1_CASES_TYPE", "cases must be list")
    case_map: dict[str, dict] = {}
    for row in cases:
        if not isinstance(row, dict):
            return fail("E_SEULGI_V1_CASE_ROW_TYPE", f"type={type(row).__name__}")
        case_id = str(row.get("id", "")).strip()
        if not case_id:
            return fail("E_SEULGI_V1_CASE_ID_MISSING", "id missing")
        case_map[case_id] = row
    if sorted(case_map.keys()) != sorted(REQUIRED_CASES):
        return fail("E_SEULGI_V1_CASE_SET", f"case_ids={sorted(case_map.keys())}")

    target_case = case_map["s03_target_madi_fixed_apply_later"]
    accepted_madi = target_case.get("accepted_madi")
    target_madi = target_case.get("target_madi")
    if not isinstance(accepted_madi, int) or not isinstance(target_madi, int):
        return fail("E_SEULGI_V1_TARGET_MADI_TYPE", "accepted_madi/target_madi must be int")
    if target_madi <= accepted_madi:
        return fail("E_SEULGI_V1_TARGET_MADI_ORDER", f"accepted={accepted_madi} target={target_madi}")
    if not bool(target_case.get("target_madi_fixed", False)):
        return fail("E_SEULGI_V1_TARGET_MADI_FIXED", "target_madi_fixed must be true")

    trace_case = case_map["s04_trace_non_hash"]
    if not bool(trace_case.get("trace_fields_non_hash", False)):
        return fail("E_SEULGI_V1_TRACE_NON_HASH", "trace_fields_non_hash must be true")
    if not bool(trace_case.get("state_hash_stable_with_trace_delta", False)):
        return fail(
            "E_SEULGI_V1_TRACE_HASH_STABLE",
            "state_hash_stable_with_trace_delta must be true",
        )

    reject_case = case_map["s02_gatekeeper_reject_no_commit"]
    if bool(reject_case.get("expected_commit", True)):
        return fail("E_SEULGI_V1_REJECT_EXPECT_COMMIT", "gatekeeper reject case must not commit")

    replay_case = case_map["s01_replay_source_recall_forbidden"]
    if not bool(replay_case.get("replay_no_source_recall", False)):
        return fail("E_SEULGI_V1_REPLAY_CASE", "replay_no_source_recall must be true")

    try:
        golden_rows = load_jsonl(pack / "golden.jsonl")
    except ValueError as exc:
        return fail("E_SEULGI_V1_GOLDEN_INVALID", str(exc))
    if not golden_rows:
        return fail("E_SEULGI_V1_GOLDEN_EMPTY", "golden.jsonl must be non-empty")

    golden_ids: list[str] = []
    for row in golden_rows:
        case_id = str(row.get("id", "")).strip()
        if not case_id:
            return fail("E_SEULGI_V1_GOLDEN_ID_MISSING", "id missing")
        if case_id not in REQUIRED_CASES:
            return fail("E_SEULGI_V1_GOLDEN_UNKNOWN_CASE", case_id)
        if not bool(row.get("expect_ok", False)):
            return fail("E_SEULGI_V1_GOLDEN_EXPECT_OK", case_id)
        expected_commit = row.get("expected_commit")
        if not isinstance(expected_commit, bool):
            return fail("E_SEULGI_V1_GOLDEN_EXPECTED_COMMIT", case_id)
        golden_ids.append(case_id)
    if sorted(golden_ids) != sorted(REQUIRED_CASES):
        return fail("E_SEULGI_V1_GOLDEN_CASE_SET", f"golden_ids={sorted(golden_ids)}")

    print("[seulgi-v1-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(case_map)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
