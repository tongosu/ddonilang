#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    with tempfile.TemporaryDirectory(prefix="seamgrim_bridge_surface_api_parity_selftest_") as tmp:
        report_path = Path(tmp) / "report.detjson"
        cmd = [
            sys.executable,
            str(root / "tests" / "run_seamgrim_bridge_surface_api_parity_check.py"),
            "--out",
            str(report_path),
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
            print(proc.stdout)
            print(proc.stderr, file=sys.stderr)
            return proc.returncode
        report = json.loads(report_path.read_text(encoding="utf-8"))
        assert report["schema"] == "ddn.seamgrim_bridge_surface_api_parity.v1"
        assert report["case_count"] == 3
        assert report["doc_match_count"] == 3
        assert report["schema_match_count"] == 3
        assert report["source_input_hash_match_count"] == 3
        assert report["title_match_count"] == 3
        for case in report.get("cases", []):
            assert case.get("ok") is True
            assert case.get("doc_match") is True
            assert case.get("schema_match") is True
            assert case.get("source_input_hash_match") is True
            assert case.get("title_match") is True
    print("[seamgrim-bridge-surface-api-parity-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
