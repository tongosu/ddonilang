#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path


LEGACY_WARNING_KEYS = (
    "legacy_show",
    "legacy_solver_bind_eq",
    "legacy_storage_block",
)


def fail(message: str) -> int:
    print(message)
    return 1


def run_schema_audit(root: Path, report_path: Path, include_preview: bool) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        str(root / "solutions" / "seamgrim_ui_mvp" / "tools" / "lesson_schema_audit.py"),
        "--json-out",
        str(report_path),
        "--limit",
        "0",
    ]
    if include_preview:
        cmd.append("--include-preview")
    return subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def summarize_rows(rows: list[dict], top: int) -> list[dict[str, object]]:
    warned: list[dict[str, object]] = []
    for row in rows:
        if not isinstance(row, dict):
            continue
        path = str(row.get("path", "")).strip()
        if not path:
            continue
        warning_count = 0
        for key in LEGACY_WARNING_KEYS:
            warning_count += int(row.get(key, 0) or 0)
        if warning_count <= 0:
            continue
        warned.append(
            {
                "path": path,
                "warning_count": warning_count,
                "legacy_show": int(row.get("legacy_show", 0) or 0),
                "legacy_solver_bind_eq": int(row.get("legacy_solver_bind_eq", 0) or 0),
                "legacy_storage_block": int(row.get("legacy_storage_block", 0) or 0),
            }
        )
    warned.sort(key=lambda item: (-int(item["warning_count"]), str(item["path"])))
    return warned[: max(0, top)]


def main() -> int:
    parser = argparse.ArgumentParser(description="Check seamgrim lesson legacy warning tokens")
    parser.add_argument(
        "--report",
        default="build/reports/seamgrim_lesson_warning_tokens_report.detjson",
        help="output report path",
    )
    parser.add_argument(
        "--require-zero",
        action="store_true",
        help="fail when warning token total is non-zero",
    )
    parser.add_argument(
        "--include-preview",
        action="store_true",
        help="include *.age3.preview.ddn while scanning",
    )
    parser.add_argument(
        "--top",
        type=int,
        default=10,
        help="max rows recorded in top_legacy_rows",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    out_path = root / args.report
    out_path.parent.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory(prefix="seamgrim_lesson_warning_tokens_") as temp_dir:
        audit_json = Path(temp_dir) / "lesson_schema_audit.detjson"
        proc = run_schema_audit(root, audit_json, include_preview=bool(args.include_preview))
        if proc.returncode != 0:
            detail = (proc.stderr or proc.stdout or "").strip()
            if not detail:
                detail = "lesson_schema_audit command failed"
            return fail(f"check=lesson_warning_tokens detail=audit_failed:{detail}")
        try:
            audit_doc = json.loads(audit_json.read_text(encoding="utf-8"))
        except Exception as exc:
            return fail(f"check=lesson_warning_tokens detail=audit_json_parse_failed:{exc}")

    totals_raw = audit_doc.get("totals")
    rows_raw = audit_doc.get("rows")
    if not isinstance(totals_raw, dict) or not isinstance(rows_raw, list):
        return fail("check=lesson_warning_tokens detail=audit_report_shape_invalid")

    warning_totals: dict[str, int] = {}
    total_warning_tokens = 0
    for key in LEGACY_WARNING_KEYS:
        value = int(totals_raw.get(key, 0) or 0)
        warning_totals[key] = value
        total_warning_tokens += value

    top_legacy_rows = summarize_rows(rows_raw, top=int(args.top))
    files_with_warning = sum(1 for row in rows_raw if isinstance(row, dict) and int(row.get("legacy_score", 0) or 0) > 0)

    report = {
        "schema": "seamgrim.lesson_warning_tokens_report.v1",
        "ok": (total_warning_tokens == 0) if bool(args.require_zero) else True,
        "require_zero": bool(args.require_zero),
        "include_preview": bool(args.include_preview),
        "legacy_warning_keys": list(LEGACY_WARNING_KEYS),
        "files": int(audit_doc.get("files", 0) or 0),
        "files_with_warning": files_with_warning,
        "total_warning_tokens": total_warning_tokens,
        "warning_totals": warning_totals,
        "top_legacy_rows": top_legacy_rows,
    }
    out_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    top_digest = "-"
    if top_legacy_rows:
        chunks = [f"{row['path']}:{row['warning_count']}" for row in top_legacy_rows[:3]]
        top_digest = ",".join(chunks)

    if args.require_zero and total_warning_tokens > 0:
        return fail(
            "check=lesson_warning_tokens detail="
            f"legacy_warning_tokens_nonzero:count={total_warning_tokens}:files={files_with_warning}:top={top_digest}"
        )

    print(
        "check=lesson_warning_tokens detail="
        f"ok:count={total_warning_tokens}:files={files_with_warning}:require_zero={int(bool(args.require_zero))}:top={top_digest}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
