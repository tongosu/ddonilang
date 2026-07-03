#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PASS = "connect_endpoint_solve_range_case_suite_summary_pass_v1"
MIXED = "connect_endpoint_solve_range_case_suite_summary_mixed_v1"
DIRECT = "connect_endpoint_solve_range_case_suite_summary_direct_v1"
UNSUPPORTED = "connect_endpoint_solve_range_case_suite_summary_unsupported_v1"
CLOSURE = "connect_flow_v1t_closure_v1"
PACKS = [PASS, MIXED, DIRECT, UNSUPPORTED, CLOSURE]
BUNDLED = [
    "connect_flow_v1s_closure_v1",
    PASS,
    MIXED,
    DIRECT,
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


def stdout_text(pack: str, index: int = 0) -> str:
    row = read_rows(pack)[index]
    return "\n".join(str(item) for item in row.get("stdout", []))


def require_files() -> int:
    required = [
        ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_CASE_SUITE_SUMMARY_V1T.md",
        ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_CASE_SUITE_DETAIL_REPORT_V1S.md",
        ROOT / "pack" / "connect_flow_v1s_closure_v1" / "contract.detjson",
        ROOT / "pack" / "connect_endpoint_solve_range_case_suite_detail_pass_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_endpoint_solve_range_case_suite_detail_mixed_v1" / "golden.jsonl",
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
        return fail("E_CONNECT_CASE_SUITE_SUMMARY_MISSING", str(missing))
    return 0


def check_contracts() -> int:
    pass_contract = read_json(ROOT / "pack" / PASS / "contract.detjson")
    if pass_contract.get("expected_kind") != "endpoint_solve_range_case_suite_summary":
        return fail("E_CONNECT_CASE_SUITE_SUMMARY_PASS_KIND", str(pass_contract))
    if pass_contract.get("expected_pass_cases") != ["voltage-pass", "flow-pass"]:
        return fail("E_CONNECT_CASE_SUITE_SUMMARY_PASS_CASES", str(pass_contract))

    mixed_contract = read_json(ROOT / "pack" / MIXED / "contract.detjson")
    if mixed_contract.get("expected_fail_actual_pass") != ["unexpected-success"]:
        return fail("E_CONNECT_CASE_SUITE_SUMMARY_UNEXPECTED_SUCCESS", str(mixed_contract))
    if mixed_contract.get("expected_pass_actual_fail") != ["unexpected-fail"]:
        return fail("E_CONNECT_CASE_SUITE_SUMMARY_UNEXPECTED_FAILURE", str(mixed_contract))

    direct_contract = read_json(ROOT / "pack" / DIRECT / "contract.detjson")
    if direct_contract.get("expected_parity_separator") != "---":
        return fail("E_CONNECT_CASE_SUITE_SUMMARY_DIRECT_CONTRACT", str(direct_contract))

    unsupported_contract = read_json(ROOT / "pack" / UNSUPPORTED / "contract.detjson")
    if unsupported_contract.get("expected_error_codes") != [
        "connect_case_suite_summary_expected_suite",
        "connect_case_suite_summary_malformed_case_result",
    ]:
        return fail("E_CONNECT_CASE_SUITE_SUMMARY_UNSUPPORTED_CONTRACT", str(unsupported_contract))

    closure = read_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure.get("bundled_packs") != BUNDLED:
        return fail("E_CONNECT_CASE_SUITE_SUMMARY_CLOSURE_BUNDLE", str(closure.get("bundled_packs")))
    return 0


def check_golden_rows() -> int:
    pass_text = stdout_text(PASS)
    for token in [
        "endpoint_solve_range_case_suite_summary",
        "2",
        "0",
        "참",
        "voltage-pass",
        "flow-pass",
        "---",
    ]:
        if token not in pass_text:
            return fail("E_CONNECT_CASE_SUITE_SUMMARY_PASS_STDOUT", token)
    before, _, after = pass_text.partition("\n---\n")
    if not before or "voltage-pass" not in after or "flow-pass" not in after:
        return fail("E_CONNECT_CASE_SUITE_SUMMARY_PASS_DIRECT_PARITY", pass_text)

    mixed_text = stdout_text(MIXED)
    for token in [
        "pass-default",
        "expected-fail",
        "unexpected-fail",
        "unexpected-success",
        "거짓",
    ]:
        if token not in mixed_text:
            return fail("E_CONNECT_CASE_SUITE_SUMMARY_MIXED_STDOUT", token)

    direct_text = stdout_text(DIRECT)
    direct_before, _, direct_after = direct_text.partition("\n---\n")
    if not direct_before or not direct_after:
        return fail("E_CONNECT_CASE_SUITE_SUMMARY_DIRECT_SEPARATOR", direct_text)
    for token in ["endpoint_solve_range_case_suite_summary", "direct-voltage", "참"]:
        if direct_before.count(token) != direct_after.count(token):
            return fail("E_CONNECT_CASE_SUITE_SUMMARY_DIRECT_PARITY", token)

    unsupported_rows = read_rows(UNSUPPORTED)
    expected_codes = [
        "connect_case_suite_summary_expected_suite",
        "connect_case_suite_summary_malformed_case_result",
    ]
    if [row.get("expected_error_code") for row in unsupported_rows] != expected_codes:
        return fail("E_CONNECT_CASE_SUITE_SUMMARY_UNSUPPORTED_ROWS", str(unsupported_rows))
    if any(row.get("exit_code") != 1 for row in unsupported_rows):
        return fail("E_CONNECT_CASE_SUITE_SUMMARY_UNSUPPORTED_EXIT", str(unsupported_rows))

    if read_rows(CLOSURE)[0].get("stdout") != [CLOSURE, *BUNDLED]:
        return fail("E_CONNECT_CASE_SUITE_SUMMARY_CLOSURE_STDOUT", str(read_rows(CLOSURE)[0]))
    return 0


def check_docs() -> int:
    text = (ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_CASE_SUITE_SUMMARY_V1T.md").read_text(
        encoding="utf-8"
    )
    for token in [
        "이음관계.풀고범위스위트요약",
        "이음관계.풀고범위실행요약",
        "endpoint_solve_range_case_suite_summary",
        "기대실패통과케이스들",
        "connect_case_suite_summary_expected_suite",
        "connect_case_suite_summary_malformed_case_result",
    ]:
        if token not in text:
            return fail("E_CONNECT_CASE_SUITE_SUMMARY_DOC", token)
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
        return fail("E_CONNECT_CASE_SUITE_SUMMARY_GOLDEN", str(result.returncode))
    return 0


def main() -> int:
    for check in (require_files, check_contracts, check_golden_rows, check_docs, run_golden):
        rc = check()
        if rc:
            return rc
    print("[connect-endpoint-solve-range-case-suite-summary-v1t] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
