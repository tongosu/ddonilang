#!/usr/bin/env python
from __future__ import annotations

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
    AGE5_COMBINED_HEAVY_ENV_KEY,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FRAGMENTS,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FIELDS,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FRAGMENTS,
    AGE5_COMBINED_HEAVY_MODE,
    AGE5_COMBINED_HEAVY_POLICY_MARKER,
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
    AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT,
    AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_FRAGMENT,
    AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_KEY,
    AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_FIELDS_TEXT,
    AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_DEFAULT,
    AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_FRAGMENT,
    AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_KEY,
    build_age4_proof_snapshot,
    build_age4_proof_snapshot_text,
    build_age4_proof_source_snapshot_fields,
    build_age5_combined_heavy_timeout_policy_fields,
)

PROGRESS_ENV_KEY = "DDN_AGE5_COMBINED_HEAVY_POLICY_SELFTEST_PROGRESS_JSON"
TOOLS_SCRIPTS_DIR = Path(__file__).resolve().parent.parent / "tools" / "scripts"


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


def run_age5_combined_policy_fast_path(
    *,
    provider: str,
    env_patch: dict[str, str],
    output_format: str,
) -> tuple[dict[str, object], str]:
    impl = load_age5_combined_policy_impl()
    with patched_environ(env_patch):
        payload = impl["resolve"](provider)
    if output_format == "json":
        return payload, json.dumps(payload, ensure_ascii=False)
    if output_format == "shell":
        return payload, impl["render_shell"](payload)
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


def read_progress_stage_history(progress_doc: dict[str, object]) -> list[str]:
    raw = progress_doc.get("stage_history", [])
    if not isinstance(raw, list):
        return []
    stages: list[str] = []
    for item in raw:
        stage = str(item).strip() or "-"
        if stage == "-" or stage in stages:
            continue
        stages.append(stage)
    return stages


def fail(detail: str, proc: subprocess.CompletedProcess[str] | None = None) -> int:
    print(f"check=age5_combined_heavy_policy_selftest detail={detail}")
    if proc is not None:
        if (proc.stdout or "").strip():
            print(proc.stdout.strip())
        if (proc.stderr or "").strip():
            print(proc.stderr.strip())
    return 1


def write_progress_snapshot(
    path_text: str,
    *,
    status: str,
    current_case: str,
    last_completed_case: str,
    current_format: str,
    last_completed_format: str,
    current_probe: str,
    last_completed_probe: str,
    cases_completed: int,
    total_elapsed_ms: int,
) -> None:
    if not str(path_text).strip():
        return
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "ddn.age5_combined_heavy_policy_selftest_progress.v1",
        "status": status,
        "current_case": current_case,
        "last_completed_case": last_completed_case,
        "current_format": current_format,
        "last_completed_format": last_completed_format,
        "current_probe": current_probe,
        "last_completed_probe": last_completed_probe,
        "cases_completed": int(cases_completed),
        "total_elapsed_ms": int(total_elapsed_ms),
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    script = "tools/scripts/resolve_age5_combined_heavy_policy.py"
    progress_path = str(os.environ.get(PROGRESS_ENV_KEY, "")).strip()
    started_at = time.perf_counter()
    current_case = "-"
    last_completed_case = "-"
    current_format = "-"
    last_completed_format = "-"
    current_probe = "-"
    last_completed_probe = "-"
    cases_completed = 0

    def update_progress(status: str) -> None:
        write_progress_snapshot(
            progress_path,
            status=status,
            current_case=current_case,
            last_completed_case=last_completed_case,
            current_format=current_format,
            last_completed_format=last_completed_format,
            current_probe=current_probe,
            last_completed_probe=last_completed_probe,
            cases_completed=cases_completed,
            total_elapsed_ms=int((time.perf_counter() - started_at) * 1000),
        )

    def start_case(name: str) -> None:
        nonlocal current_case, current_format, current_probe
        current_case = name
        current_format = "-"
        current_probe = "-"
        update_progress("running")

    def complete_case(name: str) -> None:
        nonlocal current_case, current_format, current_probe, last_completed_case, cases_completed
        last_completed_case = name
        current_case = "-"
        current_format = "-"
        current_probe = "-"
        cases_completed += 1
        update_progress("running")

    def start_format(name: str) -> None:
        nonlocal current_format, current_probe
        current_format = name
        current_probe = "-"
        update_progress("running")

    def complete_format(name: str) -> None:
        nonlocal current_format, current_probe, last_completed_format
        last_completed_format = name
        current_format = "-"
        current_probe = "-"
        update_progress("running")

    def start_probe(name: str) -> None:
        nonlocal current_probe
        current_probe = name
        update_progress("running")

    def complete_probe(name: str) -> None:
        nonlocal current_probe, last_completed_probe
        last_completed_probe = name
        current_probe = "-"
        update_progress("running")

    cases = [
        {
            "name": "gitlab_default_off",
            "provider": "gitlab",
            "env": {"CI_PIPELINE_SOURCE": "push", AGE5_COMBINED_HEAVY_ENV_KEY: "0"},
            "enabled": False,
            "reason": "default_off",
            "verify_progress": True,
            "verify_json_progress": False,
            "verify_shell_progress": True,
            "verify_text_progress": False,
        },
        {
            "name": "gitlab_schedule",
            "provider": "gitlab",
            "env": {"CI_PIPELINE_SOURCE": "schedule", AGE5_COMBINED_HEAVY_ENV_KEY: "0"},
            "enabled": True,
            "reason": "schedule",
            "verify_progress": True,
            "verify_json_progress": False,
            "verify_shell_progress": False,
            "verify_text_progress": False,
        },
        {
            "name": "gitlab_manual_optin",
            "provider": "gitlab",
            "env": {"CI_PIPELINE_SOURCE": "web", AGE5_COMBINED_HEAVY_ENV_KEY: "1"},
            "enabled": True,
            "reason": "manual_optin",
            "verify_progress": True,
            "verify_json_progress": False,
            "verify_shell_progress": False,
            "verify_text_progress": False,
        },
        {
            "name": "azure_default_off",
            "provider": "azure",
            "env": {"BUILD_REASON": "IndividualCI", AGE5_COMBINED_HEAVY_ENV_KEY: "0"},
            "enabled": False,
            "reason": "default_off",
            "verify_progress": False,
            "verify_json_progress": False,
            "verify_shell_progress": False,
            "verify_text_progress": False,
        },
        {
            "name": "azure_schedule",
            "provider": "azure",
            "env": {"BUILD_REASON": "Schedule", AGE5_COMBINED_HEAVY_ENV_KEY: "0"},
            "enabled": True,
            "reason": "schedule",
            "verify_progress": False,
            "verify_json_progress": False,
            "verify_shell_progress": False,
            "verify_text_progress": False,
        },
        {
            "name": "azure_manual_optin",
            "provider": "azure",
            "env": {"BUILD_REASON": "Manual", AGE5_COMBINED_HEAVY_ENV_KEY: "1"},
            "enabled": True,
            "reason": "manual_optin",
            "verify_progress": False,
            "verify_json_progress": False,
            "verify_shell_progress": False,
            "verify_text_progress": False,
        },
    ]

    with tempfile.TemporaryDirectory(prefix="age5_combined_heavy_policy_") as td:
        temp_root = Path(td)
        update_progress("running")
        for case in cases:
            start_case(str(case["name"]))
            env = dict(os.environ)
            env.update(case["env"])
            json_out = temp_root / f"{case['name']}.detjson"
            json_progress = temp_root / f"{case['name']}.json.progress.detjson"
            json_env = dict(env)
            json_env["DDN_AGE5_COMBINED_HEAVY_POLICY_PROGRESS_JSON"] = str(json_progress)
            start_format("json")
            start_probe("run_json")
            if bool(case.get("verify_json_progress", case["verify_progress"])):
                proc = run(
                    [py, script, "--provider", str(case["provider"]), "--format", "json", "--json-out", str(json_out)],
                    root,
                    env=json_env,
                )
                if proc.returncode != 0:
                    return fail(f"{case['name']}_runner_failed", proc)
                try:
                    stdout_doc = json.loads(str(proc.stdout or "").strip())
                except Exception:
                    return fail(f"{case['name']}_stdout_json_invalid", proc)
                try:
                    file_doc = json.loads(json_out.read_text(encoding="utf-8"))
                except Exception:
                    return fail(f"{case['name']}_file_json_invalid")
                try:
                    json_progress_doc = json.loads(json_progress.read_text(encoding="utf-8"))
                except Exception:
                    return fail(f"{case['name']}_json_progress_invalid")
                if str(json_progress_doc.get("provider", "")).strip() != str(case["provider"]):
                    return fail(f"{case['name']}_json_progress_provider_mismatch")
                if str(json_progress_doc.get("format", "")).strip() != "json":
                    return fail(f"{case['name']}_json_progress_format_mismatch")
                if str(json_progress_doc.get("status", "")).strip() != "pass":
                    return fail(f"{case['name']}_json_progress_status_mismatch")
                if str(json_progress_doc.get("last_completed_stage", "")).strip() != "emit_json":
                    return fail(f"{case['name']}_json_progress_stage_mismatch")
            else:
                stdout_doc, _ = run_age5_combined_policy_fast_path(
                    provider=str(case["provider"]),
                    env_patch=dict(case["env"]),
                    output_format="json",
                )
                file_doc = stdout_doc
                json_out.write_text(json.dumps(stdout_doc, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
                proc = subprocess.CompletedProcess(
                    [py, script, "--provider", str(case["provider"]), "--format", "json", "--json-out", str(json_out)],
                    0,
                    json.dumps(stdout_doc, ensure_ascii=False),
                    "",
                )
            if bool(stdout_doc.get("enabled", False)) != bool(case["enabled"]):
                return fail(f"{case['name']}_enabled_mismatch", proc)
            if str(stdout_doc.get("reason", "")).strip() != str(case["reason"]):
                return fail(f"{case['name']}_reason_mismatch", proc)
            if str(stdout_doc.get("scope", "")).strip() != AGE5_COMBINED_HEAVY_MODE:
                return fail(f"{case['name']}_scope_mismatch", proc)
            if str(stdout_doc.get("combined_report_schema", "")).strip() != AGE5_COMBINED_HEAVY_REPORT_SCHEMA:
                return fail(f"{case['name']}_report_schema_mismatch", proc)
            if list(stdout_doc.get("combined_required_reports", [])) != list(AGE5_COMBINED_HEAVY_REQUIRED_REPORTS):
                return fail(f"{case['name']}_required_reports_mismatch", proc)
            if list(stdout_doc.get("combined_required_criteria", [])) != list(AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA):
                return fail(f"{case['name']}_required_criteria_mismatch", proc)
            if list(stdout_doc.get("combined_child_summary_keys", [])) != list(AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS):
                return fail(f"{case['name']}_combined_child_summary_keys_mismatch", proc)
            if str(stdout_doc.get("combined_child_summary_keys_text", "")).strip() != AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS_TEXT:
                return fail(f"{case['name']}_combined_child_summary_keys_text_mismatch", proc)
            if dict(stdout_doc.get("combined_child_summary_default_fields", {})) != dict(
                AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS
            ):
                return fail(f"{case['name']}_combined_child_summary_default_fields_mismatch", proc)
            if str(stdout_doc.get("combined_child_summary_default_fields_text", "")).strip() != (
                AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT
            ):
                return fail(f"{case['name']}_combined_child_summary_default_fields_text_mismatch", proc)
            if dict(stdout_doc.get("combined_timeout_policy_fields", {})) != dict(
                build_age5_combined_heavy_timeout_policy_fields()
            ):
                return fail(f"{case['name']}_combined_timeout_policy_fields_mismatch", proc)
            if str(stdout_doc.get("combined_timeout_policy_fields_text", "")).strip() != (
                AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_FIELDS_TEXT
            ):
                return fail(f"{case['name']}_combined_timeout_policy_fields_text_mismatch", proc)
            if str(stdout_doc.get(AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DEFAULT_KEY, "")).strip() != (
                AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DISABLED
            ):
                return fail(f"{case['name']}_combined_timeout_mode_default_mismatch", proc)
            if str(stdout_doc.get(AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_KEY, "")).strip() != (
                AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_TEXT
            ):
                return fail(f"{case['name']}_combined_timeout_mode_allowed_values_mismatch", proc)
            if str(stdout_doc.get(AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_KEY, "")).strip() != (
                AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_DEFAULT
            ):
                return fail(f"{case['name']}_combined_timeout_mode_preview_only_mismatch", proc)
            if str(stdout_doc.get(AGE5_COMBINED_HEAVY_TIMEOUT_MODE_SCOPE_KEY, "")).strip() != (
                AGE5_COMBINED_HEAVY_MODE
            ):
                return fail(f"{case['name']}_combined_timeout_mode_scope_mismatch", proc)
            if str(stdout_doc.get(AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_KEY, "")).strip() != (
                AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_DEFAULT
            ):
                return fail(f"{case['name']}_combined_timeout_requires_optin_mismatch", proc)
            if str(stdout_doc.get(AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_KEY, "")).strip() != (
                AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_DEFAULT
            ):
                return fail(f"{case['name']}_combined_timeout_policy_reason_mismatch", proc)
            expected_age4_proof_snapshot = build_age4_proof_snapshot()
            expected_age4_proof_source_snapshot = build_age4_proof_source_snapshot_fields(
                top_snapshot=expected_age4_proof_snapshot
            )
            if str(stdout_doc.get("age4_proof_snapshot_fields_text", "")).strip() != AGE4_PROOF_SNAPSHOT_FIELDS_TEXT:
                return fail(f"{case['name']}_age4_proof_snapshot_fields_text_mismatch", proc)
            if str(stdout_doc.get("age4_proof_source_snapshot_fields_text", "")).strip() != (
                AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT
            ):
                return fail(f"{case['name']}_age4_proof_source_snapshot_fields_text_mismatch", proc)
            if str(stdout_doc.get("age4_proof_snapshot_text", "")).strip() != (
                build_age4_proof_snapshot_text(expected_age4_proof_snapshot)
            ):
                return fail(f"{case['name']}_age4_proof_snapshot_text_mismatch", proc)
            for key, expected in expected_age4_proof_snapshot.items():
                if str(stdout_doc.get(key, "")).strip() != str(expected):
                    return fail(f"{case['name']}_{key}_mismatch", proc)
            for key, expected in expected_age4_proof_source_snapshot.items():
                if str(stdout_doc.get(key, "")).strip() != str(expected):
                    return fail(f"{case['name']}_{key}_mismatch", proc)
            if str(stdout_doc.get(AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY, "")).strip() != AGE5_CLOSE_DIGEST_SELFTEST_OK_DEFAULT:
                return fail(f"{case['name']}_digest_selftest_default_value_mismatch", proc)
            if dict(stdout_doc.get("combined_digest_selftest_default_field", {})) != {
                AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_DEFAULT
            }:
                return fail(f"{case['name']}_digest_selftest_default_field_mismatch", proc)
            if str(stdout_doc.get(AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY, "")).strip() != (
                AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT
            ):
                return fail(f"{case['name']}_digest_selftest_default_field_text_mismatch", proc)
            if dict(stdout_doc.get("combined_child_summary_default_text_transport_fields", {})) != dict(
                AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS
            ):
                return fail(f"{case['name']}_combined_child_summary_default_text_transport_fields_mismatch", proc)
            if str(stdout_doc.get("combined_child_summary_default_text_transport_fields_text", "")).strip() != (
                AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS_TEXT
            ):
                return fail(f"{case['name']}_combined_child_summary_default_text_transport_fields_text_mismatch", proc)
            if dict(stdout_doc.get("combined_contract_summary_fields", {})) != dict(
                AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS
            ):
                return fail(f"{case['name']}_combined_contract_summary_fields_mismatch", proc)
            if str(stdout_doc.get("combined_contract_summary_fields_text", "")).strip() != AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT:
                return fail(f"{case['name']}_combined_contract_summary_fields_text_mismatch", proc)
            if dict(stdout_doc.get("combined_full_summary_contract_fields", {})) != dict(
                AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS
            ):
                return fail(f"{case['name']}_combined_full_summary_contract_fields_mismatch", proc)
            if str(stdout_doc.get("combined_full_summary_contract_fields_text", "")).strip() != (
                AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT
            ):
                return fail(f"{case['name']}_combined_full_summary_contract_fields_text_mismatch", proc)
            if dict(stdout_doc.get("combined_full_summary_text_transport_fields", {})) != dict(
                AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FIELDS
            ):
                return fail(f"{case['name']}_combined_full_summary_text_transport_fields_mismatch", proc)
            if str(stdout_doc.get("combined_full_summary_text_transport_fields_text", "")).strip() != (
                AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FIELDS_TEXT
            ):
                return fail(f"{case['name']}_combined_full_summary_text_transport_fields_text_mismatch", proc)
            if stdout_doc != file_doc:
                return fail(f"{case['name']}_json_out_mismatch")
            complete_probe("run_json")
            complete_format("json")

            start_format("shell")
            shell_text = ""
            shell_proc = subprocess.CompletedProcess(
                [py, script, "--provider", str(case["provider"]), "--format", "shell"],
                0,
                "",
                "",
            )
            start_probe("run_shell")
            if bool(case.get("verify_shell_progress", case["verify_progress"])):
                shell_progress = temp_root / f"{case['name']}.shell.progress.detjson"
                shell_env = dict(env)
                shell_env["DDN_AGE5_COMBINED_HEAVY_POLICY_PROGRESS_JSON"] = str(shell_progress)
                shell_proc = run(
                    [py, script, "--provider", str(case["provider"]), "--format", "shell"],
                    root,
                    env=shell_env,
                )
                if shell_proc.returncode != 0:
                    return fail(f"{case['name']}_shell_failed", shell_proc)
                shell_text = str(shell_proc.stdout or "")
                try:
                    shell_progress_doc = json.loads(shell_progress.read_text(encoding="utf-8"))
                except Exception:
                    return fail(f"{case['name']}_shell_progress_invalid")
                if str(shell_progress_doc.get("format", "")).strip() != "shell":
                    return fail(f"{case['name']}_shell_progress_format_mismatch")
                if str(shell_progress_doc.get("status", "")).strip() != "pass":
                    return fail(f"{case['name']}_shell_progress_status_mismatch")
                if str(shell_progress_doc.get("last_completed_stage", "")).strip() != "emit_shell":
                    return fail(f"{case['name']}_shell_progress_stage_mismatch")
                seen_shell_stages = read_progress_stage_history(shell_progress_doc)
                expected_shell_stages = [
                    "import_contract_module.import_module",
                    "import_contract_module.bind_symbols",
                    "build_shell_snapshot",
                    "build_shell_source_snapshot",
                    "render_shell_text",
                    "emit_shell_stdout",
                    "emit_shell",
                ]
                for stage in expected_shell_stages:
                    if stage not in seen_shell_stages:
                        return fail(f"{case['name']}_shell_progress_missing_stage_{stage}")
            else:
                _, shell_text = run_age5_combined_policy_fast_path(
                    provider=str(case["provider"]),
                    env_patch=dict(case["env"]),
                    output_format="shell",
                )
            expected_export = (
                f"export {AGE5_COMBINED_HEAVY_ENV_KEY}=1"
                if bool(case["enabled"])
                else f"export {AGE5_COMBINED_HEAVY_ENV_KEY}=0"
            )
            if expected_export not in shell_text:
                return fail(f"{case['name']}_shell_export_mismatch", shell_proc)
            if AGE5_COMBINED_HEAVY_POLICY_MARKER not in shell_text:
                return fail(f"{case['name']}_shell_marker_missing", shell_proc)
            if AGE5_COMBINED_HEAVY_SCOPE_FRAGMENT not in shell_text:
                return fail(f"{case['name']}_shell_scope_missing", shell_proc)
            if AGE5_COMBINED_HEAVY_REPORTS_FRAGMENT not in shell_text:
                return fail(f"{case['name']}_shell_reports_fragment_missing", shell_proc)
            if AGE5_COMBINED_HEAVY_CRITERIA_FRAGMENT not in shell_text:
                return fail(f"{case['name']}_shell_criteria_fragment_missing", shell_proc)
            if AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS_FRAGMENT not in shell_text:
                return fail(f"{case['name']}_shell_child_summary_keys_fragment_missing", shell_proc)
            if AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_FRAGMENT not in shell_text:
                return fail(f"{case['name']}_shell_child_summary_default_fields_fragment_missing", shell_proc)
            if AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DEFAULT_FRAGMENT not in shell_text:
                return fail(f"{case['name']}_shell_timeout_mode_default_fragment_missing", shell_proc)
            if AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_FRAGMENT not in shell_text:
                return fail(f"{case['name']}_shell_timeout_mode_allowed_values_fragment_missing", shell_proc)
            if AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_FRAGMENT not in shell_text:
                return fail(f"{case['name']}_shell_timeout_mode_preview_only_fragment_missing", shell_proc)
            if AGE5_COMBINED_HEAVY_TIMEOUT_MODE_SCOPE_FRAGMENT not in shell_text:
                return fail(f"{case['name']}_shell_timeout_mode_scope_fragment_missing", shell_proc)
            if AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_FRAGMENT not in shell_text:
                return fail(f"{case['name']}_shell_timeout_requires_optin_fragment_missing", shell_proc)
            if AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_FRAGMENT not in shell_text:
                return fail(f"{case['name']}_shell_timeout_policy_reason_fragment_missing", shell_proc)
            if f"age4_proof_snapshot_fields_text={AGE4_PROOF_SNAPSHOT_FIELDS_TEXT}" not in shell_text:
                return fail(f"{case['name']}_shell_age4_proof_snapshot_fields_text_missing", shell_proc)
            if f"age4_proof_source_snapshot_fields_text={AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT}" not in shell_text:
                return fail(f"{case['name']}_shell_age4_proof_source_snapshot_fields_text_missing", shell_proc)
            if (
                "age4_proof_snapshot_text="
                + build_age4_proof_snapshot_text(build_age4_proof_snapshot())
            ) not in shell_text:
                return fail(f"{case['name']}_shell_age4_proof_snapshot_text_missing", shell_proc)
            if "age4_proof_gate_result_snapshot_present=0" not in shell_text:
                return fail(f"{case['name']}_shell_age4_proof_gate_result_snapshot_present_missing", shell_proc)
            if "age4_proof_gate_result_snapshot_parity=0" not in shell_text:
                return fail(f"{case['name']}_shell_age4_proof_gate_result_snapshot_parity_missing", shell_proc)
            if "age4_proof_final_status_parse_snapshot_present=0" not in shell_text:
                return fail(
                    f"{case['name']}_shell_age4_proof_final_status_parse_snapshot_present_missing",
                    shell_proc,
                )
            if "age4_proof_final_status_parse_snapshot_parity=0" not in shell_text:
                return fail(
                    f"{case['name']}_shell_age4_proof_final_status_parse_snapshot_parity_missing",
                    shell_proc,
                )
            if AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT not in shell_text:
                return fail(f"{case['name']}_shell_digest_selftest_default_fragment_missing", shell_proc)
            if AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT not in shell_text:
                return fail(f"{case['name']}_shell_digest_selftest_default_field_fragment_missing", shell_proc)
            if AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_FRAGMENT not in shell_text:
                return fail(f"{case['name']}_shell_digest_selftest_default_text_fragment_missing", shell_proc)
            for fragment in AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FRAGMENTS:
                if fragment not in shell_text:
                    return fail(f"{case['name']}_shell_child_summary_default_transport_fragment_missing", shell_proc)
            for fragment in AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FRAGMENTS:
                if fragment not in shell_text:
                    return fail(f"{case['name']}_shell_contract_fragment_missing", shell_proc)
            for fragment in AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FRAGMENTS:
                if fragment not in shell_text:
                    return fail(f"{case['name']}_shell_full_summary_fragment_missing", shell_proc)
            for fragment in AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FRAGMENTS:
                if fragment not in shell_text:
                    return fail(f"{case['name']}_shell_full_summary_transport_fragment_missing", shell_proc)
            complete_probe("run_shell")
            complete_format("shell")

            start_format("text")
            text_out = ""
            text_proc = subprocess.CompletedProcess(
                [py, script, "--provider", str(case["provider"]), "--format", "text"],
                0,
                "",
                "",
            )
            start_probe("run_text")
            if bool(case.get("verify_text_progress", case["verify_progress"])):
                text_progress = temp_root / f"{case['name']}.text.progress.detjson"
                text_env = dict(env)
                text_env["DDN_AGE5_COMBINED_HEAVY_POLICY_PROGRESS_JSON"] = str(text_progress)
                text_proc = run(
                    [py, script, "--provider", str(case["provider"]), "--format", "text"],
                    root,
                    env=text_env,
                )
                if text_proc.returncode != 0:
                    return fail(f"{case['name']}_text_failed", text_proc)
                text_out = str(text_proc.stdout or "")
                try:
                    text_progress_doc = json.loads(text_progress.read_text(encoding="utf-8"))
                except Exception:
                    return fail(f"{case['name']}_text_progress_invalid")
                if str(text_progress_doc.get("format", "")).strip() != "text":
                    return fail(f"{case['name']}_text_progress_format_mismatch")
                if str(text_progress_doc.get("status", "")).strip() != "pass":
                    return fail(f"{case['name']}_text_progress_status_mismatch")
                if str(text_progress_doc.get("last_completed_stage", "")).strip() != "emit_text":
                    return fail(f"{case['name']}_text_progress_stage_mismatch")
                seen_text_stages = read_progress_stage_history(text_progress_doc)
                expected_text_stages = [
                    "import_contract_module.import_module",
                    "import_contract_module.bind_symbols",
                    "build_text_snapshot",
                    "build_text_source_snapshot",
                    "render_text_line",
                    "emit_text_stdout",
                    "emit_text",
                ]
                for stage in expected_text_stages:
                    if stage not in seen_text_stages:
                        return fail(f"{case['name']}_text_progress_missing_stage_{stage}")
            else:
                _, text_out = run_age5_combined_policy_fast_path(
                    provider=str(case["provider"]),
                    env_patch=dict(case["env"]),
                    output_format="text",
                )
            complete_probe("run_text")
            start_probe("validate_text_policy")
            if AGE5_COMBINED_HEAVY_SCOPE_FRAGMENT not in text_out:
                return fail(f"{case['name']}_text_scope_missing", text_proc)
            if AGE5_COMBINED_HEAVY_REPORTS_FRAGMENT not in text_out:
                return fail(f"{case['name']}_text_reports_fragment_missing", text_proc)
            if AGE5_COMBINED_HEAVY_CRITERIA_FRAGMENT not in text_out:
                return fail(f"{case['name']}_text_criteria_fragment_missing", text_proc)
            if AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS_FRAGMENT not in text_out:
                return fail(f"{case['name']}_text_child_summary_keys_fragment_missing", text_proc)
            if AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_FRAGMENT not in text_out:
                return fail(f"{case['name']}_text_child_summary_default_fields_fragment_missing", text_proc)
            if AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DEFAULT_FRAGMENT not in text_out:
                return fail(f"{case['name']}_text_timeout_mode_default_fragment_missing", text_proc)
            if AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_FRAGMENT not in text_out:
                return fail(f"{case['name']}_text_timeout_mode_allowed_values_fragment_missing", text_proc)
            if AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_FRAGMENT not in text_out:
                return fail(f"{case['name']}_text_timeout_mode_preview_only_fragment_missing", text_proc)
            if AGE5_COMBINED_HEAVY_TIMEOUT_MODE_SCOPE_FRAGMENT not in text_out:
                return fail(f"{case['name']}_text_timeout_mode_scope_fragment_missing", text_proc)
            if AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_FRAGMENT not in text_out:
                return fail(f"{case['name']}_text_timeout_requires_optin_fragment_missing", text_proc)
            if AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_FRAGMENT not in text_out:
                return fail(f"{case['name']}_text_timeout_policy_reason_fragment_missing", text_proc)
            complete_probe("validate_text_policy")
            start_probe("validate_text_age4")
            if f"age4_proof_snapshot_fields_text={AGE4_PROOF_SNAPSHOT_FIELDS_TEXT}" not in text_out:
                return fail(f"{case['name']}_text_age4_proof_snapshot_fields_text_missing", text_proc)
            if f"age4_proof_source_snapshot_fields_text={AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT}" not in text_out:
                return fail(f"{case['name']}_text_age4_proof_source_snapshot_fields_text_missing", text_proc)
            if (
                "age4_proof_snapshot_text="
                + build_age4_proof_snapshot_text(build_age4_proof_snapshot())
            ) not in text_out:
                return fail(f"{case['name']}_text_age4_proof_snapshot_text_missing", text_proc)
            if "age4_proof_gate_result_snapshot_present=0" not in text_out:
                return fail(f"{case['name']}_text_age4_proof_gate_result_snapshot_present_missing", text_proc)
            if "age4_proof_gate_result_snapshot_parity=0" not in text_out:
                return fail(f"{case['name']}_text_age4_proof_gate_result_snapshot_parity_missing", text_proc)
            if "age4_proof_final_status_parse_snapshot_present=0" not in text_out:
                return fail(
                    f"{case['name']}_text_age4_proof_final_status_parse_snapshot_present_missing",
                    text_proc,
                )
            if "age4_proof_final_status_parse_snapshot_parity=0" not in text_out:
                return fail(
                    f"{case['name']}_text_age4_proof_final_status_parse_snapshot_parity_missing",
                    text_proc,
                )
            if AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT not in text_out:
                return fail(f"{case['name']}_text_digest_selftest_default_fragment_missing", text_proc)
            if AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT not in text_out:
                return fail(f"{case['name']}_text_digest_selftest_default_field_fragment_missing", text_proc)
            if AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_FRAGMENT not in text_out:
                return fail(f"{case['name']}_text_digest_selftest_default_text_fragment_missing", text_proc)
            complete_probe("validate_text_age4")
            start_probe("validate_text_summary")
            for fragment in AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FRAGMENTS:
                if fragment not in text_out:
                    return fail(f"{case['name']}_text_child_summary_default_transport_fragment_missing", text_proc)
            for fragment in AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FRAGMENTS:
                if fragment not in text_out:
                    return fail(f"{case['name']}_text_contract_fragment_missing", text_proc)
            for fragment in AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FRAGMENTS:
                if fragment not in text_out:
                    return fail(f"{case['name']}_text_full_summary_fragment_missing", text_proc)
            for fragment in AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FRAGMENTS:
                if fragment not in text_out:
                    return fail(f"{case['name']}_text_full_summary_transport_fragment_missing", text_proc)
            complete_probe("validate_text_summary")
            complete_format("text")
            complete_case(str(case["name"]))

    update_progress("pass")
    print("age5 combined heavy policy selftest ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
