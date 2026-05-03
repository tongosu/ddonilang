#!/usr/bin/env python
from __future__ import annotations

import json
import os
import shutil
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path

from _teul_cli_freshness import build_teul_cli_cmd as shared_build_teul_cli_cmd

EXPECTED_CODE = "E_MAP_DOT_KEY_MISSING"
PROGRESS_ENV_KEY = "DDN_MAP_ACCESS_CONTRACT_CHECK_PROGRESS_JSON"
_PROGRESS_PARENT_READY: set[str] = set()


def fail(msg: str) -> int:
    print(f"[map-access-contract-check] fail: {msg}")
    return 1


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
    parent_key = str(out.parent)
    if parent_key not in _PROGRESS_PARENT_READY:
        out.parent.mkdir(parents=True, exist_ok=True)
        _PROGRESS_PARENT_READY.add(parent_key)
    payload = {
        "schema": "ddn.map_access_contract_check.progress.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
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


def teul_cli_candidates(root: Path) -> list[Path]:
    suffix = ".exe" if os.name == "nt" else ""
    return [
        root / "target" / "debug" / f"teul-cli{suffix}",
        root / "target" / "release" / f"teul-cli{suffix}",
        Path(f"I:/home/urihanl/ddn/codex/target/debug/teul-cli{suffix}"),
    ]


def run_teul_cli(root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    cmd = build_teul_cli_cmd(*args)
    return subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def build_teul_cli_cmd(*args: str) -> list[str]:
    root = Path(__file__).resolve().parent.parent
    return shared_build_teul_cli_cmd(
        root,
        list(args),
        candidates=teul_cli_candidates(root),
        include_which=True,
    )


def spawn_teul_cli(root: Path, *args: str) -> subprocess.Popen[str]:
    return subprocess.Popen(
        build_teul_cli_cmd(*args),
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def collect_teul_cli_process(proc: subprocess.Popen[str]) -> subprocess.CompletedProcess[str]:
    stdout, stderr = proc.communicate()
    return subprocess.CompletedProcess(proc.args, proc.returncode, stdout, stderr)


def merged_output(proc: subprocess.CompletedProcess[str]) -> str:
    return ((proc.stdout or "") + "\n" + (proc.stderr or "")).strip()


def nonempty_stdout_lines(proc: subprocess.CompletedProcess[str]) -> list[str]:
    return [line.strip() for line in (proc.stdout or "").splitlines() if line.strip()]


def run_canon_case(root: Path) -> int:
    input_path = "pack/lang_consistency_v1/c05_map_dot_nested_write_canon/input.ddn"
    expected_path = root / "pack" / "lang_consistency_v1" / "c05_map_dot_nested_write_canon" / "expected_canon.ddn"
    expected = expected_path.read_text(encoding="utf-8").strip()
    proc = run_teul_cli(root, "canon", input_path)
    if proc.returncode != 0:
        return fail(f"canon case must pass: {merged_output(proc)}")
    actual = (proc.stdout or "").strip()
    if actual != expected:
        return fail(f"canon output mismatch expected={expected!r} actual={actual!r}")
    return 0


def run_write_success_case(root: Path) -> int:
    input_path = "pack/lang_consistency_v1/c06_map_dot_nested_write_run/input.ddn"
    proc = run_teul_cli(root, "run", input_path)
    if proc.returncode != 0:
        return fail(f"map write success case must pass: {merged_output(proc)}")
    lines = nonempty_stdout_lines(proc)
    if not lines:
        return fail("map write success case missing stdout")
    if lines[0] != "9":
        return fail(f"map write success first line must be 9, got={lines[0]!r}")
    return 0


def run_missing_key_case_process(root: Path, input_path: str) -> subprocess.CompletedProcess[str]:
    return run_teul_cli(root, "run", input_path)


def spawn_missing_key_case_process(root: Path, input_path: str) -> subprocess.Popen[str]:
    return spawn_teul_cli(root, "run", input_path)


def validate_missing_key_returncode(
    proc: subprocess.CompletedProcess[str],
    marker: str,
) -> int:
    if proc.returncode == 0:
        return fail(f"{marker} case must fail but passed")
    return 0


def validate_missing_key_error_code(
    proc: subprocess.CompletedProcess[str],
    marker: str,
) -> int:
    output = merged_output(proc)
    if EXPECTED_CODE not in output:
        return fail(f"{marker} case missing {EXPECTED_CODE}: {output}")
    return 0


def validate_missing_key_marker(
    proc: subprocess.CompletedProcess[str],
    marker: str,
) -> int:
    output = merged_output(proc)
    if "키가 없습니다: 속도" not in output:
        return fail(f"{marker} case missing key marker: {output}")
    return 0


def spawn_optional_lookup_case_process(root: Path) -> subprocess.Popen[str]:
    input_path = "pack/lang_consistency_v1/c11_map_optional_lookup_run/input.ddn"
    return spawn_teul_cli(root, "run", input_path)


def validate_optional_lookup_returncode(proc: subprocess.CompletedProcess[str]) -> int:
    if proc.returncode != 0:
        return fail(f"optional lookup case must pass: {merged_output(proc)}")
    return 0


def validate_optional_lookup_line_count(proc: subprocess.CompletedProcess[str]) -> int:
    lines = nonempty_stdout_lines(proc)
    if len(lines) < 2:
        return fail(f"optional lookup stdout missing lines: {lines!r}")
    return 0


def validate_optional_lookup_values(proc: subprocess.CompletedProcess[str]) -> int:
    lines = nonempty_stdout_lines(proc)
    if lines[:2] != ["7", "없음"]:
        return fail(f"optional lookup stdout mismatch: {lines!r}")
    return 0


def main() -> int:
    root = Path(__file__).resolve().parent.parent
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

    def complete_probe(name: str) -> None:
        nonlocal current_probe, last_completed_probe
        current_probe = "-"
        last_completed_probe = name
        update_progress("running")

    update_progress("running")

    start_case("canon_case")
    start_probe("run_canon_case")
    rc = run_canon_case(root)
    complete_probe("run_canon_case")
    if rc != 0:
        update_progress("fail")
        return rc
    complete_case("canon_case")

    start_case("write_success_case")
    start_probe("run_write_success_case")
    rc = run_write_success_case(root)
    complete_probe("run_write_success_case")
    if rc != 0:
        update_progress("fail")
        return rc
    complete_case("write_success_case")

    start_case("missing_write_case")
    start_probe("run_missing_key_case.missing_write.run_teul_cli")
    missing_write_child = spawn_missing_key_case_process(
        root,
        "pack/lang_consistency_v1/c07_map_dot_nested_write_missing_key_EXPECT_FAIL/input.ddn",
    )
    missing_read_child = spawn_missing_key_case_process(
        root,
        "pack/lang_consistency_v1/c08_map_dot_read_missing_key_EXPECT_FAIL/input.ddn",
    )
    optional_lookup_child = spawn_optional_lookup_case_process(root)
    missing_write_proc = collect_teul_cli_process(missing_write_child)
    missing_read_prefetched_proc = collect_teul_cli_process(missing_read_child)
    optional_lookup_prefetched_proc = collect_teul_cli_process(optional_lookup_child)
    complete_probe("run_missing_key_case.missing_write.run_teul_cli")
    start_probe("run_missing_key_case.missing_write.validate_returncode")
    rc = validate_missing_key_returncode(missing_write_proc, "missing-write")
    complete_probe("run_missing_key_case.missing_write.validate_returncode")
    if rc != 0:
        update_progress("fail")
        return rc
    start_probe("run_missing_key_case.missing_write.validate_error_code")
    rc = validate_missing_key_error_code(missing_write_proc, "missing-write")
    complete_probe("run_missing_key_case.missing_write.validate_error_code")
    if rc != 0:
        update_progress("fail")
        return rc
    start_probe("run_missing_key_case.missing_write.validate_missing_key_marker")
    rc = validate_missing_key_marker(missing_write_proc, "missing-write")
    complete_probe("run_missing_key_case.missing_write.validate_missing_key_marker")
    if rc != 0:
        update_progress("fail")
        return rc
    complete_case("missing_write_case")

    start_case("missing_read_case")
    start_probe("run_missing_key_case.missing_read.run_teul_cli_prefetched")
    missing_read_proc = missing_read_prefetched_proc
    complete_probe("run_missing_key_case.missing_read.run_teul_cli_prefetched")
    start_probe("run_missing_key_case.missing_read.validate_returncode")
    rc = validate_missing_key_returncode(missing_read_proc, "missing-read")
    complete_probe("run_missing_key_case.missing_read.validate_returncode")
    if rc != 0:
        update_progress("fail")
        return rc
    start_probe("run_missing_key_case.missing_read.validate_error_code")
    rc = validate_missing_key_error_code(missing_read_proc, "missing-read")
    complete_probe("run_missing_key_case.missing_read.validate_error_code")
    if rc != 0:
        update_progress("fail")
        return rc
    start_probe("run_missing_key_case.missing_read.validate_missing_key_marker")
    rc = validate_missing_key_marker(missing_read_proc, "missing-read")
    complete_probe("run_missing_key_case.missing_read.validate_missing_key_marker")
    if rc != 0:
        update_progress("fail")
        return rc
    complete_case("missing_read_case")

    start_case("optional_lookup_case")
    start_probe("run_optional_lookup_case.run_teul_cli_prefetched")
    optional_lookup_proc = optional_lookup_prefetched_proc
    optional_lookup_returncode = int(optional_lookup_proc.returncode or 0)
    if optional_lookup_proc.returncode is None:
        optional_lookup_proc = subprocess.CompletedProcess(
            optional_lookup_proc.args,
            optional_lookup_returncode,
            optional_lookup_proc.stdout,
            optional_lookup_proc.stderr,
        )
    complete_probe("run_optional_lookup_case.run_teul_cli_prefetched")
    start_probe("run_optional_lookup_case.validate_returncode")
    rc = validate_optional_lookup_returncode(optional_lookup_proc)
    complete_probe("run_optional_lookup_case.validate_returncode")
    if rc != 0:
        update_progress("fail")
        return rc
    start_probe("run_optional_lookup_case.validate_line_count")
    rc = validate_optional_lookup_line_count(optional_lookup_proc)
    complete_probe("run_optional_lookup_case.validate_line_count")
    if rc != 0:
        update_progress("fail")
        return rc
    start_probe("run_optional_lookup_case.validate_values")
    rc = validate_optional_lookup_values(optional_lookup_proc)
    complete_probe("run_optional_lookup_case.validate_values")
    if rc != 0:
        update_progress("fail")
        return rc
    complete_case("optional_lookup_case")

    update_progress("pass")
    print("map access contract check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
