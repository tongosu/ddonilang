#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SINGLE = "connect_endpoint_relation_flatten_single_v1"
RELATION_SET = "connect_endpoint_relation_flatten_relation_set_v1"
STATEMENT_SET = "connect_endpoint_relation_flatten_statement_set_v1"
NORMALIZE = "connect_endpoint_relation_normalize_v1"
CLOSURE = "connect_flow_v1h_closure_v1"
PACKS = [SINGLE, RELATION_SET, STATEMENT_SET, NORMALIZE, CLOSURE]
BUNDLED = [
    "connect_flow_v1g_closure_v1",
    "connect_endpoint_relation_flatten_single_v1",
    "connect_endpoint_relation_flatten_relation_set_v1",
    "connect_endpoint_relation_flatten_statement_set_v1",
    "connect_endpoint_relation_normalize_v1",
]


def fail(message: str) -> None:
    print(f"[connect-endpoint-relation-normalize-v1h] FAIL: {message}", file=sys.stderr)
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


def require_contract(pack: str, count: int, kinds: list[str]) -> dict:
    contract = read_json(ROOT / "pack" / pack / "contract.detjson")
    if contract.get("schema") != "ddn.connect_endpoint_relation_normalize.pack.contract.v1":
        fail(f"{pack} schema mismatch")
    if contract.get("flat_relation_count") != count:
        fail(f"{pack} flat relation count mismatch")
    if contract.get("flat_relation_kinds") != kinds:
        fail(f"{pack} flat relation kinds mismatch")
    if contract.get("normalized_kind") not in (None, "endpoint_relation_flat_set"):
        fail(f"{pack} normalized kind mismatch")
    if contract.get("order") != "source_order_preserved":
        fail(f"{pack} must preserve source order")
    if contract.get("dedupe_or_merge") is not False:
        fail(f"{pack} must not dedupe or merge")
    if contract.get("solver_claim") is not False:
        fail(f"{pack} must not claim solver integration")
    return contract


def main() -> int:
    for path in [
        ROOT / "CONNECT_ENDPOINT_STATEMENT_APPEND_V1G.md",
        ROOT / "pack" / "connect_flow_v1g_closure_v1" / "contract.detjson",
        ROOT / "pack" / "connect_endpoint_statement_append_same_pair_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_endpoint_statement_append_mixed_pair_v1" / "golden.jsonl",
        ROOT / "CONNECT_ENDPOINT_RELATION_NORMALIZE_V1H.md",
    ]:
        require_file(path)

    for pack in PACKS:
        pack_dir = ROOT / "pack" / pack
        if not pack_dir.is_dir():
            fail(f"missing pack directory: pack/{pack}")
        for name in ["README.md", "input.ddn", "contract.detjson", "golden.jsonl"]:
            require_file(pack_dir / name)

    require_contract(SINGLE, 1, ["endpoint_equality"])
    require_stdout(
        SINGLE,
        [
            "1",
            "endpoint_equality",
            "전압",
            "endpoint_relation_flat_set",
            "1",
            "endpoint_equality",
        ],
    )

    relation_contract = require_contract(
        RELATION_SET,
        3,
        ["endpoint_equality", "endpoint_flow", "endpoint_flow"],
    )
    if relation_contract.get("flat_relation_channels") != ["체결값", "재화", "돈"]:
        fail("relation-set channels must preserve source order")
    require_stdout(
        RELATION_SET,
        [
            "3",
            "endpoint_equality",
            "체결값",
            "endpoint_flow",
            "재화",
            "endpoint_flow",
            "돈",
            "endpoint_relation_flat_set",
            "3",
        ],
    )

    statement_contract = require_contract(
        STATEMENT_SET,
        3,
        ["endpoint_equality", "endpoint_flow", "endpoint_carried_property"],
    )
    if statement_contract.get("nested_relation_set_flattened") is not True:
        fail("statement-set pack must flatten nested relation_set")
    require_stdout(
        STATEMENT_SET,
        [
            "3",
            "endpoint_equality",
            "전압",
            "endpoint_flow",
            "대출금",
            "endpoint_carried_property",
            "위험",
            "대출금",
            "endpoint_relation_flat_set",
            "3",
        ],
    )

    normalize_contract = read_json(ROOT / "pack" / NORMALIZE / "contract.detjson")
    if normalize_contract.get("output_kind") != "endpoint_relation_flat_set":
        fail("normalize pack output kind mismatch")
    if normalize_contract.get("flat_relation_count") != 2:
        fail("normalize pack relation count mismatch")
    if normalize_contract.get("solver_claim") is not False:
        fail("normalize pack must not claim solver integration")
    require_stdout(
        NORMALIZE,
        [
            "endpoint_relation_flat_set",
            "2",
            "endpoint_flow",
            "대출금",
            "endpoint_carried_property",
            "위험",
            "왼쪽에서오른쪽",
        ],
    )

    closure_contract = read_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure_contract.get("bundled_packs") != BUNDLED:
        fail(f"closure bundled_packs mismatch: {closure_contract.get('bundled_packs')}")
    if closure_contract.get("normalized_kind") != "endpoint_relation_flat_set":
        fail("closure must record endpoint_relation_flat_set")
    if closure_contract.get("source_order") != "preserved":
        fail("closure must preserve source order")
    if closure_contract.get("dedupe_or_merge") is not False:
        fail("closure must not dedupe or merge")
    if closure_contract.get("solver_claim") is not False:
        fail("closure must not claim solver integration")
    require_stdout(CLOSURE, ["connect_flow_v1h_closure_v1", *BUNDLED])

    root_doc = (ROOT / "CONNECT_ENDPOINT_RELATION_NORMALIZE_V1H.md").read_text(encoding="utf-8")
    for marker in [
        "이음관계.관계목록",
        "이음관계.정규화",
        "endpoint_relation_flat_set",
        "connect_flow_v1h_closure_v1",
        "solver",
        "docs/ssot/**",
    ]:
        if marker not in root_doc:
            fail(f"root doc missing marker: {marker}")

    run([sys.executable, "tests/run_pack_golden.py", *PACKS])
    print("[connect-endpoint-relation-normalize-v1h] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
