#!/usr/bin/env python
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

REUSE_REPORT_ENV_KEY = "DDN_AGE4_PROOF_TRANSPORT_CONTRACT_SELFTEST_REUSE_REPORT"
REPORT_PATH_ENV_KEY = "DDN_AGE4_PROOF_TRANSPORT_CONTRACT_SELFTEST_REPORT_JSON"
PROGRESS_PATH_ENV_KEY = "DDN_AGE4_PROOF_TRANSPORT_CONTRACT_SELFTEST_PROGRESS_JSON"


def fail(msg: str) -> int:
    print(f"[age4-proof-transport-report-selftest] fail: {msg}")
    return 1


def load_json(path: Path) -> dict:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError(f"json root must be object: {path}")
    return data


def env_path(key: str) -> Path | None:
    raw = str(os.environ.get(key, "")).strip()
    if not raw:
        return None
    return Path(raw)


def run_selftest(report: Path, progress: Path) -> subprocess.CompletedProcess[str]:
    cmd = [sys.executable, "tests/run_age4_proof_transport_contract_selftest.py"]
    env = dict(os.environ)
    env[REPORT_PATH_ENV_KEY] = str(report)
    env[PROGRESS_PATH_ENV_KEY] = str(progress)
    env["DDN_AGE4_PROOF_TRANSPORT_DYNAMIC_WORKER_MODE"] = "off"
    return subprocess.run(
        cmd,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="age4_proof_transport_report_selftest_") as tmp:
        report = Path(tmp) / "age4_proof_transport_contract_selftest.detjson"
        progress = Path(tmp) / "age4_proof_transport_contract_selftest.progress.detjson"
        reuse_report = str(os.environ.get(REUSE_REPORT_ENV_KEY, "")).strip() == "1"
        env_report = env_path(REPORT_PATH_ENV_KEY)
        env_progress = env_path(PROGRESS_PATH_ENV_KEY)

        if reuse_report and env_report is not None and env_report.exists():
            report = env_report
            if env_progress is not None:
                progress = env_progress
        else:
            proc = run_selftest(report, progress)
            if proc.returncode != 0:
                return fail(f"selftest failed out={proc.stdout} err={proc.stderr}")

        if not report.exists():
            return fail(f"report missing: {report}")
        doc = load_json(report)
        if str(doc.get("schema", "")) != "ddn.ci.age4_proof_transport_contract_selftest.report.v1":
            return fail(f"schema mismatch: {doc.get('schema')}")
        if not bool(doc.get("overall_ok", False)):
            return fail("overall_ok must be true")
        if str(doc.get("checks_text", "")).strip() == "":
            return fail("checks_text must be non-empty")

        requested = int(doc.get("max_workers_requested", 0))
        effective = int(doc.get("max_workers_effective", 0))
        if requested <= 0 or effective <= 0:
            return fail("max_workers_requested/effective must be positive")

        policy = doc.get("dynamic_worker_policy")
        if not isinstance(policy, dict):
            return fail("dynamic_worker_policy must be object")
        if str(policy.get("mode", "")).strip() == "":
            return fail("dynamic_worker_policy.mode missing")
        if not isinstance(policy.get("decisions"), list):
            return fail("dynamic_worker_policy.decisions must be list")

        runs = doc.get("script_runs")
        if not isinstance(runs, list) or not runs:
            return fail("script_runs must be non-empty list")
        for row in runs:
            if not isinstance(row, dict):
                return fail("script_runs row must be object")
            if str(row.get("script", "")).strip() == "":
                return fail("script_runs.script missing")
            if str(row.get("check", "")).strip() == "":
                return fail("script_runs.check missing")
            if int(row.get("returncode", 1)) != 0:
                return fail("script_runs.returncode must be 0 for success report")
            if int(row.get("observed_elapsed_ms", 0)) < 0:
                return fail("script_runs.observed_elapsed_ms must be >= 0")

        if progress.exists():
            progress_doc = load_json(progress)
            if str(progress_doc.get("status", "")) != "completed":
                return fail(f"progress status must be completed: {progress_doc.get('status')}")

    print("[age4-proof-transport-report-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
