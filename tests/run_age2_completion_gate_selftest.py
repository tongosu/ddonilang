#!/usr/bin/env python
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

REUSE_REPORT_ENV_KEY = "DDN_AGE2_COMPLETION_GATE_SELFTEST_REUSE_REPORT"
REPORT_PATH_ENV_KEY = "DDN_AGE2_COMPLETION_GATE_REPORT_JSON"
MUST_REPORT_PATH_ENV_KEY = "DDN_AGE2_COMPLETION_GATE_MUST_REPORT_JSON"
SHOULD_REPORT_PATH_ENV_KEY = "DDN_AGE2_COMPLETION_GATE_SHOULD_REPORT_JSON"


def fail(msg: str) -> int:
    print(f"[age2-completion-gate-selftest] fail: {msg}")
    return 1


def run_gate(report: Path, must_report: Path, should_report: Path) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_age2_completion_gate.py",
        "--report-out",
        str(report),
        "--must-report-out",
        str(must_report),
        "--should-report-out",
        str(should_report),
    ]
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


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


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="age2_completion_gate_selftest_") as tmp:
        report = Path(tmp) / "age2_completion_gate.detjson"
        must_report = Path(tmp) / "age2_completion_must_pack_report.detjson"
        should_report = Path(tmp) / "age2_completion_should_pack_report.detjson"

        reuse_report = str(os.environ.get(REUSE_REPORT_ENV_KEY, "")).strip() == "1"
        env_report = env_path(REPORT_PATH_ENV_KEY)
        env_must_report = env_path(MUST_REPORT_PATH_ENV_KEY)
        env_should_report = env_path(SHOULD_REPORT_PATH_ENV_KEY)
        if reuse_report and env_report is not None and env_report.exists():
            report = env_report
            if env_must_report is not None:
                must_report = env_must_report
            if env_should_report is not None:
                should_report = env_should_report
        else:
            proc = run_gate(report, must_report, should_report)
            if proc.returncode != 0:
                return fail(f"gate failed out={proc.stdout} err={proc.stderr}")
        if not report.exists():
            return fail(f"report missing: {report}")

        doc = load_json(report)
        if str(doc.get("schema", "")) != "ddn.age2.completion_gate.v1":
            return fail(f"schema mismatch: {doc.get('schema')}")
        if not bool(doc.get("overall_ok", False)):
            return fail("overall_ok must be true")

        criteria = doc.get("criteria")
        if not isinstance(criteria, list):
            return fail("criteria must be list")
        criteria_map: dict[str, bool] = {}
        for row in criteria:
            if not isinstance(row, dict):
                continue
            name = str(row.get("name", "")).strip()
            if not name:
                continue
            criteria_map[name] = bool(row.get("ok", False))

        required = [
            "age2_ssot_pack_contract_sync",
            "must_pack_set_pass",
            "should_pack_set_pass",
            "strict_should_gate",
        ]
        missing = [name for name in required if name not in criteria_map]
        if missing:
            return fail(f"missing criteria: {missing}")
        failed = [name for name in required if not criteria_map.get(name, False)]
        if failed:
            return fail(f"criteria must pass: {failed}")
        failure_codes = doc.get("failure_codes")
        if not isinstance(failure_codes, list):
            return fail("failure_codes must be list")

        sync = doc.get("ssot_pack_contract_sync")
        if not isinstance(sync, dict):
            return fail("ssot_pack_contract_sync must be object")
        if sync.get("missing_docs") not in ([], tuple()):
            return fail(f"ssot missing docs: {sync.get('missing_docs')}")
        if not isinstance(sync.get("ssot_must_expected"), list) or not sync.get("ssot_must_expected"):
            return fail("ssot_must_expected must be non-empty list")
        if not isinstance(sync.get("ssot_should_expected"), list) or not sync.get("ssot_should_expected"):
            return fail("ssot_should_expected must be non-empty list")

        shard_policy = doc.get("shard_policy")
        if not isinstance(shard_policy, dict):
            return fail("shard_policy must be object")
        requested = shard_policy.get("requested")
        effective = shard_policy.get("effective")
        if not isinstance(requested, dict) or not isinstance(effective, dict):
            return fail("shard_policy requested/effective must be object")
        lookback = shard_policy.get("lookback")
        if int(lookback or 0) <= 0:
            return fail("shard_policy lookback must be positive")
        source_reports_used = shard_policy.get("source_reports_used")
        if not isinstance(source_reports_used, dict):
            return fail("shard_policy source_reports_used must be object")
        must_used = source_reports_used.get("must")
        should_used = source_reports_used.get("should")
        if must_used is None or int(must_used) < 0:
            return fail("shard_policy source_reports_used.must must be non-negative")
        if should_used is None or int(should_used) < 0:
            return fail("shard_policy source_reports_used.should must be non-negative")
        must_shards = doc.get("must_shards")
        if not isinstance(must_shards, list) or not must_shards:
            return fail("must_shards must be non-empty list")
        for row in must_shards:
            if not isinstance(row, dict):
                return fail("must_shards row must be object")
            if int(row.get("shard_index", 0)) <= 0:
                return fail("must_shards shard_index must be positive")
            if not isinstance(row.get("packs"), list):
                return fail("must_shards packs must be list")

        must_report_doc = str(doc.get("must_report_path", "")).strip()
        if must_report_doc:
            must_report = Path(must_report_doc)
        if not must_report.exists():
            return fail(f"must report missing: {must_report}")
        run_should = bool(doc.get("run_should", True))
        should_report_doc = str(doc.get("should_report_path", "")).strip()
        if run_should:
            should_shards = doc.get("should_shards")
            if not isinstance(should_shards, list) or not should_shards:
                return fail("should_shards must be non-empty list when run_should")
            if should_report_doc and should_report_doc != "-":
                should_report = Path(should_report_doc)
            if not should_report.exists():
                return fail(f"should report missing: {should_report}")

    print("[age2-completion-gate-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
