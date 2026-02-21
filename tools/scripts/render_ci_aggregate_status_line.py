#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path


SCHEMA = "ddn.ci.aggregate_gate_status_line.v1"


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


def failed_count(doc: dict | None, key: str) -> int:
    if not isinstance(doc, dict):
        return -1
    rows = doc.get(key)
    if not isinstance(rows, list):
        return 0
    return len(rows)


def build_line(report_path: Path, payload: dict | None) -> tuple[str, bool]:
    if not isinstance(payload, dict):
        parts = [
            f"schema={q(SCHEMA)}",
            "status=fail",
            "overall_ok=0",
            "seamgrim_failed_steps=-1",
            "age3_failed_criteria=-1",
            "age4_failed_criteria=-1",
            "oi_failed_packs=-1",
            f"report_path={q(report_path)}",
            f"generated_at_utc={q('-')}",
            f"reason={q('invalid_or_missing_report')}",
        ]
        return " ".join(parts) + "\n", False

    overall_ok = bool(payload.get("overall_ok", False))
    seamgrim = payload.get("seamgrim") if isinstance(payload.get("seamgrim"), dict) else None
    age3 = payload.get("age3") if isinstance(payload.get("age3"), dict) else None
    age4 = payload.get("age4") if isinstance(payload.get("age4"), dict) else None
    oi = payload.get("oi405_406") if isinstance(payload.get("oi405_406"), dict) else None
    seamgrim_failed = failed_count(seamgrim, "failed_steps")
    age3_failed = failed_count(age3, "failed_criteria")
    age4_failed = failed_count(age4, "failed_criteria")
    oi_failed = failed_count(oi, "failed_packs")
    reason = "-"
    if not overall_ok:
        digest = payload.get("failure_digest")
        if isinstance(digest, list) and digest:
            reason = str(digest[0])[:220]
    parts = [
        f"schema={q(SCHEMA)}",
        f"status={'pass' if overall_ok else 'fail'}",
        f"overall_ok={int(overall_ok)}",
        f"seamgrim_failed_steps={seamgrim_failed}",
        f"age3_failed_criteria={age3_failed}",
        f"age4_failed_criteria={age4_failed}",
        f"oi_failed_packs={oi_failed}",
        f"report_path={q(report_path)}",
        f"generated_at_utc={q(str(payload.get('generated_at_utc', '-')))}",
        f"reason={q(reason)}",
    ]
    return " ".join(parts) + "\n", overall_ok


def main() -> int:
    parser = argparse.ArgumentParser(description="Render one-line aggregate gate status")
    parser.add_argument("aggregate_report", help="path to ddn.ci.aggregate_report.v1")
    parser.add_argument("--out", required=True, help="output status-line text path")
    parser.add_argument("--fail-on-bad", action="store_true", help="return non-zero when status is fail")
    args = parser.parse_args()

    report_path = Path(args.aggregate_report)
    out_path = Path(args.out)
    payload = load_json(report_path)
    line, overall_ok = build_line(report_path, payload)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(line, encoding="utf-8")
    print(f"[ci-aggregate-status-line] out={out_path} overall_ok={int(overall_ok)}")
    if args.fail_on_bad and not overall_ok:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
