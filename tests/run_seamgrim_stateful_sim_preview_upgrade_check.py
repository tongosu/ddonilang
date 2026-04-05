#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    tool = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "lesson_stateful_sim_preview_upgrade.py"
    if not tool.exists():
        print(f"missing tool: {tool}")
        return 1

    with tempfile.TemporaryDirectory(prefix="seamgrim_stateful_preview_upgrade_") as temp_dir:
        report_path = Path(temp_dir) / "stateful_upgrade.detjson"
        cmd = [
            sys.executable,
            str(tool),
            "--include-inputs",
            "--json-out",
            str(report_path),
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
            print(f"check=stateful_preview_upgrade detail=tool_failed:{detail}")
            return 1
        if not report_path.exists():
            print("check=stateful_preview_upgrade detail=report_missing")
            return 1
        try:
            payload = json.loads(report_path.read_text(encoding="utf-8"))
        except Exception as exc:
            print(f"check=stateful_preview_upgrade detail=report_parse_failed:{exc}")
            return 1

    if payload.get("schema") != "seamgrim.stateful_sim_preview_upgrade.v1":
        print("check=stateful_preview_upgrade detail=schema_mismatch")
        return 1
    convertible = int(payload.get("convertible", 0) or 0)
    rows = payload.get("rows")
    if not isinstance(rows, list) or not rows:
        print("check=stateful_preview_upgrade detail=rows_missing")
        return 1
    if convertible > 0:
        converted_rows = [row for row in rows if isinstance(row, dict) and row.get("converted")]
        if not converted_rows:
            print("check=stateful_preview_upgrade detail=converted_rows_missing")
            return 1

    print(f"check=stateful_preview_upgrade detail=ok:convertible={convertible}:rows={len(rows)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
