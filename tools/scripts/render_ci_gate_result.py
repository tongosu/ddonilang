#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path


SCHEMA = "ddn.ci.gate_result.v1"


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def load_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8").strip()
    except Exception:
        return ""


def build_result(
    final_parse_path: Path,
    summary_line_path: Path,
    gate_index_path: Path | None,
    final_parse_doc: dict | None,
) -> tuple[dict, bool]:
    if not isinstance(final_parse_doc, dict):
        payload = {
            "schema": SCHEMA,
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "ok": False,
            "status": "fail",
            "overall_ok": False,
            "failed_steps": -1,
            "aggregate_status": "fail",
            "reason": "invalid_or_missing_final_parse",
            "summary_line_path": str(summary_line_path),
            "summary_line": load_text(summary_line_path),
            "final_status_parse_path": str(final_parse_path),
            "gate_index_path": str(gate_index_path) if gate_index_path is not None else "",
        }
        return payload, False

    parsed = final_parse_doc.get("parsed")
    if not isinstance(parsed, dict):
        payload = {
            "schema": SCHEMA,
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "ok": False,
            "status": "fail",
            "overall_ok": False,
            "failed_steps": -1,
            "aggregate_status": "fail",
            "reason": "invalid_final_parse_payload",
            "summary_line_path": str(summary_line_path),
            "summary_line": load_text(summary_line_path),
            "final_status_parse_path": str(final_parse_path),
            "gate_index_path": str(gate_index_path) if gate_index_path is not None else "",
        }
        return payload, False

    status = str(parsed.get("status", "fail")).strip() or "fail"
    overall_ok = str(parsed.get("overall_ok", "0")).strip() == "1"
    try:
        failed_steps = int(parsed.get("failed_steps", "-1"))
    except ValueError:
        failed_steps = -1
    aggregate_status = str(parsed.get("aggregate_status", "fail")).strip() or "fail"
    reason = str(parsed.get("reason", "-")).strip() or "-"
    summary_line = load_text(summary_line_path)
    ok = status == "pass" and overall_ok and aggregate_status == "pass" and failed_steps == 0
    payload = {
        "schema": SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": ok,
        "status": status,
        "overall_ok": overall_ok,
        "failed_steps": failed_steps,
        "aggregate_status": aggregate_status,
        "reason": reason,
        "summary_line_path": str(summary_line_path),
        "summary_line": summary_line,
        "final_status_parse_path": str(final_parse_path),
        "gate_index_path": str(gate_index_path) if gate_index_path is not None else "",
    }
    return payload, ok


def main() -> int:
    parser = argparse.ArgumentParser(description="Render compact CI gate result JSON")
    parser.add_argument("--final-status-parse", required=True, help="path to ci_gate_final_status_line_parse.detjson")
    parser.add_argument("--summary-line", required=True, help="path to ci_gate_summary_line.txt")
    parser.add_argument("--gate-index", default="", help="optional path to ci_gate_report_index.detjson")
    parser.add_argument("--out", required=True, help="output ci_gate_result.detjson path")
    parser.add_argument("--fail-on-bad", action="store_true", help="return non-zero when result status is fail")
    args = parser.parse_args()

    final_parse_path = Path(args.final_status_parse)
    summary_line_path = Path(args.summary_line)
    gate_index_path = Path(args.gate_index) if args.gate_index.strip() else None
    out_path = Path(args.out)
    final_parse_doc = load_json(final_parse_path)
    payload, ok = build_result(final_parse_path, summary_line_path, gate_index_path, final_parse_doc)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"[ci-gate-result] out={out_path} ok={int(ok)} status={payload.get('status')}")
    if args.fail_on_bad and not ok:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
