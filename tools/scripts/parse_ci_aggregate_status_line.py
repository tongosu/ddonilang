#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


EXPECTED_SCHEMA = "ddn.ci.aggregate_gate_status_line.v1"
EXPECTED_KEYS = [
    "schema",
    "status",
    "overall_ok",
    "seamgrim_failed_steps",
    "age3_failed_criteria",
    "age4_failed_criteria",
    "age5_failed_criteria",
    "oi_failed_packs",
    "report_path",
    "generated_at_utc",
    "reason",
]
TOKEN_RE = re.compile(r'([A-Za-z0-9_]+)=("([^"\\]|\\.)*"|[^ \t]+)')


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def parse_tokens(text: str) -> dict[str, str] | None:
    line = text.strip()
    if not line:
        return None
    pos = 0
    out: dict[str, str] = {}
    for match in TOKEN_RE.finditer(line):
        if line[pos : match.start()].strip():
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
    if line[pos:].strip():
        return None
    return out


def parse_status_line(path: Path) -> tuple[dict[str, str] | None, str]:
    if not path.exists():
        return None, f"missing status line: {path}"
    text = path.read_text(encoding="utf-8").strip()
    parsed = parse_tokens(text)
    if parsed is None:
        return None, "invalid token format"
    keys = list(parsed.keys())
    if keys != EXPECTED_KEYS:
        return None, f"key order mismatch expected={EXPECTED_KEYS} got={keys}"
    if parsed.get("schema") != EXPECTED_SCHEMA:
        return None, f"schema mismatch: {parsed.get('schema')}"
    if parsed.get("status") not in {"pass", "fail"}:
        return None, f"invalid status: {parsed.get('status')}"
    if parsed.get("overall_ok") not in {"0", "1"}:
        return None, f"invalid overall_ok: {parsed.get('overall_ok')}"
    for key in ("seamgrim_failed_steps", "age3_failed_criteria", "age4_failed_criteria", "age5_failed_criteria", "oi_failed_packs"):
        try:
            int(parsed.get(key, "0"))
        except ValueError:
            return None, f"{key} must be int"
    return parsed, ""


def compact_line(parsed: dict[str, str]) -> str:
    return (
        f"aggregate_gate_status={parsed.get('status', 'fail')} "
        f"overall_ok={parsed.get('overall_ok', '0')} "
        f"seamgrim_failed={parsed.get('seamgrim_failed_steps', '-1')} "
        f"age3_failed={parsed.get('age3_failed_criteria', '-1')} "
        f"age4_failed={parsed.get('age4_failed_criteria', '-1')} "
        f"age5_failed={parsed.get('age5_failed_criteria', '-1')} "
        f"oi_failed={parsed.get('oi_failed_packs', '-1')} "
        f"reason={parsed.get('reason', '-')}"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Parse ci_aggregate_status_line.txt and print compact one-line status")
    parser.add_argument("--status-line", required=True, help="path to ci_aggregate_status_line.txt")
    parser.add_argument("--aggregate-report", help="optional path to ci_aggregate_report.detjson for cross-check")
    parser.add_argument("--json-out", help="optional parsed json output")
    parser.add_argument("--fail-on-invalid", action="store_true", help="return non-zero when parse/validation fails")
    parser.add_argument("--fail-on-fail", action="store_true", help="return non-zero when parsed status=fail")
    args = parser.parse_args()

    status_line_path = Path(args.status_line)
    parsed, error = parse_status_line(status_line_path)
    if parsed is None:
        print(f"[ci-aggregate-status-line-parse] invalid reason={error}")
        if args.fail_on_invalid:
            return 1
        return 0

    if args.aggregate_report:
        report_doc = load_json(Path(args.aggregate_report))
        if not isinstance(report_doc, dict):
            print(
                "[ci-aggregate-status-line-parse] invalid "
                f"reason=invalid_aggregate_report path={args.aggregate_report}"
            )
            if args.fail_on_invalid:
                return 1
            return 0
        expected_status = "pass" if bool(report_doc.get("overall_ok", False)) else "fail"
        expected_ok = "1" if bool(report_doc.get("overall_ok", False)) else "0"
        if parsed.get("status") != expected_status or parsed.get("overall_ok") != expected_ok:
            print(
                "[ci-aggregate-status-line-parse] invalid "
                f"reason=status_mismatch line_status={parsed.get('status')} report_status={expected_status} "
                f"line_ok={parsed.get('overall_ok')} report_ok={expected_ok}"
            )
            if args.fail_on_invalid:
                return 1
            return 0

    compact = compact_line(parsed)
    print(f"[ci-aggregate-status-line-parse] {compact}")

    if args.json_out:
        out = Path(args.json_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "schema": "ddn.ci.aggregate_gate_status_line_parse.v1",
            "status_line_path": str(status_line_path),
            "parsed": parsed,
            "compact_line": compact,
        }
        out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    if args.fail_on_fail and parsed.get("status") != "pass":
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
