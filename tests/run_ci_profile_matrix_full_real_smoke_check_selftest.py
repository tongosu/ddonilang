#!/usr/bin/env python
from __future__ import annotations

import os
import json
import subprocess
import sys
import tempfile
import time
import io
from contextlib import redirect_stderr, redirect_stdout
from importlib.util import module_from_spec, spec_from_file_location
from pathlib import Path
from typing import Callable

from _ci_profile_matrix_full_real_smoke_contract import (
    PROFILE_MATRIX_FULL_REAL_SMOKE_ALLOW_FLAG,
    PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SCRIPT,
    PROFILE_MATRIX_FULL_REAL_SMOKE_MODE_MARKER,
    PROFILE_MATRIX_FULL_REAL_SMOKE_SELFTEST_SCRIPT_OVERRIDE_ENV_KEY,
    PROFILE_MATRIX_FULL_REAL_SMOKE_STATUS_PASS,
    PROFILE_MATRIX_GATE_SELFTEST_FULL_REAL_TOKENS,
)

PROGRESS_ENV_KEY = "DDN_PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SELFTEST_PROGRESS_JSON"
FAKE_GATE_PROGRESS_ENV_KEY = "DDN_PROFILE_MATRIX_FULL_REAL_SMOKE_FAKE_GATE_PROGRESS_JSON"


def run(
    cmd: list[str],
    cwd: Path,
    env: dict[str, str] | None = None,
    *,
    probe_prefix: str = "",
    start_probe: Callable[[str], None] | None = None,
    complete_probe: Callable[[str], None] | None = None,
) -> subprocess.CompletedProcess[str]:
    child_progress_path = str((env or {}).get(FAKE_GATE_PROGRESS_ENV_KEY, "")).strip()

    def begin(name: str) -> None:
        if probe_prefix and start_probe is not None:
            start_probe(f"{probe_prefix}.{name}")

    def end(name: str) -> None:
        if probe_prefix and complete_probe is not None:
            complete_probe(f"{probe_prefix}.{name}")

    if not probe_prefix:
        return subprocess.run(
            cmd,
            cwd=cwd,
            env=env,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )

    begin("spawn_process")
    proc = subprocess.Popen(
        cmd,
        cwd=cwd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    end("spawn_process")
    begin("wait_exit")
    active_child_probe = "-"
    while True:
        try:
            stdout, stderr = proc.communicate(timeout=0.005)
            break
        except subprocess.TimeoutExpired:
            next_child_probe = resolve_fake_gate_child_probe(child_progress_path)
            if next_child_probe == active_child_probe:
                continue
            if active_child_probe != "-":
                end(f"wait_exit.{active_child_probe}")
            active_child_probe = next_child_probe
            if active_child_probe != "-":
                begin(f"wait_exit.{active_child_probe}")
    if active_child_probe != "-":
        end(f"wait_exit.{active_child_probe}")
    end("wait_exit")
    begin("collect_output")
    completed = subprocess.CompletedProcess(cmd, proc.returncode, stdout, stderr)
    end("collect_output")
    return completed


def fail(detail: str, proc: subprocess.CompletedProcess[str] | None = None) -> int:
    print(f"check=ci_profile_matrix_full_real_smoke_check_selftest detail={detail}")
    if proc is not None:
        if (proc.stdout or "").strip():
            print(proc.stdout.strip())
        if (proc.stderr or "").strip():
            print(proc.stderr.strip())
    return 1


def run_smoke_check_fast_path(
    *, root: Path, argv: list[str], env_patch: dict[str, str] | None = None
) -> subprocess.CompletedProcess[str]:
    script_path = root / PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SCRIPT
    tests_dir = root / "tests"
    module_name = "_ci_profile_matrix_full_real_smoke_check_selftest_fastpath_explicit_optin"
    spec = spec_from_file_location(module_name, script_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"unable to load smoke check helper: {script_path}")
    module = module_from_spec(spec)
    stdout_buf = io.StringIO()
    stderr_buf = io.StringIO()
    previous_argv = sys.argv[:]
    sys_path_added = False
    if str(tests_dir) not in sys.path:
        sys.path.insert(0, str(tests_dir))
        sys_path_added = True
    previous_env: dict[str, str | None] = {}
    if isinstance(env_patch, dict):
        for key, value in env_patch.items():
            previous_env[key] = os.environ.get(key)
            os.environ[key] = str(value)
    try:
        with redirect_stdout(stdout_buf), redirect_stderr(stderr_buf):
            sys.argv = [str(script_path), *argv]
            spec.loader.exec_module(module)
            try:
                returncode = int(module.main())
            except SystemExit as exc:
                code = exc.code
                if code is None:
                    returncode = 0
                elif isinstance(code, int):
                    returncode = int(code)
                else:
                    returncode = 1
    finally:
        sys.argv = previous_argv
        if isinstance(env_patch, dict):
            for key, value in previous_env.items():
                if value is None:
                    os.environ.pop(key, None)
                else:
                    os.environ[key] = value
        if sys_path_added:
            try:
                sys.path.remove(str(tests_dir))
            except ValueError:
                pass
    return subprocess.CompletedProcess(
        [sys.executable, str(script_path), *argv],
        returncode,
        stdout_buf.getvalue(),
        stderr_buf.getvalue(),
    )


def resolve_fake_gate_child_probe(path_text: str) -> str:
    if not str(path_text).strip():
        return "-"
    path = Path(path_text)
    if not path.exists():
        return "child_progress_missing"
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return "child_progress_invalid"
    if not isinstance(payload, dict):
        return "child_progress_invalid"
    stage = str(payload.get("stage", "")).strip()
    if not stage:
        return "child_progress_no_stage"
    return f"child_{stage}"


def write_fake_gate_selftest(path: Path, *, lines: list[str], returncode: int) -> None:
    payload = """#!/usr/bin/env python
from __future__ import annotations

import json
import sys
import os
from pathlib import Path

LINES = {lines!r}
RETURN_CODE = {returncode}
PROGRESS_ENV_KEY = {progress_env_key!r}


def write_progress(stage: str) -> None:
    progress_path = str(os.environ.get(PROGRESS_ENV_KEY, "")).strip()
    if not progress_path:
        return
    out = Path(progress_path)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {{
        "schema": "ddn.ci.fake_gate_selftest.progress.v1",
        "stage": stage,
    }}
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\\n", encoding="utf-8")



def main() -> int:
    write_progress("bootstrap")
    write_progress("emit_markers")
    for line in LINES:
        print(line)
    write_progress("exit_pending")
    return RETURN_CODE


if __name__ == "__main__":
    raise SystemExit(main())
""".format(lines=lines, returncode=int(returncode), progress_env_key=FAKE_GATE_PROGRESS_ENV_KEY)
    path.write_text(payload, encoding="utf-8")


def write_progress_snapshot(
    path_text: str,
    *,
    status: str,
    current_case: str,
    last_completed_case: str,
    total_elapsed_ms: int,
    current_probe: str,
    last_completed_probe: str,
) -> None:
    if not str(path_text).strip():
        return
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "ddn.ci.profile_matrix_full_real_smoke_check_selftest.progress.v1",
        "status": status,
        "current_case": current_case,
        "last_completed_case": last_completed_case,
        "total_elapsed_ms": str(int(total_elapsed_ms)),
        "current_probe": current_probe,
        "last_completed_probe": last_completed_probe,
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    smoke_script = str(root / PROFILE_MATRIX_FULL_REAL_SMOKE_CHECK_SCRIPT)
    progress_path = str(os.environ.get(PROGRESS_ENV_KEY, "")).strip()
    started_at = time.perf_counter()
    current_case = "-"
    last_completed_case = "-"
    current_probe = "-"
    last_completed_probe = "-"

    def update_progress(status: str) -> None:
        elapsed_ms = int((time.perf_counter() - started_at) * 1000)
        write_progress_snapshot(
            progress_path,
            status=status,
            current_case=current_case,
            last_completed_case=last_completed_case,
            total_elapsed_ms=elapsed_ms,
            current_probe=current_probe,
            last_completed_probe=last_completed_probe,
        )

    def start_case(name: str) -> None:
        nonlocal current_case
        current_case = name
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

    def complete_probe(name: str) -> None:
        nonlocal current_probe, last_completed_probe
        current_probe = "-"
        last_completed_probe = name
        update_progress("running")

    update_progress("running")
    start_case("explicit_optin_should_fail")
    start_probe("run_smoke")
    no_optin = run_smoke_check_fast_path(root=root, argv=[])
    complete_probe("run_smoke")
    if no_optin.returncode == 0:
        update_progress("fail")
        return fail("explicit_optin_should_fail", no_optin)
    stdout = str(no_optin.stdout or "")
    start_probe("validate_mode_marker")
    if PROFILE_MATRIX_FULL_REAL_SMOKE_MODE_MARKER not in stdout:
        update_progress("fail")
        return fail("explicit_optin_mode_marker_missing", no_optin)
    complete_probe("validate_mode_marker")
    start_probe("validate_explicit_optin_failure")
    if "ci_profile_matrix_full_real_smoke_status=fail reason=explicit_optin_required" not in stdout:
        update_progress("fail")
        return fail("explicit_optin_failure_marker_missing", no_optin)
    complete_probe("validate_explicit_optin_failure")
    complete_case("explicit_optin_should_fail")

    required_markers = list(PROFILE_MATRIX_GATE_SELFTEST_FULL_REAL_TOKENS[4:])

    with tempfile.TemporaryDirectory(prefix="ci_profile_matrix_full_real_smoke_check_") as td:
        temp_root = Path(td)
        py = sys.executable
        ok_script = temp_root / "fake_gate_selftest_ok.py"
        fail_script = temp_root / "fake_gate_selftest_fail.py"
        missing_marker_script = temp_root / "fake_gate_selftest_missing_marker.py"
        child_progress_path = temp_root / "fake_gate_progress.detjson"

        write_fake_gate_selftest(ok_script, lines=required_markers, returncode=0)
        write_fake_gate_selftest(fail_script, lines=required_markers, returncode=1)
        write_fake_gate_selftest(missing_marker_script, lines=required_markers[:-1], returncode=0)

        override_env = dict(os.environ)
        override_env[PROFILE_MATRIX_FULL_REAL_SMOKE_SELFTEST_SCRIPT_OVERRIDE_ENV_KEY] = str(ok_script)
        override_env[FAKE_GATE_PROGRESS_ENV_KEY] = str(child_progress_path)
        start_case("override_ok_should_pass")
        start_probe("run_smoke")
        if child_progress_path.exists():
            child_progress_path.unlink()
        ok_proc = run_smoke_check_fast_path(
            root=root,
            argv=[PROFILE_MATRIX_FULL_REAL_SMOKE_ALLOW_FLAG],
            env_patch=override_env,
        )
        complete_probe("run_smoke")
        if ok_proc.returncode != 0:
            update_progress("fail")
            return fail("override_ok_should_pass", ok_proc)
        ok_stdout = str(ok_proc.stdout or "")
        start_probe("validate_mode_marker")
        if PROFILE_MATRIX_FULL_REAL_SMOKE_MODE_MARKER not in ok_stdout:
            update_progress("fail")
            return fail("override_ok_mode_marker_missing", ok_proc)
        complete_probe("validate_mode_marker")
        start_probe("validate_pass_marker")
        if PROFILE_MATRIX_FULL_REAL_SMOKE_STATUS_PASS not in ok_stdout:
            update_progress("fail")
            return fail("override_ok_pass_marker_missing", ok_proc)
        complete_probe("validate_pass_marker")
        complete_case("override_ok_should_pass")

        override_env[PROFILE_MATRIX_FULL_REAL_SMOKE_SELFTEST_SCRIPT_OVERRIDE_ENV_KEY] = str(fail_script)
        start_case("override_failure_should_fail")
        start_probe("run_smoke")
        if child_progress_path.exists():
            child_progress_path.unlink()
        fail_proc = run_smoke_check_fast_path(
            root=root,
            argv=[PROFILE_MATRIX_FULL_REAL_SMOKE_ALLOW_FLAG],
            env_patch=override_env,
        )
        complete_probe("run_smoke")
        if fail_proc.returncode == 0:
            update_progress("fail")
            return fail("override_failure_should_fail", fail_proc)
        start_probe("validate_selftest_failed_marker")
        if "ci_profile_matrix_full_real_smoke_status=fail reason=selftest_failed" not in str(fail_proc.stdout or ""):
            update_progress("fail")
            return fail("override_failure_marker_missing", fail_proc)
        complete_probe("validate_selftest_failed_marker")
        complete_case("override_failure_should_fail")

        override_env[PROFILE_MATRIX_FULL_REAL_SMOKE_SELFTEST_SCRIPT_OVERRIDE_ENV_KEY] = str(missing_marker_script)
        start_case("override_missing_marker_should_fail")
        start_probe("run_smoke")
        if child_progress_path.exists():
            child_progress_path.unlink()
        missing_marker_proc = run_smoke_check_fast_path(
            root=root,
            argv=[PROFILE_MATRIX_FULL_REAL_SMOKE_ALLOW_FLAG],
            env_patch=override_env,
        )
        complete_probe("run_smoke")
        if missing_marker_proc.returncode == 0:
            update_progress("fail")
            return fail("override_missing_marker_should_fail", missing_marker_proc)
        start_probe("validate_missing_marker_failure")
        if "ci_profile_matrix_full_real_smoke_status=fail reason=marker_missing marker=" not in str(
            missing_marker_proc.stdout or ""
        ):
            update_progress("fail")
            return fail("override_missing_marker_failure_marker_missing", missing_marker_proc)
        complete_probe("validate_missing_marker_failure")
        complete_case("override_missing_marker_should_fail")

    update_progress("pass")
    print("[ci-profile-matrix-full-real-smoke-check-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
