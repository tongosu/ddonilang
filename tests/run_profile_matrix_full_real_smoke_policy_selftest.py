#!/usr/bin/env python
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import time
from contextlib import contextmanager
from importlib.util import module_from_spec, spec_from_file_location
from pathlib import Path

from _ci_profile_matrix_full_real_smoke_contract import (
    PROFILE_MATRIX_FULL_REAL_SMOKE_MODE,
    PROFILE_MATRIX_FULL_REAL_SMOKE_SCOPE_FRAGMENT,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_CORE_LANG,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_FULL,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_SEAMGRIM,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_CORE_LANG,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_FULL,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_SEAMGRIM,
    PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT,
)

PROGRESS_ENV_KEY = "DDN_PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SELFTEST_PROGRESS_JSON"


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


def fail(detail: str, proc: subprocess.CompletedProcess[str] | None = None) -> int:
    print(f"check=profile_matrix_full_real_smoke_policy_selftest detail={detail}")
    if proc is not None:
        if (proc.stdout or "").strip():
            print(proc.stdout.strip())
        if (proc.stderr or "").strip():
            print(proc.stderr.strip())
    return 1


@contextmanager
def patched_environ(env_patch: dict[str, str]):
    previous = dict(os.environ)
    os.environ.update(env_patch)
    try:
        yield
    finally:
        os.environ.clear()
        os.environ.update(previous)


def load_policy_impl(root: Path) -> dict[str, object]:
    script_path = root / "tools/scripts/resolve_profile_matrix_full_real_smoke_policy.py"
    spec = spec_from_file_location("_resolve_profile_matrix_full_real_smoke_policy_selftest_impl", script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"unable to load profile matrix policy helper: {script_path}")
    module = module_from_spec(spec)
    spec.loader.exec_module(module)
    return {
        "resolve": getattr(module, "resolve"),
        "render_shell": getattr(module, "render_shell"),
        "scope_fragment": getattr(module, "PROFILE_MATRIX_FULL_REAL_SMOKE_SCOPE_FRAGMENT"),
    }


def run_profile_matrix_full_real_smoke_policy_fast_path(
    *,
    root: Path,
    provider: str,
    env_patch: dict[str, str],
    output_format: str,
) -> tuple[dict[str, object], str]:
    impl = load_policy_impl(root)
    with patched_environ(env_patch):
        payload = impl["resolve"](provider)
    if output_format == "json":
        return payload, json.dumps(payload, ensure_ascii=False)
    if output_format == "shell":
        return payload, impl["render_shell"](payload)
    status = "enabled" if bool(payload.get("enabled", False)) else "disabled"
    text_out = (
        "[ci-profile-matrix-full-real-smoke-policy] "
        f"provider={payload['provider']} status={status} reason={payload['reason']} "
        f"{impl['scope_fragment']} "
        f"step_timeout_defaults={payload.get('step_timeout_defaults_text', PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT)}"
    )
    return payload, text_out


def write_progress_snapshot(
    path_text: str,
    *,
    status: str,
    current_case: str,
    last_completed_case: str,
    current_format: str,
    last_completed_format: str,
    total_elapsed_ms: int,
) -> None:
    if not str(path_text).strip():
        return
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "ddn.ci.profile_matrix_full_real_smoke_policy_selftest.progress.v1",
        "status": status,
        "current_case": current_case,
        "last_completed_case": last_completed_case,
        "current_format": current_format,
        "last_completed_format": last_completed_format,
        "total_elapsed_ms": str(int(total_elapsed_ms)),
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    script = "tools/scripts/resolve_profile_matrix_full_real_smoke_policy.py"
    progress_path = str(os.environ.get(PROGRESS_ENV_KEY, "")).strip()
    started_at = time.perf_counter()
    current_case = "-"
    last_completed_case = "-"
    current_format = "-"
    last_completed_format = "-"

    def update_progress(status: str) -> None:
        elapsed_ms = int((time.perf_counter() - started_at) * 1000)
        write_progress_snapshot(
            progress_path,
            status=status,
            current_case=current_case,
            last_completed_case=last_completed_case,
            current_format=current_format,
            last_completed_format=last_completed_format,
            total_elapsed_ms=elapsed_ms,
        )

    def start_case(name: str) -> None:
        nonlocal current_case, current_format
        current_case = name
        current_format = "-"
        update_progress("running")

    def complete_case(name: str) -> None:
        nonlocal current_case, current_format, last_completed_case
        current_case = "-"
        current_format = "-"
        last_completed_case = name
        update_progress("running")

    def start_format(name: str) -> None:
        nonlocal current_format
        current_format = name
        update_progress("running")

    def complete_format(name: str) -> None:
        nonlocal current_format, last_completed_format
        current_format = "-"
        last_completed_format = name
        update_progress("running")

    cases = [
        {
            "name": "gitlab_default_off",
            "provider": "gitlab",
            "env": {"CI_PIPELINE_SOURCE": "push", "DDN_CI_PROFILE_GATE_WITH_PROFILE_MATRIX_FULL_REAL_SMOKE": "0"},
            "enabled": False,
            "reason": "default_off",
            "fast_path_json": True,
            "fast_path_shell": True,
            "fast_path_text": True,
        },
        {
            "name": "gitlab_schedule",
            "provider": "gitlab",
            "env": {"CI_PIPELINE_SOURCE": "schedule", "DDN_CI_PROFILE_GATE_WITH_PROFILE_MATRIX_FULL_REAL_SMOKE": "0"},
            "enabled": True,
            "reason": "schedule",
            "fast_path_json": True,
            "fast_path_shell": True,
            "fast_path_text": True,
        },
        {
            "name": "gitlab_manual_optin",
            "provider": "gitlab",
            "env": {"CI_PIPELINE_SOURCE": "web", "DDN_CI_PROFILE_GATE_WITH_PROFILE_MATRIX_FULL_REAL_SMOKE": "1"},
            "enabled": True,
            "reason": "manual_optin",
            "fast_path_json": True,
            "fast_path_shell": True,
            "fast_path_text": True,
        },
        {
            "name": "azure_default_off",
            "provider": "azure",
            "env": {"BUILD_REASON": "IndividualCI", "DDN_CI_PROFILE_GATE_WITH_PROFILE_MATRIX_FULL_REAL_SMOKE": "0"},
            "enabled": False,
            "reason": "default_off",
            "fast_path_json": True,
            "fast_path_shell": True,
            "fast_path_text": True,
        },
        {
            "name": "azure_schedule",
            "provider": "azure",
            "env": {"BUILD_REASON": "Schedule", "DDN_CI_PROFILE_GATE_WITH_PROFILE_MATRIX_FULL_REAL_SMOKE": "0"},
            "enabled": True,
            "reason": "schedule",
            "fast_path_json": True,
            "fast_path_shell": True,
            "fast_path_text": True,
        },
        {
            "name": "azure_manual_optin",
            "provider": "azure",
            "env": {"BUILD_REASON": "Manual", "DDN_CI_PROFILE_GATE_WITH_PROFILE_MATRIX_FULL_REAL_SMOKE": "1"},
            "enabled": True,
            "reason": "manual_optin",
            "fast_path_json": True,
            "fast_path_shell": True,
            "fast_path_text": True,
        },
    ]

    with tempfile.TemporaryDirectory(prefix="profile_matrix_full_real_smoke_policy_") as td:
        temp_root = Path(td)
        update_progress("running")
        for case in cases:
            start_case(str(case["name"]))
            env = dict(os.environ)
            env.update(case["env"])
            json_out = temp_root / f"{case['name']}.detjson"
            start_format("json")
            if bool(case.get("fast_path_json", False)):
                payload, json_text = run_profile_matrix_full_real_smoke_policy_fast_path(
                    root=root,
                    provider=str(case["provider"]),
                    env_patch=dict(case["env"]),
                    output_format="json",
                )
                json_out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
                proc = subprocess.CompletedProcess(
                    [py, script, "--provider", str(case["provider"]), "--format", "json", "--json-out", str(json_out)],
                    0,
                    json_text,
                    "",
                )
                stdout_doc = payload
                file_doc = payload
            else:
                proc = run(
                    [py, script, "--provider", str(case["provider"]), "--format", "json", "--json-out", str(json_out)],
                    root,
                    env=env,
                )
                if proc.returncode != 0:
                    update_progress("fail")
                    return fail(f"{case['name']}_runner_failed", proc)
                try:
                    stdout_doc = json.loads(str(proc.stdout or "").strip())
                except Exception:
                    update_progress("fail")
                    return fail(f"{case['name']}_stdout_json_invalid", proc)
                try:
                    file_doc = json.loads(json_out.read_text(encoding="utf-8"))
                except Exception:
                    update_progress("fail")
                    return fail(f"{case['name']}_file_json_invalid")
            if bool(stdout_doc.get("enabled", False)) != bool(case["enabled"]):
                update_progress("fail")
                return fail(f"{case['name']}_enabled_mismatch", proc)
            if str(stdout_doc.get("reason", "")).strip() != str(case["reason"]):
                update_progress("fail")
                return fail(f"{case['name']}_reason_mismatch", proc)
            if str(stdout_doc.get("scope", "")).strip() != PROFILE_MATRIX_FULL_REAL_SMOKE_MODE:
                update_progress("fail")
                return fail(f"{case['name']}_scope_mismatch", proc)
            step_timeout_env_keys = stdout_doc.get("step_timeout_env_keys")
            if not isinstance(step_timeout_env_keys, dict):
                update_progress("fail")
                return fail(f"{case['name']}_step_timeout_env_keys_missing", proc)
            expected_env_keys = {
                "core_lang": PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_CORE_LANG,
                "full": PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_FULL,
                "seamgrim": PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_SEAMGRIM,
            }
            if step_timeout_env_keys != expected_env_keys:
                update_progress("fail")
                return fail(f"{case['name']}_step_timeout_env_keys_mismatch", proc)
            step_timeout_defaults = stdout_doc.get("step_timeout_defaults_sec")
            if not isinstance(step_timeout_defaults, dict):
                update_progress("fail")
                return fail(f"{case['name']}_step_timeout_defaults_missing", proc)
            expected_defaults = {
                "core_lang": float(PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_CORE_LANG),
                "full": float(PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_FULL),
                "seamgrim": float(PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_SEAMGRIM),
            }
            if step_timeout_defaults != expected_defaults:
                update_progress("fail")
                return fail(f"{case['name']}_step_timeout_defaults_mismatch", proc)
            if (
                str(stdout_doc.get("step_timeout_defaults_text", "")).strip()
                != PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT
            ):
                update_progress("fail")
                return fail(f"{case['name']}_step_timeout_defaults_text_mismatch", proc)
            if stdout_doc != file_doc:
                update_progress("fail")
                return fail(f"{case['name']}_json_out_mismatch")
            complete_format("json")

            start_format("shell")
            if bool(case.get("fast_path_shell", False)):
                _, shell_text = run_profile_matrix_full_real_smoke_policy_fast_path(
                    root=root,
                    provider=str(case["provider"]),
                    env_patch=dict(case["env"]),
                    output_format="shell",
                )
                shell_proc = subprocess.CompletedProcess(
                    [py, script, "--provider", str(case["provider"]), "--format", "shell"],
                    0,
                    shell_text,
                    "",
                )
            else:
                shell_proc = run([py, script, "--provider", str(case["provider"]), "--format", "shell"], root, env=env)
                if shell_proc.returncode != 0:
                    update_progress("fail")
                    return fail(f"{case['name']}_shell_failed", shell_proc)
                shell_text = str(shell_proc.stdout or "")
            expected_export = (
                "export DDN_CI_PROFILE_GATE_WITH_PROFILE_MATRIX_FULL_REAL_SMOKE=1"
                if bool(case["enabled"])
                else "export DDN_CI_PROFILE_GATE_WITH_PROFILE_MATRIX_FULL_REAL_SMOKE=0"
            )
            if expected_export not in shell_text:
                update_progress("fail")
                return fail(f"{case['name']}_shell_export_mismatch", shell_proc)
            expected_timeout_exports = [
                f"export {PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_CORE_LANG}={PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_CORE_LANG:g}",
                f"export {PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_FULL}={PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_FULL:g}",
                f"export {PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_ENV_KEY_SEAMGRIM}={PROFILE_MATRIX_FULL_REAL_SMOKE_STEP_TIMEOUT_DEFAULT_SEC_SEAMGRIM:g}",
            ]
            for timeout_export in expected_timeout_exports:
                if timeout_export not in shell_text:
                    update_progress("fail")
                    return fail(f"{case['name']}_shell_timeout_export_missing", shell_proc)
            if "[ci-profile-matrix-full-real-smoke] enabled" not in shell_text:
                update_progress("fail")
                return fail(f"{case['name']}_shell_marker_missing", shell_proc)
            if PROFILE_MATRIX_FULL_REAL_SMOKE_SCOPE_FRAGMENT not in shell_text:
                update_progress("fail")
                return fail(f"{case['name']}_shell_scope_missing", shell_proc)
            complete_format("shell")
            start_format("text")
            if bool(case.get("fast_path_text", False)):
                _, text_out = run_profile_matrix_full_real_smoke_policy_fast_path(
                    root=root,
                    provider=str(case["provider"]),
                    env_patch=dict(case["env"]),
                    output_format="text",
                )
                text_proc = subprocess.CompletedProcess(
                    [py, script, "--provider", str(case["provider"]), "--format", "text"],
                    0,
                    text_out,
                    "",
                )
            else:
                text_proc = run([py, script, "--provider", str(case["provider"]), "--format", "text"], root, env=env)
                if text_proc.returncode != 0:
                    update_progress("fail")
                    return fail(f"{case['name']}_text_failed", text_proc)
                text_out = str(text_proc.stdout or "")
            if f"provider={case['provider']}" not in text_out:
                update_progress("fail")
                return fail(f"{case['name']}_text_provider_missing", text_proc)
            expected_status = "enabled" if bool(case["enabled"]) else "disabled"
            if f"status={expected_status}" not in text_out:
                update_progress("fail")
                return fail(f"{case['name']}_text_status_missing", text_proc)
            if f"reason={case['reason']}" not in text_out:
                update_progress("fail")
                return fail(f"{case['name']}_text_reason_missing", text_proc)
            if PROFILE_MATRIX_FULL_REAL_SMOKE_SCOPE_FRAGMENT not in text_out:
                update_progress("fail")
                return fail(f"{case['name']}_text_scope_missing", text_proc)
            if f"step_timeout_defaults={PROFILE_MATRIX_FULL_REAL_SMOKE_TIMEOUT_DEFAULTS_TEXT}" not in text_out:
                update_progress("fail")
                return fail(f"{case['name']}_text_timeout_defaults_missing", text_proc)
            complete_format("text")
            complete_case(str(case["name"]))

    update_progress("pass")
    print("profile matrix full real smoke policy selftest ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
