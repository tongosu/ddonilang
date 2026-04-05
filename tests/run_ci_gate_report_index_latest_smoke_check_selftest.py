#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from _ci_latest_smoke_contract import (
    LATEST_SMOKE_SKIP_REASON_CI_GATE_RESULT_STATUS_NOT_PASS,
    LATEST_SMOKE_SKIP_REASON_EXPECTED,
    LATEST_SMOKE_SKIP_REASON_FAST_FAIL_PATH,
    LATEST_SMOKE_SKIP_REASON_FLAG_DISABLED,
    LATEST_SMOKE_SKIP_REASON_PENDING_FAILURE_SUMMARY_REGENERATION,
)
from run_ci_gate_report_index_check_selftest import build_index_case


def fail(message: str) -> int:
    print(f"[ci-gate-report-index-latest-smoke-check-selftest] fail: {message}")
    return 1


def run_smoke(report_dir: Path, *extra_args: str) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_ci_gate_report_index_latest_smoke_check.py",
        "--report-dir",
        str(report_dir),
    ]
    cmd.extend(extra_args)
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")

def check_latest_smoke_skip_reason_contract() -> str | None:
    expected_reason_set = {
        LATEST_SMOKE_SKIP_REASON_FAST_FAIL_PATH,
        LATEST_SMOKE_SKIP_REASON_FLAG_DISABLED,
        LATEST_SMOKE_SKIP_REASON_PENDING_FAILURE_SUMMARY_REGENERATION,
        LATEST_SMOKE_SKIP_REASON_CI_GATE_RESULT_STATUS_NOT_PASS,
    }
    if set(LATEST_SMOKE_SKIP_REASON_EXPECTED) != expected_reason_set:
        return (
            "latest smoke skip reason contract mismatch: "
            f"expected={sorted(expected_reason_set)} observed={sorted(set(LATEST_SMOKE_SKIP_REASON_EXPECTED))}"
        )
    return None


def main() -> int:
    reason_contract_issue = check_latest_smoke_skip_reason_contract()
    if reason_contract_issue is not None:
        return fail(reason_contract_issue)

    with tempfile.TemporaryDirectory(prefix="ci_gate_report_index_latest_smoke_selftest_") as td:
        root = Path(td)

        missing_dir = root / "missing"
        missing_proc = run_smoke(missing_dir)
        if missing_proc.returncode != 0:
            return fail(f"missing dir case must pass: out={missing_proc.stdout} err={missing_proc.stderr}")
        if "skip report_dir_missing=" not in missing_proc.stdout:
            return fail(f"missing dir skip marker mismatch: out={missing_proc.stdout}")

        empty_dir = root / "empty"
        empty_dir.mkdir(parents=True, exist_ok=True)
        empty_proc = run_smoke(empty_dir)
        if empty_proc.returncode != 0:
            return fail(f"empty dir case must pass: out={empty_proc.stdout} err={empty_proc.stderr}")
        if "skip index_missing" not in empty_proc.stdout:
            return fail(f"empty dir skip marker mismatch: out={empty_proc.stdout}")

        ok_index = build_index_case(root, "latest_smoke_ok", sanity_profile="full")
        ok_proc = run_smoke(ok_index.parent)
        if ok_proc.returncode != 0:
            return fail(f"ok case failed: out={ok_proc.stdout} err={ok_proc.stderr}")
        if "[ci-gate-report-index-latest-smoke-check] ok " not in ok_proc.stdout:
            return fail(f"ok marker missing: out={ok_proc.stdout}")
        if "profile=full" not in ok_proc.stdout:
            return fail(f"ok profile marker mismatch: out={ok_proc.stdout}")

        prefixed_case_root = root / "prefixed_case"
        prefixed_index = build_index_case(prefixed_case_root, "latest_smoke_prefixed", sanity_profile="core_lang")
        prefixed_doc = json.loads(prefixed_index.read_text(encoding="utf-8"))
        prefixed_name = "zzprefix_ci_gate_report_index.detjson"
        prefixed_index_path = prefixed_index.parent / prefixed_name
        prefixed_doc["report_prefix"] = "zzprefix"
        prefixed_doc["report_prefix_source"] = "arg"
        prefixed_index_path.write_text(json.dumps(prefixed_doc, ensure_ascii=False, indent=2), encoding="utf-8")

        reports = prefixed_doc.get("reports", {})
        if not isinstance(reports, dict):
            return fail("prefixed case reports missing")
        result_path = Path(str(reports.get("ci_gate_result_json", "")).strip())
        if not result_path.exists():
            return fail("prefixed case result path missing")
        result_doc = json.loads(result_path.read_text(encoding="utf-8"))
        if not isinstance(result_doc, dict):
            return fail("prefixed case result doc invalid")
        result_doc["gate_index_path"] = str(prefixed_index_path)
        result_path.write_text(json.dumps(result_doc, ensure_ascii=False, indent=2), encoding="utf-8")

        prefix_match_proc = run_smoke(prefixed_index.parent, "--prefix", "zzprefix")
        if prefix_match_proc.returncode != 0:
            return fail(f"prefix match case failed: out={prefix_match_proc.stdout} err={prefix_match_proc.stderr}")
        if prefixed_name not in prefix_match_proc.stdout.replace("\\", "/"):
            return fail(f"prefix match selected index mismatch: out={prefix_match_proc.stdout}")
        if "profile=core_lang" not in prefix_match_proc.stdout:
            return fail(f"prefix match profile marker mismatch: out={prefix_match_proc.stdout}")

        prefix_miss_proc = run_smoke(prefixed_index.parent, "--prefix", "missing_prefix")
        if prefix_miss_proc.returncode != 0:
            return fail(f"prefix miss case must pass: out={prefix_miss_proc.stdout} err={prefix_miss_proc.stderr}")
        if "skip index_missing" not in prefix_miss_proc.stdout:
            return fail(f"prefix miss skip marker mismatch: out={prefix_miss_proc.stdout}")

    print("[ci-gate-report-index-latest-smoke-check-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
