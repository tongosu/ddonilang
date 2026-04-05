#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path

from _ci_profile_matrix_selftest_lib import (
    PROFILE_MATRIX_SELFTEST_PROFILES,
    PROFILE_MATRIX_SELFTEST_LIGHTWEIGHT_FULL_REAL_ENV_KEY,
    build_lightweight_profile_gate_lines,
    build_profile_matrix_selftest_fixture,
    expected_profile_matrix_summary_values,
    parse_profile_matrix_selftest_real_profiles,
    validate_profile_matrix_aggregate_summary,
)
from _ci_profile_matrix_full_real_smoke_contract import (
    PROFILE_MATRIX_GATE_SELFTEST_FULL_REAL_FLAG,
    PROFILE_MATRIX_GATE_SELFTEST_LIGHTWEIGHT_FALSE_MARKER,
    PROFILE_MATRIX_GATE_SELFTEST_OK_MARKER,
    PROFILE_MATRIX_GATE_SELFTEST_REAL_PROFILES_FLAG,
    PROFILE_MATRIX_GATE_SELFTEST_REAL_PROFILES_MARKER,
    PROFILE_MATRIX_GATE_SELFTEST_SKIPPED_REAL_PROFILES_MARKER,
)
from ci_check_error_codes import CI_PROFILE_MATRIX_CODES as CODES


MATRIX_SCHEMA = "ddn.ci.profile_matrix_gate.v1"
MATRIX_QUICK_ENV_KEY = "DDN_CI_PROFILE_MATRIX_QUICK_GATES"
MATRIX_WARN_QUICK_ENV_INVALID = "W_CI_PROFILE_MATRIX_QUICK_ENV_INVALID"
VALID_REAL_PROFILES = PROFILE_MATRIX_SELFTEST_PROFILES


def run(cmd: list[str], cwd: Path, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def write_lightweight_profile_gate_override(path: Path, profile_name: str) -> None:
    quick_lines = build_lightweight_profile_gate_lines(profile_name, quick=True)
    full_lines = build_lightweight_profile_gate_lines(profile_name, quick=False)
    script = """#!/usr/bin/env python
from __future__ import annotations

import argparse


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--quick", action="store_true")
    parser.add_argument("--report-dir", default="")
    parser.add_argument("--report-prefix", default="")
    parser.add_argument("--json-out", default="")
    args, _unknown = parser.parse_known_args()
    lines = {quick_lines} if args.quick else {full_lines}
    for line in lines:
        print(line)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
""".format(
        quick_lines=json.dumps(quick_lines, ensure_ascii=False),
        full_lines=json.dumps(full_lines, ensure_ascii=False),
    )
    path.write_text(script, encoding="utf-8")


def expect(cond: bool, msg: str, proc: subprocess.CompletedProcess[str] | None = None) -> int:
    if cond:
        return 0
    print(f"check=ci_profile_matrix_gate_selftest detail={msg}")
    if proc is not None:
        if (proc.stdout or "").strip():
            print(proc.stdout.strip())
        if (proc.stderr or "").strip():
            print(proc.stderr.strip())
    return 1


def expect_marker(proc: subprocess.CompletedProcess[str], token: str, msg: str) -> int:
    return expect(token in (proc.stdout or ""), msg, proc)


def expect_aggregate_summary_sanity(
    summary: object,
    profile: str,
    expected_values: dict[str, str],
    expected_present: bool,
    expected_gate_marker: bool,
    prefix: str,
) -> int:
    issue = validate_profile_matrix_aggregate_summary(
        summary,
        profile=profile,
        expected_values=expected_values,
        expected_present=expected_present,
        expected_gate_marker=expected_gate_marker,
    )
    if issue is None:
        return 0
    return expect(False, f"{prefix}_{issue}")


def resolve_negative_value(expected: str) -> str:
    if expected == "1":
        return "0"
    if expected == "0":
        return "1"
    if expected == "na":
        return "1"
    return "__forced_mismatch__"


def expect_summary_key_negative_cases(*, key: str, label: str) -> int:
    expected_issue = f"aggregate_summary_{key}_mismatch"
    for profile in PROFILE_MATRIX_SELFTEST_PROFILES:
        expected_values = expected_profile_matrix_summary_values(profile)
        broken_values = dict(expected_values)
        broken_values[key] = resolve_negative_value(str(expected_values.get(key, "")))
        summary = {
            "expected_present": True,
            "present": True,
            "status": "pass",
            "ok": True,
            "profile": profile,
            "sync_profile": profile,
            "profile_ok": True,
            "sync_profile_ok": True,
            "values_ok": True,
            "missing_keys": [],
            "mismatched_keys": [],
            "gate_marker_expected": False,
            "gate_marker_ok": True,
            "gate_marker_present": False,
            "values": broken_values,
        }
        issue = validate_profile_matrix_aggregate_summary(
            summary,
            profile=profile,
            expected_values=expected_values,
            expected_present=True,
            expected_gate_marker=False,
        )
        if expect(issue == expected_issue, f"{label}_negative_issue_mismatch:{profile}:{issue}") != 0:
            return 1
    return 0


def expect_sync_graph_export_negative_cases() -> int:
    return expect_summary_key_negative_cases(
        key="ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok",
        label="sync_graph_export",
    )


def expect_sync_numeric_factor_policy_negative_cases() -> int:
    return expect_summary_key_negative_cases(
        key="ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok",
        label="sync_numeric_factor_policy",
    )


def expect_sanity_numeric_factor_policy_negative_cases() -> int:
    return expect_summary_key_negative_cases(
        key="ci_sanity_seamgrim_numeric_factor_policy_ok",
        label="sanity_numeric_factor_policy",
    )


def run_matrix(
    py: str,
    root: Path,
    report_path: Path,
    profiles: str,
    dry_run: bool,
    stop_on_fail: bool = False,
    quick_gates: bool = False,
    full_aggregate_gates: bool = False,
    with_profile_matrix_full_real_smoke: bool = False,
    step_timeout_sec: float = 0.0,
    env: dict[str, str] | None = None,
    profile_gate_overrides: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    cmd = [
        py,
        "tests/run_ci_profile_matrix_gate.py",
        "--profiles",
        profiles,
        "--report-dir",
        str(report_path.parent),
        "--report-prefix",
        "profile_matrix_selftest",
        "--json-out",
        str(report_path),
    ]
    if dry_run:
        cmd.append("--dry-run")
    if stop_on_fail:
        cmd.append("--stop-on-fail")
    if quick_gates:
        cmd.append("--quick-gates")
    if full_aggregate_gates:
        cmd.append("--full-aggregate-gates")
    if with_profile_matrix_full_real_smoke:
        cmd.append("--with-profile-matrix-full-real-smoke")
    if float(step_timeout_sec) > 0.0:
        cmd.extend(["--step-timeout-sec", str(float(step_timeout_sec))])
    if profile_gate_overrides:
        for name, path in sorted(profile_gate_overrides.items()):
            cmd.extend(["--profile-gate-override", f"{name}={path}"])
    return run(cmd, root, env=env)


def main() -> int:
    parser = argparse.ArgumentParser(description="Selftest for CI profile matrix gate")
    parser.add_argument(
        PROFILE_MATRIX_GATE_SELFTEST_REAL_PROFILES_FLAG,
        default="core_lang,full,seamgrim",
        help="comma-separated real profile subset to execute (core_lang,full,seamgrim)",
    )
    parser.add_argument(
        "--dry-selftest",
        action="store_true",
        help="force dry selftest mode (same effect as DDN_CI_PROFILE_MATRIX_SELFTEST_DRY=1)",
    )
    parser.add_argument(
        "--quick-selftest",
        action="store_true",
        help="force quick selftest mode (same effect as DDN_CI_PROFILE_MATRIX_SELFTEST_QUICK=1)",
    )
    parser.add_argument(
        PROFILE_MATRIX_GATE_SELFTEST_FULL_REAL_FLAG,
        action="store_true",
        help="disable lightweight real-profile override fixtures and run the actual profile gates",
    )
    parser.add_argument(
        "--matrix-full-aggregate-gates",
        action="store_true",
        help="pass --full-aggregate-gates to run_ci_profile_matrix_gate.py invocations",
    )
    parser.add_argument(
        "--matrix-with-profile-matrix-full-real-smoke",
        action="store_true",
        help="pass --with-profile-matrix-full-real-smoke to run_ci_profile_matrix_gate.py invocations",
    )
    parser.add_argument("--json-out", default="", help="optional selftest report path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    selected_real_profiles, invalid_real_profiles = parse_profile_matrix_selftest_real_profiles(args.real_profiles)
    if invalid_real_profiles:
        print(f"check=ci_profile_matrix_gate_selftest detail=invalid_real_profiles:{','.join(invalid_real_profiles)}")
        return 1
    if not selected_real_profiles:
        print("check=ci_profile_matrix_gate_selftest detail=real_profiles_empty")
        return 1
    skipped_real_profiles = [name for name in VALID_REAL_PROFILES if name not in selected_real_profiles]
    matrix_selftest_dry_env = str(os.environ.get("DDN_CI_PROFILE_MATRIX_SELFTEST_DRY", "")).strip().lower() in {
        "1",
        "true",
        "yes",
        "on",
    }
    matrix_selftest_quick_env = str(os.environ.get("DDN_CI_PROFILE_MATRIX_SELFTEST_QUICK", "")).strip().lower() in {
        "1",
        "true",
        "yes",
        "on",
    }
    matrix_selftest_full_real_env = str(
        os.environ.get(PROFILE_MATRIX_SELFTEST_LIGHTWEIGHT_FULL_REAL_ENV_KEY, "")
    ).strip().lower() in {
        "1",
        "true",
        "yes",
        "on",
    }
    matrix_selftest_dry = bool(args.dry_selftest or matrix_selftest_dry_env)
    matrix_selftest_quick = bool(args.quick_selftest or matrix_selftest_quick_env)
    matrix_selftest_lightweight_real = bool(not args.full_real_profiles and not matrix_selftest_full_real_env)
    matrix_full_aggregate_gates = bool(args.matrix_full_aggregate_gates)
    matrix_with_profile_matrix_full_real_smoke = bool(args.matrix_with_profile_matrix_full_real_smoke)
    real_profiles_marker = (
        PROFILE_MATRIX_GATE_SELFTEST_REAL_PROFILES_MARKER
        if list(selected_real_profiles) == ["core_lang", "full", "seamgrim"]
        else f"ci_profile_matrix_selftest_real_profiles={','.join(selected_real_profiles)}"
    )
    skipped_real_profiles_marker = (
        PROFILE_MATRIX_GATE_SELFTEST_SKIPPED_REAL_PROFILES_MARKER
        if not skipped_real_profiles
        else "ci_profile_matrix_selftest_skipped_real_profiles={}".format(",".join(skipped_real_profiles))
    )
    lightweight_marker = (
        PROFILE_MATRIX_GATE_SELFTEST_LIGHTWEIGHT_FALSE_MARKER
        if not matrix_selftest_lightweight_real
        else "ci_profile_matrix_selftest_lightweight_real_profiles=true"
    )
    print(real_profiles_marker)
    print(skipped_real_profiles_marker)
    print(lightweight_marker)
    if expect_sync_graph_export_negative_cases() != 0:
        return 1
    if expect_sync_numeric_factor_policy_negative_cases() != 0:
        return 1
    if expect_sanity_numeric_factor_policy_negative_cases() != 0:
        return 1
    selftest_report: dict[str, object] = build_profile_matrix_selftest_fixture(
        selected_real_profiles,
        quick=matrix_selftest_quick,
        dry=matrix_selftest_dry,
    )

    with tempfile.TemporaryDirectory(prefix="ci_profile_matrix_gate_selftest_") as td:
        report_dir = Path(td) / "reports"
        report_dir.mkdir(parents=True, exist_ok=True)

        dry_report = report_dir / "matrix_dry.detjson"
        proc_dry = run_matrix(
            py=py,
            root=root,
            report_path=dry_report,
            profiles="core_lang,full,seamgrim",
            dry_run=True,
        )
        if expect(proc_dry.returncode == 0, "dry_run_should_pass", proc_dry) != 0:
            return 1
        if expect_marker(proc_dry, "ci_profile_matrix_status=pass", "dry_run_status_marker_missing") != 0:
            return 1
        if expect_marker(proc_dry, "quick_gates=false", "dry_run_quick_gates_marker_missing") != 0:
            return 1
        if expect_marker(proc_dry, "quick_source=none", "dry_run_quick_source_marker_missing") != 0:
            return 1
        if expect_marker(proc_dry, "quick_reason=none_no_inputs", "dry_run_quick_reason_marker_missing") != 0:
            return 1
        if expect_marker(proc_dry, "quick_reason_ok=true", "dry_run_quick_reason_ok_marker_missing") != 0:
            return 1
        if expect_marker(proc_dry, "quick_env_parse_ok=true", "dry_run_quick_env_parse_marker_missing") != 0:
            return 1
        if expect_marker(proc_dry, "quick_env_state=empty", "dry_run_quick_env_state_marker_missing") != 0:
            return 1
        if expect_marker(proc_dry, "quick_env_warning=none", "dry_run_quick_env_warning_marker_missing") != 0:
            return 1
        if expect_marker(proc_dry, "quick_steps=0/3", "dry_run_quick_steps_marker_missing") != 0:
            return 1
        if expect_marker(proc_dry, "quick_contract_ok=true", "dry_run_quick_contract_marker_missing") != 0:
            return 1
        if expect_marker(proc_dry, "warnings=0", "dry_run_warning_count_marker_missing") != 0:
            return 1
        if expect_marker(proc_dry, "code=OK", "dry_run_code_marker_missing") != 0:
            return 1
        if expect(dry_report.exists(), "dry_run_report_missing", proc_dry) != 0:
            return 1
        dry_doc = load_json(dry_report)
        if expect(str(dry_doc.get("schema", "")) == MATRIX_SCHEMA, "dry_run_schema_mismatch") != 0:
            return 1
        if expect(str(dry_doc.get("status", "")) == "pass", "dry_run_status_mismatch") != 0:
            return 1
        if expect(str(dry_doc.get("code", "")) == "OK", "dry_run_code_mismatch") != 0:
            return 1
        if expect(bool(dry_doc.get("dry_run", False)), "dry_run_flag_mismatch") != 0:
            return 1
        if expect(bool(dry_doc.get("step_timeout_enabled", True)) is False, "dry_run_timeout_enabled_mismatch") != 0:
            return 1
        if expect(float(dry_doc.get("step_timeout_sec", -1.0)) == 0.0, "dry_run_timeout_sec_mismatch") != 0:
            return 1
        if expect(
            list(dry_doc.get("profiles", [])) == ["core_lang", "full", "seamgrim"],
            "dry_run_profiles_mismatch",
        ) != 0:
            return 1
        if expect(int(len(dry_doc.get("steps", []))) == 3, "dry_run_steps_count_mismatch") != 0:
            return 1
        dry_rows = dry_doc.get("steps", [])
        if expect(int(dry_doc.get("total_elapsed_ms", -1)) == 0, "dry_run_total_elapsed_ms_mismatch") != 0:
            return 1
        if (
            expect(
                isinstance(dry_rows, list) and all(bool(dict(row).get("quick_applied", True)) is False for row in dry_rows),
                "dry_run_rows_quick_applied_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(dry_rows, list) and all(int(dict(row).get("elapsed_ms", -1)) == 0 for row in dry_rows),
                "dry_run_rows_elapsed_ms_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(dry_rows, list) and all(bool(dict(row).get("timed_out", True)) is False for row in dry_rows),
                "dry_run_rows_timed_out_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(dry_rows, list) and all(float(dict(row).get("timeout_sec", -1.0)) == 0.0 for row in dry_rows),
                "dry_run_rows_timeout_sec_mismatch",
            )
            != 0
        ):
            return 1
        if expect(int(dry_doc.get("quick_steps_count", -1)) == 0, "dry_run_quick_steps_count_mismatch") != 0:
            return 1
        if expect(list(dry_doc.get("quick_enabled_profiles", [])) == [], "dry_run_quick_enabled_profiles_doc_mismatch") != 0:
            return 1
        if (
            expect(
                list(dry_doc.get("quick_disabled_profiles", [])) == ["core_lang", "full", "seamgrim"],
                "dry_run_quick_disabled_profiles_doc_mismatch",
            )
            != 0
        ):
            return 1
        if expect(int(dry_doc.get("quick_profile_count", -1)) == 3, "dry_run_quick_profile_count_doc_mismatch") != 0:
            return 1
        if expect(bool(dry_doc.get("quick_profile_flags_complete", False)), "dry_run_quick_profile_flags_complete_doc_mismatch") != 0:
            return 1
        if expect(int(dry_doc.get("quick_steps_total", -1)) == 3, "dry_run_quick_steps_total_mismatch") != 0:
            return 1
        if expect(bool(dry_doc.get("quick_steps_all", True)) is False, "dry_run_quick_steps_all_mismatch") != 0:
            return 1
        if expect(bool(dry_doc.get("quick_gates_env_parse_ok", False)), "dry_run_quick_env_parse_doc_mismatch") != 0:
            return 1
        if expect(str(dry_doc.get("quick_gates_env_state", "")) == "empty", "dry_run_quick_env_state_doc_mismatch") != 0:
            return 1
        if expect(str(dry_doc.get("quick_gates_env_normalized", "")) == "", "dry_run_quick_env_normalized_doc_mismatch") != 0:
            return 1
        if expect(str(dry_doc.get("quick_decision_reason", "")) == "none_no_inputs", "dry_run_quick_reason_doc_mismatch") != 0:
            return 1
        if expect(str(dry_doc.get("quick_decision_expected_reason", "")) == "none_no_inputs", "dry_run_quick_expected_reason_doc_mismatch") != 0:
            return 1
        if expect(bool(dry_doc.get("quick_decision_contract_ok", False)), "dry_run_quick_reason_contract_doc_mismatch") != 0:
            return 1
        if expect(list(dry_doc.get("quick_decision_contract_issues", [])) == [], "dry_run_quick_reason_contract_issues_doc_mismatch") != 0:
            return 1
        if expect(bool(dry_doc.get("quick_gates_source_uses_arg", True)) is False, "dry_run_source_uses_arg_doc_mismatch") != 0:
            return 1
        if expect(bool(dry_doc.get("quick_gates_source_uses_env", True)) is False, "dry_run_source_uses_env_doc_mismatch") != 0:
            return 1
        if expect(str(dry_doc.get("quick_gates_env_warning", "")) == "none", "dry_run_quick_env_warning_doc_mismatch") != 0:
            return 1
        if expect(int(dry_doc.get("warning_count", -1)) == 0, "dry_run_warning_count_doc_mismatch") != 0:
            return 1
        if expect(bool(dry_doc.get("has_warnings", True)) is False, "dry_run_has_warnings_doc_mismatch") != 0:
            return 1
        if expect(list(dry_doc.get("warning_codes", [])) == [], "dry_run_warning_codes_doc_mismatch") != 0:
            return 1
        if expect(dict(dry_doc.get("warning_code_counts", {})) == {}, "dry_run_warning_code_counts_doc_mismatch") != 0:
            return 1
        if expect(list(dry_doc.get("warnings", [])) == [], "dry_run_warnings_doc_mismatch") != 0:
            return 1
        if expect(bool(dry_doc.get("quick_contract_ok", False)), "dry_run_quick_contract_doc_mismatch") != 0:
            return 1
        if expect(list(dry_doc.get("quick_contract_issues", [])) == [], "dry_run_quick_contract_issues_doc_mismatch") != 0:
            return 1
        if expect(bool(dry_doc.get("aggregate_summary_sanity_ok", False)), "dry_run_aggregate_summary_sanity_ok_mismatch") != 0:
            return 1
        if (
            expect(
                list(dry_doc.get("aggregate_summary_sanity_checked_profiles", [])) == [],
                "dry_run_aggregate_summary_checked_profiles_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                list(dry_doc.get("aggregate_summary_sanity_failed_profiles", [])) == [],
                "dry_run_aggregate_summary_failed_profiles_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                list(dry_doc.get("aggregate_summary_sanity_skipped_profiles", [])) == ["core_lang", "full", "seamgrim"],
                "dry_run_aggregate_summary_skipped_profiles_mismatch",
            )
            != 0
        ):
            return 1
        dry_aggregate_summary = dry_doc.get("aggregate_summary_sanity_by_profile", {})
        if not isinstance(dry_aggregate_summary, dict):
            return expect(False, "dry_run_aggregate_summary_by_profile_missing")
        if (
            expect_aggregate_summary_sanity(
                dry_aggregate_summary.get("core_lang"),
                "core_lang",
                expected_profile_matrix_summary_values("core_lang"),
                expected_present=False,
                expected_gate_marker=False,
                prefix="dry_run_core_lang",
            )
            != 0
        ):
            return 1
        if (
            expect_aggregate_summary_sanity(
                dry_aggregate_summary.get("full"),
                "full",
                expected_profile_matrix_summary_values("full"),
                expected_present=False,
                expected_gate_marker=True,
                prefix="dry_run_full",
            )
            != 0
        ):
            return 1
        if (
            expect_aggregate_summary_sanity(
                dry_aggregate_summary.get("seamgrim"),
                "seamgrim",
                expected_profile_matrix_summary_values("seamgrim"),
                expected_present=False,
                expected_gate_marker=True,
                prefix="dry_run_seamgrim",
            )
            != 0
        ):
            return 1

        passthrough_flags_report = report_dir / "matrix_passthrough_flags.detjson"
        proc_passthrough_flags = run_matrix(
            py=py,
            root=root,
            report_path=passthrough_flags_report,
            profiles="core_lang,full",
            dry_run=True,
            full_aggregate_gates=True,
            with_profile_matrix_full_real_smoke=True,
        )
        if expect(proc_passthrough_flags.returncode == 0, "passthrough_flags_dry_run_should_pass", proc_passthrough_flags) != 0:
            return 1
        if expect_marker(
            proc_passthrough_flags,
            "full_aggregate_gates=true",
            "passthrough_flags_full_aggregate_marker_missing",
        ) != 0:
            return 1
        if expect_marker(
            proc_passthrough_flags,
            "full_real_smoke_gates=true",
            "passthrough_flags_full_real_smoke_marker_missing",
        ) != 0:
            return 1
        if expect_marker(
            proc_passthrough_flags,
            "full_aggregate_contract_ok=true",
            "passthrough_flags_full_aggregate_contract_marker_missing",
        ) != 0:
            return 1
        if expect_marker(
            proc_passthrough_flags,
            "full_real_smoke_contract_ok=true",
            "passthrough_flags_full_real_smoke_contract_marker_missing",
        ) != 0:
            return 1
        if expect(passthrough_flags_report.exists(), "passthrough_flags_report_missing", proc_passthrough_flags) != 0:
            return 1
        passthrough_flags_doc = load_json(passthrough_flags_report)
        if expect(bool(passthrough_flags_doc.get("full_aggregate_gates", False)), "passthrough_flags_full_aggregate_doc_mismatch") != 0:
            return 1
        if expect(
            bool(passthrough_flags_doc.get("with_profile_matrix_full_real_smoke", False)),
            "passthrough_flags_full_real_smoke_doc_mismatch",
        ) != 0:
            return 1
        if expect(int(passthrough_flags_doc.get("full_aggregate_steps_count", -1)) == 2, "passthrough_flags_full_aggregate_steps_count_mismatch") != 0:
            return 1
        if expect(int(passthrough_flags_doc.get("full_real_smoke_steps_count", -1)) == 2, "passthrough_flags_full_real_smoke_steps_count_mismatch") != 0:
            return 1
        if expect(bool(passthrough_flags_doc.get("full_aggregate_contract_ok", False)), "passthrough_flags_full_aggregate_contract_doc_mismatch") != 0:
            return 1
        if expect(bool(passthrough_flags_doc.get("full_real_smoke_contract_ok", False)), "passthrough_flags_full_real_smoke_contract_doc_mismatch") != 0:
            return 1
        passthrough_rows = passthrough_flags_doc.get("steps", [])
        if expect(int(len(passthrough_rows)) == 2, "passthrough_flags_steps_len_mismatch") != 0:
            return 1
        if (
            expect(
                isinstance(passthrough_rows, list)
                and all("--full-aggregate" in list(dict(row).get("cmd", [])) for row in passthrough_rows),
                "passthrough_flags_cmd_full_aggregate_missing",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(passthrough_rows, list)
                and all(
                    "--with-profile-matrix-full-real-smoke" in list(dict(row).get("cmd", []))
                    for row in passthrough_rows
                ),
                "passthrough_flags_cmd_full_real_smoke_missing",
            )
            != 0
        ):
            return 1

        invalid_timeout_report = report_dir / "matrix_invalid_timeout.detjson"
        proc_invalid_timeout = run(
            [
                py,
                "tests/run_ci_profile_matrix_gate.py",
                "--profiles",
                "core_lang",
                "--report-dir",
                str(invalid_timeout_report.parent),
                "--report-prefix",
                "profile_matrix_selftest_invalid_timeout",
                "--json-out",
                str(invalid_timeout_report),
                "--dry-run",
                "--step-timeout-sec",
                "-0.5",
            ],
            root,
        )
        if expect(proc_invalid_timeout.returncode != 0, "invalid_timeout_case_should_fail", proc_invalid_timeout) != 0:
            return 1
        if expect(int(proc_invalid_timeout.returncode) == 2, "invalid_timeout_returncode_mismatch", proc_invalid_timeout) != 0:
            return 1
        if (
            expect(
                "step timeout must be >= 0" in str(proc_invalid_timeout.stderr or ""),
                "invalid_timeout_stderr_mismatch",
                proc_invalid_timeout,
            )
            != 0
        ):
            return 1
        if expect(not invalid_timeout_report.exists(), "invalid_timeout_report_should_not_exist", proc_invalid_timeout) != 0:
            return 1

        invalid_report = report_dir / "matrix_invalid.detjson"
        proc_invalid = run_matrix(
            py=py,
            root=root,
            report_path=invalid_report,
            profiles="core_lang,broken",
            dry_run=True,
        )
        if expect(proc_invalid.returncode != 0, "invalid_profile_should_fail", proc_invalid) != 0:
            return 1
        if (
            expect_marker(
                proc_invalid,
                f"code={CODES['PROFILE_INVALID']}",
                "invalid_profile_code_marker_missing",
            )
            != 0
        ):
            return 1
        invalid_doc = load_json(invalid_report)
        if expect(str(invalid_doc.get("status", "")) == "fail", "invalid_profile_status_mismatch") != 0:
            return 1
        if (
            expect(
                str(invalid_doc.get("code", "")) == CODES["PROFILE_INVALID"],
                "invalid_profile_code_mismatch",
            )
            != 0
        ):
            return 1
        if expect(list(invalid_doc.get("invalid_profiles", [])) == ["broken"], "invalid_profile_list_mismatch") != 0:
            return 1

        dedupe_report = report_dir / "matrix_dedupe.detjson"
        proc_dedupe = run_matrix(
            py=py,
            root=root,
            report_path=dedupe_report,
            profiles="full,core_lang,full,seamgrim",
            dry_run=True,
        )
        if expect(proc_dedupe.returncode == 0, "dedupe_case_should_pass", proc_dedupe) != 0:
            return 1
        dedupe_doc = load_json(dedupe_report)
        if expect(
            list(dedupe_doc.get("profiles", [])) == ["full", "core_lang", "seamgrim"],
            "dedupe_profiles_order_mismatch",
        ) != 0:
            return 1

        env_quick_report = report_dir / "matrix_env_quick.detjson"
        proc_env_quick = run_matrix(
            py=py,
            root=root,
            report_path=env_quick_report,
            profiles="core_lang",
            dry_run=True,
            env={**os.environ, MATRIX_QUICK_ENV_KEY: "1"},
        )
        if expect(proc_env_quick.returncode == 0, "env_quick_case_should_pass", proc_env_quick) != 0:
            return 1
        if expect_marker(proc_env_quick, "quick_gates=true", "env_quick_stdout_flag_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick, "quick_source=env", "env_quick_stdout_source_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick, "quick_reason=env_only_true", "env_quick_stdout_reason_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick, "quick_reason_ok=true", "env_quick_stdout_reason_ok_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick, "quick_env_parse_ok=true", "env_quick_stdout_parse_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick, "quick_env_state=true", "env_quick_stdout_state_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick, "quick_env_warning=none", "env_quick_stdout_warning_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick, "quick_steps=1/1", "env_quick_stdout_steps_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick, "quick_contract_ok=true", "env_quick_quick_contract_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick, "warnings=0", "env_quick_warning_count_marker_missing") != 0:
            return 1
        env_quick_doc = load_json(env_quick_report)
        if expect(bool(env_quick_doc.get("quick_gates", False)), "env_quick_effective_flag_mismatch") != 0:
            return 1
        if expect(bool(env_quick_doc.get("quick_gates_arg", True)) is False, "env_quick_arg_flag_mismatch") != 0:
            return 1
        if expect(bool(env_quick_doc.get("quick_gates_env", False)), "env_quick_env_flag_mismatch") != 0:
            return 1
        if expect(bool(env_quick_doc.get("quick_gates_env_parse_ok", False)), "env_quick_env_parse_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_doc.get("quick_gates_env_raw", "")) == "1", "env_quick_env_raw_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_doc.get("quick_gates_env_state", "")) == "true", "env_quick_env_state_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_doc.get("quick_gates_env_normalized", "")) == "1", "env_quick_env_normalized_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_doc.get("quick_decision_reason", "")) == "env_only_true", "env_quick_reason_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_doc.get("quick_decision_expected_reason", "")) == "env_only_true", "env_quick_expected_reason_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_doc.get("quick_decision_contract_ok", False)), "env_quick_reason_contract_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_doc.get("quick_decision_contract_issues", [])) == [], "env_quick_reason_contract_issues_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_doc.get("quick_gates_source_uses_arg", True)) is False, "env_quick_source_uses_arg_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_doc.get("quick_gates_source_uses_env", False)), "env_quick_source_uses_env_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_doc.get("quick_gates_env_warning", "")) == "none", "env_quick_env_warning_doc_mismatch") != 0:
            return 1
        if expect(int(env_quick_doc.get("warning_count", -1)) == 0, "env_quick_warning_count_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_doc.get("has_warnings", True)) is False, "env_quick_has_warnings_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_doc.get("warning_codes", [])) == [], "env_quick_warning_codes_doc_mismatch") != 0:
            return 1
        if expect(dict(env_quick_doc.get("warning_code_counts", {})) == {}, "env_quick_warning_code_counts_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_doc.get("warnings", [])) == [], "env_quick_warnings_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_doc.get("quick_contract_ok", False)), "env_quick_quick_contract_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_doc.get("quick_contract_issues", [])) == [], "env_quick_quick_contract_issues_doc_mismatch") != 0:
            return 1
        if (
            expect(
                str(env_quick_doc.get("quick_gates_env_key", "")) == MATRIX_QUICK_ENV_KEY,
                "env_quick_env_key_mismatch",
            )
            != 0
        ):
            return 1
        if expect(str(env_quick_doc.get("quick_gates_source", "")) == "env", "env_quick_source_mismatch") != 0:
            return 1
        if expect(int(env_quick_doc.get("quick_steps_count", -1)) == 1, "env_quick_steps_count_mismatch") != 0:
            return 1
        if expect(list(env_quick_doc.get("quick_enabled_profiles", [])) == ["core_lang"], "env_quick_enabled_profiles_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_doc.get("quick_disabled_profiles", [])) == [], "env_quick_disabled_profiles_doc_mismatch") != 0:
            return 1
        if expect(int(env_quick_doc.get("quick_profile_count", -1)) == 1, "env_quick_profile_count_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_doc.get("quick_profile_flags_complete", False)), "env_quick_profile_flags_complete_doc_mismatch") != 0:
            return 1
        if expect(int(env_quick_doc.get("quick_steps_total", -1)) == 1, "env_quick_steps_total_mismatch") != 0:
            return 1
        if expect(bool(env_quick_doc.get("quick_steps_all", False)), "env_quick_steps_all_mismatch") != 0:
            return 1
        env_quick_rows = env_quick_doc.get("steps", [])
        env_quick_cmd = env_quick_rows[0].get("cmd", []) if isinstance(env_quick_rows, list) and env_quick_rows else []
        if expect(isinstance(env_quick_cmd, list) and "--quick" in env_quick_cmd, "env_quick_cmd_flag_missing") != 0:
            return 1
        if (
            expect(
                isinstance(env_quick_rows, list)
                and bool(dict(env_quick_rows[0]).get("quick_applied", False)),
                "env_quick_row_quick_applied_mismatch",
            )
            != 0
        ):
            return 1

        env_quick_both_report = report_dir / "matrix_env_quick_both.detjson"
        proc_env_quick_both = run_matrix(
            py=py,
            root=root,
            report_path=env_quick_both_report,
            profiles="core_lang",
            dry_run=True,
            quick_gates=True,
            env={**os.environ, MATRIX_QUICK_ENV_KEY: "1"},
        )
        if expect(proc_env_quick_both.returncode == 0, "env_quick_both_case_should_pass", proc_env_quick_both) != 0:
            return 1
        if expect_marker(proc_env_quick_both, "quick_gates=true", "env_quick_both_stdout_flag_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_both, "quick_source=arg+env", "env_quick_both_stdout_source_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_both, "quick_reason=arg_and_env_true", "env_quick_both_stdout_reason_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_both, "quick_reason_ok=true", "env_quick_both_stdout_reason_ok_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_both, "quick_env_parse_ok=true", "env_quick_both_stdout_parse_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_both, "quick_env_state=true", "env_quick_both_stdout_state_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_both, "quick_env_warning=none", "env_quick_both_stdout_warning_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_both, "quick_steps=1/1", "env_quick_both_stdout_steps_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_both, "quick_contract_ok=true", "env_quick_both_quick_contract_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_both, "warnings=0", "env_quick_both_warning_count_marker_missing") != 0:
            return 1
        env_quick_both_doc = load_json(env_quick_both_report)
        if expect(bool(env_quick_both_doc.get("quick_gates", False)), "env_quick_both_effective_flag_mismatch") != 0:
            return 1
        if expect(bool(env_quick_both_doc.get("quick_gates_arg", False)), "env_quick_both_arg_flag_mismatch") != 0:
            return 1
        if expect(bool(env_quick_both_doc.get("quick_gates_env", False)), "env_quick_both_env_flag_mismatch") != 0:
            return 1
        if expect(bool(env_quick_both_doc.get("quick_gates_env_parse_ok", False)), "env_quick_both_env_parse_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_both_doc.get("quick_gates_env_raw", "")) == "1", "env_quick_both_env_raw_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_both_doc.get("quick_gates_env_normalized", "")) == "1", "env_quick_both_env_normalized_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_both_doc.get("quick_gates_env_state", "")) == "true", "env_quick_both_env_state_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_both_doc.get("quick_decision_reason", "")) == "arg_and_env_true", "env_quick_both_reason_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_both_doc.get("quick_decision_expected_reason", "")) == "arg_and_env_true", "env_quick_both_expected_reason_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_both_doc.get("quick_decision_contract_ok", False)), "env_quick_both_reason_contract_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_both_doc.get("quick_decision_contract_issues", [])) == [], "env_quick_both_reason_contract_issues_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_both_doc.get("quick_gates_env_warning", "")) == "none", "env_quick_both_env_warning_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_both_doc.get("quick_gates_source_uses_arg", False)), "env_quick_both_source_uses_arg_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_both_doc.get("quick_gates_source_uses_env", False)), "env_quick_both_source_uses_env_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_both_doc.get("quick_gates_source", "")) == "arg+env", "env_quick_both_source_mismatch") != 0:
            return 1
        if expect(int(env_quick_both_doc.get("quick_steps_count", -1)) == 1, "env_quick_both_steps_count_mismatch") != 0:
            return 1
        if expect(list(env_quick_both_doc.get("quick_enabled_profiles", [])) == ["core_lang"], "env_quick_both_enabled_profiles_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_both_doc.get("quick_disabled_profiles", [])) == [], "env_quick_both_disabled_profiles_doc_mismatch") != 0:
            return 1
        if expect(int(env_quick_both_doc.get("quick_profile_count", -1)) == 1, "env_quick_both_profile_count_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_both_doc.get("quick_profile_flags_complete", False)), "env_quick_both_profile_flags_complete_doc_mismatch") != 0:
            return 1
        if expect(int(env_quick_both_doc.get("quick_steps_total", -1)) == 1, "env_quick_both_steps_total_mismatch") != 0:
            return 1
        if expect(bool(env_quick_both_doc.get("quick_steps_all", False)), "env_quick_both_steps_all_mismatch") != 0:
            return 1
        if expect(int(env_quick_both_doc.get("warning_count", -1)) == 0, "env_quick_both_warning_count_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_both_doc.get("has_warnings", True)) is False, "env_quick_both_has_warnings_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_both_doc.get("warning_codes", [])) == [], "env_quick_both_warning_codes_doc_mismatch") != 0:
            return 1
        if expect(dict(env_quick_both_doc.get("warning_code_counts", {})) == {}, "env_quick_both_warning_code_counts_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_both_doc.get("warnings", [])) == [], "env_quick_both_warnings_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_both_doc.get("quick_contract_ok", False)), "env_quick_both_quick_contract_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_both_doc.get("quick_contract_issues", [])) == [], "env_quick_both_quick_contract_issues_doc_mismatch") != 0:
            return 1
        env_quick_both_rows = env_quick_both_doc.get("steps", [])
        env_quick_both_cmd = (
            env_quick_both_rows[0].get("cmd", [])
            if isinstance(env_quick_both_rows, list) and env_quick_both_rows
            else []
        )
        if expect(isinstance(env_quick_both_cmd, list) and "--quick" in env_quick_both_cmd, "env_quick_both_cmd_flag_mismatch") != 0:
            return 1
        if (
            expect(
                isinstance(env_quick_both_rows, list)
                and bool(dict(env_quick_both_rows[0]).get("quick_applied", False)),
                "env_quick_both_row_quick_applied_mismatch",
            )
            != 0
        ):
            return 1

        env_quick_off_report = report_dir / "matrix_env_quick_off.detjson"
        proc_env_quick_off = run_matrix(
            py=py,
            root=root,
            report_path=env_quick_off_report,
            profiles="core_lang",
            dry_run=True,
            env={**os.environ, MATRIX_QUICK_ENV_KEY: "0"},
        )
        if expect(proc_env_quick_off.returncode == 0, "env_quick_off_case_should_pass", proc_env_quick_off) != 0:
            return 1
        if expect_marker(proc_env_quick_off, "quick_gates=false", "env_quick_off_stdout_flag_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_off, "quick_source=none", "env_quick_off_stdout_source_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_off, "quick_reason=none_with_env_false", "env_quick_off_stdout_reason_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_off, "quick_reason_ok=true", "env_quick_off_stdout_reason_ok_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_off, "quick_env_parse_ok=true", "env_quick_off_stdout_parse_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_off, "quick_env_state=false", "env_quick_off_stdout_state_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_off, "quick_env_warning=none", "env_quick_off_stdout_warning_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_off, "quick_steps=0/1", "env_quick_off_stdout_steps_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_off, "quick_contract_ok=true", "env_quick_off_quick_contract_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_off, "warnings=0", "env_quick_off_warning_count_marker_missing") != 0:
            return 1
        env_quick_off_doc = load_json(env_quick_off_report)
        if expect(bool(env_quick_off_doc.get("quick_gates", True)) is False, "env_quick_off_effective_flag_mismatch") != 0:
            return 1
        if expect(bool(env_quick_off_doc.get("quick_gates_env", True)) is False, "env_quick_off_env_flag_mismatch") != 0:
            return 1
        if expect(bool(env_quick_off_doc.get("quick_gates_env_parse_ok", False)), "env_quick_off_env_parse_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_off_doc.get("quick_gates_env_raw", "")) == "0", "env_quick_off_env_raw_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_off_doc.get("quick_gates_env_state", "")) == "false", "env_quick_off_env_state_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_off_doc.get("quick_gates_env_normalized", "")) == "0", "env_quick_off_env_normalized_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_off_doc.get("quick_decision_reason", "")) == "none_with_env_false", "env_quick_off_reason_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_off_doc.get("quick_decision_expected_reason", "")) == "none_with_env_false", "env_quick_off_expected_reason_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_off_doc.get("quick_decision_contract_ok", False)), "env_quick_off_reason_contract_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_off_doc.get("quick_decision_contract_issues", [])) == [], "env_quick_off_reason_contract_issues_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_off_doc.get("quick_gates_source_uses_arg", True)) is False, "env_quick_off_source_uses_arg_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_off_doc.get("quick_gates_source_uses_env", True)) is False, "env_quick_off_source_uses_env_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_off_doc.get("quick_gates_env_warning", "")) == "none", "env_quick_off_env_warning_doc_mismatch") != 0:
            return 1
        if expect(int(env_quick_off_doc.get("warning_count", -1)) == 0, "env_quick_off_warning_count_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_off_doc.get("has_warnings", True)) is False, "env_quick_off_has_warnings_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_off_doc.get("warning_codes", [])) == [], "env_quick_off_warning_codes_doc_mismatch") != 0:
            return 1
        if expect(dict(env_quick_off_doc.get("warning_code_counts", {})) == {}, "env_quick_off_warning_code_counts_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_off_doc.get("warnings", [])) == [], "env_quick_off_warnings_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_off_doc.get("quick_contract_ok", False)), "env_quick_off_quick_contract_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_off_doc.get("quick_contract_issues", [])) == [], "env_quick_off_quick_contract_issues_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_off_doc.get("quick_gates_source", "")) == "none", "env_quick_off_source_mismatch") != 0:
            return 1
        if expect(int(env_quick_off_doc.get("quick_steps_count", -1)) == 0, "env_quick_off_steps_count_mismatch") != 0:
            return 1
        if expect(list(env_quick_off_doc.get("quick_enabled_profiles", [])) == [], "env_quick_off_enabled_profiles_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_off_doc.get("quick_disabled_profiles", [])) == ["core_lang"], "env_quick_off_disabled_profiles_doc_mismatch") != 0:
            return 1
        if expect(int(env_quick_off_doc.get("quick_profile_count", -1)) == 1, "env_quick_off_profile_count_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_off_doc.get("quick_profile_flags_complete", False)), "env_quick_off_profile_flags_complete_doc_mismatch") != 0:
            return 1
        if expect(int(env_quick_off_doc.get("quick_steps_total", -1)) == 1, "env_quick_off_steps_total_mismatch") != 0:
            return 1
        if expect(bool(env_quick_off_doc.get("quick_steps_all", True)) is False, "env_quick_off_steps_all_mismatch") != 0:
            return 1
        env_quick_off_rows = env_quick_off_doc.get("steps", [])
        env_quick_off_cmd = (
            env_quick_off_rows[0].get("cmd", [])
            if isinstance(env_quick_off_rows, list) and env_quick_off_rows
            else []
        )
        if (
            expect(
                isinstance(env_quick_off_cmd, list) and "--quick" not in env_quick_off_cmd,
                "env_quick_off_cmd_flag_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(env_quick_off_rows, list)
                and bool(dict(env_quick_off_rows[0]).get("quick_applied", True)) is False,
                "env_quick_off_row_quick_applied_mismatch",
            )
            != 0
        ):
            return 1

        env_quick_invalid_report = report_dir / "matrix_env_quick_invalid.detjson"
        proc_env_quick_invalid = run_matrix(
            py=py,
            root=root,
            report_path=env_quick_invalid_report,
            profiles="core_lang",
            dry_run=True,
            env={**os.environ, MATRIX_QUICK_ENV_KEY: "maybe"},
        )
        if expect(proc_env_quick_invalid.returncode == 0, "env_quick_invalid_case_should_pass", proc_env_quick_invalid) != 0:
            return 1
        if expect_marker(proc_env_quick_invalid, "quick_gates=false", "env_quick_invalid_stdout_flag_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_invalid, "quick_source=none", "env_quick_invalid_stdout_source_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_invalid, "quick_reason=none_with_env_invalid", "env_quick_invalid_stdout_reason_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_invalid, "quick_reason_ok=true", "env_quick_invalid_stdout_reason_ok_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_invalid, "quick_env_parse_ok=false", "env_quick_invalid_stdout_parse_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_invalid, "quick_env_state=invalid", "env_quick_invalid_stdout_state_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_invalid, "quick_env_warning=invalid_value", "env_quick_invalid_stdout_warning_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_invalid, "quick_steps=0/1", "env_quick_invalid_stdout_steps_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_invalid, "quick_contract_ok=true", "env_quick_invalid_quick_contract_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_invalid, "warnings=1", "env_quick_invalid_warning_count_marker_missing") != 0:
            return 1
        env_quick_invalid_doc = load_json(env_quick_invalid_report)
        if expect(bool(env_quick_invalid_doc.get("quick_gates", True)) is False, "env_quick_invalid_effective_flag_mismatch") != 0:
            return 1
        if expect(bool(env_quick_invalid_doc.get("quick_gates_env", True)) is False, "env_quick_invalid_env_flag_mismatch") != 0:
            return 1
        if (
            expect(
                bool(env_quick_invalid_doc.get("quick_gates_env_parse_ok", True)) is False,
                "env_quick_invalid_env_parse_doc_mismatch",
            )
            != 0
        ):
            return 1
        if expect(str(env_quick_invalid_doc.get("quick_gates_env_raw", "")) == "maybe", "env_quick_invalid_env_raw_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_invalid_doc.get("quick_gates_env_state", "")) == "invalid", "env_quick_invalid_env_state_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_invalid_doc.get("quick_gates_env_normalized", "")) == "maybe", "env_quick_invalid_env_normalized_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_invalid_doc.get("quick_decision_reason", "")) == "none_with_env_invalid", "env_quick_invalid_reason_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_invalid_doc.get("quick_decision_expected_reason", "")) == "none_with_env_invalid", "env_quick_invalid_expected_reason_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_invalid_doc.get("quick_decision_contract_ok", False)), "env_quick_invalid_reason_contract_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_invalid_doc.get("quick_decision_contract_issues", [])) == [], "env_quick_invalid_reason_contract_issues_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_invalid_doc.get("quick_gates_source_uses_arg", True)) is False, "env_quick_invalid_source_uses_arg_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_invalid_doc.get("quick_gates_source_uses_env", True)) is False, "env_quick_invalid_source_uses_env_doc_mismatch") != 0:
            return 1
        if (
            expect(
                str(env_quick_invalid_doc.get("quick_gates_env_warning", "")) == "invalid_value",
                "env_quick_invalid_env_warning_doc_mismatch",
            )
            != 0
        ):
            return 1
        if expect(int(env_quick_invalid_doc.get("warning_count", -1)) == 1, "env_quick_invalid_warning_count_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_invalid_doc.get("has_warnings", False)), "env_quick_invalid_has_warnings_doc_mismatch") != 0:
            return 1
        invalid_warning_codes = env_quick_invalid_doc.get("warning_codes", [])
        if (
            expect(
                isinstance(invalid_warning_codes, list) and MATRIX_WARN_QUICK_ENV_INVALID in invalid_warning_codes,
                "env_quick_invalid_warning_codes_doc_mismatch",
            )
            != 0
        ):
            return 1
        invalid_warning_code_counts = env_quick_invalid_doc.get("warning_code_counts", {})
        if (
            expect(
                isinstance(invalid_warning_code_counts, dict)
                and int(invalid_warning_code_counts.get(MATRIX_WARN_QUICK_ENV_INVALID, 0)) == 1,
                "env_quick_invalid_warning_code_counts_doc_mismatch",
            )
            != 0
        ):
            return 1
        invalid_warnings = env_quick_invalid_doc.get("warnings", [])
        if (
            expect(
                isinstance(invalid_warnings, list)
                and len(invalid_warnings) == 1
                and str(dict(invalid_warnings[0]).get("code", "")) == MATRIX_WARN_QUICK_ENV_INVALID,
                "env_quick_invalid_warnings_doc_mismatch",
            )
            != 0
        ):
            return 1
        if expect(bool(env_quick_invalid_doc.get("quick_contract_ok", False)), "env_quick_invalid_quick_contract_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_invalid_doc.get("quick_contract_issues", [])) == [], "env_quick_invalid_quick_contract_issues_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_invalid_doc.get("quick_gates_source", "")) == "none", "env_quick_invalid_source_mismatch") != 0:
            return 1
        if expect(int(env_quick_invalid_doc.get("quick_steps_count", -1)) == 0, "env_quick_invalid_steps_count_mismatch") != 0:
            return 1
        if expect(list(env_quick_invalid_doc.get("quick_enabled_profiles", [])) == [], "env_quick_invalid_enabled_profiles_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_invalid_doc.get("quick_disabled_profiles", [])) == ["core_lang"], "env_quick_invalid_disabled_profiles_doc_mismatch") != 0:
            return 1
        if expect(int(env_quick_invalid_doc.get("quick_profile_count", -1)) == 1, "env_quick_invalid_profile_count_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_invalid_doc.get("quick_profile_flags_complete", False)), "env_quick_invalid_profile_flags_complete_doc_mismatch") != 0:
            return 1
        if expect(int(env_quick_invalid_doc.get("quick_steps_total", -1)) == 1, "env_quick_invalid_steps_total_mismatch") != 0:
            return 1
        env_quick_invalid_rows = env_quick_invalid_doc.get("steps", [])
        env_quick_invalid_cmd = (
            env_quick_invalid_rows[0].get("cmd", [])
            if isinstance(env_quick_invalid_rows, list) and env_quick_invalid_rows
            else []
        )
        if (
            expect(
                isinstance(env_quick_invalid_cmd, list) and "--quick" not in env_quick_invalid_cmd,
                "env_quick_invalid_cmd_flag_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(env_quick_invalid_rows, list)
                and bool(dict(env_quick_invalid_rows[0]).get("quick_applied", True)) is False,
                "env_quick_invalid_row_quick_applied_mismatch",
            )
            != 0
        ):
            return 1

        env_quick_invalid_cli_report = report_dir / "matrix_env_quick_invalid_cli.detjson"
        proc_env_quick_invalid_cli = run_matrix(
            py=py,
            root=root,
            report_path=env_quick_invalid_cli_report,
            profiles="core_lang",
            dry_run=True,
            quick_gates=True,
            env={**os.environ, MATRIX_QUICK_ENV_KEY: "maybe"},
        )
        if (
            expect(
                proc_env_quick_invalid_cli.returncode == 0,
                "env_quick_invalid_cli_case_should_pass",
                proc_env_quick_invalid_cli,
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_quick_invalid_cli,
                "quick_source=arg",
                "env_quick_invalid_cli_stdout_source_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_quick_invalid_cli,
                "quick_reason=arg_with_env_invalid",
                "env_quick_invalid_cli_stdout_reason_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_quick_invalid_cli,
                "quick_reason_ok=true",
                "env_quick_invalid_cli_stdout_reason_ok_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_quick_invalid_cli,
                "quick_env_state=invalid",
                "env_quick_invalid_cli_stdout_state_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_quick_invalid_cli,
                "quick_env_warning=invalid_value",
                "env_quick_invalid_cli_stdout_warning_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_quick_invalid_cli,
                "quick_steps=1/1",
                "env_quick_invalid_cli_stdout_steps_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_quick_invalid_cli,
                "quick_contract_ok=true",
                "env_quick_invalid_cli_quick_contract_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_quick_invalid_cli,
                "warnings=1",
                "env_quick_invalid_cli_warning_count_marker_missing",
            )
            != 0
        ):
            return 1
        env_quick_invalid_cli_doc = load_json(env_quick_invalid_cli_report)
        if expect(bool(env_quick_invalid_cli_doc.get("quick_gates", False)), "env_quick_invalid_cli_effective_flag_mismatch") != 0:
            return 1
        if expect(bool(env_quick_invalid_cli_doc.get("quick_gates_arg", False)), "env_quick_invalid_cli_arg_flag_mismatch") != 0:
            return 1
        if expect(bool(env_quick_invalid_cli_doc.get("quick_gates_env", True)) is False, "env_quick_invalid_cli_env_flag_mismatch") != 0:
            return 1
        if (
            expect(
                bool(env_quick_invalid_cli_doc.get("quick_gates_env_parse_ok", True)) is False,
                "env_quick_invalid_cli_env_parse_doc_mismatch",
            )
            != 0
        ):
            return 1
        if expect(str(env_quick_invalid_cli_doc.get("quick_gates_env_state", "")) == "invalid", "env_quick_invalid_cli_env_state_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_invalid_cli_doc.get("quick_gates_env_normalized", "")) == "maybe", "env_quick_invalid_cli_env_normalized_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_invalid_cli_doc.get("quick_decision_reason", "")) == "arg_with_env_invalid", "env_quick_invalid_cli_reason_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_invalid_cli_doc.get("quick_decision_expected_reason", "")) == "arg_with_env_invalid", "env_quick_invalid_cli_expected_reason_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_invalid_cli_doc.get("quick_decision_contract_ok", False)), "env_quick_invalid_cli_reason_contract_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_invalid_cli_doc.get("quick_decision_contract_issues", [])) == [], "env_quick_invalid_cli_reason_contract_issues_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_invalid_cli_doc.get("quick_gates_source", "")) == "arg", "env_quick_invalid_cli_source_mismatch") != 0:
            return 1
        if expect(bool(env_quick_invalid_cli_doc.get("quick_gates_source_uses_arg", False)), "env_quick_invalid_cli_source_uses_arg_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_invalid_cli_doc.get("quick_gates_source_uses_env", True)) is False, "env_quick_invalid_cli_source_uses_env_doc_mismatch") != 0:
            return 1
        if expect(int(env_quick_invalid_cli_doc.get("warning_count", -1)) == 1, "env_quick_invalid_cli_warning_count_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_invalid_cli_doc.get("has_warnings", False)), "env_quick_invalid_cli_has_warnings_doc_mismatch") != 0:
            return 1
        invalid_cli_warning_codes = env_quick_invalid_cli_doc.get("warning_codes", [])
        if (
            expect(
                isinstance(invalid_cli_warning_codes, list) and MATRIX_WARN_QUICK_ENV_INVALID in invalid_cli_warning_codes,
                "env_quick_invalid_cli_warning_codes_doc_mismatch",
            )
            != 0
        ):
            return 1
        invalid_cli_warning_code_counts = env_quick_invalid_cli_doc.get("warning_code_counts", {})
        if (
            expect(
                isinstance(invalid_cli_warning_code_counts, dict)
                and int(invalid_cli_warning_code_counts.get(MATRIX_WARN_QUICK_ENV_INVALID, 0)) == 1,
                "env_quick_invalid_cli_warning_code_counts_doc_mismatch",
            )
            != 0
        ):
            return 1
        if expect(bool(env_quick_invalid_cli_doc.get("quick_contract_ok", False)), "env_quick_invalid_cli_quick_contract_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_invalid_cli_doc.get("quick_contract_issues", [])) == [], "env_quick_invalid_cli_quick_contract_issues_doc_mismatch") != 0:
            return 1
        if expect(int(env_quick_invalid_cli_doc.get("quick_steps_count", -1)) == 1, "env_quick_invalid_cli_steps_count_mismatch") != 0:
            return 1
        if expect(list(env_quick_invalid_cli_doc.get("quick_enabled_profiles", [])) == ["core_lang"], "env_quick_invalid_cli_enabled_profiles_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_invalid_cli_doc.get("quick_disabled_profiles", [])) == [], "env_quick_invalid_cli_disabled_profiles_doc_mismatch") != 0:
            return 1
        if expect(int(env_quick_invalid_cli_doc.get("quick_profile_count", -1)) == 1, "env_quick_invalid_cli_profile_count_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_invalid_cli_doc.get("quick_profile_flags_complete", False)), "env_quick_invalid_cli_profile_flags_complete_doc_mismatch") != 0:
            return 1
        if expect(int(env_quick_invalid_cli_doc.get("quick_steps_total", -1)) == 1, "env_quick_invalid_cli_steps_total_mismatch") != 0:
            return 1
        if expect(bool(env_quick_invalid_cli_doc.get("quick_steps_all", False)), "env_quick_invalid_cli_steps_all_mismatch") != 0:
            return 1
        invalid_cli_rows = env_quick_invalid_cli_doc.get("steps", [])
        invalid_cli_cmd = (
            invalid_cli_rows[0].get("cmd", [])
            if isinstance(invalid_cli_rows, list) and invalid_cli_rows
            else []
        )
        if expect(isinstance(invalid_cli_cmd, list) and "--quick" in invalid_cli_cmd, "env_quick_invalid_cli_cmd_flag_mismatch") != 0:
            return 1
        if (
            expect(
                isinstance(invalid_cli_rows, list)
                and bool(dict(invalid_cli_rows[0]).get("quick_applied", False)),
                "env_quick_invalid_cli_row_quick_applied_mismatch",
            )
            != 0
        ):
            return 1

        env_quick_cli_report = report_dir / "matrix_env_quick_cli.detjson"
        proc_env_quick_cli = run_matrix(
            py=py,
            root=root,
            report_path=env_quick_cli_report,
            profiles="core_lang",
            dry_run=True,
            quick_gates=True,
            env={**os.environ, MATRIX_QUICK_ENV_KEY: "0"},
        )
        if expect(proc_env_quick_cli.returncode == 0, "env_quick_cli_case_should_pass", proc_env_quick_cli) != 0:
            return 1
        if expect_marker(proc_env_quick_cli, "quick_gates=true", "env_quick_cli_stdout_flag_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_cli, "quick_source=arg", "env_quick_cli_stdout_source_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_cli, "quick_reason=arg_with_env_false", "env_quick_cli_stdout_reason_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_cli, "quick_reason_ok=true", "env_quick_cli_stdout_reason_ok_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_cli, "quick_env_parse_ok=true", "env_quick_cli_stdout_parse_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_cli, "quick_env_state=false", "env_quick_cli_stdout_state_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_cli, "quick_env_warning=none", "env_quick_cli_stdout_warning_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_cli, "quick_steps=1/1", "env_quick_cli_stdout_steps_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_cli, "quick_contract_ok=true", "env_quick_cli_quick_contract_marker_missing") != 0:
            return 1
        if expect_marker(proc_env_quick_cli, "warnings=0", "env_quick_cli_warning_count_marker_missing") != 0:
            return 1
        env_quick_cli_doc = load_json(env_quick_cli_report)
        if expect(bool(env_quick_cli_doc.get("quick_gates", False)), "env_quick_cli_effective_flag_mismatch") != 0:
            return 1
        if expect(bool(env_quick_cli_doc.get("quick_gates_arg", False)), "env_quick_cli_arg_flag_mismatch") != 0:
            return 1
        if expect(bool(env_quick_cli_doc.get("quick_gates_env", True)) is False, "env_quick_cli_env_flag_mismatch") != 0:
            return 1
        if expect(bool(env_quick_cli_doc.get("quick_gates_env_parse_ok", False)), "env_quick_cli_env_parse_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_cli_doc.get("quick_gates_env_raw", "")) == "0", "env_quick_cli_env_raw_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_cli_doc.get("quick_gates_env_state", "")) == "false", "env_quick_cli_env_state_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_cli_doc.get("quick_gates_env_normalized", "")) == "0", "env_quick_cli_env_normalized_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_cli_doc.get("quick_decision_reason", "")) == "arg_with_env_false", "env_quick_cli_reason_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_cli_doc.get("quick_decision_expected_reason", "")) == "arg_with_env_false", "env_quick_cli_expected_reason_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_cli_doc.get("quick_decision_contract_ok", False)), "env_quick_cli_reason_contract_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_cli_doc.get("quick_decision_contract_issues", [])) == [], "env_quick_cli_reason_contract_issues_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_cli_doc.get("quick_gates_source_uses_arg", False)), "env_quick_cli_source_uses_arg_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_cli_doc.get("quick_gates_source_uses_env", True)) is False, "env_quick_cli_source_uses_env_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_cli_doc.get("quick_gates_env_warning", "")) == "none", "env_quick_cli_env_warning_doc_mismatch") != 0:
            return 1
        if expect(int(env_quick_cli_doc.get("warning_count", -1)) == 0, "env_quick_cli_warning_count_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_cli_doc.get("has_warnings", True)) is False, "env_quick_cli_has_warnings_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_cli_doc.get("warning_codes", [])) == [], "env_quick_cli_warning_codes_doc_mismatch") != 0:
            return 1
        if expect(dict(env_quick_cli_doc.get("warning_code_counts", {})) == {}, "env_quick_cli_warning_code_counts_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_cli_doc.get("warnings", [])) == [], "env_quick_cli_warnings_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_cli_doc.get("quick_contract_ok", False)), "env_quick_cli_quick_contract_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_cli_doc.get("quick_contract_issues", [])) == [], "env_quick_cli_quick_contract_issues_doc_mismatch") != 0:
            return 1
        if expect(str(env_quick_cli_doc.get("quick_gates_source", "")) == "arg", "env_quick_cli_source_mismatch") != 0:
            return 1
        if expect(int(env_quick_cli_doc.get("quick_steps_count", -1)) == 1, "env_quick_cli_steps_count_mismatch") != 0:
            return 1
        if expect(list(env_quick_cli_doc.get("quick_enabled_profiles", [])) == ["core_lang"], "env_quick_cli_enabled_profiles_doc_mismatch") != 0:
            return 1
        if expect(list(env_quick_cli_doc.get("quick_disabled_profiles", [])) == [], "env_quick_cli_disabled_profiles_doc_mismatch") != 0:
            return 1
        if expect(int(env_quick_cli_doc.get("quick_profile_count", -1)) == 1, "env_quick_cli_profile_count_doc_mismatch") != 0:
            return 1
        if expect(bool(env_quick_cli_doc.get("quick_profile_flags_complete", False)), "env_quick_cli_profile_flags_complete_doc_mismatch") != 0:
            return 1
        if expect(int(env_quick_cli_doc.get("quick_steps_total", -1)) == 1, "env_quick_cli_steps_total_mismatch") != 0:
            return 1
        if expect(bool(env_quick_cli_doc.get("quick_steps_all", False)), "env_quick_cli_steps_all_mismatch") != 0:
            return 1
        env_quick_cli_rows = env_quick_cli_doc.get("steps", [])
        env_quick_cli_cmd = (
            env_quick_cli_rows[0].get("cmd", [])
            if isinstance(env_quick_cli_rows, list) and env_quick_cli_rows
            else []
        )
        if expect(isinstance(env_quick_cli_cmd, list) and "--quick" in env_quick_cli_cmd, "env_quick_cli_cmd_flag_missing") != 0:
            return 1
        if (
            expect(
                isinstance(env_quick_cli_rows, list)
                and bool(dict(env_quick_cli_rows[0]).get("quick_applied", False)),
                "env_quick_cli_row_quick_applied_mismatch",
            )
            != 0
        ):
            return 1

        cli_override_missing_marker_gate = report_dir / "fake_core_lang_cli_override_missing_marker_gate.py"
        cli_override_missing_marker_gate.write_text(
            "print('fake core_lang cli override gate without pass marker')\nraise SystemExit(0)\n",
            encoding="utf-8",
        )
        cli_override_missing_marker_report = report_dir / "matrix_cli_override_missing_marker.detjson"
        proc_cli_override_missing_marker = run_matrix(
            py=py,
            root=root,
            report_path=cli_override_missing_marker_report,
            profiles="core_lang",
            dry_run=False,
            stop_on_fail=True,
            profile_gate_overrides={"core_lang": str(cli_override_missing_marker_gate)},
        )
        if (
            expect(
                proc_cli_override_missing_marker.returncode != 0,
                "cli_override_case_should_fail",
                proc_cli_override_missing_marker,
            )
            != 0
        ):
            return 1
        if expect_marker(
            proc_cli_override_missing_marker,
            f"code={CODES['STEP_FAIL']}",
            "cli_override_case_code_marker_missing",
        ) != 0:
            return 1
        if (
            expect_marker(
                proc_cli_override_missing_marker,
                "step=core_lang",
                "cli_override_case_step_marker_missing",
            )
            != 0
        ):
            return 1
        cli_override_missing_marker_doc = load_json(cli_override_missing_marker_report)
        if (
            expect(
                str(cli_override_missing_marker_doc.get("status", "")) == "fail",
                "cli_override_status_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(cli_override_missing_marker_doc.get("code", "")) == CODES["STEP_FAIL"],
                "cli_override_code_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(cli_override_missing_marker_doc.get("step", "")) == "core_lang",
                "cli_override_step_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(len(cli_override_missing_marker_doc.get("steps", []))) == 1,
                "cli_override_steps_count_mismatch",
            )
            != 0
        ):
            return 1

        missing_script_report = report_dir / "matrix_missing_script.detjson"
        missing_script_path = report_dir / "fake_core_lang_missing_script_gate.py"
        proc_missing_script = run_matrix(
            py=py,
            root=root,
            report_path=missing_script_report,
            profiles="core_lang",
            dry_run=False,
            stop_on_fail=True,
            env={**os.environ, "DDN_CI_PROFILE_MATRIX_GATE_OVERRIDE_CORE_LANG": str(missing_script_path)},
        )
        if expect(proc_missing_script.returncode != 0, "missing_script_case_should_fail", proc_missing_script) != 0:
            return 1
        if expect_marker(
            proc_missing_script,
            f"code={CODES['STEP_FAIL']}",
            "missing_script_case_code_marker_missing",
        ) != 0:
            return 1
        if expect_marker(proc_missing_script, "step=core_lang", "missing_script_case_step_marker_missing") != 0:
            return 1
        missing_script_doc = load_json(missing_script_report)
        if expect(str(missing_script_doc.get("status", "")) == "fail", "missing_script_status_mismatch") != 0:
            return 1
        if expect(str(missing_script_doc.get("code", "")) == CODES["STEP_FAIL"], "missing_script_code_mismatch") != 0:
            return 1
        if expect(str(missing_script_doc.get("step", "")) == "core_lang", "missing_script_step_mismatch") != 0:
            return 1
        if expect(int(len(missing_script_doc.get("steps", []))) == 1, "missing_script_steps_count_mismatch") != 0:
            return 1
        missing_script_row = dict(missing_script_doc.get("steps", [{}])[0])
        if expect(bool(missing_script_row.get("ok", True)) is False, "missing_script_row_ok_mismatch") != 0:
            return 1
        if expect(int(missing_script_row.get("returncode", -1)) == 127, "missing_script_row_returncode_mismatch") != 0:
            return 1
        if expect("missing script:" in str(missing_script_row.get("stderr_head", "")), "missing_script_row_stderr_mismatch") != 0:
            return 1

        missing_script_full_report = report_dir / "matrix_missing_script_full.detjson"
        missing_script_full_path = report_dir / "fake_full_missing_script_gate.py"
        proc_missing_script_full = run_matrix(
            py=py,
            root=root,
            report_path=missing_script_full_report,
            profiles="full",
            dry_run=False,
            stop_on_fail=True,
            env={**os.environ, "DDN_CI_PROFILE_MATRIX_GATE_OVERRIDE_FULL": str(missing_script_full_path)},
        )
        if expect(proc_missing_script_full.returncode != 0, "missing_script_full_case_should_fail", proc_missing_script_full) != 0:
            return 1
        if expect_marker(
            proc_missing_script_full,
            f"code={CODES['STEP_FAIL']}",
            "missing_script_full_case_code_marker_missing",
        ) != 0:
            return 1
        if expect_marker(proc_missing_script_full, "step=full", "missing_script_full_case_step_marker_missing") != 0:
            return 1
        missing_script_full_doc = load_json(missing_script_full_report)
        if expect(str(missing_script_full_doc.get("status", "")) == "fail", "missing_script_full_status_mismatch") != 0:
            return 1
        if expect(str(missing_script_full_doc.get("code", "")) == CODES["STEP_FAIL"], "missing_script_full_code_mismatch") != 0:
            return 1
        if expect(str(missing_script_full_doc.get("step", "")) == "full", "missing_script_full_step_mismatch") != 0:
            return 1
        if expect(int(len(missing_script_full_doc.get("steps", []))) == 1, "missing_script_full_steps_count_mismatch") != 0:
            return 1
        missing_script_full_row = dict(missing_script_full_doc.get("steps", [{}])[0])
        if expect(bool(missing_script_full_row.get("ok", True)) is False, "missing_script_full_row_ok_mismatch") != 0:
            return 1
        if expect(int(missing_script_full_row.get("returncode", -1)) == 127, "missing_script_full_row_returncode_mismatch") != 0:
            return 1
        if expect(
            "missing script:" in str(missing_script_full_row.get("stderr_head", "")),
            "missing_script_full_row_stderr_mismatch",
        ) != 0:
            return 1

        missing_script_seamgrim_report = report_dir / "matrix_missing_script_seamgrim.detjson"
        missing_script_seamgrim_path = report_dir / "fake_seamgrim_missing_script_gate.py"
        proc_missing_script_seamgrim = run_matrix(
            py=py,
            root=root,
            report_path=missing_script_seamgrim_report,
            profiles="seamgrim",
            dry_run=False,
            stop_on_fail=True,
            env={**os.environ, "DDN_CI_PROFILE_MATRIX_GATE_OVERRIDE_SEAMGRIM": str(missing_script_seamgrim_path)},
        )
        if (
            expect(
                proc_missing_script_seamgrim.returncode != 0,
                "missing_script_seamgrim_case_should_fail",
                proc_missing_script_seamgrim,
            )
            != 0
        ):
            return 1
        if expect_marker(
            proc_missing_script_seamgrim,
            f"code={CODES['STEP_FAIL']}",
            "missing_script_seamgrim_case_code_marker_missing",
        ) != 0:
            return 1
        if (
            expect_marker(
                proc_missing_script_seamgrim,
                "step=seamgrim",
                "missing_script_seamgrim_case_step_marker_missing",
            )
            != 0
        ):
            return 1
        missing_script_seamgrim_doc = load_json(missing_script_seamgrim_report)
        if (
            expect(
                str(missing_script_seamgrim_doc.get("status", "")) == "fail",
                "missing_script_seamgrim_status_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(missing_script_seamgrim_doc.get("code", "")) == CODES["STEP_FAIL"],
                "missing_script_seamgrim_code_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(missing_script_seamgrim_doc.get("step", "")) == "seamgrim",
                "missing_script_seamgrim_step_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(len(missing_script_seamgrim_doc.get("steps", []))) == 1,
                "missing_script_seamgrim_steps_count_mismatch",
            )
            != 0
        ):
            return 1
        missing_script_seamgrim_row = dict(missing_script_seamgrim_doc.get("steps", [{}])[0])
        if (
            expect(
                bool(missing_script_seamgrim_row.get("ok", True)) is False,
                "missing_script_seamgrim_row_ok_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(missing_script_seamgrim_row.get("returncode", -1)) == 127,
                "missing_script_seamgrim_row_returncode_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                "missing script:" in str(missing_script_seamgrim_row.get("stderr_head", "")),
                "missing_script_seamgrim_row_stderr_mismatch",
            )
            != 0
        ):
            return 1

        missing_marker_gate = report_dir / "fake_core_lang_missing_marker_gate.py"
        missing_marker_gate.write_text(
            "print('fake core_lang gate executed without pass marker')\nraise SystemExit(0)\n",
            encoding="utf-8",
        )
        missing_marker_report = report_dir / "matrix_missing_marker.detjson"
        proc_missing_marker = run_matrix(
            py=py,
            root=root,
            report_path=missing_marker_report,
            profiles="core_lang",
            dry_run=False,
            stop_on_fail=True,
            env={**os.environ, "DDN_CI_PROFILE_MATRIX_GATE_OVERRIDE_CORE_LANG": str(missing_marker_gate)},
        )
        if expect(proc_missing_marker.returncode != 0, "missing_marker_case_should_fail", proc_missing_marker) != 0:
            return 1
        if expect_marker(
            proc_missing_marker,
            f"code={CODES['STEP_FAIL']}",
            "missing_marker_case_code_marker_missing",
        ) != 0:
            return 1
        if expect_marker(proc_missing_marker, "step=core_lang", "missing_marker_case_step_marker_missing") != 0:
            return 1
        missing_marker_doc = load_json(missing_marker_report)
        if expect(str(missing_marker_doc.get("status", "")) == "fail", "missing_marker_status_mismatch") != 0:
            return 1
        if expect(str(missing_marker_doc.get("code", "")) == CODES["STEP_FAIL"], "missing_marker_code_mismatch") != 0:
            return 1
        if expect(str(missing_marker_doc.get("step", "")) == "core_lang", "missing_marker_step_mismatch") != 0:
            return 1
        if expect(int(len(missing_marker_doc.get("steps", []))) == 1, "missing_marker_steps_count_mismatch") != 0:
            return 1
        missing_row = dict(missing_marker_doc.get("steps", [{}])[0])
        if expect(bool(missing_row.get("ok", True)) is False, "missing_marker_row_ok_mismatch") != 0:
            return 1
        if expect(int(missing_row.get("returncode", -1)) == 0, "missing_marker_row_returncode_mismatch") != 0:
            return 1

        missing_marker_full_gate = report_dir / "fake_full_missing_marker_gate.py"
        missing_marker_full_gate.write_text(
            "print('fake full gate executed without pass marker')\nraise SystemExit(0)\n",
            encoding="utf-8",
        )
        missing_marker_full_report = report_dir / "matrix_missing_marker_full.detjson"
        proc_missing_marker_full = run_matrix(
            py=py,
            root=root,
            report_path=missing_marker_full_report,
            profiles="full",
            dry_run=False,
            stop_on_fail=True,
            env={**os.environ, "DDN_CI_PROFILE_MATRIX_GATE_OVERRIDE_FULL": str(missing_marker_full_gate)},
        )
        if (
            expect(
                proc_missing_marker_full.returncode != 0,
                "missing_marker_full_case_should_fail",
                proc_missing_marker_full,
            )
            != 0
        ):
            return 1
        if expect_marker(
            proc_missing_marker_full,
            f"code={CODES['STEP_FAIL']}",
            "missing_marker_full_case_code_marker_missing",
        ) != 0:
            return 1
        if expect_marker(proc_missing_marker_full, "step=full", "missing_marker_full_case_step_marker_missing") != 0:
            return 1
        missing_marker_full_doc = load_json(missing_marker_full_report)
        if expect(str(missing_marker_full_doc.get("status", "")) == "fail", "missing_marker_full_status_mismatch") != 0:
            return 1
        if expect(str(missing_marker_full_doc.get("code", "")) == CODES["STEP_FAIL"], "missing_marker_full_code_mismatch") != 0:
            return 1
        if expect(str(missing_marker_full_doc.get("step", "")) == "full", "missing_marker_full_step_mismatch") != 0:
            return 1
        if expect(int(len(missing_marker_full_doc.get("steps", []))) == 1, "missing_marker_full_steps_count_mismatch") != 0:
            return 1
        missing_marker_full_row = dict(missing_marker_full_doc.get("steps", [{}])[0])
        if expect(bool(missing_marker_full_row.get("ok", True)) is False, "missing_marker_full_row_ok_mismatch") != 0:
            return 1
        if expect(int(missing_marker_full_row.get("returncode", -1)) == 0, "missing_marker_full_row_returncode_mismatch") != 0:
            return 1

        missing_marker_seamgrim_gate = report_dir / "fake_seamgrim_missing_marker_gate.py"
        missing_marker_seamgrim_gate.write_text(
            "print('fake seamgrim gate executed without pass marker')\nraise SystemExit(0)\n",
            encoding="utf-8",
        )
        missing_marker_seamgrim_report = report_dir / "matrix_missing_marker_seamgrim.detjson"
        proc_missing_marker_seamgrim = run_matrix(
            py=py,
            root=root,
            report_path=missing_marker_seamgrim_report,
            profiles="seamgrim",
            dry_run=False,
            stop_on_fail=True,
            env={**os.environ, "DDN_CI_PROFILE_MATRIX_GATE_OVERRIDE_SEAMGRIM": str(missing_marker_seamgrim_gate)},
        )
        if (
            expect(
                proc_missing_marker_seamgrim.returncode != 0,
                "missing_marker_seamgrim_case_should_fail",
                proc_missing_marker_seamgrim,
            )
            != 0
        ):
            return 1
        if expect_marker(
            proc_missing_marker_seamgrim,
            f"code={CODES['STEP_FAIL']}",
            "missing_marker_seamgrim_case_code_marker_missing",
        ) != 0:
            return 1
        if (
            expect_marker(
                proc_missing_marker_seamgrim,
                "step=seamgrim",
                "missing_marker_seamgrim_case_step_marker_missing",
            )
            != 0
        ):
            return 1
        missing_marker_seamgrim_doc = load_json(missing_marker_seamgrim_report)
        if (
            expect(
                str(missing_marker_seamgrim_doc.get("status", "")) == "fail",
                "missing_marker_seamgrim_status_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(missing_marker_seamgrim_doc.get("code", "")) == CODES["STEP_FAIL"],
                "missing_marker_seamgrim_code_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(missing_marker_seamgrim_doc.get("step", "")) == "seamgrim",
                "missing_marker_seamgrim_step_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(len(missing_marker_seamgrim_doc.get("steps", []))) == 1,
                "missing_marker_seamgrim_steps_count_mismatch",
            )
            != 0
        ):
            return 1
        missing_marker_seamgrim_row = dict(missing_marker_seamgrim_doc.get("steps", [{}])[0])
        if (
            expect(
                bool(missing_marker_seamgrim_row.get("ok", True)) is False,
                "missing_marker_seamgrim_row_ok_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(missing_marker_seamgrim_row.get("returncode", -1)) == 0,
                "missing_marker_seamgrim_row_returncode_mismatch",
            )
            != 0
        ):
            return 1

        fixed64_reason_cases: list[tuple[str, str, str, str]] = [
            (
                "core_lang",
                "ci_profile_core_lang_aggregate_smoke_status=fail reason=fixed64_threeway_inputs_schema_mismatch",
                "matrix_fixed64_reason_core_lang.detjson",
                "fake_core_lang_fixed64_reason_gate.py",
            ),
            (
                "full",
                "ci_profile_full_status=fail reason=aggregate_fixed64_threeway_inputs_schema_mismatch",
                "matrix_fixed64_reason_full.detjson",
                "fake_full_fixed64_reason_gate.py",
            ),
            (
                "seamgrim",
                "ci_profile_seamgrim_status=fail reason=aggregate_fixed64_threeway_inputs_schema_mismatch",
                "matrix_fixed64_reason_seamgrim.detjson",
                "fake_seamgrim_fixed64_reason_gate.py",
            ),
        ]
        for profile_name, fail_reason_token, report_name, gate_name in fixed64_reason_cases:
            fixed64_reason_gate = report_dir / gate_name
            fixed64_reason_gate.write_text(
                f"print('{fail_reason_token}')\nraise SystemExit(1)\n",
                encoding="utf-8",
            )
            fixed64_reason_report = report_dir / report_name
            proc_fixed64_reason = run_matrix(
                py=py,
                root=root,
                report_path=fixed64_reason_report,
                profiles=profile_name,
                dry_run=False,
                stop_on_fail=True,
                profile_gate_overrides={profile_name: str(fixed64_reason_gate)},
            )
            if (
                expect(
                    proc_fixed64_reason.returncode != 0,
                    f"fixed64_reason_{profile_name}_case_should_fail",
                    proc_fixed64_reason,
                )
                != 0
            ):
                return 1
            if (
                expect_marker(
                    proc_fixed64_reason,
                    f"code={CODES['STEP_FAIL']}",
                    f"fixed64_reason_{profile_name}_code_marker_missing",
                )
                != 0
            ):
                return 1
            if (
                expect_marker(
                    proc_fixed64_reason,
                    f"step={profile_name}",
                    f"fixed64_reason_{profile_name}_step_marker_missing",
                )
                != 0
            ):
                return 1
            fixed64_reason_doc = load_json(fixed64_reason_report)
            if (
                expect(
                    str(fixed64_reason_doc.get("status", "")) == "fail",
                    f"fixed64_reason_{profile_name}_status_mismatch",
                )
                != 0
            ):
                return 1
            if (
                expect(
                    str(fixed64_reason_doc.get("code", "")) == CODES["STEP_FAIL"],
                    f"fixed64_reason_{profile_name}_code_mismatch",
                )
                != 0
            ):
                return 1
            if (
                expect(
                    str(fixed64_reason_doc.get("step", "")) == profile_name,
                    f"fixed64_reason_{profile_name}_step_mismatch",
                )
                != 0
            ):
                return 1
            if (
                expect(
                    int(len(fixed64_reason_doc.get("steps", []))) == 1,
                    f"fixed64_reason_{profile_name}_steps_count_mismatch",
                )
                != 0
            ):
                return 1
            fixed64_reason_row = dict(fixed64_reason_doc.get("steps", [{}])[0])
            if (
                expect(
                    bool(fixed64_reason_row.get("ok", True)) is False,
                    f"fixed64_reason_{profile_name}_row_ok_mismatch",
                )
                != 0
            ):
                return 1
            if (
                expect(
                    int(fixed64_reason_row.get("returncode", -1)) == 1,
                    f"fixed64_reason_{profile_name}_row_returncode_mismatch",
                )
                != 0
            ):
                return 1
            if (
                expect(
                    fail_reason_token in str(fixed64_reason_row.get("stdout_head", "")),
                    f"fixed64_reason_{profile_name}_stdout_head_token_mismatch",
                )
                != 0
            ):
                return 1

        timeout_gate = report_dir / "fake_core_lang_timeout_gate.py"
        timeout_gate.write_text(
            "import time\n"
            "print('fake core_lang timeout gate started', flush=True)\n"
            "time.sleep(1.5)\n"
            "print('ci_profile_core_lang_status=pass', flush=True)\n",
            encoding="utf-8",
        )
        timeout_report = report_dir / "matrix_timeout.detjson"
        timeout_sec = 0.05
        proc_timeout = run_matrix(
            py=py,
            root=root,
            report_path=timeout_report,
            profiles="core_lang",
            dry_run=False,
            stop_on_fail=True,
            step_timeout_sec=timeout_sec,
            profile_gate_overrides={"core_lang": str(timeout_gate)},
        )
        if expect(proc_timeout.returncode != 0, "timeout_case_should_fail", proc_timeout) != 0:
            return 1
        if expect_marker(proc_timeout, f"code={CODES['STEP_FAIL']}", "timeout_case_code_marker_missing") != 0:
            return 1
        if expect_marker(proc_timeout, "step=core_lang", "timeout_case_step_marker_missing") != 0:
            return 1
        if expect_marker(proc_timeout, "timeouts=1", "timeout_case_timeouts_marker_missing") != 0:
            return 1
        timeout_doc = load_json(timeout_report)
        if expect(str(timeout_doc.get("status", "")) == "fail", "timeout_status_mismatch") != 0:
            return 1
        if expect(str(timeout_doc.get("code", "")) == CODES["STEP_FAIL"], "timeout_code_mismatch") != 0:
            return 1
        if expect(str(timeout_doc.get("step", "")) == "core_lang", "timeout_step_mismatch") != 0:
            return 1
        if expect(str(timeout_doc.get("msg", "")) == "profile step timeout: core_lang", "timeout_msg_mismatch") != 0:
            return 1
        if expect(bool(timeout_doc.get("step_timeout_enabled", False)), "timeout_enabled_mismatch") != 0:
            return 1
        if (
            expect(
                abs(float(timeout_doc.get("step_timeout_sec", -1.0)) - timeout_sec) < 1e-9,
                "timeout_sec_mismatch",
            )
            != 0
        ):
            return 1
        if expect(list(timeout_doc.get("timed_out_steps", [])) == ["core_lang"], "timeout_steps_list_mismatch") != 0:
            return 1
        if expect(int(timeout_doc.get("timed_out_step_count", -1)) == 1, "timeout_steps_count_mismatch") != 0:
            return 1
        if expect(int(len(timeout_doc.get("steps", []))) == 1, "timeout_steps_len_mismatch") != 0:
            return 1
        timeout_row = dict(timeout_doc.get("steps", [{}])[0])
        if expect(bool(timeout_row.get("ok", True)) is False, "timeout_row_ok_mismatch") != 0:
            return 1
        if expect(int(timeout_row.get("returncode", -1)) == 124, "timeout_row_returncode_mismatch") != 0:
            return 1
        if expect(bool(timeout_row.get("timed_out", False)), "timeout_row_timed_out_mismatch") != 0:
            return 1
        if (
            expect(
                abs(float(timeout_row.get("timeout_sec", -1.0)) - timeout_sec) < 1e-9,
                "timeout_row_timeout_sec_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                "step timeout after" in str(timeout_row.get("stderr_head", "")),
                "timeout_row_stderr_mismatch",
            )
            != 0
        ):
            return 1

        quick_timeout_report = report_dir / "matrix_quick_timeout.detjson"
        proc_quick_timeout = run_matrix(
            py=py,
            root=root,
            report_path=quick_timeout_report,
            profiles="core_lang",
            dry_run=False,
            stop_on_fail=True,
            quick_gates=True,
            step_timeout_sec=timeout_sec,
            profile_gate_overrides={"core_lang": str(timeout_gate)},
        )
        if expect(proc_quick_timeout.returncode != 0, "quick_timeout_case_should_fail", proc_quick_timeout) != 0:
            return 1
        if expect_marker(proc_quick_timeout, "quick_gates=true", "quick_timeout_quick_gates_marker_missing") != 0:
            return 1
        if expect_marker(proc_quick_timeout, "quick_source=arg", "quick_timeout_quick_source_marker_missing") != 0:
            return 1
        if expect_marker(proc_quick_timeout, "quick_steps=1/1", "quick_timeout_quick_steps_marker_missing") != 0:
            return 1
        if expect_marker(proc_quick_timeout, "timeouts=1", "quick_timeout_timeouts_marker_missing") != 0:
            return 1
        quick_timeout_doc = load_json(quick_timeout_report)
        if expect(bool(quick_timeout_doc.get("quick_gates", False)), "quick_timeout_quick_gates_mismatch") != 0:
            return 1
        if expect(bool(quick_timeout_doc.get("quick_gates_arg", False)), "quick_timeout_quick_gates_arg_mismatch") != 0:
            return 1
        if expect(str(quick_timeout_doc.get("quick_gates_source", "")) == "arg", "quick_timeout_quick_source_mismatch") != 0:
            return 1
        if expect(int(quick_timeout_doc.get("quick_steps_count", -1)) == 1, "quick_timeout_quick_steps_count_mismatch") != 0:
            return 1
        if expect(int(quick_timeout_doc.get("quick_steps_total", -1)) == 1, "quick_timeout_quick_steps_total_mismatch") != 0:
            return 1
        if expect(list(quick_timeout_doc.get("quick_enabled_profiles", [])) == ["core_lang"], "quick_timeout_quick_enabled_profiles_mismatch") != 0:
            return 1
        if expect(int(quick_timeout_doc.get("timed_out_step_count", -1)) == 1, "quick_timeout_timed_out_count_mismatch") != 0:
            return 1
        if expect(list(quick_timeout_doc.get("timed_out_steps", [])) == ["core_lang"], "quick_timeout_timed_out_steps_mismatch") != 0:
            return 1
        if expect(str(quick_timeout_doc.get("step", "")) == "core_lang", "quick_timeout_step_mismatch") != 0:
            return 1
        if expect(int(len(quick_timeout_doc.get("steps", []))) == 1, "quick_timeout_steps_len_mismatch") != 0:
            return 1
        quick_timeout_row = dict(quick_timeout_doc.get("steps", [{}])[0])
        if expect(bool(quick_timeout_row.get("quick_applied", False)), "quick_timeout_row_quick_applied_mismatch") != 0:
            return 1
        if expect(bool(quick_timeout_row.get("timed_out", False)), "quick_timeout_row_timed_out_mismatch") != 0:
            return 1
        if expect(int(quick_timeout_row.get("returncode", -1)) == 124, "quick_timeout_row_returncode_mismatch") != 0:
            return 1

        env_quick_timeout_report = report_dir / "matrix_env_quick_timeout.detjson"
        proc_env_quick_timeout = run_matrix(
            py=py,
            root=root,
            report_path=env_quick_timeout_report,
            profiles="core_lang",
            dry_run=False,
            stop_on_fail=True,
            step_timeout_sec=timeout_sec,
            env={**os.environ, MATRIX_QUICK_ENV_KEY: "1"},
            profile_gate_overrides={"core_lang": str(timeout_gate)},
        )
        if (
            expect(
                proc_env_quick_timeout.returncode != 0,
                "env_quick_timeout_case_should_fail",
                proc_env_quick_timeout,
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_quick_timeout,
                "quick_gates=true",
                "env_quick_timeout_quick_gates_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_quick_timeout,
                "quick_source=env",
                "env_quick_timeout_quick_source_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_quick_timeout,
                "quick_steps=1/1",
                "env_quick_timeout_quick_steps_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_quick_timeout,
                "timeouts=1",
                "env_quick_timeout_timeouts_marker_missing",
            )
            != 0
        ):
            return 1
        env_quick_timeout_doc = load_json(env_quick_timeout_report)
        if expect(bool(env_quick_timeout_doc.get("quick_gates", False)), "env_quick_timeout_quick_gates_mismatch") != 0:
            return 1
        if expect(bool(env_quick_timeout_doc.get("quick_gates_arg", True)) is False, "env_quick_timeout_quick_gates_arg_mismatch") != 0:
            return 1
        if expect(bool(env_quick_timeout_doc.get("quick_gates_env", False)), "env_quick_timeout_quick_gates_env_mismatch") != 0:
            return 1
        if expect(str(env_quick_timeout_doc.get("quick_gates_source", "")) == "env", "env_quick_timeout_quick_source_mismatch") != 0:
            return 1
        if expect(int(env_quick_timeout_doc.get("quick_steps_count", -1)) == 1, "env_quick_timeout_quick_steps_count_mismatch") != 0:
            return 1
        if expect(int(env_quick_timeout_doc.get("quick_steps_total", -1)) == 1, "env_quick_timeout_quick_steps_total_mismatch") != 0:
            return 1
        if expect(list(env_quick_timeout_doc.get("quick_enabled_profiles", [])) == ["core_lang"], "env_quick_timeout_quick_enabled_profiles_mismatch") != 0:
            return 1
        if expect(int(env_quick_timeout_doc.get("timed_out_step_count", -1)) == 1, "env_quick_timeout_timed_out_count_mismatch") != 0:
            return 1
        if expect(list(env_quick_timeout_doc.get("timed_out_steps", [])) == ["core_lang"], "env_quick_timeout_timed_out_steps_mismatch") != 0:
            return 1
        if expect(str(env_quick_timeout_doc.get("step", "")) == "core_lang", "env_quick_timeout_step_mismatch") != 0:
            return 1
        if expect(int(len(env_quick_timeout_doc.get("steps", []))) == 1, "env_quick_timeout_steps_len_mismatch") != 0:
            return 1
        env_quick_timeout_row = dict(env_quick_timeout_doc.get("steps", [{}])[0])
        if (
            expect(
                bool(env_quick_timeout_row.get("quick_applied", False)),
                "env_quick_timeout_row_quick_applied_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                bool(env_quick_timeout_row.get("timed_out", False)),
                "env_quick_timeout_row_timed_out_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(env_quick_timeout_row.get("returncode", -1)) == 124,
                "env_quick_timeout_row_returncode_mismatch",
            )
            != 0
        ):
            return 1

        quick_invalid_env_timeout_report = report_dir / "matrix_quick_invalid_env_timeout.detjson"
        proc_quick_invalid_env_timeout = run_matrix(
            py=py,
            root=root,
            report_path=quick_invalid_env_timeout_report,
            profiles="core_lang",
            dry_run=False,
            stop_on_fail=True,
            quick_gates=True,
            step_timeout_sec=timeout_sec,
            env={**os.environ, MATRIX_QUICK_ENV_KEY: "maybe"},
            profile_gate_overrides={"core_lang": str(timeout_gate)},
        )
        if (
            expect(
                proc_quick_invalid_env_timeout.returncode != 0,
                "quick_invalid_env_timeout_case_should_fail",
                proc_quick_invalid_env_timeout,
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_quick_invalid_env_timeout,
                "quick_gates=true",
                "quick_invalid_env_timeout_quick_gates_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_quick_invalid_env_timeout,
                "quick_source=arg",
                "quick_invalid_env_timeout_quick_source_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_quick_invalid_env_timeout,
                "quick_env_parse_ok=false",
                "quick_invalid_env_timeout_quick_env_parse_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_quick_invalid_env_timeout,
                "quick_env_state=invalid",
                "quick_invalid_env_timeout_quick_env_state_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_quick_invalid_env_timeout,
                "quick_env_warning=invalid_value",
                "quick_invalid_env_timeout_quick_env_warning_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_quick_invalid_env_timeout,
                "warnings=1",
                "quick_invalid_env_timeout_warning_count_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_quick_invalid_env_timeout,
                "timeouts=1",
                "quick_invalid_env_timeout_timeouts_marker_missing",
            )
            != 0
        ):
            return 1
        quick_invalid_env_timeout_doc = load_json(quick_invalid_env_timeout_report)
        if (
            expect(
                bool(quick_invalid_env_timeout_doc.get("quick_gates", False)),
                "quick_invalid_env_timeout_quick_gates_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                bool(quick_invalid_env_timeout_doc.get("quick_gates_arg", False)),
                "quick_invalid_env_timeout_quick_gates_arg_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                bool(quick_invalid_env_timeout_doc.get("quick_gates_env", True)) is False,
                "quick_invalid_env_timeout_quick_gates_env_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                bool(quick_invalid_env_timeout_doc.get("quick_gates_env_parse_ok", True)) is False,
                "quick_invalid_env_timeout_quick_env_parse_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(quick_invalid_env_timeout_doc.get("quick_gates_env_state", "")) == "invalid",
                "quick_invalid_env_timeout_quick_env_state_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(quick_invalid_env_timeout_doc.get("quick_gates_env_warning", "")) == "invalid_value",
                "quick_invalid_env_timeout_quick_env_warning_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(quick_invalid_env_timeout_doc.get("quick_gates_source", "")) == "arg",
                "quick_invalid_env_timeout_quick_source_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(quick_invalid_env_timeout_doc.get("warning_count", -1)) == 1,
                "quick_invalid_env_timeout_warning_count_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                bool(quick_invalid_env_timeout_doc.get("has_warnings", False)),
                "quick_invalid_env_timeout_has_warnings_mismatch",
            )
            != 0
        ):
            return 1
        warning_codes = list(quick_invalid_env_timeout_doc.get("warning_codes", []))
        if (
            expect(
                warning_codes == [MATRIX_WARN_QUICK_ENV_INVALID],
                "quick_invalid_env_timeout_warning_codes_mismatch",
            )
            != 0
        ):
            return 1
        warning_code_counts = dict(quick_invalid_env_timeout_doc.get("warning_code_counts", {}))
        if (
            expect(
                int(warning_code_counts.get(MATRIX_WARN_QUICK_ENV_INVALID, 0)) == 1,
                "quick_invalid_env_timeout_warning_code_counts_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(quick_invalid_env_timeout_doc.get("timed_out_step_count", -1)) == 1,
                "quick_invalid_env_timeout_timed_out_count_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                list(quick_invalid_env_timeout_doc.get("timed_out_steps", [])) == ["core_lang"],
                "quick_invalid_env_timeout_timed_out_steps_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(len(quick_invalid_env_timeout_doc.get("steps", []))) == 1,
                "quick_invalid_env_timeout_steps_len_mismatch",
            )
            != 0
        ):
            return 1
        quick_invalid_env_timeout_row = dict(quick_invalid_env_timeout_doc.get("steps", [{}])[0])
        if (
            expect(
                bool(quick_invalid_env_timeout_row.get("quick_applied", False)),
                "quick_invalid_env_timeout_row_quick_applied_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                bool(quick_invalid_env_timeout_row.get("timed_out", False)),
                "quick_invalid_env_timeout_row_timed_out_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(quick_invalid_env_timeout_row.get("returncode", -1)) == 124,
                "quick_invalid_env_timeout_row_returncode_mismatch",
            )
            != 0
        ):
            return 1

        env_invalid_timeout_report = report_dir / "matrix_env_invalid_timeout.detjson"
        proc_env_invalid_timeout = run_matrix(
            py=py,
            root=root,
            report_path=env_invalid_timeout_report,
            profiles="core_lang",
            dry_run=False,
            stop_on_fail=True,
            step_timeout_sec=timeout_sec,
            env={**os.environ, MATRIX_QUICK_ENV_KEY: "maybe"},
            profile_gate_overrides={"core_lang": str(timeout_gate)},
        )
        if (
            expect(
                proc_env_invalid_timeout.returncode != 0,
                "env_invalid_timeout_case_should_fail",
                proc_env_invalid_timeout,
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_invalid_timeout,
                "quick_gates=false",
                "env_invalid_timeout_quick_gates_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_invalid_timeout,
                "quick_source=none",
                "env_invalid_timeout_quick_source_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_invalid_timeout,
                "quick_reason=none_with_env_invalid",
                "env_invalid_timeout_quick_reason_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_invalid_timeout,
                "quick_env_parse_ok=false",
                "env_invalid_timeout_quick_env_parse_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_invalid_timeout,
                "quick_env_state=invalid",
                "env_invalid_timeout_quick_env_state_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_invalid_timeout,
                "quick_env_warning=invalid_value",
                "env_invalid_timeout_quick_env_warning_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_invalid_timeout,
                "quick_steps=0/1",
                "env_invalid_timeout_quick_steps_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_invalid_timeout,
                "warnings=1",
                "env_invalid_timeout_warning_count_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_env_invalid_timeout,
                "timeouts=1",
                "env_invalid_timeout_timeouts_marker_missing",
            )
            != 0
        ):
            return 1
        env_invalid_timeout_doc = load_json(env_invalid_timeout_report)
        if (
            expect(
                bool(env_invalid_timeout_doc.get("quick_gates", True)) is False,
                "env_invalid_timeout_quick_gates_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                bool(env_invalid_timeout_doc.get("quick_gates_arg", True)) is False,
                "env_invalid_timeout_quick_gates_arg_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                bool(env_invalid_timeout_doc.get("quick_gates_env", True)) is False,
                "env_invalid_timeout_quick_gates_env_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(env_invalid_timeout_doc.get("quick_gates_source", "")) == "none",
                "env_invalid_timeout_quick_source_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(env_invalid_timeout_doc.get("quick_decision_reason", "")) == "none_with_env_invalid",
                "env_invalid_timeout_quick_reason_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                bool(env_invalid_timeout_doc.get("quick_gates_env_parse_ok", True)) is False,
                "env_invalid_timeout_quick_env_parse_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(env_invalid_timeout_doc.get("quick_gates_env_state", "")) == "invalid",
                "env_invalid_timeout_quick_env_state_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                str(env_invalid_timeout_doc.get("quick_gates_env_warning", "")) == "invalid_value",
                "env_invalid_timeout_quick_env_warning_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(env_invalid_timeout_doc.get("quick_steps_count", -1)) == 0,
                "env_invalid_timeout_quick_steps_count_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(env_invalid_timeout_doc.get("quick_steps_total", -1)) == 1,
                "env_invalid_timeout_quick_steps_total_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                list(env_invalid_timeout_doc.get("quick_enabled_profiles", [])) == [],
                "env_invalid_timeout_quick_enabled_profiles_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                list(env_invalid_timeout_doc.get("quick_disabled_profiles", [])) == ["core_lang"],
                "env_invalid_timeout_quick_disabled_profiles_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(env_invalid_timeout_doc.get("warning_count", -1)) == 1,
                "env_invalid_timeout_warning_count_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                bool(env_invalid_timeout_doc.get("has_warnings", False)),
                "env_invalid_timeout_has_warnings_mismatch",
            )
            != 0
        ):
            return 1
        env_invalid_warning_codes = list(env_invalid_timeout_doc.get("warning_codes", []))
        if (
            expect(
                env_invalid_warning_codes == [MATRIX_WARN_QUICK_ENV_INVALID],
                "env_invalid_timeout_warning_codes_mismatch",
            )
            != 0
        ):
            return 1
        env_invalid_warning_code_counts = dict(env_invalid_timeout_doc.get("warning_code_counts", {}))
        if (
            expect(
                int(env_invalid_warning_code_counts.get(MATRIX_WARN_QUICK_ENV_INVALID, 0)) == 1,
                "env_invalid_timeout_warning_code_counts_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(env_invalid_timeout_doc.get("timed_out_step_count", -1)) == 1,
                "env_invalid_timeout_timed_out_count_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                list(env_invalid_timeout_doc.get("timed_out_steps", [])) == ["core_lang"],
                "env_invalid_timeout_timed_out_steps_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(len(env_invalid_timeout_doc.get("steps", []))) == 1,
                "env_invalid_timeout_steps_len_mismatch",
            )
            != 0
        ):
            return 1
        env_invalid_timeout_row = dict(env_invalid_timeout_doc.get("steps", [{}])[0])
        if (
            expect(
                bool(env_invalid_timeout_row.get("quick_applied", True)) is False,
                "env_invalid_timeout_row_quick_applied_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                bool(env_invalid_timeout_row.get("timed_out", False)),
                "env_invalid_timeout_row_timed_out_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(env_invalid_timeout_row.get("returncode", -1)) == 124,
                "env_invalid_timeout_row_returncode_mismatch",
            )
            != 0
        ):
            return 1

        timeout_pass_gate = report_dir / "fake_core_lang_timeout_pass_gate.py"
        timeout_pass_gate.write_text(
            "print('fake core_lang timeout pass gate')\n"
            "print('ci_profile_core_lang_status=pass')\n",
            encoding="utf-8",
        )
        timeout_pass_report = report_dir / "matrix_timeout_pass.detjson"
        timeout_pass_sec = 0.5
        proc_timeout_pass = run_matrix(
            py=py,
            root=root,
            report_path=timeout_pass_report,
            profiles="core_lang",
            dry_run=False,
            stop_on_fail=True,
            step_timeout_sec=timeout_pass_sec,
            profile_gate_overrides={"core_lang": str(timeout_pass_gate)},
        )
        if expect(proc_timeout_pass.returncode == 0, "timeout_pass_case_should_pass", proc_timeout_pass) != 0:
            return 1
        if (
            expect_marker(
                proc_timeout_pass,
                "ci_profile_matrix_status=pass",
                "timeout_pass_status_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_timeout_pass,
                "timeouts=0",
                "timeout_pass_timeouts_marker_missing",
            )
            != 0
        ):
            return 1
        timeout_pass_doc = load_json(timeout_pass_report)
        if expect(str(timeout_pass_doc.get("status", "")) == "pass", "timeout_pass_status_mismatch") != 0:
            return 1
        if expect(str(timeout_pass_doc.get("code", "")) == "OK", "timeout_pass_code_mismatch") != 0:
            return 1
        if expect(str(timeout_pass_doc.get("step", "")) == "all", "timeout_pass_step_mismatch") != 0:
            return 1
        if expect(bool(timeout_pass_doc.get("step_timeout_enabled", False)), "timeout_pass_enabled_mismatch") != 0:
            return 1
        if (
            expect(
                abs(float(timeout_pass_doc.get("step_timeout_sec", -1.0)) - timeout_pass_sec) < 1e-9,
                "timeout_pass_sec_mismatch",
            )
            != 0
        ):
            return 1
        if expect(int(timeout_pass_doc.get("timed_out_step_count", -1)) == 0, "timeout_pass_count_mismatch") != 0:
            return 1
        if expect(list(timeout_pass_doc.get("timed_out_steps", [])) == [], "timeout_pass_steps_list_mismatch") != 0:
            return 1
        if expect(int(len(timeout_pass_doc.get("steps", []))) == 1, "timeout_pass_steps_len_mismatch") != 0:
            return 1
        timeout_pass_row = dict(timeout_pass_doc.get("steps", [{}])[0])
        if expect(bool(timeout_pass_row.get("ok", False)), "timeout_pass_row_ok_mismatch") != 0:
            return 1
        if expect(bool(timeout_pass_row.get("timed_out", True)) is False, "timeout_pass_row_timed_out_mismatch") != 0:
            return 1
        if expect(int(timeout_pass_row.get("returncode", -1)) == 0, "timeout_pass_row_returncode_mismatch") != 0:
            return 1
        if (
            expect(
                abs(float(timeout_pass_row.get("timeout_sec", -1.0)) - timeout_pass_sec) < 1e-9,
                "timeout_pass_row_timeout_sec_mismatch",
            )
            != 0
        ):
            return 1

        timeout_continue_full_gate = report_dir / "fake_full_timeout_continue_pass_gate.py"
        timeout_continue_full_gate.write_text(
            "print('fake full timeout continue pass gate')\n"
            "print('ci_profile_full_status=pass')\n",
            encoding="utf-8",
        )
        timeout_continue_report = report_dir / "matrix_timeout_continue.detjson"
        proc_timeout_continue = run_matrix(
            py=py,
            root=root,
            report_path=timeout_continue_report,
            profiles="core_lang,full",
            dry_run=False,
            stop_on_fail=False,
            step_timeout_sec=timeout_sec,
            profile_gate_overrides={
                "core_lang": str(timeout_gate),
                "full": str(timeout_continue_full_gate),
            },
        )
        if (
            expect(
                proc_timeout_continue.returncode != 0,
                "timeout_continue_case_should_fail",
                proc_timeout_continue,
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_timeout_continue,
                "step=core_lang",
                "timeout_continue_step_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_timeout_continue,
                "timeouts=1",
                "timeout_continue_timeouts_marker_missing",
            )
            != 0
        ):
            return 1
        timeout_continue_doc = load_json(timeout_continue_report)
        if (
            expect(
                str(timeout_continue_doc.get("step", "")) == "core_lang",
                "timeout_continue_doc_step_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(timeout_continue_doc.get("timed_out_step_count", -1)) == 1,
                "timeout_continue_timed_out_count_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                list(timeout_continue_doc.get("timed_out_steps", [])) == ["core_lang"],
                "timeout_continue_timed_out_steps_mismatch",
            )
            != 0
        ):
            return 1
        timeout_continue_rows = timeout_continue_doc.get("steps", [])
        if (
            expect(
                int(len(timeout_continue_rows)) == 2,
                "timeout_continue_steps_len_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(timeout_continue_rows, list)
                and str(dict(timeout_continue_rows[0]).get("profile", "")) == "core_lang",
                "timeout_continue_first_profile_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(timeout_continue_rows, list)
                and str(dict(timeout_continue_rows[1]).get("profile", "")) == "full",
                "timeout_continue_second_profile_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(timeout_continue_rows, list)
                and bool(dict(timeout_continue_rows[0]).get("timed_out", False)),
                "timeout_continue_first_timed_out_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(timeout_continue_rows, list)
                and bool(dict(timeout_continue_rows[1]).get("timed_out", True)) is False,
                "timeout_continue_second_timed_out_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(timeout_continue_rows, list)
                and bool(dict(timeout_continue_rows[1]).get("ok", False)),
                "timeout_continue_second_ok_mismatch",
            )
            != 0
        ):
            return 1

        timeout_continue_quick_full_gate = report_dir / "fake_full_timeout_continue_quick_pass_gate.py"
        timeout_continue_quick_full_gate.write_text(
            "print('fake full timeout continue quick pass gate')\n"
            "print('ci_profile_full_status=pass')\n",
            encoding="utf-8",
        )
        timeout_continue_quick_report = report_dir / "matrix_timeout_continue_quick.detjson"
        proc_timeout_continue_quick = run_matrix(
            py=py,
            root=root,
            report_path=timeout_continue_quick_report,
            profiles="core_lang,full",
            dry_run=False,
            stop_on_fail=False,
            quick_gates=True,
            step_timeout_sec=timeout_sec,
            profile_gate_overrides={
                "core_lang": str(timeout_gate),
                "full": str(timeout_continue_quick_full_gate),
            },
        )
        if (
            expect(
                proc_timeout_continue_quick.returncode != 0,
                "timeout_continue_quick_case_should_fail",
                proc_timeout_continue_quick,
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_timeout_continue_quick,
                "quick_gates=true",
                "timeout_continue_quick_gates_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_timeout_continue_quick,
                "quick_source=arg",
                "timeout_continue_quick_source_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_timeout_continue_quick,
                "quick_steps=2/2",
                "timeout_continue_quick_steps_marker_missing",
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_timeout_continue_quick,
                "timeouts=1",
                "timeout_continue_quick_timeouts_marker_missing",
            )
            != 0
        ):
            return 1
        timeout_continue_quick_doc = load_json(timeout_continue_quick_report)
        if (
            expect(
                str(timeout_continue_quick_doc.get("step", "")) == "core_lang",
                "timeout_continue_quick_doc_step_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                bool(timeout_continue_quick_doc.get("quick_gates", False)),
                "timeout_continue_quick_gates_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(timeout_continue_quick_doc.get("quick_steps_count", -1)) == 2,
                "timeout_continue_quick_steps_count_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(timeout_continue_quick_doc.get("quick_steps_total", -1)) == 2,
                "timeout_continue_quick_steps_total_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                list(timeout_continue_quick_doc.get("quick_enabled_profiles", [])) == ["core_lang", "full"],
                "timeout_continue_quick_enabled_profiles_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(timeout_continue_quick_doc.get("timed_out_step_count", -1)) == 1,
                "timeout_continue_quick_timed_out_count_mismatch",
            )
            != 0
        ):
            return 1
        timeout_continue_quick_rows = timeout_continue_quick_doc.get("steps", [])
        if (
            expect(
                int(len(timeout_continue_quick_rows)) == 2,
                "timeout_continue_quick_steps_len_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(timeout_continue_quick_rows, list)
                and str(dict(timeout_continue_quick_rows[0]).get("profile", "")) == "core_lang",
                "timeout_continue_quick_first_profile_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(timeout_continue_quick_rows, list)
                and str(dict(timeout_continue_quick_rows[1]).get("profile", "")) == "full",
                "timeout_continue_quick_second_profile_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(timeout_continue_quick_rows, list)
                and bool(dict(timeout_continue_quick_rows[0]).get("quick_applied", False)),
                "timeout_continue_quick_first_quick_applied_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(timeout_continue_quick_rows, list)
                and bool(dict(timeout_continue_quick_rows[1]).get("quick_applied", False)),
                "timeout_continue_quick_second_quick_applied_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(timeout_continue_quick_rows, list)
                and bool(dict(timeout_continue_quick_rows[0]).get("timed_out", False)),
                "timeout_continue_quick_first_timed_out_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(timeout_continue_quick_rows, list)
                and bool(dict(timeout_continue_quick_rows[1]).get("timed_out", True)) is False,
                "timeout_continue_quick_second_timed_out_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(timeout_continue_quick_rows, list)
                and bool(dict(timeout_continue_quick_rows[1]).get("ok", False)),
                "timeout_continue_quick_second_ok_mismatch",
            )
            != 0
        ):
            return 1

        timeout_stop_on_fail_full_gate = report_dir / "fake_full_fast_pass_gate.py"
        timeout_stop_on_fail_full_gate.write_text(
            "print('ci_profile_full_status=pass')\n",
            encoding="utf-8",
        )
        timeout_stop_on_fail_report = report_dir / "matrix_timeout_stop_on_fail.detjson"
        proc_timeout_stop_on_fail = run_matrix(
            py=py,
            root=root,
            report_path=timeout_stop_on_fail_report,
            profiles="core_lang,full",
            dry_run=False,
            stop_on_fail=True,
            step_timeout_sec=timeout_sec,
            profile_gate_overrides={
                "core_lang": str(timeout_gate),
                "full": str(timeout_stop_on_fail_full_gate),
            },
        )
        if (
            expect(
                proc_timeout_stop_on_fail.returncode != 0,
                "timeout_stop_on_fail_case_should_fail",
                proc_timeout_stop_on_fail,
            )
            != 0
        ):
            return 1
        if (
            expect_marker(
                proc_timeout_stop_on_fail,
                "step=core_lang",
                "timeout_stop_on_fail_step_marker_missing",
            )
            != 0
        ):
            return 1
        timeout_stop_on_fail_doc = load_json(timeout_stop_on_fail_report)
        if (
            expect(
                str(timeout_stop_on_fail_doc.get("step", "")) == "core_lang",
                "timeout_stop_on_fail_doc_step_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                int(timeout_stop_on_fail_doc.get("timed_out_step_count", -1)) == 1,
                "timeout_stop_on_fail_timed_out_count_mismatch",
            )
            != 0
        ):
            return 1
        timeout_stop_on_fail_rows = timeout_stop_on_fail_doc.get("steps", [])
        if (
            expect(
                int(len(timeout_stop_on_fail_rows)) == 1,
                "timeout_stop_on_fail_steps_len_mismatch",
            )
            != 0
        ):
            return 1
        if (
            expect(
                isinstance(timeout_stop_on_fail_rows, list)
                and str(dict(timeout_stop_on_fail_rows[0]).get("profile", "")) == "core_lang",
                "timeout_stop_on_fail_rows_profile_mismatch",
            )
            != 0
        ):
            return 1

        if "core_lang" in selected_real_profiles:
            real_report = report_dir / "matrix_real_core_lang.detjson"
            real_core_lang_gate_override = report_dir / "fake_core_lang_real_gate.py"
            if not matrix_selftest_dry and matrix_selftest_lightweight_real:
                write_lightweight_profile_gate_override(real_core_lang_gate_override, "core_lang")
            real_core_lang_env = None
            if not matrix_selftest_dry and not matrix_selftest_lightweight_real:
                real_core_lang_env = dict(os.environ)
                real_core_lang_env["DDN_CI_PROFILE_GATE_FULL_AGGREGATE"] = "1"
            real_start = time.perf_counter()
            proc_real = run_matrix(
                py=py,
                root=root,
                report_path=real_report,
                profiles="core_lang",
                dry_run=matrix_selftest_dry,
                stop_on_fail=True,
                quick_gates=matrix_selftest_quick,
                full_aggregate_gates=matrix_full_aggregate_gates,
                with_profile_matrix_full_real_smoke=matrix_with_profile_matrix_full_real_smoke,
                env=real_core_lang_env,
                profile_gate_overrides=(
                    {"core_lang": str(real_core_lang_gate_override)}
                    if not matrix_selftest_dry and matrix_selftest_lightweight_real
                    else None
                ),
            )
            real_elapsed = time.perf_counter() - real_start
            print(
                "[ci-profile-matrix-selftest] real profile=core_lang elapsed_sec={:.3f} quick={} dry={}".format(
                    real_elapsed,
                    str(bool(matrix_selftest_quick)).lower(),
                    str(bool(matrix_selftest_dry)).lower(),
                )
            )
            if expect(proc_real.returncode == 0, "real_core_lang_should_pass", proc_real) != 0:
                return 1
            if not matrix_selftest_dry:
                if expect_marker(proc_real, "ci_profile_core_lang_status=pass", "real_core_lang_profile_marker_missing") != 0:
                    return 1
                if expect_marker(proc_real, "contract tier unsupported check ok", "real_core_lang_contract_tier_marker_missing") != 0:
                    return 1
                if (
                    expect_marker(
                        proc_real,
                        "contract tier age3 min enforcement check ok",
                        "real_core_lang_contract_tier_age3_min_enforcement_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if expect_marker(proc_real, "map access contract check ok", "real_core_lang_map_access_marker_missing") != 0:
                    return 1
                if expect_marker(proc_real, "gaji registry strict/audit check ok", "real_core_lang_registry_marker_missing") != 0:
                    return 1
                if expect_marker(proc_real, "[stdlib-catalog-check] ok", "real_core_lang_stdlib_catalog_marker_missing") != 0:
                    return 1
                if expect_marker(proc_real, "[tensor-v0-cli-check] ok", "real_core_lang_tensor_cli_marker_missing") != 0:
                    return 1
                if (
                    expect_marker(
                        proc_real,
                        "[fixed64-darwin-schedule-policy] ok",
                        "real_core_lang_fixed64_schedule_policy_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real,
                        "[fixed64-darwin-real-report]",
                        "real_core_lang_fixed64_real_report_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real,
                        "seamgrim featured seed catalog autogen check ok",
                        "real_core_lang_featured_seed_catalog_autogen_check_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real,
                        "[ci-profile-core-lang] runtime5 summary is emitted but not required for core_lang contract",
                        "real_core_lang_runtime5_na_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if matrix_selftest_quick:
                    if (
                        expect_marker(
                            proc_real,
                            "[ci-profile-core-lang] aggregate gate skipped by --quick",
                            "real_core_lang_quick_skip_marker_missing",
                        )
                        != 0
                    ):
                        return 1
                else:
                    if expect_marker(
                        proc_real,
                        "[ci-gate-report-index-check] ok index=",
                        "real_core_lang_index_contract_marker_missing",
                    ) != 0:
                        return 1
            if expect_marker(proc_real, "ci_profile_matrix_status=pass", "real_core_lang_matrix_marker_missing") != 0:
                return 1
            real_doc = load_json(real_report)
            if expect(bool(real_doc.get("ok", False)), "real_core_lang_ok_mismatch") != 0:
                return 1
            if expect(str(real_doc.get("step", "")) == "all", "real_core_lang_step_mismatch") != 0:
                return 1
            if expect(int(len(real_doc.get("steps", []))) == 1, "real_core_lang_steps_count_mismatch") != 0:
                return 1
            if expect(int(real_doc.get("total_elapsed_ms", -1)) >= 0, "real_core_lang_total_elapsed_ms_mismatch") != 0:
                return 1
            real_row = dict(real_doc.get("steps", [{}])[0])
            if expect(int(real_row.get("elapsed_ms", -1)) >= 0, "real_core_lang_row_elapsed_ms_mismatch") != 0:
                return 1
            if (
                expect(
                    int(real_doc.get("total_elapsed_ms", -1)) >= int(real_row.get("elapsed_ms", -1)),
                    "real_core_lang_total_vs_row_elapsed_mismatch",
                )
                != 0
            ):
                return 1
            if matrix_selftest_quick:
                real_rows = real_doc.get("steps", [])
                real_cmd = real_rows[0].get("cmd", []) if isinstance(real_rows, list) and real_rows else []
                if expect(isinstance(real_cmd, list) and "--quick" in real_cmd, "real_core_lang_quick_cmd_flag_missing") != 0:
                    return 1
            if expect(bool(real_doc.get("aggregate_summary_sanity_ok", False)), "real_core_lang_aggregate_summary_ok_mismatch") != 0:
                return 1
            if matrix_selftest_dry or matrix_selftest_quick:
                if (
                    expect(
                        list(real_doc.get("aggregate_summary_sanity_checked_profiles", [])) == [],
                        "real_core_lang_aggregate_summary_checked_profiles_mismatch",
                    )
                    != 0
                ):
                    return 1
            else:
                if (
                    expect(
                        list(real_doc.get("aggregate_summary_sanity_checked_profiles", [])) == ["core_lang"],
                        "real_core_lang_aggregate_summary_checked_profiles_mismatch",
                    )
                    != 0
                ):
                    return 1
            if (
                expect(
                    list(real_doc.get("aggregate_summary_sanity_failed_profiles", [])) == [],
                    "real_core_lang_aggregate_summary_failed_profiles_mismatch",
                )
                != 0
            ):
                return 1
            real_aggregate_summary = real_doc.get("aggregate_summary_sanity_by_profile", {})
            if not isinstance(real_aggregate_summary, dict):
                return expect(False, "real_core_lang_aggregate_summary_by_profile_missing")
            if (
                expect_aggregate_summary_sanity(
                    real_aggregate_summary.get("core_lang"),
                    "core_lang",
                    expected_profile_matrix_summary_values("core_lang"),
                    expected_present=bool(not matrix_selftest_dry and not matrix_selftest_quick),
                    expected_gate_marker=False,
                    prefix="real_core_lang",
                )
                != 0
            ):
                return 1
            real_total_elapsed_ms = max(0, int(real_doc.get("total_elapsed_ms", 0)))
            real_step_elapsed_ms = max(0, int(real_row.get("elapsed_ms", 0)))
            selftest_report["total_elapsed_ms"] = int(selftest_report.get("total_elapsed_ms", 0)) + real_total_elapsed_ms
            real_profiles = dict(selftest_report.get("real_profiles", {}))
            real_profiles["core_lang"] = {
                "selected": True,
                "skipped": False,
                "status": "pass",
                "ok": True,
                "total_elapsed_ms": real_total_elapsed_ms,
                "step_elapsed_ms": real_step_elapsed_ms,
            }
            selftest_report["real_profiles"] = real_profiles
            aggregate_summary = dict(selftest_report.get("aggregate_summary_sanity_by_profile", {}))
            aggregate_summary["core_lang"] = dict(real_aggregate_summary.get("core_lang", {}))
            selftest_report["aggregate_summary_sanity_by_profile"] = aggregate_summary
            checked_profiles = [
                str(item).strip()
                for item in selftest_report.get("aggregate_summary_sanity_checked_profiles", [])
                if str(item).strip()
            ]
            failed_profiles = [
                str(item).strip()
                for item in selftest_report.get("aggregate_summary_sanity_failed_profiles", [])
                if str(item).strip()
            ]
            skipped_profiles = [
                str(item).strip()
                for item in selftest_report.get("aggregate_summary_sanity_skipped_profiles", [])
                if str(item).strip()
            ]
            checked_profiles = [item for item in checked_profiles if item != "core_lang"]
            failed_profiles = [item for item in failed_profiles if item != "core_lang"]
            skipped_profiles = [item for item in skipped_profiles if item != "core_lang"]
            if matrix_selftest_dry or matrix_selftest_quick:
                skipped_profiles.append("core_lang")
            else:
                checked_profiles.append("core_lang")
            selftest_report["aggregate_summary_sanity_checked_profiles"] = checked_profiles
            selftest_report["aggregate_summary_sanity_failed_profiles"] = failed_profiles
            selftest_report["aggregate_summary_sanity_skipped_profiles"] = skipped_profiles
        else:
            print("[ci-profile-matrix-selftest] skip real profile=core_lang by --real-profiles")

        if "full" in selected_real_profiles:
            real_full_report = report_dir / "matrix_real_full.detjson"
            real_full_gate_override = report_dir / "fake_full_real_gate.py"
            if not matrix_selftest_dry and matrix_selftest_lightweight_real:
                write_lightweight_profile_gate_override(real_full_gate_override, "full")
            real_full_env = None
            if not matrix_selftest_dry and not matrix_selftest_lightweight_real:
                real_full_env = dict(os.environ)
                real_full_env["DDN_CI_PROFILE_GATE_FULL_AGGREGATE"] = "1"
            real_full_start = time.perf_counter()
            proc_real_full = run_matrix(
                py=py,
                root=root,
                report_path=real_full_report,
                profiles="full",
                dry_run=matrix_selftest_dry,
                stop_on_fail=True,
                quick_gates=matrix_selftest_quick,
                full_aggregate_gates=matrix_full_aggregate_gates,
                with_profile_matrix_full_real_smoke=matrix_with_profile_matrix_full_real_smoke,
                env=real_full_env,
                profile_gate_overrides=(
                    {"full": str(real_full_gate_override)}
                    if not matrix_selftest_dry and matrix_selftest_lightweight_real
                    else None
                ),
            )
            real_full_elapsed = time.perf_counter() - real_full_start
            print(
                "[ci-profile-matrix-selftest] real profile=full elapsed_sec={:.3f} quick={} dry={}".format(
                    real_full_elapsed,
                    str(bool(matrix_selftest_quick)).lower(),
                    str(bool(matrix_selftest_dry)).lower(),
                )
            )
            if expect(proc_real_full.returncode == 0, "real_full_should_pass", proc_real_full) != 0:
                return 1
            if not matrix_selftest_dry:
                if expect_marker(proc_real_full, "ci_profile_full_status=pass", "real_full_profile_marker_missing") != 0:
                    return 1
                if expect_marker(proc_real_full, "contract tier unsupported check ok", "real_full_contract_tier_marker_missing") != 0:
                    return 1
                if (
                    expect_marker(
                        proc_real_full,
                        "contract tier age3 min enforcement check ok",
                        "real_full_contract_tier_age3_min_enforcement_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if expect_marker(proc_real_full, "map access contract check ok", "real_full_map_access_marker_missing") != 0:
                    return 1
                if expect_marker(proc_real_full, "gaji registry strict/audit check ok", "real_full_registry_marker_missing") != 0:
                    return 1
                if expect_marker(proc_real_full, "[stdlib-catalog-check] ok", "real_full_stdlib_catalog_marker_missing") != 0:
                    return 1
                if expect_marker(proc_real_full, "[tensor-v0-cli-check] ok", "real_full_tensor_cli_marker_missing") != 0:
                    return 1
                if (
                    expect_marker(
                        proc_real_full,
                        "[fixed64-darwin-schedule-policy] ok",
                        "real_full_fixed64_schedule_policy_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_full,
                        "[fixed64-darwin-real-report]",
                        "real_full_fixed64_real_report_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_full,
                        "seamgrim featured seed catalog autogen check ok",
                        "real_full_featured_seed_catalog_autogen_check_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_full,
                        "seamgrim ci gate featured seed catalog step check ok",
                        "real_full_featured_seed_catalog_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_full,
                        "seamgrim ci gate featured seed catalog autogen step check ok",
                        "real_full_featured_seed_catalog_autogen_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_full,
                        "seamgrim ci gate lesson warning step check ok",
                        "real_full_lesson_warning_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_full,
                        "seamgrim ci gate stateful preview step check ok",
                        "real_full_stateful_preview_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if matrix_selftest_quick:
                    if (
                        expect_marker(
                            proc_real_full,
                            "[ci-profile-full] aggregate gate skipped by --quick",
                            "real_full_quick_skip_marker_missing",
                        )
                        != 0
                    ):
                        return 1
                else:
                    if expect_marker(
                        proc_real_full,
                        "[ci-gate-report-index-check] ok index=",
                        "real_full_index_contract_marker_missing",
                    ) != 0:
                        return 1
                    if expect_marker(
                        proc_real_full,
                        "[ci-gate-summary-report-check] ok status=pass",
                        "real_full_summary_contract_marker_missing",
                    ) != 0:
                        return 1
                    if expect_marker(
                        proc_real_full,
                        "[ci-profile-full] aggregate summary sanity markers ok",
                        "real_full_summary_sanity_markers_missing",
                    ) != 0:
                        return 1
                    if expect_marker(
                        proc_real_full,
                        "[ci-emit-artifacts-check] ok index=",
                        "real_full_emit_artifacts_contract_marker_missing",
                    ) != 0:
                        return 1
            if expect_marker(proc_real_full, "ci_profile_matrix_status=pass", "real_full_matrix_marker_missing") != 0:
                return 1
            real_full_doc = load_json(real_full_report)
            if expect(bool(real_full_doc.get("ok", False)), "real_full_ok_mismatch") != 0:
                return 1
            if expect(str(real_full_doc.get("step", "")) == "all", "real_full_step_mismatch") != 0:
                return 1
            if expect(int(len(real_full_doc.get("steps", []))) == 1, "real_full_steps_count_mismatch") != 0:
                return 1
            if expect(int(real_full_doc.get("total_elapsed_ms", -1)) >= 0, "real_full_total_elapsed_ms_mismatch") != 0:
                return 1
            real_full_row = dict(real_full_doc.get("steps", [{}])[0])
            if expect(int(real_full_row.get("elapsed_ms", -1)) >= 0, "real_full_row_elapsed_ms_mismatch") != 0:
                return 1
            if (
                expect(
                    int(real_full_doc.get("total_elapsed_ms", -1)) >= int(real_full_row.get("elapsed_ms", -1)),
                    "real_full_total_vs_row_elapsed_mismatch",
                )
                != 0
            ):
                return 1
            if matrix_selftest_quick:
                real_full_rows = real_full_doc.get("steps", [])
                real_full_cmd = real_full_rows[0].get("cmd", []) if isinstance(real_full_rows, list) and real_full_rows else []
                if expect(
                    isinstance(real_full_cmd, list) and "--quick" in real_full_cmd,
                    "real_full_quick_cmd_flag_missing",
                ) != 0:
                    return 1
            if expect(bool(real_full_doc.get("aggregate_summary_sanity_ok", False)), "real_full_aggregate_summary_ok_mismatch") != 0:
                return 1
            if matrix_selftest_dry or matrix_selftest_quick:
                if (
                    expect(
                        list(real_full_doc.get("aggregate_summary_sanity_checked_profiles", [])) == [],
                        "real_full_aggregate_summary_checked_profiles_mismatch",
                    )
                    != 0
                ):
                    return 1
            else:
                if (
                    expect(
                        list(real_full_doc.get("aggregate_summary_sanity_checked_profiles", [])) == ["full"],
                        "real_full_aggregate_summary_checked_profiles_mismatch",
                    )
                    != 0
                ):
                    return 1
            if (
                expect(
                    list(real_full_doc.get("aggregate_summary_sanity_failed_profiles", [])) == [],
                    "real_full_aggregate_summary_failed_profiles_mismatch",
                )
                != 0
            ):
                return 1
            real_full_aggregate_summary = real_full_doc.get("aggregate_summary_sanity_by_profile", {})
            if not isinstance(real_full_aggregate_summary, dict):
                return expect(False, "real_full_aggregate_summary_by_profile_missing")
            if (
                expect_aggregate_summary_sanity(
                    real_full_aggregate_summary.get("full"),
                    "full",
                    expected_profile_matrix_summary_values("full"),
                    expected_present=bool(not matrix_selftest_dry and not matrix_selftest_quick),
                    expected_gate_marker=True,
                    prefix="real_full",
                )
                != 0
            ):
                return 1
            real_full_total_elapsed_ms = max(0, int(real_full_doc.get("total_elapsed_ms", 0)))
            real_full_step_elapsed_ms = max(0, int(real_full_row.get("elapsed_ms", 0)))
            selftest_report["total_elapsed_ms"] = int(selftest_report.get("total_elapsed_ms", 0)) + real_full_total_elapsed_ms
            real_profiles = dict(selftest_report.get("real_profiles", {}))
            real_profiles["full"] = {
                "selected": True,
                "skipped": False,
                "status": "pass",
                "ok": True,
                "total_elapsed_ms": real_full_total_elapsed_ms,
                "step_elapsed_ms": real_full_step_elapsed_ms,
            }
            selftest_report["real_profiles"] = real_profiles
            aggregate_summary = dict(selftest_report.get("aggregate_summary_sanity_by_profile", {}))
            aggregate_summary["full"] = dict(real_full_aggregate_summary.get("full", {}))
            selftest_report["aggregate_summary_sanity_by_profile"] = aggregate_summary
            checked_profiles = [
                str(item).strip()
                for item in selftest_report.get("aggregate_summary_sanity_checked_profiles", [])
                if str(item).strip()
            ]
            failed_profiles = [
                str(item).strip()
                for item in selftest_report.get("aggregate_summary_sanity_failed_profiles", [])
                if str(item).strip()
            ]
            skipped_profiles = [
                str(item).strip()
                for item in selftest_report.get("aggregate_summary_sanity_skipped_profiles", [])
                if str(item).strip()
            ]
            checked_profiles = [item for item in checked_profiles if item != "full"]
            failed_profiles = [item for item in failed_profiles if item != "full"]
            skipped_profiles = [item for item in skipped_profiles if item != "full"]
            if matrix_selftest_dry or matrix_selftest_quick:
                skipped_profiles.append("full")
            else:
                checked_profiles.append("full")
            selftest_report["aggregate_summary_sanity_checked_profiles"] = checked_profiles
            selftest_report["aggregate_summary_sanity_failed_profiles"] = failed_profiles
            selftest_report["aggregate_summary_sanity_skipped_profiles"] = skipped_profiles
        else:
            print("[ci-profile-matrix-selftest] skip real profile=full by --real-profiles")

        if "seamgrim" in selected_real_profiles:
            real_seamgrim_report = report_dir / "matrix_real_seamgrim.detjson"
            real_seamgrim_gate_override = report_dir / "fake_seamgrim_real_gate.py"
            if not matrix_selftest_dry and matrix_selftest_lightweight_real:
                write_lightweight_profile_gate_override(real_seamgrim_gate_override, "seamgrim")
            real_seamgrim_env = None
            if not matrix_selftest_dry and not matrix_selftest_lightweight_real:
                real_seamgrim_env = dict(os.environ)
                real_seamgrim_env["DDN_CI_PROFILE_GATE_FULL_AGGREGATE"] = "1"
            real_seamgrim_start = time.perf_counter()
            proc_real_seamgrim = run_matrix(
                py=py,
                root=root,
                report_path=real_seamgrim_report,
                profiles="seamgrim",
                dry_run=matrix_selftest_dry,
                stop_on_fail=True,
                quick_gates=matrix_selftest_quick,
                full_aggregate_gates=matrix_full_aggregate_gates,
                with_profile_matrix_full_real_smoke=matrix_with_profile_matrix_full_real_smoke,
                env=real_seamgrim_env,
                profile_gate_overrides=(
                    {"seamgrim": str(real_seamgrim_gate_override)}
                    if not matrix_selftest_dry and matrix_selftest_lightweight_real
                    else None
                ),
            )
            real_seamgrim_elapsed = time.perf_counter() - real_seamgrim_start
            print(
                "[ci-profile-matrix-selftest] real profile=seamgrim elapsed_sec={:.3f} quick={} dry={}".format(
                    real_seamgrim_elapsed,
                    str(bool(matrix_selftest_quick)).lower(),
                    str(bool(matrix_selftest_dry)).lower(),
                )
            )
            if expect(proc_real_seamgrim.returncode == 0, "real_seamgrim_should_pass", proc_real_seamgrim) != 0:
                return 1
            if not matrix_selftest_dry:
                if (
                    expect_marker(
                        proc_real_seamgrim,
                        "ci_profile_seamgrim_status=pass",
                        "real_seamgrim_profile_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_seamgrim,
                        "[fixed64-darwin-schedule-policy] ok",
                        "real_seamgrim_fixed64_schedule_policy_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_seamgrim,
                        "[fixed64-darwin-real-report]",
                        "real_seamgrim_fixed64_real_report_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_seamgrim,
                        "seamgrim ci gate seed meta step check ok",
                        "real_seamgrim_seed_meta_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_seamgrim,
                        "seamgrim ci gate runtime5 passthrough check ok",
                        "real_seamgrim_runtime5_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_seamgrim,
                        "seamgrim featured seed catalog autogen check ok",
                        "real_seamgrim_featured_seed_catalog_autogen_check_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_seamgrim,
                        "seamgrim ci gate featured seed catalog step check ok",
                        "real_seamgrim_featured_seed_catalog_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_seamgrim,
                        "seamgrim ci gate featured seed catalog autogen step check ok",
                        "real_seamgrim_featured_seed_catalog_autogen_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_seamgrim,
                        "seamgrim ci gate lesson warning step check ok",
                        "real_seamgrim_lesson_warning_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_seamgrim,
                        "seamgrim ci gate stateful preview step check ok",
                        "real_seamgrim_stateful_preview_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_seamgrim,
                        "seamgrim interface boundary contract check ok",
                        "real_seamgrim_boundary_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_seamgrim,
                        "overlay compare diag parity check ok",
                        "real_seamgrim_overlay_compare_diag_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if (
                    expect_marker(
                        proc_real_seamgrim,
                        "[seamgrim-wasm-cli-diag-parity] ok",
                        "real_seamgrim_wasm_cli_diag_parity_marker_missing",
                    )
                    != 0
                ):
                    return 1
                if matrix_selftest_quick:
                    if (
                        expect_marker(
                            proc_real_seamgrim,
                            "[ci-profile-seamgrim] aggregate gate skipped by --quick",
                            "real_seamgrim_quick_skip_marker_missing",
                        )
                        != 0
                    ):
                        return 1
                else:
                    if (
                        expect_marker(
                            proc_real_seamgrim,
                            "[ci-gate-report-index-check] ok index=",
                            "real_seamgrim_index_contract_marker_missing",
                        )
                        != 0
                    ):
                        return 1
                    if (
                        expect_marker(
                            proc_real_seamgrim,
                            "[ci-gate-summary-report-check] ok status=pass",
                            "real_seamgrim_summary_contract_marker_missing",
                        )
                        != 0
                    ):
                        return 1
                    if (
                        expect_marker(
                            proc_real_seamgrim,
                            "[ci-profile-seamgrim] aggregate summary sanity markers ok",
                            "real_seamgrim_summary_sanity_markers_missing",
                        )
                        != 0
                    ):
                        return 1
                    if (
                        expect_marker(
                            proc_real_seamgrim,
                            "[ci-emit-artifacts-check] ok index=",
                            "real_seamgrim_emit_artifacts_contract_marker_missing",
                        )
                        != 0
                    ):
                        return 1
            if (
                expect_marker(
                    proc_real_seamgrim,
                    "ci_profile_matrix_status=pass",
                    "real_seamgrim_matrix_marker_missing",
                )
                != 0
            ):
                return 1
            real_seamgrim_doc = load_json(real_seamgrim_report)
            if expect(bool(real_seamgrim_doc.get("ok", False)), "real_seamgrim_ok_mismatch") != 0:
                return 1
            if expect(str(real_seamgrim_doc.get("step", "")) == "all", "real_seamgrim_step_mismatch") != 0:
                return 1
            if expect(int(len(real_seamgrim_doc.get("steps", []))) == 1, "real_seamgrim_steps_count_mismatch") != 0:
                return 1
            if expect(int(real_seamgrim_doc.get("total_elapsed_ms", -1)) >= 0, "real_seamgrim_total_elapsed_ms_mismatch") != 0:
                return 1
            real_seamgrim_row = dict(real_seamgrim_doc.get("steps", [{}])[0])
            if expect(int(real_seamgrim_row.get("elapsed_ms", -1)) >= 0, "real_seamgrim_row_elapsed_ms_mismatch") != 0:
                return 1
            if (
                expect(
                    int(real_seamgrim_doc.get("total_elapsed_ms", -1)) >= int(real_seamgrim_row.get("elapsed_ms", -1)),
                    "real_seamgrim_total_vs_row_elapsed_mismatch",
                )
                != 0
            ):
                return 1
            if matrix_selftest_quick:
                real_seamgrim_rows = real_seamgrim_doc.get("steps", [])
                real_seamgrim_cmd = (
                    real_seamgrim_rows[0].get("cmd", [])
                    if isinstance(real_seamgrim_rows, list) and real_seamgrim_rows
                    else []
                )
                if expect(
                    isinstance(real_seamgrim_cmd, list) and "--quick" in real_seamgrim_cmd,
                    "real_seamgrim_quick_cmd_flag_missing",
                ) != 0:
                    return 1
            if expect(bool(real_seamgrim_doc.get("aggregate_summary_sanity_ok", False)), "real_seamgrim_aggregate_summary_ok_mismatch") != 0:
                return 1
            if matrix_selftest_dry or matrix_selftest_quick:
                if (
                    expect(
                        list(real_seamgrim_doc.get("aggregate_summary_sanity_checked_profiles", [])) == [],
                        "real_seamgrim_aggregate_summary_checked_profiles_mismatch",
                    )
                    != 0
                ):
                    return 1
            else:
                if (
                    expect(
                        list(real_seamgrim_doc.get("aggregate_summary_sanity_checked_profiles", [])) == ["seamgrim"],
                        "real_seamgrim_aggregate_summary_checked_profiles_mismatch",
                    )
                    != 0
                ):
                    return 1
            if (
                expect(
                    list(real_seamgrim_doc.get("aggregate_summary_sanity_failed_profiles", [])) == [],
                    "real_seamgrim_aggregate_summary_failed_profiles_mismatch",
                )
                != 0
            ):
                return 1
            real_seamgrim_aggregate_summary = real_seamgrim_doc.get("aggregate_summary_sanity_by_profile", {})
            if not isinstance(real_seamgrim_aggregate_summary, dict):
                return expect(False, "real_seamgrim_aggregate_summary_by_profile_missing")
            if (
                expect_aggregate_summary_sanity(
                    real_seamgrim_aggregate_summary.get("seamgrim"),
                    "seamgrim",
                    expected_profile_matrix_summary_values("seamgrim"),
                    expected_present=bool(not matrix_selftest_dry and not matrix_selftest_quick),
                    expected_gate_marker=True,
                    prefix="real_seamgrim",
                )
                != 0
            ):
                return 1
            real_seamgrim_total_elapsed_ms = max(0, int(real_seamgrim_doc.get("total_elapsed_ms", 0)))
            real_seamgrim_step_elapsed_ms = max(0, int(real_seamgrim_row.get("elapsed_ms", 0)))
            selftest_report["total_elapsed_ms"] = (
                int(selftest_report.get("total_elapsed_ms", 0)) + real_seamgrim_total_elapsed_ms
            )
            real_profiles = dict(selftest_report.get("real_profiles", {}))
            real_profiles["seamgrim"] = {
                "selected": True,
                "skipped": False,
                "status": "pass",
                "ok": True,
                "total_elapsed_ms": real_seamgrim_total_elapsed_ms,
                "step_elapsed_ms": real_seamgrim_step_elapsed_ms,
            }
            selftest_report["real_profiles"] = real_profiles
            aggregate_summary = dict(selftest_report.get("aggregate_summary_sanity_by_profile", {}))
            aggregate_summary["seamgrim"] = dict(real_seamgrim_aggregate_summary.get("seamgrim", {}))
            selftest_report["aggregate_summary_sanity_by_profile"] = aggregate_summary
            checked_profiles = [
                str(item).strip()
                for item in selftest_report.get("aggregate_summary_sanity_checked_profiles", [])
                if str(item).strip()
            ]
            failed_profiles = [
                str(item).strip()
                for item in selftest_report.get("aggregate_summary_sanity_failed_profiles", [])
                if str(item).strip()
            ]
            skipped_profiles = [
                str(item).strip()
                for item in selftest_report.get("aggregate_summary_sanity_skipped_profiles", [])
                if str(item).strip()
            ]
            checked_profiles = [item for item in checked_profiles if item != "seamgrim"]
            failed_profiles = [item for item in failed_profiles if item != "seamgrim"]
            skipped_profiles = [item for item in skipped_profiles if item != "seamgrim"]
            if matrix_selftest_dry or matrix_selftest_quick:
                skipped_profiles.append("seamgrim")
            else:
                checked_profiles.append("seamgrim")
            selftest_report["aggregate_summary_sanity_checked_profiles"] = checked_profiles
            selftest_report["aggregate_summary_sanity_failed_profiles"] = failed_profiles
            selftest_report["aggregate_summary_sanity_skipped_profiles"] = skipped_profiles
        else:
            print("[ci-profile-matrix-selftest] skip real profile=seamgrim by --real-profiles")

    aggregate_summary_rows = selftest_report.get("aggregate_summary_sanity_by_profile", {})
    if isinstance(aggregate_summary_rows, dict):
        failed_profiles = [
            str(name)
            for name, row in aggregate_summary_rows.items()
            if isinstance(row, dict)
            and bool(row.get("expected_present", False))
            and not bool(row.get("ok", False))
        ]
        selftest_report["aggregate_summary_sanity_failed_profiles"] = failed_profiles
        selftest_report["aggregate_summary_sanity_ok"] = bool(len(failed_profiles) == 0)

    if args.json_out:
        write_json(Path(args.json_out), selftest_report)

    print(PROFILE_MATRIX_GATE_SELFTEST_OK_MARKER)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
