#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from ci_check_error_codes import SUMMARY_REPORT_CODES as CODES


def fail(msg: str) -> int:
    print(f"[ci-gate-summary-report-selftest] fail: {msg}")
    return 1


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text.rstrip() + "\n", encoding="utf-8")


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def run_check(summary: Path, index: Path, require_pass: bool = False) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_ci_gate_summary_report_check.py",
        "--summary",
        str(summary),
        "--index",
        str(index),
    ]
    if require_pass:
        cmd.append("--require-pass")
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def build_pass_case(root: Path, name: str) -> tuple[Path, Path]:
    case_dir = root / name
    case_dir.mkdir(parents=True, exist_ok=True)
    summary_path = case_dir / "ci_gate_summary.txt"
    index_path = case_dir / "ci_gate_report_index.detjson"
    summary_line = case_dir / "ci_gate_summary_line.txt"
    result = case_dir / "ci_gate_result.detjson"
    badge = case_dir / "ci_gate_badge.detjson"
    brief = case_dir / "ci_fail_brief.txt"
    triage = case_dir / "ci_fail_triage.detjson"
    age3_status = case_dir / "age3_close_status.detjson"
    age4_status = case_dir / "age4_close_report.detjson"
    age5_status = case_dir / "age5_close_report.detjson"
    phase3_cleanup = case_dir / "seamgrim_phase3_cleanup_gate_report.detjson"
    fixed64_threeway_report = case_dir / "fixed64_cross_platform_threeway_gate.detjson"
    for path in (
        summary_line,
        result,
        badge,
        brief,
        triage,
        age3_status,
        age4_status,
        age5_status,
        phase3_cleanup,
        fixed64_threeway_report,
    ):
        write_text(path, "{}")
    write_json(
        index_path,
        {
            "schema": "ddn.ci.aggregate_gate.index.v1",
            "reports": {
                "summary_line": str(summary_line),
                "ci_gate_result_json": str(result),
                "ci_gate_badge_json": str(badge),
                "ci_fail_triage_json": str(triage),
                "age3_close_status_json": str(age3_status),
                "age4_close": str(age4_status),
                "age5_close": str(age5_status),
                "seamgrim_phase3_cleanup": str(phase3_cleanup),
                "fixed64_threeway_gate": str(fixed64_threeway_report),
            },
        },
    )
    lines = [
        "[ci-gate-summary] PASS",
        "[ci-gate-summary] failed_steps=(none)",
        f"[ci-gate-summary] report_index={index_path}",
        f"[ci-gate-summary] summary_line={summary_line}",
        f"[ci-gate-summary] ci_gate_result={result}",
        f"[ci-gate-summary] ci_gate_badge={badge}",
        f"[ci-gate-summary] ci_fail_brief_hint={brief}",
        "[ci-gate-summary] ci_fail_brief_exists=1",
        f"[ci-gate-summary] ci_fail_triage_hint={triage}",
        "[ci-gate-summary] ci_fail_triage_exists=1",
        f"[ci-gate-summary] age3_status={age3_status}",
        f"[ci-gate-summary] age4_status={age4_status}",
        f"[ci-gate-summary] age5_status={age5_status}",
        f"[ci-gate-summary] seamgrim_phase3_cleanup={phase3_cleanup}",
        f"[ci-gate-summary] fixed64_threeway_report={fixed64_threeway_report}",
        "[ci-gate-summary] fixed64_threeway_status=pending_darwin",
        "[ci-gate-summary] fixed64_threeway_ok=1",
    ]
    write_text(summary_path, "\n".join(lines))
    return summary_path, index_path


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="ci_gate_summary_report_selftest_") as tmp:
        root = Path(tmp)

        summary_ok, index_ok = build_pass_case(root, "ok")
        proc_ok = run_check(summary_ok, index_ok, require_pass=True)
        if proc_ok.returncode != 0:
            return fail(f"ok case failed: out={proc_ok.stdout} err={proc_ok.stderr}")

        summary_missing_key, index_missing_key = build_pass_case(root, "missing_key")
        text_missing_key = summary_missing_key.read_text(encoding="utf-8")
        text_missing_key = text_missing_key.replace("[ci-gate-summary] ci_gate_badge=", "[ci-gate-summary] REMOVED=")
        write_text(summary_missing_key, text_missing_key)
        proc_missing_key = run_check(summary_missing_key, index_missing_key, require_pass=True)
        if proc_missing_key.returncode == 0:
            return fail("missing key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_missing_key.stderr:
            return fail(f"missing key code mismatch: err={proc_missing_key.stderr}")

        summary_bad_index, index_bad_index = build_pass_case(root, "bad_index")
        bad_text = summary_bad_index.read_text(encoding="utf-8").replace(
            f"[ci-gate-summary] report_index={index_bad_index}",
            "[ci-gate-summary] report_index=wrong.index.detjson",
        )
        write_text(summary_bad_index, bad_text)
        proc_bad_index = run_check(summary_bad_index, index_bad_index, require_pass=True)
        if proc_bad_index.returncode == 0:
            return fail("report_index mismatch case must fail")
        if f"fail code={CODES['REPORT_INDEX_MISMATCH']}" not in proc_bad_index.stderr:
            return fail(f"report_index code mismatch: err={proc_bad_index.stderr}")

        summary_bad_brief, index_bad_brief = build_pass_case(root, "bad_brief")
        bad_brief_text = summary_bad_brief.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_fail_brief_exists=1",
            "[ci-gate-summary] ci_fail_brief_exists=0",
        )
        write_text(summary_bad_brief, bad_brief_text)
        proc_bad_brief = run_check(summary_bad_brief, index_bad_brief, require_pass=True)
        if proc_bad_brief.returncode == 0:
            return fail("brief exists mismatch case must fail")
        if f"fail code={CODES['BRIEF_EXISTS_MISMATCH']}" not in proc_bad_brief.stderr:
            return fail(f"brief exists code mismatch: err={proc_bad_brief.stderr}")

    print("[ci-gate-summary-report-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
