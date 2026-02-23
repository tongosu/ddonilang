#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import re
import sys
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


def parse_tokens(line: str) -> dict[str, str] | None:
    text = line.strip()
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


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate ci_aggregate_status_line.txt format")
    parser.add_argument("--status-line", required=True, help="path to ci_aggregate_status_line.txt")
    parser.add_argument("--aggregate-report", required=True, help="path to ci_aggregate_report.detjson")
    parser.add_argument("--require-pass", action="store_true", help="also require overall_ok=true")
    args = parser.parse_args()

    status_line_path = Path(args.status_line)
    aggregate_report_path = Path(args.aggregate_report)
    if not status_line_path.exists():
        print(f"missing status-line: {status_line_path}", file=sys.stderr)
        return 1
    parsed = parse_tokens(status_line_path.read_text(encoding="utf-8"))
    if parsed is None:
        print(f"invalid status-line format: {status_line_path}", file=sys.stderr)
        return 1
    if list(parsed.keys()) != EXPECTED_KEYS:
        print("status-line key order mismatch", file=sys.stderr)
        return 1
    if parsed.get("schema") != EXPECTED_SCHEMA:
        print("status-line schema mismatch", file=sys.stderr)
        return 1
    if parsed.get("status") not in {"pass", "fail"}:
        print("status-line status invalid", file=sys.stderr)
        return 1
    if parsed.get("overall_ok") not in {"0", "1"}:
        print("status-line overall_ok invalid", file=sys.stderr)
        return 1

    for key in ("seamgrim_failed_steps", "age3_failed_criteria", "age4_failed_criteria", "age5_failed_criteria", "oi_failed_packs"):
        try:
            int(parsed.get(key, "0"))
        except ValueError:
            print(f"status-line {key} invalid int", file=sys.stderr)
            return 1

    report = load_json(aggregate_report_path)
    if report is None:
        print(f"invalid aggregate report: {aggregate_report_path}", file=sys.stderr)
        return 1

    overall_ok = bool(report.get("overall_ok", False))
    expected_status = "pass" if overall_ok else "fail"
    if parsed.get("status") != expected_status:
        print("status mismatch with aggregate report", file=sys.stderr)
        return 1
    if parsed.get("overall_ok") != ("1" if overall_ok else "0"):
        print("overall_ok mismatch with aggregate report", file=sys.stderr)
        return 1

    seamgrim = report.get("seamgrim") if isinstance(report.get("seamgrim"), dict) else {}
    age3 = report.get("age3") if isinstance(report.get("age3"), dict) else {}
    age4 = report.get("age4") if isinstance(report.get("age4"), dict) else {}
    age5 = report.get("age5") if isinstance(report.get("age5"), dict) else {}
    oi = report.get("oi405_406") if isinstance(report.get("oi405_406"), dict) else {}
    expected_counts = {
        "seamgrim_failed_steps": len(seamgrim.get("failed_steps", [])) if isinstance(seamgrim.get("failed_steps"), list) else 0,
        "age3_failed_criteria": len(age3.get("failed_criteria", [])) if isinstance(age3.get("failed_criteria"), list) else 0,
        "age4_failed_criteria": len(age4.get("failed_criteria", [])) if isinstance(age4.get("failed_criteria"), list) else 0,
        "age5_failed_criteria": len(age5.get("failed_criteria", [])) if isinstance(age5.get("failed_criteria"), list) else 0,
        "oi_failed_packs": len(oi.get("failed_packs", [])) if isinstance(oi.get("failed_packs"), list) else 0,
    }
    for key, expected in expected_counts.items():
        if int(parsed.get(key, "-1")) != expected:
            print(f"{key} mismatch: line={parsed.get(key)} report={expected}", file=sys.stderr)
            return 1

    if str(parsed.get("report_path", "")).strip() != str(aggregate_report_path):
        print("report_path mismatch", file=sys.stderr)
        return 1
    if args.require_pass and not overall_ok:
        print("aggregate overall_ok=false", file=sys.stderr)
        return 1

    print(
        "[ci-aggregate-status-line-check] ok "
        f"status={parsed.get('status')} overall_ok={parsed.get('overall_ok')} "
        f"seamgrim_failed={parsed.get('seamgrim_failed_steps')} "
        f"age3_failed={parsed.get('age3_failed_criteria')} "
        f"age4_failed={parsed.get('age4_failed_criteria')} "
        f"age5_failed={parsed.get('age5_failed_criteria')} "
        f"oi_failed={parsed.get('oi_failed_packs')}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
