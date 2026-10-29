#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PASS = "connect_endpoint_solve_range_text_report_pass_v1"
FAIL = "connect_endpoint_solve_range_text_report_fail_v1"
UNIT = "connect_endpoint_solve_range_text_report_unit_v1"
MISSING = "connect_endpoint_solve_range_text_report_missing_value_v1"
UNSUPPORTED = "connect_endpoint_solve_range_text_report_unsupported_v1"
CLOSURE = "connect_flow_v1q_closure_v1"
PACKS = [PASS, FAIL, UNIT, MISSING, UNSUPPORTED, CLOSURE]
BUNDLED = [
    "connect_flow_v1p_closure_v1",
    PASS,
    FAIL,
    UNIT,
    MISSING,
    UNSUPPORTED,
]
HEADER = "변수\t경로\t값상태\t값\t범위상태\t하한\t상한\t위반"


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


def stdout_text(pack: str) -> str:
    row = read_rows(pack)[0]
    return "\n".join(str(item) for item in row.get("stdout", []))


def require_files() -> int:
    required = [
        ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_TEXT_REPORT_V1Q.md",
        ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_REPORT_V1P.md",
        ROOT / "pack" / "connect_flow_v1p_closure_v1" / "contract.detjson",
        ROOT / "pack" / "connect_endpoint_solve_range_report_pass_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_endpoint_solve_range_report_missing_value_v1" / "golden.jsonl",
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
        return fail("E_CONNECT_TEXT_REPORT_MISSING", str(missing))
    return 0


def check_contracts() -> int:
    pass_contract = read_json(ROOT / "pack" / PASS / "contract.detjson")
    if pass_contract.get("text_header") != HEADER:
        return fail("E_CONNECT_TEXT_REPORT_HEADER_CONTRACT", str(pass_contract))
    if pass_contract.get("output_kind") != "endpoint_solve_range_text_report":
        return fail("E_CONNECT_TEXT_REPORT_PASS_KIND", str(pass_contract))

    fail_contract = read_json(ROOT / "pack" / FAIL / "contract.detjson")
    if fail_contract.get("expected_violation_reason") != "below_min":
        return fail("E_CONNECT_TEXT_REPORT_FAIL_REASON", str(fail_contract))

    unit_contract = read_json(ROOT / "pack" / UNIT / "contract.detjson")
    if unit_contract.get("expected_unit") != "KRW":
        return fail("E_CONNECT_TEXT_REPORT_UNIT", str(unit_contract))

    missing_contract = read_json(ROOT / "pack" / MISSING / "contract.detjson")
    if missing_contract.get("expected_violation_reason") != "missing_value":
        return fail("E_CONNECT_TEXT_REPORT_MISSING_REASON", str(missing_contract))

    unsupported = read_json(ROOT / "pack" / UNSUPPORTED / "contract.detjson")
    if unsupported.get("expected_error_code") != "connect_report_text_expected_solve_range_report":
        return fail("E_CONNECT_TEXT_REPORT_UNSUPPORTED", str(unsupported))

    closure = read_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure.get("bundled_packs") != BUNDLED:
        return fail("E_CONNECT_TEXT_REPORT_CLOSURE_BUNDLE", str(closure.get("bundled_packs")))
    return 0


def check_golden_rows() -> int:
    pass_text = stdout_text(PASS)
    for token in [
        HEADER,
        "ep_001\t전지.양극.전압\t값있음\t5\t범위없음\t\t\t",
        "ep_002\t전구.왼핀.전압\t값있음\t5\t통과\t0\t10\t",
    ]:
        if token not in pass_text:
            return fail("E_CONNECT_TEXT_REPORT_PASS_STDOUT", token)

    fail_text = stdout_text(FAIL)
    if "below_min" not in fail_text or "ep_002\t전구.왼핀.전류\t값있음\t-5\t실패\t0\t10\tbelow_min" not in fail_text:
        return fail("E_CONNECT_TEXT_REPORT_FAIL_STDOUT", fail_text)

    unit_text = stdout_text(UNIT)
    for token in ["5@KRW", "-5@KRW", "-10@KRW", "0@KRW"]:
        if token not in unit_text:
            return fail("E_CONNECT_TEXT_REPORT_UNIT_STDOUT", token)

    missing_text = stdout_text(MISSING)
    if "missing_value" not in missing_text:
        return fail("E_CONNECT_TEXT_REPORT_MISSING_VALUE", missing_text)
    if "ep_002\t전구.왼핀.전압\t누락\t\t범위없음\t\t\t" not in missing_text:
        return fail("E_CONNECT_TEXT_REPORT_MISSING_NO_RANGE", missing_text)

    unsupported = read_rows(UNSUPPORTED)[0]
    if unsupported.get("expected_error_code") != "connect_report_text_expected_solve_range_report":
        return fail("E_CONNECT_TEXT_REPORT_UNSUPPORTED_ROW", str(unsupported))
    if unsupported.get("exit_code") != 1:
        return fail("E_CONNECT_TEXT_REPORT_UNSUPPORTED_EXIT", str(unsupported))

    if read_rows(CLOSURE)[0].get("stdout") != [CLOSURE, *BUNDLED]:
        return fail("E_CONNECT_TEXT_REPORT_CLOSURE_STDOUT", str(read_rows(CLOSURE)[0]))
    return 0


def check_docs() -> int:
    text = (ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_TEXT_REPORT_V1Q.md").read_text(encoding="utf-8")
    for token in [
        "이음관계.보고서문자표",
        "이음관계.풀고범위문자표",
        "endpoint_solve_range_report",
        "변수\\t경로\\t값상태\\t값\\t범위상태\\t하한\\t상한\\t위반",
        "connect_report_text_expected_solve_range_report",
    ]:
        if token not in text:
            return fail("E_CONNECT_TEXT_REPORT_DOC", token)
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
        return fail("E_CONNECT_TEXT_REPORT_GOLDEN", str(result.returncode))
    return 0


def main() -> int:
    for check in (require_files, check_contracts, check_golden_rows, check_docs, run_golden):
        rc = check()
        if rc:
            return rc
    print("[connect-endpoint-solve-range-text-report-v1q] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
