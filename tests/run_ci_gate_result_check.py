#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


EXPECTED_SCHEMA = "ddn.ci.gate_result.v1"


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate ci_gate_result.detjson")
    parser.add_argument("--result", required=True, help="path to ci_gate_result.detjson")
    parser.add_argument("--final-status-parse", required=True, help="path to ci_gate_final_status_line_parse.detjson")
    parser.add_argument("--summary-line", required=True, help="path to ci_gate_summary_line.txt")
    parser.add_argument("--require-pass", action="store_true", help="also require status=pass and ok=true")
    args = parser.parse_args()

    result_path = Path(args.result)
    final_parse_path = Path(args.final_status_parse)
    summary_line_path = Path(args.summary_line)
    result_doc = load_json(result_path)
    if result_doc is None:
        print(f"invalid result json: {result_path}", file=sys.stderr)
        return 1
    if result_doc.get("schema") != EXPECTED_SCHEMA:
        print(f"schema mismatch: {result_doc.get('schema')}", file=sys.stderr)
        return 1

    final_parse_doc = load_json(final_parse_path)
    if final_parse_doc is None:
        print(f"invalid final parse json: {final_parse_path}", file=sys.stderr)
        return 1
    parsed = final_parse_doc.get("parsed")
    if not isinstance(parsed, dict):
        print("final parse json missing parsed object", file=sys.stderr)
        return 1

    summary_line = summary_line_path.read_text(encoding="utf-8").strip() if summary_line_path.exists() else ""
    if not summary_line:
        print(f"summary line missing/empty: {summary_line_path}", file=sys.stderr)
        return 1
    if str(result_doc.get("summary_line", "")).strip() != summary_line:
        print("summary_line mismatch", file=sys.stderr)
        return 1
    if str(result_doc.get("summary_line_path", "")).strip() != str(summary_line_path):
        print("summary_line_path mismatch", file=sys.stderr)
        return 1
    if str(result_doc.get("final_status_parse_path", "")).strip() != str(final_parse_path):
        print("final_status_parse_path mismatch", file=sys.stderr)
        return 1

    expected_status = str(parsed.get("status", "fail")).strip() or "fail"
    expected_overall_ok = str(parsed.get("overall_ok", "0")).strip() == "1"
    expected_aggregate_status = str(parsed.get("aggregate_status", "fail")).strip() or "fail"
    try:
        expected_failed_steps = int(parsed.get("failed_steps", "-1"))
    except ValueError:
        expected_failed_steps = -1

    if str(result_doc.get("status", "")).strip() != expected_status:
        print("status mismatch", file=sys.stderr)
        return 1
    if bool(result_doc.get("overall_ok", False)) != expected_overall_ok:
        print("overall_ok mismatch", file=sys.stderr)
        return 1
    if str(result_doc.get("aggregate_status", "")).strip() != expected_aggregate_status:
        print("aggregate_status mismatch", file=sys.stderr)
        return 1
    if int(result_doc.get("failed_steps", -1)) != expected_failed_steps:
        print("failed_steps mismatch", file=sys.stderr)
        return 1

    expected_ok = (
        expected_status == "pass"
        and expected_overall_ok
        and expected_aggregate_status == "pass"
        and expected_failed_steps == 0
    )
    if bool(result_doc.get("ok", False)) != expected_ok:
        print("ok mismatch", file=sys.stderr)
        return 1
    if args.require_pass and not expected_ok:
        print("result is not pass", file=sys.stderr)
        return 1

    print(
        "[ci-gate-result-check] ok "
        f"status={result_doc.get('status')} ok={int(bool(result_doc.get('ok', False)))} "
        f"failed_steps={result_doc.get('failed_steps')}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
