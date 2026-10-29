#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    tool = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "lesson_sim_conversion_planner.py"
    if not tool.exists():
        print(f"missing tool: {tool}")
        return 1

    with tempfile.TemporaryDirectory(prefix="seamgrim_sim_conversion_planner_") as temp_dir:
        out_path = Path(temp_dir) / "plan.detjson"
        cmd = [
            sys.executable,
            str(tool),
            "--json-out",
            str(out_path),
            "--limit",
            "3",
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
            print(f"check=sim_conversion_planner detail=tool_failed:{detail}")
            return 1
        if not out_path.exists():
            print("check=sim_conversion_planner detail=report_missing")
            return 1
        try:
            payload = json.loads(out_path.read_text(encoding="utf-8"))
        except Exception as exc:
            print(f"check=sim_conversion_planner detail=report_parse_failed:{exc}")
            return 1

    if payload.get("schema") != "seamgrim.sim_conversion_plan.v1":
        print("check=sim_conversion_planner detail=schema_mismatch")
        return 1
    if not isinstance(payload.get("category_counts"), dict):
        print("check=sim_conversion_planner detail=category_counts_missing")
        return 1
    rows = payload.get("rows")
    if not isinstance(rows, list) or not rows:
        print("check=sim_conversion_planner detail=rows_missing")
        return 1
    if any("category" not in row for row in rows if isinstance(row, dict)):
        print("check=sim_conversion_planner detail=row_category_missing")
        return 1

    print("check=sim_conversion_planner detail=ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
