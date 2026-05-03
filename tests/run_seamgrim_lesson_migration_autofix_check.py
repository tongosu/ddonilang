#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


REQUIRED_TOTAL_KEYS = (
    "range_rewrites",
    "range_skipped",
    "range_hash_rewrites",
    "range_hash_skipped",
    "setup_colon_rewrites",
    "hook_colon_rewrites",
    "hook_alias_rewrites",
)


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    tool = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "lesson_migration_autofix.py"
    if not tool.exists():
        print(f"check=lesson_migration_autofix detail=tool_missing:{tool}")
        return 1

    with tempfile.TemporaryDirectory(prefix="seamgrim_lesson_migration_autofix_") as temp_dir:
        out_path = Path(temp_dir) / "lesson_migration_autofix.detjson"
        cmd = [
            sys.executable,
            str(tool),
            "--json-out",
            str(out_path),
            "--limit",
            "0",
        ]
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
            print(f"check=lesson_migration_autofix detail=tool_failed:{detail}")
            return 1
        if not out_path.exists():
            print("check=lesson_migration_autofix detail=report_missing")
            return 1
        try:
            payload = json.loads(out_path.read_text(encoding="utf-8"))
        except Exception as exc:
            print(f"check=lesson_migration_autofix detail=report_parse_failed:{exc}")
            return 1

    if payload.get("schema") != "seamgrim.lesson.migration_autofix.v1":
        print("check=lesson_migration_autofix detail=schema_mismatch")
        return 1
    totals = payload.get("totals")
    if not isinstance(totals, dict):
        print("check=lesson_migration_autofix detail=totals_missing")
        return 1
    for key in REQUIRED_TOTAL_KEYS:
        value = totals.get(key)
        if not isinstance(value, int):
            print(f"check=lesson_migration_autofix detail=total_key_invalid:{key}")
            return 1
    print(
        "check=lesson_migration_autofix detail="
        f"ok:changed={payload.get('changed', '-')}:targets={payload.get('targets', '-')}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
