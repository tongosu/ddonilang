#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from contextlib import nullcontext
from pathlib import Path

from _ci_age5_combined_heavy_contract import (
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_REPORT_SCHEMA,
    AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA_TEXT,
    AGE5_COMBINED_HEAVY_REQUIRED_REPORTS_TEXT,
)
from _ci_profile_matrix_full_real_smoke_contract import (
    PROFILE_MATRIX_FULL_REAL_SMOKE_ALLOW_FLAG,
    PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SCRIPT,
    PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STATUS_PASS,
)
from _ci_profile_matrix_selftest_lib import (
    expected_profile_matrix_summary_values,
    format_profile_matrix_summary_values,
)


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


def emit(proc: subprocess.CompletedProcess[str]) -> tuple[str, str]:
    stdout = (proc.stdout or "").strip()
    stderr = (proc.stderr or "").strip()
    if stdout:
        print(stdout)
    if stderr:
        print(stderr)
    return stdout, stderr


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def parse_summary(path: Path) -> dict[str, str]:
    kv: dict[str, str] = {}
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line.startswith("[ci-gate-summary] "):
            continue
        body = line[len("[ci-gate-summary] ") :]
        if "=" not in body:
            continue
        key, value = body.split("=", 1)
        key = key.strip()
        value = value.strip()
        if key:
            kv[key] = value
    return kv


AGE4_PROOF_OK_KEY = "age4_proof_ok"
AGE4_PROOF_FAILED_CRITERIA_KEY = "age4_proof_failed_criteria"
AGE4_PROOF_FAILED_PREVIEW_KEY = "age4_proof_failed_preview"
AGE4_PROOF_SUMMARY_HASH_KEY = "age4_proof_summary_hash"
AGE4_PROOF_MARKER_PREFIX = "[ci-profile-seamgrim] aggregate age4 proof ok="
CORE_LANG_PROFILE_MATRIX_VALUES = format_profile_matrix_summary_values(
    expected_profile_matrix_summary_values("core_lang")
)
FULL_PROFILE_MATRIX_VALUES = format_profile_matrix_summary_values(
    expected_profile_matrix_summary_values("full")
)
SEAMGRIM_PROFILE_MATRIX_VALUES = format_profile_matrix_summary_values(
    expected_profile_matrix_summary_values("seamgrim")
)
PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY = "DDN_CI_PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_SEC"
PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_PROFILE_ENV_KEY = (
    "DDN_CI_PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_SEC_SEAMGRIM"
)
PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC = 1500.0


PROFILE_MATRIX_SUMMARY_REQUIRED_KEYS = (
    "ci_profile_matrix_gate_selftest_aggregate_summary_sanity_ok",
    "ci_profile_matrix_gate_selftest_aggregate_summary_checked_profiles",
    "ci_profile_matrix_gate_selftest_aggregate_summary_failed_profiles",
    "ci_profile_matrix_gate_selftest_aggregate_summary_skipped_profiles",
    "ci_profile_matrix_gate_selftest_seamgrim_aggregate_summary_status",
    "ci_profile_matrix_gate_selftest_seamgrim_aggregate_summary_ok",
    "ci_profile_matrix_gate_selftest_seamgrim_aggregate_summary_values",
)
RUNTIME_HELPER_SUMMARY_SELFTEST_MISMATCH_ENV = "DDN_CI_PROFILE_GATE_FORCE_RUNTIME_HELPER_SUMMARY_MISMATCH"
RUNTIME_HELPER_SUMMARY_SELFTEST_MISMATCH_KEY_ENV = "DDN_CI_PROFILE_GATE_FORCE_RUNTIME_HELPER_SUMMARY_MISMATCH_KEY"
RUNTIME_HELPER_SUMMARY_SELFTEST_MARKER_PREFIX = "[ci-profile-seamgrim] runtime helper summary selftest mismatch applied key="
GROUP_ID_SUMMARY_SELFTEST_MISMATCH_ENV = "DDN_CI_PROFILE_GATE_FORCE_GROUP_ID_SUMMARY_MISMATCH"
GROUP_ID_SUMMARY_SELFTEST_MARKER_PREFIX = "[ci-profile-seamgrim] group_id summary selftest mismatch applied key=seamgrim_group_id_summary_status"
RUNTIME_HELPER_SUMMARY_EXPECTED = (
    ("ci_sanity_pipeline_emit_flags_ok", "na"),
    ("ci_sanity_pipeline_emit_flags_selftest_ok", "na"),
    ("ci_sanity_age5_combined_heavy_policy_selftest_ok", "1"),
    ("ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok", "1"),
    ("ci_sanity_dynamic_source_profile_split_selftest_ok", "1"),
    ("ci_sanity_fixed64_darwin_real_report_live_check_selftest_ok", "1"),
    ("ci_sanity_fixed64_threeway_inputs_selftest_ok", "1"),
    ("ci_sanity_age5_combined_heavy_report_schema", AGE5_COMBINED_HEAVY_REPORT_SCHEMA),
    ("ci_sanity_age5_combined_heavy_required_reports", AGE5_COMBINED_HEAVY_REQUIRED_REPORTS_TEXT),
    ("ci_sanity_age5_combined_heavy_required_criteria", AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA_TEXT),
    ("ci_sanity_age5_combined_heavy_child_summary_default_fields", AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT),
    ("ci_sanity_age5_combined_heavy_combined_contract_summary_fields", AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT),
    ("ci_sanity_age5_combined_heavy_full_summary_contract_fields", AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT),
    ("ci_sync_readiness_ci_sanity_pipeline_emit_flags_ok", "na"),
    ("ci_sync_readiness_ci_sanity_pipeline_emit_flags_selftest_ok", "na"),
    ("ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok", "0"),
    ("ci_sanity_seamgrim_numeric_factor_policy_ok", "1"),
    ("ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok", "1"),
    ("ci_sync_readiness_ci_sanity_age5_combined_heavy_policy_selftest_ok", "1"),
    ("ci_sync_readiness_ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok", "1"),
    ("ci_sync_readiness_ci_sanity_dynamic_source_profile_split_selftest_ok", "1"),
    ("ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_check_selftest_ok", "1"),
    ("ci_sync_readiness_ci_sanity_fixed64_threeway_inputs_selftest_ok", "1"),
    ("ci_sync_readiness_ci_sanity_age5_combined_heavy_report_schema", AGE5_COMBINED_HEAVY_REPORT_SCHEMA),
    ("ci_sync_readiness_ci_sanity_age5_combined_heavy_required_reports", AGE5_COMBINED_HEAVY_REQUIRED_REPORTS_TEXT),
    ("ci_sync_readiness_ci_sanity_age5_combined_heavy_required_criteria", AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA_TEXT),
    ("ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields", AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT),
    ("ci_sync_readiness_ci_sanity_age5_combined_heavy_combined_contract_summary_fields", AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT),
    ("ci_sync_readiness_ci_sanity_age5_combined_heavy_full_summary_contract_fields", AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT),
)


def profile_matrix_summary_ok(summary_kv: dict[str, str], profile: str, expected_values: str) -> bool:
    return (
        summary_kv.get("ci_profile_matrix_gate_selftest_aggregate_summary_sanity_ok", "") == "1"
        and summary_kv.get("ci_profile_matrix_gate_selftest_aggregate_summary_checked_profiles", "") == "core_lang,full,seamgrim"
        and summary_kv.get("ci_profile_matrix_gate_selftest_aggregate_summary_failed_profiles", "") == "-"
        and summary_kv.get("ci_profile_matrix_gate_selftest_aggregate_summary_skipped_profiles", "") == "-"
        and summary_kv.get("ci_profile_matrix_gate_selftest_core_lang_aggregate_summary_status", "") == "pass"
        and summary_kv.get("ci_profile_matrix_gate_selftest_core_lang_aggregate_summary_ok", "") == "1"
        and summary_kv.get("ci_profile_matrix_gate_selftest_core_lang_aggregate_summary_values", "")
        == CORE_LANG_PROFILE_MATRIX_VALUES
        and summary_kv.get("ci_profile_matrix_gate_selftest_full_aggregate_summary_status", "") == "pass"
        and summary_kv.get("ci_profile_matrix_gate_selftest_full_aggregate_summary_ok", "") == "1"
        and summary_kv.get("ci_profile_matrix_gate_selftest_full_aggregate_summary_values", "")
        == FULL_PROFILE_MATRIX_VALUES
        and summary_kv.get(f"ci_profile_matrix_gate_selftest_{profile}_aggregate_summary_status", "") == "pass"
        and summary_kv.get(f"ci_profile_matrix_gate_selftest_{profile}_aggregate_summary_ok", "") == "1"
        and summary_kv.get(f"ci_profile_matrix_gate_selftest_{profile}_aggregate_summary_values", "") == expected_values
    )


def runtime_helper_summary_ok(summary_kv: dict[str, str]) -> bool:
    return all(summary_kv.get(key, "") == expected for key, expected in RUNTIME_HELPER_SUMMARY_EXPECTED)


def group_id_summary_ok(summary_kv: dict[str, str]) -> bool:
    return summary_kv.get("seamgrim_group_id_summary_status", "") == "ok"


def age4_proof_summary_ok(summary_kv: dict[str, str]) -> bool:
    ok_value = summary_kv.get(AGE4_PROOF_OK_KEY, "")
    failed_value = summary_kv.get(AGE4_PROOF_FAILED_CRITERIA_KEY, "")
    failed_preview = summary_kv.get(AGE4_PROOF_FAILED_PREVIEW_KEY, "")
    summary_hash = summary_kv.get(AGE4_PROOF_SUMMARY_HASH_KEY, "")
    if ok_value not in {"0", "1"}:
        return False
    try:
        failed_num = int(str(failed_value))
    except Exception:
        return False
    if failed_num < 0:
        return False
    return bool(str(failed_preview).strip()) and bool(str(summary_hash).strip())


def age4_proof_summary_marker(summary_kv: dict[str, str]) -> str:
    return (
        f"{AGE4_PROOF_MARKER_PREFIX}{summary_kv.get(AGE4_PROOF_OK_KEY, '')} "
        f"failed={summary_kv.get(AGE4_PROOF_FAILED_CRITERIA_KEY, '')} "
        f"preview={summary_kv.get(AGE4_PROOF_FAILED_PREVIEW_KEY, '')} "
        f"hash={summary_kv.get(AGE4_PROOF_SUMMARY_HASH_KEY, '')}"
    )


def maybe_force_runtime_helper_summary_mismatch(summary_kv: dict[str, str]) -> None:
    raw = str(os.environ.get(RUNTIME_HELPER_SUMMARY_SELFTEST_MISMATCH_ENV, "")).strip().lower()
    if raw not in {"1", "true", "yes", "on"}:
        return
    known_keys = {key for key, _ in RUNTIME_HELPER_SUMMARY_EXPECTED}
    target_key = str(os.environ.get(RUNTIME_HELPER_SUMMARY_SELFTEST_MISMATCH_KEY_ENV, "")).strip()
    if target_key not in known_keys:
        target_key = RUNTIME_HELPER_SUMMARY_EXPECTED[0][0]
    summary_kv[target_key] = "__forced_mismatch__"
    print(f"{RUNTIME_HELPER_SUMMARY_SELFTEST_MARKER_PREFIX}{target_key}")


def maybe_force_group_id_summary_mismatch(summary_kv: dict[str, str]) -> None:
    raw = str(os.environ.get(GROUP_ID_SUMMARY_SELFTEST_MISMATCH_ENV, "")).strip().lower()
    if raw not in {"1", "true", "yes", "on"}:
        return
    summary_kv["seamgrim_group_id_summary_status"] = "__forced_mismatch__"
    print(GROUP_ID_SUMMARY_SELFTEST_MARKER_PREFIX)


def truthy_env(name: str) -> bool:
    return str(os.environ.get(name, "")).strip().lower() in {"1", "true", "yes", "on"}


def resolve_profile_matrix_full_real_smoke_step_timeout(arg_value: float | None) -> tuple[float, str]:
    def parse_timeout(raw: str) -> float | None:
        text = str(raw).strip()
        if not text:
            return None
        try:
            parsed = float(text)
        except Exception:
            return None
        return max(0.0, parsed)

    if arg_value is not None:
        return max(0.0, float(arg_value)), "cli_arg"
    profile_env = parse_timeout(os.environ.get(PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_PROFILE_ENV_KEY, ""))
    if profile_env is not None:
        return profile_env, PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_PROFILE_ENV_KEY
    common_env = parse_timeout(os.environ.get(PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY, ""))
    if common_env is not None:
        return common_env, PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY
    return PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC, "default_policy"


def main() -> int:
    parser = argparse.ArgumentParser(description="Run CI profile gate for seamgrim")
    parser.add_argument(
        "--quick",
        action="store_true",
        help="skip aggregate gate/index contract stage (sanity + sync-readiness only)",
    )
    parser.add_argument(
        "--full-aggregate",
        action="store_true",
        help="run heavy aggregate gate path instead of contract-only aggregate smoke path",
    )
    parser.add_argument(
        "--with-profile-matrix-full-real-smoke",
        action="store_true",
        help="after heavy aggregate success, run profile-matrix full-real 3-profile smoke",
    )
    parser.add_argument(
        "--profile-matrix-full-real-smoke-step-timeout-sec",
        type=float,
        default=None,
        help="optional per-profile timeout(sec) forwarded to profile-matrix full-real smoke check",
    )
    parser.add_argument("--report-dir", default="", help="optional external report directory")
    parser.add_argument("--report-prefix", default="", help="optional external report prefix")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    profile = "seamgrim"
    use_full_aggregate = bool(args.full_aggregate or truthy_env("DDN_CI_PROFILE_GATE_FULL_AGGREGATE"))
    use_profile_matrix_full_real_smoke = bool(
        args.with_profile_matrix_full_real_smoke
        or truthy_env(PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY)
    )
    (
        profile_matrix_full_real_smoke_step_timeout_sec,
        profile_matrix_full_real_smoke_step_timeout_source,
    ) = resolve_profile_matrix_full_real_smoke_step_timeout(
        args.profile_matrix_full_real_smoke_step_timeout_sec
    )
    sync_profile_marker = "sanity_profile=seamgrim"
    sanity_required_markers = (
        "ci_sanity_status=pass",
        f"profile={profile}",
        "[fixed64-darwin-schedule-policy] ok",
        "[fixed64-darwin-real-report]",
        "[fixed64-darwin-readiness-selftest] ok",
        "ci profile matrix lightweight contract selftest ok",
        "ci profile matrix snapshot helper selftest ok",
        "seamgrim ci gate seed meta step check ok",
        "seamgrim featured seed catalog autogen check ok",
        "seamgrim ci gate featured seed catalog step check ok",
        "seamgrim ci gate featured seed catalog autogen step check ok",
        "seamgrim ci gate runtime5 passthrough check ok",
        "seamgrim ci gate lesson warning step check ok",
        "seamgrim ci gate stateful preview step check ok",
        "seamgrim interface boundary contract check ok",
        "overlay session wired consistency check ok",
        "overlay session diag parity check ok",
        "overlay compare diag parity check ok",
        "[seamgrim-wasm-cli-diag-parity] ok",
    )

    temp_ctx = (
        tempfile.TemporaryDirectory(prefix="ci_profile_seamgrim_gate_")
        if not args.report_dir.strip()
        else nullcontext(None)
    )
    with temp_ctx as td:
        report_dir = Path(args.report_dir) if args.report_dir.strip() else (Path(td) / "reports")
        report_dir.mkdir(parents=True, exist_ok=True)
        prefix = args.report_prefix.strip() or "ci_profile_seamgrim"
        sanity_report = report_dir / f"{prefix}.ci_sanity_gate.detjson"
        report = report_dir / f"{prefix}.ci_sync_readiness.detjson"

        proc = run(
            [
                py,
                "tests/run_ci_sanity_gate.py",
                "--profile",
                profile,
                "--json-out",
                str(sanity_report),
            ],
            root,
        )
        stdout, _ = emit(proc)
        if proc.returncode != 0:
            print("ci_profile_seamgrim_status=fail reason=sanity_gate_failed")
            return proc.returncode
        if any(marker not in stdout for marker in sanity_required_markers):
            print("ci_profile_seamgrim_status=fail reason=pass_marker_missing")
            return 1

        sync_proc = run(
            [
                py,
                "tests/run_ci_sync_readiness_check.py",
                "--report-dir",
                str(report_dir),
                "--report-prefix",
                prefix,
                "--json-out",
                str(report),
                "--validate-only-sanity-json",
                str(sanity_report),
                "--skip-aggregate",
                "--sanity-profile",
                profile,
            ],
            root,
        )
        sync_stdout, _ = emit(sync_proc)
        if sync_proc.returncode != 0:
            print("ci_profile_seamgrim_status=fail reason=sync_readiness_failed")
            return sync_proc.returncode
        if "ci_sync_readiness_status=pass" not in sync_stdout or sync_profile_marker not in sync_stdout:
            print("ci_profile_seamgrim_status=fail reason=sync_readiness_pass_marker_missing")
            return 1

        report_proc = run(
            [
                py,
                "tests/run_ci_sync_readiness_report_check.py",
                "--report",
                str(report),
                "--require-pass",
                "--sanity-profile",
                profile,
            ],
            root,
        )
        _, _ = emit(report_proc)
        if report_proc.returncode != 0:
            print("ci_profile_seamgrim_status=fail reason=sync_readiness_report_check_failed")
            return report_proc.returncode

        if use_profile_matrix_full_real_smoke:
            print("[ci-profile-seamgrim] profile-matrix full-real smoke enabled")
            if profile_matrix_full_real_smoke_step_timeout_sec > 0.0:
                print(
                    "[ci-profile-seamgrim] profile-matrix full-real smoke step timeout sec="
                    f"{profile_matrix_full_real_smoke_step_timeout_sec:g} "
                    f"source={profile_matrix_full_real_smoke_step_timeout_source}"
                )
        if use_profile_matrix_full_real_smoke and (args.quick or truthy_env("DDN_CI_PROFILE_GATE_SKIP_AGGREGATE")):
            print("ci_profile_seamgrim_status=fail reason=profile_matrix_full_real_smoke_requires_aggregate")
            return 1
        if use_profile_matrix_full_real_smoke and not use_full_aggregate:
            print("ci_profile_seamgrim_status=fail reason=profile_matrix_full_real_smoke_requires_full_aggregate")
            return 1

        if args.quick or truthy_env("DDN_CI_PROFILE_GATE_SKIP_AGGREGATE"):
            if args.quick:
                print("[ci-profile-seamgrim] aggregate gate skipped by --quick")
            else:
                print("[ci-profile-seamgrim] aggregate gate skipped by DDN_CI_PROFILE_GATE_SKIP_AGGREGATE=1")
            print("ci_profile_seamgrim_status=pass")
            return 0

        if use_full_aggregate:
            print("[ci-profile-seamgrim] aggregate gate uses full mode")
        else:
            print("[ci-profile-seamgrim] aggregate gate uses contract-only mode")

        aggregate_cmd = [
            py,
            "tests/run_ci_aggregate_gate.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            prefix,
            "--ci-sanity-profile",
            profile,
            "--skip-core-tests",
            "--compact-step-logs",
            "--quiet-success-logs",
            "--profile-matrix-selftest-real-profiles",
            "core_lang,full,seamgrim",
        ]
        if use_full_aggregate:
            aggregate_cmd.append("--profile-matrix-selftest-full-aggregate-gates")
        if use_profile_matrix_full_real_smoke:
            aggregate_cmd.append("--profile-matrix-selftest-with-profile-matrix-full-real-smoke")
        if not use_full_aggregate:
            aggregate_cmd.append("--profile-matrix-selftest-dry")
        if not use_full_aggregate:
            aggregate_cmd.append("--contract-only-aggregate")
        aggregate_proc = run(aggregate_cmd, root)
        _, _ = emit(aggregate_proc)
        if aggregate_proc.returncode != 0:
            print("ci_profile_seamgrim_status=fail reason=aggregate_gate_failed")
            return aggregate_proc.returncode

        index_report = report_dir / f"{prefix}.ci_gate_report_index.detjson"
        summary_report = report_dir / f"{prefix}.ci_gate_summary.txt"
        index_proc = run(
            [
                py,
                "tests/run_ci_gate_report_index_check.py",
                "--index",
                str(index_report),
                "--sanity-profile",
                profile,
                "--enforce-profile-step-contract",
            ],
            root,
        )
        _, _ = emit(index_proc)
        if index_proc.returncode != 0:
            print("ci_profile_seamgrim_status=fail reason=aggregate_index_contract_failed")
            return index_proc.returncode
        index_doc = load_json(index_report)
        if not isinstance(index_doc, dict):
            print("ci_profile_seamgrim_status=fail reason=aggregate_index_json_invalid")
            return 1
        reports = index_doc.get("reports")
        if not isinstance(reports, dict):
            print("ci_profile_seamgrim_status=fail reason=aggregate_index_reports_missing_or_invalid")
            return 1
        fixed64_threeway_inputs_raw = str(reports.get("fixed64_threeway_inputs", "")).strip()
        if not fixed64_threeway_inputs_raw:
            print("ci_profile_seamgrim_status=fail reason=aggregate_fixed64_threeway_inputs_path_missing")
            return 1
        fixed64_threeway_inputs_path = Path(fixed64_threeway_inputs_raw.replace("\\", "/"))
        if not fixed64_threeway_inputs_path.exists():
            print("ci_profile_seamgrim_status=fail reason=aggregate_fixed64_threeway_inputs_report_missing")
            return 1
        fixed64_threeway_inputs_doc = load_json(fixed64_threeway_inputs_path)
        if not isinstance(fixed64_threeway_inputs_doc, dict):
            print("ci_profile_seamgrim_status=fail reason=aggregate_fixed64_threeway_inputs_report_invalid")
            return 1
        if str(fixed64_threeway_inputs_doc.get("schema", "")).strip() != "ddn.fixed64.threeway_inputs.v1":
            print("ci_profile_seamgrim_status=fail reason=aggregate_fixed64_threeway_inputs_schema_mismatch")
            return 1

        summary_proc = run(
            [
                py,
                "tests/run_ci_gate_summary_report_check.py",
                "--summary",
                str(summary_report),
                "--index",
                str(index_report),
                "--require-pass",
            ],
            root,
        )
        _, _ = emit(summary_proc)
        if summary_proc.returncode != 0:
            print("ci_profile_seamgrim_status=fail reason=aggregate_summary_contract_failed")
            return summary_proc.returncode
        if not summary_report.exists():
            print("ci_profile_seamgrim_status=fail reason=aggregate_summary_missing")
            return 1
        summary_kv = parse_summary(summary_report)
        if summary_kv.get("ci_sanity_gate_profile", "") != profile:
            print("ci_profile_seamgrim_status=fail reason=aggregate_summary_sanity_profile_mismatch")
            return 1
        if summary_kv.get("ci_sync_readiness_sanity_profile", "") != profile:
            print("ci_profile_seamgrim_status=fail reason=aggregate_summary_sync_profile_mismatch")
            return 1
        if summary_kv.get("ci_sanity_pack_golden_lang_consistency_ok", "") != "0":
            print("ci_profile_seamgrim_status=fail reason=aggregate_summary_lang_consistency_mismatch")
            return 1
        if summary_kv.get("ci_sanity_pack_golden_metadata_ok", "") != "0":
            print("ci_profile_seamgrim_status=fail reason=aggregate_summary_pack_golden_metadata_mismatch")
            return 1
        if summary_kv.get("ci_sanity_pack_golden_graph_export_ok", "") != "0":
            print("ci_profile_seamgrim_status=fail reason=aggregate_summary_pack_golden_graph_export_mismatch")
            return 1
        if summary_kv.get("ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok", "") != "0":
            print("ci_profile_seamgrim_status=fail reason=aggregate_summary_sync_pack_golden_graph_export_mismatch")
            return 1
        if summary_kv.get("ci_sanity_seamgrim_numeric_factor_policy_ok", "") != "1":
            print("ci_profile_seamgrim_status=fail reason=aggregate_summary_sanity_numeric_factor_policy_mismatch")
            return 1
        if summary_kv.get("ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok", "") != "1":
            print("ci_profile_seamgrim_status=fail reason=aggregate_summary_sync_numeric_factor_policy_mismatch")
            return 1
        if summary_kv.get("ci_sanity_canon_ast_dpack_ok", "") != "0":
            print("ci_profile_seamgrim_status=fail reason=aggregate_summary_canon_ast_dpack_mismatch")
            return 1
        if not age4_proof_summary_ok(summary_kv):
            print("ci_profile_seamgrim_status=fail reason=aggregate_summary_age4_proof_mismatch")
            return 1
        print(age4_proof_summary_marker(summary_kv))
        maybe_force_runtime_helper_summary_mismatch(summary_kv)
        if not runtime_helper_summary_ok(summary_kv):
            print("ci_profile_seamgrim_status=fail reason=aggregate_summary_runtime_helper_contract_mismatch")
            return 1
        maybe_force_group_id_summary_mismatch(summary_kv)
        if not group_id_summary_ok(summary_kv):
            print("ci_profile_seamgrim_status=fail reason=aggregate_summary_group_id_summary_mismatch")
            return 1
        if not profile_matrix_summary_ok(summary_kv, profile, SEAMGRIM_PROFILE_MATRIX_VALUES):
            print("ci_profile_seamgrim_status=fail reason=aggregate_summary_profile_matrix_contract_mismatch")
            return 1
        print("[ci-profile-seamgrim] aggregate summary sanity markers ok")

        emit_artifacts_proc = run(
            [
                py,
                "tests/run_ci_emit_artifacts_check.py",
                "--report-dir",
                str(report_dir),
                "--prefix",
                prefix,
                "--require-brief",
                "--require-triage",
                "--allow-triage-exists-upgrade",
            ],
            root,
        )
        _, _ = emit(emit_artifacts_proc)
        if emit_artifacts_proc.returncode != 0:
            print("ci_profile_seamgrim_status=fail reason=aggregate_emit_artifacts_contract_failed")
            return emit_artifacts_proc.returncode

        if use_profile_matrix_full_real_smoke:
            smoke_cmd = [py, PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SCRIPT, PROFILE_MATRIX_FULL_REAL_SMOKE_ALLOW_FLAG]
            if profile_matrix_full_real_smoke_step_timeout_sec > 0.0:
                smoke_cmd.extend(
                    ["--step-timeout-sec", str(profile_matrix_full_real_smoke_step_timeout_sec)]
                )
            smoke_proc = run(smoke_cmd, root)
            smoke_stdout, _ = emit(smoke_proc)
            if smoke_proc.returncode != 0:
                print("ci_profile_seamgrim_status=fail reason=profile_matrix_full_real_smoke_failed")
                return smoke_proc.returncode
            if PROFILE_MATRIX_FULL_REAL_SMOKE_STATUS_PASS not in smoke_stdout:
                print("ci_profile_seamgrim_status=fail reason=profile_matrix_full_real_smoke_marker_missing")
                return 1

        print("ci_profile_seamgrim_status=pass")
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
