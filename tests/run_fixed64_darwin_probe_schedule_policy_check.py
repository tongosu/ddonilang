#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path


SCHEMA = "ddn.fixed64.darwin_probe_schedule_policy.v1"


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="fixed64 darwin probe cadence policy check (schedule interval must be shorter than max-age)."
    )
    parser.add_argument("--max-age-minutes", type=float, default=360.0, help="fixed64 threeway report max age (minutes)")
    parser.add_argument(
        "--schedule-interval-minutes",
        type=float,
        default=180.0,
        help="darwin probe production schedule interval (minutes)",
    )
    parser.add_argument("--json-out", default="", help="optional detjson output path")
    args = parser.parse_args()

    payload: dict[str, object] = {
        "schema": SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": False,
        "status": "fail",
        "reason": "-",
        "max_age_minutes": float(args.max_age_minutes),
        "schedule_interval_minutes": float(args.schedule_interval_minutes),
    }

    max_age = float(args.max_age_minutes)
    interval = float(args.schedule_interval_minutes)
    if max_age <= 0:
        payload["reason"] = "max_age_minutes must be > 0"
        if args.json_out.strip():
            write_json(Path(args.json_out).resolve(), payload)
        print("[fixed64-darwin-schedule-policy] fail: max_age_minutes must be > 0", file=sys.stderr)
        return 1
    if interval <= 0:
        payload["reason"] = "schedule_interval_minutes must be > 0"
        if args.json_out.strip():
            write_json(Path(args.json_out).resolve(), payload)
        print("[fixed64-darwin-schedule-policy] fail: schedule_interval_minutes must be > 0", file=sys.stderr)
        return 1
    if interval >= max_age:
        payload["reason"] = "schedule interval must be shorter than max age"
        if args.json_out.strip():
            write_json(Path(args.json_out).resolve(), payload)
        print(
            "[fixed64-darwin-schedule-policy] fail: "
            f"schedule_interval_minutes={interval:.3f} must be < max_age_minutes={max_age:.3f}",
            file=sys.stderr,
        )
        return 1

    payload["ok"] = True
    payload["status"] = "pass"
    payload["reason"] = "-"
    if args.json_out.strip():
        write_json(Path(args.json_out).resolve(), payload)
    print(
        "[fixed64-darwin-schedule-policy] ok "
        f"max_age_minutes={max_age:.3f} schedule_interval_minutes={interval:.3f}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
