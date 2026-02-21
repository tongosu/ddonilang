#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

from ci_check_error_codes import FAILURE_SUMMARY_CODES as CODES

DETAIL_RE = re.compile(r"^failed_step_detail=([^ ]+) rc=([-]?\d+) cmd=(.+)$")
LOGS_RE = re.compile(r"^failed_step_logs=([^ ]+) stdout=([^ ]+) stderr=([^ ]+)$")


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def fail(msg: str, code: str = "E_CHECK") -> int:
    print(f"[ci-gate-failure-summary-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def parse_summary(path: Path) -> tuple[str | None, dict[str, str], list[tuple[str, str]]]:
    status: str | None = None
    kv: dict[str, str] = {}
    parsed_rows: list[tuple[str, str]] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
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
        parsed_rows.append((key, value))
    return status, kv, parsed_rows


def parse_detail_rows(rows: list[tuple[str, str]]) -> list[tuple[str, int, str]]:
    out: list[tuple[str, int, str]] = []
    for key, value in rows:
        if key != "failed_step_detail":
            continue
        match = DETAIL_RE.match(f"{key}={value}")
        if not match:
            continue
        name = str(match.group(1)).strip()
        rc = int(match.group(2))
        cmd = str(match.group(3)).strip()
        out.append((name, rc, cmd))
    return out


def parse_log_rows(rows: list[tuple[str, str]]) -> list[tuple[str, str, str]]:
    out: list[tuple[str, str, str]] = []
    for key, value in rows:
        if key != "failed_step_logs":
            continue
        match = LOGS_RE.match(f"{key}={value}")
        if not match:
            continue
        name = str(match.group(1)).strip()
        stdout_path = str(match.group(2)).strip()
        stderr_path = str(match.group(3)).strip()
        out.append((name, stdout_path, stderr_path))
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate ci_gate_summary failure detail/log block")
    parser.add_argument("--summary", required=True, help="path to ci_gate_summary.txt")
    parser.add_argument("--index", required=True, help="path to ci_gate_report_index.detjson")
    parser.add_argument("--require-pass", action="store_true", help="require summary PASS")
    args = parser.parse_args()

    summary_path = Path(args.summary)
    index_path = Path(args.index)
    if not summary_path.exists():
        return fail(f"missing summary file: {summary_path}", code=CODES["SUMMARY_MISSING"])
    index_doc = load_json(index_path)
    if index_doc is None:
        return fail(f"invalid index json: {index_path}", code=CODES["INDEX_INVALID"])

    status, kv, rows = parse_summary(summary_path)
    if status not in {"pass", "fail"}:
        return fail("missing PASS/FAIL summary header", code=CODES["STATUS_HEADER_MISSING"])
    if args.require_pass and status != "pass":
        return fail("require-pass set but summary status is not PASS", code=CODES["REQUIRE_PASS"])

    failed_steps_value = str(kv.get("failed_steps", "")).strip()
    detail_rows = parse_detail_rows(rows)
    log_rows = parse_log_rows(rows)
    detail_row_count = sum(1 for key, _ in rows if key == "failed_step_detail")
    log_row_count = sum(1 for key, _ in rows if key == "failed_step_logs")
    if detail_row_count != len(detail_rows):
        return fail("failed_step_detail format invalid", code=CODES["DETAIL_FORMAT_INVALID"])
    if log_row_count != len(log_rows):
        return fail("failed_step_logs format invalid", code=CODES["LOGS_FORMAT_INVALID"])

    if status == "pass":
        if failed_steps_value != "(none)":
            return fail(
                f"PASS summary requires failed_steps=(none), got={failed_steps_value}",
                code=CODES["PASS_FAILED_STEPS_NOT_NONE"],
            )
        if detail_rows:
            return fail("PASS summary must not contain failed_step_detail", code=CODES["PASS_HAS_DETAIL"])
        if log_rows:
            return fail("PASS summary must not contain failed_step_logs", code=CODES["PASS_HAS_LOGS"])
        print(f"[ci-gate-failure-summary-check] ok status={status} summary={summary_path}")
        return 0

    if not failed_steps_value or failed_steps_value == "(none)":
        return fail("FAIL summary requires non-empty failed_steps", code=CODES["FAIL_FAILED_STEPS_EMPTY"])
    failed_steps = [item.strip() for item in failed_steps_value.split(",") if item.strip()]
    if not failed_steps:
        return fail("failed_steps list is empty", code=CODES["FAIL_FAILED_STEPS_PARSE_EMPTY"])
    if len(set(failed_steps)) != len(failed_steps):
        return fail("failed_steps contains duplicates", code=CODES["FAIL_FAILED_STEPS_DUPLICATE"])
    if not detail_rows:
        return fail("FAIL summary missing failed_step_detail rows", code=CODES["FAIL_DETAIL_MISSING"])

    detail_names = {name for name, _, _ in detail_rows}
    if len(detail_names) != len(detail_rows):
        return fail("failed_step_detail contains duplicate step rows", code=CODES["FAIL_DETAIL_DUPLICATE"])
    for name in detail_names:
        if name not in failed_steps:
            return fail(
                f"failed_step_detail step not in failed_steps: {name}",
                code=CODES["FAIL_DETAIL_NOT_IN_FAILED_STEPS"],
            )
    for name in failed_steps:
        if name not in detail_names:
            return fail(
                f"failed_steps missing failed_step_detail row: {name}",
                code=CODES["FAIL_DETAIL_MISSING_FOR_STEP"],
            )
    for name, rc, cmd in detail_rows:
        if rc == 0:
            return fail(f"failed_step_detail rc must be non-zero: {name}", code=CODES["FAIL_DETAIL_RC_ZERO"])
        if not cmd:
            return fail(f"failed_step_detail cmd is empty: {name}", code=CODES["FAIL_DETAIL_CMD_EMPTY"])

    steps = index_doc.get("steps")
    step_rows = steps if isinstance(steps, list) else []
    for name, rc, _ in detail_rows:
        matched = False
        for row in step_rows:
            if not isinstance(row, dict):
                continue
            row_name = str(row.get("name", "")).strip()
            row_rc = int(row.get("returncode", 0))
            if row_name == name and row_rc == rc:
                matched = True
                break
        if not matched:
            return fail(f"detail row has no matching index step: {name} rc={rc}", code=CODES["DETAIL_INDEX_MISMATCH"])

    step_log_dir = str(index_doc.get("step_log_dir", "")).strip()
    step_log_required = bool(step_log_dir)
    if step_log_required and not log_rows:
        return fail("index.step_log_dir set but FAIL summary has no failed_step_logs rows", code=CODES["FAIL_LOGS_MISSING"])
    log_names = {name for name, _, _ in log_rows}
    if len(log_names) != len(log_rows):
        return fail("failed_step_logs contains duplicate step rows", code=CODES["FAIL_LOGS_DUPLICATE"])
    for name, stdout_path, stderr_path in log_rows:
        if name not in failed_steps:
            return fail(
                f"failed_step_logs step not in failed_steps: {name}",
                code=CODES["FAIL_LOGS_NOT_IN_FAILED_STEPS"],
            )
        for label, raw_path in (("stdout", stdout_path), ("stderr", stderr_path)):
            if raw_path == "-":
                continue
            path = Path(raw_path)
            if step_log_required and not path.exists():
                return fail(f"failed_step_logs {label} path missing: {path}", code=CODES["FAIL_LOG_PATH_MISSING"])
    if step_log_required:
        for name in failed_steps:
            if name not in log_names:
                return fail(
                    f"failed_steps missing failed_step_logs row: {name}",
                    code=CODES["FAIL_LOGS_MISSING_FOR_STEP"],
                )

    print(
        f"[ci-gate-failure-summary-check] ok status={status} "
        f"failed_steps={len(failed_steps)} detail_rows={len(detail_rows)} log_rows={len(log_rows)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
