#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    tool = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "lesson_legacy_warning_autofix.py"
    if not tool.exists():
        print(f"missing tool: {tool}")
        return 1

    with tempfile.TemporaryDirectory(prefix="seamgrim_lesson_autofix_") as temp_dir:
        out_path = Path(temp_dir) / "autofix.detjson"
        cmd = [
            sys.executable,
            str(tool),
            "--include-inputs",
            "--json-out",
            str(out_path),
            "--limit",
            "5",
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
            detail = (proc.stderr or proc.stdout or "").strip()
            print(f"check=lesson_autofix detail=tool_failed:{detail}")
            return 1
        if not out_path.exists():
            print("check=lesson_autofix detail=report_missing")
            return 1
        try:
            payload = json.loads(out_path.read_text(encoding="utf-8"))
        except Exception as exc:
            print(f"check=lesson_autofix detail=report_parse_failed:{exc}")
            return 1

    if payload.get("schema") != "seamgrim.lesson_legacy_warning_autofix.v1":
        print("check=lesson_autofix detail=schema_mismatch")
        return 1
    totals = payload.get("totals")
    if not isinstance(totals, dict):
        print("check=lesson_autofix detail=totals_missing")
        return 1
    if int(totals.get("solver_skipped", 0)) != 0:
        print(f"check=lesson_autofix detail=solver_skipped_nonzero:{totals.get('solver_skipped', 0)}")
        return 1

    print("check=lesson_autofix detail=ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
