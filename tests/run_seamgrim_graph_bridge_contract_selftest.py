#!/usr/bin/env python
from __future__ import annotations

import json
import importlib.util
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str) -> int:
    print(f"[seamgrim-graph-bridge-contract-selftest] fail: {msg}")
    return 1


def load_export_graph_module(root: Path):
    script_path = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "export_graph.py"
    spec = importlib.util.spec_from_file_location("seamgrim_export_graph", script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"module load failed: {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def find_case(cases: list[dict], case_path: str) -> dict | None:
    for item in cases:
        if str(item.get("case", "")) == case_path:
            return item
    return None


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    with tempfile.TemporaryDirectory(prefix="seamgrim_graph_bridge_contract_") as tmp:
        out_path = Path(tmp) / "graph_bridge_contract.detjson"
        cmd = [
            sys.executable,
            "tests/run_seamgrim_graph_golden.py",
            "--out",
            str(out_path),
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
            return fail(f"runner failed out={proc.stdout} err={proc.stderr}")
        if not out_path.exists():
            return fail("report output missing")

        report = json.loads(out_path.read_text(encoding="utf-8"))
        if report.get("schema") != "ddn.seamgrim_graph_bridge_contract.v1":
            return fail(f"report schema mismatch: {report.get('schema')}")
        if report.get("case_count") != 3:
            return fail(f"case_count mismatch: {report.get('case_count')}")
        if report.get("meta_header_case_count") != 2:
            return fail(f"meta_header_case_count mismatch: {report.get('meta_header_case_count')}")
        if report.get("input_hash_match_count") != 3:
            return fail(f"input_hash_match_count mismatch: {report.get('input_hash_match_count')}")
        if report.get("result_hash_match_count") != 3:
            return fail(f"result_hash_match_count mismatch: {report.get('result_hash_match_count')}")
        if report.get("snapshot_hash_match_count") != 3:
            return fail(f"snapshot_hash_match_count mismatch: {report.get('snapshot_hash_match_count')}")

        cases = report.get("cases")
        if not isinstance(cases, list) or len(cases) != 3:
            return fail("cases payload mismatch")

        line_meta_case = find_case(cases, "c01_line_meta_header/input.ddn")
        if not line_meta_case:
            return fail("line meta header case missing")
        if line_meta_case.get("input_name") != "line_meta_header":
            return fail(f"input_name mismatch: {line_meta_case.get('input_name')}")
        if line_meta_case.get("input_desc") != "메타 헤더 포함 직선 그래프":
            return fail(f"input_desc mismatch: {line_meta_case.get('input_desc')}")

        line_no_meta_case = find_case(cases, "c03_line_no_meta_same_body/input.ddn")
        if not line_no_meta_case:
            return fail("line no-meta case missing")
        if line_no_meta_case.get("meta_header_present"):
            return fail("no-meta case should not have meta header")
        if line_meta_case.get("actual_input_hash") != line_no_meta_case.get("actual_input_hash"):
            return fail("meta/no-meta equivalent case input hash mismatch")
        if line_meta_case.get("actual_result_hash") != line_no_meta_case.get("actual_result_hash"):
            return fail("meta/no-meta equivalent case result hash mismatch")

        for item in cases:
            if not item.get("ok"):
                return fail(f"case not ok: {item}")
            if item.get("graph_schema") != "seamgrim.graph.v0":
                return fail(f"graph schema mismatch: {item.get('graph_schema')}")
            for key in (
                "graph_match",
                "input_hash_match",
                "result_hash_match",
                "snapshot_input_match",
                "snapshot_result_match",
            ):
                if not item.get(key):
                    return fail(f"case boolean mismatch key={key} item={item}")

        module = load_export_graph_module(root)
        canon_meta, raw_meta, body = module.parse_guide_meta_header("#원자\nx <- 1.\n")
        if canon_meta:
            return fail(f"#원자 should not be parsed as canonical meta: {canon_meta}")
        if raw_meta:
            return fail(f"#원자 should not be parsed as raw meta: {raw_meta}")
        if not str(body).startswith("#원자"):
            return fail(f"#원자 line should remain in body: {body!r}")

    print("[seamgrim-graph-bridge-contract-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
