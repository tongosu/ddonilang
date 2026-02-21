#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

from ci_check_error_codes import EMIT_ARTIFACTS_CODES as CODES
from ci_check_error_codes import SUMMARY_VERIFY_CODES

INDEX_SCHEMA = "ddn.ci.aggregate_gate.index.v1"
TRIAGE_SCHEMA = "ddn.ci.fail_triage.v1"
TOKEN_RE = re.compile(r'([A-Za-z0-9_]+)=("([^"\\]|\\.)*"|[^ \t]+)')
SUMMARY_VERIFY_CODES_SET = set(SUMMARY_VERIFY_CODES.values())


def fail(msg: str, code: str = "E_CHECK") -> int:
    print(f"[ci-emit-artifacts-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8-sig"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def load_line(path: Path) -> str:
    if not path.exists():
        return ""
    try:
        return path.read_text(encoding="utf-8-sig").strip()
    except Exception:
        return ""


def normalize_path_text(raw: str) -> str:
    text = str(raw).strip()
    return text.replace("\\", "/")


def resolve_path(raw: str) -> Path | None:
    text = str(raw).strip()
    if not text:
        return None
    return Path(text.replace("\\", "/"))


def parse_tokens(line: str) -> dict[str, str] | None:
    text = str(line).strip()
    if not text:
        return None
    out: dict[str, str] = {}
    pos = 0
    for match in TOKEN_RE.finditer(text):
        if text[pos : match.start()].strip():
            return None
        key = match.group(1)
        raw = match.group(2)
        if raw.startswith('"'):
            try:
                value = json.loads(raw)
            except Exception:
                return None
        else:
            value = raw
        out[key] = str(value)
        pos = match.end()
    if text[pos:].strip():
        return None
    return out


def clip(text: str, limit: int) -> str:
    value = str(text).strip()
    if len(value) <= limit:
        return value
    return value[: max(0, limit - 3)] + "..."


def parse_summary_report(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        lines = path.read_text(encoding="utf-8-sig").splitlines()
    except Exception:
        return None
    status = ""
    failed_steps: list[str] = []
    failed_step_details: dict[str, str] = {}
    failed_step_logs: dict[str, dict[str, str]] = {}
    prefix = "[ci-gate-summary] "
    for raw in lines:
        line = str(raw).strip()
        if not line.startswith(prefix):
            continue
        body = line[len(prefix) :].strip()
        if body == "PASS":
            status = "pass"
            continue
        if body == "FAIL":
            status = "fail"
            continue
        if body.startswith("failed_steps="):
            payload = body[len("failed_steps=") :].strip()
            if payload in ("", "-", "(none)"):
                failed_steps = []
            else:
                failed_steps = [token.strip() for token in payload.split(",") if token.strip()]
            continue
        if body.startswith("failed_step_detail="):
            payload = body[len("failed_step_detail=") :]
            step_id = payload.split(" ", 1)[0].strip()
            if step_id:
                failed_step_details[step_id] = body
            continue
        if body.startswith("failed_step_logs="):
            payload = body[len("failed_step_logs=") :]
            parts = payload.split(" ", 1)
            step_id = parts[0].strip()
            if not step_id:
                continue
            row = {"stdout": "", "stderr": ""}
            if len(parts) > 1 and parts[1].strip():
                tokens = parse_tokens(parts[1].strip())
                if isinstance(tokens, dict):
                    row["stdout"] = str(tokens.get("stdout", "")).strip()
                    row["stderr"] = str(tokens.get("stderr", "")).strip()
            failed_step_logs[step_id] = row
    return {
        "status": status,
        "failed_steps": failed_steps,
        "failed_step_details": failed_step_details,
        "failed_step_logs": failed_step_logs,
    }


def select_latest_index(report_dir: Path, pattern: str, prefix: str) -> tuple[Path | None, dict | None]:
    candidates = sorted(
        report_dir.glob(pattern),
        key=lambda p: (p.stat().st_mtime_ns, str(p)),
        reverse=True,
    )
    for path in candidates:
        doc = load_json(path)
        if not isinstance(doc, dict):
            continue
        if str(doc.get("schema", "")).strip() != INDEX_SCHEMA:
            continue
        if prefix and str(doc.get("report_prefix", "")).strip() != prefix:
            continue
        return path, doc
    return None, None


def artifact_path(index_doc: dict, key: str) -> Path | None:
    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return None
    return resolve_path(str(reports.get(key, "")).strip())


def artifact_path_text(index_doc: dict, key: str) -> str:
    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return ""
    return str(reports.get(key, "")).strip()


def validate_triage_artifact_row(name: str, row: dict) -> str | None:
    path_text = str(row.get("path", "")).strip()
    path_norm = str(row.get("path_norm", "")).strip()
    exists_value = row.get("exists")
    if not path_text:
        return f"triage artifacts.{name}.path missing"
    expected_norm = normalize_path_text(path_text)
    if path_norm != expected_norm:
        return f"triage artifacts.{name}.path_norm mismatch triage={path_norm} expected={expected_norm}"
    if not isinstance(exists_value, bool):
        return f"triage artifacts.{name}.exists must be bool"
    resolved = resolve_path(path_text)
    if resolved is None:
        return f"triage artifacts.{name}.path resolve failed"
    expected_exists = bool(resolved.exists())
    if exists_value != expected_exists:
        return (
            f"triage artifacts.{name}.exists mismatch triage={int(exists_value)} "
            f"actual={int(expected_exists)} path={resolved}"
        )
    return None


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate emitter outputs (brief/triage) against report index")
    parser.add_argument("--report-dir", default="build/reports", help="report directory")
    parser.add_argument("--index-pattern", default="*.ci_gate_report_index.detjson", help="index file glob")
    parser.add_argument("--prefix", default="", help="optional report prefix")
    parser.add_argument("--require-brief", action="store_true", help="require ci_fail_brief_txt artifact")
    parser.add_argument("--require-triage", action="store_true", help="require ci_fail_triage_json artifact")
    args = parser.parse_args()

    report_dir = Path(args.report_dir)
    if not report_dir.exists():
        return fail(f"missing report-dir: {report_dir}", code=CODES["REPORT_DIR_MISSING"])
    index_path, index_doc = select_latest_index(report_dir, args.index_pattern, args.prefix.strip())
    if index_path is None or not isinstance(index_doc, dict):
        return fail(
            f"index not found in {report_dir} pattern={args.index_pattern} prefix={args.prefix.strip() or '-'}",
            code=CODES["INDEX_NOT_FOUND"],
        )

    result_path = artifact_path(index_doc, "ci_gate_result_json")
    if result_path is None:
        return fail("index missing reports.ci_gate_result_json", code=CODES["INDEX_RESULT_PATH_MISSING"])
    result_doc = load_json(result_path)
    if not isinstance(result_doc, dict):
        return fail(f"invalid result json: {result_path}", code=CODES["RESULT_JSON_INVALID"])
    if str(result_doc.get("schema", "")).strip() != "ddn.ci.gate_result.v1":
        return fail(f"result schema mismatch: {result_doc.get('schema')}", code=CODES["RESULT_SCHEMA_MISMATCH"])
    result_status = str(result_doc.get("status", "")).strip() or "unknown"
    result_reason = str(result_doc.get("reason", "-")).strip() or "-"
    try:
        result_failed_steps = int(result_doc.get("failed_steps", 0))
    except Exception:
        return fail("result failed_steps must be int", code=CODES["RESULT_FAILED_STEPS_TYPE"])
    if result_failed_steps < 0:
        return fail(f"result failed_steps invalid: {result_failed_steps}", code=CODES["RESULT_FAILED_STEPS_NEGATIVE"])
    index_prefix = str(index_doc.get("report_prefix", "")).strip()
    summary_line_path = artifact_path(index_doc, "summary_line")
    summary_line_text = load_line(summary_line_path) if summary_line_path is not None else ""
    summary_path = artifact_path(index_doc, "summary")
    if summary_path is None:
        return fail("index missing reports.summary", code=CODES["INDEX_SUMMARY_PATH_MISSING"])
    summary_status = ""
    summary_failed_steps: list[str] = []
    summary_failed_step_details: dict[str, str] = {}
    summary_failed_step_logs: dict[str, dict[str, str]] = {}
    summary_report_exists = bool(summary_path.exists())
    if summary_report_exists:
        summary_report = parse_summary_report(summary_path)
        if not isinstance(summary_report, dict):
            return fail(f"invalid summary report: {summary_path}")
        summary_status = str(summary_report.get("status", "")).strip()
        parsed_failed_steps = summary_report.get("failed_steps")
        parsed_failed_step_details = summary_report.get("failed_step_details")
        parsed_failed_step_logs = summary_report.get("failed_step_logs")
        if not isinstance(parsed_failed_steps, list):
            return fail("summary failed_steps must be list")
        if not isinstance(parsed_failed_step_details, dict):
            return fail("summary failed_step_details must be object")
        if not isinstance(parsed_failed_step_logs, dict):
            return fail("summary failed_step_logs must be object")
        summary_failed_steps = [str(step).strip() for step in parsed_failed_steps if str(step).strip()]
        summary_failed_step_details = {str(k).strip(): str(v) for k, v in parsed_failed_step_details.items() if str(k).strip()}
        summary_failed_step_logs = {
            str(k).strip(): dict(v) for k, v in parsed_failed_step_logs.items() if str(k).strip() and isinstance(v, dict)
        }
    if result_status not in ("pass", "fail"):
        return fail(f"unsupported result status: {result_status}", code=CODES["RESULT_STATUS_UNSUPPORTED"])
    if result_status == "pass" and result_failed_steps != 0:
        return fail(
            f"pass result must have failed_steps=0, got {result_failed_steps}",
            code=CODES["RESULT_PASS_FAILED_STEPS"],
        )
    if result_status == "fail" and result_failed_steps <= 0:
        return fail(
            f"fail result must have failed_steps>0, got {result_failed_steps}",
            code=CODES["RESULT_FAIL_FAILED_STEPS"],
        )
    summary_status_known = summary_status in ("pass", "fail")
    if summary_report_exists and summary_status_known:
        if summary_status != result_status:
            return fail(
                f"summary status mismatch summary={summary_status or '-'} result={result_status}",
                code=CODES["SUMMARY_STATUS_MISMATCH"],
            )
        if result_status == "pass":
            if summary_failed_steps:
                return fail(f"pass summary must have empty failed_steps, got {','.join(summary_failed_steps)}")
            if summary_failed_step_details:
                return fail("pass summary must not contain failed_step_detail rows")
            if summary_failed_step_logs:
                return fail("pass summary must not contain failed_step_logs rows")
        if result_status == "fail" and not summary_failed_steps:
            return fail("fail summary missing failed_steps")

    reports = index_doc.get("reports")
    if not isinstance(reports, dict):
        return fail("index.reports missing", code=CODES["INDEX_REPORTS_MISSING"])
    for key in ("ci_fail_brief_txt", "ci_fail_triage_json"):
        if not str(reports.get(key, "")).strip():
            return fail(f"index missing reports.{key}", code=CODES["INDEX_REPORT_KEY_MISSING"])

    brief_path = artifact_path(index_doc, "ci_fail_brief_txt")
    brief_required = bool(args.require_brief)
    if brief_path is None:
        if brief_required:
            return fail("index missing reports.ci_fail_brief_txt", code=CODES["INDEX_BRIEF_PATH_MISSING"])
    elif brief_required or brief_path.exists():
        brief_line = load_line(brief_path)
        if not brief_line:
            return fail(f"missing/empty brief file: {brief_path}", code=CODES["BRIEF_REQUIRED_MISSING"])
        brief_tokens = parse_tokens(brief_line)
        if brief_tokens is None:
            return fail("brief token format invalid")
        brief_status = str(brief_tokens.get("status", "")).strip()
        if brief_status and brief_status != result_status:
            return fail(f"brief status mismatch brief={brief_status} result={result_status}")
        brief_reason = str(brief_tokens.get("reason", "")).strip() or "-"
        if brief_reason != result_reason:
            return fail(f"brief reason mismatch brief={brief_reason} result={result_reason}")
        if "failed_steps_count" not in brief_tokens:
            return fail("brief missing failed_steps_count")
        try:
            brief_failed_steps_count = int(str(brief_tokens.get("failed_steps_count", "-1")))
        except Exception:
            return fail("brief failed_steps_count must be int")
        if brief_failed_steps_count < 0:
            return fail(f"brief failed_steps_count invalid: {brief_failed_steps_count}")
        if brief_failed_steps_count != result_failed_steps:
            return fail(
                f"brief failed_steps_count mismatch brief={brief_failed_steps_count} result={result_failed_steps}"
            )
        for key in ("failed_steps", "top_step", "top_message", "final_line"):
            if key not in brief_tokens:
                return fail(f"brief missing {key}")
        if summary_line_text:
            brief_final_line = str(brief_tokens.get("final_line", "")).strip()
            expected_brief_final_line = clip(summary_line_text, 220)
            if brief_final_line != expected_brief_final_line:
                return fail(
                    f"brief final_line mismatch brief={brief_final_line} expected={expected_brief_final_line}"
                )
    elif brief_required:
        return fail(f"missing brief file: {brief_path}", code=CODES["BRIEF_REQUIRED_MISSING"])

    triage_path = artifact_path(index_doc, "ci_fail_triage_json")
    triage_doc: dict | None = None
    triage_required = bool(args.require_triage)
    if triage_path is None:
        if triage_required:
            return fail("index missing reports.ci_fail_triage_json", code=CODES["INDEX_TRIAGE_PATH_MISSING"])
    elif triage_required or triage_path.exists():
        triage_doc = load_json(triage_path)
        if not isinstance(triage_doc, dict):
            return fail(f"invalid triage json: {triage_path}", code=CODES["TRIAGE_REQUIRED_MISSING"])
        if not summary_report_exists:
            return fail(f"triage exists but summary report missing: {summary_path}")
        if str(triage_doc.get("schema", "")).strip() != TRIAGE_SCHEMA:
            return fail(f"triage schema mismatch: {triage_doc.get('schema')}")
        triage_status = str(triage_doc.get("status", "")).strip() or "unknown"
        if triage_status != result_status:
            return fail(f"triage status mismatch triage={triage_status} result={result_status}")
        triage_reason = str(triage_doc.get("reason", "-")).strip() or "-"
        if triage_reason != result_reason:
            return fail(f"triage reason mismatch triage={triage_reason} result={result_reason}")
        triage_final_line = str(triage_doc.get("final_line", "")).strip()
        if summary_line_text:
            expected_triage_final_line = clip(summary_line_text, 360)
            if triage_final_line != expected_triage_final_line:
                return fail(
                    f"triage final_line mismatch triage={triage_final_line} expected={expected_triage_final_line}"
                )
        triage_prefix = str(triage_doc.get("report_prefix", "")).strip()
        if index_prefix and triage_prefix != index_prefix:
            return fail(f"triage report_prefix mismatch triage={triage_prefix} index={index_prefix}")
        if not isinstance(triage_doc.get("summary_verify_ok"), bool):
            return fail("triage summary_verify_ok must be bool")
        summary_verify_ok = bool(triage_doc.get("summary_verify_ok"))
        summary_verify_issues = triage_doc.get("summary_verify_issues")
        if not isinstance(summary_verify_issues, list):
            return fail("triage summary_verify_issues must be list")
        summary_verify_top_issue = str(triage_doc.get("summary_verify_top_issue", "")).strip()
        if not summary_verify_top_issue:
            return fail("triage summary_verify_top_issue missing")
        try:
            summary_verify_issues_count = int(triage_doc.get("summary_verify_issues_count", -1))
        except Exception:
            return fail("triage summary_verify_issues_count must be int")
        if summary_verify_issues_count != len(summary_verify_issues):
            return fail(
                f"triage summary_verify_issues_count mismatch triage={summary_verify_issues_count} "
                f"actual={len(summary_verify_issues)}"
            )
        parsed_summary_verify_issues: list[str] = []
        for idx, item in enumerate(summary_verify_issues):
            code = str(item).strip()
            if not code:
                return fail(f"triage summary_verify_issues[{idx}] empty")
            if code not in SUMMARY_VERIFY_CODES_SET:
                return fail(f"triage summary_verify_issues[{idx}] invalid code: {code}")
            parsed_summary_verify_issues.append(code)
        if summary_verify_ok and parsed_summary_verify_issues:
            return fail("triage summary_verify_ok=1 requires empty summary_verify_issues")
        if summary_verify_ok and summary_verify_top_issue != "-":
            return fail(f"triage summary_verify_ok=1 requires summary_verify_top_issue='-', got={summary_verify_top_issue}")
        if (not summary_verify_ok) and not parsed_summary_verify_issues:
            return fail("triage summary_verify_ok=0 requires non-empty summary_verify_issues")
        if (not summary_verify_ok) and summary_verify_top_issue not in parsed_summary_verify_issues:
            return fail(
                f"triage summary_verify_top_issue must be one of summary_verify_issues "
                f"top={summary_verify_top_issue}"
            )
        if summary_verify_top_issue != "-" and summary_verify_top_issue not in SUMMARY_VERIFY_CODES_SET:
            return fail(f"triage summary_verify_top_issue invalid code: {summary_verify_top_issue}")

        summary_hint = str(triage_doc.get("summary_report_path_hint", "")).strip()
        summary_norm = str(triage_doc.get("summary_report_path_hint_norm", "")).strip()
        expected_summary = str(summary_path)
        expected_summary_norm = normalize_path_text(expected_summary)
        if expected_summary and summary_hint != expected_summary:
            return fail(f"triage summary_report_path_hint mismatch triage={summary_hint} index={expected_summary}")
        if expected_summary_norm and summary_norm != expected_summary_norm:
            return fail(
                f"triage summary_report_path_hint_norm mismatch triage={summary_norm} index={expected_summary_norm}"
            )

        failed_steps = triage_doc.get("failed_steps")
        try:
            failed_steps_count = int(triage_doc.get("failed_steps_count", -1))
        except Exception:
            return fail("triage failed_steps_count must be int")
        if not isinstance(failed_steps, list):
            return fail("triage failed_steps must be list")
        if failed_steps_count != len(failed_steps):
            return fail(f"triage failed_steps_count mismatch triage={failed_steps_count} actual={len(failed_steps)}")
        if result_status == "pass":
            if failed_steps_count != 0:
                return fail(f"pass triage failed_steps_count must be 0, got {failed_steps_count}")
            if failed_steps:
                return fail("pass triage must have empty failed_steps")
        if result_status == "fail" and failed_steps_count <= 0:
            return fail("fail triage failed_steps_count must be >0")
        if brief_path is not None and brief_path.exists():
            brief_line = load_line(brief_path)
            brief_tokens = parse_tokens(brief_line) if brief_line else None
            if isinstance(brief_tokens, dict):
                try:
                    brief_failed_steps_count = int(str(brief_tokens.get("failed_steps_count", "-1")))
                except Exception:
                    return fail("brief failed_steps_count parse failed")
                if brief_failed_steps_count != failed_steps_count:
                    return fail(
                        f"brief/triage failed_steps_count mismatch brief={brief_failed_steps_count} triage={failed_steps_count}"
                    )
        triage_step_ids: list[str] = []
        triage_step_logs: dict[str, dict[str, str]] = {}
        for idx, row in enumerate(failed_steps):
            if not isinstance(row, dict):
                return fail(f"triage failed_steps[{idx}] must be object")
            step_id = str(row.get("step_id", "")).strip()
            if not step_id:
                return fail(f"triage failed_steps[{idx}].step_id missing")
            if step_id in triage_step_logs:
                return fail(f"triage failed_steps duplicate step_id: {step_id}")
            triage_step_ids.append(step_id)
            stdout_path = str(row.get("stdout_log_path", "")).strip()
            stderr_path = str(row.get("stderr_log_path", "")).strip()
            stdout_norm = str(row.get("stdout_log_path_norm", "")).strip()
            stderr_norm = str(row.get("stderr_log_path_norm", "")).strip()
            triage_step_logs[step_id] = {"stdout": stdout_path, "stderr": stderr_path}
            if stdout_path and stdout_norm != normalize_path_text(stdout_path):
                return fail(
                    f"triage failed_steps[{idx}].stdout_log_path_norm mismatch "
                    f"triage={stdout_norm} expected={normalize_path_text(stdout_path)}"
                )
            if stderr_path and stderr_norm != normalize_path_text(stderr_path):
                return fail(
                    f"triage failed_steps[{idx}].stderr_log_path_norm mismatch "
                    f"triage={stderr_norm} expected={normalize_path_text(stderr_path)}"
                )
            for label, path_text in (("stdout", stdout_path), ("stderr", stderr_path)):
                if not path_text:
                    continue
                resolved = resolve_path(path_text)
                if resolved is None:
                    return fail(f"triage failed_steps[{idx}] {label} path resolve failed")
                if not resolved.exists():
                    return fail(f"triage failed_steps[{idx}] {label} path missing: {resolved}")
        if result_status == "fail" and summary_status == "fail":
            summary_step_set = set(str(step).strip() for step in summary_failed_steps if str(step).strip())
            triage_step_set = set(triage_step_ids)
            if not triage_step_set:
                return fail("fail triage missing failed step ids")
            if triage_step_set != summary_step_set:
                return fail(
                    f"triage/summary failed_steps mismatch triage={','.join(sorted(triage_step_set))} "
                    f"summary={','.join(sorted(summary_step_set))}"
                )
            for step_id in triage_step_ids:
                if step_id not in summary_failed_step_details:
                    return fail(f"summary missing failed_step_detail for step={step_id}")
                summary_logs_row = summary_failed_step_logs.get(step_id)
                if not isinstance(summary_logs_row, dict):
                    return fail(f"summary missing failed_step_logs for step={step_id}")
                triage_logs_row = triage_step_logs.get(step_id, {})
                for label in ("stdout", "stderr"):
                    triage_log = str(triage_logs_row.get(label, "")).strip()
                    summary_log = str(summary_logs_row.get(label, "")).strip()
                    if triage_log and not summary_log:
                        return fail(f"summary missing {label} log for step={step_id}")
                    if triage_log and summary_log and triage_log != summary_log:
                        return fail(
                            f"triage/summary {label} log mismatch step={step_id} triage={triage_log} summary={summary_log}"
                        )

        artifacts = triage_doc.get("artifacts")
        if not isinstance(artifacts, dict):
            return fail("triage artifacts missing")
        for key in ("summary", "summary_line", "ci_gate_result_json", "ci_fail_brief_txt", "ci_fail_triage_json"):
            row = artifacts.get(key)
            if not isinstance(row, dict):
                return fail(f"triage artifacts missing key={key}")
        for key, row in artifacts.items():
            if not isinstance(row, dict):
                return fail(f"triage artifacts.{key} must be object")
            issue = validate_triage_artifact_row(str(key), row)
            if issue:
                return fail(issue)
            expected_path = artifact_path_text(index_doc, str(key))
            if expected_path:
                row_path = str(row.get("path", "")).strip()
                if row_path != expected_path:
                    return fail(
                        f"triage artifacts.{key}.path mismatch triage={row_path} index={expected_path}"
                    )
                expected_norm = normalize_path_text(expected_path)
                row_norm = str(row.get("path_norm", "")).strip()
                if row_norm != expected_norm:
                    return fail(
                        f"triage artifacts.{key}.path_norm mismatch triage={row_norm} index={expected_norm}"
                    )
    elif triage_required:
        return fail(f"missing triage json: {triage_path}", code=CODES["TRIAGE_REQUIRED_MISSING"])

    print(
        f"[ci-emit-artifacts-check] ok index={index_path} status={result_status} "
        f"require_brief={int(bool(args.require_brief))} require_triage={int(bool(args.require_triage))}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
