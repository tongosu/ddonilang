#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

INJECTION = "connect_endpoint_boundary_value_injection_v1"
SOLVE_REMAP = "connect_endpoint_boundary_value_solve_remap_v1"
UNSUPPORTED = "connect_endpoint_boundary_value_unsupported_v1"
CLOSURE = "connect_flow_v1k_closure_v1"
PACKS = [INJECTION, SOLVE_REMAP, UNSUPPORTED, CLOSURE]
BUNDLED = [
    "connect_flow_v1j_closure_v1",
    INJECTION,
    SOLVE_REMAP,
    UNSUPPORTED,
]
ERROR_MARKERS = [
    "connect_boundary_value_duplicate_path",
    "connect_boundary_value_unknown_path",
    "connect_boundary_value_non_numeric",
    "connect_boundary_value_expected_formula_set",
    "connect_boundary_value_malformed_item",
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
        ROOT / "CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_V1J.md",
        ROOT / "pack" / "connect_flow_v1j_closure_v1" / "contract.detjson",
        ROOT / "pack" / "connect_endpoint_solve_result_remap_success_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_endpoint_formula_relation_solve_v1" / "golden.jsonl",
        ROOT / "CONNECT_ENDPOINT_BOUNDARY_VALUE_INJECTION_V1K.md",
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
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_MISSING", str(missing))
    return 0


def check_contracts() -> int:
    injection = read_json(ROOT / "pack" / INJECTION / "contract.detjson")
    if injection.get("schema") != "ddn.connect_endpoint_boundary_value.pack.contract.v1":
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_INJECTION_SCHEMA", str(injection))
    if injection.get("output_kind") != "endpoint_formula_relation_set_with_values":
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_INJECTION_KIND", str(injection.get("output_kind")))
    if injection.get("value_relation_pack_family") != "relation_equation":
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_RELATION_FAMILY", str(injection))
    if injection.get("value_relation_item_kind") != "방정식":
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_RELATION_KIND", str(injection))
    if injection.get("solver_auto_run") is not False:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_INJECTION_AUTORUN", str(injection))

    solve = read_json(ROOT / "pack" / SOLVE_REMAP / "contract.detjson")
    if solve.get("output_kind") != "endpoint_solve_result":
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_SOLVE_KIND", str(solve.get("output_kind")))
    if solve.get("expected_result_kind") != "성공":
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_SOLVE_RESULT", str(solve))
    if solve.get("explicit_solver_call") != "방정식풀기" or solve.get("solver_auto_run") is not False:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_SOLVE_AUTORUN", str(solve))

    unsupported = read_json(ROOT / "pack" / UNSUPPORTED / "contract.detjson")
    markers = unsupported.get("expected_error_codes")
    if markers != ERROR_MARKERS:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_UNSUPPORTED_MARKERS", str(markers))
    if unsupported.get("golden_case") != "duplicate_endpoint_path":
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_UNSUPPORTED_CASE", str(unsupported))

    closure = read_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure.get("bundled_packs") != BUNDLED:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_CLOSURE_BUNDLE", str(closure.get("bundled_packs")))
    if closure.get("solver_auto_run") is not False:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_CLOSURE_AUTORUN", str(closure))
    return 0


def check_golden_rows() -> int:
    injection_stdout = read_rows(INJECTION)[0].get("stdout")
    if injection_stdout != [
        "endpoint_formula_relation_set_with_values",
        "2",
        "2",
        "1",
        "방정식",
        "(#ascii) 수식{ ep_001 }",
        "(#ascii) 수식{ 5 }",
        "ep_001",
        "전지.양극.전압",
        "5",
    ]:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_INJECTION_STDOUT", str(injection_stdout))

    solve_stdout = read_rows(SOLVE_REMAP)[0].get("stdout")
    if solve_stdout != [
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
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_SOLVE_STDOUT", str(solve_stdout))

    unsupported_row = read_rows(UNSUPPORTED)[0]
    if unsupported_row.get("expected_error_code") != "connect_boundary_value_duplicate_path":
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_UNSUPPORTED_ROW", str(unsupported_row))
    if unsupported_row.get("exit_code") != 1:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_UNSUPPORTED_EXIT", str(unsupported_row))

    closure_stdout = read_rows(CLOSURE)[0].get("stdout")
    if closure_stdout != [CLOSURE, *BUNDLED]:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_CLOSURE_STDOUT", str(closure_stdout))
    return 0


def check_docs() -> int:
    text = (ROOT / "CONNECT_ENDPOINT_BOUNDARY_VALUE_INJECTION_V1K.md").read_text(encoding="utf-8")
    required = [
        "이음관계.값관계목록",
        "이음관계.값주입",
        "endpoint_formula_relation_set_with_values",
        "connect_boundary_value_duplicate_path",
        "connect_boundary_value_unknown_path",
        "connect_boundary_value_non_numeric",
        "connect_boundary_value_expected_formula_set",
        "connect_boundary_value_malformed_item",
        "solver 자동 실행",
    ]
    for token in required:
        if token not in text:
            return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_DOC", token)
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
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_VALUE_GOLDEN", str(result.returncode))
    return 0


def main() -> int:
    for check in (require_files, check_contracts, check_golden_rows, check_docs, run_golden):
        rc = check()
        if rc:
            return rc
    print("[connect-endpoint-boundary-value-v1k] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
