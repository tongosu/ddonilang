#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str) -> int:
    text = f"[seamgrim-group-id-summary-selftest] fail: {msg}"
    try:
        print(text)
    except UnicodeEncodeError:
        sys.stdout.buffer.write(text.encode("utf-8", errors="replace") + b"\n")
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    script = root / "tests" / "run_seamgrim_group_id_summary_check.py"
    with tempfile.TemporaryDirectory(prefix="seamgrim-group-id-summary-") as tmp:
        out = Path(tmp) / "group_id_summary.detjson"
        proc = subprocess.run(
            [sys.executable, str(script), "--json-out", str(out)],
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if proc.returncode != 0:
            return fail(f"script_failed stdout={proc.stdout} stderr={proc.stderr}")
        if not out.exists():
            return fail("summary json missing")
        doc = json.loads(out.read_text(encoding="utf-8"))
        if doc.get("schema") != "ddn.seamgrim.group_id_summary.v1":
            return fail(f"schema mismatch: {doc.get('schema')}")
        if doc.get("category_counts") != {"overlay": 2, "synthesis": 2}:
            return fail(f"category counts mismatch: {doc.get('category_counts')}")
        rows = doc.get("rows")
        if not isinstance(rows, list) or len(rows) != 4:
            return fail(f"row count mismatch: {rows}")
        categories = [row.get("category") for row in rows]
        if categories != [
            "overlay_compare",
            "overlay_session",
            "synthesis_seed",
            "synthesis_scene_session",
        ]:
            return fail(f"category order mismatch: {categories}")

    print("[seamgrim-group-id-summary-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
