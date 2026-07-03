#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SIGN = "connect_flow_sign_convention_v1"
FLOW = "connect_endpoint_flow_v1"
CLOSURE = "connect_flow_v1c_closure_v1"
PACKS = [SIGN, FLOW, CLOSURE]
BUNDLED = [
    "connect_endpoint_equality_v1",
    "connect_flow_sign_convention_v1",
    "connect_endpoint_flow_v1",
    "connect_subset_lowering_v1",
    "connect_relation_bridge_v1",
    "connect_wasm_cli_parity_v1",
]


def fail(message: str) -> None:
    print(f"[connect-flow-v1c] FAIL: {message}", file=sys.stderr)
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


def run(args: list[str]) -> None:
    proc = subprocess.run(args, cwd=ROOT, text=True, encoding="utf-8", errors="replace")
    if proc.returncode != 0:
        fail(f"command failed: {' '.join(args)}")


def require_single_stdout(pack: str, expected: list[str]) -> None:
    rows = read_jsonl(ROOT / "pack" / pack / "golden.jsonl")
    if len(rows) != 1:
        fail(f"pack/{pack}/golden.jsonl must have exactly one row")
    if rows[0].get("exit_code") != 0:
        fail(f"pack/{pack} must be a passing golden row")
    if rows[0].get("stdout") != expected:
        fail(f"pack/{pack} stdout mismatch: {rows[0].get('stdout')}")


def main() -> int:
    for path in [
        ROOT / "CONNECT_ENDPOINT_V1B.md",
        ROOT / "SEUM_ASSERTION_BLOCK_V1B.md",
        ROOT / "pack" / "connect_endpoint_equality_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_subset_lowering_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_relation_bridge_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_wasm_cli_parity_v1" / "golden.jsonl",
        ROOT / "CONNECT_FLOW_OI_RESOLUTION_AND_V1C.md",
    ]:
        require_file(path)

    for pack in PACKS:
        pack_dir = ROOT / "pack" / pack
        if not pack_dir.is_dir():
            fail(f"missing pack directory: pack/{pack}")
        for name in ["README.md", "input.ddn", "contract.detjson", "golden.jsonl"]:
            require_file(pack_dir / name)

    sign_contract = read_json(ROOT / "pack" / SIGN / "contract.detjson")
    if sign_contract.get("oi_id") != "OI-CONNECT-FLOW-SIGN-CONVENTION-01":
        fail("sign convention contract must name the open issue id")
    if sign_contract.get("convention_id") != "left_plus_right_zero":
        fail("sign convention must be left_plus_right_zero")
    if sign_contract.get("reverse_flow_supported") is not False:
        fail("reverse flow must remain unsupported in V1C")
    if sign_contract.get("carried_property_supported") is not False:
        fail("carried property must remain unsupported in V1C")
    if sign_contract.get("solver_claim") is not False:
        fail("sign convention pack must not claim solver integration")
    require_single_stdout(
        SIGN,
        [
            "connect_flow_sign_convention_v1",
            "left_plus_right_zero",
            "왼쪽에서오른쪽",
            "Y은/는 흐르게",
        ],
    )

    flow_contract = read_json(ROOT / "pack" / FLOW / "contract.detjson")
    artifact = flow_contract.get("artifact") or {}
    expected_artifact = {
        "__이음관계종류": "endpoint_flow",
        "왼쪽": "전지.양극.전류",
        "오른쪽": "전구.왼핀.전류",
        "규칙": "흐르게",
        "채널": "전류",
        "부호규약": "left_plus_right_zero",
        "방향": "왼쪽에서오른쪽",
    }
    if artifact != expected_artifact:
        fail(f"endpoint flow artifact mismatch: {artifact}")
    if flow_contract.get("solver_claim") is not False:
        fail("endpoint flow must not claim solver integration")
    require_single_stdout(FLOW, list(expected_artifact.values()))

    closure_contract = read_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure_contract.get("bundled_packs") != BUNDLED:
        fail(f"closure bundled_packs mismatch: {closure_contract.get('bundled_packs')}")
    if closure_contract.get("flow_convention") != "left_plus_right_zero":
        fail("closure must record left_plus_right_zero")
    require_single_stdout(
        CLOSURE,
        [
            "connect_flow_v1c_closure_v1",
            *BUNDLED,
            "left_plus_right_zero",
        ],
    )

    root_doc = (ROOT / "CONNECT_FLOW_OI_RESOLUTION_AND_V1C.md").read_text(encoding="utf-8")
    for marker in [
        "left_plus_right_zero",
        "거슬러 흐르게",
        "실리게",
        "connect_flow_v1c_closure_v1",
        "docs/ssot/**",
    ]:
        if marker not in root_doc:
            fail(f"root doc missing marker: {marker}")

    run([sys.executable, "tests/run_pack_golden.py", *PACKS])
    print("[connect-flow-v1c] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

