#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SCHEMA = "ddn.fixed64.cross_platform_threeway_gate.v1"
DARWIN_REPORT_NAME = "fixed64_cross_platform_probe_darwin.detjson"
PROGRESS_ENV_KEY = "DDN_FIXED64_CROSS_PLATFORM_THREEWAY_GATE_PROGRESS_JSON"
MATRIX_PROGRESS_ENV_KEY = "DDN_FIXED64_CROSS_PLATFORM_MATRIX_CHECK_PROGRESS_JSON"

DEFAULT_REPORT_OUT_REL = "build/reports/fixed64_cross_platform_threeway_gate.detjson"
DEFAULT_WINDOWS_REPORT_REL = "build/reports/fixed64_cross_platform_probe_windows.detjson"
DEFAULT_LINUX_REPORT_REL = "build/reports/fixed64_cross_platform_probe_linux.detjson"
DEFAULT_DARWIN_REPORT_REL = "build/reports/fixed64_cross_platform_probe_darwin.detjson"


def resolve_preferred_report_dir() -> Path:
    if os.name == "nt":
        preferred = Path("I:/home/urihanl/ddn/codex/build/reports")
        fallback = Path("C:/ddn/codex/build/reports")
        for candidate in (preferred, fallback):
            try:
                candidate.mkdir(parents=True, exist_ok=True)
            except OSError:
                continue
            return candidate
    local = ROOT / "build" / "reports"
    local.mkdir(parents=True, exist_ok=True)
    return local


def resolve_default_report_path(arg_value: str, default_rel: str, report_dir: Path) -> Path:
    text = str(arg_value or "").strip()
    default_rel_norm = default_rel.replace("\\", "/")
    text_norm = text.replace("\\", "/")
    if not text or text_norm == default_rel_norm:
        return (report_dir / Path(default_rel).name).resolve()
    return Path(text).resolve()


def write_report(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def write_progress_snapshot(
    path_text: str,
    *,
    status: str,
    current_stage: str,
    last_completed_stage: str,
    total_elapsed_ms: int,
) -> None:
    if not str(path_text).strip():
        return
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "ddn.fixed64.cross_platform_threeway_gate.progress.v1",
        "status": status,
        "current_stage": current_stage,
        "last_completed_stage": last_completed_stage,
        "total_elapsed_ms": int(total_elapsed_ms),
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def parse_iso8601_utc(value: str) -> datetime | None:
    text = value.strip()
    if not text:
        return None
    if text.endswith("Z"):
        text = text[:-1] + "+00:00"
    try:
        parsed = datetime.fromisoformat(text)
    except ValueError:
        return None
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def check_report_freshness(
    named_reports: list[tuple[str, Path]],
    *,
    max_age_minutes: float,
) -> tuple[list[dict[str, object]], list[str]]:
    details: list[dict[str, object]] = []
    errors: list[str] = []
    now_utc = datetime.now(timezone.utc)
    max_age_seconds = max_age_minutes * 60.0

    for name, path in named_reports:
        detail: dict[str, object] = {
            "name": name,
            "path": str(path),
            "ok": False,
            "generated_at_utc": "",
            "age_seconds": None,
        }
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            errors.append(f"{name} report load failed")
            details.append(detail)
            continue
        if not isinstance(payload, dict):
            errors.append(f"{name} report is not object")
            details.append(detail)
            continue
        generated_raw = str(payload.get("generated_at_utc", "")).strip()
        detail["generated_at_utc"] = generated_raw
        generated_at = parse_iso8601_utc(generated_raw)
        if generated_at is None:
            errors.append(f"{name} report generated_at_utc parse failed")
            details.append(detail)
            continue
        age_seconds = max(0.0, (now_utc - generated_at).total_seconds())
        detail["age_seconds"] = age_seconds
        if age_seconds > max_age_seconds:
            errors.append(
                f"{name} report is stale: age_seconds={age_seconds:.1f} > max_age_seconds={max_age_seconds:.1f}"
            )
            details.append(detail)
            continue
        detail["ok"] = True
        details.append(detail)

    return details, errors


def run_resolve_threeway_inputs(
    *,
    python_exe: str,
    report_dir: Path,
    darwin_report: Path,
    resolve_json_out: str,
    strict_invalid: bool,
    require_when_env: str,
    candidates: list[str],
) -> tuple[bool, dict]:
    resolve_report = (
        Path(resolve_json_out).resolve()
        if resolve_json_out.strip()
        else report_dir / "fixed64_threeway_inputs.detjson"
    )
    cmd = [
        python_exe,
        "tools/scripts/resolve_fixed64_threeway_inputs.py",
        "--report-dir",
        str(report_dir),
        "--json-out",
        str(resolve_report),
    ]
    if strict_invalid:
        cmd.append("--strict-invalid")
    if require_when_env.strip():
        cmd.extend(["--require-when-env", require_when_env.strip()])
    for candidate in candidates:
        item = str(candidate or "").strip()
        if item:
            cmd.extend(["--candidate", item])

    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )

    result: dict[str, object] = {
        "cmd": cmd,
        "returncode": int(proc.returncode),
        "stdout": (proc.stdout or "").strip().splitlines(),
        "stderr": (proc.stderr or "").strip().splitlines(),
        "json_out": str(resolve_report),
        "ok": False,
    }

    resolve_payload: dict | None = None
    try:
        parsed = json.loads(resolve_report.read_text(encoding="utf-8"))
        if isinstance(parsed, dict):
            resolve_payload = parsed
    except Exception:
        resolve_payload = None
    result["payload"] = resolve_payload if isinstance(resolve_payload, dict) else {}

    if proc.returncode != 0:
        result["reason"] = "resolve command failed"
        return False, result

    if not isinstance(resolve_payload, dict) or not bool(resolve_payload.get("ok", False)):
        result["reason"] = "resolve report invalid or ok=false"
        return False, result
    resolve_status = str(resolve_payload.get("status", "")).strip().lower()
    if resolve_status != "staged":
        result["reason"] = f"resolve status is not staged: {resolve_status or '-'}"
        return False, result

    staged_default = (report_dir / DARWIN_REPORT_NAME).resolve()
    copied_to_darwin_report = ""
    if darwin_report.resolve() != staged_default and not darwin_report.exists() and staged_default.exists():
        try:
            darwin_report.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(staged_default, darwin_report)
            copied_to_darwin_report = str(darwin_report)
        except Exception as exc:
            result["reason"] = f"copy staged darwin report failed: {exc}"
            return False, result

    result["copied_to_darwin_report"] = copied_to_darwin_report
    result["ok"] = True
    result["reason"] = "-"
    return True, result


def main() -> int:
    parser = argparse.ArgumentParser(
        description="fixed64 cross-platform 3way(windows/linux/darwin) gate check"
    )
    parser.add_argument("--python", default=sys.executable, help="python executable path")
    parser.add_argument(
        "--report-out",
        default="",
        help=f"detjson 출력 경로(기본: {DEFAULT_REPORT_OUT_REL})",
    )
    parser.add_argument(
        "--windows-report",
        default=DEFAULT_WINDOWS_REPORT_REL,
        help="windows probe report path",
    )
    parser.add_argument(
        "--linux-report",
        default=DEFAULT_LINUX_REPORT_REL,
        help="linux probe report path",
    )
    parser.add_argument(
        "--darwin-report",
        default=DEFAULT_DARWIN_REPORT_REL,
        help="darwin probe report path",
    )
    parser.add_argument(
        "--require-darwin",
        action="store_true",
        help="darwin report 누락을 실패로 처리",
    )
    parser.add_argument(
        "--max-report-age-minutes",
        type=float,
        default=0.0,
        help="report generated_at_utc 최대 허용 경과 시간(분). 0 이하면 비활성",
    )
    parser.add_argument(
        "--resolve-threeway-inputs",
        action="store_true",
        help="run resolve_fixed64_threeway_inputs.py before checking reports",
    )
    parser.add_argument(
        "--resolve-inputs-json-out",
        default="",
        help="resolve step detjson output path (default: <report-dir>/fixed64_threeway_inputs.detjson)",
    )
    parser.add_argument(
        "--resolve-inputs-strict-invalid",
        action="store_true",
        help="pass --strict-invalid to resolve_fixed64_threeway_inputs.py",
    )
    parser.add_argument(
        "--resolve-inputs-require-when-env",
        default="",
        help="pass --require-when-env value to resolve_fixed64_threeway_inputs.py",
    )
    parser.add_argument(
        "--resolve-input-candidate",
        action="append",
        default=[],
        help="additional candidate path passed to resolve_fixed64_threeway_inputs.py (repeatable)",
    )
    args = parser.parse_args()
    progress_path = str(os.environ.get(PROGRESS_ENV_KEY, "")).strip()
    started_at = time.perf_counter()
    current_stage = "-"
    last_completed_stage = "-"

    def update_progress(status: str) -> None:
        write_progress_snapshot(
            progress_path,
            status=status,
            current_stage=current_stage,
            last_completed_stage=last_completed_stage,
            total_elapsed_ms=int((time.perf_counter() - started_at) * 1000),
        )

    def transition_stage(name: str) -> None:
        nonlocal current_stage, last_completed_stage
        normalized = str(name).strip() or "-"
        if current_stage not in ("", "-") and current_stage != normalized:
            last_completed_stage = current_stage
        current_stage = normalized
        update_progress("running")

    def complete_stage(name: str) -> None:
        nonlocal current_stage, last_completed_stage
        normalized = str(name).strip() or "-"
        if normalized not in ("", "-"):
            last_completed_stage = normalized
        current_stage = "-"
        update_progress("running")

    if args.max_report_age_minutes < 0:
        print("[fixed64-3way-gate] failed invalid --max-report-age-minutes", file=sys.stderr)
        print(" - --max-report-age-minutes must be >= 0", file=sys.stderr)
        return 1

    update_progress("running")

    report_dir = resolve_preferred_report_dir()
    report_out = (
        Path(args.report_out).resolve()
        if args.report_out.strip()
        else report_dir / Path(DEFAULT_REPORT_OUT_REL).name
    )
    windows_report = resolve_default_report_path(args.windows_report, DEFAULT_WINDOWS_REPORT_REL, report_dir)
    linux_report = resolve_default_report_path(args.linux_report, DEFAULT_LINUX_REPORT_REL, report_dir)
    darwin_report = resolve_default_report_path(args.darwin_report, DEFAULT_DARWIN_REPORT_REL, report_dir)

    payload: dict[str, object] = {
        "schema": SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": False,
        "status": "fail",
        "reason": "-",
        "reports": {
            "windows": str(windows_report),
            "linux": str(linux_report),
            "darwin": str(darwin_report),
        },
        "freshness": {
            "enabled": bool(args.max_report_age_minutes > 0),
            "max_age_minutes": float(args.max_report_age_minutes),
            "details": [],
            "errors": [],
        },
        "resolve_inputs": {
            "enabled": bool(args.resolve_threeway_inputs),
            "ok": False,
            "reason": "disabled",
        },
    }

    if args.resolve_threeway_inputs:
        transition_stage("resolve_inputs")
        resolve_ok, resolve_result = run_resolve_threeway_inputs(
            python_exe=args.python,
            report_dir=darwin_report.parent.resolve(),
            darwin_report=darwin_report,
            resolve_json_out=args.resolve_inputs_json_out,
            strict_invalid=bool(args.resolve_inputs_strict_invalid),
            require_when_env=str(args.resolve_inputs_require_when_env or ""),
            candidates=list(args.resolve_input_candidate or []),
        )
        payload["resolve_inputs"] = resolve_result
        if not resolve_ok:
            payload["reason"] = "resolve threeway inputs failed"
            write_report(report_out, payload)
            complete_stage("resolve_inputs")
            update_progress("fail")
            print(f"[fixed64-3way-gate] failed report={report_out}", file=sys.stderr)
            print(" - resolve threeway inputs failed", file=sys.stderr)
            for item in list(resolve_result.get("stderr", []) or [])[:20]:
                print(f" - {item}", file=sys.stderr)
            return 1
        complete_stage("resolve_inputs")

    transition_stage("check_required_reports")
    missing_required: list[str] = []
    if not windows_report.exists():
        missing_required.append("windows")
    if not linux_report.exists():
        missing_required.append("linux")
    if missing_required:
        payload["reason"] = f"missing required reports: {','.join(missing_required)}"
        write_report(report_out, payload)
        complete_stage("check_required_reports")
        update_progress("fail")
        print(f"[fixed64-3way-gate] failed report={report_out}", file=sys.stderr)
        print(f" - missing required reports: {','.join(missing_required)}", file=sys.stderr)
        return 1
    complete_stage("check_required_reports")

    freshness_targets: list[tuple[str, Path]] = [
        ("windows", windows_report),
        ("linux", linux_report),
    ]
    if darwin_report.exists():
        freshness_targets.append(("darwin", darwin_report))
    if args.max_report_age_minutes > 0:
        transition_stage("freshness_check")
        freshness_details, freshness_errors = check_report_freshness(
            freshness_targets,
            max_age_minutes=args.max_report_age_minutes,
        )
        payload["freshness"] = {
            "enabled": True,
            "max_age_minutes": float(args.max_report_age_minutes),
            "details": freshness_details,
            "errors": freshness_errors,
        }
        if freshness_errors:
            payload["reason"] = "report freshness check failed"
            write_report(report_out, payload)
            complete_stage("freshness_check")
            update_progress("fail")
            print(f"[fixed64-3way-gate] failed report={report_out}", file=sys.stderr)
            for item in freshness_errors[:20]:
                print(f" - {item}", file=sys.stderr)
            return 1
        complete_stage("freshness_check")

    darwin_exists = darwin_report.exists()
    transition_stage("check_darwin_requirements")
    if not darwin_exists and args.require_darwin:
        payload["reason"] = "darwin report missing"
        write_report(report_out, payload)
        complete_stage("check_darwin_requirements")
        update_progress("fail")
        print(f"[fixed64-3way-gate] failed report={report_out}", file=sys.stderr)
        print(" - darwin report missing", file=sys.stderr)
        return 1
    complete_stage("check_darwin_requirements")

    matrix_reports: list[Path] = [windows_report, linux_report]
    required_systems = ["windows", "linux"]
    if darwin_exists:
        matrix_reports.append(darwin_report)
        required_systems.append("darwin")

    cmd = [args.python, "tests/run_fixed64_cross_platform_matrix_check.py"]
    for matrix_report in matrix_reports:
        cmd.extend(["--report", str(matrix_report)])
    cmd.extend(["--require-systems", ",".join(required_systems)])

    matrix_progress_path = report_out.with_name(report_out.stem + ".matrix.progress.detjson")
    matrix_progress_path.unlink(missing_ok=True)
    matrix_env = os.environ.copy()
    matrix_env[MATRIX_PROGRESS_ENV_KEY] = str(matrix_progress_path)

    def read_child_stage(progress_path: Path) -> str:
        if not progress_path.exists():
            return "-"
        try:
            child_payload = json.loads(progress_path.read_text(encoding="utf-8"))
        except Exception:
            return "-"
        if not isinstance(child_payload, dict):
            return "-"
        current = str(child_payload.get("current_stage", "")).strip() or "-"
        if current not in ("", "-"):
            return current
        last_completed = str(child_payload.get("last_completed_stage", "")).strip() or "-"
        return last_completed if last_completed not in ("", "-") else "-"

    transition_stage("matrix_check.spawn_process")
    proc = subprocess.Popen(
        cmd,
        cwd=ROOT,
        env=matrix_env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    transition_stage("matrix_check.wait_exit")
    last_child_stage = "-"
    while True:
        child_stage = read_child_stage(matrix_progress_path)
        if child_stage not in ("", "-") and child_stage != last_child_stage:
            last_child_stage = child_stage
            transition_stage(f"matrix_check.wait_exit.{child_stage}")
        if proc.poll() is not None:
            break
        time.sleep(0.005)
    child_stage = read_child_stage(matrix_progress_path)
    if child_stage not in ("", "-") and child_stage != last_child_stage:
        transition_stage(f"matrix_check.wait_exit.{child_stage}")
    stdout_text, stderr_text = proc.communicate()
    transition_stage("matrix_check.collect_output")
    payload["cmd"] = cmd
    payload["returncode"] = int(proc.returncode)
    payload["stdout"] = (stdout_text or "").strip().splitlines()
    payload["stderr"] = (stderr_text or "").strip().splitlines()
    if proc.returncode != 0:
        payload["reason"] = "matrix check failed"
        write_report(report_out, payload)
        complete_stage("matrix_check.collect_output")
        transition_stage("matrix_check.validate_result")
        complete_stage("matrix_check.validate_result")
        update_progress("fail")
        print(f"[fixed64-3way-gate] failed report={report_out}", file=sys.stderr)
        if stdout_text:
            print(stdout_text, end="" if stdout_text.endswith("\n") else "\n", file=sys.stderr)
        if stderr_text:
            print(stderr_text, end="" if stderr_text.endswith("\n") else "\n", file=sys.stderr)
        return int(proc.returncode)

    payload["ok"] = True
    if darwin_exists:
        payload["status"] = "pass_3way"
        payload["reason"] = "-"
    else:
        payload["status"] = "pending_darwin"
        payload["reason"] = "darwin report missing"
    write_report(report_out, payload)
    complete_stage("matrix_check.collect_output")
    transition_stage("matrix_check.validate_result")
    complete_stage("matrix_check.validate_result")
    update_progress("pass")
    if darwin_exists:
        print(f"[fixed64-3way-gate] ok report={report_out}")
    else:
        print(f"[fixed64-3way-gate] pending darwin report={darwin_report}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
