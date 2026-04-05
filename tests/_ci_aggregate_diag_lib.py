from __future__ import annotations

import json
from collections.abc import Callable
from datetime import datetime, timezone
from pathlib import Path

from _ci_age3_completion_gate_contract import (
    AGE3_COMPLETION_GATE_CRITERIA_NAMES,
    age3_completion_gate_criteria_summary_key,
    age3_completion_gate_criteria_sync_summary_key,
)
from _ci_aggregate_diag_specs import (
    CI_SANITY_DEFAULT_SNAPSHOT,
    CI_SANITY_SNAPSHOT_FIELD_SPECS,
    CI_SANITY_STEP_OK_SPECS,
    CI_SANITY_SUMMARY_LINE_SPECS,
    CI_SYNC_READINESS_DEFAULT_SNAPSHOT,
    CI_SYNC_READINESS_SNAPSHOT_FIELD_SPECS,
    CI_SYNC_READINESS_SUMMARY_LINE_SPECS,
    CONTROL_EXPOSURE_MORE_RE,
    CONTROL_EXPOSURE_POLICY_DEFAULT_SNAPSHOT,
    CONTROL_EXPOSURE_POLICY_SNAPSHOT_FIELD_SPECS,
    CONTROL_EXPOSURE_POLICY_SUMMARY_LINE_SPECS,
    CONTROL_EXPOSURE_VIOLATION_RE,
    FIXED64_THREEWAY_DEFAULT_SNAPSHOT,
    FIXED64_THREEWAY_SNAPSHOT_FIELD_SPECS,
    FIXED64_THREEWAY_SUMMARY_LINE_SPECS,
    PROFILE_MATRIX_AGGREGATE_SUMMARY_VALUE_KEYS,
    PROFILE_MATRIX_DEFAULT_SNAPSHOT,
    PROFILE_MATRIX_SNAPSHOT_FIELD_SPECS,
    PROFILE_MATRIX_SUMMARY_LINE_SPECS,
    REWRITE_OVERLAY_REPORT_DEFAULT_SNAPSHOT,
    REWRITE_OVERLAY_REPORT_SNAPSHOT_FIELD_SPECS,
    REWRITE_OVERLAY_REPORT_SUMMARY_LINE_SPECS,
    RUNTIME5_CHECKLIST_DEFAULT_SNAPSHOT,
    RUNTIME5_CHECKLIST_ITEMS_MISSING_SNAPSHOT,
    RUNTIME5_CHECKLIST_ROW_SPECS,
    RUNTIME5_CHECKLIST_SNAPSHOT_FIELD_SPECS,
    RUNTIME5_CHECKLIST_SUMMARY_LINE_SPECS,
    SEAMGRIM_FOCUS_STEP_SPECS,
    SEAMGRIM_STEP_DEFAULT_SNAPSHOT,
    SEAMGRIM_STEP_SNAPSHOT_FIELD_SPECS,
    SEAMGRIM_STEP_SUMMARY_LINE_SPECS,
)


def load_payload(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def clip_line(text: str, limit: int = 140) -> str:
    normalized = " ".join(text.split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


def read_compact_line(path: Path, limit: int = 220) -> str:
    try:
        text = path.read_text(encoding="utf-8").strip()
    except Exception:
        return "-"
    if not text:
        return "-"
    return clip_line(text, limit)


def read_compact_from_parse(parse_path: Path, fallback: str) -> str:
    payload = load_payload(parse_path)
    if isinstance(payload, dict):
        compact = str(payload.get("compact_line", "")).strip()
        if compact:
            return clip_line(compact, 220)
    return fallback


def append_summary_lines(
    lines: list[str],
    report_key: str,
    report_path: Path,
    snapshot: dict[str, str],
    specs: tuple[tuple[str, str], ...],
) -> None:
    lines.append(f"[ci-gate-summary] {report_key}={report_path}")
    for summary_key, snapshot_key in specs:
        lines.append(f"[ci-gate-summary] {summary_key}={snapshot[snapshot_key]}")


def append_prefixed_summary_lines(
    lines: list[str],
    prefix: str,
    snapshot: dict[str, str],
    specs: tuple[tuple[str, str], ...],
) -> None:
    for summary_suffix, snapshot_key in specs:
        lines.append(f"[ci-gate-summary] {prefix}_{summary_suffix}={snapshot[snapshot_key]}")


def build_identity_field_specs(*keys: str) -> tuple[tuple[str, str], ...]:
    return tuple((key, key) for key in keys)


def build_snapshot(
    default_snapshot: dict[str, str],
    field_specs: tuple[tuple[str, str], ...],
    resolver: Callable[[str, str], str],
) -> dict[str, str]:
    snapshot = dict(default_snapshot)
    for output_key, source_key in field_specs:
        snapshot[output_key] = str(resolver(output_key, source_key))
    return snapshot


def build_typed_snapshot(
    default_snapshot: dict[str, str],
    field_specs: tuple[tuple[str, str, str, str], ...],
    resolver: Callable[[str, str, str, str], str],
) -> dict[str, str]:
    snapshot = dict(default_snapshot)
    for output_key, field_kind, source_key, default_value in field_specs:
        snapshot[output_key] = str(resolver(output_key, field_kind, source_key, str(default_value)))
    return snapshot


def get_seamgrim_step(report_path: Path, step_name: str) -> tuple[dict | None, str]:
    doc = load_payload(report_path)
    if not isinstance(doc, dict):
        return None, "missing_report"
    steps = doc.get("steps")
    if not isinstance(steps, list):
        return None, "steps_missing"
    for row in steps:
        if not isinstance(row, dict):
            continue
        if str(row.get("name", "")).strip() == step_name:
            return row, "ok"
    return None, "step_missing"


def load_seamgrim_step_snapshot(report_path: Path, step_name: str) -> dict[str, str]:
    step, status = get_seamgrim_step(report_path, step_name)
    if not isinstance(step, dict):
        resolved_snapshot = {
            **SEAMGRIM_STEP_DEFAULT_SNAPSHOT,
            "status": status,
            "detail": {
                "missing_report": "seamgrim report missing or invalid",
                "steps_missing": "seamgrim report has no steps",
                "step_missing": f"{step_name} step missing",
            }.get(status, "step unavailable"),
        }
        return build_snapshot(
            SEAMGRIM_STEP_DEFAULT_SNAPSHOT,
            SEAMGRIM_STEP_SNAPSHOT_FIELD_SPECS,
            lambda _, source_key: resolved_snapshot[source_key],
        )

    ok_text = "1" if bool(step.get("ok", False)) else "0"
    diagnostics = step.get("diagnostics")
    diag_rows = diagnostics if isinstance(diagnostics, list) else []
    detail = ""
    if diag_rows:
        first = diag_rows[0] if isinstance(diag_rows[0], dict) else {}
        detail = str(first.get("detail", "")).strip()
    if not detail and ok_text != "1":
        stderr_text = str(step.get("stderr", "")).strip()
        stdout_text = str(step.get("stdout", "")).strip()
        detail = stderr_text or stdout_text
    if not detail:
        detail = "-"
    resolved_snapshot = {
        "status": "ok" if ok_text == "1" else "failed",
        "ok": ok_text,
        "diag_count": str(len(diag_rows)),
        "detail": clip_line(detail, 200),
    }
    return build_snapshot(
        SEAMGRIM_STEP_DEFAULT_SNAPSHOT,
        SEAMGRIM_STEP_SNAPSHOT_FIELD_SPECS,
        lambda _, source_key: resolved_snapshot[source_key],
    )


def parse_control_exposure_violations(step: dict) -> tuple[list[dict[str, str]], int]:
    sources: list[str] = []
    diagnostics = step.get("diagnostics")
    if isinstance(diagnostics, list):
        for row in diagnostics:
            if isinstance(row, dict):
                text = str(row.get("detail", "")).strip()
                if text:
                    sources.append(text)
    stdout_text = str(step.get("stdout", "")).strip()
    if stdout_text:
        sources.append(stdout_text)
    stderr_text = str(step.get("stderr", "")).strip()
    if stderr_text:
        sources.append(stderr_text)

    dedup: set[tuple[str, str, str]] = set()
    overflow_count = 0
    for text in sources:
        for match in CONTROL_EXPOSURE_VIOLATION_RE.finditer(text):
            kind = str(match.group("kind")).strip()
            file_path = str(match.group("file")).strip()
            name = str(match.group("name")).strip()
            if kind and file_path and name:
                dedup.add((kind, file_path, name))
        more_match = CONTROL_EXPOSURE_MORE_RE.search(text)
        if more_match:
            try:
                overflow_count = max(overflow_count, max(0, int(more_match.group("count"))))
            except Exception:
                pass

    rows = [
        {"kind": kind, "file": file_path, "name": name}
        for (kind, file_path, name) in sorted(dedup, key=lambda item: (item[1], item[0], item[2]))
    ]
    return rows, overflow_count


def write_control_exposure_failure_report(report_path: Path, seamgrim_report_path: Path) -> dict[str, str]:
    report_path.parent.mkdir(parents=True, exist_ok=True)
    step, status = get_seamgrim_step(seamgrim_report_path, "control_exposure_policy")
    payload: dict[str, object] = {
        "schema": "seamgrim.control_exposure_policy.failures.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "source_report": str(seamgrim_report_path),
        "status": status,
        "step_ok": False,
        "violation_count": 0,
        "overflow_count": 0,
        "violations": [],
    }
    if isinstance(step, dict):
        violations, overflow_count = parse_control_exposure_violations(step)
        step_ok = bool(step.get("ok", False))
        payload["step_ok"] = step_ok
        payload["status"] = "ok" if step_ok and not violations else "failed"
        payload["violation_count"] = len(violations) + overflow_count
        payload["overflow_count"] = overflow_count
        payload["violations"] = violations
        diagnostics = step.get("diagnostics")
        if isinstance(diagnostics, list):
            payload["diagnostics"] = diagnostics

    report_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    top = "-"
    violations = payload.get("violations")
    if isinstance(violations, list) and violations:
        parts: list[str] = []
        for row in violations[:5]:
            if not isinstance(row, dict):
                continue
            parts.append(f"{row.get('kind', '-')}:" f"{row.get('file', '-')}:" f"{row.get('name', '-')}")
        if parts:
            top = clip_line(", ".join(parts), 220)
    violation_count = payload.get("violation_count", 0)
    resolved_snapshot = {
        "status": str(payload.get("status", status)).strip() or status,
        "step_ok": "1" if bool(payload.get("step_ok", False)) else "0",
        "violation_count": str(max(0, int(violation_count))) if str(violation_count).strip() else "0",
        "top": top,
    }
    return build_snapshot(
        CONTROL_EXPOSURE_POLICY_DEFAULT_SNAPSHOT,
        CONTROL_EXPOSURE_POLICY_SNAPSHOT_FIELD_SPECS,
        lambda _, source_key: resolved_snapshot[source_key],
    )


def load_rewrite_overlay_report_snapshot(report_path: Path) -> dict[str, str]:
    doc = load_payload(report_path)
    if not isinstance(doc, dict):
        return dict(REWRITE_OVERLAY_REPORT_DEFAULT_SNAPSHOT)
    ok_text = "1" if bool(doc.get("ok", False)) else "0"
    issue_count_raw = doc.get("issue_count", 0)
    try:
        issue_count = max(0, int(issue_count_raw))
    except Exception:
        issue_count = 0
    issues = doc.get("issues")
    top = "-"
    if isinstance(issues, list) and issues:
        parts: list[str] = []
        for row in issues[:5]:
            if not isinstance(row, dict):
                continue
            lesson_id = str(row.get("lesson_id", "")).strip() or "?"
            code = str(row.get("code", "")).strip() or "?"
            parts.append(f"{lesson_id}:{code}")
        if parts:
            top = clip_line(", ".join(parts), 220)
    resolved_snapshot = {
        "status": "ok" if ok_text == "1" else "failed",
        "ok": ok_text,
        "violation_count": str(issue_count),
        "top": top,
    }
    return build_snapshot(
        REWRITE_OVERLAY_REPORT_DEFAULT_SNAPSHOT,
        REWRITE_OVERLAY_REPORT_SNAPSHOT_FIELD_SPECS,
        lambda _, source_key: resolved_snapshot[source_key],
    )


def append_seamgrim_focus_summary_lines(
    lines: list[str],
    seamgrim_report: Path,
    control_exposure_report: Path,
    rewrite_overlay_quality_report: Path,
    control_exposure_snapshot: dict[str, str] | None = None,
    rewrite_overlay_snapshot: dict[str, str] | None = None,
) -> dict[str, str]:
    snapshot = control_exposure_snapshot or write_control_exposure_failure_report(
        control_exposure_report,
        seamgrim_report,
    )
    rewrite_snapshot = rewrite_overlay_snapshot or load_rewrite_overlay_report_snapshot(rewrite_overlay_quality_report)
    append_summary_lines(
        lines,
        "seamgrim_control_exposure_policy_report",
        control_exposure_report,
        snapshot,
        CONTROL_EXPOSURE_POLICY_SUMMARY_LINE_SPECS,
    )
    append_summary_lines(
        lines,
        "seamgrim_rewrite_overlay_quality_report",
        rewrite_overlay_quality_report,
        rewrite_snapshot,
        REWRITE_OVERLAY_REPORT_SUMMARY_LINE_SPECS,
    )
    for summary_prefix, step_name in SEAMGRIM_FOCUS_STEP_SPECS:
        append_prefixed_summary_lines(
            lines,
            summary_prefix,
            load_seamgrim_step_snapshot(seamgrim_report, step_name),
            SEAMGRIM_STEP_SUMMARY_LINE_SPECS,
        )
    return snapshot


def load_fixed64_threeway_snapshot(report_path: Path) -> dict[str, str]:
    doc = load_payload(report_path)
    if not isinstance(doc, dict):
        return dict(FIXED64_THREEWAY_DEFAULT_SNAPSHOT)
    return build_snapshot(
        FIXED64_THREEWAY_DEFAULT_SNAPSHOT,
        FIXED64_THREEWAY_SNAPSHOT_FIELD_SPECS,
        lambda _, source_key: (
            str(doc.get(source_key, "")).strip() or "unknown"
            if source_key == "status"
            else ("1" if bool(doc.get(source_key, False)) else "0")
            if source_key == "ok"
            else clip_line(str(doc.get(source_key, "-")).strip() or "-", 200)
        ),
    )


def append_fixed64_threeway_summary_lines(lines: list[str], report_path: Path) -> None:
    snap = load_fixed64_threeway_snapshot(report_path)
    append_summary_lines(
        lines,
        "fixed64_threeway_report",
        report_path,
        snap,
        FIXED64_THREEWAY_SUMMARY_LINE_SPECS,
    )


def load_ci_sanity_snapshot(report_path: Path) -> dict[str, str]:
    doc = load_payload(report_path)
    if not isinstance(doc, dict):
        return dict(CI_SANITY_DEFAULT_SNAPSHOT)
    status = str(doc.get("status", "")).strip() or "unknown"
    code = str(doc.get("code", "")).strip() or "-"
    step = str(doc.get("step", "")).strip() or "-"
    profile = str(doc.get("profile", "")).strip() or "full"
    msg = clip_line(str(doc.get("msg", "")).strip() or "-", 200)
    pipeline_emit_flags_ok = str(doc.get("ci_sanity_pipeline_emit_flags_ok", "")).strip() or "0"
    pipeline_emit_flags_selftest_ok = str(doc.get("ci_sanity_pipeline_emit_flags_selftest_ok", "")).strip() or "0"
    age2_completion_gate_ok = str(doc.get("ci_sanity_age2_completion_gate_ok", "")).strip() or "0"
    age2_completion_gate_selftest_ok = (
        str(doc.get("ci_sanity_age2_completion_gate_selftest_ok", "")).strip() or "0"
    )
    age2_close_ok = str(doc.get("ci_sanity_age2_close_ok", "")).strip() or "0"
    age2_close_selftest_ok = str(doc.get("ci_sanity_age2_close_selftest_ok", "")).strip() or "0"
    age3_completion_gate_ok = str(doc.get("ci_sanity_age3_completion_gate_ok", "")).strip() or "0"
    age3_completion_gate_selftest_ok = (
        str(doc.get("ci_sanity_age3_completion_gate_selftest_ok", "")).strip() or "0"
    )
    age3_close_ok = str(doc.get("ci_sanity_age3_close_ok", "")).strip() or "0"
    age3_close_selftest_ok = str(doc.get("ci_sanity_age3_close_selftest_ok", "")).strip() or "0"
    age2_completion_gate_failure_codes = (
        str(doc.get("ci_sanity_age2_completion_gate_failure_codes", "")).strip() or "-"
    )
    age2_completion_gate_failure_code_count = (
        str(doc.get("ci_sanity_age2_completion_gate_failure_code_count", "")).strip() or "0"
    )
    age3_completion_gate_failure_codes = (
        str(doc.get("ci_sanity_age3_completion_gate_failure_codes", "")).strip() or "-"
    )
    age3_completion_gate_failure_code_count = (
        str(doc.get("ci_sanity_age3_completion_gate_failure_code_count", "")).strip() or "0"
    )
    age3_bogae_geoul_visibility_smoke_ok = (
        str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_ok", "")).strip() or "0"
    )
    age3_bogae_geoul_visibility_smoke_report_path = (
        str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_report_path", "")).strip() or "-"
    )
    age3_bogae_geoul_visibility_smoke_report_exists = (
        str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists", "")).strip() or "0"
    )
    age3_bogae_geoul_visibility_smoke_schema = (
        str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_schema", "")).strip() or "-"
    )
    age3_bogae_geoul_visibility_smoke_overall_ok = (
        str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok", "")).strip() or "0"
    )
    age3_bogae_geoul_visibility_smoke_checks_ok = (
        str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok", "")).strip() or "0"
    )
    age3_bogae_geoul_visibility_smoke_sim_state_hash_changes = (
        str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes", "")).strip() or "0"
    )
    age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes = (
        str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes", "")).strip() or "0"
    )
    seamgrim_wasm_web_step_check_ok = (
        str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_ok", "")).strip() or "0"
    )
    seamgrim_wasm_web_step_check_report_path = (
        str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_report_path", "")).strip() or "-"
    )
    seamgrim_wasm_web_step_check_report_exists = (
        str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_report_exists", "")).strip() or "0"
    )
    seamgrim_wasm_web_step_check_schema = (
        str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_schema", "")).strip() or "-"
    )
    seamgrim_wasm_web_step_check_checked_files = (
        str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_checked_files", "")).strip() or "-"
    )
    seamgrim_wasm_web_step_check_missing_count = (
        str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_missing_count", "")).strip() or "-"
    )
    fixed64_darwin_real_report_live_report_path = (
        str(doc.get("ci_sanity_fixed64_darwin_real_report_live_report_path", "")).strip() or "-"
    )
    fixed64_darwin_real_report_live_report_exists = (
        str(doc.get("ci_sanity_fixed64_darwin_real_report_live_report_exists", "")).strip() or "0"
    )
    fixed64_darwin_real_report_live_status = (
        str(doc.get("ci_sanity_fixed64_darwin_real_report_live_status", "")).strip() or "-"
    )
    fixed64_darwin_real_report_live_resolved_status = (
        str(doc.get("ci_sanity_fixed64_darwin_real_report_live_resolved_status", "")).strip() or "-"
    )
    fixed64_darwin_real_report_live_resolved_source = (
        str(doc.get("ci_sanity_fixed64_darwin_real_report_live_resolved_source", "")).strip() or "-"
    )
    fixed64_darwin_real_report_live_resolve_invalid_hit_count = (
        str(doc.get("ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count", "")).strip() or "-"
    )
    fixed64_darwin_real_report_live_resolved_source_zip = (
        str(doc.get("ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip", "")).strip() or "-"
    )
    fixed64_darwin_real_report_live_check_selftest_ok = (
        str(doc.get("ci_sanity_fixed64_darwin_real_report_live_check_selftest_ok", "")).strip() or "0"
    )
    fixed64_threeway_inputs_selftest_ok = (
        str(doc.get("ci_sanity_fixed64_threeway_inputs_selftest_ok", "")).strip() or "0"
    )
    age5_combined_heavy_policy_ok = (
        str(doc.get("ci_sanity_age5_combined_heavy_policy_selftest_ok", "")).strip() or "0"
    )
    age5_combined_heavy_report_schema = (
        str(doc.get("ci_sanity_age5_combined_heavy_report_schema", "")).strip() or "-"
    )
    age5_combined_heavy_required_reports = (
        str(doc.get("ci_sanity_age5_combined_heavy_required_reports", "")).strip() or "-"
    )
    age5_combined_heavy_required_criteria = (
        str(doc.get("ci_sanity_age5_combined_heavy_required_criteria", "")).strip() or "-"
    )
    age5_combined_heavy_child_summary_default_fields = (
        str(doc.get("ci_sanity_age5_combined_heavy_child_summary_default_fields", "")).strip() or "-"
    )
    age5_combined_heavy_combined_contract_summary_fields = (
        str(doc.get("ci_sanity_age5_combined_heavy_combined_contract_summary_fields", "")).strip() or "-"
    )
    age5_combined_heavy_full_summary_contract_fields = (
        str(doc.get("ci_sanity_age5_combined_heavy_full_summary_contract_fields", "")).strip() or "-"
    )
    profile_matrix_policy_ok = (
        str(doc.get("ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok", "")).strip() or "0"
    )
    dynamic_source_profile_split_selftest_ok = (
        str(doc.get("ci_sanity_dynamic_source_profile_split_selftest_ok", "")).strip() or "0"
    )
    steps = doc.get("steps")
    def read_step_ok(step_name: str) -> str:
        if not isinstance(steps, list):
            return "0"
        for row in steps:
            if not isinstance(row, dict):
                continue
            row_step = str(row.get("step", row.get("name", ""))).strip()
            if row_step != step_name:
                continue
            row_ok = bool(row.get("ok", False))
            rc_raw = row.get("returncode", 1)
            try:
                row_rc_ok = int(rc_raw) == 0
            except Exception:
                row_rc_ok = False
            return "1" if row_ok and row_rc_ok else "0"
        return "0"

    if isinstance(steps, list):
        step_count = len(steps)
        failed_steps = len(
            [
                row
                for row in steps
                if isinstance(row, dict) and not bool(row.get("ok", False))
            ]
        )
    else:
        step_count = 0
        failed_steps = 0
    snapshot = build_snapshot(
        CI_SANITY_DEFAULT_SNAPSHOT,
        CI_SANITY_SNAPSHOT_FIELD_SPECS,
        lambda _, source_key: {
            "status": status,
            "ok": "1" if status == "pass" else "0",
            "code": code,
            "step": step,
            "profile": profile,
            "msg": msg,
            "step_count": str(max(0, int(step_count))),
            "failed_steps": str(max(0, int(failed_steps))),
        }.get(
            source_key,
            str(doc.get(source_key, "")).strip()
            or str(CI_SANITY_DEFAULT_SNAPSHOT.get(source_key, "0")),
        ),
    )
    for output_key, step_name in CI_SANITY_STEP_OK_SPECS:
        snapshot[output_key] = read_step_ok(step_name)
    # Profile-specific `na` values (e.g. core_lang age3_close_ok) must preserve
    # the sanity report payload instead of step-derived fallback.
    snapshot["age2_completion_gate_ok"] = age2_completion_gate_ok
    snapshot["age2_completion_gate_selftest_ok"] = age2_completion_gate_selftest_ok
    snapshot["age2_close_ok"] = age2_close_ok
    snapshot["age2_close_selftest_ok"] = age2_close_selftest_ok
    snapshot["age3_completion_gate_ok"] = age3_completion_gate_ok
    snapshot["age3_completion_gate_selftest_ok"] = age3_completion_gate_selftest_ok
    snapshot["age3_close_ok"] = age3_close_ok
    snapshot["age3_close_selftest_ok"] = age3_close_selftest_ok
    snapshot["pipeline_emit_flags_ok"] = pipeline_emit_flags_ok
    snapshot["pipeline_emit_flags_selftest_ok"] = pipeline_emit_flags_selftest_ok
    snapshot["age2_completion_gate_failure_codes"] = age2_completion_gate_failure_codes
    snapshot["age2_completion_gate_failure_code_count"] = age2_completion_gate_failure_code_count
    snapshot["age3_completion_gate_failure_codes"] = age3_completion_gate_failure_codes
    snapshot["age3_completion_gate_failure_code_count"] = age3_completion_gate_failure_code_count
    for criteria_name in AGE3_COMPLETION_GATE_CRITERIA_NAMES:
        summary_key = age3_completion_gate_criteria_summary_key(criteria_name)
        snapshot[f"age3_completion_gate_criteria_{criteria_name}_ok"] = (
            str(doc.get(summary_key, "")).strip() or "0"
        )
    snapshot["age3_bogae_geoul_visibility_smoke_ok"] = age3_bogae_geoul_visibility_smoke_ok
    snapshot["age3_bogae_geoul_visibility_smoke_report_path"] = age3_bogae_geoul_visibility_smoke_report_path
    snapshot["age3_bogae_geoul_visibility_smoke_report_exists"] = age3_bogae_geoul_visibility_smoke_report_exists
    snapshot["age3_bogae_geoul_visibility_smoke_schema"] = age3_bogae_geoul_visibility_smoke_schema
    snapshot["age3_bogae_geoul_visibility_smoke_overall_ok"] = age3_bogae_geoul_visibility_smoke_overall_ok
    snapshot["age3_bogae_geoul_visibility_smoke_checks_ok"] = age3_bogae_geoul_visibility_smoke_checks_ok
    snapshot["age3_bogae_geoul_visibility_smoke_sim_state_hash_changes"] = (
        age3_bogae_geoul_visibility_smoke_sim_state_hash_changes
    )
    snapshot["age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes"] = (
        age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes
    )
    snapshot["seamgrim_wasm_web_step_check_ok"] = seamgrim_wasm_web_step_check_ok
    snapshot["seamgrim_wasm_web_step_check_report_path"] = seamgrim_wasm_web_step_check_report_path
    snapshot["seamgrim_wasm_web_step_check_report_exists"] = seamgrim_wasm_web_step_check_report_exists
    snapshot["seamgrim_wasm_web_step_check_schema"] = seamgrim_wasm_web_step_check_schema
    snapshot["seamgrim_wasm_web_step_check_checked_files"] = seamgrim_wasm_web_step_check_checked_files
    snapshot["seamgrim_wasm_web_step_check_missing_count"] = seamgrim_wasm_web_step_check_missing_count
    snapshot["fixed64_darwin_real_report_live_report_path"] = fixed64_darwin_real_report_live_report_path
    snapshot["fixed64_darwin_real_report_live_report_exists"] = fixed64_darwin_real_report_live_report_exists
    snapshot["fixed64_darwin_real_report_live_status"] = fixed64_darwin_real_report_live_status
    snapshot["fixed64_darwin_real_report_live_resolved_status"] = fixed64_darwin_real_report_live_resolved_status
    snapshot["fixed64_darwin_real_report_live_resolved_source"] = fixed64_darwin_real_report_live_resolved_source
    snapshot["fixed64_darwin_real_report_live_resolve_invalid_hit_count"] = (
        fixed64_darwin_real_report_live_resolve_invalid_hit_count
    )
    snapshot["fixed64_darwin_real_report_live_resolved_source_zip"] = (
        fixed64_darwin_real_report_live_resolved_source_zip
    )
    snapshot["fixed64_darwin_real_report_live_check_selftest_ok"] = (
        fixed64_darwin_real_report_live_check_selftest_ok
    )
    snapshot["fixed64_threeway_inputs_selftest_ok"] = fixed64_threeway_inputs_selftest_ok
    snapshot["age5_combined_heavy_policy_selftest_ok"] = age5_combined_heavy_policy_ok
    snapshot["age5_combined_heavy_report_schema"] = age5_combined_heavy_report_schema
    snapshot["age5_combined_heavy_required_reports"] = age5_combined_heavy_required_reports
    snapshot["age5_combined_heavy_required_criteria"] = age5_combined_heavy_required_criteria
    snapshot["age5_combined_heavy_child_summary_default_fields"] = (
        age5_combined_heavy_child_summary_default_fields
    )
    snapshot["age5_combined_heavy_combined_contract_summary_fields"] = (
        age5_combined_heavy_combined_contract_summary_fields
    )
    snapshot["age5_combined_heavy_full_summary_contract_fields"] = (
        age5_combined_heavy_full_summary_contract_fields
    )
    snapshot["profile_matrix_full_real_smoke_policy_selftest_ok"] = profile_matrix_policy_ok
    snapshot["dynamic_source_profile_split_selftest_ok"] = dynamic_source_profile_split_selftest_ok
    return snapshot


def append_ci_sanity_summary_lines(lines: list[str], report_path: Path) -> None:
    snap = load_ci_sanity_snapshot(report_path)
    append_summary_lines(
        lines,
        "ci_sanity_gate_report",
        report_path,
        snap,
        CI_SANITY_SUMMARY_LINE_SPECS,
    )


def load_ci_sync_readiness_snapshot(report_path: Path) -> dict[str, str]:
    doc = load_payload(report_path)
    if not isinstance(doc, dict):
        return dict(CI_SYNC_READINESS_DEFAULT_SNAPSHOT)
    status = str(doc.get("status", "")).strip() or "unknown"
    code = str(doc.get("code", "")).strip() or "-"
    step = str(doc.get("step", "")).strip() or "-"
    sanity_profile = str(doc.get("sanity_profile", "")).strip() or "full"
    msg = clip_line(str(doc.get("msg", "")).strip() or "-", 200)
    pipeline_emit_flags_ok = str(doc.get("ci_sanity_pipeline_emit_flags_ok", "")).strip() or "0"
    pipeline_emit_flags_selftest_ok = str(doc.get("ci_sanity_pipeline_emit_flags_selftest_ok", "")).strip() or "0"
    age2_completion_gate_ok = str(doc.get("ci_sanity_age2_completion_gate_ok", "")).strip() or "0"
    age2_completion_gate_selftest_ok = (
        str(doc.get("ci_sanity_age2_completion_gate_selftest_ok", "")).strip() or "0"
    )
    age2_close_ok = str(doc.get("ci_sanity_age2_close_ok", "")).strip() or "0"
    age2_close_selftest_ok = str(doc.get("ci_sanity_age2_close_selftest_ok", "")).strip() or "0"
    age3_completion_gate_ok = str(doc.get("ci_sanity_age3_completion_gate_ok", "")).strip() or "0"
    age3_completion_gate_selftest_ok = (
        str(doc.get("ci_sanity_age3_completion_gate_selftest_ok", "")).strip() or "0"
    )
    age3_close_ok = str(doc.get("ci_sanity_age3_close_ok", "")).strip() or "0"
    age3_close_selftest_ok = str(doc.get("ci_sanity_age3_close_selftest_ok", "")).strip() or "0"
    age2_completion_gate_failure_codes = (
        str(doc.get("ci_sanity_age2_completion_gate_failure_codes", "")).strip() or "-"
    )
    age2_completion_gate_failure_code_count = (
        str(doc.get("ci_sanity_age2_completion_gate_failure_code_count", "")).strip() or "0"
    )
    age3_completion_gate_failure_codes = (
        str(doc.get("ci_sanity_age3_completion_gate_failure_codes", "")).strip() or "-"
    )
    age3_completion_gate_failure_code_count = (
        str(doc.get("ci_sanity_age3_completion_gate_failure_code_count", "")).strip() or "0"
    )
    age3_bogae_geoul_visibility_smoke_ok = (
        str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_ok", "")).strip() or "0"
    )
    age3_bogae_geoul_visibility_smoke_report_path = (
        str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_report_path", "")).strip() or "-"
    )
    age3_bogae_geoul_visibility_smoke_report_exists = (
        str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists", "")).strip() or "0"
    )
    age3_bogae_geoul_visibility_smoke_schema = (
        str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_schema", "")).strip() or "-"
    )
    age3_bogae_geoul_visibility_smoke_overall_ok = (
        str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok", "")).strip() or "0"
    )
    age3_bogae_geoul_visibility_smoke_checks_ok = (
        str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok", "")).strip() or "0"
    )
    age3_bogae_geoul_visibility_smoke_sim_state_hash_changes = (
        str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes", "")).strip() or "0"
    )
    age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes = (
        str(doc.get("ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes", "")).strip() or "0"
    )
    seamgrim_wasm_web_step_check_ok = (
        str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_ok", "")).strip() or "0"
    )
    seamgrim_wasm_web_step_check_report_path = (
        str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_report_path", "")).strip() or "-"
    )
    seamgrim_wasm_web_step_check_report_exists = (
        str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_report_exists", "")).strip() or "0"
    )
    seamgrim_wasm_web_step_check_schema = (
        str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_schema", "")).strip() or "-"
    )
    seamgrim_wasm_web_step_check_checked_files = (
        str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_checked_files", "")).strip() or "-"
    )
    seamgrim_wasm_web_step_check_missing_count = (
        str(doc.get("ci_sanity_seamgrim_wasm_web_step_check_missing_count", "")).strip() or "-"
    )
    fixed64_darwin_real_report_live_report_path = (
        str(doc.get("ci_sanity_fixed64_darwin_real_report_live_report_path", "")).strip() or "-"
    )
    fixed64_darwin_real_report_live_report_exists = (
        str(doc.get("ci_sanity_fixed64_darwin_real_report_live_report_exists", "")).strip() or "0"
    )
    fixed64_darwin_real_report_live_status = (
        str(doc.get("ci_sanity_fixed64_darwin_real_report_live_status", "")).strip() or "-"
    )
    fixed64_darwin_real_report_live_resolved_status = (
        str(doc.get("ci_sanity_fixed64_darwin_real_report_live_resolved_status", "")).strip() or "-"
    )
    fixed64_darwin_real_report_live_resolved_source = (
        str(doc.get("ci_sanity_fixed64_darwin_real_report_live_resolved_source", "")).strip() or "-"
    )
    fixed64_darwin_real_report_live_resolve_invalid_hit_count = (
        str(doc.get("ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count", "")).strip() or "-"
    )
    fixed64_darwin_real_report_live_resolved_source_zip = (
        str(doc.get("ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip", "")).strip() or "-"
    )
    sync_fixed64_darwin_real_report_live_report_path = (
        str(
            doc.get(
                "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_report_path",
                "",
            )
        ).strip()
        or "-"
    )
    sync_fixed64_darwin_real_report_live_report_exists = (
        str(
            doc.get(
                "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_report_exists",
                "",
            )
        ).strip()
        or "0"
    )
    sync_fixed64_darwin_real_report_live_status = (
        str(
            doc.get(
                "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_status",
                "",
            )
        ).strip()
        or "-"
    )
    sync_fixed64_darwin_real_report_live_resolved_status = (
        str(
            doc.get(
                "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_status",
                "",
            )
        ).strip()
        or "-"
    )
    sync_fixed64_darwin_real_report_live_resolved_source = (
        str(
            doc.get(
                "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_source",
                "",
            )
        ).strip()
        or "-"
    )
    sync_fixed64_darwin_real_report_live_resolve_invalid_hit_count = (
        str(
            doc.get(
                "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count",
                "",
            )
        ).strip()
        or "-"
    )
    sync_fixed64_darwin_real_report_live_resolved_source_zip = (
        str(
            doc.get(
                "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip",
                "",
            )
        ).strip()
        or "-"
    )
    sync_age3_bogae_geoul_visibility_smoke_ok = (
        str(doc.get("ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_ok", "")).strip() or "0"
    )
    sync_age3_bogae_geoul_visibility_smoke_report_path = (
        str(doc.get("ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_path", "")).strip() or "-"
    )
    sync_age3_bogae_geoul_visibility_smoke_report_exists = (
        str(doc.get("ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists", "")).strip() or "0"
    )
    sync_age3_bogae_geoul_visibility_smoke_schema = (
        str(doc.get("ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_schema", "")).strip() or "-"
    )
    sync_age3_bogae_geoul_visibility_smoke_overall_ok = (
        str(doc.get("ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok", "")).strip() or "0"
    )
    sync_age3_bogae_geoul_visibility_smoke_checks_ok = (
        str(doc.get("ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok", "")).strip() or "0"
    )
    sync_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes = (
        str(doc.get("ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes", "")).strip()
        or "0"
    )
    sync_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes = (
        str(doc.get("ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes", "")).strip()
        or "0"
    )
    sync_seamgrim_wasm_web_step_check_ok = (
        str(doc.get("ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_ok", "")).strip() or "0"
    )
    sync_seamgrim_wasm_web_step_check_report_path = (
        str(doc.get("ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_path", "")).strip() or "-"
    )
    sync_seamgrim_wasm_web_step_check_report_exists = (
        str(doc.get("ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_exists", "")).strip() or "0"
    )
    sync_seamgrim_wasm_web_step_check_schema = (
        str(doc.get("ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_schema", "")).strip() or "-"
    )
    sync_seamgrim_wasm_web_step_check_checked_files = (
        str(doc.get("ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_checked_files", "")).strip() or "-"
    )
    sync_seamgrim_wasm_web_step_check_missing_count = (
        str(doc.get("ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_missing_count", "")).strip() or "-"
    )
    sync_age2_completion_gate_failure_codes = (
        str(doc.get("ci_sync_readiness_ci_sanity_age2_completion_gate_failure_codes", "")).strip() or "-"
    )
    sync_age2_completion_gate_failure_code_count = (
        str(doc.get("ci_sync_readiness_ci_sanity_age2_completion_gate_failure_code_count", "")).strip() or "0"
    )
    sync_age3_completion_gate_failure_codes = (
        str(doc.get("ci_sync_readiness_ci_sanity_age3_completion_gate_failure_codes", "")).strip() or "-"
    )
    sync_age3_completion_gate_failure_code_count = (
        str(doc.get("ci_sync_readiness_ci_sanity_age3_completion_gate_failure_code_count", "")).strip() or "0"
    )
    age5_combined_heavy_policy_ok = (
        str(doc.get("ci_sanity_age5_combined_heavy_policy_selftest_ok", "")).strip() or "0"
    )
    age5_combined_heavy_report_schema = (
        str(doc.get("ci_sanity_age5_combined_heavy_report_schema", "")).strip() or "-"
    )
    age5_combined_heavy_required_reports = (
        str(doc.get("ci_sanity_age5_combined_heavy_required_reports", "")).strip() or "-"
    )
    age5_combined_heavy_required_criteria = (
        str(doc.get("ci_sanity_age5_combined_heavy_required_criteria", "")).strip() or "-"
    )
    age5_combined_heavy_child_summary_default_fields = (
        str(doc.get("ci_sanity_age5_combined_heavy_child_summary_default_fields", "")).strip() or "-"
    )
    age5_combined_heavy_combined_contract_summary_fields = (
        str(doc.get("ci_sanity_age5_combined_heavy_combined_contract_summary_fields", "")).strip() or "-"
    )
    age5_combined_heavy_full_summary_contract_fields = (
        str(doc.get("ci_sanity_age5_combined_heavy_full_summary_contract_fields", "")).strip() or "-"
    )
    profile_matrix_policy_ok = (
        str(doc.get("ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok", "")).strip() or "0"
    )
    dynamic_source_profile_split_selftest_ok = (
        str(doc.get("ci_sanity_dynamic_source_profile_split_selftest_ok", "")).strip() or "0"
    )
    steps = doc.get("steps")
    if isinstance(steps, list):
        step_count = len(steps)
    else:
        step_count = 0
    return build_snapshot(
        CI_SYNC_READINESS_DEFAULT_SNAPSHOT,
        CI_SYNC_READINESS_SNAPSHOT_FIELD_SPECS,
        lambda _, source_key: {
            "status": status,
            "ok": "1" if status == "pass" else "0",
            "code": code,
            "step": step,
            "sanity_profile": sanity_profile,
            "msg": msg,
            "step_count": str(max(0, int(step_count))),
            "ci_sanity_pipeline_emit_flags_ok": pipeline_emit_flags_ok,
            "ci_sanity_pipeline_emit_flags_selftest_ok": pipeline_emit_flags_selftest_ok,
            "ci_sanity_age2_completion_gate_ok": age2_completion_gate_ok,
            "ci_sanity_age2_completion_gate_selftest_ok": age2_completion_gate_selftest_ok,
            "ci_sanity_age2_close_ok": age2_close_ok,
            "ci_sanity_age2_close_selftest_ok": age2_close_selftest_ok,
            "ci_sanity_age3_completion_gate_ok": age3_completion_gate_ok,
            "ci_sanity_age3_completion_gate_selftest_ok": age3_completion_gate_selftest_ok,
            "ci_sanity_age3_close_ok": age3_close_ok,
            "ci_sanity_age3_close_selftest_ok": age3_close_selftest_ok,
            "ci_sanity_age2_completion_gate_failure_codes": age2_completion_gate_failure_codes,
            "ci_sanity_age2_completion_gate_failure_code_count": age2_completion_gate_failure_code_count,
            "ci_sanity_age3_completion_gate_failure_codes": age3_completion_gate_failure_codes,
            "ci_sanity_age3_completion_gate_failure_code_count": age3_completion_gate_failure_code_count,
            "ci_sanity_age3_bogae_geoul_visibility_smoke_ok": age3_bogae_geoul_visibility_smoke_ok,
            "ci_sanity_age3_bogae_geoul_visibility_smoke_report_path": age3_bogae_geoul_visibility_smoke_report_path,
            "ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists": (
                age3_bogae_geoul_visibility_smoke_report_exists
            ),
            "ci_sanity_age3_bogae_geoul_visibility_smoke_schema": age3_bogae_geoul_visibility_smoke_schema,
            "ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok": (
                age3_bogae_geoul_visibility_smoke_overall_ok
            ),
            "ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok": age3_bogae_geoul_visibility_smoke_checks_ok,
            "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes": (
                age3_bogae_geoul_visibility_smoke_sim_state_hash_changes
            ),
            "ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes": (
                age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes
            ),
            "ci_sanity_seamgrim_wasm_web_step_check_ok": seamgrim_wasm_web_step_check_ok,
            "ci_sanity_seamgrim_wasm_web_step_check_report_path": seamgrim_wasm_web_step_check_report_path,
            "ci_sanity_seamgrim_wasm_web_step_check_report_exists": seamgrim_wasm_web_step_check_report_exists,
            "ci_sanity_seamgrim_wasm_web_step_check_schema": seamgrim_wasm_web_step_check_schema,
            "ci_sanity_seamgrim_wasm_web_step_check_checked_files": seamgrim_wasm_web_step_check_checked_files,
            "ci_sanity_seamgrim_wasm_web_step_check_missing_count": seamgrim_wasm_web_step_check_missing_count,
            "ci_sanity_fixed64_darwin_real_report_live_report_path": (
                fixed64_darwin_real_report_live_report_path
            ),
            "ci_sanity_fixed64_darwin_real_report_live_report_exists": (
                fixed64_darwin_real_report_live_report_exists
            ),
            "ci_sanity_fixed64_darwin_real_report_live_status": fixed64_darwin_real_report_live_status,
            "ci_sanity_fixed64_darwin_real_report_live_resolved_status": (
                fixed64_darwin_real_report_live_resolved_status
            ),
            "ci_sanity_fixed64_darwin_real_report_live_resolved_source": (
                fixed64_darwin_real_report_live_resolved_source
            ),
            "ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count": (
                fixed64_darwin_real_report_live_resolve_invalid_hit_count
            ),
            "ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip": (
                fixed64_darwin_real_report_live_resolved_source_zip
            ),
            "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_report_path": (
                sync_fixed64_darwin_real_report_live_report_path
            ),
            "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_report_exists": (
                sync_fixed64_darwin_real_report_live_report_exists
            ),
            "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_status": (
                sync_fixed64_darwin_real_report_live_status
            ),
            "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_status": (
                sync_fixed64_darwin_real_report_live_resolved_status
            ),
            "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_source": (
                sync_fixed64_darwin_real_report_live_resolved_source
            ),
            "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolve_invalid_hit_count": (
                sync_fixed64_darwin_real_report_live_resolve_invalid_hit_count
            ),
            "ci_sync_readiness_ci_sanity_fixed64_darwin_real_report_live_resolved_source_zip": (
                sync_fixed64_darwin_real_report_live_resolved_source_zip
            ),
            "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_ok": (
                sync_age3_bogae_geoul_visibility_smoke_ok
            ),
            "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_path": (
                sync_age3_bogae_geoul_visibility_smoke_report_path
            ),
            "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_report_exists": (
                sync_age3_bogae_geoul_visibility_smoke_report_exists
            ),
            "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_schema": (
                sync_age3_bogae_geoul_visibility_smoke_schema
            ),
            "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_overall_ok": (
                sync_age3_bogae_geoul_visibility_smoke_overall_ok
            ),
            "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_checks_ok": (
                sync_age3_bogae_geoul_visibility_smoke_checks_ok
            ),
            "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes": (
                sync_age3_bogae_geoul_visibility_smoke_sim_state_hash_changes
            ),
            "ci_sync_readiness_ci_sanity_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes": (
                sync_age3_bogae_geoul_visibility_smoke_sim_bogae_hash_changes
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_ok": (
                sync_seamgrim_wasm_web_step_check_ok
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_path": (
                sync_seamgrim_wasm_web_step_check_report_path
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_report_exists": (
                sync_seamgrim_wasm_web_step_check_report_exists
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_schema": (
                sync_seamgrim_wasm_web_step_check_schema
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_checked_files": (
                sync_seamgrim_wasm_web_step_check_checked_files
            ),
            "ci_sync_readiness_ci_sanity_seamgrim_wasm_web_step_check_missing_count": (
                sync_seamgrim_wasm_web_step_check_missing_count
            ),
            "ci_sync_readiness_ci_sanity_age2_completion_gate_failure_codes": (
                sync_age2_completion_gate_failure_codes
            ),
            "ci_sync_readiness_ci_sanity_age2_completion_gate_failure_code_count": (
                sync_age2_completion_gate_failure_code_count
            ),
            "ci_sync_readiness_ci_sanity_age3_completion_gate_failure_codes": (
                sync_age3_completion_gate_failure_codes
            ),
            "ci_sync_readiness_ci_sanity_age3_completion_gate_failure_code_count": (
                sync_age3_completion_gate_failure_code_count
            ),
            "ci_sanity_age5_combined_heavy_policy_selftest_ok": age5_combined_heavy_policy_ok,
            "ci_sanity_age5_combined_heavy_report_schema": age5_combined_heavy_report_schema,
            "ci_sanity_age5_combined_heavy_required_reports": age5_combined_heavy_required_reports,
            "ci_sanity_age5_combined_heavy_required_criteria": age5_combined_heavy_required_criteria,
            "ci_sanity_age5_combined_heavy_child_summary_default_fields": (
                age5_combined_heavy_child_summary_default_fields
            ),
            "ci_sanity_age5_combined_heavy_combined_contract_summary_fields": (
                age5_combined_heavy_combined_contract_summary_fields
            ),
            "ci_sanity_age5_combined_heavy_full_summary_contract_fields": (
                age5_combined_heavy_full_summary_contract_fields
            ),
            "ci_sanity_profile_matrix_full_real_smoke_policy_selftest_ok": profile_matrix_policy_ok,
            "ci_sanity_dynamic_source_profile_split_selftest_ok": (
                dynamic_source_profile_split_selftest_ok
            ),
        }.get(source_key, str(doc.get(source_key, "")).strip() or "0"),
    )


def append_ci_sync_readiness_summary_lines(lines: list[str], report_path: Path) -> None:
    snap = load_ci_sync_readiness_snapshot(report_path)
    append_summary_lines(
        lines,
        "ci_sync_readiness_report",
        report_path,
        snap,
        CI_SYNC_READINESS_SUMMARY_LINE_SPECS,
    )


def resolve_summary_compact(
    ci_gate_result_line_path: Path,
    final_status_parse_path: Path,
    final_status_line_path: Path,
) -> str:
    try:
        line = ci_gate_result_line_path.read_text(encoding="utf-8").strip()
    except Exception:
        line = "-"
    if line != "-":
        return line
    payload = load_payload(final_status_parse_path)
    if isinstance(payload, dict):
        compact = str(payload.get("compact_line", "")).strip()
        if compact:
            return compact
    try:
        fallback = final_status_line_path.read_text(encoding="utf-8").strip()
    except Exception:
        fallback = "-"
    return fallback if fallback else "-"


def append_runtime_5min_summary_lines(
    lines: list[str],
    with_runtime_5min: bool,
    runtime_5min_report: Path,
    runtime_5min_browse_selection_report: Path,
) -> None:
    if not with_runtime_5min:
        return
    lines.append(f"[ci-gate-summary] seamgrim_runtime_5min={runtime_5min_report}")
    lines.append(
        "[ci-gate-summary] seamgrim_runtime_5min_browse_selection=" f"{runtime_5min_browse_selection_report}"
    )


def load_runtime_5min_checklist_snapshot(report_path: Path) -> dict[str, str]:
    doc = load_payload(report_path)
    if not isinstance(doc, dict):
        return dict(RUNTIME5_CHECKLIST_DEFAULT_SNAPSHOT)
    ok_text = "1" if bool(doc.get("ok", False)) else "0"
    items = doc.get("items")
    if not isinstance(items, list):
        return build_snapshot(
            {
                **RUNTIME5_CHECKLIST_DEFAULT_SNAPSHOT,
                "ok": ok_text,
                **RUNTIME5_CHECKLIST_ITEMS_MISSING_SNAPSHOT,
            },
            RUNTIME5_CHECKLIST_SNAPSHOT_FIELD_SPECS,
            lambda _, source_key: {
                "ok": ok_text,
                **RUNTIME5_CHECKLIST_ITEMS_MISSING_SNAPSHOT,
            }[source_key],
        )
    rows_by_name: dict[str, dict] = {}
    for row in items:
        if not isinstance(row, dict):
            continue
        row_name = str(row.get("name", "")).strip()
        rows_by_name[row_name] = row

    def summarize_row(row: object) -> tuple[str, str, str]:
        if not isinstance(row, dict):
            return ("na", "-", "not_executed")
        elapsed_raw = row.get("elapsed_ms")
        try:
            elapsed_text = str(max(0, int(elapsed_raw)))
        except Exception:
            elapsed_text = "-"
        ok = bool(row.get("ok", False))
        return ("1" if ok else "0", elapsed_text, "ok" if ok else "failed")

    resolved_snapshot = {
        **RUNTIME5_CHECKLIST_DEFAULT_SNAPSHOT,
        "ok": ok_text,
    }
    for item_prefix, row_name in RUNTIME5_CHECKLIST_ROW_SPECS:
        item_ok, item_elapsed_text, item_status = summarize_row(rows_by_name.get(row_name))
        resolved_snapshot[f"{item_prefix}_ok"] = item_ok
        resolved_snapshot[f"{item_prefix}_elapsed_ms"] = item_elapsed_text
        resolved_snapshot[f"{item_prefix}_status"] = item_status
    return build_snapshot(
        RUNTIME5_CHECKLIST_DEFAULT_SNAPSHOT,
        RUNTIME5_CHECKLIST_SNAPSHOT_FIELD_SPECS,
        lambda _, source_key: resolved_snapshot[source_key],
    )


def append_runtime_5min_checklist_summary_lines(
    lines: list[str],
    with_5min_checklist: bool,
    checklist_report: Path,
) -> None:
    if not with_5min_checklist:
        return
    snap = load_runtime_5min_checklist_snapshot(checklist_report)
    append_summary_lines(
        lines,
        "seamgrim_5min_checklist",
        checklist_report,
        snap,
        RUNTIME5_CHECKLIST_SUMMARY_LINE_SPECS,
    )


def load_ci_profile_matrix_selftest_snapshot(report_path: Path) -> dict[str, str]:
    doc = load_payload(report_path)
    if not isinstance(doc, dict):
        snap = dict(PROFILE_MATRIX_DEFAULT_SNAPSHOT)
        snap["aggregate_summary_sanity_ok"] = "0"
        snap["aggregate_summary_sanity_checked_profiles"] = "-"
        snap["aggregate_summary_sanity_failed_profiles"] = "-"
        snap["aggregate_summary_sanity_skipped_profiles"] = "-"
        for profile_name in ("core_lang", "full", "seamgrim"):
            snap[f"{profile_name}_aggregate_summary_status"] = "-"
            snap[f"{profile_name}_aggregate_summary_ok"] = "0"
            snap[f"{profile_name}_aggregate_summary_values"] = "-"
        return snap
    if str(doc.get("schema", "")).strip() != "ddn.ci.profile_matrix_gate_selftest.v1":
        snap = dict(PROFILE_MATRIX_DEFAULT_SNAPSHOT)
        snap["status"] = "invalid_schema"
        snap["aggregate_summary_sanity_ok"] = "0"
        snap["aggregate_summary_sanity_checked_profiles"] = "-"
        snap["aggregate_summary_sanity_failed_profiles"] = "-"
        snap["aggregate_summary_sanity_skipped_profiles"] = "-"
        for profile_name in ("core_lang", "full", "seamgrim"):
            snap[f"{profile_name}_aggregate_summary_status"] = "-"
            snap[f"{profile_name}_aggregate_summary_ok"] = "0"
            snap[f"{profile_name}_aggregate_summary_values"] = "-"
        return snap

    def normalize_profiles(raw: object) -> str:
        if not isinstance(raw, list):
            return "-"
        names = [str(item).strip() for item in raw if str(item).strip()]
        return ",".join(names) if names else "-"

    real_profiles = doc.get("real_profiles")

    def normalize_elapsed(profile_name: str) -> str:
        if not isinstance(real_profiles, dict):
            return "-"
        row = real_profiles.get(profile_name)
        if not isinstance(row, dict):
            return "-"
        elapsed_raw = row.get("total_elapsed_ms")
        if elapsed_raw is None:
            return "-"
        try:
            return str(max(0, int(elapsed_raw)))
        except Exception:
            return "-"

    def normalize_int_value(raw: object, default_value: str) -> str:
        try:
            return str(max(0, int(raw)))
        except Exception:
            return default_value
    def aggregate_summary_row(profile_name: str) -> dict | None:
        block = doc.get("aggregate_summary_sanity_by_profile")
        if not isinstance(block, dict):
            return None
        row = block.get(profile_name)
        return row if isinstance(row, dict) else None

    def join_summary_values(row: dict | None) -> str:
        if not isinstance(row, dict):
            return "-"
        values = row.get("values")
        if not isinstance(values, dict):
            return "-"
        parts = [str(values.get(key, "")).strip() or "-" for key in PROFILE_MATRIX_AGGREGATE_SUMMARY_VALUE_KEYS]
        return "/".join(parts)

    snap = build_typed_snapshot(
        PROFILE_MATRIX_DEFAULT_SNAPSHOT,
        PROFILE_MATRIX_SNAPSHOT_FIELD_SPECS,
        lambda _, field_kind, source_key, default_value: (
            str(doc.get(source_key, "")).strip() or default_value
            if field_kind == "text"
            else ("1" if bool(doc.get(source_key, False)) else "0")
            if field_kind == "bool_text"
            else normalize_profiles(doc.get(source_key))
            if field_kind == "names_text"
            else normalize_elapsed(source_key)
            if field_kind == "elapsed_text"
            else normalize_int_value(doc.get(source_key), default_value)
            if field_kind == "int_text"
            else default_value
        ),
    )
    snap["aggregate_summary_sanity_ok"] = "1" if bool(doc.get("aggregate_summary_sanity_ok", False)) else "0"
    snap["aggregate_summary_sanity_checked_profiles"] = normalize_profiles(doc.get("aggregate_summary_sanity_checked_profiles"))
    snap["aggregate_summary_sanity_failed_profiles"] = normalize_profiles(doc.get("aggregate_summary_sanity_failed_profiles"))
    snap["aggregate_summary_sanity_skipped_profiles"] = normalize_profiles(doc.get("aggregate_summary_sanity_skipped_profiles"))
    for profile_name in ("core_lang", "full", "seamgrim"):
        row = aggregate_summary_row(profile_name)
        snap[f"{profile_name}_aggregate_summary_status"] = (
            str(row.get("status", "")).strip() if isinstance(row, dict) else "-"
        ) or "-"
        snap[f"{profile_name}_aggregate_summary_ok"] = (
            "1" if isinstance(row, dict) and bool(row.get("ok", False)) else "0"
        )
        snap[f"{profile_name}_aggregate_summary_values"] = join_summary_values(row)
    return snap
def append_ci_profile_matrix_selftest_summary_lines(lines: list[str], report_path: Path) -> None:
    snap = load_ci_profile_matrix_selftest_snapshot(report_path)
    append_summary_lines(
        lines,
        "ci_profile_matrix_gate_selftest_report",
        report_path,
        snap,
        PROFILE_MATRIX_SUMMARY_LINE_SPECS,
    )


def print_failure_block(
    steps_log: list[dict[str, object]],
    seamgrim_report: Path,
    age3_close_report: Path,
    age4_close_report: Path,
    age5_close_report: Path,
    oi_report: Path,
    aggregate_report: Path,
    max_digest: int = 3,
    max_step_details: int = 3,
) -> list[str]:
    failed_steps = [str(row.get("name", "-")) for row in steps_log if not bool(row.get("ok", False))]
    if not failed_steps:
        return []
    lines = [
        "[ci-gate-summary] FAIL",
        f"[ci-gate-summary] failed_steps={','.join(failed_steps)}",
    ]
    detailed_rows = [row for row in steps_log if not bool(row.get("ok", False))]
    for row in detailed_rows[:max_step_details]:
        name = str(row.get("name", "-"))
        rc = int(row.get("returncode", -1))
        cmd = row.get("cmd")
        cmd_text = " ".join(str(token) for token in cmd) if isinstance(cmd, list) else "-"
        stdout_log = str(row.get("stdout_log_path", "")).strip()
        stderr_log = str(row.get("stderr_log_path", "")).strip()
        lines.append(f"[ci-gate-summary] failed_step_detail={name} rc={rc} cmd={clip_line(cmd_text, 160)}")
        if stdout_log or stderr_log:
            lines.append(
                f"[ci-gate-summary] failed_step_logs={name} " f"stdout={stdout_log or '-'} stderr={stderr_log or '-'}"
            )

    seamgrim_doc = load_payload(seamgrim_report)
    if isinstance(seamgrim_doc, dict):
        failed = seamgrim_doc.get("failure_digest")
        if isinstance(failed, list) and failed:
            top = " | ".join(clip_line(str(line)) for line in failed[:max_digest])
            lines.append(f"[ci-gate-summary] seamgrim_digest={top}")

    age3_doc = load_payload(age3_close_report)
    if isinstance(age3_doc, dict):
        failed = age3_doc.get("failure_digest")
        if isinstance(failed, list) and failed:
            top = " | ".join(clip_line(str(line)) for line in failed[:max_digest])
            lines.append(f"[ci-gate-summary] age3_digest={top}")

    age4_doc = load_payload(age4_close_report)
    if isinstance(age4_doc, dict):
        failed = age4_doc.get("failure_digest")
        if isinstance(failed, list) and failed:
            top = " | ".join(clip_line(str(line)) for line in failed[:max_digest])
            lines.append(f"[ci-gate-summary] age4_digest={top}")

    age5_doc = load_payload(age5_close_report)
    if isinstance(age5_doc, dict):
        failed = age5_doc.get("failure_digest")
        if isinstance(failed, list) and failed:
            top = " | ".join(clip_line(str(line)) for line in failed[:max_digest])
            lines.append(f"[ci-gate-summary] age5_digest={top}")

    oi_doc = load_payload(oi_report)
    if isinstance(oi_doc, dict):
        failed = oi_doc.get("failure_digest")
        if isinstance(failed, list) and failed:
            top = " | ".join(clip_line(str(line)) for line in failed[:max_digest])
            lines.append(f"[ci-gate-summary] oi_digest={top}")

    aggregate_doc = load_payload(aggregate_report)
    if isinstance(aggregate_doc, dict):
        failed = aggregate_doc.get("failure_digest")
        if isinstance(failed, list) and failed:
            top = " | ".join(clip_line(str(line)) for line in failed[:max_digest])
            lines.append(f"[ci-gate-summary] aggregate_digest={top}")
    return lines


def print_report_paths(
    seamgrim_report: Path,
    seamgrim_ui_age3_report: Path,
    seamgrim_phase3_cleanup_report: Path,
    seamgrim_browse_selection_report: Path,
    seamgrim_runtime_5min_report: Path,
    seamgrim_runtime_5min_browse_selection_report: Path,
    seamgrim_5min_checklist_report: Path,
    seamgrim_lesson_warning_tokens_report: Path,
    seamgrim_control_exposure_failures_report: Path,
    seamgrim_rewrite_overlay_quality_report: Path,
    seamgrim_wasm_cli_diag_parity_report: Path,
    age3_close_report: Path,
    age4_close_report: Path,
    age5_close_report: Path,
    age4_pack_report: Path,
    age3_close_summary_md: Path,
    age3_close_status_json: Path,
    age3_close_status_line: Path,
    age3_close_badge_json: Path,
    aggregate_status_line: Path,
    aggregate_status_parse_json: Path,
    final_status_line: Path,
    final_status_parse_json: Path,
    summary_line_path: Path,
    ci_gate_result_json: Path,
    ci_gate_result_parse_json: Path,
    ci_gate_result_line_path: Path,
    ci_gate_badge_json: Path,
    ci_fail_brief_txt: Path,
    ci_fail_triage_json: Path,
    oi_report: Path,
    oi_pack_report: Path,
    aggregate_report: Path,
    ci_profile_matrix_gate_selftest_report: Path,
    ci_sanity_gate_report: Path,
    ci_sync_readiness_report: Path,
    summary_path: Path,
) -> None:
    print("[ci-gate] reports")
    print(f" - seamgrim={seamgrim_report}")
    print(f" - seamgrim_ui_age3={seamgrim_ui_age3_report}")
    print(f" - seamgrim_phase3_cleanup={seamgrim_phase3_cleanup_report}")
    print(f" - seamgrim_browse_selection={seamgrim_browse_selection_report}")
    print(f" - seamgrim_runtime_5min={seamgrim_runtime_5min_report}")
    print(f" - seamgrim_runtime_5min_browse_selection={seamgrim_runtime_5min_browse_selection_report}")
    print(f" - seamgrim_5min_checklist={seamgrim_5min_checklist_report}")
    print(f" - seamgrim_lesson_warning_tokens={seamgrim_lesson_warning_tokens_report}")
    print(f" - seamgrim_control_exposure_failures={seamgrim_control_exposure_failures_report}")
    print(f" - seamgrim_rewrite_overlay_quality={seamgrim_rewrite_overlay_quality_report}")
    print(f" - seamgrim_wasm_cli_diag_parity={seamgrim_wasm_cli_diag_parity_report}")
    print(f" - age3_close={age3_close_report}")
    print(f" - age4_close={age4_close_report}")
    print(f" - age5_close={age5_close_report}")
    print(f" - age4_pack={age4_pack_report}")
    print(f" - age3_close_summary_md={age3_close_summary_md}")
    print(f" - age3_close_status_json={age3_close_status_json}")
    print(f" - age3_close_status_line={age3_close_status_line}")
    print(f" - age3_close_badge_json={age3_close_badge_json}")
    print(f" - aggregate_status_line={aggregate_status_line}")
    print(f" - aggregate_status_parse_json={aggregate_status_parse_json}")
    print(f" - final_status_line={final_status_line}")
    print(f" - final_status_parse_json={final_status_parse_json}")
    print(f" - summary_line={summary_line_path}")
    print(f" - ci_gate_result_json={ci_gate_result_json}")
    print(f" - ci_gate_result_parse_json={ci_gate_result_parse_json}")
    print(f" - ci_gate_result_line={ci_gate_result_line_path}")
    print(f" - ci_gate_badge_json={ci_gate_badge_json}")
    print(f" - ci_fail_brief_txt={ci_fail_brief_txt}")
    print(f" - ci_fail_triage_json={ci_fail_triage_json}")
    print(f" - oi_close={oi_report}")
    print(f" - oi_pack={oi_pack_report}")
    print(f" - aggregate={aggregate_report}")
    print(f" - ci_profile_matrix_gate_selftest={ci_profile_matrix_gate_selftest_report}")
    print(f" - ci_sanity_gate={ci_sanity_gate_report}")
    print(f" - ci_sync_readiness={ci_sync_readiness_report}")
    print(f" - summary={summary_path}")


def write_summary(path: Path, lines: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    body = "\n".join(lines).rstrip() + "\n"
    path.write_text(body, encoding="utf-8")
    print(f"[ci-gate] summary_report={path}")


def write_summary_line(path: Path, line: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(line.rstrip() + "\n", encoding="utf-8")
    print(f"[ci-gate] summary_line_report={path}")
