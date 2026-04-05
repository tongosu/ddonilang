#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def fail(msg: str) -> int:
    print(f"[overlay-group-id-report-selftest] fail: {msg}")
    return 1


def run_json(root: Path, cmd: list[str], report_path: Path) -> dict:
    proc = subprocess.run(
        cmd,
        cwd=root,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        raise RuntimeError(f"command_failed stdout={proc.stdout} stderr={proc.stderr}")
    return json.loads(report_path.read_text(encoding="utf-8"))


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    with tempfile.TemporaryDirectory(prefix="overlay-group-id-report-") as tmp:
        tmp_dir = Path(tmp)

        compare_report = tmp_dir / "overlay_compare.detjson"
        compare_doc = run_json(
            root,
            [
                "node",
                "--no-warnings",
                "tests/seamgrim_overlay_compare_pack_runner.mjs",
                "--pack-root",
                "pack/seamgrim_overlay_param_compare_v0",
                "--case-file",
                "c77_group_id_normalized_equal_ok/case.detjson",
                "--json-out",
                str(compare_report),
                "--quiet",
            ],
            compare_report,
        )
        compare_row = (compare_doc.get("cases") or [None])[0]
        if not isinstance(compare_row, dict):
            return fail("compare row missing")
        if compare_row.get("actual_group_id") != "pendulum.variant":
            return fail(f"compare actual_group_id mismatch: {compare_row.get('actual_group_id')}")
        if compare_row.get("actual_baseline_group_id") != "pendulum.variant":
            return fail(
                f"compare actual_baseline_group_id mismatch: {compare_row.get('actual_baseline_group_id')}"
            )
        if compare_row.get("actual_variant_group_id") != "pendulum.variant":
            return fail(
                f"compare actual_variant_group_id mismatch: {compare_row.get('actual_variant_group_id')}"
            )

        session_report = tmp_dir / "overlay_session.detjson"
        session_doc = run_json(
            root,
            [
                "node",
                "--no-warnings",
                "tests/seamgrim_overlay_session_pack_runner.mjs",
                "--pack-root",
                "pack/seamgrim_overlay_session_roundtrip_v0",
                "--case-file",
                "c01_role_priority_restore_ok/case.detjson",
                "--json-out",
                str(session_report),
                "--quiet",
            ],
            session_report,
        )
        session_row = (session_doc.get("cases") or [None])[0]
        if not isinstance(session_row, dict):
            return fail("session row missing")
        if session_row.get("expected_variant_group_id") != "pendulum.variant":
            return fail(f"session expected_variant_group_id mismatch: {session_row.get('expected_variant_group_id')}")
        if session_row.get("actual_variant_group_id") != "pendulum.variant":
            return fail(f"session actual_variant_group_id mismatch: {session_row.get('actual_variant_group_id')}")
        if session_row.get("actual_baseline_group_id") is not None:
            return fail(f"session actual_baseline_group_id expected null: {session_row.get('actual_baseline_group_id')}")

    print("[overlay-group-id-report-selftest] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
