#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path


def load_payload(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def build_status(report_path: Path, payload: dict | None) -> tuple[dict, bool]:
    if not isinstance(payload, dict):
        status = {
            "schema": "ddn.seamgrim.age3_close_status.v1",
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "report_path": str(report_path),
            "overall_ok": False,
            "status": "fail",
            "failed_criteria": [],
            "criteria_total": 0,
            "criteria_failed_count": 0,
            "reason": "invalid_or_missing_report",
        }
        return status, False

    criteria = payload.get("criteria")
    failed_criteria: list[str] = []
    if isinstance(criteria, list):
        for row in criteria:
            if isinstance(row, dict) and not bool(row.get("ok", False)):
                failed_criteria.append(str(row.get("name", "-")))
    overall_ok = bool(payload.get("overall_ok", False)) and len(failed_criteria) == 0
    status = {
        "schema": "ddn.seamgrim.age3_close_status.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "report_path": str(report_path),
        "overall_ok": overall_ok,
        "status": "pass" if overall_ok else "fail",
        "failed_criteria": failed_criteria,
        "criteria_total": len(criteria) if isinstance(criteria, list) else 0,
        "criteria_failed_count": len(failed_criteria),
        "seamgrim_report_path": str(payload.get("seamgrim_report_path", "")),
        "ui_age3_report_path": str(payload.get("ui_age3_report_path", "")),
    }
    return status, overall_ok


def main() -> int:
    parser = argparse.ArgumentParser(description="Render compact AGE3 close status JSON")
    parser.add_argument("report", help="path to ddn.seamgrim.age3_close_report.v1")
    parser.add_argument("--out", required=True, help="output status json path")
    parser.add_argument("--fail-on-bad", action="store_true", help="return non-zero when status=fail")
    args = parser.parse_args()

    report_path = Path(args.report)
    out_path = Path(args.out)
    payload = load_payload(report_path)
    status, overall_ok = build_status(report_path, payload)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(status, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(
        f"[age3-close-status] out={out_path} status={status.get('status')} "
        f"failed={status.get('criteria_failed_count', 0)}"
    )
    if args.fail_on_bad and not overall_ok:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
