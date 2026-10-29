#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def fail(detail: str) -> int:
    print(f"[age4-proof-report-selftest] fail: {detail}")
    return 1


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def run_report(report_out: Path, summary_out: Path) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_age4_proof_artifact_report.py",
        "--report-out",
        str(report_out),
        "--proof-summary-out",
        str(summary_out),
    ]
    return subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="age4_proof_artifact_report_") as td:
        root = Path(td)
        report_out = root / "age4_proof_artifact_report.detjson"
        summary_out = root / "proof_artifact_summary.detjson"

        proc = run_report(report_out, summary_out)
        if proc.returncode != 0:
            return fail(f"runner_rc={proc.returncode} stdout={proc.stdout.strip()} stderr={proc.stderr.strip()}")
        stdout = str(proc.stdout or "")
        if "[age4-proof-report] overall_ok=1" not in stdout:
            return fail("runner_summary_line_missing")
        if "failed_preview=-" not in stdout:
            return fail("runner_failed_preview_missing")

        if not report_out.exists():
            return fail("report_missing")
        if not summary_out.exists():
            return fail("summary_missing")

        report = load_json(report_out)
        summary = load_json(summary_out)

        if str(report.get("schema", "")).strip() != "ddn.age4.proof_artifact_report.v1":
            return fail("report_schema_mismatch")
        if str(summary.get("schema", "")).strip() != "ddn.proof_artifact_summary.v1":
            return fail("summary_schema_mismatch")
        if not bool(report.get("overall_ok", False)):
            return fail("report_overall_not_ok")
        if int(report.get("artifact_count", -1)) != 17:
            return fail("artifact_count_mismatch")
        if int(report.get("verified_count", -1)) != 4:
            return fail("verified_count_mismatch")
        if int(report.get("unverified_count", -1)) != 13:
            return fail("unverified_count_mismatch")
        if str(report.get("failed_criteria_preview", "")).strip() != "-":
            return fail("failed_criteria_preview_mismatch")
        if str(report.get("proof_summary_hash", "")).strip() != str(summary.get("summary_hash", "")).strip():
            return fail("proof_summary_hash_mismatch")
        if len(report.get("criteria", [])) != 7:
            return fail("criteria_count_mismatch")
        if report.get("failure_digest") not in ([], None):
            return fail("failure_digest_should_be_empty")

        runtime_error_counts = {
            str(row.get("code", "")).strip(): int(row.get("count", 0) or 0)
            for row in report.get("runtime_error_counts", [])
            if isinstance(row, dict)
        }
        if runtime_error_counts != {
            "E_OPEN_DENIED": 1,
            "E_OPEN_LOG_PARSE": 4,
            "E_OPEN_LOG_TAMPER": 4,
            "E_OPEN_REPLAY_MISS": 4,
        }:
            return fail(f"runtime_error_counts_mismatch={runtime_error_counts}")
        if int(report.get("runtime_error_artifact_count", -1)) != 13:
            return fail("runtime_error_artifact_count_mismatch")
        if int(report.get("runtime_error_state_hash_present_count", -1)) != 13:
            return fail("runtime_error_state_hash_present_count_mismatch")
        if int(summary.get("runtime_error_artifact_count", -1)) != 13:
            return fail("summary_runtime_error_artifact_count_mismatch")
        if int(summary.get("runtime_error_state_hash_present_count", -1)) != 13:
            return fail("summary_runtime_error_state_hash_present_count_mismatch")
        if list(summary.get("runtime_error_missing_state_hash_entries", [])) != []:
            return fail("summary_runtime_error_missing_state_hash_entries_not_empty")

    print("[age4-proof-report-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
