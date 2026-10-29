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
    parser = argparse.ArgumentParser(description="Check seamgrim full gate surface contract")
    parser.add_argument("--out", help="optional detjson output path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    with tempfile.TemporaryDirectory(prefix="seamgrim_full_gate_surface_contract_") as tmp:
        tmp_dir = Path(tmp)
        scene_report_path = tmp_dir / "scene_session.detjson"

        checks = [
            ("export_graph_preprocess", [sys.executable, "tests/run_seamgrim_export_graph_preprocess_check.py"]),
            (
                "scene_session",
                [
                    sys.executable,
                    "tests/run_seamgrim_scene_session_check.py",
                    "--json-out",
                    str(scene_report_path),
                ],
            ),
            ("lesson_schema_gate", [sys.executable, "tests/run_seamgrim_lesson_schema_gate.py"]),
            ("full_gate", [sys.executable, "tests/run_seamgrim_full_gate_check.py"]),
        ]

        summaries: list[dict] = []
        failures: list[str] = []
        scene_report: dict | None = None
        for name, cmd in checks:
            proc = run_step(root, cmd)
            ok = proc.returncode == 0
            detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or "-"
            if name == "scene_session" and ok and scene_report_path.exists():
                scene_report = json.loads(scene_report_path.read_text(encoding="utf-8"))
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

        report = {
            "schema": "ddn.seamgrim_full_gate_surface_contract.v1",
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "ok": not failures,
            "check_count": len(summaries),
            "ok_count": sum(1 for item in summaries if item.get("ok")),
            "scene_session_report_schema": scene_report.get("schema", "") if isinstance(scene_report, dict) else "",
            "scene_session_pack_count": len(scene_report.get("packs", [])) if isinstance(scene_report, dict) else 0,
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
            "seamgrim full gate surface contract ok "
            f"checks={report['check_count']} scene_packs={report['scene_session_pack_count']}"
        )
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
