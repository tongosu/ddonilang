#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path


MATRIX_SCHEMA = "ddn.ci.profile_matrix_gate.v1"
MATRIX_OK = "OK"
MATRIX_PROFILE_INVALID = "E_CI_PROFILE_MATRIX_PROFILE_INVALID"
MATRIX_STEP_FAIL = "E_CI_PROFILE_MATRIX_STEP_FAIL"
MATRIX_WARN_QUICK_ENV_INVALID = "W_CI_PROFILE_MATRIX_QUICK_ENV_INVALID"

VALID_PROFILES = ("core_lang", "full", "seamgrim")
AGGREGATE_SUMMARY_SANITY_KEYS = (
    "ci_sanity_pack_golden_lang_consistency_ok",
    "ci_sanity_pack_golden_metadata_ok",
    "ci_sanity_pack_golden_graph_export_ok",
    "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok",
    "ci_sanity_canon_ast_dpack_ok",
    "ci_sanity_seamgrim_numeric_factor_policy_ok",
    "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok",
)
PROFILE_AGGREGATE_SUMMARY_EXPECTATIONS = {
    "core_lang": {
        "ci_sanity_pack_golden_lang_consistency_ok": "1",
        "ci_sanity_pack_golden_metadata_ok": "1",
        "ci_sanity_pack_golden_graph_export_ok": "1",
        "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok": "1",
        "ci_sanity_canon_ast_dpack_ok": "1",
        "ci_sanity_seamgrim_numeric_factor_policy_ok": "na",
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok": "na",
    },
    "full": {
        "ci_sanity_pack_golden_lang_consistency_ok": "1",
        "ci_sanity_pack_golden_metadata_ok": "1",
        "ci_sanity_pack_golden_graph_export_ok": "1",
        "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok": "1",
        "ci_sanity_canon_ast_dpack_ok": "1",
        "ci_sanity_seamgrim_numeric_factor_policy_ok": "1",
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok": "1",
    },
    "seamgrim": {
        "ci_sanity_pack_golden_lang_consistency_ok": "0",
        "ci_sanity_pack_golden_metadata_ok": "0",
        "ci_sanity_pack_golden_graph_export_ok": "0",
        "ci_sync_readiness_ci_sanity_pack_golden_graph_export_ok": "0",
        "ci_sanity_canon_ast_dpack_ok": "0",
        "ci_sanity_seamgrim_numeric_factor_policy_ok": "1",
        "ci_sync_readiness_ci_sanity_seamgrim_numeric_factor_policy_ok": "1",
    },
}
PROFILE_AGGREGATE_SUMMARY_SUCCESS_MARKERS = {
    "full": "[ci-profile-full] aggregate summary sanity markers ok",
    "seamgrim": "[ci-profile-seamgrim] aggregate summary sanity markers ok",
}
PROFILE_GATE_SCRIPTS = {
    "core_lang": "tests/run_ci_profile_core_lang_gate.py",
    "full": "tests/run_ci_profile_full_gate.py",
    "seamgrim": "tests/run_ci_profile_seamgrim_gate.py",
}
PROFILE_PASS_MARKERS = {
    "core_lang": "ci_profile_core_lang_status=pass",
    "full": "ci_profile_full_status=pass",
    "seamgrim": "ci_profile_seamgrim_status=pass",
}
QUICK_GATES_ENV_KEY = "DDN_CI_PROFILE_MATRIX_QUICK_GATES"
ENV_FLAG_TRUE_VALUES = {"1", "true", "yes", "on"}
ENV_FLAG_FALSE_VALUES = {"0", "false", "no", "off"}


def parse_env_flag(name: str) -> tuple[bool, bool, str, str, str]:
    raw_value = str(os.environ.get(name, ""))
    normalized = raw_value.strip().lower()
    if not normalized:
        return False, True, raw_value, normalized, "empty"
    if normalized in ENV_FLAG_TRUE_VALUES:
        return True, True, raw_value, normalized, "true"
    if normalized in ENV_FLAG_FALSE_VALUES:
        return False, True, raw_value, normalized, "false"
    return False, False, raw_value, normalized, "invalid"


def resolve_quick_gates_source(quick_gates_arg: bool, quick_gates_env: bool) -> str:
    if quick_gates_arg and quick_gates_env:
        return "arg+env"
    if quick_gates_arg:
        return "arg"
    if quick_gates_env:
        return "env"
    return "none"


def source_uses_arg(source: str) -> bool:
    return source in {"arg", "arg+env"}


def source_uses_env(source: str) -> bool:
    return source in {"env", "arg+env"}


def resolve_quick_decision_reason(quick_gates_arg: bool, quick_gates_env: bool, quick_gates_env_state: str) -> str:
    state = str(quick_gates_env_state).strip().lower()
    if quick_gates_arg:
        if state == "true":
            return "arg_and_env_true"
        if state == "false":
            return "arg_with_env_false"
        if state == "invalid":
            return "arg_with_env_invalid"
        return "arg_only"
    if quick_gates_env:
        return "env_only_true"
    if state == "false":
        return "none_with_env_false"
    if state == "invalid":
        return "none_with_env_invalid"
    if state == "empty":
        return "none_no_inputs"
    return "none_unknown"


def expected_quick_decision_reason(quick_gates_arg: bool, quick_gates_env_state: str) -> str:
    state = str(quick_gates_env_state).strip().lower()
    if quick_gates_arg:
        if state == "true":
            return "arg_and_env_true"
        if state == "false":
            return "arg_with_env_false"
        if state == "invalid":
            return "arg_with_env_invalid"
        return "arg_only"
    if state == "true":
        return "env_only_true"
    if state == "false":
        return "none_with_env_false"
    if state == "invalid":
        return "none_with_env_invalid"
    if state == "empty":
        return "none_no_inputs"
    return "none_unknown"


def parse_profile_gate_overrides(raw: list[str]) -> tuple[dict[str, str], list[str]]:
    overrides: dict[str, str] = {}
    invalid: list[str] = []
    for token in raw:
        item = str(token).strip()
        if not item or "=" not in item:
            invalid.append(item or "(empty)")
            continue
        profile, path = item.split("=", 1)
        profile = profile.strip()
        path = path.strip()
        if profile not in VALID_PROFILES or not path:
            invalid.append(item)
            continue
        overrides[profile] = path
    return overrides, invalid


def resolve_profile_gate_script(profile: str, profile_gate_overrides: dict[str, str]) -> str:
    cli_override = profile_gate_overrides.get(profile, "").strip()
    if cli_override:
        return cli_override
    env_key = f"DDN_CI_PROFILE_MATRIX_GATE_OVERRIDE_{profile.upper()}"
    override = str(os.environ.get(env_key, "")).strip()
    if override:
        return override
    return PROFILE_GATE_SCRIPTS[profile]


def clip(text: str, limit: int = 180) -> str:
    normalized = " ".join(str(text).split())
    if not normalized:
        return "-"
    if len(normalized) <= limit:
        return normalized
    return normalized[: limit - 3] + "..."


def parse_profiles(raw: str) -> tuple[list[str], list[str]]:
    seen: set[str] = set()
    ordered: list[str] = []
    invalid: list[str] = []
    for token in str(raw).split(","):
        name = token.strip()
        if not name:
            continue
        if name not in VALID_PROFILES:
            if name not in invalid:
                invalid.append(name)
            continue
        if name in seen:
            continue
        seen.add(name)
        ordered.append(name)
    return ordered, invalid


def decode_timeout_text(value: object) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode("utf-8", errors="replace")
    return str(value)


def run_step(cmd: list[str], cwd: Path, timeout_sec: float | None = None) -> tuple[subprocess.CompletedProcess[str], bool]:
    env = dict(os.environ)
    env["PYTHONUNBUFFERED"] = "1"
    try:
        proc = subprocess.run(
            cmd,
            cwd=cwd,
            env=env,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=timeout_sec,
        )
        return proc, False
    except subprocess.TimeoutExpired as exc:
        stdout = decode_timeout_text(exc.stdout)
        stderr = decode_timeout_text(exc.stderr)
        timeout_note = "step timeout after {:.3f}s".format(float(timeout_sec or 0.0))
        if stderr.strip():
            stderr = f"{stderr.rstrip()}\n{timeout_note}"
        else:
            stderr = timeout_note
        return subprocess.CompletedProcess(exc.cmd, 124, stdout, stderr), True


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


def build_core_lang_sanity_markers_from_report(path: Path) -> list[str]:
    payload = load_json(path)
    if not isinstance(payload, dict):
        return []
    total_elapsed_ms = str(payload.get("total_elapsed_ms", "-")).strip() or "-"
    current_step = str(payload.get("current_step", "-")).strip() or "-"
    last_completed_step = str(payload.get("last_completed_step", "-")).strip() or "-"
    slowest_step = "-"
    slowest_elapsed_ms = "-"
    steps = payload.get("steps")
    if isinstance(steps, list):
        ranked: list[tuple[int, str]] = []
        for row in steps:
            if not isinstance(row, dict):
                continue
            step = str(row.get("step", "")).strip()
            if not step:
                continue
            try:
                elapsed_ms = int(row.get("elapsed_ms", 0))
            except Exception:
                elapsed_ms = 0
            ranked.append((elapsed_ms, step))
        if ranked:
            ranked.sort(key=lambda item: (-item[0], item[1]))
            slowest_elapsed_ms = str(ranked[0][0])
            slowest_step = ranked[0][1]
    return [
        f"ci_profile_core_lang_sanity_total_elapsed_ms={total_elapsed_ms}",
        f"ci_profile_core_lang_sanity_slowest_step={slowest_step}",
        f"ci_profile_core_lang_sanity_slowest_elapsed_ms={slowest_elapsed_ms}",
        f"ci_sanity_current_step={current_step}",
        f"ci_sanity_last_completed_step={last_completed_step}",
    ]


def emit_profile_partial_markers(profile: str, profile_report_dir: Path, child_prefix: str) -> None:
    if profile != "core_lang":
        return
    sanity_report = profile_report_dir / f"{child_prefix}.ci_sanity_gate.detjson"
    for marker in build_core_lang_sanity_markers_from_report(sanity_report):
        print(marker)
    progress_report = profile_report_dir / f"{child_prefix}.ci_sanity_gate.pipeline_emit_flags_check.progress.detjson"
    payload = load_json(progress_report)
    if isinstance(payload, dict):
        current_section = str(payload.get("current_section", "-")).strip() or "-"
        last_completed_section = str(payload.get("last_completed_section", "-")).strip() or "-"
        total_elapsed_ms = str(payload.get("total_elapsed_ms", "-")).strip() or "-"
        print(f"ci_pipeline_emit_flags_current_section={current_section}")
        print(f"ci_pipeline_emit_flags_last_completed_section={last_completed_section}")
        print(f"ci_pipeline_emit_flags_total_elapsed_ms={total_elapsed_ms}")
    selftest_progress_report = (
        profile_report_dir / f"{child_prefix}.ci_sanity_gate.pipeline_emit_flags_selftest.progress.detjson"
    )
    selftest_payload = load_json(selftest_progress_report)
    if isinstance(selftest_payload, dict):
        current_case = str(selftest_payload.get("current_case", "-")).strip() or "-"
        last_completed_case = str(selftest_payload.get("last_completed_case", "-")).strip() or "-"
        current_probe = str(selftest_payload.get("current_probe", "-")).strip() or "-"
        last_completed_probe = str(selftest_payload.get("last_completed_probe", "-")).strip() or "-"
        total_elapsed_ms = str(selftest_payload.get("total_elapsed_ms", "-")).strip() or "-"
        print(f"ci_pipeline_emit_flags_selftest_current_case={current_case}")
        print(f"ci_pipeline_emit_flags_selftest_last_completed_case={last_completed_case}")
        print(f"ci_pipeline_emit_flags_selftest_current_probe={current_probe}")
        print(f"ci_pipeline_emit_flags_selftest_last_completed_probe={last_completed_probe}")
        print(f"ci_pipeline_emit_flags_selftest_total_elapsed_ms={total_elapsed_ms}")
    age5_policy_selftest_progress_report = (
        profile_report_dir / f"{child_prefix}.ci_sanity_gate.age5_combined_heavy_policy_selftest.progress.detjson"
    )
    age5_policy_selftest_payload = load_json(age5_policy_selftest_progress_report)
    if isinstance(age5_policy_selftest_payload, dict):
        current_case = str(age5_policy_selftest_payload.get("current_case", "-")).strip() or "-"
        last_completed_case = str(age5_policy_selftest_payload.get("last_completed_case", "-")).strip() or "-"
        current_format = str(age5_policy_selftest_payload.get("current_format", "-")).strip() or "-"
        last_completed_format = str(age5_policy_selftest_payload.get("last_completed_format", "-")).strip() or "-"
        current_probe = str(age5_policy_selftest_payload.get("current_probe", "-")).strip() or "-"
        last_completed_probe = str(age5_policy_selftest_payload.get("last_completed_probe", "-")).strip() or "-"
        total_elapsed_ms = str(age5_policy_selftest_payload.get("total_elapsed_ms", "-")).strip() or "-"
        print(f"ci_age5_combined_heavy_policy_selftest_current_case={current_case}")
        print(f"ci_age5_combined_heavy_policy_selftest_last_completed_case={last_completed_case}")
        print(f"ci_age5_combined_heavy_policy_selftest_current_format={current_format}")
        print(f"ci_age5_combined_heavy_policy_selftest_last_completed_format={last_completed_format}")
        print(f"ci_age5_combined_heavy_policy_selftest_current_probe={current_probe}")
        print(f"ci_age5_combined_heavy_policy_selftest_last_completed_probe={last_completed_probe}")
        print(f"ci_age5_combined_heavy_policy_selftest_total_elapsed_ms={total_elapsed_ms}")
    profile_matrix_policy_selftest_progress_report = (
        profile_report_dir / f"{child_prefix}.ci_sanity_gate.profile_matrix_full_real_smoke_policy_selftest.progress.detjson"
    )
    profile_matrix_policy_selftest_payload = load_json(profile_matrix_policy_selftest_progress_report)
    if isinstance(profile_matrix_policy_selftest_payload, dict):
        current_case = str(profile_matrix_policy_selftest_payload.get("current_case", "-")).strip() or "-"
        last_completed_case = str(profile_matrix_policy_selftest_payload.get("last_completed_case", "-")).strip() or "-"
        current_format = str(profile_matrix_policy_selftest_payload.get("current_format", "-")).strip() or "-"
        last_completed_format = (
            str(profile_matrix_policy_selftest_payload.get("last_completed_format", "-")).strip() or "-"
        )
        total_elapsed_ms = str(profile_matrix_policy_selftest_payload.get("total_elapsed_ms", "-")).strip() or "-"
        print(f"ci_profile_matrix_full_real_smoke_policy_selftest_current_case={current_case}")
        print(f"ci_profile_matrix_full_real_smoke_policy_selftest_last_completed_case={last_completed_case}")
        print(f"ci_profile_matrix_full_real_smoke_policy_selftest_current_format={current_format}")
        print(f"ci_profile_matrix_full_real_smoke_policy_selftest_last_completed_format={last_completed_format}")
        print(f"ci_profile_matrix_full_real_smoke_policy_selftest_total_elapsed_ms={total_elapsed_ms}")
    profile_matrix_smoke_check_selftest_progress_report = (
        profile_report_dir / f"{child_prefix}.ci_sanity_gate.profile_matrix_full_real_smoke_check_selftest.progress.detjson"
    )
    profile_matrix_smoke_check_selftest_payload = load_json(profile_matrix_smoke_check_selftest_progress_report)
    if isinstance(profile_matrix_smoke_check_selftest_payload, dict):
        current_case = str(profile_matrix_smoke_check_selftest_payload.get("current_case", "-")).strip() or "-"
        last_completed_case = (
            str(profile_matrix_smoke_check_selftest_payload.get("last_completed_case", "-")).strip() or "-"
        )
        current_probe = str(profile_matrix_smoke_check_selftest_payload.get("current_probe", "-")).strip() or "-"
        last_completed_probe = (
            str(profile_matrix_smoke_check_selftest_payload.get("last_completed_probe", "-")).strip() or "-"
        )
        total_elapsed_ms = (
            str(profile_matrix_smoke_check_selftest_payload.get("total_elapsed_ms", "-")).strip() or "-"
        )
        print(f"ci_profile_matrix_full_real_smoke_check_selftest_current_case={current_case}")
        print(f"ci_profile_matrix_full_real_smoke_check_selftest_last_completed_case={last_completed_case}")
        print(f"ci_profile_matrix_full_real_smoke_check_selftest_current_probe={current_probe}")
        print(f"ci_profile_matrix_full_real_smoke_check_selftest_last_completed_probe={last_completed_probe}")
        print(f"ci_profile_matrix_full_real_smoke_check_selftest_total_elapsed_ms={total_elapsed_ms}")
    fixed64_readiness_progress_report = (
        profile_report_dir / f"{child_prefix}.ci_sanity_gate.fixed64_darwin_real_report_readiness_check_selftest.progress.detjson"
    )
    fixed64_readiness_payload = load_json(fixed64_readiness_progress_report)
    if isinstance(fixed64_readiness_payload, dict):
        current_case = str(fixed64_readiness_payload.get("current_case", "-")).strip() or "-"
        last_completed_case = str(fixed64_readiness_payload.get("last_completed_case", "-")).strip() or "-"
        current_probe = str(fixed64_readiness_payload.get("current_probe", "-")).strip() or "-"
        last_completed_probe = str(fixed64_readiness_payload.get("last_completed_probe", "-")).strip() or "-"
        total_elapsed_ms = str(fixed64_readiness_payload.get("total_elapsed_ms", "-")).strip() or "-"
        print(f"ci_fixed64_darwin_real_report_readiness_check_selftest_current_case={current_case}")
        print(f"ci_fixed64_darwin_real_report_readiness_check_selftest_last_completed_case={last_completed_case}")
        print(f"ci_fixed64_darwin_real_report_readiness_check_selftest_current_probe={current_probe}")
        print(f"ci_fixed64_darwin_real_report_readiness_check_selftest_last_completed_probe={last_completed_probe}")
        print(f"ci_fixed64_darwin_real_report_readiness_check_selftest_total_elapsed_ms={total_elapsed_ms}")
    map_access_progress_report = (
        profile_report_dir / f"{child_prefix}.ci_sanity_gate.map_access_contract_check.progress.detjson"
    )
    map_access_payload = load_json(map_access_progress_report)
    if isinstance(map_access_payload, dict):
        current_case = str(map_access_payload.get("current_case", "-")).strip() or "-"
        last_completed_case = str(map_access_payload.get("last_completed_case", "-")).strip() or "-"
        current_probe = str(map_access_payload.get("current_probe", "-")).strip() or "-"
        last_completed_probe = str(map_access_payload.get("last_completed_probe", "-")).strip() or "-"
        total_elapsed_ms = str(map_access_payload.get("total_elapsed_ms", "-")).strip() or "-"
        print(f"ci_map_access_contract_check_current_case={current_case}")
        print(f"ci_map_access_contract_check_last_completed_case={last_completed_case}")
        print(f"ci_map_access_contract_check_current_probe={current_probe}")
        print(f"ci_map_access_contract_check_last_completed_probe={last_completed_probe}")
        print(f"ci_map_access_contract_check_total_elapsed_ms={total_elapsed_ms}")
    tensor_v0_cli_progress_report = (
        profile_report_dir / f"{child_prefix}.ci_sanity_gate.tensor_v0_cli_check.progress.detjson"
    )
    tensor_v0_cli_payload = load_json(tensor_v0_cli_progress_report)
    if isinstance(tensor_v0_cli_payload, dict):
        current_case = str(tensor_v0_cli_payload.get("current_case", "-")).strip() or "-"
        last_completed_case = str(tensor_v0_cli_payload.get("last_completed_case", "-")).strip() or "-"
        current_probe = str(tensor_v0_cli_payload.get("current_probe", "-")).strip() or "-"
        last_completed_probe = str(tensor_v0_cli_payload.get("last_completed_probe", "-")).strip() or "-"
        total_elapsed_ms = str(tensor_v0_cli_payload.get("total_elapsed_ms", "-")).strip() or "-"
        print(f"ci_tensor_v0_cli_check_current_case={current_case}")
        print(f"ci_tensor_v0_cli_check_last_completed_case={last_completed_case}")
        print(f"ci_tensor_v0_cli_check_current_probe={current_probe}")
        print(f"ci_tensor_v0_cli_check_last_completed_probe={last_completed_probe}")
        print(f"ci_tensor_v0_cli_check_total_elapsed_ms={total_elapsed_ms}")
    pack_golden_age5_surface_selftest_progress_report = (
        profile_report_dir / f"{child_prefix}.ci_sanity_gate.ci_pack_golden_age5_surface_selftest.progress.detjson"
    )
    pack_golden_age5_surface_selftest_payload = load_json(pack_golden_age5_surface_selftest_progress_report)
    if isinstance(pack_golden_age5_surface_selftest_payload, dict):
        current_case = str(pack_golden_age5_surface_selftest_payload.get("current_case", "-")).strip() or "-"
        last_completed_case = (
            str(pack_golden_age5_surface_selftest_payload.get("last_completed_case", "-")).strip() or "-"
        )
        current_probe = str(pack_golden_age5_surface_selftest_payload.get("current_probe", "-")).strip() or "-"
        last_completed_probe = (
            str(pack_golden_age5_surface_selftest_payload.get("last_completed_probe", "-")).strip() or "-"
        )
        total_elapsed_ms = (
            str(pack_golden_age5_surface_selftest_payload.get("total_elapsed_ms", "-")).strip() or "-"
        )
        print(f"ci_pack_golden_age5_surface_selftest_current_case={current_case}")
        print(f"ci_pack_golden_age5_surface_selftest_last_completed_case={last_completed_case}")
        print(f"ci_pack_golden_age5_surface_selftest_current_probe={current_probe}")
        print(f"ci_pack_golden_age5_surface_selftest_last_completed_probe={last_completed_probe}")
        print(f"ci_pack_golden_age5_surface_selftest_total_elapsed_ms={total_elapsed_ms}")
    pack_golden_guideblock_selftest_progress_report = (
        profile_report_dir / f"{child_prefix}.ci_sanity_gate.ci_pack_golden_guideblock_selftest.progress.detjson"
    )
    pack_golden_guideblock_selftest_payload = load_json(pack_golden_guideblock_selftest_progress_report)
    if isinstance(pack_golden_guideblock_selftest_payload, dict):
        current_case = str(pack_golden_guideblock_selftest_payload.get("current_case", "-")).strip() or "-"
        last_completed_case = (
            str(pack_golden_guideblock_selftest_payload.get("last_completed_case", "-")).strip() or "-"
        )
        current_probe = str(pack_golden_guideblock_selftest_payload.get("current_probe", "-")).strip() or "-"
        last_completed_probe = (
            str(pack_golden_guideblock_selftest_payload.get("last_completed_probe", "-")).strip() or "-"
        )
        total_elapsed_ms = str(pack_golden_guideblock_selftest_payload.get("total_elapsed_ms", "-")).strip() or "-"
        print(f"ci_pack_golden_guideblock_selftest_current_case={current_case}")
        print(f"ci_pack_golden_guideblock_selftest_last_completed_case={last_completed_case}")
        print(f"ci_pack_golden_guideblock_selftest_current_probe={current_probe}")
        print(f"ci_pack_golden_guideblock_selftest_last_completed_probe={last_completed_probe}")
        print(f"ci_pack_golden_guideblock_selftest_total_elapsed_ms={total_elapsed_ms}")
    pack_golden_exec_policy_selftest_progress_report = (
        profile_report_dir / f"{child_prefix}.ci_sanity_gate.ci_pack_golden_exec_policy_selftest.progress.detjson"
    )
    pack_golden_exec_policy_selftest_payload = load_json(pack_golden_exec_policy_selftest_progress_report)
    if isinstance(pack_golden_exec_policy_selftest_payload, dict):
        current_case = str(pack_golden_exec_policy_selftest_payload.get("current_case", "-")).strip() or "-"
        last_completed_case = (
            str(pack_golden_exec_policy_selftest_payload.get("last_completed_case", "-")).strip() or "-"
        )
        current_probe = str(pack_golden_exec_policy_selftest_payload.get("current_probe", "-")).strip() or "-"
        last_completed_probe = (
            str(pack_golden_exec_policy_selftest_payload.get("last_completed_probe", "-")).strip() or "-"
        )
        total_elapsed_ms = (
            str(pack_golden_exec_policy_selftest_payload.get("total_elapsed_ms", "-")).strip() or "-"
        )
        print(f"ci_pack_golden_exec_policy_selftest_current_case={current_case}")
        print(f"ci_pack_golden_exec_policy_selftest_last_completed_case={last_completed_case}")
        print(f"ci_pack_golden_exec_policy_selftest_current_probe={current_probe}")
        print(f"ci_pack_golden_exec_policy_selftest_last_completed_probe={last_completed_probe}")
        print(f"ci_pack_golden_exec_policy_selftest_total_elapsed_ms={total_elapsed_ms}")
    pack_golden_jjaim_flatten_selftest_progress_report = (
        profile_report_dir / f"{child_prefix}.ci_sanity_gate.ci_pack_golden_jjaim_flatten_selftest.progress.detjson"
    )
    pack_golden_jjaim_flatten_selftest_payload = load_json(pack_golden_jjaim_flatten_selftest_progress_report)
    if isinstance(pack_golden_jjaim_flatten_selftest_payload, dict):
        current_case = str(pack_golden_jjaim_flatten_selftest_payload.get("current_case", "-")).strip() or "-"
        last_completed_case = (
            str(pack_golden_jjaim_flatten_selftest_payload.get("last_completed_case", "-")).strip() or "-"
        )
        current_probe = str(pack_golden_jjaim_flatten_selftest_payload.get("current_probe", "-")).strip() or "-"
        last_completed_probe = (
            str(pack_golden_jjaim_flatten_selftest_payload.get("last_completed_probe", "-")).strip() or "-"
        )
        total_elapsed_ms = (
            str(pack_golden_jjaim_flatten_selftest_payload.get("total_elapsed_ms", "-")).strip() or "-"
        )
        print(f"ci_pack_golden_jjaim_flatten_selftest_current_case={current_case}")
        print(f"ci_pack_golden_jjaim_flatten_selftest_last_completed_case={last_completed_case}")
        print(f"ci_pack_golden_jjaim_flatten_selftest_current_probe={current_probe}")
        print(f"ci_pack_golden_jjaim_flatten_selftest_last_completed_probe={last_completed_probe}")
        print(f"ci_pack_golden_jjaim_flatten_selftest_total_elapsed_ms={total_elapsed_ms}")
    pack_golden_event_model_selftest_progress_report = (
        profile_report_dir / f"{child_prefix}.ci_sanity_gate.ci_pack_golden_event_model_selftest.progress.detjson"
    )
    pack_golden_event_model_selftest_payload = load_json(pack_golden_event_model_selftest_progress_report)
    if isinstance(pack_golden_event_model_selftest_payload, dict):
        current_case = str(pack_golden_event_model_selftest_payload.get("current_case", "-")).strip() or "-"
        last_completed_case = (
            str(pack_golden_event_model_selftest_payload.get("last_completed_case", "-")).strip() or "-"
        )
        current_probe = str(pack_golden_event_model_selftest_payload.get("current_probe", "-")).strip() or "-"
        last_completed_probe = (
            str(pack_golden_event_model_selftest_payload.get("last_completed_probe", "-")).strip() or "-"
        )
        total_elapsed_ms = (
            str(pack_golden_event_model_selftest_payload.get("total_elapsed_ms", "-")).strip() or "-"
        )
        print(f"ci_pack_golden_event_model_selftest_current_case={current_case}")
        print(f"ci_pack_golden_event_model_selftest_last_completed_case={last_completed_case}")
        print(f"ci_pack_golden_event_model_selftest_current_probe={current_probe}")
        print(f"ci_pack_golden_event_model_selftest_last_completed_probe={last_completed_probe}")
        print(f"ci_pack_golden_event_model_selftest_total_elapsed_ms={total_elapsed_ms}")
    pack_golden_lang_consistency_selftest_progress_report = (
        profile_report_dir / f"{child_prefix}.ci_sanity_gate.ci_pack_golden_lang_consistency_selftest.progress.detjson"
    )
    pack_golden_lang_consistency_selftest_payload = load_json(
        pack_golden_lang_consistency_selftest_progress_report
    )
    if isinstance(pack_golden_lang_consistency_selftest_payload, dict):
        current_case = str(pack_golden_lang_consistency_selftest_payload.get("current_case", "-")).strip() or "-"
        last_completed_case = (
            str(pack_golden_lang_consistency_selftest_payload.get("last_completed_case", "-")).strip() or "-"
        )
        current_probe = str(pack_golden_lang_consistency_selftest_payload.get("current_probe", "-")).strip() or "-"
        last_completed_probe = (
            str(pack_golden_lang_consistency_selftest_payload.get("last_completed_probe", "-")).strip() or "-"
        )
        total_elapsed_ms = (
            str(pack_golden_lang_consistency_selftest_payload.get("total_elapsed_ms", "-")).strip() or "-"
        )
        print(f"ci_pack_golden_lang_consistency_selftest_current_case={current_case}")
        print(f"ci_pack_golden_lang_consistency_selftest_last_completed_case={last_completed_case}")
        print(f"ci_pack_golden_lang_consistency_selftest_current_probe={current_probe}")
        print(f"ci_pack_golden_lang_consistency_selftest_last_completed_probe={last_completed_probe}")
        print(f"ci_pack_golden_lang_consistency_selftest_total_elapsed_ms={total_elapsed_ms}")
    w94_social_pack_check_progress_report = (
        profile_report_dir / f"{child_prefix}.ci_sanity_gate.w94_social_pack_check.progress.detjson"
    )
    w94_social_pack_check_payload = load_json(w94_social_pack_check_progress_report)
    if isinstance(w94_social_pack_check_payload, dict):
        current_case = str(w94_social_pack_check_payload.get("current_case", "-")).strip() or "-"
        last_completed_case = str(w94_social_pack_check_payload.get("last_completed_case", "-")).strip() or "-"
        current_probe = str(w94_social_pack_check_payload.get("current_probe", "-")).strip() or "-"
        last_completed_probe = str(w94_social_pack_check_payload.get("last_completed_probe", "-")).strip() or "-"
        total_elapsed_ms = str(w94_social_pack_check_payload.get("total_elapsed_ms", "-")).strip() or "-"
        print(f"w94_social_pack_check_current_case={current_case}")
        print(f"w94_social_pack_check_last_completed_case={last_completed_case}")
        print(f"w94_social_pack_check_current_probe={current_probe}")
        print(f"w94_social_pack_check_last_completed_probe={last_completed_probe}")
        print(f"w94_social_pack_check_total_elapsed_ms={total_elapsed_ms}")
    w95_cert_pack_check_progress_report = (
        profile_report_dir / f"{child_prefix}.ci_sanity_gate.w95_cert_pack_check.progress.detjson"
    )
    w95_cert_pack_check_payload = load_json(w95_cert_pack_check_progress_report)
    if isinstance(w95_cert_pack_check_payload, dict):
        current_case = str(w95_cert_pack_check_payload.get("current_case", "-")).strip() or "-"
        last_completed_case = str(w95_cert_pack_check_payload.get("last_completed_case", "-")).strip() or "-"
        current_probe = str(w95_cert_pack_check_payload.get("current_probe", "-")).strip() or "-"
        last_completed_probe = str(w95_cert_pack_check_payload.get("last_completed_probe", "-")).strip() or "-"
        total_elapsed_ms = str(w95_cert_pack_check_payload.get("total_elapsed_ms", "-")).strip() or "-"
        print(f"w95_cert_pack_check_current_case={current_case}")
        print(f"w95_cert_pack_check_last_completed_case={last_completed_case}")
        print(f"w95_cert_pack_check_current_probe={current_probe}")
        print(f"w95_cert_pack_check_last_completed_probe={last_completed_probe}")
        print(f"w95_cert_pack_check_total_elapsed_ms={total_elapsed_ms}")
    w96_somssi_pack_check_progress_report = (
        profile_report_dir / f"{child_prefix}.ci_sanity_gate.w96_somssi_pack_check.progress.detjson"
    )
    w96_somssi_pack_check_payload = load_json(w96_somssi_pack_check_progress_report)
    if isinstance(w96_somssi_pack_check_payload, dict):
        current_case = str(w96_somssi_pack_check_payload.get("current_case", "-")).strip() or "-"
        last_completed_case = str(w96_somssi_pack_check_payload.get("last_completed_case", "-")).strip() or "-"
        current_probe = str(w96_somssi_pack_check_payload.get("current_probe", "-")).strip() or "-"
        last_completed_probe = str(w96_somssi_pack_check_payload.get("last_completed_probe", "-")).strip() or "-"
        total_elapsed_ms = str(w96_somssi_pack_check_payload.get("total_elapsed_ms", "-")).strip() or "-"
        print(f"w96_somssi_pack_check_current_case={current_case}")
        print(f"w96_somssi_pack_check_last_completed_case={last_completed_case}")
        print(f"w96_somssi_pack_check_current_probe={current_probe}")
        print(f"w96_somssi_pack_check_last_completed_probe={last_completed_probe}")
        print(f"w96_somssi_pack_check_total_elapsed_ms={total_elapsed_ms}")
    w97_self_heal_pack_check_progress_report = (
        profile_report_dir / f"{child_prefix}.ci_sanity_gate.w97_self_heal_pack_check.progress.detjson"
    )
    w97_self_heal_pack_check_payload = load_json(w97_self_heal_pack_check_progress_report)
    if isinstance(w97_self_heal_pack_check_payload, dict):
        current_case = str(w97_self_heal_pack_check_payload.get("current_case", "-")).strip() or "-"
        last_completed_case = str(w97_self_heal_pack_check_payload.get("last_completed_case", "-")).strip() or "-"
        current_probe = str(w97_self_heal_pack_check_payload.get("current_probe", "-")).strip() or "-"
        last_completed_probe = str(w97_self_heal_pack_check_payload.get("last_completed_probe", "-")).strip() or "-"
        total_elapsed_ms = str(w97_self_heal_pack_check_payload.get("total_elapsed_ms", "-")).strip() or "-"
        print(f"w97_self_heal_pack_check_current_case={current_case}")
        print(f"w97_self_heal_pack_check_last_completed_case={last_completed_case}")
        print(f"w97_self_heal_pack_check_current_probe={current_probe}")
        print(f"w97_self_heal_pack_check_last_completed_probe={last_completed_probe}")
        print(f"w97_self_heal_pack_check_total_elapsed_ms={total_elapsed_ms}")


def has_cmd_flag(cmd: object, flag: str) -> bool:
    return isinstance(cmd, list) and str(flag).strip() in cmd


def has_quick_flag(cmd: object) -> bool:
    return has_cmd_flag(cmd, "--quick")


def parse_ci_gate_summary(stdout: str) -> dict[str, str]:
    kv: dict[str, str] = {}
    for raw in str(stdout).splitlines():
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


def build_aggregate_summary_sanity(profile: str, stdout: str, expected_present: bool, reason: str) -> dict[str, object]:
    expected_values = dict(PROFILE_AGGREGATE_SUMMARY_EXPECTATIONS.get(profile, {}))
    if not expected_present:
        return {
            "expected_present": False,
            "present": False,
            "status": "skipped",
            "reason": reason,
            "expected_profile": profile,
            "expected_sync_profile": profile,
            "profile": "",
            "sync_profile": "",
            "expected_values": expected_values,
            "values": {key: "" for key in AGGREGATE_SUMMARY_SANITY_KEYS},
            "missing_keys": [],
            "mismatched_keys": [],
            "profile_ok": True,
            "sync_profile_ok": True,
            "values_ok": True,
            "gate_marker_expected": False,
            "gate_marker_present": False,
            "gate_marker_ok": True,
            "ok": True,
        }

    summary_kv = parse_ci_gate_summary(stdout)
    actual_values = {
        key: str(summary_kv.get(key, "")).strip()
        for key in AGGREGATE_SUMMARY_SANITY_KEYS
    }
    missing_keys = [key for key, value in actual_values.items() if not value]
    mismatched_keys = [
        key
        for key, expected in expected_values.items()
        if actual_values.get(key, "") != str(expected)
    ]
    profile_value = str(summary_kv.get("ci_sanity_gate_profile", "")).strip()
    sync_profile_value = str(summary_kv.get("ci_sync_readiness_sanity_profile", "")).strip()
    profile_ok = bool(profile_value == profile)
    sync_profile_ok = bool(sync_profile_value == profile)
    values_ok = bool(not mismatched_keys and not missing_keys)
    gate_marker = str(PROFILE_AGGREGATE_SUMMARY_SUCCESS_MARKERS.get(profile, "")).strip()
    gate_marker_expected = bool(gate_marker)
    gate_marker_present = bool(gate_marker and gate_marker in str(stdout))
    gate_marker_ok = bool((not gate_marker_expected) or gate_marker_present)
    present = bool(len(summary_kv) > 0)
    reason_value = "ok"
    if not present:
        reason_value = "missing"
    elif not profile_ok:
        reason_value = "profile_mismatch"
    elif not sync_profile_ok:
        reason_value = "sync_profile_mismatch"
    elif mismatched_keys:
        reason_value = f"mismatched_keys:{','.join(mismatched_keys)}"
    elif missing_keys:
        reason_value = f"missing_keys:{','.join(missing_keys)}"
    elif not gate_marker_ok:
        reason_value = "gate_marker_missing"
    ok = bool(present and profile_ok and sync_profile_ok and values_ok and gate_marker_ok)
    return {
        "expected_present": True,
        "present": present,
        "status": "pass" if ok else "fail",
        "reason": reason_value,
        "expected_profile": profile,
        "expected_sync_profile": profile,
        "profile": profile_value,
        "sync_profile": sync_profile_value,
        "expected_values": expected_values,
        "values": actual_values,
        "missing_keys": missing_keys,
        "mismatched_keys": mismatched_keys,
        "profile_ok": profile_ok,
        "sync_profile_ok": sync_profile_ok,
        "values_ok": values_ok,
        "gate_marker_expected": gate_marker_expected,
        "gate_marker_present": gate_marker_present,
        "gate_marker_ok": gate_marker_ok,
        "ok": ok,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Run CI profile gates as a single matrix entrypoint")
    parser.add_argument(
        "--profiles",
        default="core_lang,full,seamgrim",
        help="comma-separated profiles (core_lang,full,seamgrim)",
    )
    parser.add_argument("--report-dir", default="build/reports", help="report directory")
    parser.add_argument("--report-prefix", default="dev_ci_profile_matrix", help="report file prefix")
    parser.add_argument("--json-out", default="", help="optional explicit matrix report path")
    parser.add_argument("--dry-run", action="store_true", help="print planned matrix steps only")
    parser.add_argument("--stop-on-fail", action="store_true", help="stop matrix on first failed profile step")
    parser.add_argument(
        "--quick-gates",
        action="store_true",
        help="append --quick to each profile gate (skip aggregate/index stage in gate scripts)",
    )
    parser.add_argument(
        "--full-aggregate-gates",
        action="store_true",
        help="append --full-aggregate to each profile gate",
    )
    parser.add_argument(
        "--with-profile-matrix-full-real-smoke",
        action="store_true",
        help="append --with-profile-matrix-full-real-smoke to each profile gate",
    )
    parser.add_argument(
        "--profile-gate-override",
        action="append",
        default=[],
        help="profile gate script override in profile=path form (repeatable)",
    )
    parser.add_argument(
        "--step-timeout-sec",
        type=float,
        default=0.0,
        help="optional per-profile step timeout in seconds (0 disables timeout)",
    )
    args = parser.parse_args()
    if float(args.step_timeout_sec) < 0.0:
        print("step timeout must be >= 0", file=sys.stderr)
        return 2

    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    step_timeout_enabled = bool(float(args.step_timeout_sec) > 0.0)
    step_timeout_sec = float(args.step_timeout_sec) if step_timeout_enabled else 0.0
    profiles, invalid_profiles = parse_profiles(args.profiles)
    profile_gate_overrides, invalid_override_specs = parse_profile_gate_overrides(args.profile_gate_override)
    quick_gates_env, quick_gates_env_parse_ok, quick_gates_env_raw, quick_gates_env_normalized, quick_gates_env_state = parse_env_flag(QUICK_GATES_ENV_KEY)
    quick_gates_env_warning = "none"
    warnings: list[dict[str, str]] = []
    if str(quick_gates_env_raw).strip() and not quick_gates_env_parse_ok:
        quick_gates_env_warning = "invalid_value"
        warnings.append(
            {
                "code": MATRIX_WARN_QUICK_ENV_INVALID,
                "field": QUICK_GATES_ENV_KEY,
                "raw": str(quick_gates_env_raw),
                "msg": "invalid quick env value; expected true/false set",
            }
        )
    quick_gates_arg = bool(args.quick_gates)
    quick_gates = bool(quick_gates_arg or quick_gates_env)
    quick_gates_source = resolve_quick_gates_source(quick_gates_arg, quick_gates_env)
    quick_gates_source_uses_arg = source_uses_arg(quick_gates_source)
    quick_gates_source_uses_env = source_uses_env(quick_gates_source)
    quick_decision_reason = resolve_quick_decision_reason(quick_gates_arg, quick_gates_env, quick_gates_env_state)
    quick_decision_expected_reason = expected_quick_decision_reason(quick_gates_arg, quick_gates_env_state)
    if invalid_override_specs:
        invalid_profiles.extend([f"override:{item}" for item in invalid_override_specs])
    if not profiles:
        invalid_profiles = invalid_profiles or ["(empty)"]

    report_dir = Path(args.report_dir)
    report_dir.mkdir(parents=True, exist_ok=True)
    out_path = Path(args.json_out) if args.json_out.strip() else (report_dir / f"{args.report_prefix}.ci_profile_matrix_gate.detjson")

    rows: list[dict[str, object]] = []
    status = "pass"
    code = MATRIX_OK
    step = "all"
    msg = "-"
    matrix_started = time.perf_counter()

    if invalid_profiles:
        status = "fail"
        code = MATRIX_PROFILE_INVALID
        step = "profile_list"
        msg = f"invalid profiles: {','.join(invalid_profiles)}"
    else:
        for profile in profiles:
            script_rel = resolve_profile_gate_script(profile, profile_gate_overrides)
            script = Path(script_rel)
            if not script.is_absolute():
                script = root / script
            profile_report_dir = report_dir / f"{args.report_prefix}.profiles" / profile
            profile_report_dir.mkdir(parents=True, exist_ok=True)
            child_prefix = f"{args.report_prefix}.{profile}"
            cmd = [py, script_rel]
            if quick_gates:
                cmd.append("--quick")
            if bool(args.full_aggregate_gates):
                cmd.append("--full-aggregate")
            if bool(args.with_profile_matrix_full_real_smoke):
                cmd.append("--with-profile-matrix-full-real-smoke")
            cmd.extend(["--report-dir", str(profile_report_dir), "--report-prefix", child_prefix])
            if not script.exists():
                quick_applied = has_quick_flag(cmd)
                row = {
                    "profile": profile,
                    "script": script_rel,
                    "ok": False,
                    "returncode": 127,
                    "cmd": cmd,
                    "quick_applied": bool(quick_applied),
                    "full_aggregate_applied": bool(has_cmd_flag(cmd, "--full-aggregate")),
                    "full_real_smoke_applied": bool(
                        has_cmd_flag(cmd, "--with-profile-matrix-full-real-smoke")
                    ),
                    "timed_out": False,
                    "timeout_sec": step_timeout_sec,
                    "elapsed_ms": 0,
                    "stdout_head": "-",
                    "stderr_head": f"missing script: {script}",
                    "aggregate_summary_sanity": build_aggregate_summary_sanity(
                        profile,
                        "",
                        expected_present=False,
                        reason="missing_script",
                    ),
                }
                rows.append(row)
                status = "fail"
                code = MATRIX_STEP_FAIL
                step = profile
                msg = f"missing script: {script_rel}"
                if args.stop_on_fail:
                    break
                continue

            if args.dry_run:
                quick_applied = has_quick_flag(cmd)
                row = {
                    "profile": profile,
                    "script": script_rel,
                    "ok": True,
                    "returncode": 0,
                    "cmd": cmd,
                    "quick_applied": bool(quick_applied),
                    "full_aggregate_applied": bool(has_cmd_flag(cmd, "--full-aggregate")),
                    "full_real_smoke_applied": bool(
                        has_cmd_flag(cmd, "--with-profile-matrix-full-real-smoke")
                    ),
                    "timed_out": False,
                    "timeout_sec": step_timeout_sec,
                    "elapsed_ms": 0,
                    "stdout_head": "[dry-run]",
                    "stderr_head": "-",
                    "aggregate_summary_sanity": build_aggregate_summary_sanity(
                        profile,
                        "",
                        expected_present=False,
                        reason="dry_run",
                    ),
                }
                rows.append(row)
                continue

            step_started = time.perf_counter()
            proc, timed_out = run_step(
                cmd,
                root,
                timeout_sec=step_timeout_sec if step_timeout_enabled else None,
            )
            elapsed_ms = int(round((time.perf_counter() - step_started) * 1000.0))
            stdout = proc.stdout or ""
            stderr = proc.stderr or ""
            if stdout.strip():
                print(stdout, end="" if stdout.endswith("\n") else "\n")
            if stderr.strip():
                print(stderr, end="" if stderr.endswith("\n") else "\n", file=sys.stderr)
            if timed_out:
                emit_profile_partial_markers(profile, profile_report_dir, child_prefix)
            marker = PROFILE_PASS_MARKERS[profile]
            ok = proc.returncode == 0 and marker in stdout
            quick_applied = has_quick_flag(cmd)
            row = {
                "profile": profile,
                "script": script_rel,
                "ok": ok,
                "returncode": int(proc.returncode),
                "cmd": cmd,
                "quick_applied": bool(quick_applied),
                "full_aggregate_applied": bool(has_cmd_flag(cmd, "--full-aggregate")),
                "full_real_smoke_applied": bool(
                    has_cmd_flag(cmd, "--with-profile-matrix-full-real-smoke")
                ),
                "timed_out": bool(timed_out),
                "timeout_sec": step_timeout_sec,
                "elapsed_ms": elapsed_ms,
                "stdout_head": clip(stdout),
                "stderr_head": clip(stderr),
                "aggregate_summary_sanity": build_aggregate_summary_sanity(
                    profile,
                    stdout,
                    expected_present=bool(not quick_applied),
                    reason="quick_gates" if quick_applied else "aggregate_expected",
                ),
            }
            rows.append(row)
            if not ok:
                status = "fail"
                code = MATRIX_STEP_FAIL
                step = profile
                msg = f"profile step timeout: {profile}" if timed_out else f"profile step failed: {profile}"
                if args.stop_on_fail:
                    break

    quick_steps_profiles = [
        str(row.get("profile", ""))
        for row in rows
        if bool(row.get("quick_applied", False))
    ]
    quick_profile_flags: dict[str, bool] = {
        str(row.get("profile", "")): bool(row.get("quick_applied", False))
        for row in rows
        if str(row.get("profile", ""))
    }
    quick_enabled_profiles = [name for name, enabled in quick_profile_flags.items() if bool(enabled)]
    quick_disabled_profiles = [name for name, enabled in quick_profile_flags.items() if not bool(enabled)]
    quick_profile_count = int(len(quick_profile_flags))
    quick_steps_count = int(len(quick_steps_profiles))
    quick_steps_total = int(len(rows))
    quick_steps_all = bool(quick_steps_count == quick_steps_total) if quick_steps_total > 0 else bool(not quick_gates)
    quick_profile_flags_complete = bool(quick_profile_count == quick_steps_total)
    quick_contract_issues: list[str] = []
    if bool(quick_gates) != bool(quick_gates_arg or quick_gates_env):
        quick_contract_issues.append("quick_gates_input_mismatch")
    if bool(quick_gates_source_uses_arg) != bool(quick_gates_arg):
        quick_contract_issues.append("quick_source_arg_flag_mismatch")
    if bool(quick_gates_source_uses_env) != bool(quick_gates_env):
        quick_contract_issues.append("quick_source_env_flag_mismatch")
    if bool(quick_gates_source_uses_arg or quick_gates_source_uses_env) != bool(quick_gates):
        quick_contract_issues.append("quick_source_enabled_mismatch")
    if quick_steps_total != int(len(rows)):
        quick_contract_issues.append("quick_steps_total_mismatch")
    if quick_steps_count != int(sum(1 for row in rows if bool(row.get("quick_applied", False)))):
        quick_contract_issues.append("quick_steps_count_mismatch")
    if quick_steps_profiles != quick_enabled_profiles:
        quick_contract_issues.append("quick_enabled_profiles_mismatch")
    if quick_profile_count != quick_steps_total:
        quick_contract_issues.append("quick_profile_count_mismatch")
    if bool(quick_profile_flags_complete) != bool(quick_profile_count == quick_steps_total):
        quick_contract_issues.append("quick_profile_flags_complete_mismatch")
    expected_quick_steps_all = bool(quick_steps_count == quick_steps_total) if quick_steps_total > 0 else bool(not quick_gates)
    if bool(quick_steps_all) != bool(expected_quick_steps_all):
        quick_contract_issues.append("quick_steps_all_mismatch")
    for idx, row in enumerate(rows):
        row_cmd = row.get("cmd")
        row_quick_applied = bool(row.get("quick_applied", False))
        if row_quick_applied != has_quick_flag(row_cmd):
            quick_contract_issues.append(f"quick_row_flag_mismatch:{idx}")
    quick_contract_ok = bool(len(quick_contract_issues) == 0)
    full_aggregate_steps_count = int(
        sum(1 for row in rows if bool(row.get("full_aggregate_applied", False)))
    )
    full_real_smoke_steps_count = int(
        sum(1 for row in rows if bool(row.get("full_real_smoke_applied", False)))
    )
    full_aggregate_contract_ok = bool(
        full_aggregate_steps_count == quick_steps_total if bool(args.full_aggregate_gates) else full_aggregate_steps_count == 0
    )
    full_real_smoke_contract_ok = bool(
        full_real_smoke_steps_count == quick_steps_total
        if bool(args.with_profile_matrix_full_real_smoke)
        else full_real_smoke_steps_count == 0
    )
    quick_decision_contract_issues: list[str] = []
    if str(quick_decision_reason) != str(quick_decision_expected_reason):
        quick_decision_contract_issues.append("quick_reason_expected_mismatch")
    if str(quick_decision_reason).startswith("arg_") and not bool(quick_gates_arg):
        quick_decision_contract_issues.append("quick_reason_arg_prefix_mismatch")
    if str(quick_decision_reason).startswith("none_") and bool(quick_gates_arg):
        quick_decision_contract_issues.append("quick_reason_none_prefix_mismatch")
    if str(quick_decision_reason) in {"env_only_true", "arg_and_env_true"} and str(quick_gates_env_state) != "true":
        quick_decision_contract_issues.append("quick_reason_env_true_state_mismatch")
    if str(quick_decision_reason) in {"none_with_env_false", "arg_with_env_false"} and str(quick_gates_env_state) != "false":
        quick_decision_contract_issues.append("quick_reason_env_false_state_mismatch")
    if str(quick_decision_reason) in {"none_with_env_invalid", "arg_with_env_invalid"} and str(quick_gates_env_state) != "invalid":
        quick_decision_contract_issues.append("quick_reason_env_invalid_state_mismatch")
    if str(quick_decision_reason) in {"none_no_inputs", "arg_only"} and str(quick_gates_env_state) != "empty":
        quick_decision_contract_issues.append("quick_reason_env_empty_state_mismatch")
    quick_decision_contract_ok = bool(len(quick_decision_contract_issues) == 0)
    warning_count = int(len(warnings))
    has_warnings = bool(warning_count > 0)
    timed_out_steps = [
        str(row.get("profile", ""))
        for row in rows
        if bool(row.get("timed_out", False)) and str(row.get("profile", ""))
    ]
    timed_out_step_count = int(len(timed_out_steps))
    warning_code_counts: dict[str, int] = {}
    for item in warnings:
        code_key = str(item.get("code", "")).strip()
        if not code_key:
            continue
        warning_code_counts[code_key] = int(warning_code_counts.get(code_key, 0) + 1)
    total_elapsed_ms = 0 if args.dry_run else int(round((time.perf_counter() - matrix_started) * 1000.0))
    aggregate_summary_sanity_by_profile = {
        str(row.get("profile", "")): row.get("aggregate_summary_sanity", {})
        for row in rows
        if str(row.get("profile", ""))
    }
    aggregate_summary_sanity_checked_profiles = [
        str(profile_name)
        for profile_name, item in aggregate_summary_sanity_by_profile.items()
        if isinstance(item, dict) and bool(item.get("expected_present", False))
    ]
    aggregate_summary_sanity_failed_profiles = [
        str(profile_name)
        for profile_name, item in aggregate_summary_sanity_by_profile.items()
        if isinstance(item, dict)
        and bool(item.get("expected_present", False))
        and not bool(item.get("ok", False))
    ]
    aggregate_summary_sanity_skipped_profiles = [
        str(profile_name)
        for profile_name, item in aggregate_summary_sanity_by_profile.items()
        if isinstance(item, dict) and str(item.get("status", "")).strip() == "skipped"
    ]
    aggregate_summary_sanity_ok = bool(len(aggregate_summary_sanity_failed_profiles) == 0)

    payload = {
        "schema": MATRIX_SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": status,
        "ok": status == "pass",
        "code": code,
        "step": step,
        "msg": msg,
        "profiles": profiles,
        "invalid_profiles": invalid_profiles,
        "profile_gate_overrides": profile_gate_overrides,
        "invalid_override_specs": invalid_override_specs,
        "steps": rows,
        "total_elapsed_ms": total_elapsed_ms,
        "dry_run": bool(args.dry_run),
        "step_timeout_enabled": bool(step_timeout_enabled),
        "step_timeout_sec": step_timeout_sec,
        "quick_gates": bool(quick_gates),
        "quick_gates_arg": bool(quick_gates_arg),
        "quick_gates_env": bool(quick_gates_env),
        "quick_gates_env_parse_ok": bool(quick_gates_env_parse_ok),
        "quick_gates_env_raw": quick_gates_env_raw,
        "quick_gates_env_normalized": quick_gates_env_normalized,
        "quick_gates_env_state": quick_gates_env_state,
        "quick_gates_env_warning": quick_gates_env_warning,
        "quick_gates_env_key": QUICK_GATES_ENV_KEY,
        "quick_gates_source": quick_gates_source,
        "quick_gates_source_uses_arg": bool(quick_gates_source_uses_arg),
        "quick_gates_source_uses_env": bool(quick_gates_source_uses_env),
        "quick_decision_reason": quick_decision_reason,
        "quick_decision_expected_reason": quick_decision_expected_reason,
        "quick_decision_contract_ok": bool(quick_decision_contract_ok),
        "quick_decision_contract_issues": quick_decision_contract_issues,
        "quick_steps_profiles": quick_steps_profiles,
        "quick_enabled_profiles": quick_enabled_profiles,
        "quick_disabled_profiles": quick_disabled_profiles,
        "quick_profile_flags": quick_profile_flags,
        "quick_profile_count": quick_profile_count,
        "quick_profile_flags_complete": bool(quick_profile_flags_complete),
        "quick_steps_count": quick_steps_count,
        "quick_steps_total": quick_steps_total,
        "quick_steps_all": bool(quick_steps_all),
        "quick_contract_ok": bool(quick_contract_ok),
        "quick_contract_issues": quick_contract_issues,
        "full_aggregate_gates": bool(args.full_aggregate_gates),
        "with_profile_matrix_full_real_smoke": bool(args.with_profile_matrix_full_real_smoke),
        "full_aggregate_steps_count": full_aggregate_steps_count,
        "full_real_smoke_steps_count": full_real_smoke_steps_count,
        "full_aggregate_contract_ok": bool(full_aggregate_contract_ok),
        "full_real_smoke_contract_ok": bool(full_real_smoke_contract_ok),
        "warning_count": warning_count,
        "has_warnings": has_warnings,
        "timed_out_steps": timed_out_steps,
        "timed_out_step_count": timed_out_step_count,
        "warning_codes": [str(item.get("code", "")) for item in warnings],
        "warning_code_counts": warning_code_counts,
        "warnings": warnings,
        "aggregate_summary_sanity_by_profile": aggregate_summary_sanity_by_profile,
        "aggregate_summary_sanity_checked_profiles": aggregate_summary_sanity_checked_profiles,
        "aggregate_summary_sanity_failed_profiles": aggregate_summary_sanity_failed_profiles,
        "aggregate_summary_sanity_skipped_profiles": aggregate_summary_sanity_skipped_profiles,
        "aggregate_summary_sanity_ok": aggregate_summary_sanity_ok,
    }
    write_json(out_path, payload)

    print(
        "ci_profile_matrix_status={} code={} step={} profiles={} quick_gates={} quick_source={} quick_reason={} quick_reason_ok={} quick_env_parse_ok={} quick_env_state={} quick_env_warning={} quick_steps={}/{} quick_contract_ok={} full_aggregate_gates={} full_real_smoke_gates={} full_aggregate_contract_ok={} full_real_smoke_contract_ok={} warnings={} timeouts={} msg=\"{}\" report=\"{}\"".format(
            status,
            code,
            step,
            ",".join(profiles) if profiles else "-",
            str(bool(quick_gates)).lower(),
            quick_gates_source,
            quick_decision_reason,
            str(bool(quick_decision_contract_ok)).lower(),
            str(bool(quick_gates_env_parse_ok)).lower(),
            quick_gates_env_state,
            quick_gates_env_warning,
            quick_steps_count,
            quick_steps_total,
            str(bool(quick_contract_ok)).lower(),
            str(bool(args.full_aggregate_gates)).lower(),
            str(bool(args.with_profile_matrix_full_real_smoke)).lower(),
            str(bool(full_aggregate_contract_ok)).lower(),
            str(bool(full_real_smoke_contract_ok)).lower(),
            warning_count,
            timed_out_step_count,
            msg,
            out_path,
        )
    )
    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
