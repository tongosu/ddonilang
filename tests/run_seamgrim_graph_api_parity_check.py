#!/usr/bin/env python
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor
import json
import subprocess
import sys
import tempfile
from pathlib import Path

from _seamgrim_parity_server_lib import start_parity_server, stop_parity_server

def canonical_json(data) -> str:
    return json.dumps(data, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def write_json(path: Path, data: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def load_cases(pack_dir: Path) -> list[dict]:
    rows = []
    for idx, line in enumerate(pack_dir.joinpath("golden.jsonl").read_text(encoding="utf-8-sig").splitlines(), 1):
        if not line.strip():
            continue
        data = json.loads(line)
        if "input_path" not in data:
            raise ValueError(f"{pack_dir}/golden.jsonl line {idx}: missing input_path")
        rows.append(data)
    return rows


def run_export_graph(root: Path, input_path: Path) -> dict:
    export_graph = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "export_graph.py"
    with tempfile.TemporaryDirectory(prefix="seamgrim_graph_api_parity_export_") as tmp:
        out_path = Path(tmp) / "graph.json"
        cmd = [sys.executable, str(export_graph), str(input_path), str(out_path)]
        proc = subprocess.run(
            cmd,
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if proc.returncode != 0:
            raise RuntimeError(proc.stderr.strip() or proc.stdout.strip() or "export_graph failed")
        return json.loads(out_path.read_text(encoding="utf-8"))


def main() -> int:
    parser = argparse.ArgumentParser(description="Check export_graph and ddn_exec_server graph parity")
    parser.add_argument("--pack", default="pack/seamgrim_graph_v0_basics")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=18789)
    parser.add_argument("--timeout-sec", type=float, default=15.0)
    parser.add_argument(
        "--require-existing-server",
        action="store_true",
        help="fail when ddn_exec_server is not already alive at --host/--port (do not spawn)",
    )
    parser.add_argument("--out", help="optional detjson output path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    pack_dir = root / Path(args.pack)
    if not pack_dir.exists():
        print(f"missing {pack_dir}")
        return 1

    try:
        server_module, base_url, started_proc = start_parity_server(
            root=root,
            module_name="seamgrim_ddn_exec_server_check",
            host=args.host,
            port=int(args.port),
            timeout_sec=float(args.timeout_sec),
            require_existing_server=bool(args.require_existing_server),
        )
    except RuntimeError as exc:
        print(str(exc))
        return 1

    try:
        cases = load_cases(pack_dir)
        summaries: list[dict] = []
        failures: list[str] = []
        for case in cases:
            input_rel = str(case["input_path"])
            input_path = pack_dir / input_rel
            ddn_text = input_path.read_text(encoding="utf-8")
            with ThreadPoolExecutor(max_workers=2) as case_pool:
                standalone_future = case_pool.submit(run_export_graph, root, input_path)
                payload_future = case_pool.submit(server_module.post_run, base_url, ddn_text)
                standalone = standalone_future.result()
                payload = payload_future.result()
            if not isinstance(payload, dict) or not payload.get("ok"):
                failures.append(f"{input_rel}: api run failed")
                summaries.append({"case": input_rel, "ok": False, "detail": "api run failed"})
                continue
            api_graph = payload.get("graph", {})
            standalone_meta = standalone.get("meta", {}) if isinstance(standalone, dict) else {}
            api_meta = api_graph.get("meta", {}) if isinstance(api_graph, dict) else {}
            standalone_points = standalone.get("series", [{}])[0].get("points", []) if isinstance(standalone, dict) else []
            api_points = api_graph.get("series", [{}])[0].get("points", []) if isinstance(api_graph, dict) else []

            points_match = canonical_json(standalone_points) == canonical_json(api_points)
            input_hash_match = standalone_meta.get("source_input_hash") == api_meta.get("source_input_hash")
            result_hash_match = standalone_meta.get("result_hash") == api_meta.get("result_hash")
            input_name_match = standalone_meta.get("input_name") == api_meta.get("input_name")
            input_desc_match = standalone_meta.get("input_desc") == api_meta.get("input_desc")
            graph_schema_match = (
                standalone.get("schema") == "seamgrim.graph.v0"
                and api_graph.get("schema") == "seamgrim.graph.v0"
            )

            detail_parts = []
            for label, ok in (
                ("points", points_match),
                ("input_hash", input_hash_match),
                ("result_hash", result_hash_match),
                ("input_name", input_name_match),
                ("input_desc", input_desc_match),
                ("schema", graph_schema_match),
            ):
                if not ok:
                    detail_parts.append(label)
            ok = not detail_parts
            if not ok:
                failures.append(f"{input_rel}: mismatch {','.join(detail_parts)}")
            summaries.append(
                {
                    "case": input_rel,
                    "ok": ok,
                    "points_match": points_match,
                    "input_hash_match": input_hash_match,
                    "result_hash_match": result_hash_match,
                    "input_name_match": input_name_match,
                    "input_desc_match": input_desc_match,
                    "graph_schema_match": graph_schema_match,
                    "standalone_input_hash": standalone_meta.get("source_input_hash", ""),
                    "api_input_hash": api_meta.get("source_input_hash", ""),
                    "standalone_result_hash": standalone_meta.get("result_hash", ""),
                    "api_result_hash": api_meta.get("result_hash", ""),
                    "detail": "ok" if ok else ",".join(detail_parts),
                }
            )

        report = {
            "schema": "ddn.seamgrim_graph_api_parity.v1",
            "pack": str(Path(args.pack)).replace("\\", "/"),
            "case_count": len(summaries),
            "points_match_count": sum(1 for item in summaries if item.get("points_match")),
            "input_hash_match_count": sum(1 for item in summaries if item.get("input_hash_match")),
            "result_hash_match_count": sum(1 for item in summaries if item.get("result_hash_match")),
            "meta_match_count": sum(
                1
                for item in summaries
                if item.get("input_name_match") and item.get("input_desc_match")
            ),
            "cases": summaries,
        }
        if args.out:
            write_json(root / Path(args.out), report)
        if failures:
            for failure in failures:
                print(failure)
            return 1
        print(
            "seamgrim graph api parity ok "
            f"cases={report['case_count']} points={report['points_match_count']}"
        )
        return 0
    finally:
        stop_parity_server(started_proc)


if __name__ == "__main__":
    raise SystemExit(main())
