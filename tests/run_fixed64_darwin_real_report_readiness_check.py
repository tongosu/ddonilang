#!/usr/bin/env python
from __future__ import annotations

import argparse
import contextlib
import importlib.util
import io
import json
import os
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Callable


ROOT = Path(__file__).resolve().parent.parent
SCHEMA = "ddn.fixed64.darwin_real_report_readiness.v1"
PROGRESS_ENV_KEY = "DDN_FIXED64_DARWIN_REAL_REPORT_READINESS_PROGRESS_JSON"
THREEWAY_PROGRESS_ENV_KEY = "DDN_FIXED64_CROSS_PLATFORM_THREEWAY_GATE_PROGRESS_JSON"
CONTRACT_MODULE_NAME = "_ddn_fixed64_darwin_real_report_readiness_contract_fastpath"
CONTRACT_MODULE_CACHE = None
DARWIN_REPORT_NAME = "fixed64_cross_platform_probe_darwin.detjson"
DEFAULT_DARWIN_REPORT_REL = "build/reports/fixed64_cross_platform_probe_darwin.detjson"
DEFAULT_WINDOWS_REPORT_REL = "build/reports/fixed64_cross_platform_probe_windows.detjson"
DEFAULT_LINUX_REPORT_REL = "build/reports/fixed64_cross_platform_probe_linux.detjson"
DEFAULT_INPUTS_REPORT_REL = "build/reports/fixed64_threeway_inputs.detjson"
DEFAULT_JSON_OUT_REL = "build/reports/fixed64_darwin_real_report_readiness.detjson"


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


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
        "schema": "ddn.fixed64.darwin_real_report_readiness.progress.v1",
        "status": status,
        "current_stage": current_stage,
        "last_completed_stage": last_completed_stage,
        "total_elapsed_ms": int(total_elapsed_ms),
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def patched_environ(env_patch: dict[str, str]):
    class _PatchedEnviron:
        def __enter__(self_inner):
            self_inner._originals: dict[str, str | None] = {}
            for key, value in env_patch.items():
                self_inner._originals[key] = os.environ.get(key)
                os.environ[key] = value
            return self_inner

        def __exit__(self_inner, exc_type, exc, tb):
            for key, original in self_inner._originals.items():
                if original is None:
                    os.environ.pop(key, None)
                else:
                    os.environ[key] = original
            return False

    return _PatchedEnviron()


def load_contract_module():
    global CONTRACT_MODULE_CACHE
    if CONTRACT_MODULE_CACHE is not None:
        return CONTRACT_MODULE_CACHE
    script_path = ROOT / "tests" / "run_fixed64_darwin_real_report_contract_check.py"
    spec = importlib.util.spec_from_file_location(CONTRACT_MODULE_NAME, script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"unable to load contract module: {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    CONTRACT_MODULE_CACHE = module
    return module


def run_cmd(
    cmd: list[str],
    *,
    env: dict[str, str] | None = None,
    transition_stage: Callable[[str], None] | None = None,
    stage_prefix: str = "",
    child_progress_path: Path | None = None,
) -> subprocess.CompletedProcess[str]:
    def transition(name: str) -> None:
        if callable(transition_stage) and str(name).strip():
            transition_stage(name)

    def read_child_stage(progress_path: Path) -> str:
        if not progress_path.exists():
            return "-"
        try:
            payload = json.loads(progress_path.read_text(encoding="utf-8"))
        except Exception:
            return "-"
        if not isinstance(payload, dict):
            return "-"
        current_stage = str(payload.get("current_stage", "")).strip() or "-"
        if current_stage not in ("", "-"):
            return current_stage
        last_completed_stage = str(payload.get("last_completed_stage", "")).strip() or "-"
        return last_completed_stage if last_completed_stage not in ("", "-") else "-"

    if stage_prefix:
        transition(f"{stage_prefix}.spawn_process")
    proc = subprocess.Popen(
        cmd,
        cwd=ROOT,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if stage_prefix:
        transition(f"{stage_prefix}.wait_exit")
    last_child_stage = "-"
    while True:
        if child_progress_path is not None:
            child_stage = read_child_stage(child_progress_path)
            if child_stage not in ("", "-") and child_stage != last_child_stage:
                last_child_stage = child_stage
                transition(f"{stage_prefix}.wait_exit.{child_stage}")
        if proc.poll() is not None:
            break
        time.sleep(0.005)
    if child_progress_path is not None:
        child_stage = read_child_stage(child_progress_path)
        if child_stage not in ("", "-") and child_stage != last_child_stage:
            transition(f"{stage_prefix}.wait_exit.{child_stage}")
    stdout_text, stderr_text = proc.communicate()
    if stage_prefix:
        transition(f"{stage_prefix}.collect_output")
    return subprocess.CompletedProcess(
        cmd,
        int(proc.returncode),
        stdout_text,
        stderr_text,
    )


def run_contract_check_fast_path(
    *,
    report_path: Path,
    inputs_report: Path,
    max_age_minutes: float,
    json_out: Path,
    transition_stage: Callable[[str], None] | None = None,
    stage_prefix: str = "contract_cmd",
) -> subprocess.CompletedProcess[str]:
    def transition(name: str) -> None:
        if callable(transition_stage) and str(name).strip():
            transition_stage(name)

    module = load_contract_module()
    argv = [
        "tests/run_fixed64_darwin_real_report_contract_check.py",
        "--report",
        str(report_path),
        "--inputs-report",
        str(inputs_report),
        "--max-age-minutes",
        str(float(max_age_minutes)),
        "--json-out",
        str(json_out),
    ]
    stdout_buffer = io.StringIO()
    stderr_buffer = io.StringIO()
    transition(f"{stage_prefix}.fast_path")
    env_patch = {"DDN_ENABLE_DARWIN_PROBE": "1"}
    with patched_environ(env_patch):
        old_argv = sys.argv[:]
        try:
            sys.argv = argv
            with contextlib.redirect_stdout(stdout_buffer), contextlib.redirect_stderr(stderr_buffer):
                returncode = int(module.main())
        finally:
            sys.argv = old_argv
    return subprocess.CompletedProcess(argv, returncode, stdout_buffer.getvalue(), stderr_buffer.getvalue())


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
        description="Darwin 실측 리포트 반입 readiness 점검(계약 + 가능한 경우 3way 게이트)."
    )
    parser.add_argument("--python", default=sys.executable, help="python executable path")
    parser.add_argument(
        "--darwin-report",
        default=DEFAULT_DARWIN_REPORT_REL,
        help="darwin probe report path",
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
        "--inputs-report",
        default=DEFAULT_INPUTS_REPORT_REL,
        help="fixed64 threeway inputs report path",
    )
    parser.add_argument(
        "--max-age-minutes",
        type=float,
        default=360.0,
        help="max age minutes for darwin contract/3way freshness",
    )
    parser.add_argument(
        "--json-out",
        default=DEFAULT_JSON_OUT_REL,
        help="readiness detjson path",
    )
    parser.add_argument(
        "--resolve-threeway-inputs",
        action="store_true",
        help="run resolve_fixed64_threeway_inputs.py before contract check",
    )
    parser.add_argument(
        "--resolve-inputs-json-out",
        default="",
        help="resolve step detjson output path (default: <darwin-report-dir>/fixed64_threeway_inputs.detjson)",
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

    if args.max_age_minutes < 0:
        print("[fixed64-darwin-readiness] fail: --max-age-minutes must be >= 0", file=sys.stderr)
        return 1

    update_progress("running")

    report_dir = resolve_preferred_report_dir()
    darwin_report = resolve_default_report_path(args.darwin_report, DEFAULT_DARWIN_REPORT_REL, report_dir)
    windows_report = resolve_default_report_path(args.windows_report, DEFAULT_WINDOWS_REPORT_REL, report_dir)
    linux_report = resolve_default_report_path(args.linux_report, DEFAULT_LINUX_REPORT_REL, report_dir)
    inputs_report = resolve_default_report_path(args.inputs_report, DEFAULT_INPUTS_REPORT_REL, report_dir)
    json_out = resolve_default_report_path(args.json_out, DEFAULT_JSON_OUT_REL, report_dir)
    threeway_report = json_out.with_name("fixed64_cross_platform_threeway_gate.readiness.detjson")

    payload: dict[str, object] = {
        "schema": SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": False,
        "status": "fail",
        "reason": "-",
        "reports": {
            "darwin": str(darwin_report),
            "windows": str(windows_report),
            "linux": str(linux_report),
            "inputs": str(inputs_report),
            "threeway_out": str(threeway_report),
        },
        "contract_check": {},
        "threeway_check": {
            "attempted": False,
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
            payload["status"] = "fail_resolve"
            payload["reason"] = "resolve threeway inputs failed"
            write_json(json_out, payload)
            complete_stage("resolve_inputs")
            update_progress("fail")
            print(f"[fixed64-darwin-readiness] fail_resolve out={json_out}", file=sys.stderr)
            return 1
        complete_stage("resolve_inputs")

    env = os.environ.copy()
    env["DDN_ENABLE_DARWIN_PROBE"] = "1"

    contract_out = json_out.with_name(json_out.stem + ".contract.detjson")
    contract_proc = run_contract_check_fast_path(
        report_path=darwin_report,
        inputs_report=inputs_report,
        max_age_minutes=float(args.max_age_minutes),
        json_out=contract_out,
        transition_stage=transition_stage,
        stage_prefix="contract_cmd",
    )
    complete_stage("contract_cmd.fast_path")
    transition_stage("contract_cmd.validate_result")
    payload["contract_check"] = {
        "cmd": contract_proc.args if isinstance(contract_proc.args, list) else [str(contract_proc.args)],
        "returncode": int(contract_proc.returncode),
        "stdout": (contract_proc.stdout or "").strip().splitlines(),
        "stderr": (contract_proc.stderr or "").strip().splitlines(),
    }
    if contract_proc.returncode != 0:
        payload["status"] = "fail_contract"
        payload["reason"] = "darwin real report contract failed"
        write_json(json_out, payload)
        complete_stage("contract_cmd.validate_result")
        update_progress("fail")
        print(f"[fixed64-darwin-readiness] fail_contract out={json_out}", file=sys.stderr)
        return int(contract_proc.returncode) if int(contract_proc.returncode) != 0 else 1
    complete_stage("contract_cmd.validate_result")

    transition_stage("check_two_way_reports")
    if not windows_report.exists() or not linux_report.exists():
        missing: list[str] = []
        if not windows_report.exists():
            missing.append("windows")
        if not linux_report.exists():
            missing.append("linux")
        payload["ok"] = True
        payload["status"] = "pass_contract_only"
        payload["reason"] = f"missing two-way reports: {','.join(missing)}"
        write_json(json_out, payload)
        complete_stage("check_two_way_reports")
        update_progress("pass")
        print(
            "[fixed64-darwin-readiness] pass_contract_only "
            f"missing={','.join(missing)} out={json_out}"
        )
        return 0
    complete_stage("check_two_way_reports")

    threeway_cmd = [
        args.python,
        "tests/run_fixed64_cross_platform_threeway_gate.py",
        "--report-out",
        str(threeway_report),
        "--windows-report",
        str(windows_report),
        "--linux-report",
        str(linux_report),
        "--darwin-report",
        str(darwin_report),
        "--require-darwin",
        "--max-report-age-minutes",
        str(float(args.max_age_minutes)),
    ]
    threeway_progress_path = threeway_report.with_name(threeway_report.stem + ".progress.detjson")
    threeway_progress_path.unlink(missing_ok=True)
    threeway_env = env.copy()
    threeway_env[THREEWAY_PROGRESS_ENV_KEY] = str(threeway_progress_path)
    threeway_proc = run_cmd(
        threeway_cmd,
        env=threeway_env,
        transition_stage=transition_stage,
        stage_prefix="threeway_cmd",
        child_progress_path=threeway_progress_path,
    )
    complete_stage("threeway_cmd.collect_output")
    transition_stage("threeway_cmd.validate_result")
    payload["threeway_check"] = {
        "attempted": True,
        "cmd": threeway_cmd,
        "returncode": int(threeway_proc.returncode),
        "stdout": (threeway_proc.stdout or "").strip().splitlines(),
        "stderr": (threeway_proc.stderr or "").strip().splitlines(),
    }
    if threeway_proc.returncode != 0:
        payload["status"] = "fail_3way"
        payload["reason"] = "3way gate failed after contract pass"
        write_json(json_out, payload)
        complete_stage("threeway_cmd.validate_result")
        update_progress("fail")
        print(f"[fixed64-darwin-readiness] fail_3way out={json_out}", file=sys.stderr)
        return int(threeway_proc.returncode) if int(threeway_proc.returncode) != 0 else 1

    payload["ok"] = True
    payload["status"] = "pass_3way"
    payload["reason"] = "-"
    write_json(json_out, payload)
    complete_stage("threeway_cmd.validate_result")
    update_progress("pass")
    print(f"[fixed64-darwin-readiness] pass_3way out={json_out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
