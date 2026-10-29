#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

INJECTION = "connect_endpoint_unit_boundary_injection_v1"
SOLVE_REMAP = "connect_endpoint_unit_boundary_solve_remap_v1"
EXPLICIT_SOLVE = "connect_endpoint_unit_boundary_explicit_solve_v1"
DIM_CONFLICT = "connect_endpoint_unit_boundary_dim_conflict_v1"
INCOMPATIBLE_UNIT = "connect_endpoint_unit_boundary_incompatible_unit_v1"
CLOSURE = "connect_flow_v1m_closure_v1"
PACKS = [
    INJECTION,
    SOLVE_REMAP,
    EXPLICIT_SOLVE,
    DIM_CONFLICT,
    INCOMPATIBLE_UNIT,
    CLOSURE,
]
BUNDLED = [
    "connect_flow_v1l_closure_v1",
    INJECTION,
    SOLVE_REMAP,
    EXPLICIT_SOLVE,
    DIM_CONFLICT,
    INCOMPATIBLE_UNIT,
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
        ROOT / "CONNECT_ENDPOINT_UNIT_BOUNDARY_VALUE_V1M.md",
        ROOT / "CONNECT_ENDPOINT_EXPLICIT_SOLVE_HELPER_V1L.md",
        ROOT / "pack" / "connect_flow_v1l_closure_v1" / "contract.detjson",
        ROOT / "pack" / "connect_endpoint_explicit_solve_flow_value_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_endpoint_boundary_value_injection_v1" / "golden.jsonl",
        ROOT / "pack" / "lang_unit_temp_smoke_v1" / "golden.jsonl",
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
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_MISSING", str(missing))
    return 0


def check_contracts() -> int:
    injection = read_json(ROOT / "pack" / INJECTION / "contract.detjson")
    if injection.get("output_kind") != "endpoint_formula_relation_set_with_values":
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_INJECTION_KIND", str(injection))
    if injection.get("solver_relation_value_policy") != "canonical_base_numeric":
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_INJECTION_POLICY", str(injection))
    if injection.get("unit_metadata_fields") != ["단위차원", "단위기호"]:
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_METADATA", str(injection))

    solve = read_json(ROOT / "pack" / SOLVE_REMAP / "contract.detjson")
    if solve.get("explicit_solver_call") != "방정식풀기":
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_SOLVE_CALL", str(solve))
    if solve.get("unit_remap_policy") != "component_seed":
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_SOLVE_POLICY", str(solve))

    explicit = read_json(ROOT / "pack" / EXPLICIT_SOLVE / "contract.detjson")
    if explicit.get("helper") != "이음관계.풀기":
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_EXPLICIT_HELPER", str(explicit))
    if explicit.get("implicit_statement_solve") is not False:
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_EXPLICIT_AUTORUN", str(explicit))

    dim = read_json(ROOT / "pack" / DIM_CONFLICT / "contract.detjson")
    if dim.get("expected_error_code") != "connect_unit_boundary_dim_conflict":
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_DIM_MARKER", str(dim))

    incompatible = read_json(ROOT / "pack" / INCOMPATIBLE_UNIT / "contract.detjson")
    if incompatible.get("expected_error_code") != "connect_unit_boundary_incompatible_unit":
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_INCOMPAT_MARKER", str(incompatible))

    closure = read_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure.get("bundled_packs") != BUNDLED:
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_CLOSURE_BUNDLE", str(closure.get("bundled_packs")))
    if closure.get("unit_boundary_value") is not True:
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_CLOSURE_FLAG", str(closure))
    return 0


def check_golden_rows() -> int:
    injection_stdout = read_rows(INJECTION)[0].get("stdout")
    if injection_stdout != [
        "endpoint_formula_relation_set_with_values",
        "(#ascii) 수식{ 5 }",
        "5@KRW",
        "KRW",
        "KRW",
    ]:
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_INJECTION_STDOUT", str(injection_stdout))

    for pack in [SOLVE_REMAP, EXPLICIT_SOLVE]:
        stdout = read_rows(pack)[0].get("stdout")
        if stdout != ["endpoint_solve_result", "성공", "5@KRW", "-5@KRW"]:
            return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_SOLVE_STDOUT", f"{pack}: {stdout}")

    dim_row = read_rows(DIM_CONFLICT)[0]
    if dim_row.get("expected_error_code") != "connect_unit_boundary_dim_conflict":
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_DIM_ROW", str(dim_row))
    if dim_row.get("exit_code") != 1:
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_DIM_EXIT", str(dim_row))

    incompatible_row = read_rows(INCOMPATIBLE_UNIT)[0]
    if incompatible_row.get("expected_error_code") != "connect_unit_boundary_incompatible_unit":
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_INCOMPAT_ROW", str(incompatible_row))
    if incompatible_row.get("exit_code") != 1:
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_INCOMPAT_EXIT", str(incompatible_row))

    closure_stdout = read_rows(CLOSURE)[0].get("stdout")
    if closure_stdout != [CLOSURE, *BUNDLED]:
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_CLOSURE_STDOUT", str(closure_stdout))
    return 0


def check_docs() -> int:
    text = (ROOT / "CONNECT_ENDPOINT_UNIT_BOUNDARY_VALUE_V1M.md").read_text(encoding="utf-8")
    required = [
        "이음관계.값주입",
        "이음관계.풀이원복",
        "이음관계.풀기",
        "endpoint_formula_relation_set_with_values",
        "단위차원",
        "단위기호",
        "connect_unit_boundary_dim_conflict",
        "connect_unit_boundary_incompatible_unit",
        "solver에는 canonical base numeric",
    ]
    for token in required:
        if token not in text:
            return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_DOC", token)
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
        return fail("E_CONNECT_ENDPOINT_UNIT_BOUNDARY_GOLDEN", str(result.returncode))
    return 0


def main() -> int:
    for check in (require_files, check_contracts, check_golden_rows, check_docs, run_golden):
        rc = check()
        if rc:
            return rc
    print("[connect-endpoint-unit-boundary-v1m] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
