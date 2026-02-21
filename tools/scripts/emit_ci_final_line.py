#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
from datetime import datetime, timezone
from pathlib import Path

from ci_verify_codes import SUMMARY_VERIFY_CODES as VERIFY_CODES

INDEX_SCHEMA = "ddn.ci.aggregate_gate.index.v1"
SUMMARY_PATTERNS = (
    "*.ci_gate_summary_line.txt",
    "*.ci_gate_result_line.txt",
    "*.ci_gate_final_status_line.txt",
    "*.ci_aggregate_status_line.txt",
)
ARTIFACT_KEYS = (
    "summary",
    "summary_line",
    "ci_gate_result_json",
    "ci_gate_result_parse_json",
    "ci_gate_badge_json",
    "final_status_line",
    "final_status_parse_json",
    "aggregate_status_line",
    "aggregate_status_parse_json",
    "age3_close_status_json",
    "age3_close_status_line",
    "age3_close_badge_json",
    "ci_fail_brief_txt",
    "ci_fail_triage_json",
)
FAILED_STEP_PRIORITY = (
    "seamgrim_ci_gate",
    "age3_close",
    "oi405_406_close",
    "aggregate_combine",
    "aggregate_status_line_check",
    "final_status_line_check",
    "ci_gate_result_check",
    "summary_line_check",
    "ci_gate_outputs_consistency_check",
)
FAILED_STEP_PRIORITY_MAP = {name: idx for idx, name in enumerate(FAILED_STEP_PRIORITY)}
SUMMARY_DETAIL_RE = re.compile(r"^failed_step_detail=([^ ]+) rc=([-]?\d+) cmd=(.+)$")
SUMMARY_LOGS_RE = re.compile(r"^failed_step_logs=([^ ]+) stdout=([^ ]+) stderr=([^ ]+)$")


def clip(text: str, limit: int = 240) -> str:
    s = str(text).strip()
    if len(s) <= limit:
        return s
    return s[: max(0, limit - 3)] + "..."


def load_json(path: Path) -> dict | None:
    try:
        data = json.loads(path.read_text(encoding="utf-8-sig"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def load_line(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8-sig").strip()
    except Exception:
        return ""


def read_tail_lines(path: Path, line_count: int) -> list[str]:
    if line_count <= 0:
        return []
    try:
        content = path.read_text(encoding="utf-8-sig", errors="replace")
    except Exception:
        return []
    lines = content.splitlines()
    if not lines:
        return []
    tail = lines[-line_count:]
    out: list[str] = []
    for line in tail:
        stripped = str(line).rstrip()
        if stripped:
            out.append(stripped)
    return out


def first_nonempty_line(path: Path, prefer_errorish: bool) -> str:
    try:
        content = path.read_text(encoding="utf-8-sig", errors="replace")
    except Exception:
        return ""
    lines = [str(line).strip() for line in content.splitlines() if str(line).strip()]
    if not lines:
        return ""
    if not prefer_errorish:
        return lines[0]
    error_tokens = ("error", "failed", "fail", "exception", "traceback", "panic", "launch_error")
    lowered_lines = [line.lower() for line in lines]
    for idx, lowered in enumerate(lowered_lines):
        if any(token in lowered for token in error_tokens):
            return lines[idx]
    return lines[0]


def sorted_failed_rows(steps: list[dict]) -> list[dict]:
    indexed_rows = []
    for idx, row in enumerate(steps):
        if not isinstance(row, dict):
            continue
        if bool(row.get("ok", False)):
            continue
        name = str(row.get("name", "-")).strip()
        priority = FAILED_STEP_PRIORITY_MAP.get(name, len(FAILED_STEP_PRIORITY))
        indexed_rows.append((priority, idx, row))
    indexed_rows.sort(key=lambda item: (item[0], item[1]))
    return [row for _, __, row in indexed_rows]


def quote_token(text: str) -> str:
    escaped = str(text).replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def normalize_path(raw_path: str) -> Path:
    return Path(str(raw_path).replace("\\", "/"))


def normalize_path_text(raw_path: str) -> str:
    text = str(raw_path).strip()
    if not text:
        return ""
    return text.replace("\\", "/")


def failed_row_details(row: dict) -> tuple[str, str, str, str]:
    name = str(row.get("name", "-")).strip() or "-"
    stdout_log = str(row.get("stdout_log_path", "")).strip()
    stderr_log = str(row.get("stderr_log_path", "")).strip()
    stderr_path = normalize_path(stderr_log) if stderr_log else None
    stdout_path = normalize_path(stdout_log) if stdout_log else None
    brief = ""
    if stderr_path is not None and stderr_path.exists():
        brief = first_nonempty_line(stderr_path, prefer_errorish=True)
    if not brief and stdout_path is not None and stdout_path.exists():
        brief = first_nonempty_line(stdout_path, prefer_errorish=True)
    return name, stdout_log, stderr_log, brief


def build_failure_brief_line(
    index_doc: dict | None,
    result_doc: dict | None,
    final_line: str,
    limit: int,
) -> str:
    status = str(result_doc.get("status", "")).strip() if isinstance(result_doc, dict) else ""
    reason = str(result_doc.get("reason", "-")).strip() if isinstance(result_doc, dict) else "-"
    if not status:
        status = "unknown"
    if not reason:
        reason = "-"
    failed_steps: list[str] = []
    top_step = "-"
    top_message = "-"
    if isinstance(index_doc, dict):
        steps = index_doc.get("steps")
        if isinstance(steps, list):
            failed_rows = sorted_failed_rows(steps)
            failed_steps = [str(row.get("name", "-")).strip() or "-" for row in failed_rows]
            if failed_rows:
                name, _, _, brief = failed_row_details(failed_rows[0])
                top_step = name
                if brief:
                    top_message = brief
    failed_steps_count = len(failed_steps)
    failed_steps_joined = ",".join(failed_steps[: max(1, limit)]) if failed_steps else "-"
    compact = clip(final_line, 220) if final_line else "-"
    return (
        f"status={status} "
        f"reason={quote_token(clip(reason, 180))} "
        f"failed_steps_count={failed_steps_count} "
        f"failed_steps={quote_token(clip(failed_steps_joined, 180))} "
        f"top_step={top_step} "
        f"top_message={quote_token(clip(top_message, 180))} "
        f"final_line={quote_token(compact)}"
    )


def resolve_failure_brief_out(raw: str, prefix: str) -> Path:
    token = "__PREFIX__"
    p = str(raw).strip()
    if token in p:
        resolved_prefix = prefix.strip() or "noprefix"
        p = p.replace(token, resolved_prefix)
    return Path(p)


def write_failure_brief(path: Path, line: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(line.rstrip() + "\n", encoding="utf-8")
    print(f"[ci-final-meta] failure_brief_out={path}")


def failed_steps_payload(index_doc: dict | None, limit: int = 8) -> list[dict[str, object]]:
    if not isinstance(index_doc, dict):
        return []
    steps = index_doc.get("steps")
    if not isinstance(steps, list):
        return []
    out: list[dict[str, object]] = []
    for row in sorted_failed_rows(steps)[: max(1, limit)]:
        if not isinstance(row, dict):
            continue
        name, stdout_log, stderr_log, brief = failed_row_details(row)
        out.append(
            {
                "name": name,
                "returncode": int(row.get("returncode", -1)),
                "stdout_log_path": stdout_log,
                "stdout_log_path_norm": normalize_path_text(stdout_log),
                "stderr_log_path": stderr_log,
                "stderr_log_path_norm": normalize_path_text(stderr_log),
                "brief": clip(brief, 220) if brief else "",
            }
        )
    return out


def aggregate_digest_payload(index_doc: dict | None, limit: int = 8) -> list[str]:
    if not isinstance(index_doc, dict):
        return []
    aggregate_path = artifact_path(index_doc, "aggregate")
    if aggregate_path is None or not aggregate_path.exists():
        return []
    aggregate_doc = load_json(aggregate_path)
    if not isinstance(aggregate_doc, dict):
        return []
    failure_digest = aggregate_doc.get("failure_digest")
    if isinstance(failure_digest, list) and failure_digest:
        return [clip(str(item), 260) for item in failure_digest[: max(1, limit)]]
    for bucket_key in ("seamgrim", "age3", "oi405_406"):
        bucket = aggregate_doc.get(bucket_key)
        if not isinstance(bucket, dict):
            continue
        digest = bucket.get("failure_digest")
        if not isinstance(digest, list) or not digest:
            continue
        return [clip(f"{bucket_key}:{item}", 260) for item in digest[: max(1, limit)]]
    return []


def artifacts_payload(index_doc: dict | None) -> dict[str, dict[str, object]]:
    out: dict[str, dict[str, object]] = {}
    if not isinstance(index_doc, dict):
        return out
    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return out
    for key in sorted(reports.keys()):
        raw = str(reports.get(key, "")).strip()
        if not raw:
            continue
        path = normalize_path(raw)
        out[str(key)] = {
            "path": raw,
            "path_norm": normalize_path_text(raw),
            "exists": bool(path.exists()),
        }
    return out


def write_triage_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"[ci-final-meta] triage_json_out={path}")


def patch_triage_artifact_row(payload: dict, key: str, path: Path) -> None:
    artifacts = payload.get("artifacts")
    if not isinstance(artifacts, dict):
        artifacts = {}
        payload["artifacts"] = artifacts
    raw = str(path)
    artifacts[key] = {
        "path": raw,
        "path_norm": normalize_path_text(raw),
        "exists": True,
    }


def patch_triage_output_refs(payload: dict, brief_path: Path | None, triage_path: Path | None) -> None:
    if brief_path is not None:
        patch_triage_artifact_row(payload, "ci_fail_brief_txt", brief_path)
    if triage_path is not None:
        patch_triage_artifact_row(payload, "ci_fail_triage_json", triage_path)


def select_latest_index(report_dir: Path, pattern: str, prefix: str) -> tuple[Path | None, dict | None]:
    candidates = sorted(
        report_dir.glob(pattern),
        key=lambda p: (p.stat().st_mtime_ns, str(p)),
        reverse=True,
    )
    selected_path: Path | None = None
    selected_doc: dict | None = None
    for path in candidates:
        doc = load_json(path)
        if not isinstance(doc, dict):
            continue
        if str(doc.get("schema", "")).strip() != INDEX_SCHEMA:
            continue
        if prefix and str(doc.get("report_prefix", "")).strip() != prefix:
            continue
        selected_path = path
        selected_doc = doc
        break
    if selected_path is not None:
        return selected_path, selected_doc
    for path in candidates:
        doc = load_json(path)
        if not isinstance(doc, dict):
            continue
        if str(doc.get("schema", "")).strip() != INDEX_SCHEMA:
            continue
        return path, doc
    return None, None


def first_existing_line(report_dir: Path) -> str:
    for pattern in SUMMARY_PATTERNS:
        files = sorted(
            report_dir.glob(pattern),
            key=lambda p: (p.stat().st_mtime_ns, str(p)),
            reverse=True,
        )
        for path in files:
            line = load_line(path)
            if line:
                print(f"[ci-final-meta] fallback_line_source={path}")
                return line
    return ""


def artifact_path(index_doc: dict, key: str) -> Path | None:
    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return None
    raw_path = str(reports.get(key, "")).strip()
    if not raw_path:
        return None
    return normalize_path(raw_path)


def print_artifact_lines(index_doc: dict) -> None:
    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return
    for key in ARTIFACT_KEYS:
        raw_path = str(reports.get(key, "")).strip()
        if not raw_path:
            continue
        path = normalize_path(raw_path)
        print(f"[ci-artifact] key={key} exists={int(path.exists())} path={path}")


def print_result_meta(index_doc: dict) -> None:
    result_path = artifact_path(index_doc, "ci_gate_result_json")
    if result_path is None or not result_path.exists():
        return
    result_doc = load_json(result_path)
    if not isinstance(result_doc, dict):
        return
    status = str(result_doc.get("status", "-")).strip() or "-"
    ok = int(bool(result_doc.get("ok", False)))
    failed_steps = result_doc.get("failed_steps", "-")
    aggregate_status = str(result_doc.get("aggregate_status", "-")).strip() or "-"
    print(
        f"[ci-final-meta] result_status={status} ok={ok} "
        f"failed_steps={failed_steps} aggregate_status={aggregate_status}"
    )
    badge_path = artifact_path(index_doc, "ci_gate_badge_json")
    if badge_path is None or not badge_path.exists():
        return
    badge_doc = load_json(badge_path)
    if not isinstance(badge_doc, dict):
        return
    badge_status = str(badge_doc.get("status", "-")).strip() or "-"
    color = str(badge_doc.get("color", "-")).strip() or "-"
    print(f"[ci-final-meta] badge_status={badge_status} badge_color={color}")


def parse_summary_report(path: Path) -> tuple[str | None, dict[str, str], list[tuple[str, str]]]:
    try:
        text = path.read_text(encoding="utf-8-sig")
    except Exception:
        return None, {}, []
    status: str | None = None
    kv: dict[str, str] = {}
    rows: list[tuple[str, str]] = []
    for raw in text.splitlines():
        line = str(raw).strip()
        if not line.startswith("[ci-gate-summary] "):
            continue
        body = line[len("[ci-gate-summary] ") :]
        if body in {"PASS", "FAIL"}:
            status = body.lower()
            continue
        if "=" not in body:
            continue
        key, value = body.split("=", 1)
        key = key.strip()
        value = value.strip()
        if not key:
            continue
        kv[key] = value
        rows.append((key, value))
    return status, kv, rows


def summary_failed_step_names(value: str) -> list[str]:
    raw = str(value).strip()
    if not raw or raw == "(none)":
        return []
    return [item.strip() for item in raw.split(",") if item.strip()]


def verify_summary_report(index_doc: dict, result_doc: dict | None) -> tuple[bool, list[str], int, int]:
    def add_issue(out: list[str], code: str) -> None:
        if code not in out:
            out.append(code)

    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return False, [VERIFY_CODES["REPORTS_MISSING"]], 0, 0
    summary_raw = str(reports.get("summary", "")).strip()
    if not summary_raw:
        return False, [VERIFY_CODES["SUMMARY_PATH_MISSING"]], 0, 0
    summary_path = normalize_path(summary_raw)
    if not summary_path.exists():
        return False, [VERIFY_CODES["SUMMARY_FILE_MISSING"]], 0, 0
    status, kv, rows = parse_summary_report(summary_path)
    if status not in {"pass", "fail"}:
        return False, [VERIFY_CODES["STATUS_MISSING"]], 0, 0

    expected_status = str(result_doc.get("status", "")).strip() if isinstance(result_doc, dict) else ""
    issues: list[str] = []
    if expected_status in {"pass", "fail"} and expected_status != status:
        add_issue(issues, VERIFY_CODES["STATUS_MISMATCH"])

    detail_rows = [value for key, value in rows if key == "failed_step_detail"]
    log_rows = [value for key, value in rows if key == "failed_step_logs"]

    if status == "pass":
        if str(kv.get("failed_steps", "")).strip() != "(none)":
            add_issue(issues, VERIFY_CODES["PASS_FAILED_STEPS_NOT_NONE"])
        if detail_rows:
            add_issue(issues, VERIFY_CODES["PASS_HAS_DETAIL"])
        if log_rows:
            add_issue(issues, VERIFY_CODES["PASS_HAS_LOGS"])
        return len(issues) == 0, issues, len(detail_rows), len(log_rows)

    failed_steps = summary_failed_step_names(str(kv.get("failed_steps", "")))
    if not failed_steps:
        add_issue(issues, VERIFY_CODES["FAIL_FAILED_STEPS_EMPTY"])

    parsed_detail_steps: list[str] = []
    for row in detail_rows:
        match = SUMMARY_DETAIL_RE.match(f"failed_step_detail={row}")
        if not match:
            add_issue(issues, VERIFY_CODES["DETAIL_FORMAT_INVALID"])
            continue
        name = str(match.group(1)).strip()
        rc = int(match.group(2))
        cmd = str(match.group(3)).strip()
        parsed_detail_steps.append(name)
        if rc == 0:
            add_issue(issues, VERIFY_CODES["DETAIL_RC_ZERO"])
        if not cmd:
            add_issue(issues, VERIFY_CODES["DETAIL_CMD_EMPTY"])
        if failed_steps and name not in failed_steps:
            add_issue(issues, VERIFY_CODES["DETAIL_NOT_IN_FAILED_STEPS"])
    if not detail_rows:
        add_issue(issues, VERIFY_CODES["FAIL_DETAIL_MISSING"])

    parsed_log_steps: list[str] = []
    for row in log_rows:
        match = SUMMARY_LOGS_RE.match(f"failed_step_logs={row}")
        if not match:
            add_issue(issues, VERIFY_CODES["LOGS_FORMAT_INVALID"])
            continue
        name = str(match.group(1)).strip()
        stdout_path = str(match.group(2)).strip()
        stderr_path = str(match.group(3)).strip()
        parsed_log_steps.append(name)
        if failed_steps and name not in failed_steps:
            add_issue(issues, VERIFY_CODES["LOGS_NOT_IN_FAILED_STEPS"])
        for raw_path in (stdout_path, stderr_path):
            if raw_path == "-":
                continue
            path = normalize_path(raw_path)
            if not path.exists():
                add_issue(issues, VERIFY_CODES["LOG_PATH_MISSING"])

    steps = index_doc.get("steps")
    if isinstance(steps, list):
        index_failed = [str(row.get("name", "")).strip() for row in steps if isinstance(row, dict) and not bool(row.get("ok", False))]
        index_failed_set = {name for name in index_failed if name}
        if index_failed_set and failed_steps:
            for name in failed_steps:
                if name not in index_failed_set:
                    add_issue(issues, VERIFY_CODES["SUMMARY_FAILED_STEP_NOT_IN_INDEX"])
        for name in parsed_detail_steps:
            if index_failed_set and name not in index_failed_set:
                add_issue(issues, VERIFY_CODES["DETAIL_NOT_IN_INDEX"])
        for name in parsed_log_steps:
            if index_failed_set and name not in index_failed_set:
                add_issue(issues, VERIFY_CODES["LOGS_NOT_IN_INDEX"])
    return len(issues) == 0, issues, len(detail_rows), len(log_rows)


def print_summary_verify(index_doc: dict, result_doc: dict | None) -> bool:
    summary_ok, summary_issues, detail_count, logs_count = verify_summary_report(index_doc, result_doc)
    issue_count = len(summary_issues)
    top_issue_code = summary_issues[0] if summary_issues else "-"
    top_issues = ",".join(summary_issues[:3]) if summary_issues else "-"
    if summary_ok:
        print(
            f"[ci-fail-verify] summary=ok detail_rows={detail_count} "
            f"log_rows={logs_count} issue_count={issue_count} "
            f"top_issue_code={top_issue_code} top_issues={top_issues}"
        )
        return True
    print(
        f"[ci-fail-verify] summary=fail detail_rows={detail_count} "
        f"log_rows={logs_count} issue_count={issue_count} "
        f"top_issue_code={top_issue_code} top_issues={clip(top_issues, 260)}"
    )
    return False


def build_triage_payload(
    index_doc: dict | None,
    result_doc: dict | None,
    final_line: str,
    summary_verify_ok: bool | None = None,
    summary_verify_issues: list[str] | None = None,
    max_steps: int = 8,
    max_digest: int = 8,
) -> dict:
    status = str(result_doc.get("status", "unknown")).strip() if isinstance(result_doc, dict) else "unknown"
    reason = str(result_doc.get("reason", "-")).strip() if isinstance(result_doc, dict) else "-"
    if not status:
        status = "unknown"
    if not reason:
        reason = "-"
    prefix = str(index_doc.get("report_prefix", "")).strip() if isinstance(index_doc, dict) else ""
    summary_path_hint = "-"
    if isinstance(index_doc, dict):
        reports = index_doc.get("reports")
        if isinstance(reports, dict):
            summary_path_hint = str(reports.get("summary", "")).strip() or "-"
    failed_steps = failed_steps_payload(index_doc, limit=max_steps)
    digest = aggregate_digest_payload(index_doc, limit=max_digest)
    if summary_verify_ok is None:
        if isinstance(index_doc, dict):
            summary_verify_ok, verify_issues, _, _ = verify_summary_report(index_doc, result_doc)
            if summary_verify_issues is None:
                summary_verify_issues = verify_issues
        else:
            summary_verify_ok = False
    if summary_verify_issues is None:
        summary_verify_issues = []
    summary_verify_issue_codes = [str(item) for item in summary_verify_issues[:16]]
    summary_verify_top_issue = summary_verify_issue_codes[0] if summary_verify_issue_codes else "-"
    payload = {
        "schema": "ddn.ci.fail_triage.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "status": status,
        "reason": reason,
        "report_prefix": prefix,
        "final_line": clip(final_line, 360) if final_line else "-",
        "summary_verify_ok": bool(summary_verify_ok),
        "summary_verify_issues": summary_verify_issue_codes,
        "summary_verify_issues_count": len(summary_verify_issue_codes),
        "summary_verify_top_issue": summary_verify_top_issue,
        "failed_steps": failed_steps,
        "failed_steps_count": len(failed_steps),
        "aggregate_digest": digest,
        "aggregate_digest_count": len(digest),
        "summary_report_path_hint": summary_path_hint,
        "summary_report_path_hint_norm": normalize_path_text(summary_path_hint) if summary_path_hint != "-" else "-",
        "artifacts": artifacts_payload(index_doc),
    }
    return payload


def load_result_doc(index_doc: dict) -> dict | None:
    result_path = artifact_path(index_doc, "ci_gate_result_json")
    if result_path is None or not result_path.exists():
        return None
    result_doc = load_json(result_path)
    return result_doc if isinstance(result_doc, dict) else None


def print_failure_digest(
    index_doc: dict,
    result_doc: dict | None,
    limit: int,
    tail_lines: int,
) -> bool:
    if limit <= 0:
        return print_summary_verify(index_doc, result_doc)

    reason = "-"
    status = "-"
    if isinstance(result_doc, dict):
        reason = str(result_doc.get("reason", "-")).strip() or "-"
        status = str(result_doc.get("status", "-")).strip() or "-"
    print(f"[ci-fail] status={status} reason={clip(reason, 220)}")

    steps = index_doc.get("steps")
    if isinstance(steps, list):
        failed_rows = sorted_failed_rows(steps)
        failed_steps = [str(row.get("name", "-")) for row in failed_rows]
        if failed_steps:
            joined = ",".join(failed_steps[:limit])
            print(f"[ci-fail] failed_steps={joined}")
        for row in failed_rows[:limit]:
            name, stdout_log, stderr_log, brief = failed_row_details(row)
            if brief:
                print(f"[ci-fail-brief] step={name} message={clip(brief, 220)}")
            if stdout_log or stderr_log:
                print(
                    f"[ci-fail] step_logs={name} "
                    f"stdout={stdout_log or '-'} stderr={stderr_log or '-'}"
                )
            if tail_lines <= 0:
                continue
            selected_stream = ""
            selected_path: Path | None = None
            for stream_name, raw_path in (("stderr", stderr_log), ("stdout", stdout_log)):
                if not raw_path:
                    continue
                candidate = normalize_path(raw_path)
                if not candidate.exists():
                    continue
                selected_stream = stream_name
                selected_path = candidate
                break
            if selected_path is None:
                continue
            tail = read_tail_lines(selected_path, tail_lines)
            if not tail:
                continue
            print(
                f"[ci-fail-tail] step={name} stream={selected_stream} "
                f"path={selected_path} lines={len(tail)}"
            )
            for line in tail:
                print(f"[ci-fail-tail] {clip(line, 240)}")

    aggregate_path = artifact_path(index_doc, "aggregate")
    aggregate_doc = load_json(aggregate_path) if aggregate_path is not None and aggregate_path.exists() else None
    digest_printed = False
    if isinstance(aggregate_doc, dict):
        failure_digest = aggregate_doc.get("failure_digest")
        if isinstance(failure_digest, list) and failure_digest:
            for item in failure_digest[:limit]:
                print(f"[ci-fail] digest={clip(str(item), 260)}")
            digest_printed = True
        if not digest_printed:
            for bucket_key in ("seamgrim", "age3", "oi405_406"):
                bucket = aggregate_doc.get(bucket_key)
                if not isinstance(bucket, dict):
                    continue
                digest = bucket.get("failure_digest")
                if not isinstance(digest, list) or not digest:
                    continue
                for item in digest[:limit]:
                    print(f"[ci-fail] {bucket_key}={clip(str(item), 240)}")
                digest_printed = True
                break
    if not digest_printed:
        print("[ci-fail] digest=-")

    return print_summary_verify(index_doc, result_doc)


def line_from_index(index_doc: dict) -> str:
    for key in ("summary_line", "ci_gate_result_line", "final_status_line", "aggregate_status_line"):
        path = artifact_path(index_doc, key)
        if path is None:
            continue
        line = load_line(path)
        if line:
            print(f"[ci-final-meta] primary_line_source={path}")
            return line
    return ""


def main() -> int:
    parser = argparse.ArgumentParser(description="Emit single final CI status line from aggregate gate reports")
    parser.add_argument("--report-dir", default="build/reports", help="report directory")
    parser.add_argument("--index-pattern", default="*.ci_gate_report_index.detjson", help="index file glob")
    parser.add_argument("--prefix", default="", help="optional expected report prefix")
    parser.add_argument("--print-artifacts", action="store_true", help="print key artifact paths and existence")
    parser.add_argument(
        "--print-failure-digest",
        type=int,
        default=0,
        help="on failed status, print up to N failure-digest lines",
    )
    parser.add_argument(
        "--print-failure-tail-lines",
        type=int,
        default=0,
        help="on failed status, print up to N tail lines from failed step logs (stderr first)",
    )
    parser.add_argument(
        "--failure-brief-out",
        default="",
        help="optional one-line failure-brief txt output path (supports __PREFIX__ token)",
    )
    parser.add_argument(
        "--triage-json-out",
        default="",
        help="optional ci failure triage json output path (supports __PREFIX__ token)",
    )
    parser.add_argument(
        "--fail-on-summary-verify-error",
        action="store_true",
        help="return non-zero when fail-case summary(detail/log rows) verification fails",
    )
    parser.add_argument("--require-final-line", action="store_true", help="return non-zero when final line is missing")
    args = parser.parse_args()

    report_dir = Path(args.report_dir)
    if not report_dir.exists():
        print(f"[ci-final-meta] report_dir_missing={report_dir}")
        if args.require_final_line:
            print("[ci-final] status=unknown reason=report_dir_missing")
            return 1
        print("[ci-final] status=unknown reason=report_dir_missing")
        return 0

    index_path, index_doc = select_latest_index(report_dir, args.index_pattern, args.prefix.strip())
    result_doc: dict | None = None
    if index_path is not None and isinstance(index_doc, dict):
        prefix = str(index_doc.get("report_prefix", "")).strip() or "-"
        print(f"[ci-final-meta] report_index={index_path} prefix={prefix}")
        step_log_dir = str(index_doc.get("step_log_dir", "")).strip()
        if step_log_dir:
            print(f"[ci-final-meta] step_log_dir={step_log_dir}")
        if args.print_artifacts:
            print_artifact_lines(index_doc)
        print_result_meta(index_doc)
        result_doc = load_result_doc(index_doc)
        final_line = line_from_index(index_doc)
    else:
        print("[ci-final-meta] report_index=missing")
        final_line = ""

    if not final_line:
        final_line = first_existing_line(report_dir)

    if final_line:
        print(f"[ci-final] {clip(final_line, 360)}")
        status = str(result_doc.get("status", "")).strip() if isinstance(result_doc, dict) else ""
        prefix_value = str(index_doc.get("report_prefix", "")).strip() if isinstance(index_doc, dict) else ""
        brief_path_resolved = (
            resolve_failure_brief_out(args.failure_brief_out, prefix_value) if args.failure_brief_out.strip() else None
        )
        triage_path_resolved = (
            resolve_failure_brief_out(args.triage_json_out, prefix_value) if args.triage_json_out.strip() else None
        )
        summary_verify_ok: bool | None = None
        summary_verify_issues: list[str] | None = None
        if args.print_failure_digest > 0 and status and status != "pass" and isinstance(index_doc, dict):
            summary_verify_ok = print_failure_digest(
                index_doc,
                result_doc,
                args.print_failure_digest,
                max(0, int(args.print_failure_tail_lines)),
            )
            if summary_verify_ok is False:
                ok, issues, _, _ = verify_summary_report(index_doc, result_doc)
                if not ok:
                    summary_verify_issues = issues
        if (
            args.fail_on_summary_verify_error
            and status
            and status != "pass"
            and isinstance(index_doc, dict)
        ):
            if summary_verify_ok is None:
                summary_verify_ok = print_summary_verify(index_doc, result_doc)
            if not summary_verify_ok:
                if brief_path_resolved is not None:
                    brief_line = build_failure_brief_line(
                        index_doc,
                        result_doc,
                        final_line,
                        max(1, int(args.print_failure_digest) or 6),
                    )
                    write_failure_brief(brief_path_resolved, brief_line)
                if triage_path_resolved is not None:
                    triage_payload = build_triage_payload(
                        index_doc,
                        result_doc,
                        final_line,
                        summary_verify_ok=False,
                        summary_verify_issues=summary_verify_issues,
                    )
                    patch_triage_output_refs(triage_payload, brief_path_resolved, triage_path_resolved)
                    write_triage_json(triage_path_resolved, triage_payload)
                return 2
        if brief_path_resolved is not None:
            brief_line = build_failure_brief_line(index_doc, result_doc, final_line, max(1, int(args.print_failure_digest) or 6))
            write_failure_brief(brief_path_resolved, brief_line)
        if triage_path_resolved is not None:
            triage_payload = build_triage_payload(
                index_doc,
                result_doc,
                final_line,
                summary_verify_ok=summary_verify_ok,
                summary_verify_issues=summary_verify_issues,
            )
            patch_triage_output_refs(triage_payload, brief_path_resolved, triage_path_resolved)
            write_triage_json(triage_path_resolved, triage_payload)
        return 0

    print("[ci-final] status=unknown reason=final_line_missing")
    if args.print_failure_digest > 0 and isinstance(index_doc, dict):
        summary_verify_ok = print_failure_digest(
            index_doc,
            result_doc,
            args.print_failure_digest,
            max(0, int(args.print_failure_tail_lines)),
        )
    else:
        summary_verify_ok = None
    prefix_value = str(index_doc.get("report_prefix", "")).strip() if isinstance(index_doc, dict) else ""
    brief_path_resolved = (
        resolve_failure_brief_out(args.failure_brief_out, prefix_value) if args.failure_brief_out.strip() else None
    )
    triage_path_resolved = (
        resolve_failure_brief_out(args.triage_json_out, prefix_value) if args.triage_json_out.strip() else None
    )
    if brief_path_resolved is not None:
        brief_line = build_failure_brief_line(index_doc, result_doc, final_line="", limit=max(1, int(args.print_failure_digest) or 6))
        write_failure_brief(brief_path_resolved, brief_line)
    if triage_path_resolved is not None:
        triage_payload = build_triage_payload(
            index_doc,
            result_doc,
            final_line="",
            summary_verify_ok=summary_verify_ok,
            summary_verify_issues=None,
        )
        patch_triage_output_refs(triage_payload, brief_path_resolved, triage_path_resolved)
        write_triage_json(triage_path_resolved, triage_payload)
    return 1 if args.require_final_line else 0


if __name__ == "__main__":
    raise SystemExit(main())
