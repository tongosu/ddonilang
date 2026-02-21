#!/usr/bin/env python3
import argparse
import json
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Print compact summary from ddn.oi.close_report.v1"
    )
    parser.add_argument("report_path", help="path to ddn.oi.close_report.v1 json file")
    parser.add_argument(
        "--max-digest",
        type=int,
        default=10,
        help="maximum failure_digest lines to print (default: 10)",
    )
    parser.add_argument(
        "--max-slowest",
        type=int,
        default=3,
        help="maximum slowest pack lines to print (default: 3)",
    )
    parser.add_argument(
        "--only-failed",
        action="store_true",
        help="show slowest packs only among failed packs",
    )
    args = parser.parse_args()

    report_path = Path(args.report_path)
    if not report_path.exists():
        print(f"[oi-close] report missing: {report_path}", file=sys.stderr)
        return 0

    try:
        doc = json.loads(report_path.read_text(encoding="utf-8"))
    except Exception as exc:
        print(f"[oi-close] report parse failed: {report_path} ({exc})", file=sys.stderr)
        return 0

    if not isinstance(doc, dict):
        print(f"[oi-close] invalid report root: {report_path}", file=sys.stderr)
        return 0

    schema = str(doc.get("schema", ""))
    if schema != "ddn.oi.close_report.v1":
        print(f"[oi-close] unexpected schema: {schema or 'missing'}", file=sys.stderr)
        return 0

    overall_ok = bool(doc.get("overall_ok", False))
    packs = doc.get("packs")
    pack_count = len(packs) if isinstance(packs, list) else 0
    print(f"[oi-close] overall_ok={str(overall_ok).lower()} pack_count={pack_count}")

    digest = doc.get("failure_digest")
    if isinstance(digest, list) and digest:
        print("[oi-close] failure_digest:")
        max_digest = max(0, int(args.max_digest))
        shown = digest[:max_digest]
        for line in shown:
            print(f"  - {line}")
        if len(digest) > len(shown):
            print(f"  - ... ({len(digest) - len(shown)} more)")
    else:
        print("[oi-close] failure_digest: (none)")

    strict = bool(doc.get("strict", False))
    strict_errors = doc.get("strict_errors")
    if strict:
        lines = strict_errors if isinstance(strict_errors, list) else []
        if lines:
            print("[oi-close] strict_errors:")
            for line in lines[:5]:
                print(f"  - {line}")
            if len(lines) > 5:
                print(f"  - ... ({len(lines) - 5} more)")
        else:
            print("[oi-close] strict_errors: (none)")

    slow_rows: list[tuple[str, int]] = []
    if isinstance(packs, list):
        for row in packs:
            if not isinstance(row, dict):
                continue
            if args.only_failed and bool(row.get("ok", False)):
                continue
            name = row.get("pack")
            elapsed = row.get("elapsed_ms")
            if isinstance(name, str) and isinstance(elapsed, int):
                slow_rows.append((name, elapsed))
    if slow_rows:
        slow_rows.sort(key=lambda x: x[1], reverse=True)
        max_slowest = max(0, int(args.max_slowest))
        top = ", ".join([f"{name}:{elapsed}ms" for name, elapsed in slow_rows[:max_slowest]])
        print(f"[oi-close] slowest_packs={top}")
    elif args.only_failed:
        print("[oi-close] slowest_packs=(none, failed only)")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
