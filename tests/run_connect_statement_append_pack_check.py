#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SAME = "connect_endpoint_statement_append_same_pair_v1"
MIXED = "connect_endpoint_statement_append_mixed_pair_v1"
BOUNDARY = "connect_endpoint_statement_append_boundary_v1"
CLOSURE = "connect_flow_v1g_closure_v1"
PACKS = [SAME, MIXED, BOUNDARY, CLOSURE]
BUNDLED = [
    "connect_flow_v1f_closure_v1",
    "connect_endpoint_statement_append_same_pair_v1",
    "connect_endpoint_statement_append_mixed_pair_v1",
    "connect_endpoint_statement_append_boundary_v1",
]


def fail(message: str) -> None:
    print(f"[connect-statement-append-v1g] FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def require_file(path: Path) -> None:
    if not path.is_file():
        fail(f"missing file: {path.relative_to(ROOT)}")


def read_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"{path.relative_to(ROOT)} invalid JSON: {exc}")


def read_jsonl(path: Path) -> list[dict]:
    rows = []
    for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.strip():
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError as exc:
            fail(f"{path.relative_to(ROOT)}:{lineno}: invalid JSONL: {exc}")
    return rows


def require_stdout(pack: str, expected: list[object]) -> None:
    rows = read_jsonl(ROOT / "pack" / pack / "golden.jsonl")
    if len(rows) != 1:
        fail(f"pack/{pack}/golden.jsonl must have exactly one row")
    if rows[0].get("exit_code") != 0:
        fail(f"pack/{pack} must be a passing golden row")
    if rows[0].get("stdout") != expected:
        fail(f"pack/{pack} stdout mismatch: {rows[0].get('stdout')}")


def run(args: list[str]) -> None:
    proc = subprocess.run(args, cwd=ROOT, text=True, encoding="utf-8", errors="replace")
    if proc.returncode != 0:
        fail(f"command failed: {' '.join(args)}")


def require_statement_contract(pack: str, mixed: bool) -> dict:
    contract = read_json(ROOT / "pack" / pack / "contract.detjson")
    artifact = contract.get("artifact", {})
    if artifact.get("__이음관계종류") != "endpoint_statement_set":
        fail(f"{pack} must use endpoint_statement_set")
    if artifact.get("대상") != "이음관계":
        fail(f"{pack} target mismatch")
    if artifact.get("개수") != 2:
        fail(f"{pack} statement count mismatch")
    items = artifact.get("items")
    if not isinstance(items, list) or len(items) != 2:
        fail(f"{pack} must define two statement items")
    if contract.get("statement_order") != "source_order_preserved":
        fail(f"{pack} must preserve source order")
    if contract.get("dedupe_or_merge") is not False:
        fail(f"{pack} must not dedupe or merge")
    if contract.get("single_statement_shape_changed") is not False:
        fail(f"{pack} must keep single statement shape")
    if contract.get("solver_claim") is not False:
        fail(f"{pack} must not claim solver integration")
    expected_policy = "mixed_pair_allowed" if mixed else "same_pair"
    if artifact.get("endpoint_pair_policy") != expected_policy:
        fail(f"{pack} endpoint pair policy mismatch")
    return contract


def main() -> int:
    for path in [
        ROOT / "CONNECT_CARRIED_PROPERTY_V1F.md",
        ROOT / "pack" / "connect_flow_v1f_closure_v1" / "contract.detjson",
        ROOT / "pack" / "connect_endpoint_carried_property_forward_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_endpoint_carried_property_reverse_v1" / "golden.jsonl",
        ROOT / "CONNECT_ENDPOINT_STATEMENT_APPEND_V1G.md",
    ]:
        require_file(path)

    for pack in PACKS:
        pack_dir = ROOT / "pack" / pack
        if not pack_dir.is_dir():
            fail(f"missing pack directory: pack/{pack}")
        for name in ["README.md", "input.ddn", "contract.detjson", "golden.jsonl"]:
            require_file(pack_dir / name)

    same_contract = require_statement_contract(SAME, mixed=False)
    same_items = same_contract["artifact"]["items"]
    if [item.get("__이음관계종류") for item in same_items] != [
        "endpoint_equality",
        "endpoint_flow",
    ]:
        fail("same-pair item kind order mismatch")
    require_stdout(
        SAME,
        [
            "endpoint_statement_set",
            "이음관계",
            "2",
            "endpoint_equality",
            "전지.양극.전압",
            "전구.왼핀.전압",
            "같게",
            "전압",
            "endpoint_flow",
            "전지.양극.전류",
            "전구.왼핀.전류",
            "흐르게",
            "전류",
            "왼쪽에서오른쪽",
        ],
    )

    mixed_contract = require_statement_contract(MIXED, mixed=True)
    mixed_items = mixed_contract["artifact"]["items"]
    if mixed_items[1].get("__이음관계종류") != "endpoint_relation_set":
        fail("mixed-pair second item must keep endpoint_relation_set shape")
    relations = mixed_items[1].get("relations")
    if not isinstance(relations, list) or len(relations) != 2:
        fail("mixed-pair relation set must keep two inner relations")
    if relations[1].get("__이음관계종류") != "endpoint_carried_property":
        fail("mixed-pair carried property relation missing")
    require_stdout(
        MIXED,
        [
            "endpoint_statement_set",
            "이음관계",
            "2",
            "endpoint_equality",
            "전지.양극.전압",
            "전구.왼핀.전압",
            "endpoint_relation_set",
            "은행.대출창구",
            "기업1.차입끝",
            "endpoint_flow",
            "대출금",
            "endpoint_carried_property",
            "위험",
            "대출금",
            "왼쪽에서오른쪽",
        ],
    )

    boundary_contract = read_json(ROOT / "pack" / BOUNDARY / "contract.detjson")
    policy = boundary_contract.get("boundary_policy", {})
    for key in [
        "non_endpoint_statement_breaks_block",
        "different_target_endpoint_statement_breaks_block",
    ]:
        if policy.get(key) is not True:
            fail(f"boundary policy must set {key}=true")
    if policy.get("non_consecutive_same_target_collection") is not False:
        fail("non-consecutive same target collection must remain false")
    if policy.get("reassignment_error_claim") is not False:
        fail("boundary pack must not claim reassignment error")
    require_stdout(
        BOUNDARY,
        ["endpoint_flow", "전류", "1", "endpoint_flow", "전류", "endpoint_flow"],
    )

    closure_contract = read_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure_contract.get("bundled_packs") != BUNDLED:
        fail(f"closure bundled_packs mismatch: {closure_contract.get('bundled_packs')}")
    if closure_contract.get("statement_set_kind") != "endpoint_statement_set":
        fail("closure must record endpoint_statement_set")
    if closure_contract.get("same_target_consecutive_only") is not True:
        fail("closure must be same-target consecutive only")
    if closure_contract.get("mixed_endpoint_pair_allowed") is not True:
        fail("closure must allow mixed endpoint pairs")
    if closure_contract.get("dedupe_or_merge") is not False:
        fail("closure must not dedupe or merge")
    if closure_contract.get("solver_claim") is not False:
        fail("closure must not claim solver integration")
    require_stdout(CLOSURE, ["connect_flow_v1g_closure_v1", *BUNDLED])

    root_doc = (ROOT / "CONNECT_ENDPOINT_STATEMENT_APPEND_V1G.md").read_text(encoding="utf-8")
    for marker in [
        "endpoint_statement_set",
        "block",
        "mixed_pair",
        "non-endpoint",
        "connect_flow_v1g_closure_v1",
        "docs/ssot/**",
    ]:
        if marker not in root_doc:
            fail(f"root doc missing marker: {marker}")

    run([sys.executable, "tests/run_pack_golden.py", *PACKS])
    print("[connect-statement-append-v1g] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
