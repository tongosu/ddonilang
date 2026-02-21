#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def load_text(path: Path) -> str:
    if not path.exists():
        return ""
    try:
        return path.read_text(encoding="utf-8").strip()
    except Exception:
        return ""


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate consistency across final CI gate artifacts")
    parser.add_argument("--summary-line", required=True, help="path to ci_gate_summary_line.txt")
    parser.add_argument("--result", required=True, help="path to ci_gate_result.detjson")
    parser.add_argument("--result-parse", required=True, help="path to ci_gate_result_parse.detjson")
    parser.add_argument("--badge", required=True, help="path to ci_gate_badge.detjson")
    parser.add_argument("--final-status-parse", required=True, help="path to ci_gate_final_status_line_parse.detjson")
    parser.add_argument("--require-pass", action="store_true", help="also require all statuses to be pass")
    args = parser.parse_args()

    summary_line_path = Path(args.summary_line)
    result_path = Path(args.result)
    result_parse_path = Path(args.result_parse)
    badge_path = Path(args.badge)
    final_status_parse_path = Path(args.final_status_parse)

    summary_line = load_text(summary_line_path)
    result_doc = load_json(result_path)
    result_parse_doc = load_json(result_parse_path)
    badge_doc = load_json(badge_path)
    final_status_parse_doc = load_json(final_status_parse_path)

    errors: list[str] = []
    if not summary_line:
        errors.append(f"missing summary_line: {summary_line_path}")
    if result_doc is None:
        errors.append(f"invalid result json: {result_path}")
    if result_parse_doc is None:
        errors.append(f"invalid result parse json: {result_parse_path}")
    if badge_doc is None:
        errors.append(f"invalid badge json: {badge_path}")
    if final_status_parse_doc is None:
        errors.append(f"invalid final status parse json: {final_status_parse_path}")
    if errors:
        for line in errors:
            print(line, file=sys.stderr)
        return 1

    compact = str(result_parse_doc.get("compact_line", "")).strip()
    if summary_line != compact:
        errors.append("summary_line != result_parse.compact_line")

    parsed_result = result_parse_doc.get("parsed")
    if not isinstance(parsed_result, dict):
        errors.append("result_parse.parsed missing")
        parsed_result = {}

    result_status = str(result_doc.get("status", "fail")).strip() or "fail"
    parse_status = str(parsed_result.get("status", "fail")).strip() or "fail"
    if result_status != parse_status:
        errors.append(f"result.status mismatch: result={result_status} parse={parse_status}")

    result_ok = bool(result_doc.get("ok", False))
    parse_ok = bool(parsed_result.get("ok", False))
    if result_ok != parse_ok:
        errors.append(f"result.ok mismatch: result={int(result_ok)} parse={int(parse_ok)}")

    badge_status = str(badge_doc.get("status", "fail")).strip() or "fail"
    badge_ok = bool(badge_doc.get("ok", False))
    if badge_status != result_status:
        errors.append(f"badge.status mismatch: badge={badge_status} result={result_status}")
    if badge_ok != result_ok:
        errors.append(f"badge.ok mismatch: badge={int(badge_ok)} result={int(result_ok)}")

    final_parsed = final_status_parse_doc.get("parsed")
    final_status = ""
    if isinstance(final_parsed, dict):
        final_status = str(final_parsed.get("status", "fail")).strip() or "fail"
        if final_status != result_status:
            errors.append(f"final.status mismatch: final={final_status} result={result_status}")
    else:
        errors.append("final_status_parse.parsed missing")

    if args.require_pass:
        if result_status != "pass" or not result_ok or badge_status != "pass" or not badge_ok or final_status != "pass":
            errors.append("require-pass violated")

    if errors:
        print("ci gate outputs consistency check failed", file=sys.stderr)
        for line in errors[:16]:
            print(f" - {line}", file=sys.stderr)
        return 1

    print(
        "[ci-gate-outputs-consistency-check] ok "
        f"status={result_status} ok={int(result_ok)} summary_line={summary_line}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
