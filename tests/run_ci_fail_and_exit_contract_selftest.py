#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from _ci_aggregate_gate_mock_harness import build_step_map, run_aggregate_gate_with_mock_failure
from _ci_fail_and_exit_contract import (
    FAIL_AND_EXIT_BLOCK_FORBIDDEN_TOKENS,
    FAIL_AND_EXIT_BLOCK_REQUIRED_TOKENS,
    validate_fail_and_exit_block_contract,
)
from _ci_latest_smoke_contract import (
    LATEST_SMOKE_SKIP_REASON_CI_GATE_RESULT_STATUS_NOT_PASS,
    LATEST_SMOKE_SKIP_REASON_FAST_FAIL_PATH,
    LATEST_SMOKE_SKIP_REASON_FLAG_DISABLED,
    LATEST_SMOKE_SKIP_REASON_PENDING_FAILURE_SUMMARY_REGENERATION,
)


def build_valid_gate_text() -> str:
    required_lines = "\n        ".join(FAIL_AND_EXIT_BLOCK_REQUIRED_TOKENS)
    return (
        "def fail_and_exit(exit_code: int, message: str) -> int:\n"
        "    if True:\n"
        f"        {required_lines}\n"
        "    return exit_code\n\n"
        "if args.contract_only_aggregate:\n"
        "    pass\n"
    )


def main() -> int:
    valid_text = build_valid_gate_text()
    valid_issues = validate_fail_and_exit_block_contract(valid_text)
    if valid_issues:
        print("fail_and_exit contract selftest failed: valid case produced issues")
        for issue in valid_issues[:8]:
            print(f" - {issue}")
        return 1

    missing_required_text = valid_text.replace(FAIL_AND_EXIT_BLOCK_REQUIRED_TOKENS[-1], "")
    missing_required_issues = validate_fail_and_exit_block_contract(missing_required_text)
    if not any("missing required token in fail_and_exit:" in issue for issue in missing_required_issues):
        print("fail_and_exit contract selftest failed: missing-required case was not detected")
        return 1

    forbidden_text = valid_text.replace(
        "    return exit_code\n\n",
        f"    {FAIL_AND_EXIT_BLOCK_FORBIDDEN_TOKENS[0]}\n    return exit_code\n\n",
    )
    forbidden_issues = validate_fail_and_exit_block_contract(forbidden_text)
    if not any("forbidden direct rerun token in fail_and_exit:" in issue for issue in forbidden_issues):
        print("fail_and_exit contract selftest failed: forbidden-token case was not detected")
        return 1

    missing_boundary_issues = validate_fail_and_exit_block_contract("def unrelated():\n    return 0\n")
    if "fail_and_exit block boundary not found" not in missing_boundary_issues:
        print("fail_and_exit contract selftest failed: boundary-missing case was not detected")
        return 1

    with tempfile.TemporaryDirectory(prefix="ci_fail_and_exit_contract_skip_selftest_") as td:
        report_dir = Path(td) / "reports"
        report_dir.mkdir(parents=True, exist_ok=True)
        report_prefix = "ci_fail_and_exit_contract_skip_case"
        aggregate_cmd = [
            sys.executable,
            "tests/run_ci_aggregate_gate.py",
            "--contract-only-aggregate",
            "--ci-sanity-profile",
            "core_lang",
            "--fast-fail",
            "--skip-fail-and-exit-contract-selftest",
            "--skip-5min-checklist",
            "--compact-step-logs",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            report_prefix,
        ]
        aggregate_proc = subprocess.run(
            aggregate_cmd,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if aggregate_proc.returncode != 0:
            print(
                "fail_and_exit contract selftest failed: skip-flag aggregate run failed "
                f"rc={aggregate_proc.returncode}"
            )
            return 1

        index_path = report_dir / f"{report_prefix}.ci_gate_report_index.detjson"
        if not index_path.exists():
            print("fail_and_exit contract selftest failed: skip-flag index report missing")
            return 1
        index_doc = json.loads(index_path.read_text(encoding="utf-8"))
        steps = index_doc.get("steps")
        if not isinstance(steps, list):
            print("fail_and_exit contract selftest failed: skip-flag index steps missing")
            return 1
        skip_step = None
        for row in steps:
            if isinstance(row, dict) and str(row.get("name", "")).strip() == "ci_fail_and_exit_contract_selftest":
                skip_step = row
                break
        if not isinstance(skip_step, dict):
            print("fail_and_exit contract selftest failed: skip-flag step record missing")
            return 1
        if int(skip_step.get("returncode", 127)) != 0 or not bool(skip_step.get("ok", False)):
            print("fail_and_exit contract selftest failed: skip-flag step status mismatch")
            return 1
        cmd_tokens = [str(part).strip() for part in skip_step.get("cmd", []) if str(part).strip()]
        cmd_text = " ".join(cmd_tokens)
        is_contract_only_stub = cmd_tokens == ["contract-only", "ci_fail_and_exit_contract_selftest"]
        if not is_contract_only_stub and (
            f"ci_fail_and_exit_contract_selftest: skipped reason={LATEST_SMOKE_SKIP_REASON_FLAG_DISABLED}" not in cmd_text
        ):
            print("fail_and_exit contract selftest failed: skip-flag marker/stub command missing")
            return 1
        if any("tests/run_ci_fail_and_exit_contract_selftest.py" in token for token in cmd_tokens):
            print("fail_and_exit contract selftest failed: skip-flag command must not run real selftest script")
            return 1

        report_index_check_cmd = [
            sys.executable,
            "tests/run_ci_gate_report_index_check.py",
            "--index",
            str(index_path),
            "--sanity-profile",
            "core_lang",
            "--enforce-profile-step-contract",
        ]
        report_index_check_proc = subprocess.run(
            report_index_check_cmd,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if report_index_check_proc.returncode != 0:
            print(
                "fail_and_exit contract selftest failed: skip-flag report-index contract check failed "
                f"rc={report_index_check_proc.returncode}"
            )
            return 1

    with tempfile.TemporaryDirectory(prefix="ci_latest_smoke_fast_fail_mock_") as td:
        report_dir = Path(td) / "reports"
        report_dir.mkdir(parents=True, exist_ok=True)
        report_prefix = "ci_latest_smoke_fast_fail_skip_case"
        fast_fail_smoke_result = run_aggregate_gate_with_mock_failure(
            report_dir=report_dir,
            report_prefix=report_prefix,
            gate_args=[
                "--fast-fail",
                "--run-report-index-latest-smoke",
                "--skip-5min-checklist",
                "--compact-step-logs",
                "--report-dir",
                str(report_dir),
                "--report-prefix",
                report_prefix,
            ],
            fail_step_name="seamgrim_ci_gate",
            fail_returncode=17,
            observed_step_name="ci_gate_report_index_latest_smoke_check",
        )
        if int(fast_fail_smoke_result.get("returncode", 0)) == 0:
            print("fail_and_exit contract selftest failed: latest smoke fast-fail case expected non-zero exit")
            return 1
        fast_fail_cmd_tokens = [
            str(part).strip() for part in fast_fail_smoke_result.get("observed_cmd", []) if str(part).strip()
        ]
        fast_fail_cmd_text = " ".join(fast_fail_cmd_tokens)
        if (
            f"ci_gate_report_index_latest_smoke_check: skipped reason={LATEST_SMOKE_SKIP_REASON_FAST_FAIL_PATH}"
            not in fast_fail_cmd_text
        ):
            print("fail_and_exit contract selftest failed: latest smoke fast-fail skip marker missing")
            return 1
        if any("tests/run_ci_gate_report_index_latest_smoke_check.py" in token for token in fast_fail_cmd_tokens):
            print("fail_and_exit contract selftest failed: fast-fail path must not run latest smoke script")
            return 1
        fast_fail_step_map = build_step_map(
            fast_fail_smoke_result["index_doc"] if isinstance(fast_fail_smoke_result.get("index_doc"), dict) else {}
        )
        fast_fail_smoke_row = fast_fail_step_map.get("ci_gate_report_index_latest_smoke_check")
        if not isinstance(fast_fail_smoke_row, dict):
            print("fail_and_exit contract selftest failed: latest smoke fast-fail step row missing")
            return 1
        if int(fast_fail_smoke_row.get("returncode", 127)) != 0 or not bool(fast_fail_smoke_row.get("ok", False)):
            print("fail_and_exit contract selftest failed: latest smoke fast-fail step status mismatch")
            return 1

    with tempfile.TemporaryDirectory(prefix="ci_latest_smoke_flag_disabled_mock_") as td:
        report_dir = Path(td) / "reports"
        report_dir.mkdir(parents=True, exist_ok=True)
        report_prefix = "ci_latest_smoke_flag_disabled_case"
        flag_disabled_result = run_aggregate_gate_with_mock_failure(
            report_dir=report_dir,
            report_prefix=report_prefix,
            gate_args=[
                "--skip-5min-checklist",
                "--compact-step-logs",
                "--report-dir",
                str(report_dir),
                "--report-prefix",
                report_prefix,
            ],
            fail_step_name="__no_fail_step__",
            fail_returncode=0,
            observed_step_name="ci_gate_report_index_latest_smoke_check",
        )
        flag_disabled_cmd_tokens = [
            str(part).strip() for part in flag_disabled_result.get("observed_cmd", []) if str(part).strip()
        ]
        flag_disabled_cmd_text = " ".join(flag_disabled_cmd_tokens)
        if (
            f"ci_gate_report_index_latest_smoke_check: skipped reason={LATEST_SMOKE_SKIP_REASON_FLAG_DISABLED}"
            not in flag_disabled_cmd_text
        ):
            print("fail_and_exit contract selftest failed: latest smoke flag-disabled skip marker missing")
            return 1
        if any("tests/run_ci_gate_report_index_latest_smoke_check.py" in token for token in flag_disabled_cmd_tokens):
            print("fail_and_exit contract selftest failed: flag-disabled path must not run latest smoke script")
            return 1
        flag_disabled_step_map = build_step_map(
            flag_disabled_result["index_doc"] if isinstance(flag_disabled_result.get("index_doc"), dict) else {}
        )
        flag_disabled_row = flag_disabled_step_map.get("ci_gate_report_index_latest_smoke_check")
        if not isinstance(flag_disabled_row, dict):
            print("fail_and_exit contract selftest failed: latest smoke flag-disabled step row missing")
            return 1
        if int(flag_disabled_row.get("returncode", 127)) != 0 or not bool(flag_disabled_row.get("ok", False)):
            print("fail_and_exit contract selftest failed: latest smoke flag-disabled step status mismatch")
            return 1

    with tempfile.TemporaryDirectory(prefix="ci_latest_smoke_pending_regen_mock_") as td:
        report_dir = Path(td) / "reports"
        report_dir.mkdir(parents=True, exist_ok=True)
        report_prefix = "ci_latest_smoke_pending_regen_case"
        pending_regen_result = run_aggregate_gate_with_mock_failure(
            report_dir=report_dir,
            report_prefix=report_prefix,
            gate_args=[
                "--run-report-index-latest-smoke",
                "--skip-5min-checklist",
                "--compact-step-logs",
                "--report-dir",
                str(report_dir),
                "--report-prefix",
                report_prefix,
            ],
            fail_step_name="seamgrim_ci_gate",
            fail_returncode=17,
            observed_step_name="ci_gate_report_index_latest_smoke_check",
        )
        pending_regen_cmd_tokens = [
            str(part).strip() for part in pending_regen_result.get("observed_cmd", []) if str(part).strip()
        ]
        pending_regen_cmd_text = " ".join(pending_regen_cmd_tokens)
        if (
            "ci_gate_report_index_latest_smoke_check: skipped reason="
            f"{LATEST_SMOKE_SKIP_REASON_PENDING_FAILURE_SUMMARY_REGENERATION}"
            not in pending_regen_cmd_text
        ):
            print("fail_and_exit contract selftest failed: latest smoke pending-regeneration skip marker missing")
            return 1
        if any("tests/run_ci_gate_report_index_latest_smoke_check.py" in token for token in pending_regen_cmd_tokens):
            print("fail_and_exit contract selftest failed: pending-regeneration path must not run latest smoke script")
            return 1
        pending_regen_step_map = build_step_map(
            pending_regen_result["index_doc"] if isinstance(pending_regen_result.get("index_doc"), dict) else {}
        )
        pending_regen_row = pending_regen_step_map.get("ci_gate_report_index_latest_smoke_check")
        if not isinstance(pending_regen_row, dict):
            print("fail_and_exit contract selftest failed: latest smoke pending-regeneration step row missing")
            return 1
        if int(pending_regen_row.get("returncode", 127)) != 0 or not bool(pending_regen_row.get("ok", False)):
            print("fail_and_exit contract selftest failed: latest smoke pending-regeneration step status mismatch")
            return 1

    with tempfile.TemporaryDirectory(prefix="ci_latest_smoke_not_pass_mock_") as td:
        report_dir = Path(td) / "reports"
        report_dir.mkdir(parents=True, exist_ok=True)
        report_prefix = "ci_latest_smoke_not_pass_case"
        precreated_result_path = report_dir / f"{report_prefix}.ci_gate_result.detjson"
        precreated_result_path.write_text('{"status":"fail"}\n', encoding="utf-8")
        not_pass_result = run_aggregate_gate_with_mock_failure(
            report_dir=report_dir,
            report_prefix=report_prefix,
            gate_args=[
                "--run-report-index-latest-smoke",
                "--skip-5min-checklist",
                "--compact-step-logs",
                "--report-dir",
                str(report_dir),
                "--report-prefix",
                report_prefix,
            ],
            fail_step_name="__no_fail_step__",
            fail_returncode=0,
            observed_step_name="ci_gate_report_index_latest_smoke_check",
        )
        not_pass_cmd_tokens = [str(part).strip() for part in not_pass_result.get("observed_cmd", []) if str(part).strip()]
        not_pass_cmd_text = " ".join(not_pass_cmd_tokens)
        if (
            "ci_gate_report_index_latest_smoke_check: skipped reason="
            f"{LATEST_SMOKE_SKIP_REASON_CI_GATE_RESULT_STATUS_NOT_PASS}"
            not in not_pass_cmd_text
        ):
            print("fail_and_exit contract selftest failed: latest smoke not-pass skip marker missing")
            return 1
        if any("tests/run_ci_gate_report_index_latest_smoke_check.py" in token for token in not_pass_cmd_tokens):
            print("fail_and_exit contract selftest failed: not-pass path must not run latest smoke script")
            return 1
        not_pass_step_map = build_step_map(
            not_pass_result["index_doc"] if isinstance(not_pass_result.get("index_doc"), dict) else {}
        )
        not_pass_row = not_pass_step_map.get("ci_gate_report_index_latest_smoke_check")
        if not isinstance(not_pass_row, dict):
            print("fail_and_exit contract selftest failed: latest smoke not-pass step row missing")
            return 1
        if int(not_pass_row.get("returncode", 127)) != 0 or not bool(not_pass_row.get("ok", False)):
            print("fail_and_exit contract selftest failed: latest smoke not-pass step status mismatch")
            return 1

    with tempfile.TemporaryDirectory(prefix="ci_latest_smoke_execute_mock_") as td:
        report_dir = Path(td) / "reports"
        report_dir.mkdir(parents=True, exist_ok=True)
        report_prefix = "ci_latest_smoke_execute_case"
        precreated_result_path = report_dir / f"{report_prefix}.ci_gate_result.detjson"
        precreated_result_path.write_text('{"status":"pass"}\n', encoding="utf-8")
        execute_smoke_result = run_aggregate_gate_with_mock_failure(
            report_dir=report_dir,
            report_prefix=report_prefix,
            gate_args=[
                "--run-report-index-latest-smoke",
                "--skip-5min-checklist",
                "--compact-step-logs",
                "--report-dir",
                str(report_dir),
                "--report-prefix",
                report_prefix,
            ],
            fail_step_name="__no_fail_step__",
            fail_returncode=0,
            observed_step_name="ci_gate_report_index_latest_smoke_check",
        )
        if int(execute_smoke_result.get("returncode", 1)) != 0:
            print("fail_and_exit contract selftest failed: latest smoke execute case expected zero exit")
            return 1
        execute_cmd_tokens = [str(part).strip() for part in execute_smoke_result.get("observed_cmd", []) if str(part).strip()]
        if not any("tests/run_ci_gate_report_index_latest_smoke_check.py" in token for token in execute_cmd_tokens):
            print("fail_and_exit contract selftest failed: latest smoke execute command missing script path")
            return 1
        execute_step_map = build_step_map(
            execute_smoke_result["index_doc"] if isinstance(execute_smoke_result.get("index_doc"), dict) else {}
        )
        execute_smoke_row = execute_step_map.get("ci_gate_report_index_latest_smoke_check")
        if not isinstance(execute_smoke_row, dict):
            print("fail_and_exit contract selftest failed: latest smoke execute step row missing")
            return 1
        if int(execute_smoke_row.get("returncode", 127)) != 0 or not bool(execute_smoke_row.get("ok", False)):
            print("fail_and_exit contract selftest failed: latest smoke execute step status mismatch")
            return 1

    with tempfile.TemporaryDirectory(prefix="ci_fail_and_exit_contract_non_contract_mock_") as td:
        report_dir = Path(td) / "reports"
        report_dir.mkdir(parents=True, exist_ok=True)
        report_prefix = "ci_fail_and_exit_contract_skip_non_contract_mock"
        mock_result = run_aggregate_gate_with_mock_failure(
            report_dir=report_dir,
            report_prefix=report_prefix,
            gate_args=[
                "--fast-fail",
                "--skip-fail-and-exit-contract-selftest",
                "--skip-5min-checklist",
                "--compact-step-logs",
                "--report-dir",
                str(report_dir),
                "--report-prefix",
                report_prefix,
            ],
            fail_step_name="seamgrim_ci_gate",
            fail_returncode=17,
            observed_step_name="ci_fail_and_exit_contract_selftest",
        )

        mock_rc = int(mock_result.get("returncode", 0))
        if mock_rc == 0:
            print("fail_and_exit contract selftest failed: mock non-contract case expected non-zero exit")
            return 1

        skip_cmd_tokens = [
            str(part).strip() for part in mock_result.get("observed_cmd", []) if str(part).strip()
        ]
        if (
            f"ci_fail_and_exit_contract_selftest: skipped reason={LATEST_SMOKE_SKIP_REASON_FLAG_DISABLED}"
            not in " ".join(skip_cmd_tokens)
        ):
            print("fail_and_exit contract selftest failed: mock non-contract skip marker command missing")
            return 1
        if any("tests/run_ci_fail_and_exit_contract_selftest.py" in token for token in skip_cmd_tokens):
            print("fail_and_exit contract selftest failed: mock non-contract run must skip real selftest script")
            return 1

        index_path = Path(str(mock_result.get("index_path", "")).strip())
        if not index_path.exists():
            print("fail_and_exit contract selftest failed: mock non-contract index report missing")
            return 1
        index_doc = mock_result.get("index_doc")
        if not isinstance(index_doc, dict):
            print("fail_and_exit contract selftest failed: mock non-contract index payload missing")
            return 1
        step_map = build_step_map(index_doc)

        seamgrim_row = step_map.get("seamgrim_ci_gate")
        if not isinstance(seamgrim_row, dict):
            print("fail_and_exit contract selftest failed: mock non-contract seamgrim_ci_gate step missing")
            return 1
        if int(seamgrim_row.get("returncode", 0)) != 17 or bool(seamgrim_row.get("ok", True)):
            print("fail_and_exit contract selftest failed: mock non-contract seamgrim_ci_gate fail state mismatch")
            return 1

        skip_row = step_map.get("ci_fail_and_exit_contract_selftest")
        if not isinstance(skip_row, dict):
            print(
                "fail_and_exit contract selftest failed: mock non-contract ci_fail_and_exit_contract_selftest step missing"
            )
            return 1
        if int(skip_row.get("returncode", 127)) != 0 or not bool(skip_row.get("ok", False)):
            print(
                "fail_and_exit contract selftest failed: mock non-contract ci_fail_and_exit_contract_selftest status mismatch"
            )
            return 1

    print("fail_and_exit contract selftest ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
