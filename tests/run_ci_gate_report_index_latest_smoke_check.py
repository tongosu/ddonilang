#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path


def default_report_dir() -> str:
    preferred = Path("I:/home/urihanl/ddn/codex/build/reports")
    if os.name == "nt":
        try:
            preferred.mkdir(parents=True, exist_ok=True)
        except OSError:
            pass
        return str(preferred)
    return "build/reports"


def select_latest_index(report_dir: Path, pattern: str, prefix: str) -> Path | None:
    candidates = sorted(
        report_dir.glob(pattern),
        key=lambda p: (p.stat().st_mtime_ns, str(p)),
        reverse=True,
    )
    prefix_text = str(prefix).strip()
    for path in candidates:
        if not prefix_text:
            return path
        if path.name.startswith(prefix_text):
            return path
        if path.stem.startswith(prefix_text):
            return path
    return None


def resolve_sanity_profile(index_path: Path) -> str:
    try:
        doc = json.loads(index_path.read_text(encoding="utf-8"))
    except Exception:
        return "full"
    if not isinstance(doc, dict):
        return "full"
    profile = str(doc.get("ci_sanity_profile", "")).strip()
    if profile in {"core_lang", "full", "seamgrim"}:
        return profile
    return "full"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run run_ci_gate_report_index_check.py against latest aggregate index"
    )
    parser.add_argument("--report-dir", default=default_report_dir(), help="report directory")
    parser.add_argument(
        "--index-pattern",
        default="*ci_gate_report_index.detjson",
        help="index file glob pattern",
    )
    parser.add_argument("--prefix", default="", help="optional report prefix filter")
    parser.add_argument(
        "--no-enforce-profile-step-contract",
        action="store_true",
        help="disable --enforce-profile-step-contract",
    )
    args = parser.parse_args()

    report_dir = Path(args.report_dir)
    if not report_dir.exists():
        print(f"[ci-gate-report-index-latest-smoke-check] skip report_dir_missing={report_dir}")
        return 0

    index_path = select_latest_index(report_dir, str(args.index_pattern), str(args.prefix))
    if index_path is None:
        print(
            "[ci-gate-report-index-latest-smoke-check] "
            f"skip index_missing report_dir={report_dir} pattern={args.index_pattern}"
        )
        return 0

    sanity_profile = resolve_sanity_profile(index_path)
    cmd = [
        sys.executable,
        "tests/run_ci_gate_report_index_check.py",
        "--index",
        str(index_path),
        "--sanity-profile",
        sanity_profile,
    ]
    if not bool(args.no_enforce_profile_step_contract):
        cmd.append("--enforce-profile-step-contract")

    proc = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")
    if proc.returncode != 0:
        print(
            "[ci-gate-report-index-latest-smoke-check] "
            f"fail index={index_path} profile={sanity_profile} rc={proc.returncode}"
        )
        stdout_text = proc.stdout.strip()
        stderr_text = proc.stderr.strip()
        if stdout_text:
            print(stdout_text)
        if stderr_text:
            print(stderr_text)
        return int(proc.returncode)

    print(
        "[ci-gate-report-index-latest-smoke-check] "
        f"ok index={index_path} profile={sanity_profile}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
