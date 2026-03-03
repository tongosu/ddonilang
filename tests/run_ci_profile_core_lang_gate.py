#!/usr/bin/env python
from __future__ import annotations

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


def emit(proc: subprocess.CompletedProcess[str]) -> tuple[str, str]:
    stdout = (proc.stdout or "").strip()
    stderr = (proc.stderr or "").strip()
    if stdout:
        print(stdout)
    if stderr:
        print(stderr)
    return stdout, stderr


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    profile = "core_lang"
    sync_profile_marker = "sanity_profile=core_lang"

    proc = run([py, "tests/run_ci_sanity_gate.py", "--profile", profile], root)
    stdout, _ = emit(proc)
    if proc.returncode != 0:
        print("ci_profile_core_lang_status=fail reason=sanity_gate_failed")
        return proc.returncode
    if "ci_sanity_status=pass" not in stdout or f"profile={profile}" not in stdout:
        print("ci_profile_core_lang_status=fail reason=pass_marker_missing")
        return 1

    with tempfile.TemporaryDirectory(prefix="ci_profile_core_lang_gate_") as td:
        report_dir = Path(td) / "reports"
        report_dir.mkdir(parents=True, exist_ok=True)
        prefix = "ci_profile_core_lang"
        report = report_dir / f"{prefix}.ci_sync_readiness.detjson"

        sync_proc = run(
            [
                py,
                "tests/run_ci_sync_readiness_check.py",
                "--report-dir",
                str(report_dir),
                "--report-prefix",
                prefix,
                "--json-out",
                str(report),
                "--skip-aggregate",
                "--sanity-profile",
                profile,
            ],
            root,
        )
        sync_stdout, _ = emit(sync_proc)
        if sync_proc.returncode != 0:
            print("ci_profile_core_lang_status=fail reason=sync_readiness_failed")
            return sync_proc.returncode
        if "ci_sync_readiness_status=pass" not in sync_stdout or sync_profile_marker not in sync_stdout:
            print("ci_profile_core_lang_status=fail reason=sync_readiness_pass_marker_missing")
            return 1

        report_proc = run(
            [
                py,
                "tests/run_ci_sync_readiness_report_check.py",
                "--report",
                str(report),
                "--require-pass",
                "--sanity-profile",
                profile,
            ],
            root,
        )
        _, _ = emit(report_proc)
        if report_proc.returncode != 0:
            print("ci_profile_core_lang_status=fail reason=sync_readiness_report_check_failed")
            return report_proc.returncode

    print("ci_profile_core_lang_status=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
