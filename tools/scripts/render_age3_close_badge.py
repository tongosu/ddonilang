#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def build_badge(
    status_path: Path,
    status_line_path: Path | None,
    status_doc: dict | None,
) -> tuple[dict, bool]:
    if not isinstance(status_doc, dict):
        payload = {
            "schema": "ddn.seamgrim.age3_close_badge.v1",
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "label": "age3",
            "message": "invalid-status",
            "color": "lightgray",
            "status": "fail",
            "overall_ok": False,
            "criteria_total": 0,
            "criteria_failed_count": -1,
            "report_path": "-",
            "status_path": str(status_path),
            "status_line_path": str(status_line_path) if status_line_path is not None else "",
            "reason": "invalid_or_missing_status",
        }
        return payload, False

    status = str(status_doc.get("status", "fail")).strip() or "fail"
    overall_ok = bool(status_doc.get("overall_ok", False))
    total = int(status_doc.get("criteria_total", 0))
    failed = int(status_doc.get("criteria_failed_count", 0))
    if overall_ok:
        message = "pass"
        color = "brightgreen"
    else:
        message = f"fail ({failed}/{total})"
        color = "red"
    payload = {
        "schema": "ddn.seamgrim.age3_close_badge.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "label": "age3",
        "message": message,
        "color": color,
        "status": status,
        "overall_ok": overall_ok,
        "criteria_total": total,
        "criteria_failed_count": failed,
        "report_path": str(status_doc.get("report_path", "-")),
        "status_path": str(status_path),
        "status_line_path": str(status_line_path) if status_line_path is not None else "",
        "reason": "-" if overall_ok else ",".join(status_doc.get("failed_criteria", []))[:240],
    }
    return payload, overall_ok


def main() -> int:
    parser = argparse.ArgumentParser(description="Render AGE3 close badge json")
    parser.add_argument("status_json", help="path to ddn.seamgrim.age3_close_status.v1")
    parser.add_argument("--status-line", default="", help="optional path to age3 close status line")
    parser.add_argument("--out", required=True, help="output badge json path")
    parser.add_argument("--fail-on-bad", action="store_true", help="return non-zero when badge status is fail")
    args = parser.parse_args()

    status_path = Path(args.status_json)
    status_line_path = Path(args.status_line) if args.status_line.strip() else None
    out_path = Path(args.out)
    status_doc = load_json(status_path)
    badge, overall_ok = build_badge(status_path, status_line_path, status_doc)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(badge, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(
        f"[age3-close-badge] out={out_path} status={badge.get('status')} "
        f"failed={badge.get('criteria_failed_count', 0)}"
    )
    if args.fail_on_bad and not overall_ok:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
