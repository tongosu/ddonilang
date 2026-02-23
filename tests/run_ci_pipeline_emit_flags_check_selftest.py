#!/usr/bin/env python
from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text.rstrip() + "\n", encoding="utf-8")


def build_gitlab_text(aggregate_line: str) -> str:
    return "\n".join(
        [
            "image: rust:1.75",
            "script:",
            f"  - {aggregate_line}",
            "  - export DDN_ENABLE_DARWIN_PROBE=1",
            "  - python tests/run_fixed64_darwin_probe_artifact.py --require-darwin --report-dir build/reports --json-out build/reports/fixed64_darwin_probe_artifact.detjson",
            "  - python tools/scripts/resolve_fixed64_threeway_inputs.py --report-dir build/reports --json-out build/reports/fixed64_threeway_inputs.detjson --strict-invalid --require-when-env DDN_ENABLE_DARWIN_PROBE",
            "  - export DDN_REQUIRE_FIXED64_3WAY=1",
            "  - python tests/run_ci_sanity_gate.py --json-out build/reports/ci_sanity_gate.detjson",
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
            "    - build/reports/fixed64_cross_platform_probe_darwin.detjson",
            "",
        ]
    )


def build_azure_text(aggregate_line: str) -> str:
    return "\n".join(
        [
            "steps:",
            "  - script: |",
            f"      {aggregate_line}",
            "      export DDN_ENABLE_DARWIN_PROBE=1",
            "      python tests/run_fixed64_darwin_probe_artifact.py --require-darwin --report-dir build/reports --json-out build/reports/fixed64_darwin_probe_artifact.detjson",
            "      python tools/scripts/resolve_fixed64_threeway_inputs.py --report-dir build/reports --json-out build/reports/fixed64_threeway_inputs.detjson --strict-invalid --require-when-env DDN_ENABLE_DARWIN_PROBE",
            "      export DDN_REQUIRE_FIXED64_3WAY=1",
            "      python tests/run_ci_sanity_gate.py --json-out build/reports/ci_sanity_gate.detjson",
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


def run_check(root: Path, gitlab_path: Path, azure_path: Path) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        "tests/run_ci_pipeline_emit_flags_check.py",
        "--gitlab",
        str(gitlab_path),
        "--azure",
        str(azure_path),
    ]
    return run(cmd, cwd=root)


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    aggregate_ok = (
        "python tests/run_ci_aggregate_gate.py --report-dir build/reports --skip-core-tests "
        "--fast-fail --backup-hygiene --auto-prefix-env CI_PIPELINE_ID,CI_JOB_ID "
        "--clean-prefixed-reports --quiet-success-logs --compact-step-logs "
        "--step-log-dir build/reports --step-log-failed-only --checklist-skip-seed-cli "
        "--checklist-skip-ui-common --require-fixed64-3way"
    )

    with tempfile.TemporaryDirectory(prefix="ci_pipeline_flags_selftest_") as td:
        temp_root = Path(td)
        gitlab = temp_root / "gitlab-ci.yml"
        azure = temp_root / "azure-pipelines.yml"

        write_text(gitlab, build_gitlab_text(aggregate_ok))
        write_text(azure, build_azure_text(aggregate_ok))
        pass_proc = run_check(root, gitlab, azure)
        if pass_proc.returncode != 0:
            print("check=ci_pipeline_emit_flags_selftest detail=baseline_should_pass")
            if pass_proc.stdout.strip():
                print(pass_proc.stdout.strip())
            if pass_proc.stderr.strip():
                print(pass_proc.stderr.strip())
            return 1

        aggregate_missing = aggregate_ok.replace("--checklist-skip-ui-common", "")
        write_text(gitlab, build_gitlab_text(aggregate_ok))
        write_text(azure, build_azure_text(aggregate_missing))
        miss_proc = run_check(root, gitlab, azure)
        if miss_proc.returncode == 0:
            print("check=ci_pipeline_emit_flags_selftest detail=missing_token_should_fail")
            return 1
        merged_miss = "\n".join([miss_proc.stdout or "", miss_proc.stderr or ""])
        if "azure.aggregate: line#1 missing token --checklist-skip-ui-common" not in merged_miss:
            print("check=ci_pipeline_emit_flags_selftest detail=missing_token_message_missing")
            if miss_proc.stdout.strip():
                print(miss_proc.stdout.strip())
            if miss_proc.stderr.strip():
                print(miss_proc.stderr.strip())
            return 1

        aggregate_forbidden = f"{aggregate_ok} --skip-5min-checklist"
        write_text(gitlab, build_gitlab_text(aggregate_forbidden))
        write_text(azure, build_azure_text(aggregate_ok))
        forbid_proc = run_check(root, gitlab, azure)
        if forbid_proc.returncode == 0:
            print("check=ci_pipeline_emit_flags_selftest detail=forbidden_token_should_fail")
            return 1
        merged_forbid = "\n".join([forbid_proc.stdout or "", forbid_proc.stderr or ""])
        if "gitlab.aggregate: line#1 forbidden token --skip-5min-checklist" not in merged_forbid:
            print("check=ci_pipeline_emit_flags_selftest detail=forbidden_token_message_missing")
            if forbid_proc.stdout.strip():
                print(forbid_proc.stdout.strip())
            if forbid_proc.stderr.strip():
                print(forbid_proc.stderr.strip())
            return 1

        write_text(gitlab, build_gitlab_text("python tests/run_ci_sanity_gate.py --json-out build/reports/ci_sanity_gate.detjson"))
        write_text(azure, build_azure_text("python tests/run_ci_sanity_gate.py --json-out build/reports/ci_sanity_gate.detjson"))
        missing_line_proc = run_check(root, gitlab, azure)
        if missing_line_proc.returncode == 0:
            print("check=ci_pipeline_emit_flags_selftest detail=missing_aggregate_line_should_fail")
            return 1
        merged_missing_line = "\n".join([missing_line_proc.stdout or "", missing_line_proc.stderr or ""])
        if "missing aggregate gate invocation line" not in merged_missing_line:
            print("check=ci_pipeline_emit_flags_selftest detail=missing_aggregate_line_message_missing")
            if missing_line_proc.stdout.strip():
                print(missing_line_proc.stdout.strip())
            if missing_line_proc.stderr.strip():
                print(missing_line_proc.stderr.strip())
            return 1

    print("ci pipeline emit flags check selftest ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
