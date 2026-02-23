#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def expect(cond: bool, msg: str, proc: subprocess.CompletedProcess[str] | None = None) -> int:
    if cond:
        return 0
    print(f"check=ci_sync_readiness_selftest detail={msg}")
    if proc is not None:
        if (proc.stdout or "").strip():
            print(proc.stdout.strip())
        if (proc.stderr or "").strip():
            print(proc.stderr.strip())
    return 1


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    py = sys.executable

    with tempfile.TemporaryDirectory(prefix="ci_sync_readiness_selftest_") as td:
        report_dir = Path(td) / "reports"
        report_dir.mkdir(parents=True, exist_ok=True)
        prefix = "sync_selftest"
        expected_default_report = report_dir / f"{prefix}.ci_sync_readiness.detjson"

        quick_cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            prefix,
            "--skip-aggregate",
        ]
        quick_proc = run(quick_cmd, cwd=root)
        if expect(quick_proc.returncode == 0, "quick_mode_should_pass", quick_proc) != 0:
            return 1
        if expect(expected_default_report.exists(), "default_report_missing", quick_proc) != 0:
            return 1
        doc = load_json(expected_default_report)
        if expect(str(doc.get("schema", "")) == "ddn.ci.sync_readiness.v1", "schema_mismatch") != 0:
            return 1
        if expect(bool(doc.get("ok", False)), "ok_should_be_true") != 0:
            return 1
        if expect(bool(doc.get("skip_aggregate", False)), "skip_aggregate_should_be_true") != 0:
            return 1
        if expect(int(doc.get("steps_count", 0)) == 4, "steps_count_quick_should_be_4") != 0:
            return 1
        steps = doc.get("steps")
        if expect(isinstance(steps, list), "steps_should_be_list") != 0:
            return 1
        step_names = [str(row.get("name", "")) for row in steps if isinstance(row, dict)]
        if expect(
            step_names
            == [
                "pipeline_emit_flags_check",
                "pipeline_emit_flags_selftest",
                "sanity_gate_diagnostics_check",
                "sanity_gate",
            ],
            "unexpected_quick_steps",
        ) != 0:
            return 1

        custom_json = report_dir / "sync_readiness.custom.detjson"
        custom_cmd = [
            py,
            "tests/run_ci_sync_readiness_check.py",
            "--report-dir",
            str(report_dir),
            "--report-prefix",
            f"{prefix}_custom",
            "--skip-aggregate",
            "--json-out",
            str(custom_json),
        ]
        custom_proc = run(custom_cmd, cwd=root)
        if expect(custom_proc.returncode == 0, "custom_json_mode_should_pass", custom_proc) != 0:
            return 1
        if expect(custom_json.exists(), "custom_json_report_missing", custom_proc) != 0:
            return 1
        custom_doc = load_json(custom_json)
        if expect(bool(custom_doc.get("ok", False)), "custom_ok_should_be_true") != 0:
            return 1
        if expect(int(custom_doc.get("steps_count", 0)) == 4, "custom_steps_count_should_be_4") != 0:
            return 1

    print("ci sync readiness check selftest ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
