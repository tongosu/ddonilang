#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str) -> int:
    print(f"[seamgrim-ddn-exec-server-surface-contract-selftest] fail: {msg}")
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    with tempfile.TemporaryDirectory(prefix="seamgrim_ddn_exec_server_surface_contract_selftest_") as tmp:
        out_path = Path(tmp) / "ddn_exec_server_surface.detjson"
        proc = subprocess.run(
            [
                sys.executable,
                "tests/run_seamgrim_ddn_exec_server_surface_contract_check.py",
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
        if report.get("schema") != "ddn.seamgrim_ddn_exec_server_surface_contract.v1":
            return fail(f"schema mismatch: {report.get('schema')}")
        if report.get("ok") is not True:
            return fail("report not ok")
        if report.get("check_count") != 4:
            return fail(f"check_count mismatch: {report.get('check_count')}")
        if report.get("ok_count") != 4:
            return fail(f"ok_count mismatch: {report.get('ok_count')}")
        if report.get("graph_report_schema") != "ddn.seamgrim_graph_api_parity.v1":
            return fail(f"graph_report_schema mismatch: {report.get('graph_report_schema')}")
        graph_case_count = int(report.get("graph_case_count", 0))
        if graph_case_count < 2:
            return fail(f"graph_case_count too small: {graph_case_count}")
        if report.get("bridge_report_schema") != "ddn.seamgrim_bridge_surface_api_parity.v1":
            return fail(f"bridge_report_schema mismatch: {report.get('bridge_report_schema')}")
        bridge_case_count = int(report.get("bridge_case_count", 0))
        if bridge_case_count < 3:
            return fail(f"bridge_case_count too small: {bridge_case_count}")
        if report.get("space2d_report_schema") != "ddn.seamgrim_space2d_api_parity.v1":
            return fail(f"space2d_report_schema mismatch: {report.get('space2d_report_schema')}")
        space2d_case_count = int(report.get("space2d_case_count", 0))
        if space2d_case_count < 2:
            return fail(f"space2d_case_count too small: {space2d_case_count}")
        checks = report.get("checks")
        if not isinstance(checks, list) or len(checks) != 4:
            return fail("checks payload mismatch")
        for expected_name in (
            "ddn_exec_server_gate",
            "graph_api_parity",
            "bridge_surface_api_parity",
            "space2d_api_parity",
        ):
            row = next((item for item in checks if item.get("name") == expected_name), None)
            if row is None:
                return fail(f"missing check row: {expected_name}")
            if row.get("ok") is not True:
                return fail(f"check not ok: {row}")
    print("[seamgrim-ddn-exec-server-surface-contract-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
