#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path


SCHEMA = "ddn.ci.gate_final_status_line.v1"


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def q(value: object) -> str:
    return json.dumps(str(value), ensure_ascii=False)


def count_failed_steps(index_doc: dict | None) -> int:
    if not isinstance(index_doc, dict):
        return -1
    steps = index_doc.get("steps")
    if not isinstance(steps, list):
        return -1
    return sum(1 for row in steps if isinstance(row, dict) and not bool(row.get("ok", False)))


def build_line(
    parse_path: Path,
    parse_doc: dict | None,
    index_path: Path,
    index_doc: dict | None,
) -> tuple[str, bool]:
    if not isinstance(parse_doc, dict):
        parts = [
            f"schema={q(SCHEMA)}",
            "status=fail",
            "overall_ok=0",
            "failed_steps=-1",
            "aggregate_status=fail",
            f"report_index={q(index_path)}",
            f"aggregate_status_line={q('-')}",
            f"aggregate_status_parse={q(parse_path)}",
            f"generated_at_utc={q('-')}",
            f"reason={q('invalid_or_missing_aggregate_status_parse')}",
        ]
        return " ".join(parts) + "\n", False

    parsed = parse_doc.get("parsed")
    if not isinstance(parsed, dict):
        parts = [
            f"schema={q(SCHEMA)}",
            "status=fail",
            "overall_ok=0",
            "failed_steps=-1",
            "aggregate_status=fail",
            f"report_index={q(index_path)}",
            f"aggregate_status_line={q(str(parse_doc.get('status_line_path', '-')))}",
            f"aggregate_status_parse={q(parse_path)}",
            f"generated_at_utc={q('-')}",
            f"reason={q('invalid_parse_payload')}",
        ]
        return " ".join(parts) + "\n", False

    aggregate_status = str(parsed.get("status", "fail")).strip() or "fail"
    overall_ok = str(parsed.get("overall_ok", "0")).strip() == "1"
    failed_steps = count_failed_steps(index_doc)
    reason = str(parsed.get("reason", "-")).strip() or "-"
    if reason == "-" and failed_steps > 0:
        reason = f"failed_steps={failed_steps}"
    final_status = "pass" if overall_ok and aggregate_status == "pass" and failed_steps == 0 else "fail"
    parts = [
        f"schema={q(SCHEMA)}",
        f"status={final_status}",
        f"overall_ok={int(overall_ok)}",
        f"failed_steps={failed_steps}",
        f"aggregate_status={aggregate_status}",
        f"report_index={q(index_path)}",
        f"aggregate_status_line={q(str(parse_doc.get('status_line_path', '-')))}",
        f"aggregate_status_parse={q(parse_path)}",
        f"generated_at_utc={q(str(parsed.get('generated_at_utc', '-')))}",
        f"reason={q(reason)}",
    ]
    final_ok = final_status == "pass"
    return " ".join(parts) + "\n", final_ok


def main() -> int:
    parser = argparse.ArgumentParser(description="Render one-line final CI gate status")
    parser.add_argument("--aggregate-status-parse", required=True, help="path to aggregate status-line parse detjson")
    parser.add_argument(
        "--gate-index",
        default="build/reports/ci_gate_report_index.detjson",
        help="path to aggregate gate index report",
    )
    parser.add_argument("--out", required=True, help="output final status-line txt path")
    parser.add_argument("--fail-on-bad", action="store_true", help="return non-zero when status is fail")
    args = parser.parse_args()

    parse_path = Path(args.aggregate_status_parse)
    index_path = Path(args.gate_index)
    out_path = Path(args.out)
    parse_doc = load_json(parse_path)
    index_doc = load_json(index_path)
    line, ok = build_line(parse_path, parse_doc, index_path, index_doc)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(line, encoding="utf-8")
    print(f"[ci-gate-final-status-line] out={out_path} ok={int(ok)}")
    if args.fail_on_bad and not ok:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
