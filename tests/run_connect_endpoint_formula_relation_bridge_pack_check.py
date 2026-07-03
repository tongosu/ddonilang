#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

BRIDGE = "connect_endpoint_formula_relation_bridge_v1"
SOLVE = "connect_endpoint_formula_relation_solve_v1"
UNSUPPORTED = "connect_endpoint_formula_relation_unsupported_v1"
CLOSURE = "connect_flow_v1i_closure_v1"
PACKS = [BRIDGE, SOLVE, UNSUPPORTED, CLOSURE]
BUNDLED = [
    "connect_flow_v1h_closure_v1",
    BRIDGE,
    SOLVE,
    UNSUPPORTED,
    "relation_solve_ddn_bridge_v2",
    "relation_solve_wasm_cli_parity_v2",
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
        ROOT / "CONNECT_ENDPOINT_RELATION_NORMALIZE_V1H.md",
        ROOT / "pack" / "connect_flow_v1h_closure_v1" / "contract.detjson",
        ROOT / "pack" / "relation_solve_ddn_bridge_v2" / "golden.jsonl",
        ROOT / "pack" / "relation_solve_wasm_cli_parity_v2" / "golden.jsonl",
        ROOT / "CONNECT_ENDPOINT_FORMULA_RELATION_BRIDGE_V1I.md",
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
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_MISSING", str(missing))
    return 0


def check_contracts() -> int:
    bridge = read_json(ROOT / "pack" / BRIDGE / "contract.detjson")
    if bridge.get("schema") != "ddn.connect_endpoint_formula_relation_bridge.pack.contract.v1":
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_BRIDGE_SCHEMA", str(bridge.get("schema")))
    if bridge.get("output_kind") != "endpoint_formula_relation_set":
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_OUTPUT_KIND", str(bridge.get("output_kind")))
    if bridge.get("variable_mapping") != "first_seen_endpoint_path_order":
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_MAPPING", str(bridge.get("variable_mapping")))
    if bridge.get("same_path_reuses_variable") is not True:
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_REUSE", str(bridge.get("same_path_reuses_variable")))
    if bridge.get("solver_auto_run") is not False:
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_AUTOSOLVE", str(bridge.get("solver_auto_run")))

    solve = read_json(ROOT / "pack" / SOLVE / "contract.detjson")
    if solve.get("explicit_solver_call") != "방정식풀기":
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_SOLVE_CALL", str(solve.get("explicit_solver_call")))
    if solve.get("solver_auto_run") is not False:
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_SOLVE_AUTORUN", str(solve.get("solver_auto_run")))

    unsupported = read_json(ROOT / "pack" / UNSUPPORTED / "contract.detjson")
    if unsupported.get("unsupported_kind") != "endpoint_carried_property":
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_UNSUPPORTED_KIND", str(unsupported.get("unsupported_kind")))
    if unsupported.get("expected_error_code") != "endpoint_carried_property":
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_UNSUPPORTED_MARKER", str(unsupported.get("expected_error_code")))

    closure = read_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure.get("bundled_packs") != BUNDLED:
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_CLOSURE_BUNDLE", str(closure.get("bundled_packs")))
    if closure.get("solver_auto_run") is not False:
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_CLOSURE_AUTORUN", str(closure.get("solver_auto_run")))
    if closure.get("output_kind") != "endpoint_formula_relation_set":
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_CLOSURE_KIND", str(closure.get("output_kind")))
    return 0


def check_golden_rows() -> int:
    bridge_rows = read_rows(BRIDGE)
    if len(bridge_rows) != 1:
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_BRIDGE_ROWS", str(len(bridge_rows)))
    bridge_stdout = bridge_rows[0].get("stdout")
    required_bridge_tokens = [
        "endpoint_formula_relation_set",
        "방정식",
        "(#ascii) 수식{ ep_001 }",
        "(#ascii) 수식{ ep_002 }",
        "(#ascii) 수식{ ep_001 + ep_002 }",
        "전지.양극.전압",
        "전구.왼핀.전압",
    ]
    for token in required_bridge_tokens:
        if token not in bridge_stdout:
            return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_BRIDGE_STDOUT", token)

    solve_rows = read_rows(SOLVE)
    if solve_rows[0].get("stdout") != ['#성공(해=(ep_001=0, ep_002=0))']:
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_SOLVE_STDOUT", str(solve_rows[0].get("stdout")))

    unsupported_rows = read_rows(UNSUPPORTED)
    row = unsupported_rows[0]
    if row.get("expected_error_code") != "endpoint_carried_property" or row.get("exit_code") != 1:
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_UNSUPPORTED_ROW", str(row))

    closure_rows = read_rows(CLOSURE)
    if closure_rows[0].get("stdout") != [CLOSURE, *BUNDLED]:
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_CLOSURE_STDOUT", str(closure_rows[0].get("stdout")))
    return 0


def check_docs() -> int:
    text = (ROOT / "CONNECT_ENDPOINT_FORMULA_RELATION_BRIDGE_V1I.md").read_text(encoding="utf-8")
    required = [
        "이음관계.방정식목록",
        "이음관계.방정식화",
        "endpoint_formula_relation_set",
        "ep_001",
        "endpoint_carried_property",
        "solver 자동 실행",
    ]
    for token in required:
        if token not in text:
            return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_DOC", token)
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
        return fail("E_CONNECT_ENDPOINT_FORMULA_RELATION_GOLDEN", str(result.returncode))
    return 0


def main() -> int:
    for check in (require_files, check_contracts, check_golden_rows, check_docs, run_golden):
        rc = check()
        if rc:
            return rc
    print("[connect-endpoint-formula-relation-v1i] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
