#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


EXPECTED_SCHEMA = "ddn.seamgrim.age3_close_status_line.v1"
EXPECTED_KEYS = [
    "schema",
    "status",
    "overall_ok",
    "criteria_total",
    "criteria_failed_count",
    "report_path",
    "generated_at_utc",
    "status_path",
    "reason",
]

TOKEN_RE = re.compile(r'([A-Za-z0-9_]+)=("([^"\\]|\\.)*"|[^ \t]+)')


def fail(msg: str) -> int:
    print(f"[age3-status-line-check] fail: {msg}")
    return 1


def parse_line(line: str) -> dict[str, str] | None:
    text = line.strip()
    if not text:
        return None
    pos = 0
    out: dict[str, str] = {}
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


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate age3_close_status_line.txt format")
    parser.add_argument("--status-line", required=True, help="path to age3_close_status_line.txt")
    parser.add_argument("--status-json", help="optional path to age3_close_status.detjson for cross-check")
    parser.add_argument("--require-pass", action="store_true", help="require status=pass and overall_ok=1")
    args = parser.parse_args()

    line_path = Path(args.status_line)
    if not line_path.exists():
        return fail(f"missing status line file: {line_path}")
    line_raw = line_path.read_text(encoding="utf-8")
    line = line_raw.strip()
    parsed = parse_line(line)
    if parsed is None:
        return fail("invalid token format")

    keys = list(parsed.keys())
    if keys != EXPECTED_KEYS:
        return fail(f"key order mismatch expected={EXPECTED_KEYS} got={keys}")
    if parsed.get("schema") != EXPECTED_SCHEMA:
        return fail(f"schema mismatch: {parsed.get('schema')}")

    status = parsed.get("status", "")
    if status not in {"pass", "fail"}:
        return fail(f"invalid status: {status}")
    if parsed.get("overall_ok") not in {"0", "1"}:
        return fail(f"invalid overall_ok: {parsed.get('overall_ok')}")
    if not parsed.get("status_path"):
        return fail("status_path is empty")
    if not parsed.get("report_path"):
        return fail("report_path is empty")

    try:
        criteria_total = int(parsed.get("criteria_total", "0"))
        criteria_failed_count = int(parsed.get("criteria_failed_count", "0"))
    except ValueError:
        return fail("criteria_total/criteria_failed_count must be int")
    if criteria_total < 0:
        return fail(f"criteria_total must be >= 0: {criteria_total}")
    if criteria_failed_count < -1:
        return fail(f"criteria_failed_count must be >= -1: {criteria_failed_count}")

    if status == "pass":
        if parsed.get("overall_ok") != "1":
            return fail("status=pass requires overall_ok=1")
        if criteria_failed_count != 0:
            return fail("status=pass requires criteria_failed_count=0")
        if parsed.get("reason") != "-":
            return fail("status=pass requires reason='-'")
    else:
        if parsed.get("overall_ok") != "0":
            return fail("status=fail requires overall_ok=0")

    if args.require_pass and status != "pass":
        return fail("require-pass set but status is fail")

    if args.status_json:
        status_json_path = Path(args.status_json)
        doc = load_json(status_json_path)
        if not isinstance(doc, dict):
            return fail(f"invalid status json: {status_json_path}")
        expected_status = str(doc.get("status", "")).strip()
        expected_overall = "1" if bool(doc.get("overall_ok", False)) else "0"
        expected_total = str(int(doc.get("criteria_total", 0)))
        expected_failed = str(int(doc.get("criteria_failed_count", 0)))
        expected_report = str(doc.get("report_path", "")).strip() or "-"
        if status != expected_status:
            return fail(f"status mismatch line={status} json={expected_status}")
        if parsed.get("overall_ok") != expected_overall:
            return fail(f"overall_ok mismatch line={parsed.get('overall_ok')} json={expected_overall}")
        if parsed.get("criteria_total") != expected_total:
            return fail(f"criteria_total mismatch line={parsed.get('criteria_total')} json={expected_total}")
        if parsed.get("criteria_failed_count") != expected_failed:
            return fail(
                f"criteria_failed_count mismatch line={parsed.get('criteria_failed_count')} json={expected_failed}"
            )
        if parsed.get("report_path") != expected_report:
            return fail(f"report_path mismatch line={parsed.get('report_path')} json={expected_report}")

    print(
        f"[age3-status-line-check] ok status={status} overall_ok={parsed.get('overall_ok')} "
        f"criteria_total={criteria_total} criteria_failed_count={criteria_failed_count}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
