#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
DEFAULT_SCAN_ROOT = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons"
PROMOTE_BACKUP_SUFFIX = ".before_age3_promote.bak"

PATTERNS: dict[str, re.Pattern[str]] = {
    "priority_range_comment": re.compile(r"//\s*범위\s*\([^)\n]*\)", re.MULTILINE),
    "priority_range_hash": re.compile(r"#\s*범위\s*(?:\(|:)", re.MULTILINE),
    "priority_setup_colon": re.compile(r"^\s*채비\s*:\s*\{", re.MULTILINE),
    "info_legacy_show": re.compile(r"보여주기\s*\.", re.MULTILINE),
    "info_legacy_start_colon": re.compile(r"\(\s*(?:시작|처음)\s*\)\s*할때\s*:", re.MULTILINE),
    "info_legacy_tick_colon": re.compile(r"\(\s*(?:매마디|매틱)\s*\)\s*마다\s*:", re.MULTILINE),
    "info_legacy_tick_interval_colon": re.compile(r"\(\s*[1-9][0-9]*\s*마디\s*\)\s*마다\s*:", re.MULTILINE),
    "info_legacy_start_alias": re.compile(r"\(\s*처음\s*\)\s*할때\b", re.MULTILINE),
    "info_legacy_tick_alias": re.compile(r"\(\s*매틱\s*\)\s*마다\b", re.MULTILINE),
}
PRIORITY_KEYS = ("priority_range_comment", "priority_range_hash", "priority_setup_colon")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Scan lesson DDN files for migration-priority legacy patterns.",
    )
    parser.add_argument(
        "--scan-root",
        default=str(DEFAULT_SCAN_ROOT),
        help="root directory to scan recursively for *.ddn",
    )
    parser.add_argument(
        "--json-out",
        help="optional json output path",
    )
    parser.add_argument(
        "--include-preview",
        action="store_true",
        help="include *.age3.preview.ddn files",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=20,
        help="max rows to print in console (default: 20)",
    )
    parser.add_argument(
        "--fail-on-priority",
        action="store_true",
        help="return non-zero when priority_total is greater than zero",
    )
    return parser.parse_args()


def should_include(path: Path, include_preview: bool) -> bool:
    if path.suffix != ".ddn":
        return False
    if path.name.endswith(".bak.ddn"):
        return False
    if path.stem.endswith(PROMOTE_BACKUP_SUFFIX):
        return False
    if (not include_preview) and path.stem.endswith(".age3.preview"):
        return False
    return True


def scan_file(path: Path) -> dict[str, int]:
    text = path.read_text(encoding="utf-8")
    counts: dict[str, int] = {}
    for key, pattern in PATTERNS.items():
        counts[key] = len(pattern.findall(text))
    counts["priority_total"] = sum(int(counts[key]) for key in PRIORITY_KEYS)
    counts["legacy_total"] = (
        int(counts["priority_total"])
        + int(counts["info_legacy_show"])
        + int(counts["info_legacy_start_colon"])
        + int(counts["info_legacy_tick_colon"])
        + int(counts["info_legacy_tick_interval_colon"])
        + int(counts["info_legacy_start_alias"])
        + int(counts["info_legacy_tick_alias"])
    )
    return counts


def merge_counts(total: dict[str, int], row: dict[str, int]) -> None:
    for key, value in row.items():
        total[key] = int(total.get(key, 0)) + int(value)


def main() -> int:
    args = parse_args()
    scan_root = Path(args.scan_root).resolve()
    if not scan_root.exists():
        raise SystemExit(f"scan root not found: {scan_root}")

    files = sorted(path for path in scan_root.rglob("*.ddn") if should_include(path, bool(args.include_preview)))
    rows: list[dict[str, object]] = []
    totals: dict[str, int] = {key: 0 for key in PATTERNS}
    totals["priority_total"] = 0
    totals["legacy_total"] = 0

    for file_path in files:
        counts = scan_file(file_path)
        merge_counts(totals, counts)
        try:
            rel_path = file_path.relative_to(ROOT)
            path_text = str(rel_path)
        except ValueError:
            path_text = str(file_path)
        rows.append({"path": path_text, **counts})

    rows_sorted = sorted(
        rows,
        key=lambda item: (
            -int(item.get("priority_total", 0)),
            -int(item.get("legacy_total", 0)),
            str(item.get("path", "")),
        ),
    )

    print(f"files={len(rows_sorted)}")
    print(
        "summary:",
        " ".join(
            [
                f"priority_range_comment={totals['priority_range_comment']}",
                f"priority_range_hash={totals['priority_range_hash']}",
                f"priority_setup_colon={totals['priority_setup_colon']}",
                f"info_legacy_show={totals['info_legacy_show']}",
                f"info_legacy_start_colon={totals['info_legacy_start_colon']}",
                f"info_legacy_tick_colon={totals['info_legacy_tick_colon']}",
                f"info_legacy_tick_interval_colon={totals['info_legacy_tick_interval_colon']}",
                f"info_legacy_start_alias={totals['info_legacy_start_alias']}",
                f"info_legacy_tick_alias={totals['info_legacy_tick_alias']}",
                f"priority_total={totals['priority_total']}",
            ]
        ),
    )

    limit = max(0, int(args.limit))
    for row in rows_sorted[:limit]:
        if int(row.get("legacy_total", 0)) <= 0:
            continue
        print(
            f"[legacy] {row['path']} priority={row['priority_total']} "
            f"range={row['priority_range_comment']} range_hash={row['priority_range_hash']} setup_colon={row['priority_setup_colon']} "
            f"show={row['info_legacy_show']} start_colon={row['info_legacy_start_colon']} "
            f"tick_colon={row['info_legacy_tick_colon']} interval_tick_colon={row['info_legacy_tick_interval_colon']} "
            f"start_alias={row['info_legacy_start_alias']} tick_alias={row['info_legacy_tick_alias']}"
        )

    if args.json_out:
        out_path = Path(args.json_out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        report = {
            "schema": "seamgrim.lesson.migration_lint.v1",
            "scan_root": str(scan_root),
            "include_preview": bool(args.include_preview),
            "files": len(rows_sorted),
            "totals": totals,
            "rows": rows_sorted,
            "top_priority_rows": [row for row in rows_sorted if int(row.get("priority_total", 0)) > 0][:20],
        }
        out_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    if args.fail_on_priority and int(totals.get("priority_total", 0)) > 0:
        print(
            "check=lesson_migration_lint detail="
            f"priority_nonzero:count={totals['priority_total']}:files={len(rows_sorted)}"
        )
        return 1

    print(
        "check=lesson_migration_lint detail="
        f"ok:priority_total={totals['priority_total']}:files={len(rows_sorted)}:include_preview={int(bool(args.include_preview))}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
