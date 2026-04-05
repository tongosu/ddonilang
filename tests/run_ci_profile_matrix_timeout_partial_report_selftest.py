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


def fail(detail: str, proc: subprocess.CompletedProcess[str] | None = None) -> int:
    print(f"check=ci_profile_matrix_timeout_partial_report_selftest detail={detail}")
    if proc is not None:
        if (proc.stdout or "").strip():
            print(proc.stdout.strip())
        if (proc.stderr or "").strip():
            print(proc.stderr.strip())
    return 1


def write_fake_gate(path: Path) -> None:
    payload = """#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import time
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--report-dir", default="")
    parser.add_argument("--report-prefix", default="")
    parser.add_argument("--quick", action="store_true")
    args = parser.parse_args()
    report_dir = Path(args.report_dir)
    report_dir.mkdir(parents=True, exist_ok=True)
    prefix = args.report_prefix.strip() or "ci_profile_core_lang"
    sanity_report = report_dir / f"{prefix}.ci_sanity_gate.detjson"
    pipeline_progress = report_dir / f"{prefix}.ci_sanity_gate.pipeline_emit_flags_check.progress.detjson"
    selftest_progress = report_dir / f"{prefix}.ci_sanity_gate.pipeline_emit_flags_selftest.progress.detjson"
    payload = {
        "schema": "ddn.ci.sanity_gate.v1",
        "status": "running",
        "code": "RUNNING",
        "step": "pipeline_emit_flags_selftest",
        "msg": "-",
        "profile": "core_lang",
        "current_step": "pipeline_emit_flags_selftest",
        "last_completed_step": "pipeline_emit_flags_check",
        "steps": [
            {
                "step": "pipeline_emit_flags_check",
                "ok": True,
                "returncode": 0,
                "cmd": ["python", "tests/run_ci_pipeline_emit_flags_check.py"],
                "elapsed_ms": 1234,
            }
        ],
        "steps_count": 1,
        "total_elapsed_ms": 2345,
    }
    sanity_report.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\\n", encoding="utf-8")
    pipeline_progress.write_text(
        json.dumps(
            {
                "schema": "ddn.ci.pipeline_emit_flags_progress.v1",
                "status": "running",
                "current_section": "profile_matrix_policy_runtime",
                "last_completed_section": "gitlab_static,azure_static",
                "sections_completed": 2,
                "total_elapsed_ms": 1789,
            },
            ensure_ascii=False,
            indent=2,
        )
        + "\\n",
        encoding="utf-8",
    )
    selftest_progress.write_text(
        json.dumps(
            {
                "schema": "ddn.ci.pipeline_emit_flags_selftest_progress.v1",
                "status": "running",
                "current_case": "broken_age5_policy_helper_scope_should_fail",
                "last_completed_case": "missing_featured_seed_catalog_autogen_should_fail",
                "current_probe": "wait_runtime_ui_check.age5_combined_policy_runtime",
                "last_completed_probe": "spawn_runtime_ui_check",
                "cases_completed": 10,
                "total_elapsed_ms": 2876,
            },
            ensure_ascii=False,
            indent=2,
        )
        + "\\n",
        encoding="utf-8",
    )
    time.sleep(2.0)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
"""
    path.write_text(payload, encoding="utf-8")


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    with tempfile.TemporaryDirectory(prefix="ci_profile_matrix_timeout_partial_report_") as td:
        temp_root = Path(td)
        fake_gate = temp_root / "fake_core_lang_gate.py"
        report_dir = temp_root / "reports"
        matrix_report = temp_root / "matrix.detjson"
        write_fake_gate(fake_gate)
        proc = run(
            [
                py,
                "tests/run_ci_profile_matrix_gate.py",
                "--profiles",
                "core_lang",
                "--report-dir",
                str(report_dir),
                "--report-prefix",
                "partial_probe",
                "--step-timeout-sec",
                "0.2",
                "--json-out",
                str(matrix_report),
                "--profile-gate-override",
                f"core_lang={fake_gate}",
            ],
            root,
        )
        if proc.returncode == 0:
            return fail("matrix_timeout_case_should_fail", proc)
        stdout = str(proc.stdout or "")
        required = (
            "ci_profile_core_lang_sanity_total_elapsed_ms=2345",
            "ci_profile_core_lang_sanity_slowest_step=pipeline_emit_flags_check",
            "ci_profile_core_lang_sanity_slowest_elapsed_ms=1234",
            "ci_sanity_current_step=pipeline_emit_flags_selftest",
            "ci_sanity_last_completed_step=pipeline_emit_flags_check",
            "ci_pipeline_emit_flags_current_section=profile_matrix_policy_runtime",
            "ci_pipeline_emit_flags_last_completed_section=gitlab_static,azure_static",
            "ci_pipeline_emit_flags_total_elapsed_ms=1789",
            "ci_pipeline_emit_flags_selftest_current_case=broken_age5_policy_helper_scope_should_fail",
            "ci_pipeline_emit_flags_selftest_last_completed_case=missing_featured_seed_catalog_autogen_should_fail",
            "ci_pipeline_emit_flags_selftest_current_probe=wait_runtime_ui_check.age5_combined_policy_runtime",
            "ci_pipeline_emit_flags_selftest_last_completed_probe=spawn_runtime_ui_check",
            "ci_pipeline_emit_flags_selftest_total_elapsed_ms=2876",
            "step timeout after 0.200s",
        )
        for marker in required:
            if marker not in stdout and marker not in str(proc.stderr or ""):
                return fail(f"missing_marker:{marker}", proc)
    print("[ci-profile-matrix-timeout-partial-report-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
