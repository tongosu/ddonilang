#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
import time
from contextlib import contextmanager
from pathlib import Path

from _ci_age5_combined_heavy_contract import (
    AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
    AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_DEFAULT,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_FRAGMENT,
    AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
    AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_FRAGMENT,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FRAGMENTS,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS_TEXT,
    AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS_FRAGMENT,
    AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS,
    AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FRAGMENTS,
    AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FRAGMENTS,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FIELDS,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FRAGMENTS,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_ENV_KEY,
    AGE5_COMBINED_HEAVY_MODE,
    AGE5_COMBINED_HEAVY_POLICY_SCHEMA,
    AGE5_COMBINED_HEAVY_POLICY_DIGEST_SCRIPT,
    AGE5_COMBINED_HEAVY_POLICY_REPORT_PATH,
    AGE5_COMBINED_HEAVY_POLICY_SUMMARY_PATH,
    AGE5_COMBINED_HEAVY_POLICY_SCRIPT,
    AGE5_COMBINED_HEAVY_POLICY_TEXT_PATH,
    AGE5_COMBINED_HEAVY_REPORTS_FRAGMENT,
    AGE5_COMBINED_HEAVY_REPORT_SCHEMA,
    AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA,
    AGE5_COMBINED_HEAVY_REQUIRED_REPORTS,
    AGE5_COMBINED_HEAVY_CRITERIA_FRAGMENT,
    AGE5_COMBINED_HEAVY_SCOPE_FRAGMENT,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_FRAGMENT,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_TEXT,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DEFAULT_FRAGMENT,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DEFAULT_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_DEFAULT,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_FRAGMENT,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_SCOPE_FRAGMENT,
    AGE5_COMBINED_HEAVY_TIMEOUT_MODE_SCOPE_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT,
    AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_FRAGMENT,
    AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_DEFAULT,
    AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_FRAGMENT,
    AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_KEY,
    AZURE_AGE5_COMBINED_HEAVY_POLICY_TOKENS,
    GITLAB_AGE5_COMBINED_HEAVY_POLICY_TOKENS,
    build_age4_proof_snapshot,
    build_age4_proof_snapshot_text,
    build_age4_proof_source_snapshot_fields,
    build_age5_combined_heavy_timeout_policy_fields,
)
from _ci_profile_matrix_full_real_smoke_contract import (
    PROFILE_MATRIX_FULL_AGGREGATE_ENV_KEY,
    AZURE_PROFILE_MATRIX_SELFTEST_GATE_TOKENS,
    AZURE_PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_TOKENS,
    GITLAB_PROFILE_MATRIX_SELFTEST_GATE_TOKENS,
    GITLAB_PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_TOKENS,
    PROFILE_MATRIX_SELFTEST_GATE_FLAGS_HELPER_SCRIPT,
    PROFILE_MATRIX_SELFTEST_GATE_FLAGS_POLICY_SCHEMA,
    PROFILE_MATRIX_SELFTEST_GATE_FLAGS_VAR,
    PROFILE_MATRIX_SELFTEST_FULL_AGGREGATE_FLAG,
    PROFILE_MATRIX_SELFTEST_FULL_REAL_SMOKE_FLAG,
    PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY,
    PROFILE_MATRIX_FULL_REAL_SMOKE_MODE,
    PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SCHEMA,
    PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SCRIPT,
    PROFILE_MATRIX_FULL_REAL_SMOKE_SCOPE_FRAGMENT,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_CORE_LANG,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_FULL,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_SEAMGRIM,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_CORE_LANG,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_FULL,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_SEAMGRIM,
    PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT,
)


PROGRESS_ENV_KEY = "DDN_CI_PIPELINE_EMIT_FLAGS_PROGRESS_JSON"
HELPER_PROGRESS_SCHEMA = "ddn.age5_combined_heavy_policy_progress.v1"
HELPER_PARENT_PENDING_STAGE = "spawn_pending_parent"
TOOLS_SCRIPTS_DIR = Path(__file__).resolve().parent.parent / "tools" / "scripts"
AGE5_COMBINED_POLICY_FAST_PATH_REASONS = {"default_off", "schedule", "manual_optin"}
_RUNTIME_CHECK_CACHE: dict[tuple[str, str], tuple[str, ...]] = {}
_PROFILE_MATRIX_POLICY_IMPL: dict[str, object] | None = None
_PROFILE_MATRIX_SELFTEST_GATE_POLICY_IMPL: dict[str, object] | None = None
RUNTIME_CONTRACT_MINIMAL_ENV_KEY = "DDN_CI_PIPELINE_RUNTIME_CONTRACT_MINIMAL"


def fail(msg: str) -> int:
    print(f"[ci-pipeline-emit-flags-check] fail: {msg}", file=sys.stderr)
    return 1


def load_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except Exception:
        return ""


def run(
    cmd: list[str],
    cwd: Path,
    env: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def load_json_doc(path: Path | None) -> dict[str, object] | None:
    if path is None or not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def build_runtime_cache_key(path: Path) -> str:
    try:
        resolved = path.resolve()
    except Exception:
        resolved = path
    try:
        stat = resolved.stat()
        return f"{resolved}|{int(stat.st_mtime_ns)}|{int(stat.st_size)}"
    except Exception:
        return str(resolved)


def is_runtime_contract_minimal_mode() -> bool:
    raw = str(os.environ.get(RUNTIME_CONTRACT_MINIMAL_ENV_KEY, "")).strip().lower()
    return raw in {"1", "true", "yes", "on"}


@contextmanager
def patched_environ(patch: dict[str, str]):
    original = {key: os.environ.get(key) for key in patch}
    try:
        for key, value in patch.items():
            os.environ[key] = str(value)
        yield
    finally:
        for key, value in original.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value


def load_age5_combined_policy_impl():
    if str(TOOLS_SCRIPTS_DIR) not in sys.path:
        sys.path.insert(0, str(TOOLS_SCRIPTS_DIR))
    from _resolve_age5_combined_heavy_policy_impl import (
        _load_contract_symbols,
        build_text_payload_fields,
        build_text_source_fields,
        render_shell,
        render_text_line,
        resolve,
    )
    _load_contract_symbols()

    return {
        "resolve": resolve,
        "render_shell": render_shell,
        "build_text_payload_fields": build_text_payload_fields,
        "build_text_source_fields": build_text_source_fields,
        "render_text_line": render_text_line,
    }


def load_profile_matrix_full_real_smoke_policy_impl() -> dict[str, object]:
    global _PROFILE_MATRIX_POLICY_IMPL
    if _PROFILE_MATRIX_POLICY_IMPL is not None:
        return _PROFILE_MATRIX_POLICY_IMPL
    if str(TOOLS_SCRIPTS_DIR) not in sys.path:
        sys.path.insert(0, str(TOOLS_SCRIPTS_DIR))
    from resolve_profile_matrix_full_real_smoke_policy import render_shell, resolve

    _PROFILE_MATRIX_POLICY_IMPL = {
        "resolve": resolve,
        "render_shell": render_shell,
    }
    return _PROFILE_MATRIX_POLICY_IMPL


def load_profile_matrix_selftest_gate_flags_impl() -> dict[str, object]:
    global _PROFILE_MATRIX_SELFTEST_GATE_POLICY_IMPL
    if _PROFILE_MATRIX_SELFTEST_GATE_POLICY_IMPL is not None:
        return _PROFILE_MATRIX_SELFTEST_GATE_POLICY_IMPL
    if str(TOOLS_SCRIPTS_DIR) not in sys.path:
        sys.path.insert(0, str(TOOLS_SCRIPTS_DIR))
    from resolve_profile_matrix_selftest_gate_flags import render_shell, resolve

    _PROFILE_MATRIX_SELFTEST_GATE_POLICY_IMPL = {
        "resolve": resolve,
        "render_shell": render_shell,
    }
    return _PROFILE_MATRIX_SELFTEST_GATE_POLICY_IMPL


def run_profile_matrix_policy_fast_path(
    *,
    provider: str,
    env_patch: dict[str, str],
    output_format: str,
) -> tuple[dict[str, object], str]:
    impl = load_profile_matrix_full_real_smoke_policy_impl()
    with patched_environ(env_patch):
        payload = impl["resolve"](provider)
    if output_format == "json":
        return payload, json.dumps(payload, ensure_ascii=False)
    if output_format == "shell":
        return payload, impl["render_shell"](payload)
    status = "enabled" if bool(payload.get("enabled", False)) else "disabled"
    text = (
        "[ci-profile-matrix-full-real-smoke-policy] "
        f"provider={payload['provider']} status={status} reason={payload['reason']} "
        f"{PROFILE_MATRIX_FULL_REAL_SMOKE_SCOPE_FRAGMENT} "
        f"step_timeout_defaults={payload.get('step_timeout_defaults_text', PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT)}"
    )
    return payload, text


def run_profile_matrix_selftest_gate_fast_path(
    *,
    provider: str,
    env_patch: dict[str, str],
    output_format: str,
) -> tuple[dict[str, object], str]:
    impl = load_profile_matrix_selftest_gate_flags_impl()
    with patched_environ(env_patch):
        payload = impl["resolve"](provider)
    if output_format == "json":
        return payload, json.dumps(payload, ensure_ascii=False)
    if output_format == "shell":
        return payload, impl["render_shell"](payload)
    status = "enabled" if bool(payload.get("enabled", False)) else "disabled"
    text = (
        "[ci-profile-matrix-selftest-gate-flags-policy] "
        f"provider={payload['provider']} status={status} "
        f"full_aggregate={str(bool(payload['full_aggregate_enabled'])).lower()} "
        f"full_real_smoke={str(bool(payload['full_real_smoke_enabled'])).lower()} "
        f"flags={payload['flags_text'] or '-'} "
        f"step_timeout_defaults={payload.get('step_timeout_defaults_text', PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT)}"
    )
    return payload, text


def run_age5_combined_policy_fast_path(
    *,
    provider: str,
    env_patch: dict[str, str],
    output_format: str,
    progress_hook,
    progress_prefix: str,
):
    impl = load_age5_combined_policy_impl()
    if progress_hook is not None:
        progress_hook(f"{progress_prefix}.fast_path.resolve")
    with patched_environ(env_patch):
        payload = impl["resolve"](provider)
    if output_format == "json":
        return payload, json.dumps(payload, ensure_ascii=False)
    if output_format == "shell":
        if progress_hook is not None:
            progress_hook(f"{progress_prefix}.fast_path.render_shell")
        return payload, impl["render_shell"](payload)
    if progress_hook is not None:
        progress_hook(f"{progress_prefix}.fast_path.render_text")
    age4_fields_text, age4_snapshot, age4_snapshot_text = impl["build_text_payload_fields"](payload)
    (
        age4_source_fields_text,
        gate_present,
        gate_parity,
        final_present,
        final_parity,
    ) = impl["build_text_source_fields"](payload, age4_snapshot)
    text_out = impl["render_text_line"](
        payload,
        age4_proof_snapshot_fields_text=age4_fields_text,
        age4_proof_snapshot_text=age4_snapshot_text,
        age4_proof_source_snapshot_fields_text=age4_source_fields_text,
        age4_proof_gate_result_snapshot_present=gate_present,
        age4_proof_gate_result_snapshot_parity=gate_parity,
        age4_proof_final_status_parse_snapshot_present=final_present,
        age4_proof_final_status_parse_snapshot_parity=final_parity,
    )
    return payload, text_out


def write_helper_seed_progress(
    path: Path,
    *,
    provider: str,
    output_format: str,
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": HELPER_PROGRESS_SCHEMA,
        "provider": provider,
        "format": output_format,
        "status": "running",
        "current_stage": HELPER_PARENT_PENDING_STAGE,
        "last_completed_stage": "-",
        "total_elapsed_ms": 0,
    }
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def run_with_progress(
    cmd: list[str],
    *,
    cwd: Path,
    env: dict[str, str],
    progress_path: Path,
    on_progress,
    on_phase=None,
) -> subprocess.CompletedProcess[str]:
    with tempfile.TemporaryDirectory(prefix="ci_pipeline_emit_flags_progress_") as td:
        stdout_path = Path(td) / "stdout.txt"
        stderr_path = Path(td) / "stderr.txt"
        with stdout_path.open("w", encoding="utf-8") as stdout_file, stderr_path.open(
            "w", encoding="utf-8"
        ) as stderr_file:
            proc = subprocess.Popen(
                cmd,
                cwd=cwd,
                env=env,
                stdout=stdout_file,
                stderr=stderr_file,
                text=True,
                encoding="utf-8",
                errors="replace",
            )
            if on_phase is not None:
                on_phase("launch_process")
                on_phase("wait_first_progress")
            saw_first_progress = False
            wait_first_progress_detail = "-"
            while proc.poll() is None:
                doc = load_json_doc(progress_path)
                if isinstance(doc, dict):
                    stage = str(doc.get("current_stage", "-")).strip() or "-"
                    if stage == "-":
                        stage = str(doc.get("last_completed_stage", "-")).strip() or "-"
                    if stage not in {"-", HELPER_PARENT_PENDING_STAGE} and not saw_first_progress:
                        saw_first_progress = True
                        if on_phase is not None:
                            on_phase("wait_exit")
                    elif stage == "-" and not saw_first_progress and on_phase is not None:
                        next_detail = "wait_first_progress.file_present_no_stage"
                        if wait_first_progress_detail != next_detail:
                            wait_first_progress_detail = next_detail
                            on_phase(next_detail)
                elif not saw_first_progress and on_phase is not None:
                    next_detail = "wait_first_progress.file_missing"
                    if wait_first_progress_detail != next_detail:
                        wait_first_progress_detail = next_detail
                        on_phase(next_detail)
                on_progress(doc)
                time.sleep(0.05)
            if on_phase is not None and not saw_first_progress:
                on_phase("wait_exit")
            if on_phase is not None:
                on_phase("collect_stdio")
        stdout = stdout_path.read_text(encoding="utf-8", errors="replace")
        stderr = stderr_path.read_text(encoding="utf-8", errors="replace")
    on_progress(load_json_doc(progress_path))
    return subprocess.CompletedProcess(cmd, int(proc.returncode or 0), stdout, stderr)


def write_progress_snapshot(
    path_text: str,
    *,
    status: str,
    current_section: str,
    last_completed_section: str,
    sections_completed: int,
    total_elapsed_ms: int,
) -> None:
    if not str(path_text).strip():
        return
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "ddn.ci.pipeline_emit_flags_progress.v1",
        "status": status,
        "current_section": current_section,
        "last_completed_section": last_completed_section,
        "sections_completed": sections_completed,
        "total_elapsed_ms": total_elapsed_ms,
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def require_tokens(text: str, label: str, tokens: list[str], errors: list[str]) -> None:
    for token in tokens:
        if token not in text:
            errors.append(f"{label}: missing token {token}")


def forbid_tokens(text: str, label: str, tokens: list[str], errors: list[str]) -> None:
    for token in tokens:
        if token in text:
            errors.append(f"{label}: forbidden token {token}")


def extract_aggregate_command_lines(text: str) -> list[str]:
    rows: list[str] = []
    for raw in text.splitlines():
        line = raw.strip()
        if "tests/run_ci_aggregate_gate.py" in line:
            rows.append(line)
    return rows


def require_tokens_in_all_lines(lines: list[str], label: str, tokens: list[str], errors: list[str]) -> None:
    if not lines:
        errors.append(f"{label}: missing aggregate gate invocation line")
        return
    for idx, line in enumerate(lines, start=1):
        for token in tokens:
            if token not in line:
                errors.append(f"{label}: line#{idx} missing token {token}")


def forbid_tokens_in_all_lines(lines: list[str], label: str, tokens: list[str], errors: list[str]) -> None:
    if not lines:
        return
    for idx, line in enumerate(lines, start=1):
        for token in tokens:
            if token in line:
                errors.append(f"{label}: line#{idx} forbidden token {token}")


def check_profile_matrix_selftest_gate_flags_runtime(
    root: Path,
    helper_path: Path,
    errors: list[str],
) -> None:
    if not helper_path.exists():
        errors.append(f"profile_matrix_selftest_gate.runtime: missing helper {helper_path}")
        return

    py = sys.executable
    default_helper_path = (root / PROFILE_MATRIX_SELFTEST_GATE_FLAGS_HELPER_SCRIPT).resolve()
    try:
        helper_resolved = helper_path.resolve()
    except Exception:
        helper_resolved = helper_path
    helper_is_default = helper_resolved == default_helper_path
    cases = [
        ("gitlab", "gitlab_default_off", {"CI_PIPELINE_SOURCE": "push", PROFILE_MATRIX_FULL_AGGREGATE_ENV_KEY: "0", PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY: "0"}, False, False),
        ("gitlab", "gitlab_full_aggregate_optin", {"CI_PIPELINE_SOURCE": "push", PROFILE_MATRIX_FULL_AGGREGATE_ENV_KEY: "1", PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY: "0"}, True, False),
        ("gitlab", "gitlab_full_real_smoke_optin", {"CI_PIPELINE_SOURCE": "push", PROFILE_MATRIX_FULL_AGGREGATE_ENV_KEY: "0", PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY: "1"}, False, True),
        ("gitlab", "gitlab_both_optin", {"CI_PIPELINE_SOURCE": "push", PROFILE_MATRIX_FULL_AGGREGATE_ENV_KEY: "1", PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY: "1"}, True, True),
        ("azure", "azure_default_off", {"BUILD_REASON": "IndividualCI", PROFILE_MATRIX_FULL_AGGREGATE_ENV_KEY: "0", PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY: "0"}, False, False),
        ("azure", "azure_full_aggregate_optin", {"BUILD_REASON": "Manual", PROFILE_MATRIX_FULL_AGGREGATE_ENV_KEY: "1", PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY: "0"}, True, False),
        ("azure", "azure_full_real_smoke_optin", {"BUILD_REASON": "Manual", PROFILE_MATRIX_FULL_AGGREGATE_ENV_KEY: "0", PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY: "1"}, False, True),
        ("azure", "azure_both_optin", {"BUILD_REASON": "Manual", PROFILE_MATRIX_FULL_AGGREGATE_ENV_KEY: "1", PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY: "1"}, True, True),
    ]
    if is_runtime_contract_minimal_mode() and cases:
        cases = [cases[0]]

    expected_timeout_keys = {
        "core_lang": PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_CORE_LANG,
        "full": PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_FULL,
        "seamgrim": PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_SEAMGRIM,
    }
    expected_timeout_defaults = {
        "core_lang": float(PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_CORE_LANG),
        "full": float(PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_FULL),
        "seamgrim": float(PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_SEAMGRIM),
    }

    for provider, label, env_patch, expected_full_aggregate, expected_full_real_smoke in cases:
        env = dict(os.environ)
        env.update(env_patch)
        expected_flags: list[str] = []
        if expected_full_aggregate:
            expected_flags.append(PROFILE_MATRIX_SELFTEST_FULL_AGGREGATE_FLAG)
        if expected_full_real_smoke:
            expected_flags.append(PROFILE_MATRIX_SELFTEST_FULL_REAL_SMOKE_FLAG)
        expected_flags_text = " ".join(expected_flags)
        expected_enabled = bool(expected_flags)
        use_fast_path = helper_is_default

        if use_fast_path:
            doc, json_out = run_profile_matrix_selftest_gate_fast_path(
                provider=provider,
                env_patch=env_patch,
                output_format="json",
            )
            json_proc = subprocess.CompletedProcess(
                [py, str(helper_path), "--provider", provider, "--format", "json"],
                0,
                json_out,
                "",
            )
        else:
            json_proc = run(
                [py, str(helper_path), "--provider", provider, "--format", "json"],
                cwd=root,
                env=env,
            )
        if json_proc.returncode != 0:
            errors.append(f"{label}.runtime_json: rc={json_proc.returncode}")
            continue
        if not use_fast_path:
            try:
                doc = json.loads(str(json_proc.stdout or "").strip())
            except Exception:
                errors.append(f"{label}.runtime_json: invalid json")
                continue
            if not isinstance(doc, dict):
                errors.append(f"{label}.runtime_json: payload not object")
                continue
        if str(doc.get("schema", "")).strip() != PROFILE_MATRIX_SELFTEST_GATE_FLAGS_POLICY_SCHEMA:
            errors.append(f"{label}.runtime_json: schema mismatch")
        if str(doc.get("provider", "")).strip() != provider:
            errors.append(f"{label}.runtime_json: provider mismatch")
        if str(doc.get("scope", "")).strip() != PROFILE_MATRIX_FULL_REAL_SMOKE_MODE:
            errors.append(f"{label}.runtime_json: scope mismatch")
        if str(doc.get("full_aggregate_env_key", "")).strip() != PROFILE_MATRIX_FULL_AGGREGATE_ENV_KEY:
            errors.append(f"{label}.runtime_json: full_aggregate_env_key mismatch")
        if str(doc.get("full_real_smoke_env_key", "")).strip() != PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY:
            errors.append(f"{label}.runtime_json: full_real_smoke_env_key mismatch")
        if str(doc.get("selftest_flags_env_key", "")).strip() != PROFILE_MATRIX_SELFTEST_GATE_FLAGS_VAR:
            errors.append(f"{label}.runtime_json: selftest_flags_env_key mismatch")
        if bool(doc.get("full_aggregate_enabled", False)) != expected_full_aggregate:
            errors.append(f"{label}.runtime_json: full_aggregate_enabled mismatch")
        if bool(doc.get("full_real_smoke_enabled", False)) != expected_full_real_smoke:
            errors.append(f"{label}.runtime_json: full_real_smoke_enabled mismatch")
        if bool(doc.get("enabled", False)) != expected_enabled:
            errors.append(f"{label}.runtime_json: enabled mismatch")
        if list(doc.get("flags", [])) != expected_flags:
            errors.append(f"{label}.runtime_json: flags mismatch")
        if str(doc.get("flags_text", "")).strip() != expected_flags_text:
            errors.append(f"{label}.runtime_json: flags_text mismatch")
        if str(doc.get("full_aggregate_raw", "")).strip() != str(env_patch.get(PROFILE_MATRIX_FULL_AGGREGATE_ENV_KEY, "")).strip():
            errors.append(f"{label}.runtime_json: full_aggregate_raw mismatch")
        if str(doc.get("full_real_smoke_raw", "")).strip() != str(env_patch.get(PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY, "")).strip():
            errors.append(f"{label}.runtime_json: full_real_smoke_raw mismatch")
        if dict(doc.get("step_timeout_env_keys", {})) != expected_timeout_keys:
            errors.append(f"{label}.runtime_json: step_timeout_env_keys mismatch")
        if dict(doc.get("step_timeout_defaults_sec", {})) != expected_timeout_defaults:
            errors.append(f"{label}.runtime_json: step_timeout_defaults_sec mismatch")
        if str(doc.get("step_timeout_defaults_text", "")).strip() != PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT:
            errors.append(f"{label}.runtime_json: step_timeout_defaults_text mismatch")

        if use_fast_path:
            _, shell_out = run_profile_matrix_selftest_gate_fast_path(
                provider=provider,
                env_patch=env_patch,
                output_format="shell",
            )
            shell_proc = subprocess.CompletedProcess(
                [py, str(helper_path), "--provider", provider, "--format", "shell"],
                0,
                shell_out,
                "",
            )
        else:
            shell_proc = run(
                [py, str(helper_path), "--provider", provider, "--format", "shell"],
                cwd=root,
                env=env,
            )
        if shell_proc.returncode != 0:
            errors.append(f"{label}.runtime_shell: rc={shell_proc.returncode}")
            continue
        shell_out = str(shell_proc.stdout or "")
        if f'export {PROFILE_MATRIX_SELFTEST_GATE_FLAGS_VAR}="{expected_flags_text}"' not in shell_out:
            errors.append(f"{label}.runtime_shell: export mismatch")
        if f"provider={provider}" not in shell_out:
            errors.append(f"{label}.runtime_shell: provider missing")
        if f"{PROFILE_MATRIX_SELFTEST_GATE_FLAGS_VAR}=${{{PROFILE_MATRIX_SELFTEST_GATE_FLAGS_VAR}:-}}" not in shell_out:
            errors.append(f"{label}.runtime_shell: flags env expansion missing")
        if f"step_timeout_defaults={PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT}" not in shell_out:
            errors.append(f"{label}.runtime_shell: step_timeout_defaults missing")

        if use_fast_path:
            _, text_out = run_profile_matrix_selftest_gate_fast_path(
                provider=provider,
                env_patch=env_patch,
                output_format="text",
            )
            text_proc = subprocess.CompletedProcess(
                [py, str(helper_path), "--provider", provider, "--format", "text"],
                0,
                text_out,
                "",
            )
        else:
            text_proc = run(
                [py, str(helper_path), "--provider", provider, "--format", "text"],
                cwd=root,
                env=env,
            )
        if text_proc.returncode != 0:
            errors.append(f"{label}.runtime_text: rc={text_proc.returncode}")
            continue
        text_out = str(text_proc.stdout or "")
        expected_status = "enabled" if expected_enabled else "disabled"
        if f"provider={provider}" not in text_out:
            errors.append(f"{label}.runtime_text: provider missing")
        if f"status={expected_status}" not in text_out:
            errors.append(f"{label}.runtime_text: status missing")
        if f"full_aggregate={str(expected_full_aggregate).lower()}" not in text_out:
            errors.append(f"{label}.runtime_text: full_aggregate missing")
        if f"full_real_smoke={str(expected_full_real_smoke).lower()}" not in text_out:
            errors.append(f"{label}.runtime_text: full_real_smoke missing")
        expected_flags_fragment = expected_flags_text if expected_flags_text else "-"
        if f"flags={expected_flags_fragment}" not in text_out:
            errors.append(f"{label}.runtime_text: flags missing")
        if f"step_timeout_defaults={PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT}" not in text_out:
            errors.append(f"{label}.runtime_text: step_timeout_defaults missing")


def check_profile_matrix_full_real_smoke_policy_runtime(
    root: Path,
    helper_path: Path,
    errors: list[str],
) -> None:
    if not helper_path.exists():
        errors.append(f"policy.runtime: missing helper {helper_path}")
        return

    py = sys.executable
    default_helper_path = (root / PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SCRIPT).resolve()
    try:
        helper_resolved = helper_path.resolve()
    except Exception:
        helper_resolved = helper_path
    helper_is_default = helper_resolved == default_helper_path
    cases = [
        ("gitlab", "gitlab_default_off", {"CI_PIPELINE_SOURCE": "push", PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY: "0"}, False, "default_off"),
        ("gitlab", "gitlab_schedule", {"CI_PIPELINE_SOURCE": "schedule", PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY: "0"}, True, "schedule"),
        ("gitlab", "gitlab_manual_optin", {"CI_PIPELINE_SOURCE": "web", PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY: "1"}, True, "manual_optin"),
        ("azure", "azure_default_off", {"BUILD_REASON": "IndividualCI", PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY: "0"}, False, "default_off"),
        ("azure", "azure_schedule", {"BUILD_REASON": "Schedule", PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY: "0"}, True, "schedule"),
        ("azure", "azure_manual_optin", {"BUILD_REASON": "Manual", PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY: "1"}, True, "manual_optin"),
    ]
    if is_runtime_contract_minimal_mode() and cases:
        cases = [cases[0]]

    for provider, label, env_patch, expected_enabled, expected_reason in cases:
        env = dict(os.environ)
        env.update(env_patch)
        use_fast_path = helper_is_default

        if use_fast_path:
            doc, json_out = run_profile_matrix_policy_fast_path(
                provider=provider,
                env_patch=env_patch,
                output_format="json",
            )
            json_proc = subprocess.CompletedProcess(
                [py, str(helper_path), "--provider", provider, "--format", "json"],
                0,
                json_out,
                "",
            )
        else:
            json_proc = run(
                [py, str(helper_path), "--provider", provider, "--format", "json"],
                cwd=root,
                env=env,
            )
        if json_proc.returncode != 0:
            errors.append(f"{label}.runtime_json: rc={json_proc.returncode}")
            continue
        if not use_fast_path:
            try:
                doc = json.loads(str(json_proc.stdout or "").strip())
            except Exception:
                errors.append(f"{label}.runtime_json: invalid json")
                continue
            if not isinstance(doc, dict):
                errors.append(f"{label}.runtime_json: payload not object")
                continue
        if str(doc.get("schema", "")).strip() != PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SCHEMA:
            errors.append(f"{label}.runtime_json: schema mismatch")
        if str(doc.get("provider", "")).strip() != provider:
            errors.append(f"{label}.runtime_json: provider mismatch")
        if str(doc.get("env_key", "")).strip() != PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY:
            errors.append(f"{label}.runtime_json: env_key mismatch")
        if str(doc.get("scope", "")).strip() != PROFILE_MATRIX_FULL_REAL_SMOKE_MODE:
            errors.append(f"{label}.runtime_json: scope mismatch")
        if bool(doc.get("enabled", False)) != expected_enabled:
            errors.append(f"{label}.runtime_json: enabled mismatch")
        if str(doc.get("reason", "")).strip() != expected_reason:
            errors.append(f"{label}.runtime_json: reason mismatch")
        expected_timeout_env_keys = {
            "core_lang": PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_CORE_LANG,
            "full": PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_FULL,
            "seamgrim": PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_SEAMGRIM,
        }
        timeout_env_keys = doc.get("step_timeout_env_keys")
        if not isinstance(timeout_env_keys, dict):
            errors.append(f"{label}.runtime_json: step_timeout_env_keys missing")
        elif timeout_env_keys != expected_timeout_env_keys:
            errors.append(f"{label}.runtime_json: step_timeout_env_keys mismatch")
        expected_timeout_defaults = {
            "core_lang": float(PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_CORE_LANG),
            "full": float(PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_FULL),
            "seamgrim": float(PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_SEAMGRIM),
        }
        timeout_defaults = doc.get("step_timeout_defaults_sec")
        if not isinstance(timeout_defaults, dict):
            errors.append(f"{label}.runtime_json: step_timeout_defaults_sec missing")
        elif timeout_defaults != expected_timeout_defaults:
            errors.append(f"{label}.runtime_json: step_timeout_defaults_sec mismatch")
        if str(doc.get("step_timeout_defaults_text", "")).strip() != PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT:
            errors.append(f"{label}.runtime_json: step_timeout_defaults_text mismatch")

        if use_fast_path:
            _, shell_out = run_profile_matrix_policy_fast_path(
                provider=provider,
                env_patch=env_patch,
                output_format="shell",
            )
            shell_proc = subprocess.CompletedProcess(
                [py, str(helper_path), "--provider", provider, "--format", "shell"],
                0,
                shell_out,
                "",
            )
        else:
            shell_proc = run(
                [py, str(helper_path), "--provider", provider, "--format", "shell"],
                cwd=root,
                env=env,
            )
        if shell_proc.returncode != 0:
            errors.append(f"{label}.runtime_shell: rc={shell_proc.returncode}")
            continue
        shell_text = str(shell_proc.stdout or "")
        expected_export = f"export {PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY}={'1' if expected_enabled else '0'}"
        if expected_export not in shell_text:
            errors.append(f"{label}.runtime_shell: export mismatch")
        expected_timeout_exports = [
            f"export {PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_CORE_LANG}={PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_CORE_LANG:g}",
            f"export {PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_FULL}={PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_FULL:g}",
            f"export {PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_SEAMGRIM}={PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_SEAMGRIM:g}",
        ]
        for timeout_export in expected_timeout_exports:
            if timeout_export not in shell_text:
                errors.append(f"{label}.runtime_shell: timeout export missing")
        if PROFILE_MATRIX_FULL_REAL_SMOKE_SCOPE_FRAGMENT not in shell_text:
            errors.append(f"{label}.runtime_shell: scope missing")

        if use_fast_path:
            _, text_out = run_profile_matrix_policy_fast_path(
                provider=provider,
                env_patch=env_patch,
                output_format="text",
            )
            text_proc = subprocess.CompletedProcess(
                [py, str(helper_path), "--provider", provider, "--format", "text"],
                0,
                text_out,
                "",
            )
        else:
            text_proc = run(
                [py, str(helper_path), "--provider", provider, "--format", "text"],
                cwd=root,
                env=env,
            )
        if text_proc.returncode != 0:
            errors.append(f"{label}.runtime_text: rc={text_proc.returncode}")
            continue
        text_out = str(text_proc.stdout or "")
        expected_status = "enabled" if expected_enabled else "disabled"
        if f"provider={provider}" not in text_out:
            errors.append(f"{label}.runtime_text: provider missing")
        if f"status={expected_status}" not in text_out:
            errors.append(f"{label}.runtime_text: status missing")
        if f"reason={expected_reason}" not in text_out:
            errors.append(f"{label}.runtime_text: reason missing")
        if PROFILE_MATRIX_FULL_REAL_SMOKE_SCOPE_FRAGMENT not in text_out:
            errors.append(f"{label}.runtime_text: scope missing")
        if f"step_timeout_defaults={PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT}" not in text_out:
            errors.append(f"{label}.runtime_text: step_timeout_defaults missing")


def check_age5_combined_heavy_policy_runtime(
    root: Path,
    helper_path: Path,
    errors: list[str],
    progress_hook=None,
) -> None:
    if not helper_path.exists():
        errors.append(f"age5_combined.runtime: missing helper {helper_path}")
        return

    py = sys.executable
    default_helper_path = (root / AGE5_COMBINED_HEAVY_POLICY_SCRIPT).resolve()
    try:
        helper_resolved = helper_path.resolve()
    except Exception:
        helper_resolved = helper_path
    helper_is_default = helper_resolved == default_helper_path
    cases = [
        ("gitlab", "gitlab_default_off", {"CI_PIPELINE_SOURCE": "push", AGE5_COMBINED_HEAVY_ENV_KEY: "0"}, False, "default_off"),
        ("gitlab", "gitlab_schedule", {"CI_PIPELINE_SOURCE": "schedule", AGE5_COMBINED_HEAVY_ENV_KEY: "0"}, True, "schedule"),
        ("gitlab", "gitlab_manual_optin", {"CI_PIPELINE_SOURCE": "web", AGE5_COMBINED_HEAVY_ENV_KEY: "1"}, True, "manual_optin"),
        ("azure", "azure_default_off", {"BUILD_REASON": "IndividualCI", AGE5_COMBINED_HEAVY_ENV_KEY: "0"}, False, "default_off"),
        ("azure", "azure_schedule", {"BUILD_REASON": "Schedule", AGE5_COMBINED_HEAVY_ENV_KEY: "0"}, True, "schedule"),
        ("azure", "azure_manual_optin", {"BUILD_REASON": "Manual", AGE5_COMBINED_HEAVY_ENV_KEY: "1"}, True, "manual_optin"),
    ]
    if is_runtime_contract_minimal_mode() and cases:
        cases = [cases[0]]

    for provider, label, env_patch, expected_enabled, expected_reason in cases:
        env = dict(os.environ)
        env.update(env_patch)
        progress_dir = Path(tempfile.mkdtemp(prefix="age5_combined_policy_runtime_"))
        json_progress_path = progress_dir / f"{provider}_{label}.json.progress.detjson"
        shell_progress_path = progress_dir / f"{provider}_{label}.shell.progress.detjson"
        text_progress_path = progress_dir / f"{provider}_{label}.text.progress.detjson"
        use_fast_path = (
            helper_is_default
            and expected_reason in AGE5_COMBINED_POLICY_FAST_PATH_REASONS
        )

        def helper_progress(prefix: str, doc: dict[str, object] | None) -> None:
            if progress_hook is None or not isinstance(doc, dict):
                return
            stage = str(doc.get("current_stage", "-")).strip() or "-"
            if stage == "-":
                stage = str(doc.get("last_completed_stage", "-")).strip() or "-"
            if stage != "-":
                progress_hook(f"{prefix}.helper.{stage}")

        if use_fast_path:
            doc, json_stdout = run_age5_combined_policy_fast_path(
                provider=provider,
                env_patch=env_patch,
                output_format="json",
                progress_hook=progress_hook,
                progress_prefix=f"age5_combined_policy_runtime.json.{label}",
            )
            json_proc = subprocess.CompletedProcess(
                [py, "-S", str(helper_path), "--provider", provider, "--format", "json"],
                0,
                json_stdout,
                "",
            )
        else:
            json_env = dict(env)
            json_env["DDN_AGE5_COMBINED_HEAVY_POLICY_PROGRESS_JSON"] = str(json_progress_path)
            write_helper_seed_progress(
                json_progress_path,
                provider=provider,
                output_format="json",
            )
            json_proc = run_with_progress(
                [py, "-S", str(helper_path), "--provider", provider, "--format", "json"],
                cwd=root,
                env=json_env,
                progress_path=json_progress_path,
                on_progress=lambda doc: helper_progress(
                    f"age5_combined_policy_runtime.json.{label}",
                    doc,
                ),
                on_phase=(
                    (lambda phase: progress_hook(f"age5_combined_policy_runtime.json.{label}.{phase}"))
                    if progress_hook is not None
                    else None
                ),
            )
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.json.{label}.collect_output")
        if json_proc.returncode != 0:
            errors.append(f"{label}.age5_runtime_json: rc={json_proc.returncode}")
            continue
        if not use_fast_path:
            try:
                doc = json.loads(str(json_proc.stdout or "").strip())
            except Exception:
                errors.append(f"{label}.age5_runtime_json: invalid json")
                continue
            if not isinstance(doc, dict):
                errors.append(f"{label}.age5_runtime_json: payload not object")
                continue
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.json.{label}.validate_contract.core_fields")
        if str(doc.get("schema", "")).strip() != AGE5_COMBINED_HEAVY_POLICY_SCHEMA:
            errors.append(f"{label}.age5_runtime_json: schema mismatch")
        if str(doc.get("provider", "")).strip() != provider:
            errors.append(f"{label}.age5_runtime_json: provider mismatch")
        if str(doc.get("env_key", "")).strip() != AGE5_COMBINED_HEAVY_ENV_KEY:
            errors.append(f"{label}.age5_runtime_json: env_key mismatch")
        if str(doc.get("scope", "")).strip() != AGE5_COMBINED_HEAVY_MODE:
            errors.append(f"{label}.age5_runtime_json: scope mismatch")
        if str(doc.get("combined_report_schema", "")).strip() != AGE5_COMBINED_HEAVY_REPORT_SCHEMA:
            errors.append(f"{label}.age5_runtime_json: combined_report_schema mismatch")
        if list(doc.get("combined_required_reports", [])) != list(AGE5_COMBINED_HEAVY_REQUIRED_REPORTS):
            errors.append(f"{label}.age5_runtime_json: combined_required_reports mismatch")
        if list(doc.get("combined_required_criteria", [])) != list(AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA):
            errors.append(f"{label}.age5_runtime_json: combined_required_criteria mismatch")
        if list(doc.get("combined_child_summary_keys", [])) != list(AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS):
            errors.append(f"{label}.age5_runtime_json: combined_child_summary_keys mismatch")
        if str(doc.get("combined_child_summary_keys_text", "")).strip() != AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS_TEXT:
            errors.append(f"{label}.age5_runtime_json: combined_child_summary_keys_text mismatch")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.json.{label}.validate_contract.child_summary")
        if dict(doc.get("combined_child_summary_default_fields", {})) != dict(
            AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS
        ):
            errors.append(f"{label}.age5_runtime_json: combined_child_summary_default_fields mismatch")
        if str(doc.get("combined_child_summary_default_fields_text", "")).strip() != (
            AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT
        ):
            errors.append(f"{label}.age5_runtime_json: combined_child_summary_default_fields_text mismatch")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.json.{label}.validate_contract.timeout_policy")
        if dict(doc.get("combined_timeout_policy_fields", {})) != dict(
            build_age5_combined_heavy_timeout_policy_fields()
        ):
            errors.append(f"{label}.age5_runtime_json: combined_timeout_policy_fields mismatch")
        if str(doc.get("combined_timeout_policy_fields_text", "")).strip() != (
            AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_FIELDS_TEXT
        ):
            errors.append(f"{label}.age5_runtime_json: combined_timeout_policy_fields_text mismatch")
        if str(doc.get(AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DEFAULT_KEY, "")).strip() != (
            AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED
        ):
            errors.append(f"{label}.age5_runtime_json: combined_timeout_mode_default mismatch")
        if str(doc.get(AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_KEY, "")).strip() != (
            AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_TEXT
        ):
            errors.append(f"{label}.age5_runtime_json: combined_timeout_mode_allowed_values mismatch")
        if str(doc.get(AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_KEY, "")).strip() != (
            AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_DEFAULT
        ):
            errors.append(f"{label}.age5_runtime_json: combined_timeout_mode_preview_only mismatch")
        if str(doc.get(AGE5_COMBINED_HEAVY_TIMEOUT_MODE_SCOPE_KEY, "")).strip() != AGE5_COMBINED_HEAVY_MODE:
            errors.append(f"{label}.age5_runtime_json: combined_timeout_mode_scope mismatch")
        if str(doc.get(AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_KEY, "")).strip() != (
            AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_DEFAULT
        ):
            errors.append(f"{label}.age5_runtime_json: combined_timeout_requires_optin mismatch")
        if str(doc.get(AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_KEY, "")).strip() != (
            AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT
        ):
            errors.append(f"{label}.age5_runtime_json: combined_timeout_policy_reason mismatch")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.json.{label}.validate_contract.age4_snapshot")
        expected_age4_proof_snapshot = build_age4_proof_snapshot()
        expected_age4_proof_source_snapshot = build_age4_proof_source_snapshot_fields(
            top_snapshot=expected_age4_proof_snapshot
        )
        if str(doc.get("age4_proof_snapshot_fields_text", "")).strip() != AGE4_PROOF_SNAPSHOT_FIELDS_TEXT:
            errors.append(f"{label}.age5_runtime_json: age4_proof_snapshot_fields_text mismatch")
        if str(doc.get("age4_proof_source_snapshot_fields_text", "")).strip() != AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT:
            errors.append(f"{label}.age5_runtime_json: age4_proof_source_snapshot_fields_text mismatch")
        if str(doc.get("age4_proof_snapshot_text", "")).strip() != (
            build_age4_proof_snapshot_text(expected_age4_proof_snapshot)
        ):
            errors.append(f"{label}.age5_runtime_json: age4_proof_snapshot_text mismatch")
        for key, expected in expected_age4_proof_snapshot.items():
            if str(doc.get(key, "")).strip() != str(expected):
                errors.append(f"{label}.age5_runtime_json: {key} mismatch")
        for key, expected in expected_age4_proof_source_snapshot.items():
            if str(doc.get(key, "")).strip() != str(expected):
                errors.append(f"{label}.age5_runtime_json: {key} mismatch")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.json.{label}.validate_contract.digest_defaults")
        if str(doc.get(AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY, "")).strip() != AGE5_CLOSE_DIGEST_SELFTEST_OK_DEFAULT:
            errors.append(f"{label}.age5_runtime_json: digest selftest default mismatch")
        if dict(doc.get("combined_digest_selftest_default_field", {})) != {
            AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_DEFAULT
        }:
            errors.append(f"{label}.age5_runtime_json: combined_digest_selftest_default_field mismatch")
        if str(doc.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip() != AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT:
            errors.append(f"{label}.age5_runtime_json: combined_digest_selftest_default_field_text mismatch")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.json.{label}.validate_contract.summary_transport")
        if dict(doc.get("combined_child_summary_default_text_transport_fields", {})) != dict(
            AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS
        ):
            errors.append(f"{label}.age5_runtime_json: combined_child_summary_default_text_transport_fields mismatch")
        if str(doc.get("combined_child_summary_default_text_transport_fields_text", "")).strip() != (
            AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS_TEXT
        ):
            errors.append(
                f"{label}.age5_runtime_json: combined_child_summary_default_text_transport_fields_text mismatch"
            )
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.json.{label}.validate_contract.report_contract")
        if dict(doc.get("combined_contract_summary_fields", {})) != dict(AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS):
            errors.append(f"{label}.age5_runtime_json: combined_contract_summary_fields mismatch")
        if str(doc.get("combined_contract_summary_fields_text", "")).strip() != AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT:
            errors.append(f"{label}.age5_runtime_json: combined_contract_summary_fields_text mismatch")
        if dict(doc.get("combined_full_summary_contract_fields", {})) != dict(AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS):
            errors.append(f"{label}.age5_runtime_json: combined_full_summary_contract_fields mismatch")
        if str(doc.get("combined_full_summary_contract_fields_text", "")).strip() != AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT:
            errors.append(f"{label}.age5_runtime_json: combined_full_summary_contract_fields_text mismatch")
        if dict(doc.get("combined_full_summary_text_transport_fields", {})) != dict(
            AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FIELDS
        ):
            errors.append(f"{label}.age5_runtime_json: combined_full_summary_text_transport_fields mismatch")
        if str(doc.get("combined_full_summary_text_transport_fields_text", "")).strip() != (
            AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FIELDS_TEXT
        ):
            errors.append(f"{label}.age5_runtime_json: combined_full_summary_text_transport_fields_text mismatch")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.json.{label}.validate_contract.result.enabled")
        if bool(doc.get("enabled", False)) != expected_enabled:
            errors.append(f"{label}.age5_runtime_json: enabled mismatch")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.json.{label}.validate_contract.result.reason")
        if str(doc.get("reason", "")).strip() != expected_reason:
            errors.append(f"{label}.age5_runtime_json: reason mismatch")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.json.{label}.validate_contract.result.done")

        if use_fast_path:
            _, shell_text = run_age5_combined_policy_fast_path(
                provider=provider,
                env_patch=env_patch,
                output_format="shell",
                progress_hook=progress_hook,
                progress_prefix=f"age5_combined_policy_runtime.shell.{label}",
            )
            shell_proc = subprocess.CompletedProcess(
                [py, "-S", str(helper_path), "--provider", provider, "--format", "shell"],
                0,
                shell_text,
                "",
            )
        else:
            shell_env = dict(env)
            shell_env["DDN_AGE5_COMBINED_HEAVY_POLICY_PROGRESS_JSON"] = str(shell_progress_path)
            write_helper_seed_progress(
                shell_progress_path,
                provider=provider,
                output_format="shell",
            )
            shell_proc = run_with_progress(
                [py, "-S", str(helper_path), "--provider", provider, "--format", "shell"],
                cwd=root,
                env=shell_env,
                progress_path=shell_progress_path,
                on_progress=lambda doc: helper_progress(
                    f"age5_combined_policy_runtime.shell.{label}",
                    doc,
                ),
                on_phase=(
                    (lambda phase: progress_hook(f"age5_combined_policy_runtime.shell.{label}.{phase}"))
                    if progress_hook is not None
                    else None
                ),
            )
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.shell.{label}.collect_output")
        if shell_proc.returncode != 0:
            errors.append(f"{label}.age5_runtime_shell: rc={shell_proc.returncode}")
            continue
        shell_text = str(shell_proc.stdout or "")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.shell.{label}.validate_export_marker")
        expected_export = f"export {AGE5_COMBINED_HEAVY_ENV_KEY}={'1' if expected_enabled else '0'}"
        if expected_export not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: export mismatch")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.shell.{label}.validate_policy_fragments")
        if AGE5_COMBINED_HEAVY_SCOPE_FRAGMENT not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: scope missing")
        if AGE5_COMBINED_HEAVY_REPORTS_FRAGMENT not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: reports fragment missing")
        if AGE5_COMBINED_HEAVY_CRITERIA_FRAGMENT not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: criteria fragment missing")
        if AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS_FRAGMENT not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: child summary keys fragment missing")
        if AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_FRAGMENT not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: child summary default fields fragment missing")
        if AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DEFAULT_FRAGMENT not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: timeout mode default fragment missing")
        if AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_FRAGMENT not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: timeout mode allowed values fragment missing")
        if AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_FRAGMENT not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: timeout mode preview only fragment missing")
        if AGE5_COMBINED_HEAVY_TIMEOUT_MODE_SCOPE_FRAGMENT not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: timeout mode scope fragment missing")
        if AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_FRAGMENT not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: timeout requires optin fragment missing")
        if AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_FRAGMENT not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: timeout policy reason fragment missing")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.shell.{label}.validate_age4_snapshot")
        if f"age4_proof_snapshot_fields_text={AGE4_PROOF_SNAPSHOT_FIELDS_TEXT}" not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: age4_proof_snapshot_fields_text missing")
        if f"age4_proof_source_snapshot_fields_text={AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT}" not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: age4_proof_source_snapshot_fields_text missing")
        if (
            "age4_proof_snapshot_text="
            + build_age4_proof_snapshot_text(build_age4_proof_snapshot())
        ) not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: age4_proof_snapshot_text missing")
        if "age4_proof_gate_result_snapshot_present=0" not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: age4_proof_gate_result_snapshot_present missing")
        if "age4_proof_gate_result_snapshot_parity=0" not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: age4_proof_gate_result_snapshot_parity missing")
        if "age4_proof_final_status_parse_snapshot_present=0" not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: age4_proof_final_status_parse_snapshot_present missing")
        if "age4_proof_final_status_parse_snapshot_parity=0" not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: age4_proof_final_status_parse_snapshot_parity missing")
        if AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: digest selftest default fragment missing")
        if AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: digest selftest default field fragment missing")
        if AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_FRAGMENT not in shell_text:
            errors.append(f"{label}.age5_runtime_shell: digest selftest default text fragment missing")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.shell.{label}.validate_summary_fragments.child_summary_transport")
        for fragment in AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FRAGMENTS:
            if fragment not in shell_text:
                errors.append(f"{label}.age5_runtime_shell: child summary default transport fragment missing {fragment}")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.shell.{label}.validate_summary_fragments.combined_contract")
        for fragment in AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FRAGMENTS:
            if fragment not in shell_text:
                errors.append(f"{label}.age5_runtime_shell: contract fragment missing {fragment}")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.shell.{label}.validate_summary_fragments.full_summary")
        for fragment in AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FRAGMENTS:
            if fragment not in shell_text:
                errors.append(f"{label}.age5_runtime_shell: full summary fragment missing {fragment}")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.shell.{label}.validate_summary_fragments.full_summary_transport")
        for fragment in AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FRAGMENTS:
            if fragment not in shell_text:
                errors.append(f"{label}.age5_runtime_shell: full summary transport fragment missing {fragment}")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.shell.{label}.validate_summary_fragments.done")

        if use_fast_path:
            _, text_out = run_age5_combined_policy_fast_path(
                provider=provider,
                env_patch=env_patch,
                output_format="text",
                progress_hook=progress_hook,
                progress_prefix=f"age5_combined_policy_runtime.text.{label}",
            )
            text_proc = subprocess.CompletedProcess(
                [py, "-S", str(helper_path), "--provider", provider, "--format", "text"],
                0,
                text_out,
                "",
            )
        else:
            text_env = dict(env)
            text_env["DDN_AGE5_COMBINED_HEAVY_POLICY_PROGRESS_JSON"] = str(text_progress_path)
            write_helper_seed_progress(
                text_progress_path,
                provider=provider,
                output_format="text",
            )
            text_proc = run_with_progress(
                [py, "-S", str(helper_path), "--provider", provider, "--format", "text"],
                cwd=root,
                env=text_env,
                progress_path=text_progress_path,
                on_progress=lambda doc: helper_progress(
                    f"age5_combined_policy_runtime.text.{label}",
                    doc,
                ),
                on_phase=(
                    (lambda phase: progress_hook(f"age5_combined_policy_runtime.text.{label}.{phase}"))
                    if progress_hook is not None
                    else None
                ),
            )
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.text.{label}.collect_output")
        if text_proc.returncode != 0:
            errors.append(f"{label}.age5_runtime_text: rc={text_proc.returncode}")
            continue
        text_out = str(text_proc.stdout or "")
        expected_status = "enabled" if expected_enabled else "disabled"
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.text.{label}.validate_policy_fragments")
        if f"provider={provider}" not in text_out:
            errors.append(f"{label}.age5_runtime_text: provider missing")
        if f"status={expected_status}" not in text_out:
            errors.append(f"{label}.age5_runtime_text: status missing")
        if f"reason={expected_reason}" not in text_out:
            errors.append(f"{label}.age5_runtime_text: reason missing")
        if AGE5_COMBINED_HEAVY_SCOPE_FRAGMENT not in text_out:
            errors.append(f"{label}.age5_runtime_text: scope missing")
        if AGE5_COMBINED_HEAVY_REPORTS_FRAGMENT not in text_out:
            errors.append(f"{label}.age5_runtime_text: reports fragment missing")
        if AGE5_COMBINED_HEAVY_CRITERIA_FRAGMENT not in text_out:
            errors.append(f"{label}.age5_runtime_text: criteria fragment missing")
        if AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS_FRAGMENT not in text_out:
            errors.append(f"{label}.age5_runtime_text: child summary keys fragment missing")
        if AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_FRAGMENT not in text_out:
            errors.append(f"{label}.age5_runtime_text: child summary default fields fragment missing")
        if AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DEFAULT_FRAGMENT not in text_out:
            errors.append(f"{label}.age5_runtime_text: timeout mode default fragment missing")
        if AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_FRAGMENT not in text_out:
            errors.append(f"{label}.age5_runtime_text: timeout mode allowed values fragment missing")
        if AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_FRAGMENT not in text_out:
            errors.append(f"{label}.age5_runtime_text: timeout mode preview only fragment missing")
        if AGE5_COMBINED_HEAVY_TIMEOUT_MODE_SCOPE_FRAGMENT not in text_out:
            errors.append(f"{label}.age5_runtime_text: timeout mode scope fragment missing")
        if AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_FRAGMENT not in text_out:
            errors.append(f"{label}.age5_runtime_text: timeout requires optin fragment missing")
        if AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_FRAGMENT not in text_out:
            errors.append(f"{label}.age5_runtime_text: timeout policy reason fragment missing")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.text.{label}.validate_age4_snapshot")
        if f"age4_proof_snapshot_fields_text={AGE4_PROOF_SNAPSHOT_FIELDS_TEXT}" not in text_out:
            errors.append(f"{label}.age5_runtime_text: age4_proof_snapshot_fields_text missing")
        if f"age4_proof_source_snapshot_fields_text={AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT}" not in text_out:
            errors.append(f"{label}.age5_runtime_text: age4_proof_source_snapshot_fields_text missing")
        if (
            "age4_proof_snapshot_text="
            + build_age4_proof_snapshot_text(build_age4_proof_snapshot())
        ) not in text_out:
            errors.append(f"{label}.age5_runtime_text: age4_proof_snapshot_text missing")
        if "age4_proof_gate_result_snapshot_present=0" not in text_out:
            errors.append(f"{label}.age5_runtime_text: age4_proof_gate_result_snapshot_present missing")
        if "age4_proof_gate_result_snapshot_parity=0" not in text_out:
            errors.append(f"{label}.age5_runtime_text: age4_proof_gate_result_snapshot_parity missing")
        if "age4_proof_final_status_parse_snapshot_present=0" not in text_out:
            errors.append(f"{label}.age5_runtime_text: age4_proof_final_status_parse_snapshot_present missing")
        if "age4_proof_final_status_parse_snapshot_parity=0" not in text_out:
            errors.append(f"{label}.age5_runtime_text: age4_proof_final_status_parse_snapshot_parity missing")
        if AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT not in text_out:
            errors.append(f"{label}.age5_runtime_text: digest selftest default fragment missing")
        if AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT not in text_out:
            errors.append(f"{label}.age5_runtime_text: digest selftest default field fragment missing")
        if AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_FRAGMENT not in text_out:
            errors.append(f"{label}.age5_runtime_text: digest selftest default text fragment missing")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.text.{label}.validate_summary_fragments.child_summary_transport")
        for fragment in AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FRAGMENTS:
            if fragment not in text_out:
                errors.append(f"{label}.age5_runtime_text: child summary default transport fragment missing {fragment}")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.text.{label}.validate_summary_fragments.combined_contract")
        for fragment in AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FRAGMENTS:
            if fragment not in text_out:
                errors.append(f"{label}.age5_runtime_text: contract fragment missing {fragment}")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.text.{label}.validate_summary_fragments.full_summary")
        for fragment in AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FRAGMENTS:
            if fragment not in text_out:
                errors.append(f"{label}.age5_runtime_text: full summary fragment missing {fragment}")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.text.{label}.validate_summary_fragments.full_summary_transport")
        for fragment in AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FRAGMENTS:
            if fragment not in text_out:
                errors.append(f"{label}.age5_runtime_text: full summary transport fragment missing {fragment}")
        if progress_hook is not None:
            progress_hook(f"age5_combined_policy_runtime.text.{label}.validate_summary_fragments.done")


def main() -> int:
    parser = argparse.ArgumentParser(description="Check CI pipeline scripts keep required aggregate/emitter flags")
    parser.add_argument("--gitlab", default=".gitlab-ci.yml", help="path to .gitlab-ci.yml")
    parser.add_argument("--azure", default="azure-pipelines.yml", help="path to azure-pipelines.yml")
    parser.add_argument(
        "--profile-matrix-full-real-smoke-helper",
        default=PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SCRIPT,
        help="path to the profile-matrix full-real smoke policy helper script",
    )
    parser.add_argument(
        "--profile-matrix-selftest-gate-helper",
        default=PROFILE_MATRIX_SELFTEST_GATE_FLAGS_HELPER_SCRIPT,
        help="path to the profile-matrix selftest gate helper script",
    )
    parser.add_argument(
        "--age5-combined-heavy-helper",
        default=AGE5_COMBINED_HEAVY_POLICY_SCRIPT,
        help="path to the AGE5 combined heavy policy helper script",
    )
    args = parser.parse_args()
    progress_path = str(os.environ.get(PROGRESS_ENV_KEY, "")).strip()
    started = time.perf_counter()
    last_completed_section = "-"

    def update_progress(status: str, current_section: str) -> None:
        completed = [] if last_completed_section == "-" else [item for item in last_completed_section.split(",") if item]
        write_progress_snapshot(
            progress_path,
            status=status,
            current_section=current_section,
            last_completed_section=last_completed_section,
            sections_completed=len(completed),
            total_elapsed_ms=int(round((time.perf_counter() - started) * 1000.0)),
        )

    def complete_section(section: str) -> None:
        nonlocal last_completed_section
        completed = [] if last_completed_section == "-" else [item for item in last_completed_section.split(",") if item]
        completed.append(section)
        last_completed_section = ",".join(completed)
        write_progress_snapshot(
            progress_path,
            status="running",
            current_section="-",
            last_completed_section=last_completed_section,
            sections_completed=len(completed),
            total_elapsed_ms=int(round((time.perf_counter() - started) * 1000.0)),
        )

    def fail_with_errors() -> int:
        write_progress_snapshot(
            progress_path,
            status="fail",
            current_section="-",
            last_completed_section=last_completed_section,
            sections_completed=(
                0 if last_completed_section == "-" else len([item for item in last_completed_section.split(",") if item])
            ),
            total_elapsed_ms=int(round((time.perf_counter() - started) * 1000.0)),
        )
        print("[ci-pipeline-emit-flags-check] detected issues:")
        for row in errors[:24]:
            print(f" - {row}")
        return 1

    gitlab_path = Path(args.gitlab)
    azure_path = Path(args.azure)
    helper_path = Path(args.profile_matrix_full_real_smoke_helper)
    selftest_gate_helper_path = Path(args.profile_matrix_selftest_gate_helper)
    age5_helper_path = Path(args.age5_combined_heavy_helper)
    if not gitlab_path.exists():
        return fail(f"missing file: {gitlab_path}")
    if not azure_path.exists():
        return fail(f"missing file: {azure_path}")

    gitlab_text = load_text(gitlab_path)
    azure_text = load_text(azure_path)
    errors: list[str] = []

    aggregate_tokens = [
        "tests/run_ci_aggregate_gate.py",
        "--backup-hygiene",
        "--quiet-success-logs",
        "--compact-step-logs",
        "--step-log-dir build/reports",
        "--step-log-failed-only",
        "--checklist-skip-seed-cli",
        "--checklist-skip-ui-common",
        "--runtime-5min-skip-ui-common",
        "--runtime-5min-skip-showcase-check",
        "--fixed64-threeway-max-report-age-minutes",
        f"${PROFILE_MATRIX_SELFTEST_GATE_FLAGS_VAR}",
    ]
    fixed64_threeway_tokens = [
        "DDN_ENABLE_DARWIN_PROBE",
        'DDN_ENABLE_DARWIN_PROBE: "0"',
        "DDN_DARWIN_PROBE_ARCHIVE_KEEP",
        "DDN_FIXED64_THREEWAY_MAX_AGE_MINUTES",
        "DDN_DARWIN_PROBE_SCHEDULE_INTERVAL_MINUTES",
        "tests/run_fixed64_darwin_probe_artifact.py",
        "tests/run_fixed64_darwin_probe_schedule_policy_check.py",
        "--require-darwin",
        "--archive-dir build/reports/darwin_probe_archive",
        "--archive-keep",
        "--max-age-minutes",
        "--schedule-interval-minutes",
        "--json-out build/reports/fixed64_darwin_probe_schedule_policy.detjson",
        "fixed64_darwin_probe_artifact.detjson",
        "tools/scripts/resolve_fixed64_threeway_inputs.py",
        "--json-out build/reports/fixed64_threeway_inputs.detjson",
        "--strict-invalid",
        "--require-when-env DDN_ENABLE_DARWIN_PROBE",
        "DDN_REQUIRE_FIXED64_3WAY",
        "--require-fixed64-3way",
        "fixed64_cross_platform_probe_darwin.detjson",
    ]
    azure_darwin_guard_tokens = [
        "[ci-fixed64-darwin] DDN_ENABLE_DARWIN_PROBE=1 requires darwin agent",
        "uname -s",
    ]
    gitlab_darwin_guard_tokens = [
        "[ci-fixed64-darwin] DDN_ENABLE_DARWIN_PROBE=1 requires darwin agent",
        "uname -s",
    ]
    emit_tokens = [
        "tools/scripts/emit_ci_final_line.py",
        "--print-artifacts",
        "--print-failure-digest 6",
        "--print-failure-tail-lines 20",
        "--fail-on-summary-verify-error",
        "--failure-brief-out build/reports/__PREFIX__.ci_fail_brief.txt",
        "--triage-json-out build/reports/__PREFIX__.ci_fail_triage.detjson",
    ]
    emit_require_tokens = [
        "tools/scripts/emit_ci_final_line.py",
        "--require-final-line",
        "--fail-on-summary-verify-error",
        "--failure-brief-out build/reports/__PREFIX__.ci_fail_brief.txt",
        "--triage-json-out build/reports/__PREFIX__.ci_fail_triage.detjson",
    ]
    emit_artifacts_check_tokens = [
        "tests/run_ci_emit_artifacts_check.py",
        "--report-dir build/reports",
        "--require-brief",
        "--require-triage",
    ]
    sanity_tokens = [
        "tests/run_ci_sanity_gate.py",
        "--json-out build/reports/ci_sanity_gate.detjson",
    ]
    featured_seed_catalog_autogen_tokens = [
        "python3 solutions/seamgrim_ui_mvp/tools/sync_featured_seed_catalog.py --check",
        "python solutions/seamgrim_ui_mvp/tools/sync_featured_seed_catalog.py --check",
        "if [ \"$rc\" -eq 0 ]; then",
    ]
    checklist_forbidden_tokens = [
        "--skip-5min-checklist",
        "--with-5min-checklist",
    ]

    update_progress("running", "gitlab_static")
    gitlab_aggregate_lines = extract_aggregate_command_lines(gitlab_text)
    require_tokens_in_all_lines(gitlab_aggregate_lines, "gitlab.aggregate", aggregate_tokens, errors)
    forbid_tokens_in_all_lines(gitlab_aggregate_lines, "gitlab.aggregate", checklist_forbidden_tokens, errors)
    require_tokens(gitlab_text, "gitlab.fixed64_threeway", fixed64_threeway_tokens, errors)
    require_tokens(
        gitlab_text,
        "gitlab.profile_matrix_full_real_smoke",
        GITLAB_PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_TOKENS,
        errors,
    )
    require_tokens(
        gitlab_text,
        "gitlab.profile_matrix_selftest_gate",
        GITLAB_PROFILE_MATRIX_SELFTEST_GATE_TOKENS,
        errors,
    )
    require_tokens(
        gitlab_text,
        "gitlab.age5_combined_heavy",
        GITLAB_AGE5_COMBINED_HEAVY_POLICY_TOKENS,
        errors,
    )
    require_tokens(
        gitlab_text,
        "gitlab.age5_combined_heavy.top_payload",
        [
            f"{AGE5_COMBINED_HEAVY_POLICY_SCRIPT} --provider gitlab --format shell --json-out {AGE5_COMBINED_HEAVY_POLICY_REPORT_PATH}",
            f"{AGE5_COMBINED_HEAVY_POLICY_SCRIPT} --provider gitlab --format text > {AGE5_COMBINED_HEAVY_POLICY_TEXT_PATH}",
            f"{AGE5_COMBINED_HEAVY_POLICY_DIGEST_SCRIPT} {AGE5_COMBINED_HEAVY_POLICY_REPORT_PATH} --policy-text {AGE5_COMBINED_HEAVY_POLICY_TEXT_PATH} --summary-out {AGE5_COMBINED_HEAVY_POLICY_SUMMARY_PATH} > {AGE5_COMBINED_HEAVY_POLICY_SUMMARY_PATH}",
        ],
        errors,
    )
    require_tokens(gitlab_text, "gitlab.darwin_guard", gitlab_darwin_guard_tokens, errors)
    require_tokens(gitlab_text, "gitlab.sanity", sanity_tokens, errors)
    require_tokens(
        gitlab_text,
        "gitlab.featured_seed_catalog_autogen",
        featured_seed_catalog_autogen_tokens,
        errors,
    )
    require_tokens(gitlab_text, "gitlab.emit", emit_tokens, errors)
    require_tokens(gitlab_text, "gitlab.emit.require", emit_require_tokens, errors)
    require_tokens(gitlab_text, "gitlab.emit.artifacts_check", emit_artifacts_check_tokens, errors)
    require_tokens(
        gitlab_text,
        "gitlab.artifacts",
        [
            "build/reports/*.ci_gate_step_*.stdout.txt",
            "build/reports/*.ci_gate_step_*.stderr.txt",
            "build/reports/*.ci_fail_brief.txt",
            "build/reports/*.ci_fail_triage.detjson",
            "build/reports/darwin_probe_archive/*.detjson",
        ],
        errors,
    )
    complete_section("gitlab_static")

    update_progress("running", "azure_static")
    azure_aggregate_lines = extract_aggregate_command_lines(azure_text)
    require_tokens_in_all_lines(azure_aggregate_lines, "azure.aggregate", aggregate_tokens, errors)
    forbid_tokens_in_all_lines(azure_aggregate_lines, "azure.aggregate", checklist_forbidden_tokens, errors)
    require_tokens(azure_text, "azure.fixed64_threeway", fixed64_threeway_tokens, errors)
    require_tokens(
        azure_text,
        "azure.profile_matrix_full_real_smoke",
        AZURE_PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_TOKENS,
        errors,
    )
    require_tokens(
        azure_text,
        "azure.profile_matrix_selftest_gate",
        AZURE_PROFILE_MATRIX_SELFTEST_GATE_TOKENS,
        errors,
    )
    require_tokens(
        azure_text,
        "azure.age5_combined_heavy",
        AZURE_AGE5_COMBINED_HEAVY_POLICY_TOKENS,
        errors,
    )
    require_tokens(
        azure_text,
        "azure.age5_combined_heavy.top_payload",
        [
            f"{AGE5_COMBINED_HEAVY_POLICY_SCRIPT} --provider azure --format shell --json-out {AGE5_COMBINED_HEAVY_POLICY_REPORT_PATH}",
            f"{AGE5_COMBINED_HEAVY_POLICY_SCRIPT} --provider azure --format text > {AGE5_COMBINED_HEAVY_POLICY_TEXT_PATH}",
            f"{AGE5_COMBINED_HEAVY_POLICY_DIGEST_SCRIPT} {AGE5_COMBINED_HEAVY_POLICY_REPORT_PATH} --policy-text {AGE5_COMBINED_HEAVY_POLICY_TEXT_PATH} --summary-out {AGE5_COMBINED_HEAVY_POLICY_SUMMARY_PATH} > {AGE5_COMBINED_HEAVY_POLICY_SUMMARY_PATH}",
        ],
        errors,
    )
    require_tokens(azure_text, "azure.darwin_guard", azure_darwin_guard_tokens, errors)
    require_tokens(azure_text, "azure.sanity", sanity_tokens, errors)
    require_tokens(
        azure_text,
        "azure.featured_seed_catalog_autogen",
        featured_seed_catalog_autogen_tokens,
        errors,
    )
    require_tokens(azure_text, "azure.emit", emit_tokens, errors)
    require_tokens(azure_text, "azure.emit.require", emit_require_tokens, errors)
    require_tokens(azure_text, "azure.emit.artifacts_check", emit_artifacts_check_tokens, errors)
    require_tokens(
        azure_text,
        "azure.publish",
        [
            "PublishBuildArtifacts@1",
            "PathtoPublish: build/reports",
        ],
        errors,
    )
    require_tokens(
        azure_text,
        "azure.schedule",
        [
            "schedules:",
            'cron: "0 */3 * * *"',
            "displayName: fixed64 darwin probe cadence",
            "always: true",
        ],
        errors,
    )
    complete_section("azure_static")

    if errors:
        return fail_with_errors()

    update_progress("running", "profile_matrix_selftest_gate_runtime")
    profile_selftest_gate_cache_key = (
        "profile_matrix_selftest_gate_runtime",
        build_runtime_cache_key(selftest_gate_helper_path),
    )
    cached_profile_selftest_gate_errors = _RUNTIME_CHECK_CACHE.get(profile_selftest_gate_cache_key)
    if cached_profile_selftest_gate_errors is None:
        before_count = len(errors)
        check_profile_matrix_selftest_gate_flags_runtime(
            root=Path(__file__).resolve().parent.parent,
            helper_path=selftest_gate_helper_path,
            errors=errors,
        )
        _RUNTIME_CHECK_CACHE[profile_selftest_gate_cache_key] = tuple(errors[before_count:])
    else:
        errors.extend(cached_profile_selftest_gate_errors)
    complete_section("profile_matrix_selftest_gate_runtime")
    if errors:
        return fail_with_errors()

    update_progress("running", "profile_matrix_policy_runtime")
    profile_cache_key = ("profile_matrix_policy_runtime", build_runtime_cache_key(helper_path))
    cached_profile_errors = _RUNTIME_CHECK_CACHE.get(profile_cache_key)
    if cached_profile_errors is None:
        before_count = len(errors)
        check_profile_matrix_full_real_smoke_policy_runtime(
            root=Path(__file__).resolve().parent.parent,
            helper_path=helper_path,
            errors=errors,
        )
        _RUNTIME_CHECK_CACHE[profile_cache_key] = tuple(errors[before_count:])
    else:
        errors.extend(cached_profile_errors)
    complete_section("profile_matrix_policy_runtime")
    if errors:
        return fail_with_errors()
    update_progress("running", "age5_combined_policy_runtime")
    age5_progress_hook = (lambda current: update_progress("running", current)) if progress_path else None
    age5_cache_key = ("age5_combined_policy_runtime", build_runtime_cache_key(age5_helper_path))
    cached_age5_errors = _RUNTIME_CHECK_CACHE.get(age5_cache_key)
    if cached_age5_errors is None:
        before_count = len(errors)
        check_age5_combined_heavy_policy_runtime(
            root=Path(__file__).resolve().parent.parent,
            helper_path=age5_helper_path,
            errors=errors,
            progress_hook=age5_progress_hook,
        )
        _RUNTIME_CHECK_CACHE[age5_cache_key] = tuple(errors[before_count:])
    else:
        errors.extend(cached_age5_errors)
    complete_section("age5_combined_policy_runtime")

    if errors:
        return fail_with_errors()

    write_progress_snapshot(
        progress_path,
        status="pass",
        current_section="-",
        last_completed_section=last_completed_section,
        sections_completed=(0 if last_completed_section == "-" else len([item for item in last_completed_section.split(",") if item])),
        total_elapsed_ms=int(round((time.perf_counter() - started) * 1000.0)),
    )
    print(f"[ci-pipeline-emit-flags-check] ok gitlab={gitlab_path} azure={azure_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
