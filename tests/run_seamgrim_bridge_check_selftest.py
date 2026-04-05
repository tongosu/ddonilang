#!/usr/bin/env python
from __future__ import annotations

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


def fail(message: str) -> int:
    safe_print(f"[seamgrim-bridge-check-selftest] fail: {message}")
    return 1


def run_cmd(cmd: list[str], *, cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    export_graph = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "export_graph.py"
    bridge_check = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "bridge_check.py"
    input_path = root / "pack" / "seamgrim_graph_v0_basics" / "c01_line_meta_header" / "input.ddn"
    if not export_graph.exists():
        return fail(f"missing script: {export_graph}")
    if not bridge_check.exists():
        return fail(f"missing script: {bridge_check}")
    if not input_path.exists():
        return fail(f"missing input: {input_path}")

    with tempfile.TemporaryDirectory(prefix="seamgrim_bridge_check_selftest_") as tmp:
        tmp_dir = Path(tmp)
        graph_path = tmp_dir / "graph.v0.json"
        snapshot_path = tmp_dir / "snapshot.v0.json"
        bom_input_path = tmp_dir / "input_bom.ddn"
        bad_snapshot_path = tmp_dir / "snapshot_bad.v0.json"
        report_ok_path = tmp_dir / "bridge_check_ok.detjson"
        report_bom_path = tmp_dir / "bridge_check_bom.detjson"
        report_fail_path = tmp_dir / "bridge_check_fail.detjson"

        export_proc = run_cmd(
            [sys.executable, str(export_graph), str(input_path), str(graph_path)],
            cwd=root,
        )
        if export_proc.returncode != 0:
            return fail(f"export_graph failed: {(export_proc.stderr or export_proc.stdout).strip()}")
        graph_doc = json.loads(graph_path.read_text(encoding="utf-8"))
        input_text = input_path.read_text(encoding="utf-8")

        snapshot_doc = {
            "schema": "seamgrim.snapshot.v0",
            "ts": "2026-03-29T00:00:00Z",
            "note": "bridge_check_selftest",
            "run": {
                "id": "selftest",
                "label": "selftest",
                "source": {"kind": "ddn", "text": input_text},
                "inputs": {},
                "graph": graph_doc,
                "hash": {
                    "input": str(graph_doc.get("meta", {}).get("source_input_hash", "")),
                    "result": str(graph_doc.get("meta", {}).get("result_hash", "")),
                },
            },
        }
        snapshot_path.write_text(json.dumps(snapshot_doc, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

        ok_proc = run_cmd(
            [
                sys.executable,
                str(bridge_check),
                "--graph",
                str(graph_path),
                "--snapshot",
                str(snapshot_path),
                "--input-ddn",
                str(input_path),
                "--out",
                str(report_ok_path),
            ],
            cwd=root,
        )
        if ok_proc.returncode != 0:
            return fail(f"bridge_check expected ok but failed: {(ok_proc.stderr or ok_proc.stdout).strip()}")
        report_ok = json.loads(report_ok_path.read_text(encoding="utf-8"))
        if report_ok.get("schema") != "ddn.seamgrim_bridge_check.v1":
            return fail(f"report schema mismatch: {report_ok.get('schema')}")
        for key in (
            "ok",
            "graph_schema_ok",
            "snapshot_schema_ok",
            "graph_snapshot_match",
            "input_hash_match",
            "result_hash_match",
            "snapshot_hash_input_match",
            "snapshot_hash_result_match",
        ):
            if report_ok.get(key) is not True:
                return fail(f"ok report flag mismatch key={key} value={report_ok.get(key)!r}")

        # BOM(utf-8-sig) 입력 파일도 source_input_hash 비교가 동일하게 통과해야 한다.
        bom_input_path.write_text(input_text, encoding="utf-8-sig")
        bom_proc = run_cmd(
            [
                sys.executable,
                str(bridge_check),
                "--graph",
                str(graph_path),
                "--snapshot",
                str(snapshot_path),
                "--input-ddn",
                str(bom_input_path),
                "--out",
                str(report_bom_path),
            ],
            cwd=root,
        )
        if bom_proc.returncode != 0:
            return fail(f"bridge_check bom expected ok but failed: {(bom_proc.stderr or bom_proc.stdout).strip()}")
        report_bom = json.loads(report_bom_path.read_text(encoding="utf-8"))
        for key in ("ok", "input_hash_match", "result_hash_match", "snapshot_hash_input_match", "snapshot_hash_result_match"):
            if report_bom.get(key) is not True:
                return fail(f"bom report flag mismatch key={key} value={report_bom.get(key)!r}")

        bad_snapshot = json.loads(snapshot_path.read_text(encoding="utf-8"))
        bad_snapshot["run"]["hash"]["result"] = "sha256:badbadbad"
        bad_snapshot_path.write_text(
            json.dumps(bad_snapshot, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )
        fail_proc = run_cmd(
            [
                sys.executable,
                str(bridge_check),
                "--graph",
                str(graph_path),
                "--snapshot",
                str(bad_snapshot_path),
                "--input-ddn",
                str(input_path),
                "--out",
                str(report_fail_path),
            ],
            cwd=root,
        )
        if fail_proc.returncode == 0:
            return fail("bridge_check should fail on snapshot hash mismatch")
        report_fail = json.loads(report_fail_path.read_text(encoding="utf-8"))
        if report_fail.get("ok") is not False:
            return fail("fail report ok flag should be false")
        errors = report_fail.get("errors", [])
        if not any("snapshot.run.hash.result mismatch" in str(row) for row in errors):
            return fail(f"expected snapshot hash error missing: {errors}")

    safe_print("[seamgrim-bridge-check-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
