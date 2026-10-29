#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path


def run_step(root: Path, cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Check seamgrim ddn_exec_server surface contract")
    parser.add_argument("--out", help="optional detjson output path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    with tempfile.TemporaryDirectory(prefix="seamgrim_ddn_exec_server_surface_contract_") as tmp:
        tmp_dir = Path(tmp)
        graph_report_path = tmp_dir / "graph_api_parity.detjson"
        bridge_report_path = tmp_dir / "bridge_surface_api_parity.detjson"
        space2d_report_path = tmp_dir / "space2d_api_parity.detjson"

        checks = [
            ("ddn_exec_server_gate", [sys.executable, "tests/run_seamgrim_ddn_exec_server_gate_check.py"]),
            (
                "graph_api_parity",
                [sys.executable, "tests/run_seamgrim_graph_api_parity_check.py", "--out", str(graph_report_path)],
            ),
            (
                "bridge_surface_api_parity",
                [
                    sys.executable,
                    "tests/run_seamgrim_bridge_surface_api_parity_check.py",
                    "--out",
                    str(bridge_report_path),
                ],
            ),
            (
                "space2d_api_parity",
                [sys.executable, "tests/run_seamgrim_space2d_api_parity_check.py", "--out", str(space2d_report_path)],
            ),
        ]

        summaries: list[dict] = []
        failures: list[str] = []
        report_meta: dict[str, dict] = {}
        for name, cmd in checks:
            proc = run_step(root, cmd)
            ok = proc.returncode == 0
            detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or "-"
            if not ok:
                failures.append(f"{name}: {detail}")
            summaries.append(
                {
                    "name": name,
                    "ok": ok,
                    "returncode": proc.returncode,
                    "detail": detail,
                }
            )

        for key, path in (
            ("graph", graph_report_path),
            ("bridge", bridge_report_path),
            ("space2d", space2d_report_path),
        ):
            if path.exists():
                payload = json.loads(path.read_text(encoding="utf-8"))
                report_meta[key] = {
                    "schema": payload.get("schema", ""),
                    "case_count": int(payload.get("case_count", 0)),
                    "ok_like_count": int(
                        payload.get("doc_match_count", 0)
                        if key != "graph"
                        else payload.get("points_match_count", 0)
                    ),
                }
            else:
                report_meta[key] = {"schema": "", "case_count": 0, "ok_like_count": 0}

        report = {
            "schema": "ddn.seamgrim_ddn_exec_server_surface_contract.v1",
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "ok": not failures,
            "check_count": len(summaries),
            "ok_count": sum(1 for item in summaries if item.get("ok")),
            "graph_report_schema": report_meta["graph"]["schema"],
            "graph_case_count": report_meta["graph"]["case_count"],
            "bridge_report_schema": report_meta["bridge"]["schema"],
            "bridge_case_count": report_meta["bridge"]["case_count"],
            "space2d_report_schema": report_meta["space2d"]["schema"],
            "space2d_case_count": report_meta["space2d"]["case_count"],
            "checks": summaries,
        }
        if args.out:
            out = root / Path(args.out)
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        if failures:
            for failure in failures:
                print(failure)
            return 1
        print(
            "seamgrim ddn_exec_server surface contract ok "
            f"checks={report['check_count']} graph_cases={report['graph_case_count']}"
        )
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
