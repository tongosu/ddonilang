#!/usr/bin/env python
from __future__ import annotations

import json
import socket
import subprocess
import sys
import tempfile
from pathlib import Path


def safe_print(text: str) -> None:
    message = str(text)
    try:
        print(message)
        return
    except UnicodeEncodeError:
        pass
    enc = getattr(sys.stdout, "encoding", None) or "utf-8"
    sys.stdout.write(message.encode(enc, errors="replace").decode(enc, errors="replace") + "\n")


def fail(msg: str) -> int:
    safe_print(f"[seamgrim-graph-api-parity-selftest] fail: {msg}")
    return 1


def pick_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        sock.listen(1)
        return int(sock.getsockname()[1])


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    port = pick_free_port()
    with tempfile.TemporaryDirectory(prefix="seamgrim_graph_api_parity_") as tmp:
        out_path = Path(tmp) / "graph_api_parity.detjson"
        proc = subprocess.run(
            [
                sys.executable,
                "tests/run_seamgrim_graph_api_parity_check.py",
                "--host",
                "127.0.0.1",
                "--port",
                str(port),
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
        if report.get("schema") != "ddn.seamgrim_graph_api_parity.v1":
            return fail(f"schema mismatch: {report.get('schema')}")
        case_count = int(report.get("case_count", 0))
        if case_count < 2:
            return fail(f"case_count too small: {case_count}")
        if report.get("points_match_count") != case_count:
            return fail(f"points_match_count mismatch: {report.get('points_match_count')}")
        if report.get("input_hash_match_count") != case_count:
            return fail(f"input_hash_match_count mismatch: {report.get('input_hash_match_count')}")
        if report.get("result_hash_match_count") != case_count:
            return fail(f"result_hash_match_count mismatch: {report.get('result_hash_match_count')}")
        if report.get("meta_match_count") != case_count:
            return fail(f"meta_match_count mismatch: {report.get('meta_match_count')}")
        cases = report.get("cases")
        if not isinstance(cases, list) or len(cases) != case_count:
            return fail("cases payload mismatch")
        for item in cases:
            if not item.get("ok"):
                return fail(f"case not ok: {item}")
            for key in (
                "points_match",
                "input_hash_match",
                "result_hash_match",
                "input_name_match",
                "input_desc_match",
                "graph_schema_match",
            ):
                if not item.get(key):
                    return fail(f"case field mismatch key={key} item={item}")
    safe_print("[seamgrim-graph-api-parity-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
