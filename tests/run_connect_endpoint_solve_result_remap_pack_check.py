#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SUCCESS = "connect_endpoint_solve_result_remap_success_v1"
PARTIAL = "connect_endpoint_solve_result_remap_partial_v1"
FAILURE = "connect_endpoint_solve_result_remap_failure_v1"
UNSUPPORTED = "connect_endpoint_solve_result_remap_unsupported_v1"
CLOSURE = "connect_flow_v1j_closure_v1"
PACKS = [SUCCESS, PARTIAL, FAILURE, UNSUPPORTED, CLOSURE]
BUNDLED = [
    "connect_flow_v1i_closure_v1",
    SUCCESS,
    PARTIAL,
    FAILURE,
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
        ROOT / "CONNECT_ENDPOINT_FORMULA_RELATION_BRIDGE_V1I.md",
        ROOT / "pack" / "connect_flow_v1i_closure_v1" / "contract.detjson",
        ROOT / "pack" / "connect_endpoint_formula_relation_solve_v1" / "golden.jsonl",
        ROOT / "pack" / "relation_solve_ddn_bridge_v2" / "golden.jsonl",
        ROOT / "CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_V1J.md",
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
        return fail("E_CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_MISSING", str(missing))
    return 0


def check_contracts() -> int:
    for pack, expected_kind in [
        (SUCCESS, "성공"),
        (PARTIAL, "부분성공"),
        (FAILURE, "실패"),
    ]:
        contract = read_json(ROOT / "pack" / pack / "contract.detjson")
        if contract.get("schema") != "ddn.connect_endpoint_solve_result_remap.pack.contract.v1":
            return fail("E_CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_SCHEMA", pack)
        if contract.get("output_kind") != "endpoint_solve_result":
            return fail("E_CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_KIND", pack)
        if contract.get("expected_result_kind") != expected_kind:
            return fail("E_CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_RESULT_KIND", pack)
        if contract.get("solver_auto_run") is not False:
            return fail("E_CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_AUTORUN", pack)

    unsupported = read_json(ROOT / "pack" / UNSUPPORTED / "contract.detjson")
    if unsupported.get("unsupported_case") != "solver_binding_not_in_variable_mapping":
        return fail("E_CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_UNSUPPORTED_CASE", str(unsupported))
    if unsupported.get("expected_error_code") != "endpoint_variable_mapping":
        return fail("E_CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_UNSUPPORTED_MARKER", str(unsupported))

    closure = read_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure.get("bundled_packs") != BUNDLED:
        return fail("E_CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_CLOSURE_BUNDLE", str(closure.get("bundled_packs")))
    if closure.get("output_kind") != "endpoint_solve_result":
        return fail("E_CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_CLOSURE_KIND", str(closure.get("output_kind")))
    if closure.get("solver_auto_run") is not False:
        return fail("E_CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_CLOSURE_AUTORUN", str(closure.get("solver_auto_run")))
    return 0


def check_golden_rows() -> int:
    success_stdout = read_rows(SUCCESS)[0].get("stdout")
    if success_stdout != [
        "endpoint_solve_result",
        "성공",
        "차림[]",
        "ep_001",
        "전지.양극.전압",
        "0",
        "ep_002",
        "전구.왼핀.전압",
        "0",
    ]:
        return fail("E_CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_SUCCESS_STDOUT", str(success_stdout))

    partial_stdout = read_rows(PARTIAL)[0].get("stdout")
    if partial_stdout != [
        "endpoint_solve_result",
        "부분성공",
        "차림[ep_002]",
        "ep_001",
        "전지.양극.전압",
        "0",
    ]:
        return fail("E_CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_PARTIAL_STDOUT", str(partial_stdout))

    failure_stdout = read_rows(FAILURE)[0].get("stdout")
    if failure_stdout != [
        "endpoint_solve_result",
        "실패",
        "차림[]",
        "차림[ep_001, ep_002]",
        '#실패(사유="no_solution")',
    ]:
        return fail("E_CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_FAILURE_STDOUT", str(failure_stdout))

    unsupported_row = read_rows(UNSUPPORTED)[0]
    if unsupported_row.get("expected_error_code") != "endpoint_variable_mapping" or unsupported_row.get("exit_code") != 1:
        return fail("E_CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_UNSUPPORTED_ROW", str(unsupported_row))

    closure_stdout = read_rows(CLOSURE)[0].get("stdout")
    if closure_stdout != [CLOSURE, *BUNDLED]:
        return fail("E_CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_CLOSURE_STDOUT", str(closure_stdout))
    return 0


def check_docs() -> int:
    text = (ROOT / "CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_V1J.md").read_text(encoding="utf-8")
    required = [
        "이음관계.풀이값목록",
        "이음관계.풀이원복",
        "endpoint_solve_result",
        "부분성공",
        "누락변수들",
        "solver 자동 실행",
    ]
    for token in required:
        if token not in text:
            return fail("E_CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_DOC", token)
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
        return fail("E_CONNECT_ENDPOINT_SOLVE_RESULT_REMAP_GOLDEN", str(result.returncode))
    return 0


def main() -> int:
    for check in (require_files, check_contracts, check_golden_rows, check_docs, run_golden):
        rc = check()
        if rc:
            return rc
    print("[connect-endpoint-solve-result-remap-v1j] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
