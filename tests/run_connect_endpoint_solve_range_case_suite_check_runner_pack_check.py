#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PASS = "connect_endpoint_solve_range_case_suite_check_runner_pass_v1"
FAIL = "connect_endpoint_solve_range_case_suite_check_runner_fail_v1"
UNSUPPORTED = "connect_endpoint_solve_range_case_suite_check_runner_unsupported_v1"
CLOSURE = "connect_flow_v1v_closure_v1"
PACKS = [PASS, FAIL, UNSUPPORTED, CLOSURE]
BUNDLED = [
    "connect_flow_v1u_closure_v1",
    PASS,
    FAIL,
    UNSUPPORTED,
]

RUNNER = "tests/connect_endpoint_solve_range_case_suite_check_runner.py"
EXPECTED_KIND = "endpoint_solve_range_case_suite_check"
EXPECTED_STDOUT_MARKER = "connect_case_suite_check_runner_expected_check_stdout"
TOOL_FAILED_MARKER = "connect_case_suite_check_runner_tool_failed"
INVALID_JUDGEMENT_MARKER = "connect_case_suite_check_runner_invalid_judgement"


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


def run_runner(input_path: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, RUNNER, input_path],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )


def require_files() -> int:
    required = [
        ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_CASE_SUITE_CHECK_RUNNER_V1V.md",
        ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_CASE_SUITE_CHECK_V1U.md",
        ROOT / RUNNER,
        ROOT / "pack" / "connect_flow_v1u_closure_v1" / "contract.detjson",
        ROOT / "pack" / "connect_endpoint_solve_range_case_suite_check_pass_v1" / "golden.jsonl",
        ROOT / "pack" / "connect_endpoint_solve_range_case_suite_check_fail_v1" / "golden.jsonl",
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
    required.extend(
        [
            ROOT / "pack" / UNSUPPORTED / "non_check.ddn",
            ROOT / "pack" / UNSUPPORTED / "invalid_judgement.ddn",
            ROOT / "pack" / UNSUPPORTED / "tool_failed.ddn",
        ]
    )
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_CONNECT_CASE_SUITE_CHECK_RUNNER_MISSING", str(missing))
    return 0


def check_contracts() -> int:
    pass_contract = read_json(ROOT / "pack" / PASS / "contract.detjson")
    if pass_contract.get("runner_expected_exit") != 0:
        return fail("E_CONNECT_CASE_SUITE_CHECK_RUNNER_PASS_EXIT", str(pass_contract))
    if pass_contract.get("expected_stdout_prefix") != [EXPECTED_KIND, "통과"]:
        return fail("E_CONNECT_CASE_SUITE_CHECK_RUNNER_PASS_PREFIX", str(pass_contract))

    fail_contract = read_json(ROOT / "pack" / FAIL / "contract.detjson")
    if fail_contract.get("runner_expected_exit") != 1:
        return fail("E_CONNECT_CASE_SUITE_CHECK_RUNNER_FAIL_EXIT", str(fail_contract))
    if fail_contract.get("expected_stdout_prefix") != [EXPECTED_KIND, "실패"]:
        return fail("E_CONNECT_CASE_SUITE_CHECK_RUNNER_FAIL_PREFIX", str(fail_contract))

    unsupported_contract = read_json(ROOT / "pack" / UNSUPPORTED / "contract.detjson")
    if unsupported_contract.get("runner_expected_errors") != [
        EXPECTED_STDOUT_MARKER,
        INVALID_JUDGEMENT_MARKER,
        TOOL_FAILED_MARKER,
    ]:
        return fail("E_CONNECT_CASE_SUITE_CHECK_RUNNER_UNSUPPORTED_CONTRACT", str(unsupported_contract))

    closure = read_json(ROOT / "pack" / CLOSURE / "contract.detjson")
    if closure.get("bundled_packs") != BUNDLED:
        return fail("E_CONNECT_CASE_SUITE_CHECK_RUNNER_CLOSURE_BUNDLE", str(closure.get("bundled_packs")))
    return 0


def check_golden_rows() -> int:
    pass_rows = read_rows(PASS)
    if pass_rows[0].get("stdout", [])[:2] != [EXPECTED_KIND, "통과"]:
        return fail("E_CONNECT_CASE_SUITE_CHECK_RUNNER_PASS_STDOUT", str(pass_rows[0]))
    if pass_rows[0].get("exit_code") != 0:
        return fail("E_CONNECT_CASE_SUITE_CHECK_RUNNER_PASS_GOLDEN_EXIT", str(pass_rows[0]))

    fail_rows = read_rows(FAIL)
    if fail_rows[0].get("stdout", [])[:2] != [EXPECTED_KIND, "실패"]:
        return fail("E_CONNECT_CASE_SUITE_CHECK_RUNNER_FAIL_STDOUT", str(fail_rows[0]))
    if fail_rows[0].get("exit_code") != 0:
        return fail("E_CONNECT_CASE_SUITE_CHECK_RUNNER_FAIL_GOLDEN_EXIT", str(fail_rows[0]))

    unsupported_rows = read_rows(UNSUPPORTED)
    if [row.get("stdout", [])[:2] for row in unsupported_rows[:2]] != [
        ["not_endpoint_solve_range_case_suite_check", "통과"],
        [EXPECTED_KIND, "보류"],
    ]:
        return fail("E_CONNECT_CASE_SUITE_CHECK_RUNNER_UNSUPPORTED_STDOUT", str(unsupported_rows))
    if unsupported_rows[2].get("expected_error_code") != "connect_case_suite_check_expected_summary":
        return fail("E_CONNECT_CASE_SUITE_CHECK_RUNNER_TOOL_FAIL_GOLDEN", str(unsupported_rows[2]))

    if read_rows(CLOSURE)[0].get("stdout") != [CLOSURE, *BUNDLED]:
        return fail("E_CONNECT_CASE_SUITE_CHECK_RUNNER_CLOSURE_STDOUT", str(read_rows(CLOSURE)[0]))
    return 0


def check_runner() -> int:
    cases = [
        (f"pack/{PASS}/input.ddn", 0, ""),
        (f"pack/{FAIL}/input.ddn", 1, ""),
        (f"pack/{UNSUPPORTED}/non_check.ddn", 2, EXPECTED_STDOUT_MARKER),
        (f"pack/{UNSUPPORTED}/invalid_judgement.ddn", 2, INVALID_JUDGEMENT_MARKER),
        (f"pack/{UNSUPPORTED}/tool_failed.ddn", 2, TOOL_FAILED_MARKER),
    ]
    for input_path, expected_exit, expected_marker in cases:
        result = run_runner(input_path)
        if result.returncode != expected_exit:
            return fail(
                "E_CONNECT_CASE_SUITE_CHECK_RUNNER_EXIT",
                f"{input_path}: expected {expected_exit}, got {result.returncode}\n{result.stdout}",
            )
        if expected_marker and expected_marker not in result.stdout:
            return fail("E_CONNECT_CASE_SUITE_CHECK_RUNNER_MARKER", f"{input_path}: {result.stdout}")
    return 0


def check_docs() -> int:
    text = (ROOT / "CONNECT_ENDPOINT_SOLVE_RANGE_CASE_SUITE_CHECK_RUNNER_V1V.md").read_text(
        encoding="utf-8"
    )
    for token in [
        "connect_endpoint_solve_range_case_suite_check_runner.py",
        "판정.__이음관계종류 보여주기",
        "판정.판정 보여주기",
        "connect_case_suite_check_runner_expected_check_stdout",
        "connect_case_suite_check_runner_tool_failed",
        "connect_case_suite_check_runner_invalid_judgement",
        "exit `1`",
        "exit `2`",
    ]:
        if token not in text:
            return fail("E_CONNECT_CASE_SUITE_CHECK_RUNNER_DOC", token)
    return 0


def run_golden() -> int:
    result = subprocess.run(
        [sys.executable, "tests/run_pack_golden.py", *PACKS],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode != 0:
        print(result.stdout)
        return fail("E_CONNECT_CASE_SUITE_CHECK_RUNNER_GOLDEN", str(result.returncode))
    return 0


def main() -> int:
    for check in (require_files, check_contracts, check_golden_rows, check_runner, check_docs, run_golden):
        rc = check()
        if rc:
            return rc
    print("[connect-endpoint-solve-range-case-suite-check-runner-v1v] OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
