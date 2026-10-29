#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


EXPECTED_SCHEMA = "ddn.ci.gate_badge.v1"


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate ci_gate_badge.detjson")
    parser.add_argument("--badge", required=True, help="path to ci_gate_badge.detjson")
    parser.add_argument("--result", required=True, help="path to ci_gate_result.detjson")
    parser.add_argument("--require-pass", action="store_true", help="also require status=pass and ok=true")
    args = parser.parse_args()

    badge_path = Path(args.badge)
    result_path = Path(args.result)
    badge = load_json(badge_path)
    result = load_json(result_path)
    if badge is None:
        print(f"invalid badge json: {badge_path}", file=sys.stderr)
        return 1
    if result is None:
        print(f"invalid result json: {result_path}", file=sys.stderr)
        return 1
    if badge.get("schema") != EXPECTED_SCHEMA:
        print(f"badge schema mismatch: {badge.get('schema')}", file=sys.stderr)
        return 1
    if str(badge.get("label", "")).strip() != "ci-gate":
        print("badge label mismatch", file=sys.stderr)
        return 1
    if str(badge.get("result_path", "")).strip() != str(result_path):
        print("badge result_path mismatch", file=sys.stderr)
        return 1

    result_status = str(result.get("status", "fail")).strip() or "fail"
    result_ok = bool(result.get("ok", False))
    badge_status = str(badge.get("status", "fail")).strip() or "fail"
    badge_ok = bool(badge.get("ok", False))
    if badge_status != result_status:
        print("badge status mismatch", file=sys.stderr)
        return 1
    if badge_ok != result_ok:
        print("badge ok mismatch", file=sys.stderr)
        return 1
    if args.require_pass and not (badge_status == "pass" and badge_ok):
        print("badge is not pass", file=sys.stderr)
        return 1

    print(
        "[ci-gate-badge-check] ok "
        f"status={badge_status} ok={int(badge_ok)} message={badge.get('message')}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
