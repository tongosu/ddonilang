#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


def sanitize_report_prefix(value: str) -> str:
    raw = value.strip()
    if not raw:
        return ""
    out_chars: list[str] = []
    for ch in raw:
        if ch.isalnum() or ch in ("-", "_", "."):
            out_chars.append(ch)
        else:
            out_chars.append("_")
    sanitized = "".join(out_chars).strip("._-")
    return sanitized


def sanitize_step_name(value: str) -> str:
    raw = str(value).strip()
    if not raw:
        return "step"
    out_chars: list[str] = []
    for ch in raw:
        if ch.isalnum() or ch in ("-", "_", "."):
            out_chars.append(ch)
        else:
            out_chars.append("_")
    sanitized = "".join(out_chars).strip("._-")
    return sanitized or "step"


def report_path(report_dir: Path, base_name: str, prefix: str) -> Path:
    if not prefix:
        return report_dir / base_name
    return report_dir / f"{prefix}.{base_name}"


def cleanup_prefixed_reports(
    report_dir: Path,
    prefix: str,
    base_names: list[str],
    dry_run: bool,
) -> int:
    count = 0
    for base_name in base_names:
        target = report_path(report_dir, base_name, prefix)
        if not target.exists():
            continue
        if dry_run:
            print(f"[ci-gate] clean(dry-run) would_remove={target}")
        else:
            target.unlink(missing_ok=True)
            print(f"[ci-gate] clean removed={target}")
        count += 1
    return count


def cleanup_prefixed_step_logs(step_log_dir: Path, prefix: str, dry_run: bool) -> int:
    pattern = f"{prefix}.ci_gate_step_*.txt" if prefix else "ci_gate_step_*.txt"
    count = 0
    for target in sorted(step_log_dir.glob(pattern)):
        if not target.exists():
            continue
        if dry_run:
            print(f"[ci-gate] clean(dry-run) would_remove={target}")
        else:
            target.unlink(missing_ok=True)
            print(f"[ci-gate] clean removed={target}")
        count += 1
    return count


def print_report_paths(
    seamgrim_report: Path,
    seamgrim_ui_age3_report: Path,
    age3_close_report: Path,
    age4_close_report: Path,
    age4_pack_report: Path,
    age3_close_summary_md: Path,
    age3_close_status_json: Path,
    age3_close_status_line: Path,
    age3_close_badge_json: Path,
    aggregate_status_line: Path,
    aggregate_status_parse_json: Path,
    final_status_line: Path,
    final_status_parse_json: Path,
    summary_line_path: Path,
    ci_gate_result_json: Path,
    ci_gate_result_parse_json: Path,
    ci_gate_result_line_path: Path,
    ci_gate_badge_json: Path,
    ci_fail_brief_txt: Path,
    ci_fail_triage_json: Path,
    oi_report: Path,
    oi_pack_report: Path,
    aggregate_report: Path,
    summary_path: Path,
) -> None:
    print("[ci-gate] reports")
    print(f" - seamgrim={seamgrim_report}")
    print(f" - seamgrim_ui_age3={seamgrim_ui_age3_report}")
    print(f" - age3_close={age3_close_report}")
    print(f" - age4_close={age4_close_report}")
    print(f" - age4_pack={age4_pack_report}")
    print(f" - age3_close_summary_md={age3_close_summary_md}")
    print(f" - age3_close_status_json={age3_close_status_json}")
    print(f" - age3_close_status_line={age3_close_status_line}")
    print(f" - age3_close_badge_json={age3_close_badge_json}")
    print(f" - aggregate_status_line={aggregate_status_line}")
    print(f" - aggregate_status_parse_json={aggregate_status_parse_json}")
    print(f" - final_status_line={final_status_line}")
    print(f" - final_status_parse_json={final_status_parse_json}")
    print(f" - summary_line={summary_line_path}")
    print(f" - ci_gate_result_json={ci_gate_result_json}")
    print(f" - ci_gate_result_parse_json={ci_gate_result_parse_json}")
    print(f" - ci_gate_result_line={ci_gate_result_line_path}")
    print(f" - ci_gate_badge_json={ci_gate_badge_json}")
    print(f" - ci_fail_brief_txt={ci_fail_brief_txt}")
    print(f" - ci_fail_triage_json={ci_fail_triage_json}")
    print(f" - oi_close={oi_report}")
    print(f" - oi_pack={oi_pack_report}")
    print(f" - aggregate={aggregate_report}")
    print(f" - summary={summary_path}")


def output_line_count(stdout: str, stderr: str) -> int:
    return len(stdout.splitlines()) + len(stderr.splitlines())


def run_step(
    root: Path,
    name: str,
    cmd: list[str],
    quiet_success_logs: bool,
    compact_step_logs: bool,
    step_log_failed_only: bool,
    stdout_log_path: Path | None = None,
    stderr_log_path: Path | None = None,
) -> dict[str, object]:
    if stdout_log_path is not None:
        stdout_log_path.parent.mkdir(parents=True, exist_ok=True)
    if stderr_log_path is not None:
        stderr_log_path.parent.mkdir(parents=True, exist_ok=True)
    if compact_step_logs:
        print(f"[ci-gate] step={name} start")
    else:
        print(f"[ci-gate] step={name} cmd={' '.join(cmd)}")
    try:
        proc = subprocess.run(
            cmd,
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
    except OSError as exc:
        err_text = f"[ci-gate] step={name} launch_error={exc}\n"
        print(err_text.rstrip(), file=sys.stderr)
        if compact_step_logs:
            print(f"[ci-gate] step={name} cmd={' '.join(cmd)}")
        if stderr_log_path is not None:
            stderr_log_path.write_text(err_text, encoding="utf-8")
        print(f"[ci-gate] step={name} exit=127")
        return {
            "returncode": 127,
            "stdout_line_count": 0,
            "stderr_line_count": 1,
            "stdout_log_path": "",
            "stderr_log_path": str(stderr_log_path) if stderr_log_path is not None else "",
        }
    should_write_step_logs = not step_log_failed_only or proc.returncode != 0
    written_stdout_path = ""
    written_stderr_path = ""
    if should_write_step_logs and stdout_log_path is not None:
        stdout_log_path.write_text(proc.stdout or "", encoding="utf-8")
        written_stdout_path = str(stdout_log_path)
    if should_write_step_logs and stderr_log_path is not None:
        stderr_log_path.write_text(proc.stderr or "", encoding="utf-8")
        written_stderr_path = str(stderr_log_path)
    if compact_step_logs and proc.returncode != 0:
        print(f"[ci-gate] step={name} cmd={' '.join(cmd)}")
    if quiet_success_logs and proc.returncode == 0:
        line_count = output_line_count(proc.stdout or "", proc.stderr or "")
        if line_count > 0 and not compact_step_logs:
            print(f"[ci-gate] step={name} output_suppressed=1 line_count={line_count}")
    else:
        if proc.stdout:
            print(proc.stdout, end="")
        if proc.stderr:
            print(proc.stderr, end="", file=sys.stderr)
    print(f"[ci-gate] step={name} exit={proc.returncode}")
    return {
        "returncode": int(proc.returncode),
        "stdout_line_count": len((proc.stdout or "").splitlines()),
        "stderr_line_count": len((proc.stderr or "").splitlines()),
        "stdout_log_path": written_stdout_path,
        "stderr_log_path": written_stderr_path,
    }


def load_payload(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def clip_line(text: str, limit: int = 140) -> str:
    normalized = " ".join(text.split())
    if len(normalized) <= limit:
        return normalized
    return normalized[:limit] + "..."


def read_compact_line(path: Path, limit: int = 220) -> str:
    try:
        text = path.read_text(encoding="utf-8").strip()
    except Exception:
        return "-"
    if not text:
        return "-"
    return clip_line(text, limit)


def read_compact_from_parse(parse_path: Path, fallback: str) -> str:
    payload = load_payload(parse_path)
    if isinstance(payload, dict):
        compact = str(payload.get("compact_line", "")).strip()
        if compact:
            return clip_line(compact, 220)
    return fallback


def resolve_summary_compact(
    ci_gate_result_line_path: Path,
    final_status_parse_path: Path,
    final_status_line_path: Path,
) -> str:
    line = read_compact_line(ci_gate_result_line_path)
    if line != "-":
        return line
    return read_compact_from_parse(
        final_status_parse_path,
        fallback=read_compact_line(final_status_line_path),
    )


def print_failure_block(
    steps_log: list[dict[str, object]],
    seamgrim_report: Path,
    age3_close_report: Path,
    age4_close_report: Path,
    oi_report: Path,
    aggregate_report: Path,
    max_digest: int = 3,
    max_step_details: int = 3,
) -> list[str]:
    failed_steps = [str(row.get("name", "-")) for row in steps_log if not bool(row.get("ok", False))]
    if not failed_steps:
        return []
    lines = [
        "[ci-gate-summary] FAIL",
        f"[ci-gate-summary] failed_steps={','.join(failed_steps)}",
    ]
    detailed_rows = [row for row in steps_log if not bool(row.get("ok", False))]
    for row in detailed_rows[:max_step_details]:
        name = str(row.get("name", "-"))
        rc = int(row.get("returncode", -1))
        cmd = row.get("cmd")
        cmd_text = " ".join(str(token) for token in cmd) if isinstance(cmd, list) else "-"
        stdout_log = str(row.get("stdout_log_path", "")).strip()
        stderr_log = str(row.get("stderr_log_path", "")).strip()
        lines.append(f"[ci-gate-summary] failed_step_detail={name} rc={rc} cmd={clip_line(cmd_text, 160)}")
        if stdout_log or stderr_log:
            lines.append(
                f"[ci-gate-summary] failed_step_logs={name} "
                f"stdout={stdout_log or '-'} stderr={stderr_log or '-'}"
            )

    seamgrim_doc = load_payload(seamgrim_report)
    if isinstance(seamgrim_doc, dict):
        failed = seamgrim_doc.get("failure_digest")
        if isinstance(failed, list) and failed:
            top = " | ".join(clip_line(str(line)) for line in failed[:max_digest])
            lines.append(f"[ci-gate-summary] seamgrim_digest={top}")

    age3_doc = load_payload(age3_close_report)
    if isinstance(age3_doc, dict):
        failed = age3_doc.get("failure_digest")
        if isinstance(failed, list) and failed:
            top = " | ".join(clip_line(str(line)) for line in failed[:max_digest])
            lines.append(f"[ci-gate-summary] age3_digest={top}")

    age4_doc = load_payload(age4_close_report)
    if isinstance(age4_doc, dict):
        failed = age4_doc.get("failure_digest")
        if isinstance(failed, list) and failed:
            top = " | ".join(clip_line(str(line)) for line in failed[:max_digest])
            lines.append(f"[ci-gate-summary] age4_digest={top}")

    oi_doc = load_payload(oi_report)
    if isinstance(oi_doc, dict):
        failed = oi_doc.get("failure_digest")
        if isinstance(failed, list) and failed:
            top = " | ".join(clip_line(str(line)) for line in failed[:max_digest])
            lines.append(f"[ci-gate-summary] oi_digest={top}")

    aggregate_doc = load_payload(aggregate_report)
    if isinstance(aggregate_doc, dict):
        failed = aggregate_doc.get("failure_digest")
        if isinstance(failed, list) and failed:
            top = " | ".join(clip_line(str(line)) for line in failed[:max_digest])
            lines.append(f"[ci-gate-summary] aggregate_digest={top}")
    return lines


def write_summary(path: Path, lines: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    body = "\n".join(lines).rstrip() + "\n"
    path.write_text(body, encoding="utf-8")
    print(f"[ci-gate] summary_report={path}")


def write_summary_line(path: Path, line: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(line.rstrip() + "\n", encoding="utf-8")
    print(f"[ci-gate] summary_line_report={path}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Run seamgrim+OI close gates and enforce aggregate result")
    parser.add_argument(
        "--report-dir",
        default="build/reports",
        help="directory for seamgrim/oi/aggregate reports",
    )
    parser.add_argument(
        "--fast-fail",
        action="store_true",
        help="stop immediately when seamgrim or OI close gate fails",
    )
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument(
        "--core-tests",
        action="store_true",
        help="run core test steps inside aggregate gate (full mode)",
    )
    mode.add_argument(
        "--skip-core-tests",
        action="store_true",
        help="skip core test steps inside aggregate gate (fast mode, default)",
    )
    parser.add_argument(
        "--report-prefix",
        default="",
        help="optional prefix for report file names (safe chars only)",
    )
    parser.add_argument(
        "--auto-prefix-env",
        default="",
        help="comma-separated env keys used as prefix source when --report-prefix is empty",
    )
    parser.add_argument(
        "--clean-prefixed-reports",
        action="store_true",
        help="remove existing prefixed report files before execution",
    )
    parser.add_argument(
        "--clean-dry-run",
        action="store_true",
        help="with --clean-prefixed-reports, print targets only (no delete)",
    )
    parser.add_argument(
        "--print-report-paths",
        action="store_true",
        help="print resolved report file paths before running checks",
    )
    parser.add_argument(
        "--quiet-success-logs",
        action="store_true",
        help="suppress child stdout/stderr for successful steps (failed steps remain verbose)",
    )
    parser.add_argument(
        "--compact-step-logs",
        action="store_true",
        help="print compact step start/exit lines (full command shown only on failures)",
    )
    parser.add_argument(
        "--step-log-dir",
        default="",
        help="optional directory to write per-step stdout/stderr logs",
    )
    parser.add_argument(
        "--step-log-failed-only",
        action="store_true",
        help="when set with --step-log-dir, write step stdout/stderr files only for failed steps",
    )
    parser.add_argument(
        "--full-pass-summary",
        action="store_true",
        help="print full PASS summary block (default prints compact PASS summary)",
    )
    parser.add_argument(
        "--report-index-json",
        default="",
        help="optional path to write run index json (step return codes + resolved report paths)",
    )
    parser.add_argument(
        "--report-index-base-name",
        default="ci_gate_report_index.detjson",
        help="base file name for index report when --report-index-json is empty",
    )
    parser.add_argument(
        "--summary-txt",
        default="",
        help="optional path to write ci gate summary text",
    )
    parser.add_argument(
        "--summary-base-name",
        default="ci_gate_summary.txt",
        help="base file name for summary report when --summary-txt is empty",
    )
    parser.add_argument(
        "--age3-summary-md",
        default="",
        help="optional path to write age3 close markdown summary",
    )
    parser.add_argument(
        "--age3-summary-base-name",
        default="age3_close_summary.md",
        help="base file name for age3 close markdown summary when --age3-summary-md is empty",
    )
    parser.add_argument(
        "--age3-status-json",
        default="",
        help="optional path to write age3 close status json",
    )
    parser.add_argument(
        "--age3-status-base-name",
        default="age3_close_status.detjson",
        help="base file name for age3 close status when --age3-status-json is empty",
    )
    parser.add_argument(
        "--age3-status-line-txt",
        default="",
        help="optional path to write one-line age3 close status text",
    )
    parser.add_argument(
        "--age3-status-line-base-name",
        default="age3_close_status_line.txt",
        help="base file name for one-line age3 status when --age3-status-line-txt is empty",
    )
    parser.add_argument(
        "--age3-badge-json",
        default="",
        help="optional path to write age3 close badge json",
    )
    parser.add_argument(
        "--age3-badge-base-name",
        default="age3_close_badge.detjson",
        help="base file name for age3 close badge json when --age3-badge-json is empty",
    )
    parser.add_argument(
        "--aggregate-status-line-txt",
        default="",
        help="optional path to write one-line aggregate gate status text",
    )
    parser.add_argument(
        "--aggregate-status-line-base-name",
        default="ci_aggregate_status_line.txt",
        help="base file name for aggregate gate one-line status when --aggregate-status-line-txt is empty",
    )
    parser.add_argument(
        "--aggregate-status-parse-json",
        default="",
        help="optional path to write parsed aggregate status-line json",
    )
    parser.add_argument(
        "--aggregate-status-parse-base-name",
        default="ci_aggregate_status_line_parse.detjson",
        help="base file name for aggregate status-line parse json when --aggregate-status-parse-json is empty",
    )
    parser.add_argument(
        "--final-status-line-txt",
        default="",
        help="optional path to write final CI gate one-line status text",
    )
    parser.add_argument(
        "--final-status-line-base-name",
        default="ci_gate_final_status_line.txt",
        help="base file name for final CI gate one-line status when --final-status-line-txt is empty",
    )
    parser.add_argument(
        "--final-status-parse-json",
        default="",
        help="optional path to write parsed final status-line json",
    )
    parser.add_argument(
        "--final-status-parse-base-name",
        default="ci_gate_final_status_line_parse.detjson",
        help="base file name for final status-line parse json when --final-status-parse-json is empty",
    )
    parser.add_argument(
        "--summary-line-txt",
        default="",
        help="optional path to write single-line ci gate summary text",
    )
    parser.add_argument(
        "--summary-line-base-name",
        default="ci_gate_summary_line.txt",
        help="base file name for single-line ci gate summary when --summary-line-txt is empty",
    )
    parser.add_argument(
        "--ci-gate-result-json",
        default="",
        help="optional path to write compact CI gate result json",
    )
    parser.add_argument(
        "--ci-gate-result-base-name",
        default="ci_gate_result.detjson",
        help="base file name for compact CI gate result json when --ci-gate-result-json is empty",
    )
    parser.add_argument(
        "--ci-gate-result-parse-json",
        default="",
        help="optional path to write parsed ci gate result json",
    )
    parser.add_argument(
        "--ci-gate-result-parse-base-name",
        default="ci_gate_result_parse.detjson",
        help="base file name for parsed ci gate result json when --ci-gate-result-parse-json is empty",
    )
    parser.add_argument(
        "--ci-gate-result-line-txt",
        default="",
        help="optional path to write one-line ci gate result text",
    )
    parser.add_argument(
        "--ci-gate-result-line-base-name",
        default="ci_gate_result_line.txt",
        help="base file name for one-line ci gate result text when --ci-gate-result-line-txt is empty",
    )
    parser.add_argument(
        "--ci-gate-badge-json",
        default="",
        help="optional path to write ci gate badge json",
    )
    parser.add_argument(
        "--ci-gate-badge-base-name",
        default="ci_gate_badge.detjson",
        help="base file name for ci gate badge json when --ci-gate-badge-json is empty",
    )
    parser.add_argument(
        "--ci-fail-brief-txt",
        default="",
        help="optional path hint for ci failure brief one-line txt (external emitter output)",
    )
    parser.add_argument(
        "--ci-fail-brief-base-name",
        default="ci_fail_brief.txt",
        help="base file name for ci failure brief when --ci-fail-brief-txt is empty",
    )
    parser.add_argument(
        "--ci-fail-triage-json",
        default="",
        help="optional path hint for ci failure triage json (external emitter output)",
    )
    parser.add_argument(
        "--ci-fail-triage-base-name",
        default="ci_fail_triage.detjson",
        help="base file name for ci failure triage json when --ci-fail-triage-json is empty",
    )
    parser.add_argument(
        "--backup-hygiene",
        action="store_true",
        help="run lesson backup hygiene step before seamgrim gate (move *.bak.ddn + verify empty)",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    py = sys.executable
    report_dir = Path(args.report_dir)
    report_dir.mkdir(parents=True, exist_ok=True)
    prefix_source = "arg"
    raw_prefix = args.report_prefix.strip()
    if not raw_prefix and args.auto_prefix_env.strip():
        for env_key in [item.strip() for item in args.auto_prefix_env.split(",") if item.strip()]:
            value = os.environ.get(env_key, "").strip()
            if value:
                raw_prefix = value
                prefix_source = f"env:{env_key}"
                break
    prefix = sanitize_report_prefix(raw_prefix)
    if raw_prefix and not prefix:
        print("[ci-gate] invalid --report-prefix after sanitize", file=sys.stderr)
        return 2
    if prefix:
        print(f"[ci-gate] report_prefix={prefix} source={prefix_source}")
    if args.clean_prefixed_reports and not prefix:
        print("[ci-gate] --clean-prefixed-reports requires non-empty prefix", file=sys.stderr)
        return 2
    explicit_step_log_dir = args.step_log_dir.strip()
    step_log_dir = Path(explicit_step_log_dir) if explicit_step_log_dir else None
    if step_log_dir is not None:
        step_log_dir.mkdir(parents=True, exist_ok=True)

    seamgrim_base_name = "seamgrim_ci_gate_report.json"
    seamgrim_ui_age3_base_name = "seamgrim_ui_age3_gate_report.detjson"
    age3_close_base_name = "age3_close_report.detjson"
    age4_close_base_name = "age4_close_report.detjson"
    age4_pack_base_name = "age4_close_pack_report.detjson"
    oi_close_base_name = "oi405_406_close_report.detjson"
    oi_pack_base_name = "oi405_406_pack_report.detjson"
    aggregate_base_name = "ci_aggregate_report.detjson"
    backup_hygiene_move_base_name = "seamgrim_backup_hygiene_move.detjson"
    backup_hygiene_verify_base_name = "seamgrim_backup_hygiene_verify.detjson"
    report_base_names = [
        seamgrim_base_name,
        seamgrim_ui_age3_base_name,
        age3_close_base_name,
        age4_close_base_name,
        age4_pack_base_name,
        backup_hygiene_move_base_name,
        backup_hygiene_verify_base_name,
        args.age3_summary_base_name,
        args.age3_status_base_name,
        args.age3_status_line_base_name,
        args.age3_badge_base_name,
        args.aggregate_status_line_base_name,
        args.aggregate_status_parse_base_name,
        args.final_status_line_base_name,
        args.final_status_parse_base_name,
        args.ci_gate_result_base_name,
        args.ci_gate_result_parse_base_name,
        args.ci_gate_result_line_base_name,
        args.ci_gate_badge_base_name,
        args.ci_fail_brief_base_name,
        args.ci_fail_triage_base_name,
        oi_close_base_name,
        oi_pack_base_name,
        aggregate_base_name,
        args.report_index_base_name,
        args.summary_base_name,
        args.summary_line_base_name,
    ]
    if args.clean_prefixed_reports:
        removed_count = cleanup_prefixed_reports(
            report_dir,
            prefix,
            report_base_names,
            dry_run=bool(args.clean_dry_run),
        )
        if step_log_dir is not None:
            removed_count += cleanup_prefixed_step_logs(
                step_log_dir,
                prefix,
                dry_run=bool(args.clean_dry_run),
            )
        print(f"[ci-gate] clean done count={removed_count} dry_run={int(bool(args.clean_dry_run))}")

    seamgrim_report = report_path(report_dir, seamgrim_base_name, prefix)
    seamgrim_ui_age3_report = report_path(report_dir, seamgrim_ui_age3_base_name, prefix)
    age3_close_report = report_path(report_dir, age3_close_base_name, prefix)
    age4_close_report = report_path(report_dir, age4_close_base_name, prefix)
    age4_pack_report = report_path(report_dir, age4_pack_base_name, prefix)
    backup_hygiene_move_report = report_path(report_dir, backup_hygiene_move_base_name, prefix)
    backup_hygiene_verify_report = report_path(report_dir, backup_hygiene_verify_base_name, prefix)
    explicit_age3_summary_md = args.age3_summary_md.strip()
    if explicit_age3_summary_md:
        age3_close_summary_md = Path(explicit_age3_summary_md)
    else:
        age3_close_summary_md = report_path(report_dir, args.age3_summary_base_name, prefix)
    explicit_age3_status_json = args.age3_status_json.strip()
    if explicit_age3_status_json:
        age3_close_status_json = Path(explicit_age3_status_json)
    else:
        age3_close_status_json = report_path(report_dir, args.age3_status_base_name, prefix)
    explicit_age3_status_line_txt = args.age3_status_line_txt.strip()
    if explicit_age3_status_line_txt:
        age3_close_status_line = Path(explicit_age3_status_line_txt)
    else:
        age3_close_status_line = report_path(report_dir, args.age3_status_line_base_name, prefix)
    explicit_age3_badge_json = args.age3_badge_json.strip()
    if explicit_age3_badge_json:
        age3_close_badge_json = Path(explicit_age3_badge_json)
    else:
        age3_close_badge_json = report_path(report_dir, args.age3_badge_base_name, prefix)
    explicit_aggregate_status_line_txt = args.aggregate_status_line_txt.strip()
    if explicit_aggregate_status_line_txt:
        aggregate_status_line = Path(explicit_aggregate_status_line_txt)
    else:
        aggregate_status_line = report_path(report_dir, args.aggregate_status_line_base_name, prefix)
    explicit_aggregate_status_parse_json = args.aggregate_status_parse_json.strip()
    if explicit_aggregate_status_parse_json:
        aggregate_status_parse_json = Path(explicit_aggregate_status_parse_json)
    else:
        aggregate_status_parse_json = report_path(report_dir, args.aggregate_status_parse_base_name, prefix)
    explicit_final_status_line_txt = args.final_status_line_txt.strip()
    if explicit_final_status_line_txt:
        final_status_line = Path(explicit_final_status_line_txt)
    else:
        final_status_line = report_path(report_dir, args.final_status_line_base_name, prefix)
    explicit_final_status_parse_json = args.final_status_parse_json.strip()
    if explicit_final_status_parse_json:
        final_status_parse_json = Path(explicit_final_status_parse_json)
    else:
        final_status_parse_json = report_path(report_dir, args.final_status_parse_base_name, prefix)
    oi_report = report_path(report_dir, oi_close_base_name, prefix)
    oi_pack_report = report_path(report_dir, oi_pack_base_name, prefix)
    aggregate_report = report_path(report_dir, aggregate_base_name, prefix)
    explicit_index_json = args.report_index_json.strip()
    if explicit_index_json:
        index_report_path = Path(explicit_index_json)
    else:
        index_report_path = report_path(report_dir, args.report_index_base_name, prefix)
    explicit_summary_txt = args.summary_txt.strip()
    if explicit_summary_txt:
        summary_path = Path(explicit_summary_txt)
    else:
        summary_path = report_path(report_dir, args.summary_base_name, prefix)
    explicit_summary_line_txt = args.summary_line_txt.strip()
    if explicit_summary_line_txt:
        summary_line_path = Path(explicit_summary_line_txt)
    else:
        summary_line_path = report_path(report_dir, args.summary_line_base_name, prefix)
    explicit_ci_gate_result_json = args.ci_gate_result_json.strip()
    if explicit_ci_gate_result_json:
        ci_gate_result_json = Path(explicit_ci_gate_result_json)
    else:
        ci_gate_result_json = report_path(report_dir, args.ci_gate_result_base_name, prefix)
    explicit_ci_gate_result_parse_json = args.ci_gate_result_parse_json.strip()
    if explicit_ci_gate_result_parse_json:
        ci_gate_result_parse_json = Path(explicit_ci_gate_result_parse_json)
    else:
        ci_gate_result_parse_json = report_path(report_dir, args.ci_gate_result_parse_base_name, prefix)
    explicit_ci_gate_result_line_txt = args.ci_gate_result_line_txt.strip()
    if explicit_ci_gate_result_line_txt:
        ci_gate_result_line_path = Path(explicit_ci_gate_result_line_txt)
    else:
        ci_gate_result_line_path = report_path(report_dir, args.ci_gate_result_line_base_name, prefix)
    explicit_ci_gate_badge_json = args.ci_gate_badge_json.strip()
    if explicit_ci_gate_badge_json:
        ci_gate_badge_json = Path(explicit_ci_gate_badge_json)
    else:
        ci_gate_badge_json = report_path(report_dir, args.ci_gate_badge_base_name, prefix)
    explicit_ci_fail_brief_txt = args.ci_fail_brief_txt.strip()
    if explicit_ci_fail_brief_txt:
        ci_fail_brief_txt = Path(explicit_ci_fail_brief_txt)
    else:
        ci_fail_brief_txt = report_path(report_dir, args.ci_fail_brief_base_name, prefix)
    explicit_ci_fail_triage_json = args.ci_fail_triage_json.strip()
    if explicit_ci_fail_triage_json:
        ci_fail_triage_json = Path(explicit_ci_fail_triage_json)
    else:
        ci_fail_triage_json = report_path(report_dir, args.ci_fail_triage_base_name, prefix)
    if args.print_report_paths:
        print_report_paths(
            seamgrim_report,
            seamgrim_ui_age3_report,
            age3_close_report,
            age4_close_report,
            age4_pack_report,
            age3_close_summary_md,
            age3_close_status_json,
            age3_close_status_line,
            age3_close_badge_json,
            aggregate_status_line,
            aggregate_status_parse_json,
            final_status_line,
            final_status_parse_json,
            summary_line_path,
            ci_gate_result_json,
            ci_gate_result_parse_json,
            ci_gate_result_line_path,
            ci_gate_badge_json,
            ci_fail_brief_txt,
            ci_fail_triage_json,
            oi_report,
            oi_pack_report,
            aggregate_report,
            summary_path,
        )
        print(f" - index={index_report_path}")
        if step_log_dir is not None:
            print(f" - step_log_dir={step_log_dir}")
            print(f" - step_log_failed_only={int(bool(args.step_log_failed_only))}")
        print(f" - backup_hygiene_move={backup_hygiene_move_report}")
        print(f" - backup_hygiene_verify={backup_hygiene_verify_report}")
    run_core_tests = bool(args.core_tests)
    steps_log: list[dict[str, object]] = []

    def step_log_paths(name: str) -> tuple[Path | None, Path | None]:
        if step_log_dir is None:
            return None, None
        safe_name = sanitize_step_name(name)
        base = f"ci_gate_step_{safe_name}"
        stdout_path = report_path(step_log_dir, f"{base}.stdout.txt", prefix)
        stderr_path = report_path(step_log_dir, f"{base}.stderr.txt", prefix)
        return stdout_path, stderr_path

    def run_and_record(name: str, cmd: list[str]) -> int:
        stdout_log_path, stderr_log_path = step_log_paths(name)
        step_result = run_step(
            root,
            name,
            cmd,
            quiet_success_logs=bool(args.quiet_success_logs),
            compact_step_logs=bool(args.compact_step_logs),
            step_log_failed_only=bool(args.step_log_failed_only),
            stdout_log_path=stdout_log_path,
            stderr_log_path=stderr_log_path,
        )
        rc = int(step_result.get("returncode", 127))
        steps_log.append(
            {
                "name": name,
                "returncode": rc,
                "cmd": cmd,
                "ok": rc == 0,
                "stdout_line_count": int(step_result.get("stdout_line_count", 0)),
                "stderr_line_count": int(step_result.get("stderr_line_count", 0)),
                "stdout_log_path": str(step_result.get("stdout_log_path", "")).strip(),
                "stderr_log_path": str(step_result.get("stderr_log_path", "")).strip(),
            }
        )
        return rc

    def write_index(overall_ok: bool, announce: bool = True) -> None:
        index_path = index_report_path
        index_path.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "schema": "ddn.ci.aggregate_gate.index.v1",
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "report_prefix": prefix,
            "report_prefix_source": prefix_source if prefix else "",
            "report_dir": str(report_dir),
            "step_log_dir": str(step_log_dir) if step_log_dir is not None else "",
            "step_log_failed_only": bool(args.step_log_failed_only),
            "reports": {
                "seamgrim": str(seamgrim_report),
                "seamgrim_ui_age3": str(seamgrim_ui_age3_report),
                "age3_close": str(age3_close_report),
                "age4_close": str(age4_close_report),
                "age4_pack": str(age4_pack_report),
                "backup_hygiene_move": str(backup_hygiene_move_report),
                "backup_hygiene_verify": str(backup_hygiene_verify_report),
                "age3_close_summary_md": str(age3_close_summary_md),
                "age3_close_status_json": str(age3_close_status_json),
                "age3_close_status_line": str(age3_close_status_line),
                "age3_close_badge_json": str(age3_close_badge_json),
                "aggregate_status_line": str(aggregate_status_line),
                "aggregate_status_parse_json": str(aggregate_status_parse_json),
                "final_status_line": str(final_status_line),
                "final_status_parse_json": str(final_status_parse_json),
                "summary": str(summary_path),
                "summary_line": str(summary_line_path),
                "ci_gate_result_json": str(ci_gate_result_json),
                "ci_gate_result_parse_json": str(ci_gate_result_parse_json),
                "ci_gate_result_line": str(ci_gate_result_line_path),
                "ci_gate_badge_json": str(ci_gate_badge_json),
                "ci_fail_brief_txt": str(ci_fail_brief_txt),
                "ci_fail_triage_json": str(ci_fail_triage_json),
                "oi_close": str(oi_report),
                "oi_pack": str(oi_pack_report),
                "aggregate": str(aggregate_report),
            },
            "steps": steps_log,
            "overall_ok": bool(overall_ok),
        }
        index_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        if announce:
            print(f"[ci-gate] report_index={index_path}")

    def run_backup_hygiene_move() -> int:
        return run_and_record(
            "backup_hygiene_move",
            [
                py,
                "scripts/seamgrim_manage_lesson_backups.py",
                "--mode",
                "move",
                "--json-out",
                str(backup_hygiene_move_report),
            ],
        )

    def run_backup_hygiene_verify() -> int:
        return run_and_record(
            "backup_hygiene_verify",
            [
                py,
                "scripts/seamgrim_manage_lesson_backups.py",
                "--mode",
                "list",
                "--fail-on-targets",
                "--json-out",
                str(backup_hygiene_verify_report),
            ],
        )

    def render_age3_summary() -> int:
        return run_and_record(
            "age3_close_summary",
            [
                py,
                "tools/scripts/render_age3_close_summary.py",
                str(age3_close_report),
                "--out",
                str(age3_close_summary_md),
                "--fail-on-bad",
            ],
        )

    def render_age3_status() -> int:
        return run_and_record(
            "age3_close_status",
            [
                py,
                "tools/scripts/render_age3_close_status.py",
                str(age3_close_report),
                "--out",
                str(age3_close_status_json),
                "--fail-on-bad",
            ],
        )

    def render_age3_status_line() -> int:
        return run_and_record(
            "age3_close_status_line",
            [
                py,
                "tools/scripts/render_age3_close_status_line.py",
                str(age3_close_status_json),
                "--out",
                str(age3_close_status_line),
                "--fail-on-bad",
            ],
        )

    def render_age3_badge() -> int:
        return run_and_record(
            "age3_close_badge",
            [
                py,
                "tools/scripts/render_age3_close_badge.py",
                str(age3_close_status_json),
                "--status-line",
                str(age3_close_status_line),
                "--out",
                str(age3_close_badge_json),
                "--fail-on-bad",
            ],
        )

    def parse_age3_status_line() -> int:
        return run_and_record(
            "age3_close_status_line_parse",
            [
                py,
                "tools/scripts/parse_age3_close_status_line.py",
                "--status-line",
                str(age3_close_status_line),
                "--status-json",
                str(age3_close_status_json),
                "--fail-on-invalid",
            ],
        )

    def check_age3_status_line(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_age3_status_line_check.py",
            "--status-line",
            str(age3_close_status_line),
            "--status-json",
            str(age3_close_status_json),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("age3_close_status_line_check", cmd)

    def check_age3_badge(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_age3_badge_check.py",
            "--badge",
            str(age3_close_badge_json),
            "--status-json",
            str(age3_close_status_json),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("age3_close_badge_check", cmd)

    def render_aggregate_status_line(fail_on_bad: bool) -> int:
        cmd = [
            py,
            "tools/scripts/render_ci_aggregate_status_line.py",
            str(aggregate_report),
            "--out",
            str(aggregate_status_line),
        ]
        if fail_on_bad:
            cmd.append("--fail-on-bad")
        return run_and_record("aggregate_status_line", cmd)

    def check_aggregate_status_line(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_ci_aggregate_status_line_check.py",
            "--status-line",
            str(aggregate_status_line),
            "--aggregate-report",
            str(aggregate_report),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("aggregate_status_line_check", cmd)

    def render_final_status_line(fail_on_bad: bool) -> int:
        cmd = [
            py,
            "tools/scripts/render_ci_gate_final_status_line.py",
            "--aggregate-status-parse",
            str(aggregate_status_parse_json),
            "--gate-index",
            str(index_report_path),
            "--out",
            str(final_status_line),
        ]
        if fail_on_bad:
            cmd.append("--fail-on-bad")
        return run_and_record("final_status_line", cmd)

    def check_final_status_line(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_ci_gate_final_status_line_check.py",
            "--status-line",
            str(final_status_line),
            "--aggregate-status-parse",
            str(aggregate_status_parse_json),
            "--gate-index",
            str(index_report_path),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("final_status_line_check", cmd)

    def parse_final_status_line() -> int:
        return run_and_record(
            "final_status_line_parse",
            [
                py,
                "tools/scripts/parse_ci_gate_final_status_line.py",
                "--status-line",
                str(final_status_line),
                "--json-out",
                str(final_status_parse_json),
                "--compact-out",
                str(summary_line_path),
                "--fail-on-invalid",
            ],
        )

    def parse_aggregate_status_line() -> int:
        return run_and_record(
            "aggregate_status_line_parse",
            [
                py,
                "tools/scripts/parse_ci_aggregate_status_line.py",
                "--status-line",
                str(aggregate_status_line),
                "--aggregate-report",
                str(aggregate_report),
                "--json-out",
                str(aggregate_status_parse_json),
                "--fail-on-invalid",
            ],
        )

    def check_summary_line(require_pass: bool, use_result_parse: bool) -> int:
        cmd = [
            py,
            "tests/run_ci_gate_summary_line_check.py",
            "--summary-line",
            str(summary_line_path),
        ]
        if use_result_parse:
            cmd.extend(["--ci-gate-result-parse", str(ci_gate_result_parse_json)])
        else:
            cmd.extend(["--final-status-parse", str(final_status_parse_json)])
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("summary_line_check", cmd)

    def render_ci_gate_result(fail_on_bad: bool) -> int:
        cmd = [
            py,
            "tools/scripts/render_ci_gate_result.py",
            "--final-status-parse",
            str(final_status_parse_json),
            "--summary-line",
            str(summary_line_path),
            "--gate-index",
            str(index_report_path),
            "--out",
            str(ci_gate_result_json),
        ]
        if fail_on_bad:
            cmd.append("--fail-on-bad")
        return run_and_record("ci_gate_result", cmd)

    def check_ci_gate_result(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_ci_gate_result_check.py",
            "--result",
            str(ci_gate_result_json),
            "--final-status-parse",
            str(final_status_parse_json),
            "--summary-line",
            str(summary_line_path),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("ci_gate_result_check", cmd)

    def parse_ci_gate_result(fail_on_fail: bool) -> int:
        cmd = [
            py,
            "tools/scripts/parse_ci_gate_result.py",
            "--result",
            str(ci_gate_result_json),
            "--json-out",
            str(ci_gate_result_parse_json),
            "--compact-out",
            str(ci_gate_result_line_path),
            "--fail-on-invalid",
        ]
        if fail_on_fail:
            cmd.append("--fail-on-fail")
        rc = run_and_record("ci_gate_result_parse", cmd)
        if rc == 0:
            compact = read_compact_line(ci_gate_result_line_path)
            if compact != "-":
                write_summary_line(summary_line_path, compact)
        return rc

    def render_ci_gate_badge(fail_on_bad: bool) -> int:
        cmd = [
            py,
            "tools/scripts/render_ci_gate_badge.py",
            str(ci_gate_result_json),
            "--out",
            str(ci_gate_badge_json),
        ]
        if fail_on_bad:
            cmd.append("--fail-on-bad")
        return run_and_record("ci_gate_badge", cmd)

    def check_ci_gate_badge(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_ci_gate_badge_check.py",
            "--badge",
            str(ci_gate_badge_json),
            "--result",
            str(ci_gate_result_json),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("ci_gate_badge_check", cmd)

    def check_ci_gate_outputs_consistency(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_ci_gate_outputs_consistency_check.py",
            "--summary-line",
            str(summary_line_path),
            "--result",
            str(ci_gate_result_json),
            "--result-parse",
            str(ci_gate_result_parse_json),
            "--badge",
            str(ci_gate_badge_json),
            "--final-status-parse",
            str(final_status_parse_json),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("ci_gate_outputs_consistency_check", cmd)

    def check_ci_gate_failure_summary(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_ci_gate_failure_summary_check.py",
            "--summary",
            str(summary_path),
            "--index",
            str(index_report_path),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("ci_gate_failure_summary_check", cmd)

    def check_ci_gate_summary_report(require_pass: bool) -> int:
        cmd = [
            py,
            "tests/run_ci_gate_summary_report_check.py",
            "--summary",
            str(summary_path),
            "--index",
            str(index_report_path),
        ]
        if require_pass:
            cmd.append("--require-pass")
        return run_and_record("ci_gate_summary_report_check", cmd)

    def check_ci_gate_summary_report_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_gate_summary_report_check_selftest.py",
        ]
        return run_and_record("ci_gate_summary_report_selftest", cmd)

    def check_ci_aggregate_gate_age4_diagnostics() -> int:
        cmd = [
            py,
            "tests/run_ci_aggregate_gate_age4_diagnostics_check.py",
        ]
        return run_and_record("ci_aggregate_gate_age4_diagnostics_check", cmd)

    def check_ci_builtin_name_sync() -> int:
        cmd = [
            py,
            "tests/run_builtin_name_sync_check.py",
        ]
        return run_and_record("ci_builtin_name_sync_check", cmd)

    def check_ci_aggregate_status_line_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_aggregate_status_line_selftest.py",
        ]
        return run_and_record("ci_aggregate_status_line_selftest", cmd)

    def check_ci_combine_reports_age4_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_combine_reports_age4_selftest.py",
        ]
        return run_and_record("ci_combine_reports_age4_selftest", cmd)

    def check_ci_final_line_emitter() -> int:
        cmd = [
            py,
            "tests/run_ci_final_line_emitter_check.py",
        ]
        return run_and_record("ci_final_line_emitter_check", cmd)

    def check_ci_pipeline_emit_flags() -> int:
        cmd = [
            py,
            "tests/run_ci_pipeline_emit_flags_check.py",
        ]
        return run_and_record("ci_pipeline_emit_flags_check", cmd)

    def check_ci_backup_hygiene_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_backup_hygiene_selftest.py",
        ]
        return run_and_record("ci_backup_hygiene_selftest", cmd)

    def check_ci_emit_artifacts_baseline() -> int:
        cmd = [
            py,
            "tests/run_ci_emit_artifacts_check.py",
            "--report-dir",
            str(report_dir),
        ]
        if prefix:
            cmd.extend(["--prefix", prefix])
        return run_and_record("ci_emit_artifacts_baseline_check", cmd)

    def emit_ci_final_line_for_artifacts() -> int:
        # triage artifact의 summary.exists를 최종 단계에서도 안정적으로 유지하기 위해
        # summary 파일이 아직 없으면 임시 헤더를 먼저 생성한다.
        if not summary_path.exists():
            summary_path.parent.mkdir(parents=True, exist_ok=True)
            summary_path.write_text("[ci-gate-summary] PREVIEW\n", encoding="utf-8")
        cmd = [
            py,
            "tools/scripts/emit_ci_final_line.py",
            "--report-dir",
            str(report_dir),
            "--print-failure-digest",
            "6",
            "--print-failure-tail-lines",
            "20",
            "--fail-on-summary-verify-error",
            "--failure-brief-out",
            str(ci_fail_brief_txt),
            "--triage-json-out",
            str(ci_fail_triage_json),
            "--require-final-line",
        ]
        if prefix:
            cmd.extend(["--prefix", prefix])
        return run_and_record("ci_emit_artifacts_generate", cmd)

    def check_ci_emit_artifacts_required() -> int:
        cmd = [
            py,
            "tests/run_ci_emit_artifacts_check.py",
            "--report-dir",
            str(report_dir),
            "--require-brief",
            "--require-triage",
        ]
        if prefix:
            cmd.extend(["--prefix", prefix])
        return run_and_record("ci_emit_artifacts_required_check", cmd)

    def check_ci_emit_artifacts_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_emit_artifacts_check_selftest.py",
        ]
        return run_and_record("ci_emit_artifacts_selftest", cmd)

    def check_ci_gate_failure_summary_selftest() -> int:
        cmd = [
            py,
            "tests/run_ci_gate_failure_summary_check_selftest.py",
        ]
        return run_and_record("ci_gate_failure_summary_selftest", cmd)

    def fail_and_exit(exit_code: int, message: str) -> int:
        print(message, file=sys.stderr)
        render_age3_status()
        render_age3_status_line()
        render_age3_badge()
        parse_age3_status_line()
        check_age3_status_line(require_pass=False)
        check_age3_badge(require_pass=False)
        render_age3_summary()
        render_aggregate_status_line(fail_on_bad=False)
        parse_aggregate_status_line()
        check_aggregate_status_line(require_pass=False)
        write_index(False, announce=False)
        render_final_status_line(fail_on_bad=False)
        check_final_status_line(require_pass=False)
        parse_final_status_line()
        render_ci_gate_result(fail_on_bad=False)
        check_ci_gate_result(require_pass=False)
        parse_ci_gate_result(fail_on_fail=False)
        render_ci_gate_badge(fail_on_bad=False)
        check_ci_gate_badge(require_pass=False)
        check_summary_line(require_pass=False, use_result_parse=True)
        check_ci_gate_outputs_consistency(require_pass=False)
        check_ci_final_line_emitter()
        check_ci_pipeline_emit_flags()
        check_ci_builtin_name_sync()
        check_ci_backup_hygiene_selftest()
        check_ci_aggregate_gate_age4_diagnostics()
        check_ci_aggregate_status_line_selftest()
        check_ci_gate_summary_report_selftest()
        check_ci_combine_reports_age4_selftest()
        check_ci_gate_failure_summary_selftest()
        check_ci_emit_artifacts_selftest()
        write_index(False)
        check_ci_emit_artifacts_baseline()
        lines = print_failure_block(
            steps_log,
            seamgrim_report,
            age3_close_report,
            age4_close_report,
            oi_report,
            aggregate_report,
        )
        lines.append(f"[ci-gate-summary] age3_status={age3_close_status_json}")
        lines.append(f"[ci-gate-summary] age4_status={age4_close_report}")
        lines.append(f"[ci-gate-summary] age3_status_line={age3_close_status_line}")
        lines.append(f"[ci-gate-summary] age3_badge={age3_close_badge_json}")
        lines.append(f"[ci-gate-summary] age3_status_compact={read_compact_line(age3_close_status_line)}")
        lines.append(f"[ci-gate-summary] aggregate_status_line={aggregate_status_line}")
        lines.append(f"[ci-gate-summary] aggregate_status_parse={aggregate_status_parse_json}")
        lines.append(f"[ci-gate-summary] aggregate_status_compact={read_compact_line(aggregate_status_line)}")
        lines.append(f"[ci-gate-summary] final_status_line={final_status_line}")
        lines.append(f"[ci-gate-summary] final_status_parse={final_status_parse_json}")
        lines.append(f"[ci-gate-summary] summary_line={summary_line_path}")
        lines.append(f"[ci-gate-summary] ci_gate_result={ci_gate_result_json}")
        lines.append(f"[ci-gate-summary] ci_gate_result_parse={ci_gate_result_parse_json}")
        lines.append(f"[ci-gate-summary] ci_gate_result_line={ci_gate_result_line_path}")
        lines.append(f"[ci-gate-summary] ci_gate_badge={ci_gate_badge_json}")
        lines.append(f"[ci-gate-summary] ci_fail_brief_hint={ci_fail_brief_txt}")
        lines.append(f"[ci-gate-summary] ci_fail_brief_exists={int(ci_fail_brief_txt.exists())}")
        lines.append(f"[ci-gate-summary] ci_fail_triage_hint={ci_fail_triage_json}")
        lines.append(f"[ci-gate-summary] ci_fail_triage_exists={int(ci_fail_triage_json.exists())}")
        lines.append(f"[ci-gate-summary] final_status_compact={read_compact_line(final_status_line)}")
        lines.append(f"[ci-gate-summary] age3_summary={age3_close_summary_md}")
        for line in lines:
            print(line)
        if lines:
            write_summary(summary_path, lines)
            check_ci_gate_summary_report(require_pass=False)
            check_ci_gate_failure_summary(require_pass=False)
        summary_compact = resolve_summary_compact(
            ci_gate_result_line_path,
            final_status_parse_json,
            final_status_line,
        )
        write_summary_line(summary_line_path, summary_compact)
        print(f"[ci-gate-summary-line] {summary_compact}")
        return exit_code

    if run_core_tests:
        core_lint_rc = run_and_record(
            "core_fixed64_lint",
            ["cargo", "test", "-p", "ddonirang-core", "fixed64_lint_gate_no_float_in_core"],
        )
        if args.fast_fail and core_lint_rc != 0:
            return fail_and_exit(core_lint_rc, "[ci-gate] fast-fail: core fixed64 lint failed")
        core_all_rc = run_and_record(
            "core_tests",
            ["cargo", "test", "-p", "ddonirang-core"],
        )
        if args.fast_fail and core_all_rc != 0:
            return fail_and_exit(core_all_rc, "[ci-gate] fast-fail: core tests failed")

    backup_hygiene_move_rc = 0
    backup_hygiene_verify_rc = 0
    if args.backup_hygiene:
        backup_hygiene_move_rc = run_backup_hygiene_move()
        if args.fast_fail and backup_hygiene_move_rc != 0:
            return fail_and_exit(backup_hygiene_move_rc, "[ci-gate] fast-fail: backup hygiene move failed")
        backup_hygiene_verify_rc = run_backup_hygiene_verify()
        if args.fast_fail and backup_hygiene_verify_rc != 0:
            return fail_and_exit(backup_hygiene_verify_rc, "[ci-gate] fast-fail: backup hygiene verify failed")

    seamgrim_rc = run_and_record(
        "seamgrim_ci_gate",
        [
            py,
            "tests/run_seamgrim_ci_gate.py",
            "--strict-graph",
            "--require-promoted",
            "--print-drilldown",
            "--json-out",
            str(seamgrim_report),
            "--ui-age3-json-out",
            str(seamgrim_ui_age3_report),
        ],
    )
    if args.fast_fail and seamgrim_rc != 0:
        run_and_record(
            "seamgrim_digest",
            [
                py,
                "tools/scripts/print_seamgrim_ci_gate_digest.py",
                str(seamgrim_report),
                "--only-failed",
            ],
        )
        return fail_and_exit(seamgrim_rc, "[ci-gate] fast-fail: seamgrim gate failed")

    age3_rc = run_and_record(
        "age3_close",
        [
            py,
            "tests/run_age3_close.py",
            "--seamgrim-report",
            str(seamgrim_report),
            "--ui-age3-report",
            str(seamgrim_ui_age3_report),
            "--report-out",
            str(age3_close_report),
        ],
    )
    if args.fast_fail and age3_rc != 0:
        run_and_record(
            "age3_close_digest",
            [
                py,
                "tools/scripts/print_age3_close_digest.py",
                str(age3_close_report),
                "--top",
                "6",
                "--only-failed",
            ],
        )
        return fail_and_exit(age3_rc, "[ci-gate] fast-fail: AGE3 close gate failed")
    age3_status_rc = render_age3_status()
    if args.fast_fail and age3_status_rc != 0:
        return fail_and_exit(age3_status_rc, "[ci-gate] fast-fail: AGE3 close status generation failed")
    age3_status_line_rc = render_age3_status_line()
    if args.fast_fail and age3_status_line_rc != 0:
        return fail_and_exit(age3_status_line_rc, "[ci-gate] fast-fail: AGE3 close status-line generation failed")
    age3_badge_rc = render_age3_badge()
    if args.fast_fail and age3_badge_rc != 0:
        return fail_and_exit(age3_badge_rc, "[ci-gate] fast-fail: AGE3 close badge generation failed")
    age3_status_line_parse_rc = parse_age3_status_line()
    if args.fast_fail and age3_status_line_parse_rc != 0:
        return fail_and_exit(age3_status_line_parse_rc, "[ci-gate] fast-fail: AGE3 status-line parse failed")
    age3_status_line_check_rc = check_age3_status_line(require_pass=True)
    if args.fast_fail and age3_status_line_check_rc != 0:
        return fail_and_exit(age3_status_line_check_rc, "[ci-gate] fast-fail: AGE3 status-line check failed")
    age3_badge_check_rc = check_age3_badge(require_pass=True)
    if args.fast_fail and age3_badge_check_rc != 0:
        return fail_and_exit(age3_badge_check_rc, "[ci-gate] fast-fail: AGE3 badge check failed")
    age3_summary_rc = render_age3_summary()
    if args.fast_fail and age3_summary_rc != 0:
        return fail_and_exit(age3_summary_rc, "[ci-gate] fast-fail: AGE3 close summary generation failed")

    age4_rc = run_and_record(
        "age4_close",
        [
            py,
            "tests/run_age4_close.py",
            "--report-out",
            str(age4_close_report),
            "--pack-report-out",
            str(age4_pack_report),
        ],
    )
    if args.fast_fail and age4_rc != 0:
        run_and_record(
            "age4_close_digest",
            [
                py,
                "tools/scripts/print_age4_close_digest.py",
                str(age4_close_report),
                "--top",
                "6",
                "--only-failed",
            ],
        )
        return fail_and_exit(age4_rc, "[ci-gate] fast-fail: AGE4 close gate failed")

    oi_rc = run_and_record(
        "oi405_406_close",
        [
            py,
            "tests/run_oi405_406_close.py",
            "--strict",
            "--report-out",
            str(oi_report),
            "--pack-report-out",
            str(oi_pack_report),
        ],
    )
    if args.fast_fail and oi_rc != 0:
        run_and_record(
            "seamgrim_digest",
            [
                py,
                "tools/scripts/print_seamgrim_ci_gate_digest.py",
                str(seamgrim_report),
                "--only-failed",
            ],
        )
        run_and_record(
            "oi405_406_digest",
            [
                py,
                "tools/scripts/print_oi405_406_digest.py",
                str(oi_report),
                "--max-digest",
                "5",
                "--max-slowest",
                "3",
                "--only-failed",
            ],
        )
        return fail_and_exit(oi_rc, "[ci-gate] fast-fail: OI close gate failed")

    run_and_record(
        "seamgrim_digest",
        [
            py,
            "tools/scripts/print_seamgrim_ci_gate_digest.py",
            str(seamgrim_report),
            "--only-failed",
        ],
    )
    run_and_record(
        "age3_close_digest",
        [
            py,
            "tools/scripts/print_age3_close_digest.py",
            str(age3_close_report),
            "--top",
            "6",
            "--only-failed",
        ],
    )
    run_and_record(
        "age4_close_digest",
        [
            py,
            "tools/scripts/print_age4_close_digest.py",
            str(age4_close_report),
            "--top",
            "6",
            "--only-failed",
        ],
    )
    run_and_record(
        "oi405_406_digest",
        [
            py,
            "tools/scripts/print_oi405_406_digest.py",
            str(oi_report),
            "--max-digest",
            "5",
            "--max-slowest",
            "3",
            "--only-failed",
        ],
    )
    # combine 단계에서 index 링크를 바로 참조할 수 있도록 선기록.
    write_index(
        bool(
            backup_hygiene_move_rc == 0
            and backup_hygiene_verify_rc == 0
            and seamgrim_rc == 0
            and age3_rc == 0
            and age3_status_rc == 0
            and age3_status_line_rc == 0
            and age3_badge_rc == 0
            and age3_status_line_parse_rc == 0
            and age3_status_line_check_rc == 0
            and age3_badge_check_rc == 0
            and age3_summary_rc == 0
            and age4_rc == 0
            and oi_rc == 0
        ),
        announce=False,
    )
    combine_rc = run_and_record(
        "aggregate_combine",
        [
            py,
            "tools/scripts/combine_ci_reports.py",
            "--print-summary",
            "--fail-on-bad",
            "--require-age3",
            "--require-age4",
            "--seamgrim-report",
            str(seamgrim_report),
            "--age3-report",
            str(age3_close_report),
            "--age4-report",
            str(age4_close_report),
            "--age3-status",
            str(age3_close_status_json),
            "--age3-status-line",
            str(age3_close_status_line),
            "--age3-badge",
            str(age3_close_badge_json),
            "--oi-report",
            str(oi_report),
            "--out",
            str(aggregate_report),
            "--index-report-path",
            str(index_report_path),
        ]
    )
    run_and_record(
        "aggregate_digest",
        [
            py,
            "tools/scripts/print_ci_aggregate_digest.py",
            str(aggregate_report),
            "--top",
            "1",
            "--only-failed",
            "--show-steps",
        ],
    )
    aggregate_status_line_rc = render_aggregate_status_line(fail_on_bad=True)
    if args.fast_fail and aggregate_status_line_rc != 0:
        return fail_and_exit(aggregate_status_line_rc, "[ci-gate] fast-fail: aggregate status-line generation failed")
    aggregate_status_line_parse_rc = parse_aggregate_status_line()
    if args.fast_fail and aggregate_status_line_parse_rc != 0:
        return fail_and_exit(aggregate_status_line_parse_rc, "[ci-gate] fast-fail: aggregate status-line parse failed")
    aggregate_status_line_check_rc = check_aggregate_status_line(require_pass=True)
    if args.fast_fail and aggregate_status_line_check_rc != 0:
        return fail_and_exit(aggregate_status_line_check_rc, "[ci-gate] fast-fail: aggregate status-line check failed")

    write_index(
        bool(
            combine_rc == 0
            and backup_hygiene_move_rc == 0
            and backup_hygiene_verify_rc == 0
            and seamgrim_rc == 0
            and age3_rc == 0
            and age3_status_rc == 0
            and age3_status_line_rc == 0
            and age3_badge_rc == 0
            and age3_status_line_parse_rc == 0
            and age3_status_line_check_rc == 0
            and age3_badge_check_rc == 0
            and age3_summary_rc == 0
            and age4_rc == 0
            and oi_rc == 0
            and aggregate_status_line_rc == 0
            and aggregate_status_line_parse_rc == 0
            and aggregate_status_line_check_rc == 0
        ),
        announce=False,
    )
    final_status_line_rc = render_final_status_line(fail_on_bad=True)
    if args.fast_fail and final_status_line_rc != 0:
        return fail_and_exit(final_status_line_rc, "[ci-gate] fast-fail: final status-line generation failed")
    final_status_line_check_rc = check_final_status_line(require_pass=True)
    if args.fast_fail and final_status_line_check_rc != 0:
        return fail_and_exit(final_status_line_check_rc, "[ci-gate] fast-fail: final status-line check failed")
    final_status_line_parse_rc = parse_final_status_line()
    if args.fast_fail and final_status_line_parse_rc != 0:
        return fail_and_exit(final_status_line_parse_rc, "[ci-gate] fast-fail: final status-line parse failed")
    ci_gate_result_rc = render_ci_gate_result(fail_on_bad=True)
    if args.fast_fail and ci_gate_result_rc != 0:
        return fail_and_exit(ci_gate_result_rc, "[ci-gate] fast-fail: ci gate result generation failed")
    ci_gate_result_check_rc = check_ci_gate_result(require_pass=True)
    if args.fast_fail and ci_gate_result_check_rc != 0:
        return fail_and_exit(ci_gate_result_check_rc, "[ci-gate] fast-fail: ci gate result check failed")
    ci_gate_result_parse_rc = parse_ci_gate_result(fail_on_fail=True)
    if args.fast_fail and ci_gate_result_parse_rc != 0:
        return fail_and_exit(ci_gate_result_parse_rc, "[ci-gate] fast-fail: ci gate result parse failed")
    ci_gate_badge_rc = render_ci_gate_badge(fail_on_bad=True)
    if args.fast_fail and ci_gate_badge_rc != 0:
        return fail_and_exit(ci_gate_badge_rc, "[ci-gate] fast-fail: ci gate badge generation failed")
    ci_gate_badge_check_rc = check_ci_gate_badge(require_pass=True)
    if args.fast_fail and ci_gate_badge_check_rc != 0:
        return fail_and_exit(ci_gate_badge_check_rc, "[ci-gate] fast-fail: ci gate badge check failed")
    summary_line_check_rc = check_summary_line(require_pass=True, use_result_parse=True)
    if args.fast_fail and summary_line_check_rc != 0:
        return fail_and_exit(summary_line_check_rc, "[ci-gate] fast-fail: summary-line check failed")
    ci_gate_outputs_consistency_rc = check_ci_gate_outputs_consistency(require_pass=True)
    if args.fast_fail and ci_gate_outputs_consistency_rc != 0:
        return fail_and_exit(ci_gate_outputs_consistency_rc, "[ci-gate] fast-fail: ci gate outputs consistency check failed")
    ci_final_line_emitter_rc = check_ci_final_line_emitter()
    if args.fast_fail and ci_final_line_emitter_rc != 0:
        return fail_and_exit(ci_final_line_emitter_rc, "[ci-gate] fast-fail: ci final line emitter check failed")
    ci_pipeline_emit_flags_rc = check_ci_pipeline_emit_flags()
    if args.fast_fail and ci_pipeline_emit_flags_rc != 0:
        return fail_and_exit(ci_pipeline_emit_flags_rc, "[ci-gate] fast-fail: ci pipeline emit flags check failed")
    ci_builtin_name_sync_rc = check_ci_builtin_name_sync()
    if args.fast_fail and ci_builtin_name_sync_rc != 0:
        return fail_and_exit(ci_builtin_name_sync_rc, "[ci-gate] fast-fail: ci builtin name sync check failed")
    ci_backup_hygiene_selftest_rc = check_ci_backup_hygiene_selftest()
    if args.fast_fail and ci_backup_hygiene_selftest_rc != 0:
        return fail_and_exit(
            ci_backup_hygiene_selftest_rc,
            "[ci-gate] fast-fail: ci backup hygiene selftest failed",
        )
    ci_emit_artifacts_baseline_rc = check_ci_emit_artifacts_baseline()
    if args.fast_fail and ci_emit_artifacts_baseline_rc != 0:
        return fail_and_exit(ci_emit_artifacts_baseline_rc, "[ci-gate] fast-fail: ci emit artifacts baseline check failed")
    ci_emit_artifacts_generate_rc = emit_ci_final_line_for_artifacts()
    if args.fast_fail and ci_emit_artifacts_generate_rc != 0:
        return fail_and_exit(ci_emit_artifacts_generate_rc, "[ci-gate] fast-fail: ci emit artifacts generate failed")
    ci_emit_artifacts_required_rc = check_ci_emit_artifacts_required()
    if args.fast_fail and ci_emit_artifacts_required_rc != 0:
        return fail_and_exit(ci_emit_artifacts_required_rc, "[ci-gate] fast-fail: ci emit artifacts required check failed")
    ci_emit_artifacts_selftest_rc = check_ci_emit_artifacts_selftest()
    if args.fast_fail and ci_emit_artifacts_selftest_rc != 0:
        return fail_and_exit(ci_emit_artifacts_selftest_rc, "[ci-gate] fast-fail: ci emit artifacts selftest failed")
    ci_aggregate_gate_age4_diagnostics_rc = check_ci_aggregate_gate_age4_diagnostics()
    if args.fast_fail and ci_aggregate_gate_age4_diagnostics_rc != 0:
        return fail_and_exit(
            ci_aggregate_gate_age4_diagnostics_rc,
            "[ci-gate] fast-fail: ci aggregate gate age4 diagnostics check failed",
        )
    ci_gate_summary_report_selftest_rc = check_ci_gate_summary_report_selftest()
    if args.fast_fail and ci_gate_summary_report_selftest_rc != 0:
        return fail_and_exit(
            ci_gate_summary_report_selftest_rc,
            "[ci-gate] fast-fail: ci gate summary report selftest failed",
        )
    ci_aggregate_status_line_selftest_rc = check_ci_aggregate_status_line_selftest()
    if args.fast_fail and ci_aggregate_status_line_selftest_rc != 0:
        return fail_and_exit(
            ci_aggregate_status_line_selftest_rc,
            "[ci-gate] fast-fail: ci aggregate status-line selftest failed",
        )
    ci_combine_reports_age4_selftest_rc = check_ci_combine_reports_age4_selftest()
    if args.fast_fail and ci_combine_reports_age4_selftest_rc != 0:
        return fail_and_exit(
            ci_combine_reports_age4_selftest_rc,
            "[ci-gate] fast-fail: ci combine age4 selftest failed",
        )
    ci_gate_failure_summary_selftest_rc = check_ci_gate_failure_summary_selftest()
    if args.fast_fail and ci_gate_failure_summary_selftest_rc != 0:
        return fail_and_exit(
            ci_gate_failure_summary_selftest_rc,
            "[ci-gate] fast-fail: ci gate failure summary selftest failed",
        )

    write_index(
        bool(
            combine_rc == 0
            and backup_hygiene_move_rc == 0
            and backup_hygiene_verify_rc == 0
            and seamgrim_rc == 0
            and age3_rc == 0
            and age3_status_rc == 0
            and age3_status_line_rc == 0
            and age3_badge_rc == 0
            and age3_status_line_parse_rc == 0
            and age3_status_line_check_rc == 0
            and age3_badge_check_rc == 0
            and age3_summary_rc == 0
            and age4_rc == 0
            and oi_rc == 0
            and aggregate_status_line_rc == 0
            and aggregate_status_line_parse_rc == 0
            and aggregate_status_line_check_rc == 0
            and final_status_line_rc == 0
            and final_status_line_check_rc == 0
            and final_status_line_parse_rc == 0
            and summary_line_check_rc == 0
            and ci_gate_result_rc == 0
            and ci_gate_result_check_rc == 0
            and ci_gate_result_parse_rc == 0
            and ci_gate_badge_rc == 0
            and ci_gate_badge_check_rc == 0
            and ci_gate_outputs_consistency_rc == 0
            and ci_final_line_emitter_rc == 0
            and ci_pipeline_emit_flags_rc == 0
            and ci_builtin_name_sync_rc == 0
            and ci_backup_hygiene_selftest_rc == 0
            and ci_emit_artifacts_baseline_rc == 0
            and ci_emit_artifacts_generate_rc == 0
            and ci_emit_artifacts_required_rc == 0
            and ci_emit_artifacts_selftest_rc == 0
            and ci_aggregate_gate_age4_diagnostics_rc == 0
            and ci_gate_summary_report_selftest_rc == 0
            and ci_aggregate_status_line_selftest_rc == 0
            and ci_combine_reports_age4_selftest_rc == 0
            and ci_gate_failure_summary_selftest_rc == 0
        )
    )

    if combine_rc != 0:
        lines = print_failure_block(
            steps_log,
            seamgrim_report,
            age3_close_report,
            age4_close_report,
            oi_report,
            aggregate_report,
        )
        lines.append(f"[ci-gate-summary] age4_status={age4_close_report}")
        lines.append(f"[ci-gate-summary] aggregate_status_line={aggregate_status_line}")
        lines.append(f"[ci-gate-summary] aggregate_status_parse={aggregate_status_parse_json}")
        lines.append(f"[ci-gate-summary] aggregate_status_compact={read_compact_line(aggregate_status_line)}")
        lines.append(f"[ci-gate-summary] final_status_line={final_status_line}")
        lines.append(f"[ci-gate-summary] final_status_parse={final_status_parse_json}")
        lines.append(f"[ci-gate-summary] summary_line={summary_line_path}")
        lines.append(f"[ci-gate-summary] ci_gate_result={ci_gate_result_json}")
        lines.append(f"[ci-gate-summary] ci_gate_result_parse={ci_gate_result_parse_json}")
        lines.append(f"[ci-gate-summary] ci_gate_result_line={ci_gate_result_line_path}")
        lines.append(f"[ci-gate-summary] ci_gate_badge={ci_gate_badge_json}")
        lines.append(f"[ci-gate-summary] ci_fail_brief_hint={ci_fail_brief_txt}")
        lines.append(f"[ci-gate-summary] ci_fail_brief_exists={int(ci_fail_brief_txt.exists())}")
        lines.append(f"[ci-gate-summary] ci_fail_triage_hint={ci_fail_triage_json}")
        lines.append(f"[ci-gate-summary] ci_fail_triage_exists={int(ci_fail_triage_json.exists())}")
        lines.append(f"[ci-gate-summary] final_status_compact={read_compact_line(final_status_line)}")
        for line in lines:
            print(line)
        if lines:
            write_summary(summary_path, lines)
            check_ci_gate_summary_report(require_pass=False)
            check_ci_gate_failure_summary(require_pass=False)
        summary_compact = resolve_summary_compact(
            ci_gate_result_line_path,
            final_status_parse_json,
            final_status_line,
        )
        write_summary_line(summary_line_path, summary_compact)
        print(f"[ci-gate-summary-line] {summary_compact}")
        return combine_rc
    if (
        backup_hygiene_move_rc != 0
        or backup_hygiene_verify_rc != 0
        or seamgrim_rc != 0
        or age3_rc != 0
        or age3_status_rc != 0
        or age3_status_line_rc != 0
        or age3_badge_rc != 0
        or age3_status_line_parse_rc != 0
        or age3_status_line_check_rc != 0
        or age3_badge_check_rc != 0
        or age3_summary_rc != 0
        or age4_rc != 0
        or oi_rc != 0
        or aggregate_status_line_rc != 0
        or aggregate_status_line_parse_rc != 0
        or aggregate_status_line_check_rc != 0
        or final_status_line_rc != 0
        or final_status_line_check_rc != 0
        or final_status_line_parse_rc != 0
        or summary_line_check_rc != 0
        or ci_gate_result_rc != 0
        or ci_gate_result_check_rc != 0
        or ci_gate_result_parse_rc != 0
        or ci_gate_badge_rc != 0
        or ci_gate_badge_check_rc != 0
        or ci_gate_outputs_consistency_rc != 0
        or ci_final_line_emitter_rc != 0
        or ci_pipeline_emit_flags_rc != 0
        or ci_builtin_name_sync_rc != 0
        or ci_backup_hygiene_selftest_rc != 0
        or ci_emit_artifacts_baseline_rc != 0
        or ci_emit_artifacts_generate_rc != 0
        or ci_emit_artifacts_required_rc != 0
        or ci_emit_artifacts_selftest_rc != 0
        or ci_aggregate_gate_age4_diagnostics_rc != 0
        or ci_gate_summary_report_selftest_rc != 0
        or ci_aggregate_status_line_selftest_rc != 0
        or ci_combine_reports_age4_selftest_rc != 0
        or ci_gate_failure_summary_selftest_rc != 0
    ):
        print("[ci-gate] aggregate reported success but sub-step failed", file=sys.stderr)
        lines = print_failure_block(
            steps_log,
            seamgrim_report,
            age3_close_report,
            age4_close_report,
            oi_report,
            aggregate_report,
        )
        lines.append(f"[ci-gate-summary] age4_status={age4_close_report}")
        lines.append(f"[ci-gate-summary] aggregate_status_line={aggregate_status_line}")
        lines.append(f"[ci-gate-summary] aggregate_status_parse={aggregate_status_parse_json}")
        lines.append(f"[ci-gate-summary] aggregate_status_compact={read_compact_line(aggregate_status_line)}")
        lines.append(f"[ci-gate-summary] final_status_line={final_status_line}")
        lines.append(f"[ci-gate-summary] final_status_parse={final_status_parse_json}")
        lines.append(f"[ci-gate-summary] summary_line={summary_line_path}")
        lines.append(f"[ci-gate-summary] ci_gate_result={ci_gate_result_json}")
        lines.append(f"[ci-gate-summary] ci_gate_result_parse={ci_gate_result_parse_json}")
        lines.append(f"[ci-gate-summary] ci_gate_result_line={ci_gate_result_line_path}")
        lines.append(f"[ci-gate-summary] ci_gate_badge={ci_gate_badge_json}")
        lines.append(f"[ci-gate-summary] ci_fail_brief_hint={ci_fail_brief_txt}")
        lines.append(f"[ci-gate-summary] ci_fail_brief_exists={int(ci_fail_brief_txt.exists())}")
        lines.append(f"[ci-gate-summary] ci_fail_triage_hint={ci_fail_triage_json}")
        lines.append(f"[ci-gate-summary] ci_fail_triage_exists={int(ci_fail_triage_json.exists())}")
        lines.append(f"[ci-gate-summary] final_status_compact={read_compact_line(final_status_line)}")
        for line in lines:
            print(line)
        if lines:
            write_summary(summary_path, lines)
            check_ci_gate_summary_report(require_pass=False)
            check_ci_gate_failure_summary(require_pass=False)
        summary_compact = resolve_summary_compact(
            ci_gate_result_line_path,
            final_status_parse_json,
            final_status_line,
        )
        write_summary_line(summary_line_path, summary_compact)
        print(f"[ci-gate-summary-line] {summary_compact}")
        return 2
    if args.full_pass_summary:
        pass_lines = [
            "[ci-gate-summary] PASS",
            "[ci-gate-summary] failed_steps=(none)",
            f"[ci-gate-summary] report_index={index_report_path}",
            f"[ci-gate-summary] age3_status={age3_close_status_json}",
            f"[ci-gate-summary] age4_status={age4_close_report}",
            f"[ci-gate-summary] age3_status_line={age3_close_status_line}",
            f"[ci-gate-summary] age3_badge={age3_close_badge_json}",
            f"[ci-gate-summary] age3_status_compact={read_compact_line(age3_close_status_line)}",
            f"[ci-gate-summary] aggregate_status_line={aggregate_status_line}",
            f"[ci-gate-summary] aggregate_status_parse={aggregate_status_parse_json}",
            f"[ci-gate-summary] aggregate_status_compact={read_compact_line(aggregate_status_line)}",
            f"[ci-gate-summary] final_status_line={final_status_line}",
            f"[ci-gate-summary] final_status_parse={final_status_parse_json}",
            f"[ci-gate-summary] summary_line={summary_line_path}",
            f"[ci-gate-summary] ci_gate_result={ci_gate_result_json}",
            f"[ci-gate-summary] ci_gate_result_parse={ci_gate_result_parse_json}",
            f"[ci-gate-summary] ci_gate_result_line={ci_gate_result_line_path}",
            f"[ci-gate-summary] ci_gate_badge={ci_gate_badge_json}",
            f"[ci-gate-summary] ci_fail_brief_hint={ci_fail_brief_txt}",
            f"[ci-gate-summary] ci_fail_brief_exists={int(ci_fail_brief_txt.exists())}",
            f"[ci-gate-summary] ci_fail_triage_hint={ci_fail_triage_json}",
            f"[ci-gate-summary] ci_fail_triage_exists={int(ci_fail_triage_json.exists())}",
            f"[ci-gate-summary] final_status_compact={read_compact_line(final_status_line)}",
            f"[ci-gate-summary] age3_summary={age3_close_summary_md}",
        ]
    else:
        pass_lines = [
            "[ci-gate-summary] PASS",
            "[ci-gate-summary] failed_steps=(none)",
            f"[ci-gate-summary] report_index={index_report_path}",
            f"[ci-gate-summary] summary_line={summary_line_path}",
            f"[ci-gate-summary] ci_gate_result={ci_gate_result_json}",
            f"[ci-gate-summary] ci_gate_badge={ci_gate_badge_json}",
            f"[ci-gate-summary] ci_fail_brief_hint={ci_fail_brief_txt}",
            f"[ci-gate-summary] ci_fail_brief_exists={int(ci_fail_brief_txt.exists())}",
            f"[ci-gate-summary] ci_fail_triage_hint={ci_fail_triage_json}",
            f"[ci-gate-summary] ci_fail_triage_exists={int(ci_fail_triage_json.exists())}",
            f"[ci-gate-summary] age3_status={age3_close_status_json}",
            f"[ci-gate-summary] age4_status={age4_close_report}",
        ]
    for line in pass_lines:
        print(line)
    write_summary(summary_path, pass_lines)
    summary_report_check_rc = check_ci_gate_summary_report(require_pass=True)
    if summary_report_check_rc != 0:
        return 2
    summary_failure_check_rc = check_ci_gate_failure_summary(require_pass=True)
    if summary_failure_check_rc != 0:
        return 2
    summary_compact = resolve_summary_compact(
        ci_gate_result_line_path,
        final_status_parse_json,
        final_status_line,
    )
    write_summary_line(summary_line_path, summary_compact)
    print(f"[ci-gate-summary-line] {summary_compact}")
    print("[ci-gate] all checks passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
