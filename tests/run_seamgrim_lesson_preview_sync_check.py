#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Check lesson preview/source sync via lesson_schema_promote dry-run")
    parser.add_argument(
        "--require-synced",
        action="store_true",
        help="fail when promote dry-run reports would_apply > 0",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    root = Path(__file__).resolve().parent.parent
    tool = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "lesson_schema_promote.py"
    if not tool.exists():
        print(f"check=lesson_preview_sync detail=tool_missing:{tool}")
        return 1

    with tempfile.TemporaryDirectory(prefix="seamgrim_lesson_preview_sync_") as temp_dir:
        report_path = Path(temp_dir) / "lesson_preview_sync.detjson"
        cmd = [
            sys.executable,
            str(tool),
            "--include-inputs",
            "--fail-on-missing-preview",
            "--json-out",
            str(report_path),
        ]
        if bool(args.require_synced):
            cmd.append("--fail-on-would-apply")

        proc = subprocess.run(
            cmd,
            cwd=root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        stdout = str(proc.stdout or "").strip()
        stderr = str(proc.stderr or "").strip()

        if proc.returncode != 0:
            merged = "\n".join([line for line in [stdout, stderr] if line]).strip()
            if "PROMOTE_FAIL would_apply=" in merged:
                print(f"check=lesson_preview_sync detail=would_apply_nonzero:{merged}")
                return 1
            if "PROMOTE_FAIL missing_preview=" in merged:
                print(f"check=lesson_preview_sync detail=missing_preview_nonzero:{merged}")
                return 1
            detail = merged or "promote_dry_run_failed"
            print(f"check=lesson_preview_sync detail=tool_failed:{detail}")
            return 1

        if not report_path.exists():
            print("check=lesson_preview_sync detail=report_missing")
            return 1
        try:
            payload = json.loads(report_path.read_text(encoding="utf-8"))
        except Exception as exc:
            print(f"check=lesson_preview_sync detail=report_parse_failed:{exc}")
            return 1

    if payload.get("schema") != "seamgrim.lesson.schema_promote.v1":
        print("check=lesson_preview_sync detail=schema_mismatch")
        return 1

    targets = int(payload.get("targets", 0))
    would_apply = int(payload.get("would_apply", 0))
    missing_preview = int(payload.get("skipped_no_preview", 0))
    if missing_preview > 0:
        print(
            "check=lesson_preview_sync detail="
            f"missing_preview_nonzero:targets={targets}:missing_preview={missing_preview}:would_apply={would_apply}"
        )
        return 1
    if bool(args.require_synced) and would_apply > 0:
        print(
            "check=lesson_preview_sync detail="
            f"would_apply_nonzero:targets={targets}:would_apply={would_apply}:missing_preview={missing_preview}"
        )
        return 1

    print(
        "check=lesson_preview_sync detail="
        f"ok:targets={targets}:would_apply={would_apply}:missing_preview={missing_preview}:require_synced={int(bool(args.require_synced))}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
