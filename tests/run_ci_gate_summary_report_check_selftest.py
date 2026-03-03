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
    ci_sanity_gate = case_dir / "ci_sanity_gate.detjson"
    ci_sync_readiness = case_dir / "ci_sync_readiness.detjson"
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
        ci_sanity_gate,
        ci_sync_readiness,
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
                "ci_sanity_gate": str(ci_sanity_gate),
                "ci_sync_readiness": str(ci_sync_readiness),
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
        "[ci-gate-summary] ci_pack_golden_overlay_compare_selftest_ok=1",
        "[ci-gate-summary] ci_pack_golden_overlay_session_selftest_ok=1",
        f"[ci-gate-summary] age3_status={age3_status}",
        f"[ci-gate-summary] age4_status={age4_status}",
        f"[ci-gate-summary] age5_status={age5_status}",
        f"[ci-gate-summary] seamgrim_phase3_cleanup={phase3_cleanup}",
        f"[ci-gate-summary] ci_sanity_gate_report={ci_sanity_gate}",
        "[ci-gate-summary] ci_sanity_gate_status=pass",
        "[ci-gate-summary] ci_sanity_gate_ok=1",
        "[ci-gate-summary] ci_sanity_gate_code=OK",
        "[ci-gate-summary] ci_sanity_gate_step=all",
        "[ci-gate-summary] ci_sanity_gate_profile=full",
        "[ci-gate-summary] ci_sanity_gate_msg=-",
        "[ci-gate-summary] ci_sanity_gate_step_count=14",
        "[ci-gate-summary] ci_sanity_gate_failed_steps=0",
        "[ci-gate-summary] ci_sanity_seamgrim_interface_boundary_ok=1",
        "[ci-gate-summary] ci_sanity_overlay_session_wired_consistency_ok=1",
        "[ci-gate-summary] ci_sanity_overlay_session_diag_parity_ok=1",
        "[ci-gate-summary] ci_sanity_overlay_compare_diag_parity_ok=1",
        f"[ci-gate-summary] ci_sync_readiness_report={ci_sync_readiness}",
        "[ci-gate-summary] ci_sync_readiness_status=pass",
        "[ci-gate-summary] ci_sync_readiness_ok=1",
        "[ci-gate-summary] ci_sync_readiness_code=OK",
        "[ci-gate-summary] ci_sync_readiness_step=all",
        "[ci-gate-summary] ci_sync_readiness_sanity_profile=full",
        "[ci-gate-summary] ci_sync_readiness_msg=-",
        "[ci-gate-summary] ci_sync_readiness_step_count=1",
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

        summary_missing_sanity, index_missing_sanity = build_pass_case(root, "missing_sanity")
        text_missing_sanity = summary_missing_sanity.read_text(encoding="utf-8")
        text_missing_sanity = text_missing_sanity.replace(
            "[ci-gate-summary] ci_sanity_gate_status=pass",
            "[ci-gate-summary] REMOVED_SANITY_STATUS=pass",
        )
        write_text(summary_missing_sanity, text_missing_sanity)
        proc_missing_sanity = run_check(summary_missing_sanity, index_missing_sanity, require_pass=True)
        if proc_missing_sanity.returncode == 0:
            return fail("missing ci_sanity key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_missing_sanity.stderr:
            return fail(f"missing ci_sanity key code mismatch: err={proc_missing_sanity.stderr}")
        summary_missing_sync, index_missing_sync = build_pass_case(root, "missing_sync")
        text_missing_sync = summary_missing_sync.read_text(encoding="utf-8")
        text_missing_sync = text_missing_sync.replace(
            "[ci-gate-summary] ci_sync_readiness_status=pass",
            "[ci-gate-summary] REMOVED_SYNC_STATUS=pass",
        )
        write_text(summary_missing_sync, text_missing_sync)
        proc_missing_sync = run_check(summary_missing_sync, index_missing_sync, require_pass=True)
        if proc_missing_sync.returncode == 0:
            return fail("missing ci_sync key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_missing_sync.stderr:
            return fail(f"missing ci_sync key code mismatch: err={proc_missing_sync.stderr}")

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

        summary_bad_sanity_steps, index_bad_sanity_steps = build_pass_case(root, "bad_sanity_steps")
        bad_sanity_steps_text = summary_bad_sanity_steps.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_sanity_gate_step_count=14",
            "[ci-gate-summary] ci_sanity_gate_step_count=1",
        )
        write_text(summary_bad_sanity_steps, bad_sanity_steps_text)
        proc_bad_sanity_steps = run_check(summary_bad_sanity_steps, index_bad_sanity_steps, require_pass=True)
        if proc_bad_sanity_steps.returncode == 0:
            return fail("low ci_sanity_gate_step_count case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_steps.stderr:
            return fail(f"low ci_sanity_gate_step_count code mismatch: err={proc_bad_sanity_steps.stderr}")
        summary_bad_sync_steps, index_bad_sync_steps = build_pass_case(root, "bad_sync_steps")
        bad_sync_steps_text = summary_bad_sync_steps.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_sync_readiness_step_count=1",
            "[ci-gate-summary] ci_sync_readiness_step_count=0",
        )
        write_text(summary_bad_sync_steps, bad_sync_steps_text)
        proc_bad_sync_steps = run_check(summary_bad_sync_steps, index_bad_sync_steps, require_pass=True)
        if proc_bad_sync_steps.returncode == 0:
            return fail("low ci_sync_readiness_step_count case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sync_steps.stderr:
            return fail(f"low ci_sync_readiness_step_count code mismatch: err={proc_bad_sync_steps.stderr}")
        summary_bad_sanity_parity, index_bad_sanity_parity = build_pass_case(root, "bad_sanity_parity")
        bad_sanity_parity_text = summary_bad_sanity_parity.read_text(encoding="utf-8").replace(
            "[ci-gate-summary] ci_sanity_overlay_compare_diag_parity_ok=1",
            "[ci-gate-summary] ci_sanity_overlay_compare_diag_parity_ok=0",
        )
        write_text(summary_bad_sanity_parity, bad_sanity_parity_text)
        proc_bad_sanity_parity = run_check(summary_bad_sanity_parity, index_bad_sanity_parity, require_pass=True)
        if proc_bad_sanity_parity.returncode == 0:
            return fail("ci_sanity overlay compare parity key case must fail")
        if f"fail code={CODES['PASS_KEY_MISSING']}" not in proc_bad_sanity_parity.stderr:
            return fail(f"ci_sanity overlay compare parity code mismatch: err={proc_bad_sanity_parity.stderr}")

    print("[ci-gate-summary-report-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
