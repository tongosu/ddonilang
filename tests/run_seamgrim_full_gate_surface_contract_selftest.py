#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str) -> int:
    print(f"[seamgrim-full-gate-surface-contract-selftest] fail: {msg}")
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    with tempfile.TemporaryDirectory(prefix="seamgrim_full_gate_surface_contract_selftest_") as tmp:
        out_path = Path(tmp) / "full_gate_surface.detjson"
        proc = subprocess.run(
            [
                sys.executable,
                "tests/run_seamgrim_full_gate_surface_contract_check.py",
                "--out",
                str(out_path),
            ],
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if proc.returncode != 0:
            return fail(f"runner failed out={proc.stdout} err={proc.stderr}")
        if not out_path.exists():
            return fail("report output missing")
        report = json.loads(out_path.read_text(encoding="utf-8"))
        if report.get("schema") != "ddn.seamgrim_full_gate_surface_contract.v1":
            return fail(f"schema mismatch: {report.get('schema')}")
        if report.get("ok") is not True:
            return fail("report not ok")
        if report.get("check_count") != 4:
            return fail(f"check_count mismatch: {report.get('check_count')}")
        if report.get("ok_count") != 4:
            return fail(f"ok_count mismatch: {report.get('ok_count')}")
        if report.get("scene_session_report_schema") != "ddn.seamgrim.scene_session_check_report.v1":
            return fail(f"scene report schema mismatch: {report.get('scene_session_report_schema')}")
        if report.get("scene_session_pack_count") != 1:
            return fail(f"scene_session_pack_count mismatch: {report.get('scene_session_pack_count')}")
        checks = report.get("checks")
        if not isinstance(checks, list) or len(checks) != 4:
            return fail("checks payload mismatch")
        for expected_name in ("export_graph_preprocess", "scene_session", "lesson_schema_gate", "full_gate"):
            row = next((item for item in checks if item.get("name") == expected_name), None)
            if row is None:
                return fail(f"missing check row: {expected_name}")
            if row.get("ok") is not True:
                return fail(f"check not ok: {row}")
    print("[seamgrim-full-gate-surface-contract-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
