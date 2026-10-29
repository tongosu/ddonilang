#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
from pathlib import Path


DEFAULT_FORBIDDEN_PATTERNS = [
    "build/reports/seamgrim_lesson_inventory.json",
    "net::ERR_ABORTED 404",
    "404 (Not Found)",
    "MODULE_TYPELESS_PACKAGE_JSON",
]


def find_forbidden_pattern(payload: dict, patterns: list[str]) -> str:
    haystacks = [
        str(payload.get("stdout") or ""),
        str(payload.get("stderr") or ""),
    ]
    for pattern in patterns:
        token = str(pattern or "").strip()
        if not token:
            continue
        for hay in haystacks:
            if token in hay:
                return token
    return ""


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate browse selection flow json report")
    parser.add_argument("--report", required=True, help="browse selection flow report path")
    parser.add_argument(
        "--schema",
        default="seamgrim.browse_selection_flow_check.v1",
        help="expected schema value",
    )
    parser.add_argument(
        "--allow-fail",
        action="store_true",
        help="allow report ok=false",
    )
    parser.add_argument(
        "--allow-forbidden-pattern",
        action="store_true",
        help="allow known 404 noise patterns in stdout/stderr",
    )
    parser.add_argument(
        "--forbid-pattern",
        action="append",
        default=[],
        help="additional forbidden output pattern (repeatable)",
    )
    args = parser.parse_args()

    report_path = Path(str(args.report))
    if not report_path.exists():
        print(f"check=browse_selection_report_missing detail=path={report_path}")
        return 1

    try:
        payload = json.loads(report_path.read_text(encoding="utf-8"))
    except Exception as exc:
        print(f"check=browse_selection_report_parse_failed detail={exc}")
        return 1

    if not isinstance(payload, dict):
        print("check=browse_selection_report_invalid detail=payload_not_object")
        return 1

    schema = str(payload.get("schema", "")).strip()
    if schema != str(args.schema):
        print(
            "check=browse_selection_report_invalid "
            f"detail=schema expected={args.schema} actual={schema or '-'}"
        )
        return 1

    ok = bool(payload.get("ok", False))
    if not args.allow_fail and not ok:
        detail = str(payload.get("stderr") or payload.get("stdout") or "report_ok_false").strip()
        print(f"check=browse_selection_report_failed detail={detail}")
        return 1

    patterns = [*DEFAULT_FORBIDDEN_PATTERNS, *list(args.forbid_pattern or [])]
    if not args.allow_forbidden_pattern:
        matched = find_forbidden_pattern(payload, patterns)
        if matched:
            print(
                "check=browse_selection_report_forbidden_pattern "
                f"detail=pattern={matched}"
            )
            return 1

    print("seamgrim browse selection report check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
