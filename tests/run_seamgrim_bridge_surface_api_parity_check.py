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

EXPORTERS = {
    "text": "export_text.py",
    "table": "export_table.py",
    "structure": "export_structure.py",
}


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
        if "surface" not in data or "input_path" not in data:
            raise ValueError(f"{pack_dir}/golden.jsonl line {idx}: missing surface/input_path")
        rows.append(data)
    return rows


def run_export_surface(root: Path, surface: str, input_path: Path) -> dict:
    script_name = EXPORTERS.get(surface)
    if not script_name:
        raise ValueError(f"unsupported surface: {surface}")
    exporter = root / "solutions" / "seamgrim_ui_mvp" / "tools" / script_name
    with tempfile.TemporaryDirectory(prefix=f"seamgrim_{surface}_api_parity_export_") as tmp:
        out_path = Path(tmp) / f"{surface}.json"
        cmd = [sys.executable, str(exporter), str(input_path), str(out_path)]
        proc = subprocess.run(
            cmd,
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if proc.returncode != 0:
            raise RuntimeError(proc.stderr.strip() or proc.stdout.strip() or f"{surface} exporter failed")
        return json.loads(out_path.read_text(encoding="utf-8"))


def main() -> int:
    parser = argparse.ArgumentParser(description="Check seamgrim standalone/API parity for text/table/structure")
    parser.add_argument("--pack", default="pack/seamgrim_bridge_surface_v0_basics")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=18790)
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
            module_name="seamgrim_ddn_exec_server_check_for_bridge_surface",
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
            surface = str(case["surface"])
            input_path = pack_dir / input_rel
            ddn_text = input_path.read_text(encoding="utf-8")
            with ThreadPoolExecutor(max_workers=2) as case_pool:
                standalone_future = case_pool.submit(run_export_surface, root, surface, input_path)
                payload_future = case_pool.submit(server_module.post_run, base_url, ddn_text)
                standalone_doc = standalone_future.result()
                payload = payload_future.result()
            if not isinstance(payload, dict) or not payload.get("ok"):
                failures.append(f"{surface}:{input_rel}: api run failed")
                summaries.append({"case": input_rel, "surface": surface, "ok": False, "detail": "api run failed"})
                continue
            api_doc = payload.get(surface, {})
            standalone_meta = standalone_doc.get("meta", {}) if isinstance(standalone_doc, dict) else {}
            api_meta = api_doc.get("meta", {}) if isinstance(api_doc, dict) else {}

            doc_match = canonical_json(standalone_doc) == canonical_json(api_doc)
            schema_match = standalone_doc.get("schema") == api_doc.get("schema")
            source_hash_match = standalone_meta.get("source_input_hash") == api_meta.get("source_input_hash")
            title_match = standalone_meta.get("title") == api_meta.get("title")

            detail_parts = []
            for label, ok in (
                ("doc", doc_match),
                ("schema", schema_match),
                ("source_input_hash", source_hash_match),
                ("title", title_match),
            ):
                if not ok:
                    detail_parts.append(label)
            ok = not detail_parts
            if not ok:
                failures.append(f"{surface}:{input_rel}: mismatch {','.join(detail_parts)}")
            summaries.append(
                {
                    "case": input_rel,
                    "surface": surface,
                    "ok": ok,
                    "doc_match": doc_match,
                    "schema_match": schema_match,
                    "source_input_hash_match": source_hash_match,
                    "title_match": title_match,
                    "standalone_source_input_hash": standalone_meta.get("source_input_hash", ""),
                    "api_source_input_hash": api_meta.get("source_input_hash", ""),
                    "detail": "ok" if ok else ",".join(detail_parts),
                }
            )

        report = {
            "schema": "ddn.seamgrim_bridge_surface_api_parity.v1",
            "pack": str(Path(args.pack)).replace("\\", "/"),
            "case_count": len(summaries),
            "doc_match_count": sum(1 for item in summaries if item.get("doc_match")),
            "schema_match_count": sum(1 for item in summaries if item.get("schema_match")),
            "source_input_hash_match_count": sum(1 for item in summaries if item.get("source_input_hash_match")),
            "title_match_count": sum(1 for item in summaries if item.get("title_match")),
            "cases": summaries,
        }
        if args.out:
            write_json(root / Path(args.out), report)
        if failures:
            for failure in failures:
                print(failure)
            return 1
        print(
            "seamgrim bridge surface api parity ok "
            f"cases={report['case_count']} docs={report['doc_match_count']}"
        )
        return 0
    finally:
        stop_parity_server(started_proc)


if __name__ == "__main__":
    raise SystemExit(main())
