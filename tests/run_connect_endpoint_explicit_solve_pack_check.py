#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

EQUAL = "connect_endpoint_explicit_solve_equal_value_v1"
FLOW = "connect_endpoint_explicit_solve_flow_value_v1"
FLAT = "connect_endpoint_explicit_solve_flat_set_value_v1"
UNSUPPORTED = "connect_endpoint_explicit_solve_unsupported_v1"
CLOSURE = "connect_flow_v1l_closure_v1"
PACKS = [EQUAL, FLOW, FLAT, UNSUPPORTED, CLOSURE]
BUNDLED = [
    "connect_flow_v1k_closure_v1",
    EQUAL,
    FLOW,
    FLAT,
    UNSUPPORTED,
]


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def read_rows(pack: str) -> list[dict]:
    path = ROOT / "pack" / pack / "golden.jsonl"
    return [
        json.loads(line)
        for line in path.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]


def require_files() -> int:
    required = [
        ROOT / "CONNECT_ENDPOINT_BOUNDARY_VALUE_INJECTION_V1K.md",
        ROOT / "pack" / "connect_flow_v1k_closure_v1" / "contract.detjson",
        ROOT / "pack" / "connect_endpoint_boundary_value_injection_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_endpoint_relation_normalize_v1" / "golden.jsonl",
        ROOT / "CONNECT_ENDPOINT_EXPLICIT_SOLVE_HELPER_V1L.md",
    ]
    for pack in PACKS:
        required.extend(
            [
                ROOT / "pack" / pack / "README.md",
                ROOT / "pack" / pack / "input.ddn",
                ROOT / "pack" / pack / "contract.detjson",
                ROOT / "pack" / pack / "golden.jsonl",
            ]
        )
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_MISSING", str(missing))
    return 0


def check_contracts() -> int:
    for pack, input_kind in [
        (EQUAL, "raw_endpoint_equality"),
        (FLOW, "raw_endpoint_flow"),
        (FLAT, "endpoint_relation_flat_set"),
    ]:
        contract = read_json(ROOT / "pack" / pack / "contract.detjson")
        if contract.get("schema") != "ddn.connect_endpoint_explicit_solve.pack.contract.v1":
            return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_SCHEMA", pack)
        if contract.get("input_kind") != input_kind:
            return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_INPUT_KIND", str(contract))
        if contract.get("output_kind") != "endpoint_solve_result":
            return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_OUTPUT_KIND", str(contract))
        if contract.get("expected_result_kind") != "성공":
            return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_RESULT_KIND", str(contract))
        if contract.get("explicit_helper") != "이음관계.풀기":
            return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_HELPER", str(contract))
        if contract.get("implicit_statement_solve") is not False:
            return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_AUTORUN", str(contract))

    flat = read_json(ROOT / "pack" / FLAT / "contract.detjson")
    if flat.get("flat_set_handling") != "skip_normalize_inside_explicit_solve":
        return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_FLAT_POLICY", str(flat))

    unsupported = read_json(ROOT / "pack" / UNSUPPORTED / "contract.detjson")
    if unsupported.get("expected_error_code") != "connect_boundary_value_duplicate_path":
        return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_UNSUPPORTED_MARKER", str(unsupported))
    if unsupported.get("implicit_statement_solve") is not False:
        return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_UNSUPPORTED_AUTORUN", str(unsupported))

    closure = read_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure.get("bundled_packs") != BUNDLED:
        return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_CLOSURE_BUNDLE", str(closure.get("bundled_packs")))
    if closure.get("implicit_statement_solve") is not False:
        return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_CLOSURE_AUTORUN", str(closure))
    return 0


def check_golden_rows() -> int:
    equal_stdout = read_rows(EQUAL)[0].get("stdout")
    if equal_stdout != [
        "endpoint_solve_result",
        "성공",
        "차림[]",
        "ep_001",
        "전지.양극.전압",
        "5",
        "ep_002",
        "전구.왼핀.전압",
        "5",
    ]:
        return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_EQUAL_STDOUT", str(equal_stdout))

    flow_stdout = read_rows(FLOW)[0].get("stdout")
    if flow_stdout != [
        "endpoint_solve_result",
        "성공",
        "전지.양극.전류",
        "5",
        "전구.왼핀.전류",
        "-5",
    ]:
        return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_FLOW_STDOUT", str(flow_stdout))

    flat_stdout = read_rows(FLAT)[0].get("stdout")
    if flat_stdout != [
        "endpoint_relation_flat_set",
        "endpoint_solve_result",
        "성공",
        "5",
        "5",
    ]:
        return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_FLAT_STDOUT", str(flat_stdout))

    unsupported_row = read_rows(UNSUPPORTED)[0]
    if unsupported_row.get("expected_error_code") != "connect_boundary_value_duplicate_path":
        return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_UNSUPPORTED_ROW", str(unsupported_row))
    if unsupported_row.get("exit_code") != 1:
        return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_UNSUPPORTED_EXIT", str(unsupported_row))

    closure_stdout = read_rows(CLOSURE)[0].get("stdout")
    if closure_stdout != [CLOSURE, *BUNDLED]:
        return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_CLOSURE_STDOUT", str(closure_stdout))
    return 0


def check_docs() -> int:
    text = (ROOT / "CONNECT_ENDPOINT_EXPLICIT_SOLVE_HELPER_V1L.md").read_text(encoding="utf-8")
    required = [
        "이음관계.풀기",
        "endpoint_relation_flat_set",
        "정규화 단계를 skip",
        "endpoint_solve_result",
        "자동 solver 실행",
        "connect_endpoint_explicit_solve_flat_set_value_v1",
    ]
    for token in required:
        if token not in text:
            return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_DOC", token)
    return 0


def run_golden() -> int:
    result = subprocess.run(
        [sys.executable, "tests/run_pack_golden.py", *PACKS],
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode != 0:
        print(result.stdout)
        return fail("E_CONNECT_ENDPOINT_EXPLICIT_SOLVE_GOLDEN", str(result.returncode))
    return 0


def main() -> int:
    for check in (require_files, check_contracts, check_golden_rows, check_docs, run_golden):
        rc = check()
        if rc:
            return rc
    print("[connect-endpoint-explicit-solve-v1l] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
