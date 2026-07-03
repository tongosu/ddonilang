#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PASS = "connect_endpoint_boundary_range_check_pass_v1"
FAIL = "connect_endpoint_boundary_range_check_fail_v1"
MISSING = "connect_endpoint_boundary_range_missing_value_v1"
UNIT_PASS = "connect_endpoint_boundary_range_unit_pass_v1"
UNIT_CONFLICT = "connect_endpoint_boundary_range_unit_conflict_v1"
UNSUPPORTED = "connect_endpoint_boundary_range_unsupported_v1"
CLOSURE = "connect_flow_v1n_closure_v1"
PACKS = [PASS, FAIL, MISSING, UNIT_PASS, UNIT_CONFLICT, UNSUPPORTED, CLOSURE]
BUNDLED = [
    "connect_flow_v1m_closure_v1",
    PASS,
    FAIL,
    MISSING,
    UNIT_PASS,
    UNIT_CONFLICT,
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
        ROOT / "CONNECT_ENDPOINT_BOUNDARY_RANGE_CHECK_V1N.md",
        ROOT / "CONNECT_ENDPOINT_UNIT_BOUNDARY_VALUE_V1M.md",
        ROOT / "pack" / "connect_flow_v1m_closure_v1" / "contract.detjson",
        ROOT / "pack" / "connect_endpoint_unit_boundary_explicit_solve_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_endpoint_unit_boundary_solve_remap_v1" / "golden.jsonl",
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
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_MISSING", str(missing))
    return 0


def check_contracts() -> int:
    pass_contract = read_json(ROOT / "pack" / PASS / "contract.detjson")
    if pass_contract.get("output_kind") != "endpoint_range_check":
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_PASS_KIND", str(pass_contract))
    if pass_contract.get("expected_result") != "통과":
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_PASS_RESULT", str(pass_contract))
    if pass_contract.get("solver_constraint") is not False:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_SOLVER_CONSTRAINT", str(pass_contract))

    fail_contract = read_json(ROOT / "pack" / FAIL / "contract.detjson")
    if fail_contract.get("expected_violation_reason") != "above_max":
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_FAIL_REASON", str(fail_contract))

    missing_contract = read_json(ROOT / "pack" / MISSING / "contract.detjson")
    if missing_contract.get("expected_violation_reason") != "missing_value":
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_MISSING_REASON", str(missing_contract))
    if missing_contract.get("missing_value_has_value_field") is not False:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_MISSING_SHAPE", str(missing_contract))
    if missing_contract.get("unknown_path_is_error") is not True:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_UNKNOWN_POLICY", str(missing_contract))

    unit = read_json(ROOT / "pack" / UNIT_PASS / "contract.detjson")
    if unit.get("unit_policy") != "connect_flow_v1m_canonical_base_numeric":
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_UNIT_POLICY", str(unit))

    conflict = read_json(ROOT / "pack" / UNIT_CONFLICT / "contract.detjson")
    if conflict.get("expected_error_code") != "connect_boundary_range_incompatible_unit":
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_INCOMPAT_MARKER", str(conflict))
    if conflict.get("also_covered_error_code") != "connect_boundary_range_dim_conflict":
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_DIM_MARKER", str(conflict))

    unsupported = read_json(ROOT / "pack" / UNSUPPORTED / "contract.detjson")
    expected = [
        "connect_boundary_range_duplicate_path",
        "connect_boundary_range_unknown_path",
        "connect_boundary_range_non_numeric",
        "connect_boundary_range_malformed_item",
        "connect_boundary_range_expected_solve_result",
    ]
    if unsupported.get("expected_error_codes") != expected:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_UNSUPPORTED_MARKERS", str(unsupported))

    closure = read_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure.get("bundled_packs") != BUNDLED:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_CLOSURE_BUNDLE", str(closure.get("bundled_packs")))
    if closure.get("post_solve_validation") is not True:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_CLOSURE_FLAG", str(closure))
    if closure.get("solver_constraint") is not False:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_CLOSURE_SOLVER_FLAG", str(closure))
    return 0


def check_golden_rows() -> int:
    if read_rows(PASS)[0].get("stdout") != ["endpoint_range_check", "통과", "0", "0"]:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_PASS_STDOUT", str(read_rows(PASS)[0]))
    if read_rows(FAIL)[0].get("stdout") != [
        "endpoint_range_check",
        "실패",
        "1",
        "above_max",
        "5",
        "4",
    ]:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_FAIL_STDOUT", str(read_rows(FAIL)[0]))
    if read_rows(MISSING)[0].get("stdout") != [
        "endpoint_range_check",
        "실패",
        "1",
        "missing_value",
        "전구.왼핀.전압",
        "0",
        "10",
    ]:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_MISSING_STDOUT", str(read_rows(MISSING)[0]))
    if read_rows(UNIT_PASS)[0].get("stdout") != ["endpoint_range_check", "통과", "0"]:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_UNIT_STDOUT", str(read_rows(UNIT_PASS)[0]))
    conflict = read_rows(UNIT_CONFLICT)[0]
    if conflict.get("expected_error_code") != "connect_boundary_range_incompatible_unit":
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_CONFLICT_ROW", str(conflict))
    if conflict.get("exit_code") != 1:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_CONFLICT_EXIT", str(conflict))
    unsupported = read_rows(UNSUPPORTED)[0]
    if unsupported.get("expected_error_code") != "connect_boundary_range_duplicate_path":
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_UNSUPPORTED_ROW", str(unsupported))
    if unsupported.get("exit_code") != 1:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_UNSUPPORTED_EXIT", str(unsupported))
    if read_rows(CLOSURE)[0].get("stdout") != [CLOSURE, *BUNDLED]:
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_CLOSURE_STDOUT", str(read_rows(CLOSURE)[0]))
    return 0


def check_docs() -> int:
    text = (ROOT / "CONNECT_ENDPOINT_BOUNDARY_RANGE_CHECK_V1N.md").read_text(encoding="utf-8")
    required = [
        "이음관계.범위위반목록",
        "이음관계.범위검사",
        "endpoint_range_check",
        "missing_value",
        "connect_boundary_range_incompatible_unit",
        "connect_boundary_range_dim_conflict",
        "post-solve validation",
    ]
    for token in required:
        if token not in text:
            return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_DOC", token)
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
        return fail("E_CONNECT_ENDPOINT_BOUNDARY_RANGE_GOLDEN", str(result.returncode))
    return 0


def main() -> int:
    for check in (require_files, check_contracts, check_golden_rows, check_docs, run_golden):
        rc = check()
        if rc:
            return rc
    print("[connect-endpoint-boundary-range-v1n] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
