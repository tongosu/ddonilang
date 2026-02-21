#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path


SCHEMA = "ddn.seamgrim.age3_close_status_line.v1"


def load_payload(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def q(value: object) -> str:
    return json.dumps(str(value), ensure_ascii=False)


def build_line(status_path: Path, payload: dict | None) -> tuple[str, bool]:
    if not isinstance(payload, dict):
        parts = [
            f"schema={q(SCHEMA)}",
            "status=fail",
            "overall_ok=0",
            "criteria_total=0",
            "criteria_failed_count=-1",
            f"report_path={q('-')}",
            f"generated_at_utc={q('-')}",
            f"status_path={q(status_path)}",
            f"reason={q('invalid_or_missing_status')}",
        ]
        return " ".join(parts) + "\n", False

    status = str(payload.get("status", "fail")).strip() or "fail"
    overall_ok = bool(payload.get("overall_ok", False))
    failed = int(payload.get("criteria_failed_count", 0))
    total = int(payload.get("criteria_total", 0))
    report_path = str(payload.get("report_path", "")).strip()
    generated_at_utc = str(payload.get("generated_at_utc", "")).strip()
    parts = [
        f"schema={q(SCHEMA)}",
        f"status={status}",
        f"overall_ok={int(overall_ok)}",
        f"criteria_total={total}",
        f"criteria_failed_count={failed}",
        f"report_path={q(report_path or '-')}",
        f"generated_at_utc={q(generated_at_utc or '-')}",
        f"status_path={q(status_path)}",
        f"reason={q('-')}",
    ]
    return " ".join(parts) + "\n", overall_ok


def main() -> int:
    parser = argparse.ArgumentParser(description="Render one-line AGE3 close status")
    parser.add_argument("status_json", help="path to ddn.seamgrim.age3_close_status.v1")
    parser.add_argument("--out", required=True, help="output text file path")
    parser.add_argument("--fail-on-bad", action="store_true", help="return non-zero when status is fail")
    args = parser.parse_args()

    status_path = Path(args.status_json)
    out_path = Path(args.out)
    payload = load_payload(status_path)
    line, overall_ok = build_line(status_path, payload)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(line, encoding="utf-8")
    print(f"[age3-close-status-line] out={out_path} overall_ok={int(overall_ok)}")
    if args.fail_on_bad and not overall_ok:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
