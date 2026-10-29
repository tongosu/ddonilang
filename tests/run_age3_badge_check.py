#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


EXPECTED_SCHEMA = "ddn.seamgrim.age3_close_badge.v1"


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate age3_close_badge.detjson format")
    parser.add_argument("--badge", required=True, help="path to age3_close_badge.detjson")
    parser.add_argument("--status-json", required=True, help="path to age3_close_status.detjson")
    parser.add_argument("--require-pass", action="store_true", help="also require overall_ok=true")
    args = parser.parse_args()

    badge_path = Path(args.badge)
    status_path = Path(args.status_json)
    badge = load_json(badge_path)
    status = load_json(status_path)
    if badge is None:
        print(f"invalid badge json: {badge_path}", file=sys.stderr)
        return 1
    if status is None:
        print(f"invalid status json: {status_path}", file=sys.stderr)
        return 1

    errors: list[str] = []
    if badge.get("schema") != EXPECTED_SCHEMA:
        errors.append(f"schema mismatch: {badge.get('schema')}")
    if str(badge.get("label", "")).strip() != "age3":
        errors.append("label must be age3")
    if not str(badge.get("message", "")).strip():
        errors.append("message is empty")
    if not str(badge.get("color", "")).strip():
        errors.append("color is empty")

    status_status = str(status.get("status", "fail")).strip() or "fail"
    badge_status = str(badge.get("status", "fail")).strip() or "fail"
    if badge_status != status_status:
        errors.append(f"status mismatch: badge={badge_status} status={status_status}")

    badge_ok = bool(badge.get("overall_ok", False))
    status_ok = bool(status.get("overall_ok", False))
    if badge_ok != status_ok:
        errors.append(f"overall_ok mismatch: badge={int(badge_ok)} status={int(status_ok)}")

    if int(badge.get("criteria_total", 0)) != int(status.get("criteria_total", 0)):
        errors.append("criteria_total mismatch")
    if int(badge.get("criteria_failed_count", 0)) != int(status.get("criteria_failed_count", 0)):
        errors.append("criteria_failed_count mismatch")
    if str(badge.get("status_path", "")).strip() != str(status_path):
        errors.append("status_path mismatch")

    if args.require_pass and not badge_ok:
        errors.append("badge overall_ok=false")

    if errors:
        print(f"age3 badge check failed: {badge_path}", file=sys.stderr)
        for line in errors[:12]:
            print(f" - {line}", file=sys.stderr)
        return 1

    print(
        f"age3 badge check ok: status={badge_status} "
        f"overall_ok={int(badge_ok)} failed={int(badge.get('criteria_failed_count', 0))}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
