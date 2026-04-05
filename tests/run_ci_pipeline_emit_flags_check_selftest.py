#!/usr/bin/env python
from __future__ import annotations

import json
import io
import os
from contextlib import redirect_stderr, redirect_stdout
import importlib
import runpy
import subprocess
import sys
import tempfile
import time
from pathlib import Path

from _ci_age5_combined_heavy_contract import (
    AGE5_COMBINED_HEAVY_ENV_KEY,
    AGE5_COMBINED_HEAVY_POLICY_DIGEST_SCRIPT,
    AGE5_COMBINED_HEAVY_POLICY_REPORT_PATH,
    AGE5_COMBINED_HEAVY_POLICY_SCHEMA,
    AGE5_COMBINED_HEAVY_POLICY_SCRIPT,
    AGE5_COMBINED_HEAVY_POLICY_SUMMARY_PATH,
    AGE5_COMBINED_HEAVY_POLICY_TEXT_PATH,
)
from _ci_profile_matrix_full_real_smoke_contract import (
    PROFILE_MATRIX_FULL_AGGREGATE_ENV_KEY,
    PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY,
    PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SCHEMA,
    PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SCRIPT,
    PROFILE_MATRIX_SELFTEST_GATE_FLAGS_HELPER_SCRIPT,
    PROFILE_MATRIX_SELFTEST_GATE_FLAGS_VAR,
)

PROGRESS_ENV_KEY = "DDN_CI_PIPELINE_EMIT_FLAGS_SELFTEST_PROGRESS_JSON"
PIPELINE_PROGRESS_ENV_KEY = "DDN_CI_PIPELINE_EMIT_FLAGS_PROGRESS_JSON"
RUNTIME_CONTRACT_MINIMAL_ENV_KEY = "DDN_CI_PIPELINE_RUNTIME_CONTRACT_MINIMAL"
_MODULE_CACHE: dict[str, object] = {}


def _run_module_main(
    module_name: str,
    cmd: list[str],
    argv: list[str],
    cwd: Path,
    env: dict[str, str] | None,
) -> subprocess.CompletedProcess[str]:
    module = _MODULE_CACHE.get(module_name)
    if module is None:
        module = importlib.import_module(module_name)
        _MODULE_CACHE[module_name] = module

    stdout_buf = io.StringIO()
    stderr_buf = io.StringIO()
    old_argv = sys.argv
    old_cwd = Path.cwd()
    old_env = os.environ.copy() if env is not None else None
    returncode = 0
    try:
        sys.argv = argv
        os.chdir(cwd)
        if env is not None:
            os.environ.clear()
            os.environ.update(env)
        with redirect_stdout(stdout_buf), redirect_stderr(stderr_buf):
            try:
                code = module.main()
                if code is None:
                    returncode = 0
                elif isinstance(code, int):
                    returncode = code
                else:
                    returncode = 1
                    stderr_buf.write(str(code))
            except SystemExit as exc:
                code = exc.code
                if code is None:
                    returncode = 0
                elif isinstance(code, int):
                    returncode = code
                else:
                    returncode = 1
                    stderr_buf.write(str(code))
            except Exception as exc:  # pragma: no cover - defensive fallback
                returncode = 1
                stderr_buf.write(f"{type(exc).__name__}: {exc}")
    finally:
        sys.argv = old_argv
        os.chdir(old_cwd)
        if old_env is not None:
            os.environ.clear()
            os.environ.update(old_env)
    return subprocess.CompletedProcess(
        args=cmd,
        returncode=returncode,
        stdout=stdout_buf.getvalue(),
        stderr=stderr_buf.getvalue(),
    )


def run(
    cmd: list[str],
    cwd: Path,
    env: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    if len(cmd) >= 2 and str(cmd[1]).endswith(".py"):
        script = str(cmd[1])
        script_norm = script.replace("\\", "/")
        argv = [script, *[str(arg) for arg in cmd[2:]]]
        if script_norm.endswith("tests/run_ci_pipeline_emit_flags_check.py"):
            return _run_module_main("run_ci_pipeline_emit_flags_check", cmd, argv, cwd, env)
        stdout_buf = io.StringIO()
        stderr_buf = io.StringIO()
        old_argv = sys.argv
        old_cwd = Path.cwd()
        old_env = os.environ.copy() if env is not None else None
        returncode = 0
        try:
            sys.argv = argv
            os.chdir(cwd)
            if env is not None:
                os.environ.clear()
                os.environ.update(env)
            with redirect_stdout(stdout_buf), redirect_stderr(stderr_buf):
                try:
                    runpy.run_path(script, run_name="__main__")
                except SystemExit as exc:
                    code = exc.code
                    if code is None:
                        returncode = 0
                    elif isinstance(code, int):
                        returncode = code
                    else:
                        returncode = 1
                        stderr_buf.write(str(code))
                except Exception as exc:  # pragma: no cover - defensive fallback
                    returncode = 1
                    stderr_buf.write(f"{type(exc).__name__}: {exc}")
        finally:
            sys.argv = old_argv
            os.chdir(old_cwd)
            if old_env is not None:
                os.environ.clear()
                os.environ.update(old_env)
        return subprocess.CompletedProcess(
            args=cmd,
            returncode=returncode,
            stdout=stdout_buf.getvalue(),
            stderr=stderr_buf.getvalue(),
        )
    return subprocess.run(
        cmd,
        cwd=cwd,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text.rstrip() + "\n", encoding="utf-8")


def write_progress_snapshot(
    path_text: str,
    *,
    status: str,
    current_case: str,
    last_completed_case: str,
    current_probe: str,
    last_completed_probe: str,
    cases_completed: int,
    total_elapsed_ms: int,
) -> None:
    if not str(path_text).strip():
        return
    out = Path(path_text)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "ddn.ci.pipeline_emit_flags_selftest_progress.v1",
        "status": status,
        "current_case": current_case,
        "last_completed_case": last_completed_case,
        "current_probe": current_probe,
        "last_completed_probe": last_completed_probe,
        "cases_completed": int(cases_completed),
        "total_elapsed_ms": int(total_elapsed_ms),
    }
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def build_gitlab_text(aggregate_line: str) -> str:
    return "\n".join(
        [
            "image: rust:1.75",
            "variables:",
            '  DDN_ENABLE_DARWIN_PROBE: "0"',
            f'  {PROFILE_MATRIX_FULL_AGGREGATE_ENV_KEY}: "0"',
            f'  {PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY}: "0"',
            f'  {AGE5_COMBINED_HEAVY_ENV_KEY}: "0"',
            "script:",
            f"  - eval \"$(python3 {PROFILE_MATRIX_SELFTEST_GATE_FLAGS_HELPER_SCRIPT} --provider gitlab --format shell)\" || rc=$?",
            "  - if [ \"$rc\" -ne 0 ]; then",
            f"  -   eval \"$(python {PROFILE_MATRIX_SELFTEST_GATE_FLAGS_HELPER_SCRIPT} --provider gitlab --format shell)\" || rc=$?",
            "  - fi",
            f"  - {aggregate_line}",
            f"  - eval \"$(python3 {PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SCRIPT} --provider gitlab --format shell)\" || rc=$?",
            "  - if [ \"$rc\" -ne 0 ]; then",
            f"  -   eval \"$(python {PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SCRIPT} --provider gitlab --format shell)\" || rc=$?",
            "  - fi",
            f"  - eval \"$(python3 {AGE5_COMBINED_HEAVY_POLICY_SCRIPT} --provider gitlab --format shell --json-out {AGE5_COMBINED_HEAVY_POLICY_REPORT_PATH})\" || rc=$?",
            "  - if [ \"$rc\" -ne 0 ]; then",
            f"  -   eval \"$(python {AGE5_COMBINED_HEAVY_POLICY_SCRIPT} --provider gitlab --format shell --json-out {AGE5_COMBINED_HEAVY_POLICY_REPORT_PATH})\" || rc=$?",
            "  - fi",
            f"  - python3 {AGE5_COMBINED_HEAVY_POLICY_SCRIPT} --provider gitlab --format text > {AGE5_COMBINED_HEAVY_POLICY_TEXT_PATH} || rc=$?",
            "  - if [ \"$rc\" -ne 0 ]; then",
            f"  -   python {AGE5_COMBINED_HEAVY_POLICY_SCRIPT} --provider gitlab --format text > {AGE5_COMBINED_HEAVY_POLICY_TEXT_PATH} || rc=$?",
            "  - fi",
            f"  - python3 {AGE5_COMBINED_HEAVY_POLICY_DIGEST_SCRIPT} {AGE5_COMBINED_HEAVY_POLICY_REPORT_PATH} --policy-text {AGE5_COMBINED_HEAVY_POLICY_TEXT_PATH} --summary-out {AGE5_COMBINED_HEAVY_POLICY_SUMMARY_PATH} > {AGE5_COMBINED_HEAVY_POLICY_SUMMARY_PATH} || rc=$?",
            "  - if [ \"$rc\" -ne 0 ]; then",
            f"  -   python {AGE5_COMBINED_HEAVY_POLICY_DIGEST_SCRIPT} {AGE5_COMBINED_HEAVY_POLICY_REPORT_PATH} --policy-text {AGE5_COMBINED_HEAVY_POLICY_TEXT_PATH} --summary-out {AGE5_COMBINED_HEAVY_POLICY_SUMMARY_PATH} > {AGE5_COMBINED_HEAVY_POLICY_SUMMARY_PATH} || rc=$?",
            "  - fi",
            "  - export DDN_ENABLE_DARWIN_PROBE=1",
            "  - export DDN_DARWIN_PROBE_ARCHIVE_KEEP=30",
            "  - export DDN_FIXED64_THREEWAY_MAX_AGE_MINUTES=360",
            "  - export DDN_DARWIN_PROBE_SCHEDULE_INTERVAL_MINUTES=180",
            "  - python tests/run_fixed64_darwin_probe_artifact.py --require-darwin --report-dir build/reports --json-out build/reports/fixed64_darwin_probe_artifact.detjson --archive-dir build/reports/darwin_probe_archive --archive-keep ${DDN_DARWIN_PROBE_ARCHIVE_KEEP:-30}",
            "  - python tests/run_fixed64_darwin_probe_schedule_policy_check.py --max-age-minutes ${DDN_FIXED64_THREEWAY_MAX_AGE_MINUTES:-360} --schedule-interval-minutes ${DDN_DARWIN_PROBE_SCHEDULE_INTERVAL_MINUTES:-180} --json-out build/reports/fixed64_darwin_probe_schedule_policy.detjson",
            "  - python tools/scripts/resolve_fixed64_threeway_inputs.py --report-dir build/reports --json-out build/reports/fixed64_threeway_inputs.detjson --strict-invalid --require-when-env DDN_ENABLE_DARWIN_PROBE",
            "  - export DDN_REQUIRE_FIXED64_3WAY=1",
            "  - python tests/run_ci_sanity_gate.py --json-out build/reports/ci_sanity_gate.detjson",
            "  - if [ \"$rc\" -eq 0 ]; then",
            "  -   python3 solutions/seamgrim_ui_mvp/tools/sync_featured_seed_catalog.py --check || rc=$?",
            "  -   if [ \"$rc\" -ne 0 ]; then",
            "  -     python solutions/seamgrim_ui_mvp/tools/sync_featured_seed_catalog.py --check || rc=$?",
            "  -   fi",
            "  - fi",
            "  - python tools/scripts/emit_ci_final_line.py --report-dir build/reports --print-artifacts --print-failure-digest 6 --print-failure-tail-lines 20 --fail-on-summary-verify-error --failure-brief-out build/reports/__PREFIX__.ci_fail_brief.txt --triage-json-out build/reports/__PREFIX__.ci_fail_triage.detjson",
            "  - python tools/scripts/emit_ci_final_line.py --report-dir build/reports --require-final-line --fail-on-summary-verify-error --failure-brief-out build/reports/__PREFIX__.ci_fail_brief.txt --triage-json-out build/reports/__PREFIX__.ci_fail_triage.detjson",
            "  - python tests/run_ci_emit_artifacts_check.py --report-dir build/reports --require-brief --require-triage",
            '  - echo "[ci-fixed64-darwin] DDN_ENABLE_DARWIN_PROBE=1 requires darwin agent"',
            "  - uname -s",
            "artifacts:",
            "  paths:",
            "    - build/reports/*.ci_gate_step_*.stdout.txt",
            "    - build/reports/*.ci_gate_step_*.stderr.txt",
            "    - build/reports/*.ci_fail_brief.txt",
            "    - build/reports/*.ci_fail_triage.detjson",
            "    - build/reports/darwin_probe_archive/*.detjson",
            "    - build/reports/fixed64_cross_platform_probe_darwin.detjson",
            "",
        ]
    )


def build_azure_text(aggregate_line: str) -> str:
    return "\n".join(
        [
            "schedules:",
            "  - cron: \"0 */3 * * *\"",
            "    displayName: fixed64 darwin probe cadence",
            "    branches:",
            "      include:",
            "        - main",
            "        - master",
            "    always: true",
            "variables:",
            '  DDN_ENABLE_DARWIN_PROBE: "0"',
            f'  {PROFILE_MATRIX_FULL_AGGREGATE_ENV_KEY}: "0"',
            f'  {PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY}: "0"',
            f'  {AGE5_COMBINED_HEAVY_ENV_KEY}: "0"',
            "steps:",
            "  - script: |",
            f"      eval \"$(python3 {PROFILE_MATRIX_SELFTEST_GATE_FLAGS_HELPER_SCRIPT} --provider azure --format shell)\" || rc=$?",
            "      if [ \"$rc\" -ne 0 ]; then",
            f"        eval \"$(python {PROFILE_MATRIX_SELFTEST_GATE_FLAGS_HELPER_SCRIPT} --provider azure --format shell)\" || rc=$?",
            "      fi",
            f"      {aggregate_line}",
            f"      eval \"$(python3 {PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SCRIPT} --provider azure --format shell)\" || rc=$?",
            "      if [ \"$rc\" -ne 0 ]; then",
            f"        eval \"$(python {PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SCRIPT} --provider azure --format shell)\" || rc=$?",
            "      fi",
            f"      eval \"$(python3 {AGE5_COMBINED_HEAVY_POLICY_SCRIPT} --provider azure --format shell --json-out {AGE5_COMBINED_HEAVY_POLICY_REPORT_PATH})\" || rc=$?",
            "      if [ \"$rc\" -ne 0 ]; then",
            f"        eval \"$(python {AGE5_COMBINED_HEAVY_POLICY_SCRIPT} --provider azure --format shell --json-out {AGE5_COMBINED_HEAVY_POLICY_REPORT_PATH})\" || rc=$?",
            "      fi",
            f"      python3 {AGE5_COMBINED_HEAVY_POLICY_SCRIPT} --provider azure --format text > {AGE5_COMBINED_HEAVY_POLICY_TEXT_PATH} || rc=$?",
            "      if [ \"$rc\" -ne 0 ]; then",
            f"        python {AGE5_COMBINED_HEAVY_POLICY_SCRIPT} --provider azure --format text > {AGE5_COMBINED_HEAVY_POLICY_TEXT_PATH} || rc=$?",
            "      fi",
            f"      python3 {AGE5_COMBINED_HEAVY_POLICY_DIGEST_SCRIPT} {AGE5_COMBINED_HEAVY_POLICY_REPORT_PATH} --policy-text {AGE5_COMBINED_HEAVY_POLICY_TEXT_PATH} --summary-out {AGE5_COMBINED_HEAVY_POLICY_SUMMARY_PATH} > {AGE5_COMBINED_HEAVY_POLICY_SUMMARY_PATH} || rc=$?",
            "      if [ \"$rc\" -ne 0 ]; then",
            f"        python {AGE5_COMBINED_HEAVY_POLICY_DIGEST_SCRIPT} {AGE5_COMBINED_HEAVY_POLICY_REPORT_PATH} --policy-text {AGE5_COMBINED_HEAVY_POLICY_TEXT_PATH} --summary-out {AGE5_COMBINED_HEAVY_POLICY_SUMMARY_PATH} > {AGE5_COMBINED_HEAVY_POLICY_SUMMARY_PATH} || rc=$?",
            "      fi",
            "      export DDN_ENABLE_DARWIN_PROBE=1",
            "      export DDN_DARWIN_PROBE_ARCHIVE_KEEP=30",
            "      export DDN_FIXED64_THREEWAY_MAX_AGE_MINUTES=360",
            "      export DDN_DARWIN_PROBE_SCHEDULE_INTERVAL_MINUTES=180",
            "      python tests/run_fixed64_darwin_probe_artifact.py --require-darwin --report-dir build/reports --json-out build/reports/fixed64_darwin_probe_artifact.detjson --archive-dir build/reports/darwin_probe_archive --archive-keep ${DDN_DARWIN_PROBE_ARCHIVE_KEEP:-30}",
            "      python tests/run_fixed64_darwin_probe_schedule_policy_check.py --max-age-minutes ${DDN_FIXED64_THREEWAY_MAX_AGE_MINUTES:-360} --schedule-interval-minutes ${DDN_DARWIN_PROBE_SCHEDULE_INTERVAL_MINUTES:-180} --json-out build/reports/fixed64_darwin_probe_schedule_policy.detjson",
            "      python tools/scripts/resolve_fixed64_threeway_inputs.py --report-dir build/reports --json-out build/reports/fixed64_threeway_inputs.detjson --strict-invalid --require-when-env DDN_ENABLE_DARWIN_PROBE",
            "      export DDN_REQUIRE_FIXED64_3WAY=1",
            "      python tests/run_ci_sanity_gate.py --json-out build/reports/ci_sanity_gate.detjson",
            "      if [ \"$rc\" -eq 0 ]; then",
            "        python3 solutions/seamgrim_ui_mvp/tools/sync_featured_seed_catalog.py --check || rc=$?",
            "        if [ \"$rc\" -ne 0 ]; then",
            "          python solutions/seamgrim_ui_mvp/tools/sync_featured_seed_catalog.py --check || rc=$?",
            "        fi",
            "      fi",
            "      python tools/scripts/emit_ci_final_line.py --report-dir build/reports --print-artifacts --print-failure-digest 6 --print-failure-tail-lines 20 --fail-on-summary-verify-error --failure-brief-out build/reports/__PREFIX__.ci_fail_brief.txt --triage-json-out build/reports/__PREFIX__.ci_fail_triage.detjson",
            "      python tools/scripts/emit_ci_final_line.py --report-dir build/reports --require-final-line --fail-on-summary-verify-error --failure-brief-out build/reports/__PREFIX__.ci_fail_brief.txt --triage-json-out build/reports/__PREFIX__.ci_fail_triage.detjson",
            "      python tests/run_ci_emit_artifacts_check.py --report-dir build/reports --require-brief --require-triage",
            '      echo "[ci-fixed64-darwin] DDN_ENABLE_DARWIN_PROBE=1 requires darwin agent"',
            "      uname -s",
            "  - task: PublishBuildArtifacts@1",
            "    inputs:",
            "      PathtoPublish: build/reports",
            "      ArtifactName: test_reports",
            "  - script: |",
            "      echo fixed64_cross_platform_probe_darwin.detjson",
            "",
        ]
    )


def run_check(
    root: Path,
    gitlab_path: Path,
    azure_path: Path,
    helper_path: Path | None = None,
    env: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_ci_pipeline_emit_flags_check.py",
        "--gitlab",
        str(gitlab_path),
        "--azure",
        str(azure_path),
    ]
    if helper_path is not None:
        cmd.extend(["--profile-matrix-full-real-smoke-helper", str(helper_path)])
    return run(cmd, cwd=root, env=env)


def load_progress_payload(path: Path | None) -> dict[str, object] | None:
    if path is None or not path.exists():
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return payload if isinstance(payload, dict) else None


def run_check_with_progress(
    root: Path,
    gitlab_path: Path,
    azure_path: Path,
    *,
    progress_path: Path | None = None,
    helper_path: Path | None = None,
    on_progress: callable | None = None,
) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_ci_pipeline_emit_flags_check.py",
        "--gitlab",
        str(gitlab_path),
        "--azure",
        str(azure_path),
    ]
    if helper_path is not None:
        cmd.extend(["--profile-matrix-full-real-smoke-helper", str(helper_path)])
    env = dict(os.environ)
    if progress_path is not None:
        env[PIPELINE_PROGRESS_ENV_KEY] = str(progress_path)
    if on_progress is not None:
        on_progress(load_progress_payload(progress_path))
    proc = run(cmd, cwd=root, env=env)
    if on_progress is not None:
        on_progress(load_progress_payload(progress_path))
    return proc


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    progress_path = str(os.environ.get(PROGRESS_ENV_KEY, "")).strip()
    started_at = time.perf_counter()
    current_case = "-"
    last_completed_case = "-"
    current_probe = "-"
    last_completed_probe = "-"
    cases_completed = 0

    def update_progress(status: str) -> None:
        write_progress_snapshot(
            progress_path,
            status=status,
            current_case=current_case,
            last_completed_case=last_completed_case,
            current_probe=current_probe,
            last_completed_probe=last_completed_probe,
            cases_completed=cases_completed,
            total_elapsed_ms=int((time.perf_counter() - started_at) * 1000),
        )

    def start_case(name: str) -> None:
        nonlocal current_case
        current_case = name
        update_progress("running")

    def complete_case(name: str) -> None:
        nonlocal current_case, last_completed_case, current_probe, last_completed_probe, cases_completed
        last_completed_case = name
        cases_completed += 1
        current_case = "-"
        current_probe = "-"
        last_completed_probe = "-"
        update_progress("running")

    def start_probe(name: str) -> None:
        nonlocal current_probe
        current_probe = name
        update_progress("running")

    def set_probe(name: str) -> None:
        nonlocal current_probe
        current_probe = name
        update_progress("running")

    def complete_probe(name: str) -> None:
        nonlocal current_probe, last_completed_probe
        last_completed_probe = name
        current_probe = "-"
        update_progress("running")

    update_progress("running")
    aggregate_ok = (
        "python tests/run_ci_aggregate_gate.py --report-dir build/reports --skip-core-tests "
        "--fast-fail --backup-hygiene --auto-prefix-env CI_PIPELINE_ID,CI_JOB_ID "
        "--clean-prefixed-reports --quiet-success-logs --compact-step-logs "
        "--step-log-dir build/reports --step-log-failed-only --checklist-skip-seed-cli "
        "--checklist-skip-ui-common --runtime-5min-skip-ui-common --runtime-5min-skip-showcase-check "
        "--fixed64-threeway-max-report-age-minutes ${DDN_FIXED64_THREEWAY_MAX_AGE_MINUTES:-360} "
        "--require-fixed64-3way "
        f"${PROFILE_MATRIX_SELFTEST_GATE_FLAGS_VAR}"
    )
    baseline_gitlab_text = build_gitlab_text(aggregate_ok)
    baseline_azure_text = build_azure_text(aggregate_ok)

    with tempfile.TemporaryDirectory(prefix="ci_pipeline_flags_selftest_") as td:
        temp_root = Path(td)
        gitlab = temp_root / "gitlab-ci.yml"
        azure = temp_root / "azure-pipelines.yml"

        start_case("baseline_should_pass")
        write_text(gitlab, baseline_gitlab_text)
        write_text(azure, baseline_azure_text)
        pass_proc = run_check(root, gitlab, azure)
        if pass_proc.returncode != 0:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=baseline_should_pass")
            if pass_proc.stdout.strip():
                print(pass_proc.stdout.strip())
            if pass_proc.stderr.strip():
                print(pass_proc.stderr.strip())
            return 1
        complete_case("baseline_should_pass")

        start_case("missing_token_should_fail")
        aggregate_missing = aggregate_ok.replace("--checklist-skip-ui-common", "")
        write_text(gitlab, baseline_gitlab_text)
        write_text(azure, build_azure_text(aggregate_missing))
        miss_proc = run_check(root, gitlab, azure)
        if miss_proc.returncode == 0:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=missing_token_should_fail")
            return 1
        merged_miss = "\n".join([miss_proc.stdout or "", miss_proc.stderr or ""])
        if "azure.aggregate: line#1 missing token --checklist-skip-ui-common" not in merged_miss:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=missing_token_message_missing")
            if miss_proc.stdout.strip():
                print(miss_proc.stdout.strip())
            if miss_proc.stderr.strip():
                print(miss_proc.stderr.strip())
            return 1
        complete_case("missing_token_should_fail")

        start_case("missing_runtime_ui_token_should_fail")
        start_probe("prepare_runtime_ui_variant")
        aggregate_missing_runtime_ui = aggregate_ok.replace("--runtime-5min-skip-ui-common", "")
        write_text(gitlab, baseline_gitlab_text)
        write_text(azure, build_azure_text(aggregate_missing_runtime_ui))
        complete_probe("prepare_runtime_ui_variant")
        start_probe("spawn_runtime_ui_check")
        complete_probe("spawn_runtime_ui_check")
        start_probe("wait_runtime_ui_check")
        runtime_ui_progress = temp_root / "runtime_ui_missing.progress.detjson"

        def reflect_runtime_ui_progress(payload: dict[str, object] | None) -> None:
            if not isinstance(payload, dict):
                return
            current_section = str(payload.get("current_section", "-")).strip() or "-"
            last_section = str(payload.get("last_completed_section", "-")).strip() or "-"
            if current_section != "-":
                set_probe(f"wait_runtime_ui_check.{current_section}")
            elif last_section != "-":
                parts = [item.strip() for item in last_section.split(",") if item.strip()]
                section = parts[-1] if parts else last_section
                set_probe(f"wait_runtime_ui_check.{section}")

        miss_runtime_proc = run_check_with_progress(
            root,
            gitlab,
            azure,
            progress_path=runtime_ui_progress,
            on_progress=reflect_runtime_ui_progress,
        )
        if miss_runtime_proc.returncode == 0:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=missing_runtime_ui_token_should_fail")
            return 1
        complete_probe("wait_runtime_ui_check")
        start_probe("collect_runtime_ui_output")
        merged_miss_runtime = "\n".join([miss_runtime_proc.stdout or "", miss_runtime_proc.stderr or ""])
        complete_probe("collect_runtime_ui_output")
        start_probe("validate_runtime_ui_message")
        if "azure.aggregate: line#1 missing token --runtime-5min-skip-ui-common" not in merged_miss_runtime:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=missing_runtime_ui_token_message_missing")
            if miss_runtime_proc.stdout.strip():
                print(miss_runtime_proc.stdout.strip())
            if miss_runtime_proc.stderr.strip():
                print(miss_runtime_proc.stderr.strip())
            return 1
        complete_probe("validate_runtime_ui_message")
        complete_case("missing_runtime_ui_token_should_fail")

        start_case("missing_runtime_showcase_token_should_fail")
        aggregate_missing_runtime_showcase = aggregate_ok.replace("--runtime-5min-skip-showcase-check", "")
        write_text(gitlab, baseline_gitlab_text)
        write_text(azure, build_azure_text(aggregate_missing_runtime_showcase))
        miss_runtime_showcase_proc = run_check(root, gitlab, azure)
        if miss_runtime_showcase_proc.returncode == 0:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=missing_runtime_showcase_token_should_fail")
            return 1
        merged_miss_runtime_showcase = "\n".join(
            [miss_runtime_showcase_proc.stdout or "", miss_runtime_showcase_proc.stderr or ""]
        )
        if "azure.aggregate: line#1 missing token --runtime-5min-skip-showcase-check" not in merged_miss_runtime_showcase:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=missing_runtime_showcase_token_message_missing")
            if miss_runtime_showcase_proc.stdout.strip():
                print(miss_runtime_showcase_proc.stdout.strip())
            if miss_runtime_showcase_proc.stderr.strip():
                print(miss_runtime_showcase_proc.stderr.strip())
            return 1
        complete_case("missing_runtime_showcase_token_should_fail")

        start_case("missing_fixed64_age_token_should_fail")
        aggregate_missing_fixed64_age = aggregate_ok.replace("--fixed64-threeway-max-report-age-minutes ${DDN_FIXED64_THREEWAY_MAX_AGE_MINUTES:-360} ", "")
        write_text(gitlab, baseline_gitlab_text)
        write_text(azure, build_azure_text(aggregate_missing_fixed64_age))
        miss_fixed64_age_proc = run_check(root, gitlab, azure)
        if miss_fixed64_age_proc.returncode == 0:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=missing_fixed64_age_token_should_fail")
            return 1
        merged_miss_fixed64_age = "\n".join([miss_fixed64_age_proc.stdout or "", miss_fixed64_age_proc.stderr or ""])
        if "azure.aggregate: line#1 missing token --fixed64-threeway-max-report-age-minutes" not in merged_miss_fixed64_age:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=missing_fixed64_age_token_message_missing")
            if miss_fixed64_age_proc.stdout.strip():
                print(miss_fixed64_age_proc.stdout.strip())
            if miss_fixed64_age_proc.stderr.strip():
                print(miss_fixed64_age_proc.stderr.strip())
            return 1
        complete_case("missing_fixed64_age_token_should_fail")

        start_case("missing_archive_keep_env_should_fail")
        gitlab_without_archive_keep_env = baseline_gitlab_text.replace(
            "DDN_DARWIN_PROBE_ARCHIVE_KEEP",
            "DARWIN_ARCHIVE_KEEP_ENV_BROKEN",
        )
        write_text(gitlab, gitlab_without_archive_keep_env)
        write_text(azure, baseline_azure_text)
        miss_archive_keep_env_proc = run_check(root, gitlab, azure)
        if miss_archive_keep_env_proc.returncode == 0:
            print("check=ci_pipeline_emit_flags_selftest detail=missing_archive_keep_env_should_fail")
            return 1
        merged_archive_keep_env = "\n".join(
            [miss_archive_keep_env_proc.stdout or "", miss_archive_keep_env_proc.stderr or ""]
        )
        if "gitlab.fixed64_threeway: missing token DDN_DARWIN_PROBE_ARCHIVE_KEEP" not in merged_archive_keep_env:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=missing_archive_keep_env_message_missing")
            if miss_archive_keep_env_proc.stdout.strip():
                print(miss_archive_keep_env_proc.stdout.strip())
            if miss_archive_keep_env_proc.stderr.strip():
                print(miss_archive_keep_env_proc.stderr.strip())
            return 1
        complete_case("missing_archive_keep_env_should_fail")

        start_case("missing_schedule_interval_env_should_fail")
        gitlab_without_schedule_interval_env = baseline_gitlab_text.replace(
            "DDN_DARWIN_PROBE_SCHEDULE_INTERVAL_MINUTES",
            "DARWIN_PROBE_SCHEDULE_INTERVAL_ENV_BROKEN",
        )
        write_text(gitlab, gitlab_without_schedule_interval_env)
        write_text(azure, baseline_azure_text)
        miss_schedule_interval_env_proc = run_check(root, gitlab, azure)
        if miss_schedule_interval_env_proc.returncode == 0:
            print("check=ci_pipeline_emit_flags_selftest detail=missing_schedule_interval_env_should_fail")
            return 1
        merged_schedule_interval_env = "\n".join(
            [miss_schedule_interval_env_proc.stdout or "", miss_schedule_interval_env_proc.stderr or ""]
        )
        if "gitlab.fixed64_threeway: missing token DDN_DARWIN_PROBE_SCHEDULE_INTERVAL_MINUTES" not in merged_schedule_interval_env:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=missing_schedule_interval_env_message_missing")
            if miss_schedule_interval_env_proc.stdout.strip():
                print(miss_schedule_interval_env_proc.stdout.strip())
            if miss_schedule_interval_env_proc.stderr.strip():
                print(miss_schedule_interval_env_proc.stderr.strip())
            return 1
        complete_case("missing_schedule_interval_env_should_fail")

        start_case("missing_full_real_smoke_helper_should_fail")
        gitlab_without_full_real_smoke_helper = baseline_gitlab_text.replace(
            PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SCRIPT,
            "tools/scripts/MISSING_profile_matrix_full_real_smoke_policy.py",
        )
        write_text(gitlab, gitlab_without_full_real_smoke_helper)
        write_text(azure, baseline_azure_text)
        miss_full_real_smoke_helper_proc = run_check(root, gitlab, azure)
        if miss_full_real_smoke_helper_proc.returncode == 0:
            print("check=ci_pipeline_emit_flags_selftest detail=missing_full_real_smoke_helper_should_fail")
            return 1
        merged_full_real_smoke_helper = "\n".join(
            [miss_full_real_smoke_helper_proc.stdout or "", miss_full_real_smoke_helper_proc.stderr or ""]
        )
        if (
            f"gitlab.profile_matrix_full_real_smoke: missing token {PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SCRIPT}"
            not in merged_full_real_smoke_helper
        ):
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=missing_full_real_smoke_helper_message_missing")
            if miss_full_real_smoke_helper_proc.stdout.strip():
                print(miss_full_real_smoke_helper_proc.stdout.strip())
            if miss_full_real_smoke_helper_proc.stderr.strip():
                print(miss_full_real_smoke_helper_proc.stderr.strip())
            return 1
        complete_case("missing_full_real_smoke_helper_should_fail")

        start_case("missing_selftest_gate_helper_should_fail")
        gitlab_without_selftest_gate_helper = baseline_gitlab_text.replace(
            PROFILE_MATRIX_SELFTEST_GATE_FLAGS_HELPER_SCRIPT,
            "tools/scripts/MISSING_profile_matrix_selftest_gate_flags.py",
        )
        write_text(gitlab, gitlab_without_selftest_gate_helper)
        write_text(azure, baseline_azure_text)
        miss_selftest_gate_helper_proc = run_check(root, gitlab, azure)
        if miss_selftest_gate_helper_proc.returncode == 0:
            print("check=ci_pipeline_emit_flags_selftest detail=missing_selftest_gate_helper_should_fail")
            return 1
        merged_selftest_gate_helper = "\n".join(
            [miss_selftest_gate_helper_proc.stdout or "", miss_selftest_gate_helper_proc.stderr or ""]
        )
        if (
            f"gitlab.profile_matrix_selftest_gate: missing token {PROFILE_MATRIX_SELFTEST_GATE_FLAGS_HELPER_SCRIPT}"
            not in merged_selftest_gate_helper
        ):
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=missing_selftest_gate_helper_message_missing")
            if miss_selftest_gate_helper_proc.stdout.strip():
                print(miss_selftest_gate_helper_proc.stdout.strip())
            if miss_selftest_gate_helper_proc.stderr.strip():
                print(miss_selftest_gate_helper_proc.stderr.strip())
            return 1
        complete_case("missing_selftest_gate_helper_should_fail")

        start_case("missing_azure_provider_policy_should_fail")
        azure_without_provider_token = baseline_azure_text.replace(
            "--provider azure --format shell",
            "--provider BROKEN --format shell",
        )
        write_text(gitlab, baseline_gitlab_text)
        write_text(azure, azure_without_provider_token)
        miss_azure_provider_proc = run_check(root, gitlab, azure)
        if miss_azure_provider_proc.returncode == 0:
            print("check=ci_pipeline_emit_flags_selftest detail=missing_azure_provider_policy_should_fail")
            return 1
        merged_azure_provider = "\n".join(
            [miss_azure_provider_proc.stdout or "", miss_azure_provider_proc.stderr or ""]
        )
        if "azure.profile_matrix_full_real_smoke: missing token --provider azure --format shell" not in merged_azure_provider:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=missing_azure_provider_policy_message_missing")
            if miss_azure_provider_proc.stdout.strip():
                print(miss_azure_provider_proc.stdout.strip())
            if miss_azure_provider_proc.stderr.strip():
                print(miss_azure_provider_proc.stderr.strip())
            return 1
        complete_case("missing_azure_provider_policy_should_fail")

        start_case("missing_enable_default_env_should_fail")
        gitlab_without_enable_default_env = baseline_gitlab_text.replace(
            'DDN_ENABLE_DARWIN_PROBE: "0"',
            'DDN_ENABLE_DARWIN_PROBE: "BROKEN"',
        )
        write_text(gitlab, gitlab_without_enable_default_env)
        write_text(azure, baseline_azure_text)
        miss_enable_default_env_proc = run_check(root, gitlab, azure)
        if miss_enable_default_env_proc.returncode == 0:
            print("check=ci_pipeline_emit_flags_selftest detail=missing_enable_default_env_should_fail")
            return 1
        merged_enable_default_env = "\n".join(
            [miss_enable_default_env_proc.stdout or "", miss_enable_default_env_proc.stderr or ""]
        )
        if 'gitlab.fixed64_threeway: missing token DDN_ENABLE_DARWIN_PROBE: "0"' not in merged_enable_default_env:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=missing_enable_default_env_message_missing")
            if miss_enable_default_env_proc.stdout.strip():
                print(miss_enable_default_env_proc.stdout.strip())
            if miss_enable_default_env_proc.stderr.strip():
                print(miss_enable_default_env_proc.stderr.strip())
            return 1
        complete_case("missing_enable_default_env_should_fail")

        start_case("missing_featured_seed_catalog_autogen_should_fail")
        gitlab_without_featured_seed_catalog_autogen = baseline_gitlab_text.replace(
            "  -   python3 solutions/seamgrim_ui_mvp/tools/sync_featured_seed_catalog.py --check || rc=$?\n",
            "",
        )
        write_text(gitlab, gitlab_without_featured_seed_catalog_autogen)
        write_text(azure, baseline_azure_text)
        miss_featured_seed_catalog_autogen_proc = run_check(root, gitlab, azure)
        if miss_featured_seed_catalog_autogen_proc.returncode == 0:
            print("check=ci_pipeline_emit_flags_selftest detail=missing_featured_seed_catalog_autogen_should_fail")
            return 1
        merged_featured_seed_catalog_autogen = "\n".join(
            [miss_featured_seed_catalog_autogen_proc.stdout or "", miss_featured_seed_catalog_autogen_proc.stderr or ""]
        )
        if (
            "gitlab.featured_seed_catalog_autogen: missing token python3 solutions/seamgrim_ui_mvp/tools/sync_featured_seed_catalog.py --check"
            not in merged_featured_seed_catalog_autogen
        ):
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=missing_featured_seed_catalog_autogen_message_missing")
            if miss_featured_seed_catalog_autogen_proc.stdout.strip():
                print(miss_featured_seed_catalog_autogen_proc.stdout.strip())
            if miss_featured_seed_catalog_autogen_proc.stderr.strip():
                print(miss_featured_seed_catalog_autogen_proc.stderr.strip())
            return 1
        complete_case("missing_featured_seed_catalog_autogen_should_fail")

        start_case("broken_policy_helper_scope_should_fail")
        broken_helper = temp_root / "broken_profile_matrix_full_real_smoke_policy.py"
        write_text(
            broken_helper,
            "\n".join(
                [
                    "import argparse, json",
                    f"SCHEMA = {PROFILE_MATRIX_FULL_REAL_SMOKE_POLICY_SCHEMA!r}",
                    f"ENV_KEY = {PROFILE_MATRIX_FULL_REAL_SMOKE_ENV_KEY!r}",
                    "parser = argparse.ArgumentParser()",
                    "parser.add_argument('--provider', required=True)",
                    "parser.add_argument('--format', default='text')",
                    "args = parser.parse_args()",
                    "payload = {",
                    "    'schema': SCHEMA,",
                    "    'provider': args.provider,",
                    "    'env_key': ENV_KEY,",
                    "    'scope': 'BROKEN_SCOPE',",
                    "    'enabled': False,",
                    "    'reason': 'default_off',",
                    "}",
                    "if args.format == 'json':",
                    "    print(json.dumps(payload))",
                    "elif args.format == 'shell':",
                    "    print(f'export {ENV_KEY}=0\\n[ci-profile-matrix-full-real-smoke] enabled provider={args.provider} reason=default_off scope=BROKEN_SCOPE')",
                    "else:",
                    "    print(f'[ci-profile-matrix-full-real-smoke-policy] provider={args.provider} status=disabled reason=default_off scope=BROKEN_SCOPE')",
                    "",
                ]
            ),
        )
        write_text(gitlab, baseline_gitlab_text)
        write_text(azure, baseline_azure_text)
        broken_helper_env = dict(os.environ)
        broken_helper_env[RUNTIME_CONTRACT_MINIMAL_ENV_KEY] = "1"
        broken_helper_proc = run_check(
            root,
            gitlab,
            azure,
            helper_path=broken_helper,
            env=broken_helper_env,
        )
        if broken_helper_proc.returncode == 0:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=broken_policy_helper_scope_should_fail")
            return 1
        merged_broken_helper = "\n".join(
            [broken_helper_proc.stdout or "", broken_helper_proc.stderr or ""]
        )
        if "runtime_json: scope mismatch" not in merged_broken_helper:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=broken_policy_helper_scope_message_missing")
            if broken_helper_proc.stdout.strip():
                print(broken_helper_proc.stdout.strip())
            if broken_helper_proc.stderr.strip():
                print(broken_helper_proc.stderr.strip())
            return 1
        complete_case("broken_policy_helper_scope_should_fail")

        start_case("broken_age5_policy_helper_scope_should_fail")
        broken_age5_helper = temp_root / "broken_age5_combined_heavy_policy.py"
        write_text(
            broken_age5_helper,
            "\n".join(
                [
                    "import argparse, json",
                    f"SCHEMA = {AGE5_COMBINED_HEAVY_POLICY_SCHEMA!r}",
                    f"ENV_KEY = {AGE5_COMBINED_HEAVY_ENV_KEY!r}",
                    "parser = argparse.ArgumentParser()",
                    "parser.add_argument('--provider', required=True)",
                    "parser.add_argument('--format', default='text')",
                    "args = parser.parse_args()",
                    "payload = {",
                    "    'schema': SCHEMA,",
                    "    'provider': args.provider,",
                    "    'env_key': ENV_KEY,",
                    "    'scope': 'BROKEN_SCOPE',",
                    "    'enabled': False,",
                    "    'reason': 'default_off',",
                    "}",
                    "if args.format == 'json':",
                    "    print(json.dumps(payload))",
                    "elif args.format == 'shell':",
                    "    print(f'export {ENV_KEY}=0\\n[age5-combined-heavy-policy] enabled provider={args.provider} reason=default_off scope=BROKEN_SCOPE')",
                    "else:",
                    "    print(f'[age5-combined-heavy-policy] provider={args.provider} status=disabled reason=default_off scope=BROKEN_SCOPE')",
                    "",
                ]
            ),
        )
        write_text(gitlab, baseline_gitlab_text)
        write_text(azure, baseline_azure_text)
        broken_age5_helper_env = dict(os.environ)
        broken_age5_helper_env[RUNTIME_CONTRACT_MINIMAL_ENV_KEY] = "1"
        broken_age5_helper_proc = run(
            [
                sys.executable,
                "tests/run_ci_pipeline_emit_flags_check.py",
                "--gitlab",
                str(gitlab),
                "--azure",
                str(azure),
                "--age5-combined-heavy-helper",
                str(broken_age5_helper),
            ],
            cwd=root,
            env=broken_age5_helper_env,
        )
        if broken_age5_helper_proc.returncode == 0:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=broken_age5_policy_helper_scope_should_fail")
            return 1
        merged_broken_age5_helper = "\n".join(
            [broken_age5_helper_proc.stdout or "", broken_age5_helper_proc.stderr or ""]
        )
        if "age5_runtime_json: scope mismatch" not in merged_broken_age5_helper:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=broken_age5_policy_helper_scope_message_missing")
            if broken_age5_helper_proc.stdout.strip():
                print(broken_age5_helper_proc.stdout.strip())
            if broken_age5_helper_proc.stderr.strip():
                print(broken_age5_helper_proc.stderr.strip())
            return 1
        complete_case("broken_age5_policy_helper_scope_should_fail")

        start_case("forbidden_token_should_fail")
        aggregate_forbidden = f"{aggregate_ok} --skip-5min-checklist"
        write_text(gitlab, build_gitlab_text(aggregate_forbidden))
        write_text(azure, baseline_azure_text)
        forbid_proc = run_check(root, gitlab, azure)
        if forbid_proc.returncode == 0:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=forbidden_token_should_fail")
            return 1
        merged_forbid = "\n".join([forbid_proc.stdout or "", forbid_proc.stderr or ""])
        if "gitlab.aggregate: line#1 forbidden token --skip-5min-checklist" not in merged_forbid:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=forbidden_token_message_missing")
            if forbid_proc.stdout.strip():
                print(forbid_proc.stdout.strip())
            if forbid_proc.stderr.strip():
                print(forbid_proc.stderr.strip())
            return 1
        complete_case("forbidden_token_should_fail")

        start_case("missing_aggregate_line_should_fail")
        write_text(gitlab, build_gitlab_text("python tests/run_ci_sanity_gate.py --json-out build/reports/ci_sanity_gate.detjson"))
        write_text(azure, build_azure_text("python tests/run_ci_sanity_gate.py --json-out build/reports/ci_sanity_gate.detjson"))
        missing_line_proc = run_check(root, gitlab, azure)
        if missing_line_proc.returncode == 0:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=missing_aggregate_line_should_fail")
            return 1
        merged_missing_line = "\n".join([missing_line_proc.stdout or "", missing_line_proc.stderr or ""])
        if "missing aggregate gate invocation line" not in merged_missing_line:
            update_progress("fail")
            print("check=ci_pipeline_emit_flags_selftest detail=missing_aggregate_line_message_missing")
            if missing_line_proc.stdout.strip():
                print(missing_line_proc.stdout.strip())
            if missing_line_proc.stderr.strip():
                print(missing_line_proc.stderr.strip())
            return 1
        complete_case("missing_aggregate_line_should_fail")

    update_progress("pass")
    print("ci pipeline emit flags check selftest ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
