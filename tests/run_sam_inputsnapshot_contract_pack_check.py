#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


PACK_SCHEMA = "ddn.sam_inputsnapshot_contract.pack.contract.v1"
ORDER_KEYS = ["agent_id", "recv_seq"]
FORBIDDEN_ORDERING_KEYS = ["intent_kind", "payload_hash", "accepted_madi", "target_madi"]
REQUIRED_CASES = {
    "c01_replay_reinject_only",
    "c02_duplicate_recv_seq_reject",
    "c03_reverse_recv_seq_reject",
}


def fail(code: str, msg: str) -> int:
    print(f"[sam-inputsnapshot-contract-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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


def recv_seq_violation(records: list[dict]) -> str:
    last_by_agent: dict[int, int] = {}
    for row in records:
        agent_id = int(row.get("agent_id", -1))
        recv_seq = int(row.get("recv_seq", -1))
        prev = last_by_agent.get(agent_id)
        if prev is not None:
            if recv_seq == prev:
                return "E_SAM_INPUTSNAPSHOT_DUPLICATE_RECV_SEQ"
            if recv_seq < prev:
                return "E_SAM_INPUTSNAPSHOT_REVERSE_RECV_SEQ"
        last_by_agent[agent_id] = recv_seq
    return ""


def main() -> int:
    parser = argparse.ArgumentParser(description="Sam InputSnapshot contract pack checker")
    parser.add_argument("--pack", default="pack/sam_inputsnapshot_contract_v1")
    args = parser.parse_args()

    pack = Path(args.pack)
    required_files = [
        pack / "README.md",
        pack / "intent.md",
        pack / "contract.detjson",
        pack / "golden.jsonl",
        pack / "fixtures" / "inputsnapshot_ok.detjsonl",
        pack / "fixtures" / "inputsnapshot_duplicate.detjsonl",
        pack / "fixtures" / "inputsnapshot_reverse.detjsonl",
    ]
    missing = [str(path).replace("\\", "/") for path in required_files if not path.exists()]
    if missing:
        return fail("E_SAM_INPUTSNAPSHOT_PACK_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
    except ValueError as exc:
        return fail("E_SAM_INPUTSNAPSHOT_CONTRACT_INVALID", str(exc))
    if not isinstance(contract, dict):
        return fail("E_SAM_INPUTSNAPSHOT_CONTRACT_TYPE", "contract root must be object")
    if str(contract.get("schema", "")).strip() != PACK_SCHEMA:
        return fail("E_SAM_INPUTSNAPSHOT_SCHEMA", f"schema={contract.get('schema')}")

    ordering = contract.get("execution_ordering")
    if not isinstance(ordering, dict):
        return fail("E_SAM_INPUTSNAPSHOT_ORDERING_TYPE", "execution_ordering must be object")
    if ordering.get("keys") != ORDER_KEYS:
        return fail("E_SAM_INPUTSNAPSHOT_ORDER_KEYS", f"keys={ordering.get('keys')}")
    if ordering.get("forbidden_ordering_keys") != FORBIDDEN_ORDERING_KEYS:
        return fail(
            "E_SAM_INPUTSNAPSHOT_FORBIDDEN_KEYS",
            f"forbidden={ordering.get('forbidden_ordering_keys')}",
        )
    for key in ("strict_monotonic_recv_seq_per_agent", "duplicate_recv_seq_reject", "reverse_recv_seq_reject"):
        if not bool(ordering.get(key, False)):
            return fail("E_SAM_INPUTSNAPSHOT_ORDERING_FLAG", key)

    replay = contract.get("replay")
    if not isinstance(replay, dict):
        return fail("E_SAM_INPUTSNAPSHOT_REPLAY_TYPE", "replay must be object")
    if not bool(replay.get("recorded_injections_only", False)):
        return fail("E_SAM_INPUTSNAPSHOT_REPLAY_ONLY", "recorded_injections_only must be true")
    if not bool(replay.get("source_recall_forbidden", False)):
        return fail("E_SAM_INPUTSNAPSHOT_SOURCE_RECALL", "source_recall_forbidden must be true")

    cases = contract.get("cases")
    if not isinstance(cases, list):
        return fail("E_SAM_INPUTSNAPSHOT_CASES_TYPE", "cases must be list")
    case_ids = {str(row.get("id", "")).strip() for row in cases if isinstance(row, dict)}
    if case_ids != REQUIRED_CASES:
        return fail("E_SAM_INPUTSNAPSHOT_CASE_SET", f"cases={sorted(case_ids)}")

    try:
        golden = load_jsonl(pack / "golden.jsonl")
    except ValueError as exc:
        return fail("E_SAM_INPUTSNAPSHOT_GOLDEN_INVALID", str(exc))
    golden_ids = {str(row.get("id", "")).strip() for row in golden}
    if golden_ids != REQUIRED_CASES:
        return fail("E_SAM_INPUTSNAPSHOT_GOLDEN_SET", f"golden={sorted(golden_ids)}")
    for row in golden:
        if not bool(row.get("expect_ok", False)):
            return fail("E_SAM_INPUTSNAPSHOT_GOLDEN_EXPECT_OK", str(row.get("id")))

    try:
        ok_rows = load_jsonl(pack / "fixtures" / "inputsnapshot_ok.detjsonl")
        dup_rows = load_jsonl(pack / "fixtures" / "inputsnapshot_duplicate.detjsonl")
        rev_rows = load_jsonl(pack / "fixtures" / "inputsnapshot_reverse.detjsonl")
    except ValueError as exc:
        return fail("E_SAM_INPUTSNAPSHOT_FIXTURE_INVALID", str(exc))

    if recv_seq_violation(ok_rows):
        return fail("E_SAM_INPUTSNAPSHOT_FIXTURE_OK_INVALID", "ok fixture must pass monotonic check")
    if recv_seq_violation(dup_rows) != "E_SAM_INPUTSNAPSHOT_DUPLICATE_RECV_SEQ":
        return fail("E_SAM_INPUTSNAPSHOT_FIXTURE_DUPLICATE", "duplicate fixture must emit duplicate code")
    if recv_seq_violation(rev_rows) != "E_SAM_INPUTSNAPSHOT_REVERSE_RECV_SEQ":
        return fail("E_SAM_INPUTSNAPSHOT_FIXTURE_REVERSE", "reverse fixture must emit reverse code")

    print("[sam-inputsnapshot-contract-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(case_ids)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
