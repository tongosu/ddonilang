#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from ci_check_error_codes import EMIT_ARTIFACTS_CODES as CODES

def fail(msg: str) -> int:
    print(f"[ci-emit-artifacts-check-selftest] fail: {msg}")
    return 1


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text.rstrip() + "\n", encoding="utf-8")


def run_check(report_dir: Path, *extra: str) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_ci_emit_artifacts_check.py",
        "--report-dir",
        str(report_dir),
        *extra,
    ]
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def build_case(
    report_dir: Path,
    prefix: str,
    status: str,
    with_brief: bool,
    with_triage: bool,
    broken_norm: bool = False,
    broken_brief: bool = False,
    broken_triage_final: bool = False,
    broken_artifact_ref: bool = False,
    broken_summary: bool = False,
    broken_verify_issue: bool = False,
) -> None:
    index_path = report_dir / f"{prefix}.ci_gate_report_index.detjson"
    result_path = report_dir / f"{prefix}.ci_gate_result.detjson"
    summary_path = report_dir / f"{prefix}.ci_gate_summary.txt"
    summary_line_path = report_dir / f"{prefix}.ci_gate_summary_line.txt"
    brief_path = report_dir / f"{prefix}.ci_fail_brief.txt"
    triage_path = report_dir / f"{prefix}.ci_fail_triage.detjson"

    failed_steps_count = 0 if status == "pass" else 1
    sample_step_id = "sample_fail"
    sample_stdout_path = report_dir / f"{prefix}.ci_gate_step_{sample_step_id}.stdout.txt"
    sample_stderr_path = report_dir / f"{prefix}.ci_gate_step_{sample_step_id}.stderr.txt"
    if failed_steps_count > 0:
        write_text(sample_stdout_path, "[sample-fail] stdout")
        write_text(sample_stderr_path, "[sample-fail] stderr")
    if status == "pass" or broken_summary:
        write_text(summary_path, "[ci-gate-summary] PASS\n[ci-gate-summary] failed_steps=(none)")
    else:
        write_text(
            summary_path,
            "\n".join(
                [
                    "[ci-gate-summary] FAIL",
                    f"[ci-gate-summary] failed_steps={sample_step_id}",
                    f"[ci-gate-summary] failed_step_detail={sample_step_id} rc=1 cmd=python tests/run_sample_fail.py",
                    f"[ci-gate-summary] failed_step_logs={sample_step_id} stdout={sample_stdout_path} stderr={sample_stderr_path}",
                ]
            ),
        )
    summary_line = (
        f"ci_gate_result_status={status} ok={1 if status == 'pass' else 0} "
        f"overall_ok={1 if status == 'pass' else 0} failed_steps={failed_steps_count} "
        f"aggregate_status={status} reason=-"
    )
    write_text(summary_line_path, summary_line)
    write_json(
        result_path,
        {
            "schema": "ddn.ci.gate_result.v1",
            "status": status,
            "ok": status == "pass",
            "reason": "-",
            "failed_steps": failed_steps_count,
        },
    )
    if with_brief:
        brief_status = "fail" if broken_brief else status
        brief_reason = "bad_reason" if broken_brief else "-"
        brief_failed_steps = 99 if broken_brief else failed_steps_count
        brief_final_line = "-" if broken_brief else summary_line
        write_text(
            brief_path,
            f'status={brief_status} reason="{brief_reason}" failed_steps_count={brief_failed_steps} failed_steps="-" top_step=- top_message="-" final_line="{brief_final_line}"',
        )
    if with_triage:
        triage_final_line = "-" if broken_triage_final else summary_line
        failed_steps_rows: list[dict[str, object]] = []
        if failed_steps_count > 0:
            failed_steps_rows.append(
                {
                    "step_id": sample_step_id,
                    "message": "sample failure",
                    "stdout_log_path": str(sample_stdout_path),
                    "stdout_log_path_norm": str(sample_stdout_path).replace("\\", "/"),
                    "stderr_log_path": str(sample_stderr_path),
                    "stderr_log_path_norm": str(sample_stderr_path).replace("\\", "/"),
                }
            )
        triage_summary_line_path = summary_line_path
        triage_summary_line_norm = str(summary_line_path).replace("\\", "/")
        if broken_artifact_ref:
            alt_summary_line_path = report_dir / f"{prefix}.alt_summary_line.txt"
            write_text(alt_summary_line_path, "ci_gate_result_status=pass ok=1 overall_ok=1 failed_steps=0")
            triage_summary_line_path = alt_summary_line_path
            triage_summary_line_norm = str(alt_summary_line_path).replace("\\", "/")
        write_json(
            triage_path,
            {
                "schema": "ddn.ci.fail_triage.v1",
                "generated_at_utc": "2026-02-21T00:00:00+00:00",
                "status": status,
                "reason": "-",
                "report_prefix": prefix,
                "final_line": triage_final_line,
                "summary_verify_ok": False if broken_verify_issue else True,
                "summary_verify_issues": ["bad_issue_token"] if broken_verify_issue else [],
                "summary_verify_issues_count": 1 if broken_verify_issue else 0,
                "summary_verify_top_issue": "bad_issue_token" if broken_verify_issue else "-",
                "failed_steps": failed_steps_rows,
                "failed_steps_count": failed_steps_count,
                "aggregate_digest": [],
                "aggregate_digest_count": 0,
                "summary_report_path_hint": str(summary_path),
                "summary_report_path_hint_norm": (
                    "BROKEN/PATH" if broken_norm else str(summary_path).replace("\\", "/")
                ),
                "artifacts": {
                    "summary": {
                        "path": str(summary_path),
                        "path_norm": "BROKEN/PATH" if broken_norm else str(summary_path).replace("\\", "/"),
                        "exists": True,
                    },
                    "summary_line": {
                        "path": str(triage_summary_line_path),
                        "path_norm": triage_summary_line_norm,
                        "exists": True,
                    },
                    "ci_gate_result_json": {
                        "path": str(result_path),
                        "path_norm": str(result_path).replace("\\", "/"),
                        "exists": True,
                    },
                    "ci_fail_brief_txt": {
                        "path": str(brief_path),
                        "path_norm": str(brief_path).replace("\\", "/"),
                        "exists": bool(with_brief),
                    },
                    "ci_fail_triage_json": {
                        "path": str(triage_path),
                        "path_norm": str(triage_path).replace("\\", "/"),
                        "exists": True,
                    },
                },
            },
        )

    write_json(
        index_path,
        {
            "schema": "ddn.ci.aggregate_gate.index.v1",
            "report_prefix": prefix,
            "reports": {
                "summary": str(summary_path),
                "summary_line": str(summary_line_path),
                "ci_gate_result_json": str(result_path),
                "ci_fail_brief_txt": str(brief_path),
                "ci_fail_triage_json": str(triage_path),
            },
            "steps": [],
        },
    )


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="ci_emit_artifacts_selftest_") as tmp:
        report_dir = Path(tmp)

        build_case(report_dir, "okcase", status="pass", with_brief=True, with_triage=True)
        proc_ok = run_check(report_dir, "--prefix", "okcase", "--require-brief", "--require-triage")
        if proc_ok.returncode != 0:
            return fail(f"okcase failed rc={proc_ok.returncode} out={proc_ok.stdout} err={proc_ok.stderr}")

        build_case(report_dir, "failcase", status="fail", with_brief=True, with_triage=True)
        proc_fail = run_check(report_dir, "--prefix", "failcase", "--require-brief", "--require-triage")
        if proc_fail.returncode != 0:
            return fail(f"failcase failed rc={proc_fail.returncode} out={proc_fail.stdout} err={proc_fail.stderr}")

        build_case(report_dir, "missbrief", status="pass", with_brief=False, with_triage=True)
        proc_miss_brief = run_check(report_dir, "--prefix", "missbrief", "--require-brief", "--require-triage")
        if proc_miss_brief.returncode == 0:
            return fail("missbrief must fail")
        if f"fail code={CODES['BRIEF_REQUIRED_MISSING']}" not in proc_miss_brief.stderr:
            return fail(f"missbrief error code missing: err={proc_miss_brief.stderr}")

        build_case(report_dir, "misstriage", status="pass", with_brief=True, with_triage=False)
        proc_miss_triage = run_check(report_dir, "--prefix", "misstriage", "--require-brief", "--require-triage")
        if proc_miss_triage.returncode == 0:
            return fail("misstriage must fail")
        if f"fail code={CODES['TRIAGE_REQUIRED_MISSING']}" not in proc_miss_triage.stderr:
            return fail(f"misstriage error code missing: err={proc_miss_triage.stderr}")

        build_case(report_dir, "badnorm", status="pass", with_brief=True, with_triage=True, broken_norm=True)
        proc_bad_norm = run_check(report_dir, "--prefix", "badnorm", "--require-brief", "--require-triage")
        if proc_bad_norm.returncode == 0:
            return fail("badnorm must fail")

        build_case(report_dir, "badbrief", status="pass", with_brief=True, with_triage=True, broken_brief=True)
        proc_bad_brief = run_check(report_dir, "--prefix", "badbrief", "--require-brief", "--require-triage")
        if proc_bad_brief.returncode == 0:
            return fail("badbrief must fail")

        build_case(
            report_dir,
            "badtriagefinal",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_triage_final=True,
        )
        proc_bad_triage_final = run_check(
            report_dir,
            "--prefix",
            "badtriagefinal",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_triage_final.returncode == 0:
            return fail("badtriagefinal must fail")

        build_case(
            report_dir,
            "badartifactref",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_artifact_ref=True,
        )
        proc_bad_artifact_ref = run_check(
            report_dir,
            "--prefix",
            "badartifactref",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_artifact_ref.returncode == 0:
            return fail("badartifactref must fail")

        build_case(
            report_dir,
            "badsummary",
            status="fail",
            with_brief=True,
            with_triage=True,
            broken_summary=True,
        )
        proc_bad_summary = run_check(
            report_dir,
            "--prefix",
            "badsummary",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_summary.returncode == 0:
            return fail("badsummary must fail")

        build_case(
            report_dir,
            "badverifyissue",
            status="pass",
            with_brief=True,
            with_triage=True,
            broken_verify_issue=True,
        )
        proc_bad_verify_issue = run_check(
            report_dir,
            "--prefix",
            "badverifyissue",
            "--require-brief",
            "--require-triage",
        )
        if proc_bad_verify_issue.returncode == 0:
            return fail("badverifyissue must fail")

    print("[ci-emit-artifacts-check-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
