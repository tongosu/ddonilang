#!/usr/bin/env python
from __future__ import annotations

import argparse
import sys
from pathlib import Path


def fail(msg: str) -> int:
    print(f"[ci-pipeline-emit-flags-check] fail: {msg}", file=sys.stderr)
    return 1


def load_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except Exception:
        return ""


def require_tokens(text: str, label: str, tokens: list[str], errors: list[str]) -> None:
    for token in tokens:
        if token not in text:
            errors.append(f"{label}: missing token {token}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Check CI pipeline scripts keep required aggregate/emitter flags")
    parser.add_argument("--gitlab", default=".gitlab-ci.yml", help="path to .gitlab-ci.yml")
    parser.add_argument("--azure", default="azure-pipelines.yml", help="path to azure-pipelines.yml")
    args = parser.parse_args()

    gitlab_path = Path(args.gitlab)
    azure_path = Path(args.azure)
    if not gitlab_path.exists():
        return fail(f"missing file: {gitlab_path}")
    if not azure_path.exists():
        return fail(f"missing file: {azure_path}")

    gitlab_text = load_text(gitlab_path)
    azure_text = load_text(azure_path)
    errors: list[str] = []

    aggregate_tokens = [
        "tests/run_ci_aggregate_gate.py",
        "--backup-hygiene",
        "--quiet-success-logs",
        "--compact-step-logs",
        "--step-log-dir build/reports",
        "--step-log-failed-only",
    ]
    fixed64_threeway_tokens = [
        "DDN_REQUIRE_FIXED64_3WAY",
        "--require-fixed64-3way",
        "DDN_DARWIN_PROBE_REPORT",
        "fixed64_cross_platform_probe_darwin.detjson",
    ]
    emit_tokens = [
        "tools/scripts/emit_ci_final_line.py",
        "--print-artifacts",
        "--print-failure-digest 6",
        "--print-failure-tail-lines 20",
        "--fail-on-summary-verify-error",
        "--failure-brief-out build/reports/__PREFIX__.ci_fail_brief.txt",
        "--triage-json-out build/reports/__PREFIX__.ci_fail_triage.detjson",
    ]
    emit_require_tokens = [
        "tools/scripts/emit_ci_final_line.py",
        "--require-final-line",
        "--fail-on-summary-verify-error",
        "--failure-brief-out build/reports/__PREFIX__.ci_fail_brief.txt",
        "--triage-json-out build/reports/__PREFIX__.ci_fail_triage.detjson",
    ]
    emit_artifacts_check_tokens = [
        "tests/run_ci_emit_artifacts_check.py",
        "--report-dir build/reports",
        "--require-brief",
        "--require-triage",
    ]
    sanity_tokens = [
        "tests/run_ci_sanity_gate.py",
        "--json-out build/reports/ci_sanity_gate.detjson",
    ]

    require_tokens(gitlab_text, "gitlab.aggregate", aggregate_tokens, errors)
    require_tokens(gitlab_text, "gitlab.fixed64_threeway", fixed64_threeway_tokens, errors)
    require_tokens(gitlab_text, "gitlab.sanity", sanity_tokens, errors)
    require_tokens(gitlab_text, "gitlab.emit", emit_tokens, errors)
    require_tokens(gitlab_text, "gitlab.emit.require", emit_require_tokens, errors)
    require_tokens(gitlab_text, "gitlab.emit.artifacts_check", emit_artifacts_check_tokens, errors)
    require_tokens(
        gitlab_text,
        "gitlab.artifacts",
        [
            "build/reports/*.ci_gate_step_*.stdout.txt",
            "build/reports/*.ci_gate_step_*.stderr.txt",
            "build/reports/*.ci_fail_brief.txt",
            "build/reports/*.ci_fail_triage.detjson",
        ],
        errors,
    )

    require_tokens(azure_text, "azure.aggregate", aggregate_tokens, errors)
    require_tokens(azure_text, "azure.fixed64_threeway", fixed64_threeway_tokens, errors)
    require_tokens(azure_text, "azure.sanity", sanity_tokens, errors)
    require_tokens(azure_text, "azure.emit", emit_tokens, errors)
    require_tokens(azure_text, "azure.emit.require", emit_require_tokens, errors)
    require_tokens(azure_text, "azure.emit.artifacts_check", emit_artifacts_check_tokens, errors)
    require_tokens(
        azure_text,
        "azure.publish",
        [
            "PublishBuildArtifacts@1",
            "PathtoPublish: build/reports",
        ],
        errors,
    )

    if errors:
        print("[ci-pipeline-emit-flags-check] detected issues:")
        for row in errors[:24]:
            print(f" - {row}")
        return 1

    print(f"[ci-pipeline-emit-flags-check] ok gitlab={gitlab_path} azure={azure_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
