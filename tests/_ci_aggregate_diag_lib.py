from __future__ import annotations

import json
import re
from datetime import datetime, timezone
from pathlib import Path


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


CONTROL_EXPOSURE_VIOLATION_RE = re.compile(
    r"(?P<kind>[a-z_]+):(?P<file>[^:\s,]+\.ddn):(?P<name>[A-Za-z0-9_가-힣]+)",
    re.UNICODE,
)
CONTROL_EXPOSURE_MORE_RE = re.compile(r"\.\.\.\s*\((?P<count>\d+)\s+more\)", re.UNICODE)


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
        detail = {
            "missing_report": "seamgrim report missing or invalid",
            "steps_missing": "seamgrim report has no steps",
            "step_missing": f"{step_name} step missing",
        }.get(status, "step unavailable")
        return {
            "status": status,
            "ok": "0",
            "diag_count": "0",
            "detail": detail,
        }

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
    return {
        "status": "ok" if ok_text == "1" else "failed",
        "ok": ok_text,
        "diag_count": str(len(diag_rows)),
        "detail": clip_line(detail, 200),
    }


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
    return {
        "status": str(payload.get("status", status)).strip() or status,
        "step_ok": "1" if bool(payload.get("step_ok", False)) else "0",
        "violation_count": str(max(0, int(violation_count))) if str(violation_count).strip() else "0",
        "top": top,
    }


def load_rewrite_overlay_report_snapshot(report_path: Path) -> dict[str, str]:
    doc = load_payload(report_path)
    if not isinstance(doc, dict):
        return {
            "status": "missing_report",
            "ok": "0",
            "violation_count": "0",
            "top": "-",
        }
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
    return {
        "status": "ok" if ok_text == "1" else "failed",
        "ok": ok_text,
        "violation_count": str(issue_count),
        "top": top,
    }


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
    seed_meta = load_seamgrim_step_snapshot(seamgrim_report, "seed_meta_files")
    seed_overlay_quality = load_seamgrim_step_snapshot(seamgrim_report, "seed_overlay_quality")
    rewrite_overlay_quality = load_seamgrim_step_snapshot(seamgrim_report, "rewrite_overlay_quality")
    pendulum_surface_contract = load_seamgrim_step_snapshot(seamgrim_report, "pendulum_surface_contract")
    seed_export = load_seamgrim_step_snapshot(seamgrim_report, "seed_pendulum_export")
    pendulum_runtime_visual = load_seamgrim_step_snapshot(seamgrim_report, "pendulum_runtime_visual")
    seed_runtime_visual_pack = load_seamgrim_step_snapshot(seamgrim_report, "seed_runtime_visual_pack")
    runtime_fallback_metrics = load_seamgrim_step_snapshot(seamgrim_report, "runtime_fallback_metrics")
    runtime_fallback_policy = load_seamgrim_step_snapshot(seamgrim_report, "runtime_fallback_policy")
    pendulum = load_seamgrim_step_snapshot(seamgrim_report, "pendulum_bogae_shape")
    lines.append(f"[ci-gate-summary] seamgrim_control_exposure_policy_report={control_exposure_report}")
    lines.append(f"[ci-gate-summary] seamgrim_control_exposure_policy_status={snapshot['status']}")
    lines.append(f"[ci-gate-summary] seamgrim_control_exposure_policy_ok={snapshot['step_ok']}")
    lines.append(f"[ci-gate-summary] seamgrim_control_exposure_policy_violations={snapshot['violation_count']}")
    lines.append(f"[ci-gate-summary] seamgrim_control_exposure_policy_top={snapshot['top']}")
    lines.append(f"[ci-gate-summary] seamgrim_rewrite_overlay_quality_report={rewrite_overlay_quality_report}")
    lines.append(f"[ci-gate-summary] seamgrim_rewrite_overlay_quality_violations={rewrite_snapshot['violation_count']}")
    lines.append(f"[ci-gate-summary] seamgrim_rewrite_overlay_quality_top={rewrite_snapshot['top']}")
    lines.append(f"[ci-gate-summary] seamgrim_seed_meta_files_status={seed_meta['status']}")
    lines.append(f"[ci-gate-summary] seamgrim_seed_meta_files_ok={seed_meta['ok']}")
    lines.append(f"[ci-gate-summary] seamgrim_seed_meta_files_diag_count={seed_meta['diag_count']}")
    lines.append(f"[ci-gate-summary] seamgrim_seed_meta_files_detail={seed_meta['detail']}")
    lines.append(f"[ci-gate-summary] seamgrim_seed_overlay_quality_status={seed_overlay_quality['status']}")
    lines.append(f"[ci-gate-summary] seamgrim_seed_overlay_quality_ok={seed_overlay_quality['ok']}")
    lines.append(f"[ci-gate-summary] seamgrim_seed_overlay_quality_diag_count={seed_overlay_quality['diag_count']}")
    lines.append(f"[ci-gate-summary] seamgrim_seed_overlay_quality_detail={seed_overlay_quality['detail']}")
    lines.append(f"[ci-gate-summary] seamgrim_rewrite_overlay_quality_status={rewrite_overlay_quality['status']}")
    lines.append(f"[ci-gate-summary] seamgrim_rewrite_overlay_quality_ok={rewrite_overlay_quality['ok']}")
    lines.append(
        f"[ci-gate-summary] seamgrim_rewrite_overlay_quality_diag_count={rewrite_overlay_quality['diag_count']}"
    )
    lines.append(f"[ci-gate-summary] seamgrim_rewrite_overlay_quality_detail={rewrite_overlay_quality['detail']}")
    lines.append(f"[ci-gate-summary] seamgrim_pendulum_surface_contract_status={pendulum_surface_contract['status']}")
    lines.append(f"[ci-gate-summary] seamgrim_pendulum_surface_contract_ok={pendulum_surface_contract['ok']}")
    lines.append(
        f"[ci-gate-summary] seamgrim_pendulum_surface_contract_diag_count={pendulum_surface_contract['diag_count']}"
    )
    lines.append(f"[ci-gate-summary] seamgrim_pendulum_surface_contract_detail={pendulum_surface_contract['detail']}")
    lines.append(f"[ci-gate-summary] seamgrim_seed_pendulum_export_status={seed_export['status']}")
    lines.append(f"[ci-gate-summary] seamgrim_seed_pendulum_export_ok={seed_export['ok']}")
    lines.append(f"[ci-gate-summary] seamgrim_seed_pendulum_export_diag_count={seed_export['diag_count']}")
    lines.append(f"[ci-gate-summary] seamgrim_seed_pendulum_export_detail={seed_export['detail']}")
    lines.append(f"[ci-gate-summary] seamgrim_pendulum_runtime_visual_status={pendulum_runtime_visual['status']}")
    lines.append(f"[ci-gate-summary] seamgrim_pendulum_runtime_visual_ok={pendulum_runtime_visual['ok']}")
    lines.append(
        f"[ci-gate-summary] seamgrim_pendulum_runtime_visual_diag_count={pendulum_runtime_visual['diag_count']}"
    )
    lines.append(f"[ci-gate-summary] seamgrim_pendulum_runtime_visual_detail={pendulum_runtime_visual['detail']}")
    lines.append(f"[ci-gate-summary] seamgrim_seed_runtime_visual_pack_status={seed_runtime_visual_pack['status']}")
    lines.append(f"[ci-gate-summary] seamgrim_seed_runtime_visual_pack_ok={seed_runtime_visual_pack['ok']}")
    lines.append(
        f"[ci-gate-summary] seamgrim_seed_runtime_visual_pack_diag_count={seed_runtime_visual_pack['diag_count']}"
    )
    lines.append(f"[ci-gate-summary] seamgrim_seed_runtime_visual_pack_detail={seed_runtime_visual_pack['detail']}")
    lines.append(f"[ci-gate-summary] seamgrim_runtime_fallback_metrics_status={runtime_fallback_metrics['status']}")
    lines.append(f"[ci-gate-summary] seamgrim_runtime_fallback_metrics_ok={runtime_fallback_metrics['ok']}")
    lines.append(
        f"[ci-gate-summary] seamgrim_runtime_fallback_metrics_diag_count={runtime_fallback_metrics['diag_count']}"
    )
    lines.append(f"[ci-gate-summary] seamgrim_runtime_fallback_metrics_detail={runtime_fallback_metrics['detail']}")
    lines.append(f"[ci-gate-summary] seamgrim_runtime_fallback_policy_status={runtime_fallback_policy['status']}")
    lines.append(f"[ci-gate-summary] seamgrim_runtime_fallback_policy_ok={runtime_fallback_policy['ok']}")
    lines.append(
        f"[ci-gate-summary] seamgrim_runtime_fallback_policy_diag_count={runtime_fallback_policy['diag_count']}"
    )
    lines.append(f"[ci-gate-summary] seamgrim_runtime_fallback_policy_detail={runtime_fallback_policy['detail']}")
    lines.append(f"[ci-gate-summary] seamgrim_pendulum_bogae_shape_status={pendulum['status']}")
    lines.append(f"[ci-gate-summary] seamgrim_pendulum_bogae_shape_ok={pendulum['ok']}")
    lines.append(f"[ci-gate-summary] seamgrim_pendulum_bogae_shape_diag_count={pendulum['diag_count']}")
    lines.append(f"[ci-gate-summary] seamgrim_pendulum_bogae_shape_detail={pendulum['detail']}")
    return snapshot


def load_fixed64_threeway_snapshot(report_path: Path) -> dict[str, str]:
    doc = load_payload(report_path)
    if not isinstance(doc, dict):
        return {
            "status": "missing_report",
            "ok": "0",
            "reason": "report missing or invalid",
        }
    status = str(doc.get("status", "")).strip() or "unknown"
    ok_text = "1" if bool(doc.get("ok", False)) else "0"
    reason = clip_line(str(doc.get("reason", "-")).strip() or "-", 200)
    return {
        "status": status,
        "ok": ok_text,
        "reason": reason,
    }


def append_fixed64_threeway_summary_lines(lines: list[str], report_path: Path) -> None:
    snap = load_fixed64_threeway_snapshot(report_path)
    lines.append(f"[ci-gate-summary] fixed64_threeway_report={report_path}")
    lines.append(f"[ci-gate-summary] fixed64_threeway_status={snap['status']}")
    lines.append(f"[ci-gate-summary] fixed64_threeway_ok={snap['ok']}")
    lines.append(f"[ci-gate-summary] fixed64_threeway_reason={snap['reason']}")


def resolve_summary_compact(
    ci_gate_result_line_path: Path,
    final_status_parse_path: Path,
    final_status_line_path: Path,
) -> str:
    line = read_compact_line(ci_gate_result_line_path)
    if line != "-":
        return line
    return read_compact_from_parse(
        final_status_parse_path,
        fallback=read_compact_line(final_status_line_path),
    )


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
        return {
            "ok": "0",
            "rewrite_ok": "na",
            "rewrite_elapsed_ms": "-",
            "rewrite_status": "missing_report",
        }
    ok_text = "1" if bool(doc.get("ok", False)) else "0"
    items = doc.get("items")
    if not isinstance(items, list):
        return {
            "ok": ok_text,
            "rewrite_ok": "na",
            "rewrite_elapsed_ms": "-",
            "rewrite_status": "items_missing",
        }
    rewrite_row = None
    for row in items:
        if not isinstance(row, dict):
            continue
        if str(row.get("name", "")).strip() == "rewrite_motion_projectile_fallback":
            rewrite_row = row
            break
    if not isinstance(rewrite_row, dict):
        return {
            "ok": ok_text,
            "rewrite_ok": "na",
            "rewrite_elapsed_ms": "-",
            "rewrite_status": "not_executed",
        }
    elapsed_raw = rewrite_row.get("elapsed_ms")
    try:
        elapsed_text = str(max(0, int(elapsed_raw)))
    except Exception:
        elapsed_text = "-"
    return {
        "ok": ok_text,
        "rewrite_ok": "1" if bool(rewrite_row.get("ok", False)) else "0",
        "rewrite_elapsed_ms": elapsed_text,
        "rewrite_status": "ok" if bool(rewrite_row.get("ok", False)) else "failed",
    }


def append_runtime_5min_checklist_summary_lines(
    lines: list[str],
    with_5min_checklist: bool,
    checklist_report: Path,
) -> None:
    if not with_5min_checklist:
        return
    snap = load_runtime_5min_checklist_snapshot(checklist_report)
    lines.append(f"[ci-gate-summary] seamgrim_5min_checklist={checklist_report}")
    lines.append(f"[ci-gate-summary] seamgrim_5min_checklist_ok={snap['ok']}")
    lines.append(f"[ci-gate-summary] seamgrim_runtime_5min_rewrite_motion_projectile={snap['rewrite_ok']}")
    lines.append(f"[ci-gate-summary] seamgrim_runtime_5min_rewrite_elapsed_ms={snap['rewrite_elapsed_ms']}")
    lines.append(f"[ci-gate-summary] seamgrim_runtime_5min_rewrite_status={snap['rewrite_status']}")


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
    seamgrim_control_exposure_failures_report: Path,
    seamgrim_rewrite_overlay_quality_report: Path,
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
    print(f" - seamgrim_control_exposure_failures={seamgrim_control_exposure_failures_report}")
    print(f" - seamgrim_rewrite_overlay_quality={seamgrim_rewrite_overlay_quality_report}")
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
