#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PASS = "connect_endpoint_solve_range_report_pass_v1"
FAIL = "connect_endpoint_solve_range_report_fail_v1"
UNIT = "connect_endpoint_solve_range_report_unit_v1"
MISSING = "connect_endpoint_solve_range_report_missing_value_v1"
UNSUPPORTED = "connect_endpoint_solve_range_report_unsupported_v1"
CLOSURE = "connect_flow_v1p_closure_v1"
PACKS = [PASS, FAIL, UNIT, MISSING, UNSUPPORTED, CLOSURE]
BUNDLED = [
    "connect_flow_v1o_closure_v1",
    PASS,
    FAIL,
    UNIT,
    MISSING,
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
        ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_REPORT_V1P.md",
        ROOT / "CONNECT_ENDPOINT_EXPLICIT_SOLVE_RANGE_CHECK_V1O.md",
        ROOT / "pack" / "connect_flow_v1o_closure_v1" / "contract.detjson",
        ROOT / "pack" / "connect_endpoint_explicit_solve_range_pass_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_endpoint_explicit_solve_range_missing_value_v1" / "golden.jsonl",
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
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_MISSING", str(missing))
    return 0


def check_contracts() -> int:
    pass_contract = read_json(ROOT / "pack" / PASS / "contract.detjson")
    if pass_contract.get("output_kind") != "endpoint_solve_range_report":
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_PASS_KIND", str(pass_contract))
    if pass_contract.get("expected_left_range_status") != "범위없음":
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_PASS_LEFT", str(pass_contract))
    if pass_contract.get("expected_right_range_status") != "통과":
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_PASS_RIGHT", str(pass_contract))

    fail_contract = read_json(ROOT / "pack" / FAIL / "contract.detjson")
    if fail_contract.get("expected_violation_reason") != "below_min":
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_FAIL_REASON", str(fail_contract))

    unit_contract = read_json(ROOT / "pack" / UNIT / "contract.detjson")
    if unit_contract.get("unit_policy") != "connect_flow_v1m_canonical_base_numeric":
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_UNIT_POLICY", str(unit_contract))

    missing_contract = read_json(ROOT / "pack" / MISSING / "contract.detjson")
    if missing_contract.get("missing_with_range_status") != "실패":
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_MISSING_WITH_RANGE", str(missing_contract))
    if missing_contract.get("missing_without_range_status") != "범위없음":
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_MISSING_WITHOUT_RANGE", str(missing_contract))
    if missing_contract.get("missing_value_has_value_field") is not False:
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_MISSING_VALUE_SHAPE", str(missing_contract))

    unsupported = read_json(ROOT / "pack" / UNSUPPORTED / "contract.detjson")
    if unsupported.get("primary_expected_error_code") != "connect_boundary_range_unknown_path":
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_UNSUPPORTED_PRIMARY", str(unsupported))
    if unsupported.get("unknown_path_has_report_row") is not False:
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_UNKNOWN_ROW", str(unsupported))

    closure = read_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure.get("bundled_packs") != BUNDLED:
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_CLOSURE_BUNDLE", str(closure.get("bundled_packs")))
    if closure.get("auto_solve") is not False:
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_AUTO_SOLVE", str(closure))
    if closure.get("solver_constraint") is not False:
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_SOLVER_CONSTRAINT", str(closure))
    return 0


def check_golden_rows() -> int:
    if read_rows(PASS)[0].get("stdout") != [
        "endpoint_solve_range_report",
        "통과",
        "2",
        "2",
        "0",
        "1",
        "0",
        "범위없음",
        "통과",
        "2",
    ]:
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_PASS_STDOUT", str(read_rows(PASS)[0]))
    if read_rows(FAIL)[0].get("stdout") != [
        "endpoint_solve_range_report",
        "실패",
        "1",
        "실패",
        "1",
        "below_min",
        "-5",
        "0",
    ]:
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_FAIL_STDOUT", str(read_rows(FAIL)[0]))
    if read_rows(UNIT)[0].get("stdout") != [
        "endpoint_solve_range_report",
        "통과",
        "0",
        "-5@KRW",
        "-10@KRW",
        "0@KRW",
    ]:
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_UNIT_STDOUT", str(read_rows(UNIT)[0]))
    if read_rows(MISSING)[0].get("stdout") != [
        "endpoint_solve_range_report",
        "실패",
        "실패",
        "2",
        "누락",
        "실패",
        "missing_value",
        "누락",
        "범위없음",
        "0",
    ]:
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_MISSING_STDOUT", str(read_rows(MISSING)[0]))
    unsupported = read_rows(UNSUPPORTED)[0]
    if unsupported.get("expected_error_code") != "connect_boundary_range_unknown_path":
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_UNSUPPORTED_ROW", str(unsupported))
    if unsupported.get("exit_code") != 1:
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_UNSUPPORTED_EXIT", str(unsupported))
    if read_rows(CLOSURE)[0].get("stdout") != [CLOSURE, *BUNDLED]:
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_CLOSURE_STDOUT", str(read_rows(CLOSURE)[0]))
    return 0


def check_docs() -> int:
    text = (ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_REPORT_V1P.md").read_text(encoding="utf-8")
    required = [
        "이음관계.풀고범위행목록",
        "이음관계.풀고범위보고서",
        "endpoint_solve_range_report",
        "endpoint_solve_range_report_row",
        "missing_value",
        "unknown range path",
    ]
    for token in required:
        if token not in text:
            return fail("E_CONNECT_SOLVE_RANGE_REPORT_DOC", token)
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
        return fail("E_CONNECT_SOLVE_RANGE_REPORT_GOLDEN", str(result.returncode))
    return 0


def main() -> int:
    for check in (require_files, check_contracts, check_golden_rows, check_docs, run_golden):
        rc = check()
        if rc:
            return rc
    print("[connect-endpoint-solve-range-report-v1p] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
