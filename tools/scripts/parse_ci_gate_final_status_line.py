#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


EXPECTED_SCHEMA = "ddn.ci.gate_final_status_line.v1"
EXPECTED_KEYS = [
    "schema",
    "status",
    "overall_ok",
    "failed_steps",
    "aggregate_status",
    "report_index",
    "aggregate_status_line",
    "aggregate_status_parse",
    "generated_at_utc",
    "reason",
]
TOKEN_RE = re.compile(r'([A-Za-z0-9_]+)=("([^"\\]|\\.)*"|[^ \t]+)')


def parse_tokens(text: str) -> dict[str, str] | None:
    line = text.strip()
    if not line:
        return None
    out: dict[str, str] = {}
    pos = 0
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
    parsed = parse_tokens(path.read_text(encoding="utf-8"))
    if parsed is None:
        return None, "invalid token format"
    if list(parsed.keys()) != EXPECTED_KEYS:
        return None, "key order mismatch"
    if parsed.get("schema") != EXPECTED_SCHEMA:
        return None, f"schema mismatch: {parsed.get('schema')}"
    if parsed.get("status") not in {"pass", "fail"}:
        return None, f"invalid status: {parsed.get('status')}"
    if parsed.get("overall_ok") not in {"0", "1"}:
        return None, f"invalid overall_ok: {parsed.get('overall_ok')}"
    if parsed.get("aggregate_status") not in {"pass", "fail"}:
        return None, f"invalid aggregate_status: {parsed.get('aggregate_status')}"
    try:
        int(parsed.get("failed_steps", "-1"))
    except ValueError:
        return None, "failed_steps must be int"
    return parsed, ""


def compact_line(parsed: dict[str, str]) -> str:
    return (
        f"ci_gate_status={parsed.get('status', 'fail')} "
        f"overall_ok={parsed.get('overall_ok', '0')} "
        f"failed_steps={parsed.get('failed_steps', '-1')} "
        f"aggregate_status={parsed.get('aggregate_status', 'fail')} "
        f"reason={parsed.get('reason', '-')}"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Parse ci_gate_final_status_line.txt and print compact status")
    parser.add_argument("--status-line", required=True, help="path to ci_gate_final_status_line.txt")
    parser.add_argument("--json-out", help="optional parse result detjson path")
    parser.add_argument("--compact-out", help="optional compact one-line txt path")
    parser.add_argument("--fail-on-invalid", action="store_true", help="return non-zero when parse/validation fails")
    parser.add_argument("--fail-on-fail", action="store_true", help="return non-zero when parsed status=fail")
    args = parser.parse_args()

    status_line_path = Path(args.status_line)
    parsed, error = parse_status_line(status_line_path)
    if parsed is None:
        print(f"[ci-gate-final-status-line-parse] invalid reason={error}")
        if args.fail_on_invalid:
            return 1
        return 0

    compact = compact_line(parsed)
    print(f"[ci-gate-final-status-line-parse] {compact}")

    if args.json_out:
        out = Path(args.json_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "schema": "ddn.ci.gate_final_status_line_parse.v1",
            "status_line_path": str(status_line_path),
            "parsed": parsed,
            "compact_line": compact,
        }
        out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    if args.compact_out:
        out = Path(args.compact_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(compact + "\n", encoding="utf-8")

    if args.fail_on_fail and parsed.get("status") != "pass":
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
