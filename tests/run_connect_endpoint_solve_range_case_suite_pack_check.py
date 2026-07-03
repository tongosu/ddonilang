#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PASS = "connect_endpoint_solve_range_case_suite_pass_v1"
MIXED = "connect_endpoint_solve_range_case_suite_mixed_v1"
UNIT = "connect_endpoint_solve_range_case_suite_unit_v1"
TEXT = "connect_endpoint_solve_range_case_suite_text_v1"
UNSUPPORTED = "connect_endpoint_solve_range_case_suite_unsupported_v1"
CLOSURE = "connect_flow_v1r_closure_v1"
PACKS = [PASS, MIXED, UNIT, TEXT, UNSUPPORTED, CLOSURE]
BUNDLED = [
    "connect_flow_v1q_closure_v1",
    PASS,
    MIXED,
    UNIT,
    TEXT,
    UNSUPPORTED,
]
HEADER = "이름\t기대\t실제\t통과"


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
        ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_CASE_SUITE_V1R.md",
        ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_TEXT_REPORT_V1Q.md",
        ROOT / "pack" / "connect_flow_v1q_closure_v1" / "contract.detjson",
        ROOT / "pack" / "connect_endpoint_solve_range_text_report_pass_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_endpoint_solve_range_text_report_fail_v1" / "golden.jsonl",
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
        return fail("E_CONNECT_CASE_SUITE_MISSING", str(missing))
    return 0


def check_contracts() -> int:
    pass_contract = read_json(ROOT / "pack" / PASS / "contract.detjson")
    if pass_contract.get("expected_overall_pass") is not True:
        return fail("E_CONNECT_CASE_SUITE_PASS_CONTRACT", str(pass_contract))
    if pass_contract.get("expected_pass_count") != 2:
        return fail("E_CONNECT_CASE_SUITE_PASS_COUNT", str(pass_contract))

    mixed_contract = read_json(ROOT / "pack" / MIXED / "contract.detjson")
    if mixed_contract.get("expected_overall_pass") is not False:
        return fail("E_CONNECT_CASE_SUITE_MIXED_CONTRACT", str(mixed_contract))
    required_rows = mixed_contract.get("required_rows") or []
    if "unexpected-success\t실패\t통과\t거짓" not in required_rows:
        return fail("E_CONNECT_CASE_SUITE_UNEXPECTED_SUCCESS_CONTRACT", str(required_rows))

    unit_contract = read_json(ROOT / "pack" / UNIT / "contract.detjson")
    if unit_contract.get("expected_unit") != "KRW":
        return fail("E_CONNECT_CASE_SUITE_UNIT_CONTRACT", str(unit_contract))

    text_contract = read_json(ROOT / "pack" / TEXT / "contract.detjson")
    if text_contract.get("text_header") != HEADER:
        return fail("E_CONNECT_CASE_SUITE_TEXT_HEADER", str(text_contract))

    unsupported = read_json(ROOT / "pack" / UNSUPPORTED / "contract.detjson")
    if unsupported.get("expected_error_code") != "connect_case_suite_invalid_expected_result":
        return fail("E_CONNECT_CASE_SUITE_UNSUPPORTED_CONTRACT", str(unsupported))

    closure = read_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure.get("bundled_packs") != BUNDLED:
        return fail("E_CONNECT_CASE_SUITE_CLOSURE_BUNDLE", str(closure.get("bundled_packs")))
    return 0


def check_golden_rows() -> int:
    pass_text = stdout_text(PASS)
    for token in [
        "endpoint_solve_range_case_suite",
        "참",
        "2",
        "0",
        "voltage-pass\t통과\t통과\t참",
        "flow-pass\t통과\t통과\t참",
    ]:
        if token not in pass_text:
            return fail("E_CONNECT_CASE_SUITE_PASS_STDOUT", token)

    mixed_text = stdout_text(MIXED)
    for token in [
        "pass-default\t통과\t통과\t참",
        "expected-fail\t실패\t실패\t참",
        "unexpected-fail\t통과\t실패\t거짓",
        "unexpected-success\t실패\t통과\t거짓",
    ]:
        if token not in mixed_text:
            return fail("E_CONNECT_CASE_SUITE_MIXED_STDOUT", token)
    if "거짓" not in mixed_text:
        return fail("E_CONNECT_CASE_SUITE_MIXED_OVERALL", mixed_text)

    unit_text = stdout_text(UNIT)
    for token in ["5@KRW", "-5@KRW", "-10@KRW", "0@KRW", "unit-flow\t통과\t통과\t참"]:
        if token not in unit_text:
            return fail("E_CONNECT_CASE_SUITE_UNIT_STDOUT", token)

    text_text = stdout_text(TEXT)
    if HEADER not in text_text or "text-pass\t통과\t통과\t참" not in text_text:
        return fail("E_CONNECT_CASE_SUITE_TEXT_STDOUT", text_text)

    unsupported = read_rows(UNSUPPORTED)[0]
    if unsupported.get("expected_error_code") != "connect_case_suite_invalid_expected_result":
        return fail("E_CONNECT_CASE_SUITE_UNSUPPORTED_ROW", str(unsupported))
    if unsupported.get("exit_code") != 1:
        return fail("E_CONNECT_CASE_SUITE_UNSUPPORTED_EXIT", str(unsupported))

    if read_rows(CLOSURE)[0].get("stdout") != [CLOSURE, *BUNDLED]:
        return fail("E_CONNECT_CASE_SUITE_CLOSURE_STDOUT", str(read_rows(CLOSURE)[0]))
    return 0


def check_docs() -> int:
    text = (ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_CASE_SUITE_V1R.md").read_text(encoding="utf-8")
    for token in [
        "이음관계.풀고범위케이스",
        "이음관계.풀고범위스위트",
        "이음관계.풀고범위스위트문자표",
        "endpoint_solve_range_case_suite",
        "이름\\t기대\\t실제\\t통과",
        "connect_case_suite_invalid_expected_result",
    ]:
        if token not in text:
            return fail("E_CONNECT_CASE_SUITE_DOC", token)
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
        return fail("E_CONNECT_CASE_SUITE_GOLDEN", str(result.returncode))
    return 0


def main() -> int:
    for check in (require_files, check_contracts, check_golden_rows, check_docs, run_golden):
        rc = check()
        if rc:
            return rc
    print("[connect-endpoint-solve-range-case-suite-v1r] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
