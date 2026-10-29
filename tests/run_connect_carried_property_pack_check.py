#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

FORWARD = "connect_endpoint_carried_property_forward_v1"
REVERSE = "connect_endpoint_carried_property_reverse_v1"
CLOSURE = "connect_flow_v1f_closure_v1"
PACKS = [FORWARD, REVERSE, CLOSURE]
BUNDLED = [
    "connect_flow_v1e_closure_v1",
    "connect_endpoint_carried_property_forward_v1",
    "connect_endpoint_carried_property_reverse_v1",
]


def fail(message: str) -> None:
    print(f"[connect-carried-property-v1f] FAIL: {message}", file=sys.stderr)
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


def require_carried_contract(
    pack: str,
    left: str,
    right: str,
    carrier: str,
    prop: str,
    carrier_rule: str,
    direction: str,
) -> None:
    contract = read_json(ROOT / "pack" / pack / "contract.detjson")
    artifact = contract.get("artifact", {})
    if artifact.get("__이음관계종류") != "endpoint_relation_set":
        fail(f"{pack} must use endpoint_relation_set")
    if artifact.get("왼쪽끝") != left or artifact.get("오른쪽끝") != right:
        fail(f"{pack} endpoint roots mismatch")
    relations = artifact.get("relations")
    if not isinstance(relations, list) or len(relations) != 2:
        fail(f"{pack} must define flow + carried-property relations")

    flow, carried = relations
    if flow.get("__이음관계종류") != "endpoint_flow":
        fail(f"{pack} first relation must be endpoint_flow")
    if flow.get("채널") != carrier or flow.get("규칙") != carrier_rule:
        fail(f"{pack} carrier flow rule/channel mismatch")
    if flow.get("부호규약") != "left_plus_right_zero":
        fail(f"{pack} carrier flow must keep left_plus_right_zero")
    if flow.get("방향") != direction:
        fail(f"{pack} carrier flow direction mismatch")

    if carried.get("__이음관계종류") != "endpoint_carried_property":
        fail(f"{pack} second relation must be endpoint_carried_property")
    if carried.get("왼쪽운반자") != f"{left}.{carrier}":
        fail(f"{pack} left carrier path mismatch")
    if carried.get("오른쪽운반자") != f"{right}.{carrier}":
        fail(f"{pack} right carrier path mismatch")
    if carried.get("속성") != prop or carried.get("운반채널") != carrier:
        fail(f"{pack} carried property/channel mismatch")
    if carried.get("규칙") != "실리게":
        fail(f"{pack} carried-property rule mismatch")
    if carried.get("운반규칙") != carrier_rule or carried.get("운반방향") != direction:
        fail(f"{pack} carrier metadata mismatch")

    if contract.get("relation_order") != "source_order_preserved":
        fail(f"{pack} must preserve source order")
    if contract.get("carrier_flow_required") is not True:
        fail(f"{pack} must require a carrier flow")
    if contract.get("standalone_carried_property_supported") is not False:
        fail(f"{pack} must reject standalone carried property")
    if contract.get("solver_claim") is not False:
        fail(f"{pack} must not claim solver integration")


def main() -> int:
    for path in [
        ROOT / "CONNECT_MULTI_INNER_SENTENCE_V1E.md",
        ROOT / "pack" / "connect_flow_v1e_closure_v1" / "contract.detjson",
        ROOT / "pack" / "connect_endpoint_multi_inner_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_endpoint_multi_inner_econ_v1" / "golden.jsonl",
        ROOT / "CONNECT_CARRIED_PROPERTY_V1F.md",
    ]:
        require_file(path)

    for pack in PACKS:
        pack_dir = ROOT / "pack" / pack
        if not pack_dir.is_dir():
            fail(f"missing pack directory: pack/{pack}")
        for name in ["README.md", "input.ddn", "contract.detjson", "golden.jsonl"]:
            require_file(pack_dir / name)

    require_carried_contract(
        FORWARD,
        left="은행.대출창구",
        right="기업1.차입끝",
        carrier="대출금",
        prop="위험",
        carrier_rule="흐르게",
        direction="왼쪽에서오른쪽",
    )
    require_stdout(
        FORWARD,
        [
            "endpoint_relation_set",
            "은행.대출창구",
            "기업1.차입끝",
            "endpoint_flow",
            "흐르게",
            "대출금",
            "왼쪽에서오른쪽",
            "endpoint_carried_property",
            "은행.대출창구.대출금",
            "기업1.차입끝.대출금",
            "위험",
            "대출금",
            "실리게",
            "흐르게",
            "왼쪽에서오른쪽",
        ],
    )

    require_carried_contract(
        REVERSE,
        left="가계1.구매끝",
        right="장터.소매끝",
        carrier="돈",
        prop="재화",
        carrier_rule="거슬러 흐르게",
        direction="오른쪽에서왼쪽",
    )
    require_stdout(
        REVERSE,
        [
            "endpoint_relation_set",
            "가계1.구매끝",
            "장터.소매끝",
            "endpoint_flow",
            "거슬러 흐르게",
            "돈",
            "오른쪽에서왼쪽",
            "endpoint_carried_property",
            "가계1.구매끝.돈",
            "장터.소매끝.돈",
            "재화",
            "돈",
            "실리게",
            "거슬러 흐르게",
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
    if closure_contract.get("carried_property_supported") is not True:
        fail("V1F closure must support carried property")
    if closure_contract.get("standalone_carried_property_supported") is not False:
        fail("V1F closure must reject standalone carried property")
    if closure_contract.get("carrier_flow_required") is not True:
        fail("V1F closure must require carrier flow")
    if closure_contract.get("solver_claim") is not False:
        fail("V1F closure must not claim solver integration")
    require_stdout(CLOSURE, ["connect_flow_v1f_closure_v1", *BUNDLED])

    root_doc = (ROOT / "CONNECT_CARRIED_PROPERTY_V1F.md").read_text(encoding="utf-8")
    for marker in [
        "endpoint_carried_property",
        "운반채널",
        "standalone",
        "connect_flow_v1f_closure_v1",
        "docs/ssot/**",
    ]:
        if marker not in root_doc:
            fail(f"root doc missing marker: {marker}")

    run([sys.executable, "tests/run_pack_golden.py", *PACKS])
    print("[connect-carried-property-v1f] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
