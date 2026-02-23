#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path


def load_payload(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def main() -> int:
    parser = argparse.ArgumentParser(description="Print digest from ddn.age5_close_report.v1")
    parser.add_argument("report", help="path to age5 close report detjson")
    parser.add_argument("--top", type=int, default=8, help="max failure digest lines")
    parser.add_argument("--only-failed", action="store_true", help="print digest only when overall_ok=false")
    args = parser.parse_args()

    path = Path(args.report)
    payload = load_payload(path)
    if payload is None:
        print(f"[age5-close] report missing_or_invalid: {path}")
        return 0

    overall_ok = bool(payload.get("overall_ok", False))
    criteria = payload.get("criteria")
    total = len(criteria) if isinstance(criteria, list) else 0
    failed = (
        sum(1 for row in criteria if isinstance(row, dict) and not bool(row.get("ok", False)))
        if isinstance(criteria, list)
        else 0
    )
    print(f"[age5-close] overall_ok={int(overall_ok)} criteria={total} failed={failed} report={path}")
    if args.only_failed and overall_ok:
        return 0

    digest = payload.get("failure_digest")
    if isinstance(digest, list) and digest:
        for line in digest[: max(1, int(args.top))]:
            print(f" - {line}")
    else:
        print(" - failure_digest=(none)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
