#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

MULTI = "connect_endpoint_multi_inner_v1"
ECON = "connect_endpoint_multi_inner_econ_v1"
CLOSURE = "connect_flow_v1e_closure_v1"
PACKS = [MULTI, ECON, CLOSURE]
BUNDLED = [
    "connect_flow_v1d_closure_v1",
    "connect_endpoint_multi_inner_v1",
    "connect_endpoint_multi_inner_econ_v1",
]


def fail(message: str) -> None:
    print(f"[connect-multi-inner-v1e] FAIL: {message}", file=sys.stderr)
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


def require_stdout(pack: str, expected: list[str]) -> None:
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


def main() -> int:
    for path in [
        ROOT / "CONNECT_REVERSE_FLOW_V1D.md",
        ROOT / "pack" / "connect_flow_v1d_closure_v1" / "contract.detjson",
        ROOT / "pack" / "connect_endpoint_equality_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_endpoint_flow_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_endpoint_reverse_flow_v1" / "golden.jsonl",
        ROOT / "CONNECT_MULTI_INNER_SENTENCE_V1E.md",
    ]:
        require_file(path)

    for pack in PACKS:
        pack_dir = ROOT / "pack" / pack
        if not pack_dir.is_dir():
            fail(f"missing pack directory: pack/{pack}")
        for name in ["README.md", "input.ddn", "contract.detjson", "golden.jsonl"]:
            require_file(pack_dir / name)

    multi_contract = read_json(ROOT / "pack" / MULTI / "contract.detjson")
    artifact = multi_contract.get("artifact", {})
    if artifact.get("__이음관계종류") != "endpoint_relation_set":
        fail("multi pack must use endpoint_relation_set")
    if artifact.get("왼쪽끝") != "전지.양극" or artifact.get("오른쪽끝") != "전구.왼핀":
        fail("multi pack endpoint roots mismatch")
    relations = artifact.get("relations")
    if not isinstance(relations, list) or len(relations) != 2:
        fail("multi pack must define two relation artifacts")
    if [item.get("채널") for item in relations] != ["전압", "전류"]:
        fail("multi pack relation channel order mismatch")
    if multi_contract.get("relation_order") != "source_order_preserved":
        fail("multi pack must preserve source order")
    if multi_contract.get("single_inner_artifact_shape_changed") is not False:
        fail("single inner artifact shape must remain unchanged")
    if multi_contract.get("solver_claim") is not False:
        fail("multi pack must not claim solver integration")
    require_stdout(
        MULTI,
        [
            "endpoint_relation_set",
            "전지.양극",
            "전구.왼핀",
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
            "left_plus_right_zero",
            "왼쪽에서오른쪽",
        ],
    )

    econ_contract = read_json(ROOT / "pack" / ECON / "contract.detjson")
    econ_artifact = econ_contract.get("artifact", {})
    econ_relations = econ_artifact.get("relations")
    if econ_artifact.get("__이음관계종류") != "endpoint_relation_set":
        fail("econ pack must use endpoint_relation_set")
    if not isinstance(econ_relations, list) or len(econ_relations) != 3:
        fail("econ pack must define three relation artifacts")
    if [item.get("채널") for item in econ_relations] != ["체결값", "재화", "돈"]:
        fail("econ pack relation channel order mismatch")
    if [item.get("규칙") for item in econ_relations] != ["같게", "흐르게", "거슬러 흐르게"]:
        fail("econ pack relation rule order mismatch")
    if econ_contract.get("carried_property_supported") is not False:
        fail("실리게 must remain unsupported in V1E")
    if econ_contract.get("solver_claim") is not False:
        fail("econ pack must not claim solver integration")
    require_stdout(
        ECON,
        [
            "endpoint_relation_set",
            "가계1.구매끝",
            "장터.소매끝",
            "endpoint_equality",
            "같게",
            "체결값",
            "endpoint_flow",
            "흐르게",
            "재화",
            "왼쪽에서오른쪽",
            "endpoint_flow",
            "거슬러 흐르게",
            "돈",
            "오른쪽에서왼쪽",
        ],
    )

    closure_contract = read_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure_contract.get("bundled_packs") != BUNDLED:
        fail(f"closure bundled_packs mismatch: {closure_contract.get('bundled_packs')}")
    if closure_contract.get("relation_set_kind") != "endpoint_relation_set":
        fail("closure must record endpoint_relation_set")
    if closure_contract.get("relation_order") != "source_order_preserved":
        fail("closure must preserve source order")
    if closure_contract.get("carried_property_supported") is not False:
        fail("실리게 must remain unsupported in V1E closure")
    if closure_contract.get("solver_claim") is not False:
        fail("V1E closure must not claim solver integration")
    require_stdout(CLOSURE, ["connect_flow_v1e_closure_v1", *BUNDLED])

    root_doc = (ROOT / "CONNECT_MULTI_INNER_SENTENCE_V1E.md").read_text(encoding="utf-8")
    for marker in [
        "endpoint_relation_set",
        "관계들",
        "source order",
        "실리게",
        "connect_flow_v1e_closure_v1",
        "docs/ssot/**",
    ]:
        if marker not in root_doc:
            fail(f"root doc missing marker: {marker}")

    run([sys.executable, "tests/run_pack_golden.py", *PACKS])
    print("[connect-multi-inner-v1e] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

