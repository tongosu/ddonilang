#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PASS = "connect_endpoint_solve_range_case_suite_detail_pass_v1"
MIXED = "connect_endpoint_solve_range_case_suite_detail_mixed_v1"
UNIT = "connect_endpoint_solve_range_case_suite_detail_unit_v1"
UNSUPPORTED = "connect_endpoint_solve_range_case_suite_detail_unsupported_v1"
CLOSURE = "connect_flow_v1s_closure_v1"
PACKS = [PASS, MIXED, UNIT, UNSUPPORTED, CLOSURE]
BUNDLED = [
    "connect_flow_v1r_closure_v1",
    PASS,
    MIXED,
    UNIT,
    UNSUPPORTED,
]
SUMMARY_HEADER = "이름\t기대\t실제\t통과"
ROW_HEADER = "변수\t경로\t값상태\t값\t범위상태\t하한\t상한\t위반"


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


def stdout_text(pack: str, index: int = 0) -> str:
    row = read_rows(pack)[index]
    return "\n".join(str(item) for item in row.get("stdout", []))


def require_files() -> int:
    required = [
        ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_CASE_SUITE_DETAIL_REPORT_V1S.md",
        ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_CASE_SUITE_V1R.md",
        ROOT / "pack" / "connect_flow_v1r_closure_v1" / "contract.detjson",
        ROOT / "pack" / "connect_endpoint_solve_range_case_suite_pass_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_endpoint_solve_range_case_suite_mixed_v1" / "golden.jsonl",
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
    required.append(ROOT / "pack" / UNSUPPORTED / "malformed.ddn")
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_CONNECT_CASE_SUITE_DETAIL_MISSING", str(missing))
    return 0


def check_contracts() -> int:
    pass_contract = read_json(ROOT / "pack" / PASS / "contract.detjson")
    if pass_contract.get("section_separator") != "blank_line":
        return fail("E_CONNECT_CASE_SUITE_DETAIL_SEPARATOR", str(pass_contract))
    if pass_contract.get("summary_header") != SUMMARY_HEADER:
        return fail("E_CONNECT_CASE_SUITE_DETAIL_HEADER", str(pass_contract))
    if pass_contract.get("expected_case_sections") != ["## voltage-pass", "## flow-pass"]:
        return fail("E_CONNECT_CASE_SUITE_DETAIL_CASES", str(pass_contract))

    mixed_contract = read_json(ROOT / "pack" / MIXED / "contract.detjson")
    required_rows = mixed_contract.get("required_rows") or []
    if "unexpected-success\t실패\t통과\t거짓" not in required_rows:
        return fail("E_CONNECT_CASE_SUITE_DETAIL_MIXED_CONTRACT", str(required_rows))

    unit_contract = read_json(ROOT / "pack" / UNIT / "contract.detjson")
    if unit_contract.get("expected_unit") != "KRW":
        return fail("E_CONNECT_CASE_SUITE_DETAIL_UNIT_CONTRACT", str(unit_contract))

    unsupported_contract = read_json(ROOT / "pack" / UNSUPPORTED / "contract.detjson")
    if unsupported_contract.get("expected_error_codes") != [
        "connect_case_suite_detail_expected_suite",
        "connect_case_suite_detail_malformed_case_result",
    ]:
        return fail("E_CONNECT_CASE_SUITE_DETAIL_UNSUPPORTED_CONTRACT", str(unsupported_contract))

    closure = read_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure.get("bundled_packs") != BUNDLED:
        return fail("E_CONNECT_CASE_SUITE_DETAIL_CLOSURE_BUNDLE", str(closure.get("bundled_packs")))
    return 0


def check_golden_rows() -> int:
    pass_text = stdout_text(PASS)
    for token in [
        SUMMARY_HEADER,
        "voltage-pass\t통과\t통과\t참",
        "flow-pass\t통과\t통과\t참",
        "## voltage-pass\n기대\t통과\n실제\t통과\n통과\t참\n" + ROW_HEADER,
        "## flow-pass\n기대\t통과\n실제\t통과\n통과\t참\n" + ROW_HEADER,
        "ep_002\t전구.왼핀.전류\t값있음\t-5\t통과\t-10\t0\t",
        "---",
    ]:
        if token not in pass_text:
            return fail("E_CONNECT_CASE_SUITE_DETAIL_PASS_STDOUT", token)
    before, _, after = pass_text.partition("\n---\n")
    if not before or before != after:
        return fail("E_CONNECT_CASE_SUITE_DETAIL_DIRECT_PARITY", pass_text)

    mixed_text = stdout_text(MIXED)
    for token in [
        "expected-fail\t실패\t실패\t참",
        "unexpected-fail\t통과\t실패\t거짓",
        "unexpected-success\t실패\t통과\t거짓",
        "## expected-fail",
        "below_min",
    ]:
        if token not in mixed_text:
            return fail("E_CONNECT_CASE_SUITE_DETAIL_MIXED_STDOUT", token)

    unit_text = stdout_text(UNIT)
    for token in ["unit-flow\t통과\t통과\t참", "5@KRW", "-5@KRW", "-10@KRW", "0@KRW"]:
        if token not in unit_text:
            return fail("E_CONNECT_CASE_SUITE_DETAIL_UNIT_STDOUT", token)

    unsupported_rows = read_rows(UNSUPPORTED)
    expected_codes = [
        "connect_case_suite_detail_expected_suite",
        "connect_case_suite_detail_malformed_case_result",
    ]
    if [row.get("expected_error_code") for row in unsupported_rows] != expected_codes:
        return fail("E_CONNECT_CASE_SUITE_DETAIL_UNSUPPORTED_ROWS", str(unsupported_rows))
    if any(row.get("exit_code") != 1 for row in unsupported_rows):
        return fail("E_CONNECT_CASE_SUITE_DETAIL_UNSUPPORTED_EXIT", str(unsupported_rows))

    if read_rows(CLOSURE)[0].get("stdout") != [CLOSURE, *BUNDLED]:
        return fail("E_CONNECT_CASE_SUITE_DETAIL_CLOSURE_STDOUT", str(read_rows(CLOSURE)[0]))
    return 0


def check_docs() -> int:
    text = (ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_CASE_SUITE_DETAIL_REPORT_V1S.md").read_text(
        encoding="utf-8"
    )
    for token in [
        "이음관계.풀고범위스위트상세문자표",
        "이음관계.풀고범위실행상세문자표",
        "case section 사이도 빈 줄 1개",
        "connect_case_suite_detail_expected_suite",
        "connect_case_suite_detail_malformed_case_result",
    ]:
        if token not in text:
            return fail("E_CONNECT_CASE_SUITE_DETAIL_DOC", token)
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
        return fail("E_CONNECT_CASE_SUITE_DETAIL_GOLDEN", str(result.returncode))
    return 0


def main() -> int:
    for check in (require_files, check_contracts, check_golden_rows, check_docs, run_golden):
        rc = check()
        if rc:
            return rc
    print("[connect-endpoint-solve-range-case-suite-detail-v1s] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
