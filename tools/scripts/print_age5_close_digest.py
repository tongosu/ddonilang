#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TESTS_DIR = ROOT / "tests"
import sys
if str(TESTS_DIR) not in sys.path:
    sys.path.insert(0, str(TESTS_DIR))

from _ci_age5_combined_heavy_contract import (  # type: ignore
    AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY,
    AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY,
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY,
    AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY,
    AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
    AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY,
    AGE5_FULL_REAL_CORE_LANG_SANITY_ELAPSED_FIELDS_TEXT,
    AGE5_FULL_REAL_CORE_LANG_SANITY_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_ELAPSED_FIELDS_TEXT,
    AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROBE_FIELDS_TEXT,
    AGE5_FULL_REAL_FIXED64_DARWIN_REAL_REPORT_READINESS_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_MAP_ACCESS_CONTRACT_CHECK_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_CI_PACK_GOLDEN_EXEC_POLICY_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_CI_PACK_GOLDEN_AGE5_SURFACE_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_CI_PACK_GOLDEN_GUIDEBLOCK_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_CI_PACK_GOLDEN_JJAIM_FLATTEN_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_CI_PACK_GOLDEN_EVENT_MODEL_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_CI_PACK_GOLDEN_LANG_CONSISTENCY_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_W94_SOCIAL_PACK_CHECK_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_W95_CERT_PACK_CHECK_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_W96_SOMSSI_PACK_CHECK_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_W97_SELF_HEAL_PACK_CHECK_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_TENSOR_V0_CLI_CHECK_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SELFTEST_PROGRESS_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_ELAPSED_MAP_FIELDS_TEXT,
    AGE5_FULL_REAL_PROFILE_STATUS_MAP_FIELDS_TEXT,
    AGE5_FULL_REAL_TIMEOUT_BREAKDOWN_FIELDS_TEXT,
    build_age4_proof_snapshot,
    build_age4_proof_source_snapshot_fields,
    build_age4_proof_snapshot_text,
    build_age5_combined_heavy_full_real_source_trace,
    build_age5_close_digest_selftest_default_field,
    build_age5_full_real_core_lang_sanity_elapsed_summary,
    build_age5_full_real_elapsed_summary,
    build_age5_full_real_pipeline_emit_flags_progress,
    build_age5_full_real_pipeline_emit_flags_selftest_progress,
    build_age5_full_real_pipeline_emit_flags_selftest_probe,
    build_age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress,
    build_age5_full_real_map_access_contract_check_progress,
    build_age5_full_real_ci_pack_golden_exec_policy_selftest_progress,
    build_age5_full_real_ci_pack_golden_age5_surface_selftest_progress,
    build_age5_full_real_ci_pack_golden_guideblock_selftest_progress,
    build_age5_full_real_ci_pack_golden_jjaim_flatten_selftest_progress,
    build_age5_full_real_ci_pack_golden_event_model_selftest_progress,
    build_age5_full_real_ci_pack_golden_lang_consistency_selftest_progress,
    build_age5_full_real_w94_social_pack_check_progress,
    build_age5_full_real_w95_cert_pack_check_progress,
    build_age5_full_real_w96_somssi_pack_check_progress,
    build_age5_full_real_w97_self_heal_pack_check_progress,
    build_age5_full_real_tensor_v0_cli_check_progress,
    build_age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress,
    build_age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress,
    build_age5_full_real_profile_elapsed_map,
    build_age5_full_real_profile_status_map,
    build_age5_full_real_timeout_breakdown,
)

# diagnostics token anchors:
# combined_digest_selftest_default_field_text=
# combined_digest_selftest_default_field=


def load_payload(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def main() -> int:
    parser = argparse.ArgumentParser(description="Print digest from ddn.age5_close_report.v1")
    parser.add_argument("report", help="path to age5 close report detjson")
    parser.add_argument("--top", type=int, default=8, help="max failure digest lines")
    parser.add_argument("--only-failed", action="store_true", help="print digest only when overall_ok=false")
    args = parser.parse_args()

    path = Path(args.report)
    payload = load_payload(path)
    if payload is None:
        print(f"[age5-close] report missing_or_invalid: {path}")
        return 0

    overall_ok = bool(payload.get("overall_ok", False))
    criteria = payload.get("criteria")
    total = len(criteria) if isinstance(criteria, list) else 0
    failed = (
        sum(1 for row in criteria if isinstance(row, dict) and not bool(row.get("ok", False)))
        if isinstance(criteria, list)
        else 0
    )
    full_real_status = str(payload.get("age5_combined_heavy_full_real_status", "skipped")).strip() or "skipped"
    runtime_helper_negative_status = (
        str(payload.get("age5_combined_heavy_runtime_helper_negative_status", "skipped")).strip() or "skipped"
    )
    group_id_summary_negative_status = (
        str(payload.get("age5_combined_heavy_group_id_summary_negative_status", "skipped")).strip() or "skipped"
    )
    combined_heavy_child_timeout_sec = str(payload.get("combined_heavy_child_timeout_sec", "0")).strip() or "0"
    combined_heavy_timeout_present = (
        str(payload.get("age5_combined_heavy_timeout_present", "0")).strip() or "0"
    )
    combined_heavy_timeout_targets = (
        str(payload.get("age5_combined_heavy_timeout_targets", "-")).strip() or "-"
    )
    combined_heavy_timeout_mode = (
        str(payload.get(AGE5_COMBINED_HEAVY_TIMEOUT_MODE_KEY, AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED)).strip()
        or AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED
    )
    full_real_elapsed_fields_text = (
        str(payload.get("age5_full_real_elapsed_fields_text", AGE5_FULL_REAL_ELAPSED_FIELDS_TEXT)).strip()
        or AGE5_FULL_REAL_ELAPSED_FIELDS_TEXT
    )
    full_real_elapsed_summary = build_age5_full_real_elapsed_summary(
        age5_full_real_total_elapsed_ms=payload.get("age5_full_real_total_elapsed_ms", "-"),
        age5_full_real_slowest_profile=payload.get("age5_full_real_slowest_profile", "-"),
        age5_full_real_slowest_elapsed_ms=payload.get("age5_full_real_slowest_elapsed_ms", "-"),
        age5_full_real_elapsed_present=str(payload.get("age5_full_real_elapsed_present", "0")).strip() == "1",
    )
    full_real_core_lang_sanity_elapsed_fields_text = (
        str(
            payload.get(
                "age5_full_real_core_lang_sanity_elapsed_fields_text",
                AGE5_FULL_REAL_CORE_LANG_SANITY_ELAPSED_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_CORE_LANG_SANITY_ELAPSED_FIELDS_TEXT
    )
    full_real_core_lang_sanity_elapsed_summary = build_age5_full_real_core_lang_sanity_elapsed_summary(
        age5_full_real_core_lang_sanity_total_elapsed_ms=payload.get(
            "age5_full_real_core_lang_sanity_total_elapsed_ms", "-"
        ),
        age5_full_real_core_lang_sanity_slowest_step=payload.get(
            "age5_full_real_core_lang_sanity_slowest_step", "-"
        ),
        age5_full_real_core_lang_sanity_slowest_elapsed_ms=payload.get(
            "age5_full_real_core_lang_sanity_slowest_elapsed_ms", "-"
        ),
        age5_full_real_core_lang_sanity_elapsed_present=(
            str(payload.get("age5_full_real_core_lang_sanity_elapsed_present", "0")).strip() == "1"
        ),
    )
    full_real_core_lang_sanity_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_core_lang_sanity_progress_fields_text",
                AGE5_FULL_REAL_CORE_LANG_SANITY_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_CORE_LANG_SANITY_PROGRESS_FIELDS_TEXT
    )
    full_real_core_lang_sanity_current_step = (
        str(payload.get("age5_full_real_core_lang_sanity_current_step", "-")).strip() or "-"
    )
    full_real_core_lang_sanity_last_completed_step = (
        str(payload.get("age5_full_real_core_lang_sanity_last_completed_step", "-")).strip() or "-"
    )
    full_real_core_lang_sanity_progress_present = (
        str(payload.get("age5_full_real_core_lang_sanity_progress_present", "0")).strip() or "0"
    )
    full_real_pipeline_emit_flags_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_pipeline_emit_flags_progress_fields_text",
                AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_PROGRESS_FIELDS_TEXT
    )
    full_real_pipeline_emit_flags_progress = build_age5_full_real_pipeline_emit_flags_progress(
        age5_full_real_pipeline_emit_flags_current_section=payload.get(
            "age5_full_real_pipeline_emit_flags_current_section", "-"
        ),
        age5_full_real_pipeline_emit_flags_last_completed_section=payload.get(
            "age5_full_real_pipeline_emit_flags_last_completed_section", "-"
        ),
        age5_full_real_pipeline_emit_flags_total_elapsed_ms=payload.get(
            "age5_full_real_pipeline_emit_flags_total_elapsed_ms", "-"
        ),
        age5_full_real_pipeline_emit_flags_progress_present=(
            str(payload.get("age5_full_real_pipeline_emit_flags_progress_present", "0")).strip() == "1"
        ),
    )
    full_real_pipeline_emit_flags_selftest_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_pipeline_emit_flags_selftest_progress_fields_text",
                AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROGRESS_FIELDS_TEXT
    )
    full_real_pipeline_emit_flags_selftest_progress = (
        build_age5_full_real_pipeline_emit_flags_selftest_progress(
            age5_full_real_pipeline_emit_flags_selftest_current_case=payload.get(
                "age5_full_real_pipeline_emit_flags_selftest_current_case", "-"
            ),
            age5_full_real_pipeline_emit_flags_selftest_last_completed_case=payload.get(
                "age5_full_real_pipeline_emit_flags_selftest_last_completed_case", "-"
            ),
            age5_full_real_pipeline_emit_flags_selftest_total_elapsed_ms=payload.get(
                "age5_full_real_pipeline_emit_flags_selftest_total_elapsed_ms", "-"
            ),
            age5_full_real_pipeline_emit_flags_selftest_progress_present=(
                str(payload.get("age5_full_real_pipeline_emit_flags_selftest_progress_present", "0")).strip()
                == "1"
            ),
        )
    )
    full_real_pipeline_emit_flags_selftest_probe_fields_text = (
        str(
            payload.get(
                "age5_full_real_pipeline_emit_flags_selftest_probe_fields_text",
                AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROBE_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_PIPELINE_EMIT_FLAGS_SELFTEST_PROBE_FIELDS_TEXT
    )
    full_real_pipeline_emit_flags_selftest_probe = (
        build_age5_full_real_pipeline_emit_flags_selftest_probe(
            age5_full_real_pipeline_emit_flags_selftest_current_probe=payload.get(
                "age5_full_real_pipeline_emit_flags_selftest_current_probe", "-"
            ),
            age5_full_real_pipeline_emit_flags_selftest_last_completed_probe=payload.get(
                "age5_full_real_pipeline_emit_flags_selftest_last_completed_probe", "-"
            ),
            age5_full_real_pipeline_emit_flags_selftest_probe_present=(
                str(payload.get("age5_full_real_pipeline_emit_flags_selftest_probe_present", "0")).strip()
                == "1"
            ),
        )
    )
    full_real_age5_combined_policy_selftest_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_age5_combined_policy_selftest_progress_fields_text",
                "age5_full_real_age5_combined_policy_selftest_current_case=-|"
                "age5_full_real_age5_combined_policy_selftest_last_completed_case=-|"
                "age5_full_real_age5_combined_policy_selftest_current_format=-|"
                "age5_full_real_age5_combined_policy_selftest_last_completed_format=-|"
                "age5_full_real_age5_combined_policy_selftest_current_probe=-|"
                "age5_full_real_age5_combined_policy_selftest_last_completed_probe=-|"
                "age5_full_real_age5_combined_policy_selftest_total_elapsed_ms=-|"
                "age5_full_real_age5_combined_policy_selftest_progress_present=0",
            )
        ).strip()
        or "age5_full_real_age5_combined_policy_selftest_current_case=-|"
        "age5_full_real_age5_combined_policy_selftest_last_completed_case=-|"
        "age5_full_real_age5_combined_policy_selftest_current_format=-|"
        "age5_full_real_age5_combined_policy_selftest_last_completed_format=-|"
        "age5_full_real_age5_combined_policy_selftest_current_probe=-|"
        "age5_full_real_age5_combined_policy_selftest_last_completed_probe=-|"
        "age5_full_real_age5_combined_policy_selftest_total_elapsed_ms=-|"
        "age5_full_real_age5_combined_policy_selftest_progress_present=0"
    )
    full_real_age5_combined_policy_selftest_current_case = (
        str(payload.get("age5_full_real_age5_combined_policy_selftest_current_case", "-")).strip() or "-"
    )
    full_real_age5_combined_policy_selftest_last_completed_case = (
        str(payload.get("age5_full_real_age5_combined_policy_selftest_last_completed_case", "-")).strip() or "-"
    )
    full_real_age5_combined_policy_selftest_current_format = (
        str(payload.get("age5_full_real_age5_combined_policy_selftest_current_format", "-")).strip() or "-"
    )
    full_real_age5_combined_policy_selftest_last_completed_format = (
        str(payload.get("age5_full_real_age5_combined_policy_selftest_last_completed_format", "-")).strip() or "-"
    )
    full_real_age5_combined_policy_selftest_current_probe = (
        str(payload.get("age5_full_real_age5_combined_policy_selftest_current_probe", "-")).strip() or "-"
    )
    full_real_age5_combined_policy_selftest_last_completed_probe = (
        str(payload.get("age5_full_real_age5_combined_policy_selftest_last_completed_probe", "-")).strip() or "-"
    )
    full_real_age5_combined_policy_selftest_total_elapsed_ms = (
        str(payload.get("age5_full_real_age5_combined_policy_selftest_total_elapsed_ms", "-")).strip() or "-"
    )
    full_real_age5_combined_policy_selftest_progress_present = (
        str(payload.get("age5_full_real_age5_combined_policy_selftest_progress_present", "0")).strip() or "0"
    )
    full_real_profile_matrix_full_real_smoke_policy_selftest_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_fields_text",
                AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SELFTEST_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SELFTEST_PROGRESS_FIELDS_TEXT
    )
    full_real_profile_matrix_full_real_smoke_policy_selftest_progress = (
        build_age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress(
            age5_full_real_profile_matrix_full_real_smoke_policy_selftest_current_case=payload.get(
                "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_current_case", "-"
            ),
            age5_full_real_profile_matrix_full_real_smoke_policy_selftest_last_completed_case=payload.get(
                "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_last_completed_case", "-"
            ),
            age5_full_real_profile_matrix_full_real_smoke_policy_selftest_current_format=payload.get(
                "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_current_format", "-"
            ),
            age5_full_real_profile_matrix_full_real_smoke_policy_selftest_last_completed_format=payload.get(
                "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_last_completed_format", "-"
            ),
            age5_full_real_profile_matrix_full_real_smoke_policy_selftest_total_elapsed_ms=payload.get(
                "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_total_elapsed_ms", "-"
            ),
            age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_present=(
                str(
                    payload.get(
                        "age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_present",
                        "0",
                    )
                ).strip()
                == "1"
            ),
        )
    )
    full_real_profile_matrix_full_real_smoke_check_selftest_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_fields_text",
                AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT
    )
    full_real_profile_matrix_full_real_smoke_check_selftest_progress = (
        build_age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress(
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_case=payload.get(
                "age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_case", "-"
            ),
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_case=payload.get(
                "age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_case", "-"
            ),
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_total_elapsed_ms=payload.get(
                "age5_full_real_profile_matrix_full_real_smoke_check_selftest_total_elapsed_ms", "-"
            ),
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_probe=payload.get(
                "age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_probe", "-"
            ),
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_probe=payload.get(
                "age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_probe", "-"
            ),
            age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_present=(
                str(
                    payload.get(
                        "age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_present",
                        "0",
                    )
                ).strip()
                == "1"
            ),
        )
    )
    full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_fields_text",
                AGE5_FULL_REAL_FIXED64_DARWIN_REAL_REPORT_READINESS_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_FIXED64_DARWIN_REAL_REPORT_READINESS_CHECK_SELFTEST_PROGRESS_FIELDS_TEXT
    )
    full_real_fixed64_darwin_real_report_readiness_check_selftest_progress = (
        build_age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress(
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_case=payload.get(
                "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_case", "-"
            ),
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_case=payload.get(
                "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_case", "-"
            ),
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_total_elapsed_ms=payload.get(
                "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_total_elapsed_ms", "-"
            ),
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_probe=payload.get(
                "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_probe", "-"
            ),
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_probe=payload.get(
                "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_probe", "-"
            ),
            age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_present=(
                str(
                    payload.get(
                        "age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_present",
                        "0",
                    )
                ).strip()
                == "1"
            ),
        )
    )
    full_real_map_access_contract_check_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_map_access_contract_check_progress_fields_text",
                AGE5_FULL_REAL_MAP_ACCESS_CONTRACT_CHECK_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_MAP_ACCESS_CONTRACT_CHECK_PROGRESS_FIELDS_TEXT
    )
    full_real_map_access_contract_check_progress = (
        build_age5_full_real_map_access_contract_check_progress(
            age5_full_real_map_access_contract_check_current_case=payload.get(
                "age5_full_real_map_access_contract_check_current_case", "-"
            ),
            age5_full_real_map_access_contract_check_last_completed_case=payload.get(
                "age5_full_real_map_access_contract_check_last_completed_case", "-"
            ),
            age5_full_real_map_access_contract_check_total_elapsed_ms=payload.get(
                "age5_full_real_map_access_contract_check_total_elapsed_ms", "-"
            ),
            age5_full_real_map_access_contract_check_current_probe=payload.get(
                "age5_full_real_map_access_contract_check_current_probe", "-"
            ),
            age5_full_real_map_access_contract_check_last_completed_probe=payload.get(
                "age5_full_real_map_access_contract_check_last_completed_probe", "-"
            ),
            age5_full_real_map_access_contract_check_progress_present=(
                str(payload.get("age5_full_real_map_access_contract_check_progress_present", "0")).strip()
                == "1"
            ),
        )
    )
    full_real_tensor_v0_cli_check_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_tensor_v0_cli_check_progress_fields_text",
                AGE5_FULL_REAL_TENSOR_V0_CLI_CHECK_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_TENSOR_V0_CLI_CHECK_PROGRESS_FIELDS_TEXT
    )
    full_real_tensor_v0_cli_check_progress = (
        build_age5_full_real_tensor_v0_cli_check_progress(
            age5_full_real_tensor_v0_cli_check_current_case=payload.get(
                "age5_full_real_tensor_v0_cli_check_current_case", "-"
            ),
            age5_full_real_tensor_v0_cli_check_last_completed_case=payload.get(
                "age5_full_real_tensor_v0_cli_check_last_completed_case", "-"
            ),
            age5_full_real_tensor_v0_cli_check_total_elapsed_ms=payload.get(
                "age5_full_real_tensor_v0_cli_check_total_elapsed_ms", "-"
            ),
            age5_full_real_tensor_v0_cli_check_current_probe=payload.get(
                "age5_full_real_tensor_v0_cli_check_current_probe", "-"
            ),
            age5_full_real_tensor_v0_cli_check_last_completed_probe=payload.get(
                "age5_full_real_tensor_v0_cli_check_last_completed_probe", "-"
            ),
            age5_full_real_tensor_v0_cli_check_progress_present=(
                str(payload.get("age5_full_real_tensor_v0_cli_check_progress_present", "0")).strip()
                == "1"
            ),
        )
    )
    full_real_ci_pack_golden_exec_policy_selftest_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_ci_pack_golden_exec_policy_selftest_progress_fields_text",
                AGE5_FULL_REAL_CI_PACK_GOLDEN_EXEC_POLICY_SELFTEST_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_CI_PACK_GOLDEN_EXEC_POLICY_SELFTEST_PROGRESS_FIELDS_TEXT
    )
    full_real_ci_pack_golden_exec_policy_selftest_progress = (
        build_age5_full_real_ci_pack_golden_exec_policy_selftest_progress(
            age5_full_real_ci_pack_golden_exec_policy_selftest_current_case=payload.get(
                "age5_full_real_ci_pack_golden_exec_policy_selftest_current_case", "-"
            ),
            age5_full_real_ci_pack_golden_exec_policy_selftest_last_completed_case=payload.get(
                "age5_full_real_ci_pack_golden_exec_policy_selftest_last_completed_case", "-"
            ),
            age5_full_real_ci_pack_golden_exec_policy_selftest_total_elapsed_ms=payload.get(
                "age5_full_real_ci_pack_golden_exec_policy_selftest_total_elapsed_ms", "-"
            ),
            age5_full_real_ci_pack_golden_exec_policy_selftest_current_probe=payload.get(
                "age5_full_real_ci_pack_golden_exec_policy_selftest_current_probe", "-"
            ),
            age5_full_real_ci_pack_golden_exec_policy_selftest_last_completed_probe=payload.get(
                "age5_full_real_ci_pack_golden_exec_policy_selftest_last_completed_probe", "-"
            ),
            age5_full_real_ci_pack_golden_exec_policy_selftest_progress_present=(
                str(payload.get("age5_full_real_ci_pack_golden_exec_policy_selftest_progress_present", "0")).strip()
                == "1"
            ),
        )
    )
    full_real_ci_pack_golden_age5_surface_selftest_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_ci_pack_golden_age5_surface_selftest_progress_fields_text",
                AGE5_FULL_REAL_CI_PACK_GOLDEN_AGE5_SURFACE_SELFTEST_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_CI_PACK_GOLDEN_AGE5_SURFACE_SELFTEST_PROGRESS_FIELDS_TEXT
    )
    full_real_ci_pack_golden_age5_surface_selftest_progress = (
        build_age5_full_real_ci_pack_golden_age5_surface_selftest_progress(
            age5_full_real_ci_pack_golden_age5_surface_selftest_current_case=payload.get(
                "age5_full_real_ci_pack_golden_age5_surface_selftest_current_case", "-"
            ),
            age5_full_real_ci_pack_golden_age5_surface_selftest_last_completed_case=payload.get(
                "age5_full_real_ci_pack_golden_age5_surface_selftest_last_completed_case", "-"
            ),
            age5_full_real_ci_pack_golden_age5_surface_selftest_total_elapsed_ms=payload.get(
                "age5_full_real_ci_pack_golden_age5_surface_selftest_total_elapsed_ms", "-"
            ),
            age5_full_real_ci_pack_golden_age5_surface_selftest_current_probe=payload.get(
                "age5_full_real_ci_pack_golden_age5_surface_selftest_current_probe", "-"
            ),
            age5_full_real_ci_pack_golden_age5_surface_selftest_last_completed_probe=payload.get(
                "age5_full_real_ci_pack_golden_age5_surface_selftest_last_completed_probe", "-"
            ),
            age5_full_real_ci_pack_golden_age5_surface_selftest_progress_present=(
                str(
                    payload.get(
                        "age5_full_real_ci_pack_golden_age5_surface_selftest_progress_present",
                        "0",
                    )
                ).strip()
                == "1"
            ),
        )
    )
    full_real_ci_pack_golden_guideblock_selftest_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_ci_pack_golden_guideblock_selftest_progress_fields_text",
                AGE5_FULL_REAL_CI_PACK_GOLDEN_GUIDEBLOCK_SELFTEST_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_CI_PACK_GOLDEN_GUIDEBLOCK_SELFTEST_PROGRESS_FIELDS_TEXT
    )
    full_real_ci_pack_golden_guideblock_selftest_progress = (
        build_age5_full_real_ci_pack_golden_guideblock_selftest_progress(
            age5_full_real_ci_pack_golden_guideblock_selftest_current_case=payload.get(
                "age5_full_real_ci_pack_golden_guideblock_selftest_current_case", "-"
            ),
            age5_full_real_ci_pack_golden_guideblock_selftest_last_completed_case=payload.get(
                "age5_full_real_ci_pack_golden_guideblock_selftest_last_completed_case", "-"
            ),
            age5_full_real_ci_pack_golden_guideblock_selftest_total_elapsed_ms=payload.get(
                "age5_full_real_ci_pack_golden_guideblock_selftest_total_elapsed_ms", "-"
            ),
            age5_full_real_ci_pack_golden_guideblock_selftest_current_probe=payload.get(
                "age5_full_real_ci_pack_golden_guideblock_selftest_current_probe", "-"
            ),
            age5_full_real_ci_pack_golden_guideblock_selftest_last_completed_probe=payload.get(
                "age5_full_real_ci_pack_golden_guideblock_selftest_last_completed_probe", "-"
            ),
            age5_full_real_ci_pack_golden_guideblock_selftest_progress_present=(
                str(payload.get("age5_full_real_ci_pack_golden_guideblock_selftest_progress_present", "0")).strip()
                == "1"
            ),
        )
    )
    full_real_ci_pack_golden_jjaim_flatten_selftest_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_ci_pack_golden_jjaim_flatten_selftest_progress_fields_text",
                AGE5_FULL_REAL_CI_PACK_GOLDEN_JJAIM_FLATTEN_SELFTEST_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_CI_PACK_GOLDEN_JJAIM_FLATTEN_SELFTEST_PROGRESS_FIELDS_TEXT
    )
    full_real_ci_pack_golden_jjaim_flatten_selftest_progress = (
        build_age5_full_real_ci_pack_golden_jjaim_flatten_selftest_progress(
            age5_full_real_ci_pack_golden_jjaim_flatten_selftest_current_case=payload.get(
                "age5_full_real_ci_pack_golden_jjaim_flatten_selftest_current_case", "-"
            ),
            age5_full_real_ci_pack_golden_jjaim_flatten_selftest_last_completed_case=payload.get(
                "age5_full_real_ci_pack_golden_jjaim_flatten_selftest_last_completed_case", "-"
            ),
            age5_full_real_ci_pack_golden_jjaim_flatten_selftest_total_elapsed_ms=payload.get(
                "age5_full_real_ci_pack_golden_jjaim_flatten_selftest_total_elapsed_ms", "-"
            ),
            age5_full_real_ci_pack_golden_jjaim_flatten_selftest_current_probe=payload.get(
                "age5_full_real_ci_pack_golden_jjaim_flatten_selftest_current_probe", "-"
            ),
            age5_full_real_ci_pack_golden_jjaim_flatten_selftest_last_completed_probe=payload.get(
                "age5_full_real_ci_pack_golden_jjaim_flatten_selftest_last_completed_probe", "-"
            ),
            age5_full_real_ci_pack_golden_jjaim_flatten_selftest_progress_present=(
                str(payload.get("age5_full_real_ci_pack_golden_jjaim_flatten_selftest_progress_present", "0")).strip()
                == "1"
            ),
        )
    )
    full_real_ci_pack_golden_event_model_selftest_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_ci_pack_golden_event_model_selftest_progress_fields_text",
                AGE5_FULL_REAL_CI_PACK_GOLDEN_EVENT_MODEL_SELFTEST_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_CI_PACK_GOLDEN_EVENT_MODEL_SELFTEST_PROGRESS_FIELDS_TEXT
    )
    full_real_ci_pack_golden_event_model_selftest_progress = (
        build_age5_full_real_ci_pack_golden_event_model_selftest_progress(
            age5_full_real_ci_pack_golden_event_model_selftest_current_case=payload.get(
                "age5_full_real_ci_pack_golden_event_model_selftest_current_case", "-"
            ),
            age5_full_real_ci_pack_golden_event_model_selftest_last_completed_case=payload.get(
                "age5_full_real_ci_pack_golden_event_model_selftest_last_completed_case", "-"
            ),
            age5_full_real_ci_pack_golden_event_model_selftest_total_elapsed_ms=payload.get(
                "age5_full_real_ci_pack_golden_event_model_selftest_total_elapsed_ms", "-"
            ),
            age5_full_real_ci_pack_golden_event_model_selftest_current_probe=payload.get(
                "age5_full_real_ci_pack_golden_event_model_selftest_current_probe", "-"
            ),
            age5_full_real_ci_pack_golden_event_model_selftest_last_completed_probe=payload.get(
                "age5_full_real_ci_pack_golden_event_model_selftest_last_completed_probe", "-"
            ),
            age5_full_real_ci_pack_golden_event_model_selftest_progress_present=(
                str(payload.get("age5_full_real_ci_pack_golden_event_model_selftest_progress_present", "0")).strip()
                == "1"
            ),
        )
    )
    full_real_ci_pack_golden_lang_consistency_selftest_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_ci_pack_golden_lang_consistency_selftest_progress_fields_text",
                AGE5_FULL_REAL_CI_PACK_GOLDEN_LANG_CONSISTENCY_SELFTEST_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_CI_PACK_GOLDEN_LANG_CONSISTENCY_SELFTEST_PROGRESS_FIELDS_TEXT
    )
    full_real_ci_pack_golden_lang_consistency_selftest_progress = (
        build_age5_full_real_ci_pack_golden_lang_consistency_selftest_progress(
            age5_full_real_ci_pack_golden_lang_consistency_selftest_current_case=payload.get(
                "age5_full_real_ci_pack_golden_lang_consistency_selftest_current_case", "-"
            ),
            age5_full_real_ci_pack_golden_lang_consistency_selftest_last_completed_case=payload.get(
                "age5_full_real_ci_pack_golden_lang_consistency_selftest_last_completed_case", "-"
            ),
            age5_full_real_ci_pack_golden_lang_consistency_selftest_total_elapsed_ms=payload.get(
                "age5_full_real_ci_pack_golden_lang_consistency_selftest_total_elapsed_ms", "-"
            ),
            age5_full_real_ci_pack_golden_lang_consistency_selftest_current_probe=payload.get(
                "age5_full_real_ci_pack_golden_lang_consistency_selftest_current_probe", "-"
            ),
            age5_full_real_ci_pack_golden_lang_consistency_selftest_last_completed_probe=payload.get(
                "age5_full_real_ci_pack_golden_lang_consistency_selftest_last_completed_probe", "-"
            ),
            age5_full_real_ci_pack_golden_lang_consistency_selftest_progress_present=(
                str(payload.get("age5_full_real_ci_pack_golden_lang_consistency_selftest_progress_present", "0")).strip()
                == "1"
            ),
        )
    )
    full_real_w94_social_pack_check_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_w94_social_pack_check_progress_fields_text",
                AGE5_FULL_REAL_W94_SOCIAL_PACK_CHECK_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_W94_SOCIAL_PACK_CHECK_PROGRESS_FIELDS_TEXT
    )
    full_real_w94_social_pack_check_progress = build_age5_full_real_w94_social_pack_check_progress(
        age5_full_real_w94_social_pack_check_current_case=payload.get(
            "age5_full_real_w94_social_pack_check_current_case", "-"
        ),
        age5_full_real_w94_social_pack_check_last_completed_case=payload.get(
            "age5_full_real_w94_social_pack_check_last_completed_case", "-"
        ),
        age5_full_real_w94_social_pack_check_total_elapsed_ms=payload.get(
            "age5_full_real_w94_social_pack_check_total_elapsed_ms", "-"
        ),
        age5_full_real_w94_social_pack_check_current_probe=payload.get(
            "age5_full_real_w94_social_pack_check_current_probe", "-"
        ),
        age5_full_real_w94_social_pack_check_last_completed_probe=payload.get(
            "age5_full_real_w94_social_pack_check_last_completed_probe", "-"
        ),
        age5_full_real_w94_social_pack_check_progress_present=(
            str(payload.get("age5_full_real_w94_social_pack_check_progress_present", "0")).strip() == "1"
        ),
    )
    full_real_w95_cert_pack_check_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_w95_cert_pack_check_progress_fields_text",
                AGE5_FULL_REAL_W95_CERT_PACK_CHECK_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_W95_CERT_PACK_CHECK_PROGRESS_FIELDS_TEXT
    )
    full_real_w95_cert_pack_check_progress = build_age5_full_real_w95_cert_pack_check_progress(
        age5_full_real_w95_cert_pack_check_current_case=payload.get(
            "age5_full_real_w95_cert_pack_check_current_case", "-"
        ),
        age5_full_real_w95_cert_pack_check_last_completed_case=payload.get(
            "age5_full_real_w95_cert_pack_check_last_completed_case", "-"
        ),
        age5_full_real_w95_cert_pack_check_total_elapsed_ms=payload.get(
            "age5_full_real_w95_cert_pack_check_total_elapsed_ms", "-"
        ),
        age5_full_real_w95_cert_pack_check_current_probe=payload.get(
            "age5_full_real_w95_cert_pack_check_current_probe", "-"
        ),
        age5_full_real_w95_cert_pack_check_last_completed_probe=payload.get(
            "age5_full_real_w95_cert_pack_check_last_completed_probe", "-"
        ),
        age5_full_real_w95_cert_pack_check_progress_present=(
            str(payload.get("age5_full_real_w95_cert_pack_check_progress_present", "0")).strip() == "1"
        ),
    )
    full_real_w96_somssi_pack_check_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_w96_somssi_pack_check_progress_fields_text",
                AGE5_FULL_REAL_W96_SOMSSI_PACK_CHECK_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_W96_SOMSSI_PACK_CHECK_PROGRESS_FIELDS_TEXT
    )
    full_real_w96_somssi_pack_check_progress = build_age5_full_real_w96_somssi_pack_check_progress(
        age5_full_real_w96_somssi_pack_check_current_case=payload.get(
            "age5_full_real_w96_somssi_pack_check_current_case", "-"
        ),
        age5_full_real_w96_somssi_pack_check_last_completed_case=payload.get(
            "age5_full_real_w96_somssi_pack_check_last_completed_case", "-"
        ),
        age5_full_real_w96_somssi_pack_check_total_elapsed_ms=payload.get(
            "age5_full_real_w96_somssi_pack_check_total_elapsed_ms", "-"
        ),
        age5_full_real_w96_somssi_pack_check_current_probe=payload.get(
            "age5_full_real_w96_somssi_pack_check_current_probe", "-"
        ),
        age5_full_real_w96_somssi_pack_check_last_completed_probe=payload.get(
            "age5_full_real_w96_somssi_pack_check_last_completed_probe", "-"
        ),
        age5_full_real_w96_somssi_pack_check_progress_present=(
            str(payload.get("age5_full_real_w96_somssi_pack_check_progress_present", "0")).strip() == "1"
        ),
    )
    full_real_w97_self_heal_pack_check_progress_fields_text = (
        str(
            payload.get(
                "age5_full_real_w97_self_heal_pack_check_progress_fields_text",
                AGE5_FULL_REAL_W97_SELF_HEAL_PACK_CHECK_PROGRESS_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_W97_SELF_HEAL_PACK_CHECK_PROGRESS_FIELDS_TEXT
    )
    full_real_w97_self_heal_pack_check_progress = build_age5_full_real_w97_self_heal_pack_check_progress(
        age5_full_real_w97_self_heal_pack_check_current_case=payload.get(
            "age5_full_real_w97_self_heal_pack_check_current_case", "-"
        ),
        age5_full_real_w97_self_heal_pack_check_last_completed_case=payload.get(
            "age5_full_real_w97_self_heal_pack_check_last_completed_case", "-"
        ),
        age5_full_real_w97_self_heal_pack_check_total_elapsed_ms=payload.get(
            "age5_full_real_w97_self_heal_pack_check_total_elapsed_ms", "-"
        ),
        age5_full_real_w97_self_heal_pack_check_current_probe=payload.get(
            "age5_full_real_w97_self_heal_pack_check_current_probe", "-"
        ),
        age5_full_real_w97_self_heal_pack_check_last_completed_probe=payload.get(
            "age5_full_real_w97_self_heal_pack_check_last_completed_probe", "-"
        ),
        age5_full_real_w97_self_heal_pack_check_progress_present=(
            str(payload.get("age5_full_real_w97_self_heal_pack_check_progress_present", "0")).strip() == "1"
        ),
    )
    full_real_profile_elapsed_map_fields_text = (
        str(
            payload.get(
                "age5_full_real_profile_elapsed_map_fields_text",
                AGE5_FULL_REAL_PROFILE_ELAPSED_MAP_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_PROFILE_ELAPSED_MAP_FIELDS_TEXT
    )
    full_real_profile_elapsed_map = build_age5_full_real_profile_elapsed_map(
        age5_full_real_profile_elapsed_map=payload.get("age5_full_real_profile_elapsed_map", "-"),
        age5_full_real_profile_elapsed_map_present=(
            str(payload.get("age5_full_real_profile_elapsed_map_present", "0")).strip() == "1"
        ),
    )
    full_real_profile_status_map_fields_text = (
        str(
            payload.get(
                "age5_full_real_profile_status_map_fields_text",
                AGE5_FULL_REAL_PROFILE_STATUS_MAP_FIELDS_TEXT,
            )
        ).strip()
        or AGE5_FULL_REAL_PROFILE_STATUS_MAP_FIELDS_TEXT
    )
    full_real_profile_status_map = build_age5_full_real_profile_status_map(
        age5_full_real_profile_status_map=payload.get("age5_full_real_profile_status_map", "-"),
        age5_full_real_profile_status_map_present=(
            str(payload.get("age5_full_real_profile_status_map_present", "0")).strip() == "1"
        ),
    )
    full_real_timeout_breakdown_fields_text = (
        str(payload.get("age5_full_real_timeout_breakdown_fields_text", AGE5_FULL_REAL_TIMEOUT_BREAKDOWN_FIELDS_TEXT)).strip()
        or AGE5_FULL_REAL_TIMEOUT_BREAKDOWN_FIELDS_TEXT
    )
    full_real_timeout_breakdown = build_age5_full_real_timeout_breakdown(
        age5_full_real_timeout_step=payload.get("age5_full_real_timeout_step", "-"),
        age5_full_real_timeout_profiles=payload.get("age5_full_real_timeout_profiles", "-"),
        age5_full_real_timeout_present=str(payload.get("age5_full_real_timeout_present", "0")).strip() == "1",
    )
    age5_close_digest_selftest_ok = str(payload.get("age5_close_digest_selftest_ok", "0")).strip() or "0"
    age5_digest_selftest_default_text = (
        str(payload.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT)).strip()
        or AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT
    )
    age5_digest_selftest_default_field = payload.get("combined_digest_selftest_default_field")
    if not isinstance(age5_digest_selftest_default_field, dict):
        age5_digest_selftest_default_field = build_age5_close_digest_selftest_default_field()
    child_summary_default_fields = (
        str(payload.get("ci_sanity_age5_combined_heavy_child_summary_default_fields", "")).strip() or "-"
    )
    sync_child_summary_default_fields = (
        str(payload.get("ci_sync_readiness_ci_sanity_age5_combined_heavy_child_summary_default_fields", "")).strip()
        or "-"
    )
    full_real_source_trace = payload.get("full_real_source_trace")
    if not isinstance(full_real_source_trace, dict):
        full_real_source_trace = build_age5_combined_heavy_full_real_source_trace()
    full_real_source_check_exists = (
        str(full_real_source_trace.get("smoke_check_script_exists", "0")).strip() or "0"
    )
    full_real_source_selftest_exists = (
        str(full_real_source_trace.get("smoke_check_selftest_script_exists", "0")).strip() or "0"
    )
    age4_proof_snapshot_fields_text = (
        str(payload.get("age4_proof_snapshot_fields_text", AGE4_PROOF_SNAPSHOT_FIELDS_TEXT)).strip()
        or AGE4_PROOF_SNAPSHOT_FIELDS_TEXT
    )
    age4_proof_snapshot = build_age4_proof_snapshot(
        age4_proof_ok=payload.get("age4_proof_ok", "0"),
        age4_proof_failed_criteria=payload.get("age4_proof_failed_criteria", "-1"),
        age4_proof_failed_preview=payload.get("age4_proof_failed_preview", "-"),
    )
    age4_proof_snapshot_text = (
        str(payload.get("age4_proof_snapshot_text", "")).strip()
        or build_age4_proof_snapshot_text(age4_proof_snapshot)
    )
    age4_proof_source_fields = build_age4_proof_source_snapshot_fields(top_snapshot=age4_proof_snapshot)
    age4_proof_gate_result_present = (
        str(
            payload.get(
                AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY,
                age4_proof_source_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY],
            )
        ).strip()
        or age4_proof_source_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY]
    )
    age4_proof_gate_result_parity = (
        str(
            payload.get(
                AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY,
                age4_proof_source_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY],
            )
        ).strip()
        or age4_proof_source_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY]
    )
    age4_proof_final_status_parse_present = (
        str(
            payload.get(
                AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY,
                age4_proof_source_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY],
            )
        ).strip()
        or age4_proof_source_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY]
    )
    age4_proof_final_status_parse_parity = (
        str(
            payload.get(
                AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY,
                age4_proof_source_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY],
            )
        ).strip()
        or age4_proof_source_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY]
    )
    policy_contract = payload.get("policy_contract")
    if not isinstance(policy_contract, dict):
        policy_contract = {}
    policy_age4_proof_snapshot = build_age4_proof_snapshot(
        age4_proof_ok=policy_contract.get("age4_proof_ok", "0"),
        age4_proof_failed_criteria=policy_contract.get("age4_proof_failed_criteria", "-1"),
        age4_proof_failed_preview=policy_contract.get("age4_proof_failed_preview", "-"),
    )
    policy_age4_proof_source_fields = build_age4_proof_source_snapshot_fields(
        top_snapshot=policy_age4_proof_snapshot
    )
    policy_age4_proof_snapshot_fields_text = (
        str(policy_contract.get("age4_proof_snapshot_fields_text", AGE4_PROOF_SNAPSHOT_FIELDS_TEXT)).strip()
        or AGE4_PROOF_SNAPSHOT_FIELDS_TEXT
    )
    policy_age4_proof_source_snapshot_fields_text = (
        str(policy_contract.get("age4_proof_source_snapshot_fields_text", AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT)).strip()
        or AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT
    )
    policy_age4_proof_snapshot_text = (
        str(policy_contract.get("age4_proof_snapshot_text", "")).strip()
        or build_age4_proof_snapshot_text(policy_age4_proof_snapshot)
    )
    policy_age4_proof_gate_result_present = (
        str(
            policy_contract.get(
                AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY,
                policy_age4_proof_source_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY],
            )
        ).strip()
        or policy_age4_proof_source_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PRESENT_KEY]
    )
    policy_age4_proof_gate_result_parity = (
        str(
            policy_contract.get(
                AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY,
                policy_age4_proof_source_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY],
            )
        ).strip()
        or policy_age4_proof_source_fields[AGE4_PROOF_GATE_RESULT_SNAPSHOT_PARITY_KEY]
    )
    policy_age4_proof_final_status_parse_present = (
        str(
            policy_contract.get(
                AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY,
                policy_age4_proof_source_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY],
            )
        ).strip()
        or policy_age4_proof_source_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PRESENT_KEY]
    )
    policy_age4_proof_final_status_parse_parity = (
        str(
            policy_contract.get(
                AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY,
                policy_age4_proof_source_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY],
            )
        ).strip()
        or policy_age4_proof_source_fields[AGE4_PROOF_FINAL_STATUS_PARSE_SNAPSHOT_PARITY_KEY]
    )
    print(
        f"[age5-close] overall_ok={int(overall_ok)} criteria={total} failed={failed} "
        f"age5_full_real={full_real_status} "
        f"age5_runtime_helper_negative={runtime_helper_negative_status} "
        f"age5_group_id_summary_negative={group_id_summary_negative_status} "
        f"age5_combined_heavy_child_timeout_sec={combined_heavy_child_timeout_sec} "
        f"age5_combined_heavy_timeout_mode={combined_heavy_timeout_mode} "
        f"age5_combined_heavy_timeout_present={combined_heavy_timeout_present} "
        f"age5_combined_heavy_timeout_targets={combined_heavy_timeout_targets} "
        f"age5_full_real_elapsed_fields_text={full_real_elapsed_fields_text} "
        f"age5_full_real_total_elapsed_ms={full_real_elapsed_summary['age5_full_real_total_elapsed_ms']} "
        f"age5_full_real_slowest_profile={full_real_elapsed_summary['age5_full_real_slowest_profile']} "
        f"age5_full_real_slowest_elapsed_ms={full_real_elapsed_summary['age5_full_real_slowest_elapsed_ms']} "
        f"age5_full_real_elapsed_present={full_real_elapsed_summary['age5_full_real_elapsed_present']} "
        f"age5_full_real_core_lang_sanity_elapsed_fields_text={full_real_core_lang_sanity_elapsed_fields_text} "
        f"age5_full_real_core_lang_sanity_total_elapsed_ms={full_real_core_lang_sanity_elapsed_summary['age5_full_real_core_lang_sanity_total_elapsed_ms']} "
        f"age5_full_real_core_lang_sanity_slowest_step={full_real_core_lang_sanity_elapsed_summary['age5_full_real_core_lang_sanity_slowest_step']} "
        f"age5_full_real_core_lang_sanity_slowest_elapsed_ms={full_real_core_lang_sanity_elapsed_summary['age5_full_real_core_lang_sanity_slowest_elapsed_ms']} "
        f"age5_full_real_core_lang_sanity_elapsed_present={full_real_core_lang_sanity_elapsed_summary['age5_full_real_core_lang_sanity_elapsed_present']} "
        f"age5_full_real_core_lang_sanity_progress_fields_text={full_real_core_lang_sanity_progress_fields_text} "
        f"age5_full_real_core_lang_sanity_current_step={full_real_core_lang_sanity_current_step} "
        f"age5_full_real_core_lang_sanity_last_completed_step={full_real_core_lang_sanity_last_completed_step} "
        f"age5_full_real_core_lang_sanity_progress_present={full_real_core_lang_sanity_progress_present} "
        f"age5_full_real_pipeline_emit_flags_progress_fields_text={full_real_pipeline_emit_flags_progress_fields_text} "
        f"age5_full_real_pipeline_emit_flags_current_section={full_real_pipeline_emit_flags_progress['age5_full_real_pipeline_emit_flags_current_section']} "
        f"age5_full_real_pipeline_emit_flags_last_completed_section={full_real_pipeline_emit_flags_progress['age5_full_real_pipeline_emit_flags_last_completed_section']} "
        f"age5_full_real_pipeline_emit_flags_total_elapsed_ms={full_real_pipeline_emit_flags_progress['age5_full_real_pipeline_emit_flags_total_elapsed_ms']} "
        f"age5_full_real_pipeline_emit_flags_progress_present={full_real_pipeline_emit_flags_progress['age5_full_real_pipeline_emit_flags_progress_present']} "
        f"age5_full_real_pipeline_emit_flags_selftest_progress_fields_text={full_real_pipeline_emit_flags_selftest_progress_fields_text} "
        f"age5_full_real_pipeline_emit_flags_selftest_current_case={full_real_pipeline_emit_flags_selftest_progress['age5_full_real_pipeline_emit_flags_selftest_current_case']} "
        f"age5_full_real_pipeline_emit_flags_selftest_last_completed_case={full_real_pipeline_emit_flags_selftest_progress['age5_full_real_pipeline_emit_flags_selftest_last_completed_case']} "
        f"age5_full_real_pipeline_emit_flags_selftest_total_elapsed_ms={full_real_pipeline_emit_flags_selftest_progress['age5_full_real_pipeline_emit_flags_selftest_total_elapsed_ms']} "
        f"age5_full_real_pipeline_emit_flags_selftest_progress_present={full_real_pipeline_emit_flags_selftest_progress['age5_full_real_pipeline_emit_flags_selftest_progress_present']} "
        f"age5_full_real_pipeline_emit_flags_selftest_probe_fields_text={full_real_pipeline_emit_flags_selftest_probe_fields_text} "
        f"age5_full_real_pipeline_emit_flags_selftest_current_probe={full_real_pipeline_emit_flags_selftest_probe['age5_full_real_pipeline_emit_flags_selftest_current_probe']} "
        f"age5_full_real_pipeline_emit_flags_selftest_last_completed_probe={full_real_pipeline_emit_flags_selftest_probe['age5_full_real_pipeline_emit_flags_selftest_last_completed_probe']} "
        f"age5_full_real_pipeline_emit_flags_selftest_probe_present={full_real_pipeline_emit_flags_selftest_probe['age5_full_real_pipeline_emit_flags_selftest_probe_present']} "
        f"age5_full_real_age5_combined_policy_selftest_progress_fields_text={full_real_age5_combined_policy_selftest_progress_fields_text} "
        f"age5_full_real_age5_combined_policy_selftest_current_case={full_real_age5_combined_policy_selftest_current_case} "
        f"age5_full_real_age5_combined_policy_selftest_last_completed_case={full_real_age5_combined_policy_selftest_last_completed_case} "
        f"age5_full_real_age5_combined_policy_selftest_current_format={full_real_age5_combined_policy_selftest_current_format} "
        f"age5_full_real_age5_combined_policy_selftest_last_completed_format={full_real_age5_combined_policy_selftest_last_completed_format} "
        f"age5_full_real_age5_combined_policy_selftest_current_probe={full_real_age5_combined_policy_selftest_current_probe} "
        f"age5_full_real_age5_combined_policy_selftest_last_completed_probe={full_real_age5_combined_policy_selftest_last_completed_probe} "
        f"age5_full_real_age5_combined_policy_selftest_total_elapsed_ms={full_real_age5_combined_policy_selftest_total_elapsed_ms} "
        f"age5_full_real_age5_combined_policy_selftest_progress_present={full_real_age5_combined_policy_selftest_progress_present} "
        f"age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_fields_text={full_real_profile_matrix_full_real_smoke_policy_selftest_progress_fields_text} "
        f"age5_full_real_profile_matrix_full_real_smoke_policy_selftest_current_case={full_real_profile_matrix_full_real_smoke_policy_selftest_progress['age5_full_real_profile_matrix_full_real_smoke_policy_selftest_current_case']} "
        f"age5_full_real_profile_matrix_full_real_smoke_policy_selftest_last_completed_case={full_real_profile_matrix_full_real_smoke_policy_selftest_progress['age5_full_real_profile_matrix_full_real_smoke_policy_selftest_last_completed_case']} "
        f"age5_full_real_profile_matrix_full_real_smoke_policy_selftest_current_format={full_real_profile_matrix_full_real_smoke_policy_selftest_progress['age5_full_real_profile_matrix_full_real_smoke_policy_selftest_current_format']} "
        f"age5_full_real_profile_matrix_full_real_smoke_policy_selftest_last_completed_format={full_real_profile_matrix_full_real_smoke_policy_selftest_progress['age5_full_real_profile_matrix_full_real_smoke_policy_selftest_last_completed_format']} "
        f"age5_full_real_profile_matrix_full_real_smoke_policy_selftest_total_elapsed_ms={full_real_profile_matrix_full_real_smoke_policy_selftest_progress['age5_full_real_profile_matrix_full_real_smoke_policy_selftest_total_elapsed_ms']} "
        f"age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_present={full_real_profile_matrix_full_real_smoke_policy_selftest_progress['age5_full_real_profile_matrix_full_real_smoke_policy_selftest_progress_present']} "
        f"age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_fields_text={full_real_profile_matrix_full_real_smoke_check_selftest_progress_fields_text} "
        f"age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_case={full_real_profile_matrix_full_real_smoke_check_selftest_progress['age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_case']} "
        f"age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_case={full_real_profile_matrix_full_real_smoke_check_selftest_progress['age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_case']} "
        f"age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_probe={full_real_profile_matrix_full_real_smoke_check_selftest_progress['age5_full_real_profile_matrix_full_real_smoke_check_selftest_current_probe']} "
        f"age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_probe={full_real_profile_matrix_full_real_smoke_check_selftest_progress['age5_full_real_profile_matrix_full_real_smoke_check_selftest_last_completed_probe']} "
        f"age5_full_real_profile_matrix_full_real_smoke_check_selftest_total_elapsed_ms={full_real_profile_matrix_full_real_smoke_check_selftest_progress['age5_full_real_profile_matrix_full_real_smoke_check_selftest_total_elapsed_ms']} "
        f"age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_present={full_real_profile_matrix_full_real_smoke_check_selftest_progress['age5_full_real_profile_matrix_full_real_smoke_check_selftest_progress_present']} "
        f"age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_fields_text={full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_fields_text} "
        f"age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_case={full_real_fixed64_darwin_real_report_readiness_check_selftest_progress['age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_case']} "
        f"age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_case={full_real_fixed64_darwin_real_report_readiness_check_selftest_progress['age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_case']} "
        f"age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_probe={full_real_fixed64_darwin_real_report_readiness_check_selftest_progress['age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_current_probe']} "
        f"age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_probe={full_real_fixed64_darwin_real_report_readiness_check_selftest_progress['age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_last_completed_probe']} "
        f"age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_total_elapsed_ms={full_real_fixed64_darwin_real_report_readiness_check_selftest_progress['age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_total_elapsed_ms']} "
        f"age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_present={full_real_fixed64_darwin_real_report_readiness_check_selftest_progress['age5_full_real_fixed64_darwin_real_report_readiness_check_selftest_progress_present']} "
        f"age5_full_real_map_access_contract_check_progress_fields_text={full_real_map_access_contract_check_progress_fields_text} "
        f"age5_full_real_map_access_contract_check_current_case={full_real_map_access_contract_check_progress['age5_full_real_map_access_contract_check_current_case']} "
        f"age5_full_real_map_access_contract_check_last_completed_case={full_real_map_access_contract_check_progress['age5_full_real_map_access_contract_check_last_completed_case']} "
        f"age5_full_real_map_access_contract_check_current_probe={full_real_map_access_contract_check_progress['age5_full_real_map_access_contract_check_current_probe']} "
        f"age5_full_real_map_access_contract_check_last_completed_probe={full_real_map_access_contract_check_progress['age5_full_real_map_access_contract_check_last_completed_probe']} "
        f"age5_full_real_map_access_contract_check_total_elapsed_ms={full_real_map_access_contract_check_progress['age5_full_real_map_access_contract_check_total_elapsed_ms']} "
        f"age5_full_real_map_access_contract_check_progress_present={full_real_map_access_contract_check_progress['age5_full_real_map_access_contract_check_progress_present']} "
        f"age5_full_real_tensor_v0_cli_check_progress_fields_text={full_real_tensor_v0_cli_check_progress_fields_text} "
        f"age5_full_real_tensor_v0_cli_check_current_case={full_real_tensor_v0_cli_check_progress['age5_full_real_tensor_v0_cli_check_current_case']} "
        f"age5_full_real_tensor_v0_cli_check_last_completed_case={full_real_tensor_v0_cli_check_progress['age5_full_real_tensor_v0_cli_check_last_completed_case']} "
        f"age5_full_real_tensor_v0_cli_check_current_probe={full_real_tensor_v0_cli_check_progress['age5_full_real_tensor_v0_cli_check_current_probe']} "
        f"age5_full_real_tensor_v0_cli_check_last_completed_probe={full_real_tensor_v0_cli_check_progress['age5_full_real_tensor_v0_cli_check_last_completed_probe']} "
        f"age5_full_real_tensor_v0_cli_check_total_elapsed_ms={full_real_tensor_v0_cli_check_progress['age5_full_real_tensor_v0_cli_check_total_elapsed_ms']} "
        f"age5_full_real_tensor_v0_cli_check_progress_present={full_real_tensor_v0_cli_check_progress['age5_full_real_tensor_v0_cli_check_progress_present']} "
        f"age5_full_real_ci_pack_golden_exec_policy_selftest_progress_fields_text={full_real_ci_pack_golden_exec_policy_selftest_progress_fields_text} "
        f"age5_full_real_ci_pack_golden_exec_policy_selftest_current_case={full_real_ci_pack_golden_exec_policy_selftest_progress['age5_full_real_ci_pack_golden_exec_policy_selftest_current_case']} "
        f"age5_full_real_ci_pack_golden_exec_policy_selftest_last_completed_case={full_real_ci_pack_golden_exec_policy_selftest_progress['age5_full_real_ci_pack_golden_exec_policy_selftest_last_completed_case']} "
        f"age5_full_real_ci_pack_golden_exec_policy_selftest_current_probe={full_real_ci_pack_golden_exec_policy_selftest_progress['age5_full_real_ci_pack_golden_exec_policy_selftest_current_probe']} "
        f"age5_full_real_ci_pack_golden_exec_policy_selftest_last_completed_probe={full_real_ci_pack_golden_exec_policy_selftest_progress['age5_full_real_ci_pack_golden_exec_policy_selftest_last_completed_probe']} "
        f"age5_full_real_ci_pack_golden_exec_policy_selftest_total_elapsed_ms={full_real_ci_pack_golden_exec_policy_selftest_progress['age5_full_real_ci_pack_golden_exec_policy_selftest_total_elapsed_ms']} "
        f"age5_full_real_ci_pack_golden_exec_policy_selftest_progress_present={full_real_ci_pack_golden_exec_policy_selftest_progress['age5_full_real_ci_pack_golden_exec_policy_selftest_progress_present']} "
        f"age5_full_real_ci_pack_golden_age5_surface_selftest_progress_fields_text={full_real_ci_pack_golden_age5_surface_selftest_progress_fields_text} "
        f"age5_full_real_ci_pack_golden_age5_surface_selftest_current_case={full_real_ci_pack_golden_age5_surface_selftest_progress['age5_full_real_ci_pack_golden_age5_surface_selftest_current_case']} "
        f"age5_full_real_ci_pack_golden_age5_surface_selftest_last_completed_case={full_real_ci_pack_golden_age5_surface_selftest_progress['age5_full_real_ci_pack_golden_age5_surface_selftest_last_completed_case']} "
        f"age5_full_real_ci_pack_golden_age5_surface_selftest_current_probe={full_real_ci_pack_golden_age5_surface_selftest_progress['age5_full_real_ci_pack_golden_age5_surface_selftest_current_probe']} "
        f"age5_full_real_ci_pack_golden_age5_surface_selftest_last_completed_probe={full_real_ci_pack_golden_age5_surface_selftest_progress['age5_full_real_ci_pack_golden_age5_surface_selftest_last_completed_probe']} "
        f"age5_full_real_ci_pack_golden_age5_surface_selftest_total_elapsed_ms={full_real_ci_pack_golden_age5_surface_selftest_progress['age5_full_real_ci_pack_golden_age5_surface_selftest_total_elapsed_ms']} "
        f"age5_full_real_ci_pack_golden_age5_surface_selftest_progress_present={full_real_ci_pack_golden_age5_surface_selftest_progress['age5_full_real_ci_pack_golden_age5_surface_selftest_progress_present']} "
        f"age5_full_real_ci_pack_golden_guideblock_selftest_progress_fields_text={full_real_ci_pack_golden_guideblock_selftest_progress_fields_text} "
        f"age5_full_real_ci_pack_golden_guideblock_selftest_current_case={full_real_ci_pack_golden_guideblock_selftest_progress['age5_full_real_ci_pack_golden_guideblock_selftest_current_case']} "
        f"age5_full_real_ci_pack_golden_guideblock_selftest_last_completed_case={full_real_ci_pack_golden_guideblock_selftest_progress['age5_full_real_ci_pack_golden_guideblock_selftest_last_completed_case']} "
        f"age5_full_real_ci_pack_golden_guideblock_selftest_current_probe={full_real_ci_pack_golden_guideblock_selftest_progress['age5_full_real_ci_pack_golden_guideblock_selftest_current_probe']} "
        f"age5_full_real_ci_pack_golden_guideblock_selftest_last_completed_probe={full_real_ci_pack_golden_guideblock_selftest_progress['age5_full_real_ci_pack_golden_guideblock_selftest_last_completed_probe']} "
        f"age5_full_real_ci_pack_golden_guideblock_selftest_total_elapsed_ms={full_real_ci_pack_golden_guideblock_selftest_progress['age5_full_real_ci_pack_golden_guideblock_selftest_total_elapsed_ms']} "
        f"age5_full_real_ci_pack_golden_guideblock_selftest_progress_present={full_real_ci_pack_golden_guideblock_selftest_progress['age5_full_real_ci_pack_golden_guideblock_selftest_progress_present']} "
        f"age5_full_real_ci_pack_golden_jjaim_flatten_selftest_progress_fields_text={full_real_ci_pack_golden_jjaim_flatten_selftest_progress_fields_text} "
        f"age5_full_real_ci_pack_golden_jjaim_flatten_selftest_current_case={full_real_ci_pack_golden_jjaim_flatten_selftest_progress['age5_full_real_ci_pack_golden_jjaim_flatten_selftest_current_case']} "
        f"age5_full_real_ci_pack_golden_jjaim_flatten_selftest_last_completed_case={full_real_ci_pack_golden_jjaim_flatten_selftest_progress['age5_full_real_ci_pack_golden_jjaim_flatten_selftest_last_completed_case']} "
        f"age5_full_real_ci_pack_golden_jjaim_flatten_selftest_current_probe={full_real_ci_pack_golden_jjaim_flatten_selftest_progress['age5_full_real_ci_pack_golden_jjaim_flatten_selftest_current_probe']} "
        f"age5_full_real_ci_pack_golden_jjaim_flatten_selftest_last_completed_probe={full_real_ci_pack_golden_jjaim_flatten_selftest_progress['age5_full_real_ci_pack_golden_jjaim_flatten_selftest_last_completed_probe']} "
        f"age5_full_real_ci_pack_golden_jjaim_flatten_selftest_total_elapsed_ms={full_real_ci_pack_golden_jjaim_flatten_selftest_progress['age5_full_real_ci_pack_golden_jjaim_flatten_selftest_total_elapsed_ms']} "
        f"age5_full_real_ci_pack_golden_jjaim_flatten_selftest_progress_present={full_real_ci_pack_golden_jjaim_flatten_selftest_progress['age5_full_real_ci_pack_golden_jjaim_flatten_selftest_progress_present']} "
        f"age5_full_real_ci_pack_golden_event_model_selftest_progress_fields_text={full_real_ci_pack_golden_event_model_selftest_progress_fields_text} "
        f"age5_full_real_ci_pack_golden_event_model_selftest_current_case={full_real_ci_pack_golden_event_model_selftest_progress['age5_full_real_ci_pack_golden_event_model_selftest_current_case']} "
        f"age5_full_real_ci_pack_golden_event_model_selftest_last_completed_case={full_real_ci_pack_golden_event_model_selftest_progress['age5_full_real_ci_pack_golden_event_model_selftest_last_completed_case']} "
        f"age5_full_real_ci_pack_golden_event_model_selftest_current_probe={full_real_ci_pack_golden_event_model_selftest_progress['age5_full_real_ci_pack_golden_event_model_selftest_current_probe']} "
        f"age5_full_real_ci_pack_golden_event_model_selftest_last_completed_probe={full_real_ci_pack_golden_event_model_selftest_progress['age5_full_real_ci_pack_golden_event_model_selftest_last_completed_probe']} "
        f"age5_full_real_ci_pack_golden_event_model_selftest_total_elapsed_ms={full_real_ci_pack_golden_event_model_selftest_progress['age5_full_real_ci_pack_golden_event_model_selftest_total_elapsed_ms']} "
        f"age5_full_real_ci_pack_golden_event_model_selftest_progress_present={full_real_ci_pack_golden_event_model_selftest_progress['age5_full_real_ci_pack_golden_event_model_selftest_progress_present']} "
        f"age5_full_real_ci_pack_golden_lang_consistency_selftest_progress_fields_text={full_real_ci_pack_golden_lang_consistency_selftest_progress_fields_text} "
        f"age5_full_real_ci_pack_golden_lang_consistency_selftest_current_case={full_real_ci_pack_golden_lang_consistency_selftest_progress['age5_full_real_ci_pack_golden_lang_consistency_selftest_current_case']} "
        f"age5_full_real_ci_pack_golden_lang_consistency_selftest_last_completed_case={full_real_ci_pack_golden_lang_consistency_selftest_progress['age5_full_real_ci_pack_golden_lang_consistency_selftest_last_completed_case']} "
        f"age5_full_real_ci_pack_golden_lang_consistency_selftest_current_probe={full_real_ci_pack_golden_lang_consistency_selftest_progress['age5_full_real_ci_pack_golden_lang_consistency_selftest_current_probe']} "
        f"age5_full_real_ci_pack_golden_lang_consistency_selftest_last_completed_probe={full_real_ci_pack_golden_lang_consistency_selftest_progress['age5_full_real_ci_pack_golden_lang_consistency_selftest_last_completed_probe']} "
        f"age5_full_real_ci_pack_golden_lang_consistency_selftest_total_elapsed_ms={full_real_ci_pack_golden_lang_consistency_selftest_progress['age5_full_real_ci_pack_golden_lang_consistency_selftest_total_elapsed_ms']} "
        f"age5_full_real_ci_pack_golden_lang_consistency_selftest_progress_present={full_real_ci_pack_golden_lang_consistency_selftest_progress['age5_full_real_ci_pack_golden_lang_consistency_selftest_progress_present']} "
        f"age5_full_real_w94_social_pack_check_progress_fields_text={full_real_w94_social_pack_check_progress_fields_text} "
        f"age5_full_real_w94_social_pack_check_current_case={full_real_w94_social_pack_check_progress['age5_full_real_w94_social_pack_check_current_case']} "
        f"age5_full_real_w94_social_pack_check_last_completed_case={full_real_w94_social_pack_check_progress['age5_full_real_w94_social_pack_check_last_completed_case']} "
        f"age5_full_real_w94_social_pack_check_current_probe={full_real_w94_social_pack_check_progress['age5_full_real_w94_social_pack_check_current_probe']} "
        f"age5_full_real_w94_social_pack_check_last_completed_probe={full_real_w94_social_pack_check_progress['age5_full_real_w94_social_pack_check_last_completed_probe']} "
        f"age5_full_real_w94_social_pack_check_total_elapsed_ms={full_real_w94_social_pack_check_progress['age5_full_real_w94_social_pack_check_total_elapsed_ms']} "
        f"age5_full_real_w94_social_pack_check_progress_present={full_real_w94_social_pack_check_progress['age5_full_real_w94_social_pack_check_progress_present']} "
        f"age5_full_real_w95_cert_pack_check_progress_fields_text={full_real_w95_cert_pack_check_progress_fields_text} "
        f"age5_full_real_w95_cert_pack_check_current_case={full_real_w95_cert_pack_check_progress['age5_full_real_w95_cert_pack_check_current_case']} "
        f"age5_full_real_w95_cert_pack_check_last_completed_case={full_real_w95_cert_pack_check_progress['age5_full_real_w95_cert_pack_check_last_completed_case']} "
        f"age5_full_real_w95_cert_pack_check_current_probe={full_real_w95_cert_pack_check_progress['age5_full_real_w95_cert_pack_check_current_probe']} "
        f"age5_full_real_w95_cert_pack_check_last_completed_probe={full_real_w95_cert_pack_check_progress['age5_full_real_w95_cert_pack_check_last_completed_probe']} "
        f"age5_full_real_w95_cert_pack_check_total_elapsed_ms={full_real_w95_cert_pack_check_progress['age5_full_real_w95_cert_pack_check_total_elapsed_ms']} "
        f"age5_full_real_w95_cert_pack_check_progress_present={full_real_w95_cert_pack_check_progress['age5_full_real_w95_cert_pack_check_progress_present']} "
        f"age5_full_real_w96_somssi_pack_check_progress_fields_text={full_real_w96_somssi_pack_check_progress_fields_text} "
        f"age5_full_real_w96_somssi_pack_check_current_case={full_real_w96_somssi_pack_check_progress['age5_full_real_w96_somssi_pack_check_current_case']} "
        f"age5_full_real_w96_somssi_pack_check_last_completed_case={full_real_w96_somssi_pack_check_progress['age5_full_real_w96_somssi_pack_check_last_completed_case']} "
        f"age5_full_real_w96_somssi_pack_check_current_probe={full_real_w96_somssi_pack_check_progress['age5_full_real_w96_somssi_pack_check_current_probe']} "
        f"age5_full_real_w96_somssi_pack_check_last_completed_probe={full_real_w96_somssi_pack_check_progress['age5_full_real_w96_somssi_pack_check_last_completed_probe']} "
        f"age5_full_real_w96_somssi_pack_check_total_elapsed_ms={full_real_w96_somssi_pack_check_progress['age5_full_real_w96_somssi_pack_check_total_elapsed_ms']} "
        f"age5_full_real_w96_somssi_pack_check_progress_present={full_real_w96_somssi_pack_check_progress['age5_full_real_w96_somssi_pack_check_progress_present']} "
        f"age5_full_real_w97_self_heal_pack_check_progress_fields_text={full_real_w97_self_heal_pack_check_progress_fields_text} "
        f"age5_full_real_w97_self_heal_pack_check_current_case={full_real_w97_self_heal_pack_check_progress['age5_full_real_w97_self_heal_pack_check_current_case']} "
        f"age5_full_real_w97_self_heal_pack_check_last_completed_case={full_real_w97_self_heal_pack_check_progress['age5_full_real_w97_self_heal_pack_check_last_completed_case']} "
        f"age5_full_real_w97_self_heal_pack_check_current_probe={full_real_w97_self_heal_pack_check_progress['age5_full_real_w97_self_heal_pack_check_current_probe']} "
        f"age5_full_real_w97_self_heal_pack_check_last_completed_probe={full_real_w97_self_heal_pack_check_progress['age5_full_real_w97_self_heal_pack_check_last_completed_probe']} "
        f"age5_full_real_w97_self_heal_pack_check_total_elapsed_ms={full_real_w97_self_heal_pack_check_progress['age5_full_real_w97_self_heal_pack_check_total_elapsed_ms']} "
        f"age5_full_real_w97_self_heal_pack_check_progress_present={full_real_w97_self_heal_pack_check_progress['age5_full_real_w97_self_heal_pack_check_progress_present']} "
        f"age5_full_real_profile_elapsed_map_fields_text={full_real_profile_elapsed_map_fields_text} "
        f"age5_full_real_profile_elapsed_map={full_real_profile_elapsed_map['age5_full_real_profile_elapsed_map']} "
        f"age5_full_real_profile_elapsed_map_present={full_real_profile_elapsed_map['age5_full_real_profile_elapsed_map_present']} "
        f"age5_full_real_profile_status_map_fields_text={full_real_profile_status_map_fields_text} "
        f"age5_full_real_profile_status_map={full_real_profile_status_map['age5_full_real_profile_status_map']} "
        f"age5_full_real_profile_status_map_present={full_real_profile_status_map['age5_full_real_profile_status_map_present']} "
        f"age5_full_real_timeout_breakdown_fields_text={full_real_timeout_breakdown_fields_text} "
        f"age5_full_real_timeout_step={full_real_timeout_breakdown['age5_full_real_timeout_step']} "
        f"age5_full_real_timeout_profiles={full_real_timeout_breakdown['age5_full_real_timeout_profiles']} "
        f"age5_full_real_timeout_present={full_real_timeout_breakdown['age5_full_real_timeout_present']} "
        f"age5_full_real_source_check={full_real_source_check_exists} "
        f"age5_full_real_source_selftest={full_real_source_selftest_exists} "
        f"age4_proof_snapshot_fields_text={age4_proof_snapshot_fields_text} "
        f"age4_proof_snapshot_text={age4_proof_snapshot_text} "
        f"age4_proof_gate_result_present={age4_proof_gate_result_present} "
        f"age4_proof_gate_result_parity={age4_proof_gate_result_parity} "
        f"age4_proof_final_status_parse_present={age4_proof_final_status_parse_present} "
        f"age4_proof_final_status_parse_parity={age4_proof_final_status_parse_parity} "
        f"age5_policy_age4_proof_snapshot_fields_text={policy_age4_proof_snapshot_fields_text} "
        f"age5_policy_age4_proof_source_snapshot_fields_text={policy_age4_proof_source_snapshot_fields_text} "
        f"age5_policy_age4_proof_snapshot_text={policy_age4_proof_snapshot_text} "
        f"age5_policy_age4_proof_gate_result_present={policy_age4_proof_gate_result_present} "
        f"age5_policy_age4_proof_gate_result_parity={policy_age4_proof_gate_result_parity} "
        f"age5_policy_age4_proof_final_status_parse_present={policy_age4_proof_final_status_parse_present} "
        f"age5_policy_age4_proof_final_status_parse_parity={policy_age4_proof_final_status_parse_parity} "
        f"age5_close_digest_selftest_ok={age5_close_digest_selftest_ok} "
        f"{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY}={age5_digest_selftest_default_text} "
        f"combined_digest_selftest_default_field={json.dumps(age5_digest_selftest_default_field, ensure_ascii=False, sort_keys=True, separators=(',', ':'))} "
        f"age5_child_summary_defaults={child_summary_default_fields} "
        f"age5_sync_child_summary_defaults={sync_child_summary_default_fields} "
        f"report={path}"
    )
    if args.only_failed and overall_ok:
        return 0

    digest = payload.get("failure_digest")
    if isinstance(digest, list) and digest:
        for line in digest[: max(1, int(args.top))]:
            print(f" - {line}")
    else:
        print(" - failure_digest=(none)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
