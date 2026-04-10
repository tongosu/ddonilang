#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

from _ci_age3_completion_gate_contract import (
    AGE3_COMPLETION_GATE_CRITERIA_NAMES,
    AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS,
    age3_completion_gate_criteria_summary_key,
)
from _ci_age5_combined_heavy_contract import build_age5_combined_heavy_sanity_contract_fields
from _selftest_exec_cache import EXEC_CACHE_ENV_KEY, mark_script_ok, reset_exec_cache


FAIL_CODE_RE = re.compile(r"fail code=([A-Z0-9_]+)")
AGE5_COMBINED_HEAVY_CONTRACT_SUMMARY_KEYS = (
    "ci_sanity_age5_combined_heavy_report_schema",
    "ci_sanity_age5_combined_heavy_required_reports",
    "ci_sanity_age5_combined_heavy_required_criteria",
    "ci_sanity_age5_combined_heavy_child_summary_default_fields",
    "ci_sanity_age5_combined_heavy_combined_contract_summary_fields",
    "ci_sanity_age5_combined_heavy_full_summary_contract_fields",
)
PACK_GOLDEN_GRAPH_EXPORT_SUMMARY_KEY = "ci_sanity_pack_golden_graph_export_ok"
PACK_GOLDEN_GRAPH_EXPORT_REQUIRED_PROFILES = {"full", "core_lang"}

SANITY_SUMMARY_STEP_FIELDS = (
    ("ci_sanity_pipeline_emit_flags_ok", "pipeline_emit_flags_check", {"full", "core_lang"}),
    ("ci_sanity_pipeline_emit_flags_selftest_ok", "pipeline_emit_flags_selftest", {"full", "core_lang"}),
    (
        "ci_sanity_emit_artifacts_sanity_contract_selftest_ok",
        "ci_emit_artifacts_sanity_contract_selftest",
        {"full", "core_lang", "seamgrim"},
    ),
    ("ci_sanity_age2_completion_gate_ok", "age2_completion_gate", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age2_completion_gate_selftest_ok", "age2_completion_gate_selftest", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age2_close_ok", "age2_close", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age2_close_selftest_ok", "age2_close_selftest", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_completion_gate_ok", "age3_completion_gate", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_completion_gate_selftest_ok", "age3_completion_gate_selftest", {"full", "core_lang", "seamgrim"}),
    ("ci_sanity_age3_close_ok", "age3_close", {"full", "seamgrim"}),
    ("ci_sanity_age3_close_selftest_ok", "age3_close_selftest", {"full", "core_lang", "seamgrim"}),
    (
        "ci_sanity_age5_combined_heavy_policy_selftest_ok",
        "age5_combined_heavy_policy_selftest",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok",
        "profile_matrix_full_real_smoke_policy_selftest",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_dynamic_source_profile_split_selftest_ok",
        "ci_sanity_dynamic_source_profile_split_selftest",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_fixed64_live_summary_fields_selftest_ok",
        "ci_sanity_fixed64_live_summary_fields_selftest",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_fixed64_darwin_real_report_live_check_selftest_ok",
        "fixed64_darwin_real_report_live_check_selftest",
        {"full", "core_lang", "seamgrim"},
    ),
    (
        "ci_sanity_fixed64_threeway_inputs_selftest_ok",
        "fixed64_threeway_inputs_selftest",
        {"full", "core_lang", "seamgrim"},
    ),
)
AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_ENABLED_PROFILES = {"full", "core_lang", "seamgrim"}
AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA = "ddn.bogae_geoul_visibility_smoke.v1"
AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SUMMARY_KEYS = (
    "ci_sanity_age3_bogae_geoul_visibility_smoke_ok",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_schema",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes",
    "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes",
)
SEAMGRIM_WASM_WEB_STEP_CHECK_ENABLED_PROFILES = {"seamgrim"}
SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA = "ddn.seamgrim_ci_gate_wasm_web_smoke_step_check.v1"
SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES = 20
SEAMGRIM_WASM_WEB_STEP_CHECK_SUMMARY_KEYS = (
    "ci_sanity_seamgrim_wasm_web_step_check_ok",
    "ci_sanity_seamgrim_wasm_web_step_check_report_path",
    "ci_sanity_seamgrim_wasm_web_step_check_report_exists",
    "ci_sanity_seamgrim_wasm_web_step_check_schema",
    "ci_sanity_seamgrim_wasm_web_step_check_checked_files",
    "ci_sanity_seamgrim_wasm_web_step_check_missing_count",
)
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_ENABLED_PROFILES = {"seamgrim"}
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA = "ddn.pack_evidence_tier_runner_check.v1"
SEAMGRIM_PACK_EVIDENCE_TIER_MAX_DOCS_ISSUES = 10
SEAMGRIM_PACK_EVIDENCE_TIER_EXPECTED_REPO_ISSUES = 0
SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SUMMARY_KEYS = (
    "ci_sanity_seamgrim_pack_evidence_tier_runner_ok",
    "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path",
    "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists",
    "ci_sanity_seamgrim_pack_evidence_tier_runner_schema",
    "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count",
    "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count",
)
SEAMGRIM_NUMERIC_FACTOR_POLICY_ENABLED_PROFILES = {"full", "seamgrim"}
SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA = "ddn.numeric_factor_route_diag_contract.v1"
SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS: dict[str, int] = {
    "bit_limit": 512,
    "pollard_iters": 200000,
    "pollard_c_seeds": 64,
    "pollard_x0_seeds": 6,
    "fallback_limit": 1000000,
    "small_prime_max": 101,
}
SEAMGRIM_NUMERIC_FACTOR_POLICY_KEYS = (
    "bit_limit",
    "pollard_iters",
    "pollard_c_seeds",
    "pollard_x0_seeds",
    "fallback_limit",
    "small_prime_max",
)
SEAMGRIM_NUMERIC_FACTOR_POLICY_SUMMARY_KEYS = (
    "ci_sanity_seamgrim_numeric_factor_policy_ok",
    "ci_sanity_seamgrim_numeric_factor_policy_report_path",
    "ci_sanity_seamgrim_numeric_factor_policy_report_exists",
    "ci_sanity_seamgrim_numeric_factor_policy_schema",
    "ci_sanity_seamgrim_numeric_factor_policy_text",
    "ci_sanity_seamgrim_numeric_factor_policy_bit_limit",
    "ci_sanity_seamgrim_numeric_factor_policy_pollard_iters",
    "ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds",
    "ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds",
    "ci_sanity_seamgrim_numeric_factor_policy_fallback_limit",
    "ci_sanity_seamgrim_numeric_factor_policy_small_prime_max",
)
FIXED64_DARWIN_REAL_REPORT_LIVE_SUMMARY_ENABLED_PROFILES = {"full", "core_lang", "seamgrim"}
FIXED64_DARWIN_REAL_REPORT_LIVE_SUMMARY_KEYS = (
    "ci_sanity_fixed64_darwin_real_report_live_report_path",
    "ci_sanity_fixed64_darwin_real_report_live_report_exists",
    "ci_sanity_fixed64_darwin_real_report_live_status",
    "ci_sanity_fixed64_darwin_real_report_live_resolved_status",
    "ci_sanity_fixed64_darwin_real_report_live_resolved_source",
    "ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count",
    "ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip",
)
AGE3_COMPLETION_GATE_CRITERIA_ENABLED_PROFILES = {"full", "core_lang", "seamgrim"}

CORE_LANG_PROFILE_STEPS = {
    "backup_hygiene_selftest",
    "pipeline_emit_flags_check",
    "pipeline_emit_flags_selftest",
    "ci_emit_artifacts_sanity_contract_selftest",
    "age5_combined_heavy_policy_selftest",
    "profile_matrix_full_real_smoke_policy_selftest",
    "profile_matrix_full_real_smoke_check_selftest",
    "fixed64_darwin_probe_schedule_policy_check",
    "fixed64_darwin_real_report_contract_check",
    "fixed64_darwin_real_report_live_check",
    "ci_sanity_fixed64_live_summary_fields_selftest",
    "fixed64_darwin_real_report_live_check_selftest",
    "fixed64_darwin_real_report_readiness_check_selftest",
    "fixed64_threeway_inputs_selftest",
    "fixed64_cross_platform_threeway_gate_selftest",
    "featured_seed_catalog_autogen_check",
    "ci_profile_split_contract_check",
    "ci_sanity_dynamic_source_profile_split_selftest",
    "ci_profile_matrix_lightweight_contract_selftest",
    "ci_profile_matrix_snapshot_helper_selftest",
    "ssot_sync_dr173_177_check",
    "age2_completion_gate",
    "age2_completion_gate_selftest",
    "age2_close_selftest",
    "age2_close",
    "age3_completion_gate",
    "age3_completion_gate_selftest",
    "age3_close_selftest",
    "gate0_contract_abort_state_check",
    "contract_tier_unsupported_check",
    "contract_tier_age3_min_enforcement_check",
    "map_access_contract_check",
    "gaji_registry_strict_audit_check",
    "gaji_registry_defaults_check",
    "stdlib_catalog_check",
    "stdlib_catalog_check_selftest",
    "tensor_v0_pack_check",
    "tensor_v0_cli_check",
    "nurigym_shared_sync_priority_tiebreak_pack_check",
    "nurigym_shared_sync_action_pipeline_pack_check",
    "nuri_gym_contract_check",
    "sam_inputsnapshot_contract_pack_check",
    "sam_ai_ordering_pack_check",
    "seulgi_gatekeeper_pack_check",
    "sam_seulgi_family_contract_selftest",
    "external_intent_seulgi_walk_alignment_check_selftest",
    "age5_close_pack_contract_selftest",
    "age5_close_combined_report_contract_selftest",
    "ci_gate_summary_line_check_selftest",
    "w107_transport_contract_summary_selftest",
    "age4_proof_transport_contract_selftest",
    "age4_proof_transport_contract_report_selftest",
    "age4_proof_transport_contract_summary_selftest",
    "age4_proof_quantifier_case_analysis_selftest",
    "proof_case_analysis_completion_parity_selftest",
    "age1_immediate_proof_operation_matrix_selftest",
    "age1_immediate_proof_operation_contract_selftest",
    "age1_immediate_proof_operation_contract_summary_selftest",
    "age1_immediate_proof_operation_transport_contract_summary_selftest",
    "age1_immediate_proof_solver_search_matrix_selftest",
    "proof_solver_search_operation_parity_selftest",
    "proof_solver_operation_family_selftest",
    "proof_operation_family_selftest",
    "proof_operation_family_contract_selftest",
    "proof_family_selftest",
    "proof_family_contract_selftest",
    "proof_family_contract_summary_selftest",
    "proof_family_transport_contract_selftest",
    "proof_family_transport_contract_summary_selftest",
    "lang_surface_family_selftest",
    "lang_surface_family_contract_selftest",
    "lang_surface_family_contract_summary_selftest",
    "lang_surface_family_transport_contract_selftest",
    "lang_surface_family_transport_contract_summary_selftest",
    "lang_runtime_family_selftest",
    "lang_runtime_family_contract_selftest",
    "lang_runtime_family_contract_summary_selftest",
    "lang_runtime_family_transport_contract_selftest",
    "lang_runtime_family_transport_contract_summary_selftest",
    "gate0_runtime_family_selftest",
    "gate0_runtime_family_contract_selftest",
    "gate0_runtime_family_contract_summary_selftest",
    "gate0_runtime_family_transport_contract_selftest",
    "gate0_runtime_family_transport_contract_summary_selftest",
    "gate0_family_selftest",
    "gate0_family_contract_selftest",
    "gate0_family_transport_contract_selftest",
    "gate0_family_contract_summary_selftest",
    "gate0_family_transport_contract_summary_selftest",
    "gate0_transport_family_selftest",
    "gate0_transport_family_contract_selftest",
    "gate0_transport_family_contract_summary_selftest",
    "gate0_transport_family_transport_contract_selftest",
    "gate0_transport_family_transport_contract_summary_selftest",
    "gate0_surface_family_selftest",
    "gate0_surface_family_contract_selftest",
    "gate0_surface_family_contract_summary_selftest",
    "gate0_surface_family_transport_contract_selftest",
    "gate0_surface_family_transport_contract_summary_selftest",
    "gate0_surface_transport_family_selftest",
    "gate0_surface_transport_family_contract_selftest",
    "gate0_surface_transport_family_contract_summary_selftest",
    "gate0_surface_transport_family_transport_contract_selftest",
    "gate0_surface_transport_family_transport_contract_summary_selftest",
    "compound_update_reject_contract_selftest",
    "bogae_shape_alias_contract_selftest",
    "bogae_alias_family_selftest",
    "bogae_alias_family_contract_selftest",
    "bogae_alias_family_contract_summary_selftest",
    "bogae_alias_family_transport_contract_selftest",
    "bogae_alias_family_transport_contract_summary_selftest",
    "bogae_alias_viewer_family_selftest",
    "ci_pack_golden_age5_surface_selftest",
    "ci_pack_golden_guideblock_selftest",
    "ci_pack_golden_exec_policy_selftest",
    "ci_pack_golden_jjaim_flatten_selftest",
    "ci_pack_golden_event_model_selftest",
    "ci_pack_golden_lang_consistency_selftest",
    "ci_pack_golden_graph_export_selftest",
    "alrim_dispatch_runtime_contract_selftest",
    "w49_golden_index_selfcheck",
    "w111_golden_index_selfcheck",
    "w109_golden_index_selfcheck",
    "w107_golden_index_selfcheck",
    "w107_progress_contract_selftest",
    "proof_artifact_certificate_contract_selftest",
    "proof_certificate_digest_axes_selftest",
    "proof_certificate_candidate_manifest_selftest",
    "proof_certificate_candidate_profile_split_selftest",
    "proof_certificate_candidate_layers_selftest",
    "proof_certificate_v1_promotion_candidate_selftest",
    "proof_certificate_v1_draft_pack_selftest",
    "proof_certificate_v1_draft_artifact_selftest",
    "proof_certificate_v1_draft_artifact_layers_selftest",
    "proof_certificate_v1_draft_contract_selftest",
    "proof_certificate_v1_schema_candidate_selftest",
    "proof_certificate_v1_schema_candidate_split_selftest",
    "proof_certificate_v1_promotion_selftest",
    "proof_certificate_v1_family_selftest",
    "proof_certificate_v1_family_contract_selftest",
    "proof_certificate_v1_family_contract_summary_selftest",
    "proof_certificate_v1_family_transport_contract_selftest",
    "proof_certificate_v1_family_transport_contract_summary_selftest",
    "proof_certificate_family_selftest",
    "proof_certificate_family_contract_selftest",
    "proof_certificate_family_contract_summary_selftest",
    "proof_certificate_family_transport_contract_selftest",
    "proof_certificate_family_transport_contract_summary_selftest",
    "proof_certificate_v1_runtime_emit_selftest",
    "proof_certificate_v1_signed_emit_selftest",
    "proof_certificate_v1_signed_emit_profile_selftest",
    "proof_certificate_v1_verify_bundle_selftest",
    "proof_certificate_v1_verify_report_selftest",
    "proof_certificate_v1_verify_report_digest_contract_selftest",
    "proof_certificate_v1_verify_report_digest_transport_contract_summary_selftest",
    "proof_certificate_v1_consumer_contract_selftest",
    "proof_certificate_v1_consumer_transport_contract_selftest",
    "proof_certificate_v1_consumer_transport_contract_summary_selftest",
    "proof_certificate_v1_signed_contract_selftest",
    "proof_artifact_cert_subject_pack_check",
    "ci_pack_golden_metadata_selftest",
    "ci_canon_ast_dpack_selftest",
    "w92_aot_pack_check",
    "w93_universe_pack_check",
    "w94_social_pack_check",
    "w95_cert_pack_check",
    "w96_somssi_pack_check",
    "w97_self_heal_pack_check",
}

SEAMGRIM_PROFILE_STEPS = {
    "age5_combined_heavy_policy_selftest",
    "profile_matrix_full_real_smoke_policy_selftest",
    "profile_matrix_full_real_smoke_check_selftest",
    "fixed64_darwin_probe_schedule_policy_check",
    "fixed64_darwin_real_report_contract_check",
    "fixed64_darwin_real_report_live_check",
    "ci_sanity_fixed64_live_summary_fields_selftest",
    "fixed64_darwin_real_report_live_check_selftest",
    "fixed64_darwin_real_report_readiness_check_selftest",
    "fixed64_threeway_inputs_selftest",
    "fixed64_cross_platform_threeway_gate_selftest",
    "featured_seed_catalog_autogen_check",
    "ci_profile_split_contract_check",
    "ci_sanity_dynamic_source_profile_split_selftest",
    "ci_emit_artifacts_sanity_contract_selftest",
    "ci_profile_matrix_lightweight_contract_selftest",
    "ci_profile_matrix_snapshot_helper_selftest",
    "ssot_sync_dr173_177_check",
    "age2_completion_gate",
    "age2_completion_gate_selftest",
    "age2_close_selftest",
    "age2_close",
    "age3_completion_gate",
    "age3_completion_gate_selftest",
    "age3_close_selftest",
    "age3_close",
    "sam_seulgi_family_contract_selftest",
    "external_intent_seulgi_walk_alignment_check_selftest",
    "seamgrim_ci_gate_seed_meta_step_check",
    "seamgrim_ci_gate_featured_seed_catalog_step_check",
    "seamgrim_ci_gate_featured_seed_catalog_autogen_step_check",
    "seamgrim_ci_gate_runtime5_passthrough_check",
    "seamgrim_ci_gate_lesson_warning_step_check",
    "seamgrim_ci_gate_stateful_preview_step_check",
    "seamgrim_ci_gate_wasm_web_smoke_step_check",
    "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
    "seamgrim_ci_gate_pack_evidence_tier_step_check",
    "seamgrim_ci_gate_pack_evidence_tier_step_check_selftest",
    "seamgrim_ci_gate_pack_evidence_tier_runner_check",
    "seamgrim_ci_gate_pack_evidence_tier_report_check",
    "seamgrim_ci_gate_pack_evidence_tier_report_check_selftest",
    "seamgrim_interface_boundary_contract_check",
    "seamgrim_overlay_session_wired_consistency_check",
    "seamgrim_overlay_session_diag_parity_check",
    "seamgrim_overlay_compare_diag_parity_check",
    "seamgrim_wasm_cli_diag_parity_check",
}


def clip(text: str, limit: int = 180) -> str:
    normalized = " ".join(str(text).split())
    if not normalized:
        return "-"
    if len(normalized) <= limit:
        return normalized
    return normalized[: limit - 3] + "..."


def parse_fail_code(stdout: str, stderr: str, default_code: str) -> str:
    payload = f"{stdout}\n{stderr}"
    match = FAIL_CODE_RE.search(payload)
    if match:
        return match.group(1)
    return default_code


def first_message(stdout: str, stderr: str) -> str:
    for raw in (stderr.splitlines() + stdout.splitlines()):
        line = raw.strip()
        if line:
            return clip(line)
    return "-"


def emit_text_safely(text: str, stream) -> None:
    if not text:
        return
    try:
        stream.write(text)
    except UnicodeEncodeError:
        encoding = getattr(stream, "encoding", None) or "utf-8"
        data = text.encode(encoding, errors="replace")
        buffer = getattr(stream, "buffer", None)
        if buffer is not None:
            buffer.write(data)
        else:
            stream.write(data.decode(encoding, errors="replace"))
    stream.flush()


def run_step(cmd: list[str], extra_env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    env = None
    if extra_env:
        env = dict(os.environ)
        env.update(extra_env)
    return subprocess.run(
        cmd,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def maybe_mark_step_script_ok(cmd: list[str]) -> None:
    if len(cmd) < 2:
        return
    script_path = str(cmd[1]).strip()
    if not script_path.endswith(".py"):
        return
    mark_script_ok(script_path)


def resolve_completion_gate_report_dir() -> Path:
    candidates = (
        Path("I:/home/urihanl/ddn/codex/build/reports"),
        Path("C:/ddn/codex/build/reports"),
        Path("build/reports"),
    )
    for base in candidates:
        try:
            base.mkdir(parents=True, exist_ok=True)
            return base
        except OSError:
            continue
    fallback = Path("build/reports")
    fallback.mkdir(parents=True, exist_ok=True)
    return fallback


def build_completion_gate_report_paths(profile: str) -> dict[str, str]:
    base = resolve_completion_gate_report_dir()
    run_token = f"{profile}.{os.getpid()}.{int(time.time() * 1000)}"
    return {
        "age2_gate": str(base / f"age2_completion_gate.{run_token}.detjson"),
        "age2_must": str(base / f"age2_completion_must_pack_report.{run_token}.detjson"),
        "age2_should": str(base / f"age2_completion_should_pack_report.{run_token}.detjson"),
        "age2_close": str(base / f"age2_close_report.{run_token}.detjson"),
        "age3_gate": str(base / f"age3_completion_gate.{run_token}.detjson"),
        "age3_pack": str(base / f"age3_completion_pack_report.{run_token}.detjson"),
        "age3_close": str(base / f"age3_close_report.{run_token}.detjson"),
        "seamgrim_gate_report": str(base / f"seamgrim_ci_gate_report.{run_token}.json"),
        "seamgrim_ui_age3_report": str(base / f"seamgrim_ui_age3_gate_report.{run_token}.detjson"),
        "age2_dynamic_source": str(base / f"age2_completion_gate.{profile}.dynamic.last.detjson"),
        "age3_dynamic_source": str(base / f"age3_completion_gate.{profile}.dynamic.last.detjson"),
        "fixed64_threeway_selftest_out_dir": str(base / f"fixed64_threeway_gate_selftest.{run_token}"),
        "fixed64_live_report": str(base / f"fixed64_darwin_real_report_live_check.{run_token}.detjson"),
        "age4_transport_report": str(base / f"age4_proof_transport_contract_selftest.{run_token}.detjson"),
        "age4_dynamic_source": str(base / f"age4_proof_transport_contract_selftest.{profile}.dynamic.last.detjson"),
        "age5_surface_report": str(base / f"pack_golden_age5_surface_selftest.{run_token}.detjson"),
        "age5_dynamic_source": str(base / f"pack_golden_age5_surface_selftest.{profile}.dynamic.last.detjson"),
        "seamgrim_wasm_web_step_check_report": str(
            base / f"age3_completion_gate.{run_token}.seamgrim_wasm_web_step_check.detjson"
        ),
        "seamgrim_pack_evidence_tier_runner_report": str(
            base / f"age3_completion_gate.{run_token}.seamgrim_pack_evidence_tier_runner_check.detjson"
        ),
        "seamgrim_wasm_cli_diag_parity_report": str(
            base / f"age3_completion_gate.{run_token}.seamgrim_wasm_cli_diag_parity.detjson"
        ),
    }


def resolve_dynamic_hint_lookback(profile: str) -> int:
    # core_lang은 반복 실행 밀도가 높아 최근 3회 스무딩을 유지한다.
    if profile == "core_lang":
        return 3
    # full/seamgrim은 변동성보다 민첩한 반영을 위해 최근 1회 기준을 사용한다.
    return 1


def resolve_dynamic_worker_lookback(profile: str) -> int:
    # AGE4는 워커 수 변동 민감도가 있어 core_lang에서는 3회 median 기반으로 완화한다.
    if profile == "core_lang":
        return 3
    return 1


def derive_age3_bogae_geoul_visibility_smoke_report_path(age3_gate_report_path_text: str) -> str:
    gate_path = Path(str(age3_gate_report_path_text).strip())
    return str(gate_path.parent / f"{gate_path.stem}.bogae_geoul_visibility_smoke.detjson")


def build_age3_bogae_geoul_visibility_smoke_summary_fields(
    profile: str,
    rows: list[dict[str, object]],
    completion_gate_reports: dict[str, str] | None,
) -> dict[str, str]:
    report_path_default = "-"
    if isinstance(completion_gate_reports, dict):
        age3_gate_report_path_text = str(completion_gate_reports.get("age3_gate", "")).strip()
        if age3_gate_report_path_text:
            report_path_default = derive_age3_bogae_geoul_visibility_smoke_report_path(age3_gate_report_path_text)

    if profile not in AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_ENABLED_PROFILES:
        return {
            "ci_sanity_age3_bogae_geoul_visibility_smoke_ok": "na",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path": "-",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists": "na",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_schema": "-",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok": "na",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok": "na",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes": "na",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes": "na",
        }

    step_index = {
        str(row.get("step", "")).strip(): row
        for row in rows
        if isinstance(row, dict) and str(row.get("step", "")).strip()
    }
    age3_step = step_index.get("age3_completion_gate")
    if age3_step is None:
        return {
            "ci_sanity_age3_bogae_geoul_visibility_smoke_ok": "pending",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path": report_path_default,
            "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists": "0",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_schema": "-",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok": "pending",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok": "pending",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes": "pending",
            "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes": "pending",
        }

    try:
        step_rc = int(age3_step.get("returncode", -1))
    except Exception:
        step_rc = -1
    age3_step_ok = bool(age3_step.get("ok", False)) and step_rc == 0

    age3_gate_report_path_text = ""
    if isinstance(completion_gate_reports, dict):
        age3_gate_report_path_text = str(completion_gate_reports.get("age3_gate", "")).strip()
    age3_gate_report_doc = load_json_snapshot(age3_gate_report_path_text)

    smoke_report_path_text = (
        str(age3_gate_report_doc.get("bogae_geoul_visibility_smoke_report_path", "")).strip()
        if isinstance(age3_gate_report_doc, dict)
        else ""
    )
    if not smoke_report_path_text:
        smoke_report_path_text = (
            derive_age3_bogae_geoul_visibility_smoke_report_path(age3_gate_report_path_text)
            if age3_gate_report_path_text
            else report_path_default
        )

    smoke_report_doc = load_json_snapshot(smoke_report_path_text)
    smoke_report_exists = isinstance(smoke_report_doc, dict)
    smoke_schema = "-"
    smoke_overall_ok = "0"
    smoke_checks_ok = "0"
    smoke_sim_state_hash_changes = "0"
    smoke_sim_bogae_hash_changes = "0"
    if isinstance(smoke_report_doc, dict):
        smoke_schema = str(smoke_report_doc.get("schema", "")).strip() or "-"
        smoke_overall_ok = "1" if bool(smoke_report_doc.get("overall_ok", False)) else "0"
        checks = smoke_report_doc.get("checks")
        smoke_checks_ok = "1" if isinstance(checks, list) and bool(checks) else "0"
        sim_hash_delta = smoke_report_doc.get("simulation_hash_delta")
        if isinstance(sim_hash_delta, dict):
            smoke_sim_state_hash_changes = "1" if bool(sim_hash_delta.get("state_hash_changes", False)) else "0"
            smoke_sim_bogae_hash_changes = "1" if bool(sim_hash_delta.get("bogae_hash_changes", False)) else "0"
    smoke_ok = (
        smoke_report_exists
        and smoke_schema == AGE3_BOGAE_GEOUL_VISIBILITY_SMOKE_SCHEMA
        and smoke_overall_ok == "1"
        and smoke_checks_ok == "1"
        and smoke_sim_state_hash_changes == "1"
        and smoke_sim_bogae_hash_changes == "1"
    )
    return {
        "ci_sanity_age3_bogae_geoul_visibility_smoke_ok": "1" if age3_step_ok and smoke_ok else "0",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path": smoke_report_path_text or "-",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists": "1" if smoke_report_exists else "0",
        "ci_sanity_age3_bogae_geoul_visibility_smoke_schema": smoke_schema,
        "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok": smoke_overall_ok,
        "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok": smoke_checks_ok,
        "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes": smoke_sim_state_hash_changes,
        "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes": smoke_sim_bogae_hash_changes,
    }


def build_seamgrim_wasm_web_step_check_summary_fields(
    profile: str,
    rows: list[dict[str, object]],
    completion_gate_reports: dict[str, str] | None,
) -> dict[str, str]:
    report_path_default = "-"
    if isinstance(completion_gate_reports, dict):
        report_path_default = str(completion_gate_reports.get("seamgrim_wasm_web_step_check_report", "")).strip() or "-"

    if profile not in SEAMGRIM_WASM_WEB_STEP_CHECK_ENABLED_PROFILES:
        return {
            "ci_sanity_seamgrim_wasm_web_step_check_ok": "na",
            "ci_sanity_seamgrim_wasm_web_step_check_report_path": "-",
            "ci_sanity_seamgrim_wasm_web_step_check_report_exists": "na",
            "ci_sanity_seamgrim_wasm_web_step_check_schema": "-",
            "ci_sanity_seamgrim_wasm_web_step_check_checked_files": "-",
            "ci_sanity_seamgrim_wasm_web_step_check_missing_count": "-",
        }

    step_index = {
        str(row.get("step", "")).strip(): row
        for row in rows
        if isinstance(row, dict) and str(row.get("step", "")).strip()
    }
    step_row = step_index.get("seamgrim_ci_gate_wasm_web_smoke_step_check")
    if step_row is None:
        return {
            "ci_sanity_seamgrim_wasm_web_step_check_ok": "pending",
            "ci_sanity_seamgrim_wasm_web_step_check_report_path": report_path_default,
            "ci_sanity_seamgrim_wasm_web_step_check_report_exists": "0",
            "ci_sanity_seamgrim_wasm_web_step_check_schema": "-",
            "ci_sanity_seamgrim_wasm_web_step_check_checked_files": "pending",
            "ci_sanity_seamgrim_wasm_web_step_check_missing_count": "pending",
        }

    try:
        step_rc = int(step_row.get("returncode", -1))
    except Exception:
        step_rc = -1
    step_ok = bool(step_row.get("ok", False)) and step_rc == 0

    report_path_text = report_path_default
    if isinstance(completion_gate_reports, dict):
        report_path_text = str(completion_gate_reports.get("seamgrim_wasm_web_step_check_report", "")).strip() or report_path_default

    report_doc = load_json_snapshot(report_path_text)
    report_exists = isinstance(report_doc, dict)
    report_schema = "-"
    report_checked_files = "0"
    report_missing_count = "-1"
    report_ok = False
    if isinstance(report_doc, dict):
        report_schema = str(report_doc.get("schema", "")).strip() or "-"
        try:
            checked_files_num = int(report_doc.get("checked_files", -1))
        except Exception:
            checked_files_num = -1
        report_checked_files = str(checked_files_num if checked_files_num >= 0 else 0)
        try:
            missing_count_num = int(report_doc.get("missing_count", -1))
        except Exception:
            missing_count_num = -1
        report_missing_count = str(missing_count_num)
        report_ok = (
            report_schema == SEAMGRIM_WASM_WEB_STEP_CHECK_SCHEMA
            and str(report_doc.get("status", "")).strip() == "pass"
            and bool(report_doc.get("ok", False))
            and str(report_doc.get("code", "")).strip() == "OK"
            and checked_files_num >= SEAMGRIM_WASM_WEB_STEP_CHECK_MIN_FILES
            and missing_count_num == 0
            and isinstance(report_doc.get("missing"), list)
            and not report_doc.get("missing")
        )
    return {
        "ci_sanity_seamgrim_wasm_web_step_check_ok": "1" if step_ok and report_ok else "0",
        "ci_sanity_seamgrim_wasm_web_step_check_report_path": report_path_text or "-",
        "ci_sanity_seamgrim_wasm_web_step_check_report_exists": "1" if report_exists else "0",
        "ci_sanity_seamgrim_wasm_web_step_check_schema": report_schema,
        "ci_sanity_seamgrim_wasm_web_step_check_checked_files": report_checked_files,
        "ci_sanity_seamgrim_wasm_web_step_check_missing_count": report_missing_count,
    }


def _step_pass(row: dict[str, object] | None) -> bool:
    if not isinstance(row, dict):
        return False
    try:
        rc = int(row.get("returncode", -1))
    except Exception:
        rc = -1
    return bool(row.get("ok", False)) and rc == 0


def build_seamgrim_pack_evidence_tier_runner_summary_fields(
    profile: str,
    rows: list[dict[str, object]],
    completion_gate_reports: dict[str, str] | None,
) -> dict[str, str]:
    report_path_default = "-"
    if isinstance(completion_gate_reports, dict):
        report_path_default = (
            str(completion_gate_reports.get("seamgrim_pack_evidence_tier_runner_report", "")).strip() or "-"
        )

    if profile not in SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_ENABLED_PROFILES:
        return {
            "ci_sanity_seamgrim_pack_evidence_tier_runner_ok": "na",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path": "-",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists": "na",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_schema": "-",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count": "-",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count": "-",
        }

    step_index = {
        str(row.get("step", "")).strip(): row
        for row in rows
        if isinstance(row, dict) and str(row.get("step", "")).strip()
    }
    required_steps = (
        "seamgrim_ci_gate_pack_evidence_tier_step_check",
        "seamgrim_ci_gate_pack_evidence_tier_step_check_selftest",
        "seamgrim_ci_gate_pack_evidence_tier_runner_check",
        "seamgrim_ci_gate_pack_evidence_tier_report_check",
        "seamgrim_ci_gate_pack_evidence_tier_report_check_selftest",
    )
    if any(step_index.get(step_name) is None for step_name in required_steps):
        return {
            "ci_sanity_seamgrim_pack_evidence_tier_runner_ok": "pending",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path": report_path_default,
            "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists": "0",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_schema": "-",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count": "pending",
            "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count": "pending",
        }

    all_steps_ok = all(_step_pass(step_index.get(step_name)) for step_name in required_steps)

    report_path_text = report_path_default
    if isinstance(completion_gate_reports, dict):
        report_path_text = (
            str(completion_gate_reports.get("seamgrim_pack_evidence_tier_runner_report", "")).strip()
            or report_path_default
        )

    report_doc = load_json_snapshot(report_path_text)
    report_exists = isinstance(report_doc, dict)
    report_schema = "-"
    docs_issue_count = "-1"
    repo_issue_count = "-1"
    report_ok = False
    if isinstance(report_doc, dict):
        report_schema = str(report_doc.get("schema", "")).strip() or "-"
        docs_profile = report_doc.get("docs_profile")
        repo_profile = report_doc.get("repo_profile")
        if isinstance(docs_profile, dict):
            try:
                docs_issue_count = str(int(docs_profile.get("issue_count", -1)))
            except Exception:
                docs_issue_count = "-1"
        if isinstance(repo_profile, dict):
            try:
                repo_issue_count = str(int(repo_profile.get("issue_count", -1)))
            except Exception:
                repo_issue_count = "-1"
        try:
            docs_issue_num = int(docs_issue_count)
        except Exception:
            docs_issue_num = -1
        try:
            repo_issue_num = int(repo_issue_count)
        except Exception:
            repo_issue_num = -1
        report_ok = (
            report_schema == SEAMGRIM_PACK_EVIDENCE_TIER_RUNNER_SCHEMA
            and bool(report_doc.get("ok", False))
            and isinstance(docs_profile, dict)
            and isinstance(repo_profile, dict)
            and str(docs_profile.get("name", "")).strip() == "docs_ssot_rep10"
            and str(repo_profile.get("name", "")).strip() == "repo_rep10"
            and docs_issue_num >= 0
            and docs_issue_num <= SEAMGRIM_PACK_EVIDENCE_TIER_MAX_DOCS_ISSUES
            and repo_issue_num == SEAMGRIM_PACK_EVIDENCE_TIER_EXPECTED_REPO_ISSUES
        )
    return {
        "ci_sanity_seamgrim_pack_evidence_tier_runner_ok": "1" if all_steps_ok and report_ok else "0",
        "ci_sanity_seamgrim_pack_evidence_tier_runner_report_path": report_path_text or "-",
        "ci_sanity_seamgrim_pack_evidence_tier_runner_report_exists": "1" if report_exists else "0",
        "ci_sanity_seamgrim_pack_evidence_tier_runner_schema": report_schema,
        "ci_sanity_seamgrim_pack_evidence_tier_runner_docs_issue_count": docs_issue_count,
        "ci_sanity_seamgrim_pack_evidence_tier_runner_repo_issue_count": repo_issue_count,
    }


def build_seamgrim_numeric_factor_policy_summary_fields(
    profile: str,
    rows: list[dict[str, object]],
    completion_gate_reports: dict[str, str] | None,
) -> dict[str, str]:
    parity_report_path_default = "-"
    if isinstance(completion_gate_reports, dict):
        parity_report_path_default = (
            str(completion_gate_reports.get("seamgrim_wasm_cli_diag_parity_report", "")).strip() or "-"
        )

    if profile not in SEAMGRIM_NUMERIC_FACTOR_POLICY_ENABLED_PROFILES:
        return {
            "ci_sanity_seamgrim_numeric_factor_policy_ok": "na",
            "ci_sanity_seamgrim_numeric_factor_policy_report_path": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_report_exists": "na",
            "ci_sanity_seamgrim_numeric_factor_policy_schema": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_text": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_bit_limit": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_iters": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_fallback_limit": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_small_prime_max": "-",
        }

    step_index = {
        str(row.get("step", "")).strip(): row
        for row in rows
        if isinstance(row, dict) and str(row.get("step", "")).strip()
    }
    step_row = step_index.get("seamgrim_wasm_cli_diag_parity_check")
    if step_row is None:
        return {
            "ci_sanity_seamgrim_numeric_factor_policy_ok": "pending",
            "ci_sanity_seamgrim_numeric_factor_policy_report_path": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_report_exists": "0",
            "ci_sanity_seamgrim_numeric_factor_policy_schema": "-",
            "ci_sanity_seamgrim_numeric_factor_policy_text": "pending",
            "ci_sanity_seamgrim_numeric_factor_policy_bit_limit": "pending",
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_iters": "pending",
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds": "pending",
            "ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds": "pending",
            "ci_sanity_seamgrim_numeric_factor_policy_fallback_limit": "pending",
            "ci_sanity_seamgrim_numeric_factor_policy_small_prime_max": "pending",
        }
    step_ok = _step_pass(step_row)

    parity_report_doc = load_json_snapshot(parity_report_path_default)
    parity_report_exists = isinstance(parity_report_doc, dict)
    report_path_text = "-"
    report_exists = False
    report_schema = "-"
    policy_text = "-"
    parsed_policy: dict[str, int] = {}
    report_ok = False
    if isinstance(parity_report_doc, dict):
        parity_schema = str(parity_report_doc.get("schema", "")).strip() or "-"
        report_path_text = str(parity_report_doc.get("numeric_factor_policy_report_path", "")).strip() or "-"
        parity_policy_text = str(parity_report_doc.get("numeric_factor_policy_text", "")).strip() or "-"
        parity_policy_raw = parity_report_doc.get("numeric_factor_policy")
        report_doc = load_json_snapshot(report_path_text)
        report_exists = isinstance(report_doc, dict)
        if isinstance(report_doc, dict):
            report_schema = str(report_doc.get("schema", "")).strip() or "-"
            policy_text = str(report_doc.get("numeric_factor_policy_text", "")).strip() or "-"
            policy_raw = report_doc.get("numeric_factor_policy")
        else:
            policy_text = parity_policy_text
            policy_raw = parity_policy_raw
        if isinstance(policy_raw, dict):
            for key in SEAMGRIM_NUMERIC_FACTOR_POLICY_KEYS:
                try:
                    parsed_policy[key] = int(policy_raw.get(key))
                except Exception:
                    parsed_policy = {}
                    break
        report_ok = (
            parity_schema == "ddn.seamgrim.wasm_cli_diag_parity.v1"
            and str(parity_report_doc.get("status", "")).strip() == "pass"
            and bool(parity_report_doc.get("ok", False))
            and str(parity_report_doc.get("code", "")).strip() == "OK"
            and parity_report_exists
            and report_exists
            and report_schema == SEAMGRIM_NUMERIC_FACTOR_POLICY_SCHEMA
            and str(report_doc.get("status", "")).strip() == "pass"
            and bool(report_doc.get("ok", False))
            and str(report_doc.get("code", "")).strip() == "OK"
            and policy_text not in {"", "-", "pending"}
            and parsed_policy == SEAMGRIM_NUMERIC_FACTOR_POLICY_DEFAULTS
        )

    def value_or_neg1(key: str) -> str:
        if key in parsed_policy:
            return str(parsed_policy[key])
        return "-1"

    return {
        "ci_sanity_seamgrim_numeric_factor_policy_ok": "1" if step_ok and report_ok else "0",
        "ci_sanity_seamgrim_numeric_factor_policy_report_path": report_path_text or "-",
        "ci_sanity_seamgrim_numeric_factor_policy_report_exists": "1" if report_exists else "0",
        "ci_sanity_seamgrim_numeric_factor_policy_schema": report_schema,
        "ci_sanity_seamgrim_numeric_factor_policy_text": policy_text,
        "ci_sanity_seamgrim_numeric_factor_policy_bit_limit": value_or_neg1("bit_limit"),
        "ci_sanity_seamgrim_numeric_factor_policy_pollard_iters": value_or_neg1("pollard_iters"),
        "ci_sanity_seamgrim_numeric_factor_policy_pollard_c_seeds": value_or_neg1("pollard_c_seeds"),
        "ci_sanity_seamgrim_numeric_factor_policy_pollard_x0_seeds": value_or_neg1("pollard_x0_seeds"),
        "ci_sanity_seamgrim_numeric_factor_policy_fallback_limit": value_or_neg1("fallback_limit"),
        "ci_sanity_seamgrim_numeric_factor_policy_small_prime_max": value_or_neg1("small_prime_max"),
    }


def _extract_failure_codes(report_doc: dict[str, object] | None) -> list[str]:
    if not isinstance(report_doc, dict):
        return []
    raw = report_doc.get("failure_codes")
    if not isinstance(raw, list):
        return []
    out: list[str] = []
    for item in raw:
        code = str(item).strip()
        if not code or code in out:
            continue
        out.append(code)
    return out


def _extract_age3_criteria_status_map(report_doc: dict[str, object] | None) -> dict[str, bool]:
    if not isinstance(report_doc, dict):
        return {}
    raw = report_doc.get("criteria")
    if not isinstance(raw, list):
        return {}
    out: dict[str, bool] = {}
    for row in raw:
        if not isinstance(row, dict):
            continue
        name = str(row.get("name", "")).strip()
        if not name:
            continue
        out[name] = bool(row.get("ok", False))
    return out


def build_age3_completion_gate_criteria_summary_fields(
    profile: str,
    rows: list[dict[str, object]],
    completion_gate_reports: dict[str, str] | None,
) -> dict[str, str]:
    if profile not in AGE3_COMPLETION_GATE_CRITERIA_ENABLED_PROFILES:
        return {key: "na" for key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS}

    step_index = {
        str(row.get("step", "")).strip(): row
        for row in rows
        if isinstance(row, dict) and str(row.get("step", "")).strip()
    }
    age3_step = step_index.get("age3_completion_gate")
    if age3_step is None:
        return {key: "pending" for key in AGE3_COMPLETION_GATE_CRITERIA_SUMMARY_KEYS}

    report_path = ""
    if isinstance(completion_gate_reports, dict):
        report_path = str(completion_gate_reports.get("age3_gate", "")).strip()
    criteria_status_map = _extract_age3_criteria_status_map(load_json_snapshot(report_path))

    out: dict[str, str] = {}
    for criteria_name in AGE3_COMPLETION_GATE_CRITERIA_NAMES:
        key = age3_completion_gate_criteria_summary_key(criteria_name)
        if criteria_name in criteria_status_map:
            out[key] = "1" if criteria_status_map[criteria_name] else "0"
        else:
            out[key] = "0"
    return out


def build_completion_gate_failure_code_summary_fields(
    profile: str,
    rows: list[dict[str, object]],
    completion_gate_reports: dict[str, str] | None,
) -> dict[str, str]:
    enabled_profiles = {"core_lang", "full", "seamgrim"}
    if profile not in enabled_profiles:
        return {
            "ci_sanity_age2_completion_gate_failure_codes": "na",
            "ci_sanity_age2_completion_gate_failure_code_count": "na",
            "ci_sanity_age3_completion_gate_failure_codes": "na",
            "ci_sanity_age3_completion_gate_failure_code_count": "na",
        }

    step_index = {
        str(row.get("step", "")).strip(): row
        for row in rows
        if isinstance(row, dict) and str(row.get("step", "")).strip()
    }
    age2_step = step_index.get("age2_completion_gate")
    age3_step = step_index.get("age3_completion_gate")

    def resolve_codes(step_row: dict[str, object] | None, report_key: str) -> tuple[str, str]:
        if step_row is None:
            return "pending", "pending"
        report_path = ""
        if isinstance(completion_gate_reports, dict):
            report_path = str(completion_gate_reports.get(report_key, "")).strip()
        report_doc = load_json_snapshot(report_path)
        codes = _extract_failure_codes(report_doc)
        if not codes:
            return "-", "0"
        return ",".join(codes), str(len(codes))

    age2_codes, age2_count = resolve_codes(age2_step, "age2_gate")
    age3_codes, age3_count = resolve_codes(age3_step, "age3_gate")
    return {
        "ci_sanity_age2_completion_gate_failure_codes": age2_codes,
        "ci_sanity_age2_completion_gate_failure_code_count": age2_count,
        "ci_sanity_age3_completion_gate_failure_codes": age3_codes,
        "ci_sanity_age3_completion_gate_failure_code_count": age3_count,
    }


def build_fixed64_darwin_real_report_live_summary_fields(
    profile: str,
    rows: list[dict[str, object]],
    completion_gate_reports: dict[str, str] | None,
) -> dict[str, str]:
    if profile not in FIXED64_DARWIN_REAL_REPORT_LIVE_SUMMARY_ENABLED_PROFILES:
        return {key: "na" for key in FIXED64_DARWIN_REAL_REPORT_LIVE_SUMMARY_KEYS}

    report_path_default = "-"
    if isinstance(completion_gate_reports, dict):
        report_path_default = str(completion_gate_reports.get("fixed64_live_report", "")).strip() or "-"

    step_index = {
        str(row.get("step", "")).strip(): row
        for row in rows
        if isinstance(row, dict) and str(row.get("step", "")).strip()
    }
    live_step = step_index.get("fixed64_darwin_real_report_live_check")
    if live_step is None:
        return {
            "ci_sanity_fixed64_darwin_real_report_live_report_path": report_path_default,
            "ci_sanity_fixed64_darwin_real_report_live_report_exists": "pending",
            "ci_sanity_fixed64_darwin_real_report_live_status": "pending",
            "ci_sanity_fixed64_darwin_real_report_live_resolved_status": "pending",
            "ci_sanity_fixed64_darwin_real_report_live_resolved_source": "pending",
            "ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count": "pending",
            "ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip": "pending",
        }

    report_path_text = report_path_default
    report_doc = load_json_snapshot(report_path_text)
    report_exists = isinstance(report_doc, dict)
    status = "-"
    resolved_status = "-"
    resolved_source = "-"
    resolve_invalid_hit_count = "-"
    resolved_source_zip = "-"
    if isinstance(report_doc, dict):
        status = str(report_doc.get("status", "")).strip() or "-"
        resolved_status = str(report_doc.get("resolved_status", "")).strip() or "-"
        resolved_source = str(report_doc.get("resolved_source", "")).strip() or "-"
        invalid_hits = report_doc.get("resolve_invalid_hits")
        if isinstance(invalid_hits, list):
            resolve_invalid_hit_count = str(
                len([str(item).strip() for item in invalid_hits if str(item).strip()])
            )
        else:
            resolve_invalid_hit_count = "0"
        resolved_source_zip = "1" if ".zip!" in resolved_source else "0"
    return {
        "ci_sanity_fixed64_darwin_real_report_live_report_path": report_path_text or "-",
        "ci_sanity_fixed64_darwin_real_report_live_report_exists": "1" if report_exists else "0",
        "ci_sanity_fixed64_darwin_real_report_live_status": status,
        "ci_sanity_fixed64_darwin_real_report_live_resolved_status": resolved_status,
        "ci_sanity_fixed64_darwin_real_report_live_resolved_source": resolved_source,
        "ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count": resolve_invalid_hit_count,
        "ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip": resolved_source_zip,
    }


def build_sanity_summary_fields(
    profile: str,
    rows: list[dict[str, object]],
    completion_gate_reports: dict[str, str] | None = None,
) -> dict[str, str]:
    step_index = {
        str(row.get("step", "")).strip(): row for row in rows if isinstance(row, dict) and str(row.get("step", "")).strip()
    }
    out: dict[str, str] = build_age5_combined_heavy_sanity_contract_fields()
    for key, step_name, enabled_profiles in SANITY_SUMMARY_STEP_FIELDS:
        if profile not in enabled_profiles:
            out[key] = "na"
            continue
        row = step_index.get(step_name)
        if row is None:
            out[key] = "pending"
            continue
        try:
            rc = int(row.get("returncode", -1))
        except Exception:
            rc = -1
        out[key] = "1" if bool(row.get("ok", False)) and rc == 0 else "0"
    pack_graph_export_step = step_index.get("ci_pack_golden_graph_export_selftest")
    if profile in PACK_GOLDEN_GRAPH_EXPORT_REQUIRED_PROFILES:
        if pack_graph_export_step is None:
            out[PACK_GOLDEN_GRAPH_EXPORT_SUMMARY_KEY] = "pending"
        else:
            try:
                pack_graph_export_rc = int(pack_graph_export_step.get("returncode", -1))
            except Exception:
                pack_graph_export_rc = -1
            out[PACK_GOLDEN_GRAPH_EXPORT_SUMMARY_KEY] = (
                "1" if bool(pack_graph_export_step.get("ok", False)) and pack_graph_export_rc == 0 else "0"
            )
    else:
        out[PACK_GOLDEN_GRAPH_EXPORT_SUMMARY_KEY] = "0"
    out.update(
        build_age3_bogae_geoul_visibility_smoke_summary_fields(
            profile=profile,
            rows=rows,
            completion_gate_reports=completion_gate_reports,
        )
    )
    out.update(
        build_seamgrim_wasm_web_step_check_summary_fields(
            profile=profile,
            rows=rows,
            completion_gate_reports=completion_gate_reports,
        )
    )
    out.update(
        build_seamgrim_pack_evidence_tier_runner_summary_fields(
            profile=profile,
            rows=rows,
            completion_gate_reports=completion_gate_reports,
        )
    )
    out.update(
        build_seamgrim_numeric_factor_policy_summary_fields(
            profile=profile,
            rows=rows,
            completion_gate_reports=completion_gate_reports,
        )
    )
    out.update(
        build_completion_gate_failure_code_summary_fields(
            profile=profile,
            rows=rows,
            completion_gate_reports=completion_gate_reports,
        )
    )
    out.update(
        build_age3_completion_gate_criteria_summary_fields(
            profile=profile,
            rows=rows,
            completion_gate_reports=completion_gate_reports,
        )
    )
    out.update(
        build_fixed64_darwin_real_report_live_summary_fields(
            profile=profile,
            rows=rows,
            completion_gate_reports=completion_gate_reports,
        )
    )
    return out


def load_json_snapshot(path_text: str) -> dict[str, object] | None:
    if not str(path_text).strip():
        return None
    path = Path(path_text)
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def sync_json_report_snapshot(source_path_text: str, target_path_text: str) -> None:
    source = Path(str(source_path_text).strip())
    target = Path(str(target_path_text).strip())
    if not source.exists() or not str(target).strip():
        return
    payload = load_json_snapshot(str(source))
    if not isinstance(payload, dict):
        return
    write_json_snapshot(str(target), payload)


def write_json_snapshot(path_text: str, payload: dict[str, object]) -> None:
    if not str(path_text).strip():
        return
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def build_step_progress_env(step_name: str, json_out: str) -> dict[str, str] | None:
    if not str(json_out).strip():
        return None
    base = Path(json_out)
    if step_name == "pipeline_emit_flags_check":
        progress_path = base.with_name(f"{base.stem}.pipeline_emit_flags_check.progress.detjson")
        return {"DDN_CI_PIPELINE_EMIT_FLAGS_PROGRESS_JSON": str(progress_path)}
    if step_name == "pipeline_emit_flags_selftest":
        progress_path = base.with_name(f"{base.stem}.pipeline_emit_flags_selftest.progress.detjson")
        return {"DDN_CI_PIPELINE_EMIT_FLAGS_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "age5_combined_heavy_policy_selftest":
        progress_path = base.with_name(f"{base.stem}.age5_combined_heavy_policy_selftest.progress.detjson")
        return {"DDN_AGE5_COMBINED_HEAVY_POLICY_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "profile_matrix_full_real_smoke_policy_selftest":
        progress_path = base.with_name(f"{base.stem}.profile_matrix_full_real_smoke_policy_selftest.progress.detjson")
        return {"DDN_PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "profile_matrix_full_real_smoke_check_selftest":
        progress_path = base.with_name(f"{base.stem}.profile_matrix_full_real_smoke_check_selftest.progress.detjson")
        return {"DDN_PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "fixed64_darwin_real_report_readiness_check_selftest":
        progress_path = base.with_name(
            f"{base.stem}.fixed64_darwin_real_report_readiness_check_selftest.progress.detjson"
        )
        return {"DDN_FIXED64_DARWIN_REAL_REPORT_READINESS_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "map_access_contract_check":
        progress_path = base.with_name(f"{base.stem}.map_access_contract_check.progress.detjson")
        return {"DDN_MAP_ACCESS_CONTRACT_CHECK_PROGRESS_JSON": str(progress_path)}
    if step_name == "tensor_v0_cli_check":
        progress_path = base.with_name(f"{base.stem}.tensor_v0_cli_check.progress.detjson")
        return {"DDN_TENSOR_V0_CLI_CHECK_PROGRESS_JSON": str(progress_path)}
    if step_name == "ci_pack_golden_age5_surface_selftest":
        progress_path = base.with_name(f"{base.stem}.ci_pack_golden_age5_surface_selftest.progress.detjson")
        return {"DDN_CI_PACK_GOLDEN_AGE5_SURFACE_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "ci_pack_golden_guideblock_selftest":
        progress_path = base.with_name(f"{base.stem}.ci_pack_golden_guideblock_selftest.progress.detjson")
        return {"DDN_CI_PACK_GOLDEN_GUIDEBLOCK_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "ci_pack_golden_exec_policy_selftest":
        progress_path = base.with_name(f"{base.stem}.ci_pack_golden_exec_policy_selftest.progress.detjson")
        return {"DDN_CI_PACK_GOLDEN_EXEC_POLICY_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "ci_pack_golden_jjaim_flatten_selftest":
        progress_path = base.with_name(f"{base.stem}.ci_pack_golden_jjaim_flatten_selftest.progress.detjson")
        return {"DDN_CI_PACK_GOLDEN_JJAIM_FLATTEN_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "ci_pack_golden_event_model_selftest":
        progress_path = base.with_name(f"{base.stem}.ci_pack_golden_event_model_selftest.progress.detjson")
        return {"DDN_CI_PACK_GOLDEN_EVENT_MODEL_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "ci_pack_golden_lang_consistency_selftest":
        progress_path = base.with_name(f"{base.stem}.ci_pack_golden_lang_consistency_selftest.progress.detjson")
        return {"DDN_CI_PACK_GOLDEN_LANG_CONSISTENCY_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "w49_golden_index_selfcheck":
        progress_path = base.with_name(f"{base.stem}.w49_golden_index_selfcheck.progress.detjson")
        return {"DDN_W49_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON": str(progress_path)}
    if step_name == "w111_golden_index_selfcheck":
        progress_path = base.with_name(f"{base.stem}.w111_golden_index_selfcheck.progress.detjson")
        return {"DDN_W111_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON": str(progress_path)}
    if step_name == "w109_golden_index_selfcheck":
        progress_path = base.with_name(f"{base.stem}.w109_golden_index_selfcheck.progress.detjson")
        return {"DDN_W109_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON": str(progress_path)}
    if step_name in {"w107_golden_index_selfcheck", "w107_golden_index_selftest"}:
        progress_path = base.with_name(f"{base.stem}.w107_golden_index_selfcheck.progress.detjson")
        return {
            "DDN_W107_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON": str(progress_path),
            "DDN_W107_GOLDEN_INDEX_SELFTEST_PROGRESS_JSON": str(progress_path),
        }
    if step_name == "w107_progress_contract_selftest":
        progress_path = base.with_name(f"{base.stem}.w107_progress_contract_selftest.progress.detjson")
        return {"DDN_W107_PROGRESS_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "age4_proof_transport_contract_selftest":
        progress_path = base.with_name(f"{base.stem}.age4_proof_transport_contract_selftest.progress.detjson")
        return {"DDN_AGE4_PROOF_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "proof_operation_family_contract_selftest":
        progress_path = base.with_name(f"{base.stem}.proof_operation_family_contract_selftest.progress.detjson")
        return {"DDN_PROOF_OPERATION_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "proof_family_contract_selftest":
        progress_path = base.with_name(f"{base.stem}.proof_family_contract_selftest.progress.detjson")
        return {"DDN_PROOF_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "sam_seulgi_family_contract_selftest":
        progress_path = base.with_name(f"{base.stem}.sam_seulgi_family_contract_selftest.progress.detjson")
        return {"DDN_SAM_SEULGI_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "proof_family_transport_contract_selftest":
        progress_path = base.with_name(f"{base.stem}.proof_family_transport_contract_selftest.progress.detjson")
        return {"DDN_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "lang_surface_family_contract_selftest":
        progress_path = base.with_name(f"{base.stem}.lang_surface_family_contract_selftest.progress.detjson")
        return {"DDN_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "lang_surface_family_transport_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.lang_surface_family_transport_contract_selftest.progress.detjson"
        )
        return {"DDN_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "lang_runtime_family_contract_selftest":
        progress_path = base.with_name(f"{base.stem}.lang_runtime_family_contract_selftest.progress.detjson")
        return {"DDN_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "lang_runtime_family_transport_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.lang_runtime_family_transport_contract_selftest.progress.detjson"
        )
        return {"DDN_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "gate0_runtime_family_contract_selftest":
        progress_path = base.with_name(f"{base.stem}.gate0_runtime_family_contract_selftest.progress.detjson")
        return {"DDN_GATE0_RUNTIME_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "gate0_runtime_family_transport_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.gate0_runtime_family_transport_contract_selftest.progress.detjson"
        )
        return {"DDN_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "gate0_family_contract_selftest":
        progress_path = base.with_name(f"{base.stem}.gate0_family_contract_selftest.progress.detjson")
        return {"DDN_GATE0_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "gate0_family_transport_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.gate0_family_transport_contract_selftest.progress.detjson"
        )
        return {"DDN_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "gate0_transport_family_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.gate0_transport_family_contract_selftest.progress.detjson"
        )
        return {"DDN_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "gate0_transport_family_transport_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.gate0_transport_family_transport_contract_selftest.progress.detjson"
        )
        return {
            "DDN_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)
        }
    if step_name == "gate0_surface_family_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.gate0_surface_family_contract_selftest.progress.detjson"
        )
        return {"DDN_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "gate0_surface_family_transport_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.gate0_surface_family_transport_contract_selftest.progress.detjson"
        )
        return {"DDN_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "gate0_surface_transport_family_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.gate0_surface_transport_family_contract_selftest.progress.detjson"
        )
        return {
            "DDN_GATE0_SURFACE_TRANSPORT_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)
        }
    if step_name == "gate0_surface_transport_family_transport_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.gate0_surface_transport_family_transport_contract_selftest.progress.detjson"
        )
        return {
            "DDN_GATE0_SURFACE_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON": str(
                progress_path
            )
        }
    if step_name == "age1_immediate_proof_operation_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.age1_immediate_proof_operation_contract_selftest.progress.detjson"
        )
        return {"DDN_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "proof_certificate_v1_consumer_transport_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.proof_certificate_v1_consumer_transport_contract_selftest.progress.detjson"
        )
        return {
            "DDN_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)
        }
    if step_name == "proof_certificate_v1_verify_report_digest_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.proof_certificate_v1_verify_report_digest_contract_selftest.progress.detjson"
        )
        return {
            "DDN_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)
        }
    if step_name == "proof_certificate_v1_family_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.proof_certificate_v1_family_contract_selftest.progress.detjson"
        )
        return {"DDN_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "proof_certificate_v1_family_transport_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.proof_certificate_v1_family_transport_contract_selftest.progress.detjson"
        )
        return {"DDN_PROOF_CERTIFICATE_V1_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "proof_certificate_family_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.proof_certificate_family_contract_selftest.progress.detjson"
        )
        return {"DDN_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "proof_certificate_family_transport_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.proof_certificate_family_transport_contract_selftest.progress.detjson"
        )
        return {"DDN_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "bogae_alias_family_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.bogae_alias_family_contract_selftest.progress.detjson"
        )
        return {"DDN_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "bogae_alias_family_transport_contract_selftest":
        progress_path = base.with_name(
            f"{base.stem}.bogae_alias_family_transport_contract_selftest.progress.detjson"
        )
        return {"DDN_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON": str(progress_path)}
    if step_name == "w94_social_pack_check":
        progress_path = base.with_name(f"{base.stem}.w94_social_pack_check.progress.detjson")
        return {"DDN_W94_SOCIAL_PACK_CHECK_PROGRESS_JSON": str(progress_path)}
    if step_name == "w95_cert_pack_check":
        progress_path = base.with_name(f"{base.stem}.w95_cert_pack_check.progress.detjson")
        return {"DDN_W95_CERT_PACK_CHECK_PROGRESS_JSON": str(progress_path)}
    if step_name == "w96_somssi_pack_check":
        progress_path = base.with_name(f"{base.stem}.w96_somssi_pack_check.progress.detjson")
        return {"DDN_W96_SOMSSI_PACK_CHECK_PROGRESS_JSON": str(progress_path)}
    if step_name == "w97_self_heal_pack_check":
        progress_path = base.with_name(f"{base.stem}.w97_self_heal_pack_check.progress.detjson")
        return {"DDN_W97_SELF_HEAL_PACK_CHECK_PROGRESS_JSON": str(progress_path)}
    return None


def write_step_seed_progress(step_name: str, step_env: dict[str, str] | None) -> None:
    if not isinstance(step_env, dict):
        return
    if step_name == "profile_matrix_full_real_smoke_check_selftest":
        progress_path = str(step_env.get("DDN_PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SELFTEST_PROGRESS_JSON", "")).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.profile_matrix_full_real_smoke_check_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "total_elapsed_ms": "0",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "fixed64_darwin_real_report_readiness_check_selftest":
        progress_path = str(
            step_env.get("DDN_FIXED64_DARWIN_REAL_REPORT_READINESS_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.fixed64.darwin_real_report_readiness_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "total_elapsed_ms": "0",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "map_access_contract_check":
        progress_path = str(step_env.get("DDN_MAP_ACCESS_CONTRACT_CHECK_PROGRESS_JSON", "")).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.map_access_contract_check.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "total_elapsed_ms": "0",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "tensor_v0_cli_check":
        progress_path = str(step_env.get("DDN_TENSOR_V0_CLI_CHECK_PROGRESS_JSON", "")).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.tensor_v0_cli_check.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "total_elapsed_ms": "0",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "proof_certificate_v1_verify_report_digest_contract_selftest":
        progress_path = str(
            step_env.get("DDN_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.proof_certificate_v1_verify_report_digest_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "completed_checks": 0,
                "total_checks": 1,
                "checks_text": "verify_report_digest_contract",
            },
        )
        return
    if step_name == "proof_certificate_v1_family_contract_selftest":
        progress_path = str(
            step_env.get("DDN_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.proof_certificate_v1_family_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "completed_checks": 0,
                "total_checks": 4,
                "checks_text": "signed_contract,consumer_contract,promotion,family",
            },
        )
        return
    if step_name == "proof_certificate_v1_family_transport_contract_selftest":
        progress_path = str(
            step_env.get("DDN_PROOF_CERTIFICATE_V1_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.proof_certificate_v1_family_transport_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "completed_checks": 0,
                "total_checks": 9,
                "checks_text": "family_contract,aggregate_preview_summary,aggregate_status_line,final_status_line,gate_result,gate_outputs_consistency,gate_summary_line,final_line_emitter,report_index",
            },
        )
        return
    if step_name == "proof_certificate_family_contract_selftest":
        progress_path = str(
            step_env.get("DDN_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.proof_certificate_family_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "completed_checks": 0,
                "total_checks": 3,
                "checks_text": "artifact_certificate_contract,proof_certificate_v1_family,proof_certificate_family",
            },
        )
        return
    if step_name == "ci_pack_golden_age5_surface_selftest":
        progress_path = str(
            step_env.get("DDN_CI_PACK_GOLDEN_AGE5_SURFACE_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.pack_golden_age5_surface_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "total_elapsed_ms": "0",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "proof_certificate_family_transport_contract_selftest":
        progress_path = str(
            step_env.get("DDN_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.proof_certificate_family_transport_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 9,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "proof_family_transport_contract_selftest":
        progress_path = str(step_env.get("DDN_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON", "")).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.proof_family_transport_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 9,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": (
                    "family_contract,aggregate_preview_summary,aggregate_status_line,"
                    "final_status_line,gate_result,gate_outputs_consistency,"
                    "gate_summary_line,final_line_emitter,report_index"
                ),
            },
        )
        return
    if step_name == "sam_seulgi_family_contract_selftest":
        progress_path = str(
            step_env.get("DDN_SAM_SEULGI_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.sam_seulgi_family_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 6,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": (
                    "external_intent_boundary_pack,seulgi_v1_pack,"
                    "sam_inputsnapshot_contract_pack,sam_ai_ordering_pack,"
                    "seulgi_gatekeeper_pack,external_intent_seulgi_walk_alignment"
                ),
            },
        )
        return
    if step_name == "lang_surface_family_transport_contract_selftest":
        progress_path = str(
            step_env.get("DDN_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.lang_surface_family_transport_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 9,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": (
                    "family_contract,aggregate_preview_summary,aggregate_status_line,"
                    "final_status_line,gate_result,gate_outputs_consistency,"
                    "gate_summary_line,final_line_emitter,report_index"
                ),
            },
        )
        return
    if step_name == "ci_pack_golden_guideblock_selftest":
        progress_path = str(
            step_env.get("DDN_CI_PACK_GOLDEN_GUIDEBLOCK_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.pack_golden_guideblock_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "total_elapsed_ms": "0",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "ci_pack_golden_exec_policy_selftest":
        progress_path = str(
            step_env.get("DDN_CI_PACK_GOLDEN_EXEC_POLICY_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.pack_golden_exec_policy_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "total_elapsed_ms": "0",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "ci_pack_golden_jjaim_flatten_selftest":
        progress_path = str(
            step_env.get("DDN_CI_PACK_GOLDEN_JJAIM_FLATTEN_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.pack_golden_jjaim_flatten_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "total_elapsed_ms": "0",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "ci_pack_golden_event_model_selftest":
        progress_path = str(
            step_env.get("DDN_CI_PACK_GOLDEN_EVENT_MODEL_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.pack_golden_event_model_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "total_elapsed_ms": "0",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "ci_pack_golden_lang_consistency_selftest":
        progress_path = str(
            step_env.get("DDN_CI_PACK_GOLDEN_LANG_CONSISTENCY_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.pack_golden_lang_consistency_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "total_elapsed_ms": "0",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "w49_golden_index_selfcheck":
        progress_path = str(
            step_env.get("DDN_W49_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.w49_golden_index_selfcheck.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "active_cases": 0,
                "index_codes": 0,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "w111_golden_index_selfcheck":
        progress_path = str(
            step_env.get("DDN_W111_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.w111_golden_index_selfcheck.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "active_cases": 0,
                "index_codes": 0,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "w109_golden_index_selfcheck":
        progress_path = str(
            step_env.get("DDN_W109_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.w109_golden_index_selfcheck.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "active_cases": 0,
                "index_codes": 0,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name in {"w107_golden_index_selfcheck", "w107_golden_index_selftest"}:
        progress_path = str(
            step_env.get("DDN_W107_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            progress_path = str(
                step_env.get("DDN_W107_GOLDEN_INDEX_SELFTEST_PROGRESS_JSON", "")
            ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.w107_golden_index_selfcheck.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "active_cases": 0,
                "inactive_cases": 0,
                "index_codes": 0,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "w107_progress_contract_selftest":
        progress_path = str(
            step_env.get("DDN_W107_PROGRESS_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.w107_progress_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 8,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "age4_proof_transport_contract_selftest":
        progress_path = str(
            step_env.get("DDN_AGE4_PROOF_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.age4_proof_transport_contract_selftest.progress.v1",
                "status": "running",
                "current_check": "-",
                "last_completed_check": "-",
                "completed_checks": 0,
                "total_checks": 8,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "proof_operation_family_contract_selftest":
        progress_path = str(
            step_env.get("DDN_PROOF_OPERATION_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.proof_operation_family_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 4,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "proof_family_contract_selftest":
        progress_path = str(step_env.get("DDN_PROOF_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.proof_family_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 3,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": "proof_operation_family,proof_certificate_family,proof_family",
            },
        )
        return
    if step_name == "bogae_alias_family_contract_selftest":
        progress_path = str(
            step_env.get("DDN_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.bogae_alias_family_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 3,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": "shape_alias_contract,alias_family,alias_viewer_family",
            },
        )
        return
    if step_name == "bogae_alias_family_transport_contract_selftest":
        progress_path = str(
            step_env.get("DDN_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.bogae_alias_family_transport_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 9,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": (
                    "family_contract,aggregate_preview_summary,aggregate_status_line,"
                    "final_status_line,gate_result,gate_outputs_consistency,"
                    "gate_summary_line,final_line_emitter,report_index"
                ),
            },
        )
        return
    if step_name == "lang_runtime_family_contract_selftest":
        progress_path = str(step_env.get("DDN_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.lang_runtime_family_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 5,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": "lang_surface_family,stdlib_catalog,tensor_pack,tensor_cli,lang_runtime_family",
            },
        )
        return
    if step_name == "lang_runtime_family_transport_contract_selftest":
        progress_path = str(
            step_env.get("DDN_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.lang_runtime_family_transport_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 9,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": (
                    "family_contract,aggregate_preview_summary,aggregate_status_line,"
                    "final_status_line,gate_result,gate_outputs_consistency,"
                    "gate_summary_line,final_line_emitter,report_index"
                ),
            },
        )
        return
    if step_name == "gate0_runtime_family_contract_selftest":
        progress_path = str(step_env.get("DDN_GATE0_RUNTIME_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.gate0_runtime_family_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 5,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": "lang_runtime_family,w95_cert,w96_somssi,w97_self_heal,gate0_runtime_family",
            },
        )
        return
    if step_name == "gate0_runtime_family_transport_contract_selftest":
        progress_path = str(
            step_env.get("DDN_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.gate0_runtime_family_transport_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 1,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": "family_contract",
            },
        )
        return
    if step_name == "gate0_family_contract_selftest":
        progress_path = str(step_env.get("DDN_GATE0_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.gate0_family_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 5,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": "gate0_runtime_family,w92_aot,w93_universe,w94_social,gate0_family",
            },
        )
        return
    if step_name == "gate0_family_transport_contract_selftest":
        progress_path = str(
            step_env.get("DDN_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.gate0_family_transport_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 9,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": (
                    "family_contract,aggregate_preview_summary,aggregate_status_line,"
                    "final_status_line,gate_result,gate_outputs_consistency,"
                    "gate_summary_line,final_line_emitter,report_index"
                ),
            },
        )
        return
    if step_name == "gate0_transport_family_contract_selftest":
        progress_path = str(
            step_env.get("DDN_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.gate0_transport_family_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 4,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": (
                    "lang_runtime_family_transport,gate0_runtime_family_transport,"
                    "gate0_family_transport,gate0_transport_family"
                ),
            },
        )
        return
    if step_name == "gate0_transport_family_transport_contract_selftest":
        progress_path = str(
            step_env.get(
                "DDN_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON", ""
            )
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.gate0_transport_family_transport_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 9,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": (
                    "family_contract,aggregate_preview_summary,aggregate_status_line,"
                    "final_status_line,gate_result,gate_outputs_consistency,"
                    "gate_summary_line,final_line_emitter,report_index"
                ),
            },
        )
        return
    if step_name == "gate0_surface_family_contract_selftest":
        progress_path = str(step_env.get("DDN_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.gate0_surface_family_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 5,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": (
                    "lang_surface_family,lang_runtime_family,gate0_runtime_family,"
                    "gate0_family,gate0_transport_family"
                ),
            },
        )
        return
    if step_name == "gate0_surface_family_transport_contract_selftest":
        progress_path = str(
            step_env.get("DDN_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.gate0_surface_family_transport_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 9,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": (
                    "family_contract,aggregate_preview_summary,aggregate_status_line,"
                    "final_status_line,gate_result,gate_outputs_consistency,"
                    "gate_summary_line,final_line_emitter,report_index"
                ),
            },
        )
        return
    if step_name == "gate0_surface_transport_family_contract_selftest":
        progress_path = str(
            step_env.get("DDN_GATE0_SURFACE_TRANSPORT_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.gate0_surface_transport_family_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 6,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": (
                    "lang_surface_family_transport,lang_runtime_family_transport,"
                    "gate0_runtime_family_transport,gate0_family_transport,"
                    "gate0_transport_family_transport,gate0_surface_family_transport"
                ),
            },
        )
        return
    if step_name == "gate0_surface_transport_family_transport_contract_selftest":
        progress_path = str(
            step_env.get(
                "DDN_GATE0_SURFACE_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON", ""
            )
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.gate0_surface_transport_family_transport_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 7,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
                "checks_text": (
                    "family_contract,lang_surface_transport,lang_runtime_transport,"
                    "gate0_runtime_transport,gate0_family_transport,"
                    "gate0_transport_family_transport,gate0_surface_family_transport"
                ),
            },
        )
        return
    if step_name == "age1_immediate_proof_operation_contract_selftest":
        progress_path = str(
            step_env.get("DDN_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.age1_immediate_proof_operation_contract_selftest.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "completed_checks": 0,
                "total_checks": 5,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name in {"w107_golden_index_selfcheck", "w107_golden_index_selftest"}:
        progress_path = str(
            step_env.get("DDN_W107_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            progress_path = str(
                step_env.get("DDN_W107_GOLDEN_INDEX_SELFTEST_PROGRESS_JSON", "")
            ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.w107_golden_index_selfcheck.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "active_cases": 0,
                "inactive_cases": 0,
                "index_codes": 0,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "w49_golden_index_selfcheck":
        progress_path = str(
            step_env.get("DDN_W49_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.w49_golden_index_selfcheck.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "active_cases": 0,
                "index_codes": 0,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "w111_golden_index_selfcheck":
        progress_path = str(
            step_env.get("DDN_W111_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.w111_golden_index_selfcheck.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "active_cases": 0,
                "index_codes": 0,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "w109_golden_index_selfcheck":
        progress_path = str(
            step_env.get("DDN_W109_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON", "")
        ).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.ci.w109_golden_index_selfcheck.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "active_cases": 0,
                "index_codes": 0,
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "w94_social_pack_check":
        progress_path = str(step_env.get("DDN_W94_SOCIAL_PACK_CHECK_PROGRESS_JSON", "")).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.w94_social_pack_check.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "total_elapsed_ms": "0",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "w95_cert_pack_check":
        progress_path = str(step_env.get("DDN_W95_CERT_PACK_CHECK_PROGRESS_JSON", "")).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.w95_cert_pack_check.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "total_elapsed_ms": "0",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "w96_somssi_pack_check":
        progress_path = str(step_env.get("DDN_W96_SOMSSI_PACK_CHECK_PROGRESS_JSON", "")).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.w96_somssi_pack_check.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "total_elapsed_ms": "0",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )
        return
    if step_name == "w97_self_heal_pack_check":
        progress_path = str(step_env.get("DDN_W97_SELF_HEAL_PACK_CHECK_PROGRESS_JSON", "")).strip()
        if not progress_path:
            return
        write_json_snapshot(
            progress_path,
            {
                "schema": "ddn.w97_self_heal_pack_check.progress.v1",
                "status": "running",
                "current_case": "-",
                "last_completed_case": "-",
                "total_elapsed_ms": "0",
                "current_probe": "spawn_process",
                "last_completed_probe": "-",
            },
        )


def emit_step_progress_tokens(step_name: str, step_env: dict[str, str] | None) -> None:
    if not isinstance(step_env, dict):
        return
    if step_name == "w49_golden_index_selfcheck":
        payload = load_json_snapshot(str(step_env.get("DDN_W49_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON", "")).strip())
        if not isinstance(payload, dict):
            return
        print(
            f"w49_golden_index_selfcheck_current_probe={str(payload.get('current_probe', '')).strip() or '-'}"
        )
        print(
            "w49_golden_index_selfcheck_last_completed_probe="
            + (str(payload.get("last_completed_probe", "")).strip() or "-")
        )
        print(f"w49_golden_index_selfcheck_active_cases={str(payload.get('active_cases', '')).strip() or '-'}")
        print(f"w49_golden_index_selfcheck_index_codes={str(payload.get('index_codes', '')).strip() or '-'}")
        return
    if step_name == "w111_golden_index_selfcheck":
        payload = load_json_snapshot(str(step_env.get("DDN_W111_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON", "")).strip())
        if not isinstance(payload, dict):
            return
        print(
            f"w111_golden_index_selfcheck_current_probe={str(payload.get('current_probe', '')).strip() or '-'}"
        )
        print(
            "w111_golden_index_selfcheck_last_completed_probe="
            + (str(payload.get("last_completed_probe", "")).strip() or "-")
        )
        print(f"w111_golden_index_selfcheck_active_cases={str(payload.get('active_cases', '')).strip() or '-'}")
        print(f"w111_golden_index_selfcheck_index_codes={str(payload.get('index_codes', '')).strip() or '-'}")
        return
    if step_name == "w109_golden_index_selfcheck":
        payload = load_json_snapshot(str(step_env.get("DDN_W109_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON", "")).strip())
        if not isinstance(payload, dict):
            return
        print(
            f"w109_golden_index_selfcheck_current_probe={str(payload.get('current_probe', '')).strip() or '-'}"
        )
        print(
            "w109_golden_index_selfcheck_last_completed_probe="
            + (str(payload.get("last_completed_probe", "")).strip() or "-")
        )
        print(f"w109_golden_index_selfcheck_active_cases={str(payload.get('active_cases', '')).strip() or '-'}")
        print(f"w109_golden_index_selfcheck_index_codes={str(payload.get('index_codes', '')).strip() or '-'}")
        return
    if step_name in {"w107_golden_index_selfcheck", "w107_golden_index_selftest"}:
        progress_path = str(step_env.get("DDN_W107_GOLDEN_INDEX_SELFCHECK_PROGRESS_JSON", "")).strip()
        if not progress_path:
            progress_path = str(step_env.get("DDN_W107_GOLDEN_INDEX_SELFTEST_PROGRESS_JSON", "")).strip()
        payload = load_json_snapshot(progress_path)
        if not isinstance(payload, dict):
            return
        print(f"w107_golden_index_selfcheck_current_probe={str(payload.get('current_probe', '')).strip() or '-'}")
        print(
            "w107_golden_index_selfcheck_last_completed_probe="
            + (str(payload.get("last_completed_probe", "")).strip() or "-")
        )
        print(f"w107_golden_index_selfcheck_active_cases={str(payload.get('active_cases', '')).strip() or '-'}")
        print(
            "w107_golden_index_selfcheck_inactive_cases="
            + (str(payload.get("inactive_cases", "")).strip() or "-")
        )
        print(f"w107_golden_index_selfcheck_index_codes={str(payload.get('index_codes', '')).strip() or '-'}")
        print(f"w107_golden_index_selftest_current_probe={str(payload.get('current_probe', '')).strip() or '-'}")
        print(
            "w107_golden_index_selftest_last_completed_probe="
            + (str(payload.get("last_completed_probe", "")).strip() or "-")
        )
        print(f"w107_golden_index_selftest_active_cases={str(payload.get('active_cases', '')).strip() or '-'}")
        print(
            "w107_golden_index_selftest_inactive_cases="
            + (str(payload.get("inactive_cases", "")).strip() or "-")
        )
        print(f"w107_golden_index_selftest_index_codes={str(payload.get('index_codes', '')).strip() or '-'}")
        return
    if step_name != "w107_progress_contract_selftest":
        if step_name == "age4_proof_transport_contract_selftest":
            payload = load_json_snapshot(
                str(step_env.get("DDN_AGE4_PROOF_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON", "")).strip()
            )
            if not isinstance(payload, dict):
                return
            print(
                "age4_proof_transport_contract_selftest_current_probe="
                + (str(payload.get("current_probe", "")).strip() or "-")
            )
            print(
                "age4_proof_transport_contract_selftest_last_completed_probe="
                + (str(payload.get("last_completed_probe", "")).strip() or "-")
            )
            print(
                "age4_proof_transport_contract_selftest_completed_checks="
                + (str(payload.get("completed_checks", "")).strip() or "-")
            )
            print(
                "age4_proof_transport_contract_selftest_total_checks="
                + (str(payload.get("total_checks", "")).strip() or "-")
            )
            print(
                "age4_proof_transport_contract_selftest_checks_text="
                + (str(payload.get("checks_text", "")).strip() or "-")
            )
            return
        if step_name != "proof_operation_family_contract_selftest":
            if step_name == "proof_certificate_v1_family_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get(
                            "DDN_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON",
                            "",
                        )
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "proof_certificate_v1_family_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "proof_certificate_v1_family_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "proof_certificate_v1_family_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "proof_certificate_v1_family_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "proof_certificate_v1_family_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "proof_certificate_v1_family_transport_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get(
                            "DDN_PROOF_CERTIFICATE_V1_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON",
                            "",
                        )
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "proof_certificate_v1_family_transport_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "proof_certificate_v1_family_transport_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "proof_certificate_v1_family_transport_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "proof_certificate_v1_family_transport_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "proof_certificate_v1_family_transport_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "proof_certificate_family_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get(
                            "DDN_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON",
                            "",
                        )
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "proof_certificate_family_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "proof_certificate_family_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "proof_certificate_family_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "proof_certificate_family_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "proof_certificate_family_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "proof_certificate_family_transport_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get(
                            "DDN_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON",
                            "",
                        )
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "proof_certificate_family_transport_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "proof_certificate_family_transport_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "proof_certificate_family_transport_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "proof_certificate_family_transport_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "proof_certificate_family_transport_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "proof_family_contract_selftest":
                payload = load_json_snapshot(
                    str(step_env.get("DDN_PROOF_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "proof_family_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "proof_family_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "proof_family_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "proof_family_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "proof_family_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "sam_seulgi_family_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get("DDN_SAM_SEULGI_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "sam_seulgi_family_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "sam_seulgi_family_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "sam_seulgi_family_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "sam_seulgi_family_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "sam_seulgi_family_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "proof_family_transport_contract_selftest":
                payload = load_json_snapshot(
                    str(step_env.get("DDN_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON", "")).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "proof_family_transport_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "proof_family_transport_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "proof_family_transport_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "proof_family_transport_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "proof_family_transport_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "lang_surface_family_contract_selftest":
                payload = load_json_snapshot(
                    str(step_env.get("DDN_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "lang_surface_family_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "lang_surface_family_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "lang_surface_family_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "lang_surface_family_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "lang_surface_family_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "lang_surface_family_transport_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get(
                            "DDN_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON",
                            "",
                        )
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "lang_surface_family_transport_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "lang_surface_family_transport_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "lang_surface_family_transport_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "lang_surface_family_transport_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "lang_surface_family_transport_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "lang_runtime_family_contract_selftest":
                payload = load_json_snapshot(
                    str(step_env.get("DDN_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "lang_runtime_family_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "lang_runtime_family_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "lang_runtime_family_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "lang_runtime_family_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "lang_runtime_family_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "lang_runtime_family_transport_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get("DDN_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON", "")
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "lang_runtime_family_transport_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "lang_runtime_family_transport_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "lang_runtime_family_transport_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "lang_runtime_family_transport_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "lang_runtime_family_transport_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "gate0_runtime_family_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get("DDN_GATE0_RUNTIME_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "gate0_runtime_family_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "gate0_runtime_family_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "gate0_runtime_family_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "gate0_runtime_family_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "gate0_runtime_family_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "gate0_runtime_family_transport_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get(
                            "DDN_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON",
                            "",
                        )
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "gate0_runtime_family_transport_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "gate0_runtime_family_transport_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "gate0_runtime_family_transport_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "gate0_runtime_family_transport_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "gate0_runtime_family_transport_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "gate0_family_contract_selftest":
                payload = load_json_snapshot(
                    str(step_env.get("DDN_GATE0_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "gate0_family_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "gate0_family_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "gate0_family_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "gate0_family_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "gate0_family_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "gate0_family_transport_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get(
                            "DDN_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON",
                            "",
                        )
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "gate0_family_transport_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "gate0_family_transport_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "gate0_family_transport_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "gate0_family_transport_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "gate0_family_transport_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "gate0_transport_family_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get(
                            "DDN_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON",
                            "",
                        )
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "gate0_transport_family_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "gate0_transport_family_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "gate0_transport_family_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "gate0_transport_family_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "gate0_transport_family_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "gate0_transport_family_transport_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get(
                            "DDN_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON",
                            "",
                        )
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "gate0_transport_family_transport_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "gate0_transport_family_transport_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "gate0_transport_family_transport_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "gate0_transport_family_transport_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "gate0_transport_family_transport_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "gate0_surface_family_contract_selftest":
                payload = load_json_snapshot(
                    str(step_env.get("DDN_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "gate0_surface_family_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "gate0_surface_family_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "gate0_surface_family_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "gate0_surface_family_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "gate0_surface_family_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "gate0_surface_family_transport_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get(
                            "DDN_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON",
                            "",
                        )
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "gate0_surface_family_transport_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "gate0_surface_family_transport_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "gate0_surface_family_transport_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "gate0_surface_family_transport_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "gate0_surface_family_transport_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "gate0_surface_transport_family_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get(
                            "DDN_GATE0_SURFACE_TRANSPORT_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON",
                            "",
                        )
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "gate0_surface_transport_family_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "gate0_surface_transport_family_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "gate0_surface_transport_family_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "gate0_surface_transport_family_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "gate0_surface_transport_family_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "gate0_surface_transport_family_transport_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get(
                            "DDN_GATE0_SURFACE_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON",
                            "",
                        )
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "gate0_surface_transport_family_transport_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "gate0_surface_transport_family_transport_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "gate0_surface_transport_family_transport_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "gate0_surface_transport_family_transport_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "gate0_surface_transport_family_transport_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "bogae_alias_family_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get("DDN_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "bogae_alias_family_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "bogae_alias_family_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "bogae_alias_family_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "bogae_alias_family_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "bogae_alias_family_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "bogae_alias_family_transport_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get("DDN_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON", "")
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "bogae_alias_family_transport_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "bogae_alias_family_transport_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "bogae_alias_family_transport_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "bogae_alias_family_transport_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "bogae_alias_family_transport_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "proof_certificate_v1_verify_report_digest_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get(
                            "DDN_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_PROGRESS_JSON",
                            "",
                        )
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "proof_certificate_v1_verify_report_digest_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "proof_certificate_v1_verify_report_digest_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "proof_certificate_v1_verify_report_digest_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "proof_certificate_v1_verify_report_digest_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "proof_certificate_v1_verify_report_digest_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name == "proof_certificate_v1_consumer_transport_contract_selftest":
                payload = load_json_snapshot(
                    str(
                        step_env.get(
                            "DDN_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON", ""
                        )
                    ).strip()
                )
                if not isinstance(payload, dict):
                    return
                print(
                    "proof_certificate_v1_consumer_transport_contract_selftest_current_probe="
                    + (str(payload.get("current_probe", "")).strip() or "-")
                )
                print(
                    "proof_certificate_v1_consumer_transport_contract_selftest_last_completed_probe="
                    + (str(payload.get("last_completed_probe", "")).strip() or "-")
                )
                print(
                    "proof_certificate_v1_consumer_transport_contract_selftest_completed_checks="
                    + (str(payload.get("completed_checks", "")).strip() or "-")
                )
                print(
                    "proof_certificate_v1_consumer_transport_contract_selftest_total_checks="
                    + (str(payload.get("total_checks", "")).strip() or "-")
                )
                print(
                    "proof_certificate_v1_consumer_transport_contract_selftest_checks_text="
                    + (str(payload.get("checks_text", "")).strip() or "-")
                )
                return
            if step_name != "age1_immediate_proof_operation_contract_selftest":
                return
            payload = load_json_snapshot(
                str(
                    step_env.get("DDN_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_PROGRESS_JSON", "")
                ).strip()
            )
            if not isinstance(payload, dict):
                return
            print(
                "age1_immediate_proof_operation_contract_selftest_current_probe="
                + (str(payload.get("current_probe", "")).strip() or "-")
            )
            print(
                "age1_immediate_proof_operation_contract_selftest_last_completed_probe="
                + (str(payload.get("last_completed_probe", "")).strip() or "-")
            )
            print(
                "age1_immediate_proof_operation_contract_selftest_completed_checks="
                + (str(payload.get("completed_checks", "")).strip() or "-")
            )
            print(
                "age1_immediate_proof_operation_contract_selftest_total_checks="
                + (str(payload.get("total_checks", "")).strip() or "-")
            )
            print(
                "age1_immediate_proof_operation_contract_selftest_checks_text="
                + (str(payload.get("checks_text", "")).strip() or "-")
            )
            return
        payload = load_json_snapshot(
            str(step_env.get("DDN_PROOF_OPERATION_FAMILY_CONTRACT_SELFTEST_PROGRESS_JSON", "")).strip()
        )
        if not isinstance(payload, dict):
            return
        print(
            "proof_operation_family_contract_selftest_current_probe="
            + (str(payload.get("current_probe", "")).strip() or "-")
        )
        print(
            "proof_operation_family_contract_selftest_last_completed_probe="
            + (str(payload.get("last_completed_probe", "")).strip() or "-")
        )
        print(
            "proof_operation_family_contract_selftest_completed_checks="
            + (str(payload.get("completed_checks", "")).strip() or "-")
        )
        print(
            "proof_operation_family_contract_selftest_total_checks="
            + (str(payload.get("total_checks", "")).strip() or "-")
        )
        print(
            "proof_operation_family_contract_selftest_checks_text="
            + (str(payload.get("checks_text", "")).strip() or "-")
        )
        return
    payload = load_json_snapshot(
        str(step_env.get("DDN_W107_PROGRESS_CONTRACT_SELFTEST_PROGRESS_JSON", "")).strip()
    )
    if not isinstance(payload, dict):
        return
    print(f"w107_progress_contract_selftest_current_probe={str(payload.get('current_probe', '')).strip() or '-'}")
    print(
        "w107_progress_contract_selftest_last_completed_probe="
        + (str(payload.get("last_completed_probe", "")).strip() or "-")
    )
    print(
        "w107_progress_contract_selftest_completed_checks="
        + (str(payload.get("completed_checks", "")).strip() or "-")
    )
    print(
        "w107_progress_contract_selftest_total_checks="
        + (str(payload.get("total_checks", "")).strip() or "-")
    )
    print(
        "w107_progress_contract_selftest_checks_text="
        + (str(payload.get("checks_text", "")).strip() or "-")
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Run CI sanity checks and emit one-line status summary")
    parser.add_argument("--json-out", default="", help="optional path to write sanity result json")
    parser.add_argument(
        "--profile",
        choices=("full", "core_lang", "seamgrim"),
        default="full",
        help="sanity profile selector (default: full)",
    )
    parser.add_argument(
        "--from-step",
        default="",
        help="run only from this step name (inclusive)",
    )
    parser.add_argument(
        "--only-step",
        default="",
        help="run only this exact step name",
    )
    args = parser.parse_args()

    if str(args.json_out).strip():
        json_out_path = Path(args.json_out)
        exec_cache_path = json_out_path.with_name(f"{json_out_path.stem}.selftest_exec_cache.detjson")
    else:
        exec_cache_path = (
            resolve_completion_gate_report_dir()
            / f"ci_sanity_gate.{args.profile}.{os.getpid()}.selftest_exec_cache.detjson"
        )
    os.environ[EXEC_CACHE_ENV_KEY] = str(exec_cache_path)
    reset_exec_cache()

    py = sys.executable
    completion_gate_reports = build_completion_gate_report_paths(args.profile)
    steps = [
        (
            "backup_hygiene_selftest",
            [py, "tests/run_ci_backup_hygiene_selftest.py"],
            "E_CI_SANITY_BACKUP_SELFTEST_FAIL",
        ),
        (
            "pipeline_emit_flags_check",
            [py, "tests/run_ci_pipeline_emit_flags_check.py"],
            "E_CI_SANITY_PIPELINE_FLAGS_FAIL",
        ),
        (
            "pipeline_emit_flags_selftest",
            [py, "tests/run_ci_pipeline_emit_flags_check_selftest.py"],
            "E_CI_SANITY_PIPELINE_FLAGS_SELFTEST_FAIL",
        ),
        (
            "ci_emit_artifacts_sanity_contract_selftest",
            [py, "tests/run_ci_emit_artifacts_sanity_contract_check_selftest.py"],
            "E_CI_SANITY_EMIT_ARTIFACTS_SANITY_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "age5_combined_heavy_policy_selftest",
            [py, "tests/run_age5_combined_heavy_policy_selftest.py"],
            "E_CI_SANITY_AGE5_COMBINED_HEAVY_POLICY_SELFTEST_FAIL",
        ),
        (
            "profile_matrix_full_real_smoke_policy_selftest",
            [py, "tests/run_profile_matrix_full_real_smoke_policy_selftest.py"],
            "E_CI_SANITY_PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SELFTEST_FAIL",
        ),
        (
            "profile_matrix_full_real_smoke_check_selftest",
            [py, "tests/run_ci_profile_matrix_full_real_smoke_check_selftest.py"],
            "E_CI_SANITY_PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SELFTEST_FAIL",
        ),
        (
            "fixed64_darwin_probe_schedule_policy_check",
            [py, "tests/run_fixed64_darwin_probe_schedule_policy_check.py"],
            "E_CI_SANITY_FIXED64_DARWIN_PROBE_SCHEDULE_POLICY_CHECK_FAIL",
        ),
        (
            "fixed64_darwin_real_report_contract_check",
            [
                py,
                "tests/run_fixed64_darwin_real_report_contract_check.py",
                "--report",
                "build/reports/fixed64_cross_platform_probe_darwin.detjson",
                "--inputs-report",
                "build/reports/fixed64_threeway_inputs.contract.detjson",
                "--max-age-minutes",
                str(float(os.environ.get("DDN_FIXED64_THREEWAY_MAX_AGE_MINUTES", "360"))),
                "--resolve-threeway-inputs",
                "--resolve-inputs-json-out",
                "build/reports/fixed64_threeway_inputs.contract.detjson",
                "--resolve-inputs-strict-invalid",
                "--resolve-inputs-require-when-env",
                "DDN_ENABLE_DARWIN_PROBE",
                "--resolve-input-candidate",
                "build/reports/fixed64_darwin_probe_artifact.detjson",
                "--resolve-input-candidate",
                "build/reports/fixed64_darwin_probe_artifact.zip",
                "--resolve-input-candidate",
                "build/reports/fixed64_cross_platform_probe_darwin.detjson",
                "--resolve-input-candidate",
                "build/reports/darwin_probe_archive",
            ],
            "E_CI_SANITY_FIXED64_DARWIN_REAL_REPORT_CONTRACT_CHECK_FAIL",
        ),
        (
            "fixed64_darwin_real_report_live_check",
            [
                py,
                "tests/run_fixed64_darwin_real_report_live_check.py",
                "--allow-pass-contract-only",
                "--json-out",
                completion_gate_reports["fixed64_live_report"],
            ],
            "E_CI_SANITY_FIXED64_DARWIN_REAL_REPORT_LIVE_CHECK_FAIL",
        ),
        (
            "ci_sanity_fixed64_live_summary_fields_selftest",
            [
                py,
                "tests/run_ci_sanity_fixed64_live_summary_fields_selftest.py",
                "--report",
                completion_gate_reports["fixed64_live_report"],
            ],
            "E_CI_SANITY_FIXED64_LIVE_SUMMARY_FIELDS_SELFTEST_FAIL",
        ),
        (
            "fixed64_darwin_real_report_live_check_selftest",
            [py, "tests/run_fixed64_darwin_real_report_live_check_selftest.py"],
            "E_CI_SANITY_FIXED64_DARWIN_REAL_REPORT_LIVE_SELFTEST_FAIL",
        ),
        (
            "fixed64_darwin_real_report_readiness_check_selftest",
            [py, "tests/run_fixed64_darwin_real_report_readiness_check_selftest.py"],
            "E_CI_SANITY_FIXED64_DARWIN_REAL_REPORT_READINESS_SELFTEST_FAIL",
        ),
        (
            "fixed64_threeway_inputs_selftest",
            [py, "tests/run_fixed64_threeway_inputs_selftest.py"],
            "E_CI_SANITY_FIXED64_THREEWAY_INPUTS_SELFTEST_FAIL",
        ),
        (
            "fixed64_cross_platform_threeway_gate_selftest",
            [
                py,
                "tests/run_fixed64_cross_platform_threeway_gate_selftest.py",
                "--out-dir",
                completion_gate_reports["fixed64_threeway_selftest_out_dir"],
            ],
            "E_CI_SANITY_FIXED64_THREEWAY_GATE_SELFTEST_FAIL",
        ),
        (
            "featured_seed_catalog_autogen_check",
            [py, "tests/run_seamgrim_featured_seed_catalog_autogen_check.py"],
            "E_CI_SANITY_FEATURED_SEED_CATALOG_AUTOGEN_CHECK_FAIL",
        ),
        (
            "ci_profile_split_contract_check",
            [py, "tests/run_ci_profile_split_contract_check.py"],
            "E_CI_SANITY_PROFILE_SPLIT_CONTRACT_FAIL",
        ),
        (
            "ci_sanity_dynamic_source_profile_split_selftest",
            [py, "tests/run_ci_sanity_dynamic_source_profile_split_selftest.py"],
            "E_CI_SANITY_DYNAMIC_SOURCE_PROFILE_SPLIT_SELFTEST_FAIL",
        ),
        (
            "ci_profile_matrix_lightweight_contract_selftest",
            [py, "tests/run_ci_profile_matrix_lightweight_contract_selftest.py"],
            "E_CI_SANITY_PROFILE_MATRIX_LIGHTWEIGHT_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "ci_profile_matrix_snapshot_helper_selftest",
            [py, "tests/run_ci_profile_matrix_snapshot_helper_selftest.py"],
            "E_CI_SANITY_PROFILE_MATRIX_SNAPSHOT_HELPER_SELFTEST_FAIL",
        ),
        (
            "ssot_sync_dr173_177_check",
            [py, "tests/run_ssot_sync_dr173_177_check.py"],
            "E_CI_SANITY_SSOT_SYNC_DR173_177_CHECK_FAIL",
        ),
        (
            "age2_completion_gate",
            [
                py,
                "tests/run_age2_completion_gate.py",
                "--report-out",
                completion_gate_reports["age2_gate"],
                "--must-report-out",
                completion_gate_reports["age2_must"],
                "--should-report-out",
                completion_gate_reports["age2_should"],
            ],
            "E_CI_SANITY_AGE2_COMPLETION_GATE_FAIL",
        ),
        (
            "age2_completion_gate_selftest",
            [py, "tests/run_age2_completion_gate_selftest.py"],
            "E_CI_SANITY_AGE2_COMPLETION_GATE_SELFTEST_FAIL",
        ),
        (
            "age2_close_selftest",
            [py, "tests/run_age2_close_selftest.py"],
            "E_CI_SANITY_AGE2_CLOSE_SELFTEST_FAIL",
        ),
        (
            "age2_close",
            [
                py,
                "tests/run_age2_close.py",
                "--age2-report",
                completion_gate_reports["age2_gate"],
                "--must-report",
                completion_gate_reports["age2_must"],
                "--should-report",
                completion_gate_reports["age2_should"],
                "--report-out",
                completion_gate_reports["age2_close"],
                "--skip-selftest",
            ],
            "E_CI_SANITY_AGE2_CLOSE_FAIL",
        ),
        (
            "age3_completion_gate",
            [
                py,
                "tests/run_age3_completion_gate.py",
                "--report-out",
                completion_gate_reports["age3_gate"],
                "--pack-report-out",
                completion_gate_reports["age3_pack"],
            ],
            "E_CI_SANITY_AGE3_COMPLETION_GATE_FAIL",
        ),
        (
            "age3_completion_gate_selftest",
            [py, "tests/run_age3_completion_gate_selftest.py"],
            "E_CI_SANITY_AGE3_COMPLETION_GATE_SELFTEST_FAIL",
        ),
        (
            "age3_close_selftest",
            [py, "tests/run_age3_close_selftest.py"],
            "E_CI_SANITY_AGE3_CLOSE_SELFTEST_FAIL",
        ),
        (
            "age3_close",
            [
                py,
                "tests/run_age3_close.py",
                "--run-seamgrim",
                "--seamgrim-report",
                completion_gate_reports["seamgrim_gate_report"],
                "--ui-age3-report",
                completion_gate_reports["seamgrim_ui_age3_report"],
                "--report-out",
                completion_gate_reports["age3_close"],
            ],
            "E_CI_SANITY_AGE3_CLOSE_FAIL",
        ),
        (
            "gate0_contract_abort_state_check",
            [py, "tests/run_gate0_contract_abort_state_check.py"],
            "E_CI_SANITY_GATE0_CONTRACT_ABORT_STATE_CHECK_FAIL",
        ),
        (
            "contract_tier_unsupported_check",
            [py, "tests/run_contract_tier_unsupported_check.py"],
            "E_CI_SANITY_CONTRACT_TIER_UNSUPPORTED_CHECK_FAIL",
        ),
        (
            "contract_tier_age3_min_enforcement_check",
            [py, "tests/run_contract_tier_age3_min_enforcement_check.py"],
            "E_CI_SANITY_CONTRACT_TIER_AGE3_MIN_ENFORCEMENT_CHECK_FAIL",
        ),
        (
            "map_access_contract_check",
            [py, "tests/run_map_access_contract_check.py"],
            "E_CI_SANITY_MAP_ACCESS_CONTRACT_CHECK_FAIL",
        ),
        (
            "gaji_registry_strict_audit_check",
            [py, "tests/run_gaji_registry_strict_audit_check.py"],
            "E_CI_SANITY_GAJI_REGISTRY_STRICT_AUDIT_CHECK_FAIL",
        ),
        (
            "gaji_registry_defaults_check",
            [py, "tests/run_gaji_registry_defaults_check.py"],
            "E_CI_SANITY_GAJI_REGISTRY_DEFAULTS_CHECK_FAIL",
        ),
        (
            "stdlib_catalog_check",
            [py, "tests/run_stdlib_catalog_check.py"],
            "E_CI_SANITY_STDLIB_CATALOG_CHECK_FAIL",
        ),
        (
            "stdlib_catalog_check_selftest",
            [py, "tests/run_stdlib_catalog_check_selftest.py"],
            "E_CI_SANITY_STDLIB_CATALOG_CHECK_SELFTEST_FAIL",
        ),
        (
            "tensor_v0_pack_check",
            [py, "tests/run_tensor_v0_pack_check.py"],
            "E_CI_SANITY_TENSOR_V0_PACK_CHECK_FAIL",
        ),
        (
            "tensor_v0_cli_check",
            [py, "tests/run_tensor_v0_cli_check.py"],
            "E_CI_SANITY_TENSOR_V0_CLI_CHECK_FAIL",
        ),
        (
            "nurigym_shared_sync_priority_tiebreak_pack_check",
            [py, "tests/run_nurigym_shared_sync_priority_tiebreak_pack_check.py"],
            "E_CI_SANITY_NURIGYM_SHARED_SYNC_PRIORITY_TIEBREAK_PACK_CHECK_FAIL",
        ),
        (
            "nurigym_shared_sync_action_pipeline_pack_check",
            [py, "tests/run_nurigym_shared_sync_action_pipeline_pack_check.py"],
            "E_CI_SANITY_NURIGYM_SHARED_SYNC_ACTION_PIPELINE_PACK_CHECK_FAIL",
        ),
        (
            "nuri_gym_contract_check",
            [py, "tests/run_nuri_gym_contract_check.py"],
            "E_CI_SANITY_NURI_GYM_CONTRACT_CHECK_FAIL",
        ),
        (
            "sam_inputsnapshot_contract_pack_check",
            [py, "tests/run_sam_inputsnapshot_contract_pack_check.py"],
            "E_CI_SANITY_SAM_INPUTSNAPSHOT_CONTRACT_PACK_CHECK_FAIL",
        ),
        (
            "sam_ai_ordering_pack_check",
            [py, "tests/run_sam_ai_ordering_pack_check.py"],
            "E_CI_SANITY_SAM_AI_ORDERING_PACK_CHECK_FAIL",
        ),
        (
            "seulgi_gatekeeper_pack_check",
            [py, "tests/run_seulgi_gatekeeper_pack_check.py"],
            "E_CI_SANITY_SEULGI_GATEKEEPER_PACK_CHECK_FAIL",
        ),
        (
            "sam_seulgi_family_contract_selftest",
            [py, "tests/run_sam_seulgi_family_contract_selftest.py"],
            "E_CI_SANITY_SAM_SEULGI_FAMILY_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "external_intent_seulgi_walk_alignment_check_selftest",
            [py, "tests/run_external_intent_seulgi_walk_alignment_check_selftest.py"],
            "E_CI_SANITY_EXTERNAL_INTENT_SEULGI_WALK_ALIGNMENT_CHECK_SELFTEST_FAIL",
        ),
        (
            "seamgrim_ci_gate_seed_meta_step_check",
            [py, "tests/run_seamgrim_ci_gate_seed_meta_step_check.py"],
            "E_CI_SANITY_SEED_META_STEP_FAIL",
        ),
        (
            "seamgrim_ci_gate_featured_seed_catalog_step_check",
            [py, "tests/run_seamgrim_ci_gate_featured_seed_catalog_step_check.py"],
            "E_CI_SANITY_FEATURED_SEED_CATALOG_STEP_FAIL",
        ),
        (
            "seamgrim_ci_gate_featured_seed_catalog_autogen_step_check",
            [py, "tests/run_seamgrim_ci_gate_featured_seed_catalog_autogen_step_check.py"],
            "E_CI_SANITY_FEATURED_SEED_CATALOG_AUTOGEN_STEP_FAIL",
        ),
        (
            "seamgrim_ci_gate_runtime5_passthrough_check",
            [py, "tests/run_seamgrim_ci_gate_runtime5_passthrough_check.py"],
            "E_CI_SANITY_RUNTIME5_PASSTHROUGH_FAIL",
        ),
        (
            "seamgrim_ci_gate_lesson_warning_step_check",
            [py, "tests/run_seamgrim_ci_gate_lesson_warning_step_check.py"],
            "E_CI_SANITY_LESSON_WARNING_STEP_FAIL",
        ),
        (
            "seamgrim_ci_gate_stateful_preview_step_check",
            [py, "tests/run_seamgrim_ci_gate_stateful_preview_step_check.py"],
            "E_CI_SANITY_STATEFUL_PREVIEW_STEP_FAIL",
        ),
        (
            "seamgrim_ci_gate_wasm_web_smoke_step_check",
            [
                py,
                "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check.py",
                "--report-out",
                completion_gate_reports["seamgrim_wasm_web_step_check_report"],
            ],
            "E_CI_SANITY_WASM_WEB_SMOKE_STEP_FAIL",
        ),
        (
            "seamgrim_ci_gate_wasm_web_smoke_step_check_selftest",
            [
                py,
                "tests/run_seamgrim_ci_gate_wasm_web_smoke_step_check_selftest.py",
                "--verify-report",
                completion_gate_reports["seamgrim_wasm_web_step_check_report"],
            ],
            "E_CI_SANITY_WASM_WEB_SMOKE_STEP_SELFTEST_FAIL",
        ),
        (
            "seamgrim_ci_gate_pack_evidence_tier_step_check",
            [py, "tests/run_seamgrim_ci_gate_pack_evidence_tier_step_check.py"],
            "E_CI_SANITY_PACK_EVIDENCE_TIER_STEP_FAIL",
        ),
        (
            "seamgrim_ci_gate_pack_evidence_tier_step_check_selftest",
            [py, "tests/run_seamgrim_ci_gate_pack_evidence_tier_step_check_selftest.py"],
            "E_CI_SANITY_PACK_EVIDENCE_TIER_STEP_SELFTEST_FAIL",
        ),
        (
            "seamgrim_ci_gate_pack_evidence_tier_runner_check",
            [
                py,
                "tests/run_pack_evidence_tier_check.py",
                "--report-out",
                completion_gate_reports["seamgrim_pack_evidence_tier_runner_report"],
            ],
            "E_CI_SANITY_PACK_EVIDENCE_TIER_RUNNER_CHECK_FAIL",
        ),
        (
            "seamgrim_ci_gate_pack_evidence_tier_report_check",
            [
                py,
                "tests/run_pack_evidence_tier_report_check.py",
                "--report-path",
                completion_gate_reports["seamgrim_pack_evidence_tier_runner_report"],
            ],
            "E_CI_SANITY_PACK_EVIDENCE_TIER_REPORT_CHECK_FAIL",
        ),
        (
            "seamgrim_ci_gate_pack_evidence_tier_report_check_selftest",
            [
                py,
                "tests/run_pack_evidence_tier_report_check_selftest.py",
                "--verify-report",
                completion_gate_reports["seamgrim_pack_evidence_tier_runner_report"],
            ],
            "E_CI_SANITY_PACK_EVIDENCE_TIER_REPORT_CHECK_SELFTEST_FAIL",
        ),
        (
            "seamgrim_interface_boundary_contract_check",
            [py, "tests/run_seamgrim_interface_boundary_contract_check.py"],
            "E_CI_SANITY_SEAMGRIM_INTERFACE_BOUNDARY_FAIL",
        ),
        (
            "seamgrim_overlay_session_wired_consistency_check",
            [py, "tests/run_seamgrim_overlay_session_wired_consistency_check.py"],
            "E_CI_SANITY_OVERLAY_SESSION_WIRED_CONSISTENCY_FAIL",
        ),
        (
            "seamgrim_overlay_session_diag_parity_check",
            [py, "tests/run_seamgrim_overlay_session_diag_parity_check.py"],
            "E_CI_SANITY_OVERLAY_SESSION_DIAG_PARITY_FAIL",
        ),
        (
            "seamgrim_overlay_compare_diag_parity_check",
            [py, "tests/run_seamgrim_overlay_compare_diag_parity_check.py"],
            "E_CI_SANITY_OVERLAY_COMPARE_DIAG_PARITY_FAIL",
        ),
        (
            "age5_close_pack_contract_selftest",
            [py, "tests/run_age5_close_pack_contract_selftest.py"],
            "E_CI_SANITY_AGE5_CLOSE_PACK_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "age5_close_combined_report_contract_selftest",
            [py, "tests/run_age5_close_combined_report_contract_selftest.py"],
            "E_CI_SANITY_AGE5_CLOSE_COMBINED_REPORT_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "ci_gate_summary_line_check_selftest",
            [py, "tests/run_ci_gate_summary_line_check_selftest.py"],
            "E_CI_SANITY_GATE_SUMMARY_LINE_CHECK_SELFTEST_FAIL",
        ),
        (
            "w107_transport_contract_summary_selftest",
            [py, "tests/run_w107_transport_contract_summary_selftest.py"],
            "E_CI_SANITY_W107_TRANSPORT_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "age4_proof_transport_contract_selftest",
            [py, "tests/run_age4_proof_transport_contract_selftest.py"],
            "E_CI_SANITY_AGE4_PROOF_TRANSPORT_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "age4_proof_transport_contract_report_selftest",
            [py, "tests/run_age4_proof_transport_contract_report_selftest.py"],
            "E_CI_SANITY_AGE4_PROOF_TRANSPORT_CONTRACT_REPORT_SELFTEST_FAIL",
        ),
        (
            "age4_proof_transport_contract_summary_selftest",
            [py, "tests/run_age4_proof_transport_contract_summary_selftest.py"],
            "E_CI_SANITY_AGE4_PROOF_TRANSPORT_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "age4_proof_quantifier_case_analysis_selftest",
            [py, "tests/run_age4_proof_quantifier_case_analysis_selftest.py"],
            "E_CI_SANITY_AGE4_PROOF_QUANTIFIER_CASE_ANALYSIS_SELFTEST_FAIL",
        ),
        (
            "proof_case_analysis_completion_parity_selftest",
            [py, "tests/run_proof_case_analysis_completion_parity_selftest.py"],
            "E_CI_SANITY_PROOF_CASE_ANALYSIS_COMPLETION_PARITY_SELFTEST_FAIL",
        ),
        (
            "age1_immediate_proof_operation_matrix_selftest",
            [py, "tests/run_age1_immediate_proof_operation_matrix_selftest.py"],
            "E_CI_SANITY_AGE1_IMMEDIATE_PROOF_OPERATION_MATRIX_SELFTEST_FAIL",
        ),
        (
            "age1_immediate_proof_operation_contract_selftest",
            [py, "tests/run_age1_immediate_proof_operation_contract_selftest.py"],
            "E_CI_SANITY_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "age1_immediate_proof_operation_contract_summary_selftest",
            [py, "tests/run_age1_immediate_proof_operation_contract_summary_selftest.py"],
            "E_CI_SANITY_AGE1_IMMEDIATE_PROOF_OPERATION_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "age1_immediate_proof_operation_transport_contract_summary_selftest",
            [py, "tests/run_age1_immediate_proof_operation_transport_contract_summary_selftest.py"],
            "E_CI_SANITY_AGE1_IMMEDIATE_PROOF_OPERATION_TRANSPORT_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "age1_immediate_proof_solver_search_matrix_selftest",
            [py, "tests/run_age1_immediate_proof_solver_search_matrix_selftest.py"],
            "E_CI_SANITY_AGE1_IMMEDIATE_PROOF_SOLVER_SEARCH_MATRIX_SELFTEST_FAIL",
        ),
        (
            "proof_solver_search_operation_parity_selftest",
            [py, "tests/run_proof_solver_search_operation_parity_selftest.py"],
            "E_CI_SANITY_PROOF_SOLVER_SEARCH_OPERATION_PARITY_SELFTEST_FAIL",
        ),
        (
            "proof_solver_operation_family_selftest",
            [py, "tests/run_proof_solver_operation_family_selftest.py"],
            "E_CI_SANITY_PROOF_SOLVER_OPERATION_FAMILY_SELFTEST_FAIL",
        ),
        (
            "proof_operation_family_selftest",
            [py, "tests/run_proof_operation_family_selftest.py"],
            "E_CI_SANITY_PROOF_OPERATION_FAMILY_SELFTEST_FAIL",
        ),
        (
            "proof_operation_family_contract_selftest",
            [py, "tests/run_proof_operation_family_contract_selftest.py"],
            "E_CI_SANITY_PROOF_OPERATION_FAMILY_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "ci_pack_golden_age5_surface_selftest",
            [py, "tests/run_pack_golden_age5_surface_selftest.py"],
            "E_CI_SANITY_PACK_GOLDEN_AGE5_SURFACE_SELFTEST_FAIL",
        ),
        (
            "ci_pack_golden_guideblock_selftest",
            [py, "tests/run_pack_golden_guideblock_selftest.py"],
            "E_CI_SANITY_PACK_GOLDEN_GUIDEBLOCK_SELFTEST_FAIL",
        ),
        (
            "ci_pack_golden_exec_policy_selftest",
            [py, "tests/run_pack_golden_exec_policy_selftest.py"],
            "E_CI_SANITY_PACK_GOLDEN_EXEC_POLICY_SELFTEST_FAIL",
        ),
        (
            "ci_pack_golden_jjaim_flatten_selftest",
            [py, "tests/run_pack_golden_jjaim_flatten_selftest.py"],
            "E_CI_SANITY_PACK_GOLDEN_JJAIM_FLATTEN_SELFTEST_FAIL",
        ),
        (
            "ci_pack_golden_event_model_selftest",
            [py, "tests/run_pack_golden_event_model_selftest.py"],
            "E_CI_SANITY_PACK_GOLDEN_EVENT_MODEL_SELFTEST_FAIL",
        ),
        (
            "ci_pack_golden_lang_consistency_selftest",
            [py, "tests/run_pack_golden_lang_consistency_selftest.py"],
            "E_CI_SANITY_PACK_GOLDEN_LANG_CONSISTENCY_SELFTEST_FAIL",
        ),
        (
            "alrim_dispatch_runtime_contract_selftest",
            [py, "tests/run_alrim_dispatch_runtime_contract_selftest.py"],
            "E_CI_SANITY_ALRIM_DISPATCH_RUNTIME_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "w49_golden_index_selfcheck",
            [py, "tools/teul-cli/tests/run_w49_golden_index_selfcheck.py"],
            "E_CI_SANITY_W49_GOLDEN_INDEX_SELFCHECK_FAIL",
        ),
        (
            "w111_golden_index_selfcheck",
            [py, "tools/teul-cli/tests/run_w111_golden_index_selfcheck.py"],
            "E_CI_SANITY_W111_GOLDEN_INDEX_SELFCHECK_FAIL",
        ),
        (
            "w109_golden_index_selfcheck",
            [py, "tools/teul-cli/tests/run_w109_golden_index_selfcheck.py"],
            "E_CI_SANITY_W109_GOLDEN_INDEX_SELFCHECK_FAIL",
        ),
        (
            "w107_golden_index_selfcheck",
            [py, "tools/teul-cli/tests/run_w107_golden_index_selfcheck.py"],
            "E_CI_SANITY_W107_GOLDEN_INDEX_SELFCHECK_FAIL",
        ),
        (
            "w107_golden_index_selftest",
            [py, "tests/run_w107_golden_index_selftest.py"],
            "E_CI_SANITY_W107_GOLDEN_INDEX_SELFTEST_FAIL",
        ),
        (
            "w107_progress_contract_selftest",
            [py, "tests/run_w107_progress_contract_selftest.py"],
            "E_CI_SANITY_W107_PROGRESS_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "proof_artifact_certificate_contract_selftest",
            [py, "tests/run_proof_artifact_certificate_contract_selftest.py"],
            "E_CI_SANITY_PROOF_ARTIFACT_CERTIFICATE_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_digest_axes_selftest",
            [py, "tests/run_proof_certificate_digest_axes_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_DIGEST_AXES_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_candidate_manifest_selftest",
            [py, "tests/run_proof_certificate_candidate_manifest_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_CANDIDATE_MANIFEST_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_candidate_profile_split_selftest",
            [py, "tests/run_proof_certificate_candidate_profile_split_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_CANDIDATE_PROFILE_SPLIT_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_candidate_layers_selftest",
            [py, "tests/run_proof_certificate_candidate_layers_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_CANDIDATE_LAYERS_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_promotion_candidate_selftest",
            [py, "tests/run_proof_certificate_v1_promotion_candidate_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_PROMOTION_CANDIDATE_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_draft_pack_selftest",
            [py, "tests/run_proof_certificate_v1_draft_pack_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_DRAFT_PACK_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_draft_artifact_selftest",
            [py, "tests/run_proof_certificate_v1_draft_artifact_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_DRAFT_ARTIFACT_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_draft_artifact_layers_selftest",
            [py, "tests/run_proof_certificate_v1_draft_artifact_layers_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_DRAFT_ARTIFACT_LAYERS_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_draft_contract_selftest",
            [py, "tests/run_proof_certificate_v1_draft_contract_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_DRAFT_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_schema_candidate_selftest",
            [py, "tests/run_proof_certificate_v1_schema_candidate_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_SCHEMA_CANDIDATE_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_schema_candidate_split_selftest",
            [py, "tests/run_proof_certificate_v1_schema_candidate_split_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_SCHEMA_CANDIDATE_SPLIT_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_promotion_selftest",
            [py, "tests/run_proof_certificate_v1_promotion_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_PROMOTION_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_family_contract_selftest",
            [py, "tests/run_proof_certificate_v1_family_contract_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_family_contract_summary_selftest",
            [py, "tests/run_proof_certificate_v1_family_contract_summary_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_FAMILY_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_family_transport_contract_selftest",
            [py, "tests/run_proof_certificate_v1_family_transport_contract_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_FAMILY_TRANSPORT_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_family_transport_contract_summary_selftest",
            [py, "tests/run_proof_certificate_v1_family_transport_contract_summary_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_FAMILY_TRANSPORT_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_family_selftest",
            [py, "tests/run_proof_certificate_v1_family_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_FAMILY_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_family_selftest",
            [py, "tests/run_proof_certificate_family_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_FAMILY_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_family_contract_selftest",
            [py, "tests/run_proof_certificate_family_contract_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_FAMILY_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_family_contract_summary_selftest",
            [py, "tests/run_proof_certificate_family_contract_summary_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_FAMILY_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_family_transport_contract_selftest",
            [py, "tests/run_proof_certificate_family_transport_contract_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_family_transport_contract_summary_selftest",
            [py, "tests/run_proof_certificate_family_transport_contract_summary_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_FAMILY_TRANSPORT_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "proof_family_selftest",
            [py, "tests/run_proof_family_selftest.py"],
            "E_CI_SANITY_PROOF_FAMILY_SELFTEST_FAIL",
        ),
        (
            "proof_family_contract_selftest",
            [py, "tests/run_proof_family_contract_selftest.py"],
            "E_CI_SANITY_PROOF_FAMILY_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "proof_family_contract_summary_selftest",
            [py, "tests/run_proof_family_contract_summary_selftest.py"],
            "E_CI_SANITY_PROOF_FAMILY_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "proof_family_transport_contract_selftest",
            [py, "tests/run_proof_family_transport_contract_selftest.py"],
            "E_CI_SANITY_PROOF_FAMILY_TRANSPORT_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "proof_family_transport_contract_summary_selftest",
            [py, "tests/run_proof_family_transport_contract_summary_selftest.py"],
            "E_CI_SANITY_PROOF_FAMILY_TRANSPORT_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "lang_surface_family_selftest",
            [py, "tests/run_lang_surface_family_selftest.py"],
            "E_CI_SANITY_LANG_SURFACE_FAMILY_SELFTEST_FAIL",
        ),
        (
            "lang_surface_family_contract_selftest",
            [py, "tests/run_lang_surface_family_contract_selftest.py"],
            "E_CI_SANITY_LANG_SURFACE_FAMILY_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "lang_surface_family_contract_summary_selftest",
            [py, "tests/run_lang_surface_family_contract_summary_selftest.py"],
            "E_CI_SANITY_LANG_SURFACE_FAMILY_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "lang_surface_family_transport_contract_selftest",
            [py, "tests/run_lang_surface_family_transport_contract_selftest.py"],
            "E_CI_SANITY_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "lang_surface_family_transport_contract_summary_selftest",
            [py, "tests/run_lang_surface_family_transport_contract_summary_selftest.py"],
            "E_CI_SANITY_LANG_SURFACE_FAMILY_TRANSPORT_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "lang_runtime_family_selftest",
            [py, "tests/run_lang_runtime_family_selftest.py"],
            "E_CI_SANITY_LANG_RUNTIME_FAMILY_SELFTEST_FAIL",
        ),
        (
            "lang_runtime_family_contract_selftest",
            [py, "tests/run_lang_runtime_family_contract_selftest.py"],
            "E_CI_SANITY_LANG_RUNTIME_FAMILY_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "lang_runtime_family_contract_summary_selftest",
            [py, "tests/run_lang_runtime_family_contract_summary_selftest.py"],
            "E_CI_SANITY_LANG_RUNTIME_FAMILY_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "lang_runtime_family_transport_contract_selftest",
            [py, "tests/run_lang_runtime_family_transport_contract_selftest.py"],
            "E_CI_SANITY_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "lang_runtime_family_transport_contract_summary_selftest",
            [py, "tests/run_lang_runtime_family_transport_contract_summary_selftest.py"],
            "E_CI_SANITY_LANG_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "gate0_runtime_family_selftest",
            [py, "tests/run_gate0_runtime_family_selftest.py"],
            "E_CI_SANITY_GATE0_RUNTIME_FAMILY_SELFTEST_FAIL",
        ),
        (
            "gate0_runtime_family_contract_selftest",
            [py, "tests/run_gate0_runtime_family_contract_selftest.py"],
            "E_CI_SANITY_GATE0_RUNTIME_FAMILY_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "gate0_runtime_family_contract_summary_selftest",
            [py, "tests/run_gate0_runtime_family_contract_summary_selftest.py"],
            "E_CI_SANITY_GATE0_RUNTIME_FAMILY_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "gate0_runtime_family_transport_contract_selftest",
            [py, "tests/run_gate0_runtime_family_transport_contract_selftest.py"],
            "E_CI_SANITY_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "gate0_runtime_family_transport_contract_summary_selftest",
            [py, "tests/run_gate0_runtime_family_transport_contract_summary_selftest.py"],
            "E_CI_SANITY_GATE0_RUNTIME_FAMILY_TRANSPORT_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "gate0_family_selftest",
            [py, "tests/run_gate0_family_selftest.py"],
            "E_CI_SANITY_GATE0_FAMILY_SELFTEST_FAIL",
        ),
        (
            "gate0_family_contract_selftest",
            [py, "tests/run_gate0_family_contract_selftest.py"],
            "E_CI_SANITY_GATE0_FAMILY_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "gate0_family_transport_contract_selftest",
            [py, "tests/run_gate0_family_transport_contract_selftest.py"],
            "E_CI_SANITY_GATE0_FAMILY_TRANSPORT_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "gate0_family_contract_summary_selftest",
            [py, "tests/run_gate0_family_contract_summary_selftest.py"],
            "E_CI_SANITY_GATE0_FAMILY_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "gate0_family_transport_contract_summary_selftest",
            [py, "tests/run_gate0_family_transport_contract_summary_selftest.py"],
            "E_CI_SANITY_GATE0_FAMILY_TRANSPORT_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "gate0_transport_family_selftest",
            [py, "tests/run_gate0_transport_family_selftest.py"],
            "E_CI_SANITY_GATE0_TRANSPORT_FAMILY_SELFTEST_FAIL",
        ),
        (
            "gate0_transport_family_contract_selftest",
            [py, "tests/run_gate0_transport_family_contract_selftest.py"],
            "E_CI_SANITY_GATE0_TRANSPORT_FAMILY_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "gate0_transport_family_contract_summary_selftest",
            [py, "tests/run_gate0_transport_family_contract_summary_selftest.py"],
            "E_CI_SANITY_GATE0_TRANSPORT_FAMILY_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "gate0_transport_family_transport_contract_selftest",
            [py, "tests/run_gate0_transport_family_transport_contract_selftest.py"],
            "E_CI_SANITY_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "gate0_transport_family_transport_contract_summary_selftest",
            [py, "tests/run_gate0_transport_family_transport_contract_summary_selftest.py"],
            "E_CI_SANITY_GATE0_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "gate0_surface_family_selftest",
            [py, "tests/run_gate0_surface_family_selftest.py"],
            "E_CI_SANITY_GATE0_SURFACE_FAMILY_SELFTEST_FAIL",
        ),
        (
            "gate0_surface_family_contract_selftest",
            [py, "tests/run_gate0_surface_family_contract_selftest.py"],
            "E_CI_SANITY_GATE0_SURFACE_FAMILY_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "gate0_surface_family_contract_summary_selftest",
            [py, "tests/run_gate0_surface_family_contract_summary_selftest.py"],
            "E_CI_SANITY_GATE0_SURFACE_FAMILY_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "gate0_surface_family_transport_contract_selftest",
            [py, "tests/run_gate0_surface_family_transport_contract_selftest.py"],
            "E_CI_SANITY_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "gate0_surface_family_transport_contract_summary_selftest",
            [py, "tests/run_gate0_surface_family_transport_contract_summary_selftest.py"],
            "E_CI_SANITY_GATE0_SURFACE_FAMILY_TRANSPORT_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "gate0_surface_transport_family_selftest",
            [py, "tests/run_gate0_surface_transport_family_selftest.py"],
            "E_CI_SANITY_GATE0_SURFACE_TRANSPORT_FAMILY_SELFTEST_FAIL",
        ),
        (
            "gate0_surface_transport_family_contract_selftest",
            [py, "tests/run_gate0_surface_transport_family_contract_selftest.py"],
            "E_CI_SANITY_GATE0_SURFACE_TRANSPORT_FAMILY_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "gate0_surface_transport_family_contract_summary_selftest",
            [py, "tests/run_gate0_surface_transport_family_contract_summary_selftest.py"],
            "E_CI_SANITY_GATE0_SURFACE_TRANSPORT_FAMILY_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "gate0_surface_transport_family_transport_contract_selftest",
            [py, "tests/run_gate0_surface_transport_family_transport_contract_selftest.py"],
            "E_CI_SANITY_GATE0_SURFACE_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "gate0_surface_transport_family_transport_contract_summary_selftest",
            [py, "tests/run_gate0_surface_transport_family_transport_contract_summary_selftest.py"],
            "E_CI_SANITY_GATE0_SURFACE_TRANSPORT_FAMILY_TRANSPORT_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "compound_update_reject_contract_selftest",
            [py, "tests/run_compound_update_reject_contract_selftest.py"],
            "E_CI_SANITY_COMPOUND_UPDATE_REJECT_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "bogae_shape_alias_contract_selftest",
            [py, "tests/run_bogae_shape_alias_contract_selftest.py"],
            "E_CI_SANITY_BOGAE_SHAPE_ALIAS_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "bogae_alias_family_selftest",
            [py, "tests/run_bogae_alias_family_selftest.py"],
            "E_CI_SANITY_BOGAE_ALIAS_FAMILY_SELFTEST_FAIL",
        ),
        (
            "bogae_alias_family_contract_selftest",
            [py, "tests/run_bogae_alias_family_contract_selftest.py"],
            "E_CI_SANITY_BOGAE_ALIAS_FAMILY_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "bogae_alias_family_contract_summary_selftest",
            [py, "tests/run_bogae_alias_family_contract_summary_selftest.py"],
            "E_CI_SANITY_BOGAE_ALIAS_FAMILY_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "bogae_alias_family_transport_contract_selftest",
            [py, "tests/run_bogae_alias_family_transport_contract_selftest.py"],
            "E_CI_SANITY_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "bogae_alias_family_transport_contract_summary_selftest",
            [py, "tests/run_bogae_alias_family_transport_contract_summary_selftest.py"],
            "E_CI_SANITY_BOGAE_ALIAS_FAMILY_TRANSPORT_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "bogae_alias_viewer_family_selftest",
            [py, "tests/run_bogae_alias_viewer_family_selftest.py"],
            "E_CI_SANITY_BOGAE_ALIAS_VIEWER_FAMILY_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_runtime_emit_selftest",
            [py, "tests/run_proof_certificate_v1_runtime_emit_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_RUNTIME_EMIT_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_signed_emit_selftest",
            [py, "tests/run_proof_certificate_v1_signed_emit_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_SIGNED_EMIT_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_signed_emit_profile_selftest",
            [py, "tests/run_proof_certificate_v1_signed_emit_profile_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_SIGNED_EMIT_PROFILE_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_verify_bundle_selftest",
            [py, "tests/run_proof_certificate_v1_verify_bundle_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_VERIFY_BUNDLE_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_verify_report_selftest",
            [py, "tests/run_proof_certificate_v1_verify_report_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_VERIFY_REPORT_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_verify_report_digest_contract_selftest",
            [py, "tests/run_proof_certificate_v1_verify_report_digest_contract_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_verify_report_digest_transport_contract_summary_selftest",
            [py, "tests/run_proof_certificate_v1_verify_report_digest_transport_contract_summary_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_VERIFY_REPORT_DIGEST_TRANSPORT_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_consumer_contract_selftest",
            [py, "tests/run_proof_certificate_v1_consumer_contract_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_CONSUMER_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_consumer_transport_contract_selftest",
            [py, "tests/run_proof_certificate_v1_consumer_transport_contract_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_consumer_transport_contract_summary_selftest",
            [py, "tests/run_proof_certificate_v1_consumer_transport_contract_summary_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_CONSUMER_TRANSPORT_CONTRACT_SUMMARY_SELFTEST_FAIL",
        ),
        (
            "proof_certificate_v1_signed_contract_selftest",
            [py, "tests/run_proof_certificate_v1_signed_contract_selftest.py"],
            "E_CI_SANITY_PROOF_CERTIFICATE_V1_SIGNED_CONTRACT_SELFTEST_FAIL",
        ),
        (
            "proof_artifact_cert_subject_pack_check",
            [py, "tests/run_w95_cert_pack_check.py", "--pack", "pack/age4_proof_artifact_cert_subject_v1"],
            "E_CI_SANITY_PROOF_ARTIFACT_CERT_SUBJECT_PACK_CHECK_FAIL",
        ),
        (
            "ci_pack_golden_metadata_selftest",
            [py, "tests/run_pack_golden_metadata_selftest.py"],
            "E_CI_SANITY_PACK_GOLDEN_METADATA_SELFTEST_FAIL",
        ),
        (
            "ci_pack_golden_graph_export_selftest",
            [py, "tests/run_pack_golden_graph_export_selftest.py"],
            "E_CI_SANITY_PACK_GOLDEN_GRAPH_EXPORT_SELFTEST_FAIL",
        ),
        (
            "ci_canon_ast_dpack_selftest",
            [py, "tests/run_canon_ast_dpack_selftest.py"],
            "E_CI_SANITY_CANON_AST_DPACK_SELFTEST_FAIL",
        ),
        (
            "w92_aot_pack_check",
            [py, "tests/run_w92_aot_pack_check.py"],
            "E_CI_SANITY_W92_AOT_PACK_CHECK_FAIL",
        ),
        (
            "w93_universe_pack_check",
            [py, "tests/run_w93_universe_pack_check.py"],
            "E_CI_SANITY_W93_UNIVERSE_PACK_CHECK_FAIL",
        ),
        (
            "w94_social_pack_check",
            [py, "tests/run_w94_social_pack_check.py"],
            "E_CI_SANITY_W94_SOCIAL_PACK_CHECK_FAIL",
        ),
        (
            "w95_cert_pack_check",
            [py, "tests/run_w95_cert_pack_check.py"],
            "E_CI_SANITY_W95_CERT_PACK_CHECK_FAIL",
        ),
        (
            "w96_somssi_pack_check",
            [py, "tests/run_w96_somssi_pack_check.py"],
            "E_CI_SANITY_W96_SOMSSI_PACK_CHECK_FAIL",
        ),
        (
            "w97_self_heal_pack_check",
            [py, "tests/run_w97_self_heal_pack_check.py"],
            "E_CI_SANITY_W97_SELF_HEAL_PACK_CHECK_FAIL",
        ),
        (
            "seamgrim_wasm_cli_diag_parity_check",
            [
                py,
                "tests/run_seamgrim_wasm_cli_diag_parity_check.py",
                "--json-out",
                str(completion_gate_reports.get("seamgrim_wasm_cli_diag_parity_report", "")),
            ],
            "E_CI_SANITY_WASM_CLI_DIAG_PARITY_FAIL",
        ),
    ]
    if args.profile == "core_lang":
        steps = [row for row in steps if row[0] in CORE_LANG_PROFILE_STEPS]
    elif args.profile == "seamgrim":
        steps = [row for row in steps if row[0] in SEAMGRIM_PROFILE_STEPS]
    if str(args.only_step).strip() and str(args.from_step).strip():
        print("--only-step and --from-step cannot be used together", file=sys.stderr)
        return 2
    if str(args.only_step).strip():
        target = str(args.only_step).strip()
        steps = [row for row in steps if row[0] == target]
        if not steps:
            print(f"unknown step for profile={args.profile}: {target}", file=sys.stderr)
            return 2
    elif str(args.from_step).strip():
        target = str(args.from_step).strip()
        names = [row[0] for row in steps]
        if target not in names:
            print(f"unknown step for profile={args.profile}: {target}", file=sys.stderr)
            return 2
        steps = steps[names.index(target) :]
    dynamic_hint_lookback = resolve_dynamic_hint_lookback(args.profile)
    dynamic_worker_lookback = resolve_dynamic_worker_lookback(args.profile)

    rows: list[dict[str, object]] = []
    started = time.perf_counter()
    last_completed_step = "-"
    for step_name, cmd, default_code in steps:
        print(f"ci_sanity_current_step={step_name}", flush=True)
        write_json_snapshot(
            args.json_out,
            {
                "schema": "ddn.ci.sanity_gate.v1",
                "generated_at_utc": datetime.now(timezone.utc).isoformat(),
                "status": "running",
                "code": "RUNNING",
                "step": step_name,
                "msg": "-",
                "profile": args.profile,
                "current_step": step_name,
                "last_completed_step": last_completed_step,
                **build_sanity_summary_fields(args.profile, rows, completion_gate_reports),
                "steps": rows,
                "steps_count": len(rows),
                "total_elapsed_ms": int(round((time.perf_counter() - started) * 1000.0)),
            },
        )
        step_env = build_step_progress_env(step_name, args.json_out)
        if step_name == "age2_completion_gate_selftest":
            step_env = dict(step_env or {})
            step_env.update(
                {
                    "DDN_AGE2_COMPLETION_GATE_SELFTEST_REUSE_REPORT": "1",
                    "DDN_AGE2_COMPLETION_GATE_REPORT_JSON": str(completion_gate_reports.get("age2_gate", "")),
                    "DDN_AGE2_COMPLETION_GATE_MUST_REPORT_JSON": str(completion_gate_reports.get("age2_must", "")),
                    "DDN_AGE2_COMPLETION_GATE_SHOULD_REPORT_JSON": str(completion_gate_reports.get("age2_should", "")),
                }
            )
        elif step_name == "age2_completion_gate":
            step_env = dict(step_env or {})
            step_env.update(
                {
                    "DDN_AGE2_DYNAMIC_SHARD_MODE": "previous",
                    "DDN_AGE2_DYNAMIC_SHARD_SOURCE_REPORT": str(completion_gate_reports.get("age2_dynamic_source", "")),
                    "DDN_AGE2_DYNAMIC_SHARD_LOOKBACK": str(dynamic_hint_lookback),
                    "DDN_AGE2_DYNAMIC_HINT_LOOKBACK": str(dynamic_hint_lookback),
                }
            )
        elif step_name == "age3_completion_gate":
            step_env = dict(step_env or {})
            step_env.update(
                {
                    "DDN_AGE3_DYNAMIC_SHARD_MODE": "previous",
                    "DDN_AGE3_DYNAMIC_SHARD_SOURCE_REPORT": str(completion_gate_reports.get("age3_dynamic_source", "")),
                    "DDN_AGE3_DYNAMIC_SHARD_LOOKBACK": str(dynamic_hint_lookback),
                    "DDN_AGE3_DYNAMIC_HINT_LOOKBACK": str(dynamic_hint_lookback),
                }
            )
        elif step_name == "age3_completion_gate_selftest":
            step_env = dict(step_env or {})
            step_env.update(
                {
                    "DDN_AGE3_COMPLETION_GATE_SELFTEST_REUSE_REPORT": "1",
                    "DDN_AGE3_COMPLETION_GATE_REPORT_JSON": str(completion_gate_reports.get("age3_gate", "")),
                    "DDN_AGE3_COMPLETION_GATE_PACK_REPORT_JSON": str(completion_gate_reports.get("age3_pack", "")),
                }
            )
        elif step_name == "age4_proof_transport_contract_selftest":
            step_env = dict(step_env or {})
            age4_report = str(completion_gate_reports.get("age4_transport_report", ""))
            step_env.update(
                {
                    "DDN_AGE4_PROOF_TRANSPORT_CONTRACT_SELFTEST_REPORT_JSON": age4_report,
                    "DDN_AGE4_PROOF_TRANSPORT_DYNAMIC_WORKER_MODE": "previous",
                    "DDN_AGE4_PROOF_TRANSPORT_DYNAMIC_WORKER_SOURCE_REPORT": str(
                        completion_gate_reports.get("age4_dynamic_source", "")
                    ),
                    "DDN_AGE4_PROOF_TRANSPORT_DYNAMIC_WORKER_LOOKBACK": str(dynamic_worker_lookback),
                }
            )
        elif step_name == "age4_proof_transport_contract_report_selftest":
            step_env = dict(step_env or {})
            age4_report = str(completion_gate_reports.get("age4_transport_report", ""))
            step_env.update(
                {
                    "DDN_AGE4_PROOF_TRANSPORT_CONTRACT_SELFTEST_REUSE_REPORT": "1",
                    "DDN_AGE4_PROOF_TRANSPORT_CONTRACT_SELFTEST_REPORT_JSON": age4_report,
                }
            )
        elif step_name == "ci_pack_golden_age5_surface_selftest":
            step_env = dict(step_env or {})
            age5_report = str(completion_gate_reports.get("age5_surface_report", ""))
            step_env.update(
                {
                    "DDN_AGE5_SURFACE_SELFTEST_REPORT_JSON": age5_report,
                    "DDN_AGE5_SURFACE_DYNAMIC_GROUP_MODE": "previous",
                    "DDN_AGE5_SURFACE_DYNAMIC_GROUP_SOURCE_REPORT": str(
                        completion_gate_reports.get("age5_dynamic_source", "")
                    ),
                    "DDN_AGE5_SURFACE_DYNAMIC_GROUP_LOOKBACK": str(dynamic_hint_lookback),
                }
            )
        write_step_seed_progress(step_name, step_env)
        step_started = time.perf_counter()
        proc = run_step(cmd, step_env)
        elapsed_ms = int(round((time.perf_counter() - step_started) * 1000.0))
        stdout = proc.stdout or ""
        stderr = proc.stderr or ""
        if stdout.strip():
            emit_text_safely(stdout, sys.stdout)
        if stderr.strip():
            emit_text_safely(stderr, sys.stderr)
        ok = proc.returncode == 0
        row = {
            "step": step_name,
            "ok": ok,
            "returncode": int(proc.returncode),
            "cmd": cmd,
            "elapsed_ms": elapsed_ms,
        }
        if not ok:
            code = parse_fail_code(stdout, stderr, default_code)
            msg = first_message(stdout, stderr)
            row["code"] = code
            row["msg"] = msg
            rows.append(row)
            summary_fields = build_sanity_summary_fields(args.profile, rows, completion_gate_reports)
            payload = {
                "schema": "ddn.ci.sanity_gate.v1",
                "generated_at_utc": datetime.now(timezone.utc).isoformat(),
                "status": "fail",
                "code": code,
                "step": step_name,
                "msg": msg,
                "profile": args.profile,
                "current_step": step_name,
                "last_completed_step": last_completed_step,
                **summary_fields,
                "steps": rows,
                "steps_count": len(rows),
                "total_elapsed_ms": int(round((time.perf_counter() - started) * 1000.0)),
            }
            write_json_snapshot(args.json_out, payload)
            print(f'ci_sanity_status=fail code={code} step={step_name} msg="{msg}" profile={args.profile}')
            return 1
        rows.append(row)
        if step_name == "age2_completion_gate":
            sync_json_report_snapshot(
                str(completion_gate_reports.get("age2_gate", "")),
                str(completion_gate_reports.get("age2_dynamic_source", "")),
            )
        elif step_name == "age3_completion_gate":
            sync_json_report_snapshot(
                str(completion_gate_reports.get("age3_gate", "")),
                str(completion_gate_reports.get("age3_dynamic_source", "")),
            )
        elif step_name == "age4_proof_transport_contract_selftest":
            sync_json_report_snapshot(
                str(completion_gate_reports.get("age4_transport_report", "")),
                str(completion_gate_reports.get("age4_dynamic_source", "")),
            )
        elif step_name == "ci_pack_golden_age5_surface_selftest":
            sync_json_report_snapshot(
                str(completion_gate_reports.get("age5_surface_report", "")),
                str(completion_gate_reports.get("age5_dynamic_source", "")),
            )
        maybe_mark_step_script_ok(cmd)
        emit_step_progress_tokens(step_name, step_env)
        last_completed_step = step_name
        write_json_snapshot(
            args.json_out,
            {
                "schema": "ddn.ci.sanity_gate.v1",
                "generated_at_utc": datetime.now(timezone.utc).isoformat(),
                "status": "running",
                "code": "RUNNING",
                "step": step_name,
                "msg": "-",
                "profile": args.profile,
                "current_step": "-",
                "last_completed_step": last_completed_step,
                **build_sanity_summary_fields(args.profile, rows, completion_gate_reports),
                "steps": rows,
                "steps_count": len(rows),
                "total_elapsed_ms": int(round((time.perf_counter() - started) * 1000.0)),
            },
        )
        print(f"ci_sanity_last_completed_step={step_name}", flush=True)

    summary_fields = build_sanity_summary_fields(args.profile, rows, completion_gate_reports)
    payload = {
        "schema": "ddn.ci.sanity_gate.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": "pass",
        "code": "OK",
        "step": "all",
        "msg": "-",
        "profile": args.profile,
        "current_step": "-",
        "last_completed_step": last_completed_step,
        **summary_fields,
        "steps": rows,
        "steps_count": len(rows),
        "total_elapsed_ms": int(round((time.perf_counter() - started) * 1000.0)),
    }
    write_json_snapshot(args.json_out, payload)
    print(f'ci_sanity_status=pass code=OK step=all msg="-" profile={args.profile}')
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
