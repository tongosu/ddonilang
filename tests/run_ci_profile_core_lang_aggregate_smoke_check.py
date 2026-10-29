#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

from _ci_age5_combined_heavy_contract import (
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_REPORT_SCHEMA,
    AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA_TEXT,
    AGE5_COMBINED_HEAVY_REQUIRED_REPORTS_TEXT,
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
AGE4_PROOF_MARKER_PREFIX = "[ci-profile-core-lang-aggregate-smoke] age4 proof ok="
CORE_LANG_PROFILE_MATRIX_VALUES = format_profile_matrix_summary_values(
    expected_profile_matrix_summary_values("core_lang")
)
FULL_PROFILE_MATRIX_VALUES = format_profile_matrix_summary_values(
    expected_profile_matrix_summary_values("full")
)
SEAMGRIM_PROFILE_MATRIX_VALUES = format_profile_matrix_summary_values(
    expected_profile_matrix_summary_values("seamgrim")
)


CORE_LANG_AGGREGATE_SMOKE_CONTRACT_ONLY_STATUS = "ci_profile_core_lang_aggregate_smoke_status=pass aggregate_rc="
CORE_LANG_AGGREGATE_SMOKE_CONTRACT_ONLY_MODE = "mode=contract_only"
CORE_LANG_AGGREGATE_SMOKE_FULL_PASS_STATUS = "ci_profile_core_lang_aggregate_smoke_status=pass aggregate_rc=0 mode=full_pass"
CORE_LANG_AGGREGATE_SMOKE_FULL_FAIL_PREFIX = (
    "ci_profile_core_lang_aggregate_smoke_status=fail reason=aggregate_failed_in_full_mode aggregate_rc="
)
RUNTIME_HELPER_SUMMARY_SELFTEST_MISMATCH_ENV = "DDN_CI_PROFILE_AGGREGATE_SMOKE_FORCE_RUNTIME_HELPER_SUMMARY_MISMATCH"
RUNTIME_HELPER_SUMMARY_SELFTEST_MISMATCH_KEY_ENV = "DDN_CI_PROFILE_AGGREGATE_SMOKE_FORCE_RUNTIME_HELPER_SUMMARY_MISMATCH_KEY"
RUNTIME_HELPER_SUMMARY_SELFTEST_MARKER_PREFIX = (
    "[ci-profile-core-lang-aggregate-smoke] runtime helper summary selftest mismatch applied key="
)
GROUP_ID_SUMMARY_SELFTEST_MISMATCH_ENV = "DDN_CI_PROFILE_AGGREGATE_SMOKE_FORCE_GROUP_ID_SUMMARY_MISMATCH"
GROUP_ID_SUMMARY_SELFTEST_MARKER = (
    "[ci-profile-core-lang-aggregate-smoke] group_id summary selftest mismatch applied key=seamgrim_group_id_summary_status"
)


PROFILE_MATRIX_SUMMARY_REQUIRED_KEYS = (
    "ci_profile_matrix_gate_selftest_aggregate_summary_sanity_ok",
    "ci_profile_matrix_gate_selftest_aggregate_summary_checked_profiles",
    "ci_profile_matrix_gate_selftest_aggregate_summary_failed_profiles",
    "ci_profile_matrix_gate_selftest_aggregate_summary_skipped_profiles",
    "ci_profile_matrix_gate_selftest_core_lang_aggregate_summary_status",
    "ci_profile_matrix_gate_selftest_core_lang_aggregate_summary_ok",
    "ci_profile_matrix_gate_selftest_core_lang_aggregate_summary_values",
)
RUNTIME_HELPER_SUMMARY_EXPECTED = (
    ("ci_sanity_pipeline_emit_flags_ok", "1"),
    ("ci_sanity_pipeline_emit_flags_selftest_ok", "1"),
    ("ci_sanity_age5_combined_heavy_policy_selftest_ok", "1"),
    ("ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok", "1"),
    ("ci_sanity_fixed64_darwin_real_report_live_check_selftest_ok", "1"),
    ("ci_sanity_fixed64_threeway_inputs_selftest_ok", "1"),
    ("ci_sanity_age5_combined_heavy_report_schema", AGE5_COMBINED_HEAVY_REPORT_SCHEMA),
    ("ci_sanity_age5_combined_heavy_required_reports", AGE5_COMBINED_HEAVY_REQUIRED_REPORTS_TEXT),
    ("ci_sanity_age5_combined_heavy_required_criteria", AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA_TEXT),
    ("ci_sanity_age5_combined_heavy_child_summary_default_fields", AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT),
    ("ci_sanity_age5_combined_heavy_combined_contract_summary_fields", AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT),
    ("ci_sanity_age5_combined_heavy_full_summary_contract_fields", AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT),
    ("ci_sync_readiness_ci_sanity_pipeline_emit_flags_ok", "1"),
    ("ci_sync_readiness_ci_sanity_pipeline_emit_flags_selftest_ok", "1"),
    ("ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok", "1"),
    ("ci_sanity_seamgrim_numeric_factor_policy_ok", "na"),
    ("ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok", "na"),
    ("ci_sync_readiness_ci_sanity_age5_combined_heavy_policy_selftest_ok", "1"),
    ("ci_sync_readiness_ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok", "1"),
    ("ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_check_selftest_ok", "1"),
    ("ci_sync_readiness_ci_sanity_fixed64_threeway_inputs_selftest_ok", "1"),
    ("ci_sync_readiness_ci_sanity_age5_combined_heavy_report_schema", AGE5_COMBINED_HEAVY_REPORT_SCHEMA),
    ("ci_sync_readiness_ci_sanity_age5_combined_heavy_required_reports", AGE5_COMBINED_HEAVY_REQUIRED_REPORTS_TEXT),
    ("ci_sync_readiness_ci_sanity_age5_combined_heavy_required_criteria", AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA_TEXT),
    ("ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields", AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT),
    ("ci_sync_readiness_ci_sanity_age5_combined_heavy_combined_contract_summary_fields", AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT),
    ("ci_sync_readiness_ci_sanity_age5_combined_heavy_full_summary_contract_fields", AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT),
)


def profile_matrix_summary_ok(
    summary_kv: dict[str, str],
    profile: str,
    expected_values: str,
    *,
    full_aggregate: bool,
) -> bool:
    checked_profiles = "core_lang,full,seamgrim" if full_aggregate else profile
    skipped_profiles = "-" if full_aggregate else "full,seamgrim"
    return (
        summary_kv.get("ci_profile_matrix_gate_selftest_aggregate_summary_sanity_ok", "") == "1"
        and summary_kv.get("ci_profile_matrix_gate_selftest_aggregate_summary_checked_profiles", "") == checked_profiles
        and summary_kv.get("ci_profile_matrix_gate_selftest_aggregate_summary_failed_profiles", "") == "-"
        and summary_kv.get("ci_profile_matrix_gate_selftest_aggregate_summary_skipped_profiles", "") == skipped_profiles
        and summary_kv.get(f"ci_profile_matrix_gate_selftest_{profile}_aggregate_summary_status", "") == "pass"
        and summary_kv.get(f"ci_profile_matrix_gate_selftest_{profile}_aggregate_summary_ok", "") == "1"
        and summary_kv.get(f"ci_profile_matrix_gate_selftest_{profile}_aggregate_summary_values", "") == expected_values
        and (
            not full_aggregate
            or summary_kv.get("ci_profile_matrix_gate_selftest_full_aggregate_summary_values", "")
            == FULL_PROFILE_MATRIX_VALUES
        )
        and (
            not full_aggregate
            or summary_kv.get("ci_profile_matrix_gate_selftest_seamgrim_aggregate_summary_values", "")
            == SEAMGRIM_PROFILE_MATRIX_VALUES
        )
    )


def group_id_summary_ok(summary_kv: dict[str, str]) -> bool:
    return summary_kv.get("seamgrim_group_id_summary_status", "") == "ok"


def runtime_helper_summary_ok(summary_kv: dict[str, str]) -> bool:
    return all(summary_kv.get(key, "") == expected for key, expected in RUNTIME_HELPER_SUMMARY_EXPECTED)


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


def maybe_force_group_id_summary_mismatch(summary_kv: dict[str, str]) -> None:
    raw = str(os.environ.get(GROUP_ID_SUMMARY_SELFTEST_MISMATCH_ENV, "")).strip().lower()
    if raw not in {"1", "true", "yes", "on"}:
        return
    summary_kv["seamgrim_group_id_summary_status"] = "__forced_mismatch__"
    print(GROUP_ID_SUMMARY_SELFTEST_MARKER)


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


def main() -> int:
    parser = argparse.ArgumentParser(description="Run core_lang aggregate smoke check")
    parser.add_argument(
        "--full-aggregate",
        action="store_true",
        help="run heavy aggregate path instead of contract-only aggregate smoke path",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    profile = "core_lang"
    use_full_aggregate = bool(args.full_aggregate)

    with tempfile.TemporaryDirectory(prefix="ci_profile_core_lang_aggregate_smoke_") as td:
        report_dir = Path(td) / "reports"
        report_dir.mkdir(parents=True, exist_ok=True)
        prefix = "ci_profile_core_lang_aggregate_smoke"

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
            "--skip-5min-checklist",
        ]
        if use_full_aggregate:
            print("[ci-profile-core-lang-aggregate-smoke] mode=full")
            aggregate_cmd.extend(
                [
                    "--profile-matrix-selftest-real-profiles",
                    "core_lang,full,seamgrim",
                    "--profile-matrix-selftest-full-aggregate-gates",
                ]
            )
        else:
            print("[ci-profile-core-lang-aggregate-smoke] mode=contract_only")
            aggregate_cmd.extend(
                [
                    "--contract-only-aggregate",
                    "--profile-matrix-selftest-real-profiles",
                    profile,
                    "--profile-matrix-selftest-dry",
                    "--profile-matrix-selftest-quick",
                ]
            )

        aggregate_proc = run(aggregate_cmd, root)
        _, _ = emit(aggregate_proc)

        summary_report = report_dir / f"{prefix}.ci_gate_summary.txt"
        if not summary_report.exists():
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=summary_missing")
            return 1
        summary_kv = parse_summary(summary_report)
        if summary_kv.get("ci_sanity_gate_profile", "") != profile:
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=sanity_profile_marker_mismatch")
            return 1
        if summary_kv.get("ci_sync_readiness_sanity_profile", "") != profile:
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=sync_profile_marker_mismatch")
            return 1
        if summary_kv.get("ci_sanity_pack_golden_lang_consistency_ok", "") != "1":
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=lang_consistency_marker_mismatch")
            return 1
        if summary_kv.get("ci_sanity_pack_golden_metadata_ok", "") != "1":
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=pack_golden_metadata_marker_mismatch")
            return 1
        if summary_kv.get("ci_sanity_pack_golden_graph_export_ok", "") != "1":
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=pack_golden_graph_export_marker_mismatch")
            return 1
        if summary_kv.get("ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok", "") != "1":
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=sync_pack_golden_graph_export_marker_mismatch")
            return 1
        if summary_kv.get("ci_sanity_seamgrim_numeric_factor_policy_ok", "") != "na":
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=sanity_numeric_factor_policy_marker_mismatch")
            return 1
        if summary_kv.get("ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok", "") != "na":
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=sync_numeric_factor_policy_marker_mismatch")
            return 1
        if summary_kv.get("ci_sanity_canon_ast_dpack_ok", "") != "1":
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=canon_ast_dpack_marker_mismatch")
            return 1
        if summary_kv.get("ci_sanity_age5_combined_heavy_policy_selftest_ok", "") != "1":
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=age5_combined_heavy_policy_marker_mismatch")
            return 1
        if summary_kv.get("ci_sync_readiness_ci_sanity_age5_combined_heavy_policy_selftest_ok", "") != "1":
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=sync_age5_combined_heavy_policy_marker_mismatch")
            return 1
        if summary_kv.get("ci_sanity_age5_combined_heavy_report_schema", "") != AGE5_COMBINED_HEAVY_REPORT_SCHEMA:
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=age5_combined_heavy_report_schema_mismatch")
            return 1
        if summary_kv.get("ci_sanity_age5_combined_heavy_required_reports", "") != AGE5_COMBINED_HEAVY_REQUIRED_REPORTS_TEXT:
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=age5_combined_heavy_required_reports_mismatch")
            return 1
        if summary_kv.get("ci_sanity_age5_combined_heavy_required_criteria", "") != AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA_TEXT:
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=age5_combined_heavy_required_criteria_mismatch")
            return 1
        if summary_kv.get("ci_sanity_age5_combined_heavy_child_summary_default_fields", "") != AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT:
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=age5_combined_heavy_child_summary_default_fields_mismatch")
            return 1
        if summary_kv.get("ci_sanity_age5_combined_heavy_combined_contract_summary_fields", "") != AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT:
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=age5_combined_heavy_combined_contract_summary_fields_mismatch")
            return 1
        if summary_kv.get("ci_sanity_age5_combined_heavy_full_summary_contract_fields", "") != AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT:
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=age5_combined_heavy_full_summary_contract_fields_mismatch")
            return 1
        if summary_kv.get("ci_sync_readiness_ci_sanity_age5_combined_heavy_report_schema", "") != AGE5_COMBINED_HEAVY_REPORT_SCHEMA:
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=sync_age5_combined_heavy_report_schema_mismatch")
            return 1
        if summary_kv.get("ci_sync_readiness_ci_sanity_age5_combined_heavy_required_reports", "") != AGE5_COMBINED_HEAVY_REQUIRED_REPORTS_TEXT:
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=sync_age5_combined_heavy_required_reports_mismatch")
            return 1
        if summary_kv.get("ci_sync_readiness_ci_sanity_age5_combined_heavy_required_criteria", "") != AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA_TEXT:
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=sync_age5_combined_heavy_required_criteria_mismatch")
            return 1
        if summary_kv.get("ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields", "") != AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT:
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=sync_age5_combined_heavy_child_summary_default_fields_mismatch")
            return 1
        if summary_kv.get("ci_sync_readiness_ci_sanity_age5_combined_heavy_combined_contract_summary_fields", "") != AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT:
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=sync_age5_combined_heavy_combined_contract_summary_fields_mismatch")
            return 1
        if summary_kv.get("ci_sync_readiness_ci_sanity_age5_combined_heavy_full_summary_contract_fields", "") != AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT:
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=sync_age5_combined_heavy_full_summary_contract_fields_mismatch")
            return 1
        if not age4_proof_summary_ok(summary_kv):
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=aggregate_summary_age4_proof_mismatch")
            return 1
        print(age4_proof_summary_marker(summary_kv))
        maybe_force_runtime_helper_summary_mismatch(summary_kv)
        if not runtime_helper_summary_ok(summary_kv):
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=aggregate_summary_runtime_helper_contract_mismatch")
            return 1
        maybe_force_group_id_summary_mismatch(summary_kv)
        if not group_id_summary_ok(summary_kv):
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=aggregate_summary_group_id_summary_mismatch")
            return 1
        if not profile_matrix_summary_ok(
            summary_kv,
            profile,
            CORE_LANG_PROFILE_MATRIX_VALUES,
            full_aggregate=use_full_aggregate,
        ):
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=profile_matrix_aggregate_summary_contract_mismatch")
            return 1

        index_report = report_dir / f"{prefix}.ci_gate_report_index.detjson"
        index_doc = load_json(index_report)
        if not isinstance(index_doc, dict):
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=index_missing_or_invalid")
            return 1
        if str(index_doc.get("ci_sanity_profile", "")).strip() != profile:
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=index_profile_mismatch")
            return 1
        reports = index_doc.get("reports")
        if not isinstance(reports, dict):
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=index_reports_missing_or_invalid")
            return 1
        fixed64_threeway_inputs_raw = str(reports.get("fixed64_threeway_inputs", "")).strip()
        if not fixed64_threeway_inputs_raw:
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=fixed64_threeway_inputs_path_missing")
            return 1
        fixed64_threeway_inputs_path = Path(fixed64_threeway_inputs_raw.replace("\\", "/"))
        if not fixed64_threeway_inputs_path.exists():
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=fixed64_threeway_inputs_report_missing")
            return 1
        fixed64_threeway_inputs_doc = load_json(fixed64_threeway_inputs_path)
        if not isinstance(fixed64_threeway_inputs_doc, dict):
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=fixed64_threeway_inputs_report_invalid")
            return 1
        if str(fixed64_threeway_inputs_doc.get("schema", "")).strip() != "ddn.fixed64.threeway_inputs.v1":
            print("ci_profile_core_lang_aggregate_smoke_status=fail reason=fixed64_threeway_inputs_schema_mismatch")
            return 1

    if aggregate_proc.returncode != 0 and not use_full_aggregate:
        print(
            f"{CORE_LANG_AGGREGATE_SMOKE_CONTRACT_ONLY_STATUS}{aggregate_proc.returncode} "
            f"{CORE_LANG_AGGREGATE_SMOKE_CONTRACT_ONLY_MODE}"
        )
        return 0
    if aggregate_proc.returncode != 0:
        print(f"{CORE_LANG_AGGREGATE_SMOKE_FULL_FAIL_PREFIX}{aggregate_proc.returncode}")
        return aggregate_proc.returncode if aggregate_proc.returncode != 0 else 1

    if use_full_aggregate:
        print(CORE_LANG_AGGREGATE_SMOKE_FULL_PASS_STATUS)
    else:
        print(f"{CORE_LANG_AGGREGATE_SMOKE_CONTRACT_ONLY_STATUS}0 {CORE_LANG_AGGREGATE_SMOKE_CONTRACT_ONLY_MODE}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
