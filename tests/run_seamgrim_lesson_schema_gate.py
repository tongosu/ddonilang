#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from collections import Counter
from pathlib import Path


def fail(message: str, detail: str | None = None) -> int:
    print(message)
    if detail:
        print(detail)
    return 1


def run_command(root: Path, cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def canonical_json(data: object) -> str:
    return json.dumps(data, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def main() -> int:
    parser = argparse.ArgumentParser(description="Gate: seamgrim lesson AGE3 schema status")
    parser.add_argument(
        "--status-file",
        default="solutions/seamgrim_ui_mvp/lessons/schema_status.json",
        help="committed schema_status.json path",
    )
    parser.add_argument(
        "--require-promoted",
        action="store_true",
        help="promote dry-run에서 would_apply가 0이어야 통과합니다.",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    upgrade_tool = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "lesson_schema_upgrade.py"
    promote_tool = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "lesson_schema_promote.py"
    committed_status_path = root / args.status_file
    if not committed_status_path.exists():
        return fail(f"missing status file: {committed_status_path}")

    with tempfile.TemporaryDirectory(prefix="seamgrim_schema_gate_") as temp_dir:
        temp_dir_path = Path(temp_dir)
        summary_path = temp_dir_path / "summary.json"
        temp_status_path = temp_dir_path / "schema_status.generated.json"
        promote_report_path = temp_dir_path / "promote.report.json"

        enforce_cmd = [
            sys.executable,
            str(upgrade_tool),
            "--include-inputs",
            "--inject-mamadi",
            "--prefer-existing-preview",
            "--enforce-age3",
            "--quiet",
            "--summary-out",
            str(summary_path),
        ]
        enforce = run_command(root, enforce_cmd)
        if enforce.returncode != 0:
            detail = enforce.stderr.strip() or enforce.stdout.strip() or "lesson schema enforce failed"
            return fail("schema enforce command failed", detail)
        summary = json.loads(summary_path.read_text(encoding="utf-8"))
        if int(summary.get("failed", 0)) != 0:
            return fail("schema enforce failed", json.dumps(summary, ensure_ascii=False, indent=2))
        profile_counts = summary.get("profile_after_counts", {})
        non_age3_profiles = {k: v for k, v in profile_counts.items() if k != "age3_target" and int(v) > 0}
        if non_age3_profiles:
            return fail(f"non-age3 profiles found in summary: {non_age3_profiles}")

        status_cmd = [
            sys.executable,
            str(upgrade_tool),
            "--quiet",
            "--status-out",
            str(temp_status_path),
        ]
        status_run = run_command(root, status_cmd)
        if status_run.returncode != 0:
            detail = status_run.stderr.strip() or status_run.stdout.strip() or "status generation failed"
            return fail("status generation command failed", detail)

        generated = json.loads(temp_status_path.read_text(encoding="utf-8"))
        committed = json.loads(committed_status_path.read_text(encoding="utf-8"))
        if canonical_json(generated) != canonical_json(committed):
            return fail(
                "schema_status.json drift detected. regenerate with lesson_schema_upgrade.py --status-out ...",
                f"generated={temp_status_path} committed={committed_status_path}",
            )

        lessons = committed.get("lessons", [])
        profile_counter = Counter(row.get("effective_profile", "unknown") for row in lessons)
        if any(profile != "age3_target" and count > 0 for profile, count in profile_counter.items()):
            return fail(f"committed schema has non-age3 profiles: {dict(profile_counter)}")
        if any(not row.get("has_preview") for row in lessons):
            return fail("committed schema has lesson without preview")

        promote_cmd = [
            sys.executable,
            str(promote_tool),
            "--include-inputs",
            "--fail-on-missing-preview",
            "--json-out",
            str(promote_report_path),
        ]
        promote = run_command(root, promote_cmd)
        if promote.returncode != 0:
            detail = promote.stderr.strip() or promote.stdout.strip() or "lesson schema promote dry-run failed"
            return fail("schema promote dry-run command failed", detail)
        promote_report = json.loads(promote_report_path.read_text(encoding="utf-8"))
        if int(promote_report.get("skipped_no_preview", 0)) != 0:
            missing_rows = [
                row
                for row in promote_report.get("rows", [])
                if row.get("status") == "missing_preview"
            ][:5]
            return fail(
                "promote report has missing preview",
                f"missing_preview={promote_report.get('skipped_no_preview', 0)} samples={missing_rows}",
            )
        if args.require_promoted and int(promote_report.get("would_apply", 0)) != 0:
            return fail(
                "promote report has pending source updates (would_apply > 0)",
                f"would_apply={promote_report.get('would_apply', 0)} targets={promote_report.get('targets', 0)}",
            )

    print("seamgrim lesson schema gate ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
