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


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate ci_gate_summary_line.txt against parse compact_line")
    parser.add_argument("--summary-line", required=True, help="path to ci_gate_summary_line.txt")
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--final-status-parse", help="path to ci_gate_final_status_line_parse.detjson")
    source.add_argument("--ci-gate-result-parse", help="path to ci_gate_result_parse.detjson")
    parser.add_argument("--require-pass", action="store_true", help="also require ci_gate_status=pass")
    args = parser.parse_args()

    summary_line_path = Path(args.summary_line)
    parse_path = Path(args.final_status_parse) if args.final_status_parse else Path(args.ci_gate_result_parse)
    if not summary_line_path.exists():
        print(f"missing summary-line: {summary_line_path}", file=sys.stderr)
        return 1
    summary_line = summary_line_path.read_text(encoding="utf-8").strip()
    if not summary_line:
        print("summary-line is empty", file=sys.stderr)
        return 1

    parse_doc = load_json(parse_path)
    if parse_doc is None:
        print(f"invalid final status parse json: {parse_path}", file=sys.stderr)
        return 1
    compact = str(parse_doc.get("compact_line", "")).strip()
    if not compact:
        print("final status parse compact_line is empty", file=sys.stderr)
        return 1
    if summary_line != compact:
        print("summary-line mismatch with compact_line", file=sys.stderr)
        return 1
    if args.require_pass:
        parsed = parse_doc.get("parsed")
        status = str(parsed.get("status", "")).strip() if isinstance(parsed, dict) else ""
        if status != "pass":
            print("parse status is not pass", file=sys.stderr)
            return 1

    print(f"[ci-gate-summary-line-check] ok line={summary_line}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
