#!/usr/bin/env python
from __future__ import annotations

import argparse
import importlib.util
import json
import subprocess
import sys
import tempfile
from pathlib import Path


def safe_print(text: str) -> None:
    data = str(text)
    encoding = sys.stdout.encoding or "utf-8"
    try:
        print(data)
    except UnicodeEncodeError:
        print(data.encode(encoding, errors="replace").decode(encoding, errors="replace"))


def canonical_json(data) -> str:
    return json.dumps(data, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def load_export_graph_module(root: Path):
    script_path = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "export_graph.py"
    spec = importlib.util.spec_from_file_location("seamgrim_export_graph", script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"module load failed: {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def write_json(path: Path, data: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def load_cases(pack_dir: Path) -> list[dict]:
    lines = pack_dir.joinpath("golden.jsonl").read_text(encoding="utf-8-sig").splitlines()
    cases = []
    for idx, line in enumerate(lines, 1):
        if not line.strip():
            continue
        try:
            data = json.loads(line)
        except json.JSONDecodeError as exc:
            raise ValueError(f"{pack_dir}/golden.jsonl line {idx}: {exc}")
        if "input_path" not in data or "expected_graph" not in data:
            raise ValueError(f"{pack_dir}/golden.jsonl line {idx}: missing input_path/expected_graph")
        cases.append(data)
    return cases


def build_snapshot(case_name: str, input_text: str, graph: dict) -> dict:
    meta = graph.get("meta", {}) if isinstance(graph, dict) else {}
    return {
        "schema": "seamgrim.snapshot.v0",
        "ts": "2026-03-23T00:00:00Z",
        "note": case_name,
        "run": {
            "id": case_name,
            "label": case_name,
            "source": {"kind": "ddn", "text": input_text},
            "inputs": {},
            "graph": graph,
            "hash": {
                "input": meta.get("source_input_hash", ""),
                "result": meta.get("result_hash", ""),
            },
        },
    }


def run_case(root: Path, pack_dir: Path, case: dict, module) -> tuple[dict, list[str]]:
    export_graph = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "export_graph.py"
    input_path = pack_dir / case["input_path"]
    expected_path = pack_dir / case["expected_graph"]
    failures: list[str] = []
    with tempfile.TemporaryDirectory(prefix="seamgrim_graph_golden_") as tmp:
        temp_output = Path(tmp) / "graph.v0.json"
        cmd = [sys.executable, str(export_graph), str(input_path), str(temp_output)]
        result = subprocess.run(
            cmd,
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if result.returncode != 0:
            detail = result.stderr.strip() or result.stdout.strip() or "runner failed"
            return {
                "case": str(case.get("input_path", "")),
                "ok": False,
                "detail": detail,
            }, [f"{case.get('input_path')}: export failed: {detail}"]

        input_text = input_path.read_text(encoding="utf-8")
        expected = json.loads(expected_path.read_text(encoding="utf-8"))
        actual = json.loads(temp_output.read_text(encoding="utf-8"))

    graph_match = canonical_json(expected) == canonical_json(actual)
    if not graph_match:
        failures.append(f"{case.get('input_path')}: graph json mismatch")

    actual_meta = actual.get("meta", {}) if isinstance(actual, dict) else {}
    points = actual.get("series", [{}])[0].get("points", []) if isinstance(actual, dict) else []
    expected_input_hash = f"sha256:{module.hash_text(module.normalize_ddn_for_hash(input_text))}"
    expected_result_hash = f"sha256:{module.compute_result_hash(points)}"
    actual_input_hash = str(actual_meta.get("source_input_hash", "")).strip()
    actual_result_hash = str(actual_meta.get("result_hash", "")).strip()
    input_hash_match = actual_input_hash == expected_input_hash
    result_hash_match = actual_result_hash == expected_result_hash
    if not input_hash_match:
        failures.append(f"{case.get('input_path')}: source_input_hash mismatch")
    if not result_hash_match:
        failures.append(f"{case.get('input_path')}: result_hash mismatch")

    snapshot = build_snapshot(input_path.parent.name, input_text, actual)
    snapshot_hash = snapshot.get("run", {}).get("hash", {})
    snapshot_input_match = snapshot_hash.get("input") == actual_input_hash
    snapshot_result_match = snapshot_hash.get("result") == actual_result_hash
    if not snapshot_input_match:
        failures.append(f"{case.get('input_path')}: snapshot input hash mismatch")
    if not snapshot_result_match:
        failures.append(f"{case.get('input_path')}: snapshot result hash mismatch")

    actual_name = str(actual_meta.get("input_name", "")).strip()
    actual_desc = str(actual_meta.get("input_desc", "")).strip()
    meta_header_present = bool(actual_name or actual_desc)

    summary = {
        "case": str(case.get("input_path", "")),
        "ok": not failures,
        "graph_schema": actual.get("schema"),
        "graph_match": graph_match,
        "input_hash_match": input_hash_match,
        "result_hash_match": result_hash_match,
        "snapshot_input_match": snapshot_input_match,
        "snapshot_result_match": snapshot_result_match,
        "actual_input_hash": actual_input_hash,
        "expected_input_hash": expected_input_hash,
        "actual_result_hash": actual_result_hash,
        "expected_result_hash": expected_result_hash,
        "meta_header_present": meta_header_present,
        "input_name": actual_name,
        "input_desc": actual_desc,
        "stdout_lines": case.get("stdout", []),
        "detail": "ok" if not failures else "; ".join(failures),
    }
    return summary, failures


def main() -> int:
    parser = argparse.ArgumentParser(description="Run seamgrim graph bridge/hash golden checks")
    parser.add_argument(
        "--pack",
        default="pack/seamgrim_graph_v0_basics",
        help="graph pack directory",
    )
    parser.add_argument(
        "--out",
        help="optional detjson report output path",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    pack_dir = root / Path(args.pack)
    if not pack_dir.exists():
        print(f"missing {pack_dir}")
        return 1

    module = load_export_graph_module(root)
    cases = load_cases(pack_dir)
    summaries: list[dict] = []
    failures: list[str] = []
    for case in cases:
        summary, case_failures = run_case(root, pack_dir, case, module)
        summaries.append(summary)
        failures.extend(case_failures)

    report = {
        "schema": "ddn.seamgrim_graph_bridge_contract.v1",
        "pack": str(Path(args.pack)).replace("\\", "/"),
        "case_count": len(summaries),
        "meta_header_case_count": sum(1 for item in summaries if item.get("meta_header_present")),
        "input_hash_match_count": sum(1 for item in summaries if item.get("input_hash_match")),
        "result_hash_match_count": sum(1 for item in summaries if item.get("result_hash_match")),
        "snapshot_hash_match_count": sum(
            1
            for item in summaries
            if item.get("snapshot_input_match") and item.get("snapshot_result_match")
        ),
        "cases": summaries,
    }
    if args.out:
        write_json(root / Path(args.out), report)

    if failures:
        for failure in failures:
            safe_print(failure)
        return 1

    print(
        "seamgrim graph golden ok "
        f"cases={report['case_count']} meta_header_cases={report['meta_header_case_count']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
