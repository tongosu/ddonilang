#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SAMPLE_PROOFS = [
    "pack/age1_immediate_proof_smoke_v1/expected/proof.detjson",
    "pack/age4_proof_solver_deny_failure_v1/expected/proof.detjson",
    "pack/age4_proof_input_replay_missing_failure_v1/expected/proof.detjson",
    "pack/age4_proof_clock_replay_parse_failure_v1/expected/proof.detjson",
]


def fail(detail: str) -> int:
    print(f"[proof-artifact-digest-selftest] fail: {detail}")
    return 1


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def run_render(report_out: Path, *inputs: str) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tools/scripts/render_proof_artifact_summary.py",
        *inputs,
        "--report-out",
        str(report_out),
        "--top",
        "8",
    ]
    return subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def run_digest(report: Path, *extra: str) -> subprocess.CompletedProcess[str]:
    cmd = [sys.executable, "tools/scripts/print_proof_artifact_digest.py", str(report), *extra]
    return subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def main() -> int:
    proof_paths = [ROOT / rel for rel in SAMPLE_PROOFS]
    for path in proof_paths:
        if not path.exists():
            return fail(f"sample_missing={path}")

    with tempfile.TemporaryDirectory(prefix="proof_artifact_digest_selftest_") as td:
        root = Path(td)
        mixed_report = root / "proof_summary.detjson"
        success_report = root / "proof_success.detjson"

        render_proc = run_render(mixed_report, *SAMPLE_PROOFS)
        if render_proc.returncode != 0:
            return fail(f"render_rc={render_proc.returncode} stderr={render_proc.stderr.strip()}")
        render_stdout = str(render_proc.stdout or "")
        if "[proof-summary] artifacts=4 verified=1 unverified=3" not in render_stdout:
            return fail("render_summary_line_missing")

        report = load_json(mixed_report)
        if str(report.get("schema", "")).strip() != "ddn.proof_artifact_summary.v1":
            return fail("report_schema_mismatch")
        if int(report.get("artifact_count", -1)) != 4:
            return fail("artifact_count_mismatch")
        if int(report.get("verified_count", -1)) != 1:
            return fail("verified_count_mismatch")
        if int(report.get("unverified_count", -1)) != 3:
            return fail("unverified_count_mismatch")
        if not str(report.get("summary_hash", "")).startswith("sha256:"):
            return fail("summary_hash_missing")

        runtime_error_counts = {
            str(row.get("code", "")).strip(): int(row.get("count", 0) or 0)
            for row in report.get("runtime_error_counts", [])
            if isinstance(row, dict)
        }
        expected_counts = {
            "E_OPEN_DENIED": 1,
            "E_OPEN_LOG_PARSE": 1,
            "E_OPEN_REPLAY_MISS": 1,
        }
        if runtime_error_counts != expected_counts:
            return fail(f"runtime_error_counts_mismatch={runtime_error_counts}")
        if int(report.get("runtime_error_artifact_count", -1)) != 3:
            return fail("runtime_error_artifact_count_mismatch")
        if int(report.get("runtime_error_state_hash_present_count", -1)) != 3:
            return fail("runtime_error_state_hash_present_count_mismatch")

        digest_proc = run_digest(mixed_report, "--top", "4", "--only-failed")
        if digest_proc.returncode != 0:
            return fail(f"digest_rc={digest_proc.returncode}")
        digest_stdout = str(digest_proc.stdout or "")
        if "[proof-artifact] artifacts=4 verified=1 unverified=3" not in digest_stdout:
            return fail("digest_summary_line_missing")
        if "runtime_errors=E_OPEN_DENIED:1,E_OPEN_LOG_PARSE:1,E_OPEN_REPLAY_MISS:1" not in digest_stdout:
            return fail("digest_runtime_error_line_missing")
        if "runtime_error_statehash=3/3" not in digest_stdout:
            return fail("digest_runtime_error_statehash_line_missing")
        if "proof_blocks=" not in digest_stdout or "성공:1" not in digest_stdout or "실패:1" not in digest_stdout:
            return fail("digest_proof_block_line_missing")
        if " - entry=pack/age4_proof_solver_deny_failure_v1/input.ddn runtime_error=E_OPEN_DENIED" not in digest_stdout:
            return fail("digest_failure_line_missing")

        success_proc = run_render(success_report, "pack/age1_immediate_proof_smoke_v1/expected/proof.detjson")
        if success_proc.returncode != 0:
            return fail(f"success_render_rc={success_proc.returncode}")
        success_digest = run_digest(success_report, "--only-failed")
        if success_digest.returncode != 0:
            return fail(f"success_digest_rc={success_digest.returncode}")
        success_stdout = str(success_digest.stdout or "")
        if "[proof-artifact] artifacts=1 verified=1 unverified=0" not in success_stdout:
            return fail("success_digest_summary_missing")
        if "failure_digest=(none)" in success_stdout:
            return fail("success_only_failed_should_skip_digest_lines")

    print("[proof-artifact-digest-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
