#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from ci_check_error_codes import SUMMARY_VERIFY_CODES


def fail(msg: str) -> int:
    print(f"[ci-final-emitter-check] fail: {msg}")
    return 1


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text.rstrip() + "\n", encoding="utf-8")


def run_emit(report_dir: Path, *extra: str) -> subprocess.CompletedProcess[str]:
    cmd = [sys.executable, "tools/scripts/emit_ci_final_line.py", "--report-dir", str(report_dir), *extra]
    return subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")


def build_case(
    report_dir: Path,
    prefix: str,
    status: str,
    reason: str,
    with_digest: bool,
    broken_summary: bool = False,
) -> None:
    index_path = report_dir / f"{prefix}.ci_gate_report_index.detjson"
    summary_path = report_dir / f"{prefix}.ci_gate_summary_line.txt"
    summary_report_path = report_dir / f"{prefix}.ci_gate_summary.txt"
    result_path = report_dir / f"{prefix}.ci_gate_result.detjson"
    aggregate_path = report_dir / f"{prefix}.ci_aggregate_report.detjson"
    seamgrim_stdout = report_dir / f"{prefix}.seamgrim.stdout.txt"
    seamgrim_stderr = report_dir / f"{prefix}.seamgrim.stderr.txt"
    oi_stdout = report_dir / f"{prefix}.oi.stdout.txt"
    oi_stderr = report_dir / f"{prefix}.oi.stderr.txt"
    write_text(seamgrim_stdout, "sg out 1\nsg out 2\nsg out 3")
    write_text(seamgrim_stderr, "sg err 1\nsg err 2\nsg err 3")
    write_text(oi_stdout, "oi out 1\noi out 2\noi out 3")
    write_text(oi_stderr, "oi err 1\noi err 2\noi err 3")
    write_text(
        summary_path,
        f"ci_gate_result_status={status} ok={1 if status == 'pass' else 0} "
        f"overall_ok={1 if status == 'pass' else 0} failed_steps={0 if status == 'pass' else 2} "
        f"aggregate_status={status} reason={reason}",
    )
    if status == "pass":
        write_text(
            summary_report_path,
            "\n".join(
                [
                    "[ci-gate-summary] PASS",
                    "[ci-gate-summary] failed_steps=(none)",
                    f"[ci-gate-summary] report_index={index_path}",
                    f"[ci-gate-summary] summary_line={summary_path}",
                    f"[ci-gate-summary] ci_gate_result={result_path}",
                    f"[ci-gate-summary] ci_gate_badge={report_dir / f'{prefix}.ci_gate_badge.detjson'}",
                    f"[ci-gate-summary] ci_fail_brief_hint={report_dir / f'{prefix}.ci_fail_brief.txt'}",
                    "[ci-gate-summary] ci_fail_brief_exists=0",
                    f"[ci-gate-summary] age3_status={report_dir / f'{prefix}.age3_close_status.detjson'}",
                    f"[ci-gate-summary] age4_status={report_dir / f'{prefix}.age4_close_report.detjson'}",
                ]
            ),
        )
    else:
        fail_lines = [
            "[ci-gate-summary] FAIL",
            "[ci-gate-summary] failed_steps=seamgrim_ci_gate,oi405_406_close",
            "[ci-gate-summary] failed_step_detail=seamgrim_ci_gate rc=1 cmd=python tests/run_seamgrim_ci_gate.py",
            f"[ci-gate-summary] failed_step_logs=seamgrim_ci_gate stdout={seamgrim_stdout} stderr={seamgrim_stderr}",
            "[ci-gate-summary] failed_step_detail=oi405_406_close rc=1 cmd=python tests/run_oi405_406_close.py",
            f"[ci-gate-summary] failed_step_logs=oi405_406_close stdout={oi_stdout} stderr={oi_stderr}",
            f"[ci-gate-summary] report_index={index_path}",
            f"[ci-gate-summary] summary_line={summary_path}",
            f"[ci-gate-summary] ci_gate_result={result_path}",
        ]
        if broken_summary:
            fail_lines[1] = "[ci-gate-summary] failed_steps=unknown_step_only"
            fail_lines = [line for line in fail_lines if "failed_step_detail=oi405_406_close" not in line]
        write_text(summary_report_path, "\n".join(fail_lines))
    write_json(
        result_path,
        {
            "schema": "ddn.ci.gate_result.v1",
            "status": status,
            "ok": status == "pass",
            "reason": reason,
            "failed_steps": 0 if status == "pass" else 2,
            "aggregate_status": status,
        },
    )
    digest = ["step failed: seamgrim_ci_gate", "pack failed: dotbogi_write_forbidden"] if with_digest else []
    write_json(
        aggregate_path,
        {
            "schema": "ddn.ci.aggregate_report.v1",
            "overall_ok": status == "pass",
            "failure_digest": digest,
        },
    )
    write_json(
        index_path,
        {
            "schema": "ddn.ci.aggregate_gate.index.v1",
            "report_prefix": prefix,
            "reports": {
                "summary": str(summary_report_path),
                "summary_line": str(summary_path),
                "ci_gate_result_json": str(result_path),
                "aggregate": str(aggregate_path),
                "age4_close": str(report_dir / f"{prefix}.age4_close_report.detjson"),
            },
            "steps": [
                {
                    "name": "oi405_406_close",
                    "ok": status == "pass",
                    "stdout_log_path": str(oi_stdout),
                    "stderr_log_path": str(oi_stderr),
                },
                {
                    "name": "seamgrim_ci_gate",
                    "ok": status == "pass",
                    "stdout_log_path": str(seamgrim_stdout),
                    "stderr_log_path": str(seamgrim_stderr),
                },
            ],
        },
    )


def ensure_contains(text: str, needle: str) -> bool:
    return needle in text


def main() -> int:
    summary_verify_codes = set(SUMMARY_VERIFY_CODES.values())
    with tempfile.TemporaryDirectory(prefix="ci_final_emit_check_") as tmp:
        report_dir = Path(tmp)
        brief_tpl = report_dir / "__PREFIX__.ci_fail_brief.txt"
        triage_tpl = report_dir / "__PREFIX__.ci_fail_triage.detjson"

        build_case(report_dir, "passcase", status="pass", reason="-", with_digest=False)
        proc_pass = run_emit(
            report_dir,
            "--prefix",
            "passcase",
            "--print-artifacts",
            "--print-failure-digest",
            "5",
            "--failure-brief-out",
            str(brief_tpl),
            "--triage-json-out",
            str(triage_tpl),
            "--require-final-line",
        )
        if proc_pass.returncode != 0:
            return fail(f"passcase returncode={proc_pass.returncode}")
        if not ensure_contains(proc_pass.stdout, "[ci-final] ci_gate_result_status=pass"):
            return fail("passcase final line missing")
        if not ensure_contains(proc_pass.stdout, "[ci-artifact] key=summary exists=1"):
            return fail("passcase summary artifact line missing")
        if ensure_contains(proc_pass.stdout, "[ci-fail]"):
            return fail("passcase must not print ci-fail block")
        pass_brief = report_dir / "passcase.ci_fail_brief.txt"
        if not pass_brief.exists():
            return fail("passcase brief file missing")
        pass_brief_line = pass_brief.read_text(encoding="utf-8").strip()
        if not ensure_contains(pass_brief_line, "status=pass"):
            return fail("passcase brief status missing")
        pass_triage = report_dir / "passcase.ci_fail_triage.detjson"
        if not pass_triage.exists():
            return fail("passcase triage file missing")
        pass_triage_doc = json.loads(pass_triage.read_text(encoding="utf-8"))
        if str(pass_triage_doc.get("schema", "")) != "ddn.ci.fail_triage.v1":
            return fail("passcase triage schema mismatch")
        if str(pass_triage_doc.get("status", "")) != "pass":
            return fail("passcase triage status mismatch")
        if int(pass_triage_doc.get("summary_verify_issues_count", -1)) != 0:
            return fail("passcase triage summary_verify_issues_count mismatch")
        if str(pass_triage_doc.get("summary_verify_top_issue", "")).strip() != "-":
            return fail("passcase triage summary_verify_top_issue mismatch")
        pass_artifacts = pass_triage_doc.get("artifacts")
        if not isinstance(pass_artifacts, dict) or "summary" not in pass_artifacts:
            return fail("passcase triage artifacts missing summary")
        for key in ("ci_fail_brief_txt", "ci_fail_triage_json"):
            row = pass_artifacts.get(key)
            if not isinstance(row, dict):
                return fail(f"passcase triage artifacts missing {key}")
            if not bool(row.get("exists", False)):
                return fail(f"passcase triage artifacts {key} exists mismatch")

        build_case(report_dir, "failcase", status="fail", reason="aggregate_failed", with_digest=True)
        proc_fail = run_emit(
            report_dir,
            "--prefix",
            "failcase",
            "--print-artifacts",
            "--print-failure-digest",
            "5",
            "--print-failure-tail-lines",
            "2",
            "--failure-brief-out",
            str(brief_tpl),
            "--triage-json-out",
            str(triage_tpl),
            "--require-final-line",
        )
        if proc_fail.returncode != 0:
            return fail(f"failcase returncode={proc_fail.returncode}")
        if not ensure_contains(proc_fail.stdout, "[ci-final] ci_gate_result_status=fail"):
            return fail("failcase final line missing")
        if not ensure_contains(proc_fail.stdout, "[ci-artifact] key=summary exists=1"):
            return fail("failcase summary artifact line missing")
        if not ensure_contains(proc_fail.stdout, "[ci-fail] status=fail"):
            return fail("failcase ci-fail status missing")
        if not ensure_contains(proc_fail.stdout, "[ci-fail] failed_steps=seamgrim_ci_gate,oi405_406_close"):
            return fail("failcase failed_steps priority order missing")
        if not ensure_contains(proc_fail.stdout, "[ci-fail] digest="):
            return fail("failcase digest missing")
        if not ensure_contains(proc_fail.stdout, "[ci-fail] step_logs=seamgrim_ci_gate"):
            return fail("failcase step log path missing")
        if not ensure_contains(proc_fail.stdout, "[ci-fail-brief] step=seamgrim_ci_gate"):
            return fail("failcase brief message missing")
        if not ensure_contains(proc_fail.stdout, "[ci-fail-tail] step=seamgrim_ci_gate stream=stderr"):
            return fail("failcase tail header missing")
        if not ensure_contains(proc_fail.stdout, "[ci-fail-tail] sg err 3"):
            return fail("failcase tail content missing")
        if not ensure_contains(proc_fail.stdout, "[ci-fail-verify] summary=ok"):
            return fail("failcase summary verify missing")
        fail_brief = report_dir / "failcase.ci_fail_brief.txt"
        if not fail_brief.exists():
            return fail("failcase brief file missing")
        fail_brief_line = fail_brief.read_text(encoding="utf-8").strip()
        if not ensure_contains(fail_brief_line, "status=fail"):
            return fail("failcase brief status missing")
        if not ensure_contains(fail_brief_line, "top_step=seamgrim_ci_gate"):
            return fail("failcase brief top_step missing")
        fail_triage = report_dir / "failcase.ci_fail_triage.detjson"
        if not fail_triage.exists():
            return fail("failcase triage file missing")
        fail_triage_doc = json.loads(fail_triage.read_text(encoding="utf-8"))
        if str(fail_triage_doc.get("schema", "")) != "ddn.ci.fail_triage.v1":
            return fail("failcase triage schema mismatch")
        if str(fail_triage_doc.get("status", "")) != "fail":
            return fail("failcase triage status mismatch")
        if not bool(fail_triage_doc.get("summary_verify_ok", False)):
            return fail("failcase triage summary_verify_ok mismatch")
        if int(fail_triage_doc.get("summary_verify_issues_count", -1)) != 0:
            return fail("failcase triage summary_verify_issues_count mismatch")
        if str(fail_triage_doc.get("summary_verify_top_issue", "")).strip() != "-":
            return fail("failcase triage summary_verify_top_issue mismatch")
        if int(fail_triage_doc.get("failed_steps_count", 0)) <= 0:
            return fail("failcase triage failed_steps_count mismatch")
        fail_artifacts = fail_triage_doc.get("artifacts")
        if not isinstance(fail_artifacts, dict):
            return fail("failcase triage artifacts missing")
        for key in ("ci_fail_brief_txt", "ci_fail_triage_json"):
            row = fail_artifacts.get(key)
            if not isinstance(row, dict):
                return fail(f"failcase triage artifacts missing {key}")
            if not bool(row.get("exists", False)):
                return fail(f"failcase triage artifacts {key} exists mismatch")
        fail_steps = fail_triage_doc.get("failed_steps")
        if not isinstance(fail_steps, list) or not fail_steps:
            return fail("failcase triage failed_steps missing")
        first_row = fail_steps[0]
        if not isinstance(first_row, dict):
            return fail("failcase triage failed_steps row invalid")
        if "stderr_log_path_norm" not in first_row:
            return fail("failcase triage normalized stderr path missing")

        build_case(
            report_dir,
            "failsummary",
            status="fail",
            reason="aggregate_failed",
            with_digest=True,
            broken_summary=True,
        )
        proc_fail_summary = run_emit(
            report_dir,
            "--prefix",
            "failsummary",
            "--print-failure-digest",
            "4",
            "--fail-on-summary-verify-error",
            "--triage-json-out",
            str(triage_tpl),
            "--require-final-line",
        )
        if proc_fail_summary.returncode == 0:
            return fail("failsummary must fail when summary verify option is enabled")
        if not ensure_contains(proc_fail_summary.stdout, "[ci-fail-verify] summary=fail"):
            return fail("failsummary verify fail line missing")
        fail_summary_triage = report_dir / "failsummary.ci_fail_triage.detjson"
        if not fail_summary_triage.exists():
            return fail("failsummary triage file missing")
        fail_summary_triage_doc = json.loads(fail_summary_triage.read_text(encoding="utf-8"))
        if str(fail_summary_triage_doc.get("schema", "")) != "ddn.ci.fail_triage.v1":
            return fail("failsummary triage schema mismatch")
        if bool(fail_summary_triage_doc.get("summary_verify_ok", True)):
            return fail("failsummary triage summary_verify_ok mismatch")
        verify_issues = fail_summary_triage_doc.get("summary_verify_issues")
        if not isinstance(verify_issues, list) or not verify_issues:
            return fail("failsummary triage summary_verify_issues missing")
        if int(fail_summary_triage_doc.get("summary_verify_issues_count", -1)) != len(verify_issues):
            return fail("failsummary triage summary_verify_issues_count mismatch")
        top_issue = str(fail_summary_triage_doc.get("summary_verify_top_issue", "")).strip()
        if not top_issue:
            return fail("failsummary triage summary_verify_top_issue missing")
        first_issue = str(verify_issues[0]).strip()
        if first_issue not in summary_verify_codes:
            return fail(f"failsummary triage summary_verify_issues invalid: {first_issue}")
        if top_issue != first_issue:
            return fail("failsummary triage summary_verify_top_issue mismatch")

        empty_dir = report_dir / "empty"
        empty_dir.mkdir(parents=True, exist_ok=True)
        proc_empty = run_emit(empty_dir, "--require-final-line")
        if proc_empty.returncode == 0:
            return fail("empty case must fail when --require-final-line is set")
        if not ensure_contains(proc_empty.stdout, "status=unknown reason=final_line_missing"):
            return fail("empty case missing unknown status line")

    print("[ci-final-emitter-check] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
