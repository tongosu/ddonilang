#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path


REQUIRED_TOTAL_KEYS = (
    "priority_range_comment",
    "priority_setup_colon",
    "info_legacy_show",
    "info_legacy_start_colon",
    "info_legacy_tick_colon",
    "info_legacy_tick_interval_colon",
    "info_legacy_start_alias",
    "info_legacy_tick_alias",
    "priority_total",
    "legacy_total",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run lesson_migration_lint tool contract check")
    parser.add_argument(
        "--allow-priority-nonzero",
        action="store_true",
        help="do not fail when priority_total is non-zero",
    )
    parser.add_argument(
        "--include-preview",
        action="store_true",
        help="scan *.age3.preview.ddn files too",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    root = Path(__file__).resolve().parent.parent
    tool = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "lesson_migration_lint.py"
    if not tool.exists():
        print(f"check=lesson_migration_lint detail=tool_missing:{tool}")
        return 1

    with tempfile.TemporaryDirectory(prefix="seamgrim_lesson_migration_lint_") as temp_dir:
        out_path = Path(temp_dir) / "lesson_migration_lint.detjson"
        cmd = [
            sys.executable,
            str(tool),
            "--json-out",
            str(out_path),
            "--limit",
            "0",
        ]
        if bool(args.include_preview):
            cmd.append("--include-preview")
        proc = subprocess.run(
            cmd,
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if proc.returncode != 0:
            detail = (proc.stderr or proc.stdout or "").strip() or "tool_failed"
            print(f"check=lesson_migration_lint detail=tool_failed:{detail}")
            return 1
        if not out_path.exists():
            print("check=lesson_migration_lint detail=report_missing")
            return 1
        try:
            payload = json.loads(out_path.read_text(encoding="utf-8"))
        except Exception as exc:
            print(f"check=lesson_migration_lint detail=report_parse_failed:{exc}")
            return 1

    if payload.get("schema") != "seamgrim.lesson.migration_lint.v1":
        print("check=lesson_migration_lint detail=schema_mismatch")
        return 1
    totals = payload.get("totals")
    if not isinstance(totals, dict):
        print("check=lesson_migration_lint detail=totals_missing")
        return 1
    for key in REQUIRED_TOTAL_KEYS:
        value = totals.get(key)
        if not isinstance(value, int):
            print(f"check=lesson_migration_lint detail=total_key_invalid:{key}")
            return 1
    priority_total = int(totals.get("priority_total", 0))
    if priority_total > 0 and not bool(args.allow_priority_nonzero):
        print(
            "check=lesson_migration_lint detail="
            f"priority_nonzero:count={priority_total}:files={payload.get('files', '-')}"
        )
        return 1
    print(
        "check=lesson_migration_lint detail="
        f"ok:priority_total={priority_total}:files={payload.get('files', '-')}:allow_priority_nonzero={int(bool(args.allow_priority_nonzero))}:include_preview={int(bool(args.include_preview))}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
