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
    parser = argparse.ArgumentParser(description="Check seamgrim runtime fallback surface contract")
    parser.add_argument("--out", help="optional detjson output path")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    with tempfile.TemporaryDirectory(prefix="seamgrim_runtime_fallback_surface_contract_") as tmp:
        tmp_dir = Path(tmp)
        metrics_report_path = tmp_dir / "runtime_fallback_metrics.detjson"

        checks = [
            ("lesson_path_fallback", [sys.executable, "tests/run_seamgrim_lesson_path_fallback_check.py"]),
            ("shape_fallback_mode", [sys.executable, "tests/run_seamgrim_shape_fallback_mode_check.py"]),
            ("motion_projectile_fallback", [sys.executable, "tests/run_seamgrim_motion_projectile_fallback_check.py"]),
            (
                "runtime_fallback_metrics",
                [
                    sys.executable,
                    "tests/run_seamgrim_runtime_fallback_metrics_check.py",
                    "--json-out",
                    str(metrics_report_path),
                ],
            ),
            (
                "runtime_fallback_policy",
                [
                    sys.executable,
                    "tests/run_seamgrim_runtime_fallback_policy_check.py",
                    "--metrics",
                    str(metrics_report_path),
                ],
            ),
        ]

        summaries: list[dict] = []
        failures: list[str] = []
        metrics_report: dict | None = None
        for name, cmd in checks:
            proc = run_step(root, cmd)
            ok = proc.returncode == 0
            detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or "-"
            if name == "runtime_fallback_metrics" and ok and metrics_report_path.exists():
                metrics_report = json.loads(metrics_report_path.read_text(encoding="utf-8"))
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
            "schema": "ddn.seamgrim_runtime_fallback_surface_contract.v1",
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "ok": not failures,
            "check_count": len(summaries),
            "ok_count": sum(1 for item in summaries if item.get("ok")),
            "metrics_report_schema": metrics_report.get("schema", "") if isinstance(metrics_report, dict) else "",
            "metrics_total": int(metrics_report.get("total", 0)) if isinstance(metrics_report, dict) else 0,
            "metrics_fallback_count": int(metrics_report.get("fallback_count", 0)) if isinstance(metrics_report, dict) else 0,
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
            "seamgrim runtime fallback surface contract ok "
            f"checks={report['check_count']} total={report['metrics_total']} fallback={report['metrics_fallback_count']}"
        )
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
