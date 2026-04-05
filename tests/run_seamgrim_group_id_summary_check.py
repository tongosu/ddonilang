#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path


def fail(msg: str) -> int:
    print(f"[seamgrim-group-id-summary] fail: {msg}")
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


def load_seed_doc_from_report(report_path: Path) -> dict:
    try:
        doc = json.loads(report_path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise RuntimeError(f"seed report parse failed: {exc}") from exc
    if doc.get("schema") != "ddn.seamgrim.seed_runtime_visual_pack_report.v1":
        raise RuntimeError(f"seed schema mismatch: {doc.get('schema')}")
    return doc


def main() -> int:
    parser = argparse.ArgumentParser(description="Summarize seamgrim group_id propagation reports")
    parser.add_argument("--json-out", default="", help="optional summary json output path")
    parser.add_argument(
        "--seed-report",
        default="build/reports/seamgrim_seed_runtime_visual_pack_report.detjson",
        help="seed runtime visual pack report path",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    try:
        with tempfile.TemporaryDirectory(prefix="seamgrim-group-id-summary-") as tmp:
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
            if compare_doc.get("schema") != "ddn.seamgrim.overlay_compare_pack_report.v1":
                return fail(f"overlay compare schema mismatch: {compare_doc.get('schema')}")
            compare_row = (compare_doc.get("cases") or [None])[0]
            if not isinstance(compare_row, dict):
                return fail("overlay compare row missing")

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
            if session_doc.get("schema") != "ddn.seamgrim.overlay_session_pack_report.v1":
                return fail(f"overlay session schema mismatch: {session_doc.get('schema')}")
            session_row = (session_doc.get("cases") or [None])[0]
            if not isinstance(session_row, dict):
                return fail("overlay session row missing")

            seed_report_path = root / str(args.seed_report)
            if seed_report_path.exists():
                seed_doc = load_seed_doc_from_report(seed_report_path)
            else:
                seed_report_path.parent.mkdir(parents=True, exist_ok=True)
                seed_doc = run_json(
                    root,
                    [
                        sys.executable,
                        "tests/run_seamgrim_seed_runtime_visual_pack_check.py",
                        "--json-out",
                        str(seed_report_path),
                    ],
                    seed_report_path,
                )
                if seed_doc.get("schema") != "ddn.seamgrim.seed_runtime_visual_pack_report.v1":
                    return fail(f"seed schema mismatch: {seed_doc.get('schema')}")
            seed_cases = seed_doc.get("cases") or []
            seed_row = next((row for row in seed_cases if row.get("id") == "physics_pendulum_seed_v1"), None)
            if not isinstance(seed_row, dict):
                return fail("seed pendulum case missing")

            scene_report = tmp_dir / "scene_session_check.detjson"
            scene_doc = run_json(
                root,
                [
                    sys.executable,
                    "tests/run_seamgrim_scene_session_check.py",
                    "pack/edu_pilot_phys_econ/lesson_phys_01",
                    "--json-out",
                    str(scene_report),
                ],
                scene_report,
            )
            if scene_doc.get("schema") != "ddn.seamgrim.scene_session_check_report.v1":
                return fail(f"scene/session schema mismatch: {scene_doc.get('schema')}")
            scene_packs = scene_doc.get("packs") or []
            scene_row = next(
                (
                    row
                    for row in scene_packs
                    if str(row.get("pack_dir", "")).replace("\\", "/").endswith("pack/edu_pilot_phys_econ/lesson_phys_01")
                ),
                None,
            )
            if not isinstance(scene_row, dict):
                return fail("scene/session lesson summary missing")

    except RuntimeError as exc:
        return fail(str(exc))

    summary_rows = [
        {
            "category": "overlay_compare",
            "source_schema": compare_doc["schema"],
            "case_id": compare_row.get("case_id"),
            "group_ids": [
                compare_row.get("actual_group_id"),
                compare_row.get("actual_baseline_group_id"),
                compare_row.get("actual_variant_group_id"),
            ],
        },
        {
            "category": "overlay_session",
            "source_schema": session_doc["schema"],
            "case_id": session_row.get("case_id"),
            "group_ids": [
                session_row.get("expected_variant_group_id"),
                session_row.get("actual_variant_group_id"),
            ],
        },
        {
            "category": "synthesis_seed",
            "source_schema": seed_doc["schema"],
            "case_id": seed_row.get("id"),
            "group_ids": seed_row.get("group_ids"),
        },
        {
            "category": "synthesis_scene_session",
            "source_schema": scene_doc["schema"],
            "case_id": scene_row.get("pack_dir"),
            "group_ids": scene_row.get("group_ids"),
        },
    ]

    if summary_rows[0]["group_ids"] != ["pendulum.variant", "pendulum.variant", "pendulum.variant"]:
        return fail(f"overlay compare group_ids mismatch: {summary_rows[0]['group_ids']}")
    if summary_rows[1]["group_ids"] != ["pendulum.variant", "pendulum.variant"]:
        return fail(f"overlay session group_ids mismatch: {summary_rows[1]['group_ids']}")
    if summary_rows[2]["group_ids"] != ["pendulum.rod", "pendulum.bob", "pendulum.pivot"]:
        return fail(f"seed synthesis group_ids mismatch: {summary_rows[2]['group_ids']}")
    if summary_rows[3]["group_ids"] != ["pendulum.baseline", "pendulum.variant"]:
        return fail(f"scene/session synthesis group_ids mismatch: {summary_rows[3]['group_ids']}")

    category_counts = {
        "overlay": 2,
        "synthesis": 2,
    }
    summary = {
        "schema": "ddn.seamgrim.group_id_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "ok": True,
        "category_counts": category_counts,
        "rows": summary_rows,
    }

    if args.json_out:
        out = Path(args.json_out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    print("seamgrim group_id summary check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
