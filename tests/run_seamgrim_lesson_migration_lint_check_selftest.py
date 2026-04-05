#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(message: str) -> int:
    print(f"[lesson-migration-lint-selftest] fail: {message}")
    return 1


def run_tool(tool: Path, *, scan_root: Path, json_out: Path, include_preview: bool) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        str(tool),
        "--scan-root",
        str(scan_root),
        "--json-out",
        str(json_out),
        "--limit",
        "0",
    ]
    if include_preview:
        cmd.append("--include-preview")
    return subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    tool = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "lesson_migration_lint.py"
    if not tool.exists():
        return fail(f"tool missing: {tool}")

    with tempfile.TemporaryDirectory(prefix="lesson_migration_lint_selftest_") as temp_dir:
        scan_root = Path(temp_dir) / "lessons"
        write_text(
            scan_root / "l01" / "lesson.ddn",
            "채비: {\n  x: 수 <- 1. //범위(0, 1, 0.1)\n}.\n(처음)할때: {\n}.\n(매틱)마다: {\n}.\n(3마디)마다: {\n}.\n보여주기.\n",
        )
        write_text(
            scan_root / "l02" / "lesson.ddn",
            "채비 {\n  y: 수 <- 2.\n}.\n",
        )
        write_text(
            scan_root / "l03" / "lesson.age3.preview.ddn",
            "채비: {\n  z: 수 <- 3. //범위(0, 3, 1)\n}.\n",
        )

        out_without = Path(temp_dir) / "without_preview.detjson"
        proc_without = run_tool(tool, scan_root=scan_root, json_out=out_without, include_preview=False)
        if proc_without.returncode != 0:
            return fail(f"tool failed (without preview): {(proc_without.stderr or proc_without.stdout).strip()}")
        payload_without = json.loads(out_without.read_text(encoding="utf-8"))
        totals_without = payload_without.get("totals", {})
        if int(payload_without.get("files", -1)) != 2:
            return fail(f"files without preview mismatch: {payload_without.get('files')}")
        if int(totals_without.get("priority_range_comment", -1)) != 1:
            return fail(f"priority_range_comment without preview mismatch: {totals_without.get('priority_range_comment')}")
        if int(totals_without.get("priority_setup_colon", -1)) != 1:
            return fail(f"priority_setup_colon without preview mismatch: {totals_without.get('priority_setup_colon')}")
        if int(totals_without.get("info_legacy_show", -1)) != 1:
            return fail(f"info_legacy_show without preview mismatch: {totals_without.get('info_legacy_show')}")
        if int(totals_without.get("info_legacy_start_colon", -1)) != 1:
            return fail(
                f"info_legacy_start_colon without preview mismatch: {totals_without.get('info_legacy_start_colon')}"
            )
        if int(totals_without.get("info_legacy_tick_colon", -1)) != 1:
            return fail(
                f"info_legacy_tick_colon without preview mismatch: {totals_without.get('info_legacy_tick_colon')}"
            )
        if int(totals_without.get("info_legacy_tick_interval_colon", -1)) != 1:
            return fail(
                "info_legacy_tick_interval_colon without preview mismatch: "
                f"{totals_without.get('info_legacy_tick_interval_colon')}"
            )
        if int(totals_without.get("info_legacy_start_alias", -1)) != 1:
            return fail(
                f"info_legacy_start_alias without preview mismatch: {totals_without.get('info_legacy_start_alias')}"
            )
        if int(totals_without.get("info_legacy_tick_alias", -1)) != 1:
            return fail(
                f"info_legacy_tick_alias without preview mismatch: {totals_without.get('info_legacy_tick_alias')}"
            )
        if int(totals_without.get("priority_total", -1)) != 2:
            return fail(f"priority_total without preview mismatch: {totals_without.get('priority_total')}")

        out_with = Path(temp_dir) / "with_preview.detjson"
        proc_with = run_tool(tool, scan_root=scan_root, json_out=out_with, include_preview=True)
        if proc_with.returncode != 0:
            return fail(f"tool failed (with preview): {(proc_with.stderr or proc_with.stdout).strip()}")
        payload_with = json.loads(out_with.read_text(encoding="utf-8"))
        totals_with = payload_with.get("totals", {})
        if int(payload_with.get("files", -1)) != 3:
            return fail(f"files with preview mismatch: {payload_with.get('files')}")
        if int(totals_with.get("priority_range_comment", -1)) != 2:
            return fail(f"priority_range_comment with preview mismatch: {totals_with.get('priority_range_comment')}")
        if int(totals_with.get("priority_setup_colon", -1)) != 2:
            return fail(f"priority_setup_colon with preview mismatch: {totals_with.get('priority_setup_colon')}")
        if int(totals_with.get("info_legacy_start_colon", -1)) != 1:
            return fail(f"info_legacy_start_colon with preview mismatch: {totals_with.get('info_legacy_start_colon')}")
        if int(totals_with.get("info_legacy_tick_colon", -1)) != 1:
            return fail(f"info_legacy_tick_colon with preview mismatch: {totals_with.get('info_legacy_tick_colon')}")
        if int(totals_with.get("info_legacy_tick_interval_colon", -1)) != 1:
            return fail(
                "info_legacy_tick_interval_colon with preview mismatch: "
                f"{totals_with.get('info_legacy_tick_interval_colon')}"
            )
        if int(totals_with.get("info_legacy_start_alias", -1)) != 1:
            return fail(f"info_legacy_start_alias with preview mismatch: {totals_with.get('info_legacy_start_alias')}")
        if int(totals_with.get("info_legacy_tick_alias", -1)) != 1:
            return fail(f"info_legacy_tick_alias with preview mismatch: {totals_with.get('info_legacy_tick_alias')}")
        if int(totals_with.get("priority_total", -1)) != 4:
            return fail(f"priority_total with preview mismatch: {totals_with.get('priority_total')}")

    print("[lesson-migration-lint-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
