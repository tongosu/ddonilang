#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


PACK_SCHEMA = "ddn.seulgi_gatekeeper.pack.contract.v1"
REQUIRED_CASES = {
    "g01_gatekeeper_reject_no_commit",
    "g02_trace_non_hash",
    "g03_latency_target_madi_fixed",
}


def fail(code: str, msg: str) -> int:
    print(f"[seulgi-gatekeeper-pack-check] fail code={code} msg={msg}", file=sys.stderr)
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
    parser = argparse.ArgumentParser(description="Seulgi gatekeeper pack checker")
    parser.add_argument("--pack", default="pack/seulgi_gatekeeper_v1")
    args = parser.parse_args()

    pack = Path(args.pack)
    required_files = [
        pack / "README.md",
        pack / "intent.md",
        pack / "contract.detjson",
        pack / "golden.jsonl",
        pack / "fixtures" / "reject_case.detjson",
        pack / "fixtures" / "trace_case_a.detjson",
        pack / "fixtures" / "trace_case_b.detjson",
        pack / "fixtures" / "latency_case.detjson",
    ]
    missing = [str(path).replace("\\", "/") for path in required_files if not path.exists()]
    if missing:
        return fail("E_SEULGI_GATEKEEPER_PACK_FILE_MISSING", ",".join(missing))

    try:
        contract = load_json(pack / "contract.detjson")
    except ValueError as exc:
        return fail("E_SEULGI_GATEKEEPER_CONTRACT_INVALID", str(exc))
    if not isinstance(contract, dict):
        return fail("E_SEULGI_GATEKEEPER_CONTRACT_TYPE", "contract root must be object")
    if str(contract.get("schema", "")).strip() != PACK_SCHEMA:
        return fail("E_SEULGI_GATEKEEPER_SCHEMA", f"schema={contract.get('schema')}")

    gatekeeper = contract.get("gatekeeper")
    if not isinstance(gatekeeper, dict):
        return fail("E_SEULGI_GATEKEEPER_BLOCK_TYPE", "gatekeeper must be object")
    if not bool(gatekeeper.get("required", False)):
        return fail("E_SEULGI_GATEKEEPER_REQUIRED", "gatekeeper.required must be true")
    if not bool(gatekeeper.get("reject_keeps_state_uncommitted", False)):
        return fail(
            "E_SEULGI_GATEKEEPER_REJECT_NO_COMMIT",
            "gatekeeper.reject_keeps_state_uncommitted must be true",
        )

    trace = contract.get("trace")
    if not isinstance(trace, dict):
        return fail("E_SEULGI_GATEKEEPER_TRACE_TYPE", "trace must be object")
    if not bool(trace.get("trace_fields_non_hash", False)):
        return fail("E_SEULGI_GATEKEEPER_TRACE_NON_HASH", "trace_fields_non_hash must be true")
    if not bool(trace.get("state_hash_stable_with_trace_delta", False)):
        return fail(
            "E_SEULGI_GATEKEEPER_TRACE_HASH_STABLE",
            "state_hash_stable_with_trace_delta must be true",
        )

    latency = contract.get("latency")
    if not isinstance(latency, dict):
        return fail("E_SEULGI_GATEKEEPER_LATENCY_TYPE", "latency must be object")
    if not bool(latency.get("target_madi_fixed", False)):
        return fail("E_SEULGI_GATEKEEPER_TARGET_FIXED", "target_madi_fixed must be true")

    cases = contract.get("cases")
    if not isinstance(cases, list):
        return fail("E_SEULGI_GATEKEEPER_CASES_TYPE", "cases must be list")
    case_ids = {str(row.get("id", "")).strip() for row in cases if isinstance(row, dict)}
    if case_ids != REQUIRED_CASES:
        return fail("E_SEULGI_GATEKEEPER_CASE_SET", f"cases={sorted(case_ids)}")

    try:
        golden = load_jsonl(pack / "golden.jsonl")
    except ValueError as exc:
        return fail("E_SEULGI_GATEKEEPER_GOLDEN_INVALID", str(exc))
    golden_ids = {str(row.get("id", "")).strip() for row in golden}
    if golden_ids != REQUIRED_CASES:
        return fail("E_SEULGI_GATEKEEPER_GOLDEN_SET", f"golden={sorted(golden_ids)}")
    for row in golden:
        if not bool(row.get("expect_ok", False)):
            return fail("E_SEULGI_GATEKEEPER_GOLDEN_EXPECT_OK", str(row.get("id")))
        if not isinstance(row.get("expected_commit"), bool):
            return fail("E_SEULGI_GATEKEEPER_GOLDEN_EXPECT_COMMIT", str(row.get("id")))

    try:
        reject_case = load_json(pack / "fixtures" / "reject_case.detjson")
        trace_a = load_json(pack / "fixtures" / "trace_case_a.detjson")
        trace_b = load_json(pack / "fixtures" / "trace_case_b.detjson")
        latency_case = load_json(pack / "fixtures" / "latency_case.detjson")
    except ValueError as exc:
        return fail("E_SEULGI_GATEKEEPER_FIXTURE_INVALID", str(exc))

    if not isinstance(reject_case, dict) or bool(reject_case.get("expected_commit", True)):
        return fail("E_SEULGI_GATEKEEPER_REJECT_FIXTURE", "reject fixture must keep expected_commit=false")

    if not isinstance(trace_a, dict) or not isinstance(trace_b, dict):
        return fail("E_SEULGI_GATEKEEPER_TRACE_FIXTURE_TYPE", "trace fixtures must be objects")
    if str(trace_a.get("state_hash", "")).strip() != str(trace_b.get("state_hash", "")).strip():
        return fail("E_SEULGI_GATEKEEPER_TRACE_STATE_HASH", "trace fixtures must keep same state_hash")
    if str(trace_a.get("trace_id", "")).strip() == str(trace_b.get("trace_id", "")).strip():
        return fail("E_SEULGI_GATEKEEPER_TRACE_DELTA", "trace fixtures must differ in trace_id")

    if not isinstance(latency_case, dict):
        return fail("E_SEULGI_GATEKEEPER_LATENCY_FIXTURE_TYPE", "latency fixture must be object")
    accepted = latency_case.get("accepted_madi")
    latency_madi = latency_case.get("latency_madi")
    target = latency_case.get("target_madi")
    if not all(isinstance(v, int) for v in (accepted, latency_madi, target)):
        return fail("E_SEULGI_GATEKEEPER_LATENCY_FIXTURE_VALUE", "latency fixture madi values must be int")
    if target != accepted + latency_madi:
        return fail(
            "E_SEULGI_GATEKEEPER_LATENCY_FORMULA",
            f"accepted={accepted} latency={latency_madi} target={target}",
        )
    if not bool(latency_case.get("target_madi_fixed", False)):
        return fail("E_SEULGI_GATEKEEPER_LATENCY_FIXED", "latency fixture target_madi_fixed must be true")

    print("[seulgi-gatekeeper-pack-check] ok")
    print(f"pack={str(pack).replace(chr(92), '/')}")
    print(f"cases={len(case_ids)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
