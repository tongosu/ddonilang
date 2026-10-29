#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(message: str) -> int:
    print(f"[lesson-migration-autofix-selftest] fail: {message}")
    return 1


def run_tool(
    tool: Path,
    *,
    scan_root: Path,
    json_out: Path,
    apply: bool,
    include_inputs: bool,
    rewrite_setup_colon: bool,
) -> subprocess.CompletedProcess[str]:
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
    if apply:
        cmd.append("--apply")
    if include_inputs:
        cmd.append("--include-inputs")
    if rewrite_setup_colon:
        cmd.append("--rewrite-setup-colon")
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


def read_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    tool = root / "solutions" / "seamgrim_ui_mvp" / "tools" / "lesson_migration_autofix.py"
    if not tool.exists():
        return fail(f"tool missing: {tool}")

    with tempfile.TemporaryDirectory(prefix="lesson_migration_autofix_selftest_") as temp_dir:
        scan_root = Path(temp_dir) / "lessons"
        lesson_path = scan_root / "l01" / "lesson.ddn"
        input_path = scan_root / "l01" / "inputs" / "preset_1.ddn"
        skip_path = scan_root / "l02" / "lesson.ddn"

        write_text(
            lesson_path,
            "채비: {\n  k:수 <- 1. // 범위(0.1, 5, 0.1)\n  z:수 <- 2. #범위(1, 3, 0.2)\n}.\n(처음)할때: {\n}.\n(매틱)마다: {\n}.\n(3마디)마다: {\n}.\n",
        )
        write_text(
            input_path,
            "채비: {\n  x:수 <- -3. //범위(-3, 3, 0.1)\n}.\n",
        )
        write_text(
            skip_path,
            "채비 {\n  broken <- 1. // 범위(a, b)\n}.\n",
        )

        dry_out = Path(temp_dir) / "dry.detjson"
        dry_proc = run_tool(
            tool,
            scan_root=scan_root,
            json_out=dry_out,
            apply=False,
            include_inputs=False,
            rewrite_setup_colon=False,
        )
        if dry_proc.returncode != 0:
            return fail(f"dry run failed: {(dry_proc.stderr or dry_proc.stdout).strip()}")
        dry_doc = read_json(dry_out)
        dry_totals = dry_doc.get("totals", {})
        if int(dry_doc.get("targets", -1)) != 2:
            return fail(f"dry targets mismatch: {dry_doc.get('targets')}")
        if int(dry_doc.get("changed", -1)) != 1:
            return fail(f"dry changed mismatch: {dry_doc.get('changed')}")
        if int(dry_totals.get("range_rewrites", -1)) != 1:
            return fail(f"dry range_rewrites mismatch: {dry_totals.get('range_rewrites')}")
        if int(dry_totals.get("range_hash_rewrites", -1)) != 1:
            return fail(f"dry range_hash_rewrites mismatch: {dry_totals.get('range_hash_rewrites')}")
        if int(dry_totals.get("range_hash_skipped", -1)) != 0:
            return fail(f"dry range_hash_skipped mismatch: {dry_totals.get('range_hash_skipped')}")
        if int(dry_totals.get("setup_colon_rewrites", -1)) != 0:
            return fail(f"dry setup_colon_rewrites mismatch: {dry_totals.get('setup_colon_rewrites')}")
        if int(dry_totals.get("hook_colon_rewrites", -1)) != 3:
            return fail(f"dry hook_colon_rewrites mismatch: {dry_totals.get('hook_colon_rewrites')}")
        if int(dry_totals.get("hook_alias_rewrites", -1)) != 2:
            return fail(f"dry hook_alias_rewrites mismatch: {dry_totals.get('hook_alias_rewrites')}")
        if int(dry_totals.get("range_skipped", -1)) != 1:
            return fail(f"dry range_skipped mismatch: {dry_totals.get('range_skipped')}")
        src_after_dry = lesson_path.read_text(encoding="utf-8")
        if "매김 {" in src_after_dry:
            return fail("dry run must not modify source file")

        apply_out = Path(temp_dir) / "apply.detjson"
        apply_proc = run_tool(
            tool,
            scan_root=scan_root,
            json_out=apply_out,
            apply=True,
            include_inputs=True,
            rewrite_setup_colon=True,
        )
        if apply_proc.returncode != 0:
            return fail(f"apply run failed: {(apply_proc.stderr or apply_proc.stdout).strip()}")
        apply_doc = read_json(apply_out)
        apply_totals = apply_doc.get("totals", {})
        if int(apply_doc.get("targets", -1)) != 3:
            return fail(f"apply targets mismatch: {apply_doc.get('targets')}")
        if int(apply_doc.get("changed", -1)) != 2:
            return fail(f"apply changed mismatch: {apply_doc.get('changed')}")
        if int(apply_totals.get("range_rewrites", -1)) != 2:
            return fail(f"apply range_rewrites mismatch: {apply_totals.get('range_rewrites')}")
        if int(apply_totals.get("range_hash_rewrites", -1)) != 1:
            return fail(f"apply range_hash_rewrites mismatch: {apply_totals.get('range_hash_rewrites')}")
        if int(apply_totals.get("range_hash_skipped", -1)) != 0:
            return fail(f"apply range_hash_skipped mismatch: {apply_totals.get('range_hash_skipped')}")
        if int(apply_totals.get("setup_colon_rewrites", -1)) != 2:
            return fail(f"apply setup_colon_rewrites mismatch: {apply_totals.get('setup_colon_rewrites')}")
        if int(apply_totals.get("hook_colon_rewrites", -1)) != 3:
            return fail(f"apply hook_colon_rewrites mismatch: {apply_totals.get('hook_colon_rewrites')}")
        if int(apply_totals.get("hook_alias_rewrites", -1)) != 2:
            return fail(f"apply hook_alias_rewrites mismatch: {apply_totals.get('hook_alias_rewrites')}")

        lesson_text = lesson_path.read_text(encoding="utf-8")
        if "채비 {" not in lesson_text or "채비: {" in lesson_text:
            return fail("apply must rewrite setup colon in lesson")
        if "(시작)할때 {" not in lesson_text or "(처음)할때" in lesson_text:
            return fail("apply must rewrite start hook alias/colon in lesson")
        if "(매마디)마다 {" not in lesson_text or "(매틱)마다" in lesson_text:
            return fail("apply must rewrite tick hook alias/colon in lesson")
        if "(3마디)마다 {" not in lesson_text or "(3마디)마다:" in lesson_text:
            return fail("apply must rewrite interval tick hook colon in lesson")
        if "k:수 <- (1) 매김 { 범위: 0.1..5. 간격: 0.1. }." not in lesson_text:
            return fail("apply must rewrite range comment in lesson")
        if "z:수 <- (2) 매김 { 범위: 1..3. 간격: 0.2. }." not in lesson_text:
            return fail("apply must rewrite #range comment in lesson")

        input_text = input_path.read_text(encoding="utf-8")
        if "x:수 <- (-3) 매김 { 범위: -3..3. 간격: 0.1. }." not in input_text:
            return fail("apply must rewrite range comment in inputs file")

    print("[lesson-migration-autofix-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
