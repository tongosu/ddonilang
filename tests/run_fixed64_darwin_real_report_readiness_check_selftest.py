#!/usr/bin/env python
from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import os
import subprocess
import sys
import tempfile
import time
import zipfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Callable


ROOT = Path(__file__).resolve().parent.parent
READINESS_SCHEMA = "ddn.fixed64.darwin_real_report_readiness.v1"
PROBE_SCHEMA = "ddn.fixed64.cross_platform_probe.v1"
VECTOR_SCHEMA = "ddn.fixed64.determinism_vector.v1"
PROGRESS_ENV_KEY = "DDN_FIXED64_DARWIN_REAL_REPORT_READINESS_SELFTEST_PROGRESS_JSON"
READINESS_PROGRESS_ENV_KEY = "DDN_FIXED64_DARWIN_REAL_REPORT_READINESS_PROGRESS_JSON"
READINESS_FASTPATH_ENV_KEY = "DDN_FIXED64_DARWIN_REAL_REPORT_READINESS_SELFTEST_FASTPATH"
READINESS_MODULE_NAME = "_ddn_fixed64_darwin_real_report_readiness_selftest_fastpath"
CONTRACT_MODULE_NAME = "_ddn_fixed64_darwin_real_report_contract_selftest_fastpath"
SAMPLE_BLAKE3 = "3b4f486e7a86e1ba6b45a8fa89ee9998f2a05f503ae1e9c2ba1722726307e7ed"
SAMPLE_RAW = [6442450944, 2147483648, -2147483648, 2147483648, 8589934592, 2147483648, 0]
READINESS_MODULE_CACHE = None
CONTRACT_MODULE_CACHE = None


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(payload, ensure_ascii=False, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


def load_json(path: Path) -> dict | None:
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def use_readiness_fast_path() -> bool:
    value = str(os.environ.get(READINESS_FASTPATH_ENV_KEY, "1")).strip().lower()
    return value not in {"0", "false", "no", "off"}


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


def load_readiness_module():
    global READINESS_MODULE_CACHE
    if READINESS_MODULE_CACHE is not None:
        return READINESS_MODULE_CACHE
    script_path = ROOT / "tests" / "run_fixed64_darwin_real_report_readiness_check.py"
    spec = importlib.util.spec_from_file_location(READINESS_MODULE_NAME, script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"unable to load readiness module: {script_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    READINESS_MODULE_CACHE = module
    return module


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


def write_progress_snapshot(
    path_text: str,
    *,
    status: str,
    current_case: str,
    last_completed_case: str,
    current_probe: str,
    last_completed_probe: str,
    total_elapsed_ms: int,
) -> None:
    if not str(path_text).strip():
        return
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "ddn.fixed64.darwin_real_report_readiness_selftest.progress.v1",
        "status": status,
        "current_case": current_case,
        "last_completed_case": last_completed_case,
        "current_probe": current_probe,
        "last_completed_probe": last_completed_probe,
        "total_elapsed_ms": int(total_elapsed_ms),
    }
    out.write_text(
        json.dumps(payload, ensure_ascii=False, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


def make_probe(
    system: str,
    *,
    blake3_hex: str = SAMPLE_BLAKE3,
    raw_i64: list[int] | None = None,
    synthetic: bool = False,
) -> dict:
    normalized = system.strip().lower()
    if normalized == "darwin":
        release = "23.6.0"
        version = "Darwin Kernel Version 23.6.0"
        machine = "arm64"
    elif normalized == "windows":
        release = "10.0.22631"
        version = "Windows 11 Pro"
        machine = "AMD64"
    else:
        release = "6.8.0"
        version = "Linux 6.8.0-52-generic"
        machine = "x86_64"
    if synthetic:
        release = "selftest"
        version = "selftest"
        machine = "selftest"

    raw_values = list(raw_i64) if isinstance(raw_i64, list) else list(SAMPLE_RAW)
    return {
        "schema": PROBE_SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": True,
        "errors": [],
        "platform": {
            "system": system,
            "release": release,
            "version": version,
            "machine": machine,
            "python": "3.13.0",
        },
        "cmd": ["cargo", "run", "-q", "-p", "ddonirang-core", "--example", "fixed64_determinism_vector"],
        "returncode": 0,
        "probe": {
            "schema": VECTOR_SCHEMA,
            "status": "pass",
            "blake3": blake3_hex,
            "raw_i64": raw_values,
            "expected_raw_i64": raw_values,
        },
        "stdout": [
            f"schema={VECTOR_SCHEMA}",
            "status=pass",
            f"blake3={blake3_hex}",
            "raw_i64=" + ",".join(str(v) for v in raw_values),
            "expected_raw_i64=" + ",".join(str(v) for v in raw_values),
        ],
        "stderr": [],
    }


def run_readiness(
    *,
    darwin_report: Path,
    windows_report: Path,
    linux_report: Path,
    inputs_report: Path,
    json_out: Path,
    max_age_minutes: float = 360.0,
    transition_probe: Callable[[str], None] | None = None,
    resolve_threeway_inputs: bool = False,
    resolve_input_candidates: list[str] | None = None,
) -> subprocess.CompletedProcess[str]:
    def transition(name: str) -> None:
        if callable(transition_probe) and str(name).strip():
            transition_probe(name)

    if use_readiness_fast_path():
        transition("run_readiness.spawn_process")
        transition("run_readiness.wait_exit")
        transition("run_readiness.wait_exit.fast_path")
        proc = run_readiness_fast_path(
            darwin_report=darwin_report,
            windows_report=windows_report,
            linux_report=linux_report,
            inputs_report=inputs_report,
            json_out=json_out,
            max_age_minutes=max_age_minutes,
            resolve_threeway_inputs=resolve_threeway_inputs,
            resolve_input_candidates=resolve_input_candidates,
        )
        transition("run_readiness.collect_output")
        return proc

    def read_child_stage(progress_path: Path) -> str:
        child_payload = load_json(progress_path)
        if not isinstance(child_payload, dict):
            return "-"
        current_stage = str(child_payload.get("current_stage", "")).strip() or "-"
        if current_stage not in ("", "-"):
            return current_stage
        last_completed_stage = str(child_payload.get("last_completed_stage", "")).strip() or "-"
        return last_completed_stage if last_completed_stage not in ("", "-") else "-"

    readiness_progress = json_out.with_name(json_out.stem + ".runner.progress.detjson")
    readiness_progress.unlink(missing_ok=True)
    cmd = [
        sys.executable,
        "-S",
        "tests/run_fixed64_darwin_real_report_readiness_check.py",
        "--darwin-report",
        str(darwin_report),
        "--windows-report",
        str(windows_report),
        "--linux-report",
        str(linux_report),
        "--inputs-report",
        str(inputs_report),
        "--max-age-minutes",
        str(float(max_age_minutes)),
        "--json-out",
        str(json_out),
    ]
    if resolve_threeway_inputs:
        cmd.append("--resolve-threeway-inputs")
    for candidate in (resolve_input_candidates or []):
        item = str(candidate or "").strip()
        if item:
            cmd.extend(["--resolve-input-candidate", item])
    env = os.environ.copy()
    env[READINESS_PROGRESS_ENV_KEY] = str(readiness_progress)
    transition("run_readiness.spawn_process")
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
    transition("run_readiness.wait_exit")
    last_child_stage = "-"
    while True:
        if readiness_progress.exists():
            child_stage = read_child_stage(readiness_progress)
            if child_stage not in ("", "-") and child_stage != last_child_stage:
                last_child_stage = child_stage
                transition(f"run_readiness.wait_exit.{child_stage}")
        if proc.poll() is not None:
            break
        time.sleep(0.005)
    if readiness_progress.exists():
        child_stage = read_child_stage(readiness_progress)
        if child_stage not in ("", "-") and child_stage != last_child_stage:
            last_child_stage = child_stage
            transition(f"run_readiness.wait_exit.{child_stage}")
    transition("run_readiness.collect_output")
    stdout_text, stderr_text = proc.communicate()
    return subprocess.CompletedProcess(
        cmd,
        int(proc.returncode),
        stdout_text,
        stderr_text,
    )


@contextlib.contextmanager
def persistent_tmpdir(prefix: str):
    path = Path(tempfile.mkdtemp(prefix=prefix))
    yield path


def run_readiness_fast_path(
    *,
    darwin_report: Path,
    windows_report: Path,
    linux_report: Path,
    inputs_report: Path,
    json_out: Path,
    max_age_minutes: float = 360.0,
    resolve_threeway_inputs: bool = False,
    resolve_input_candidates: list[str] | None = None,
) -> subprocess.CompletedProcess[str]:
    module = load_readiness_module()
    readiness_progress = json_out.with_name(json_out.stem + ".runner.progress.detjson")
    readiness_progress.unlink(missing_ok=True)
    argv = [
        "tests/run_fixed64_darwin_real_report_readiness_check.py",
        "--darwin-report",
        str(darwin_report),
        "--windows-report",
        str(windows_report),
        "--linux-report",
        str(linux_report),
        "--inputs-report",
        str(inputs_report),
        "--max-age-minutes",
        str(float(max_age_minutes)),
        "--json-out",
        str(json_out),
    ]
    if resolve_threeway_inputs:
        argv.append("--resolve-threeway-inputs")
    for candidate in (resolve_input_candidates or []):
        item = str(candidate or "").strip()
        if item:
            argv.extend(["--resolve-input-candidate", item])
    stdout_buffer = io.StringIO()
    stderr_buffer = io.StringIO()
    with patched_environ({READINESS_PROGRESS_ENV_KEY: str(readiness_progress)}):
        old_argv = sys.argv[:]
        try:
            sys.argv = argv
            with contextlib.redirect_stdout(stdout_buffer), contextlib.redirect_stderr(stderr_buffer):
                returncode = int(module.main())
        finally:
            sys.argv = old_argv
    return subprocess.CompletedProcess(argv, returncode, stdout_buffer.getvalue(), stderr_buffer.getvalue())


def run_contract_check_fast_path(
    *,
    report_path: Path,
    inputs_report: Path,
    json_out: Path,
    max_age_minutes: float,
) -> subprocess.CompletedProcess[str]:
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


def run_readiness_contract_only_fast_path(
    *,
    darwin_report: Path,
    windows_report: Path,
    linux_report: Path,
    inputs_report: Path,
    json_out: Path,
    max_age_minutes: float = 360.0,
) -> subprocess.CompletedProcess[str]:
    contract_out = json_out.with_name(json_out.stem + ".contract.detjson")
    contract_proc = run_contract_check_fast_path(
        report_path=darwin_report,
        inputs_report=inputs_report,
        json_out=contract_out,
        max_age_minutes=max_age_minutes,
    )
    payload: dict[str, object] = {
        "schema": READINESS_SCHEMA,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": False,
        "status": "fail",
        "reason": "-",
        "reports": {
            "darwin": str(darwin_report.resolve()),
            "windows": str(windows_report.resolve()),
            "linux": str(linux_report.resolve()),
            "inputs": str(inputs_report.resolve()),
            "threeway_out": str(json_out.with_name("fixed64_cross_platform_threeway_gate.readiness.detjson")),
        },
        "contract_check": {
            "cmd": contract_proc.args if isinstance(contract_proc.args, list) else [str(contract_proc.args)],
            "returncode": int(contract_proc.returncode),
            "stdout": (contract_proc.stdout or "").strip().splitlines(),
            "stderr": (contract_proc.stderr or "").strip().splitlines(),
        },
        "threeway_check": {
            "attempted": False,
        },
    }
    if contract_proc.returncode != 0:
        payload["status"] = "fail_contract"
        payload["reason"] = "darwin real report contract failed"
        write_json(json_out, payload)
        return subprocess.CompletedProcess(
            ["readiness-contract-only-fast-path", str(darwin_report)],
            int(contract_proc.returncode) if int(contract_proc.returncode) != 0 else 1,
            "",
            f"[fixed64-darwin-readiness] fail_contract out={json_out}\n",
        )

    missing: list[str] = []
    if not windows_report.exists():
        missing.append("windows")
    if not linux_report.exists():
        missing.append("linux")
    payload["ok"] = True
    payload["status"] = "pass_contract_only"
    payload["reason"] = f"missing two-way reports: {','.join(missing)}"
    write_json(json_out, payload)
    stdout_text = (
        "[fixed64-darwin-readiness] pass_contract_only "
        f"missing={','.join(missing)} out={json_out}\n"
    )
    return subprocess.CompletedProcess(
        ["readiness-contract-only-fast-path", str(darwin_report)],
        0,
        stdout_text,
        "",
    )


def main() -> int:
    progress_path = str(os.environ.get(PROGRESS_ENV_KEY, "")).strip()
    started_at = time.perf_counter()
    current_case = "-"
    last_completed_case = "-"
    current_probe = "-"
    last_completed_probe = "-"

    def update_progress(status: str) -> None:
        write_progress_snapshot(
            progress_path,
            status=status,
            current_case=current_case,
            last_completed_case=last_completed_case,
            current_probe=current_probe,
            last_completed_probe=last_completed_probe,
            total_elapsed_ms=int((time.perf_counter() - started_at) * 1000),
        )

    def start_case(name: str) -> None:
        nonlocal current_case, current_probe
        current_case = name
        current_probe = "-"
        update_progress("running")

    def complete_case(name: str) -> None:
        nonlocal current_case, last_completed_case, current_probe
        current_case = "-"
        current_probe = "-"
        last_completed_case = name
        update_progress("running")

    def start_probe(name: str) -> None:
        nonlocal current_probe
        current_probe = name
        update_progress("running")

    def transition_probe(name: str) -> None:
        nonlocal current_probe, last_completed_probe
        normalized = str(name).strip() or "-"
        if current_probe not in ("", "-") and current_probe != normalized:
            last_completed_probe = current_probe
        current_probe = normalized
        update_progress("running")

    def complete_probe(name: str) -> None:
        nonlocal current_probe, last_completed_probe
        current_probe = "-"
        last_completed_probe = name
        update_progress("running")

    update_progress("running")
    with persistent_tmpdir("fixed64_darwin_readiness_selftest_") as base:
        darwin_report = base / "probe_darwin.detjson"
        windows_report = base / "probe_windows.detjson"
        linux_report = base / "probe_linux.detjson"
        inputs_report = base / "fixed64_threeway_inputs.detjson"
        readiness_out = base / "fixed64_darwin_readiness.detjson"

        # 1) contract fail: darwin report missing
        start_case("contract_fail_should_fail")
        write_json(inputs_report, {"schema": "ddn.fixed64.threeway_inputs.v1", "target_report": str(darwin_report)})
        windows_report.unlink(missing_ok=True)
        linux_report.unlink(missing_ok=True)
        darwin_report.unlink(missing_ok=True)
        start_probe("run_readiness.contract_only_fast_path")
        proc_contract_fail = run_readiness_contract_only_fast_path(
            darwin_report=darwin_report,
            windows_report=windows_report,
            linux_report=linux_report,
            inputs_report=inputs_report,
            json_out=readiness_out,
            max_age_minutes=360.0,
        )
        complete_probe("run_readiness.contract_only_fast_path")
        if proc_contract_fail.returncode == 0:
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] contract fail case should fail", file=sys.stderr)
            return 1
        start_probe("validate_contract_fail")
        contract_fail_doc = load_json(readiness_out)
        if not isinstance(contract_fail_doc, dict):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] contract fail report missing", file=sys.stderr)
            return 1
        if str(contract_fail_doc.get("schema", "")) != READINESS_SCHEMA:
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] contract fail schema mismatch", file=sys.stderr)
            return 1
        if str(contract_fail_doc.get("status", "")) != "fail_contract":
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] contract fail status mismatch", file=sys.stderr)
            return 1
        if bool(contract_fail_doc.get("ok", True)):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] contract fail ok should be false", file=sys.stderr)
            return 1
        complete_probe("validate_contract_fail")
        complete_case("contract_fail_should_fail")

        # 2) pass_contract_only: darwin contract pass, windows/linux missing
        start_case("pass_contract_only_should_pass")
        write_json(darwin_report, make_probe("Darwin"))
        windows_report.unlink(missing_ok=True)
        linux_report.unlink(missing_ok=True)
        start_probe("run_readiness.contract_only_fast_path")
        proc_contract_only = run_readiness_contract_only_fast_path(
            darwin_report=darwin_report,
            windows_report=windows_report,
            linux_report=linux_report,
            inputs_report=inputs_report,
            json_out=readiness_out,
            max_age_minutes=360.0,
        )
        complete_probe("run_readiness.contract_only_fast_path")
        if proc_contract_only.returncode != 0:
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] contract-only case should pass", file=sys.stderr)
            return 1
        start_probe("validate_contract_only")
        contract_only_doc = load_json(readiness_out)
        if not isinstance(contract_only_doc, dict):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] contract-only report missing", file=sys.stderr)
            return 1
        if str(contract_only_doc.get("status", "")) != "pass_contract_only":
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] contract-only status mismatch", file=sys.stderr)
            return 1
        if not bool(contract_only_doc.get("ok", False)):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] contract-only ok should be true", file=sys.stderr)
            return 1
        threeway_contract_only = contract_only_doc.get("threeway_check")
        if not isinstance(threeway_contract_only, dict) or bool(threeway_contract_only.get("attempted", True)):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] contract-only attempted should be false", file=sys.stderr)
            return 1
        complete_probe("validate_contract_only")
        complete_case("pass_contract_only_should_pass")

        # 3) pass_contract_only with resolve: darwin report staged from candidate
        start_case("resolve_inputs_contract_only_should_pass")
        darwin_report.unlink(missing_ok=True)
        windows_report.unlink(missing_ok=True)
        linux_report.unlink(missing_ok=True)
        candidate_report = base / "incoming" / "fixed64_cross_platform_probe_darwin.detjson"
        write_json(candidate_report, make_probe("Darwin"))
        proc_resolve_contract_only = run_readiness(
            darwin_report=darwin_report,
            windows_report=windows_report,
            linux_report=linux_report,
            inputs_report=inputs_report,
            json_out=readiness_out,
            transition_probe=transition_probe,
            resolve_threeway_inputs=True,
            resolve_input_candidates=[str(candidate_report)],
        )
        complete_probe("run_readiness.collect_output")
        if proc_resolve_contract_only.returncode != 0:
            update_progress("fail")
            print(
                "[fixed64-darwin-readiness-selftest] resolve contract-only case should pass",
                file=sys.stderr,
            )
            return 1
        start_probe("validate_resolve_contract_only")
        resolve_contract_only_doc = load_json(readiness_out)
        if not isinstance(resolve_contract_only_doc, dict):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] resolve contract-only report missing", file=sys.stderr)
            return 1
        if str(resolve_contract_only_doc.get("status", "")) != "pass_contract_only":
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] resolve contract-only status mismatch", file=sys.stderr)
            return 1
        resolve_payload = resolve_contract_only_doc.get("resolve_inputs")
        if not isinstance(resolve_payload, dict) or not bool(resolve_payload.get("ok", False)):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] resolve_inputs result mismatch", file=sys.stderr)
            return 1
        if int(resolve_payload.get("returncode", -1)) != 0:
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] resolve_inputs returncode mismatch", file=sys.stderr)
            return 1
        if not darwin_report.exists():
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] darwin report was not staged by resolve", file=sys.stderr)
            return 1
        complete_probe("validate_resolve_contract_only")
        complete_case("resolve_inputs_contract_only_should_pass")

        # 4) pass_contract_only with resolve(summary->zip fallback)
        start_case("resolve_inputs_summary_zip_contract_only_should_pass")
        darwin_report.unlink(missing_ok=True)
        windows_report.unlink(missing_ok=True)
        linux_report.unlink(missing_ok=True)
        zip_payload = base / "incoming_summary_zip" / "fixed64_darwin_probe_artifact.zip"
        zip_payload.parent.mkdir(parents=True, exist_ok=True)
        with zipfile.ZipFile(zip_payload, "w", compression=zipfile.ZIP_DEFLATED) as zf:
            zf.writestr(
                "nested/fixed64_cross_platform_probe_darwin.detjson",
                json.dumps(make_probe("Darwin"), ensure_ascii=False),
            )
        summary_zip_only = base / "incoming_summary_zip" / "fixed64_darwin_probe_artifact.detjson"
        relative_zip = zip_payload.relative_to(summary_zip_only.parent)
        write_json(
            summary_zip_only,
            {
                "schema": "ddn.fixed64.darwin_probe_artifact.v1",
                "generated_at_utc": datetime.now(timezone.utc).isoformat(),
                "ok": True,
                "status": "staged",
                "reason": "-",
                "probe_report": "missing_probe.detjson",
                "summary_report": str(summary_zip_only),
                "zip": {
                    "enabled": True,
                    "path": str(relative_zip).replace("\\", "/"),
                    "status": "staged",
                    "reason": "-",
                },
            },
        )
        proc_resolve_summary_zip = run_readiness(
            darwin_report=darwin_report,
            windows_report=windows_report,
            linux_report=linux_report,
            inputs_report=inputs_report,
            json_out=readiness_out,
            transition_probe=transition_probe,
            resolve_threeway_inputs=True,
            resolve_input_candidates=[str(summary_zip_only)],
        )
        complete_probe("run_readiness.collect_output")
        if proc_resolve_summary_zip.returncode != 0:
            update_progress("fail")
            print(
                "[fixed64-darwin-readiness-selftest] resolve summary zip contract-only case should pass",
                file=sys.stderr,
            )
            return 1
        start_probe("validate_resolve_summary_zip_contract_only")
        resolve_summary_zip_doc = load_json(readiness_out)
        if not isinstance(resolve_summary_zip_doc, dict):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] resolve summary zip contract-only report missing", file=sys.stderr)
            return 1
        if str(resolve_summary_zip_doc.get("status", "")) != "pass_contract_only":
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] resolve summary zip contract-only status mismatch", file=sys.stderr)
            return 1
        resolve_summary_zip_payload = resolve_summary_zip_doc.get("resolve_inputs")
        if not isinstance(resolve_summary_zip_payload, dict) or not bool(resolve_summary_zip_payload.get("ok", False)):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] resolve summary zip payload mismatch", file=sys.stderr)
            return 1
        resolve_summary_zip_inner = resolve_summary_zip_payload.get("payload")
        if not isinstance(resolve_summary_zip_inner, dict):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] resolve summary zip inner payload missing", file=sys.stderr)
            return 1
        selected_source_zip = str(resolve_summary_zip_inner.get("selected_source", ""))
        if ".zip!" not in selected_source_zip:
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] resolve summary zip selected_source mismatch", file=sys.stderr)
            return 1
        complete_probe("validate_resolve_summary_zip_contract_only")
        complete_case("resolve_inputs_summary_zip_contract_only_should_pass")

        # 5) resolve synthetic candidate should fail at resolve step
        start_case("resolve_inputs_synthetic_should_fail")
        darwin_report.unlink(missing_ok=True)
        windows_report.unlink(missing_ok=True)
        linux_report.unlink(missing_ok=True)
        synthetic_candidate = base / "incoming_synthetic" / "fixed64_cross_platform_probe_darwin.detjson"
        write_json(synthetic_candidate, make_probe("Darwin", synthetic=True))
        proc_resolve_synthetic = run_readiness(
            darwin_report=darwin_report,
            windows_report=windows_report,
            linux_report=linux_report,
            inputs_report=inputs_report,
            json_out=readiness_out,
            transition_probe=transition_probe,
            resolve_threeway_inputs=True,
            resolve_input_candidates=[str(synthetic_candidate)],
        )
        complete_probe("run_readiness.collect_output")
        if proc_resolve_synthetic.returncode == 0:
            update_progress("fail")
            print(
                "[fixed64-darwin-readiness-selftest] resolve synthetic candidate should fail",
                file=sys.stderr,
            )
            return 1
        start_probe("validate_resolve_synthetic_fail")
        resolve_synthetic_doc = load_json(readiness_out)
        if not isinstance(resolve_synthetic_doc, dict):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] resolve synthetic fail report missing", file=sys.stderr)
            return 1
        if str(resolve_synthetic_doc.get("status", "")) != "fail_resolve":
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] resolve synthetic fail status mismatch", file=sys.stderr)
            return 1
        resolve_synthetic_payload = resolve_synthetic_doc.get("resolve_inputs")
        if not isinstance(resolve_synthetic_payload, dict):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] resolve synthetic payload missing", file=sys.stderr)
            return 1
        if "resolve status is not staged" not in str(resolve_synthetic_payload.get("reason", "")):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] resolve synthetic reason mismatch", file=sys.stderr)
            return 1
        complete_probe("validate_resolve_synthetic_fail")
        complete_case("resolve_inputs_synthetic_should_fail")

        # 6) pass_3way: windows/linux/darwin all present and matching
        start_case("pass_3way_should_pass")
        write_json(windows_report, make_probe("Windows"))
        write_json(linux_report, make_probe("Linux"))
        write_json(darwin_report, make_probe("Darwin"))
        proc_pass_3way = run_readiness(
            darwin_report=darwin_report,
            windows_report=windows_report,
            linux_report=linux_report,
            inputs_report=inputs_report,
            json_out=readiness_out,
            transition_probe=transition_probe,
        )
        complete_probe("run_readiness.collect_output")
        if proc_pass_3way.returncode != 0:
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] pass_3way case should pass", file=sys.stderr)
            return 1
        start_probe("validate_pass_3way")
        pass_3way_doc = load_json(readiness_out)
        if not isinstance(pass_3way_doc, dict):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] pass_3way report missing", file=sys.stderr)
            return 1
        if str(pass_3way_doc.get("status", "")) != "pass_3way":
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] pass_3way status mismatch", file=sys.stderr)
            return 1
        if not bool(pass_3way_doc.get("ok", False)):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] pass_3way ok should be true", file=sys.stderr)
            return 1
        threeway_pass = pass_3way_doc.get("threeway_check")
        if not isinstance(threeway_pass, dict) or not bool(threeway_pass.get("attempted", False)):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] pass_3way attempted should be true", file=sys.stderr)
            return 1
        if int(threeway_pass.get("returncode", -1)) != 0:
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] pass_3way returncode mismatch", file=sys.stderr)
            return 1
        complete_probe("validate_pass_3way")
        complete_case("pass_3way_should_pass")

        # 7) fail_3way: windows/linux mismatch after contract pass
        start_case("fail_3way_should_fail")
        mismatch_raw = list(SAMPLE_RAW)
        mismatch_raw[0] += 1
        write_json(
            linux_report,
            make_probe(
                "Linux",
                blake3_hex="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                raw_i64=mismatch_raw,
            ),
        )
        proc_fail_3way = run_readiness(
            darwin_report=darwin_report,
            windows_report=windows_report,
            linux_report=linux_report,
            inputs_report=inputs_report,
            json_out=readiness_out,
            transition_probe=transition_probe,
        )
        complete_probe("run_readiness.collect_output")
        if proc_fail_3way.returncode == 0:
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] fail_3way case should fail", file=sys.stderr)
            return 1
        start_probe("validate_fail_3way")
        fail_3way_doc = load_json(readiness_out)
        if not isinstance(fail_3way_doc, dict):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] fail_3way report missing", file=sys.stderr)
            return 1
        if str(fail_3way_doc.get("status", "")) != "fail_3way":
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] fail_3way status mismatch", file=sys.stderr)
            return 1
        if bool(fail_3way_doc.get("ok", True)):
            update_progress("fail")
            print("[fixed64-darwin-readiness-selftest] fail_3way ok should be false", file=sys.stderr)
            return 1
        complete_probe("validate_fail_3way")
        complete_case("fail_3way_should_fail")

    update_progress("pass")
    print("[fixed64-darwin-readiness-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
