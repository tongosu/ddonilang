#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from _ci_age5_combined_heavy_contract import (
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY,
    AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
    AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_DEFAULT,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS,
    AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_ENV_KEY,
    AGE5_COMBINED_HEAVY_FULL_REAL_SOURCE_TRACE_TEXT,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_MODE,
    AGE5_COMBINED_HEAVY_REPORT_SCHEMA,
    AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA,
    AGE5_COMBINED_HEAVY_REQUIRED_REPORTS,
    AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT,
    AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_DEFAULT,
    AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_KEY,
    AGE5_FULL_REAL_CORE_LANG_SANITY_ELAPSED_FIELDS_TEXT,
    AGE5_FULL_REAL_ELAPSED_FIELDS_TEXT,
    AGE5_FULL_REAL_FIXED64_DARWIN_REAL_REPORT_READINESS_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROBE_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_ELAPSED_MAP_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_STATUS_MAP_FIELDS_TEXT,
    AGE5_FULL_REAL_TIMEOUT_BREAKDOWN_FIELDS_TEXT,
    build_age5_combined_heavy_child_summary_fields,
    build_age5_combined_heavy_child_summary_default_fields,
    build_age5_combined_heavy_combined_report_contract_fields,
    build_age5_combined_heavy_child_summary_default_text_transport_fields,
    build_age5_combined_heavy_full_real_source_trace,
    build_age5_combined_heavy_full_real_source_trace_text,
    build_age5_combined_heavy_full_summary_contract_fields,
    build_age5_combined_heavy_full_summary_text_transport_fields,
    build_age5_combined_heavy_timeout_policy_fields,
    build_age4_proof_snapshot,
    build_age4_proof_source_snapshot_fields,
    build_age4_proof_snapshot_text,
    build_age5_close_digest_selftest_default_field,
    build_age5_full_real_core_lang_sanity_elapsed_summary,
    build_age5_full_real_elapsed_summary,
    build_age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress,
    build_age5_full_real_pipeline_emit_flags_selftest_progress,
    build_age5_full_real_pipeline_emit_flags_selftest_probe,
    build_age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress,
    build_age5_full_real_profile_elapsed_map,
    build_age5_full_real_profile_status_map,
    build_age5_full_real_timeout_breakdown,
)


ROOT = Path(__file__).resolve().parent.parent
CONTRACT_SCHEMA = "ddn.age5.combined_heavy_real_report_contract.v1"


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def make_report(*, overall_ok: bool = True) -> dict:
    age4_proof_snapshot = build_age4_proof_snapshot(
        age4_proof_ok="0",
        age4_proof_failed_criteria="-1",
        age4_proof_failed_preview="-",
    )
    report = {
        "schema": AGE5_COMBINED_HEAVY_REPORT_SCHEMA,
        "with_combined_heavy_runtime_helper_check": True,
        "combined_heavy_env_enabled": True,
        "overall_ok": overall_ok,
        AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
        AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_KEY: build_age5_close_digest_selftest_default_field(),
        "age4_proof_snapshot_fields_text": AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
        "age4_proof_snapshot_text": build_age4_proof_snapshot_text(age4_proof_snapshot),
        "policy_contract": {
            "env_key": AGE5_COMBINED_HEAVY_ENV_KEY,
            "scope": AGE5_COMBINED_HEAVY_MODE,
            "combined_report_schema": AGE5_COMBINED_HEAVY_REPORT_SCHEMA,
            "full_real_source_trace_text": AGE5_COMBINED_HEAVY_FULL_REAL_SOURCE_TRACE_TEXT,
            "full_real_smoke_check_script": "tests/run_ci_profile_matrix_full_real_smoke_check.py",
            "full_real_smoke_check_selftest_script": "tests/run_ci_profile_matrix_full_real_smoke_check_selftest.py",
            "combined_required_reports": list(AGE5_COMBINED_HEAVY_REQUIRED_REPORTS),
            "combined_required_criteria": list(AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA),
            "combined_child_summary_keys": list(AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS),
            "combined_child_summary_default_fields": build_age5_combined_heavy_child_summary_default_fields(),
            "combined_child_summary_default_fields_text": AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT,
            "combined_timeout_policy_fields": build_age5_combined_heavy_timeout_policy_fields(),
            AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_KEY: AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_DEFAULT,
            AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_KEY: AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT,
            AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_DEFAULT,
            AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
            AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_KEY: build_age5_close_digest_selftest_default_field(),
            "combined_child_summary_default_text_transport_fields": (
                build_age5_combined_heavy_child_summary_default_text_transport_fields()
            ),
            "combined_child_summary_default_text_transport_fields_text": (
                AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS_TEXT
            ),
            **build_age4_proof_snapshot(),
            "age4_proof_source_snapshot_fields_text": AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
            "age4_proof_snapshot_text": build_age4_proof_snapshot_text(build_age4_proof_snapshot()),
            **build_age4_proof_source_snapshot_fields(top_snapshot=build_age4_proof_snapshot()),
            "combined_contract_summary_fields": build_age5_combined_heavy_combined_report_contract_fields(),
            "combined_contract_summary_fields_text": AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT,
            "combined_full_summary_contract_fields": build_age5_combined_heavy_full_summary_contract_fields(),
            "combined_full_summary_contract_fields_text": AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT,
            "combined_full_summary_text_transport_fields": build_age5_combined_heavy_full_summary_text_transport_fields(),
            "combined_full_summary_text_transport_fields_text": AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FIELDS_TEXT,
            "age4_proof_snapshot_fields_text": AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
        },
        "criteria": [
            {"name": name, "ok": True, "detail": f"{name} ok"}
            for name in AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA
        ],
        "reports": {
            "full_real": "build/reports/age5_close.full_real.detjson",
            "runtime_helper_negative": "build/reports/age5_close.runtime_helper_negative.detjson",
            "group_id_summary_negative": "build/reports/age5_close.group_id_summary_negative.detjson",
        },
        "full_real_source_trace": build_age5_combined_heavy_full_real_source_trace(
            smoke_check_script_exists=True,
            smoke_check_selftest_script_exists=True,
        ),
        "age5_combined_heavy_timeout_policy_ok": "1",
        AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_KEY: AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT,
        "age5_full_real_core_lang_sanity_elapsed_fields_text": AGE5_FULL_REAL_CORE_LANG_SANITY_ELAPSED_FIELDS_TEXT,
        "age5_full_real_elapsed_fields_text": AGE5_FULL_REAL_ELAPSED_FIELDS_TEXT,
        "age5_full_real_pipeline_emit_flags_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_pipeline_emit_flags_selftest_probe_fields_text": (
            AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROBE_FIELDS_TEXT
        ),
        "age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_fields_text": (
            AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_fields_text": (
            AGE5_FULL_REAL_FIXED64_DARWIN_REAL_REPORT_READINESS_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT
        ),
        "age5_full_real_profile_elapsed_map_fields_text": AGE5_FULL_REAL_PROFILE_ELAPSED_MAP_FIELDS_TEXT,
        "age5_full_real_profile_status_map_fields_text": AGE5_FULL_REAL_PROFILE_STATUS_MAP_FIELDS_TEXT,
        "age5_full_real_timeout_breakdown_fields_text": AGE5_FULL_REAL_TIMEOUT_BREAKDOWN_FIELDS_TEXT,
    }
    report.update(build_age5_full_real_core_lang_sanity_elapsed_summary())
    report.update(build_age5_full_real_elapsed_summary())
    report.update(build_age5_full_real_pipeline_emit_flags_selftest_progress())
    report.update(build_age5_full_real_pipeline_emit_flags_selftest_probe())
    report.update(build_age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress())
    report.update(build_age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress())
    report.update(build_age5_full_real_profile_elapsed_map())
    report.update(build_age5_full_real_profile_status_map())
    report.update(build_age5_full_real_timeout_breakdown())
    report.update(age4_proof_snapshot)
    report.update(
        build_age4_proof_source_snapshot_fields(
            top_snapshot=age4_proof_snapshot,
            gate_result_snapshot=age4_proof_snapshot,
            gate_result_present=True,
            final_status_parse_snapshot=age4_proof_snapshot,
            final_status_parse_present=True,
        )
    )
    report["full_real_source_trace_text"] = build_age5_combined_heavy_full_real_source_trace_text(
        report["full_real_source_trace"]
    )
    report.update(build_age5_combined_heavy_combined_report_contract_fields())
    report.update(build_age5_combined_heavy_full_summary_contract_fields())
    report.update(build_age5_combined_heavy_full_summary_text_transport_fields())
    report.update(build_age5_combined_heavy_child_summary_default_text_transport_fields())
    report.update(
        build_age5_combined_heavy_child_summary_fields(
            full_real_ok=True,
            runtime_helper_negative_ok=True,
            group_id_summary_negative_ok=True,
        )
    )
    return report


def run_check(report: Path, json_out: Path, *extra: str) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_age5_combined_heavy_real_report_contract_check.py",
        "--report",
        str(report),
        "--json-out",
        str(json_out),
        *extra,
    ]
    return subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="age5_combined_heavy_real_report_contract_") as tmp:
        base = Path(tmp)
        report = base / "age5_close_report.detjson"
        json_out = base / "age5_combined_heavy_real_report_contract.detjson"

        write_json(report, make_report(overall_ok=True))
        pass_proc = run_check(report, json_out)
        if pass_proc.returncode != 0:
            print("[age5-combined-heavy-real-report-selftest] fail: pass case rc!=0")
            print(pass_proc.stdout.strip())
            print(pass_proc.stderr.strip())
            return 1
        pass_doc = load_json(json_out)
        if not isinstance(pass_doc, dict) or str(pass_doc.get("schema", "")).strip() != CONTRACT_SCHEMA:
            print("[age5-combined-heavy-real-report-selftest] fail: pass contract doc invalid")
            return 1
        if str(pass_doc.get("status", "")).strip() != "pass":
            print("[age5-combined-heavy-real-report-selftest] fail: pass status mismatch")
            return 1

        bad_report = make_report(overall_ok=True)
        bad_report["ci_sanity_age5_combined_heavy_full_summary_contract_fields"] = "BROKEN"
        write_json(report, bad_report)
        bad_proc = run_check(report, json_out)
        if bad_proc.returncode == 0:
            print("[age5-combined-heavy-real-report-selftest] fail: bad contract case should fail")
            return 1

        bad_default_transport_report = make_report(overall_ok=True)
        bad_default_transport_report["ci_sanity_age5_combined_heavy_child_summary_default_fields"] = "BROKEN"
        write_json(report, bad_default_transport_report)
        bad_default_transport_proc = run_check(report, json_out)
        if bad_default_transport_proc.returncode == 0:
            print("[age5-combined-heavy-real-report-selftest] fail: bad default transport case should fail")
            return 1

        bad_age4_source_report = make_report(overall_ok=True)
        bad_age4_source_report[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY] = "0"
        write_json(report, bad_age4_source_report)
        bad_age4_source_proc = run_check(report, json_out)
        if bad_age4_source_proc.returncode == 0:
            print("[age5-combined-heavy-real-report-selftest] fail: bad age4 source parity case should fail")
            return 1

        fail_ok_report = make_report(overall_ok=False)
        write_json(report, fail_ok_report)
        allow_fail_proc = run_check(report, json_out, "--allow-overall-fail")
        if allow_fail_proc.returncode != 0:
            print("[age5-combined-heavy-real-report-selftest] fail: allow-overall-fail case rc!=0")
            print(allow_fail_proc.stdout.strip())
            print(allow_fail_proc.stderr.strip())
            return 1

    print("[age5-combined-heavy-real-report-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
