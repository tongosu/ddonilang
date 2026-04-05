#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import sys
import time
from importlib import import_module
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TESTS_DIR = ROOT / "tests"
if str(TESTS_DIR) not in sys.path:
    sys.path.insert(0, str(TESTS_DIR))

PROGRESS_ENV_KEY = "DDN_AGE5_COMBINED_HEAVY_POLICY_PROGRESS_JSON"
SCHEMA = "ddn.ci.age5_combined_heavy_policy.v1"
ENV_KEY = "DDN_AGE5_CLOSE_WITH_COMBINED_HEAVY_RUNTIME_HELPER_CHECK"
PROGRESS_STAGE_SLEEP_MS_ENV_KEY = "DDN_AGE5_COMBINED_HEAVY_POLICY_PROGRESS_STAGE_SLEEP_MS"
_CONTRACT_SYMBOLS_LOADED = False


def _load_contract_symbols(
    update_progress=None,
    complete_stage=None,
) -> None:
    global _CONTRACT_SYMBOLS_LOADED
    global SCHEMA
    global ENV_KEY
    if _CONTRACT_SYMBOLS_LOADED:
        return
    if update_progress is not None:
        update_progress("running", "import_contract_module.import_module")
    contract = import_module("_ci_age5_combined_heavy_contract")
    if complete_stage is not None:
        complete_stage("import_contract_module.import_module")
    if update_progress is not None:
        update_progress("running", "import_contract_module.bind_symbols")
    names = [
        "AGE4_PROOF_SNAPSHOT_FIELDS_TEXT",
        "AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT",
        "AGE5_CLOSE_DIGEST_SELFTEST_OK_DEFAULT",
        "AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT",
        "AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_FRAGMENT",
        "AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY",
        "AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT",
        "AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY",
        "AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_FRAGMENT",
        "AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT",
        "AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS_TEXT",
        "AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FRAGMENTS",
        "AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS",
        "AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS_TEXT",
        "AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS_FRAGMENT",
        "AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FRAGMENTS",
        "AGE5_COMBINED_HEAVY_ENV_KEY",
        "AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT",
        "AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FRAGMENTS",
        "AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FIELDS_TEXT",
        "AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FRAGMENTS",
        "AGE5_COMBINED_HEAVY_MODE",
        "AGE5_COMBINED_HEAVY_POLICY_MARKER",
        "AGE5_COMBINED_HEAVY_POLICY_SCHEMA",
        "AGE5_COMBINED_HEAVY_REPORTS_FRAGMENT",
        "AGE5_COMBINED_HEAVY_REPORT_SCHEMA",
        "AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA",
        "AGE5_COMBINED_HEAVY_REQUIRED_REPORTS",
        "AGE5_COMBINED_HEAVY_CRITERIA_FRAGMENT",
        "AGE5_COMBINED_HEAVY_SCOPE_FRAGMENT",
        "AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_FRAGMENT",
        "AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DEFAULT_FRAGMENT",
        "AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_FRAGMENT",
        "AGE5_COMBINED_HEAVY_TIMEOUT_MODE_SCOPE_FRAGMENT",
        "AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_FRAGMENT",
        "AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_FRAGMENT",
        "AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_FIELDS_TEXT",
        "AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT",
        "build_age4_proof_snapshot",
        "build_age4_proof_snapshot_text",
        "build_age4_proof_source_snapshot_fields",
        "build_age5_combined_heavy_timeout_policy_fields",
        "build_age5_combined_heavy_combined_report_contract_fields",
        "build_age5_combined_heavy_child_summary_default_fields",
        "build_age5_combined_heavy_child_summary_default_text_transport_fields",
        "build_age5_close_digest_selftest_default_field",
        "build_age5_combined_heavy_full_summary_contract_fields",
        "build_age5_combined_heavy_full_summary_text_transport_fields",
    ]
    for name in names:
        globals()[name] = getattr(contract, name)
    SCHEMA = str(getattr(contract, "AGE5_COMBINED_HEAVY_POLICY_SCHEMA"))
    ENV_KEY = str(getattr(contract, "AGE5_COMBINED_HEAVY_ENV_KEY"))
    if complete_stage is not None:
        complete_stage("import_contract_module.bind_symbols")
    _CONTRACT_SYMBOLS_LOADED = True


def truthy(raw: str) -> bool:
    return raw.strip().lower() in {"1", "true", "yes", "on"}


def _parse_provider_format(argv: list[str]) -> tuple[str, str]:
    provider = "-"
    output_format = "text"
    for idx, token in enumerate(argv):
        if token == "--provider" and idx + 1 < len(argv):
            provider = str(argv[idx + 1]).strip() or "-"
        elif token == "--format" and idx + 1 < len(argv):
            output_format = str(argv[idx + 1]).strip() or "text"
    return provider, output_format


def write_progress_snapshot(
    path_text: str,
    *,
    provider: str,
    output_format: str,
    status: str,
    current_stage: str,
    last_completed_stage: str,
    total_elapsed_ms: int,
    stage_history: list[str] | None = None,
) -> None:
    if not str(path_text).strip():
        return
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "ddn.age5_combined_heavy_policy_progress.v1",
        "provider": provider,
        "format": output_format,
        "status": status,
        "current_stage": current_stage,
        "last_completed_stage": last_completed_stage,
        "total_elapsed_ms": int(total_elapsed_ms),
        "stage_history": list(stage_history or []),
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def resolve(provider: str) -> dict[str, object]:
    provider_norm = provider.strip().lower()
    if provider_norm not in {"gitlab", "azure"}:
        raise ValueError(f"unsupported provider: {provider}")

    raw_env = str(os.environ.get(ENV_KEY, "")).strip()
    env_enabled = truthy(raw_env)

    if provider_norm == "gitlab":
        schedule_raw = str(os.environ.get("CI_PIPELINE_SOURCE", "")).strip()
        schedule_enabled = schedule_raw.lower() == "schedule"
        schedule_key = "CI_PIPELINE_SOURCE"
    else:
        schedule_raw = str(os.environ.get("BUILD_REASON", "")).strip()
        schedule_enabled = schedule_raw == "Schedule"
        schedule_key = "BUILD_REASON"

    enabled = bool(env_enabled or schedule_enabled)
    if schedule_enabled:
        reason = "schedule"
    elif env_enabled:
        reason = "manual_optin"
    else:
        reason = "default_off"
    default_age4_proof_snapshot = build_age4_proof_snapshot()
    default_age4_proof_source_snapshot = build_age4_proof_source_snapshot_fields(
        top_snapshot=default_age4_proof_snapshot
    )
    default_timeout_policy_fields = build_age5_combined_heavy_timeout_policy_fields()

    return {
        "schema": SCHEMA,
        "provider": provider_norm,
        "env_key": ENV_KEY,
        "scope": AGE5_COMBINED_HEAVY_MODE,
        "env_raw": raw_env,
        "env_enabled": env_enabled,
        "schedule_env_key": schedule_key,
        "schedule_raw": schedule_raw,
        "schedule_enabled": schedule_enabled,
        "enabled": enabled,
        "reason": reason,
        "combined_report_schema": AGE5_COMBINED_HEAVY_REPORT_SCHEMA,
        "combined_required_reports": list(AGE5_COMBINED_HEAVY_REQUIRED_REPORTS),
        "combined_required_criteria": list(AGE5_COMBINED_HEAVY_REQUIRED_CRITERIA),
        "combined_child_summary_keys": list(AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS),
        "combined_child_summary_keys_text": AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS_TEXT,
        "combined_child_summary_default_fields": build_age5_combined_heavy_child_summary_default_fields(),
        "combined_child_summary_default_fields_text": AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_TEXT,
        "combined_timeout_policy_fields": default_timeout_policy_fields,
        **default_timeout_policy_fields,
        "combined_timeout_policy_fields_text": AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_FIELDS_TEXT,
        **default_age4_proof_snapshot,
        **default_age4_proof_source_snapshot,
        "age4_proof_snapshot_fields_text": AGE4_PROOF_SNAPSHOT_FIELDS_TEXT,
        "age4_proof_source_snapshot_fields_text": AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT,
        "age4_proof_snapshot_text": build_age4_proof_snapshot_text(default_age4_proof_snapshot),
        AGE5_CLOSE_DIGEST_SELFTEST_OK_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_DEFAULT,
        "combined_digest_selftest_default_field": build_age5_close_digest_selftest_default_field(),
        AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_KEY: AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT,
        "combined_child_summary_default_text_transport_fields": (
            build_age5_combined_heavy_child_summary_default_text_transport_fields()
        ),
        "combined_child_summary_default_text_transport_fields_text": (
            AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FIELDS_TEXT
        ),
        "combined_contract_summary_fields": build_age5_combined_heavy_combined_report_contract_fields(),
        "combined_contract_summary_fields_text": AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FIELDS_TEXT,
        "combined_full_summary_contract_fields": build_age5_combined_heavy_full_summary_contract_fields(),
        "combined_full_summary_contract_fields_text": AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FIELDS_TEXT,
        "combined_full_summary_text_transport_fields": build_age5_combined_heavy_full_summary_text_transport_fields(),
        "combined_full_summary_text_transport_fields_text": AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FIELDS_TEXT,
    }


def render_shell(payload: dict[str, object]) -> str:
    enabled = "1" if bool(payload.get("enabled", False)) else "0"
    provider = str(payload.get("provider", "")).strip()
    reason = str(payload.get("reason", "")).strip()
    (
        age4_proof_snapshot_fields_text,
        age4_proof_snapshot,
        age4_proof_snapshot_text,
    ) = build_text_payload_fields(payload)
    (
        age4_proof_source_snapshot_fields_text,
        age4_proof_gate_result_snapshot_present,
        age4_proof_gate_result_snapshot_parity,
        age4_proof_final_status_parse_snapshot_present,
        age4_proof_final_status_parse_snapshot_parity,
    ) = build_text_source_fields(payload, age4_proof_snapshot)
    return render_shell_text(
        enabled=enabled,
        provider=provider,
        reason=reason,
        age4_proof_snapshot_fields_text=age4_proof_snapshot_fields_text,
        age4_proof_snapshot_text=age4_proof_snapshot_text,
        age4_proof_source_snapshot_fields_text=age4_proof_source_snapshot_fields_text,
        age4_proof_gate_result_snapshot_present=age4_proof_gate_result_snapshot_present,
        age4_proof_gate_result_snapshot_parity=age4_proof_gate_result_snapshot_parity,
        age4_proof_final_status_parse_snapshot_present=age4_proof_final_status_parse_snapshot_present,
        age4_proof_final_status_parse_snapshot_parity=age4_proof_final_status_parse_snapshot_parity,
    )


def render_shell_text(
    *,
    enabled: str,
    provider: str,
    reason: str,
    age4_proof_snapshot_fields_text: str,
    age4_proof_snapshot_text: str,
    age4_proof_source_snapshot_fields_text: str,
    age4_proof_gate_result_snapshot_present: str,
    age4_proof_gate_result_snapshot_parity: str,
    age4_proof_final_status_parse_snapshot_present: str,
    age4_proof_final_status_parse_snapshot_parity: str,
) -> str:
    contract_fragments = " ".join(AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FRAGMENTS)
    full_summary_fragments = " ".join(AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FRAGMENTS)
    full_summary_transport_fragments = " ".join(AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FRAGMENTS)
    child_summary_default_transport_fragments = " ".join(
        AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FRAGMENTS
    )
    timeout_default_fragment = AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DEFAULT_FRAGMENT
    timeout_allowed_fragment = AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_FRAGMENT
    timeout_preview_fragment = AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_FRAGMENT
    timeout_scope_fragment = AGE5_COMBINED_HEAVY_TIMEOUT_MODE_SCOPE_FRAGMENT
    return "\n".join(
        [
            f"export {ENV_KEY}={enabled}",
            (
                f'if [ "${{{ENV_KEY}:-0}}" = "1" ]; then echo "{AGE5_COMBINED_HEAVY_POLICY_MARKER} '
                f'provider={provider} reason={reason} {AGE5_COMBINED_HEAVY_SCOPE_FRAGMENT} '
                f'{AGE5_COMBINED_HEAVY_REPORTS_FRAGMENT} {AGE5_COMBINED_HEAVY_CRITERIA_FRAGMENT} '
                f'{AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS_FRAGMENT} '
                f'{AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_FRAGMENT} '
                f'{timeout_default_fragment} {timeout_allowed_fragment} {timeout_preview_fragment} {timeout_scope_fragment} '
                f'{AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_FRAGMENT} {AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_FRAGMENT} '
                f'age4_proof_snapshot_fields_text={age4_proof_snapshot_fields_text} '
                f'age4_proof_source_snapshot_fields_text={age4_proof_source_snapshot_fields_text} '
                f'age4_proof_snapshot_text={age4_proof_snapshot_text} '
                f'age4_proof_gate_result_snapshot_present={age4_proof_gate_result_snapshot_present} '
                f'age4_proof_gate_result_snapshot_parity={age4_proof_gate_result_snapshot_parity} '
                f'age4_proof_final_status_parse_snapshot_present={age4_proof_final_status_parse_snapshot_present} '
                f'age4_proof_final_status_parse_snapshot_parity={age4_proof_final_status_parse_snapshot_parity} '
                f'{AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT} '
                f'{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT} '
                f'{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_FRAGMENT} '
                f'{child_summary_default_transport_fragments} '
                f'{contract_fragments} {full_summary_fragments} {full_summary_transport_fragments}"; fi'
            ),
        ]
    )


def build_text_payload_fields(payload: dict[str, object]) -> tuple[str, dict[str, str], str]:
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
    return age4_proof_snapshot_fields_text, age4_proof_snapshot, age4_proof_snapshot_text


def build_text_source_fields(
    payload: dict[str, object], age4_proof_snapshot: dict[str, str]
) -> tuple[str, str, str, str, str]:
    age4_proof_source_snapshot_fields = build_age4_proof_source_snapshot_fields(
        top_snapshot=age4_proof_snapshot
    )
    age4_proof_source_snapshot_fields_text = (
        str(payload.get("age4_proof_source_snapshot_fields_text", AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT)).strip()
        or AGE4_PROOF_SOURCE_SNAPSHOT_FIELDS_TEXT
    )
    age4_proof_gate_result_snapshot_present = (
        str(
            payload.get(
                "age4_proof_gate_result_snapshot_present",
                age4_proof_source_snapshot_fields["age4_proof_gate_result_snapshot_present"],
            )
        ).strip()
        or age4_proof_source_snapshot_fields["age4_proof_gate_result_snapshot_present"]
    )
    age4_proof_gate_result_snapshot_parity = (
        str(
            payload.get(
                "age4_proof_gate_result_snapshot_parity",
                age4_proof_source_snapshot_fields["age4_proof_gate_result_snapshot_parity"],
            )
        ).strip()
        or age4_proof_source_snapshot_fields["age4_proof_gate_result_snapshot_parity"]
    )
    age4_proof_final_status_parse_snapshot_present = (
        str(
            payload.get(
                "age4_proof_final_status_parse_snapshot_present",
                age4_proof_source_snapshot_fields["age4_proof_final_status_parse_snapshot_present"],
            )
        ).strip()
        or age4_proof_source_snapshot_fields["age4_proof_final_status_parse_snapshot_present"]
    )
    age4_proof_final_status_parse_snapshot_parity = (
        str(
            payload.get(
                "age4_proof_final_status_parse_snapshot_parity",
                age4_proof_source_snapshot_fields["age4_proof_final_status_parse_snapshot_parity"],
            )
        ).strip()
        or age4_proof_source_snapshot_fields["age4_proof_final_status_parse_snapshot_parity"]
    )
    return (
        age4_proof_source_snapshot_fields_text,
        age4_proof_gate_result_snapshot_present,
        age4_proof_gate_result_snapshot_parity,
        age4_proof_final_status_parse_snapshot_present,
        age4_proof_final_status_parse_snapshot_parity,
    )


def render_text_line(
    payload: dict[str, object],
    *,
    age4_proof_snapshot_fields_text: str,
    age4_proof_snapshot_text: str,
    age4_proof_source_snapshot_fields_text: str,
    age4_proof_gate_result_snapshot_present: str,
    age4_proof_gate_result_snapshot_parity: str,
    age4_proof_final_status_parse_snapshot_present: str,
    age4_proof_final_status_parse_snapshot_parity: str,
) -> str:
    status = "enabled" if bool(payload.get("enabled", False)) else "disabled"
    return (
        "[age5-combined-heavy-policy] "
        f"provider={payload['provider']} status={status} reason={payload['reason']} "
        f"{AGE5_COMBINED_HEAVY_SCOPE_FRAGMENT} "
        f"{AGE5_COMBINED_HEAVY_REPORTS_FRAGMENT} "
        f"{AGE5_COMBINED_HEAVY_CRITERIA_FRAGMENT} "
        f"{AGE5_COMBINED_HEAVY_CHILD_SUMMARY_KEYS_FRAGMENT} "
        f"{AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_FIELDS_FRAGMENT} "
        f"{AGE5_COMBINED_HEAVY_TIMEOUT_MODE_DEFAULT_FRAGMENT} "
        f"{AGE5_COMBINED_HEAVY_TIMEOUT_MODE_ALLOWED_VALUES_FRAGMENT} "
        f"{AGE5_COMBINED_HEAVY_TIMEOUT_MODE_PREVIEW_ONLY_FRAGMENT} "
        f"{AGE5_COMBINED_HEAVY_TIMEOUT_MODE_SCOPE_FRAGMENT} "
        f"{AGE5_COMBINED_HEAVY_TIMEOUT_REQUIRES_OPTIN_FRAGMENT} "
        f"{AGE5_COMBINED_HEAVY_TIMEOUT_POLICY_REASON_FRAGMENT} "
        f"age4_proof_snapshot_fields_text={age4_proof_snapshot_fields_text} "
        f"age4_proof_source_snapshot_fields_text={age4_proof_source_snapshot_fields_text} "
        f"age4_proof_snapshot_text={age4_proof_snapshot_text} "
        f"age4_proof_gate_result_snapshot_present={age4_proof_gate_result_snapshot_present} "
        f"age4_proof_gate_result_snapshot_parity={age4_proof_gate_result_snapshot_parity} "
        f"age4_proof_final_status_parse_snapshot_present={age4_proof_final_status_parse_snapshot_present} "
        f"age4_proof_final_status_parse_snapshot_parity={age4_proof_final_status_parse_snapshot_parity} "
        f"{AGE5_CLOSE_DIGEST_SELFTEST_OK_FRAGMENT} "
        f"{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_FRAGMENT} "
        f"{AGE5_CLOSE_DIGEST_SELFTEST_DEFAULT_FIELD_TEXT_FRAGMENT} "
        f"{' '.join(AGE5_COMBINED_HEAVY_CHILD_SUMMARY_DEFAULT_TEXT_TRANSPORT_FRAGMENTS)} "
        f"{' '.join(AGE5_COMBINED_HEAVY_COMBINED_REPORT_CONTRACT_FRAGMENTS)} "
        f"{' '.join(AGE5_COMBINED_HEAVY_FULL_SUMMARY_CONTRACT_FRAGMENTS)} "
        f"{' '.join(AGE5_COMBINED_HEAVY_FULL_SUMMARY_TEXT_TRANSPORT_FRAGMENTS)}"
    )


def main() -> int:
    bootstrap_provider, bootstrap_format = _parse_provider_format(sys.argv[1:])
    bootstrap_stage_history = ["parse_args.start"]
    write_progress_snapshot(
        str(os.environ.get(PROGRESS_ENV_KEY, "")).strip(),
        provider=bootstrap_provider,
        output_format=bootstrap_format,
        status="running",
        current_stage="parse_args.start",
        last_completed_stage="-",
        total_elapsed_ms=0,
        stage_history=bootstrap_stage_history,
    )
    parser = argparse.ArgumentParser(description="Resolve AGE5 combined heavy policy for CI providers")
    parser.add_argument("--provider", required=True, choices=("gitlab", "azure"))
    parser.add_argument("--format", default="text", choices=("text", "json", "shell"))
    parser.add_argument("--json-out", default="", help="optional policy report output path")
    args = parser.parse_args()

    progress_path = str(os.environ.get(PROGRESS_ENV_KEY, "")).strip()
    progress_stage_sleep_ms = int(str(os.environ.get(PROGRESS_STAGE_SLEEP_MS_ENV_KEY, "0")).strip() or "0")
    started = time.perf_counter()
    last_completed_stage = "-"
    stage_history: list[str] = []

    def record_stage(stage: str) -> None:
        normalized = str(stage).strip() or "-"
        if normalized == "-":
            return
        if stage_history and stage_history[-1] == normalized:
            return
        stage_history.append(normalized)

    write_progress_snapshot(
        progress_path,
        provider=args.provider,
        output_format=args.format,
        status="running",
        current_stage="progress_setup",
        last_completed_stage=last_completed_stage,
        total_elapsed_ms=0,
        stage_history=stage_history,
    )

    def maybe_sleep_progress_stage() -> None:
        if progress_stage_sleep_ms > 0:
            time.sleep(progress_stage_sleep_ms / 1000.0)

    def update_progress(status: str, current_stage: str) -> None:
        record_stage(current_stage)
        write_progress_snapshot(
            progress_path,
            provider=args.provider,
            output_format=args.format,
            status=status,
            current_stage=current_stage,
            last_completed_stage=last_completed_stage,
            total_elapsed_ms=int((time.perf_counter() - started) * 1000),
            stage_history=stage_history,
        )
        maybe_sleep_progress_stage()

    def complete_stage(stage: str) -> None:
        nonlocal last_completed_stage
        last_completed_stage = stage
        record_stage(stage)
        write_progress_snapshot(
            progress_path,
            provider=args.provider,
            output_format=args.format,
            status="running",
            current_stage="-",
            last_completed_stage=last_completed_stage,
            total_elapsed_ms=int((time.perf_counter() - started) * 1000),
            stage_history=stage_history,
        )
        maybe_sleep_progress_stage()

    complete_stage("parse_args.done")
    update_progress("running", "progress_setup")
    complete_stage("progress_setup")
    update_progress("running", "load_contract_symbols.start")
    _load_contract_symbols(
        update_progress=update_progress,
        complete_stage=complete_stage,
    )
    update_progress("running", "resolve")
    payload = resolve(args.provider)
    complete_stage("resolve")
    if args.json_out:
        update_progress("running", "write_json_out")
        out_path = Path(args.json_out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        complete_stage("write_json_out")

    if args.format == "json":
        update_progress("running", "emit_json")
        print(json.dumps(payload, ensure_ascii=False))
        complete_stage("emit_json")
    elif args.format == "shell":
        update_progress("running", "build_shell_snapshot")
        (
            age4_proof_snapshot_fields_text,
            age4_proof_snapshot,
            age4_proof_snapshot_text,
        ) = build_text_payload_fields(payload)
        complete_stage("build_shell_snapshot")
        update_progress("running", "build_shell_source_snapshot")
        (
            age4_proof_source_snapshot_fields_text,
            age4_proof_gate_result_snapshot_present,
            age4_proof_gate_result_snapshot_parity,
            age4_proof_final_status_parse_snapshot_present,
            age4_proof_final_status_parse_snapshot_parity,
        ) = build_text_source_fields(payload, age4_proof_snapshot)
        complete_stage("build_shell_source_snapshot")
        update_progress("running", "render_shell_text")
        shell_text = render_shell_text(
            enabled="1" if bool(payload.get("enabled", False)) else "0",
            provider=str(payload.get("provider", "")).strip(),
            reason=str(payload.get("reason", "")).strip(),
            age4_proof_snapshot_fields_text=age4_proof_snapshot_fields_text,
            age4_proof_snapshot_text=age4_proof_snapshot_text,
            age4_proof_source_snapshot_fields_text=age4_proof_source_snapshot_fields_text,
            age4_proof_gate_result_snapshot_present=age4_proof_gate_result_snapshot_present,
            age4_proof_gate_result_snapshot_parity=age4_proof_gate_result_snapshot_parity,
            age4_proof_final_status_parse_snapshot_present=age4_proof_final_status_parse_snapshot_present,
            age4_proof_final_status_parse_snapshot_parity=age4_proof_final_status_parse_snapshot_parity,
        )
        complete_stage("render_shell_text")
        update_progress("running", "emit_shell_stdout")
        print(shell_text)
        complete_stage("emit_shell_stdout")
        complete_stage("emit_shell")
    else:
        update_progress("running", "build_text_snapshot")
        (
            age4_proof_snapshot_fields_text,
            age4_proof_snapshot,
            age4_proof_snapshot_text,
        ) = build_text_payload_fields(payload)
        complete_stage("build_text_snapshot")
        update_progress("running", "build_text_source_snapshot")
        (
            age4_proof_source_snapshot_fields_text,
            age4_proof_gate_result_snapshot_present,
            age4_proof_gate_result_snapshot_parity,
            age4_proof_final_status_parse_snapshot_present,
            age4_proof_final_status_parse_snapshot_parity,
        ) = build_text_source_fields(payload, age4_proof_snapshot)
        complete_stage("build_text_source_snapshot")
        update_progress("running", "render_text_line")
        text_line = render_text_line(
            payload,
            age4_proof_snapshot_fields_text=age4_proof_snapshot_fields_text,
            age4_proof_snapshot_text=age4_proof_snapshot_text,
            age4_proof_source_snapshot_fields_text=age4_proof_source_snapshot_fields_text,
            age4_proof_gate_result_snapshot_present=age4_proof_gate_result_snapshot_present,
            age4_proof_gate_result_snapshot_parity=age4_proof_gate_result_snapshot_parity,
            age4_proof_final_status_parse_snapshot_present=age4_proof_final_status_parse_snapshot_present,
            age4_proof_final_status_parse_snapshot_parity=age4_proof_final_status_parse_snapshot_parity,
        )
        complete_stage("render_text_line")
        update_progress("running", "emit_text_stdout")
        print(text_line)
        complete_stage("emit_text_stdout")
        complete_stage("emit_text")
    write_progress_snapshot(
        progress_path,
        provider=args.provider,
        output_format=args.format,
        status="pass",
        current_stage="-",
        last_completed_stage=last_completed_stage,
        total_elapsed_ms=int((time.perf_counter() - started) * 1000),
        stage_history=stage_history,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
