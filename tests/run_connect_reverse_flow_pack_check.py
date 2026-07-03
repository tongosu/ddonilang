#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

REVERSE = "connect_endpoint_reverse_flow_v1"
CLOSURE = "connect_flow_v1d_closure_v1"
PACKS = [REVERSE, CLOSURE]
BUNDLED = ["connect_flow_v1c_closure_v1", "connect_endpoint_reverse_flow_v1"]


def fail(message: str) -> None:
    print(f"[connect-reverse-flow-v1d] FAIL: {message}", file=sys.stderr)
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
        ROOT / "CONNECT_FLOW_OI_RESOLUTION_AND_V1C.md",
        ROOT / "pack" / "connect_flow_v1c_closure_v1" / "contract.detjson",
        ROOT / "pack" / "connect_endpoint_flow_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_flow_sign_convention_v1" / "golden.jsonl",
        ROOT / "CONNECT_REVERSE_FLOW_V1D.md",
    ]:
        require_file(path)

    for pack in PACKS:
        pack_dir = ROOT / "pack" / pack
        if not pack_dir.is_dir():
            fail(f"missing pack directory: pack/{pack}")
        for name in ["README.md", "input.ddn", "contract.detjson", "golden.jsonl"]:
            require_file(pack_dir / name)

    reverse_contract = read_json(ROOT / "pack" / REVERSE / "contract.detjson")
    expected_artifact = {
        "__이음관계종류": "endpoint_flow",
        "왼쪽": "가계1.구매끝.돈",
        "오른쪽": "장터.소매끝.돈",
        "규칙": "거슬러 흐르게",
        "채널": "돈",
        "부호규약": "left_plus_right_zero",
        "방향": "오른쪽에서왼쪽",
    }
    if reverse_contract.get("artifact") != expected_artifact:
        fail(f"reverse flow artifact mismatch: {reverse_contract.get('artifact')}")
    if reverse_contract.get("path_order") != "sentence_order_preserved":
        fail("reverse flow must preserve sentence endpoint path order")
    if reverse_contract.get("solver_claim") is not False:
        fail("reverse flow must not claim solver integration")
    require_stdout(REVERSE, list(expected_artifact.values()))

    closure_contract = read_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure_contract.get("bundled_packs") != BUNDLED:
        fail(f"closure bundled_packs mismatch: {closure_contract.get('bundled_packs')}")
    if closure_contract.get("flow_convention") != "left_plus_right_zero":
        fail("closure must keep left_plus_right_zero")
    if closure_contract.get("reverse_flow_direction") != "오른쪽에서왼쪽":
        fail("closure reverse_flow_direction mismatch")
    if closure_contract.get("carried_property_supported") is not False:
        fail("실리게 must remain unsupported in V1D")
    if closure_contract.get("solver_claim") is not False:
        fail("V1D closure must not claim solver integration")
    require_stdout(CLOSURE, ["connect_flow_v1d_closure_v1", *BUNDLED, "left_plus_right_zero"])

    root_doc = (ROOT / "CONNECT_REVERSE_FLOW_V1D.md").read_text(encoding="utf-8")
    for marker in [
        "거슬러 흐르게",
        "오른쪽에서왼쪽",
        "left_plus_right_zero",
        "실리게",
        "connect_flow_v1d_closure_v1",
        "docs/ssot/**",
    ]:
        if marker not in root_doc:
            fail(f"root doc missing marker: {marker}")

    run([sys.executable, "tests/run_pack_golden.py", *PACKS])
    print("[connect-reverse-flow-v1d] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

